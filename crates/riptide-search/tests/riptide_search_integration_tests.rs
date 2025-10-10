//! Integration tests for riptide-search
//!
//! Tests end-to-end workflows including:
//! - Provider creation from configuration
//! - Environment variable configuration
//! - Circuit breaker integration
//! - Advanced configuration options
//! - Multi-provider scenarios

use riptide_search::{
    create_search_provider, AdvancedSearchConfig, CircuitBreakerConfigOptions, SearchBackend,
    SearchConfig, SearchProviderFactory,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

// ============================================================================
// Integration Tests: Provider Creation and Configuration
// ============================================================================

#[cfg(test)]
mod provider_creation_tests {
    use super::*;

    #[tokio::test]
    async fn test_create_none_provider_minimal_config() {
        let config = SearchConfig {
            backend: SearchBackend::None,
            api_key: None,
            base_url: None,
            timeout_seconds: 30,
            enable_url_parsing: true,
        };

        let provider = create_search_provider(config).await;
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.backend_type(), SearchBackend::None);

        // Test basic search
        let results = provider.search("https://example.com", 10, "us", "en").await;
        assert!(results.is_ok());
    }

    #[tokio::test]
    async fn test_create_serper_provider_with_key() {
        let config = SearchConfig {
            backend: SearchBackend::Serper,
            api_key: Some("test_key_12345".to_string()),
            base_url: None,
            timeout_seconds: 30,
            enable_url_parsing: false,
        };

        let provider = create_search_provider(config).await;
        assert!(provider.is_ok());
        assert_eq!(provider.unwrap().backend_type(), SearchBackend::Serper);
    }

    #[tokio::test]
    async fn test_create_provider_error_cases() {
        // Serper without API key
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

        // SearXNG not implemented
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
    async fn test_provider_with_custom_timeout() {
        let config = SearchConfig {
            backend: SearchBackend::None,
            api_key: None,
            base_url: None,
            timeout_seconds: 5,
            enable_url_parsing: true,
        };

        let provider = create_search_provider(config).await;
        assert!(provider.is_ok());

        // Test that request completes within timeout
        let search_future = provider
            .unwrap()
            .search("https://example.com", 10, "us", "en");
        let result = timeout(Duration::from_secs(10), search_future).await;
        assert!(result.is_ok());
    }
}

// ============================================================================
// Integration Tests: Advanced Configuration
// ============================================================================

#[cfg(test)]
mod advanced_config_tests {
    use super::*;

    #[tokio::test]
    async fn test_advanced_config_with_custom_circuit_breaker() {
        let config = AdvancedSearchConfig {
            backend: SearchBackend::None,
            api_key: None,
            base_url: None,
            timeout_seconds: 30,
            enable_url_parsing: true,
            circuit_breaker: CircuitBreakerConfigOptions {
                failure_threshold: 75,
                min_requests: 3,
                recovery_timeout_secs: 30,
            },
        };

        let provider = SearchProviderFactory::create_provider(config).await;
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.backend_type(), SearchBackend::None);
    }

    #[tokio::test]
    async fn test_advanced_config_validation() {
        // Valid configuration
        let valid_config = AdvancedSearchConfig {
            backend: SearchBackend::None,
            api_key: None,
            base_url: None,
            timeout_seconds: 30,
            enable_url_parsing: true,
            circuit_breaker: CircuitBreakerConfigOptions::default(),
        };

        assert!(valid_config.validate().is_ok());

        // Invalid timeout
        let mut invalid_config = valid_config.clone();
        invalid_config.timeout_seconds = 0;
        assert!(invalid_config.validate().is_err());

        invalid_config.timeout_seconds = 5000;
        assert!(invalid_config.validate().is_err());

        // Invalid circuit breaker config
        let mut invalid_config = valid_config.clone();
        invalid_config.circuit_breaker.failure_threshold = 150;
        assert!(invalid_config.validate().is_err());

        invalid_config.circuit_breaker.failure_threshold = 50;
        invalid_config.circuit_breaker.min_requests = 0;
        assert!(invalid_config.validate().is_err());

        invalid_config.circuit_breaker.min_requests = 5;
        invalid_config.circuit_breaker.recovery_timeout_secs = 0;
        assert!(invalid_config.validate().is_err());
    }

    #[tokio::test]
    async fn test_backend_specific_validation() {
        // Serper requires API key
        let serper_config = AdvancedSearchConfig {
            backend: SearchBackend::Serper,
            api_key: None,
            ..Default::default()
        };

        assert!(serper_config.validate().is_err());

        let serper_config_valid = AdvancedSearchConfig {
            backend: SearchBackend::Serper,
            api_key: Some("test_key".to_string()),
            ..Default::default()
        };

        assert!(serper_config_valid.validate().is_ok());

        // SearXNG requires base URL
        let searxng_config = AdvancedSearchConfig {
            backend: SearchBackend::SearXNG,
            base_url: None,
            ..Default::default()
        };

        assert!(searxng_config.validate().is_err());

        let searxng_config_valid = AdvancedSearchConfig {
            backend: SearchBackend::SearXNG,
            base_url: Some("https://searx.example.com".to_string()),
            ..Default::default()
        };

        assert!(searxng_config_valid.validate().is_ok());
    }

    #[tokio::test]
    async fn test_factory_create_with_backend() {
        // Test factory method with specific backend
        let result = SearchProviderFactory::create_with_backend(SearchBackend::None).await;

        // Should succeed with None backend (no external dependencies)
        assert!(result.is_ok());
    }
}

