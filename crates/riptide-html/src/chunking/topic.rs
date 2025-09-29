//! Topic-based chunking using the TextTiling algorithm
//!
//! This module implements the TextTiling algorithm for automatic topic segmentation
//! of documents. The algorithm uses lexical cohesion analysis to identify topic
//! boundaries in text.
//!
//! ## Algorithm Overview
//!
//! TextTiling works by:
//! 1. Tokenizing text into sentences or pseudo-sentences
//! 2. Calculating lexical similarity between adjacent text blocks
//! 3. Identifying valley points (low similarity) as topic boundaries
//! 4. Applying smoothing to improve boundary detection
//!
//! ## Performance Requirements
//!
//! The implementation is optimized to add <200ms overhead per document while
//! providing intelligent topic-based segmentation for long documents.

use crate::chunking::{ChunkingStrategy, ChunkingConfig, Chunk, ChunkMetadata};
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;

/// Topic-based chunker using TextTiling algorithm
pub struct TopicChunker {
    /// Window size for coherence analysis (number of sentences)
    window_size: usize,
    /// Number of smoothing passes for boundary detection
    smoothing_passes: usize,
    /// Base chunking configuration
    config: ChunkingConfig,
}

impl TopicChunker {
    /// Create a new topic chunker
    pub fn new(window_size: usize, smoothing_passes: usize, config: ChunkingConfig) -> Self {
        Self {
            window_size: window_size.max(2), // Minimum window size of 2
            smoothing_passes: smoothing_passes.min(5), // Maximum 5 passes for performance
            config,
        }
    }

    /// Split text into pseudo-sentences for analysis (optimized)
    fn tokenize_sentences(&self, text: &str) -> Vec<String> {
        // Early exit for very large texts to maintain performance
        if text.len() > 100_000 {
            // For large texts, use simpler chunking by splitting on double newlines or periods
            return text
                .split_terminator(&['.', '!', '?'])
                .filter_map(|s| {
                    let trimmed = s.trim();
                    if trimmed.len() > 10 && trimmed.split_whitespace().count() >= 3 {
                        Some(trimmed.to_string())
                    } else {
                        None
                    }
                })
                .take(500) // Limit number of sentences for very large documents
                .collect();
        }

        let mut sentences = Vec::with_capacity(text.len() / 100); // Estimate capacity
        let mut current_sentence = String::new();
        let mut word_count = 0;

        for word in text.split_whitespace() {
            current_sentence.push_str(word);
            current_sentence.push(' ');
            word_count += 1;

            // End sentence on punctuation or after ~20 words (pseudo-sentence)
            if (word.ends_with('.') || word.ends_with('!') || word.ends_with('?') || word_count >= 20)
                && !current_sentence.trim().is_empty() {
                sentences.push(current_sentence.trim().to_string());
                current_sentence.clear();
                word_count = 0;
            }
        }

        // Add remaining content
        if !current_sentence.trim().is_empty() {
            sentences.push(current_sentence.trim().to_string());
        }

        sentences
    }

    /// Extract vocabulary from a text block
    fn extract_vocabulary(&self, text: &str) -> HashMap<String, usize> {
        let mut vocab = HashMap::new();

        for word in text.split_whitespace() {
            let cleaned = word
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase();

            if cleaned.len() > 2 && !self.is_stop_word(&cleaned) {
                *vocab.entry(cleaned).or_insert(0) += 1;
            }
        }

        vocab
    }

    /// Check if a word is a stop word
    fn is_stop_word(&self, word: &str) -> bool {
        const STOP_WORDS: &[&str] = &[
            "the", "be", "to", "of", "and", "a", "in", "that", "have",
            "i", "it", "for", "not", "on", "with", "he", "as", "you",
            "do", "at", "this", "but", "his", "by", "from", "they",
            "we", "say", "her", "she", "or", "an", "will", "my",
            "one", "all", "would", "there", "their", "what", "so",
            "up", "out", "if", "about", "who", "get", "which", "go",
        ];

        STOP_WORDS.contains(&word)
    }

