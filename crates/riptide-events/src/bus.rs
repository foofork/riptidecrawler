//! Event Bus implementation for centralized event handling
//!
//! This module provides a thread-safe event bus that can route events to multiple
//! handlers based on configuration and event types.

use super::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::task::JoinHandle;

/// Configuration for the Event Bus
#[derive(Debug, Clone)]
pub struct EventBusConfig {
    /// Maximum number of events that can be buffered
    pub buffer_size: usize,
    /// Whether to enable async handler execution
    pub async_handlers: bool,
    /// Timeout for handler execution
    pub handler_timeout: Duration,
    /// Whether to continue processing if a handler fails
    pub continue_on_handler_error: bool,
}

impl Default for EventBusConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1000,
            async_handlers: true,
            handler_timeout: Duration::from_secs(5),
            continue_on_handler_error: true,
        }
    }
}

/// Event routing configuration
#[derive(Debug, Clone, Default)]
pub enum EventRouting {
    /// All handlers receive all events
    #[default]
    Broadcast,
    /// Events are routed based on event type patterns
    PatternBased(HashMap<String, Vec<String>>), // event_pattern -> handler_names
    /// Events are routed based on severity levels
    SeverityBased(HashMap<EventSeverity, Vec<String>>), // severity -> handler_names
    /// Custom routing function
    Custom,
}

