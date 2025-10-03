//! Comprehensive tests for RipTide Core Event System
//!
//! Tests the event system including:
//! - Event trait implementations
//! - Event bus functionality
//! - Event handlers
//! - Event routing and filtering
//! - Concurrency and thread safety

use anyhow::Result;
use riptide_core::events::{
    Event, EventSeverity, EventEmitter, EventHandler, EventBus, EventBusConfig,
    BaseEvent, HandlerConfig,
};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use chrono::Utc;

// ============================================================================
// Unit Tests: BaseEvent and Event Trait
// ============================================================================

#[cfg(test)]
mod base_event_tests {
    use super::*;

    #[test]
    fn test_base_event_creation() {
        let event = BaseEvent::new("test.event", "test_source", EventSeverity::Info);

        assert_eq!(event.event_type, "test.event");
        assert_eq!(event.source, "test_source");
        assert_eq!(event.severity, EventSeverity::Info);
        assert!(!event.event_id.is_empty());
        assert!(event.metadata.is_empty());
        assert!(event.context.is_none());
    }

    #[test]
    fn test_base_event_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("key1".to_string(), "value1".to_string());
        metadata.insert("key2".to_string(), "value2".to_string());

        let event = BaseEvent::new("test.event", "test_source", EventSeverity::Info)
            .with_metadata(metadata);

        assert_eq!(event.metadata.len(), 2);
        assert_eq!(event.metadata.get("key1"), Some(&"value1".to_string()));
        assert_eq!(event.metadata.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_base_event_with_context() {
        let event = BaseEvent::new("test.event", "test_source", EventSeverity::Info)
            .with_context("Additional context data".to_string());

        assert_eq!(event.context, Some("Additional context data".to_string()));
    }

    #[test]
    fn test_base_event_add_metadata() {
        let mut event = BaseEvent::new("test.event", "test_source", EventSeverity::Info);

        event.add_metadata("dynamic_key", "dynamic_value");

        assert_eq!(event.metadata.get("dynamic_key"), Some(&"dynamic_value".to_string()));
    }

    #[test]
    fn test_base_event_serialization() {
        let event = BaseEvent::new("test.event", "test_source", EventSeverity::Info)
            .with_context("test context".to_string());

        let json = serde_json::to_string(&event);
        assert!(json.is_ok());

        let json_str = json.unwrap();
        assert!(json_str.contains("test.event"));
        assert!(json_str.contains("test_source"));
        assert!(json_str.contains("Info"));
    }

    #[test]
    fn test_base_event_unique_ids() {
        let event1 = BaseEvent::new("test.event", "test_source", EventSeverity::Info);
        let event2 = BaseEvent::new("test.event", "test_source", EventSeverity::Info);

        assert_ne!(event1.event_id, event2.event_id);
    }
}

// ============================================================================
// Unit Tests: EventSeverity
// ============================================================================

#[cfg(test)]
mod event_severity_tests {
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
    fn test_event_severity_display() {
        assert_eq!(EventSeverity::Trace.to_string(), "TRACE");
        assert_eq!(EventSeverity::Debug.to_string(), "DEBUG");
        assert_eq!(EventSeverity::Info.to_string(), "INFO");
        assert_eq!(EventSeverity::Warn.to_string(), "WARN");
        assert_eq!(EventSeverity::Error.to_string(), "ERROR");
        assert_eq!(EventSeverity::Critical.to_string(), "CRITICAL");
    }

    #[test]
    fn test_event_severity_equality() {
        assert_eq!(EventSeverity::Info, EventSeverity::Info);
        assert_ne!(EventSeverity::Info, EventSeverity::Warn);
    }

    #[test]
    fn test_event_severity_serialization() {
        let severity = EventSeverity::Warn;
        let json = serde_json::to_string(&severity).unwrap();

        let deserialized: EventSeverity = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, EventSeverity::Warn);
    }

    #[test]
    fn test_event_severity_numeric_values() {
        assert_eq!(EventSeverity::Trace as u32, 0);
        assert_eq!(EventSeverity::Debug as u32, 1);
        assert_eq!(EventSeverity::Info as u32, 2);
        assert_eq!(EventSeverity::Warn as u32, 3);
        assert_eq!(EventSeverity::Error as u32, 4);
        assert_eq!(EventSeverity::Critical as u32, 5);
    }
}

// ============================================================================
// Unit Tests: HandlerConfig
// ============================================================================

#[cfg(test)]
mod handler_config_tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_handler_config_default() {
        let config = HandlerConfig::default();

        assert!(config.enabled);
        assert_eq!(config.event_types, vec!["*"]);
        assert_eq!(config.min_severity, EventSeverity::Info);
        assert_eq!(config.batch_size, 100);
        assert_eq!(config.timeout, Duration::from_secs(5));
        assert_eq!(config.retry_attempts, 3);
    }

    #[test]
    fn test_handler_config_custom() {
        let config = HandlerConfig {
            enabled: false,
            event_types: vec!["search.*".to_string(), "pool.*".to_string()],
            min_severity: EventSeverity::Warn,
            batch_size: 50,
            timeout: Duration::from_secs(10),
            retry_attempts: 5,
        };

        assert!(!config.enabled);
        assert_eq!(config.event_types.len(), 2);
        assert_eq!(config.min_severity, EventSeverity::Warn);
        assert_eq!(config.batch_size, 50);
        assert_eq!(config.retry_attempts, 5);
    }
}

