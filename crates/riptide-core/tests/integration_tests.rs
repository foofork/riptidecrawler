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
    use std::sync::Arc;
    use std::time::Duration;

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
    // Note: Cache tests require a running Redis instance.
    // The CacheManager API has been updated to use Redis with enhanced HTTP caching support.
    // See src/cache.rs for the new API:
    // - CacheManager::new(redis_url: &str)
    // - CacheManager::new_with_config(redis_url: &str, config: CacheConfig)
    // - set_simple() and get_simple() for basic caching
    // - set() and get() for advanced HTTP caching with ETags and Last-Modified

    // These tests are commented out as they would require a Redis connection:
    /*
    use riptide_core::cache::{CacheConfig, CacheManager, CacheMetadata};

    #[tokio::test]
    #[ignore = "requires Redis"]
    async fn test_cache_manager_basic_operations() {
        let mut cache = CacheManager::new("redis://localhost:6379").await.unwrap();

        // Test simple set and get
        cache.set_simple("test_key", &"test_value", 60).await.unwrap();
        let value: Option<String> = cache.get_simple("test_key").await.unwrap();
        assert_eq!(value, Some("test_value".to_string()));

        // Test deletion
        cache.delete("test_key").await.unwrap();
        let value: Option<String> = cache.get_simple("test_key").await.unwrap();
        assert_eq!(value, None);
    }
    */
}

// Note: Instance pool tests moved to dedicated instance_pool_tests.rs file
// The InstancePool API has been refactored to AdvancedInstancePool

// Note: Event bus tests use a trait-based Event system now
// EventType enum has been removed in favor of string-based event types
// See events/types.rs for BaseEvent and PoolEvent implementations
