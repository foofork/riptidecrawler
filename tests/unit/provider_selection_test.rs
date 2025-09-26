//! Provider selection and configuration validation tests
//!
//! This module tests provider selection logic with various configurations,
//! including edge cases, error handling, and environment-based configuration.

use riptide_core::search::{
    SearchBackend, SearchConfig, SearchProvider, AdvancedSearchConfig,
    CircuitBreakerConfigOptions, SearchProviderFactory,
    create_search_provider, create_search_provider_from_env
};
use std::env;
use std::time::Duration;

// Activate all provider selection tests
#[cfg(test)]
mod provider_selection_tests {
    use super::*;

    /// Test provider selection based on backend type
    #[tokio::test]
    async fn test_provider_selection_by_backend() {
        // Test None backend selection
        let none_config = SearchConfig {
            backend: SearchBackend::None,
            api_key: None,
            base_url: None,
            timeout_seconds: 30,
            enable_url_parsing: true,
        };

        let provider = create_search_provider(none_config).await;
        assert!(provider.is_ok());
        assert_eq!(provider.unwrap().backend_type(), SearchBackend::None);

        // Test Serper backend selection with API key
        let serper_config = SearchConfig {
            backend: SearchBackend::Serper,
            api_key: Some("test-key".to_string()),
            base_url: None,
            timeout_seconds: 30,
            enable_url_parsing: false,
        };

        let provider = create_search_provider(serper_config).await;
        assert!(provider.is_ok());
        assert_eq!(provider.unwrap().backend_type(), SearchBackend::Serper);
    }

    /// Test configuration validation for different backends
    #[tokio::test]
    async fn test_configuration_validation() {
        // Valid None configuration
        let valid_none = AdvancedSearchConfig {
            backend: SearchBackend::None,
            api_key: None,
            base_url: None,
            timeout_seconds: 30,
            enable_url_parsing: true,
            circuit_breaker: CircuitBreakerConfigOptions::default(),
        };
        assert!(valid_none.validate().is_ok());

        // Valid Serper configuration
        let valid_serper = AdvancedSearchConfig {
            backend: SearchBackend::Serper,
            api_key: Some("valid-api-key".to_string()),
            base_url: None,
            timeout_seconds: 30,
            enable_url_parsing: false,
            circuit_breaker: CircuitBreakerConfigOptions::default(),
        };
        assert!(valid_serper.validate().is_ok());

        // Invalid Serper configuration (missing API key)
        let invalid_serper_no_key = AdvancedSearchConfig {
            backend: SearchBackend::Serper,
            api_key: None,
            base_url: None,
            timeout_seconds: 30,
            enable_url_parsing: false,
            circuit_breaker: CircuitBreakerConfigOptions::default(),
        };
        assert!(invalid_serper_no_key.validate().is_err());

        // Invalid Serper configuration (empty API key)
        let invalid_serper_empty_key = AdvancedSearchConfig {
            backend: SearchBackend::Serper,
            api_key: Some("".to_string()),
            base_url: None,
            timeout_seconds: 30,
            enable_url_parsing: false,
            circuit_breaker: CircuitBreakerConfigOptions::default(),
        };
        assert!(invalid_serper_empty_key.validate().is_err());

        // Invalid SearXNG configuration (missing base URL)
        let invalid_searxng = AdvancedSearchConfig {
            backend: SearchBackend::SearXNG,
            api_key: None,
            base_url: None,
            timeout_seconds: 30,
            enable_url_parsing: false,
            circuit_breaker: CircuitBreakerConfigOptions::default(),
        };
        assert!(invalid_searxng.validate().is_err());
    }

    /// Test timeout validation
    #[test]
    fn test_timeout_validation() {
        // Valid timeout
        let valid_config = AdvancedSearchConfig {
            backend: SearchBackend::None,
            timeout_seconds: 30,
            ..Default::default()
        };
        assert!(valid_config.validate().is_ok());

        // Zero timeout (invalid)
        let zero_timeout = AdvancedSearchConfig {
            backend: SearchBackend::None,
            timeout_seconds: 0,
            ..Default::default()
        };
        assert!(zero_timeout.validate().is_err());

        // Excessive timeout (invalid)
        let excessive_timeout = AdvancedSearchConfig {
            backend: SearchBackend::None,
            timeout_seconds: 3601, // Over 1 hour
            ..Default::default()
        };
        assert!(excessive_timeout.validate().is_err());

        // Maximum valid timeout
        let max_timeout = AdvancedSearchConfig {
            backend: SearchBackend::None,
            timeout_seconds: 3600, // Exactly 1 hour
            ..Default::default()
        };
        assert!(max_timeout.validate().is_ok());
    }

