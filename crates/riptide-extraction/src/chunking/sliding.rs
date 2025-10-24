//! Sliding window chunking with overlap (default mode for HTML content)

use super::{utils, Chunk, ChunkMetadata, ChunkingConfig, ChunkingStrategy};
use anyhow::Result;
use async_trait::async_trait;

/// Sliding window chunker with configurable overlap
pub struct SlidingWindowChunker {
    window_size: usize,
    overlap: usize,
    config: ChunkingConfig,
}

impl SlidingWindowChunker {
    /// Create a new sliding window chunker
    pub fn new(window_size: usize, overlap: usize, config: ChunkingConfig) -> Self {
        Self {
            window_size,
            overlap,
            config,
        }
    }
}

#[async_trait]
impl ChunkingStrategy for SlidingWindowChunker {
    async fn chunk(&self, text: &str) -> Result<Vec<Chunk>> {
        let mut chunks = Vec::new();

        if text.is_empty() {
            return Ok(chunks);
        }

        // Split content into sentences for better boundary handling
        let sentences = if self.config.preserve_sentences {
            utils::split_sentences(text)
        } else {
            split_into_words(text)
        };

        let mut current_chunk = String::new();
        let mut current_tokens = 0;
        let mut start_pos = 0;
        let mut chunk_index = 0;
        let mut sentence_buffer = Vec::new();

        for sentence in sentences.iter() {
            let sentence_tokens = utils::count_tokens(sentence);

            // If adding this sentence would exceed token limit, create a chunk
            if current_tokens + sentence_tokens > self.window_size && !current_chunk.is_empty() {
                let chunk = create_chunk(
                    &current_chunk,
                    start_pos,
                    start_pos + current_chunk.len(),
                    current_tokens,
                    chunk_index,
                    &sentence_buffer,
                );
                chunks.push(chunk);

                // Calculate overlap for next chunk
                let overlap_content = if self.overlap > 0 {
                    calculate_overlap(&sentence_buffer, self.overlap)
                } else {
                    String::new()
                };

                // Reset for next chunk
                current_chunk = overlap_content.clone();
                current_tokens = utils::count_tokens(&current_chunk);
                start_pos = if let Some(last_chunk) = chunks.last() {
                    if overlap_content.is_empty() {
                        start_pos + last_chunk.content.len()
                    } else {
                        start_pos + last_chunk.content.len() - overlap_content.len()
                    }
                } else {
                    start_pos
                };
                chunk_index += 1;
                sentence_buffer.clear();

                // Add overlap sentences to buffer
                if !overlap_content.is_empty() {
                    sentence_buffer.push(overlap_content);
                }
            }

            // Add current sentence
            if !current_chunk.is_empty() {
                current_chunk.push(' ');
            }
            current_chunk.push_str(sentence);
            current_tokens += sentence_tokens;
            sentence_buffer.push(sentence.to_string());
        }

        // Add final chunk if there's remaining content
        if !current_chunk.is_empty() {
            let chunk = create_chunk(
                &current_chunk,
                start_pos,
                start_pos + current_chunk.len(),
                current_tokens,
                chunk_index,
                &sentence_buffer,
            );
            chunks.push(chunk);
        }

        // Update total chunk count
        let total_chunks = chunks.len();
        for chunk in &mut chunks {
            chunk.total_chunks = total_chunks;
        }

        Ok(chunks)
    }

    fn name(&self) -> &str {
        "sliding_window"
    }

    fn config(&self) -> ChunkingConfig {
        self.config.clone()
    }
}

/// Create a content chunk with metadata
fn create_chunk(
    content: &str,
    start_pos: usize,
    end_pos: usize,
    token_count: usize,
    chunk_index: usize,
    sentences: &[String],
) -> Chunk {
    let word_count = content.split_whitespace().count();
    let sentence_count = sentences.len();
    let has_complete_sentences = content.trim().ends_with('.')
        || content.trim().ends_with('!')
        || content.trim().ends_with('?');
    let topic_keywords = utils::extract_topic_keywords(content);

    let metadata = ChunkMetadata {
        quality_score: 0.0, // Will be calculated
        sentence_count,
        word_count,
        has_complete_sentences,
        topic_keywords: topic_keywords.clone(),
        chunk_type: "sliding".to_string(),
        custom: std::collections::HashMap::new(),
    };

    let quality_score = utils::calculate_quality_score(content, &metadata);

    Chunk {
        id: format!("sliding_{}_{}", chunk_index, start_pos),
        content: content.to_string(),
        start_pos,
        end_pos,
        token_count,
        chunk_index,
        total_chunks: 0, // Will be updated later
        metadata: ChunkMetadata {
            quality_score,
            ..metadata
        },
    }
}

/// Calculate overlap content based on token count
fn calculate_overlap(sentences: &[String], overlap_tokens: usize) -> String {
    if sentences.is_empty() || overlap_tokens == 0 {
        return String::new();
    }

    let mut overlap_content = String::new();
    let mut tokens_used = 0;

    // Take sentences from the end until we reach overlap limit
    for sentence in sentences.iter().rev() {
        let sentence_tokens = utils::count_tokens(sentence);
        if tokens_used + sentence_tokens <= overlap_tokens {
            if !overlap_content.is_empty() {
                overlap_content = format!("{} {}", sentence, overlap_content);
            } else {
                overlap_content = sentence.clone();
            }
            tokens_used += sentence_tokens;
        } else {
            break;
        }
    }

    overlap_content
}

/// Split content into words for non-sentence-preserving mode
fn split_into_words(content: &str) -> Vec<String> {
    content
        .split_whitespace()
        .map(|word| word.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sliding_window_chunking() {
        let config = ChunkingConfig::default();
        let chunker = SlidingWindowChunker::new(100, 20, config);

        let text = "This is the first sentence. This is the second sentence. This is the third sentence. This is the fourth sentence.";
        let chunks = chunker.chunk(text).await.unwrap();

        assert!(!chunks.is_empty());
        assert_eq!(chunks[0].chunk_index, 0);
        assert_eq!(chunks[0].metadata.chunk_type, "sliding");
    }

    #[tokio::test]
    async fn test_overlap_calculation() {
        let sentences = vec![
            "First sentence.".to_string(),
            "Second sentence.".to_string(),
            "Third sentence.".to_string(),
        ];

        let overlap = calculate_overlap(&sentences, 10);
        assert!(!overlap.is_empty());
    }

    #[tokio::test]
    async fn test_empty_content() {
        let config = ChunkingConfig::default();
        let chunker = SlidingWindowChunker::new(100, 20, config);

        let chunks = chunker.chunk("").await.unwrap();
        assert!(chunks.is_empty());
    }
}
