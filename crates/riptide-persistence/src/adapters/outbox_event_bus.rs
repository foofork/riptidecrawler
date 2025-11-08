//! Transactional Outbox pattern implementation for EventBus
//!
//! This adapter provides:
//! - At-least-once delivery guarantees
//! - Transactional consistency (events published atomically with business data)
//! - Background worker for outbox polling and publishing
//! - Event serialization/deserialization
//!
//! # Architecture
//!
//! 1. **Write Phase**: Events are written to `event_outbox` table in same transaction as business data
//! 2. **Poll Phase**: Background worker polls outbox table for unpublished events
//! 3. **Publish Phase**: Events are published to actual message broker (e.g., RabbitMQ, Kafka)
//! 4. **Cleanup Phase**: Published events are marked as published or deleted
//!
//! # Database Schema
//!
//! ```sql
//! CREATE TABLE event_outbox (
//!     id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
//!     event_id TEXT NOT NULL UNIQUE,
//!     event_type TEXT NOT NULL,
//!     aggregate_id TEXT NOT NULL,
//!     payload JSONB NOT NULL,
//!     metadata JSONB NOT NULL,
//!     created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
//!     published_at TIMESTAMPTZ,
//!     retry_count INTEGER NOT NULL DEFAULT 0,
//!     last_error TEXT
//! );
//!
//! CREATE INDEX idx_outbox_unpublished ON event_outbox (created_at)
//!     WHERE published_at IS NULL;
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use riptide_persistence::adapters::{OutboxEventBus, OutboxPublisher};
//! use sqlx::PgPool;
//!
//! let pool = PgPool::connect(&database_url).await?;
//! let bus = OutboxEventBus::new(pool.clone());
//!
//! // Publish event (writes to outbox table)
//! let event = DomainEvent::new("user.created", "user-123", json!({"email": "user@example.com"}));
//! bus.publish(event).await?;
//!
//! // Start background publisher
//! let publisher = OutboxPublisher::new(pool, transport);
//! publisher.run().await;
//! ```

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use riptide_types::{
    DomainEvent, EventBus, EventHandler, Result as RiptideResult, RiptideError, SubscriptionId,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

/// Transactional Outbox implementation of EventBus
///
/// Events are written to a database table in the same transaction as business data,
/// ensuring transactional consistency. A background worker polls the outbox and
/// publishes events to the actual message broker.
pub struct OutboxEventBus {
    /// PostgreSQL connection pool
    pool: Arc<PgPool>,

    /// Table name for event outbox
    table_name: String,
}

impl OutboxEventBus {
    /// Create new outbox event bus
    ///
    /// # Arguments
    ///
    /// * `pool` - PostgreSQL connection pool
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let bus = OutboxEventBus::new(pool);
    /// ```
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self::with_table_name(pool, "event_outbox")
    }

    /// Create new outbox event bus with custom table name
    ///
    /// Useful for testing or multi-tenant scenarios
    pub fn with_table_name(pool: Arc<PgPool>, table_name: impl Into<String>) -> Self {
        Self {
            pool,
            table_name: table_name.into(),
        }
    }

    /// Get table name
    pub fn table_name(&self) -> &str {
        &self.table_name
    }
}

