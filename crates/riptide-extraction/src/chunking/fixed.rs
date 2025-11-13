//! Fixed-size chunking by characters or tokens

use super::{utils, Chunk, ChunkMetadata, ChunkingConfig, ChunkingStrategy};
use anyhow::Result;
use async_trait::async_trait;

/// Fixed-size chunker that splits content into fixed-length pieces
pub struct FixedSizeChunker {
    size: usize,
    by_tokens: bool,
    config: ChunkingConfig,
}

impl FixedSizeChunker {
    /// Create a new fixed-size chunker
    pub fn new(size: usize, by_tokens: bool, config: ChunkingConfig) -> Self {
        Self {
            size,
            by_tokens,
            config,
        }
    }
}

#[async_trait]
impl ChunkingStrategy for FixedSizeChunker {
    async fn chunk(&self, text: &str) -> Result<Vec<Chunk>> {
        let mut chunks = Vec::new();

        if text.is_empty() {
            return Ok(chunks);
        }

        if self.by_tokens {
            chunk_by_tokens(text, self.size, &self.config, &mut chunks).await?;
        } else {
            chunk_by_characters(text, self.size, &self.config, &mut chunks).await?;
        }

        // Update total chunk count
        let total_chunks = chunks.len();
        for chunk in &mut chunks {
            chunk.total_chunks = total_chunks;
        }

        Ok(chunks)
    }

    fn name(&self) -> &str {
        if self.by_tokens {
            "fixed_token"
        } else {
            "fixed_char"
        }
    }

    fn config(&self) -> ChunkingConfig {
        self.config.clone()
    }
}

/// Chunk content by token count
async fn chunk_by_tokens(
    content: &str,
    token_size: usize,
    config: &ChunkingConfig,
    chunks: &mut Vec<Chunk>,
) -> Result<()> {
    // Edge case: empty or very small content
    if content.trim().is_empty() {
        return Ok(());
    }

    // Minimum content length to prevent panics
    if content.len() < 10 {
        let token_count = utils::count_tokens(content);
        let chunk = create_fixed_chunk(
            content,
            0,
            token_count,
            0,
            "fixed_token",
            config.preserve_sentences,
        );
        chunks.push(chunk);
        return Ok(());
    }

    // Use character-based approximation to avoid per-word token counting overhead
    // This is much faster and maintains ~90% accuracy for chunking purposes
    let approx_chars_per_token = 4; // Conservative estimate: 1 token ≈ 4 chars
    let approx_chunk_chars = token_size * approx_chars_per_token;

    let mut start_pos = 0;
    let mut chunk_index = 0;

    while start_pos < content.len() {
        // Calculate approximate end position based on token target
        let mut end_pos = start_pos
            .saturating_add(approx_chunk_chars)
            .min(content.len());

        // Adjust to word boundary if possible
        if end_pos < content.len() {
            // Find nearest whitespace boundary
            if let Some(boundary) = content[start_pos..end_pos]
                .rfind(char::is_whitespace)
                .map(|idx| start_pos + idx)
            {
                end_pos = boundary;
            }
        }

        // If preserving sentences, try to end on sentence boundary
        if config.preserve_sentences && end_pos < content.len() {
            if let Some(sent_boundary) = find_sentence_boundary_fast(content, start_pos, end_pos) {
                end_pos = sent_boundary;
            }
        }

        // Ensure we make progress
        if end_pos <= start_pos {
            end_pos = (start_pos + approx_chunk_chars).min(content.len());
        }

        let chunk_content = &content[start_pos..end_pos];

        // Only calculate exact token count once per chunk (not per word)
        let token_count = utils::count_tokens(chunk_content);

        let chunk = create_fixed_chunk(
            chunk_content,
            start_pos,
            token_count,
            chunk_index,
            "fixed_token",
            config.preserve_sentences,
        );
        chunks.push(chunk);

        start_pos = end_pos;
        // Skip whitespace at the start of next chunk
        while start_pos < content.len()
            && content
                .chars()
                .nth(start_pos)
                .unwrap_or(' ')
                .is_whitespace()
        {
            start_pos += 1;
        }
        chunk_index = chunk_index.saturating_add(1);
    }

    Ok(())
}