    /// Calculate cosine similarity between two vocabulary maps (optimized)
    fn cosine_similarity(&self, vocab1: &HashMap<String, usize>, vocab2: &HashMap<String, usize>) -> f64 {
        if vocab1.is_empty() || vocab2.is_empty() {
            return 0.0;
        }

        let mut dot_product = 0.0;
        let mut norm1 = 0.0;
        let mut norm2 = 0.0;

        // Optimize by iterating over the smaller vocabulary first
        let (smaller, larger) = if vocab1.len() <= vocab2.len() {
            (vocab1, vocab2)
        } else {
            (vocab2, vocab1)
        };

        // Calculate norms and dot product efficiently
        for (word, &count1) in smaller {
            let count1_f64 = count1 as f64;
            norm1 += count1_f64 * count1_f64;

            if let Some(&count2) = larger.get(word) {
                let count2_f64 = count2 as f64;
                dot_product += count1_f64 * count2_f64;
            }
        }

        // Complete norm2 calculation
        for &count2 in larger.values() {
            let count2_f64 = count2 as f64;
            norm2 += count2_f64 * count2_f64;
        }

        // If we swapped, we need to swap back the norms
        if vocab1.len() > vocab2.len() {
            std::mem::swap(&mut norm1, &mut norm2);
        }

        if norm1 == 0.0 || norm2 == 0.0 {
            return 0.0;
        }

        dot_product / (norm1.sqrt() * norm2.sqrt())
    }

    /// Calculate depth scores using TextTiling algorithm (optimized)
    fn calculate_depth_scores(&self, sentences: &[String]) -> Vec<f64> {
        if sentences.len() < self.window_size * 2 {
            return vec![0.0; sentences.len().saturating_sub(1)];
        }

        // Pre-compute vocabulary for all sentences to avoid repeated work
        let sentence_vocabs: Vec<HashMap<String, usize>> = sentences
            .iter()
            .map(|s| self.extract_vocabulary(s))
            .collect();

        let mut scores = Vec::new();

        // Calculate similarity for each potential boundary
        for i in self.window_size..(sentences.len() - self.window_size) {
            // Combine vocabularies for left and right blocks efficiently
            let mut left_vocab = HashMap::new();
            let mut right_vocab = HashMap::new();

            // Left block: combine vocabularies from (i - window_size) to i
            for vocab in sentence_vocabs.iter().skip(i - self.window_size).take(self.window_size) {
                for (word, &count) in vocab {
                    *left_vocab.entry(word.clone()).or_insert(0) += count;
                }
            }

            // Right block: combine vocabularies from i to (i + window_size)
            for vocab in sentence_vocabs.iter().skip(i).take(self.window_size) {
                for (word, &count) in vocab {
                    *right_vocab.entry(word.clone()).or_insert(0) += count;
                }
            }

            // Calculate similarity (higher = more coherent)
            let similarity = self.cosine_similarity(&left_vocab, &right_vocab);

            // Depth score is inverse of similarity (higher = better boundary)
            scores.push(1.0 - similarity);
        }

        scores
    }

    /// Apply smoothing to depth scores
    fn smooth_scores(&self, scores: &[f64]) -> Vec<f64> {
        if scores.len() < 3 {
            return scores.to_vec();
        }

        let mut smoothed = scores.to_vec();

        for _ in 0..self.smoothing_passes {
            let mut new_smoothed = smoothed.clone();

            // Apply 3-point moving average
            for i in 1..(smoothed.len() - 1) {
                new_smoothed[i] = (smoothed[i - 1] + smoothed[i] + smoothed[i + 1]) / 3.0;
            }

            smoothed = new_smoothed;
        }

        smoothed
    }

