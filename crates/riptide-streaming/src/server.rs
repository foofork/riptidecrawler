//! HTTP Server for RipTide Streaming API
//!
//! Provides endpoints for streaming extraction results via NDJSON.

use crate::StreamingCoordinator;
use anyhow::Context;
use axum::{
    body::Body,
    extract::{Json, State},
    http::{header, Response as HttpResponse, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::trace::TraceLayer;

/// Server state shared across handlers
#[derive(Clone)]
pub struct ServerState {
    pub coordinator: Arc<RwLock<StreamingCoordinator>>,
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            coordinator: Arc::new(RwLock::new(StreamingCoordinator::new())),
        }
    }
}

/// Request for /crawl/stream endpoint
#[derive(Debug, Deserialize)]
pub struct CrawlStreamRequest {
    pub urls: Vec<String>,
    #[serde(default)]
    pub options: CrawlOptions,
}

#[derive(Debug, Deserialize, Default)]
pub struct CrawlOptions {
    #[serde(default = "default_cache_mode")]
    pub cache_mode: String,
    #[serde(default = "default_concurrency")]
    pub concurrency: u32,
    #[serde(default = "default_true")]
    pub stream: bool,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    #[serde(default = "default_user_agent")]
    pub user_agent: String,
    #[serde(default)]
    pub respect_robots: bool,
}

fn default_cache_mode() -> String {
    "disabled".to_string()
}
fn default_concurrency() -> u32 {
    5
}
fn default_true() -> bool {
    true
}
fn default_timeout() -> u64 {
    30000
}
fn default_user_agent() -> String {
    "RipTide/1.0".to_string()
}

/// Request for /deepsearch/stream endpoint
#[derive(Debug, Deserialize)]
pub struct DeepSearchRequest {
    pub query: String,
    #[serde(default = "default_limit")]
    pub limit: u32,
    #[serde(default)]
    pub include_content: bool,
    #[serde(default)]
    pub crawl_options: CrawlOptions,
}

fn default_limit() -> u32 {
    10
}

/// Create HTTP server with streaming endpoints
pub fn create_server(state: ServerState) -> Router {
    Router::new()
        .route("/crawl/stream", post(handle_crawl_stream))
        .route("/deepsearch/stream", post(handle_deepsearch_stream))
        .route("/healthz", axum::routing::get(health_check))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}

/// Health check endpoint
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "riptide-streaming",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Handle /crawl/stream requests
async fn handle_crawl_stream(
    State(_state): State<ServerState>,
    Json(request): Json<CrawlStreamRequest>,
) -> Response {
    // Validate request
    if request.urls.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "No URLs provided");
    }

    // Create streaming response
    let extraction_id = uuid::Uuid::new_v4().to_string();

    // Create mock streaming response
    let body_text = create_mock_ndjson_stream(&request.urls, &extraction_id);

    // Build response with proper error handling
    match build_streaming_response(body_text, extraction_id) {
        Ok(response) => response,
        Err(error_response) => error_response,
    }
}

/// Handle /deepsearch/stream requests
async fn handle_deepsearch_stream(
    State(_state): State<ServerState>,
    Json(request): Json<DeepSearchRequest>,
) -> Response {
    // Validate request
    if request.query.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "No query provided");
    }

    // Create streaming response
    let extraction_id = uuid::Uuid::new_v4().to_string();

    // Create mock search results
    let body_text = create_mock_search_stream(&request.query, &extraction_id, request.limit);

    // Build response with proper error handling
    match build_streaming_response(body_text, extraction_id) {
        Ok(response) => response,
        Err(error_response) => error_response,
    }
}

/// Create error response
fn error_response(status: StatusCode, message: &str) -> Response {
    let error = serde_json::json!({
        "error": {
            "type": "error",
            "message": message,
            "retryable": false,
            "status": status.as_u16()
        }
    });

    (status, Json(error)).into_response()
}

/// Build HTTP response with proper error handling
#[allow(clippy::result_large_err)]
fn build_streaming_response(body_text: String, request_id: String) -> Result<Response, Response> {
    HttpResponse::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/x-ndjson")
        .header(header::TRANSFER_ENCODING, "chunked")
        .header("x-request-id", request_id)
        .body(Body::from(body_text))
        .context("Failed to build HTTP response with streaming headers")
        .map(|response| response.into_response())
        .map_err(|e| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Failed to build response: {}", e),
            )
        })
}

/// Create mock NDJSON stream for crawl endpoint
fn create_mock_ndjson_stream(urls: &[String], request_id: &str) -> String {
    let mut lines = Vec::new();

    // Metadata line
    lines.push(
        serde_json::json!({
            "total_urls": urls.len(),
            "stream_type": "crawl",
            "request_id": request_id,
            "timestamp": chrono::Utc::now().to_rfc3339()
        })
        .to_string(),
    );

    // Result lines
    for (i, url) in urls.iter().enumerate() {
        lines.push(
            serde_json::json!({
                "result": {
                    "url": url,
                    "status": 200,
                    "processing_time_ms": 100 + (i as u64 * 50),
                    "quality_score": 0.85,
                    "from_cache": false,
                    "document": {
                        "title": format!("Page {} Title", i + 1),
                        "content": format!("Content from {}", url),
                        "word_count": 150
                    }
                },
                "progress": {
                    "completed": i + 1,
                    "total": urls.len(),
                    "success_rate": 100.0
                }
            })
            .to_string(),
        );
    }

    // Summary line
    lines.push(
        serde_json::json!({
            "successful": urls.len(),
            "failed": 0,
            "total_urls": urls.len(),
            "total_processing_time_ms": urls.len() as u64 * 150,
            "cache_hit_rate": 0.0
        })
        .to_string(),
    );

    lines.join("\n") + "\n"
}

/// Create mock NDJSON stream for search endpoint
fn create_mock_search_stream(query: &str, request_id: &str, limit: u32) -> String {
    let mut lines = Vec::new();

    // Metadata line
    lines.push(
        serde_json::json!({
            "stream_type": "deepsearch",
            "request_id": request_id,
            "timestamp": chrono::Utc::now().to_rfc3339()
        })
        .to_string(),
    );

    // Search metadata
    lines.push(
        serde_json::json!({
            "query": query,
            "urls_found": limit
        })
        .to_string(),
    );

    // Search results
    for i in 1..=limit {
        lines.push(
            serde_json::json!({
                "search_result": {
                    "url": format!("https://example.com/result{}", i),
                    "rank": i,
                    "search_title": format!("Result {} for {}", i, query),
                    "snippet": format!("This is result {} matching query: {}", i, query)
                },
                "crawl_result": {
                    "status": 200,
                    "gate_decision": "allowed"
                }
            })
            .to_string(),
        );
    }

    lines.join("\n") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_mock_ndjson_stream() {
        let urls = vec!["https://example.com".to_string()];
        let stream = create_mock_ndjson_stream(&urls, "test-id");
        assert!(stream.contains("stream_type"));
        assert!(stream.contains("crawl"));
    }
}