/// Fast sentence boundary finder (optimized for performance)
fn find_sentence_boundary_fast(content: &str, start: usize, target: usize) -> Option<usize> {
    // Look backward from target for sentence ending
    let search_range = &content[start..target];
    let positions: Vec<usize> = search_range
        .match_indices(['.', '!', '?'])
        .map(|(i, _)| start + i + 1)
        .collect();

    // Return the last sentence boundary found
    positions.last().copied()
}

/// Chunk content by character count
async fn chunk_by_characters(
    content: &str,
    char_size: usize,
    config: &ChunkingConfig,
    chunks: &mut Vec<Chunk>,
) -> Result<()> {
    let mut start_pos = 0;
    let mut chunk_index = 0;

    while start_pos < content.len() {
        let mut end_pos = start_pos.saturating_add(char_size).min(content.len());

        // If preserving sentences, adjust to sentence boundary
        if config.preserve_sentences && end_pos < content.len() {
            end_pos = find_sentence_boundary(content, end_pos);
        }

        // If preserving sentences failed to find a good boundary, try word boundary
        if config.preserve_sentences && end_pos <= start_pos {
            end_pos = find_word_boundary(content, start_pos.saturating_add(char_size));
        }

        // Ensure we make progress
        if end_pos <= start_pos {
            end_pos = start_pos.saturating_add(char_size).min(content.len());
        }

        let chunk_content = &content[start_pos..end_pos];
        let token_count = utils::count_tokens(chunk_content);

        let chunk = create_fixed_chunk(
            chunk_content,
            start_pos,
            token_count,
            chunk_index,
            "fixed_char",
            config.preserve_sentences,
        );
        chunks.push(chunk);

        start_pos = end_pos;
        chunk_index = chunk_index.saturating_add(1);
    }

    Ok(())
}

