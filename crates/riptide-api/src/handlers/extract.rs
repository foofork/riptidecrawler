//! Content extraction handler for single-URL extraction
//!
//! Provides a unified endpoint for extracting content from URLs using
//! the multi-strategy extraction pipeline and riptide-facade.

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::time::Instant;

use crate::context::ApplicationContext;

// Import HTTP DTOs from riptide-types (Phase 2C.1 - breaking circular dependency)
use riptide_types::{ExtractRequest, ExtractionMethod};

/// Parse strategy string to ExtractionMethod enum
pub(crate) fn parse_extraction_strategy(strategy_str: &str) -> Option<ExtractionMethod> {
    match strategy_str.to_lowercase().as_str() {
        "native" | "css" => Some(ExtractionMethod::HtmlCss),
        "wasm" => Some(ExtractionMethod::Wasm),
        "regex" => Some(ExtractionMethod::HtmlRegex),
        "fallback" => Some(ExtractionMethod::Fallback),
        "auto" | "multi" => None, // None means use default UnifiedExtractor
        "markdown" => None,       // Handled separately via as_markdown flag
        _ => None,
    }
}

/// Extract content from a URL using multi-strategy extraction
///
/// This endpoint provides a unified interface for content extraction,
/// delegating to the ExtractionFacade for all business logic.
#[axum::debug_handler]
#[tracing::instrument(skip(state), fields(url = %payload.url, mode = %payload.mode))]
pub async fn extract(
    State(state): State<ApplicationContext>,
    Json(payload): Json<ExtractRequest>,
) -> impl IntoResponse {
    let start = Instant::now();

    // Validate URL format (HTTP concern)
    if let Err(e) = url::Url::parse(&payload.url) {
        tracing::warn!(url = %payload.url, error = %e, "Invalid URL format");
        return crate::errors::ApiError::invalid_url(&payload.url, e.to_string()).into_response();
    }

    // Parse strategy from request
    let extraction_strategy = parse_extraction_strategy(&payload.options.strategy);

    // Build extraction options from request
    let html_options = riptide_facade::facades::HtmlExtractionOptions {
        as_markdown: payload.options.strategy == "markdown",
        clean: true,
        include_metadata: true,
        extract_links: false,
        extract_images: false,
        custom_selectors: None,
        extraction_strategy,
    };

    // Delegate to facade (handles fetch + extraction)
    match state
        .extraction_facade
        .extract_from_url(&payload.url, html_options)
        .await
    {
        Ok(extracted) => {
            // Include raw HTML only if explicitly requested
            let raw_html = if payload.options.include_html {
                extracted.raw_html.clone()
            } else {
                None
            };

            let response = riptide_types::ExtractResponse {
                url: extracted.url.clone(),
                title: extracted.title.clone(),
                content: extracted.text.clone(),
                metadata: riptide_types::ContentMetadata {
                    author: extracted.metadata.get("author").cloned(),
                    publish_date: extracted.metadata.get("publish_date").cloned(),
                    word_count: extracted.text.split_whitespace().count(),
                    language: extracted.metadata.get("language").cloned(),
                },
                strategy_used: extracted.strategy_used,
                quality_score: extracted.confidence,
                extraction_time_ms: start.elapsed().as_millis() as u64,
                parser_metadata: None,
                raw_html,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => crate::errors::ApiError::from(e).into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
