//! Comprehensive tests for the Event System
//!
//! This module tests event emission, handling, subscription, and integration
//! with various event handlers including logging, metrics, telemetry, and health.

use riptide_core::events::{
    Event, EventEmitter, EventHandler, EventSeverity, EventSubscription,
    BaseEvent, HandlerConfig,
    types::{PoolEvent, PoolOperation, PoolMetrics, ExtractionEvent, ExtractionOperation,
           HealthEvent, HealthStatus, MetricsEvent, MetricType, SystemEvent},
    handlers::{LoggingEventHandler, HealthEventHandler, TelemetryEventHandler, ComponentHealth}
};
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::broadcast;
use std::sync::Arc;
use chrono::Utc;
use anyhow::Result;
use async_trait::async_trait;

// Mock EventEmitter for testing
struct MockEventEmitter {
    sender: broadcast::Sender<Arc<dyn Event>>,
}

impl MockEventEmitter {
    fn new() -> (Self, broadcast::Receiver<Arc<dyn Event>>) {
        let (sender, receiver) = broadcast::channel(100);
        (Self { sender }, receiver)
    }
}

#[async_trait]
impl EventEmitter for MockEventEmitter {
    async fn emit_event<E: Event + 'static>(&self, event: E) -> Result<()> {
        let arc_event: Arc<dyn Event> = Arc::new(event);
        let _ = self.sender.send(arc_event);
        Ok(())
    }
}

#[cfg(test)]
mod event_system_tests {
    use super::*;

    /// Test Event trait implementation for BaseEvent
    #[test]
    fn test_base_event_implementation() {
        let mut event = BaseEvent::new("test.event", "test_source", EventSeverity::Info);
        event.add_metadata("key1", "value1");

        assert_eq!(event.event_type(), "test.event");
        assert!(!event.event_id().is_empty());
        assert_eq!(event.source(), "test_source");
        assert_eq!(event.severity(), EventSeverity::Info);
        assert!(!event.is_critical());
        assert!(event.should_trace());

        // Test metadata
        assert_eq!(event.metadata().get("key1"), Some(&"value1".to_string()));

        // Test JSON serialization
        let json = event.to_json();
        assert!(json.is_ok());
        assert!(json.unwrap().contains("test.event"));
    }

    /// Test Event trait implementation for specific event types
    #[test]
    fn test_pool_event_implementation() {
        let pool_metrics = PoolMetrics {
            available_instances: 5,
            active_instances: 3,
            total_instances: 8,
            pending_acquisitions: 2,
            success_rate: 0.95,
            avg_acquisition_time_ms: 150,
        };

        let event = PoolEvent::new(
            PoolOperation::InstanceCreated,
            "test-pool".to_string(),
            "pool_manager"
        )
        .with_instance_id("instance-123".to_string())
        .with_metrics(pool_metrics);

        assert_eq!(event.event_type(), "pool.operation");
        assert_eq!(event.severity(), EventSeverity::Info);
        assert_eq!(event.source(), "pool_manager");

        // Test JSON serialization
        let json = event.to_json();
        assert!(json.is_ok());
        assert!(json.unwrap().contains("test-pool"));
    }

    /// Test severity-based event classification
    #[test]
    fn test_event_severity_classification() {
        let critical_event = BaseEvent::new("test", "source", EventSeverity::Critical);
        let error_event = BaseEvent::new("test", "source", EventSeverity::Error);
        let warn_event = BaseEvent::new("test", "source", EventSeverity::Warn);
        let info_event = BaseEvent::new("test", "source", EventSeverity::Info);
        let debug_event = BaseEvent::new("test", "source", EventSeverity::Debug);

        assert!(critical_event.is_critical());
        assert!(error_event.is_critical());
        assert!(!warn_event.is_critical());
        assert!(!info_event.is_critical());
        assert!(!debug_event.is_critical());
    }

    /// Test event severity ordering
    #[test]
    fn test_event_severity_ordering() {
        assert!(EventSeverity::Trace < EventSeverity::Debug);
        assert!(EventSeverity::Debug < EventSeverity::Info);
        assert!(EventSeverity::Info < EventSeverity::Warn);
        assert!(EventSeverity::Warn < EventSeverity::Error);
        assert!(EventSeverity::Error < EventSeverity::Critical);
    }

    /// Test ExtractionEvent with various operations
    #[test]
    fn test_extraction_event_operations() {
        // Test successful extraction
        let success_event = ExtractionEvent::new(
            ExtractionOperation::Completed,
            "https://example.com".to_string(),
            "article".to_string(),
            "extractor"
        )
        .with_duration(Duration::from_millis(250))
        .with_content_length(5000);

        assert_eq!(success_event.severity(), EventSeverity::Info);
        assert_eq!(success_event.url, "https://example.com");
        assert_eq!(success_event.duration_ms, Some(250));
        assert_eq!(success_event.content_length, Some(5000));

        // Test failed extraction
        let failed_event = ExtractionEvent::new(
            ExtractionOperation::Failed,
            "https://example.com".to_string(),
            "article".to_string(),
            "extractor"
        )
        .with_error("Network timeout".to_string());

        assert_eq!(failed_event.severity(), EventSeverity::Error);
        assert_eq!(failed_event.error_details, Some("Network timeout".to_string()));

        // Test timeout event
        let timeout_event = ExtractionEvent::new(
            ExtractionOperation::Timeout,
            "https://slow-site.com".to_string(),
            "article".to_string(),
            "extractor"
        );

        assert_eq!(timeout_event.severity(), EventSeverity::Error);

        // Test fallback event
        let fallback_event = ExtractionEvent::new(
            ExtractionOperation::FallbackUsed,
            "https://example.com".to_string(),
            "article".to_string(),
            "extractor"
        );

        assert_eq!(fallback_event.severity(), EventSeverity::Warn);
    }

