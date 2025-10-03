//! Integration tests for riptide-core
//!
//! These tests verify the core functionality works correctly in real-world scenarios

use anyhow::Result;
use riptide_core::*;

#[cfg(test)]
mod reliability_tests {
    use super::*;
    use riptide_core::reliability::{
        ExtractionMode, ReliabilityConfig, ReliableExtractor, WasmExtractor,
    };
    use riptide_core::types::ExtractedDoc;

    struct MockWasmExtractor;

    impl WasmExtractor for MockWasmExtractor {
        fn extract(&self, html: &[u8], url: &str, _mode: &str) -> Result<ExtractedDoc> {
            Ok(ExtractedDoc {
                url: url.to_string(),
                title: Some("Test Page".to_string()),
                text: String::from_utf8_lossy(html).into_owned(),
                quality_score: Some(85),
                links: vec![],
                byline: None,
                published_iso: None,
                markdown: Some("# Test Content".to_string()),
                media: vec![],
                language: Some("en".to_string()),
                reading_time: Some(2),
                word_count: Some(100),
                categories: vec![],
                site_name: None,
                description: Some("Test description".to_string()),
            })
        }
    }

    #[tokio::test]
    async fn test_reliable_extraction_with_fallback() {
        let config = ReliabilityConfig::default();
        let extractor = ReliableExtractor::new(config).unwrap();
        let wasm_extractor = MockWasmExtractor;

        // Test fast extraction
        let result = extractor
            .extract_with_reliability(
                "https://example.com",
                ExtractionMode::Fast,
                &wasm_extractor,
                None,
            )
            .await;

        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.url, "https://example.com");
        assert!(doc.title.is_some());
    }

    #[tokio::test]
    async fn test_extraction_mode_selection() {
        let config = ReliabilityConfig::default();
        let extractor = ReliableExtractor::new(config).unwrap();
        let wasm_extractor = MockWasmExtractor;

        // Test different extraction modes
        for mode in [ExtractionMode::Fast, ExtractionMode::ProbesFirst] {
            let result = extractor
                .extract_with_reliability(
                    "https://example.com",
                    mode.clone(),
                    &wasm_extractor,
                    None,
                )
                .await;

            assert!(result.is_ok(), "Mode {:?} should succeed", mode);
        }
    }

    #[test]
    fn test_extraction_quality_evaluation() {
        let config = ReliabilityConfig::default();
        let extractor = ReliableExtractor::new(config).unwrap();

        // Test high quality document
        let high_quality = ExtractedDoc {
            url: "https://test.com".to_string(),
            title: Some("Great Article".to_string()),
            text: "Long comprehensive content".repeat(100),
            markdown: Some("# Title\n## Subtitle\n*emphasis*".to_string()),
            byline: Some("Author".to_string()),
            description: Some("Description".to_string()),
            links: vec!["link1".to_string(), "link2".to_string()],
            ..Default::default()
        };

        let score = extractor.evaluate_extraction_quality(&high_quality);
        assert!(
            score > 0.7,
            "High quality doc should score > 0.7, got {}",
            score
        );

        // Test low quality document
        let low_quality = ExtractedDoc {
            url: "https://test.com".to_string(),
            title: None,
            text: "Short".to_string(),
            markdown: None,
            ..Default::default()
        };

        let score = extractor.evaluate_extraction_quality(&low_quality);
        assert!(
            score < 0.4,
            "Low quality doc should score < 0.4, got {}",
            score
        );
    }
}

