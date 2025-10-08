//! Comprehensive unit tests for riptide-core

use riptide_core::*;
use std::time::Duration;

#[cfg(test)]
mod extraction_tests {
    use super::*;
    use riptide_core::types::{BasicExtractedDoc, ExtractionConfig};

    #[test]
    fn test_extracted_doc_creation() {
        let doc = BasicExtractedDoc {
            url: "https://example.com".to_string(),
            title: Some("Test Title".to_string()),
            text: "Test content".to_string(),
            markdown: Some("# Test Title\nTest content".to_string()),
            published_iso: Some("2024-03-15T12:00:00Z".to_string()),
            byline: Some("By John Doe".to_string()),
            language: Some("en".to_string()),
            media: vec![],
            links: vec![],
            word_count: Some(2),
            quality_score: Some(85),
            reading_time: Some(1),
            categories: vec![],
            site_name: None,
            description: None,
        };

        assert_eq!(doc.url, "https://example.com");
        assert_eq!(doc.title.unwrap(), "Test Title");
        assert_eq!(doc.word_count, Some(2));
        assert_eq!(doc.quality_score, Some(85));
    }

    #[test]
    fn test_extraction_config() {
        let config = ExtractionConfig {
            mode: "standard".to_string(),
            timeout: Duration::from_secs(30),
            max_retries: 3,
            enable_repair: true,
        };

        assert_eq!(config.mode, "standard");
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert!(config.enable_repair);
    }

    #[test]
    fn test_extraction_config_default() {
        let config = ExtractionConfig::default();
        assert_eq!(config.mode, "default");
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.max_retries, 2);
        assert!(!config.enable_repair);
    }
}

#[cfg(test)]
mod reliability_tests {
    use super::*;
    use riptide_core::reliability::{CircuitBreaker, CircuitState};

    #[test]
    fn test_circuit_breaker_initialization() {
        let breaker = CircuitBreaker::new(3, Duration::from_secs(5));
        assert_eq!(breaker.state(), CircuitState::Closed);
        assert_eq!(breaker.failure_count(), 0);
    }

    #[test]
    fn test_circuit_breaker_failure_tracking() {
        let mut breaker = CircuitBreaker::new(3, Duration::from_secs(5));

        breaker.record_failure();
        assert_eq!(breaker.failure_count(), 1);
        assert_eq!(breaker.state(), CircuitState::Closed);

        breaker.record_failure();
        breaker.record_failure();
        assert_eq!(breaker.failure_count(), 3);
        assert_eq!(breaker.state(), CircuitState::Open);
    }

