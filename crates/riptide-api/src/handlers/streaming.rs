//! Streaming handler - placeholder for Phase 4.3 completion
//!
//! NOTE: This handler will be fully refactored to use StreamingFacade
//! after all dependencies are properly wired in AppState.
//! Currently kept as stub to maintain API compatibility.

use crate::errors::ApiError;
use crate::context::ApplicationContext;
use axum::{extract::State, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use tracing::instrument;

/// DTO for stream start requests - Phase 4.3 API
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct StreamStartRequestDTO {
    pub session_id: String,
    pub format: Option<String>,
    pub buffer_size: Option<usize>,
}

/// Response for stream start - Phase 4.3 API
#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct StreamStartResponse {
    pub stream_id: String,
    pub status: String,
}

/// Response for stream status - Phase 4.3 API
#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct StreamStatusResponse {
    pub stream_id: String,
    pub state: String,
    pub chunks_processed: usize,
}

/// Phase 4.3 API endpoint - start streaming session
#[allow(dead_code)]
#[instrument(skip(_state))]
pub async fn handle_stream_start(
    State(_state): State<ApplicationContext>,
    Json(_req): Json<StreamStartRequestDTO>,
) -> Result<Json<StreamStartResponse>, ApiError> {
    // TODO Phase 4.3: Wire StreamingFacade with proper dependencies
    // For now, return stub response
    Ok(Json(StreamStartResponse {
        stream_id: "stub".to_string(),
        status: "not_implemented".to_string(),
    }))
}

/// Phase 4.3 API endpoint - get streaming session status
#[allow(dead_code)]
#[instrument(skip(_state))]
pub async fn handle_stream_status(
    State(_state): State<ApplicationContext>,
    Json(_stream_id): Json<String>,
) -> Result<Json<StreamStatusResponse>, ApiError> {
    // TODO Phase 4.3: Wire StreamingFacade with proper dependencies
    Ok(Json(StreamStatusResponse {
        stream_id: "stub".to_string(),
        state: "not_implemented".to_string(),
        chunks_processed: 0,
    }))
}

// Backward compatibility for crawl_stream
pub async fn crawl_stream(State(_state): State<ApplicationContext>) -> Result<impl IntoResponse, ApiError> {
    Ok(axum::response::Response::builder()
        .status(200)
        .header("content-type", "text/event-stream")
        .body(axum::body::Body::from("data: Stream not implemented\n\n"))
        .unwrap())
}