    /// Identify topic boundaries using valley detection (improved)
    fn identify_boundaries(&self, scores: &[f64], sentences: &[String]) -> Vec<usize> {
        if scores.len() < 2 {
            return Vec::new();
        }

        let mut boundaries = Vec::new();

        // Calculate adaptive threshold
        let mean = scores.iter().sum::<f64>() / scores.len() as f64;
        let variance = scores.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / scores.len() as f64;
        let std_dev = variance.sqrt();

        // Use more sensitive threshold for boundary detection
        let threshold = if std_dev > 0.1 {
            mean + std_dev * 0.3 // Lower multiplier for more boundaries
        } else {
            mean + 0.05 // Minimum threshold when variance is low
        };

        // Find local maxima above threshold
        for i in 1..(scores.len() - 1) {
            if scores[i] > threshold &&
               scores[i] > scores[i - 1] &&
               scores[i] > scores[i + 1] {
                // Map back to sentence index
                let sentence_index = i + self.window_size;
                if sentence_index < sentences.len() {
                    boundaries.push(sentence_index);
                }
            }
        }

        // If no boundaries found with adaptive threshold, use percentile-based approach
        if boundaries.is_empty() && scores.len() > 4 {
            let mut sorted_scores = scores.to_vec();
            sorted_scores.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
            let percentile_threshold = sorted_scores[scores.len() / 3]; // Top 33% of scores

            for i in 1..(scores.len() - 1) {
                if scores[i] >= percentile_threshold &&
                   scores[i] > scores[i - 1] &&
                   scores[i] > scores[i + 1] {
                    let sentence_index = i + self.window_size;
                    if sentence_index < sentences.len() {
                        boundaries.push(sentence_index);
                    }
                }
            }
        }

        // Ensure minimum chunk size by merging too-small segments
        self.enforce_minimum_chunk_size(boundaries, sentences)
    }

    /// Enforce minimum chunk size by merging small segments
    fn enforce_minimum_chunk_size(&self, boundaries: Vec<usize>, sentences: &[String]) -> Vec<usize> {
        if boundaries.is_empty() {
            return boundaries;
        }

        let min_chars = self.config.min_chunk_size;
        let mut filtered_boundaries = Vec::new();
        let mut last_boundary = 0;

        for &boundary in &boundaries {
            // Calculate size of segment from last boundary to current
            let segment = sentences[last_boundary..boundary].join(" ");

            if segment.len() >= min_chars {
                filtered_boundaries.push(boundary);
                last_boundary = boundary;
            }
            // If too small, skip this boundary (merge with next segment)
        }

        // Check final segment
        if last_boundary < sentences.len() {
            let final_segment = sentences[last_boundary..].join(" ");
            if final_segment.len() < min_chars && !filtered_boundaries.is_empty() {
                // Remove last boundary to merge final segment
                filtered_boundaries.pop();
            }
        }

        filtered_boundaries
    }

    /// Create chunks from sentences and boundaries
    fn create_chunks_from_boundaries(&self, sentences: &[String], boundaries: &[usize]) -> Vec<Chunk> {
        let mut chunks = Vec::new();
        let mut start_idx = 0;
        let mut char_position = 0;

        // Add boundaries + final position
        let mut all_boundaries = boundaries.to_vec();
        all_boundaries.push(sentences.len());

        for (chunk_index, &end_idx) in all_boundaries.iter().enumerate() {
            if start_idx >= end_idx {
                continue;
            }

            let chunk_sentences = &sentences[start_idx..end_idx];
            let content = chunk_sentences.join(" ");

            if content.trim().is_empty() {
                start_idx = end_idx;
                continue;
            }

            let start_pos = char_position;
            let end_pos = start_pos + content.len();

            // Calculate metadata
            let word_count = content.split_whitespace().count();
            let sentence_count = chunk_sentences.len();
            let token_count = crate::chunking::utils::count_tokens(&content);
            let topic_keywords = crate::chunking::utils::extract_topic_keywords(&content);

            let metadata = ChunkMetadata {
                quality_score: self.calculate_topic_quality_score(&content, &topic_keywords),
                sentence_count,
                word_count,
                has_complete_sentences: sentence_count > 0,
                topic_keywords,
                chunk_type: "topic".to_string(),
                custom: HashMap::new(),
            };

            let chunk = Chunk {
                id: Uuid::new_v4().to_string(),
                content: content.clone(),
                start_pos,
                end_pos,
                token_count,
                chunk_index,
                total_chunks: all_boundaries.len(), // Will be updated later
                metadata,
            };

            chunks.push(chunk);
            char_position = end_pos + 1; // +1 for space between chunks
            start_idx = end_idx;
        }

        // Update total_chunks in all chunks
        let total = chunks.len();
        for chunk in &mut chunks {
            chunk.total_chunks = total;
        }

        chunks
    }

