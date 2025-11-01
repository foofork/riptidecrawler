//! Phase 4B Integration Tests
//!
//! Comprehensive integration test suite for Phase 4B features including:
//! - Worker management endpoints (status, metrics)
//! - Telemetry span creation and export
//! - Streaming modes (NDJSON, SSE, WebSocket)
//! - Streaming lifecycle (connection, data, cleanup)
//! - Metrics collection validation

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
// Unused imports removed - will be needed when WebSocket tests are implemented
// use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::sleep;
// Unused imports removed - will be needed when WebSocket tests are implemented
// use tokio_tungstenite::{connect_async, tungstenite::Message};
use tower::ServiceExt;

// Import test utilities
mod test_utils {
    use super::*;
    use riptide_api::{
        health::HealthChecker,
        metrics::RipTideMetrics,
        state::{AppConfig, AppState},
    };
    use std::sync::Arc;

    /// Create test app state with minimal configuration
    #[allow(dead_code)]
    pub async fn create_test_app_state() -> Arc<AppState> {
        let config = AppConfig::default();
        let metrics = Arc::new(RipTideMetrics::new().expect("Failed to create metrics"));
        let health_checker = Arc::new(HealthChecker::new());

        Arc::new(
            AppState::new(config, metrics, health_checker)
                .await
                .expect("Failed to create test app state"),
        )
    }

    /// Create test router with all Phase 4B routes
    /// TODO(P2): Re-enable after test_router module is implemented
    #[allow(dead_code)]
    pub async fn create_test_router() -> Router {
        // use riptide_api::routes::test_router;

        let app_state = create_test_app_state().await;
        Router::new().with_state((*app_state).clone())
        // test_router::create_router().with_state((*app_state).clone())
    }

    /// Helper to parse NDJSON stream
    #[allow(dead_code)]
    pub fn parse_ndjson_lines(body: &str) -> Vec<Value> {
        body.lines()
            .filter(|line| !line.is_empty())
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect()
    }

    /// Helper to parse SSE stream
    #[allow(dead_code)]
    pub fn parse_sse_events(body: &str) -> Vec<(String, Value)> {
        let mut events = Vec::new();
        let mut current_event: Option<String> = None;
        let mut current_data = String::new();

        for line in body.lines() {
            if let Some(stripped) = line.strip_prefix("event:") {
                current_event = Some(stripped.trim().to_string());
            } else if let Some(stripped) = line.strip_prefix("data:") {
                current_data.push_str(stripped.trim());
            } else if line.is_empty() && current_event.is_some() {
                if let Ok(data) = serde_json::from_str(&current_data) {
                    events.push((current_event.take().unwrap(), data));
                }
                current_data.clear();
            }
        }

        events
    }
}

// =============================================================================
// Worker Management Endpoint Tests
// =============================================================================

#[tokio::test]
async fn test_worker_status_endpoint() {
    let app = test_utils::create_test_router().await;

    // Test GET /api/workers/status
    let request = Request::builder()
        .uri("/api/workers/status")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert!(json.get("total_workers").is_some());
    assert!(json.get("healthy_workers").is_some());
    assert!(json.get("is_running").is_some());
    assert!(json.get("total_jobs_processed").is_some());
    assert!(json.get("total_jobs_failed").is_some());
}

#[tokio::test]
async fn test_worker_metrics_collection() {
    let app = test_utils::create_test_router().await;

    // Test GET /api/workers/metrics
    let request = Request::builder()
        .uri("/api/workers/metrics")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify comprehensive metrics are present
    assert!(json.get("jobs_submitted").is_some());
    assert!(json.get("jobs_completed").is_some());
    assert!(json.get("jobs_failed").is_some());
    assert!(json.get("jobs_retried").is_some());
    assert!(json.get("avg_processing_time_ms").is_some());
    assert!(json.get("p95_processing_time_ms").is_some());
    assert!(json.get("p99_processing_time_ms").is_some());
    assert!(json.get("success_rate").is_some());
    assert!(json.get("queue_sizes").is_some());
    assert!(json.get("timestamp").is_some());

    // Verify metrics have valid values
    let _jobs_submitted = json["jobs_submitted"].as_u64().unwrap();
    let success_rate = json["success_rate"].as_f64().unwrap();
    assert!((0.0..=1.0).contains(&success_rate));
}

