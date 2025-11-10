//! Integration tests for streaming metrics
//!
//! Tests verify that:
//! - All metrics are properly collected
//! - Prometheus integration works correctly
//! - Metrics are exposed via /metrics endpoint
//! - Alert thresholds are correctly configured
//!
//! Note: This test file is maintained for backward compatibility testing
//! with the deprecated streaming metrics API. The streaming module has been
//! refactored and moved to the facades/adapters architecture.

#![allow(deprecated)]

// The following imports are commented out as these modules have been removed
// during the streaming module refactoring (Phase 4.3).
// use riptide_api::streaming::metrics::StreamingMetrics;
// use riptide_api::streaming::lifecycle::StreamLifecycleManager;
// use riptide_api::metrics::RipTideMetrics;
// use std::sync::Arc;
// use std::time::Duration;

// Tests using removed StreamingMetrics type have been commented out
// The metrics functionality has been moved to facades/adapters architecture

/* DISABLED - StreamingMetrics removed in Phase 4.3 refactoring
#[test]
fn test_streaming_metrics_basic() {
    let mut metrics = StreamingMetrics::default();

    // Simulate connections
    metrics.record_connection();
    assert_eq!(metrics.active_connections, 1);
    assert_eq!(metrics.total_connections, 1);

    metrics.record_connection();
    assert_eq!(metrics.active_connections, 2);
    assert_eq!(metrics.total_connections, 2);

    // Simulate disconnection
    metrics.record_disconnection(Duration::from_secs(30));
    assert_eq!(metrics.active_connections, 1);
    assert_eq!(metrics.average_connection_duration_ms, 15000.0);
}
*/

/* DISABLED - StreamingMetrics removed in Phase 4.3 refactoring
#[test]
fn test_streaming_metrics_delivery_ratio() {
    let mut metrics = StreamingMetrics::default();

    // Perfect delivery
    metrics.record_item_sent();
    metrics.record_item_sent();
    metrics.record_item_sent();
    assert_eq!(metrics.delivery_ratio(), 1.0);

    // With drops
    metrics.record_item_dropped();
    assert_eq!(metrics.delivery_ratio(), 0.75); // 3 sent, 1 dropped = 75%
}

#[test]
fn test_streaming_metrics_error_rate() {
    let mut metrics = StreamingMetrics::default();

    metrics.record_connection();
    metrics.record_connection();
    metrics.record_connection();
    metrics.record_connection();

    // 1 error out of 4 connections = 25%
    metrics.record_error();
    assert_eq!(metrics.error_rate(), 0.25);

    // 2 errors out of 4 connections = 50%
    metrics.record_error();
    assert_eq!(metrics.error_rate(), 0.5);
}

#[test]
fn test_streaming_metrics_health_ratio() {
    let mut metrics = StreamingMetrics::default();

    metrics.record_connection();
    metrics.record_connection();

    // No errors = 100% health
    assert_eq!(metrics.health_ratio(), 1.0);

    // 1 error = 50% health
    metrics.record_error();
    assert_eq!(metrics.health_ratio(), 0.5);
}

#[test]
fn test_streaming_metrics_reconnection_rate() {
    let mut metrics = StreamingMetrics::default();

    metrics.record_connection();
    metrics.record_connection();

    // No reconnections
    assert_eq!(metrics.reconnection_rate(), 0.0);

    // 1 reconnection out of 2 connections = 50%
    metrics.record_reconnection();
    assert_eq!(metrics.reconnection_rate(), 0.5);
}

#[test]
fn test_streaming_metrics_average_items_per_connection() {
    let mut metrics = StreamingMetrics::default();

    metrics.record_connection();
    metrics.record_item_sent();
    metrics.record_item_sent();

    metrics.record_connection();
    metrics.record_item_sent();

    // 3 items / 2 connections = 1.5
    assert_eq!(metrics.average_items_per_connection(), 1.5);
}
*/

/* DISABLED - StreamingMetrics removed in Phase 4.3 refactoring
#[tokio::test]
async fn test_prometheus_integration() {
    let prometheus_metrics = RipTideMetrics::new().expect("Failed to create metrics");
    let mut streaming_metrics = StreamingMetrics::default();

    // Simulate streaming activity
    streaming_metrics.record_connection();
    streaming_metrics.record_item_sent();
    streaming_metrics.record_item_sent();

    // Export to Prometheus
    streaming_metrics.to_prometheus(&prometheus_metrics);

    // Verify Prometheus metrics are updated
    let active_connections = prometheus_metrics.streaming_active_connections.get();
    assert_eq!(active_connections, 1.0);

    let total_connections = prometheus_metrics.streaming_total_connections.get();
    assert_eq!(total_connections, 1.0);
}
*/