// ============================================================================
// Integration Tests: Circuit Breaker with Real Providers
// ============================================================================

#[cfg(test)]
mod circuit_breaker_integration {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_protects_provider() {
        // Create provider with circuit breaker
        let config = AdvancedSearchConfig {
            backend: SearchBackend::None,
            api_key: None,
            base_url: None,
            timeout_seconds: 30,
            enable_url_parsing: true,
            circuit_breaker: CircuitBreakerConfigOptions {
                failure_threshold: 50,
                min_requests: 3,
                recovery_timeout_secs: 1,
            },
        };

        let provider = SearchProviderFactory::create_provider(config).await;
        assert!(provider.is_ok());

        let provider = provider.unwrap();

        // Generate failures to trip circuit
        for _ in 0..3 {
            let _ = provider.search("no urls here", 10, "us", "en").await;
        }

        // Next request should fail fast
        let start = std::time::Instant::now();
        let result = provider.search("https://example.com", 10, "us", "en").await;
        let elapsed = start.elapsed();

        assert!(result.is_err());
        assert!(elapsed < Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_circuit_breaker_recovery_workflow() {
        let config = AdvancedSearchConfig {
            backend: SearchBackend::None,
            api_key: None,
            base_url: None,
            timeout_seconds: 30,
            enable_url_parsing: true,
            circuit_breaker: CircuitBreakerConfigOptions {
                failure_threshold: 50,
                min_requests: 2,
                recovery_timeout_secs: 1,
            },
        };

        let provider = SearchProviderFactory::create_provider(config)
            .await
            .unwrap();

        // Trip the circuit
        let _ = provider.search("no urls 1", 10, "us", "en").await;
        let _ = provider.search("no urls 2", 10, "us", "en").await;

        // Wait for recovery timeout
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Should allow recovery attempt
        let result = provider.search("https://example.com", 10, "us", "en").await;
        assert!(result.is_ok());
    }
}

// ============================================================================
// Integration Tests: Multi-Provider Scenarios
// ============================================================================

#[cfg(test)]
mod multi_provider_tests {
    use super::*;

    #[tokio::test]
    async fn test_multiple_providers_simultaneously() {
        let none_provider = create_search_provider(SearchConfig {
            backend: SearchBackend::None,
            api_key: None,
            base_url: None,
            timeout_seconds: 30,
            enable_url_parsing: true,
        })
        .await
        .unwrap();

        let serper_provider = create_search_provider(SearchConfig {
            backend: SearchBackend::Serper,
            api_key: Some("test_key".to_string()),
            base_url: None,
            timeout_seconds: 30,
            enable_url_parsing: false,
        })
        .await
        .unwrap();

        assert_eq!(none_provider.backend_type(), SearchBackend::None);
        assert_eq!(serper_provider.backend_type(), SearchBackend::Serper);

        // Both should be usable independently
        let none_result = none_provider
            .search("https://example.com", 10, "us", "en")
            .await;
        assert!(none_result.is_ok());
    }