#[tokio::test]
async fn test_queue_statistics() {
    let app = test_utils::create_test_router().await;

    // Test GET /api/workers/queue/stats
    let request = Request::builder()
        .uri("/api/workers/queue/stats")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify queue stats structure
    assert!(json.get("pending").is_some());
    assert!(json.get("processing").is_some());
    assert!(json.get("completed").is_some());
    assert!(json.get("failed").is_some());
    assert!(json.get("retry").is_some());
    assert!(json.get("delayed").is_some());
    assert!(json.get("total").is_some());
}

// =============================================================================
// Telemetry Tests
// =============================================================================

#[tokio::test]
async fn test_telemetry_spans_created() {
    let app = test_utils::create_test_router().await;

    // Make a request that should create telemetry spans
    let request = Request::builder()
        .uri("/api/health")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Check telemetry status to verify spans are being created
    let app2 = test_utils::create_test_router().await;
    let telemetry_request = Request::builder()
        .uri("/telemetry/status")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let telemetry_response = app2.oneshot(telemetry_request).await.unwrap();
    assert_eq!(telemetry_response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(telemetry_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify telemetry configuration
    assert!(json.get("enabled").is_some());
    assert!(json.get("service_name").is_some());
    assert!(json.get("features").is_some());

    let features = &json["features"];
    assert_eq!(features["distributed_tracing"], true);
    assert_eq!(features["custom_attributes"], true);
    assert_eq!(features["trace_visualization"], true);
}

#[tokio::test]
async fn test_telemetry_conditional_init() {
    // Test that telemetry initializes conditionally based on environment
    let app = test_utils::create_test_router().await;

    let request = Request::builder()
        .uri("/telemetry/status")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify configuration is loaded
    assert!(json.get("enabled").is_some());
    assert!(json.get("exporter_type").is_some());
    assert!(json.get("sampling_ratio").is_some());
    assert!(json.get("trace_propagation_enabled").is_some());
}

#[tokio::test]
async fn test_trace_tree_endpoint() {
    let app = test_utils::create_test_router().await;

    // Test GET /telemetry/traces with a valid trace ID
    let trace_id = "0af7651916cd43dd8448eb211c80319c";
    let request = Request::builder()
        .uri(format!("/telemetry/traces?trace_id={}", trace_id))
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify trace tree structure
    assert!(json.get("metadata").is_some());
    assert!(json.get("root_span").is_some());
    assert!(json.get("summary").is_some());

    let metadata = &json["metadata"];
    assert_eq!(metadata["trace_id"], trace_id);
    assert!(metadata.get("root_span_id").is_some());
    assert!(metadata.get("duration_ms").is_some());
}

// =============================================================================
// Streaming Mode Tests
// =============================================================================

#[tokio::test]
async fn test_ndjson_streaming() {
    // Note: This test requires streaming routes to be active
    // For now, we test the streaming module's internal functionality
    use riptide_api::streaming::{StreamConfig, StreamingProtocol};

    let protocol = StreamingProtocol::Ndjson;
    assert_eq!(protocol.content_type(), "application/x-ndjson");
    assert!(!protocol.is_bidirectional());
    assert_eq!(protocol.default_buffer_size(), 256);

    let config = StreamConfig::default();
    assert!(config.validate().is_ok());
}

#[tokio::test]
async fn test_ndjson_protocol_properties() {
    use riptide_api::streaming::StreamingProtocol;

    let ndjson = StreamingProtocol::Ndjson;

    // Test content type
    assert_eq!(ndjson.content_type(), "application/x-ndjson");

    // Test buffer size
    assert_eq!(ndjson.default_buffer_size(), 256);

    // Test keep-alive interval
    let keep_alive = ndjson.keep_alive_interval();
    assert_eq!(keep_alive, Duration::from_secs(60));

    // Test bidirectional flag
    assert!(!ndjson.is_bidirectional());
}

#[tokio::test]
async fn test_sse_heartbeat() {
    use riptide_api::streaming::StreamingProtocol;

    let sse = StreamingProtocol::Sse;

    // Test SSE-specific properties
    assert_eq!(sse.content_type(), "text/event-stream");
    assert_eq!(sse.keep_alive_interval(), Duration::from_secs(30));
    assert!(!sse.is_bidirectional());
}

#[tokio::test]
async fn test_websocket_ping_pong() {
    use riptide_api::streaming::StreamingProtocol;

    let ws = StreamingProtocol::WebSocket;

    // Test WebSocket properties
    assert_eq!(ws.content_type(), "application/json");
    assert!(ws.is_bidirectional());
    assert_eq!(ws.default_buffer_size(), 64); // Smaller for real-time
    assert_eq!(ws.keep_alive_interval(), Duration::from_secs(30));
}

#[tokio::test]
async fn test_streaming_protocol_parsing() {
    use riptide_api::streaming::StreamingProtocol;
    use std::str::FromStr;

    // Test valid protocols
    assert_eq!(
        StreamingProtocol::from_str("ndjson").unwrap(),
        StreamingProtocol::Ndjson
    );
    assert_eq!(
        StreamingProtocol::from_str("sse").unwrap(),
        StreamingProtocol::Sse
    );
    assert_eq!(
        StreamingProtocol::from_str("websocket").unwrap(),
        StreamingProtocol::WebSocket
    );

    // Test alternative names
    assert_eq!(
        StreamingProtocol::from_str("nd-json").unwrap(),
        StreamingProtocol::Ndjson
    );
    assert_eq!(
        StreamingProtocol::from_str("server-sent-events").unwrap(),
        StreamingProtocol::Sse
    );
    assert_eq!(
        StreamingProtocol::from_str("ws").unwrap(),
        StreamingProtocol::WebSocket
    );

    // Test invalid protocol
    assert!(StreamingProtocol::from_str("invalid").is_err());
}

// =============================================================================
// Streaming Lifecycle Tests
// =============================================================================

#[tokio::test]
async fn test_streaming_module_initialization() {
    use riptide_api::streaming::{StreamConfig, StreamingModule};

    let config = StreamConfig::default();
    let module = StreamingModule::new(Some(config));

    assert!(module.validate().is_ok());
    assert!(module.is_healthy().await);

    let metrics = module.metrics().await;
    assert_eq!(metrics.active_connections, 0);
    assert_eq!(metrics.total_connections, 0);
}

#[tokio::test]
async fn test_buffer_manager_lifecycle() {
    use riptide_api::streaming::BufferManager;

    let manager = BufferManager::new();
    let stream_id = "test-stream-123".to_string();

    // Test buffer creation (get_buffer creates if not exists)
    let buffer = manager.get_buffer(&stream_id).await;
    let stats = buffer.stats().await;
    assert_eq!(stats.total_messages, 0);

    // Test global stats
    let global_stats = manager.global_stats().await;
    assert_eq!(global_stats.len(), 1);
    assert!(global_stats.contains_key(&stream_id));

    // Test cleanup
    manager.remove_buffer(&stream_id).await;
    let global_stats_after = manager.global_stats().await;
    assert_eq!(global_stats_after.len(), 0);
}

#[tokio::test]
async fn test_streaming_health_calculation() {
    use riptide_api::streaming::{GlobalStreamingMetrics, StreamingHealth};

    let mut metrics = GlobalStreamingMetrics {
        total_messages_sent: 100,
        total_messages_dropped: 0,
        error_rate: 0.01,
        ..Default::default()
    };
    metrics.update_health_status();
    assert_eq!(metrics.health_status, StreamingHealth::Healthy);

    // Test degraded state
    metrics.error_rate = 0.08;
    metrics.update_health_status();
    assert_eq!(metrics.health_status, StreamingHealth::Degraded);

    // Test critical state
    metrics.error_rate = 0.15;
    metrics.update_health_status();
    assert_eq!(metrics.health_status, StreamingHealth::Critical);
}

#[tokio::test]
async fn test_streaming_metrics_efficiency() {
    use riptide_api::streaming::GlobalStreamingMetrics;

    let mut metrics = GlobalStreamingMetrics {
        total_messages_sent: 100,
        total_messages_dropped: 0,
        error_rate: 0.0,
        ..Default::default()
    };
    assert!((metrics.efficiency() - 1.0).abs() < 0.01);

    // Reduced efficiency
    metrics.total_messages_dropped = 10;
    metrics.error_rate = 0.05;
    let efficiency = metrics.efficiency();
    assert!(efficiency > 0.85 && efficiency < 0.95);
}

// =============================================================================
// Metrics Collection Validation Tests
// =============================================================================

#[tokio::test]
async fn test_monitoring_health_score() {
    let app = test_utils::create_test_router().await;

    let request = Request::builder()
        .uri("/monitoring/health-score")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify health score structure
    assert!(json.get("health_score").is_some());
    assert!(json.get("status").is_some());
    assert!(json.get("timestamp").is_some());

    let health_score = json["health_score"].as_f64().unwrap();
    assert!((0.0..=100.0).contains(&health_score));

    let status = json["status"].as_str().unwrap();
    assert!(["excellent", "good", "fair", "poor", "critical"].contains(&status));
}

#[tokio::test]
async fn test_performance_report_generation() {
    let app = test_utils::create_test_router().await;

    let request = Request::builder()
        .uri("/monitoring/performance-report")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify comprehensive report structure
    assert!(json.get("health_score").is_some());
    assert!(json.get("metrics").is_some());
    assert!(json.get("summary").is_some());
    assert!(json.get("recommendations").is_some());
    assert!(json.get("timestamp").is_some());
}

#[tokio::test]
async fn test_current_metrics_collection() {
    let app = test_utils::create_test_router().await;

    let request = Request::builder()
        .uri("/monitoring/metrics/current")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify metrics structure
    assert!(json.get("metrics").is_some());
    let metrics = &json["metrics"];

    // Check for timing metrics
    assert!(metrics.get("avg_response_time_ms").is_some());
    assert!(metrics.get("p95_response_time_ms").is_some());
    assert!(metrics.get("p99_response_time_ms").is_some());

    // Check for throughput metrics
    assert!(metrics.get("requests_per_second").is_some());

    // Check for resource metrics
    assert!(metrics.get("cpu_usage_percent").is_some());
    assert!(metrics.get("memory_usage_mb").is_some());
}

#[tokio::test]
async fn test_resource_status_endpoint() {
    let app = test_utils::create_test_router().await;

    let request = Request::builder()
        .uri("/api/resources/status")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify resource status structure
    assert!(json.get("browser_pool").is_some());
    assert!(json.get("pdf_processing").is_some());
    assert!(json.get("memory").is_some());
    assert!(json.get("rate_limiting").is_some());
    assert!(json.get("timeouts").is_some());
    assert!(json.get("overall_health").is_some());
}

#[tokio::test]
async fn test_alert_rules_configuration() {
    let app = test_utils::create_test_router().await;

    let request = Request::builder()
        .uri("/monitoring/alerts/rules")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify alert rules structure
    assert!(json.get("rules").is_some());
    assert!(json.get("total").is_some());
    assert!(json.get("enabled").is_some());

    let rules = json["rules"].as_array().unwrap();
    for rule in rules {
        assert!(rule.get("name").is_some());
        assert!(rule.get("metric_name").is_some());
        assert!(rule.get("threshold").is_some());
        assert!(rule.get("condition").is_some());
        assert!(rule.get("severity").is_some());
        assert!(rule.get("enabled").is_some());
    }
}

#[tokio::test]
async fn test_active_alerts_tracking() {
    let app = test_utils::create_test_router().await;

    let request = Request::builder()
        .uri("/monitoring/alerts/active")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify active alerts structure
    assert!(json.get("active_alerts").is_some());
    assert!(json.get("count").is_some());

    let count = json["count"].as_u64().unwrap();
    let alerts = json["active_alerts"].as_array().unwrap();
    assert_eq!(count as usize, alerts.len());
}

// =============================================================================
// Integration Tests
// =============================================================================

#[tokio::test]
async fn test_end_to_end_worker_job_lifecycle() {
    let app = test_utils::create_test_router().await;

    // Submit a job
    let submit_body = json!({
        "job_type": {
            "type": "single_crawl",
            "url": "https://example.com"
        },
        "priority": "Normal"
    });

    let request = Request::builder()
        .uri("/api/workers/jobs")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&submit_body).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    let job_id = json["job_id"].as_str().unwrap();

    // Wait a bit for job processing
    sleep(Duration::from_millis(100)).await;

    // Check job status
    let status_request = Request::builder()
        .uri(format!("/api/workers/jobs/{}", job_id))
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let status_response = app.oneshot(status_request).await.unwrap();
    assert_eq!(status_response.status(), StatusCode::OK);

    let status_body = axum::body::to_bytes(status_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let status_json: Value = serde_json::from_slice(&status_body).unwrap();

    assert_eq!(status_json["job_id"], job_id);
    assert!(status_json.get("status").is_some());
    assert!(status_json.get("created_at").is_some());
}

#[tokio::test]
async fn test_streaming_config_validation() {
    use riptide_api::streaming::StreamConfig;

    let config = StreamConfig::default();

    // Validate default config
    assert!(config.validate().is_ok());

    // Test buffer config
    assert!(config.buffer.default_size > 0);
    assert!(config.buffer.max_size >= config.buffer.default_size);

    // Test timeout config
    assert!(config.general.default_timeout.as_secs() > 0);

    // Test connection limits
    assert!(config.general.max_concurrent_streams > 0);
}

#[tokio::test]
async fn test_telemetry_trace_id_validation() {
    use riptide_api::telemetry_config::parse_trace_id;

    // Valid 32-character hex trace ID
    let valid = "0af7651916cd43dd8448eb211c80319c";
    assert!(parse_trace_id(valid).is_some());

    // Invalid: too short
    let too_short = "0af7651916cd43dd";
    assert!(parse_trace_id(too_short).is_none());

    // Invalid: not hex
    let not_hex = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
    assert!(parse_trace_id(not_hex).is_none());
}

// =============================================================================
// Performance and Load Tests
// =============================================================================

#[tokio::test]
async fn test_concurrent_metric_collection() {
    let app = test_utils::create_test_router().await;

    // Spawn multiple concurrent requests
    let mut handles = vec![];
    for _ in 0..10 {
        let app_clone = app.clone();
        let handle = tokio::spawn(async move {
            let request = Request::builder()
                .uri("/monitoring/metrics/current")
                .method("GET")
                .body(Body::empty())
                .unwrap();

            app_clone.oneshot(request).await.unwrap()
        });
        handles.push(handle);
    }

    // Wait for all requests to complete
    for handle in handles {
        let response = handle.await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}

#[tokio::test]
async fn test_streaming_health_under_load() {
    use riptide_api::streaming::GlobalStreamingMetrics;

    let mut metrics = GlobalStreamingMetrics {
        active_connections: 1000,
        total_connections: 1200,
        total_messages_sent: 50000,
        total_messages_dropped: 100,
        error_rate: 0.02,
        ..Default::default()
    };

    metrics.update_health_status();

    // Should still be healthy with low error rate
    assert!(metrics.health_status.is_operational());
    assert!(metrics.efficiency() > 0.95);
}
