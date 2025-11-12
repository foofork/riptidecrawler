//! Event Bus Adapter for hexagonal architecture
//!
//! Adapts the concrete EventBus implementation to the EventBus port trait.

use async_trait::async_trait;
use riptide_types::ports::events::{
    EventBus as EventBusPort, EventHandler, DomainEvent, SubscriptionId,
};
use riptide_types::error::Result as RiptideResult;
use std::sync::Arc;

/// Adapter that implements the EventBus port trait for EventBus
pub struct EventBusAdapter {
    inner: Arc<riptide_events::EventBus>,
}

impl EventBusAdapter {
    /// Create a new EventBusAdapter wrapping the concrete implementation
    pub fn new(event_bus: Arc<riptide_events::EventBus>) -> Self {
        Self { inner: event_bus }
    }
}

#[async_trait]
impl EventBusPort for EventBusAdapter {
    async fn publish(&self, event: DomainEvent) -> RiptideResult<()> {
        // Convert DomainEvent to BaseEvent
        let mut base_event = riptide_events::BaseEvent::new(
            &event.event_type,
            &event.aggregate_id,
            riptide_events::EventSeverity::Info,
        );

        // Add metadata
        for (key, value) in &event.metadata {
            base_event.add_metadata(key, value);
        }

        // Add payload as metadata
        if let Ok(payload_str) = serde_json::to_string(&event.payload) {
            base_event.add_metadata("payload", &payload_str);
        }

        self.inner.emit(base_event).await.map_err(|e| {
            riptide_types::error::RiptideError::Internal {
                message: format!("Failed to publish event: {}", e),
            }
        })
    }

    async fn subscribe(&self, handler: Arc<dyn EventHandler>) -> RiptideResult<SubscriptionId> {
        // EventBus uses handler registration directly
        // Create a wrapper handler that adapts between port trait and internal trait
        struct HandlerAdapter {
            inner: Arc<dyn EventHandler>,
        }

        #[async_trait]
        impl riptide_events::EventHandler for HandlerAdapter {
            fn name(&self) -> &str {
                "event_bus_adapter"
            }

            fn can_handle(&self, _event_type: &str) -> bool {
                true // Handle all events
            }

            async fn handle(&self, event: &dyn riptide_events::Event) -> anyhow::Result<()> {
                // Convert Event to DomainEvent
                let payload = event
                    .metadata()
                    .get("payload")
                    .and_then(|p| serde_json::from_str(p).ok())
                    .unwrap_or(serde_json::json!({}));

                let domain_event = DomainEvent::new(
                    event.event_type().to_string(),
                    event.source().to_string(),
                    payload,
                );

                self.inner.handle(&domain_event).await.map_err(|e| {
                    anyhow::anyhow!("Handler error: {}", e)
                })
            }
        }

        let adapter = Arc::new(HandlerAdapter { inner: handler });

        self.inner.register_handler(adapter).await.map_err(|e| {
            riptide_types::error::RiptideError::Internal {
                message: format!("Failed to subscribe handler: {}", e),
            }
        })?;

        // Return a generated subscription ID
        Ok(uuid::Uuid::new_v4().to_string())
    }

    async fn unsubscribe(&self, _subscription_id: &str) -> RiptideResult<()> {
        // EventBus doesn't support unsubscribe by ID yet
        // This is a no-op for now
        Ok(())
    }

    async fn publish_batch(&self, events: Vec<DomainEvent>) -> RiptideResult<()> {
        // EventBus doesn't have batch support, publish sequentially
        for event in events {
            self.publish(event).await?;
        }
        Ok(())
    }
}
