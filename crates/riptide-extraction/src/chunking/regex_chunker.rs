//! Regex pattern-based chunking for custom text splitting

use super::{utils, Chunk, ChunkMetadata, ChunkingConfig, ChunkingStrategy};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use regex::Regex;

/// Regex-based chunker that splits content using custom patterns
pub struct RegexChunker {
    pattern: String,
    min_chunk_size: usize,
    config: ChunkingConfig,
    regex: Regex,
}

impl RegexChunker {
    /// Create a new regex-based chunker
    pub fn new(pattern: String, min_chunk_size: usize, config: ChunkingConfig) -> Self {
        let regex = Regex::new(&pattern).unwrap_or_else(|_| {
            // Fallback to paragraph splitting if regex is invalid
            // This fallback regex is simple and should always compile
            Regex::new(r"\n\s*\n").unwrap_or_else(|_| {
                // Last resort: use single newline
                Regex::new(r"\n").unwrap_or_else(|_| {
                    // Unreachable: newline regex is always valid
                    panic!("Failed to compile even simplest regex")
                })
            })
        });

        Self {
            pattern,
            min_chunk_size,
            config,
            regex,
        }
    }

    /// Create a chunker with a predefined pattern
    pub fn with_pattern(
        pattern_name: &str,
        min_chunk_size: usize,
        config: ChunkingConfig,
    ) -> Result<Self> {
        let pattern = get_pattern(pattern_name)
            .ok_or_else(|| anyhow!("Unknown pattern: {}", pattern_name))?;

        let regex = Regex::new(&pattern)
            .map_err(|e| anyhow!("Invalid regex pattern '{}': {}", pattern, e))?;

        Ok(Self {
            pattern,
            min_chunk_size,
            config,
            regex,
        })
    }
}

#[async_trait]
impl ChunkingStrategy for RegexChunker {
    async fn chunk(&self, text: &str) -> Result<Vec<Chunk>> {
        let mut chunks = Vec::new();

        if text.is_empty() {
            return Ok(chunks);
        }

        // Split content by regex pattern
        let splits: Vec<&str> = self.regex.split(text).collect();

        if splits.len() <= 1 {
            // No matches found, return entire content as single chunk
            let token_count = utils::count_tokens(text);
            let chunk = create_regex_chunk(text, 0, token_count, 0, &self.pattern);
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
            let should_chunk = current_chunk.len() >= self.min_chunk_size
                || utils::count_tokens(&current_chunk) >= self.config.max_tokens
                || i == splits.len() - 1; // Last split

            if should_chunk {
                let token_count = utils::count_tokens(&current_chunk);
                let chunk = create_regex_chunk(
                    &current_chunk,
                    current_pos,
                    token_count,
                    chunk_index,
                    &self.pattern,
                );
                chunks.push(chunk);

                // Reset for next chunk
                current_pos += current_chunk.len();
                current_chunk.clear();
                chunk_index += 1;
            }
        }

        // Add final chunk if there's remaining content
        if !current_chunk.is_empty() {
            let token_count = utils::count_tokens(&current_chunk);
            let chunk = create_regex_chunk(
                &current_chunk,
                current_pos,
                token_count,
                chunk_index,
                &self.pattern,
            );
            chunks.push(chunk);
        }

        // Merge small chunks if necessary
        let merged_chunks = merge_small_chunks(chunks, self.min_chunk_size);

        // Update total chunk count
        let total_chunks = merged_chunks.len();
        let mut final_chunks = merged_chunks;
        for chunk in &mut final_chunks {
            chunk.total_chunks = total_chunks;
        }

        Ok(final_chunks)
    }

    fn name(&self) -> &str {
        "regex"
    }

    fn config(&self) -> ChunkingConfig {
        self.config.clone()
    }
}

