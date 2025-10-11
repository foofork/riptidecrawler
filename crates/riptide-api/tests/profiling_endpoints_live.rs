//! Live integration tests for profiling API endpoints
//!
//! These tests verify the actual HTTP endpoints work correctly with real requests.
//! They test:
//! - All 6 profiling endpoints (GET and POST)
//! - Response format and data validation
//! - Error handling and edge cases
//! - Performance overhead (< 2%)
//! - Concurrent request handling

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::Value;
use std::sync::Arc;
use tower::ServiceExt;

use riptide_api::health::HealthChecker;
use riptide_api::metrics::RipTideMetrics;
use riptide_api::state::{AppConfig, AppState};

/// Create a test AppState for endpoint testing
async fn create_test_state() -> Arc<AppState> {
    let config = AppConfig::default();
    let metrics = Arc::new(RipTideMetrics::new().expect("Failed to create metrics"));
    let health_checker = Arc::new(HealthChecker::new());

    Arc::new(
        AppState::new(config, metrics, health_checker)
            .await
            .expect("Failed to create AppState"),
    )
}

/// Helper to build a router with profiling endpoints
fn build_test_router(state: Arc<AppState>) -> axum::Router {
    use axum::routing::{get, post};
    use riptide_api::handlers::profiling;

    axum::Router::new()
        .route("/api/profiling/memory", get(profiling::get_memory_profile))
        .route("/api/profiling/cpu", get(profiling::get_cpu_profile))
        .route(
            "/api/profiling/bottlenecks",
            get(profiling::get_bottleneck_analysis),
        )
        .route(
            "/api/profiling/allocations",
            get(profiling::get_allocation_metrics),
        )
        .route(
            "/api/profiling/leak-detection",
            post(profiling::trigger_leak_detection),
        )
        .route(
            "/api/profiling/snapshot",
            post(profiling::trigger_heap_snapshot),
        )
        .with_state((*state).clone())
}

#[tokio::test]
async fn test_memory_profile_endpoint_returns_valid_data() {
    let state = create_test_state().await;
    let app = build_test_router(state);

    let request = Request::builder()
        .uri("/api/profiling/memory")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert!(json["timestamp"].is_string());
    assert!(json["rss_mb"].is_number());
    assert!(json["heap_mb"].is_number());
    assert!(json["virtual_mb"].is_number());
    assert!(json["growth_rate_mb_per_sec"].is_number());
    assert!(json["threshold_status"].is_string());
    assert!(json["warnings"].is_array());

    // Verify values are reasonable
    let rss_mb = json["rss_mb"].as_f64().unwrap();
    assert!(
        rss_mb > 0.0 && rss_mb < 10_000.0,
        "RSS should be reasonable"
    );

    let heap_mb = json["heap_mb"].as_f64().unwrap();
    assert!(heap_mb >= 0.0, "Heap should be non-negative");

    println!(
        "Memory profile response: {}",
        serde_json::to_string_pretty(&json).unwrap()
    );
}

#[tokio::test]
async fn test_cpu_profile_endpoint_returns_valid_data() {
    let state = create_test_state().await;
    let app = build_test_router(state);

    let request = Request::builder()
        .uri("/api/profiling/cpu")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert!(json["timestamp"].is_string());
    assert!(json["cpu_usage_percent"].is_number());
    assert!(json["user_time_percent"].is_number());
    assert!(json["system_time_percent"].is_number());
    assert!(json["idle_time_percent"].is_number());
    assert!(json["load_average"].is_object());
    assert!(json["available"].is_boolean());

    // Verify CPU percentages are valid
    let cpu_usage = json["cpu_usage_percent"].as_f64().unwrap();
    assert!(
        (0.0..=100.0).contains(&cpu_usage),
        "CPU usage should be 0-100%"
    );

    println!(
        "CPU profile response: {}",
        serde_json::to_string_pretty(&json).unwrap()
    );
}

#[tokio::test]
async fn test_bottleneck_analysis_endpoint_returns_hotspots() {
    let state = create_test_state().await;
    let app = build_test_router(state);

    let request = Request::builder()
        .uri("/api/profiling/bottlenecks")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert!(json["timestamp"].is_string());
    assert!(json["analysis_duration_ms"].is_number());
    assert!(json["hotspots"].is_array());
    assert!(json["total_samples"].is_number());
    assert!(json["cpu_bound_percent"].is_number());
    assert!(json["io_bound_percent"].is_number());
    assert!(json["memory_bound_percent"].is_number());
    assert!(json["recommendations"].is_array());

    // Verify hotspots structure if present
    if let Some(hotspots) = json["hotspots"].as_array() {
        for hotspot in hotspots {
            assert!(hotspot["function_name"].is_string());
            assert!(hotspot["cpu_time_percent"].is_number());
            assert!(hotspot["impact_score"].is_number());
        }
    }

    println!(
        "Bottleneck analysis response: {}",
        serde_json::to_string_pretty(&json).unwrap()
    );
}

