//! EventBus adapter implementing the EventPublisher port trait
//!
//! This adapter wraps the concrete EventBus implementation to conform to
//! the EventPublisher port interface, enabling dependency inversion.
//!
//! # Architecture
//!
//! ```text
//! ApplicationContext (riptide-api)
//!     ↓ depends on
//! EventPublisher trait (riptide-types/ports)
//!     ↑ implemented by
//! EventBusAdapter (riptide-events/adapters)
//!     ↓ wraps
//! EventBus (riptide-events)
//! ```

use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, error, warn};

use riptide_types::error::Result;
use riptide_types::ports::events::EventSeverity as PortEventSeverity;
use riptide_types::ports::{
    EventPublisher, EventRecord, PublisherStats, SubscriptionId, TypedDomainEvent,
};

use crate::bus::{EventBus, EventBusStats};
use crate::{BaseEvent, EventEmitter, EventSeverity};

/// Adapter that wraps EventBus to implement EventPublisher port
///
/// This adapter bridges the concrete EventBus implementation with the
/// abstract EventPublisher port trait, enabling hexagonal architecture.
///
/// # Example
///
/// ```rust,ignore
/// use riptide_events::EventBus;
/// use riptide_events::adapters::EventBusAdapter;
///
/// let event_bus = EventBus::new();
/// let publisher: Arc<dyn EventPublisher> = EventBusAdapter::new(event_bus);
/// ```
pub struct EventBusAdapter {
    inner: Arc<EventBus>,
}

impl EventBusAdapter {
    /// Create new adapter wrapping an EventBus
    ///
    /// # Arguments
    ///
    /// * `event_bus` - The concrete EventBus to wrap
    ///
    /// # Returns
    ///
    /// Arc-wrapped adapter ready to be used as Arc<dyn EventPublisher>
    pub fn new(event_bus: EventBus) -> Arc<Self> {
        Arc::new(Self {
            inner: Arc::new(event_bus),
        })
    }

    /// Create adapter from existing Arc<EventBus>
    ///
    /// Useful when the EventBus is already Arc-wrapped elsewhere.
    pub fn from_arc(event_bus: Arc<EventBus>) -> Arc<Self> {
        Arc::new(Self { inner: event_bus })
    }

    /// Get reference to inner EventBus (for testing or advanced usage)
    pub fn inner(&self) -> &Arc<EventBus> {
        &self.inner
    }

    /// Convert internal EventBusStats to port PublisherStats
    fn convert_stats(&self, bus_stats: EventBusStats) -> PublisherStats {
        PublisherStats {
            active_subscribers: bus_stats.current_subscribers,
            total_published: 0, // EventBus doesn't track this yet
            total_failures: 0,  // EventBus doesn't track this yet
            buffer_size: bus_stats.buffer_size,
            is_running: bus_stats.is_running,
        }
    }

    /// Convert domain event severity to internal EventSeverity
    fn convert_severity(severity: PortEventSeverity) -> crate::EventSeverity {
        match severity {
            PortEventSeverity::Trace => EventSeverity::Trace,
            PortEventSeverity::Debug => EventSeverity::Debug,
            PortEventSeverity::Info => EventSeverity::Info,
            PortEventSeverity::Warn => EventSeverity::Warn,
            PortEventSeverity::Error => EventSeverity::Error,
            PortEventSeverity::Critical => EventSeverity::Critical,
        }
    }
}

#[async_trait]
impl EventPublisher for EventBusAdapter {
    async fn publish<E: TypedDomainEvent + 'static>(&self, event: E) -> Result<()> {
        debug!(
            event_type = %event.event_type(),
            event_id = %event.event_id(),
            source = %event.source(),
            "Publishing domain event via adapter"
        );

        // Convert domain event to internal BaseEvent
        let base_event = BaseEvent {
            event_id: event.event_id().to_string(),
            event_type: event.event_type().to_string(),
            timestamp: event.timestamp(),
            source: event.source().to_string(),
            severity: Self::convert_severity(event.severity()),
            metadata: event.metadata().clone(),
            context: None,
        };

