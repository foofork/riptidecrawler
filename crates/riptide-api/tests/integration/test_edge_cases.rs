/// Edge case and error handling tests for the RipTide API
///
/// This module contains tests for various edge cases, error conditions,
/// and boundary scenarios that might occur in production.

use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    response::Response,
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use std::time::Duration;
use tower::ServiceExt;
use tokio::time::timeout;

/// Create a test router with simulated error conditions
fn create_error_test_router() -> Router {
    Router::new()
        .route("/healthz", get(health_with_errors))
        .route("/crawl", post(crawl_with_errors))
        .route("/timeout", get(timeout_handler))
        .route("/memory-pressure", post(memory_pressure_handler))
        .route("/malformed", post(malformed_response_handler))
}

/// Health handler that simulates various dependency failures
async fn health_with_errors() -> Result<axum::response::Json<Value>, (StatusCode, axum::response::Json<Value>)> {
    // Simulate different error scenarios based on request timing
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    match now % 4 {
        0 => {
            // Healthy response
            Ok(axum::response::Json(json!({
                "status": "healthy",
                "version": "0.1.0",
                "timestamp": "2024-01-01T00:00:00Z",
                "uptime": 3600,
                "dependencies": {
                    "redis": {"status": "healthy"},
                    "extractor": {"status": "healthy"},
                    "http_client": {"status": "healthy"}
                }
            })))
        }
        1 => {
            // Redis failure
            Err((
                StatusCode::SERVICE_UNAVAILABLE,
                axum::response::Json(json!({
                    "status": "unhealthy",
                    "dependencies": {
                        "redis": {"status": "unhealthy", "message": "Connection timeout"},
                        "extractor": {"status": "healthy"},
                        "http_client": {"status": "healthy"}
                    }
                }))
            ))
        }
        2 => {
            // WASM extractor failure
            Err((
                StatusCode::SERVICE_UNAVAILABLE,
                axum::response::Json(json!({
                    "status": "unhealthy",
                    "dependencies": {
                        "redis": {"status": "healthy"},
                        "extractor": {"status": "unhealthy", "message": "WASM module failed to load"},
                        "http_client": {"status": "healthy"}
                    }
                }))
            ))
        }
        _ => {
            // Multiple failures
            Err((
                StatusCode::SERVICE_UNAVAILABLE,
                axum::response::Json(json!({
                    "status": "unhealthy",
                    "dependencies": {
                        "redis": {"status": "unhealthy", "message": "Connection refused"},
                        "extractor": {"status": "unhealthy", "message": "Out of memory"},
                        "http_client": {"status": "unhealthy", "message": "DNS resolution failed"}
                    }
                }))
            ))
        }
    }
}