    /// Test HealthEvent status mapping
    #[test]
    fn test_health_event_status_mapping() {
        let healthy_event = HealthEvent::new(
            "test_component".to_string(),
            HealthStatus::Healthy,
            "health_checker"
        );
        assert_eq!(healthy_event.severity(), EventSeverity::Info);

        let degraded_event = HealthEvent::new(
            "test_component".to_string(),
            HealthStatus::Degraded,
            "health_checker"
        );
        assert_eq!(degraded_event.severity(), EventSeverity::Warn);

        let unhealthy_event = HealthEvent::new(
            "test_component".to_string(),
            HealthStatus::Unhealthy,
            "health_checker"
        );
        assert_eq!(unhealthy_event.severity(), EventSeverity::Error);

        let critical_event = HealthEvent::new(
            "test_component".to_string(),
            HealthStatus::Critical,
            "health_checker"
        );
        assert_eq!(critical_event.severity(), EventSeverity::Critical);
    }

    /// Test MetricsEvent with tags
    #[test]
    fn test_metrics_event_with_tags() {
        let mut event = MetricsEvent::new(
            "response_time".to_string(),
            125.5,
            MetricType::Histogram,
            "metrics_collector"
        );

        event.add_tag("endpoint", "/api/v1/extract");
        event.add_tag("method", "POST");
        event.add_tag("status", "200");

        assert_eq!(event.metric_name, "response_time");
        assert_eq!(event.metric_value, 125.5);
        assert_eq!(event.tags.get("endpoint"), Some(&"/api/v1/extract".to_string()));
        assert_eq!(event.tags.get("method"), Some(&"POST".to_string()));
        assert_eq!(event.tags.get("status"), Some(&"200".to_string()));

        // Test with_tags method
        let mut tags = HashMap::new();
        tags.insert("service".to_string(), "riptide".to_string());
        tags.insert("version".to_string(), "1.0.0".to_string());

        let event_with_tags = MetricsEvent::new(
            "request_count".to_string(),
            1.0,
            MetricType::Counter,
            "api_handler"
        ).with_tags(tags);

        assert_eq!(event_with_tags.tags.get("service"), Some(&"riptide".to_string()));
        assert_eq!(event_with_tags.tags.get("version"), Some(&"1.0.0".to_string()));
    }

    /// Test SystemEvent with complex data
    #[test]
    fn test_system_event_with_complex_data() {
        let data = serde_json::json!({
            "component": "search_provider",
            "action": "provider_switched",
            "from": "serper",
            "to": "none",
            "reason": "api_rate_limit_exceeded",
            "config": {
                "timeout": 30,
                "retries": 3
            }
        });

        let event = SystemEvent::new(
            "search".to_string(),
            data,
            EventSeverity::Warn,
            "search_manager"
        );

        assert_eq!(event.category, "search");
        assert_eq!(event.severity(), EventSeverity::Warn);

        let json = event.to_json().unwrap();
        assert!(json.contains("provider_switched"));
        assert!(json.contains("api_rate_limit_exceeded"));
    }
}

#[cfg(test)]
mod event_emission_tests {
    use super::*;

    /// Test event emission through MockEventEmitter
    #[tokio::test]
    async fn test_event_emission() {
        let (emitter, mut receiver) = MockEventEmitter::new();

        let test_event = BaseEvent::new("test.event", "test_source", EventSeverity::Info);
        let event_id = test_event.event_id().to_string();

        // Emit the event
        let result = emitter.emit_event(test_event).await;
        assert!(result.is_ok());

        // Receive the event
        let received = receiver.recv().await;
        assert!(received.is_ok());

        let received_event = received.unwrap();
        assert_eq!(received_event.event_id(), event_id);
        assert_eq!(received_event.event_type(), "test.event");
        assert_eq!(received_event.source(), "test_source");
    }

    /// Test batch event emission
    #[tokio::test]
    async fn test_batch_event_emission() {
        let (emitter, mut receiver) = MockEventEmitter::new();

        let events = vec![
            BaseEvent::new("test.event1", "source1", EventSeverity::Info),
            BaseEvent::new("test.event2", "source2", EventSeverity::Warn),
            BaseEvent::new("test.event3", "source3", EventSeverity::Error),
        ];

        // Store event IDs for verification
        let event_ids: Vec<String> = events.iter().map(|e| e.event_id().to_string()).collect();

        // Emit events in batch
        let result = emitter.emit_events(events).await;
        assert!(result.is_ok());

        // Receive all events
        let mut received_ids = Vec::new();
        for _ in 0..3 {
            if let Ok(event) = receiver.recv().await {
                received_ids.push(event.event_id().to_string());
            }
        }

        assert_eq!(received_ids.len(), 3);
        for id in &event_ids {
            assert!(received_ids.contains(id));
        }
    }