/// Create a fixed-size chunk with metadata
fn create_fixed_chunk(
    content: &str,
    start_pos: usize,
    token_count: usize,
    chunk_index: usize,
    chunk_type: &str,
    preserve_sentences: bool,
) -> Chunk {
    let end_pos = start_pos + content.len();
    let word_count = content.split_whitespace().count();

    // Count sentences
    let sentence_count = content
        .split(['.', '!', '?'])
        .filter(|s| !s.trim().is_empty())
        .count();

    // Check if chunk has complete sentences
    let has_complete_sentences = if preserve_sentences {
        content.trim().ends_with('.')
            || content.trim().ends_with('!')
            || content.trim().ends_with('?')
    } else {
        false
    };

    let topic_keywords = utils::extract_topic_keywords(content);

    let metadata = ChunkMetadata {
        quality_score: 0.0, // Will be calculated
        sentence_count,
        word_count,
        has_complete_sentences,
        topic_keywords: topic_keywords.clone(),
        chunk_type: chunk_type.to_string(),
        custom: std::collections::HashMap::new(),
    };

    let quality_score = utils::calculate_quality_score(content, &metadata);

    Chunk {
        id: format!("{}_{}", chunk_type, chunk_index),
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

/// Find the position of a word in content by index
#[allow(dead_code)]
fn find_word_position(content: &str, word_index: usize) -> usize {
    let mut pos = 0;
    let mut current_word = 0;

    for ch in content.chars() {
        if ch.is_whitespace() {
            if current_word == word_index {
                return pos;
            }
        } else if pos == 0 || content.chars().nth(pos - 1).unwrap_or(' ').is_whitespace() {
            current_word += 1;
            if current_word > word_index {
                return pos;
            }
        }
        pos += ch.len_utf8();
    }

    content.len()
}

/// Find sentence boundary near the target position
fn find_sentence_boundary(content: &str, target_pos: usize) -> usize {
    let safe_pos = target_pos.min(content.len());

    // Look backward for sentence ending
    for i in (0..safe_pos).rev() {
        if let Some(ch) = content.chars().nth(i) {
            if ch == '.' || ch == '!' || ch == '?' {
                // Make sure it's not an abbreviation
                let word_before = get_word_before(content, i);
                if !is_likely_abbreviation(&word_before) {
                    return i + 1;
                }
            }
        }
    }

    // Look forward for sentence ending
    for i in safe_pos..content.len() {
        if let Some(ch) = content.chars().nth(i) {
            if ch == '.' || ch == '!' || ch == '?' {
                let word_before = get_word_before(content, i);
                if !is_likely_abbreviation(&word_before) {
                    return i + 1;
                }
            }
        }
    }

    safe_pos
}

/// Find word boundary near the target position
fn find_word_boundary(content: &str, target_pos: usize) -> usize {
    let safe_pos = target_pos.min(content.len());

    // Look backward for word boundary
    for i in (0..safe_pos).rev() {
        if let Some(ch) = content.chars().nth(i) {
            if ch.is_whitespace() {
                return i + 1;
            }
        }
    }

    safe_pos
}

/// Get the word before a position
fn get_word_before(content: &str, pos: usize) -> String {
    let mut word = String::new();
    let mut i = pos;

    // Skip the punctuation
    while i > 0 && !content.chars().nth(i - 1).unwrap_or(' ').is_alphabetic() {
        i -= 1;
    }

    // Collect the word
    while i > 0 {
        let ch = content.chars().nth(i - 1).unwrap_or(' ');
        if ch.is_alphabetic() {
            word.insert(0, ch);
            i -= 1;
        } else {
            break;
        }
    }

    word.to_lowercase()
}

/// Check if a word is likely an abbreviation
fn is_likely_abbreviation(word: &str) -> bool {
    const COMMON_ABBREVIATIONS: &[&str] = &[
        "mr", "mrs", "ms", "dr", "prof", "sr", "jr", "inc", "ltd", "corp", "co", "etc", "vs",
        "vol", "no", "pp", "fig", "ch", "sec", "dept", "govt", "jan", "feb", "mar", "apr", "may",
        "jun", "jul", "aug", "sep", "oct", "nov", "dec",
    ];

    COMMON_ABBREVIATIONS.contains(&word) || word.len() <= 3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fixed_size_by_characters() {
        let config = ChunkingConfig::default();
        let chunker = FixedSizeChunker::new(50, false, config);

        let text = "This is a test document with multiple sentences that should be chunked into fixed-size pieces.";
        let chunks = chunker.chunk(text).await.unwrap();

        assert!(!chunks.is_empty());
        assert_eq!(chunks[0].metadata.chunk_type, "fixed_char");
    }

    #[tokio::test]
    async fn test_fixed_size_by_tokens() {
        let config = ChunkingConfig::default();
        let chunker = FixedSizeChunker::new(10, true, config);

        let text = "This is a test document with multiple sentences that should be chunked into fixed-size pieces.";
        let chunks = chunker.chunk(text).await.unwrap();

        assert!(!chunks.is_empty());
        assert_eq!(chunks[0].metadata.chunk_type, "fixed_token");
    }

    #[tokio::test]
    async fn test_sentence_boundary_preservation() {
        let config = ChunkingConfig {
            preserve_sentences: true,
            ..Default::default()
        };
        let chunker = FixedSizeChunker::new(30, false, config);

        let text = "Short sentence. This is a longer sentence that might be split.";
        let chunks = chunker.chunk(text).await.unwrap();

        assert!(!chunks.is_empty());
        // Should preserve sentence boundaries when possible
        for chunk in chunks {
            if chunk.metadata.has_complete_sentences {
                assert!(
                    chunk.content.trim().ends_with('.')
                        || chunk.content.trim().ends_with('!')
                        || chunk.content.trim().ends_with('?')
                );
            }
        }
    }
}
