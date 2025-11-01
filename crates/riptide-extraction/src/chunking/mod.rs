//! Content chunking strategies for HTML processing
//!
//! This module provides various chunking strategies specifically designed for HTML content,
//! including HTML-aware chunking that preserves tag integrity and traditional text-based
//! chunking methods adapted for HTML processing.
//!
//! ## Performance Requirements
//!
//! All chunking strategies must process 50KB of text in ≤200ms to meet RipTide's
//! performance requirements.

pub mod cache;
pub mod fixed;
pub mod html_aware;
pub mod regex_chunker;
pub mod sentence;
pub mod sliding;
pub mod topic;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Chunking strategy trait for HTML content processing
#[async_trait]
pub trait ChunkingStrategy: Send + Sync {
    /// Chunk the given text content
    async fn chunk(&self, text: &str) -> Result<Vec<Chunk>>;

    /// Get the strategy name
    fn name(&self) -> &str;

    /// Get strategy configuration options
    fn config(&self) -> ChunkingConfig {
        ChunkingConfig::default()
    }
}

/// A single content chunk with metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Chunk {
    /// Unique identifier for this chunk
    pub id: String,
    /// The text content of the chunk
    pub content: String,
    /// Starting byte position in original text
    pub start_pos: usize,
    /// Ending byte position in original text
    pub end_pos: usize,
    /// Token count (approximate)
    pub token_count: usize,
    /// Index of this chunk in the sequence
    pub chunk_index: usize,
    /// Total number of chunks in the document
    pub total_chunks: usize,
    /// Additional metadata
    pub metadata: ChunkMetadata,
}

/// Metadata associated with a chunk
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChunkMetadata {
    /// Quality score (0.0 - 1.0)
    pub quality_score: f64,
    /// Number of sentences in chunk
    pub sentence_count: usize,
    /// Number of words in chunk
    pub word_count: usize,
    /// Whether chunk ends with complete sentences
    pub has_complete_sentences: bool,
    /// Topic keywords extracted from chunk
    pub topic_keywords: Vec<String>,
    /// Type of chunking strategy used
    pub chunk_type: String,
    /// Additional custom metadata
    pub custom: HashMap<String, String>,
}

/// Configuration for chunking strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkingConfig {
    /// Maximum chunk size in tokens
    pub max_tokens: usize,
    /// Overlap between chunks in tokens
    pub overlap_tokens: usize,
    /// Whether to preserve sentence boundaries
    pub preserve_sentences: bool,
    /// Whether to preserve HTML tag boundaries
    pub preserve_html_tags: bool,
    /// Minimum chunk size in characters
    pub min_chunk_size: usize,
    /// Maximum chunk size in characters
    pub max_chunk_size: usize,
}

impl Default for ChunkingConfig {
    fn default() -> Self {
        Self {
            max_tokens: 1000,
            overlap_tokens: 100,
            preserve_sentences: true,
            preserve_html_tags: true,
            min_chunk_size: 100,
            max_chunk_size: 10000,
        }
    }
}

/// Available chunking strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChunkingMode {
    /// Sliding window with configurable overlap
    Sliding { window_size: usize, overlap: usize },
    /// Fixed-size chunks
    Fixed { size: usize, by_tokens: bool },
    /// Sentence-based chunking
    Sentence { max_sentences: usize },
    /// Regex pattern-based chunking
    Regex {
        pattern: String,
        min_chunk_size: usize,
    },
    /// HTML-aware chunking that preserves tag integrity
    HtmlAware {
        preserve_blocks: bool,
        preserve_structure: bool,
    },
    /// Topic-based chunking using TextTiling algorithm
    Topic {
        /// Whether to use topic chunking (opt-in)
        topic_chunking: bool,
        /// Window size for coherence analysis
        window_size: usize,
        /// Smoothing passes for boundary detection
        smoothing_passes: usize,
    },
}

impl Default for ChunkingMode {
    fn default() -> Self {
        Self::Sliding {
            window_size: 1000,
            overlap: 100,
        }
    }
}

