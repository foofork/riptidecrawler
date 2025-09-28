//! HTML processing trait and core interfaces
//!
//! This module defines the main `HtmlProcessor` trait that provides a unified interface
//! for all HTML processing operations including extraction, chunking, and table processing.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

use crate::{ExtractedContent, ContentChunk, RegexPattern};

/// Main trait for HTML processing operations
#[async_trait]
pub trait HtmlProcessor: Send + Sync {
    /// Extract content from HTML using CSS selectors
    async fn extract_with_css(
        &self,
        html: &str,
        url: &str,
        selectors: &HashMap<String, String>,
    ) -> Result<ExtractedContent, ProcessingError>;

    /// Extract content from HTML using regex patterns
    async fn extract_with_regex(
        &self,
        html: &str,
        url: &str,
        patterns: &[RegexPattern],
    ) -> Result<ExtractedContent, ProcessingError>;

    /// Extract structured data from HTML tables
    async fn extract_tables(
        &self,
        html: &str,
        mode: TableExtractionMode,
    ) -> Result<Vec<TableData>, ProcessingError>;

    /// Chunk content into smaller pieces for processing
    async fn chunk_content(
        &self,
        content: &str,
        mode: ChunkingMode,
    ) -> Result<Vec<ContentChunk>, ProcessingError>;

    /// Get processing confidence score for given HTML
    fn confidence_score(&self, html: &str) -> f64;

    /// Get processor name/identifier
    fn processor_name(&self) -> &'static str;
}

/// Processing result with comprehensive metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingResult {
    /// Extracted content
    pub content: ExtractedContent,
    /// Processing statistics
    pub stats: ProcessingStats,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Processing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingStats {
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Memory used in bytes
    pub memory_used: u64,
    /// Number of DOM nodes processed
    pub nodes_processed: Option<u32>,
    /// Number of CSS selectors applied
    pub selectors_applied: Option<u32>,
    /// Number of regex patterns matched
    pub patterns_matched: Option<u32>,
}

/// Table extraction modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TableExtractionMode {
    /// Extract all tables found in the document
    All,
    /// Extract only tables with headers
    WithHeaders,
    /// Extract tables matching specific CSS selector
    BySelector(String),
    /// Extract tables with minimum row/column requirements
    MinSize { min_rows: usize, min_cols: usize },
}

/// Content chunking modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChunkingMode {
    /// Fixed size chunks (by character count)
    FixedSize { size: usize, overlap: usize },
    /// Sentence-based chunking
    Sentence { max_sentences: usize },
    /// Paragraph-based chunking
    Paragraph { max_paragraphs: usize },
    /// Token-based chunking (word boundaries)
    Token { max_tokens: usize, overlap: usize },
    /// Semantic chunking (topic boundaries)
    Semantic { similarity_threshold: f64 },
}

impl Default for ChunkingMode {
    fn default() -> Self {
        ChunkingMode::FixedSize {
            size: 1000,
            overlap: 100,
        }
    }
}

/// Table data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableData {
    /// Table headers (if present)
    pub headers: Vec<String>,
    /// Table rows
    pub rows: Vec<Vec<String>>,
    /// Table caption (if present)
    pub caption: Option<String>,
    /// Table metadata (class, id, etc.)
    pub metadata: HashMap<String, String>,
}

