//! Content extraction handler for single-URL extraction
//!
//! Provides a unified endpoint for extracting content from URLs using
//! the multi-strategy extraction pipeline and riptide-facade.

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::state::AppState;

/// Extract endpoint request payload
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
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
    /// Parser metadata for observability (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parser_metadata: Option<ParserMetadata>,
}

/// Content metadata
#[derive(Debug, Default, Serialize)]
pub struct ContentMetadata {
    pub author: Option<String>,
    pub publish_date: Option<String>,
    pub word_count: usize,
    pub language: Option<String>,
}

/// Parser metadata for observability
#[derive(Debug, Serialize)]
pub struct ParserMetadata {
    pub parser_used: String,
    pub confidence_score: f64,
    pub fallback_occurred: bool,
    pub parse_time_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extraction_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_error: Option<String>,
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
        return crate::errors::ApiError::invalid_url(&payload.url, e.to_string()).into_response();
    }

    tracing::info!(
        url = %payload.url,
        strategy = %payload.options.strategy,
        quality_threshold = payload.options.quality_threshold,
        "Processing extraction request via ExtractionFacade"
    );

    // First fetch the HTML content using HTTP client
    let html_result = state.http_client.get(&payload.url).send().await;

    let html = match html_result {
        Ok(response) => {
            if !response.status().is_success() {
                tracing::warn!(
                    url = %payload.url,
                    status = %response.status(),
                    "HTTP request failed"
                );
                return crate::errors::ApiError::fetch(
                    &payload.url,
                    format!("Server returned status: {}", response.status()),
                )
                .into_response();
            }
            match response.text().await {
                Ok(text) => text,
                Err(e) => {
                    tracing::error!(url = %payload.url, error = %e, "Failed to read response body");
                    return crate::errors::ApiError::fetch(
                        &payload.url,
                        format!("Failed to read response body: {}", e),
                    )
                    .into_response();
                }
            }
        }
        Err(e) => {
            tracing::error!(url = %payload.url, error = %e, "Failed to fetch URL");
            return crate::errors::ApiError::from(e).into_response();
        }
    };

    // Use ExtractionFacade based on strategy
    let extraction_start = Instant::now();

    let facade_result = match payload.options.strategy.to_lowercase().as_str() {
        "css" => {
            // Use HTML extraction with CSS
            let options = riptide_facade::facades::HtmlExtractionOptions {
                clean: true,
                include_metadata: true,
                extract_links: true,
                extract_images: false,
                as_markdown: payload.mode == "markdown",
                custom_selectors: None,
            };
            state
                .extraction_facade
                .extract_html(&html, &payload.url, options)
                .await
        }
        "wasm" => {
            // Use WASM extraction strategy
            state
                .extraction_facade
                .extract_with_strategy(
                    &html,
                    &payload.url,
                    riptide_facade::facades::ExtractionStrategy::Wasm,
                )
                .await
        }
        "multi" | "auto" => {
            // Use fallback chain for best quality
            let strategies = vec![
                riptide_facade::facades::ExtractionStrategy::Wasm,
                riptide_facade::facades::ExtractionStrategy::HtmlCss,
                riptide_facade::facades::ExtractionStrategy::Fallback,
            ];
            state
                .extraction_facade
                .extract_with_fallback(&html, &payload.url, &strategies)
                .await
        }
        _ => {
            tracing::warn!(
                strategy = %payload.options.strategy,
                "Unknown strategy, using default HTML extraction"
            );
            let options = riptide_facade::facades::HtmlExtractionOptions {
                clean: true,
                include_metadata: true,
                extract_links: true,
                extract_images: false,
                as_markdown: false,
                custom_selectors: None,
            };
            state
                .extraction_facade
                .extract_html(&html, &payload.url, options)
                .await
        }
    };

    let extraction_time_ms = extraction_start.elapsed().as_millis() as u64;

    match facade_result {
        Ok(extracted) => {
            // Convert parser metadata if available (note: facade doesn't have this yet)
            let parser_metadata = None; // Placeholder - facade would need to expose this

            let response = ExtractResponse {
                url: payload.url.clone(),
                title: extracted.title.clone(),
                content: extracted.text.clone(),
                metadata: ContentMetadata {
                    word_count: extracted.text.split_whitespace().count(),
                    author: extracted.metadata.get("author").cloned(),
                    publish_date: extracted.metadata.get("published_date").cloned(),
                    language: extracted.metadata.get("language").cloned(),
                },
                strategy_used: extracted.strategy_used.clone(),
                quality_score: extracted.confidence,
                extraction_time_ms,
                parser_metadata,
            };

            tracing::info!(
                url = %payload.url,
                strategy_used = %response.strategy_used,
                quality_score = response.quality_score,
                extraction_time_ms = response.extraction_time_ms,
                parser_metadata = ?response.parser_metadata,
                "Extraction completed successfully via ExtractionFacade"
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

            crate::errors::ApiError::extraction(e.to_string()).into_response()
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