    #[test]
    fn test_circuit_breaker_success_reset() {
        let mut breaker = CircuitBreaker::new(3, Duration::from_secs(5));

        breaker.record_failure();
        breaker.record_failure();
        assert_eq!(breaker.failure_count(), 2);

        breaker.record_success();
        assert_eq!(breaker.failure_count(), 0);
        assert_eq!(breaker.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_timeout() {
        let mut breaker = CircuitBreaker::new(1, Duration::from_millis(100));

        breaker.record_failure();
        assert_eq!(breaker.state(), CircuitState::Open);

        tokio::time::sleep(Duration::from_millis(150)).await;

        // After timeout, should transition to half-open
        assert_eq!(breaker.state(), CircuitState::HalfOpen);

        breaker.record_success();
        assert_eq!(breaker.state(), CircuitState::Closed);
    }
}

#[cfg(test)]
mod cache_tests {
    use super::*;
    use riptide_core::cache::{CacheConfig, ExtractorCache};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_cache_basic_operations() {
        let config = CacheConfig {
            max_size: 100,
            ttl: Duration::from_secs(3600),
            eviction_policy: "lru".to_string(),
        };
        let cache = ExtractorCache::new(config);

        // Test insertion
        cache.put("key1", "value1").await;
        assert_eq!(cache.get("key1").await, Some("value1".to_string()));

        // Test overwrite
        cache.put("key1", "value2").await;
        assert_eq!(cache.get("key1").await, Some("value2".to_string()));

        // Test missing key
        assert_eq!(cache.get("missing").await, None);
    }

    #[tokio::test]
    async fn test_cache_ttl_expiration() {
        let config = CacheConfig {
            max_size: 100,
            ttl: Duration::from_millis(100),
            eviction_policy: "lru".to_string(),
        };
        let cache = ExtractorCache::new(config);

        cache.put("temp", "data").await;
        assert_eq!(cache.get("temp").await, Some("data".to_string()));

        tokio::time::sleep(Duration::from_millis(150)).await;
        assert_eq!(cache.get("temp").await, None);
    }

    #[tokio::test]
    async fn test_cache_size_limit() {
        let config = CacheConfig {
            max_size: 3,
            ttl: Duration::from_secs(3600),
            eviction_policy: "lru".to_string(),
        };
        let cache = ExtractorCache::new(config);

        cache.put("key1", "value1").await;
        cache.put("key2", "value2").await;
        cache.put("key3", "value3").await;

        // Access key1 to make it recently used
        cache.get("key1").await;

        // Add another key, should evict least recently used (key2)
        cache.put("key4", "value4").await;

        assert_eq!(cache.get("key1").await, Some("value1".to_string()));
        assert_eq!(cache.get("key2").await, None); // Evicted
        assert_eq!(cache.get("key3").await, Some("value3".to_string()));
        assert_eq!(cache.get("key4").await, Some("value4".to_string()));
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let cache = ExtractorCache::new(CacheConfig::default());

        cache.put("key1", "value1").await;
        cache.put("key2", "value2").await;

        cache.clear().await;

        assert_eq!(cache.get("key1").await, None);
        assert_eq!(cache.get("key2").await, None);
        assert_eq!(cache.size().await, 0);
    }
}

#[cfg(test)]
mod component_tests {
    use super::*;
    use riptide_core::component::{ExtractorConfig, PerformanceMetrics, WasmResourceTracker};

    #[test]
    fn test_extractor_config() {
        let config = ExtractorConfig {
            mode: "advanced".to_string(),
            css_selector: Some(".content".to_string()),
            probes: vec!["probe1".to_string(), "probe2".to_string()],
            enable_repair: true,
        };

        assert_eq!(config.mode, "advanced");
        assert_eq!(config.css_selector, Some(".content".to_string()));
        assert_eq!(config.probes.len(), 2);
    }

    #[test]
    fn test_performance_metrics() {
        let mut metrics = PerformanceMetrics::default();

        metrics.requests_per_second = 100.0;
        metrics.average_latency_ms = 50;
        metrics.memory_used_mb = 256;
        metrics.success_rate = 0.99;

        assert_eq!(metrics.requests_per_second, 100.0);
        assert_eq!(metrics.average_latency_ms, 50);
        assert!(metrics.success_rate > 0.98);
    }

    #[test]
    fn test_resource_tracker() {
        let mut tracker = WasmResourceTracker::new();

        tracker.track_memory(1024 * 1024); // 1MB
        tracker.track_cpu(50.0);

        assert!(tracker.memory_usage() > 0);
        assert_eq!(tracker.cpu_usage(), 50.0);

        // Test memory limit
        assert!(tracker.check_memory_limit(512 * 1024 * 1024)); // 512MB OK
        assert!(!tracker.check_memory_limit(10 * 1024)); // 10KB too small
    }
}

#[cfg(test)]
mod event_bus_tests {
    use super::*;
    use riptide_core::events::{EventBus, ExtractionEvent};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[tokio::test]
    async fn test_event_bus_publish_subscribe() {
        let bus = Arc::new(EventBus::new());
        let received = Arc::new(RwLock::new(Vec::new()));

        let received_clone = received.clone();
        bus.subscribe("extraction_complete", move |event| {
            let received = received_clone.clone();
            async move {
                received.write().await.push(event);
            }
        })
        .await;

        bus.publish(
            "extraction_complete",
            ExtractionEvent {
                url: "https://example.com".to_string(),
                success: true,
                duration_ms: 100,
            },
        )
        .await;

        tokio::time::sleep(Duration::from_millis(50)).await;

        let events = received.read().await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].url, "https://example.com");
        assert!(events[0].success);
    }

    #[tokio::test]
    async fn test_event_bus_multiple_subscribers() {
        let bus = Arc::new(EventBus::new());
        let counter1 = Arc::new(RwLock::new(0));
        let counter2 = Arc::new(RwLock::new(0));

        let counter1_clone = counter1.clone();
        bus.subscribe("test_event", move |_| {
            let counter = counter1_clone.clone();
            async move {
                *counter.write().await += 1;
            }
        })
        .await;

        let counter2_clone = counter2.clone();
        bus.subscribe("test_event", move |_| {
            let counter = counter2_clone.clone();
            async move {
                *counter.write().await += 1;
            }
        })
        .await;

        bus.publish("test_event", ()).await;
        tokio::time::sleep(Duration::from_millis(50)).await;

        assert_eq!(*counter1.read().await, 1);
        assert_eq!(*counter2.read().await, 1);
    }

    #[tokio::test]
    async fn test_event_bus_unsubscribe() {
        let bus = Arc::new(EventBus::new());
        let counter = Arc::new(RwLock::new(0));

        let counter_clone = counter.clone();
        let id = bus
            .subscribe("test", move |_| {
                let counter = counter_clone.clone();
                async move {
                    *counter.write().await += 1;
                }
            })
            .await;

        bus.publish("test", ()).await;
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(*counter.read().await, 1);

        bus.unsubscribe(id).await;

        bus.publish("test", ()).await;
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(*counter.read().await, 1); // Should not increment
    }
}
