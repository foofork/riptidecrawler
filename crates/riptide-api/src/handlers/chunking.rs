use crate::errors::{ApiError, ApiResult};
use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use riptide_extraction::chunking::{
    create_strategy, ChunkingConfig as HtmlChunkingConfig, ChunkingMode,
};
use riptide_types::{ChunkingConfig, ExtractedDoc};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tracing::{debug, info, warn};

/// Request body for chunking content
#[derive(Deserialize, Debug)]
pub struct ChunkRequest {
    /// Content to chunk
    pub content: String,
    /// Chunking mode (topic, sliding, fixed, sentence, html-aware)
    pub chunking_mode: String,
    /// Optional parameters for chunking
    #[serde(default)]
    pub parameters: Option<ChunkParameters>,
}

/// Optional chunking parameters
#[derive(Deserialize, Debug, Default)]
pub struct ChunkParameters {
    /// Chunk size (default: 1000)
    #[serde(default = "default_chunk_size")]
    pub chunk_size: usize,
    /// Overlap size for sliding window (default: 200)
    #[serde(default = "default_overlap_size")]
    pub overlap_size: usize,
    /// Minimum chunk size (default: 100)
    #[serde(default = "default_min_chunk_size")]
    pub min_chunk_size: usize,
    /// Preserve sentence boundaries (default: true)
    #[serde(default = "default_preserve_sentences")]
    pub preserve_sentences: bool,
    /// Window size for sliding mode (optional, uses chunk_size if not provided)
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

/// Response from chunking operation
#[derive(Serialize, Debug)]
pub struct ChunkResponse {
    /// The chunked content
    pub chunks: Vec<ChunkData>,
    /// Total number of chunks
    pub chunk_count: usize,
    /// Original content length
    pub original_length: usize,
    /// Processing time in milliseconds
    pub processing_time_ms: u128,
}

/// Individual chunk data
#[derive(Serialize, Debug)]
pub struct ChunkData {
    /// Chunk index (0-based)
    pub index: usize,
    /// Chunk content
    pub content: String,
    /// Chunk length
    pub length: usize,
}

/// HTTP endpoint for content chunking
///
/// This endpoint accepts content and chunking parameters, applies the requested
/// chunking strategy, and returns the chunked content.
///
/// ## Request Body
/// - `content`: Text content to chunk
/// - `chunking_mode`: Strategy to use (topic, sliding, fixed, sentence, html-aware)
/// - `parameters`: Optional chunking parameters (chunk_size, overlap_size, etc.)
pub async fn chunk_content(
    State(state): State<AppState>,
    Json(request): Json<ChunkRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let start_time = Instant::now();

    info!(
        chunking_mode = %request.chunking_mode,
        content_length = request.content.len(),
        "Received chunk content request"
    );

    // Validate input
    if request.content.is_empty() {
        return Err(ApiError::validation("Content cannot be empty"));
    }

    if request.content.len() > 10_000_000 {
        return Err(ApiError::validation("Content too large (max 10MB)"));
    }

    // Get parameters or use defaults
    let params = request.parameters.unwrap_or_default();

    // Validate parameters
    if params.overlap_size >= params.chunk_size {
        return Err(ApiError::validation(
            "overlap_size must be less than chunk_size",
        ));
    }

    if let Some(window_size) = params.window_size {
        if window_size < params.overlap_size {
            return Err(ApiError::validation(
                "window_size must be greater than overlap_size",
            ));
        }
    }

    // Create chunking config
    let config = ChunkingConfig {
        chunking_mode: request.chunking_mode.clone(),
        chunk_size: params.chunk_size,
        overlap_size: params.overlap_size,
        min_chunk_size: params.min_chunk_size,
        preserve_sentences: params.preserve_sentences,
        topic_config: None,
    };

    // Apply chunking
    let html_config = HtmlChunkingConfig {
        max_tokens: config.chunk_size,
        overlap_tokens: config.overlap_size,
        preserve_sentences: config.preserve_sentences,
        preserve_html_tags: false,
        min_chunk_size: config.min_chunk_size,
        max_chunk_size: config.chunk_size * 2,
    };

    let chunking_mode = validate_and_create_chunking_mode(&request.chunking_mode, &params)?;

    let strategy = create_strategy(chunking_mode, html_config);

    match strategy.chunk(&request.content).await {
        Ok(chunks) => {
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
                "Content chunking completed successfully"
            );

            // Record metrics
            state.metrics.record_http_request(
                "POST",
                "/api/v1/content/chunk",
                200,
                start_time.elapsed().as_secs_f64(),
            );

            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            warn!("Content chunking failed: {}", e);
            Err(ApiError::internal(format!(
                "Failed to chunk content: {}",
                e
            )))
        }
    }
}