// ============================================================================
// Mock Event Handler for Testing
// ============================================================================

struct MockEventHandler {
    name: String,
    handled_events: Arc<Mutex<Vec<String>>>,
    event_types: Vec<String>,
}

impl MockEventHandler {
    fn new(name: &str, event_types: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            handled_events: Arc::new(Mutex::new(Vec::new())),
            event_types,
        }
    }

    fn get_handled_events(&self) -> Vec<String> {
        self.handled_events.lock().unwrap().clone()
    }
}

#[async_trait::async_trait]
impl EventHandler for MockEventHandler {
    fn name(&self) -> &str {
        &self.name
    }

    fn can_handle(&self, event_type: &str) -> bool {
        self.event_types.iter().any(|t| {
            t == "*" || t == event_type || event_type.starts_with(t.trim_end_matches('*'))
        })
    }

    async fn handle(&self, event: &dyn Event) -> Result<()> {
        let mut events = self.handled_events.lock().unwrap();
        events.push(event.event_type().to_string());
        Ok(())
    }
}

// ============================================================================
// Unit Tests: EventHandler Trait
// ============================================================================

#[cfg(test)]
mod event_handler_tests {
    use super::*;

    #[tokio::test]
    async fn test_event_handler_can_handle_wildcard() {
        let handler = MockEventHandler::new("test_handler", vec!["*".to_string()]);

        assert!(handler.can_handle("any.event.type"));
        assert!(handler.can_handle("search.started"));
        assert!(handler.can_handle("pool.created"));
    }

    #[tokio::test]
    async fn test_event_handler_can_handle_specific() {
        let handler = MockEventHandler::new(
            "test_handler",
            vec!["search.started".to_string(), "pool.created".to_string()]
        );

        assert!(handler.can_handle("search.started"));
        assert!(handler.can_handle("pool.created"));
        assert!(!handler.can_handle("other.event"));
    }

    #[tokio::test]
    async fn test_event_handler_can_handle_prefix() {
        let handler = MockEventHandler::new("test_handler", vec!["search.*".to_string()]);

        assert!(handler.can_handle("search.started"));
        assert!(handler.can_handle("search.completed"));
        assert!(handler.can_handle("search.failed"));
        assert!(!handler.can_handle("pool.created"));
    }

    #[tokio::test]
    async fn test_event_handler_handle_event() {
        let handler = MockEventHandler::new("test_handler", vec!["*".to_string()]);
        let event = BaseEvent::new("test.event", "test_source", EventSeverity::Info);

        // Convert BaseEvent to &dyn Event
        let event_ref: &dyn Event = &event;
        let result = handler.handle(event_ref).await;

        assert!(result.is_ok());
        assert_eq!(handler.get_handled_events().len(), 1);
        assert_eq!(handler.get_handled_events()[0], "test.event");
    }

    #[tokio::test]
    async fn test_event_handler_handle_multiple_events() {
        let handler = MockEventHandler::new("test_handler", vec!["*".to_string()]);

        let event1 = BaseEvent::new("event.1", "source", EventSeverity::Info);
        let event2 = BaseEvent::new("event.2", "source", EventSeverity::Warn);
        let event3 = BaseEvent::new("event.3", "source", EventSeverity::Error);

        let _ = handler.handle(&event1 as &dyn Event).await;
        let _ = handler.handle(&event2 as &dyn Event).await;
        let _ = handler.handle(&event3 as &dyn Event).await;

        let handled = handler.get_handled_events();
        assert_eq!(handled.len(), 3);
        assert!(handled.contains(&"event.1".to_string()));
        assert!(handled.contains(&"event.2".to_string()));
        assert!(handled.contains(&"event.3".to_string()));
    }
}

// ============================================================================
// Unit Tests: EventBusConfig
// ============================================================================

#[cfg(test)]
mod event_bus_config_tests {
    use super::*;
    use riptide_core::events::EventBusConfig;

    #[test]
    fn test_event_bus_config_default() {
        let config = EventBusConfig::default();

        assert_eq!(config.buffer_size, 1000);
        assert_eq!(config.min_severity, EventSeverity::Info);
        assert!(config.event_types.is_empty());
    }

    #[test]
    fn test_event_bus_config_custom() {
        let config = EventBusConfig {
            buffer_size: 5000,
            min_severity: EventSeverity::Warn,
            event_types: vec!["search.*".to_string(), "pool.*".to_string()],
        };

        assert_eq!(config.buffer_size, 5000);
        assert_eq!(config.min_severity, EventSeverity::Warn);
        assert_eq!(config.event_types.len(), 2);
    }
}

// ============================================================================
// Concurrency and Thread Safety Tests
// ============================================================================

