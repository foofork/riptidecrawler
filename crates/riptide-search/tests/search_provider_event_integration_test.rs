//! Integration tests for SearchProvider and Event System
//!
//! This module tests the integration between search providers and the event system,
//! ensuring that search operations emit appropriate events and that handlers receive them.

use anyhow::Result;
use async_trait::async_trait;
use riptide_core::events::{
    handlers::{HealthEventHandler, LoggingEventHandler},
    types::{MetricType, MetricsEvent, SystemEvent},
    Event, EventBus, EventBusConfig, EventEmitter, EventHandler, EventSeverity, HandlerConfig,
};
use riptide_search::{
    CircuitBreakerWrapper, NoneProvider, SearchBackend, SearchHit, SearchProvider,
};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

// Mock event handler for testing integration
struct SearchProviderEventHandler {
    name: String,
    events_received: Arc<AtomicU32>,
    last_event: Arc<Mutex<Option<String>>>,
}

impl SearchProviderEventHandler {
    fn new() -> Self {
        Self {
            name: "search_provider_handler".to_string(),
            events_received: Arc::new(AtomicU32::new(0)),
            last_event: Arc::new(Mutex::new(None)),
        }
    }

    fn get_events_received(&self) -> u32 {
        self.events_received.load(Ordering::Relaxed)
    }

    async fn get_last_event(&self) -> Option<String> {
        self.last_event.lock().await.clone()
    }
}

#[async_trait]
impl EventHandler for SearchProviderEventHandler {
    fn name(&self) -> &str {
        &self.name
    }

    fn can_handle(&self, event_type: &str) -> bool {
        event_type.starts_with("search.") || event_type.starts_with("system.")
    }

    async fn handle(&self, event: &dyn Event) -> Result<()> {
        self.events_received.fetch_add(1, Ordering::Relaxed);

        if let Ok(json) = event.to_json() {
            *self.last_event.lock().await = Some(json);
        }

        Ok(())
    }

    fn config(&self) -> HandlerConfig {
        HandlerConfig {
            enabled: true,
            event_types: vec!["search.*".to_string(), "system.*".to_string()],
            min_severity: EventSeverity::Debug,
            ..Default::default()
        }
    }
}

// Mock SearchProvider that emits events
struct EventEmittingSearchProvider {
    inner_provider: Box<dyn SearchProvider>,
    event_emitter: Arc<EventBus>,
}

impl std::fmt::Debug for EventEmittingSearchProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventEmittingSearchProvider")
            .field("inner_provider", &"<SearchProvider>")
            .field("event_emitter", &"<EventBus>")
            .finish()
    }
}

impl EventEmittingSearchProvider {
    fn new(provider: Box<dyn SearchProvider>, event_emitter: Arc<EventBus>) -> Self {
        Self {
            inner_provider: provider,
            event_emitter,
        }
    }

    async fn emit_search_started_event(&self, query: &str) -> Result<()> {
        let event_data = serde_json::json!({
            "query": query,
            "provider_type": self.inner_provider.backend_type().to_string()
        });

        let event = SystemEvent::new(
            "search_started".to_string(),
            event_data,
            EventSeverity::Info,
            "search_provider",
        );

        self.event_emitter.emit_event(event).await
    }

    async fn emit_search_completed_event(&self, query: &str, result_count: usize) -> Result<()> {
        let event_data = serde_json::json!({
            "query": query,
            "result_count": result_count,
            "provider_type": self.inner_provider.backend_type().to_string()
        });

        let event = SystemEvent::new(
            "search_completed".to_string(),
            event_data,
            EventSeverity::Info,
            "search_provider",
        );

        self.event_emitter.emit_event(event).await
    }

    async fn emit_search_failed_event(&self, query: &str, error: &str) -> Result<()> {
        let event_data = serde_json::json!({
            "query": query,
            "error": error,
            "provider_type": self.inner_provider.backend_type().to_string()
        });

        let event = SystemEvent::new(
            "search_failed".to_string(),
            event_data,
            EventSeverity::Error,
            "search_provider",
        );

        self.event_emitter.emit_event(event).await
    }