/// Central event bus for coordinating event emission and handling
pub struct EventBus {
    config: EventBusConfig,
    routing: EventRouting,
    handlers: Arc<RwLock<HashMap<String, Arc<dyn EventHandler>>>>,
    sender: broadcast::Sender<Arc<dyn Event>>,
    _receiver: broadcast::Receiver<Arc<dyn Event>>, // Keep for capacity
    running: Arc<std::sync::atomic::AtomicBool>,
    handler_task: Option<JoinHandle<()>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl EventBus {
    /// Create a new EventBus with default configuration
    pub fn new() -> Self {
        Self::with_config(EventBusConfig::default())
    }

    /// Create a new EventBus with custom configuration
    pub fn with_config(config: EventBusConfig) -> Self {
        let (sender, receiver) = broadcast::channel(config.buffer_size);

        Self {
            config,
            routing: EventRouting::default(),
            handlers: Arc::new(RwLock::new(HashMap::new())),
            sender,
            _receiver: receiver,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            handler_task: None,
            shutdown_tx: None,
        }
    }

    /// Start the event bus processing
    pub async fn start(&mut self) -> Result<()> {
        if self.running.load(std::sync::atomic::Ordering::Relaxed) {
            return Ok(());
        }

        self.running
            .store(true, std::sync::atomic::Ordering::Relaxed);

        let mut receiver = self.sender.subscribe();
        let handlers = self.handlers.clone();
        let config = self.config.clone();
        let routing = self.routing.clone();

        // Create shutdown channel
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);

        let handler_task = tokio::spawn(async move {
            info!("Event bus started processing");

            loop {
                tokio::select! {
                    result = receiver.recv() => {
                        match result {
                            Ok(event) => {
                                let handlers_map = handlers.read().await;
                                let target_handlers =
                                    Self::get_target_handlers(&handlers_map, &*event, &routing);

                                if config.async_handlers {
                                    // Process handlers concurrently
                                    let mut handler_futures = Vec::new();

                                    for (handler_name, handler) in target_handlers {
                                        let event_clone = event.clone();
                                        let handler_timeout = config.handler_timeout;

                                        let future = tokio::spawn(async move {
                                            let result = tokio::time::timeout(
                                                handler_timeout,
                                                handler.handle(&*event_clone),
                                            )
                                            .await;

                                            match result {
                                                Ok(Ok(_)) => {
                                                    debug!(handler = %handler_name, event_id = %event_clone.event_id(), "Handler processed event successfully");
                                                }
                                                Ok(Err(e)) => {
                                                    error!(handler = %handler_name, event_id = %event_clone.event_id(), error = %e, "Handler failed to process event");
                                                }
                                                Err(_) => {
                                                    warn!(handler = %handler_name, event_id = %event_clone.event_id(), "Handler timed out processing event");
                                                }
                                            }
                                        });

                                        handler_futures.push(future);
                                    }

                                    // Wait for all handlers to complete
                                    futures::future::join_all(handler_futures).await;
                                } else {
                                    // Process handlers sequentially
                                    for (handler_name, handler) in target_handlers {
                                        let start = Instant::now();

                                        match tokio::time::timeout(
                                            config.handler_timeout,
                                            handler.handle(&*event),
                                        )
                                        .await
                                        {
                                            Ok(Ok(_)) => {
                                                debug!(
                                                    handler = %handler_name,
                                                    event_id = %event.event_id(),
                                                    duration_ms = %start.elapsed().as_millis(),
                                                    "Handler processed event successfully"
                                                );
                                            }
                                            Ok(Err(e)) => {
                                                error!(
                                                    handler = %handler_name,
                                                    event_id = %event.event_id(),
                                                    error = %e,
                                                    "Handler failed to process event"
                                                );

                                                if !config.continue_on_handler_error {
                                                    break;
                                                }
                                            }
                                            Err(_) => {
                                                warn!(
                                                    handler = %handler_name,
                                                    event_id = %event.event_id(),
                                                    "Handler timed out processing event"
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                            Err(broadcast::error::RecvError::Closed) => {
                                info!("Event bus channel closed, stopping processing");
                                break;
                            }
                            Err(broadcast::error::RecvError::Lagged(count)) => {
                                warn!(lagged_events = %count, "Event bus receiver lagged, some events may have been lost");
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Event bus received shutdown signal");
                        break;
                    }
                }
            }

            info!("Event bus stopped processing");
        });

        self.handler_task = Some(handler_task);
        Ok(())
    }

    /// Stop the event bus processing
    pub async fn stop(&mut self) {
        self.running
            .store(false, std::sync::atomic::Ordering::Relaxed);

        // Send shutdown signal
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(()).await;
        }

        // Wait for handler task to complete
        if let Some(handler_task) = self.handler_task.take() {
            let _ = handler_task.await;
        }
    }

    /// Register an event handler
    pub async fn register_handler(&self, handler: Arc<dyn EventHandler>) -> Result<()> {
        let handler_name = handler.name().to_string();
        let mut handlers = self.handlers.write().await;

        if handlers.contains_key(&handler_name) {
            return Err(anyhow::anyhow!(
                "Handler '{}' is already registered",
                handler_name
            ));
        }

        handlers.insert(handler_name.clone(), handler);
        info!(handler_name = %handler_name, "Registered event handler");

        Ok(())
    }

    /// Unregister an event handler
    pub async fn unregister_handler(&self, handler_name: &str) -> Result<()> {
        let mut handlers = self.handlers.write().await;

        if handlers.remove(handler_name).is_some() {
            info!(handler_name = %handler_name, "Unregistered event handler");
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Handler '{}' is not registered",
                handler_name
            ))
        }
    }

    /// Get list of registered handler names
    pub async fn get_handler_names(&self) -> Vec<String> {
        let handlers = self.handlers.read().await;
        handlers.keys().cloned().collect()
    }

    /// Set event routing configuration
    pub fn set_routing(&mut self, routing: EventRouting) {
        self.routing = routing;
    }

    /// Create a subscription for receiving events
    pub fn subscribe(
        &self,
        event_types: Vec<String>,
        min_severity: EventSeverity,
    ) -> EventSubscription {
        let receiver = self.sender.subscribe();
        EventSubscription::new(receiver, event_types, min_severity)
    }

    /// Get current event bus statistics
    pub fn get_stats(&self) -> EventBusStats {
        EventBusStats {
            buffer_size: self.config.buffer_size,
            // Subtract 1 to account for the internal _receiver
            current_subscribers: self.sender.receiver_count().saturating_sub(1),
            is_running: self.running.load(std::sync::atomic::Ordering::Relaxed),
        }
    }

    /// Convenience method to emit an event (alias for emit_event)
    pub async fn emit<E: Event + 'static>(&self, event: E) -> Result<()> {
        self.emit_event(event).await
    }

    /// Get target handlers for an event based on routing configuration
    fn get_target_handlers(
        handlers: &HashMap<String, Arc<dyn EventHandler>>,
        event: &dyn Event,
        routing: &EventRouting,
    ) -> Vec<(String, Arc<dyn EventHandler>)> {
        match routing {
            EventRouting::Broadcast => handlers
                .iter()
                .filter(|(_, handler)| handler.can_handle(event.event_type()))
                .map(|(name, handler)| (name.clone(), handler.clone()))
                .collect(),
            EventRouting::PatternBased(patterns) => {
                let mut target_handlers = Vec::new();

                for (pattern, handler_names) in patterns {
                    if Self::event_matches_pattern(event.event_type(), pattern) {
                        for handler_name in handler_names {
                            if let Some(handler) = handlers.get(handler_name) {
                                if handler.can_handle(event.event_type()) {
                                    target_handlers.push((handler_name.clone(), handler.clone()));
                                }
                            }
                        }
                    }
                }

                target_handlers
            }
            EventRouting::SeverityBased(severity_map) => {
                let mut target_handlers = Vec::new();

                if let Some(handler_names) = severity_map.get(&event.severity()) {
                    for handler_name in handler_names {
                        if let Some(handler) = handlers.get(handler_name) {
                            if handler.can_handle(event.event_type()) {
                                target_handlers.push((handler_name.clone(), handler.clone()));
                            }
                        }
                    }
                }

                target_handlers
            }
            EventRouting::Custom => {
                // For custom routing, fall back to broadcast for now
                handlers
                    .iter()
                    .filter(|(_, handler)| handler.can_handle(event.event_type()))
                    .map(|(name, handler)| (name.clone(), handler.clone()))
                    .collect()
            }
        }
    }

    /// Check if an event type matches a pattern
    fn event_matches_pattern(event_type: &str, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }

        if let Some(prefix) = pattern.strip_suffix('*') {
            return event_type.starts_with(prefix);
        }

        event_type == pattern
    }
}

#[async_trait]
impl EventEmitter for EventBus {
    async fn emit_event<E: Event + 'static>(&self, event: E) -> Result<()> {
        let arc_event: Arc<dyn Event> = Arc::new(event);

        match self.sender.send(arc_event.clone()) {
            Ok(subscriber_count) => {
                debug!(
                    event_id = %arc_event.event_id(),
                    event_type = %arc_event.event_type(),
                    subscribers = %subscriber_count,
                    "Event emitted successfully"
                );
                Ok(())
            }
            Err(_) => {
                // Channel is closed or no receivers
                warn!(
                    event_id = %arc_event.event_id(),
                    event_type = %arc_event.event_type(),
                    "Failed to emit event - no active subscribers"
                );
                Err(anyhow::anyhow!("No active subscribers for event"))
            }
        }
    }
}

impl Drop for EventBus {
    fn drop(&mut self) {
        // Stop processing if still running
        self.running
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Event bus statistics
#[derive(Debug, Clone)]
pub struct EventBusStats {
    pub buffer_size: usize,
    pub current_subscribers: usize,
    pub is_running: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::LoggingEventHandler;

    #[tokio::test]
    async fn test_event_bus_creation() {
        let bus = EventBus::new();
        let stats = bus.get_stats();

        assert_eq!(stats.buffer_size, 1000);
        assert!(!stats.is_running);
        assert_eq!(stats.current_subscribers, 0);
    }

    #[tokio::test]
    async fn test_handler_registration() {
        let bus = EventBus::new();
        let handler = Arc::new(LoggingEventHandler::new());

        // Register handler
        let result = bus.register_handler(handler.clone()).await;
        assert!(result.is_ok());

        let handler_names = bus.get_handler_names().await;
        assert!(handler_names.contains(&"logging_handler".to_string()));

        // Try to register same handler again (should fail)
        let result = bus.register_handler(handler).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handler_unregistration() {
        let bus = EventBus::new();
        let handler = Arc::new(LoggingEventHandler::new());

        // Register and then unregister
        assert!(bus.register_handler(handler).await.is_ok());
        let result = bus.unregister_handler("logging_handler").await;
        assert!(result.is_ok());

        let handler_names = bus.get_handler_names().await;
        assert!(!handler_names.contains(&"logging_handler".to_string()));

        // Try to unregister non-existent handler
        let result = bus.unregister_handler("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_event_pattern_matching() {
        assert!(EventBus::event_matches_pattern("test.event", "*"));
        assert!(EventBus::event_matches_pattern("test.event", "test.*"));
        assert!(EventBus::event_matches_pattern("test.event", "test.event"));
        assert!(!EventBus::event_matches_pattern("test.event", "other.*"));
        assert!(!EventBus::event_matches_pattern("test.event", "test.other"));
    }

    #[tokio::test]
    async fn test_event_subscription() {
        let bus = EventBus::new();

        let subscription = bus.subscribe(vec!["test.*".to_string()], EventSeverity::Info);

        assert!(subscription.receiver.is_empty());
    }

    #[tokio::test]
    async fn test_event_bus_emit_without_subscribers() {
        let bus = EventBus::new();
        let event = BaseEvent::new("test.event", "source", EventSeverity::Info);

        // Should succeed even without external subscribers (internal receiver exists)
        let result = bus.emit_event(event).await;
        assert!(result.is_ok());

        // Verify no external subscribers
        let stats = bus.get_stats();
        assert_eq!(stats.current_subscribers, 0);
    }

    #[tokio::test]
    async fn test_event_bus_with_custom_config() {
        let config = EventBusConfig {
            buffer_size: 500,
            async_handlers: false,
            handler_timeout: Duration::from_secs(10),
            continue_on_handler_error: false,
        };

        let bus = EventBus::with_config(config);
        let stats = bus.get_stats();

        assert_eq!(stats.buffer_size, 500);
        assert!(!stats.is_running);
    }
}
