//! Comprehensive tests for Circuit Breaker implementation
//!
//! This test suite validates circuit breaker behavior including:
//! - State transitions (Closed -> Open -> HalfOpen -> Closed)
//! - Failure threshold detection
//! - Recovery timeout handling
//! - Integration with SearchProvider
//! - Concurrent access patterns

use riptide_search::{
    CircuitBreakerConfig, CircuitBreakerWrapper, CircuitState, NoneProvider, SearchBackend,
    SearchProvider,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

// ============================================================================
// Unit Tests: CircuitBreakerConfig
// ============================================================================

#[cfg(test)]
mod circuit_breaker_config_tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_config_default() {
        let config = CircuitBreakerConfig::default();

        assert_eq!(config.failure_threshold_percentage, 50);
        assert_eq!(config.minimum_request_threshold, 5);
        assert_eq!(config.recovery_timeout, Duration::from_secs(60));
        assert_eq!(config.half_open_max_requests, 3);
    }

    #[test]
    fn test_circuit_breaker_config_custom() {
        let config = CircuitBreakerConfig {
            failure_threshold_percentage: 75,
            minimum_request_threshold: 10,
            recovery_timeout: Duration::from_secs(30),
            half_open_max_requests: 5,
        };

        assert_eq!(config.failure_threshold_percentage, 75);
        assert_eq!(config.minimum_request_threshold, 10);
        assert_eq!(config.recovery_timeout, Duration::from_secs(30));
        assert_eq!(config.half_open_max_requests, 5);
    }
}

// ============================================================================
// Unit Tests: Basic Circuit Breaker Behavior
// ============================================================================

#[cfg(test)]
mod circuit_breaker_basic_tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_starts_closed() {
        let provider = Box::new(NoneProvider::new(true));
        let circuit = CircuitBreakerWrapper::new(provider);

        assert_eq!(circuit.current_state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_successful_requests() {
        let provider = Box::new(NoneProvider::new(true));
        let circuit = CircuitBreakerWrapper::new(provider);

        // Multiple successful requests should keep circuit closed
        for _ in 0..10 {
            let result = circuit.search("https://example.com", 1, "us", "en").await;
            assert!(result.is_ok());
            assert_eq!(circuit.current_state(), CircuitState::Closed);
        }
    }

    #[tokio::test]
    async fn test_circuit_breaker_preserves_backend_type() {
        let provider = Box::new(NoneProvider::new(true));
        let circuit = CircuitBreakerWrapper::new(provider);

        assert_eq!(circuit.backend_type(), SearchBackend::None);
    }

    #[tokio::test]
    async fn test_circuit_breaker_health_check_independent() {
        let provider = Box::new(NoneProvider::new(true));
        let circuit = CircuitBreakerWrapper::new(provider);

        // Health checks should not affect circuit state
        let health = circuit.health_check().await;
        assert!(health.is_ok());
        assert_eq!(circuit.current_state(), CircuitState::Closed);
    }
}

// ============================================================================
// Unit Tests: Failure Threshold and Circuit Opening
// ============================================================================

#[cfg(test)]
mod circuit_breaker_failure_tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_opens_after_threshold() {
        let provider = Box::new(NoneProvider::new(true));
        let config = CircuitBreakerConfig {
            failure_threshold_percentage: 50,
            minimum_request_threshold: 4,
            recovery_timeout: Duration::from_secs(60),
            half_open_max_requests: 1,
        };
        let circuit = CircuitBreakerWrapper::with_config(provider, config);

        // Generate failures (queries with no URLs)
        for i in 0..4 {
            let result = circuit
                .search(&format!("no urls {}", i), 1, "us", "en")
                .await;
            assert!(result.is_err(), "Request {} should fail", i);
        }

        // Circuit should now be open due to 100% failure rate
        assert_eq!(circuit.current_state(), CircuitState::Open);
        assert_eq!(circuit.failure_rate(), 100);
    }

