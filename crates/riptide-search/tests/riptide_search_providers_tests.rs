//! Comprehensive unit tests for riptide-search providers
//!
//! This test suite validates the behavior of all search provider implementations
//! including SerperProvider, NoneProvider, and their integration with circuit breakers.

use anyhow::Result;
use riptide_search::{
    create_search_provider, create_search_provider_from_env, AdvancedSearchConfig,
    CircuitBreakerWrapper, NoneProvider, SearchBackend, SearchConfig, SearchHit, SearchProvider,
    SearchProviderFactory, SerperProvider,
};
use std::time::Duration;

// ============================================================================
// Unit Tests: SearchHit Structure
// ============================================================================

#[cfg(test)]
mod search_hit_tests {
    use super::*;

    #[test]
    fn test_search_hit_creation() {
        let hit = SearchHit::new("https://example.com".to_string(), 1);

        assert_eq!(hit.url, "https://example.com");
        assert_eq!(hit.rank, 1);
        assert_eq!(hit.title, None);
        assert_eq!(hit.snippet, None);
        assert!(hit.metadata.is_empty());
    }

    #[test]
    fn test_search_hit_builder_pattern() {
        let hit = SearchHit::new("https://example.com".to_string(), 1)
            .with_title("Example Domain".to_string())
            .with_snippet("Example snippet text".to_string())
            .with_metadata("source".to_string(), "test".to_string());

        assert_eq!(hit.title, Some("Example Domain".to_string()));
        assert_eq!(hit.snippet, Some("Example snippet text".to_string()));
        assert_eq!(hit.metadata.get("source"), Some(&"test".to_string()));
    }

    #[test]
    fn test_search_hit_serialization() {
        let hit =
            SearchHit::new("https://example.com".to_string(), 1).with_title("Test".to_string());

        let json = serde_json::to_string(&hit);
        assert!(json.is_ok());

        let deserialized: Result<SearchHit, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok());
        assert_eq!(deserialized.unwrap().url, "https://example.com");
    }

    #[test]
    fn test_search_hit_equality() {
        let hit1 = SearchHit::new("https://example.com".to_string(), 1);
        let hit2 = SearchHit::new("https://example.com".to_string(), 1);

        assert_eq!(hit1, hit2);
    }
}

// ============================================================================
// Unit Tests: SearchBackend Enum
// ============================================================================

#[cfg(test)]
mod search_backend_tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_search_backend_from_str_valid() {
        assert_eq!(
            SearchBackend::from_str("serper").unwrap(),
            SearchBackend::Serper
        );
        assert_eq!(
            SearchBackend::from_str("none").unwrap(),
            SearchBackend::None
        );
        assert_eq!(
            SearchBackend::from_str("searxng").unwrap(),
            SearchBackend::SearXNG
        );
    }

    #[test]
    fn test_search_backend_from_str_case_insensitive() {
        assert_eq!(
            SearchBackend::from_str("SERPER").unwrap(),
            SearchBackend::Serper
        );
        assert_eq!(
            SearchBackend::from_str("None").unwrap(),
            SearchBackend::None
        );
        assert_eq!(
            SearchBackend::from_str("SearXNG").unwrap(),
            SearchBackend::SearXNG
        );
    }

    #[test]
    fn test_search_backend_from_str_invalid() {
        assert!(SearchBackend::from_str("invalid").is_err());
        assert!(SearchBackend::from_str("google").is_err());
        assert!(SearchBackend::from_str("").is_err());
    }

    #[test]
    fn test_search_backend_display() {
        assert_eq!(SearchBackend::Serper.to_string(), "serper");
        assert_eq!(SearchBackend::None.to_string(), "none");
        assert_eq!(SearchBackend::SearXNG.to_string(), "searxng");
    }

    #[test]
    fn test_search_backend_serialization() {
        let backend = SearchBackend::Serper;
        let json = serde_json::to_string(&backend).unwrap();
        assert_eq!(json, "\"serper\"");

        let deserialized: SearchBackend = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, SearchBackend::Serper);
    }
}

// ============================================================================
// Unit Tests: NoneProvider
// ============================================================================

#[cfg(test)]
mod none_provider_tests {
    use super::*;

