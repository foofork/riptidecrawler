//! Error Recovery Tests
//!
//! Tests system resilience and recovery from various failure scenarios:
//! - Redis connection failure → persistence fallback
//! - Browser crash → pool recovery
//! - Memory exhaustion → graceful degradation
//! - Stream backpressure → proper queuing
//! - Tenant quota exceeded → proper error response

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;
use tower::ServiceExt;

mod test_helpers;

#[cfg(test)]
mod error_recovery_tests {
    use super::*;

    /// Recovery Test 1: Redis connection failure
    /// Verifies graceful degradation when Redis is unavailable
    #[tokio::test]
    #[ignore = "Requires Redis service running - skip in CI"]
    async fn test_redis_connection_failure_recovery() {
        let app = test_helpers::create_minimal_test_app();

        // Attempt cache operation when Redis is unavailable
        let cache_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/cache/set")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "key": "test_key",
                            "value": "test_value"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should either work with fallback or return graceful error
        assert!(
            cache_response.status() == StatusCode::OK
                || cache_response.status() == StatusCode::SERVICE_UNAVAILABLE
        );

        // Verify system is still operational
        let health_response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(health_response.status(), StatusCode::OK);
    }

    /// Recovery Test 2: Browser crash recovery
    /// Tests browser pool recovery after crash
    #[tokio::test]
    #[cfg(feature = "sessions")]
    async fn test_browser_crash_recovery() {
        let app = test_helpers::create_minimal_test_app();

        // Create browser session
        let session_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/sessions")
                    .body(Body::from(
                        json!({"url": "https://example.com"}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(session_response.status(), StatusCode::OK);

        // Simulate browser crash by forcefully closing session
        let crash_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/sessions/test-session/crash")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await;

        // Wait for recovery
        sleep(Duration::from_secs(2)).await;

        // Verify browser pool recovered and can create new session
        let new_session_response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/sessions")
                    .body(Body::from(
                        json!({"url": "https://example.com"}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(new_session_response.status(), StatusCode::OK);
    }

    /// Recovery Test 3: Memory exhaustion graceful degradation
    /// Tests system behavior when memory limits are hit
    #[tokio::test]
    #[cfg(feature = "profiling-full")]
    async fn test_memory_exhaustion_degradation() {
        let app = test_helpers::create_minimal_test_app();

        // Start memory monitoring
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/profiling/start")
                    .body(Body::from(json!({"profile_type": "memory"}).to_string()))
                    .unwrap(),
            )
            .await;

        // Attempt to allocate large amounts of memory
        for i in 0..50 {
            let large_data = "x".repeat(1_000_000); // 1MB per request
            let _ = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/api/v1/cache/set")
                        .body(Body::from(
                            json!({
                                "key": format!("large_data_{}", i),
                                "value": large_data
                            })
                            .to_string(),
                        ))
                        .unwrap(),
                )
                .await;

            sleep(Duration::from_millis(50)).await;
        }

        // Check system status after memory pressure
        let status_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // System should still respond
        assert_eq!(status_response.status(), StatusCode::OK);

        // Check profiling report for memory warnings
        let profile_response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/profiling/report")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(profile_response.status(), StatusCode::OK);
    }

    /// Recovery Test 4: Stream backpressure handling
    /// Tests proper queuing when stream consumer is slow
    #[tokio::test]
    #[cfg(feature = "streaming")]
    async fn test_stream_backpressure_queuing() {
        let app = test_helpers::create_minimal_test_app();

        // Start high-speed stream
        let stream_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/stream/start")
                    .body(Body::from(
                        json!({
                            "source": "fast_producer",
                            "rate_limit": 1000, // items/sec
                            "buffer_size": 100
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(stream_response.status(), StatusCode::OK);

        // Check stream is handling backpressure
        sleep(Duration::from_secs(2)).await;

        let status_response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/stream/test-stream/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should show buffering/queuing status
        assert_eq!(status_response.status(), StatusCode::OK);
    }

    /// Recovery Test 5: Tenant quota exceeded error handling
    /// Tests proper error responses when quotas are exceeded
    #[tokio::test]
    #[ignore = "Requires full app state with Redis - skip in CI"]
    async fn test_tenant_quota_exceeded_errors() {
        let app = test_helpers::create_minimal_test_app();

        // Create tenant with very low limits
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/admin/tenants")
                    .header("X-Admin-Token", "test-token")
                    .body(Body::from(
                        json!({
                            "tenant_id": "limited-tenant",
                            "max_requests_per_minute": 5,
                            "max_cost_per_hour": 0.10
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await;

        // Exceed request quota
        for _ in 0..10 {
            let _ = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/api/v1/health")
                        .header("X-Tenant-ID", "limited-tenant")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await;
        }

        // Next request should return proper error
        let quota_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .header("X-Tenant-ID", "limited-tenant")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(quota_response.status(), StatusCode::TOO_MANY_REQUESTS);

        // Wait for quota reset
        sleep(Duration::from_secs(61)).await;

        // Should work again after reset
        let recovery_response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .header("X-Tenant-ID", "limited-tenant")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(recovery_response.status(), StatusCode::OK);
    }

    /// Recovery Test 6: Network timeout handling
    /// Tests recovery from network timeouts
    #[tokio::test]
    #[ignore = "Requires full app state with Redis - skip in CI"]
    async fn test_network_timeout_recovery() {
        let app = test_helpers::create_minimal_test_app();

        // Request with timeout
        let timeout_response = tokio::time::timeout(
            Duration::from_millis(100),
            app.clone().oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/extract")
                    .body(Body::from(
                        json!({
                            "url": "https://slow-endpoint.example.com",
                            "timeout_ms": 50
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            ),
        )
        .await;

        // Should either timeout or return error
        assert!(
            timeout_response.is_err()
                || timeout_response
                    .unwrap()
                    .unwrap()
                    .status()
                    .is_server_error()
        );

        // Verify system recovered and can handle new requests
        let recovery_response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(recovery_response.status(), StatusCode::OK);
    }

    /// Recovery Test 7: Invalid data handling
    /// Tests recovery from malformed requests
    #[tokio::test]
    async fn test_invalid_data_recovery() {
        let app = test_helpers::create_minimal_test_app();

        // Send malformed JSON
        let invalid_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/extract")
                    .header("content-type", "application/json")
                    .body(Body::from("{invalid json"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(invalid_response.status(), StatusCode::BAD_REQUEST);

        // Verify system still works after invalid request
        let valid_response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/extract")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({"url": "https://example.com", "mode": "standard"}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(valid_response.status(), StatusCode::OK);
    }

    /// Recovery Test 8: Cascading failure prevention
    /// Tests circuit breaker prevents cascading failures
    #[tokio::test]
    async fn test_circuit_breaker_prevents_cascade() {
        let app = test_helpers::create_minimal_test_app();

        // Cause multiple failures to trip circuit breaker
        for _ in 0..10 {
            let _ = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/api/v1/external/failing-service")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await;
        }

        // Circuit should be open, requests should fast-fail
        let circuit_open_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/external/failing-service")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should return service unavailable quickly
        assert_eq!(
            circuit_open_response.status(),
            StatusCode::SERVICE_UNAVAILABLE
        );

        // Wait for circuit breaker reset
        sleep(Duration::from_secs(5)).await;

        // Circuit should allow requests again
        let recovery_response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/external/failing-service")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should attempt request again
        assert!(
            recovery_response.status().is_server_error() || recovery_response.status().is_success()
        );
    }
}