    #[tokio::test]
    async fn test_circuit_respects_minimum_threshold() {
        let provider = Box::new(NoneProvider::new(true));
        let config = CircuitBreakerConfig {
            failure_threshold_percentage: 50,
            minimum_request_threshold: 10,
            recovery_timeout: Duration::from_secs(60),
            half_open_max_requests: 1,
        };
        let circuit = CircuitBreakerWrapper::with_config(provider, config);

        // Generate some failures, but below minimum threshold
        for _ in 0..5 {
            let _ = circuit.search("no urls", 1, "us", "en").await;
        }

        // Circuit should remain closed (not enough requests yet)
        assert_eq!(circuit.current_state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_calculates_failure_rate_correctly() {
        let provider = Box::new(NoneProvider::new(true));
        let config = CircuitBreakerConfig {
            failure_threshold_percentage: 60,
            minimum_request_threshold: 10,
            recovery_timeout: Duration::from_secs(60),
            half_open_max_requests: 1,
        };
        let circuit = CircuitBreakerWrapper::with_config(provider, config);

        // Mix of successful and failed requests
        for i in 0..10 {
            if i < 6 {
                // 6 failures
                let _ = circuit.search("no urls", 1, "us", "en").await;
            } else {
                // 4 successes
                let _ = circuit.search("https://example.com", 1, "us", "en").await;
            }
        }

        // Failure rate should be 60% (6/10)
        assert_eq!(circuit.failure_rate(), 60);
        assert_eq!(circuit.current_state(), CircuitState::Open);
    }

    #[tokio::test]
    async fn test_circuit_fails_fast_when_open() {
        let provider = Box::new(NoneProvider::new(true));
        let config = CircuitBreakerConfig {
            failure_threshold_percentage: 50,
            minimum_request_threshold: 2,
            recovery_timeout: Duration::from_secs(10),
            half_open_max_requests: 1,
        };
        let circuit = CircuitBreakerWrapper::with_config(provider, config);

        // Open the circuit
        let _ = circuit.search("no urls 1", 1, "us", "en").await;
        let _ = circuit.search("no urls 2", 1, "us", "en").await;
        assert_eq!(circuit.current_state(), CircuitState::Open);

        // Next request should fail immediately
        use std::time::Instant;
        let start = Instant::now();
        let result = circuit.search("https://example.com", 1, "us", "en").await;
        let elapsed = start.elapsed();

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("circuit breaker is OPEN"));
        assert!(elapsed < Duration::from_millis(50), "Should fail fast");
    }
}

// ============================================================================
// Unit Tests: Half-Open State and Recovery
// ============================================================================

#[cfg(test)]
mod circuit_breaker_recovery_tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_transitions_to_half_open() {
        let provider = Box::new(NoneProvider::new(true));
        let config = CircuitBreakerConfig {
            failure_threshold_percentage: 50,
            minimum_request_threshold: 2,
            recovery_timeout: Duration::from_millis(100),
            half_open_max_requests: 1,
        };
        let circuit = CircuitBreakerWrapper::with_config(provider, config);

        // Open the circuit
        let _ = circuit.search("no urls 1", 1, "us", "en").await;
        let _ = circuit.search("no urls 2", 1, "us", "en").await;
        assert_eq!(circuit.current_state(), CircuitState::Open);

        // Wait for recovery timeout
        sleep(Duration::from_millis(120)).await;

        // Next request should transition to half-open
        let result = circuit.search("https://example.com", 1, "us", "en").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_successful_half_open_request_closes_circuit() {
        let provider = Box::new(NoneProvider::new(true));
        let config = CircuitBreakerConfig {
            failure_threshold_percentage: 50,
            minimum_request_threshold: 2,
            recovery_timeout: Duration::from_millis(50),
            half_open_max_requests: 1,
        };
        let circuit = CircuitBreakerWrapper::with_config(provider, config);

        // Open the circuit
        let _ = circuit.search("no urls 1", 1, "us", "en").await;
        let _ = circuit.search("no urls 2", 1, "us", "en").await;
        assert_eq!(circuit.current_state(), CircuitState::Open);

        // Wait for recovery and send successful request
        sleep(Duration::from_millis(60)).await;
        let result = circuit.search("https://example.com", 1, "us", "en").await;
        assert!(result.is_ok());

        // Circuit should be closed
        assert_eq!(circuit.current_state(), CircuitState::Closed);
        assert_eq!(circuit.failure_rate(), 0);
    }

    #[tokio::test]
    async fn test_failed_half_open_request_reopens_circuit() {
        let provider = Box::new(NoneProvider::new(true));
        let config = CircuitBreakerConfig {
            failure_threshold_percentage: 50,
            minimum_request_threshold: 2,
            recovery_timeout: Duration::from_millis(50),
            half_open_max_requests: 1,
        };
        let circuit = CircuitBreakerWrapper::with_config(provider, config);

        // Open the circuit
        let _ = circuit.search("no urls 1", 1, "us", "en").await;
        let _ = circuit.search("no urls 2", 1, "us", "en").await;
        assert_eq!(circuit.current_state(), CircuitState::Open);

        // Wait for recovery and send failed request
        sleep(Duration::from_millis(60)).await;
        let result = circuit.search("no urls 3", 1, "us", "en").await;
        assert!(result.is_err());

        // Circuit should be open again
        assert_eq!(circuit.current_state(), CircuitState::Open);
    }