/// Create a regex-based chunk with metadata
fn create_regex_chunk(
    content: &str,
    start_pos: usize,
    token_count: usize,
    chunk_index: usize,
    pattern: &str,
) -> Chunk {
    let end_pos = start_pos + content.len();
    let word_count = content.split_whitespace().count();

    // Count sentences (rough estimate)
    let sentence_count = content.matches(['.', '!', '?']).count().max(1); // At least 1 if content exists

    // Check for complete sentences
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
        chunk_type: format!("regex_{}", sanitize_pattern(pattern)),
        custom: std::collections::HashMap::new(),
    };

    let quality_score = utils::calculate_quality_score(content, &metadata);

    Chunk {
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
fn merge_small_chunks(chunks: Vec<Chunk>, min_size: usize) -> Vec<Chunk> {
    if chunks.is_empty() {
        return chunks;
    }

    let mut merged = Vec::new();
    let mut current_chunk: Option<Chunk> = None;

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
                    curr.metadata
                        .topic_keywords
                        .extend(chunk.metadata.topic_keywords);
                    curr.metadata.quality_score =
                        utils::calculate_quality_score(&curr.content, &curr.metadata);

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
        ("paragraph", r"\n\s*\n"),
        ("heading", r"\n#{1,6}\s+"),
        ("section", r"\n---+\n|\n===+\n"),
        ("bullet_list", r"\n\s*[-*+]\s+"),
        ("numbered_list", r"\n\s*\d+\.\s+"),
        ("code_block", r"```[\s\S]*?```"),
        ("html_tags", r"<[^>]+>"),
        ("sentences", r"[.!?]+\s+"),
        ("double_newline", r"\n\s*\n"),
        ("markdown_headers", r"\n#{1,6}[^\n]*\n"),
        ("xml_elements", r"</[^>]+>"),
        ("citations", r"\[\d+\]"),
        ("footnotes", r"\n\[\d+\]:"),
    ]
}

/// Get pattern by name
pub fn get_pattern(name: &str) -> Option<String> {
    common_patterns()
        .into_iter()
        .find(|(pattern_name, _)| *pattern_name == name)
        .map(|(_, pattern)| pattern.to_string())
}

/// Get all available pattern names
pub fn pattern_names() -> Vec<&'static str> {
    common_patterns()
        .into_iter()
        .map(|(name, _)| name)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_regex_chunking_paragraphs() {
        let config = ChunkingConfig::default();
        let chunker = RegexChunker::with_pattern("paragraph", 50, config).unwrap();

        let text =
            "First paragraph content.\n\nSecond paragraph content.\n\nThird paragraph content.";
        let chunks = chunker.chunk(text).await.unwrap();

        assert!(!chunks.is_empty());
        // Paragraph pattern (\n\s*\n) sanitizes to "nsn"
        assert_eq!(chunks[0].metadata.chunk_type, "regex_nsn");
    }

    #[tokio::test]
    async fn test_regex_chunking_headings() {
        let config = ChunkingConfig::default();
        let chunker = RegexChunker::with_pattern("heading", 20, config).unwrap();

        let text = "# First Heading\nContent under first heading.\n## Second Heading\nContent under second heading.";
        let chunks = chunker.chunk(text).await.unwrap();

        assert!(!chunks.is_empty());
    }

    #[tokio::test]
    async fn test_custom_pattern() {
        let config = ChunkingConfig::default();
        let chunker = RegexChunker::new(r"\d+\.".to_string(), 20, config);

        let text = "1. First item content. 2. Second item content. 3. Third item content.";
        let chunks = chunker.chunk(text).await.unwrap();

        assert!(!chunks.is_empty());
    }

    #[tokio::test]
    async fn test_merge_small_chunks() {
        let chunks = vec![
            Chunk {
                id: "test_1".to_string(),
                content: "Small".to_string(),
                start_pos: 0,
                end_pos: 5,
                token_count: 1,
                chunk_index: 0,
                total_chunks: 0,
                metadata: ChunkMetadata {
                    quality_score: 0.5,
                    sentence_count: 1,
                    word_count: 1,
                    has_complete_sentences: false,
                    topic_keywords: vec![],
                    chunk_type: "test".to_string(),
                    custom: std::collections::HashMap::new(),
                },
            },
            Chunk {
                id: "test_2".to_string(),
                content: "Also small".to_string(),
                start_pos: 6,
                end_pos: 16,
                token_count: 2,
                chunk_index: 1,
                total_chunks: 0,
                metadata: ChunkMetadata {
                    quality_score: 0.5,
                    sentence_count: 1,
                    word_count: 2,
                    has_complete_sentences: false,
                    topic_keywords: vec![],
                    chunk_type: "test".to_string(),
                    custom: std::collections::HashMap::new(),
                },
            },
        ];

        let merged = merge_small_chunks(chunks, 50);
        assert_eq!(merged.len(), 1);
        assert!(merged[0].content.contains("Small"));
        assert!(merged[0].content.contains("Also small"));
    }

    #[tokio::test]
    async fn test_pattern_names() {
        let names = pattern_names();
        assert!(names.contains(&"paragraph"));
        assert!(names.contains(&"heading"));
        assert!(names.contains(&"section"));
    }

    #[tokio::test]
    async fn test_get_pattern() {
        assert!(get_pattern("paragraph").is_some());
        assert!(get_pattern("nonexistent").is_none());
    }
}