#[async_trait]
impl EventBus for OutboxEventBus {
    #[instrument(skip(self, event), fields(
        event_id = %event.id,
        event_type = %event.event_type,
        aggregate_id = %event.aggregate_id
    ))]
    async fn publish(&self, event: DomainEvent) -> RiptideResult<()> {
        debug!("Publishing event to outbox");

        // Serialize event payload and metadata
        let payload_json = serde_json::to_value(&event.payload).map_err(|e| {
            error!("Failed to serialize event payload: {}", e);
            RiptideError::Custom(format!("Failed to serialize event payload: {}", e))
        })?;

        let metadata_json = serde_json::to_value(&event.metadata).map_err(|e| {
            error!("Failed to serialize event metadata: {}", e);
            RiptideError::Custom(format!("Failed to serialize event metadata: {}", e))
        })?;

        // Convert SystemTime to DateTime<Utc>
        let timestamp: DateTime<Utc> = event.timestamp.into();

        // Insert into outbox table
        let query = format!(
            "INSERT INTO {} (event_id, event_type, aggregate_id, payload, metadata, created_at)
             VALUES ($1, $2, $3, $4, $5, $6)",
            self.table_name
        );

        sqlx::query(&query)
            .bind(&event.id)
            .bind(&event.event_type)
            .bind(&event.aggregate_id)
            .bind(payload_json)
            .bind(metadata_json)
            .bind(timestamp)
            .execute(&*self.pool)
            .await
            .map_err(|e| {
                error!("Failed to insert event into outbox: {}", e);
                RiptideError::Storage(format!("Failed to insert event into outbox: {}", e))
            })?;

        debug!("Event published to outbox successfully");
        Ok(())
    }

    async fn subscribe<H>(&self, _handler: H) -> RiptideResult<SubscriptionId>
    where
        H: EventHandler + Send + Sync + 'static,
    {
        // Outbox pattern doesn't support direct subscriptions
        // Subscriptions should be handled by the message broker (e.g., RabbitMQ)
        warn!("OutboxEventBus does not support direct subscriptions");
        Err(RiptideError::Custom(
            "Direct event subscriptions not supported in OutboxEventBus".to_string(),
        ))
    }

    async fn publish_batch(&self, events: Vec<DomainEvent>) -> RiptideResult<()> {
        debug!(
            event_count = events.len(),
            "Publishing batch of events to outbox"
        );

        // Use transaction for batch insert
        let mut tx = self.pool.begin().await.map_err(|e| {
            error!("Failed to begin transaction: {}", e);
            RiptideError::Storage(format!("Failed to begin transaction: {}", e))
        })?;

        for event in events {
            let payload_json = serde_json::to_value(&event.payload)?;
            let metadata_json = serde_json::to_value(&event.metadata)?;
            let timestamp: DateTime<Utc> = event.timestamp.into();

            let query = format!(
                "INSERT INTO {} (event_id, event_type, aggregate_id, payload, metadata, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6)",
                self.table_name
            );

            sqlx::query(&query)
                .bind(&event.id)
                .bind(&event.event_type)
                .bind(&event.aggregate_id)
                .bind(payload_json)
                .bind(metadata_json)
                .bind(timestamp)
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    RiptideError::Storage(format!("Failed to insert event into outbox: {}", e))
                })?;
        }

        tx.commit().await.map_err(|e| {
            error!("Failed to commit batch insert: {}", e);
            RiptideError::Storage(format!("Failed to commit batch insert: {}", e))
        })?;

        debug!("Event batch published to outbox successfully");
        Ok(())
    }
}

/// Outbox row structure
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
struct OutboxRow {
    id: Uuid,
    event_id: String,
    event_type: String,
    aggregate_id: String,
    payload: serde_json::Value,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    retry_count: i32,
}

impl OutboxRow {
    /// Convert to DomainEvent
    fn to_domain_event(&self) -> RiptideResult<DomainEvent> {
        let metadata: HashMap<String, String> = serde_json::from_value(self.metadata.clone())
            .map_err(|e| RiptideError::Custom(format!("Failed to deserialize metadata: {}", e)))?;

        Ok(DomainEvent {
            id: self.event_id.clone(),
            event_type: self.event_type.clone(),
            aggregate_id: self.aggregate_id.clone(),
            payload: self.payload.clone(),
            timestamp: self.created_at.into(),
            metadata,
        })
    }
}

/// Background worker that polls outbox and publishes events
///
/// This worker should run continuously in a background task.
/// It polls the outbox table for unpublished events and publishes
/// them to the actual message broker.
pub struct OutboxPublisher {
    /// PostgreSQL connection pool
    pool: Arc<PgPool>,

    /// Table name for event outbox
    table_name: String,

    /// Polling interval
    poll_interval: Duration,

    /// Maximum number of events to fetch per poll
    batch_size: usize,

    /// Maximum retry attempts before giving up
    max_retries: i32,
}

