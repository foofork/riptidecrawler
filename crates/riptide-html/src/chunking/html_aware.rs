//! HTML-aware chunking that preserves tag integrity

use anyhow::Result;
use async_trait::async_trait;
use scraper::{Html, ElementRef};
use std::collections::HashMap;
use super::{ChunkingStrategy, Chunk, ChunkMetadata, ChunkingConfig, utils};

/// HTML-aware chunker that preserves HTML tag boundaries and structure
pub struct HtmlAwareChunker {
    preserve_blocks: bool,
    preserve_structure: bool,
    config: ChunkingConfig,
}

impl HtmlAwareChunker {
    /// Create a new HTML-aware chunker
    pub fn new(preserve_blocks: bool, preserve_structure: bool, config: ChunkingConfig) -> Self {
        Self {
            preserve_blocks,
            preserve_structure,
            config,
        }
    }
}

#[async_trait]
impl ChunkingStrategy for HtmlAwareChunker {
    async fn chunk(&self, text: &str) -> Result<Vec<Chunk>> {
        let mut chunks = Vec::new();

        if text.is_empty() {
            return Ok(chunks);
        }

        // Check if input looks like HTML
        if text.trim_start().starts_with('<') && text.contains('>') {
            chunks = chunk_html_content(text, &self.config, self.preserve_blocks, self.preserve_structure)?;
        } else {
            // Fallback to text-based chunking for non-HTML content
            chunks = chunk_text_content(text, &self.config)?;
        }

        // Update total chunk count
        let total_chunks = chunks.len();
        for chunk in &mut chunks {
            chunk.total_chunks = total_chunks;
        }

        Ok(chunks)
    }

    fn name(&self) -> &str {
        "html_aware"
    }

    fn config(&self) -> ChunkingConfig {
        self.config.clone()
    }
}

/// Chunk HTML content while preserving tag integrity
fn chunk_html_content(
    html: &str,
    config: &ChunkingConfig,
    preserve_blocks: bool,
    preserve_structure: bool,
) -> Result<Vec<Chunk>> {
    // Parse HTML
    let document = Html::parse_document(html);

    let chunks = if preserve_structure {
        // Structure-preserving chunking: chunk by semantic elements
        chunk_by_html_structure(&document, config)?
    } else if preserve_blocks {
        // Block-preserving chunking: respect block-level elements
        chunk_by_html_blocks(&document, config)?
    } else {
        // Tag-preserving chunking: ensure no mid-tag splits
        chunk_html_safely(&document, config)?
    };

    Ok(chunks)
}

/// Chunk by HTML structure (articles, sections, divs, etc.)
fn chunk_by_html_structure(document: &Html, config: &ChunkingConfig) -> Result<Vec<Chunk>> {
    let mut chunks = Vec::new();
    let mut chunk_index = 0;

    // Define structural elements in order of preference
    let structural_selectors = vec![
        "article", "section", "main", "aside", "header", "footer", "nav",
        "div.content", "div.post", "div.article", "div[id*='content']",
        "div", "p", "h1", "h2", "h3", "h4", "h5", "h6",
    ];

    for selector in structural_selectors {
        let selector_obj = scraper::Selector::parse(selector).unwrap();

        for element in document.select(&selector_obj) {
            let content = extract_element_content(&element, true);

            if content.trim().is_empty() || content.len() < config.min_chunk_size {
                continue;
            }

            let token_count = utils::count_tokens(&content);

            // If element is too large, chunk it further
            if token_count > config.max_tokens {
                let sub_chunks = chunk_large_element(&element, config, chunk_index)?;
                for sub_chunk in sub_chunks {
                    chunks.push(sub_chunk);
                    chunk_index += 1;
                }
            } else {
                let chunk = create_html_chunk(
                    &content,
                    chunk_index,
                    token_count,
                    &element,
                    "structure",
                );
                chunks.push(chunk);
                chunk_index += 1;
            }
        }

        // If we found chunks at this level, don't go deeper
        if !chunks.is_empty() {
            break;
        }
    }

    // Fallback if no structural elements found
    if chunks.is_empty() {
        let content = document.root_element().text().collect::<Vec<_>>().join(" ");
        if !content.trim().is_empty() {
            let token_count = utils::count_tokens(&content);
            let chunk = create_simple_chunk(&content, 0, token_count, "fallback");
            chunks.push(chunk);
        }
    }

    Ok(chunks)
}

