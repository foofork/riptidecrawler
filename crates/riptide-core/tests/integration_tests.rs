//! Integration tests for riptide-core
//!
//! These tests verify the core functionality works correctly in real-world scenarios

use anyhow::Result;

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

    // Note: evaluate_extraction_quality is now private
    // Quality evaluation is tested through the public extract_with_reliability method
}

#[cfg(test)]
mod circuit_breaker_tests {
    use super::*;
    use riptide_core::circuit::{CircuitBreaker, Config as CircuitConfig, State as CircuitState};
    use riptide_core::fetch::CircuitBreakerConfig;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_circuit_breaker_state_transitions() {
        let config = CircuitConfig {
            failure_threshold: 3,
            open_cooldown_ms: 100,
            half_open_max_in_flight: 1,
        };

        let breaker = CircuitBreaker::new(config, Arc::new(riptide_core::circuit::RealClock));

        // Initial state should be closed
        assert_eq!(breaker.state(), CircuitState::Closed);

        // Record failures to open circuit
        for _ in 0..3 {
            breaker.on_failure();
        }
        assert_eq!(breaker.state(), CircuitState::Open);

        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should transition to half-open
        let _permit = breaker.try_acquire();
        assert_eq!(breaker.state(), CircuitState::HalfOpen);

        // Record success to close circuit
        breaker.on_success();
        assert_eq!(breaker.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_with_real_requests() {
        let config = CircuitConfig::default();
        let breaker = CircuitBreaker::new(config, Arc::new(riptide_core::circuit::RealClock));

        // Simulate request patterns
        async fn make_request(breaker: &CircuitBreaker, should_fail: bool) -> Result<()> {
            let _permit = breaker.try_acquire().map_err(|e| anyhow::anyhow!(e))?;

            if should_fail {
                breaker.on_failure();
                Err(anyhow::anyhow!("Request failed"))
            } else {
                breaker.on_success();
                Ok(())
            }
        }

        // Test successful requests
        for _ in 0..5 {
            let result = make_request(&breaker, false).await;
            assert!(result.is_ok());
        }

        // Test failure pattern
        for _ in 0..5 {
            let _ = make_request(&breaker, true).await;
        }

        // Circuit should be open now
        let result = make_request(&breaker, false).await;
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod cache_tests {
    use anyhow::Result;
    use riptide_core::cache::{CacheConfig, CacheManager};
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

// Note: Instance pool tests moved to dedicated instance_pool_tests.rs file
// The InstancePool API has been refactored to AdvancedInstancePool

// Note: Event bus tests use a trait-based Event system now
// EventType enum has been removed in favor of string-based event types
// See events/types.rs for BaseEvent and PoolEvent implementations
