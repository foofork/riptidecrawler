//! Sentence-based chunking using NLP boundaries

use anyhow::Result;
use crate::strategies::chunking::*;

/// Chunk content by sentence boundaries
pub async fn chunk_by_sentences(
    content: &str,
    max_sentences: usize,
    config: &ChunkingConfig,
) -> Result<Vec<ContentChunk>> {
    let mut chunks = Vec::new();

    if content.is_empty() {
        return Ok(chunks);
    }

    let sentences = detect_sentences(content);
    let mut current_chunk_sentences = Vec::new();
    let mut current_tokens = 0;
    let mut start_pos = 0;
    let mut chunk_index = 0;

    for sentence in sentences {
        let sentence_tokens = count_tokens(&sentence.text);

        // Check if adding this sentence would exceed limits
        let would_exceed_sentences = current_chunk_sentences.len() >= max_sentences;
        let would_exceed_tokens = current_tokens + sentence_tokens > config.token_max;

        if (would_exceed_sentences || would_exceed_tokens) && !current_chunk_sentences.is_empty() {
            // Create chunk from current sentences
            let chunk = create_sentence_chunk(
                &current_chunk_sentences,
                start_pos,
                current_tokens,
                chunk_index,
            );
            chunks.push(chunk);

            // Reset for next chunk
            start_pos = current_chunk_sentences.last().unwrap().end_pos;
            current_chunk_sentences.clear();
            current_tokens = 0;
            chunk_index += 1;
        }

        current_chunk_sentences.push(sentence);
        current_tokens += sentence_tokens;
    }

    // Add final chunk if there are remaining sentences
    if !current_chunk_sentences.is_empty() {
        let chunk = create_sentence_chunk(
            &current_chunk_sentences,
            start_pos,
            current_tokens,
            chunk_index,
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

/// Sentence structure with position information
#[derive(Debug, Clone)]
struct Sentence {
    text: String,
    start_pos: usize,
    end_pos: usize,
    confidence: f64,
}

/// Create a chunk from sentences
fn create_sentence_chunk(
    sentences: &[Sentence],
    start_pos: usize,
    token_count: usize,
    chunk_index: usize,
) -> ContentChunk {
    let content = sentences
        .iter()
        .map(|s| s.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");

    let end_pos = sentences.last().unwrap().end_pos;
    let word_count = content.split_whitespace().count();
    let sentence_count = sentences.len();
    let topic_keywords = extract_topic_keywords(&content);

    // All sentences are complete by definition
    let has_complete_sentences = true;

    let metadata = ChunkMetadata {
        quality_score: 0.0, // Will be calculated
        sentence_count,
        word_count,
        has_complete_sentences,
        topic_keywords: topic_keywords.clone(),
        chunk_type: "sentence".to_string(),
    };

    let quality_score = calculate_chunk_quality(&content, &metadata);

    ContentChunk {
        id: format!("sentence_{}_{}", chunk_index, start_pos),
        content,
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

/// Advanced sentence detection with confidence scoring
fn detect_sentences(content: &str) -> Vec<Sentence> {
    let mut sentences = Vec::new();
    let mut current_sentence = String::new();
    let mut sentence_start = 0;
    let mut pos = 0;

    // Split into potential sentence boundaries
    let words: Vec<&str> = content.split_whitespace().collect();

    for (i, word) in words.iter().enumerate() {
        if !current_sentence.is_empty() {
            current_sentence.push(' ');
        }
        current_sentence.push_str(word);

        // Check for sentence endings
        if is_sentence_end(word, &words, i) {
            let confidence = calculate_sentence_confidence(&current_sentence, word);

            if confidence > 0.5 { // Only accept high-confidence sentence boundaries
                let sentence = Sentence {
                    text: current_sentence.trim().to_string(),
                    start_pos: sentence_start,
                    end_pos: pos + word.len(),
                    confidence,
                };
                sentences.push(sentence);

                // Reset for next sentence
                sentence_start = pos + word.len();
                current_sentence.clear();
            }
        }

        pos += word.len() + 1; // +1 for space
    }

    // Add remaining content as final sentence
    if !current_sentence.trim().is_empty() {
        let sentence = Sentence {
            text: current_sentence.trim().to_string(),
            start_pos: sentence_start,
            end_pos: pos,
            confidence: 0.8,
        };
        sentences.push(sentence);
    }

    // Filter very short sentences
    sentences
        .into_iter()
        .filter(|s| s.text.split_whitespace().count() >= 3)
        .collect()
}

/// Check if a word represents a sentence ending
fn is_sentence_end(word: &str, words: &[&str], index: usize) -> bool {
    // Must end with sentence punctuation
    if !word.ends_with('.') && !word.ends_with('!') && !word.ends_with('?') {
        return false;
    }

    // Check for abbreviations
    if is_likely_abbreviation(word) {
        return false;
    }

    // Check if next word starts with capital (if available)
    if let Some(next_word) = words.get(index + 1) {
        if next_word.chars().next().unwrap_or('a').is_uppercase() {
            return true;
        }
    }

    // Default to sentence end if at the end of text
    index == words.len() - 1
}

/// Calculate confidence that this is a true sentence boundary
fn calculate_sentence_confidence(sentence: &str, ending_word: &str) -> f64 {
    let mut confidence = 0.7_f64; // Base confidence

    // Length bonus
    let word_count = sentence.split_whitespace().count();
    if word_count >= 5 {
        confidence += 0.2;
    } else if word_count < 3 {
        confidence -= 0.3;
    }

    // Punctuation bonus
    if ending_word.ends_with('!') || ending_word.ends_with('?') {
        confidence += 0.1;
    }

    // Penalize potential abbreviations
    if is_likely_abbreviation(ending_word) {
        confidence -= 0.4;
    }

    // Grammar indicators
    if sentence.contains(" the ") || sentence.contains(" and ") || sentence.contains(" is ") {
        confidence += 0.1;
    }

    confidence.max(0.0_f64).min(1.0_f64)
}

/// Enhanced abbreviation detection
fn is_likely_abbreviation(word: &str) -> bool {
    const COMMON_ABBREVIATIONS: &[&str] = &[
        "mr.", "mrs.", "ms.", "dr.", "prof.", "sr.", "jr.",
        "inc.", "ltd.", "corp.", "co.", "etc.", "vs.", "vol.",
        "no.", "pp.", "fig.", "ch.", "sec.", "dept.", "govt.",
        "u.s.", "u.k.", "e.g.", "i.e.", "a.m.", "p.m.",
        "jan.", "feb.", "mar.", "apr.", "may.", "jun.",
        "jul.", "aug.", "sep.", "oct.", "nov.", "dec.",
        "mon.", "tue.", "wed.", "thu.", "fri.", "sat.", "sun.",
    ];

    let lower = word.to_lowercase();

    // Check against known abbreviations
    if COMMON_ABBREVIATIONS.contains(&lower.as_str()) {
        return true;
    }

    // Check patterns that suggest abbreviations
    if word.len() <= 4 && word.chars().filter(|c| c.is_uppercase()).count() > 1 {
        return true;
    }

    // Single letter followed by period
    if word.len() == 2 && word.ends_with('.') && word.chars().next().unwrap().is_uppercase() {
        return true;
    }

    false
}