/// Chunk by HTML blocks while preserving block-level elements
fn chunk_by_html_blocks(document: &Html, config: &ChunkingConfig) -> Result<Vec<Chunk>> {
    let mut chunks = Vec::new();
    let mut current_chunk = String::new();
    let mut current_tokens = 0;
    let mut chunk_index = 0;

    // Get all block-level elements
    let block_elements = find_block_elements(document);

    for element in block_elements {
        let element_html = extract_element_content(&element, false);
        let element_tokens = utils::count_tokens(&element_html);

        // Check if adding this element would exceed limits
        if current_tokens + element_tokens > config.max_tokens && !current_chunk.is_empty() {
            // Create chunk from current content
            let chunk = create_simple_chunk(&current_chunk, chunk_index, current_tokens, "block");
            chunks.push(chunk);

            // Reset for next chunk
            current_chunk.clear();
            current_tokens = 0;
            chunk_index += 1;
        }

        // Add element to current chunk
        current_chunk.push_str(&element_html);
        current_tokens += element_tokens;
    }

    // Add final chunk
    if !current_chunk.is_empty() {
        let chunk = create_simple_chunk(&current_chunk, chunk_index, current_tokens, "block");
        chunks.push(chunk);
    }

    Ok(chunks)
}

/// Chunk HTML safely without splitting tags
fn chunk_html_safely(document: &Html, config: &ChunkingConfig) -> Result<Vec<Chunk>> {
    let mut chunks = Vec::new();
    let html_content = document.html();

    // Find safe split points (between tags, not within them)
    let safe_points = find_safe_split_points(&html_content);

    let mut start = 0;
    let mut chunk_index = 0;

    for &split_point in &safe_points {
        let chunk_content = &html_content[start..split_point];
        let token_count = utils::count_tokens(chunk_content);

        if token_count >= config.min_chunk_size.max(50) {
            let chunk = create_simple_chunk(chunk_content, chunk_index, token_count, "safe");
            chunks.push(chunk);
            chunk_index += 1;
            start = split_point;
        }
    }

    // Add remaining content
    if start < html_content.len() {
        let remaining = &html_content[start..];
        if !remaining.trim().is_empty() {
            let token_count = utils::count_tokens(remaining);
            let chunk = create_simple_chunk(remaining, chunk_index, token_count, "safe");
            chunks.push(chunk);
        }
    }

    Ok(chunks)
}

/// Chunk plain text content as fallback
fn chunk_text_content(text: &str, config: &ChunkingConfig) -> Result<Vec<Chunk>> {
    let mut chunks = Vec::new();
    let sentences = utils::split_sentences(text);

    let mut current_chunk = String::new();
    let mut current_tokens = 0;
    let mut chunk_index = 0;

    for sentence in sentences {
        let sentence_tokens = utils::count_tokens(&sentence);

        if current_tokens + sentence_tokens > config.max_tokens && !current_chunk.is_empty() {
            let chunk = create_simple_chunk(&current_chunk, chunk_index, current_tokens, "text");
            chunks.push(chunk);

            current_chunk.clear();
            current_tokens = 0;
            chunk_index += 1;
        }

        if !current_chunk.is_empty() {
            current_chunk.push(' ');
        }
        current_chunk.push_str(&sentence);
        current_tokens += sentence_tokens;
    }

    if !current_chunk.is_empty() {
        let chunk = create_simple_chunk(&current_chunk, chunk_index, current_tokens, "text");
        chunks.push(chunk);
    }

    Ok(chunks)
}

/// Extract content from an HTML element
fn extract_element_content(element: &ElementRef, text_only: bool) -> String {
    if text_only {
        element.text().collect::<Vec<_>>().join(" ")
    } else {
        element.html()
    }
}

/// Find all block-level elements in the document
fn find_block_elements(document: &Html) -> Vec<ElementRef<'_>> {
    let block_selector = scraper::Selector::parse(
        "div, p, h1, h2, h3, h4, h5, h6, article, section, aside, header, footer, nav, main, blockquote, ul, ol, li, table, tr, td, th"
    ).unwrap();

    document.select(&block_selector).collect()
}