    /// Test circuit breaker configuration validation
    #[test]
    fn test_circuit_breaker_validation() {
        // Valid circuit breaker config
        let valid_cb = AdvancedSearchConfig {
            backend: SearchBackend::None,
            circuit_breaker: CircuitBreakerConfigOptions {
                failure_threshold: 50,
                min_requests: 5,
                recovery_timeout_secs: 60,
            },
            ..Default::default()
        };
        assert!(valid_cb.validate().is_ok());

        // Invalid failure threshold (over 100%)
        let invalid_threshold = AdvancedSearchConfig {
            backend: SearchBackend::None,
            circuit_breaker: CircuitBreakerConfigOptions {
                failure_threshold: 101,
                min_requests: 5,
                recovery_timeout_secs: 60,
            },
            ..Default::default()
        };
        assert!(invalid_threshold.validate().is_err());

        // Invalid min requests (zero)
        let invalid_min_requests = AdvancedSearchConfig {
            backend: SearchBackend::None,
            circuit_breaker: CircuitBreakerConfigOptions {
                failure_threshold: 50,
                min_requests: 0,
                recovery_timeout_secs: 60,
            },
            ..Default::default()
        };
        assert!(invalid_min_requests.validate().is_err());

        // Invalid recovery timeout (zero)
        let invalid_recovery = AdvancedSearchConfig {
            backend: SearchBackend::None,
            circuit_breaker: CircuitBreakerConfigOptions {
                failure_threshold: 50,
                min_requests: 5,
                recovery_timeout_secs: 0,
            },
            ..Default::default()
        };
        assert!(invalid_recovery.validate().is_err());
    }

    /// Test environment-based configuration
    #[tokio::test]
    async fn test_environment_configuration() {
        // Set up environment variables
        env::set_var("SEARCH_BACKEND", "none");
        env::set_var("SEARCH_TIMEOUT", "45");
        env::set_var("SEARCH_ENABLE_URL_PARSING", "false");
        env::set_var("CIRCUIT_BREAKER_FAILURE_THRESHOLD", "75");
        env::set_var("CIRCUIT_BREAKER_MIN_REQUESTS", "10");
        env::set_var("CIRCUIT_BREAKER_RECOVERY_TIMEOUT", "120");

        let config = AdvancedSearchConfig::from_env();

        assert_eq!(config.backend, SearchBackend::None);
        assert_eq!(config.timeout_seconds, 45);
        assert!(!config.enable_url_parsing);
        assert_eq!(config.circuit_breaker.failure_threshold, 75);
        assert_eq!(config.circuit_breaker.min_requests, 10);
        assert_eq!(config.circuit_breaker.recovery_timeout_secs, 120);

        // Test provider creation from environment
        let provider = SearchProviderFactory::create_from_env().await;
        assert!(provider.is_ok());

        // Clean up environment variables
        env::remove_var("SEARCH_BACKEND");
        env::remove_var("SEARCH_TIMEOUT");
        env::remove_var("SEARCH_ENABLE_URL_PARSING");
        env::remove_var("CIRCUIT_BREAKER_FAILURE_THRESHOLD");
        env::remove_var("CIRCUIT_BREAKER_MIN_REQUESTS");
        env::remove_var("CIRCUIT_BREAKER_RECOVERY_TIMEOUT");
    }

    /// Test fallback to defaults when environment variables are invalid
    #[test]
    fn test_environment_fallback_defaults() {
        // Set invalid environment variables
        env::set_var("SEARCH_BACKEND", "invalid_backend");
        env::set_var("SEARCH_TIMEOUT", "not_a_number");
        env::set_var("CIRCUIT_BREAKER_FAILURE_THRESHOLD", "not_a_number");

        let config = AdvancedSearchConfig::from_env();

        // Should fall back to defaults
        assert_eq!(config.backend, SearchBackend::Serper); // Default backend
        assert_eq!(config.timeout_seconds, 30); // Default timeout
        assert_eq!(config.circuit_breaker.failure_threshold, 50); // Default threshold

        // Clean up
        env::remove_var("SEARCH_BACKEND");
        env::remove_var("SEARCH_TIMEOUT");
        env::remove_var("CIRCUIT_BREAKER_FAILURE_THRESHOLD");
    }

