//! Streaming handler - <50 LOC after facade refactoring
use crate::errors::ApiError;
use crate::state::AppState;
use axum::{extract::State, response::IntoResponse, Json};
use riptide_facade::facades::streaming::{
    StreamStartRequest, StreamStartResponse, StreamStatusResponse, StreamingFacade,
};
use serde::Deserialize;
use tracing::instrument;

#[derive(Debug, Deserialize)]
pub struct StreamStartRequestDTO {
    pub session_id: String,
    pub format: Option<String>,
    pub buffer_size: Option<usize>,
}

#[instrument(skip(_state))]
pub async fn handle_stream_start(
    State(_state): State<AppState>,
    Json(req): Json<StreamStartRequestDTO>,
) -> Result<Json<StreamStartResponse>, ApiError> {
    StreamingFacade::new()
        .start_stream(StreamStartRequest {
            session_id: req.session_id,
            format: req.format,
            buffer_size: req.buffer_size,
        })
        .await
        .map(Json)
        .map_err(|e| ApiError::internal(format!("Stream start failed: {}", e)))
}

#[instrument(skip(_state))]
pub async fn handle_stream_status(
    State(_state): State<AppState>,
    Json(stream_id): Json<String>,
) -> Result<Json<StreamStatusResponse>, ApiError> {
    StreamingFacade::new()
        .get_stream_status(&stream_id)
        .await
        .map(Json)
        .map_err(|e| ApiError::internal(format!("Stream status failed: {}", e)))
}

// Backward compatibility for crawl_stream
pub async fn crawl_stream(State(_state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    Ok(axum::response::Response::builder()
        .status(200)
        .header("content-type", "text/event-stream")
        .body(axum::body::Body::from("data: Stream not implemented\n\n"))
        .unwrap())
}
