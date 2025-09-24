//! Content chunking strategies for optimal processing

pub mod regex;
pub mod sentence;
pub mod topic;
pub mod fixed;
pub mod sliding;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Chunking configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChunkingConfig {
    pub mode: ChunkingMode,
    pub token_max: usize,
    pub overlap: usize,
    pub preserve_sentences: bool,
    pub deterministic: bool,
}

impl Default for ChunkingConfig {
    fn default() -> Self {
        Self {
            mode: ChunkingMode::Sliding,
            token_max: 1200,
            overlap: 120,
            preserve_sentences: true,
            deterministic: true,
        }
    }
}

/// Available chunking modes
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ChunkingMode {
    /// Split by regex pattern
    Regex {
        pattern: String,
        min_chunk_size: usize,
    },
    /// Split by sentence boundaries (NLP)
    Sentence {
        max_sentences: usize,
    },
    /// Split by semantic topics
    Topic {
        similarity_threshold: f64,
    },
    /// Fixed character/token count
    Fixed {
        size: usize,
        by_tokens: bool,
    },
    /// Sliding windows with overlap
    Sliding,
}

/// Content chunk with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentChunk {
    pub id: String,
    pub content: String,
    pub start_pos: usize,
    pub end_pos: usize,
    pub token_count: usize,
    pub chunk_index: usize,
    pub total_chunks: usize,
    pub metadata: ChunkMetadata,
}

/// Chunk metadata for tracking and analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub quality_score: f64,
    pub sentence_count: usize,
    pub word_count: usize,
    pub has_complete_sentences: bool,
    pub topic_keywords: Vec<String>,
    pub chunk_type: String,
}

/// Main chunking function
pub async fn chunk_content(content: &str, config: &ChunkingConfig) -> Result<Vec<ContentChunk>> {
    if content.is_empty() {
        return Ok(vec![]);
    }

    let chunks = match &config.mode {
        ChunkingMode::Regex { pattern, min_chunk_size } => {
            regex::chunk_by_regex(content, pattern, *min_chunk_size, config).await?
        }
        ChunkingMode::Sentence { max_sentences } => {
            sentence::chunk_by_sentences(content, *max_sentences, config).await?
        }
        ChunkingMode::Topic { similarity_threshold } => {
            topic::chunk_by_topics(content, *similarity_threshold, config).await?
        }
        ChunkingMode::Fixed { size, by_tokens } => {
            fixed::chunk_fixed_size(content, *size, *by_tokens, config).await?
        }
        ChunkingMode::Sliding => {
            sliding::chunk_sliding_window(content, config).await?
        }
    };

    // Post-process chunks for consistency
    let processed_chunks = if config.deterministic {
        ensure_deterministic_chunking(chunks)
    } else {
        chunks
    };

    Ok(processed_chunks)
}

/// Ensure deterministic chunking by normalizing chunk boundaries
fn ensure_deterministic_chunking(mut chunks: Vec<ContentChunk>) -> Vec<ContentChunk> {
    // Sort by start position to ensure consistent ordering
    chunks.sort_by_key(|c| c.start_pos);

    // Update chunk indices and total count
    let total_chunks = chunks.len();
    for (i, chunk) in chunks.iter_mut().enumerate() {
        chunk.chunk_index = i;
        chunk.total_chunks = total_chunks;
        chunk.id = format!("chunk_{}_{}", i, chunk.start_pos);
    }

    chunks
}

/// Calculate token count for text
pub fn count_tokens(text: &str) -> usize {
    // Use tiktoken for more accurate token counting
    match tiktoken_rs::get_bpe_from_model("gpt-3.5-turbo") {
        Ok(bpe) => bpe.encode_with_special_tokens(text).len(),
        Err(_) => {
            // Fallback: approximate tokens as words * 1.3
            (text.split_whitespace().count() as f64 * 1.3) as usize
        }
    }
}

/// Calculate quality score for a chunk
pub fn calculate_chunk_quality(content: &str, metadata: &ChunkMetadata) -> f64 {
    let mut score = 0.5; // Base score

    // Sentence completeness bonus
    if metadata.has_complete_sentences {
        score += 0.2;
    }

    // Length bonus (not too short, not too long)
    let ideal_length = 800.0;
    let length_ratio = (content.len() as f64 / ideal_length).min(1.0_f64);
    score += length_ratio * 0.2;

    // Word density bonus
    if metadata.word_count > 50 {
        score += 0.1;
    }

    // Topic keywords bonus
    if !metadata.topic_keywords.is_empty() {
        score += (metadata.topic_keywords.len().min(5) as f64 / 5.0) * 0.1;
    }

    score.min(1.0_f64)
}

/// Extract topic keywords from text
pub fn extract_topic_keywords(text: &str) -> Vec<String> {
    // Simple keyword extraction based on word frequency and length
    use std::collections::HashMap;

    let words: Vec<String> = text
        .split_whitespace()
        .filter(|word| word.len() > 3)
        .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
        .filter(|word| !word.is_empty() && !is_stop_word(word))
        .collect::<Vec<_>>();

    let mut word_counts: HashMap<String, usize> = HashMap::new();
    for word in words {
        *word_counts.entry(word).or_insert(0) += 1;
    }

    let mut sorted_words: Vec<(String, usize)> = word_counts.into_iter().collect();
    sorted_words.sort_by(|a, b| b.1.cmp(&a.1));

    sorted_words
        .into_iter()
        .take(5)
        .map(|(word, _)| word)
        .collect()
}

/// Simple stop word filter
fn is_stop_word(word: &str) -> bool {
    const STOP_WORDS: &[&str] = &[
        "the", "be", "to", "of", "and", "a", "in", "that", "have",
        "i", "it", "for", "not", "on", "with", "he", "as", "you",
        "do", "at", "this", "but", "his", "by", "from", "they",
        "we", "say", "her", "she", "or", "an", "will", "my",
        "one", "all", "would", "there", "their", "what", "so",
        "up", "out", "if", "about", "who", "get", "which", "go", "me",
    ];

    STOP_WORDS.contains(&word)
}