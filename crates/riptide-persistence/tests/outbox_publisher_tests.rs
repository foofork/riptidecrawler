//! Integration tests for OutboxPublisher
//!
//! These tests verify:
//! - Polling and publishing mechanism
//! - Retry logic with exponential backoff
//! - Concurrent publisher safety (row-level locking)
//! - Graceful shutdown via CancellationToken
//! - Error handling and recovery

#![cfg(feature = "postgres")]

use async_trait::async_trait;
use riptide_persistence::adapters::{OutboxEventBus, OutboxPublisher};
use riptide_types::{DomainEvent, EventBus, EventHandler, Result as RiptideResult, SubscriptionId};
use serde_json::json;
use sqlx::PgPool;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

/// Helper to get database URL from environment
fn database_url() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/riptide_test".to_string())
}

/// Setup test database with fresh schema
async fn setup_test_db() -> PgPool {
    let pool = PgPool::connect(&database_url())
        .await
        .expect("Failed to connect to test database");

    // Create outbox table
    sqlx::query(
        r#"
        DROP TABLE IF EXISTS event_outbox;

        CREATE TABLE event_outbox (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            event_id TEXT NOT NULL UNIQUE,
            event_type TEXT NOT NULL,
            aggregate_id TEXT NOT NULL,
            payload JSONB NOT NULL,
            metadata JSONB NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            published_at TIMESTAMPTZ,
            retry_count INTEGER NOT NULL DEFAULT 0,
            last_error TEXT,
            last_retry_at TIMESTAMPTZ
        );

        CREATE INDEX idx_outbox_unpublished ON event_outbox (created_at)
            WHERE published_at IS NULL;

        CREATE INDEX idx_outbox_retry_backoff ON event_outbox (last_retry_at)
            WHERE published_at IS NULL AND retry_count < 10;
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create test schema");

    pool
}

/// Mock EventBus that tracks published events
#[derive(Clone)]
struct MockEventBus {
    published_events: Arc<Mutex<Vec<DomainEvent>>>,
    publish_count: Arc<AtomicUsize>,
    should_fail: Arc<Mutex<bool>>,
}

impl MockEventBus {
    fn new() -> Self {
        Self {
            published_events: Arc::new(Mutex::new(Vec::new())),
            publish_count: Arc::new(AtomicUsize::new(0)),
            should_fail: Arc::new(Mutex::new(false)),
        }
    }

    async fn set_should_fail(&self, fail: bool) {
        *self.should_fail.lock().await = fail;
    }

    async fn get_published_events(&self) -> Vec<DomainEvent> {
        self.published_events.lock().await.clone()
    }
}

#[async_trait]
impl EventBus for MockEventBus {
    async fn publish(&self, event: DomainEvent) -> RiptideResult<()> {
        self.publish_count.fetch_add(1, Ordering::SeqCst);

        if *self.should_fail.lock().await {
            return Err(riptide_types::RiptideError::Custom(
                "Mock publish failure".to_string(),
            ));
        }

        self.published_events.lock().await.push(event);
        Ok(())
    }

    async fn subscribe(&self, _handler: Arc<dyn EventHandler>) -> RiptideResult<SubscriptionId> {
        Ok("mock-sub".to_string())
    }
}

#[tokio::test]
#[ignore] // Requires database
async fn test_basic_polling_and_publishing() {
    let pool = Arc::new(setup_test_db().await);
    let outbox_bus = OutboxEventBus::new(pool.clone());
    let mock_bus = Arc::new(MockEventBus::new());

    // Write events to outbox
    let event1 = DomainEvent::new(
        "user.created",
        "user-123",
        json!({"email": "user1@example.com"}),
    );
    let event2 = DomainEvent::new(
        "user.updated",
        "user-456",
        json!({"email": "user2@example.com"}),
    );

    outbox_bus.publish(event1.clone()).await.unwrap();
    outbox_bus.publish(event2.clone()).await.unwrap();

    // Create publisher
    let publisher = OutboxPublisher::builder()
        .pool(pool.clone())
        .event_bus(mock_bus.clone())
        .poll_interval(Duration::from_millis(100))
        .build();

    // Run publisher for a short time
    let cancel_token = CancellationToken::new();
    let cancel_clone = cancel_token.clone();

    let handle = tokio::spawn(async move {
        publisher.start(cancel_clone).await;
    });

    // Wait for events to be published
    tokio::time::sleep(Duration::from_millis(500)).await;
    cancel_token.cancel();
    handle.await.unwrap();

    // Verify events were published
    let published = mock_bus.get_published_events().await;
    assert_eq!(published.len(), 2);
    assert_eq!(published[0].event_type, "user.created");
    assert_eq!(published[1].event_type, "user.updated");

    // Verify events marked as published in database
    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM event_outbox WHERE published_at IS NOT NULL")
            .fetch_one(&*pool)
            .await
            .unwrap();
    assert_eq!(count.0, 2);
}

#[tokio::test]
#[ignore] // Requires database
async fn test_retry_with_exponential_backoff() {
    let pool = Arc::new(setup_test_db().await);
    let outbox_bus = OutboxEventBus::new(pool.clone());
    let mock_bus = Arc::new(MockEventBus::new());

    // Configure mock to fail initially
    mock_bus.set_should_fail(true).await;

    // Write event to outbox
    let event = DomainEvent::new("test.event", "test-123", json!({"data": "value"}));
    outbox_bus.publish(event).await.unwrap();

    // Create publisher with short intervals for testing
    let publisher = OutboxPublisher::builder()
        .pool(pool.clone())
        .event_bus(mock_bus.clone())
        .poll_interval(Duration::from_millis(100))
        .min_backoff(Duration::from_millis(50))
        .max_retries(3)
        .build();

    let cancel_token = CancellationToken::new();
    let cancel_clone = cancel_token.clone();

    let handle = tokio::spawn(async move {
        publisher.start(cancel_clone).await;
    });

    // Let it fail a few times
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Verify retry count increased
    let retry_count: (i32,) = sqlx::query_as(
        "SELECT retry_count FROM event_outbox WHERE event_id = (SELECT event_id FROM event_outbox LIMIT 1)",
    )
    .fetch_one(&*pool)
    .await
    .unwrap();
    assert!(retry_count.0 > 0, "Retry count should be incremented");

    // Verify last_error is set
    let last_error: (Option<String>,) = sqlx::query_as(
        "SELECT last_error FROM event_outbox WHERE event_id = (SELECT event_id FROM event_outbox LIMIT 1)",
    )
    .fetch_one(&*pool)
    .await
    .unwrap();
    assert!(last_error.0.is_some(), "Error should be recorded");
    assert!(last_error.0.unwrap().contains("Mock publish failure"));

    // Now allow publishing to succeed
    mock_bus.set_should_fail(false).await;

    // Wait for successful retry
    tokio::time::sleep(Duration::from_millis(500)).await;
    cancel_token.cancel();
    handle.await.unwrap();

    // Verify event was eventually published
    let published = mock_bus.get_published_events().await;
    assert_eq!(published.len(), 1);
    assert_eq!(published[0].event_type, "test.event");
}

#[tokio::test]
#[ignore] // Requires database
async fn test_concurrent_publisher_safety() {
    let pool = Arc::new(setup_test_db().await);
    let outbox_bus = OutboxEventBus::new(pool.clone());

    // Write multiple events to outbox
    for i in 0..10 {
        let event = DomainEvent::new("test.event", format!("test-{}", i), json!({"index": i}));
        outbox_bus.publish(event).await.unwrap();
    }

    // Create two publishers with the same mock bus
    let mock_bus = Arc::new(MockEventBus::new());

    let publisher1 = OutboxPublisher::builder()
        .pool(pool.clone())
        .event_bus(mock_bus.clone())
        .poll_interval(Duration::from_millis(100))
        .batch_size(5) // Small batch to encourage concurrent processing
        .build();

    let publisher2 = OutboxPublisher::builder()
        .pool(pool.clone())
        .event_bus(mock_bus.clone())
        .poll_interval(Duration::from_millis(100))
        .batch_size(5)
        .build();

    // Start both publishers concurrently
    let cancel1 = CancellationToken::new();
    let cancel2 = CancellationToken::new();

    let cancel1_clone = cancel1.clone();
    let cancel2_clone = cancel2.clone();

    let handle1 = tokio::spawn(async move {
        publisher1.start(cancel1_clone).await;
    });

    let handle2 = tokio::spawn(async move {
        publisher2.start(cancel2_clone).await;
    });

    // Wait for events to be published
    tokio::time::sleep(Duration::from_secs(1)).await;

    cancel1.cancel();
    cancel2.cancel();
    handle1.await.unwrap();
    handle2.await.unwrap();

    // Verify all events were published exactly once
    let published = mock_bus.get_published_events().await;
    assert_eq!(
        published.len(),
        10,
        "All events should be published exactly once"
    );

    // Verify no duplicate event IDs
    let mut event_ids: Vec<String> = published.iter().map(|e| e.id.clone()).collect();
    event_ids.sort();
    event_ids.dedup();
    assert_eq!(
        event_ids.len(),
        10,
        "No duplicate events should be published"
    );

    // Verify all events marked as published
    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM event_outbox WHERE published_at IS NOT NULL")
            .fetch_one(&*pool)
            .await
            .unwrap();
    assert_eq!(count.0, 10);
}

#[tokio::test]
#[ignore] // Requires database
async fn test_max_retries_exceeded() {
    let pool = Arc::new(setup_test_db().await);
    let outbox_bus = OutboxEventBus::new(pool.clone());
    let mock_bus = Arc::new(MockEventBus::new());

    // Configure mock to always fail
    mock_bus.set_should_fail(true).await;

    // Write event to outbox
    let event = DomainEvent::new("test.event", "test-123", json!({"data": "value"}));
    outbox_bus.publish(event).await.unwrap();

    // Create publisher with low max_retries
    let publisher = OutboxPublisher::builder()
        .pool(pool.clone())
        .event_bus(mock_bus.clone())
        .poll_interval(Duration::from_millis(100))
        .min_backoff(Duration::from_millis(50))
        .max_retries(2)
        .build();

    let cancel_token = CancellationToken::new();
    let cancel_clone = cancel_token.clone();

    let handle = tokio::spawn(async move {
        publisher.start(cancel_clone).await;
    });

    // Wait for max retries to be reached
    tokio::time::sleep(Duration::from_millis(800)).await;
    cancel_token.cancel();
    handle.await.unwrap();

    // Verify retry count reached max
    let retry_count: (i32,) = sqlx::query_as(
        "SELECT retry_count FROM event_outbox WHERE event_id = (SELECT event_id FROM event_outbox LIMIT 1)",
    )
    .fetch_one(&*pool)
    .await
    .unwrap();
    assert!(
        retry_count.0 >= 2,
        "Retry count should reach or exceed max_retries"
    );

    // Verify event was not published
    let published = mock_bus.get_published_events().await;
    assert_eq!(
        published.len(),
        0,
        "Event should not be published after max retries"
    );

    // Verify event still in outbox (not marked as published)
    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM event_outbox WHERE published_at IS NULL")
            .fetch_one(&*pool)
            .await
            .unwrap();
    assert_eq!(count.0, 1, "Event should remain unpublished");
}

#[tokio::test]
#[ignore] // Requires database
async fn test_batch_processing() {
    let pool = Arc::new(setup_test_db().await);
    let outbox_bus = OutboxEventBus::new(pool.clone());
    let mock_bus = Arc::new(MockEventBus::new());

    // Write many events to test batch processing
    let event_count = 250;
    for i in 0..event_count {
        let event = DomainEvent::new("test.event", format!("test-{}", i), json!({"index": i}));
        outbox_bus.publish(event).await.unwrap();
    }

    // Create publisher with batch size
    let publisher = OutboxPublisher::builder()
        .pool(pool.clone())
        .event_bus(mock_bus.clone())
        .poll_interval(Duration::from_millis(100))
        .batch_size(50)
        .build();

    let cancel_token = CancellationToken::new();
    let cancel_clone = cancel_token.clone();

    let handle = tokio::spawn(async move {
        publisher.start(cancel_clone).await;
    });

    // Wait for all events to be published (should take multiple batches)
    tokio::time::sleep(Duration::from_secs(3)).await;
    cancel_token.cancel();
    handle.await.unwrap();

    // Verify all events were published
    let published = mock_bus.get_published_events().await;
    assert_eq!(
        published.len(),
        event_count,
        "All events should be published"
    );

    // Verify all events marked as published
    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM event_outbox WHERE published_at IS NOT NULL")
            .fetch_one(&*pool)
            .await
            .unwrap();
    assert_eq!(count.0, event_count as i64);
}

#[tokio::test]
#[ignore] // Requires database
async fn test_graceful_shutdown() {
    let pool = Arc::new(setup_test_db().await);
    let outbox_bus = OutboxEventBus::new(pool.clone());
    let mock_bus = Arc::new(MockEventBus::new());

    // Write events to outbox
    for i in 0..5 {
        let event = DomainEvent::new("test.event", format!("test-{}", i), json!({"index": i}));
        outbox_bus.publish(event).await.unwrap();
    }

    let publisher = OutboxPublisher::builder()
        .pool(pool.clone())
        .event_bus(mock_bus.clone())
        .poll_interval(Duration::from_millis(100))
        .build();

    let cancel_token = CancellationToken::new();
    let cancel_clone = cancel_token.clone();

    let handle = tokio::spawn(async move {
        publisher.start(cancel_clone).await;
    });

    // Let it publish some events
    tokio::time::sleep(Duration::from_millis(300)).await;

    // Trigger graceful shutdown
    let shutdown_start = tokio::time::Instant::now();
    cancel_token.cancel();
    handle.await.unwrap();
    let shutdown_duration = shutdown_start.elapsed();

    // Verify shutdown was quick (< 1 second)
    assert!(
        shutdown_duration < Duration::from_secs(1),
        "Shutdown should be quick"
    );

    // Verify at least some events were published
    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM event_outbox WHERE published_at IS NOT NULL")
            .fetch_one(&*pool)
            .await
            .unwrap();
    assert!(
        count.0 > 0,
        "Some events should be published before shutdown"
    );
}