/// Processing errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ProcessingError {
    #[error("Invalid HTML: {0}")]
    InvalidHtml(String),

    #[error("CSS selector error: {0}")]
    CssError(String),

    #[error("Regex pattern error: {0}")]
    RegexError(String),

    #[error("Table extraction error: {0}")]
    TableError(String),

    #[error("Chunking error: {0}")]
    ChunkingError(String),

    #[error("DOM traversal error: {0}")]
    DomError(String),

    #[error("Memory limit exceeded: {0}")]
    MemoryLimit(String),

    #[error("Processing timeout: {0}")]
    Timeout(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Default HTML processor implementation
#[derive(Debug, Clone)]
pub struct DefaultHtmlProcessor {
    /// Maximum processing time in milliseconds
    pub max_processing_time: u64,
    /// Maximum memory usage in bytes
    pub max_memory_usage: u64,
    /// Enable detailed statistics collection
    pub enable_stats: bool,
}

impl Default for DefaultHtmlProcessor {
    fn default() -> Self {
        Self {
            max_processing_time: 30_000, // 30 seconds
            max_memory_usage: 100 * 1024 * 1024, // 100MB
            enable_stats: true,
        }
    }
}

#[async_trait]
impl HtmlProcessor for DefaultHtmlProcessor {
    async fn extract_with_css(
        &self,
        html: &str,
        url: &str,
        selectors: &HashMap<String, String>,
    ) -> Result<ExtractedContent, ProcessingError> {
        crate::css_extraction::extract(html, url, selectors)
            .await
            .map_err(|e| ProcessingError::CssError(e.to_string()))
    }

    async fn extract_with_regex(
        &self,
        html: &str,
        url: &str,
        patterns: &[RegexPattern],
    ) -> Result<ExtractedContent, ProcessingError> {
        crate::regex_extraction::extract(html, url, patterns)
            .await
            .map_err(|e| ProcessingError::RegexError(e.to_string()))
    }

    async fn extract_tables(
        &self,
        html: &str,
        mode: TableExtractionMode,
    ) -> Result<Vec<TableData>, ProcessingError> {
        crate::dom_utils::extract_tables(html, mode)
            .await
            .map_err(|e| ProcessingError::TableError(e.to_string()))
    }

    async fn chunk_content(
        &self,
        content: &str,
        mode: ChunkingMode,
    ) -> Result<Vec<ContentChunk>, ProcessingError> {
        chunk_content_impl(content, mode)
            .await
            .map_err(|e| ProcessingError::ChunkingError(e.to_string()))
    }

    fn confidence_score(&self, html: &str) -> f64 {
        // Basic confidence scoring based on HTML structure
        let document = scraper::Html::parse_document(html);
        let mut score = 0.0;

        // Check for title
        if let Ok(selector) = scraper::Selector::parse("title") {
            if document.select(&selector).next().is_some() {
                score += 0.2;
            }
        }

        // Check for content elements
        if let Ok(selector) = scraper::Selector::parse("p, div, article, section") {
            let count = document.select(&selector).count();
            score += (count as f64 * 0.1).min(0.5);
        }

        // Check for structured data
        if let Ok(selector) = scraper::Selector::parse("[itemscope], [vocab]") {
            if document.select(&selector).next().is_some() {
                score += 0.1;
            }
        }

        score.min(1.0)
    }

    fn processor_name(&self) -> &'static str {
        "default_html_processor"
    }
}

