//! Integration tests for memory profiling endpoint
//!
//! Tests the /api/v1/memory/profile endpoint for:
//! - Response format and structure
//! - Performance (< 10ms target)
//! - Memory metrics accuracy
//! - Component breakdown
//! - Pressure status calculation

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

mod test_helpers;

#[tokio::test]
async fn test_memory_profile_endpoint_returns_valid_json() {
    let app = test_helpers::create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/memory/profile")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).expect("Response should be valid JSON");

    // Verify required fields exist
    assert!(json.get("timestamp").is_some(), "Missing timestamp field");
    assert!(
        json.get("total_allocated_mb").is_some(),
        "Missing total_allocated_mb field"
    );
    assert!(
        json.get("peak_usage_mb").is_some(),
        "Missing peak_usage_mb field"
    );
    assert!(
        json.get("current_usage_mb").is_some(),
        "Missing current_usage_mb field"
    );
    assert!(
        json.get("by_component").is_some(),
        "Missing by_component field"
    );
    assert!(json.get("pressure").is_some(), "Missing pressure field");
    assert!(json.get("stats").is_some(), "Missing stats field");
}

#[tokio::test]
async fn test_memory_profile_component_breakdown() {
    let app = test_helpers::create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/memory/profile")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();

    let by_component = json
        .get("by_component")
        .expect("by_component field should exist");

    // Verify all component categories exist
    assert!(
        by_component.get("extraction").is_some(),
        "Missing extraction component"
    );
    assert!(by_component.get("api").is_some(), "Missing api component");
    assert!(
        by_component.get("cache").is_some(),
        "Missing cache component"
    );
    assert!(
        by_component.get("browser").is_some(),
        "Missing browser component"
    );
    assert!(
        by_component.get("other").is_some(),
        "Missing other component"
    );

    // Verify component values are reasonable numbers
    let extraction = by_component["extraction"]
        .as_u64()
        .expect("extraction should be a number");
    let api = by_component["api"]
        .as_u64()
        .expect("api should be a number");
    let cache = by_component["cache"]
        .as_u64()
        .expect("cache should be a number");
    let browser = by_component["browser"]
        .as_u64()
        .expect("browser should be a number");
    let other = by_component["other"]
        .as_u64()
        .expect("other should be a number");

    // All components are u64, so they are always >= 0 by definition
    // Just verify they were successfully parsed as numbers
    let _ = (extraction, api, cache, browser, other);
}

#[tokio::test]
async fn test_memory_profile_pressure_status() {
    let app = test_helpers::create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/memory/profile")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();

    let pressure = json
        .get("pressure")
        .expect("pressure field should exist")
        .as_str()
        .expect("pressure should be a string");

    // Pressure should be one of the valid values
    assert!(
        pressure == "normal" || pressure == "warning" || pressure == "critical",
        "Pressure should be 'normal', 'warning', or 'critical', got: {}",
        pressure
    );
}

#[tokio::test]
async fn test_memory_profile_stats_structure() {
    let app = test_helpers::create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/memory/profile")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();

    let stats = json.get("stats").expect("stats field should exist");

    // Verify stats structure
    assert!(
        stats.get("usage_percentage").is_some(),
        "Missing usage_percentage"
    );
    assert!(
        stats.get("is_under_pressure").is_some(),
        "Missing is_under_pressure"
    );
    assert!(
        stats.get("cleanup_count").is_some(),
        "Missing cleanup_count"
    );
    assert!(stats.get("gc_count").is_some(), "Missing gc_count");

    // Verify types
    let usage_percentage = stats["usage_percentage"]
        .as_f64()
        .expect("usage_percentage should be a number");
    let is_under_pressure = stats["is_under_pressure"]
        .as_bool()
        .expect("is_under_pressure should be a boolean");

    // Verify reasonable values
    assert!(
        (0.0..=100.0).contains(&usage_percentage),
        "usage_percentage should be between 0 and 100, got: {}",
        usage_percentage
    );
    // is_under_pressure is already verified to be a boolean by type system
    let _ = is_under_pressure;
}

