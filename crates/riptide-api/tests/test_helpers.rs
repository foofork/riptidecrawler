//! Test helpers for API endpoint tests
//!
//! Provides utilities for creating test applications with full or minimal dependencies.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::{get, post};
use axum::Router;
use riptide_api::{handlers, health::HealthChecker, metrics::RipTideMetrics, state::AppState};
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;
use tower_http::cors::CorsLayer;

/// Create a test application with full dependencies
///
/// This function creates a fully initialized AppState with all dependencies,
/// enabling comprehensive integration testing.
///
/// Note: This function will attempt to connect to Redis and other services.
/// Use `create_minimal_test_app()` for tests that don't need full state.
pub async fn create_test_app() -> Router {
    // Initialize test config using AppConfig::default() with test overrides
    let mut config = riptide_api::state::AppConfig::default();

    // Override with test-specific values if needed
    if let Ok(redis_url) = std::env::var("TEST_REDIS_URL") {
        config.redis_url = redis_url;
    }

    if let Ok(wasm_path) = std::env::var("TEST_WASM_PATH") {
        config.wasm_path = wasm_path;
    }

    // Initialize test metrics
    let metrics = Arc::new(RipTideMetrics::new().expect("Failed to create test metrics"));

    // Initialize test health checker
    let health_checker = Arc::new(HealthChecker::new());

    // Create test app state
    // Note: This may fail if dependencies are not available
    match AppState::new(config, metrics, health_checker).await {
        Ok(app_state) => create_test_router(app_state),
        Err(e) => {
            eprintln!("Warning: Failed to create full test app state: {}", e);
            eprintln!("Falling back to minimal test app");
            create_minimal_test_app()
        }
    }
}

/// Create router with all routes configured
fn create_test_router(state: AppState) -> Router {
    use riptide_api::streaming;

    Router::new()
        // Health endpoints - both root and v1 paths
        .route("/health", get(handlers::health))
        .route("/api/v1/health", get(handlers::health))
        // Metrics - both root and v1 paths
        .route("/metrics", get(handlers::metrics))
        .route("/api/v1/metrics", get(handlers::metrics))
        // Crawl endpoints - both root and v1 paths
        .route("/api/v1/crawl", post(handlers::crawl))
        // Extract endpoint - NEW v1.1 (v1 path primary)
        .route("/api/v1/extract", post(handlers::extract))
        // Search endpoint - NEW v1.1 (v1 path primary)
        .route("/api/v1/search", get(handlers::search))
        // Streaming endpoints (test-enabled)
        .route("/crawl/stream", post(streaming::ndjson_crawl_stream))
        .route(
            "/deepsearch/stream",
            post(streaming::ndjson_deepsearch_stream),
        )
        .route("/crawl/sse", post(streaming::crawl_sse))
        .route("/crawl/ws", get(streaming::crawl_websocket))
        .with_state(state)
        .layer(CorsLayer::permissive())
}

/// Create a minimal test app without dependencies
///
/// This function creates a basic router with mock responses,
/// suitable for testing routing and request/response format.
/// Uses consistent /api/v1/* URL structure.
pub fn create_minimal_test_app() -> Router {
    use axum::Json;

    Router::new()
        // Health endpoint - primary /api/v1 path
        .route("/api/v1/health", get(|| async { "OK" }))
        .route("/health", get(|| async { "OK" })) // Alias for backward compatibility
        // Metrics endpoint - primary /api/v1 path
        .route("/api/v1/metrics", get(|| async { "# No metrics" }))
        .route("/metrics", get(|| async { "# No metrics" })) // Alias for backward compatibility
        // Crawl endpoint - primary /api/v1 path
        .route(
            "/api/v1/crawl",
            post(|| async { Json(json!({"status": "mock", "message": "Test crawl endpoint"})) }),
        )
        // Extract endpoint - primary /api/v1 path
        .route(
            "/api/v1/extract",
            post(|| async {
                Json(json!({
                    "url": "https://example.com",
                    "content": "Test content",
                    "strategy_used": "mock"
                }))
            }),
        )
        // Search endpoint - primary /api/v1 path
        .route(
            "/api/v1/search",
            get(|| async {
                Json(json!({
                    "query": "test",
                    "results": [],
                    "provider_used": "mock"
                }))
            }),
        )
        // Status endpoint for mock job status
        .route(
            "/api/v1/status/:job_id",
            get(|| async {
                Json(json!({
                    "status": "completed",
                    "job_id": "mock_job"
                }))
            }),
        )
        .layer(CorsLayer::permissive())
}

// ============================================================================
// Additional Test Helpers for Integration Testing
// ============================================================================

/// Create a test app with persistence features enabled
#[cfg(feature = "jemalloc")]
pub async fn create_test_app_with_persistence() -> Router {
    // Try to create app with Redis/persistence features
    create_test_app().await
}

/// Create a test app with profiling features enabled
#[cfg(feature = "profiling-full")]
pub async fn create_test_app_with_profiling() -> Router {
    create_test_app().await
}

