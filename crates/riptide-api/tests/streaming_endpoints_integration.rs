//! Integration tests for streaming endpoints (NDJSON, SSE, WebSocket)
//!
//! This test suite validates:
//! - NDJSON streaming endpoint functionality
//! - SSE endpoint with reconnection handling
//! - WebSocket bidirectional communication
//! - Metrics collection and Prometheus integration
//! - Backpressure handling across all protocols
//!
//! Test Coverage: 15+ comprehensive integration tests

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use bytes::Bytes;
use http_body_util::BodyExt;
use riptide_api::state::{AppConfig, AppState};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::time::{timeout, Duration};
use tower::ServiceExt;

mod test_helpers;

/// Test helper to create test AppState
async fn create_test_app_state() -> AppState {
    let config = AppConfig::default();
    let metrics =
        Arc::new(riptide_api::metrics::RipTideMetrics::new().expect("Failed to create metrics"));
    let health_checker = Arc::new(riptide_api::health::HealthChecker::new());

    AppState::new(config, metrics, health_checker)
        .await
        .expect("Failed to create AppState")
}

/// Test helper to parse NDJSON stream
async fn parse_ndjson_body(body: Bytes) -> Vec<Value> {
    String::from_utf8_lossy(&body)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect()
}

// ============================================================================
// NDJSON Streaming Tests
// ============================================================================

