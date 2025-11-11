//! Chunking handler - <50 LOC after facade refactoring
use crate::errors::ApiError;
use crate::context::ApplicationContext;
use axum::{extract::State, Json};
use riptide_facade::facades::chunking::{
    ChunkParameters, ChunkRequest, ChunkResponse, ChunkingFacade,
};
use serde::Deserialize;
use tracing::{info, instrument};

#[derive(Debug, Deserialize)]
pub struct ChunkRequestDTO {
    pub content: String,
    pub chunking_mode: String,
    #[serde(default)]
    pub parameters: Option<ChunkParametersDTO>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ChunkParametersDTO {
    pub chunk_size: Option<usize>,
    pub overlap_size: Option<usize>,
    pub min_chunk_size: Option<usize>,
    pub preserve_sentences: Option<bool>,
    pub window_size: Option<usize>,
}

#[instrument(skip(_state))]
pub async fn handle_chunking(
    State(_state): State<ApplicationContext>,
    Json(req): Json<ChunkRequestDTO>,
) -> Result<Json<ChunkResponse>, ApiError> {
    info!(mode = %req.chunking_mode, len = req.content.len(), "Chunking");
    let params = req.parameters.map(|p| ChunkParameters {
        chunk_size: p.chunk_size.unwrap_or(1000),
        overlap_size: p.overlap_size.unwrap_or(200),
        min_chunk_size: p.min_chunk_size.unwrap_or(100),
        preserve_sentences: p.preserve_sentences.unwrap_or(true),
        window_size: p.window_size,
    });
    let response = ChunkingFacade::new()
        .chunk_content(ChunkRequest {
            content: req.content,
            chunking_mode: req.chunking_mode,
            parameters: params,
        })
        .await
        .map_err(|e| ApiError::internal(format!("Chunking failed: {}", e)))?;
    Ok(Json(response))
}

// Backward compatibility function for apply_content_chunking (not yet implemented in facade)
pub async fn apply_content_chunking(
    _doc: riptide_types::ExtractedDoc,
    _mode: String,
    _params: Option<ChunkParameters>,
) -> Result<riptide_types::ExtractedDoc, anyhow::Error> {
    anyhow::bail!("apply_content_chunking not yet implemented - use ChunkingFacade directly")
}