    async fn emit_metrics_event(&self, query: &str, duration_ms: u64) -> Result<()> {
        let mut tags = HashMap::new();
        tags.insert(
            "provider".to_string(),
            self.inner_provider.backend_type().to_string(),
        );
        tags.insert("query_length".to_string(), query.len().to_string());

        let event = MetricsEvent::new(
            "search_duration_ms".to_string(),
            duration_ms as f64,
            MetricType::Histogram,
            "search_provider",
        )
        .with_tags(tags);

        self.event_emitter.emit_event(event).await
    }
}

#[async_trait]
impl SearchProvider for EventEmittingSearchProvider {
    async fn search(
        &self,
        query: &str,
        limit: u32,
        country: &str,
        locale: &str,
    ) -> Result<Vec<SearchHit>> {
        let start_time = std::time::Instant::now();

        // Emit search started event
        if let Err(e) = self.emit_search_started_event(query).await {
            eprintln!("Failed to emit search started event: {}", e);
        }

        // Perform actual search
        let result = self
            .inner_provider
            .search(query, limit, country, locale)
            .await;

        let duration_ms = start_time.elapsed().as_millis() as u64;

        // Emit metrics event
        if let Err(e) = self.emit_metrics_event(query, duration_ms).await {
            eprintln!("Failed to emit metrics event: {}", e);
        }

        // Emit completion or failure event based on result
        match &result {
            Ok(hits) => {
                if let Err(e) = self.emit_search_completed_event(query, hits.len()).await {
                    eprintln!("Failed to emit search completed event: {}", e);
                }
            }
            Err(error) => {
                if let Err(e) = self
                    .emit_search_failed_event(query, &error.to_string())
                    .await
                {
                    eprintln!("Failed to emit search failed event: {}", e);
                }
            }
        }

        result
    }

    fn backend_type(&self) -> SearchBackend {
        self.inner_provider.backend_type()
    }

    async fn health_check(&self) -> Result<()> {
        self.inner_provider.health_check().await
    }
}

