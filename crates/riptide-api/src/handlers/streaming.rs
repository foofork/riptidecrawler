//! Streaming endpoint handlers for real-time data delivery.
//!
//! This module provides HTTP handlers for NDJSON streaming endpoints,
//! supporting both batch crawling and deep search operations with
//! real-time progress updates and backpressure handling.
#![allow(dead_code)]

use crate::models::{CrawlBody, DeepSearchBody};
use crate::state::AppState;
use crate::streaming::ndjson::streaming::NdjsonStreamingHandler;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use std::time::Instant;
use tracing::{error, info};
use uuid::Uuid;

/// Handler for NDJSON crawl streaming endpoint
///
/// Streams crawl results in real-time using Newline Delimited JSON format.
/// Each line contains a JSON object representing a single crawl result.
///
/// # Path
/// POST /v1/crawl/stream
///
/// # Request Body
/// ```json
/// {
///   "urls": ["https://example.com", "https://example.org"],
///   "options": {
///     "enable_cache": true,
///     "cache_mode": "smart",
///     "enable_gating": true
///   }
/// }
/// ```
///
/// # Response
/// Content-Type: application/x-ndjson
/// Each line is a JSON object with crawl results and progress
pub async fn crawl_stream(State(app): State<AppState>, Json(body): Json<CrawlBody>) -> Response {
    let request_id = Uuid::new_v4().to_string();
    let start_time = Instant::now();

    info!(
        request_id = %request_id,
        url_count = body.urls.len(),
        "NDJSON crawl stream request received"
    );

    // Validate request
    if body.urls.is_empty() {
        error!(request_id = %request_id, "Empty URLs list");
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "urls list cannot be empty"
            })),
        )
            .into_response();
    }

    if body.urls.len() > 1000 {
        error!(request_id = %request_id, url_count = body.urls.len(), "Too many URLs");
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "urls list cannot exceed 1000 items"
            })),
        )
            .into_response();
    }

    // Create streaming handler with buffer limit based on URL count
    let buffer_limit = (body.urls.len() * 2).clamp(256, 2048);
    let handler = NdjsonStreamingHandler::new_optimized(app, request_id.clone(), buffer_limit);

    // Execute streaming with proper error handling
    match handler.handle_crawl_stream(body, start_time).await {
        Ok(response) => {
            info!(
                request_id = %request_id,
                elapsed_ms = start_time.elapsed().as_millis(),
                "NDJSON crawl stream initiated successfully"
            );
            response
        }
        Err(e) => {
            error!(
                request_id = %request_id,
                error = %e,
                "NDJSON crawl stream initiation failed"
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to initiate stream: {}", e)
                })),
            )
                .into_response()
        }
    }
}

/// Handler for NDJSON deep search streaming endpoint
///
/// Performs web search and optionally streams crawled content for each result.
/// Uses Serper API for search and streams results in real-time.
///
/// # Path
/// POST /v1/deepsearch/stream
///
/// # Request Body
/// ```json
/// {
///   "query": "rust web scraping",
///   "limit": 10,
///   "include_content": true,
///   "crawl_options": {
///     "enable_cache": true
///   }
/// }
/// ```
///
/// # Response
/// Content-Type: application/x-ndjson
/// Each line contains search results and optionally crawled content
pub async fn deepsearch_stream(
    State(app): State<AppState>,
    Json(body): Json<DeepSearchBody>,
) -> Response {
    let request_id = Uuid::new_v4().to_string();
    let start_time = Instant::now();

    info!(
        request_id = %request_id,
        query = %body.query,
        limit = body.limit,
        include_content = body.include_content.unwrap_or(true),
        "NDJSON deep search stream request received"
    );

    // Validate request
    if body.query.trim().is_empty() {
        error!(request_id = %request_id, "Empty search query");
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "query cannot be empty"
            })),
        )
            .into_response();
    }

    let limit = body.limit.unwrap_or(10).min(50);
    if limit == 0 {
        error!(request_id = %request_id, "Invalid limit");
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "limit must be greater than 0"
            })),
        )
            .into_response();
    }

    // Check for Serper API key
    if std::env::var("SERPER_API_KEY").is_err() {
        error!(request_id = %request_id, "SERPER_API_KEY not configured");
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "error": "Deep search service not configured. SERPER_API_KEY environment variable required."
            })),
        )
            .into_response();
    }

    // Create streaming handler with buffer limit based on search limit
    let buffer_limit = (limit as usize * 2).clamp(256, 512);
    let handler = NdjsonStreamingHandler::new_optimized(app, request_id.clone(), buffer_limit);

    // Execute streaming with proper error handling
    match handler.handle_deepsearch_stream(body, start_time).await {
        Ok(response) => {
            info!(
                request_id = %request_id,
                elapsed_ms = start_time.elapsed().as_millis(),
                "NDJSON deep search stream initiated successfully"
            );
            response
        }
        Err(e) => {
            error!(
                request_id = %request_id,
                error = %e,
                "NDJSON deep search stream initiation failed"
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to initiate stream: {}", e)
                })),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CrawlOptions;
    use crate::tests::test_helpers::AppStateBuilder;

    #[tokio::test]
    #[ignore = "Requires Redis connection"]
    async fn test_crawl_stream_empty_urls() {
        let app = AppStateBuilder::new()
            .build()
            .await
            .expect("Failed to create AppState");

        let body = CrawlBody {
            urls: vec![],
            options: Some(CrawlOptions::default()),
        };

        let response = crawl_stream(State(app), Json(body)).await;
        let status = response.status();

        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    #[ignore = "Requires Redis connection"]
    async fn test_crawl_stream_too_many_urls() {
        let app = AppStateBuilder::new()
            .build()
            .await
            .expect("Failed to create AppState");

        let body = CrawlBody {
            urls: vec!["https://example.com".to_string(); 1001],
            options: Some(CrawlOptions::default()),
        };

        let response = crawl_stream(State(app), Json(body)).await;
        let status = response.status();

        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    #[ignore = "Requires Redis connection"]
    async fn test_deepsearch_stream_empty_query() {
        let app = AppStateBuilder::new()
            .build()
            .await
            .expect("Failed to create AppState");

        let body = DeepSearchBody {
            query: "".to_string(),
            limit: Some(10),
            country: None,
            locale: None,
            include_content: Some(true),
            crawl_options: None,
        };

        let response = deepsearch_stream(State(app), Json(body)).await;
        let status = response.status();

        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    #[ignore = "Requires Redis connection"]
    async fn test_deepsearch_stream_invalid_limit() {
        let app = AppStateBuilder::new()
            .build()
            .await
            .expect("Failed to create AppState");

        let body = DeepSearchBody {
            query: "test query".to_string(),
            limit: Some(0),
            country: None,
            locale: None,
            include_content: Some(true),
            crawl_options: None,
        };

        let response = deepsearch_stream(State(app), Json(body)).await;
        let status = response.status();

        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
}
