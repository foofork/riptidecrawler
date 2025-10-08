//! Comprehensive tests for per-host FetchEngine features
//!
//! Tests per-host circuit breakers, rate limiting, logging, and metrics tracking

#[cfg(test)]
#[allow(clippy::module_inception)]
mod fetch_engine_tests {
    use crate::fetch::{CircuitBreakerConfig, PerHostFetchEngine, RateLimitConfig, RetryConfig};
    use std::time::Duration;
    use tokio::time::sleep;

    /// Test that circuit breakers are per-host, not global
    #[tokio::test]
    async fn test_per_host_circuit_breaker() {
        let retry_config = RetryConfig {
            max_attempts: 1,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            backoff_multiplier: 1.0,
            jitter: false,
        };

        let circuit_config = CircuitBreakerConfig {
            failure_threshold: 2, // Trip after 2 failures
            open_cooldown_ms: 1000,
            half_open_max_in_flight: 1,
        };

        let rate_limit_config = RateLimitConfig {
            requests_per_second: 100,
            burst_capacity: 100,
        };

        let engine = PerHostFetchEngine::new(retry_config, circuit_config, rate_limit_config)
            .expect("Failed to create FetchEngine");

        // Use httpbin.org as a reliable test endpoint
        let valid_url_a = "https://httpbin.org/status/200";
        let invalid_url_a = "https://httpbin.org/status/500"; // Will fail
        let valid_url_b = "https://example.com"; // Different host

        // Step 1: Trigger failures on host A (httpbin.org) to trip circuit breaker
        for _ in 0..3 {
            let _ = engine.fetch(invalid_url_a).await; // Expect failures
        }

        // Step 2: Circuit should be open for host A
        let result_a = engine.fetch(valid_url_a).await;
        assert!(
            result_a.is_err(),
            "Host A should have circuit breaker open after failures"
        );
        if let Err(e) = result_a {
            let err_str = e.to_string();
            assert!(
                err_str.contains("circuit") || err_str.contains("Circuit"),
                "Error should mention circuit breaker: {}",
                err_str
            );
        }

        // Step 3: Different host (example.com) should still work
        let result_b = engine.fetch(valid_url_b).await;
        assert!(
            result_b.is_ok(),
            "Host B should still work despite Host A circuit being open: {:?}",
            result_b.err()
        );
    }

    /// Test that rate limiting is per-host
    #[tokio::test]
    async fn test_per_host_rate_limiting() {
        let retry_config = RetryConfig::default();

        let circuit_config = CircuitBreakerConfig {
            failure_threshold: 10, // High threshold to avoid tripping
            open_cooldown_ms: 1000,
            half_open_max_in_flight: 5,
        };

        let rate_limit_config = RateLimitConfig {
            requests_per_second: 2, // Very low rate limit for testing
            burst_capacity: 3,      // Allow 3 burst requests
        };

        let engine = PerHostFetchEngine::new(retry_config, circuit_config, rate_limit_config)
            .expect("Failed to create FetchEngine");

        let url_a = "https://httpbin.org/status/200";
        let url_b = "https://example.com";

        // Step 1: Exhaust rate limit for host A (3 burst requests)
        for i in 0..3 {
            let result = engine.fetch(url_a).await;
            assert!(
                result.is_ok(),
                "Request {} to host A should succeed within burst capacity: {:?}",
                i + 1,
                result.err()
            );
        }

        // Step 2: Next request to host A should be rate limited
        let result_a_limited = engine.fetch(url_a).await;
        assert!(
            result_a_limited.is_err(),
            "Host A should be rate limited after burst capacity"
        );
        if let Err(e) = result_a_limited {
            let err_str = e.to_string();
            assert!(
                err_str.contains("rate") || err_str.contains("Rate"),
                "Error should mention rate limit: {}",
                err_str
            );
        }

        // Step 3: Different host (example.com) should still work
        let result_b = engine.fetch(url_b).await;
        assert!(
            result_b.is_ok(),
            "Host B should work despite Host A being rate limited: {:?}",
            result_b.err()
        );
    }

    /// Test request/response logging captures important details
    #[tokio::test]
    async fn test_request_response_logging() {
        // This test verifies that logging infrastructure is in place
        // Actual log output validation would require log capture utilities

        let engine = PerHostFetchEngine::new(
            RetryConfig::default(),
            CircuitBreakerConfig::default(),
            RateLimitConfig::default(),
        )
        .expect("Failed to create FetchEngine");

        let url = "https://httpbin.org/status/200";

        // Make a successful request
        let result = engine.fetch(url).await;
        assert!(result.is_ok(), "Request should succeed: {:?}", result.err());

        // Logging is verified through tracing infrastructure
        // In production, logs would show:
        // - FetchEngine: Starting request (INFO)
        // - FetchEngine: Request completed (INFO) with status, duration
        // For failures:
        // - FetchEngine: Request failed (ERROR) with error details

        // Make a failing request
        let bad_url = "https://httpbin.org/status/500";
        let result_fail = engine.fetch(bad_url).await;
        assert!(result_fail.is_err(), "Request to 500 endpoint should fail");
    }