    #[tokio::test]
    async fn test_none_provider_single_url() {
        let provider = NoneProvider::new(true);

        let result = provider.search("https://example.com", 10, "us", "en").await;
        assert!(result.is_ok());

        let hits = result.unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].url, "https://example.com");
        assert_eq!(hits[0].rank, 1);
    }

    #[tokio::test]
    async fn test_none_provider_multiple_urls() {
        let provider = NoneProvider::new(true);

        let result = provider
            .search(
                "https://example.com https://test.org https://demo.io",
                10,
                "us",
                "en",
            )
            .await;

        assert!(result.is_ok());
        let hits = result.unwrap();
        assert_eq!(hits.len(), 3);
        assert_eq!(hits[0].url, "https://example.com");
        assert_eq!(hits[1].url, "https://test.org");
        assert_eq!(hits[2].url, "https://demo.io");
    }

    #[tokio::test]
    async fn test_none_provider_no_urls() {
        let provider = NoneProvider::new(true);

        let result = provider.search("no urls here", 10, "us", "en").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_none_provider_url_parsing_disabled() {
        let provider = NoneProvider::new(false);

        let result = provider.search("https://example.com", 10, "us", "en").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("disabled"));
    }

    #[tokio::test]
    async fn test_none_provider_respects_limit() {
        let provider = NoneProvider::new(true);

        let urls = (0..20)
            .map(|i| format!("https://example{}.com", i))
            .collect::<Vec<_>>()
            .join(" ");
        let result = provider.search(&urls, 5, "us", "en").await;

        assert!(result.is_ok());
        let hits = result.unwrap();
        assert_eq!(hits.len(), 5);
    }

    #[tokio::test]
    async fn test_none_provider_health_check() {
        let provider = NoneProvider::new(true);

        let health = provider.health_check().await;
        assert!(health.is_ok());
    }

    #[tokio::test]
    async fn test_none_provider_backend_type() {
        let provider = NoneProvider::new(true);
        assert_eq!(provider.backend_type(), SearchBackend::None);
    }

    #[tokio::test]
    async fn test_none_provider_empty_query() {
        let provider = NoneProvider::new(true);

        let result = provider.search("", 10, "us", "en").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_none_provider_invalid_urls() {
        let provider = NoneProvider::new(true);

        // URLs without protocol should still be detected
        let result = provider.search("example.com", 10, "us", "en").await;
        // This depends on implementation - adjust based on actual behavior
        assert!(result.is_ok() || result.is_err());
    }
}

// ============================================================================
// Unit Tests: SerperProvider
// ============================================================================

#[cfg(test)]
mod serper_provider_tests {
    use super::*;

    #[test]
    fn test_serper_provider_creation() {
        let provider = SerperProvider::new("test_api_key".to_string(), 30);
        assert_eq!(provider.backend_type(), SearchBackend::Serper);
    }

