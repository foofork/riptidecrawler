//! Comprehensive Event System for RipTide Core
//!
//! This module provides a flexible, type-safe event system that integrates with
//! the existing monitoring and telemetry infrastructure. It supports:
//!
//! - Event emission from pool operations and core components
//! - Event handlers for metrics collection, telemetry, and health monitoring
//! - Thread-safe event bus for centralized event coordination
//! - OpenTelemetry integration for distributed tracing
//! - Configurable event filtering and routing

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

pub mod bus;
pub mod handlers;
pub mod pool_integration;
pub mod types;

pub use bus::{EventBus, EventBusConfig, EventBusStats, EventRouting};
pub use handlers::{
    ComponentHealth, HealthEventHandler, LoggingEventHandler, MetricsEventHandler,
    TelemetryEventHandler,
};
pub use pool_integration::{
    EventAwareInstancePool, EventAwarePoolFactory, PoolEventConfig, PoolEventEmissionHelper,
};
pub use types::*;

/// Core event trait that all events must implement
pub trait Event: Send + Sync + Debug + Any {
    /// Event type identifier
    fn event_type(&self) -> &'static str;

    /// Event unique identifier
    fn event_id(&self) -> &str;

    /// Event timestamp
    fn timestamp(&self) -> DateTime<Utc>;

    /// Event source component
    fn source(&self) -> &str;

    /// Event severity level
    fn severity(&self) -> EventSeverity;

    /// Event metadata
    fn metadata(&self) -> &HashMap<String, String>;

    /// Serialize event to JSON
    fn to_json(&self) -> Result<String>;

    /// Check if event is critical (requires immediate handling)
    fn is_critical(&self) -> bool {
        matches!(
            self.severity(),
            EventSeverity::Critical | EventSeverity::Error
        )
    }

    /// Check if event should be traced (for OpenTelemetry)
    fn should_trace(&self) -> bool {
        true
    }
}

/// Event severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum EventSeverity {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    Critical = 5,
}

impl Display for EventSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventSeverity::Trace => write!(f, "TRACE"),
            EventSeverity::Debug => write!(f, "DEBUG"),
            EventSeverity::Info => write!(f, "INFO"),
            EventSeverity::Warn => write!(f, "WARN"),
            EventSeverity::Error => write!(f, "ERROR"),
            EventSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Trait for components that can emit events
#[async_trait]
pub trait EventEmitter {
    /// Emit an event
    async fn emit_event<E: Event + 'static>(&self, event: E) -> Result<()>;

    /// Emit multiple events in batch
    async fn emit_events<E: Event + 'static>(&self, events: Vec<E>) -> Result<()> {
        for event in events {
            self.emit_event(event).await?;
        }
        Ok(())
    }
}

/// Trait for event handlers
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Handler name for identification
    fn name(&self) -> &str;

    /// Check if this handler can handle the given event type
    fn can_handle(&self, event_type: &str) -> bool;

    /// Handle an event
    async fn handle(&self, event: &dyn Event) -> Result<()>;

    /// Handle event in batch (optional optimization)
    async fn handle_batch(&self, events: &[&dyn Event]) -> Result<()> {
        for event in events {
            self.handle(*event).await?;
        }
        Ok(())
    }

    /// Get handler configuration
    fn config(&self) -> HandlerConfig {
        HandlerConfig::default()
    }
}

/// Configuration for event handlers
#[derive(Debug, Clone)]
pub struct HandlerConfig {
    pub enabled: bool,
    pub event_types: Vec<String>,
    pub min_severity: EventSeverity,
    pub batch_size: usize,
    pub timeout: Duration,
    pub retry_attempts: u32,
}

impl Default for HandlerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            event_types: vec!["*".to_string()], // Handle all event types
            min_severity: EventSeverity::Info,
            batch_size: 100,
            timeout: Duration::from_secs(5),
            retry_attempts: 3,
        }
    }
}

/// Base event structure that implements common functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseEvent {
    pub event_id: String,
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub severity: EventSeverity,
    pub metadata: HashMap<String, String>,
    pub context: Option<String>, // Additional context data
}

