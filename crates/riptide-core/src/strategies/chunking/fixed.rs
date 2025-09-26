//! Fixed-size chunking by characters or tokens

use anyhow::Result;
use crate::strategies::chunking::*;

/// Chunk content into fixed-size pieces
pub async fn chunk_fixed_size(
    content: &str,
    size: usize,
    by_tokens: bool,
    config: &ChunkingConfig,
) -> Result<Vec<ContentChunk>> {
    let mut chunks = Vec::new();

    if content.is_empty() {
        return Ok(chunks);
    }

    if by_tokens {
        chunk_by_tokens(content, size, config, &mut chunks).await?;
    } else {
        chunk_by_characters(content, size, config, &mut chunks).await?;
    }

    // Update total chunk count
    let total_chunks = chunks.len();
    for chunk in &mut chunks {
        chunk.total_chunks = total_chunks;
    }

    Ok(chunks)
}

/// Chunk content by token count
async fn chunk_by_tokens(
    content: &str,
    token_size: usize,
    config: &ChunkingConfig,
    chunks: &mut Vec<ContentChunk>,
) -> Result<()> {
    let words: Vec<&str> = content.split_whitespace().collect();
    let mut current_chunk = String::new();
    let mut current_tokens = 0;
    let mut start_pos = 0;
    let mut word_start = 0;
    let mut chunk_index = 0;

    for (word_idx, word) in words.iter().enumerate() {
        let word_tokens = count_tokens(word);

        // If adding this word would exceed token limit, create a chunk
        if current_tokens + word_tokens > token_size && !current_chunk.is_empty() {
            let chunk = create_fixed_chunk(
                &current_chunk,
                start_pos,
                current_tokens,
                chunk_index,
                "fixed_token",
                config.preserve_sentences,
            );
            chunks.push(chunk);

            // Reset for next chunk
            start_pos = find_word_position(content, word_start);
            current_chunk.clear();
            current_tokens = 0;
            chunk_index += 1;
            word_start = word_idx;
        }

        // Add word to current chunk
        if !current_chunk.is_empty() {
            current_chunk.push(' ');
        }
        current_chunk.push_str(word);
        current_tokens += word_tokens;
    }

    // Add final chunk if there's remaining content
    if !current_chunk.is_empty() {
        let chunk = create_fixed_chunk(
            &current_chunk,
            start_pos,
            current_tokens,
            chunk_index,
            "fixed_token",
            config.preserve_sentences,
        );
        chunks.push(chunk);
    }

    Ok(())
}

/// Chunk content by character count
async fn chunk_by_characters(
    content: &str,
    char_size: usize,
    config: &ChunkingConfig,
    chunks: &mut Vec<ContentChunk>,
) -> Result<()> {
    let mut start_pos = 0;
    let mut chunk_index = 0;

    while start_pos < content.len() {
        let mut end_pos = (start_pos + char_size).min(content.len());

        // If preserving sentences, adjust to sentence boundary
        if config.preserve_sentences && end_pos < content.len() {
            end_pos = find_sentence_boundary(content, end_pos);
        }

        // If preserving sentences failed to find a good boundary, try word boundary
        if config.preserve_sentences && end_pos <= start_pos {
            end_pos = find_word_boundary(content, start_pos + char_size);
        }

        // Ensure we make progress
        if end_pos <= start_pos {
            end_pos = (start_pos + char_size).min(content.len());
        }

        let chunk_content = &content[start_pos..end_pos];
        let token_count = count_tokens(chunk_content);

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
        chunk_index += 1;
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
) -> ContentChunk {
    let end_pos = start_pos + content.len();
    let word_count = content.split_whitespace().count();

    // Count sentences
    let sentence_count = content
        .split(['.', '!', '?'])
        .filter(|s| !s.trim().is_empty())
        .count();

    // Check if chunk has complete sentences
    let has_complete_sentences = if preserve_sentences {
        content.trim().ends_with('.') ||
        content.trim().ends_with('!') ||
        content.trim().ends_with('?')
    } else {
        false
    };

    let topic_keywords = extract_topic_keywords(content);

    let metadata = ChunkMetadata {
        quality_score: 0.0, // Will be calculated
        sentence_count,
        word_count,
        has_complete_sentences,
        topic_keywords: topic_keywords.clone(),
        chunk_type: chunk_type.to_string(),
    };

    let quality_score = calculate_chunk_quality(content, &metadata);

    ContentChunk {
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
        "mr", "mrs", "ms", "dr", "prof", "sr", "jr",
        "inc", "ltd", "corp", "co", "etc", "vs", "vol",
        "no", "pp", "fig", "ch", "sec", "dept", "govt",
        "jan", "feb", "mar", "apr", "may", "jun",
        "jul", "aug", "sep", "oct", "nov", "dec",
    ];

    COMMON_ABBREVIATIONS.contains(&word) || word.len() <= 3
}