#[cfg(test)]
mod concurrency_tests {
    use super::*;
    use tokio::task::JoinSet;

    #[tokio::test]
    async fn test_concurrent_event_handling() {
        let handler = Arc::new(MockEventHandler::new("test_handler", vec!["*".to_string()]));
        let mut set = JoinSet::new();

        // Spawn multiple concurrent event handling tasks
        for i in 0..20 {
            let handler_clone = handler.clone();
            set.spawn(async move {
                let event = BaseEvent::new(
                    &format!("event.{}", i),
                    "test_source",
                    EventSeverity::Info
                );
                handler_clone.handle(&event as &dyn Event).await
            });
        }

        let mut success_count = 0;
        while let Some(result) = set.join_next().await {
            if let Ok(Ok(_)) = result {
                success_count += 1;
            }
        }

        assert_eq!(success_count, 20);
        assert_eq!(handler.get_handled_events().len(), 20);
    }

    #[tokio::test]
    async fn test_event_handler_thread_safety() {
        let handler = Arc::new(MockEventHandler::new("test_handler", vec!["*".to_string()]));

        // Verify Send + Sync bounds
        fn is_send_sync<T: Send + Sync>() {}
        is_send_sync::<MockEventHandler>();

        let handles: Vec<_> = (0..4).map(|i| {
            let handler_clone = handler.clone();
            tokio::spawn(async move {
                for j in 0..5 {
                    let event = BaseEvent::new(
                        &format!("event.{}.{}", i, j),
                        "source",
                        EventSeverity::Info
                    );
                    let _ = handler_clone.handle(&event as &dyn Event).await;
                }
            })
        }).collect();

        for handle in handles {
            handle.await.unwrap();
        }

        assert_eq!(handler.get_handled_events().len(), 20);
    }
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_empty_event_type() {
        let event = BaseEvent::new("", "test_source", EventSeverity::Info);
        assert_eq!(event.event_type, "");
    }

    #[test]
    fn test_empty_source() {
        let event = BaseEvent::new("test.event", "", EventSeverity::Info);
        assert_eq!(event.source, "");
    }

    #[test]
    fn test_very_long_event_type() {
        let long_type = "a".repeat(1000);
        let event = BaseEvent::new(&long_type, "source", EventSeverity::Info);
        assert_eq!(event.event_type.len(), 1000);
    }

    #[test]
    fn test_large_metadata() {
        let mut event = BaseEvent::new("test.event", "source", EventSeverity::Info);

        for i in 0..100 {
            event.add_metadata(&format!("key{}", i), &format!("value{}", i));
        }

        assert_eq!(event.metadata.len(), 100);
    }

    #[test]
    fn test_special_characters_in_metadata() {
        let mut event = BaseEvent::new("test.event", "source", EventSeverity::Info);

        event.add_metadata("key_with_spaces", "value with spaces");
        event.add_metadata("key-with-dashes", "value-with-dashes");
        event.add_metadata("key.with.dots", "value.with.dots");
        event.add_metadata("key/with/slashes", "value/with/slashes");

        assert_eq!(event.metadata.len(), 4);
    }

    #[tokio::test]
    async fn test_handler_with_empty_event_types() {
        let handler = MockEventHandler::new("test_handler", vec![]);

        assert!(!handler.can_handle("any.event"));
        assert!(!handler.can_handle("search.started"));
    }

    #[tokio::test]
    async fn test_handler_with_duplicate_event_types() {
        let handler = MockEventHandler::new(
            "test_handler",
            vec!["search.*".to_string(), "search.*".to_string()]
        );

        assert!(handler.can_handle("search.started"));
    }
}

// ============================================================================
// Performance Tests
// ============================================================================

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_event_creation_performance() {
        let start = Instant::now();

        for i in 0..1000 {
            let _ = BaseEvent::new(
                &format!("event.{}", i),
                "source",
                EventSeverity::Info
            );
        }

        let elapsed = start.elapsed();
        assert!(elapsed < std::time::Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_event_handling_performance() {
        let handler = MockEventHandler::new("test_handler", vec!["*".to_string()]);
        let start = Instant::now();

        for i in 0..1000 {
            let event = BaseEvent::new(
                &format!("event.{}", i),
                "source",
                EventSeverity::Info
            );
            let _ = handler.handle(&event as &dyn Event).await;
        }

        let elapsed = start.elapsed();
        assert!(elapsed < std::time::Duration::from_secs(1));
        assert_eq!(handler.get_handled_events().len(), 1000);
    }

    #[tokio::test]
    async fn test_metadata_access_performance() {
        let mut event = BaseEvent::new("test.event", "source", EventSeverity::Info);

        for i in 0..100 {
            event.add_metadata(&format!("key{}", i), &format!("value{}", i));
        }

        let start = Instant::now();

        for i in 0..100 {
            let _ = event.metadata.get(&format!("key{}", i));
        }

        let elapsed = start.elapsed();
        assert!(elapsed < std::time::Duration::from_millis(10));
    }
}
