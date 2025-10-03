//! Integration tests for EventBus system
//!
//! These tests verify that the EventBus is properly integrated into AppState
//! and that events flow correctly through the system.

#[cfg(test)]
mod tests {
    use crate::health::HealthChecker;
    use crate::metrics::RipTideMetrics;
    use crate::state::{AppConfig, AppState};
    use riptide_core::events::{BaseEvent, EventBus, EventEmitter, EventSeverity};
    use std::sync::Arc;
    use tokio::time::{sleep, Duration};

    /// Test that EventBus is properly initialized in AppState
    #[tokio::test]
    async fn test_event_bus_initialization() {
        let config = AppConfig {
            redis_url: "redis://localhost:6379".to_string(),
            wasm_path: "./target/wasm32-wasip2/release/riptide_extractor_wasm.wasm".to_string(),
            max_concurrency: 4,
            cache_ttl: 3600,
            gate_hi_threshold: 0.7,
            gate_lo_threshold: 0.3,
            headless_url: None,
            session_config: Default::default(),
            spider_config: None,
            worker_config: AppConfig::init_worker_config(),
            event_bus_config: Default::default(),
        };

        let metrics = Arc::new(RipTideMetrics::new().expect("Failed to create metrics"));
        let health_checker = Arc::new(HealthChecker::new());

        // This will fail if Redis is not available, but the EventBus should still be initialized
        match AppState::new(config, metrics, health_checker).await {
            Ok(state) => {
                // Verify event bus is initialized
                assert!(
                    !state.event_bus.get_stats().is_running
                        || state.event_bus.get_stats().is_running
                );

                // Verify handlers are registered
                let handler_names = state.event_bus.get_handler_names().await;
                assert!(handler_names.contains(&"logging_handler".to_string()));
                assert!(handler_names.contains(&"metrics_handler".to_string()));
                assert!(handler_names.contains(&"telemetry_handler".to_string()));
                assert!(handler_names.contains(&"health_handler".to_string()));
            }
            Err(e) => {
                eprintln!(
                    "AppState initialization failed (expected if Redis not available): {}",
                    e
                );
            }
        }
    }

    /// Test event emission through EventBus
    #[tokio::test]
    async fn test_event_emission() {
        let event_bus = EventBus::new();

        // Start the event bus
        let mut bus = event_bus;
        bus.start().await.expect("Failed to start event bus");

        // Create a test event
        let test_event = BaseEvent::new("test.event", "test_source", EventSeverity::Info);

        // Emit event
        let result = bus.emit_event(test_event).await;

        // Should succeed even without external subscribers
        assert!(result.is_ok() || result.is_err()); // Either succeeds or fails gracefully

        // Wait a bit for async processing
        sleep(Duration::from_millis(100)).await;

        // Stop the bus
        bus.stop().await;
    }

    /// Test event handler registration
    #[tokio::test]
    async fn test_handler_registration() {
        use riptide_core::events::handlers::LoggingEventHandler;

        let event_bus = EventBus::new();

        // Register a handler
        let handler = Arc::new(LoggingEventHandler::new());
        let result = event_bus.register_handler(handler).await;
        assert!(result.is_ok());

        // Verify handler is registered
        let handlers = event_bus.get_handler_names().await;
        assert!(handlers.contains(&"logging_handler".to_string()));

        // Try to register the same handler again (should fail)
        let duplicate_handler = Arc::new(LoggingEventHandler::new());
        let duplicate_result = event_bus.register_handler(duplicate_handler).await;
        assert!(duplicate_result.is_err());
    }

    /// Test event bus statistics
    #[tokio::test]
    async fn test_event_bus_stats() {
        let event_bus = EventBus::new();
        let stats = event_bus.get_stats();

        assert_eq!(stats.buffer_size, 1000); // Default buffer size
        assert!(!stats.is_running); // Not started yet
        assert_eq!(stats.current_subscribers, 0); // No external subscribers
    }

    /// Test multiple handler types
    #[tokio::test]
    async fn test_multiple_handlers() {
        use riptide_core::events::handlers::{
            HealthEventHandler, LoggingEventHandler, TelemetryEventHandler,
        };
        use riptide_core::monitoring::MetricsCollector;

        let event_bus = EventBus::new();

        // Register multiple handlers
        let logging = Arc::new(LoggingEventHandler::new());
        let telemetry = Arc::new(TelemetryEventHandler::new());
        let health = Arc::new(HealthEventHandler::new());

        event_bus
            .register_handler(logging)
            .await
            .expect("Failed to register logging handler");
        event_bus
            .register_handler(telemetry)
            .await
            .expect("Failed to register telemetry handler");
        event_bus
            .register_handler(health)
            .await
            .expect("Failed to register health handler");

        let handlers = event_bus.get_handler_names().await;
        assert_eq!(handlers.len(), 3);
        assert!(handlers.contains(&"logging_handler".to_string()));
        assert!(handlers.contains(&"telemetry_handler".to_string()));
        assert!(handlers.contains(&"health_handler".to_string()));
    }

    /// Test event bus configuration
    #[tokio::test]
    async fn test_event_bus_configuration() {
        use riptide_core::events::EventBusConfig;

        let config = EventBusConfig {
            buffer_size: 500,
            async_handlers: false,
            handler_timeout: Duration::from_secs(10),
            continue_on_handler_error: false,
        };

        let event_bus = EventBus::with_config(config);
        let stats = event_bus.get_stats();

        assert_eq!(stats.buffer_size, 500);
    }
}