/* DISABLED - StreamingMetrics removed in Phase 4.3 refactoring
#[tokio::test]
async fn test_metrics_under_load() {
    let mut metrics = StreamingMetrics::default();

    // Simulate high load
    for _ in 0..1000 {
        metrics.record_connection();
        for _ in 0..100 {
            metrics.record_item_sent();
        }
    }

    assert_eq!(metrics.active_connections, 1000);
    assert_eq!(metrics.total_connections, 1000);
    assert_eq!(metrics.total_items_sent, 100_000);
    assert_eq!(metrics.average_items_per_connection(), 100.0);
}

#[tokio::test]
async fn test_backpressure_detection() {
    let mut metrics = StreamingMetrics::default();

    metrics.record_connection();

    // Simulate backpressure scenario
    for _ in 0..100 {
        metrics.record_item_sent();
    }

    for _ in 0..20 {
        metrics.record_item_dropped();
    }

    // Delivery ratio should be 100 / 120 = 83.3%
    let delivery_ratio = metrics.delivery_ratio();
    assert!((delivery_ratio - 0.833).abs() < 0.01);

    // Should trigger warning if drop rate > 10%
    let drop_rate =
        metrics.items_dropped as f64 / (metrics.total_items_sent + metrics.items_dropped) as f64;
    assert!(
        drop_rate > 0.10,
        "Drop rate should trigger warning threshold"
    );
}

#[test]
fn test_metrics_zero_division_safety() {
    let metrics = StreamingMetrics::default();

    // All calculations should handle zero gracefully
    assert_eq!(metrics.delivery_ratio(), 1.0);
    assert_eq!(metrics.reconnection_rate(), 0.0);
    assert_eq!(metrics.health_ratio(), 1.0);
    assert_eq!(metrics.average_items_per_connection(), 0.0);
    assert_eq!(metrics.error_rate(), 0.0);
}
*/

/* DISABLED - StreamLifecycleManager removed in Phase 4.3 refactoring
#[tokio::test]
async fn test_lifecycle_integration() {
    use riptide_api::streaming::lifecycle::StreamLifecycleManager;

    let prometheus_metrics = Arc::new(RipTideMetrics::new().expect("Failed to create metrics"));
    let lifecycle = StreamLifecycleManager::new(prometheus_metrics.clone());

    // Simulate connection lifecycle
    lifecycle
        .connection_established("conn-1".to_string(), "sse".to_string())
        .await;

    // Give event handler time to process
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify metrics were updated
    let active_connections = prometheus_metrics.streaming_active_connections.get();
    assert!(active_connections > 0.0);
}
*/

/* DISABLED - StreamingMetrics type aliases removed in Phase 4.3 refactoring
#[test]
fn test_type_aliases() {
    use riptide_api::streaming::metrics::{NdjsonMetrics, SseMetrics, WebSocketMetrics};

    let sse: SseMetrics = StreamingMetrics::default();
    let ws: WebSocketMetrics = StreamingMetrics::default();
    let ndjson: NdjsonMetrics = StreamingMetrics::default();

    // All should be the same type with same behavior
    assert_eq!(sse.active_connections, 0);
    assert_eq!(ws.active_connections, 0);
    assert_eq!(ndjson.active_connections, 0);
}
*/

/* DISABLED - StreamingMetrics removed in Phase 4.3 refactoring
/// Performance test to ensure metrics collection doesn't impact throughput
#[test]
fn test_metrics_performance() {
    use std::time::Instant;

    let mut metrics = StreamingMetrics::default();
    let start = Instant::now();

    // Simulate rapid metric recording
    for _ in 0..100_000 {
        metrics.record_item_sent();
    }

    let duration = start.elapsed();

    // Recording 100k metrics should take less than 10ms
    assert!(
        duration.as_millis() < 10,
        "Metrics collection too slow: {:?}",
        duration
    );
}
*/

// NOTE: All tests in this file have been disabled because the streaming::metrics
// and streaming::lifecycle modules were removed during Phase 4.3 refactoring.
// The functionality has been moved to:
// - crates/riptide-facade/src/facades/streaming.rs (business logic)
// - crates/riptide-api/src/adapters/sse_transport.rs (SSE transport)
// - crates/riptide-api/src/adapters/websocket_transport.rs (WebSocket transport)
// - crates/riptide-api/src/metrics_transport.rs (transport-level metrics)
//
// New tests for streaming functionality should be added to:
// - crates/riptide-facade/tests/ for facade-level tests
// - crates/riptide-api/tests/ for adapter-level tests using the new structure
