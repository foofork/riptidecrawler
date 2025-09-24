//! Regex pattern-based chunking

use anyhow::{Result, anyhow};
use ::regex::Regex;
use crate::strategies::chunking::*;

/// Chunk content by regex pattern
pub async fn chunk_by_regex(
    content: &str,
    pattern: &str,
    min_chunk_size: usize,
    config: &ChunkingConfig,
) -> Result<Vec<ContentChunk>> {
    let mut chunks = Vec::new();

    if content.is_empty() {
        return Ok(chunks);
    }

    let regex = Regex::new(pattern)
        .map_err(|e| anyhow!("Invalid regex pattern '{}': {}", pattern, e))?;

    // Split content by regex pattern
    let splits: Vec<&str> = regex.split(content).collect();

    if splits.len() <= 1 {
        // No matches found, return entire content as single chunk
        let token_count = count_tokens(content);
        let chunk = create_regex_chunk(content, 0, token_count, 0, pattern);
        chunks.push(chunk);
        return Ok(chunks);
    }

    let mut current_chunk = String::new();
    let mut current_pos = 0;
    let mut chunk_index = 0;

    for (i, split) in splits.iter().enumerate() {
        let split_trimmed = split.trim();

        if split_trimmed.is_empty() {
            current_pos += split.len();
            continue;
        }

        // Add to current chunk
        if !current_chunk.is_empty() {
            current_chunk.push('\n');
        }
        current_chunk.push_str(split_trimmed);

        // Check if we should create a chunk
        let should_chunk = current_chunk.len() >= min_chunk_size ||
                          count_tokens(&current_chunk) >= config.token_max ||
                          i == splits.len() - 1; // Last split

        if should_chunk {
            let token_count = count_tokens(&current_chunk);
            let chunk = create_regex_chunk(&current_chunk, current_pos, token_count, chunk_index, pattern);
            chunks.push(chunk);

            // Reset for next chunk
            current_pos += current_chunk.len();
            current_chunk.clear();
            chunk_index += 1;
        }
    }

    // Add final chunk if there's remaining content
    if !current_chunk.is_empty() {
        let token_count = count_tokens(&current_chunk);
        let chunk = create_regex_chunk(&current_chunk, current_pos, token_count, chunk_index, pattern);
        chunks.push(chunk);
    }

    // Merge small chunks if necessary
    let merged_chunks = merge_small_chunks(chunks, min_chunk_size);

    // Update total chunk count
    let total_chunks = merged_chunks.len();
    let mut final_chunks = merged_chunks;
    for chunk in &mut final_chunks {
        chunk.total_chunks = total_chunks;
    }

    Ok(final_chunks)
}

/// Create a regex-based chunk with metadata
fn create_regex_chunk(
    content: &str,
    start_pos: usize,
    token_count: usize,
    chunk_index: usize,
    pattern: &str,
) -> ContentChunk {
    let end_pos = start_pos + content.len();
    let word_count = content.split_whitespace().count();

    // Count sentences (rough estimate)
    let sentence_count = content
        .matches(|c| c == '.' || c == '!' || c == '?')
        .count()
        .max(1); // At least 1 if content exists

    // Check for complete sentences
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
        chunk_type: format!("regex_{}", sanitize_pattern(pattern)),
    };

    let quality_score = calculate_chunk_quality(content, &metadata);

    ContentChunk {
        id: format!("regex_{}_{}", chunk_index, start_pos),
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

/// Merge chunks that are too small
fn merge_small_chunks(chunks: Vec<ContentChunk>, min_size: usize) -> Vec<ContentChunk> {
    if chunks.is_empty() {
        return chunks;
    }

    let mut merged = Vec::new();
    let mut current_chunk: Option<ContentChunk> = None;

    for chunk in chunks {
        match current_chunk {
            None => {
                current_chunk = Some(chunk);
            }
            Some(mut curr) => {
                // If current chunk is too small, try to merge
                if curr.content.len() < min_size {
                    // Merge with next chunk
                    curr.content.push('\n');
                    curr.content.push_str(&chunk.content);
                    curr.end_pos = chunk.end_pos;
                    curr.token_count += chunk.token_count;
                    curr.metadata.word_count += chunk.metadata.word_count;
                    curr.metadata.sentence_count += chunk.metadata.sentence_count;
                    curr.metadata.topic_keywords.extend(chunk.metadata.topic_keywords);
                    curr.metadata.quality_score = calculate_chunk_quality(&curr.content, &curr.metadata);

                    current_chunk = Some(curr);
                } else {
                    // Current chunk is good, push it and start new one
                    merged.push(curr);
                    current_chunk = Some(chunk);
                }
            }
        }
    }

    // Add the last chunk
    if let Some(chunk) = current_chunk {
        merged.push(chunk);
    }

    // Update chunk indices
    for (i, chunk) in merged.iter_mut().enumerate() {
        chunk.chunk_index = i;
    }

    merged
}

/// Sanitize regex pattern for use in chunk type
fn sanitize_pattern(pattern: &str) -> String {
    pattern
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .take(20)
        .collect()
}

/// Common regex patterns for content chunking
pub fn common_patterns() -> Vec<(&'static str, &'static str)> {
    vec![
        ("paragraph", r"
\s*
"),
        ("heading", r"
#{1,6}\s+"),
        ("section", r"
---+
|
===+
"),
        ("bullet_list", r"
\s*[-*+]\s+"),
        ("numbered_list", r"
\s*\d+\.\s+"),
        ("code_block", r"```[\s\S]*?```"),
        ("html_tags", r"<[^>]+>"),
        ("sentences", r"[.!?]+\s+"),
    ]
}

/// Get pattern by name
pub fn get_pattern(name: &str) -> Option<String> {
    common_patterns()
        .into_iter()
        .find(|(pattern_name, _)| *pattern_name == name)
        .map(|(_, pattern)| pattern.to_string())
}