#[tokio::test]
async fn test_memory_profile_performance() {
    let app = test_helpers::create_test_app().await;

    let start = std::time::Instant::now();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/memory/profile")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let elapsed = start.elapsed();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify response time is reasonable (< 50ms including test overhead)
    // The handler itself should be < 10ms, but test overhead adds time
    assert!(
        elapsed.as_millis() < 50,
        "Memory profile endpoint should respond in < 50ms, took: {}ms",
        elapsed.as_millis()
    );

    println!(
        "Memory profile endpoint responded in {}ms (target: <10ms handler + test overhead)",
        elapsed.as_millis()
    );
}

#[tokio::test]
async fn test_memory_metrics_are_reasonable() {
    let app = test_helpers::create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/memory/profile")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();

    let current_usage_mb = json["current_usage_mb"]
        .as_u64()
        .expect("current_usage_mb should be a number");
    let peak_usage_mb = json["peak_usage_mb"]
        .as_u64()
        .expect("peak_usage_mb should be a number");
    let total_allocated_mb = json["total_allocated_mb"]
        .as_u64()
        .expect("total_allocated_mb should be a number");

    // Verify reasonable values
    assert!(
        current_usage_mb > 0,
        "current_usage_mb should be > 0, got: {}",
        current_usage_mb
    );
    assert!(
        peak_usage_mb >= current_usage_mb,
        "peak_usage_mb should be >= current_usage_mb, got peak: {}, current: {}",
        peak_usage_mb,
        current_usage_mb
    );
    assert!(
        total_allocated_mb >= current_usage_mb,
        "total_allocated_mb should be >= current_usage_mb"
    );

    // Verify not absurdly large (< 10GB)
    assert!(
        current_usage_mb < 10_000,
        "current_usage_mb seems unreasonably large: {} MB",
        current_usage_mb
    );

    println!(
        "Memory metrics: current={} MB, peak={} MB, total_allocated={} MB",
        current_usage_mb, peak_usage_mb, total_allocated_mb
    );
}

#[tokio::test]
async fn test_timestamp_format() {
    let app = test_helpers::create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/memory/profile")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();

    let timestamp = json
        .get("timestamp")
        .expect("timestamp field should exist")
        .as_str()
        .expect("timestamp should be a string");

    // Verify it's a valid ISO 8601 timestamp
    assert!(
        chrono::DateTime::parse_from_rfc3339(timestamp).is_ok(),
        "timestamp should be valid ISO 8601 format, got: {}",
        timestamp
    );
}

#[tokio::test]
async fn test_multiple_requests_consistency() {
    let app = test_helpers::create_test_app().await;

    // Make multiple requests
    let response1 = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/memory/profile")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body1 = response1.into_body().collect().await.unwrap().to_bytes();
    let json1: Value = serde_json::from_slice(&body1).unwrap();

    // Small delay
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let response2 = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/memory/profile")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body2 = response2.into_body().collect().await.unwrap().to_bytes();
    let json2: Value = serde_json::from_slice(&body2).unwrap();

    // Both responses should have valid structure
    assert!(json1.get("current_usage_mb").is_some());
    assert!(json2.get("current_usage_mb").is_some());

    // Memory usage should be in similar range (within 50% difference)
    let usage1 = json1["current_usage_mb"].as_u64().unwrap() as f64;
    let usage2 = json2["current_usage_mb"].as_u64().unwrap() as f64;

    let difference_ratio = (usage1 - usage2).abs() / usage1.max(1.0);
    assert!(
        difference_ratio < 0.5,
        "Memory usage should be consistent across requests, got: {} MB and {} MB",
        usage1,
        usage2
    );
}

#[cfg(feature = "jemalloc")]
#[tokio::test]
async fn test_jemalloc_stats_when_enabled() {
    let app = test_helpers::create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/memory/profile")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // When jemalloc feature is enabled, jemalloc field should exist
    assert!(
        json.get("jemalloc").is_some(),
        "jemalloc field should be present when feature is enabled"
    );

    let jemalloc = json.get("jemalloc").unwrap();
    assert!(jemalloc.get("allocated_mb").is_some());
    assert!(jemalloc.get("resident_mb").is_some());
    assert!(jemalloc.get("fragmentation_ratio").is_some());
}
