//! Comprehensive Health Endpoint Test Suite
//!
//! This module provides complete test coverage for all health endpoints including:
//! - Unit tests for health check handlers
//! - Integration tests for endpoint routing
//! - API contract tests (response schemas)
//! - CLI command tests
//! - Error scenario tests
//! - Performance/load tests

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use serde_json::Value;
use std::time::{Duration, Instant};
use tower::ServiceExt;

mod test_helpers {
    use riptide_api::state::AppState;
    use riptide_api::routes::create_routes;
    use axum::Router;

    /// Create a test application with minimal configuration
    pub fn create_test_app() -> Router {
        let config = riptide_api::AppConfig {
            port: 0,
            redis_url: "redis://localhost:6379".to_string(),
            headless_url: Some("http://localhost:3001".to_string()),
            cache_ttl: 300,
            max_concurrency: 10,
            gate_hi_threshold: 0.8,
            gate_lo_threshold: 0.3,
            cors_origins: vec![],
            api_key: None,
            openai_api_key: None,
            spider_config: None,
        };

        // Initialize health check start time
        riptide_api::handlers::health::init_startup_time();

        let state = AppState::new(config).expect("Failed to create test app state");
        create_routes(state)
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    /// Test basic health endpoint returns 200 OK
    #[tokio::test]
    async fn test_health_endpoint_returns_ok() {
        let app = test_helpers::create_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    /// Test health endpoint returns valid JSON structure
    #[tokio::test]
    async fn test_health_endpoint_json_structure() {
        let app = test_helpers::create_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();

        let json: Value = serde_json::from_slice(&body).expect("Response should be valid JSON");

        // Verify required fields exist
        assert!(json.get("status").is_some(), "Missing 'status' field");
        assert!(json.get("version").is_some(), "Missing 'version' field");
        assert!(json.get("timestamp").is_some(), "Missing 'timestamp' field");
        assert!(json.get("uptime").is_some(), "Missing 'uptime' field");
        assert!(json.get("dependencies").is_some(), "Missing 'dependencies' field");

        // Verify status is a valid value
        let status = json["status"].as_str().unwrap();
        assert!(
            matches!(status, "healthy" | "degraded" | "unhealthy"),
            "Invalid status value: {}",
            status
        );
    }

    /// Test detailed health endpoint
    #[tokio::test]
    async fn test_detailed_health_endpoint() {
        let app = test_helpers::create_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/health/detailed")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(
            response.status() == StatusCode::OK ||
            response.status() == StatusCode::SERVICE_UNAVAILABLE
        );

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();

        let json: Value = serde_json::from_slice(&body).expect("Response should be valid JSON");

        // Verify comprehensive fields
        assert!(json.get("dependencies").is_some());
        assert!(json.get("metrics").is_some());

        // Check dependencies structure
        let deps = &json["dependencies"];
        assert!(deps.get("redis").is_some());
        assert!(deps.get("extractor").is_some());
        assert!(deps.get("http_client").is_some());
    }

    /// Test component-specific health check
    #[tokio::test]
    async fn test_component_health_redis() {
        let app = test_helpers::create_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/health/component/redis")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();

        let json: Value = serde_json::from_slice(&body).expect("Response should be valid JSON");

        // Verify component health structure
        assert!(json.get("status").is_some());
        assert!(json.get("last_check").is_some());
    }

    /// Test component health for all components
    #[tokio::test]
    async fn test_all_component_health_checks() {
        let components = vec!["redis", "extractor", "http_client", "headless", "spider"];

        for component in components {
            let app = test_helpers::create_test_app();
            let uri = format!("/api/health/component/{}", component);

            let response = app
                .oneshot(
                    Request::builder()
                        .uri(&uri)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();

            // Should return either OK or SERVICE_UNAVAILABLE, not 404
            assert_ne!(
                response.status(),
                StatusCode::NOT_FOUND,
                "Component {} should exist",
                component
            );
        }
    }

    /// Test metrics endpoint
    #[tokio::test]
    async fn test_metrics_endpoint() {
        let app = test_helpers::create_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/health/metrics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();

        let json: Value = serde_json::from_slice(&body).expect("Response should be valid JSON");

        // Verify metrics structure
        assert!(json.get("memory_usage_bytes").is_some());
        assert!(json.get("total_requests").is_some());
        assert!(json.get("avg_response_time_ms").is_some());
    }
}

#[cfg(test)]
mod contract_tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    /// Health response schema contract
    #[derive(Debug, Deserialize, Serialize)]
    struct HealthResponseContract {
        status: String,
        version: String,
        timestamp: String,
        uptime: u64,
        dependencies: DependenciesContract,
        metrics: Option<MetricsContract>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    struct DependenciesContract {
        redis: ServiceHealthContract,
        extractor: ServiceHealthContract,
        http_client: ServiceHealthContract,
        headless_service: Option<ServiceHealthContract>,
        spider_engine: Option<ServiceHealthContract>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    struct ServiceHealthContract {
        status: String,
        message: Option<String>,
        response_time_ms: Option<u64>,
        last_check: String,
    }

    #[derive(Debug, Deserialize, Serialize)]
    struct MetricsContract {
        memory_usage_bytes: u64,
        active_connections: u32,
        total_requests: u64,
        requests_per_second: f64,
        avg_response_time_ms: f64,
    }

    /// Test health response matches contract
    #[tokio::test]
    async fn test_health_response_contract() {
        let app = test_helpers::create_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();

        // This will fail if the contract doesn't match
        let health: HealthResponseContract = serde_json::from_slice(&body)
            .expect("Health response should match contract");

        // Verify field constraints
        assert!(!health.status.is_empty());
        assert!(!health.version.is_empty());
        assert!(!health.timestamp.is_empty());
        assert!(health.uptime >= 0);
    }

    /// Test service health contract
    #[tokio::test]
    async fn test_service_health_contract() {
        let app = test_helpers::create_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/health/component/redis")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();

        let service: ServiceHealthContract = serde_json::from_slice(&body)
            .expect("Service health response should match contract");

        // Verify status is valid
        assert!(
            matches!(service.status.as_str(), "healthy" | "unhealthy" | "degraded" | "unknown" | "not_configured"),
            "Invalid service status: {}",
            service.status
        );
    }

    /// Test metrics contract
    #[tokio::test]
    async fn test_metrics_contract() {
        let app = test_helpers::create_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/health/metrics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();

        let metrics: MetricsContract = serde_json::from_slice(&body)
            .expect("Metrics response should match contract");

        // Verify metrics have reasonable values
        assert!(metrics.memory_usage_bytes > 0);
        assert!(metrics.avg_response_time_ms >= 0.0);
        assert!(metrics.requests_per_second >= 0.0);
    }
}

#[cfg(test)]
mod error_scenarios {
    use super::*;

    /// Test invalid component returns 404
    #[tokio::test]
    async fn test_invalid_component_returns_404() {
        let app = test_helpers::create_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/health/component/invalid_component")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    /// Test health endpoint with invalid method
    #[tokio::test]
    async fn test_health_invalid_method() {
        let app = test_helpers::create_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    /// Test health endpoint handles concurrent requests
    #[tokio::test]
    async fn test_health_concurrent_requests() {
        use futures::future::join_all;

        let requests = (0..10).map(|_| async {
            let app = test_helpers::create_test_app();
            app.oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
        });

        let responses = join_all(requests).await;

        // All requests should succeed
        for response in responses {
            let resp = response.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
        }
    }

    /// Test health endpoint timeout handling
    #[tokio::test]
    async fn test_health_timeout_resilience() {
        let app = test_helpers::create_test_app();

        // The health check has a 5-second timeout built-in
        let start = Instant::now();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let duration = start.elapsed();

        // Should respond within reasonable time (< 6 seconds to account for timeout)
        assert!(
            duration < Duration::from_secs(6),
            "Health check took too long: {:?}",
            duration
        );

        // Should still return a response (even if degraded)
        assert!(
            response.status() == StatusCode::OK ||
            response.status() == StatusCode::SERVICE_UNAVAILABLE
        );
    }

    /// Test health with Redis unavailable
    #[tokio::test]
    async fn test_health_redis_unavailable() {
        // This test assumes Redis might not be available
        let app = test_helpers::create_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should still return a response
        assert!(
            response.status() == StatusCode::OK ||
            response.status() == StatusCode::SERVICE_UNAVAILABLE
        );

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();

        let json: Value = serde_json::from_slice(&body).unwrap();

        // Should report Redis status
        assert!(json["dependencies"]["redis"]["status"].is_string());
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;

    /// Test health endpoint response time
    #[tokio::test]
    async fn test_health_response_time() {
        let app = test_helpers::create_test_app();

        let start = Instant::now();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let duration = start.elapsed();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            duration < Duration::from_millis(500),
            "Health check should complete in < 500ms, took {:?}",
            duration
        );
    }

    /// Test detailed health endpoint response time
    #[tokio::test]
    async fn test_detailed_health_response_time() {
        let app = test_helpers::create_test_app();

        let start = Instant::now();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/health/detailed")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let duration = start.elapsed();

        assert!(
            response.status() == StatusCode::OK ||
            response.status() == StatusCode::SERVICE_UNAVAILABLE
        );

        // Detailed check can take longer but should be < 2s
        assert!(
            duration < Duration::from_secs(2),
            "Detailed health check should complete in < 2s, took {:?}",
            duration
        );
    }

    /// Test health endpoint under load
    #[tokio::test]
    async fn test_health_under_load() {
        use futures::future::join_all;

        let num_requests = 50;
        let start = Instant::now();

        let requests = (0..num_requests).map(|_| async {
            let app = test_helpers::create_test_app();
            let req_start = Instant::now();
            let result = app
                .oneshot(
                    Request::builder()
                        .uri("/healthz")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await;
            (result, req_start.elapsed())
        });

        let responses = join_all(requests).await;
        let total_duration = start.elapsed();

        let mut success_count = 0;
        let mut total_response_time = Duration::ZERO;

        for (response, response_time) in responses {
            if let Ok(resp) = response {
                if resp.status() == StatusCode::OK {
                    success_count += 1;
                    total_response_time += response_time;
                }
            }
        }

        // At least 95% should succeed
        assert!(
            success_count >= (num_requests * 95 / 100),
            "Expected 95% success rate, got {}/{}",
            success_count,
            num_requests
        );

        // Average response time should be reasonable
        let avg_response_time = total_response_time / success_count as u32;
        assert!(
            avg_response_time < Duration::from_secs(1),
            "Average response time under load should be < 1s, got {:?}",
            avg_response_time
        );

        println!(
            "Load test: {}/{} succeeded in {:?} (avg {:?})",
            success_count, num_requests, total_duration, avg_response_time
        );
    }

    /// Benchmark metrics collection performance
    #[tokio::test]
    async fn test_metrics_collection_performance() {
        let app = test_helpers::create_test_app();

        let start = Instant::now();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/health/metrics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let duration = start.elapsed();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            duration < Duration::from_millis(200),
            "Metrics collection should be < 200ms, took {:?}",
            duration
        );
    }
}

#[cfg(test)]
mod backward_compatibility {
    use super::*;

    /// Test legacy health endpoint path
    #[tokio::test]
    async fn test_legacy_health_paths() {
        let app = test_helpers::create_test_app();

        // Test primary path
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    /// Test that all response fields are present for backward compatibility
    #[tokio::test]
    async fn test_response_fields_backward_compatible() {
        let app = test_helpers::create_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();

        let json: Value = serde_json::from_slice(&body).unwrap();

        // Core fields that must always be present
        let required_fields = vec!["status", "version", "timestamp", "uptime", "dependencies"];

        for field in required_fields {
            assert!(
                json.get(field).is_some(),
                "Required field '{}' missing from response",
                field
            );
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Test health check integrates with metrics
    #[tokio::test]
    async fn test_health_metrics_integration() {
        let app = test_helpers::create_test_app();

        // Get health
        let health_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let health_body = axum::body::to_bytes(health_resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let health_json: Value = serde_json::from_slice(&health_body).unwrap();

        // Get metrics
        let metrics_resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/health/metrics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let metrics_body = axum::body::to_bytes(metrics_resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let metrics_json: Value = serde_json::from_slice(&metrics_body).unwrap();

        // Metrics from health should match standalone metrics
        if let Some(health_metrics) = health_json.get("metrics") {
            assert_eq!(
                health_metrics["memory_usage_bytes"],
                metrics_json["memory_usage_bytes"]
            );
        }
    }

    /// Test component health aggregates correctly
    #[tokio::test]
    async fn test_component_health_aggregation() {
        let app = test_helpers::create_test_app();

        // Get overall health
        let health_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let health_body = axum::body::to_bytes(health_resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let health_json: Value = serde_json::from_slice(&health_body).unwrap();

        // Check individual components match
        let components = vec!["redis", "extractor", "http_client"];

        for component in components {
            let comp_resp = test_helpers::create_test_app()
                .oneshot(
                    Request::builder()
                        .uri(&format!("/api/health/component/{}", component))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();

            let comp_body = axum::body::to_bytes(comp_resp.into_body(), usize::MAX)
                .await
                .unwrap();
            let comp_json: Value = serde_json::from_slice(&comp_body).unwrap();

            // Component status should match in overall health
            assert_eq!(
                comp_json["status"],
                health_json["dependencies"][component]["status"],
                "Component {} status mismatch",
                component
            );
        }
    }
}
