//! Sliding window chunking with overlap (default mode)

use anyhow::Result;
use crate::strategies::chunking::*;

/// Chunk content using sliding windows with overlap
pub async fn chunk_sliding_window(
    content: &str,
    config: &ChunkingConfig,
) -> Result<Vec<ContentChunk>> {
    let mut chunks = Vec::new();

    if content.is_empty() {
        return Ok(chunks);
    }

    let token_max = config.token_max;
    let overlap = config.overlap;

    // Split content into sentences for better boundary handling
    let sentences = if config.preserve_sentences {
        split_into_sentences(content)
    } else {
        split_into_words(content)
    };

    let mut current_chunk = String::new();
    let mut current_tokens = 0;
    let mut start_pos = 0;
    let mut chunk_index = 0;
    let mut sentence_buffer = Vec::new();

    for sentence in sentences.iter() {
        let sentence_tokens = count_tokens(sentence);

        // If adding this sentence would exceed token limit, create a chunk
        if current_tokens + sentence_tokens > token_max && !current_chunk.is_empty() {
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
            let overlap_content = if overlap > 0 {
                calculate_overlap(&sentence_buffer, overlap)
            } else {
                String::new()
            };

            // Reset for next chunk
            current_chunk = overlap_content.clone();
            current_tokens = count_tokens(&current_chunk);
            start_pos = if overlap_content.is_empty() {
                start_pos + chunks.last().unwrap().content.len()
            } else {
                start_pos + chunks.last().unwrap().content.len() - overlap_content.len()
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

/// Create a content chunk with metadata
fn create_chunk(
    content: &str,
    start_pos: usize,
    end_pos: usize,
    token_count: usize,
    chunk_index: usize,
    sentences: &[String],
) -> ContentChunk {
    let word_count = content.split_whitespace().count();
    let sentence_count = sentences.len();
    let has_complete_sentences = content.trim().ends_with('.') ||
                                 content.trim().ends_with('!') ||
                                 content.trim().ends_with('?');
    let topic_keywords = extract_topic_keywords(content);

    let metadata = ChunkMetadata {
        quality_score: 0.0, // Will be calculated
        sentence_count,
        word_count,
        has_complete_sentences,
        topic_keywords: topic_keywords.clone(),
        chunk_type: "sliding".to_string(),
    };

    let quality_score = calculate_chunk_quality(content, &metadata);

    ContentChunk {
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
        let sentence_tokens = count_tokens(sentence);
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

/// Split content into sentences
fn split_into_sentences(content: &str) -> Vec<String> {
    // Simple sentence splitting - can be enhanced with more sophisticated NLP
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

    // Add remaining content as a sentence
    if !current_sentence.trim().is_empty() {
        sentences.push(current_sentence.trim().to_string());
    }

    // Filter out very short sentences
    sentences
        .into_iter()
        .filter(|s| s.split_whitespace().count() >= 3)
        .collect()
}

/// Split content into words for non-sentence-preserving mode
fn split_into_words(content: &str) -> Vec<String> {
    content
        .split_whitespace()
        .map(|word| word.to_string())
        .collect()
}

/// Check if a word ending with punctuation is likely an abbreviation
fn is_abbreviation(word: &str) -> bool {
    const COMMON_ABBREVIATIONS: &[&str] = &[
        "mr.", "mrs.", "ms.", "dr.", "prof.", "sr.", "jr.",
        "inc.", "ltd.", "corp.", "co.", "etc.", "vs.", "vol.",
        "no.", "pp.", "fig.", "ch.", "sec.", "dept.", "govt.",
        "u.s.", "u.k.", "e.g.", "i.e.", "a.m.", "p.m.",
    ];

    let lower = word.to_lowercase();
    COMMON_ABBREVIATIONS.contains(&lower.as_str()) ||
    (word.len() <= 4 && word.chars().filter(|c| c.is_uppercase()).count() > 1)
}