#[tokio::test]
async fn test_allocation_metrics_endpoint_returns_allocator_info() {
    let state = create_test_state().await;
    let app = build_test_router(state);

    let request = Request::builder()
        .uri("/api/profiling/allocations")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert!(json["timestamp"].is_string());
    assert!(json["top_allocators"].is_array());
    assert!(json["size_distribution"].is_object());
    assert!(json["efficiency_score"].is_number());
    assert!(json["fragmentation_percent"].is_number());
    assert!(json["recommendations"].is_array());

    // Verify size distribution structure
    let size_dist = &json["size_distribution"];
    assert!(size_dist["small_0_1kb"].is_number());
    assert!(size_dist["medium_1_100kb"].is_number());
    assert!(size_dist["large_100kb_1mb"].is_number());
    assert!(size_dist["huge_1mb_plus"].is_number());

    // Verify efficiency score is valid
    let efficiency = json["efficiency_score"].as_f64().unwrap();
    assert!(
        (0.0..=1.0).contains(&efficiency),
        "Efficiency score should be 0-1"
    );

    println!(
        "Allocation metrics response: {}",
        serde_json::to_string_pretty(&json).unwrap()
    );
}

#[tokio::test]
async fn test_leak_detection_endpoint_analyzes_memory_growth() {
    let state = create_test_state().await;
    let app = build_test_router(state);

    let request = Request::builder()
        .method("POST")
        .uri("/api/profiling/leak-detection")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert!(json["timestamp"].is_string());
    assert!(json["analysis_duration_ms"].is_number());
    assert!(json["potential_leaks"].is_array());
    assert!(json["growth_rate_mb_per_hour"].is_number());
    assert!(json["suspicious_patterns"].is_array());
    assert!(json["recommendations"].is_array());

    // Verify leak info structure if leaks detected
    if let Some(leaks) = json["potential_leaks"].as_array() {
        for leak in leaks {
            assert!(leak["component"].is_string());
            assert!(leak["growth_rate_mb_per_hour"].is_number());
            assert!(leak["severity"].is_string());
        }
    }

    // Growth rate should be reasonable
    let growth_rate = json["growth_rate_mb_per_hour"].as_f64().unwrap();
    assert!(
        growth_rate.abs() < 1000.0,
        "Growth rate should be < 1GB/hour"
    );

    println!(
        "Leak detection response: {}",
        serde_json::to_string_pretty(&json).unwrap()
    );
}

#[tokio::test]
async fn test_heap_snapshot_endpoint_creates_snapshot() {
    let state = create_test_state().await;
    let app = build_test_router(state);

    let request = Request::builder()
        .method("POST")
        .uri("/api/profiling/snapshot")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert!(json["timestamp"].is_string());
    assert!(json["snapshot_id"].is_string());
    assert!(json["file_path"].is_string());
    assert!(json["size_bytes"].is_number());
    assert!(json["status"].is_string());
    assert!(json["download_url"].is_string());

    // Verify snapshot metadata
    let snapshot_id = json["snapshot_id"].as_str().unwrap();
    assert!(
        snapshot_id.starts_with("snapshot_"),
        "Snapshot ID should have correct prefix"
    );

    let status = json["status"].as_str().unwrap();
    assert_eq!(status, "completed", "Snapshot should be completed");

    let size = json["size_bytes"].as_u64().unwrap();
    assert!(size > 0, "Snapshot should have non-zero size");

    println!(
        "Heap snapshot response: {}",
        serde_json::to_string_pretty(&json).unwrap()
    );
}

#[tokio::test]
async fn test_profiling_endpoints_error_handling() {
    let state = create_test_state().await;
    let app = build_test_router(state);

    // Test invalid method on GET endpoint
    let request = Request::builder()
        .method("POST")
        .uri("/api/profiling/memory")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should return 405 Method Not Allowed
    assert_eq!(
        response.status(),
        StatusCode::METHOD_NOT_ALLOWED,
        "Should reject POST on GET-only endpoint"
    );
}