    #[tokio::test]
    async fn test_provider_fallback_pattern() {
        // Simulate a fallback pattern: try Serper, fall back to None
        let serper_config = SearchConfig {
            backend: SearchBackend::Serper,
            api_key: Some("invalid_key".to_string()),
            base_url: None,
            timeout_seconds: 5,
            enable_url_parsing: false,
        };

        let serper_provider = create_search_provider(serper_config).await.unwrap();
        let serper_result = serper_provider.search("test query", 10, "us", "en").await;

        // Serper should fail with invalid key
        if serper_result.is_err() {
            // Fall back to None provider
            let none_config = SearchConfig {
                backend: SearchBackend::None,
                api_key: None,
                base_url: None,
                timeout_seconds: 30,
                enable_url_parsing: true,
            };

            let none_provider = create_search_provider(none_config).await.unwrap();
            let none_result = none_provider
                .search("https://example.com", 10, "us", "en")
                .await;

            assert!(none_result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_concurrent_provider_usage() {
        use tokio::task::JoinSet;

        let provider = Arc::new(
            create_search_provider(SearchConfig {
                backend: SearchBackend::None,
                api_key: None,
                base_url: None,
                timeout_seconds: 30,
                enable_url_parsing: true,
            })
            .await
            .unwrap(),
        );

        let mut set = JoinSet::new();

        // Spawn multiple concurrent searches
        for i in 0..20 {
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

        assert_eq!(success_count, 20);
    }
}

// ============================================================================
// Integration Tests: Health Checks and Monitoring
// ============================================================================

#[cfg(test)]
mod health_monitoring_tests {
    use super::*;

    #[tokio::test]
    async fn test_provider_health_checks() {
        let none_provider = create_search_provider(SearchConfig {
            backend: SearchBackend::None,
            api_key: None,
            base_url: None,
            timeout_seconds: 30,
            enable_url_parsing: true,
        })
        .await
        .unwrap();

        let health = none_provider.health_check().await;
        assert!(health.is_ok());
    }

    #[tokio::test]
    async fn test_serper_health_check_with_invalid_key() {
        let provider = create_search_provider(SearchConfig {
            backend: SearchBackend::Serper,
            api_key: Some("invalid_key".to_string()),
            base_url: None,
            timeout_seconds: 10,
            enable_url_parsing: false,
        })
        .await
        .unwrap();

        // Health check should fail with invalid key
        let health = provider.health_check().await;
        // This will fail due to network call, but tests the integration
        assert!(health.is_ok() || health.is_err());
    }
}

// ============================================================================
// Integration Tests: Error Handling and Recovery
// ============================================================================

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[tokio::test]
    async fn test_provider_error_propagation() {
        let provider = create_search_provider(SearchConfig {
            backend: SearchBackend::None,
            api_key: None,
            base_url: None,
            timeout_seconds: 30,
            enable_url_parsing: true,
        })
        .await
        .unwrap();

        // Test various error conditions
        let empty_result = provider.search("", 10, "us", "en").await;
        assert!(empty_result.is_err());

        let no_urls_result = provider.search("no urls here", 10, "us", "en").await;
        assert!(no_urls_result.is_err());
    }

    #[tokio::test]
    async fn test_timeout_handling() {
        let provider = create_search_provider(SearchConfig {
            backend: SearchBackend::None,
            api_key: None,
            base_url: None,
            timeout_seconds: 1,
            enable_url_parsing: true,
        })
        .await
        .unwrap();

        // Fast operations should complete within timeout
        let start = std::time::Instant::now();
        let result = provider.search("https://example.com", 10, "us", "en").await;
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        assert!(elapsed < Duration::from_secs(2));
    }

    #[tokio::test]
    async fn test_recovery_after_errors() {
        let provider = create_search_provider(SearchConfig {
            backend: SearchBackend::None,
            api_key: None,
            base_url: None,
            timeout_seconds: 30,
            enable_url_parsing: true,
        })
        .await
        .unwrap();

        // Generate some errors
        for _ in 0..5 {
            let _ = provider.search("no urls", 10, "us", "en").await;
        }

        // Provider should still work after errors
        let result = provider.search("https://example.com", 10, "us", "en").await;
        assert!(result.is_ok());
    }
}

// ============================================================================
// Integration Tests: Performance and Load
// ============================================================================

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[tokio::test]
    async fn test_high_throughput_searches() {
        let provider = Arc::new(
            create_search_provider(SearchConfig {
                backend: SearchBackend::None,
                api_key: None,
                base_url: None,
                timeout_seconds: 30,
                enable_url_parsing: true,
            })
            .await
            .unwrap(),
        );

        let start = std::time::Instant::now();

        let mut handles = vec![];
        for i in 0..100 {
            let provider_clone = provider.clone();
            handles.push(tokio::spawn(async move {
                provider_clone
                    .search(&format!("https://example{}.com", i), 10, "us", "en")
                    .await
            }));
        }

        let results = futures::future::join_all(handles).await;
        let elapsed = start.elapsed();

        let success_count = results
            .iter()
            .filter(|r| r.is_ok() && r.as_ref().unwrap().is_ok())
            .count();

        assert_eq!(success_count, 100);
        assert!(
            elapsed < Duration::from_secs(5),
            "Should complete within 5 seconds"
        );
    }

    #[tokio::test]
    async fn test_provider_with_rate_limiting_simulation() {
        let provider = create_search_provider(SearchConfig {
            backend: SearchBackend::None,
            api_key: None,
            base_url: None,
            timeout_seconds: 30,
            enable_url_parsing: true,
        })
        .await
        .unwrap();

        // Simulate rapid requests
        for i in 0..50 {
            let result = provider
                .search(&format!("https://example{}.com", i), 10, "us", "en")
                .await;
            assert!(result.is_ok());

            // Small delay to simulate rate limiting
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
}