    /// Test event emission with different severity levels
    #[tokio::test]
    async fn test_event_emission_different_severities() {
        let (emitter, mut receiver) = MockEventEmitter::new();

        let severities = vec![
            EventSeverity::Trace,
            EventSeverity::Debug,
            EventSeverity::Info,
            EventSeverity::Warn,
            EventSeverity::Error,
            EventSeverity::Critical,
        ];

        // Emit events with different severities
        for (i, severity) in severities.iter().enumerate() {
            let event = BaseEvent::new(
                &format!("test.event{}", i),
                "test_source",
                *severity
            );
            emitter.emit_event(event).await.unwrap();
        }

        // Verify all events are received with correct severities
        let mut received_severities = Vec::new();
        for _ in 0..severities.len() {
            if let Ok(event) = receiver.recv().await {
                received_severities.push(event.severity());
            }
        }

        assert_eq!(received_severities.len(), severities.len());
        for severity in &severities {
            assert!(received_severities.contains(severity));
        }
    }
}

#[cfg(test)]
mod event_handler_tests {
    use super::*;

    /// Test LoggingEventHandler
    #[tokio::test]
    async fn test_logging_event_handler() {
        let handler = LoggingEventHandler::new();

        assert_eq!(handler.name(), "logging_handler");
        assert!(handler.can_handle("test.event"));
        assert!(handler.can_handle("any.event.type"));

        // Test handling different severity events
        let events = vec![
            BaseEvent::new("test.debug", "source", EventSeverity::Debug),
            BaseEvent::new("test.info", "source", EventSeverity::Info),
            BaseEvent::new("test.warn", "source", EventSeverity::Warn),
            BaseEvent::new("test.error", "source", EventSeverity::Error),
            BaseEvent::new("test.critical", "source", EventSeverity::Critical),
        ];

        for event in &events {
            let result = handler.handle(event).await;
            assert!(result.is_ok());
        }

        // Test batch handling
        let event_refs: Vec<&dyn Event> = events.iter().map(|e| e as &dyn Event).collect();
        let result = handler.handle_batch(&event_refs).await;
        assert!(result.is_ok());
    }

    /// Test LoggingEventHandler with minimum severity filtering
    #[tokio::test]
    async fn test_logging_handler_severity_filtering() {
        let handler = LoggingEventHandler::new()
            .with_min_severity(EventSeverity::Warn);

        // These events should be handled (>= Warn)
        let warn_event = BaseEvent::new("test.warn", "source", EventSeverity::Warn);
        let error_event = BaseEvent::new("test.error", "source", EventSeverity::Error);

        assert!(handler.handle(&warn_event).await.is_ok());
        assert!(handler.handle(&error_event).await.is_ok());

        // These events should be filtered out (< Warn)
        let debug_event = BaseEvent::new("test.debug", "source", EventSeverity::Debug);
        let info_event = BaseEvent::new("test.info", "source", EventSeverity::Info);

        // They should still return Ok but be filtered
        assert!(handler.handle(&debug_event).await.is_ok());
        assert!(handler.handle(&info_event).await.is_ok());
    }

    /// Test HealthEventHandler component health tracking
    #[tokio::test]
    async fn test_health_event_handler_component_tracking() {
        let handler = HealthEventHandler::new();

        // Initially no components should be tracked
        assert!(handler.get_component_health("test_component").is_none());
        assert_eq!(handler.get_system_health(), HealthStatus::Healthy);

        // Handle a successful event
        let success_event = BaseEvent::new("test.success", "test_component", EventSeverity::Info);
        handler.handle(&success_event).await.unwrap();

        let component_health = handler.get_component_health("test_component");
        assert!(component_health.is_some());
        assert_eq!(component_health.unwrap().success_count, 1);

        // Handle an error event
        let error_event = BaseEvent::new("test.error", "test_component", EventSeverity::Error);
        handler.handle(&error_event).await.unwrap();

        let component_health = handler.get_component_health("test_component");
        assert!(component_health.is_some());
        let health = component_health.unwrap();
        assert_eq!(health.failure_count, 1);
        assert_eq!(health.success_count, 1);
        assert_eq!(health.status, HealthStatus::Degraded); // Should degrade on error
    }

    /// Test HealthEventHandler system health calculation
    #[tokio::test]
    async fn test_health_handler_system_health_calculation() {
        let handler = HealthEventHandler::new();

        // Add healthy component
        let healthy_event = BaseEvent::new("test.success", "component1", EventSeverity::Info);
        handler.handle(&healthy_event).await.unwrap();

        // System should be healthy
        assert_eq!(handler.get_system_health(), HealthStatus::Healthy);

        // Add critical component
        let critical_event = HealthEvent::new(
            "component2".to_string(),
            HealthStatus::Critical,
            "health_checker"
        );
        handler.handle(&critical_event).await.unwrap();

        // System should be critical due to one critical component
        assert_eq!(handler.get_system_health(), HealthStatus::Critical);
    }

    /// Test ComponentHealth scoring
    #[test]
    fn test_component_health_scoring() {
        let mut health = ComponentHealth::new();

        // Initially should have perfect score
        assert_eq!(health.health_score(), 1.0);
        assert!(health.is_healthy());

        // Add some failures and successes
        health.success_count = 8;
        health.failure_count = 2;

        // Score should be 80% (8 successes out of 10 total) * 1.0 (healthy status)
        assert!((health.health_score() - 0.8).abs() < 0.01);

        // Change to degraded status
        health.status = HealthStatus::Degraded;
        // Score should be 80% * 0.8 (degraded modifier) = 64%
        assert!((health.health_score() - 0.64).abs() < 0.01);

        // Test with all failures
        health.success_count = 0;
        health.failure_count = 10;
        assert_eq!(health.health_score(), 0.0);

        // Test with no data
        health.success_count = 0;
        health.failure_count = 0;
        assert_eq!(health.health_score(), 1.0); // Should default to healthy
    }

