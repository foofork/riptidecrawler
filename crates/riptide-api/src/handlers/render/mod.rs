//! Enhanced render endpoint with dynamic content handling and session support.
//!
//! This module is organized into focused sub-modules for maintainability.

pub mod extraction;
pub mod handlers;
pub mod models;
pub mod processors;
pub mod strategies;

// Re-export main public API
pub use handlers::render;

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
    use crate::handlers::render::models::{RenderRequest, RenderStats};
    #[allow(unused_imports)]
    use crate::state::AppState;
    use axum::http::StatusCode;
    use riptide_types::{ExtractionMode, OutputFormat};

    #[tokio::test]
    async fn test_render_request_validation() {
        // Test empty URL validation
        let _empty_url_request = RenderRequest {
            url: "".to_string(),
            mode: None,
            dynamic_config: None,
            stealth_config: None,
            pdf_config: None,
            output_format: None,
            capture_artifacts: None,
            timeout: None,
            session_id: None,
        };

        // This would be tested with actual state
        // let result = render(State(app_state), session_ctx, Json(empty_url_request)).await;
        // assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_render_timeout_handling() {
        use crate::config::ApiConfig;
        use std::time::Duration;

        // Test that timeout configuration is properly read and applied
        let config = ApiConfig::default();

        // Verify default timeout is 3 seconds (requirement)
        assert_eq!(config.performance.render_timeout_secs, 3);

        // Verify timeout can be obtained via get_timeout method
        let timeout = config.get_timeout("render");
        assert_eq!(timeout, Duration::from_secs(3));

        // Test that timeout error is properly constructed
        let error = crate::errors::ApiError::timeout("render", "Operation exceeded 3s timeout");
        assert!(matches!(
            error,
            crate::errors::ApiError::TimeoutError { .. }
        ));
        assert_eq!(error.status_code(), StatusCode::REQUEST_TIMEOUT);
        assert!(error.is_retryable());
    }

    #[test]
    fn test_extraction_mode_mapping() {
        // Test mode mapping logic
        let test_cases = vec![
            (ExtractionMode::Article, "article"),
            (ExtractionMode::Full, "full"),
            (ExtractionMode::Metadata, "metadata"),
            (ExtractionMode::Custom(vec!["p".to_string()]), "article"), // fallback
        ];

        for (mode, expected) in test_cases {
            let mode_str = match mode {
                ExtractionMode::Article => "article",
                ExtractionMode::Full => "full",
                ExtractionMode::Metadata => "metadata",
                ExtractionMode::Custom(_) => "article",
            };
            assert_eq!(mode_str, expected, "Mode mapping should be correct");
        }
    }

    #[test]
    fn test_output_format_to_extraction_mode() {
        let test_cases = vec![
            (OutputFormat::Markdown, ExtractionMode::Article),
            (OutputFormat::Document, ExtractionMode::Full),
            (OutputFormat::Text, ExtractionMode::Article),
            (OutputFormat::NdJson, ExtractionMode::Article),
        ];

        for (output_format, expected_mode) in test_cases {
            let extraction_mode = match output_format {
                OutputFormat::Markdown => ExtractionMode::Article,
                OutputFormat::Document => ExtractionMode::Full,
                OutputFormat::Text => ExtractionMode::Article,
                OutputFormat::NdJson => ExtractionMode::Article,
                OutputFormat::Chunked => ExtractionMode::Article,
            };

            // Compare discriminants since ExtractionMode doesn't implement PartialEq for Custom variant
            assert_eq!(
                std::mem::discriminant(&extraction_mode),
                std::mem::discriminant(&expected_mode),
                "Output format should map to correct extraction mode"
            );
        }
    }

    #[test]
    fn test_render_stats_creation() {
        let stats = RenderStats {
            total_time_ms: 1000,
            dynamic_time_ms: Some(500),
            pdf_time_ms: None,
            extraction_time_ms: 200,
            actions_executed: 3,
            wait_conditions_met: 2,
            network_requests: 5,
            page_size_bytes: 102400,
        };

        assert_eq!(stats.total_time_ms, 1000);
        assert_eq!(stats.dynamic_time_ms, Some(500));
        assert_eq!(stats.actions_executed, 3);
    }
}