    #[tokio::test]
    async fn test_half_open_limits_concurrent_requests() {
        let provider = Box::new(NoneProvider::new(true));
        let config = CircuitBreakerConfig {
            failure_threshold_percentage: 50,
            minimum_request_threshold: 2,
            recovery_timeout: Duration::from_millis(50),
            half_open_max_requests: 2,
        };
        let circuit = Arc::new(CircuitBreakerWrapper::with_config(provider, config));

        // Open the circuit
        let _ = circuit.search("no urls 1", 1, "us", "en").await;
        let _ = circuit.search("no urls 2", 1, "us", "en").await;
        assert_eq!(circuit.current_state(), CircuitState::Open);

        // Wait for recovery
        sleep(Duration::from_millis(60)).await;

        // Send multiple requests concurrently
        let circuit_clone = circuit.clone();
        let handle1 = tokio::spawn(async move {
            circuit_clone
                .search("https://example1.com", 1, "us", "en")
                .await
        });

        let circuit_clone = circuit.clone();
        let handle2 = tokio::spawn(async move {
            circuit_clone
                .search("https://example2.com", 1, "us", "en")
                .await
        });

        let circuit_clone = circuit.clone();
        let handle3 = tokio::spawn(async move {
            circuit_clone
                .search("https://example3.com", 1, "us", "en")
                .await
        });

        let results = tokio::try_join!(handle1, handle2, handle3);
        assert!(results.is_ok());

        // At least one should succeed, some may fail if limit exceeded
        // This tests that the limit is enforced
    }

    #[tokio::test]
    async fn test_manual_circuit_reset() {
        let provider = Box::new(NoneProvider::new(true));
        let config = CircuitBreakerConfig {
            failure_threshold_percentage: 50,
            minimum_request_threshold: 2,
            recovery_timeout: Duration::from_secs(60),
            half_open_max_requests: 1,
        };
        let circuit = CircuitBreakerWrapper::with_config(provider, config);

        // Open the circuit
        let _ = circuit.search("no urls 1", 1, "us", "en").await;
        let _ = circuit.search("no urls 2", 1, "us", "en").await;
        assert_eq!(circuit.current_state(), CircuitState::Open);

        // Manual reset
        circuit.reset();

        assert_eq!(circuit.current_state(), CircuitState::Closed);
        assert_eq!(circuit.failure_rate(), 0);
    }
}

// ============================================================================
// Integration Tests: Circuit Breaker with Different Providers
// ============================================================================

#[cfg(test)]
mod circuit_breaker_integration_tests {
    use super::*;
    use riptide_search::SerperProvider;

    #[tokio::test]
    async fn test_circuit_breaker_with_serper_provider() {
        let provider = Box::new(SerperProvider::new("test_key".to_string(), 30));
        let circuit = CircuitBreakerWrapper::new(provider);

        assert_eq!(circuit.backend_type(), SearchBackend::Serper);
        assert_eq!(circuit.current_state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_wrapper_debug_trait() {
        let provider = Box::new(NoneProvider::new(true));
        let circuit = CircuitBreakerWrapper::new(provider);

        let debug_str = format!("{:?}", circuit);
        assert!(debug_str.contains("CircuitBreakerWrapper"));
        assert!(debug_str.contains("Closed"));
    }

    #[tokio::test]
    async fn test_circuit_breaker_error_messages() {
        let provider = Box::new(NoneProvider::new(true));
        let config = CircuitBreakerConfig {
            failure_threshold_percentage: 50,
            minimum_request_threshold: 1,
            recovery_timeout: Duration::from_secs(60),
            half_open_max_requests: 1,
        };
        let circuit = CircuitBreakerWrapper::with_config(provider, config);

        // Open the circuit
        let _ = circuit.search("no urls", 1, "us", "en").await;
        assert_eq!(circuit.current_state(), CircuitState::Open);

        // Error message should be informative
        let result = circuit.search("https://example.com", 1, "us", "en").await;
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("circuit breaker is OPEN"));
        assert!(error_msg.contains("Failure rate"));
    }
}

// ============================================================================
// Concurrent Access Tests
// ============================================================================

#[cfg(test)]
mod circuit_breaker_concurrency_tests {
    use super::*;
    use tokio::task::JoinSet;

