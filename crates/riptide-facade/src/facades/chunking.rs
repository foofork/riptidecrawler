//! Chunking facade for content chunking operations.

use crate::error::RiptideResult;
use crate::RiptideError;
use riptide_extraction::chunking::{create_strategy, ChunkingConfig, ChunkingMode};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tracing::info;

#[derive(Clone)]
pub struct ChunkingFacade;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkRequest {
    pub content: String,
    pub chunking_mode: String,
    #[serde(default)]
    pub parameters: Option<ChunkParameters>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChunkParameters {
    #[serde(default = "default_chunk_size")]
    pub chunk_size: usize,
    #[serde(default = "default_overlap_size")]
    pub overlap_size: usize,
    #[serde(default = "default_min_chunk_size")]
    pub min_chunk_size: usize,
    #[serde(default = "default_preserve_sentences")]
    pub preserve_sentences: bool,
    pub window_size: Option<usize>,
}

fn default_chunk_size() -> usize {
    1000
}
fn default_overlap_size() -> usize {
    200
}
fn default_min_chunk_size() -> usize {
    100
}
fn default_preserve_sentences() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkResponse {
    pub chunks: Vec<ChunkData>,
    pub chunk_count: usize,
    pub original_length: usize,
    pub processing_time_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkData {
    pub index: usize,
    pub content: String,
    pub length: usize,
}

impl ChunkingFacade {
    pub fn new() -> Self {
        Self
    }

    pub async fn chunk_content(&self, request: ChunkRequest) -> RiptideResult<ChunkResponse> {
        let start_time = Instant::now();
        info!(chunking_mode = %request.chunking_mode, content_length = request.content.len(), "Chunking content");

        if request.content.is_empty() {
            return Err(RiptideError::validation("Content cannot be empty"));
        }
        if request.content.len() > 10_000_000 {
            return Err(RiptideError::validation("Content too large (max 10MB)"));
        }

        let params = request.parameters.unwrap_or_default();
        if params.overlap_size >= params.chunk_size {
            return Err(RiptideError::validation(
                "overlap_size must be less than chunk_size",
            ));
        }

        let html_config = ChunkingConfig {
            max_tokens: params.chunk_size,
            overlap_tokens: params.overlap_size,
            preserve_sentences: params.preserve_sentences,
            preserve_html_tags: false,
            min_chunk_size: params.min_chunk_size,
            max_chunk_size: params.chunk_size * 2,
        };

        let chunking_mode =
            self.validate_and_create_chunking_mode(&request.chunking_mode, &params)?;
        let strategy = create_strategy(chunking_mode, html_config);
        let chunks = strategy
            .chunk(&request.content)
            .await
            .map_err(|e| RiptideError::Extraction(format!("Failed to chunk content: {}", e)))?;

        let chunk_data: Vec<ChunkData> = chunks
            .into_iter()
            .map(|chunk| ChunkData {
                index: chunk.chunk_index,
                content: chunk.content.clone(),
                length: chunk.content.len(),
            })
            .collect();

        let chunk_count = chunk_data.len();
        let response = ChunkResponse {
            chunks: chunk_data,
            chunk_count,
            original_length: request.content.len(),
            processing_time_ms: start_time.elapsed().as_millis(),
        };

        info!(
            chunk_count = chunk_count,
            processing_time_ms = response.processing_time_ms,
            "Content chunking completed"
        );
        Ok(response)
    }

    fn validate_and_create_chunking_mode(
        &self,
        mode: &str,
        params: &ChunkParameters,
    ) -> RiptideResult<ChunkingMode> {
        match mode {
            "topic" => Ok(ChunkingMode::Topic { topic_chunking: true, window_size: 100, smoothing_passes: 2 }),
            "sliding" => Ok(ChunkingMode::Sliding { window_size: params.window_size.unwrap_or(params.chunk_size), overlap: params.overlap_size }),
            "fixed" => Ok(ChunkingMode::Fixed { size: params.chunk_size, by_tokens: true }),
            "sentence" => Ok(ChunkingMode::Sentence { max_sentences: params.chunk_size / 20 }),
            "html-aware" => Ok(ChunkingMode::HtmlAware { preserve_blocks: true, preserve_structure: true }),
            invalid => Err(RiptideError::Validation(format!("Invalid chunking_mode '{}'. Supported: topic, sliding, fixed, sentence, html-aware", invalid))),
        }
    }
}

impl Default for ChunkingFacade {
    fn default() -> Self {
        Self::new()
    }
}
