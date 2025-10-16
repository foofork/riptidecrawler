//! Content extraction handler for single-URL extraction
//!
//! Provides a unified endpoint for extracting content from URLs using
//! the multi-strategy extraction pipeline.

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use riptide_core::strategies::StrategyConfig;
use riptide_core::types::CrawlOptions;
use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::state::AppState;
use crate::strategies_pipeline::StrategiesPipelineOrchestrator;

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
#[axum::debug_handler]
#[tracing::instrument(skip(state), fields(url = %payload.url, mode = %payload.mode))]
pub async fn extract(
    State(state): State<AppState>,
    Json(payload): Json<ExtractRequest>,
) -> impl IntoResponse {
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

    // Create CrawlOptions from extract options
    let crawl_options = CrawlOptions {
        cache_mode: "standard".to_string(),
        concurrency: 1,
        ..Default::default()
    };

    // Parse extraction strategy from user input
    let extraction_strategy = match payload.options.strategy.to_lowercase().as_str() {
        "css" => riptide_core::strategies::ExtractionStrategy::Css,
        "regex" => riptide_core::strategies::ExtractionStrategy::Regex,
        "auto" => riptide_core::strategies::ExtractionStrategy::Auto,
        "wasm" => riptide_core::strategies::ExtractionStrategy::Wasm,
        "multi" => riptide_core::strategies::ExtractionStrategy::Auto, // Map multi to auto
        _ => {
            tracing::warn!(
                strategy = %payload.options.strategy,
                "Unknown strategy, defaulting to Auto"
            );
            riptide_core::strategies::ExtractionStrategy::Auto
        }
    };

    let strategy_config = StrategyConfig {
        extraction: extraction_strategy,
        enable_metrics: true,
        validate_schema: false,
    };

    // Create strategies pipeline orchestrator
    let pipeline =
        StrategiesPipelineOrchestrator::new(state.clone(), crawl_options, Some(strategy_config));

    // Execute extraction
    match pipeline.execute_single(&payload.url).await {
        Ok(result) => {
            // Access the extracted content fields correctly
            let extracted = &result.processed_content.extracted;
            let metadata = &result.processed_content.metadata;

            let response = ExtractResponse {
                url: payload.url.clone(),
                title: Some(extracted.title.clone()),
                content: extracted.content.clone(),
                metadata: ContentMetadata {
                    word_count: metadata
                        .word_count
                        .unwrap_or_else(|| extracted.content.split_whitespace().count()),
                    author: metadata.author.clone(),
                    publish_date: metadata.published_date.as_ref().map(|dt| dt.to_rfc3339()),
                    language: metadata.language.clone(),
                },
                strategy_used: extracted.strategy_used.clone(),
                quality_score: extracted.extraction_confidence,
                extraction_time_ms: result.processing_time_ms,
            };

            tracing::info!(
                url = %payload.url,
                strategy_used = %response.strategy_used,
                quality_score = response.quality_score,
                extraction_time_ms = response.extraction_time_ms,
                from_cache = result.from_cache,
                gate_decision = %result.gate_decision,
                "Extraction completed successfully"
            );

            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            tracing::error!(
                url = %payload.url,
                error = %e,
                elapsed_ms = start.elapsed().as_millis(),
                "Extraction failed"
            );

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Extraction failed",
                    "message": e.to_string(),
                    "url": payload.url
                })),
            )
                .into_response()
        }
    }
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