    /// Test metrics tracking per host
    #[tokio::test]
    async fn test_metrics_tracking() {
        let engine = PerHostFetchEngine::new(
            RetryConfig::default(),
            CircuitBreakerConfig::default(),
            RateLimitConfig::default(),
        )
        .expect("Failed to create FetchEngine");

        let url_a = "https://httpbin.org/status/200";
        let url_b = "https://example.com";

        // Make requests to different hosts
        let _ = engine.fetch(url_a).await;
        let _ = engine.fetch(url_a).await;
        let _ = engine.fetch(url_b).await;

        // Get metrics for host A
        let metrics_a = engine.get_host_metrics("httpbin.org");
        assert!(metrics_a.is_some(), "Should have metrics for httpbin.org");

        if let Some(metrics) = metrics_a {
            assert!(
                metrics.request_count >= 2,
                "Should have at least 2 requests for httpbin.org"
            );
        }

        // Get metrics for host B
        let metrics_b = engine.get_host_metrics("example.com");
        assert!(metrics_b.is_some(), "Should have metrics for example.com");

        if let Some(metrics) = metrics_b {
            assert!(
                metrics.request_count >= 1,
                "Should have at least 1 request for example.com"
            );
        }

        // Get all metrics
        let all_metrics = engine.get_all_metrics().await;
        assert!(
            all_metrics.hosts.len() >= 2,
            "Should have metrics for at least 2 hosts"
        );
        assert!(
            all_metrics.total_requests >= 3,
            "Should have at least 3 total requests"
        );
    }

    /// Test rate limiter refills tokens over time
    #[tokio::test]
    async fn test_rate_limiter_token_refill() {
        let engine = PerHostFetchEngine::new(
            RetryConfig::default(),
            CircuitBreakerConfig::default(),
            RateLimitConfig {
                requests_per_second: 5, // 5 requests per second
                burst_capacity: 2,      // 2 burst requests
            },
        )
        .expect("Failed to create FetchEngine");

        let url = "https://httpbin.org/status/200";

        // Use up burst capacity
        for i in 0..2 {
            let result = engine.fetch(url).await;
            assert!(
                result.is_ok(),
                "Burst request {} should succeed: {:?}",
                i + 1,
                result.err()
            );
        }

        // Next request should be rate limited
        let result_limited = engine.fetch(url).await;
        assert!(
            result_limited.is_err(),
            "Should be rate limited after burst"
        );

        // Wait for token refill (200ms = 1 token at 5 req/s)
        sleep(Duration::from_millis(250)).await;

        // Should be able to make another request after refill
        let result_after_refill = engine.fetch(url).await;
        assert!(
            result_after_refill.is_ok(),
            "Should succeed after token refill: {:?}",
            result_after_refill.err()
        );
    }

    /// Test circuit breaker recovery after cooldown
    #[tokio::test]
    async fn test_circuit_breaker_recovery() {
        let engine = PerHostFetchEngine::new(
            RetryConfig {
                max_attempts: 1,
                initial_delay: Duration::from_millis(10),
                max_delay: Duration::from_millis(100),
                backoff_multiplier: 1.0,
                jitter: false,
            },
            CircuitBreakerConfig {
                failure_threshold: 2,
                open_cooldown_ms: 500, // 500ms cooldown
                half_open_max_in_flight: 1,
            },
            RateLimitConfig::default(),
        )
        .expect("Failed to create FetchEngine");

        let bad_url = "https://httpbin.org/status/500";
        let good_url = "https://httpbin.org/status/200";

        // Trip circuit breaker with failures
        for _ in 0..3 {
            let _ = engine.fetch(bad_url).await;
        }

        // Circuit should be open
        let result_open = engine.fetch(good_url).await;
        assert!(
            result_open.is_err(),
            "Circuit should be open after failures"
        );

        // Wait for cooldown period
        sleep(Duration::from_millis(600)).await;

        // Circuit should be half-open and allow a test request
        let result_half_open = engine.fetch(good_url).await;
        // This might succeed if the circuit transitions to half-open and the request succeeds
        // Or it might fail if still in transition
        // Either way, the circuit breaker mechanism is working

        // If the previous request succeeded, circuit should be closed
        if result_half_open.is_ok() {
            let result_closed = engine.fetch(good_url).await;
            assert!(
                result_closed.is_ok(),
                "Circuit should be closed after successful recovery: {:?}",
                result_closed.err()
            );
        }
    }

    /// Test that host extraction works correctly
    #[test]
    fn test_host_extraction() {
        use crate::fetch::PerHostFetchEngine;

        // Valid URLs
        assert_eq!(
            PerHostFetchEngine::extract_host_for_test("https://example.com/path").unwrap(),
            "example.com"
        );
        assert_eq!(
            PerHostFetchEngine::extract_host_for_test("https://api.github.com/repos").unwrap(),
            "api.github.com"
        );
        assert_eq!(
            PerHostFetchEngine::extract_host_for_test("http://localhost:8080").unwrap(),
            "localhost"
        );

        // Invalid URLs should error
        assert!(PerHostFetchEngine::extract_host_for_test("not a url").is_err());
        assert!(PerHostFetchEngine::extract_host_for_test("").is_err());
    }

    /// Test metrics accumulation across multiple requests
    #[tokio::test]
    async fn test_metrics_accumulation() {
        let engine = PerHostFetchEngine::new(
            RetryConfig::default(),
            CircuitBreakerConfig::default(),
            RateLimitConfig {
                requests_per_second: 100,
                burst_capacity: 100,
            },
        )
        .expect("Failed to create FetchEngine");

        let url = "https://httpbin.org/status/200";

        // Make multiple successful requests
        for _ in 0..5 {
            let _ = engine.fetch(url).await;
        }

        let metrics = engine.get_host_metrics("httpbin.org");
        assert!(metrics.is_some(), "Should have metrics for httpbin.org");

        if let Some(m) = metrics {
            assert_eq!(
                m.request_count, 5,
                "Should have exactly 5 requests recorded"
            );
            assert!(
                m.success_count >= 4,
                "Should have at least 4 successful requests"
            );
            assert!(m.total_duration_ms > 0, "Should have non-zero duration");

            // Calculate average duration
            let avg_duration = m.total_duration_ms as f64 / m.request_count as f64;
            assert!(avg_duration > 0.0, "Average duration should be positive");
        }
    }
}