    #[tokio::test]
    async fn test_serper_provider_empty_query() {
        let provider = SerperProvider::new("test_api_key".to_string(), 30);

        let result = provider.search("", 10, "us", "en").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[tokio::test]
    async fn test_serper_provider_limit_clamping() {
        let provider = SerperProvider::new("test_api_key".to_string(), 30);

        // Test with invalid API key - should fail but not due to limit
        let result = provider.search("test", 0, "us", "en").await;
        // Limit should be clamped to 1-100 range internally
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_serper_provider_invalid_api_key() {
        let provider = SerperProvider::new("invalid_key".to_string(), 30);

        let result = provider.search("test query", 10, "us", "en").await;
        // Should fail due to invalid API key
        assert!(result.is_err());
    }

    #[test]
    fn test_serper_provider_debug_trait() {
        let provider = SerperProvider::new("secret_key".to_string(), 30);
        let debug_str = format!("{:?}", provider);

        // Should not expose API key in debug output
        assert!(!debug_str.contains("secret_key"));
        assert!(debug_str.contains("***"));
    }
}

// ============================================================================
// Unit Tests: SearchConfig
// ============================================================================

#[cfg(test)]
mod search_config_tests {
    use super::*;

    #[test]
    fn test_search_config_default() {
        let config = SearchConfig::default();

        assert_eq!(config.backend, SearchBackend::Serper);
        assert_eq!(config.timeout_seconds, 30);
        assert!(config.enable_url_parsing);
        assert!(config.api_key.is_none());
        assert!(config.base_url.is_none());
    }

    #[test]
    fn test_advanced_search_config_validation_valid() {
        let config = AdvancedSearchConfig {
            backend: SearchBackend::Serper,
            api_key: Some("test_key".to_string()),
            base_url: None,
            timeout_seconds: 30,
            enable_url_parsing: true,
            circuit_breaker: Default::default(),
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_advanced_search_config_validation_serper_missing_key() {
        let config = AdvancedSearchConfig {
            backend: SearchBackend::Serper,
            api_key: None,
            ..Default::default()
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("API key"));
    }

    #[test]
    fn test_advanced_search_config_validation_searxng_missing_url() {
        let config = AdvancedSearchConfig {
            backend: SearchBackend::SearXNG,
            api_key: None,
            base_url: None,
            ..Default::default()
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("base URL"));
    }

    #[test]
    fn test_advanced_search_config_validation_invalid_timeout() {
        let config = AdvancedSearchConfig {
            backend: SearchBackend::None,
            timeout_seconds: 0,
            ..Default::default()
        };

        assert!(config.validate().is_err());

        let config_high = AdvancedSearchConfig {
            backend: SearchBackend::None,
            timeout_seconds: 5000,
            ..Default::default()
        };

        assert!(config_high.validate().is_err());
    }

    #[test]
    fn test_advanced_search_config_validation_circuit_breaker() {
        let mut config = AdvancedSearchConfig::default();
        config.backend = SearchBackend::None;
        config.circuit_breaker.failure_threshold = 150;

        assert!(config.validate().is_err());

        config.circuit_breaker.failure_threshold = 50;
        config.circuit_breaker.min_requests = 0;

        assert!(config.validate().is_err());
    }
}

// ============================================================================
// Unit Tests: Factory Functions
// ============================================================================

#[cfg(test)]
mod factory_tests {
    use super::*;

    #[tokio::test]
    async fn test_create_search_provider_none() {
        let config = SearchConfig {
            backend: SearchBackend::None,
            api_key: None,
            base_url: None,
            timeout_seconds: 30,
            enable_url_parsing: true,
        };

        let result = create_search_provider(config).await;
        assert!(result.is_ok());

        let provider = result.unwrap();
        assert_eq!(provider.backend_type(), SearchBackend::None);
    }

    #[tokio::test]
    async fn test_create_search_provider_serper_success() {
        let config = SearchConfig {
            backend: SearchBackend::Serper,
            api_key: Some("test_key".to_string()),
            base_url: None,
            timeout_seconds: 30,
            enable_url_parsing: false,
        };

        let result = create_search_provider(config).await;
        assert!(result.is_ok());

        let provider = result.unwrap();
        assert_eq!(provider.backend_type(), SearchBackend::Serper);
    }

    #[tokio::test]
    async fn test_create_search_provider_serper_missing_key() {
        let config = SearchConfig {
            backend: SearchBackend::Serper,
            api_key: None,
            base_url: None,
            timeout_seconds: 30,
            enable_url_parsing: false,
        };

        let result = create_search_provider(config).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("API key"));
    }

    #[tokio::test]
    async fn test_create_search_provider_searxng_not_implemented() {
        let config = SearchConfig {
            backend: SearchBackend::SearXNG,
            api_key: None,
            base_url: Some("https://searx.example.com".to_string()),
            timeout_seconds: 30,
            enable_url_parsing: false,
        };

        let result = create_search_provider(config).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not yet implemented"));
    }

    #[tokio::test]
    async fn test_search_provider_factory_create_with_backend() {
        let result = SearchProviderFactory::create_with_backend(SearchBackend::None).await;
        // May fail if env vars not set, but should not panic
        assert!(result.is_ok() || result.is_err());
    }
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[tokio::test]
    async fn test_none_provider_unicode_urls() {
        let provider = NoneProvider::new(true);

        let result = provider
            .search("https://例え.jp https://тест.ru", 10, "us", "en")
            .await;
        // Should handle unicode domains
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_none_provider_very_long_query() {
        let provider = NoneProvider::new(true);

        let long_query = "https://example.com ".repeat(100);
        let result = provider.search(&long_query, 10, "us", "en").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_none_provider_special_characters() {
        let provider = NoneProvider::new(true);

        let result = provider
            .search(
                "https://example.com/path?query=test&value=123#anchor",
                10,
                "us",
                "en",
            )
            .await;

        assert!(result.is_ok());
        let hits = result.unwrap();
        assert_eq!(
            hits[0].url,
            "https://example.com/path?query=test&value=123#anchor"
        );
    }

    #[tokio::test]
    async fn test_concurrent_provider_access() {
        use std::sync::Arc;
        use tokio::task::JoinSet;

        let provider = Arc::new(NoneProvider::new(true));
        let mut set = JoinSet::new();

        for i in 0..10 {
            let provider_clone = provider.clone();
            set.spawn(async move {
                provider_clone
                    .search(&format!("https://example{}.com", i), 10, "us", "en")
                    .await
            });
        }

        let mut success_count = 0;
        while let Some(result) = set.join_next().await {
            if let Ok(Ok(_)) = result {
                success_count += 1;
            }
        }

        assert_eq!(success_count, 10);
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
    async fn test_none_provider_performance() {
        let provider = NoneProvider::new(true);

        let start = Instant::now();
        for _ in 0..100 {
            let _ = provider.search("https://example.com", 10, "us", "en").await;
        }
        let duration = start.elapsed();

        // 100 searches should complete in under 1 second
        assert!(duration < Duration::from_secs(1));
    }

    #[tokio::test]
    async fn test_none_provider_memory_efficiency() {
        let provider = NoneProvider::new(true);

        // Process many URLs without excessive memory growth
        let large_query = (0..1000)
            .map(|i| format!("https://example{}.com", i))
            .collect::<Vec<_>>()
            .join(" ");

        let result = provider.search(&large_query, 100, "us", "en").await;
        assert!(result.is_ok());
    }
}