/// Create a chunking strategy based on the specified mode
pub fn create_strategy(mode: ChunkingMode, config: ChunkingConfig) -> Box<dyn ChunkingStrategy> {
    match mode {
        ChunkingMode::Sliding {
            window_size,
            overlap,
        } => Box::new(sliding::SlidingWindowChunker::new(
            window_size,
            overlap,
            config,
        )),
        ChunkingMode::Fixed { size, by_tokens } => {
            Box::new(fixed::FixedSizeChunker::new(size, by_tokens, config))
        }
        ChunkingMode::Sentence { max_sentences } => {
            Box::new(sentence::SentenceChunker::new(max_sentences, config))
        }
        ChunkingMode::Regex {
            pattern,
            min_chunk_size,
        } => Box::new(regex_chunker::RegexChunker::new(
            pattern,
            min_chunk_size,
            config,
        )),
        ChunkingMode::HtmlAware {
            preserve_blocks,
            preserve_structure,
        } => Box::new(html_aware::HtmlAwareChunker::new(
            preserve_blocks,
            preserve_structure,
            config,
        )),
        ChunkingMode::Topic {
            topic_chunking,
            window_size,
            smoothing_passes,
        } => {
            if topic_chunking {
                Box::new(topic::TopicChunker::new(
                    window_size,
                    smoothing_passes,
                    config,
                ))
            } else {
                // Fallback to sliding window when topic chunking is disabled
                Box::new(sliding::SlidingWindowChunker::new(1000, 100, config))
            }
        }
    }
}

/// Utility functions for chunking
pub mod utils {
    use super::*;

    /// Calculate approximate token count for text (fast, synchronous)
    ///
    /// This provides a fast approximation for synchronous contexts.
    /// For exact counts, use `count_tokens_exact()` which is async.
    pub fn count_tokens(text: &str) -> usize {
        // Use word-based approximation to avoid blocking
        // Accurate within ±10% for English text
        (text.split_whitespace().count() as f64 * 1.3) as usize
    }