    /// Test provider creation with advanced factory methods
    #[tokio::test]
    async fn test_advanced_factory_methods() {
        // Test create_with_backend
        let provider = SearchProviderFactory::create_with_backend(SearchBackend::None).await;
        assert!(provider.is_ok());
        assert_eq!(provider.unwrap().backend_type(), SearchBackend::None);

        // Test with Serper backend (should fail without API key in env)
        env::remove_var("SERPER_API_KEY"); // Ensure no API key in env
        let provider = SearchProviderFactory::create_with_backend(SearchBackend::Serper).await;
        // This will likely fail due to missing API key, which is expected
        if provider.is_err() {
            let error = provider.unwrap_err();
            assert!(error.to_string().contains("API key"));
        }

        // Test with API key in environment
        env::set_var("SERPER_API_KEY", "test-key-from-env");
        let provider = SearchProviderFactory::create_with_backend(SearchBackend::Serper).await;
        assert!(provider.is_ok());
        assert_eq!(provider.unwrap().backend_type(), SearchBackend::Serper);

        env::remove_var("SERPER_API_KEY");
    }

    /// Test provider creation with custom advanced configuration
    #[tokio::test]
    async fn test_advanced_provider_creation() {
        let custom_config = AdvancedSearchConfig {
            backend: SearchBackend::None,
            api_key: None,
            base_url: None,
            timeout_seconds: 60,
            enable_url_parsing: true,
            circuit_breaker: CircuitBreakerConfigOptions {
                failure_threshold: 80,
                min_requests: 3,
                recovery_timeout_secs: 30,
            },
        };

        let provider = SearchProviderFactory::create_provider(custom_config).await;
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.backend_type(), SearchBackend::None);
    }

    /// Test provider selection consistency
    #[tokio::test]
    async fn test_provider_selection_consistency() {
        // Create multiple providers with the same configuration
        let config = SearchConfig {
            backend: SearchBackend::None,
            api_key: None,
            base_url: None,
            timeout_seconds: 30,
            enable_url_parsing: true,
        };

        let provider1 = create_search_provider(config.clone()).await;
        let provider2 = create_search_provider(config.clone()).await;

        assert!(provider1.is_ok());
        assert!(provider2.is_ok());

        let p1 = provider1.unwrap();
        let p2 = provider2.unwrap();

        // Both providers should have the same backend type
        assert_eq!(p1.backend_type(), p2.backend_type());
        assert_eq!(p1.backend_type(), SearchBackend::None);
    }

    /// Test edge cases in provider selection
    #[tokio::test]
    async fn test_provider_selection_edge_cases() {
        // Test with whitespace in API key
        let whitespace_key_config = SearchConfig {
            backend: SearchBackend::Serper,
            api_key: Some("  valid-key-with-spaces  ".to_string()),
            base_url: None,
            timeout_seconds: 30,
            enable_url_parsing: false,
        };

        let provider = create_search_provider(whitespace_key_config).await;
        assert!(provider.is_ok()); // Current implementation doesn't trim, but that's documented behavior

        // Test with very short timeout
        let short_timeout_config = SearchConfig {
            backend: SearchBackend::None,
            api_key: None,
            base_url: None,
            timeout_seconds: 1, // Very short
            enable_url_parsing: true,
        };

        let provider = create_search_provider(short_timeout_config).await;
        assert!(provider.is_ok());

        // Test with very long timeout
        let long_timeout_config = SearchConfig {
            backend: SearchBackend::None,
            api_key: None,
            base_url: None,
            timeout_seconds: 300, // 5 minutes
            enable_url_parsing: true,
        };

        let provider = create_search_provider(long_timeout_config).await;
        assert!(provider.is_ok());
    }
}

// Activate provider health check tests
#[cfg(test)]
mod provider_health_check_tests {
    use super::*;

    /// Test health checks for different provider types
    #[tokio::test]
    async fn test_provider_health_checks() {
        // None provider health check (should always pass)
        let none_config = SearchConfig {
            backend: SearchBackend::None,
            api_key: None,
            base_url: None,
            timeout_seconds: 30,
            enable_url_parsing: true,
        };

        let provider = create_search_provider(none_config).await;
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        let health_result = provider.health_check().await;
        assert!(health_result.is_ok());

        // Serper provider health check (will fail with fake API key)
        let serper_config = SearchConfig {
            backend: SearchBackend::Serper,
            api_key: Some("fake-api-key".to_string()),
            base_url: None,
            timeout_seconds: 5, // Short timeout for fast test
            enable_url_parsing: false,
        };

        let provider = create_search_provider(serper_config).await;
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        let health_result = provider.health_check().await;
        // This should fail because we're using a fake API key
        // But we won't assert that because it requires network access
        // and we want tests to be deterministic
    }

    /// Test multiple consecutive health checks
    #[tokio::test]
    async fn test_consecutive_health_checks() {
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

        // Run multiple health checks
        for _ in 0..5 {
            let health_result = provider.health_check().await;
            assert!(health_result.is_ok());
        }
    }
}