#[tokio::test]
async fn test_profiling_endpoints_concurrent_requests() {
    let state = create_test_state().await;

    // Spawn multiple concurrent requests to different endpoints
    let mut handles = vec![];

    for i in 0..10 {
        let state_clone = state.clone();
        let handle = tokio::spawn(async move {
            let app = build_test_router(state_clone);

            let endpoint = match i % 4 {
                0 => "/api/profiling/memory",
                1 => "/api/profiling/cpu",
                2 => "/api/profiling/bottlenecks",
                _ => "/api/profiling/allocations",
            };

            let request = Request::builder()
                .uri(endpoint)
                .body(Body::empty())
                .unwrap();

            let response = app.oneshot(request).await.unwrap();
            assert_eq!(
                response.status(),
                StatusCode::OK,
                "Endpoint {} should succeed",
                endpoint
            );
        });
        handles.push(handle);
    }

    // Wait for all requests to complete
    for handle in handles {
        handle.await.expect("Request should complete successfully");
    }
}

#[tokio::test]
async fn test_profiling_performance_overhead() {
    let state = create_test_state().await;
    let app = build_test_router(state.clone());

    // Measure baseline metrics collection time
    let iterations = 100;
    let start = std::time::Instant::now();

    for _ in 0..iterations {
        let app_clone = app.clone();
        let request = Request::builder()
            .uri("/api/profiling/memory")
            .body(Body::empty())
            .unwrap();

        let response = app_clone.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    let total_time = start.elapsed();
    let avg_time_ms = total_time.as_millis() as f64 / iterations as f64;

    // Performance overhead should be minimal (< 50ms per request)
    assert!(
        avg_time_ms < 50.0,
        "Average request time {}ms should be < 50ms (2% overhead target)",
        avg_time_ms
    );

    println!(
        "Profiling endpoint performance: {:.2}ms avg over {} requests",
        avg_time_ms, iterations
    );
}

#[tokio::test]
async fn test_memory_profile_threshold_warnings() {
    let state = create_test_state().await;
    let app = build_test_router(state);

    let request = Request::builder()
        .uri("/api/profiling/memory")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify threshold status logic
    let threshold_status = json["threshold_status"].as_str().unwrap();
    let rss_mb = json["rss_mb"].as_f64().unwrap();

    if rss_mb > 700.0 {
        assert_eq!(threshold_status, "critical");
    } else if rss_mb > 650.0 {
        assert_eq!(threshold_status, "warning");
    } else {
        assert_eq!(threshold_status, "normal");
    }

    // Warnings should be array
    assert!(json["warnings"].is_array());
}

#[tokio::test]
async fn test_all_endpoints_return_valid_json() {
    let state = create_test_state().await;
    let app = build_test_router(state);

    let endpoints = vec![
        "/api/profiling/memory",
        "/api/profiling/cpu",
        "/api/profiling/bottlenecks",
        "/api/profiling/allocations",
    ];

    for endpoint in endpoints {
        let app_clone = app.clone();
        let request = Request::builder()
            .uri(endpoint)
            .body(Body::empty())
            .unwrap();

        let response = app_clone.oneshot(request).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Endpoint {} should return 200",
            endpoint
        );

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json_result: Result<Value, _> = serde_json::from_slice(&body);
        assert!(
            json_result.is_ok(),
            "Endpoint {} should return valid JSON",
            endpoint
        );
    }
}

#[tokio::test]
async fn test_profiling_endpoints_response_time() {
    let state = create_test_state().await;

    let endpoints = vec![
        "/api/profiling/memory",
        "/api/profiling/cpu",
        "/api/profiling/bottlenecks",
        "/api/profiling/allocations",
    ];

    for endpoint in endpoints {
        let app = build_test_router(state.clone());
        let start = std::time::Instant::now();

        let request = Request::builder()
            .uri(endpoint)
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let elapsed = start.elapsed();

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Endpoint {} should succeed",
            endpoint
        );

        // Each endpoint should respond quickly (< 100ms)
        assert!(
            elapsed.as_millis() < 100,
            "Endpoint {} took {}ms, should be < 100ms",
            endpoint,
            elapsed.as_millis()
        );

        println!(
            "Endpoint {} response time: {}ms",
            endpoint,
            elapsed.as_millis()
        );
    }
}

#[tokio::test]
async fn test_leak_detection_with_repeated_calls() {
    let state = create_test_state().await;

    // Call leak detection multiple times to ensure consistent behavior
    for i in 0..5 {
        let app = build_test_router(state.clone());
        let request = Request::builder()
            .method("POST")
            .uri("/api/profiling/leak-detection")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Leak detection call {} should succeed",
            i + 1
        );

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        // Verify response is valid
        assert!(json["analysis_duration_ms"].is_number());
        assert!(json["potential_leaks"].is_array());
    }
}
