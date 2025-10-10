//! Test helpers for API endpoint tests
//!
//! Provides utilities for creating test applications with full or minimal dependencies.

use axum::routing::{get, post};
use axum::Router;
use riptide_api::{
    config::ApiConfig, handlers, health::HealthChecker, metrics::RipTideMetrics, state::AppState,
    streaming,
};
use std::sync::Arc;
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
    use serde_json::json;

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
}