/// Validate chunking mode and create ChunkingMode enum
fn validate_and_create_chunking_mode(
    mode: &str,
    params: &ChunkParameters,
) -> Result<ChunkingMode, ApiError> {
    match mode {
        "topic" => Ok(ChunkingMode::Topic {
            topic_chunking: true,
            window_size: 100,
            smoothing_passes: 2,
        }),
        "sliding" => Ok(ChunkingMode::Sliding {
            window_size: params.window_size.unwrap_or(params.chunk_size),
            overlap: params.overlap_size,
        }),
        "fixed" => Ok(ChunkingMode::Fixed {
            size: params.chunk_size,
            by_tokens: true,
        }),
        "sentence" => Ok(ChunkingMode::Sentence {
            max_sentences: params.chunk_size / 20,
        }),
        "html-aware" => Ok(ChunkingMode::HtmlAware {
            preserve_blocks: true,
            preserve_structure: true,
        }),
        invalid_mode => {
            Err(ApiError::invalid_request(format!(
                "Invalid chunking_mode '{}'. Supported modes: topic, sliding, fixed, sentence, html-aware",
                invalid_mode
            )))
        }
    }
}

/// Apply content chunking to extracted document based on configuration.
///
/// This function takes an extracted document and applies various chunking strategies
/// to split the content into manageable pieces. Supports multiple chunking modes:
/// - Fixed: Split by fixed token size
/// - Sliding: Overlapping windows
/// - Sentence: Split by sentence boundaries
/// - Topic: Semantic topic-based chunking
/// - HTML-aware: Preserve HTML structure
pub(crate) async fn apply_content_chunking(
    mut document: ExtractedDoc,
    chunking_config: &ChunkingConfig,
) -> ApiResult<ExtractedDoc> {
    // Get the text content to chunk
    let content = if !document.text.is_empty() {
        document.text.clone()
    } else if let Some(ref markdown) = document.markdown {
        markdown.clone()
    } else {
        // No content to chunk
        return Ok(document);
    };

    let content_len = content.len();
    debug!(
        chunking_mode = %chunking_config.chunking_mode,
        chunk_size = chunking_config.chunk_size,
        content_length = content_len,
        "Applying content chunking"
    );

    // Convert ChunkingConfig to the format expected by riptide-extraction
    let html_config = HtmlChunkingConfig {
        max_tokens: chunking_config.chunk_size,
        overlap_tokens: chunking_config.overlap_size,
        preserve_sentences: chunking_config.preserve_sentences,
        preserve_html_tags: false, // For text content, don't worry about HTML tags
        min_chunk_size: chunking_config.min_chunk_size,
        max_chunk_size: chunking_config.chunk_size * 2, // Allow some flexibility
    };

    // Create the appropriate chunking strategy
    let chunking_mode = match chunking_config.chunking_mode.as_str() {
        "topic" => {
            if let Some(ref topic_config) = chunking_config.topic_config {
                ChunkingMode::Topic {
                    topic_chunking: topic_config.topic_chunking,
                    window_size: topic_config.window_size,
                    smoothing_passes: topic_config.smoothing_passes,
                }
            } else {
                ChunkingMode::Topic {
                    topic_chunking: true,
                    window_size: 100,
                    smoothing_passes: 2,
                }
            }
        }
        "sliding" => ChunkingMode::Sliding {
            window_size: chunking_config.chunk_size,
            overlap: chunking_config.overlap_size,
        },
        "fixed" => ChunkingMode::Fixed {
            size: chunking_config.chunk_size,
            by_tokens: true,
        },
        "sentence" => ChunkingMode::Sentence {
            max_sentences: chunking_config.chunk_size / 20, // Rough estimate
        },
        "html-aware" => ChunkingMode::HtmlAware {
            preserve_blocks: true,
            preserve_structure: true,
        },
        invalid_mode => {
            warn!("Invalid chunking mode '{}' provided", invalid_mode);
            return Err(crate::errors::ApiError::invalid_request(
                format!(
                    "Invalid chunking_mode '{}'. Supported modes: topic, sliding, fixed, sentence, html-aware",
                    invalid_mode
                )
            ));
        }
    };

    // Create and execute the chunking strategy
    let strategy = create_strategy(chunking_mode, html_config);

    match strategy.chunk(&content).await {
        Ok(chunks) => {
            info!(
                chunk_count = chunks.len(),
                chunking_mode = %chunking_config.chunking_mode,
                "Content chunking completed"
            );

            // For now, join chunks back together with separators for compatibility
            // In a future version, we might return chunks separately
            let chunked_text = chunks
                .into_iter()
                .map(|chunk| {
                    format!(
                        "=== CHUNK {} ===\n{}\n",
                        chunk.chunk_index + 1,
                        chunk.content
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");

            // Update the document with chunked content
            document.text = chunked_text.clone();

            // Update markdown if it was the source
            if document.markdown.is_some() && chunked_text.len() < content_len {
                document.markdown = Some(chunked_text);
            }

            Ok(document)
        }
        Err(e) => {
            warn!("Content chunking failed: {}, returning original content", e);
            Ok(document)
        }
    }
}