    /// Test TelemetryEventHandler
    #[tokio::test]
    async fn test_telemetry_event_handler() {
        let handler = TelemetryEventHandler::new();

        assert_eq!(handler.name(), "telemetry_handler");
        assert!(handler.can_handle("test.event"));

        // Test handling events with tracing
        let traceable_event = BaseEvent::new("test.trace", "source", EventSeverity::Info);
        let result = handler.handle(&traceable_event).await;
        assert!(result.is_ok());

        // Test handling critical events
        let critical_event = BaseEvent::new("test.critical", "source", EventSeverity::Critical);
        let result = handler.handle(&critical_event).await;
        assert!(result.is_ok());
    }

    /// Test HandlerConfig validation
    #[test]
    fn test_handler_config() {
        let default_config = HandlerConfig::default();

        assert!(default_config.enabled);
        assert_eq!(default_config.event_types, vec!["*"]);
        assert_eq!(default_config.min_severity, EventSeverity::Info);
        assert_eq!(default_config.batch_size, 100);
        assert_eq!(default_config.retry_attempts, 3);

        // Test custom config
        let custom_config = HandlerConfig {
            enabled: true,
            event_types: vec!["pool.*".to_string(), "extraction.*".to_string()],
            min_severity: EventSeverity::Warn,
            batch_size: 50,
            timeout: Duration::from_secs(10),
            retry_attempts: 5,
        };

        assert_eq!(custom_config.event_types.len(), 2);
        assert_eq!(custom_config.min_severity, EventSeverity::Warn);
        assert_eq!(custom_config.batch_size, 50);
        assert_eq!(custom_config.retry_attempts, 5);
    }
}

#[cfg(test)]
mod event_bus_integration_tests {
    use super::*;
    use riptide_core::events::bus::{EventBus, EventBusConfig, EventRouting};
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    /// Test EventBus with comprehensive event processing
    #[tokio::test]
    async fn test_event_bus_comprehensive_processing() {
        let mut event_bus = EventBus::new();
        let handler = Arc::new(LoggingEventHandler::new());

        event_bus.register_handler(handler.clone()).await.unwrap();
        event_bus.start().await.unwrap();

        // Test various event types
        let pool_event = PoolEvent::new(
            PoolOperation::InstanceCreated,
            "test-pool".to_string(),
            "pool_manager"
        ).with_instance_id("instance-1".to_string());

        let extraction_event = ExtractionEvent::new(
            ExtractionOperation::Completed,
            "https://example.com".to_string(),
            "article".to_string(),
            "extractor"
        ).with_duration(Duration::from_millis(250));

        let health_event = HealthEvent::new(
            "test_component".to_string(),
            HealthStatus::Healthy,
            "health_checker"
        );

        let metrics_event = MetricsEvent::new(
            "response_time".to_string(),
            125.5,
            MetricType::Histogram,
            "metrics_collector"
        );

        // Emit all events
        assert!(event_bus.emit_event(pool_event).await.is_ok());
        assert!(event_bus.emit_event(extraction_event).await.is_ok());
        assert!(event_bus.emit_event(health_event).await.is_ok());
        assert!(event_bus.emit_event(metrics_event).await.is_ok());

        // Give time for processing
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Verify event bus statistics
        let stats = event_bus.get_stats();
        assert!(stats.is_running);
        assert_eq!(stats.buffer_size, 1000);
    }

    /// Test EventBus with custom configuration
    #[tokio::test]
    async fn test_event_bus_custom_configuration() {
        let config = EventBusConfig {
            buffer_size: 500,
            async_handlers: false,
            handler_timeout: Duration::from_secs(2),
            continue_on_handler_error: true,
        };

        let mut event_bus = EventBus::with_config(config);
        let handler = Arc::new(LoggingEventHandler::new());

        event_bus.register_handler(handler).await.unwrap();
        event_bus.start().await.unwrap();

        let stats = event_bus.get_stats();
        assert_eq!(stats.buffer_size, 500);
        assert!(stats.is_running);

        // Test event processing with sequential handlers
        let test_event = BaseEvent::new("test.sequential", "source", EventSeverity::Info);
        assert!(event_bus.emit_event(test_event).await.is_ok());

        tokio::time::sleep(Duration::from_millis(50)).await;

        event_bus.stop().await;
        let stats = event_bus.get_stats();
        assert!(!stats.is_running);
    }