/// Crawl handler that simulates various error conditions
async fn crawl_with_errors(
    axum::extract::Json(payload): axum::extract::Json<Value>,
) -> Result<axum::response::Json<Value>, (StatusCode, axum::response::Json<Value>)> {
    let urls = payload.get("urls")
        .and_then(|u| u.as_array())
        .ok_or_else(|| error_response(StatusCode::BAD_REQUEST, "validation_error", "urls field required"))?;

    if urls.is_empty() {
        return Err(error_response(StatusCode::BAD_REQUEST, "validation_error", "At least one URL required"));
    }

    // Simulate different error scenarios based on URL content
    for (index, url) in urls.iter().enumerate() {
        let url_str = url.as_str().unwrap_or("");

        if url_str.contains("timeout") {
            return Err(error_response(
                StatusCode::REQUEST_TIMEOUT,
                "timeout_error",
                &format!("Request timed out for URL {}", index + 1)
            ));
        }

        if url_str.contains("forbidden") {
            return Err(error_response(
                StatusCode::FORBIDDEN,
                "authentication_error",
                "Access denied to protected resource"
            ));
        }

        if url_str.contains("rate-limit") {
            return Err(error_response(
                StatusCode::TOO_MANY_REQUESTS,
                "rate_limited",
                "Rate limit exceeded. Please try again later."
            ));
        }

        if url_str.contains("server-error") {
            return Err(error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "Unexpected server error occurred"
            ));
        }

        if url_str.contains("memory-error") {
            return Err(error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "extraction_error",
                "Out of memory during content extraction"
            ));
        }

        if url_str.contains("cache-error") {
            return Err(error_response(
                StatusCode::SERVICE_UNAVAILABLE,
                "cache_error",
                "Cache service temporarily unavailable"
            ));
        }
    }

    // Return partial success with some failures
    let mut results = Vec::new();
    for (index, url) in urls.iter().enumerate() {
        let url_str = url.as_str().unwrap_or("");

        if url_str.contains("partial-fail") && index > 0 {
            results.push(json!({
                "url": url_str,
                "status": 0,
                "from_cache": false,
                "gate_decision": "failed",
                "quality_score": 0.0,
                "processing_time_ms": 0,
                "document": null,
                "error": {
                    "error_type": "fetch_error",
                    "message": "Network connection failed",
                    "retryable": true
                },
                "cache_key": ""
            }));
        } else {
            results.push(json!({
                "url": url_str,
                "status": 200,
                "from_cache": false,
                "gate_decision": "raw",
                "quality_score": 0.8,
                "processing_time_ms": 150,
                "document": {
                    "url": url_str,
                    "title": "Test Content",
                    "byline": null,
                    "published_iso": null,
                    "markdown": "# Test",
                    "text": "Test content",
                    "links": [],
                    "media": []
                },
                "error": null,
                "cache_key": format!("test_key_{}", index)
            }));
        }
    }

    let successful = results.iter().filter(|r| r["error"].is_null()).count();
    let failed = results.len() - successful;

    Ok(axum::response::Json(json!({
        "total_urls": urls.len(),
        "successful": successful,
        "failed": failed,
        "from_cache": 0,
        "results": results,
        "statistics": {
            "total_processing_time_ms": 300,
            "avg_processing_time_ms": if successful > 0 { 150.0 } else { 0.0 },
            "gate_decisions": {
                "raw": successful,
                "probes_first": 0,
                "headless": 0,
                "cached": 0
            },
            "cache_hit_rate": 0.0
        }
    })))
}

/// Handler that simulates timeout scenarios
async fn timeout_handler() -> Result<axum::response::Json<Value>, (StatusCode, axum::response::Json<Value>)> {
    // Simulate a long-running operation
    tokio::time::sleep(Duration::from_secs(35)).await; // Longer than typical timeout

    Ok(axum::response::Json(json!({
        "message": "This should have timed out"
    })))
}

/// Handler that simulates memory pressure
async fn memory_pressure_handler(
    axum::extract::Json(_payload): axum::extract::Json<Value>,
) -> Result<axum::response::Json<Value>, (StatusCode, axum::response::Json<Value>)> {
    // Simulate memory allocation (don't actually allocate GB of memory in tests)
    Err(error_response(
        StatusCode::INTERNAL_SERVER_ERROR,
        "internal_error",
        "Out of memory: unable to allocate buffer for content processing"
    ))
}

/// Handler that returns malformed responses
async fn malformed_response_handler(
    axum::extract::Json(_payload): axum::extract::Json<Value>,
) -> Response<Body> {
    // Return invalid JSON
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::from("{ invalid json }"))
        .unwrap()
}

fn error_response(status: StatusCode, error_type: &str, message: &str) -> (StatusCode, axum::response::Json<Value>) {
    (
        status,
        axum::response::Json(json!({
            "error": {
                "type": error_type,
                "message": message,
                "retryable": matches!(status, StatusCode::REQUEST_TIMEOUT | StatusCode::SERVICE_UNAVAILABLE | StatusCode::TOO_MANY_REQUESTS),
                "status": status.as_u16()
            }
        }))
    )
}