// Activate all SearchProvider integration tests
#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Test basic integration between SearchProvider and EventBus
    #[tokio::test]
    async fn test_search_provider_event_integration() {
        // Create event bus
        let mut event_bus = EventBus::new();
        let handler = Arc::new(SearchProviderEventHandler::new());

        // Register handler
        event_bus.register_handler(handler.clone()).await.unwrap();

        // Start event bus
        event_bus.start().await.unwrap();

        // Create search provider with event integration
        let none_provider = Box::new(NoneProvider::new(true));
        let event_emitting_provider =
            EventEmittingSearchProvider::new(none_provider, Arc::new(event_bus));

        // Perform search operations
        let result = event_emitting_provider
            .search("https://example.com", 10, "us", "en")
            .await;

        assert!(result.is_ok());

        // Give some time for event processing
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Verify events were received
        let events_received = handler.get_events_received();
        assert!(events_received >= 2); // At least search_started and search_completed

        // Check last event contains search information
        let last_event_json = handler.get_last_event().await;
        assert!(last_event_json.is_some());

        let json = last_event_json.unwrap();
        assert!(json.contains("search_") || json.contains("search_duration"));
    }

    /// Test search provider failure events
    #[tokio::test]
    async fn test_search_provider_failure_events() {
        let mut event_bus = EventBus::new();
        let handler = Arc::new(SearchProviderEventHandler::new());

        event_bus.register_handler(handler.clone()).await.unwrap();
        event_bus.start().await.unwrap();

        // Create provider that will fail (None provider with invalid query)
        let none_provider = Box::new(NoneProvider::new(true));
        let event_emitting_provider =
            EventEmittingSearchProvider::new(none_provider, Arc::new(event_bus));

        // Perform search that will fail
        let result = event_emitting_provider
            .search("not a valid url", 10, "us", "en")
            .await;

        assert!(result.is_err());

        // Give time for event processing
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Verify failure events were emitted
        let events_received = handler.get_events_received();
        assert!(events_received >= 1);

        let last_event_json = handler.get_last_event().await;
        assert!(last_event_json.is_some());

        let json = last_event_json.unwrap();
        assert!(json.contains("search_failed") || json.contains("error"));
    }

    /// Test circuit breaker integration with events
    #[tokio::test]
    async fn test_circuit_breaker_event_integration() {
        let mut event_bus = EventBus::new();
        let handler = Arc::new(SearchProviderEventHandler::new());

        event_bus.register_handler(handler.clone()).await.unwrap();
        event_bus.start().await.unwrap();

        // Create circuit breaker wrapped provider
        let none_provider = Box::new(NoneProvider::new(true));
        let circuit_breaker_provider = Box::new(CircuitBreakerWrapper::new(none_provider));
        let event_emitting_provider =
            EventEmittingSearchProvider::new(circuit_breaker_provider, Arc::new(event_bus));

        // Perform successful search
        let result = event_emitting_provider
            .search("https://example.com", 10, "us", "en")
            .await;

        assert!(result.is_ok());

        tokio::time::sleep(Duration::from_millis(200)).await;

        let events_received = handler.get_events_received();
        assert!(events_received >= 2); // Should have success events
    }

    /// Test multiple handlers receiving search events
    #[tokio::test]
    async fn test_multiple_handlers_search_events() {
        let mut event_bus = EventBus::new();

        // Create multiple handlers
        let search_handler = Arc::new(SearchProviderEventHandler::new());
        let logging_handler = Arc::new(LoggingEventHandler::new());
        let health_handler = Arc::new(HealthEventHandler::new());

        // Register all handlers
        event_bus
            .register_handler(search_handler.clone())
            .await
            .unwrap();
        event_bus.register_handler(logging_handler).await.unwrap();
        event_bus.register_handler(health_handler).await.unwrap();

        event_bus.start().await.unwrap();

        // Create search provider
        let none_provider = Box::new(NoneProvider::new(true));
        let event_emitting_provider =
            EventEmittingSearchProvider::new(none_provider, Arc::new(event_bus));

        // Perform searches
        for i in 0..3 {
            let url = format!("https://example{}.com", i);
            let result = event_emitting_provider.search(&url, 10, "us", "en").await;
            assert!(result.is_ok());
        }

        tokio::time::sleep(Duration::from_millis(300)).await;

        // Verify the search-specific handler received events
        let events_received = search_handler.get_events_received();
        assert!(events_received >= 6); // 3 searches * 2+ events per search
    }

    /// Test event bus configuration with search providers
    #[tokio::test]
    async fn test_event_bus_configuration_with_search() {
        // Create event bus with custom configuration
        let custom_config = EventBusConfig {
            buffer_size: 500,
            async_handlers: false, // Sequential processing
            handler_timeout: Duration::from_secs(2),
            continue_on_handler_error: true,
        };

        let mut event_bus = EventBus::with_config(custom_config);
        let handler = Arc::new(SearchProviderEventHandler::new());

        event_bus.register_handler(handler.clone()).await.unwrap();
        event_bus.start().await.unwrap();

        let none_provider = Box::new(NoneProvider::new(true));
        let event_emitting_provider =
            EventEmittingSearchProvider::new(none_provider, Arc::new(event_bus));

        // Perform search
        let result = event_emitting_provider
            .search("https://example.com https://test.org", 10, "us", "en")
            .await;

        assert!(result.is_ok());
        let hits = result.unwrap();
        assert_eq!(hits.len(), 2); // Two URLs should be found

        tokio::time::sleep(Duration::from_millis(300)).await;

        // Verify events were processed sequentially
        let events_received = handler.get_events_received();
        assert!(events_received >= 2);
    }

    /// Test search provider health check events
    #[tokio::test]
    async fn test_search_provider_health_check_events() {
        let mut event_bus = EventBus::new();
        let handler = Arc::new(SearchProviderEventHandler::new());

        event_bus.register_handler(handler.clone()).await.unwrap();
        event_bus.start().await.unwrap();

        let none_provider = Box::new(NoneProvider::new(true));
        let event_emitting_provider =
            EventEmittingSearchProvider::new(none_provider, Arc::new(event_bus));

        // Perform health check (None provider should always pass)
        let health_result = event_emitting_provider.health_check().await;
        assert!(health_result.is_ok());

        // Health checks don't emit events in our current implementation,
        // but we could extend this to emit health status events
        let backend_type = event_emitting_provider.backend_type();
        assert_eq!(backend_type, SearchBackend::None);
    }

    /// Test concurrent search operations with events
    #[tokio::test]
    async fn test_concurrent_search_with_events() {
        let mut event_bus = EventBus::new();
        let handler = Arc::new(SearchProviderEventHandler::new());

        event_bus.register_handler(handler.clone()).await.unwrap();
        event_bus.start().await.unwrap();

        let none_provider = Box::new(NoneProvider::new(true));
        let provider = Arc::new(EventEmittingSearchProvider::new(
            none_provider,
            Arc::new(event_bus),
        ));

        // Launch concurrent searches
        let mut handles = Vec::new();
        for i in 0..5 {
            let provider_clone = provider.clone();
            let handle = tokio::spawn(async move {
                let url = format!("https://concurrent{}.com", i);
                provider_clone.search(&url, 10, "us", "en").await
            });
            handles.push(handle);
        }

        // Wait for all searches to complete
        let mut success_count = 0;
        for handle in handles {
            if let Ok(result) = handle.await {
                if result.is_ok() {
                    success_count += 1;
                }
            }
        }

        assert_eq!(success_count, 5);

        tokio::time::sleep(Duration::from_millis(300)).await;

        // Should have received events from all searches
        let events_received = handler.get_events_received();
        assert!(events_received >= 10); // 5 searches * 2+ events each
    }

    /// Test event filtering with search provider events
    #[tokio::test]
    async fn test_event_filtering_with_search() {
        let mut event_bus = EventBus::new();

        // Create handler that only handles error events
        let mut error_handler_config = HandlerConfig::default();
        error_handler_config.min_severity = EventSeverity::Error;

        let handler = Arc::new(SearchProviderEventHandler::new());
        event_bus.register_handler(handler.clone()).await.unwrap();
        event_bus.start().await.unwrap();

        let none_provider = Box::new(NoneProvider::new(true));
        let event_emitting_provider =
            EventEmittingSearchProvider::new(none_provider, Arc::new(event_bus));

        // Perform successful search (Info level events)
        let result = event_emitting_provider
            .search("https://example.com", 10, "us", "en")
            .await;
        assert!(result.is_ok());

        // Perform failed search (Error level events)
        let result = event_emitting_provider
            .search("not a url", 10, "us", "en")
            .await;
        assert!(result.is_err());

        tokio::time::sleep(Duration::from_millis(200)).await;

        // Handler should have received all events (it accepts Debug+)
        let events_received = handler.get_events_received();
        assert!(events_received >= 3); // Some from success, some from failure
    }
}