/// Find safe points to split HTML without breaking tags
fn find_safe_split_points(html: &str) -> Vec<usize> {
    let mut safe_points = Vec::new();
    let mut in_tag = false;
    let mut current_pos = 0;

    for (i, char) in html.char_indices() {
        match char {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                current_pos = i + 1;
            }
            '\n' | ' ' if !in_tag && i > current_pos + 500 => {
                // Safe split point: not in tag and reasonable distance from last split
                safe_points.push(i);
                current_pos = i;
            }
            _ => {}
        }
    }

    // Ensure we have the end position
    if !safe_points.is_empty() && safe_points.last() != Some(&html.len()) {
        safe_points.push(html.len());
    }

    safe_points
}

/// Chunk a large HTML element into smaller pieces
fn chunk_large_element(
    element: &ElementRef<'_>,
    config: &ChunkingConfig,
    start_index: usize,
) -> Result<Vec<Chunk>> {
    let mut chunks = Vec::new();
    let content = extract_element_content(element, true);

    // Use sentence-based chunking for large elements
    let sentences = utils::split_sentences(&content);
    let mut current_chunk = String::new();
    let mut current_tokens = 0;
    let mut chunk_index = start_index;

    for sentence in sentences {
        let sentence_tokens = utils::count_tokens(&sentence);

        if current_tokens + sentence_tokens > config.max_tokens && !current_chunk.is_empty() {
            let chunk = create_simple_chunk(&current_chunk, chunk_index, current_tokens, "large_element");
            chunks.push(chunk);

            current_chunk.clear();
            current_tokens = 0;
            chunk_index += 1;
        }

        if !current_chunk.is_empty() {
            current_chunk.push(' ');
        }
        current_chunk.push_str(&sentence);
        current_tokens += sentence_tokens;
    }

    if !current_chunk.is_empty() {
        let chunk = create_simple_chunk(&current_chunk, chunk_index, current_tokens, "large_element");
        chunks.push(chunk);
    }

    Ok(chunks)
}

/// Create an HTML chunk with metadata from element
fn create_html_chunk(
    content: &str,
    chunk_index: usize,
    token_count: usize,
    element: &ElementRef<'_>,
    chunk_type: &str,
) -> Chunk {
    let word_count = content.split_whitespace().count();
    let sentence_count = utils::split_sentences(content).len();
    let has_complete_sentences = content.trim().ends_with('.') ||
                                 content.trim().ends_with('!') ||
                                 content.trim().ends_with('?');
    let topic_keywords = utils::extract_topic_keywords(content);

    // Extract HTML-specific metadata
    let mut custom = HashMap::new();
    custom.insert("tag_name".to_string(), element.value().name().to_string());

    if let Some(id) = element.value().attr("id") {
        custom.insert("element_id".to_string(), id.to_string());
    }

    if let Some(class) = element.value().attr("class") {
        custom.insert("element_class".to_string(), class.to_string());
    }

    let metadata = ChunkMetadata {
        quality_score: 0.0, // Will be calculated
        sentence_count,
        word_count,
        has_complete_sentences,
        topic_keywords: topic_keywords.clone(),
        chunk_type: format!("html_{}_{}", chunk_type, element.value().name()),
        custom,
    };

    let quality_score = calculate_html_quality_score(content, &metadata);

    Chunk {
        id: format!("html_{}_{}_{}", chunk_type, element.value().name(), chunk_index),
        content: content.to_string(),
        start_pos: 0, // Would need more complex calculation for HTML
        end_pos: content.len(),
        token_count,
        chunk_index,
        total_chunks: 0, // Will be updated later
        metadata: ChunkMetadata {
            quality_score,
            ..metadata
        },
    }
}

/// Create a simple chunk without HTML element context
fn create_simple_chunk(
    content: &str,
    chunk_index: usize,
    token_count: usize,
    chunk_type: &str,
) -> Chunk {
    let word_count = content.split_whitespace().count();
    let sentence_count = utils::split_sentences(content).len();
    let has_complete_sentences = content.trim().ends_with('.') ||
                                 content.trim().ends_with('!') ||
                                 content.trim().ends_with('?');
    let topic_keywords = utils::extract_topic_keywords(content);

    let metadata = ChunkMetadata {
        quality_score: 0.0, // Will be calculated
        sentence_count,
        word_count,
        has_complete_sentences,
        topic_keywords: topic_keywords.clone(),
        chunk_type: format!("html_{}", chunk_type),
        custom: HashMap::new(),
    };

    let quality_score = utils::calculate_quality_score(content, &metadata);

    Chunk {
        id: format!("html_{}_{}", chunk_type, chunk_index),
        content: content.to_string(),
        start_pos: 0,
        end_pos: content.len(),
        token_count,
        chunk_index,
        total_chunks: 0, // Will be updated later
        metadata: ChunkMetadata {
            quality_score,
            ..metadata
        },
    }
}

