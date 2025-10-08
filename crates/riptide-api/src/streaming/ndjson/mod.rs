//! NDJSON (Newline Delimited JSON) streaming implementation.
//!
//! This module provides streaming JSON responses where each line contains
//! a complete JSON object, allowing for efficient streaming of large datasets.

pub mod handlers;
pub mod helpers;
pub mod progress;
pub mod streaming;

// Re-export main public APIs
pub use handlers::{crawl_stream, deepsearch_stream};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::streaming::ndjson::progress::OperationProgress;
    #[allow(unused_imports)]
    use axum::http::StatusCode;
    use std::time::Instant;

    #[tokio::test]
    #[ignore] // TODO: Fix AppState::new() test fixture - requires config, metrics, health_checker
    async fn test_ndjson_handler_creation() {
        // let app = AppState::new().await.expect("Failed to create AppState");
        // let request_id = "test-123".to_string();
        // let handler = NdjsonStreamingHandler::new(app, request_id.clone());
        // assert_eq!(handler.request_id, request_id);
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