// Activate robustness tests for event system
#[cfg(test)]
mod event_system_robustness_tests {
    use super::*;

    /// Test event system behavior under high load
    #[tokio::test]
    async fn test_high_load_event_processing() {
        let mut event_bus = EventBus::new();
        let handler = Arc::new(SearchProviderEventHandler::new());

        event_bus.register_handler(handler.clone()).await.unwrap();
        event_bus.start().await.unwrap();

        let none_provider = Box::new(NoneProvider::new(true));
        let provider = Arc::new(EventEmittingSearchProvider::new(
            none_provider,
            Arc::new(event_bus),
        ));

        // Generate high load
        let mut handles = Vec::new();
        for i in 0..20 {
            let provider_clone = provider.clone();
            let handle = tokio::spawn(async move {
                let url = format!("https://load-test-{}.com", i);
                for _ in 0..5 {
                    let _ = provider_clone.search(&url, 1, "us", "en").await;
                }
            });
            handles.push(handle);
        }

        // Wait for completion
        for handle in handles {
            let _ = handle.await;
        }

        tokio::time::sleep(Duration::from_millis(500)).await;

        // Should have processed many events successfully
        let events_received = handler.get_events_received();
        assert!(events_received >= 100); // 20 tasks * 5 searches * 2+ events
    }

    /// Test event system graceful degradation
    #[tokio::test]
    async fn test_event_system_graceful_degradation() {
        let config = EventBusConfig {
            buffer_size: 10, // Very small buffer to test overflow
            async_handlers: true,
            handler_timeout: Duration::from_millis(50), // Short timeout
            continue_on_handler_error: true,
        };

        let mut event_bus = EventBus::with_config(config);
        let handler = Arc::new(SearchProviderEventHandler::new());

        event_bus.register_handler(handler.clone()).await.unwrap();
        event_bus.start().await.unwrap();

        let none_provider = Box::new(NoneProvider::new(true));
        let provider = EventEmittingSearchProvider::new(none_provider, Arc::new(event_bus));

        // Try to overwhelm the system
        for i in 0..15 {
            let url = format!("https://stress-test-{}.com", i);
            let _ = provider.search(&url, 1, "us", "en").await;
        }

        tokio::time::sleep(Duration::from_millis(200)).await;

        // System should still process some events even under stress
        let events_received = handler.get_events_received();
        assert!(events_received > 0);
    }
}
