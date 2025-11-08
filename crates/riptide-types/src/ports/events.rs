//! Event bus port for domain event publishing
//!
//! This module provides backend-agnostic event bus interfaces that enable:
//! - Decoupling between event producers and consumers
//! - Testing with in-memory event buses
//! - Swapping message brokers (RabbitMQ, Kafka, NATS, etc.)
//! - Transactional outbox pattern support
//!
//! # Design Goals
//!
//! - **Loose Coupling**: Producers don't know about consumers
//! - **Testability**: In-memory bus for unit tests
//! - **Flexibility**: Support various message broker backends
//! - **Reliability**: At-least-once delivery semantics
//!
//! # Example
//!
//! ```rust,ignore
//! use riptide_types::ports::{EventBus, DomainEvent, EventHandler};
//!
//! async fn example(bus: &dyn EventBus) -> Result<()> {
//!     // Publish domain event
//!     let event = DomainEvent::new(
//!         "user.created",
//!         "user-123",
//!         serde_json::json!({"email": "user@example.com"}),
//!     );
//!     bus.publish(event).await?;
//!
//!     Ok(())
//! }
//! ```

use crate::error::Result as RiptideResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// Unique identifier for event subscriptions
pub type SubscriptionId = String;

/// Domain event representing business-significant occurrences
///
/// Events are immutable records of facts that have occurred in the system.
/// They should be named in past tense (e.g., "UserCreated", "OrderPlaced").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent {
    /// Unique event identifier (idempotency key)
    pub id: String,

    /// Event type identifier (e.g., "user.created", "order.placed")
    pub event_type: String,

    /// Aggregate root identifier this event relates to
    pub aggregate_id: String,

    /// Event payload (domain-specific data)
    pub payload: serde_json::Value,

    /// Event occurrence timestamp
    #[serde(with = "system_time_serialization")]
    pub timestamp: SystemTime,

    /// Additional event metadata (correlation IDs, causation IDs, etc.)
    pub metadata: HashMap<String, String>,
}

impl DomainEvent {
    /// Create new domain event
    ///
    /// # Arguments
    ///
    /// * `event_type` - Event type identifier
    /// * `aggregate_id` - Aggregate root this event relates to
    /// * `payload` - Event payload data
    ///
    /// # Returns
    ///
    /// New `DomainEvent` with generated ID and current timestamp
    pub fn new(
        event_type: impl Into<String>,
        aggregate_id: impl Into<String>,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            event_type: event_type.into(),
            aggregate_id: aggregate_id.into(),
            payload,
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to event
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Get correlation ID from metadata if present
    pub fn correlation_id(&self) -> Option<&str> {
        self.metadata.get("correlation_id").map(|s| s.as_str())
    }

    /// Get causation ID from metadata if present
    pub fn causation_id(&self) -> Option<&str> {
        self.metadata.get("causation_id").map(|s| s.as_str())
    }
}

/// Event bus for publishing and subscribing to domain events
///
/// Implementations must be thread-safe and support async operations.
/// Delivery semantics should be at-least-once where possible.
#[async_trait]
pub trait EventBus: Send + Sync {
    /// Publish domain event to all subscribers
    ///
    /// # Arguments
    ///
    /// * `event` - Domain event to publish
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Event published successfully (or queued for delivery)
    /// * `Err(_)` - Publishing failed
    ///
    /// # Delivery Semantics
    ///
    /// Implementations should provide at-least-once delivery where possible.
    /// For transactional consistency, use the Outbox pattern.
    async fn publish(&self, event: DomainEvent) -> RiptideResult<()>;

    /// Subscribe to events with handler
    ///
    /// # Arguments
    ///
    /// * `handler` - Event handler to invoke for matching events
    ///
    /// # Returns
    ///
    /// * `Ok(subscription_id)` - Subscription created successfully
    /// * `Err(_)` - Subscription failed
    ///
    /// # Handler Lifecycle
    ///
    /// Handlers remain active until explicitly unsubscribed.
    /// Failed handler invocations should be retried based on backend policy.
    async fn subscribe<H>(&self, handler: H) -> RiptideResult<SubscriptionId>
    where
        H: EventHandler + Send + Sync + 'static;

    /// Unsubscribe from events
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - Subscription to cancel
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Unsubscribed successfully (or subscription didn't exist)
    /// * `Err(_)` - Unsubscribe operation failed
    async fn unsubscribe(&self, subscription_id: &str) -> RiptideResult<()> {
        // Default implementation - backends should override
        let _ = subscription_id;
        Ok(())
    }

    /// Publish multiple events atomically (if supported)
    ///
    /// # Arguments
    ///
    /// * `events` - Events to publish
    ///
    /// # Returns
    ///
    /// * `Ok(())` - All events published successfully
    /// * `Err(_)` - Publishing failed (partial success possible)
    ///
    /// # Atomicity
    ///
    /// Default implementation publishes sequentially (not atomic).
    /// Backends supporting batch operations should override.
    async fn publish_batch(&self, events: Vec<DomainEvent>) -> RiptideResult<()> {
        for event in events {
            self.publish(event).await?;
        }
        Ok(())
    }
}

/// Event handler for processing domain events
///
/// Handlers should be idempotent as events may be delivered multiple times.
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Handle domain event
    ///
    /// # Arguments
    ///
    /// * `event` - Event to handle
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Event handled successfully
    /// * `Err(_)` - Handling failed (may trigger retry)
    ///
    /// # Idempotency
    ///
    /// Handlers MUST be idempotent as events may be redelivered.
    async fn handle(&self, event: &DomainEvent) -> RiptideResult<()>;

    /// Get event types this handler is interested in
    ///
    /// Default implementation returns None (matches all events).
    /// Override to filter events by type.
    fn event_types(&self) -> Option<Vec<String>> {
        None
    }
}

// Custom serialization for SystemTime
mod system_time_serialization {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = time
            .duration_since(UNIX_EPOCH)
            .map_err(serde::ser::Error::custom)?;
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(UNIX_EPOCH + Duration::from_secs(secs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_event_creation() {
        let event = DomainEvent::new(
            "user.created",
            "user-123",
            serde_json::json!({"email": "test@example.com"}),
        );

        assert_eq!(event.event_type, "user.created");
        assert_eq!(event.aggregate_id, "user-123");
        assert!(!event.id.is_empty());
    }

    #[test]
    fn test_domain_event_metadata() {
        let event = DomainEvent::new("test.event", "test-123", serde_json::json!({}))
            .with_metadata("correlation_id", "corr-456")
            .with_metadata("causation_id", "cause-789");

        assert_eq!(event.correlation_id(), Some("corr-456"));
        assert_eq!(event.causation_id(), Some("cause-789"));
    }

    #[test]
    fn test_domain_event_serialization() {
        let event = DomainEvent::new(
            "test.event",
            "test-123",
            serde_json::json!({"key": "value"}),
        );

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: DomainEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.id, deserialized.id);
        assert_eq!(event.event_type, deserialized.event_type);
        assert_eq!(event.aggregate_id, deserialized.aggregate_id);
    }
}