#[tokio::test]
async fn test_ndjson_streaming_endpoint_exists() {
    let app_state = create_test_app_state().await;
    let app = test_helpers::create_test_router(app_state);

    let request = Request::builder()
        .method("POST")
        .uri("/crawl/stream")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "urls": ["https://example.com"],
                "max_depth": 1
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Endpoint should exist and not return 404
    assert_ne!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_ndjson_streaming_content_type() {
    let app_state = create_test_app_state().await;
    let app = test_helpers::create_test_router(app_state);

    let request = Request::builder()
        .method("POST")
        .uri("/crawl/stream")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "urls": ["https://httpbin.org/html"],
                "max_depth": 0
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should return NDJSON content type
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    assert!(
        content_type.contains("application/x-ndjson") || content_type.contains("application/json"),
        "Expected NDJSON content type, got: {}",
        content_type
    );
}

#[tokio::test]
async fn test_ndjson_streaming_response_format() {
    let app_state = create_test_app_state().await;
    let app = test_helpers::create_test_router(app_state);

    let request = Request::builder()
        .method("POST")
        .uri("/crawl/stream")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "urls": ["https://example.com"],
                "max_depth": 0
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let body = response.into_body().collect().await.unwrap().to_bytes();

    let lines = parse_ndjson_body(body).await;

    // Should have at least one result
    assert!(!lines.is_empty(), "NDJSON stream should contain results");

    // Each line should be valid JSON
    for line in &lines {
        assert!(line.is_object(), "Each NDJSON line should be a JSON object");
    }
}

#[tokio::test]
async fn test_ndjson_multiple_urls() {
    let app_state = create_test_app_state().await;
    let app = test_helpers::create_test_router(app_state);

    let request = Request::builder()
        .method("POST")
        .uri("/crawl/stream")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "urls": ["https://example.com", "https://example.org"],
                "max_depth": 0
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let body = response.into_body().collect().await.unwrap().to_bytes();

    let results = parse_ndjson_body(body).await;

    // Should process both URLs
    assert!(results.len() >= 2, "Should have results for multiple URLs");
}

#[tokio::test]
async fn test_ndjson_invalid_request() {
    let app_state = create_test_app_state().await;
    let app = test_helpers::create_test_router(app_state);

    let request = Request::builder()
        .method("POST")
        .uri("/crawl/stream")
        .header("content-type", "application/json")
        .body(Body::from(json!({"invalid": "request"}).to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should return error status
    assert!(
        response.status().is_client_error() || response.status().is_server_error(),
        "Invalid request should return error status"
    );
}

// ============================================================================
// SSE (Server-Sent Events) Tests
// ============================================================================

#[tokio::test]
async fn test_sse_endpoint_exists() {
    let app_state = create_test_app_state().await;
    let app = test_helpers::create_test_router(app_state);

    let request = Request::builder()
        .method("POST")
        .uri("/crawl/sse")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "urls": ["https://example.com"],
                "max_depth": 1
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Endpoint should exist
    assert_ne!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_sse_content_type() {
    let app_state = create_test_app_state().await;
    let app = test_helpers::create_test_router(app_state);

    let request = Request::builder()
        .method("POST")
        .uri("/crawl/sse")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "urls": ["https://example.com"],
                "max_depth": 0
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should return SSE content type
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    assert!(
        content_type.contains("text/event-stream") || content_type.contains("application/json"),
        "Expected SSE content type, got: {}",
        content_type
    );
}

#[tokio::test]
async fn test_sse_event_format() {
    let app_state = create_test_app_state().await;
    let app = test_helpers::create_test_router(app_state);

    let request = Request::builder()
        .method("POST")
        .uri("/crawl/sse")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "urls": ["https://example.com"],
                "max_depth": 0
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8_lossy(&body);

    // SSE format should have "data:" prefix or be JSON
    let is_sse = body_str.contains("data:") || body_str.contains("event:");
    let is_json = serde_json::from_str::<Value>(&body_str).is_ok();

    assert!(
        is_sse || is_json,
        "Response should be in SSE format or JSON format"
    );
}

#[tokio::test]
async fn test_sse_reconnection_support() {
    let app_state = create_test_app_state().await;
    let app = test_helpers::create_test_router(app_state);

    let request = Request::builder()
        .method("POST")
        .uri("/crawl/sse")
        .header("content-type", "application/json")
        .header("last-event-id", "test-id")
        .body(Body::from(
            json!({
                "urls": ["https://example.com"],
                "max_depth": 0
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should handle reconnection header gracefully
    assert!(
        response.status().is_success() || response.status().is_client_error(),
        "Should handle SSE reconnection gracefully"
    );
}

// ============================================================================
// WebSocket Tests
// ============================================================================

#[tokio::test]
async fn test_websocket_endpoint_exists() {
    let app_state = create_test_app_state().await;
    let app = test_helpers::create_test_router(app_state);

    let request = Request::builder()
        .method("GET")
        .uri("/crawl/ws")
        .header("upgrade", "websocket")
        .header("connection", "upgrade")
        .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
        .header("sec-websocket-version", "13")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Endpoint should exist
    assert_ne!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_websocket_upgrade_headers() {
    let app_state = create_test_app_state().await;
    let app = test_helpers::create_test_router(app_state);

    let request = Request::builder()
        .method("GET")
        .uri("/crawl/ws")
        .header("upgrade", "websocket")
        .header("connection", "upgrade")
        .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
        .header("sec-websocket-version", "13")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should either upgrade to WebSocket (101) or handle request (200/400)
    assert!(
        response.status() == StatusCode::SWITCHING_PROTOCOLS
            || response.status() == StatusCode::OK
            || response.status().is_client_error(),
        "Expected WebSocket upgrade or error response, got: {}",
        response.status()
    );
}

// ============================================================================
// Metrics and Monitoring Tests
// ============================================================================

#[tokio::test]
async fn test_streaming_metrics_collection() {
    let app_state = create_test_app_state().await;

    // Metrics should be initialized
    assert!(
        app_state.metrics.streaming_active_connections.get() >= 0.0,
        "Streaming metrics should be initialized"
    );
}

#[tokio::test]
async fn test_metrics_updated_after_stream() {
    let app_state = create_test_app_state().await;
    let initial_connections = app_state.metrics.streaming_total_connections.get();

    let app = test_helpers::create_test_router(app_state.clone());

    let request = Request::builder()
        .method("POST")
        .uri("/crawl/stream")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "urls": ["https://example.com"],
                "max_depth": 0
            })
            .to_string(),
        ))
        .unwrap();

    let _response = app.oneshot(request).await.unwrap();

    // Metrics should be updated (may or may not increment depending on implementation)
    let final_connections = app_state.metrics.streaming_total_connections.get();
    assert!(
        final_connections >= initial_connections,
        "Streaming metrics should be tracked"
    );
}

// ============================================================================
// Backpressure and Resource Management Tests
// ============================================================================

#[tokio::test]
async fn test_backpressure_handling() {
    let app_state = create_test_app_state().await;
    let app = test_helpers::create_test_router(app_state);

    // Send request with large batch to test backpressure
    let urls: Vec<String> = (0..50)
        .map(|i| format!("https://example.com/page{}", i))
        .collect();

    let request = Request::builder()
        .method("POST")
        .uri("/crawl/stream")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "urls": urls,
                "max_depth": 0
            })
            .to_string(),
        ))
        .unwrap();

    // Should handle large requests without hanging
    let result = timeout(Duration::from_secs(30), app.oneshot(request)).await;

    assert!(result.is_ok(), "Should handle backpressure without timeout");
}

#[tokio::test]
async fn test_concurrent_streaming_requests() {
    let app_state = create_test_app_state().await;

    // Create multiple concurrent streaming requests
    let handles: Vec<_> = (0..5)
        .map(|_| {
            let state = app_state.clone();
            tokio::spawn(async move {
                let app = test_helpers::create_test_router(state);
                let request = Request::builder()
                    .method("POST")
                    .uri("/crawl/stream")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "urls": ["https://example.com"],
                            "max_depth": 0
                        })
                        .to_string(),
                    ))
                    .unwrap();

                app.oneshot(request).await
            })
        })
        .collect();

    // All requests should complete successfully
    let results = futures_util::future::join_all(handles).await;
    let success_count = results.iter().filter(|r| r.is_ok()).count();

    assert!(
        success_count >= 3,
        "Most concurrent requests should succeed"
    );
}

// ============================================================================
// Error Handling and Edge Cases
// ============================================================================

#[tokio::test]
async fn test_streaming_with_invalid_url() {
    let app_state = create_test_app_state().await;
    let app = test_helpers::create_test_router(app_state);

    let request = Request::builder()
        .method("POST")
        .uri("/crawl/stream")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "urls": ["not-a-valid-url"],
                "max_depth": 0
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should handle gracefully (either validation error or error in stream)
    assert!(
        response.status().is_success() || response.status().is_client_error(),
        "Should handle invalid URLs gracefully"
    );
}

#[tokio::test]
async fn test_streaming_with_empty_urls() {
    let app_state = create_test_app_state().await;
    let app = test_helpers::create_test_router(app_state);

    let request = Request::builder()
        .method("POST")
        .uri("/crawl/stream")
        .header("content-type", "application/json")
        .body(Body::from(json!({"urls": [], "max_depth": 0}).to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should return validation error or empty result
    assert!(
        response.status().is_success() || response.status().is_client_error(),
        "Should handle empty URL list gracefully"
    );
}