    /// Test concurrent event processing
    #[tokio::test]
    async fn test_concurrent_event_processing() {
        let mut event_bus = EventBus::new();
        let handler = Arc::new(LoggingEventHandler::new());
        let health_handler = Arc::new(HealthEventHandler::new());

        event_bus.register_handler(handler).await.unwrap();
        event_bus.register_handler(health_handler).await.unwrap();
        event_bus.start().await.unwrap();

        // Launch concurrent event emissions
        let event_bus = Arc::new(event_bus);
        let mut handles = Vec::new();

        for i in 0..10 {
            let bus_clone = event_bus.clone();
            let handle = tokio::spawn(async move {
                let event = BaseEvent::new(
                    &format!("concurrent.test.{}", i),
                    "concurrent_source",
                    EventSeverity::Info
                );
                bus_clone.emit_event(event).await
            });
            handles.push(handle);
        }

        // Wait for all events to be emitted
        for handle in handles {
            assert!(handle.await.unwrap().is_ok());
        }

        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    /// Test event routing configurations
    #[tokio::test]
    async fn test_event_routing_configurations() {
        // Test pattern-based routing
        let mut pattern_routing = std::collections::HashMap::new();
        pattern_routing.insert("pool.*".to_string(), vec!["logging_handler".to_string()]);
        pattern_routing.insert("health.*".to_string(), vec!["health_handler".to_string()]);

        let config = EventBusConfig::default();
        let mut event_bus = EventBus::with_config(config);
        event_bus.set_routing(EventRouting::PatternBased(pattern_routing));

        let logging_handler = Arc::new(LoggingEventHandler::new());
        let health_handler = Arc::new(HealthEventHandler::new());

        event_bus.register_handler(logging_handler).await.unwrap();
        event_bus.register_handler(health_handler).await.unwrap();
        event_bus.start().await.unwrap();

        // Test routing
        let pool_event = PoolEvent::new(
            PoolOperation::InstanceAcquired,
            "test-pool".to_string(),
            "pool_manager"
        );

        let health_event = HealthEvent::new(
            "test_component".to_string(),
            HealthStatus::Degraded,
            "health_checker"
        );

        assert!(event_bus.emit_event(pool_event).await.is_ok());
        assert!(event_bus.emit_event(health_event).await.is_ok());

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    /// Test severity-based routing
    #[tokio::test]
    async fn test_severity_based_routing() {
        let mut severity_routing = std::collections::HashMap::new();
        severity_routing.insert(EventSeverity::Error, vec!["logging_handler".to_string()]);
        severity_routing.insert(EventSeverity::Critical, vec!["logging_handler".to_string(), "health_handler".to_string()]);

        let mut event_bus = EventBus::new();
        event_bus.set_routing(EventRouting::SeverityBased(severity_routing));

        let logging_handler = Arc::new(LoggingEventHandler::new());
        let health_handler = Arc::new(HealthEventHandler::new());

        event_bus.register_handler(logging_handler).await.unwrap();
        event_bus.register_handler(health_handler).await.unwrap();
        event_bus.start().await.unwrap();

        // Test error event
        let error_event = BaseEvent::new("test.error", "source", EventSeverity::Error);
        assert!(event_bus.emit_event(error_event).await.is_ok());

        // Test critical event
        let critical_event = BaseEvent::new("test.critical", "source", EventSeverity::Critical);
        assert!(event_bus.emit_event(critical_event).await.is_ok());

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

#[cfg(test)]
mod pool_event_integration_tests {
    use super::*;
    use riptide_core::events::pool_integration::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// Test pool event emission helpers
    #[tokio::test]
    async fn test_pool_event_emission_helpers() {
        let mut event_bus = EventBus::new();
        let handler = Arc::new(LoggingEventHandler::new());

        event_bus.register_handler(handler).await.unwrap();
        event_bus.start().await.unwrap();

        let helper = PoolEventEmissionHelper::new(
            Arc::new(event_bus),
            "test-pool".to_string()
        );

        // Test all pool operation events
        assert!(helper.emit_instance_created("instance-1").await.is_ok());
        assert!(helper.emit_instance_acquired("instance-1").await.is_ok());
        assert!(helper.emit_instance_released("instance-1").await.is_ok());
        assert!(helper.emit_instance_destroyed("instance-1").await.is_ok());

        assert!(helper.emit_instance_unhealthy("instance-2", "timeout").await.is_ok());
        assert!(helper.emit_pool_exhausted(5).await.is_ok());
        assert!(helper.emit_circuit_breaker_tripped(10).await.is_ok());
        assert!(helper.emit_circuit_breaker_reset().await.is_ok());
        assert!(helper.emit_pool_warmup(3).await.is_ok());

        // Test pool metrics event
        let metrics = PoolMetrics {
            available_instances: 5,
            active_instances: 3,
            total_instances: 8,
            pending_acquisitions: 2,
            success_rate: 0.95,
            avg_acquisition_time_ms: 150,
        };

        assert!(helper.emit_pool_metrics(metrics).await.is_ok());

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    /// Test pool event configuration
    #[test]
    fn test_pool_event_configuration() {
        let default_config = PoolEventConfig::default();

        assert!(default_config.emit_instance_lifecycle);
        assert!(default_config.emit_health_events);
        assert!(default_config.emit_metrics_events);
        assert!(default_config.emit_circuit_breaker_events);
        assert_eq!(default_config.health_check_interval, Duration::from_secs(30));
        assert_eq!(default_config.metrics_emission_interval, Duration::from_secs(60));

        let custom_config = PoolEventConfig {
            emit_instance_lifecycle: false,
            emit_health_events: true,
            emit_metrics_events: false,
            emit_circuit_breaker_events: true,
            health_check_interval: Duration::from_secs(10),
            metrics_emission_interval: Duration::from_secs(30),
        };

        assert!(!custom_config.emit_instance_lifecycle);
        assert!(custom_config.emit_health_events);
        assert!(!custom_config.emit_metrics_events);
        assert!(custom_config.emit_circuit_breaker_events);
    }
}

#[cfg(test)]
mod advanced_event_handler_tests {
    use super::*;
    use riptide_core::monitoring::MetricsCollector;
    use riptide_core::events::handlers::*;

    /// Test MetricsEventHandler with real metrics
    #[tokio::test]
    async fn test_metrics_event_handler_functionality() {
        let metrics_collector = Arc::new(MetricsCollector::new());
        let handler = MetricsEventHandler::new(metrics_collector.clone());

        // Test handling pool events
        let pool_metrics = PoolMetrics {
            available_instances: 5,
            active_instances: 3,
            total_instances: 8,
            pending_acquisitions: 1,
            success_rate: 0.92,
            avg_acquisition_time_ms: 120,
        };

        let pool_event = PoolEvent::new(
            PoolOperation::InstanceAcquired,
            "metrics-test-pool".to_string(),
            "pool_manager"
        ).with_metrics(pool_metrics);

        assert!(handler.can_handle(pool_event.event_type()));
        assert!(handler.handle(&pool_event).await.is_ok());

        // Test handling extraction events
        let extraction_event = ExtractionEvent::new(
            ExtractionOperation::Completed,
            "https://test-metrics.com".to_string(),
            "article".to_string(),
            "extractor"
        ).with_duration(Duration::from_millis(200))
         .with_content_length(5000);

        assert!(handler.can_handle(extraction_event.event_type()));
        assert!(handler.handle(&extraction_event).await.is_ok());

        // Test handling metrics events
        let mut tags = HashMap::new();
        tags.insert("service".to_string(), "riptide".to_string());

        let metrics_event = MetricsEvent::new(
            "request_duration".to_string(),
            250.0,
            MetricType::Histogram,
            "api_handler"
        ).with_tags(tags);

        assert!(handler.can_handle(metrics_event.event_type()));
        assert!(handler.handle(&metrics_event).await.is_ok());

        // Test handling health events
        let mut health_metrics = HashMap::new();
        health_metrics.insert("cpu_usage".to_string(), 75.5);
        health_metrics.insert("memory_usage".to_string(), 65.2);

        let health_event = HealthEvent::new(
            "api_service".to_string(),
            HealthStatus::Healthy,
            "health_monitor"
        ).with_metrics(health_metrics);

        assert!(handler.can_handle(health_event.event_type()));
        assert!(handler.handle(&health_event).await.is_ok());
    }

    /// Test TelemetryEventHandler with OpenTelemetry integration
    #[tokio::test]
    async fn test_telemetry_event_handler_functionality() {
        let handler = TelemetryEventHandler::new();

        assert_eq!(handler.name(), "telemetry_handler");
        assert!(handler.can_handle("test.telemetry"));

        // Test with traceable events
        let traceable_event = BaseEvent::new("telemetry.test", "source", EventSeverity::Info);
        assert!(handler.handle(&traceable_event).await.is_ok());

        // Test with critical events (should create error spans)
        let critical_event = BaseEvent::new("telemetry.critical", "source", EventSeverity::Critical);
        assert!(handler.handle(&critical_event).await.is_ok());

        // Test with error events
        let error_event = BaseEvent::new("telemetry.error", "source", EventSeverity::Error);
        assert!(handler.handle(&error_event).await.is_ok());
    }

    /// Test HealthEventHandler advanced functionality
    #[tokio::test]
    async fn test_health_handler_advanced_functionality() {
        let handler = HealthEventHandler::new();

        // Test multiple component health tracking
        let components = ["database", "cache", "api_gateway", "message_queue"];

        for (i, component) in components.iter().enumerate() {
            let status = match i {
                0 => HealthStatus::Healthy,
                1 => HealthStatus::Degraded,
                2 => HealthStatus::Unhealthy,
                3 => HealthStatus::Critical,
                _ => HealthStatus::Healthy,
            };

            let health_event = HealthEvent::new(
                component.to_string(),
                status,
                "health_monitor"
            );

            handler.handle(&health_event).await.unwrap();
        }

        // Check individual component health
        for component in &components {
            let health = handler.get_component_health(component);
            assert!(health.is_some());
        }

        // Check system health (should be critical due to one critical component)
        let system_health = handler.get_system_health();
        assert_eq!(system_health, HealthStatus::Critical);

        // Test recent events
        let recent_events = handler.get_recent_events(10);
        assert_eq!(recent_events.len(), 4);

        // Test component health improvement
        for _ in 0..5 {
            let success_event = BaseEvent::new("success.test", "message_queue", EventSeverity::Info);
            handler.handle(&success_event).await.unwrap();
        }

        let queue_health = handler.get_component_health("message_queue");
        assert!(queue_health.is_some());
        let health = queue_health.unwrap();
        assert_eq!(health.success_count, 5);
    }
}

#[cfg(test)]
mod event_system_performance_tests {
    use super::*;
    use std::time::Instant;

    /// Test event system performance under load
    #[tokio::test]
    async fn test_event_system_under_load() {
        let mut event_bus = EventBus::new();
        let handler = Arc::new(LoggingEventHandler::new());

        event_bus.register_handler(handler).await.unwrap();
        event_bus.start().await.unwrap();

        let event_bus = Arc::new(event_bus);
        let start_time = Instant::now();

        // Generate high load with concurrent event emissions
        let mut handles = Vec::new();
        for i in 0..100 {
            let bus_clone = event_bus.clone();
            let handle = tokio::spawn(async move {
                for j in 0..10 {
                    let event = BaseEvent::new(
                        &format!("load.test.{}.{}", i, j),
                        "load_generator",
                        EventSeverity::Info
                    );
                    let _ = bus_clone.emit_event(event).await;
                }
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            let _ = handle.await;
        }

        let duration = start_time.elapsed();
        tokio::time::sleep(Duration::from_millis(100)).await; // Give time for processing

        // Performance should be reasonable (less than 5 seconds for 1000 events)
        assert!(duration < Duration::from_secs(5));

        println!("Processed 1000 events in {:?}", duration);
    }

    /// Test memory usage with large number of events
    #[tokio::test]
    async fn test_event_system_memory_usage() {
        let config = EventBusConfig {
            buffer_size: 10000, // Large buffer
            async_handlers: true,
            handler_timeout: Duration::from_secs(1),
            continue_on_handler_error: true,
        };

        let mut event_bus = EventBus::with_config(config);
        let handler = Arc::new(LoggingEventHandler::new());

        event_bus.register_handler(handler).await.unwrap();
        event_bus.start().await.unwrap();

        // Generate many events with various data sizes
        for i in 0..1000 {
            let mut event = BaseEvent::new(
                &format!("memory.test.{}", i),
                "memory_tester",
                EventSeverity::Info
            );

            // Add varying amounts of metadata
            for j in 0..(i % 10) {
                event.add_metadata(&format!("key_{}", j), &format!("value_{}", j));
            }

            let _ = event_bus.emit_event(event).await;
        }

        tokio::time::sleep(Duration::from_millis(200)).await;

        // Test should complete without excessive memory usage or panics
        let stats = event_bus.get_stats();
        assert!(stats.is_running);
    }
}

#[cfg(test)]
mod event_subscription_tests {
    use super::*;

    /// Test EventSubscription filtering
    #[tokio::test]
    async fn test_event_subscription_filtering() {
        let (sender, receiver) = broadcast::channel(100);

        // Create subscription that only accepts Warn+ events of type "test.*"
        let mut subscription = EventSubscription::new(
            receiver,
            vec!["test.*".to_string()],
            EventSeverity::Warn
        );

        // Send various events
        let events = vec![
            Arc::new(BaseEvent::new("test.warn", "source", EventSeverity::Warn)) as Arc<dyn Event>,
            Arc::new(BaseEvent::new("test.error", "source", EventSeverity::Error)) as Arc<dyn Event>,
            Arc::new(BaseEvent::new("test.info", "source", EventSeverity::Info)) as Arc<dyn Event>, // Should be filtered
            Arc::new(BaseEvent::new("other.warn", "source", EventSeverity::Warn)) as Arc<dyn Event>, // Should be filtered
        ];

        for event in &events {
            let _ = sender.send(event.clone());
        }

        // Should receive only the first two events (warn and error from test.*)
        let mut received_count = 0;
        while let Ok(event) = tokio::time::timeout(Duration::from_millis(100), subscription.recv()).await {
            if event.is_ok() {
                received_count += 1;
                let event = event.unwrap();
                assert!(event.event_type().starts_with("test."));
                assert!(event.severity() >= EventSeverity::Warn);
            }
        }

        assert_eq!(received_count, 2);
    }

    /// Test EventSubscription with wildcard event types
    #[tokio::test]
    async fn test_event_subscription_wildcard() {
        let (sender, receiver) = broadcast::channel(100);

        // Create subscription that accepts all event types with Info+ severity
        let mut subscription = EventSubscription::new(
            receiver,
            vec!["*".to_string()],
            EventSeverity::Info
        );

        let events = vec![
            Arc::new(BaseEvent::new("test.info", "source", EventSeverity::Info)) as Arc<dyn Event>,
            Arc::new(BaseEvent::new("other.warn", "source", EventSeverity::Warn)) as Arc<dyn Event>,
            Arc::new(BaseEvent::new("system.debug", "source", EventSeverity::Debug)) as Arc<dyn Event>, // Should be filtered
        ];

        for event in &events {
            let _ = sender.send(event.clone());
        }

        // Should receive the first two events (info and warn, debug filtered out)
        let mut received_count = 0;
        while let Ok(event) = tokio::time::timeout(Duration::from_millis(100), subscription.recv()).await {
            if event.is_ok() {
                received_count += 1;
                let event = event.unwrap();
                assert!(event.severity() >= EventSeverity::Info);
            }
        }

        assert_eq!(received_count, 2);
    }

    /// Test EventSubscription with empty event types (should accept nothing)
    #[tokio::test]
    async fn test_event_subscription_empty_types() {
        let (sender, receiver) = broadcast::channel(100);

        // Create subscription with no event types
        let mut subscription = EventSubscription::new(
            receiver,
            vec![], // Empty event types
            EventSeverity::Debug
        );

        let event = Arc::new(BaseEvent::new("test.info", "source", EventSeverity::Info)) as Arc<dyn Event>;
        let _ = sender.send(event);

        // Should timeout since no events should be accepted
        let result = tokio::time::timeout(Duration::from_millis(100), subscription.recv()).await;
        assert!(result.is_err()); // Should timeout
    }

    /// Test EventSubscription with multiple event types
    #[tokio::test]
    async fn test_event_subscription_multiple_types() {
        let (sender, receiver) = broadcast::channel(100);

        // Create subscription for multiple patterns
        let mut subscription = EventSubscription::new(
            receiver,
            vec!["pool.*".to_string(), "health.*".to_string()],
            EventSeverity::Info
        );

        // Send various events
        let pool_event = Arc::new(PoolEvent::new(
            PoolOperation::InstanceCreated,
            "test-pool".to_string(),
            "pool_manager"
        )) as Arc<dyn Event>;

        let health_event = Arc::new(HealthEvent::new(
            "test_component".to_string(),
            HealthStatus::Healthy,
            "health_checker"
        )) as Arc<dyn Event>;

        let system_event = Arc::new(BaseEvent::new(
            "system.startup",
            "system",
            EventSeverity::Info
        )) as Arc<dyn Event>;

        let _ = sender.send(pool_event);
        let _ = sender.send(health_event);
        let _ = sender.send(system_event); // Should be filtered out

        // Should receive only pool and health events
        let mut received_count = 0;
        while let Ok(event) = tokio::time::timeout(Duration::from_millis(100), subscription.recv()).await {
            if event.is_ok() {
                received_count += 1;
                let event = event.unwrap();
                assert!(event.event_type().starts_with("pool.") || event.event_type().starts_with("health."));
            }
        }

        assert_eq!(received_count, 2);
    }
}

#[cfg(test)]
mod search_provider_event_tests {
    use super::*;
    // Import search provider functionality for integration testing
    // Note: These tests activate the SearchProvider event integration

    /// Mock search provider that emits events
    struct MockEventEmittingSearchProvider {
        event_bus: Arc<EventBus>,
    }

    impl MockEventEmittingSearchProvider {
        fn new(event_bus: Arc<EventBus>) -> Self {
            Self { event_bus }
        }

        async fn perform_search(&self, query: &str) -> Result<Vec<String>> {
            // Emit search started event
            let started_event = SystemEvent::new(
                "search_started".to_string(),
                serde_json::json!({
                    "query": query,
                    "provider": "mock"
                }),
                EventSeverity::Info,
                "search_provider"
            );

            self.event_bus.emit_event(started_event).await?;

            // Simulate search processing
            tokio::time::sleep(Duration::from_millis(10)).await;

            // Simulate results
            let results = vec!["result1".to_string(), "result2".to_string()];

            // Emit search completed event
            let completed_event = SystemEvent::new(
                "search_completed".to_string(),
                serde_json::json!({
                    "query": query,
                    "result_count": results.len(),
                    "provider": "mock"
                }),
                EventSeverity::Info,
                "search_provider"
            );

            self.event_bus.emit_event(completed_event).await?;

            Ok(results)
        }

        async fn perform_failing_search(&self, query: &str) -> Result<Vec<String>> {
            // Emit search started event
            let started_event = SystemEvent::new(
                "search_started".to_string(),
                serde_json::json!({
                    "query": query,
                    "provider": "mock"
                }),
                EventSeverity::Info,
                "search_provider"
            );

            self.event_bus.emit_event(started_event).await?;

            // Emit search failed event
            let failed_event = SystemEvent::new(
                "search_failed".to_string(),
                serde_json::json!({
                    "query": query,
                    "error": "simulated failure",
                    "provider": "mock"
                }),
                EventSeverity::Error,
                "search_provider"
            );

            self.event_bus.emit_event(failed_event).await?;

            Err(anyhow::anyhow!("Simulated search failure"))
        }
    }

    /// Test search provider event integration
    #[tokio::test]
    async fn test_search_provider_event_integration() {
        let mut event_bus = EventBus::new();
        let handler = Arc::new(LoggingEventHandler::new());

        event_bus.register_handler(handler.clone()).await.unwrap();
        event_bus.start().await.unwrap();

        let search_provider = MockEventEmittingSearchProvider::new(Arc::new(event_bus));

        // Test successful search
        let results = search_provider.perform_search("test query").await;
        assert!(results.is_ok());
        assert_eq!(results.unwrap().len(), 2);

        tokio::time::sleep(Duration::from_millis(50)).await;

        // Test failing search
        let results = search_provider.perform_failing_search("failing query").await;
        assert!(results.is_err());

        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    /// Test concurrent search operations with events
    #[tokio::test]
    async fn test_concurrent_search_with_events() {
        let mut event_bus = EventBus::new();
        let handler = Arc::new(LoggingEventHandler::new());

        event_bus.register_handler(handler.clone()).await.unwrap();
        event_bus.start().await.unwrap();

        let search_provider = Arc::new(MockEventEmittingSearchProvider::new(Arc::new(event_bus)));

        // Launch concurrent searches
        let mut handles = Vec::new();
        for i in 0..5 {
            let provider_clone = search_provider.clone();
            let handle = tokio::spawn(async move {
                provider_clone.perform_search(&format!("query {}", i)).await
            });
            handles.push(handle);
        }

        // Wait for all searches to complete
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    /// Test search provider metrics events
    #[tokio::test]
    async fn test_search_provider_metrics_events() {
        let mut event_bus = EventBus::new();
        let handler = Arc::new(LoggingEventHandler::new());

        event_bus.register_handler(handler.clone()).await.unwrap();
        event_bus.start().await.unwrap();

        // Emit search-related metrics
        let search_latency = MetricsEvent::new(
            "search_latency_ms".to_string(),
            150.0,
            MetricType::Histogram,
            "search_provider"
        );

        let search_success_rate = MetricsEvent::new(
            "search_success_rate".to_string(),
            0.95,
            MetricType::Gauge,
            "search_provider"
        );

        let search_requests = MetricsEvent::new(
            "search_requests_total".to_string(),
            1.0,
            MetricType::Counter,
            "search_provider"
        );

        assert!(event_bus.emit_event(search_latency).await.is_ok());
        assert!(event_bus.emit_event(search_success_rate).await.is_ok());
        assert!(event_bus.emit_event(search_requests).await.is_ok());

        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}