impl BaseEvent {
    pub fn new(event_type: &str, source: &str, severity: EventSeverity) -> Self {
        Self {
            event_id: Uuid::new_v4().to_string(),
            event_type: event_type.to_string(),
            timestamp: Utc::now(),
            source: source.to_string(),
            severity,
            metadata: HashMap::new(),
            context: None,
        }
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }
}

/// Event subscription for receiving events
pub struct EventSubscription {
    receiver: broadcast::Receiver<Arc<dyn Event>>,
    event_types: Vec<String>,
    min_severity: EventSeverity,
}

impl EventSubscription {
    pub fn new(
        receiver: broadcast::Receiver<Arc<dyn Event>>,
        event_types: Vec<String>,
        min_severity: EventSeverity,
    ) -> Self {
        Self {
            receiver,
            event_types,
            min_severity,
        }
    }

    /// Receive next matching event
    pub async fn recv(&mut self) -> Result<Arc<dyn Event>, broadcast::error::RecvError> {
        loop {
            match self.receiver.recv().await {
                Ok(event) => {
                    // Check if event matches subscription criteria
                    if self.should_include(&*event) {
                        return Ok(event);
                    }
                    // Continue to next event if this one doesn't match
                }
                Err(e) => return Err(e),
            }
        }
    }

    fn should_include(&self, event: &dyn Event) -> bool {
        // Check severity
        if event.severity() < self.min_severity {
            return false;
        }

        // Check event types
        if self.event_types.is_empty() || self.event_types.contains(&"*".to_string()) {
            return true;
        }

        self.event_types.contains(&event.event_type().to_string())
    }
}

/// Helper macros for creating events
#[macro_export]
macro_rules! emit_info_event {
    ($emitter:expr, $event_type:expr, $source:expr, $($key:expr => $value:expr),*) => {
        {
            let mut event = $crate::events::BaseEvent::new($event_type, $source, $crate::events::EventSeverity::Info);
            $(
                event.add_metadata($key, $value);
            )*
            $emitter.emit_event(event).await
        }
    };
}

#[macro_export]
macro_rules! emit_error_event {
    ($emitter:expr, $event_type:expr, $source:expr, $error:expr, $($key:expr => $value:expr),*) => {
        {
            let mut event = $crate::events::BaseEvent::new($event_type, $source, $crate::events::EventSeverity::Error);
            event.add_metadata("error", &$error.to_string());
            $(
                event.add_metadata($key, $value);
            )*
            $emitter.emit_event(event).await
        }
    };
}

#[macro_export]
macro_rules! emit_warning_event {
    ($emitter:expr, $event_type:expr, $source:expr, $message:expr, $($key:expr => $value:expr),*) => {
        {
            let mut event = $crate::events::BaseEvent::new($event_type, $source, $crate::events::EventSeverity::Warn);
            event.add_metadata("message", $message);
            $(
                event.add_metadata($key, $value);
            )*
            $emitter.emit_event(event).await
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_severity_ordering() {
        assert!(EventSeverity::Trace < EventSeverity::Debug);
        assert!(EventSeverity::Debug < EventSeverity::Info);
        assert!(EventSeverity::Info < EventSeverity::Warn);
        assert!(EventSeverity::Warn < EventSeverity::Error);
        assert!(EventSeverity::Error < EventSeverity::Critical);
    }

    #[test]
    fn test_base_event_creation() {
        let event = BaseEvent::new("test.event", "test_source", EventSeverity::Info);

        assert_eq!(event.event_type, "test.event");
        assert_eq!(event.source, "test_source");
        assert_eq!(event.severity, EventSeverity::Info);
        assert!(!event.event_id.is_empty());
        assert!(event.metadata.is_empty());
    }

    #[test]
    fn test_base_event_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("key1".to_string(), "value1".to_string());

        let event = BaseEvent::new("test.event", "test_source", EventSeverity::Info)
            .with_metadata(metadata);

        assert_eq!(event.metadata.get("key1"), Some(&"value1".to_string()));
    }

    #[test]
    fn test_handler_config_default() {
        let config = HandlerConfig::default();

        assert!(config.enabled);
        assert_eq!(config.event_types, vec!["*"]);
        assert_eq!(config.min_severity, EventSeverity::Info);
        assert_eq!(config.batch_size, 100);
        assert_eq!(config.retry_attempts, 3);
    }
}