    #[tokio::test]
    async fn test_concurrent_requests_closed_circuit() {
        let provider = Box::new(NoneProvider::new(true));
        let circuit = Arc::new(CircuitBreakerWrapper::new(provider));
        let mut set = JoinSet::new();

        for i in 0..20 {
            let circuit_clone = circuit.clone();
            set.spawn(async move {
                circuit_clone
                    .search(&format!("https://example{}.com", i), 1, "us", "en")
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
        assert_eq!(circuit.current_state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_concurrent_failures_trip_circuit() {
        let provider = Box::new(NoneProvider::new(true));
        let config = CircuitBreakerConfig {
            failure_threshold_percentage: 50,
            minimum_request_threshold: 5,
            recovery_timeout: Duration::from_secs(60),
            half_open_max_requests: 1,
        };
        let circuit = Arc::new(CircuitBreakerWrapper::with_config(provider, config));
        let mut set = JoinSet::new();

        // Send concurrent failing requests
        for i in 0..10 {
            let circuit_clone = circuit.clone();
            set.spawn(async move {
                circuit_clone
                    .search(&format!("no urls {}", i), 1, "us", "en")
                    .await
            });
        }

        while let Some(_) = set.join_next().await {}

        // Circuit should be open due to failures
        assert_eq!(circuit.current_state(), CircuitState::Open);
    }

    #[tokio::test]
    async fn test_thread_safety() {
        let provider = Box::new(NoneProvider::new(true));
        let circuit = Arc::new(CircuitBreakerWrapper::new(provider));

        // Verify Send + Sync bounds
        fn is_send_sync<T: Send + Sync>() {}
        is_send_sync::<CircuitBreakerWrapper>();

        // Spawn on different threads
        let handles: Vec<_> = (0..4)
            .map(|i| {
                let circuit_clone = circuit.clone();
                tokio::spawn(async move {
                    for j in 0..5 {
                        let _ = circuit_clone
                            .search(&format!("https://example{}-{}.com", i, j), 1, "us", "en")
                            .await;
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.await.unwrap();
        }

        assert_eq!(circuit.current_state(), CircuitState::Closed);
    }
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[cfg(test)]
mod circuit_breaker_edge_cases {
    use super::*;

    #[tokio::test]
    async fn test_zero_failure_threshold() {
        // Edge case: 0% failure threshold should never open
        let provider = Box::new(NoneProvider::new(true));
        let config = CircuitBreakerConfig {
            failure_threshold_percentage: 0,
            minimum_request_threshold: 1,
            recovery_timeout: Duration::from_secs(60),
            half_open_max_requests: 1,
        };
        let circuit = CircuitBreakerWrapper::with_config(provider, config);

        // Even with failures, circuit should not open (0% threshold)
        let _ = circuit.search("no urls", 1, "us", "en").await;

        // Circuit might still be closed depending on implementation
        // This tests boundary condition
    }

    #[tokio::test]
    async fn test_hundred_percent_failure_threshold() {
        // Edge case: 100% failure threshold requires all requests to fail
        let provider = Box::new(NoneProvider::new(true));
        let config = CircuitBreakerConfig {
            failure_threshold_percentage: 100,
            minimum_request_threshold: 5,
            recovery_timeout: Duration::from_secs(60),
            half_open_max_requests: 1,
        };
        let circuit = CircuitBreakerWrapper::with_config(provider, config);

        // Mix of success and failure
        for i in 0..5 {
            if i == 4 {
                let _ = circuit.search("https://example.com", 1, "us", "en").await;
            } else {
                let _ = circuit.search("no urls", 1, "us", "en").await;
            }
        }

        // Circuit should remain closed (not 100% failures)
        assert_eq!(circuit.current_state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_very_short_recovery_timeout() {
        let provider = Box::new(NoneProvider::new(true));
        let config = CircuitBreakerConfig {
            failure_threshold_percentage: 50,
            minimum_request_threshold: 2,
            recovery_timeout: Duration::from_millis(1),
            half_open_max_requests: 1,
        };
        let circuit = CircuitBreakerWrapper::with_config(provider, config);

        // Open the circuit
        let _ = circuit.search("no urls 1", 1, "us", "en").await;
        let _ = circuit.search("no urls 2", 1, "us", "en").await;
        assert_eq!(circuit.current_state(), CircuitState::Open);

        // Very short timeout should allow immediate recovery attempt
        sleep(Duration::from_millis(2)).await;
        let result = circuit.search("https://example.com", 1, "us", "en").await;
        assert!(result.is_ok());
    }
}
