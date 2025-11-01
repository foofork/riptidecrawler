//! NDJSON (Newline Delimited JSON) streaming implementation.
//!
//! This module provides streaming JSON responses where each line contains
//! a complete JSON object, allowing for efficient streaming of large datasets.

pub mod handlers;
pub mod helpers;
pub mod progress;
pub mod streaming;

// Re-export main public APIs
// NDJSON handlers are exposed through crate::handlers::streaming
// No re-exports needed here

#[cfg(test)]
mod tests {
    use super::*;
    use crate::health::HealthChecker;
    use crate::metrics::RipTideMetrics;
    use crate::state::AppState;
    use crate::streaming::ndjson::progress::OperationProgress;
    use crate::streaming::ndjson::streaming::NdjsonStreamingHandler;
    #[allow(unused_imports)]
    use axum::http::StatusCode;
    use std::sync::Arc;
    use std::time::Instant;

    /// Helper to create a test AppState
    /// This will try to create a full state, but will skip test if dependencies unavailable
    async fn try_create_test_state() -> Option<AppState> {
        let mut config = crate::state::AppConfig::default();

        // Use test-specific config if available
        if let Ok(redis_url) = std::env::var("TEST_REDIS_URL") {
            config.redis_url = redis_url;
        }
        if let Ok(wasm_path) = std::env::var("TEST_WASM_PATH") {
            config.wasm_path = wasm_path;
        }

        let metrics = Arc::new(RipTideMetrics::new().ok()?);
        let health_checker = Arc::new(HealthChecker::new());

        AppState::new(config, metrics, health_checker).await.ok()
    }

    #[tokio::test]
    async fn test_ndjson_handler_creation() {
        // Try to create test state - if dependencies not available, skip test
        if let Some(app) = try_create_test_state().await {
            let request_id = "test-123".to_string();
            let handler = NdjsonStreamingHandler::new(app, request_id.clone());

            // Handler should store the request_id
            // Note: request_id field is private, so we can't directly assert on it
            // This test mainly verifies the handler can be constructed
            drop(handler); // Use the handler to avoid unused variable warning
        } else {
            // Dependencies not available (Redis, WASM, etc.)
            // This is okay in test environments - skip the test
            eprintln!("Skipping test_ndjson_handler_creation: dependencies not available");
        }
    }

    #[test]
    fn test_estimate_completion() {
        let start_time = Instant::now();

        // Test with no progress
        assert!(progress::estimate_completion(start_time, 0, 10).is_none());

        // Test with some progress
        let result = progress::estimate_completion(start_time, 2, 10);
        assert!(result.is_some());
    }

    #[test]
    fn test_operation_progress_serialization() {
        let progress = OperationProgress {
            operation_id: "test-op".to_string(),
            operation_type: "batch_crawl".to_string(),
            started_at: "2024-01-01T00:00:00Z".to_string(),
            current_phase: "processing".to_string(),
            progress_percentage: 50.0,
            items_completed: 5,
            items_total: 10,
            estimated_completion: None,
            current_item: Some("https://example.com".to_string()),
        };

        let json = serde_json::to_string(&progress).unwrap();
        assert!(json.contains("\"progress_percentage\":50.0"));
        assert!(json.contains("\"items_completed\":5"));
    }
}