#[cfg(test)]
mod circuit_breaker_tests {
    use super::*;
    use riptide_core::circuit::{CircuitBreaker, CircuitBreakerConfig, CircuitState};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_circuit_breaker_state_transitions() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            timeout: std::time::Duration::from_millis(100),
            half_open_max_calls: 1,
        };

        let breaker = CircuitBreaker::new("test", config);

        // Initial state should be closed
        assert_eq!(breaker.state(), CircuitState::Closed);

        // Record failures to open circuit
        for _ in 0..3 {
            breaker.record_failure().await;
        }
        assert_eq!(breaker.state(), CircuitState::Open);

        // Wait for timeout
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;

        // Should transition to half-open
        assert!(breaker.allow_request().await);
        assert_eq!(breaker.state(), CircuitState::HalfOpen);

        // Record success to close circuit
        breaker.record_success().await;
        breaker.record_success().await;
        assert_eq!(breaker.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_with_real_requests() {
        let config = CircuitBreakerConfig::default();
        let breaker = Arc::new(CircuitBreaker::new("http_service", config));

        // Simulate request patterns
        async fn make_request(breaker: Arc<CircuitBreaker>, should_fail: bool) -> Result<()> {
            if !breaker.allow_request().await {
                return Err(anyhow::anyhow!("Circuit open"));
            }

            if should_fail {
                breaker.record_failure().await;
                Err(anyhow::anyhow!("Request failed"))
            } else {
                breaker.record_success().await;
                Ok(())
            }
        }

        // Test successful requests
        for _ in 0..5 {
            let result = make_request(breaker.clone(), false).await;
            assert!(result.is_ok());
        }

        // Test failure pattern
        for _ in 0..5 {
            let _ = make_request(breaker.clone(), true).await;
        }

        // Circuit should be open now
        let result = make_request(breaker.clone(), false).await;
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod cache_tests {
    use super::*;
    use riptide_core::cache::{CacheConfig, CacheEntry, CacheManager};
    use std::time::Duration;

    #[tokio::test]
    async fn test_cache_manager_operations() {
        let config = CacheConfig {
            max_size: 100,
            ttl: Duration::from_secs(60),
            enable_compression: true,
            ..Default::default()
        };

        let cache = CacheManager::new(config);

        // Test set and get
        cache.set("key1", b"value1", None).await.unwrap();
        let value = cache.get("key1").await.unwrap();
        assert_eq!(value, Some(b"value1".to_vec()));

        // Test expiration
        cache
            .set("key2", b"value2", Some(Duration::from_millis(50)))
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        let value = cache.get("key2").await.unwrap();
        assert_eq!(value, None);

        // Test deletion
        cache.set("key3", b"value3", None).await.unwrap();
        cache.delete("key3").await.unwrap();
        let value = cache.get("key3").await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_cache_size_limits() {
        let config = CacheConfig {
            max_size: 3,
            ttl: Duration::from_secs(60),
            ..Default::default()
        };

        let cache = CacheManager::new(config);

        // Fill cache to capacity
        for i in 0..5 {
            cache
                .set(&format!("key{}", i), format!("value{}", i).as_bytes(), None)
                .await
                .unwrap();
        }

        // Check that only last 3 items are in cache
        assert!(cache.get("key0").await.unwrap().is_none());
        assert!(cache.get("key1").await.unwrap().is_none());
        assert!(cache.get("key2").await.unwrap().is_some());
        assert!(cache.get("key3").await.unwrap().is_some());
        assert!(cache.get("key4").await.unwrap().is_some());
    }
}

#[cfg(test)]
mod instance_pool_tests {
    use super::*;
    use riptide_core::component::{CmExtractor, ExtractorConfig};
    use riptide_core::instance_pool::{InstancePool, PoolConfig};

    #[tokio::test]
    async fn test_instance_pool_lifecycle() {
        let config = PoolConfig {
            min_instances: 1,
            max_instances: 3,
            max_idle_time: Duration::from_secs(60),
            health_check_interval: Duration::from_secs(30),
        };

        let pool = InstancePool::new(config).await.unwrap();

        // Acquire instance
        let instance = pool.acquire().await.unwrap();
        assert!(instance.is_healthy());

        // Release instance
        pool.release(instance).await;

        // Pool should maintain min instances
        let stats = pool.stats().await;
        assert!(stats.total_instances >= 1);
        assert!(stats.available_instances >= 1);
    }

    #[tokio::test]
    async fn test_instance_pool_scaling() {
        let config = PoolConfig {
            min_instances: 1,
            max_instances: 5,
            ..Default::default()
        };

        let pool = InstancePool::new(config).await.unwrap();

        // Acquire multiple instances to trigger scaling
        let mut instances = vec![];
        for _ in 0..3 {
            instances.push(pool.acquire().await.unwrap());
        }

        let stats = pool.stats().await;
        assert_eq!(stats.total_instances, 3);
        assert_eq!(stats.busy_instances, 3);

        // Release all instances
        for instance in instances {
            pool.release(instance).await;
        }

        // Pool should scale down eventually
        tokio::time::sleep(Duration::from_millis(100)).await;
        let stats = pool.stats().await;
        assert!(stats.available_instances > 0);
    }
}

#[cfg(test)]
mod event_bus_tests {
    use super::*;
    use riptide_core::events::{Event, EventBus, EventType};
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_event_publishing_and_subscription() {
        let event_bus = EventBus::new();

        // Create subscriber
        let (tx, mut rx) = mpsc::unbounded_channel();
        event_bus.subscribe(EventType::ExtractionComplete, tx).await;

        // Publish event
        let event = Event {
            event_type: EventType::ExtractionComplete,
            data: serde_json::json!({
                "url": "https://example.com",
                "success": true
            }),
            timestamp: chrono::Utc::now(),
        };

        event_bus.publish(event.clone()).await.unwrap();

        // Verify subscriber receives event
        let received = rx.recv().await.unwrap();
        assert_eq!(received.event_type, EventType::ExtractionComplete);
        assert_eq!(received.data["url"], "https://example.com");
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let event_bus = EventBus::new();

        // Create multiple subscribers
        let (tx1, mut rx1) = mpsc::unbounded_channel();
        let (tx2, mut rx2) = mpsc::unbounded_channel();

        event_bus.subscribe(EventType::CrawlStarted, tx1).await;
        event_bus.subscribe(EventType::CrawlStarted, tx2).await;

        // Publish event
        let event = Event {
            event_type: EventType::CrawlStarted,
            data: serde_json::json!({"job_id": "123"}),
            timestamp: chrono::Utc::now(),
        };

        event_bus.publish(event).await.unwrap();

        // Both subscribers should receive the event
        assert!(rx1.recv().await.is_some());
        assert!(rx2.recv().await.is_some());
    }
}