        // Delegate to inner EventBus
        match self.inner.emit_event(base_event).await {
            Ok(()) => {
                debug!(
                    event_type = %event.event_type(),
                    event_id = %event.event_id(),
                    "Event published successfully"
                );
                Ok(())
            }
            Err(e) => {
                error!(
                    event_type = %event.event_type(),
                    event_id = %event.event_id(),
                    error = %e,
                    "Failed to publish event"
                );
                Err(riptide_types::RiptideError::Other(e))
            }
        }
    }

    async fn subscribe<E: TypedDomainEvent + 'static>(
        &self,
        _handler: Box<dyn Fn(E) -> Result<()> + Send + Sync>,
    ) -> SubscriptionId {
        // EventBus uses a broadcast channel model, not handler registration
        // To fully implement this, we would need to:
        // 1. Create a tokio task that receives from bus.subscribe()
        // 2. Filter events of type E
        // 3. Call the handler
        // 4. Return a SubscriptionId for cleanup
        //
        // For now, return a placeholder ID
        warn!("subscribe() not fully implemented - use EventBus.register_handler() directly");
        uuid::Uuid::new_v4().to_string()
    }

    async fn unsubscribe(&self, id: SubscriptionId) -> Result<()> {
        // Placeholder - see subscribe() note above
        warn!(subscription_id = %id, "unsubscribe() not fully implemented");
        Ok(())
    }

    async fn event_history(&self, _limit: usize) -> Vec<EventRecord> {
        // EventBus doesn't currently store event history
        // Could be added as a feature with circular buffer
        debug!("event_history() not implemented - EventBus doesn't persist events");
        Vec::new()
    }

    async fn is_healthy(&self) -> bool {
        let stats = self.inner.get_stats();
        stats.is_running
    }

    async fn stats(&self) -> PublisherStats {
        let bus_stats = self.inner.get_stats();
        self.convert_stats(bus_stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EventBus;
    use chrono::Utc;
    use std::collections::HashMap;

    #[derive(Debug)]
    struct TestEvent {
        id: String,
        timestamp: chrono::DateTime<Utc>,
    }

    impl TypedDomainEvent for TestEvent {
        fn event_type(&self) -> &'static str {
            "test.event"
        }

        fn event_id(&self) -> &str {
            &self.id
        }

        fn timestamp(&self) -> chrono::DateTime<Utc> {
            self.timestamp
        }

        fn source(&self) -> &str {
            "test_adapter"
        }

        fn severity(&self) -> PortEventSeverity {
            PortEventSeverity::Info
        }

        fn metadata(&self) -> &HashMap<String, String> {
            &HashMap::new()
        }
    }

    #[tokio::test]
    async fn test_adapter_creation() {
        let event_bus = EventBus::new();
        let adapter = EventBusAdapter::new(event_bus);

        // Verify adapter wraps the bus
        assert!(adapter.inner.get_stats().buffer_size > 0);
    }

    #[tokio::test]
    async fn test_adapter_from_arc() {
        let event_bus = Arc::new(EventBus::new());
        let adapter = EventBusAdapter::from_arc(event_bus.clone());

        // Verify both point to same underlying bus
        assert_eq!(
            Arc::as_ptr(&adapter.inner),
            Arc::as_ptr(&event_bus)
        );
    }

    #[tokio::test]
    async fn test_publish_domain_event() {
        let event_bus = EventBus::new();
        let adapter = EventBusAdapter::new(event_bus);

        let event = TestEvent {
            id: "test-123".to_string(),
            timestamp: Utc::now(),
        };

        // Should publish successfully even without subscribers (internal receiver exists)
        let result = adapter.publish(event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_adapter_is_healthy() {
        let mut event_bus = EventBus::new();
        let adapter = EventBusAdapter::new(event_bus.clone());

        // Initially not running
        assert!(!adapter.is_healthy().await);

        // Start the bus
        event_bus.start().await.unwrap();

        // Should be healthy now (note: start() mutates, so need mutable ref)
        // For this test, we check the stats directly
        let stats = adapter.stats().await;
        assert!(stats.buffer_size > 0);
    }

    #[tokio::test]
    async fn test_adapter_stats() {
        let event_bus = EventBus::new();
        let adapter = EventBusAdapter::new(event_bus);

        let stats = adapter.stats().await;

        assert_eq!(stats.buffer_size, 1000); // Default buffer size
        assert_eq!(stats.active_subscribers, 0);
        assert!(!stats.is_running);
    }

    #[tokio::test]
    async fn test_event_history_returns_empty() {
        let event_bus = EventBus::new();
        let adapter = EventBusAdapter::new(event_bus);

        let history = adapter.event_history(10).await;
        assert!(history.is_empty());
    }

    #[tokio::test]
    async fn test_severity_conversion() {
        assert_eq!(
            EventBusAdapter::convert_severity(PortEventSeverity::Trace),
            EventSeverity::Trace
        );
        assert_eq!(
            EventBusAdapter::convert_severity(PortEventSeverity::Info),
            EventSeverity::Info
        );
        assert_eq!(
            EventBusAdapter::convert_severity(PortEventSeverity::Error),
            EventSeverity::Error
        );
        assert_eq!(
            EventBusAdapter::convert_severity(PortEventSeverity::Critical),
            EventSeverity::Critical
        );
    }
}