/// Calculate quality score for HTML chunks
fn calculate_html_quality_score(content: &str, metadata: &ChunkMetadata) -> f64 {
    let mut score = utils::calculate_quality_score(content, metadata);

    // HTML-specific bonuses
    if let Some(tag_name) = metadata.custom.get("tag_name") {
        match tag_name.as_str() {
            "article" | "section" | "main" => score += 0.2, // High semantic value
            "p" | "div" => score += 0.1, // Standard content containers
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => score += 0.15, // Headers are important
            _ => {}
        }
    }

    // Bonus for having semantic IDs or classes
    if metadata.custom.contains_key("element_id") {
        score += 0.05;
    }

    if let Some(class) = metadata.custom.get("element_class") {
        if class.contains("content") || class.contains("article") || class.contains("post") {
            score += 0.1;
        }
    }

    score.min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_html_structure_chunking() {
        let config = ChunkingConfig::default();
        let chunker = HtmlAwareChunker::new(true, true, config);

        let html = r#"
            <article>
                <h1>Title</h1>
                <p>First paragraph content.</p>
                <p>Second paragraph content.</p>
            </article>
            <section>
                <h2>Section Title</h2>
                <p>Section content.</p>
            </section>
        "#;

        let chunks = chunker.chunk(html).await.unwrap();
        assert!(!chunks.is_empty());
        assert_eq!(chunks[0].metadata.chunk_type, "html_structure_article");
    }

    #[tokio::test]
    async fn test_html_block_chunking() {
        let config = ChunkingConfig::default();
        let chunker = HtmlAwareChunker::new(true, false, config);

        let html = r#"
            <div>
                <p>First paragraph.</p>
                <p>Second paragraph.</p>
            </div>
        "#;

        let chunks = chunker.chunk(html).await.unwrap();
        assert!(!chunks.is_empty());
    }

    #[tokio::test]
    async fn test_text_fallback() {
        let config = ChunkingConfig::default();
        let chunker = HtmlAwareChunker::new(false, false, config);

        let text = "This is plain text content. It has multiple sentences. Should be chunked appropriately.";
        let chunks = chunker.chunk(text).await.unwrap();

        assert!(!chunks.is_empty());
        assert!(chunks[0].metadata.chunk_type.contains("text"));
    }

    #[tokio::test]
    async fn test_safe_split_points() {
        let html = r#"<p>Content</p><div>More content</div><span>Inline</span>"#;
        let safe_points = find_safe_split_points(html);

        // Should find points between tags
        assert!(!safe_points.is_empty());
    }

    #[tokio::test]
    async fn test_html_quality_scoring() {
        let mut metadata = ChunkMetadata {
            quality_score: 0.0,
            sentence_count: 3,
            word_count: 50,
            has_complete_sentences: true,
            topic_keywords: vec!["test".to_string()],
            chunk_type: "html_structure".to_string(),
            custom: HashMap::new(),
        };

        metadata.custom.insert("tag_name".to_string(), "article".to_string());

        let score = calculate_html_quality_score("Test content with good structure.", &metadata);
        assert!(score > 0.7); // Should have high quality score for article element
    }

    #[tokio::test]
    async fn test_large_element_chunking() {
        let html = r#"<div>Very long content that exceeds token limits. This should be split into multiple chunks. Each chunk should preserve sentence boundaries. The content continues for many more sentences to test the chunking behavior.</div>"#;
        let document = Html::parse_document(html);
        let div_selector = scraper::Selector::parse("div").unwrap();
        let element = document.select(&div_selector).next().unwrap();

        let config = ChunkingConfig {
            max_tokens: 20, // Small limit to force chunking
            ..Default::default()
        };

        let chunks = chunk_large_element(&element, &config, 0).unwrap();
        assert!(chunks.len() > 1);
    }
}