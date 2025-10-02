use crate::errors::ApiResult;
use riptide_core::types::{ExtractedDoc, ChunkingConfig};
use riptide_html::chunking::{create_strategy, ChunkingMode, ChunkingConfig as HtmlChunkingConfig};
use tracing::{debug, info, warn};

/// Apply content chunking to extracted document based on configuration.
///
/// This function takes an extracted document and applies various chunking strategies
/// to split the content into manageable pieces. Supports multiple chunking modes:
/// - Fixed: Split by fixed token size
/// - Sliding: Overlapping windows
/// - Sentence: Split by sentence boundaries
/// - Topic: Semantic topic-based chunking
/// - HTML-aware: Preserve HTML structure
pub(super) async fn apply_content_chunking(
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

    // Convert ChunkingConfig to the format expected by riptide-html
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
        },
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
        _ => {
            warn!("Unknown chunking mode '{}', falling back to sliding", chunking_config.chunking_mode);
            ChunkingMode::Sliding {
                window_size: chunking_config.chunk_size,
                overlap: chunking_config.overlap_size,
            }
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
                .map(|chunk| format!("=== CHUNK {} ===\n{}\n", chunk.chunk_index + 1, chunk.content))
                .collect::<Vec<_>>()
                .join("\n");

            // Update the document with chunked content
            document.text = chunked_text.clone();

            // Update markdown if it was the source
            if document.markdown.is_some() && chunked_text.len() < content_len {
                document.markdown = Some(chunked_text);
            }

            Ok(document)
        },
        Err(e) => {
            warn!("Content chunking failed: {}, returning original content", e);
            Ok(document)
        }
    }
}