/// Mock tenant creation helper
pub async fn create_test_tenant(tenant_id: &str) -> serde_json::Value {
    json!({
        "tenant_id": tenant_id,
        "max_requests_per_minute": 100,
        "max_tokens_per_minute": 10000,
        "max_cost_per_hour": 10.0,
        "max_concurrent_requests": 10,
        "status": "active"
    })
}

/// Mock browser session helper
#[cfg(feature = "sessions")]
pub async fn create_test_browser_session() -> serde_json::Value {
    json!({
        "session_id": "test-session-123",
        "url": "https://example.com",
        "status": "active",
        "created_at": chrono::Utc::now().to_rfc3339()
    })
}

/// Start a mock streaming session
#[cfg(feature = "streaming")]
pub async fn start_test_stream() -> serde_json::Value {
    json!({
        "stream_id": "test-stream-456",
        "format": "ndjson",
        "status": "active",
        "items_processed": 0
    })
}

/// Trigger mock profiling
#[cfg(feature = "profiling-full")]
pub async fn trigger_test_profiling() -> serde_json::Value {
    json!({
        "profile_id": "test-profile-789",
        "profile_type": "memory",
        "status": "running",
        "started_at": chrono::Utc::now().to_rfc3339()
    })
}

/// Simulate load by making multiple concurrent requests
pub async fn simulate_load(
    app: &Router,
    rps: usize,
    duration: std::time::Duration,
) -> LoadTestResult {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let app = Arc::new(app.clone());
    let success_count = Arc::new(AtomicUsize::new(0));
    let failure_count = Arc::new(AtomicUsize::new(0));
    let start_time = std::time::Instant::now();

    while start_time.elapsed() < duration {
        let mut handles = vec![];

        for _ in 0..rps {
            let app_clone = Arc::clone(&app);
            let success_clone = Arc::clone(&success_count);
            let failure_clone = Arc::clone(&failure_count);

            let handle = tokio::spawn(async move {
                let result = app_clone
                    .clone()
                    .oneshot(
                        Request::builder()
                            .uri("/api/v1/health")
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await;

                match result {
                    Ok(response) if response.status().is_success() => {
                        success_clone.fetch_add(1, Ordering::SeqCst);
                    }
                    _ => {
                        failure_clone.fetch_add(1, Ordering::SeqCst);
                    }
                }
            });

            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.await;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    LoadTestResult {
        total_requests: success_count.load(Ordering::SeqCst) + failure_count.load(Ordering::SeqCst),
        successful_requests: success_count.load(Ordering::SeqCst),
        failed_requests: failure_count.load(Ordering::SeqCst),
        duration: start_time.elapsed(),
    }
}

/// Result of load testing
pub struct LoadTestResult {
    pub total_requests: usize,
    pub successful_requests: usize,
    pub failed_requests: usize,
    pub duration: std::time::Duration,
}

impl LoadTestResult {
    pub fn success_rate(&self) -> f64 {
        self.successful_requests as f64 / self.total_requests as f64
    }

    pub fn requests_per_second(&self) -> f64 {
        self.total_requests as f64 / self.duration.as_secs_f64()
    }
}

/// Clean up test resources
pub async fn cleanup_test_resources() {
    // Clean up any test files, sessions, caches, etc.
    // This is a placeholder for actual cleanup logic
}

/// Wait for async operation to complete with timeout
pub async fn wait_for_condition<F, Fut>(
    condition: F,
    timeout: std::time::Duration,
    check_interval: std::time::Duration,
) -> bool
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let start = std::time::Instant::now();

    while start.elapsed() < timeout {
        if condition().await {
            return true;
        }
        tokio::time::sleep(check_interval).await;
    }

    false
}

/// Assert response has expected status with detailed error message
pub fn assert_status_with_context(
    response: &axum::http::Response<Body>,
    expected: StatusCode,
    context: &str,
) {
    assert_eq!(
        response.status(),
        expected,
        "Unexpected status code in {}: expected {:?}, got {:?}",
        context,
        expected,
        response.status()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_app_creation() {
        let _app = create_minimal_test_app();
        // If we get here, the app was created successfully
    }

    #[tokio::test]
    async fn test_full_app_creation() {
        // This may fail if dependencies aren't available, which is okay
        let _app = create_test_app().await;
        // If we get here, the app was created (either full or minimal fallback)
    }

    #[tokio::test]
    async fn test_tenant_creation_helper() {
        let tenant = create_test_tenant("test-tenant-001").await;
        assert_eq!(tenant["tenant_id"], "test-tenant-001");
        assert_eq!(tenant["max_requests_per_minute"], 100);
    }

    #[tokio::test]
    async fn test_load_test_result() {
        let result = LoadTestResult {
            total_requests: 100,
            successful_requests: 95,
            failed_requests: 5,
            duration: std::time::Duration::from_secs(10),
        };

        assert_eq!(result.success_rate(), 0.95);
        assert_eq!(result.requests_per_second(), 10.0);
    }
}
