//! # RipTide HTML Processing Crate
//!
//! This crate provides HTML processing and content extraction capabilities for the RipTide project.
//! It includes CSS selector-based extraction, regex pattern extraction, DOM traversal utilities,
//! and table extraction interfaces.
//!
//! ## Features
//!
//! - **CSS Extraction**: Extract content using CSS selectors with JSON mapping
//! - **Regex Extraction**: Pattern-based content extraction with configurable rules
//! - **DOM Utils**: Utilities for DOM traversal and manipulation
//! - **Table Extraction**: Interface for extracting structured data from HTML tables
//! - **Chunking**: Content chunking interface for processing large documents
//!
//! ## Usage
//!
//! ```rust
//! use riptide_html::{HtmlProcessor, css_extraction, regex_extraction};
//! use std::collections::HashMap;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let html = r#"<html><head><title>Test</title></head><body><p>Content</p></body></html>"#;
//!
//! // CSS extraction with default selectors
//! let result = css_extraction::extract_default(html, "https://example.com").await?;
//! println!("Title: {}", result.title);
//!
//! // Regex extraction with custom patterns
//! let patterns = regex_extraction::default_patterns();
//! let result = regex_extraction::extract(html, "https://example.com", &patterns).await?;
//! println!("Content: {}", result.content);
//! # Ok(())
//! # }
//! ```

pub mod processor;
pub mod css_extraction;
pub mod regex_extraction;
pub mod dom_utils;

// Re-export main interfaces
pub use processor::{HtmlProcessor, ProcessingResult, ProcessingError, ChunkingMode, TableExtractionMode};
pub use css_extraction::{CssJsonExtractor, extract as css_extract, extract_default as css_extract_default, default_selectors};
pub use regex_extraction::{RegexExtractor, extract as regex_extract, default_patterns};
pub use dom_utils::{DomTraverser, ElementInfo, traverse_elements, extract_text_content, find_tables};

// Common types for extraction
use serde::{Deserialize, Serialize};

/// Common result type for all extraction operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedContent {
    /// Extracted title
    pub title: String,
    /// Main content text
    pub content: String,
    /// Optional summary/description
    pub summary: Option<String>,
    /// Source URL
    pub url: String,
    /// Strategy used for extraction
    pub strategy_used: String,
    /// Confidence score (0.0 - 1.0)
    pub extraction_confidence: f64,
}

/// Regex pattern configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegexPattern {
    pub name: String,
    pub pattern: String,
    pub field: String,
    pub required: bool,
}

/// Extraction quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionQuality {
    pub content_length: usize,
    pub title_quality: f64,
    pub content_quality: f64,
    pub structure_score: f64,
    pub metadata_completeness: f64,
}

impl ExtractionQuality {
    pub fn overall_score(&self) -> f64 {
        (self.title_quality + self.content_quality + self.structure_score + self.metadata_completeness) / 4.0
    }
}

/// Content chunk for processed documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentChunk {
    /// Chunk content
    pub content: String,
    /// Chunk index
    pub index: usize,
    /// Start position in original text
    pub start_pos: usize,
    /// End position in original text
    pub end_pos: usize,
    /// Chunk metadata
    pub metadata: std::collections::HashMap<String, String>,
}