    /// Calculate quality score specific to topic chunking
    fn calculate_topic_quality_score(&self, content: &str, topic_keywords: &[String]) -> f64 {
        let mut score = 0.5; // Base score

        // Length score (optimal around 1000 characters for topic chunks)
        let length_ratio = (content.len() as f64 / 1000.0).min(1.0);
        score += length_ratio * 0.2;

        // Topic coherence bonus based on keyword density
        if !topic_keywords.is_empty() {
            let keyword_density = topic_keywords.len() as f64 / content.split_whitespace().count() as f64;
            score += (keyword_density * 10.0).min(0.3); // Cap at 30% bonus
        }

        // Sentence structure bonus
        let sentences = crate::chunking::utils::split_sentences(content);
        if sentences.len() >= 3 {
            score += 0.2;
        }

        score.min(1.0)
    }
}

#[async_trait]
impl ChunkingStrategy for TopicChunker {
    async fn chunk(&self, text: &str) -> Result<Vec<Chunk>> {
        if text.trim().is_empty() {
            return Ok(Vec::new());
        }

        let start_time = std::time::Instant::now();

        // Step 1: Tokenize into sentences
        let sentences = self.tokenize_sentences(text);

        // If too few sentences, fall back to single chunk
        if sentences.len() < self.window_size * 2 {
            let metadata = ChunkMetadata {
                quality_score: 0.7,
                sentence_count: sentences.len(),
                word_count: text.split_whitespace().count(),
                has_complete_sentences: true,
                topic_keywords: crate::chunking::utils::extract_topic_keywords(text),
                chunk_type: "topic-single".to_string(),
                custom: HashMap::new(),
            };

            return Ok(vec![Chunk {
                id: Uuid::new_v4().to_string(),
                content: text.to_string(),
                start_pos: 0,
                end_pos: text.len(),
                token_count: crate::chunking::utils::count_tokens(text),
                chunk_index: 0,
                total_chunks: 1,
                metadata,
            }]);
        }

        // Step 2: Calculate depth scores
        let depth_scores = self.calculate_depth_scores(&sentences);

        // Step 3: Apply smoothing
        let smoothed_scores = self.smooth_scores(&depth_scores);

        // Step 4: Identify boundaries
        let boundaries = self.identify_boundaries(&smoothed_scores, &sentences);

        // Step 5: Create chunks
        let chunks = self.create_chunks_from_boundaries(&sentences, &boundaries);

        // Performance check
        let duration = start_time.elapsed();
        if duration.as_millis() > 200 {
            eprintln!(
                "Warning: Topic chunking took {}ms for {} characters (target: <200ms)",
                duration.as_millis(),
                text.len()
            );
        }

        Ok(chunks)
    }

    fn name(&self) -> &str {
        "topic"
    }