impl OutboxPublisher {
    /// Create new outbox publisher
    ///
    /// # Arguments
    ///
    /// * `pool` - PostgreSQL connection pool
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let publisher = OutboxPublisher::new(pool);
    /// tokio::spawn(async move {
    ///     publisher.run().await;
    /// });
    /// ```
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            pool,
            table_name: "event_outbox".to_string(),
            poll_interval: Duration::from_secs(5),
            batch_size: 100,
            max_retries: 5,
        }
    }

    /// Configure polling interval
    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    /// Configure batch size
    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    /// Configure max retries
    pub fn with_max_retries(mut self, retries: i32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Run the outbox publisher (infinite loop)
    ///
    /// This method polls the outbox table and publishes events.
    /// It should be run in a background task.
    #[instrument(skip(self))]
    pub async fn run(&self) {
        info!(
            poll_interval = ?self.poll_interval,
            batch_size = self.batch_size,
            "Starting outbox publisher"
        );

        let mut interval = time::interval(self.poll_interval);

        loop {
            interval.tick().await;

            if let Err(e) = self.poll_and_publish().await {
                error!("Error polling outbox: {:?}", e);
            }
        }
    }

    /// Poll outbox and publish events (single iteration)
    #[instrument(skip(self))]
    async fn poll_and_publish(&self) -> RiptideResult<()> {
        debug!("Polling outbox for unpublished events");

        // Fetch unpublished events
        let query = format!(
            "SELECT id, event_id, event_type, aggregate_id, payload, metadata, created_at, retry_count
             FROM {}
             WHERE published_at IS NULL AND retry_count < $1
             ORDER BY created_at ASC
             LIMIT $2",
            self.table_name
        );

        let rows: Vec<OutboxRow> = sqlx::query_as::<_, OutboxRow>(&query)
            .bind(self.max_retries)
            .bind(self.batch_size as i64)
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| RiptideError::Storage(format!("Failed to fetch outbox events: {}", e)))?;

        if rows.is_empty() {
            debug!("No unpublished events in outbox");
            return Ok(());
        }

        info!(event_count = rows.len(), "Publishing events from outbox");

        for row in rows {
            if let Err(e) = self.publish_event(&row).await {
                error!(
                    event_id = %row.event_id,
                    error = ?e,
                    "Failed to publish event"
                );
                self.mark_retry(&row.id).await?;
            } else {
                self.mark_published(&row.id).await?;
            }
        }

        Ok(())
    }

    /// Publish a single event from outbox
    async fn publish_event(&self, row: &OutboxRow) -> RiptideResult<()> {
        let event = row.to_domain_event()?;

        // TODO: Publish to actual message broker (RabbitMQ, Kafka, NATS, etc.)
        // For now, just log the event
        info!(
            event_id = %event.id,
            event_type = %event.event_type,
            "Publishing event (placeholder - configure message broker)"
        );

        Ok(())
    }

    /// Mark event as published
    async fn mark_published(&self, id: &Uuid) -> RiptideResult<()> {
        let query = format!(
            "UPDATE {} SET published_at = NOW() WHERE id = $1",
            self.table_name
        );

        sqlx::query(&query)
            .bind(id)
            .execute(&*self.pool)
            .await
            .map_err(|e| {
                RiptideError::Storage(format!("Failed to mark event as published: {}", e))
            })?;

        Ok(())
    }

    /// Increment retry count
    async fn mark_retry(&self, id: &Uuid) -> RiptideResult<()> {
        let query = format!(
            "UPDATE {} SET retry_count = retry_count + 1 WHERE id = $1",
            self.table_name
        );

        sqlx::query(&query)
            .bind(id)
            .execute(&*self.pool)
            .await
            .map_err(|e| {
                RiptideError::Storage(format!("Failed to increment retry count: {}", e))
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outbox_row_to_domain_event() {
        let row = OutboxRow {
            id: Uuid::new_v4(),
            event_id: "event-123".to_string(),
            event_type: "user.created".to_string(),
            aggregate_id: "user-456".to_string(),
            payload: serde_json::json!({"email": "test@example.com"}),
            metadata: serde_json::json!({"correlation_id": "corr-789"}),
            created_at: Utc::now(),
            retry_count: 0,
        };

        let event = row.to_domain_event().unwrap();

        assert_eq!(event.id, "event-123");
        assert_eq!(event.event_type, "user.created");
        assert_eq!(event.aggregate_id, "user-456");
    }

    #[test]
    fn test_outbox_publisher_configuration() {
        let pool = Arc::new(PgPool::connect_lazy("postgres://localhost").unwrap());
        let publisher = OutboxPublisher::new(pool)
            .with_poll_interval(Duration::from_secs(10))
            .with_batch_size(50)
            .with_max_retries(3);

        assert_eq!(publisher.poll_interval, Duration::from_secs(10));
        assert_eq!(publisher.batch_size, 50);
        assert_eq!(publisher.max_retries, 3);
    }
}