/// Content chunking implementation
async fn chunk_content_impl(content: &str, mode: ChunkingMode) -> anyhow::Result<Vec<ContentChunk>> {
    let mut chunks = Vec::new();

    match mode {
        ChunkingMode::FixedSize { size, overlap } => {
            let mut start = 0;
            let mut index = 0;

            while start < content.len() {
                let end = (start + size).min(content.len());
                let chunk_content = &content[start..end];

                chunks.push(ContentChunk {
                    content: chunk_content.to_string(),
                    index,
                    start_pos: start,
                    end_pos: end,
                    metadata: HashMap::new(),
                });

                if end >= content.len() {
                    break;
                }

                start = if overlap < size {
                    end - overlap
                } else {
                    end
                };
                index += 1;
            }
        }
        ChunkingMode::Sentence { max_sentences } => {
            let sentences: Vec<&str> = content.split(['.', '!', '?']).collect();
            let mut current_chunk = String::new();
            let mut sentence_count = 0;
            let mut start_pos = 0;
            let mut index = 0;

            for sentence in sentences {
                if sentence.trim().is_empty() {
                    continue;
                }

                if sentence_count >= max_sentences && !current_chunk.is_empty() {
                    let end_pos = start_pos + current_chunk.len();
                    chunks.push(ContentChunk {
                        content: current_chunk.trim().to_string(),
                        index,
                        start_pos,
                        end_pos,
                        metadata: HashMap::new(),
                    });

                    current_chunk.clear();
                    sentence_count = 0;
                    start_pos = end_pos;
                    index += 1;
                }

                if !current_chunk.is_empty() {
                    current_chunk.push(' ');
                }
                current_chunk.push_str(sentence.trim());
                sentence_count += 1;
            }

            if !current_chunk.is_empty() {
                chunks.push(ContentChunk {
                    content: current_chunk.trim().to_string(),
                    index,
                    start_pos,
                    end_pos: content.len(),
                    metadata: HashMap::new(),
                });
            }
        }
        ChunkingMode::Paragraph { max_paragraphs } => {
            let paragraphs: Vec<&str> = content.split("\n\n").collect();
            let mut current_chunk = String::new();
            let mut paragraph_count = 0;
            let mut start_pos = 0;
            let mut index = 0;

            for paragraph in paragraphs {
                if paragraph.trim().is_empty() {
                    continue;
                }

                if paragraph_count >= max_paragraphs && !current_chunk.is_empty() {
                    let end_pos = start_pos + current_chunk.len();
                    chunks.push(ContentChunk {
                        content: current_chunk.trim().to_string(),
                        index,
                        start_pos,
                        end_pos,
                        metadata: HashMap::new(),
                    });

                    current_chunk.clear();
                    paragraph_count = 0;
                    start_pos = end_pos;
                    index += 1;
                }

                if !current_chunk.is_empty() {
                    current_chunk.push_str("\n\n");
                }
                current_chunk.push_str(paragraph.trim());
                paragraph_count += 1;
            }

            if !current_chunk.is_empty() {
                chunks.push(ContentChunk {
                    content: current_chunk.trim().to_string(),
                    index,
                    start_pos,
                    end_pos: content.len(),
                    metadata: HashMap::new(),
                });
            }
        }
        ChunkingMode::Token { max_tokens, overlap } => {
            let tokens: Vec<&str> = content.split_whitespace().collect();
            let mut start = 0;
            let mut index = 0;

            while start < tokens.len() {
                let end = (start + max_tokens).min(tokens.len());
                let chunk_tokens = &tokens[start..end];
                let chunk_content = chunk_tokens.join(" ");

                let start_pos = if start == 0 { 0 } else {
                    content.find(chunk_tokens[0]).unwrap_or(0)
                };
                let end_pos = if end >= tokens.len() {
                    content.len()
                } else {
                    start_pos + chunk_content.len()
                };

                chunks.push(ContentChunk {
                    content: chunk_content,
                    index,
                    start_pos,
                    end_pos,
                    metadata: HashMap::new(),
                });

                if end >= tokens.len() {
                    break;
                }

                start = if overlap < max_tokens {
                    end - overlap
                } else {
                    end
                };
                index += 1;
            }
        }
        ChunkingMode::Semantic { similarity_threshold: _ } => {
            // Simple implementation - split by double newlines (paragraphs)
            // In a real implementation, this would use semantic similarity
            let paragraphs: Vec<&str> = content.split("\n\n").collect();
            let mut start_pos = 0;

            for (index, paragraph) in paragraphs.iter().enumerate() {
                if paragraph.trim().is_empty() {
                    continue;
                }

                let end_pos = start_pos + paragraph.len();
                chunks.push(ContentChunk {
                    content: paragraph.trim().to_string(),
                    index,
                    start_pos,
                    end_pos,
                    metadata: HashMap::new(),
                });

                start_pos = end_pos + 2; // Account for double newline
            }
        }
    }

    Ok(chunks)
}