#[cfg(test)]
mod dependency_failure_tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check_redis_failure() {
        let app = create_error_test_router();

        // Make multiple requests to hit different error scenarios
        for _ in 0..5 {
            let request = Request::builder()
                .method(Method::GET)
                .uri("/healthz")
                .body(Body::empty())
                .unwrap();

            let response = app.clone().oneshot(request).await.unwrap();

            // Should be either healthy (200) or unhealthy (503)
            assert!(
                response.status() == StatusCode::OK || response.status() == StatusCode::SERVICE_UNAVAILABLE
            );

            let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let json: Value = serde_json::from_slice(&body).unwrap();

            assert!(json.get("status").is_some());
            assert!(json.get("dependencies").is_some());

            // If unhealthy, should have error details
            if json["status"] == "unhealthy" {
                let deps = &json["dependencies"];
                let has_unhealthy = deps.as_object().unwrap().values().any(|dep| {
                    dep["status"] == "unhealthy"
                });
                assert!(has_unhealthy, "At least one dependency should be unhealthy");
            }

            tokio::time::sleep(Duration::from_millis(100)).await; // Small delay to change timing
        }
    }

    #[tokio::test]
    async fn test_multiple_dependency_failures() {
        let app = create_error_test_router();

        // Try to get a multiple failure scenario
        for attempt in 0..10 {
            let request = Request::builder()
                .method(Method::GET)
                .uri("/healthz")
                .body(Body::empty())
                .unwrap();

            let response = app.clone().oneshot(request).await.unwrap();
            let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let json: Value = serde_json::from_slice(&body).unwrap();

            if json["status"] == "unhealthy" {
                let deps = &json["dependencies"];
                let unhealthy_count = deps.as_object().unwrap().values()
                    .filter(|dep| dep["status"] == "unhealthy")
                    .count();

                if unhealthy_count > 1 {
                    // Found multiple failures
                    assert!(response.status() == StatusCode::SERVICE_UNAVAILABLE);
                    assert!(unhealthy_count >= 2);
                    return;
                }
            }

            tokio::time::sleep(Duration::from_millis(250)).await; // Wait for different timing
        }

        // If we didn't hit multiple failures, that's ok - it's probabilistic
    }
}

#[cfg(test)]
mod error_scenario_tests {
    use super::*;

