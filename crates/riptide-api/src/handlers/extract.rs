//! Content extraction handler for single-URL extraction
//!
//! Provides a unified endpoint for extracting content from URLs using
//! the multi-strategy extraction pipeline and riptide-facade.

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::time::Instant;

use crate::state::AppState;

// Import HTTP DTOs from riptide-types (Phase 2C.1 - breaking circular dependency)
use riptide_types::ExtractRequest;

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
    let _start = Instant::now();

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

    let _html = match html_result {
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

    // Phase 2C.2: Call extraction facade (restored after circular dependency fix)
    let html_options = riptide_facade::facades::HtmlExtractionOptions {
        as_markdown: payload.options.strategy == "markdown",
        clean: true,
        include_metadata: true,
        extract_links: false,
        extract_images: false,
        custom_selectors: None,
    };

    match state
        .extraction_facade
        .extract_html(&_html, &payload.url, html_options)
        .await
    {
        Ok(extracted) => {
            tracing::info!(
                url = %payload.url,
                confidence = extracted.confidence,
                strategy = %extracted.strategy_used,
                text_length = extracted.text.len(),
                "Extraction completed successfully"
            );

            // Convert ExtractedData to ExtractResponse
            let metadata = riptide_types::ContentMetadata {
                author: extracted.metadata.get("author").cloned(),
                publish_date: extracted.metadata.get("publish_date").cloned(),
                word_count: extracted.text.split_whitespace().count(),
                language: extracted.metadata.get("language").cloned(),
            };

            let response = riptide_types::ExtractResponse {
                url: extracted.url.clone(),
                title: extracted.title.clone(),
                content: extracted.text.clone(),
                metadata,
                strategy_used: extracted.strategy_used.clone(),
                quality_score: extracted.confidence,
                extraction_time_ms: _start.elapsed().as_millis() as u64,
                parser_metadata: None,
            };

            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            tracing::error!(url = %payload.url, error = %e, "Extraction failed");
            crate::errors::ApiError::from(e).into_response()
        }
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