    fn config(&self) -> ChunkingConfig {
        self.config.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_topic_chunking_basic() {
        let config = ChunkingConfig::default();
        let chunker = TopicChunker::new(3, 2, config);

        let text = "Introduction to machine learning. Machine learning is a subset of artificial intelligence. It focuses on algorithms that learn from data. \
                   Deep learning is a special case. Deep learning uses neural networks with many layers. These networks can learn complex patterns. \
                   Natural language processing is another field. NLP deals with text and speech. It helps computers understand human language.";

        let chunks = chunker.chunk(text).await.unwrap();

        assert!(!chunks.is_empty());
        assert!(!chunks.is_empty());

        // Check metadata
        for chunk in &chunks {
            assert!(!chunk.content.is_empty());
            assert!(chunk.metadata.quality_score > 0.0);
            assert_eq!(chunk.metadata.chunk_type, "topic");
        }
    }

    #[tokio::test]
    async fn test_short_text_fallback() {
        let config = ChunkingConfig::default();
        let chunker = TopicChunker::new(5, 2, config);

        let text = "Short text.";
        let chunks = chunker.chunk(text).await.unwrap();

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].metadata.chunk_type, "topic-single");
    }

    #[tokio::test]
    async fn test_performance_requirement() {
        let config = ChunkingConfig::default();
        let chunker = TopicChunker::new(3, 2, config);

        // Generate 50KB of topic-diverse content
        let topics = vec![
            "Machine learning algorithms and artificial intelligence systems",
            "Climate change effects on global weather patterns and ecosystems",
            "Economic policies and their impact on international trade relations",
            "Advances in quantum computing and cryptographic security measures",
            "Social media influence on modern communication and society",
        ];

        let mut text = String::new();
        while text.len() < 50_000 {
            for topic in &topics {
                text.push_str(&format!(
                    "{}. This topic involves many complex concepts and ideas. \
                     Research in this area has shown significant progress. \
                     Scientists and experts continue to explore new possibilities. \
                     The implications of these developments are far-reaching. ",
                    topic
                ));
            }
        }

        let start = std::time::Instant::now();
        let chunks = chunker.chunk(&text).await.unwrap();
        let duration = start.elapsed();

        // Should meet <200ms requirement
        assert!(
            duration.as_millis() < 200,
            "Topic chunking took {}ms, expected <200ms",
            duration.as_millis()
        );

        assert!(!chunks.is_empty());

        // Verify chunks have topic keywords
        for chunk in &chunks {
            assert!(!chunk.metadata.topic_keywords.is_empty());
        }
    }

    #[tokio::test]
    async fn test_vocabulary_extraction() {
        let config = ChunkingConfig::default();
        let chunker = TopicChunker::new(3, 2, config);

        let text = "Machine learning algorithms process data efficiently";
        let vocab = chunker.extract_vocabulary(text);

        assert!(vocab.contains_key("machine"));
        assert!(vocab.contains_key("learning"));
        assert!(vocab.contains_key("algorithms"));
        assert!(!vocab.contains_key("the")); // Stop word
    }

    #[tokio::test]
    async fn test_cosine_similarity() {
        let config = ChunkingConfig::default();
        let chunker = TopicChunker::new(3, 2, config);

        let vocab1 = chunker.extract_vocabulary("machine learning algorithms");
        let vocab2 = chunker.extract_vocabulary("machine learning systems");
        let vocab3 = chunker.extract_vocabulary("weather climate patterns");

        // Similar topics should have higher similarity
        let sim_similar = chunker.cosine_similarity(&vocab1, &vocab2);
        let sim_different = chunker.cosine_similarity(&vocab1, &vocab3);

        assert!(sim_similar > sim_different);
        assert!(sim_similar > 0.0);
    }

    #[tokio::test]
    async fn test_boundary_detection() {
        let config = ChunkingConfig::default();
        let chunker = TopicChunker::new(2, 1, config);

        let sentences = vec![
            "Machine learning is complex technology.".to_string(),
            "Algorithms process data efficiently using computational methods.".to_string(),
            "Neural networks learn patterns from training data sets.".to_string(),
            "Deep learning uses multiple layers for feature extraction.".to_string(),
            "Climate change affects weather patterns around the world.".to_string(),
            "Global warming increases temperatures causing environmental shifts.".to_string(),
            "Environmental policies are important for sustainable development.".to_string(),
            "Green energy solutions reduce carbon emissions significantly.".to_string(),
        ];

        let scores = chunker.calculate_depth_scores(&sentences);
        let boundaries = chunker.identify_boundaries(&scores, &sentences);

        // Should detect topic change between ML and climate topics
        // With 8 sentences and window_size=2, we have 4 potential boundaries (indices 2,3,4,5)
        // The algorithm should find at least one boundary between the topics
        assert!(!boundaries.is_empty(), "Should detect at least one topic boundary between ML and climate topics");

        // Verify the boundary makes sense (should be around index 4 where topic changes)
        assert!(boundaries.iter().any(|&b| (3..=5).contains(&b)),
               "Boundary should be detected around the topic transition point (indices 3-5)");
    }
}