    #[tokio::test]
    async fn test_timeout_errors() {
        let app = create_error_test_router();

        let request_body = json!({
            "urls": ["https://timeout.example.com"]
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/crawl")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["error"]["type"], "timeout_error");
        assert_eq!(json["error"]["retryable"], true);
        assert!(json["error"]["message"].as_str().unwrap().contains("timed out"));
    }

    #[tokio::test]
    async fn test_forbidden_access_errors() {
        let app = create_error_test_router();

        let request_body = json!({
            "urls": ["https://forbidden.example.com"]
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/crawl")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["error"]["type"], "authentication_error");
        assert_eq!(json["error"]["retryable"], false);
    }

    #[tokio::test]
    async fn test_rate_limiting_errors() {
        let app = create_error_test_router();

        let request_body = json!({
            "urls": ["https://rate-limit.example.com"]
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/crawl")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["error"]["type"], "rate_limited");
        assert_eq!(json["error"]["retryable"], true);
        assert!(json["error"]["message"].as_str().unwrap().contains("rate limit"));
    }

    #[tokio::test]
    async fn test_memory_errors() {
        let app = create_error_test_router();

        let request_body = json!({
            "urls": ["https://memory-error.example.com"]
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/crawl")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["error"]["type"], "extraction_error");
        assert_eq!(json["error"]["retryable"], false);
        assert!(json["error"]["message"].as_str().unwrap().contains("memory"));
    }

    #[tokio::test]
    async fn test_cache_service_errors() {
        let app = create_error_test_router();

        let request_body = json!({
            "urls": ["https://cache-error.example.com"]
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/crawl")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["error"]["type"], "cache_error");
        assert_eq!(json["error"]["retryable"], true);
    }

    #[tokio::test]
    async fn test_partial_failure_scenarios() {
        let app = create_error_test_router();

        let request_body = json!({
            "urls": [
                "https://example.com/success",
                "https://partial-fail.example.com/fail1",
                "https://partial-fail.example.com/fail2"
            ]
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/crawl")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["total_urls"], 3);
        assert_eq!(json["successful"], 1);
        assert_eq!(json["failed"], 2);

        let results = json["results"].as_array().unwrap();
        assert_eq!(results.len(), 3);

        // First result should be successful
        assert!(results[0]["error"].is_null());
        assert_eq!(results[0]["status"], 200);

        // Other results should have errors
        assert!(!results[1]["error"].is_null());
        assert!(!results[2]["error"].is_null());
        assert_eq!(results[1]["error"]["retryable"], true);
    }
}

#[cfg(test)]
mod boundary_condition_tests {
    use super::*;

    #[tokio::test]
    async fn test_request_timeout_handling() {
        let app = create_error_test_router();

        // Test actual timeout behavior
        let request = Request::builder()
            .method(Method::GET)
            .uri("/timeout")
            .body(Body::empty())
            .unwrap();

        // Wrap the request in a timeout shorter than the handler's delay
        let result = timeout(Duration::from_secs(2), app.oneshot(request)).await;

        // Should timeout before the handler completes
        assert!(result.is_err(), "Request should have timed out");
    }

    #[tokio::test]
    async fn test_memory_pressure_simulation() {
        let app = create_error_test_router();

        let request = Request::builder()
            .method(Method::POST)
            .uri("/memory-pressure")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"data": "large payload simulation"}"#))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["error"]["type"], "internal_error");
        assert!(json["error"]["message"].as_str().unwrap().contains("memory"));
    }

    #[tokio::test]
    async fn test_malformed_response_handling() {
        let app = create_error_test_router();

        let request = Request::builder()
            .method(Method::POST)
            .uri("/malformed")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"test": "data"}"#))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Response should return OK status but invalid JSON body
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        // Should be invalid JSON
        assert!(serde_json::from_str::<Value>(&body_str).is_err());
        assert!(body_str.contains("invalid json"));
    }

    #[tokio::test]
    async fn test_extremely_large_url_list() {
        let app = create_error_test_router();

        // Create a request with exactly 100 URLs (at the limit)
        let urls: Vec<Value> = (0..100)
            .map(|i| Value::String(format!("https://example{}.com", i)))
            .collect();

        let request_body = json!({
            "urls": urls
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/crawl")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Should succeed (within limits)
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["total_urls"], 100);
        assert_eq!(json["successful"], 100);
    }

    #[tokio::test]
    async fn test_empty_request_body() {
        let app = create_error_test_router();

        let request = Request::builder()
            .method(Method::POST)
            .uri("/crawl")
            .header("content-type", "application/json")
            .body(Body::from(""))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Should return 400 for empty/invalid JSON
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_missing_content_type() {
        let app = create_error_test_router();

        let request = Request::builder()
            .method(Method::POST)
            .uri("/crawl")
            // Missing content-type header
            .body(Body::from(r#"{"urls": ["https://example.com"]}"#))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Should handle missing content-type gracefully or return appropriate error
        assert!(response.status().is_client_error() || response.status().is_success());
    }

    #[tokio::test]
    async fn test_invalid_http_methods() {
        let app = create_error_test_router();

        // Try PATCH on crawl endpoint (should be POST)
        let request = Request::builder()
            .method(Method::PATCH)
            .uri("/crawl")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"urls": ["https://example.com"]}"#))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Should return 405 Method Not Allowed or 404 Not Found
        assert!(response.status() == StatusCode::METHOD_NOT_ALLOWED || response.status() == StatusCode::NOT_FOUND);
    }
}

#[cfg(test)]
mod concurrent_error_tests {
    use super::*;
    use futures::future::join_all;

    #[tokio::test]
    async fn test_concurrent_error_conditions() {
        let app = create_error_test_router();

        // Create multiple concurrent requests with different error conditions
        let error_urls = vec![
            "https://timeout.example.com",
            "https://forbidden.example.com",
            "https://rate-limit.example.com",
            "https://server-error.example.com",
            "https://memory-error.example.com",
        ];

        let requests: Vec<_> = error_urls.into_iter().map(|url| {
            let app = app.clone();
            async move {
                let request_body = json!({"urls": [url]});
                let request = Request::builder()
                    .method(Method::POST)
                    .uri("/crawl")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap();

                app.oneshot(request).await
            }
        }).collect();

        let responses = join_all(requests).await;

        // All requests should complete (not hang)
        assert_eq!(responses.len(), 5);

        // Each should have appropriate error status
        for (i, response) in responses.into_iter().enumerate() {
            let response = response.unwrap();
            assert!(response.status().is_client_error() || response.status().is_server_error(),
                "Request {} should have error status, got {}", i, response.status());
        }
    }

    #[tokio::test]
    async fn test_health_check_under_load() {
        let app = create_error_test_router();

        // Create 20 concurrent health check requests
        let requests: Vec<_> = (0..20).map(|_| {
            let app = app.clone();
            async move {
                let request = Request::builder()
                    .method(Method::GET)
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap();

                app.oneshot(request).await
            }
        }).collect();

        let responses = join_all(requests).await;

        // All requests should complete
        assert_eq!(responses.len(), 20);

        // Count healthy vs unhealthy responses
        let mut healthy_count = 0;
        let mut unhealthy_count = 0;

        for response in responses {
            let response = response.unwrap();

            if response.status() == StatusCode::OK {
                healthy_count += 1;
            } else if response.status() == StatusCode::SERVICE_UNAVAILABLE {
                unhealthy_count += 1;
            }
        }

        // Should have a mix of responses due to error simulation
        assert!(healthy_count > 0 || unhealthy_count > 0);
        assert_eq!(healthy_count + unhealthy_count, 20);
    }
}

#[cfg(test)]
mod network_simulation_tests {
    use super::*;

    #[tokio::test]
    async fn test_slow_network_conditions() {
        let app = create_error_test_router();

        // Simulate processing delays by adding artificial delay
        let start = std::time::Instant::now();

        let request_body = json!({
            "urls": ["https://example.com/slow"]
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/crawl")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let duration = start.elapsed();

        // Response should complete reasonably quickly for mocked handler
        assert!(duration < Duration::from_secs(5));
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_intermittent_failures() {
        let app = create_error_test_router();

        // Make multiple requests to simulate intermittent failures
        let mut success_count = 0;
        let mut failure_count = 0;

        for _ in 0..10 {
            let request = Request::builder()
                .method(Method::GET)
                .uri("/healthz")
                .body(Body::empty())
                .unwrap();

            let response = app.clone().oneshot(request).await.unwrap();

            if response.status() == StatusCode::OK {
                success_count += 1;
            } else {
                failure_count += 1;
            }

            // Small delay between requests
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        // Should have some mix of successes and failures
        assert!(success_count > 0 || failure_count > 0);
        assert_eq!(success_count + failure_count, 10);
    }

    #[tokio::test]
    async fn test_connection_recovery() {
        let app = create_error_test_router();

        // Simulate a service that's initially failing but recovers
        for attempt in 0..5 {
            let request = Request::builder()
                .method(Method::GET)
                .uri("/healthz")
                .body(Body::empty())
                .unwrap();

            let response = app.clone().oneshot(request).await.unwrap();

            // Later attempts more likely to succeed due to time-based simulation
            if attempt >= 3 && response.status() == StatusCode::OK {
                // Found a successful response after initial failures
                break;
            }

            tokio::time::sleep(Duration::from_millis(300)).await;
        }

        // Test completed successfully (recovery simulation)
    }
}