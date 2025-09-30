//! Axum endpoint handlers for NDJSON streaming operations.
//!
//! This module contains the HTTP handlers for crawl and deepsearch streaming endpoints.

use super::streaming::NdjsonStreamingHandler;
use crate::errors::ApiError;
use crate::models::{CrawlBody, DeepSearchBody};
use crate::state::AppState;
use crate::validation::{validate_crawl_request, validate_deepsearch_request};
use axum::body::Body;
use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::time::Instant;
use tracing::{error, info};
use uuid::Uuid;

/// NDJSON streaming handler for crawl operations
///
/// Key features:
/// - TTFB < 500ms with warm cache
/// - Buffer management with 65536 bytes limit
/// - Streaming results as they complete (no batching)
/// - Zero unwrap/expect error handling
pub async fn crawl_stream(
    State(app): State<AppState>,
    Json(body): Json<CrawlBody>,
) -> impl IntoResponse {
    let start_time = Instant::now();
    let request_id = Uuid::new_v4().to_string();

    info!(
        request_id = %request_id,
        url_count = body.urls.len(),
        cache_mode = body.options.as_ref().map(|o| &o.cache_mode),
        "Received NDJSON crawl request"
    );

    // Validate the request early to fail fast - zero unwrap approach
    let validation_result = validate_crawl_request(&body);
    if let Err(e) = validation_result {
        return create_error_response(e);
    }

    // Create streaming handler with enhanced configuration
    let streaming_handler = NdjsonStreamingHandler::new_optimized(
        app.clone(),
        request_id.clone(),
        65536 // 65536 bytes buffer limit as specified
    );

    // Handle streaming with zero-error approach
    let stream_result = streaming_handler
        .handle_crawl_stream(body, start_time)
        .await;

    match stream_result {
        Ok(response) => response,
        Err(e) => {
            error!(
                request_id = %request_id,
                error = %e,
                "NDJSON crawl stream failed"
            );
            create_error_response(ApiError::from(e))
        }
    }
}

/// NDJSON streaming handler for deep search operations
///
/// Key features:
/// - Search integration with real-time streaming
/// - Content extraction streaming
/// - TTFB optimization for search operations
/// - Zero unwrap/expect error handling
pub async fn deepsearch_stream(
    State(app): State<AppState>,
    Json(body): Json<DeepSearchBody>,
) -> impl IntoResponse {
    let start_time = Instant::now();
    let request_id = Uuid::new_v4().to_string();

    info!(
        request_id = %request_id,
        query = %body.query,
        limit = body.limit,
        "Received NDJSON deep search request"
    );

    // Validate the request early to fail fast - zero unwrap approach
    let validation_result = validate_deepsearch_request(&body);
    if let Err(e) = validation_result {
        return create_error_response(e);
    }

    // Create streaming handler with enhanced configuration
    let streaming_handler = NdjsonStreamingHandler::new_optimized(
        app.clone(),
        request_id.clone(),
        65536 // 65536 bytes buffer limit as specified
    );

    // Handle streaming with zero-error approach
    let stream_result = streaming_handler
        .handle_deepsearch_stream(body, start_time)
        .await;

    match stream_result {
        Ok(response) => response,
        Err(e) => {
            error!(
                request_id = %request_id,
                error = %e,
                "NDJSON deep search stream failed"
            );
            create_error_response(ApiError::from(e))
        }
    }
}

/// Create an error response for failed request validation
fn create_error_response(error: ApiError) -> Response {
    let error_json = serde_json::json!({
        "error": {
            "type": "validation_error",
            "message": error.to_string(),
            "retryable": false
        }
    });

    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header("Content-Type", "application/json")
        .body(Body::from(error_json.to_string()))
        .unwrap_or_else(|_| {
            // If we can't even build this simple error response, return a minimal one
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap_or_else(|e| {
                    error!(error = %e, "Failed to build minimal error response, returning empty response");
                    Response::new(Body::empty())
                })
        })
}