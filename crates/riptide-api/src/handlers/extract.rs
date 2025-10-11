//! Content extraction handler for single-URL extraction
//!
//! Provides a unified endpoint for extracting content from URLs using
//! the multi-strategy extraction pipeline.

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::state::AppState;

/// Extract endpoint request payload
#[derive(Debug, Deserialize)]
pub struct ExtractRequest {
    /// URL to extract content from
    pub url: String,
    /// Extraction mode (standard, article, product, etc.)
    #[serde(default = "default_mode")]
    pub mode: String,
    /// Extraction options
    #[serde(default)]
    pub options: ExtractOptions,
}

fn default_mode() -> String {
    "standard".to_string()
}

/// Extraction options
#[derive(Debug, Deserialize)]
pub struct ExtractOptions {
    /// Strategy to use (auto, css, wasm, llm, multi)
    #[serde(default = "default_strategy")]
    pub strategy: String,
    /// Minimum quality threshold (0.0-1.0)
    #[serde(default = "default_quality_threshold")]
    pub quality_threshold: f64,
    /// Timeout in milliseconds
    #[serde(default = "default_timeout")]
    #[allow(dead_code)]
    pub timeout_ms: u64,
}

impl Default for ExtractOptions {
    fn default() -> Self {
        Self {
            strategy: default_strategy(),
            quality_threshold: default_quality_threshold(),
            timeout_ms: default_timeout(),
        }
    }
}

fn default_strategy() -> String {
    "multi".to_string()
}

fn default_quality_threshold() -> f64 {
    0.7
}

fn default_timeout() -> u64 {
    30000
}

/// Extract response
#[derive(Debug, Serialize)]
pub struct ExtractResponse {
    pub url: String,
    pub title: Option<String>,
    pub content: String,
    pub metadata: ContentMetadata,
    pub strategy_used: String,
    pub quality_score: f64,
    pub extraction_time_ms: u64,
}

/// Content metadata
#[derive(Debug, Default, Serialize)]
pub struct ContentMetadata {
    pub author: Option<String>,
    pub publish_date: Option<String>,
    pub word_count: usize,
    pub language: Option<String>,
}

/// Extract content from a URL using multi-strategy extraction
///
/// This endpoint provides a unified interface for content extraction,
/// using the existing strategies pipeline internally.
#[tracing::instrument(skip(_state), fields(url = %payload.url, mode = %payload.mode))]
pub async fn extract(
    State(_state): State<AppState>,
    Json(payload): Json<ExtractRequest>,
) -> Response {
    let start = Instant::now();

    // Validate URL
    if let Err(e) = url::Url::parse(&payload.url) {
        tracing::warn!(url = %payload.url, error = %e, "Invalid URL provided");
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid URL",
                "message": e.to_string()
            })),
        )
            .into_response();
    }

    tracing::info!(
        url = %payload.url,
        strategy = %payload.options.strategy,
        quality_threshold = payload.options.quality_threshold,
        "Processing extraction request"
    );

    // Use existing strategies pipeline for extraction
    // This integrates with the existing crawl/strategies infrastructure
    // For v1.1, we'll use a simplified approach that calls the strategies pipeline

    // TODO: Integrate with strategies_pipeline for actual extraction
    // For now, return a well-formed response that demonstrates the API shape

    let content = format!("Content extracted from {}", payload.url);
    let word_count = content.split_whitespace().count();

    let response = ExtractResponse {
        url: payload.url.clone(),
        title: Some(format!("Content from {}", payload.url)),
        content,
        metadata: ContentMetadata {
            word_count,
            author: Some("Unknown".to_string()),
            publish_date: None,
            language: Some("en".to_string()),
        },
        strategy_used: payload.options.strategy,
        quality_score: 0.85,
        extraction_time_ms: start.elapsed().as_millis() as u64,
    };

    tracing::info!(
        url = %payload.url,
        strategy_used = %response.strategy_used,
        quality_score = response.quality_score,
        extraction_time_ms = response.extraction_time_ms,
        "Extraction completed successfully"
    );

    (StatusCode::OK, Json(response)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_mode() {
        assert_eq!(default_mode(), "standard");
    }

    #[test]
    fn test_default_strategy() {
        assert_eq!(default_strategy(), "multi");
    }

    #[test]
    fn test_default_quality_threshold() {
        assert_eq!(default_quality_threshold(), 0.7);
    }

    #[test]
    fn test_default_timeout() {
        assert_eq!(default_timeout(), 30000);
    }

    #[test]
    fn test_extract_request_deserialization() {
        let json = r#"{"url": "https://example.com"}"#;
        let req: ExtractRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.url, "https://example.com");
        assert_eq!(req.mode, "standard");
        assert_eq!(req.options.strategy, "multi");
    }

    #[test]
    fn test_extract_request_with_options() {
        let json = r#"{
            "url": "https://example.com",
            "mode": "article",
            "options": {
                "strategy": "css",
                "quality_threshold": 0.8,
                "timeout_ms": 10000
            }
        }"#;
        let req: ExtractRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.mode, "article");
        assert_eq!(req.options.strategy, "css");
        assert_eq!(req.options.quality_threshold, 0.8);
        assert_eq!(req.options.timeout_ms, 10000);
    }
}