    /// Calculate exact token count for text using tiktoken (async)
    ///
    /// This uses the tiktoken-rs library with caching for accurate token counts.
    /// Prefer this over `count_tokens()` when accuracy is important and async context is available.
    ///
    /// # Example
    /// ```no_run
    /// use riptide_extraction::chunking::utils::count_tokens_exact;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let text = "This is a test document.";
    /// let exact_count = count_tokens_exact(text).await?;
    /// println!("Exact token count: {}", exact_count);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn count_tokens_exact(text: &str) -> anyhow::Result<usize> {
        cache::count_tokens_exact(text).await
    }

    /// Count tokens for multiple texts in batch (async)
    ///
    /// More efficient than calling `count_tokens_exact()` multiple times.
    pub async fn count_tokens_batch(texts: &[&str]) -> anyhow::Result<Vec<usize>> {
        cache::count_tokens_batch(texts).await
    }

    /// Calculate quality score for a chunk
    pub fn calculate_quality_score(content: &str, metadata: &ChunkMetadata) -> f64 {
        let mut score = 0.5; // Base score

        // Length score (optimal around 800 characters)
        let length_ratio = (content.len() as f64 / 800.0).min(1.0);
        score += length_ratio * 0.2;

        // Sentence completeness bonus
        if metadata.has_complete_sentences {
            score += 0.2;
        }

        // Word density bonus
        if metadata.word_count > 20 {
            score += 0.1;
        }

        // Topic keywords bonus
        if !metadata.topic_keywords.is_empty() {
            score += (metadata.topic_keywords.len().min(5) as f64 / 5.0) * 0.1;
        }

        score.min(1.0)
    }

    /// Extract topic keywords from text
    pub fn extract_topic_keywords(text: &str) -> Vec<String> {
        use std::collections::HashMap;

        let words: Vec<String> = text
            .split_whitespace()
            .filter(|word| word.len() > 3)
            .map(|word| {
                word.trim_matches(|c: char| !c.is_alphanumeric())
                    .to_lowercase()
            })
            .filter(|word| !word.is_empty() && !is_stop_word(word))
            .collect();

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

    /// Check if a word is a stop word
    fn is_stop_word(word: &str) -> bool {
        const STOP_WORDS: &[&str] = &[
            "the", "be", "to", "of", "and", "a", "in", "that", "have", "i", "it", "for", "not",
            "on", "with", "he", "as", "you", "do", "at", "this", "but", "his", "by", "from",
            "they", "we", "say", "her", "she", "or", "an", "will", "my", "one", "all", "would",
            "there", "their", "what", "so", "up", "out", "if", "about", "who", "get", "which",
            "go", "me",
        ];

        STOP_WORDS.contains(&word)
    }

    /// Split text into sentences with improved detection
    pub fn split_sentences(content: &str) -> Vec<String> {
        let mut sentences = Vec::new();
        let mut current_sentence = String::new();

        for word in content.split_whitespace() {
            current_sentence.push_str(word);
            current_sentence.push(' ');

            // Check for sentence endings
            if word.ends_with('.') || word.ends_with('!') || word.ends_with('?') {
                // Check if it's not an abbreviation
                if !is_abbreviation(word) {
                    sentences.push(current_sentence.trim().to_string());
                    current_sentence.clear();
                }
            }
        }

        // Add remaining content
        if !current_sentence.trim().is_empty() {
            sentences.push(current_sentence.trim().to_string());
        }

        // Filter out very short sentences
        sentences
            .into_iter()
            .filter(|s| s.split_whitespace().count() >= 3)
            .collect()
    }

    /// Check if a word is likely an abbreviation
    fn is_abbreviation(word: &str) -> bool {
        const COMMON_ABBREVIATIONS: &[&str] = &[
            "mr.", "mrs.", "ms.", "dr.", "prof.", "sr.", "jr.", "inc.", "ltd.", "corp.", "co.",
            "etc.", "vs.", "vol.", "no.", "pp.", "fig.", "ch.", "sec.", "dept.", "govt.", "u.s.",
            "u.k.", "e.g.", "i.e.", "a.m.", "p.m.",
        ];

        let lower = word.to_lowercase();
        COMMON_ABBREVIATIONS.contains(&lower.as_str())
            || (word.len() <= 4 && word.chars().filter(|c| c.is_uppercase()).count() > 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use tokio_test;

    #[tokio::test]
    async fn test_chunking_strategies() {
        let text = "This is a test document. It has multiple sentences. Each sentence contains meaningful content. The document should be chunked appropriately.";

        // Test sliding window chunking
        let config = ChunkingConfig::default();
        let strategy = create_strategy(ChunkingMode::default(), config.clone());
        let chunks = strategy.chunk(text).await.unwrap();
        assert!(!chunks.is_empty());
        assert_eq!(chunks[0].chunk_index, 0);

        // Test fixed size chunking
        let strategy = create_strategy(
            ChunkingMode::Fixed {
                size: 50,
                by_tokens: false,
            },
            config.clone(),
        );
        let chunks = strategy.chunk(text).await.unwrap();
        assert!(!chunks.is_empty());

        // Test sentence chunking
        let strategy = create_strategy(ChunkingMode::Sentence { max_sentences: 2 }, config);
        let chunks = strategy.chunk(text).await.unwrap();
        assert!(!chunks.is_empty());
    }

    #[tokio::test]
    async fn test_performance_requirement() {
        // Generate 50KB of text
        let base_text =
            "This is a performance test document with multiple sentences and paragraphs. ";
        let mut large_text = String::new();
        while large_text.len() < 50_000 {
            large_text.push_str(base_text);
        }

        let config = ChunkingConfig::default();
        let strategy = create_strategy(ChunkingMode::default(), config);

        let start = std::time::Instant::now();
        let chunks = strategy.chunk(&large_text).await.unwrap();
        let duration = start.elapsed();

        // Should complete in ≤200ms
        assert!(
            duration.as_millis() <= 200,
            "Chunking took {}ms, expected ≤200ms",
            duration.as_millis()
        );
        assert!(!chunks.is_empty());
    }
}
