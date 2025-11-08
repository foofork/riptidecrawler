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
use tracing::{debug, error, instrument, warn};
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

    async fn subscribe(&self, _handler: Arc<dyn EventHandler>) -> RiptideResult<SubscriptionId> {
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
#[allow(dead_code)] // Used in future publisher implementation
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
    #[allow(dead_code)] // Used in future publisher implementation
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

// DEPRECATED: OutboxPublisher has been moved to a separate module.
//
// The legacy OutboxPublisher in this file has been replaced with a more
// feature-rich implementation in `outbox_publisher.rs`. The new version includes:
//
// - Real EventBus integration (not just placeholder logging)
// - Exponential backoff retry logic
// - Row-level locking for concurrent publisher safety
// - Graceful shutdown via CancellationToken
// - Comprehensive error handling
//
// Please use: `use riptide_persistence::adapters::OutboxPublisher;`
//
// See: crates/riptide-persistence/src/adapters/outbox_publisher.rs
// Docs: docs/architecture/outbox-publisher-design.md

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

    // OutboxPublisher tests moved to outbox_publisher.rs and tests/outbox_publisher_tests.rs
}
