//! Extracted content types and quality metrics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Basic extracted document for core orchestration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BasicExtractedDoc {
    pub url: String,
    pub title: Option<String>,
    pub text: String,
    pub quality_score: Option<u8>,
    pub links: Vec<String>,
    pub byline: Option<String>,
    pub published_iso: Option<String>,
    pub markdown: Option<String>,
    pub media: Vec<String>,
    pub language: Option<String>,
    pub reading_time: Option<u32>,
    pub word_count: Option<u32>,
    pub categories: Vec<String>,
    pub site_name: Option<String>,
    pub description: Option<String>,
}

/// Alias for ExtractedDoc to maintain compatibility
pub type ExtractedDoc = BasicExtractedDoc;

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

/// Extraction quality assessment
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
        (self.title_quality
            + self.content_quality
            + self.structure_score
            + self.metadata_completeness)
            / 4.0
    }
}

/// Conversion from BasicExtractedDoc/ExtractedDoc to ExtractedContent
impl From<BasicExtractedDoc> for ExtractedContent {
    fn from(doc: BasicExtractedDoc) -> Self {
        Self {
            title: doc.title.unwrap_or_else(|| "Untitled".to_string()),
            content: doc.text,
            summary: doc.description,
            url: doc.url,
            strategy_used: "wasm_extraction".to_string(),
            extraction_confidence: doc
                .quality_score
                .map(|score| score as f64 / 100.0)
                .unwrap_or(0.8),
        }
    }
}

/// Component health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Overall component health
    pub status: String,

    /// Component version
    pub version: String,

    /// Trek-rs library version
    pub trek_version: String,

    /// Supported extraction modes
    pub capabilities: Vec<String>,

    /// Memory usage in bytes
    pub memory_usage: Option<u64>,

    /// Number of extractions performed
    pub extraction_count: Option<u64>,
}

/// Component information and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInfo {
    /// Component name
    pub name: String,

    /// Component version
    pub version: String,

    /// Component model interface version
    pub component_model_version: String,

    /// Enabled features
    pub features: Vec<String>,

    /// Supported extraction modes
    pub supported_modes: Vec<String>,

    /// Build timestamp
    pub build_timestamp: Option<String>,

    /// Git commit hash if available
    pub git_commit: Option<String>,
}

/// Statistics for extraction operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionStats {
    /// Time taken for extraction in milliseconds
    pub processing_time_ms: u64,

    /// Memory used during extraction in bytes
    pub memory_used: u64,

    /// Number of DOM nodes processed
    pub nodes_processed: Option<u32>,

    /// Number of links found
    pub links_found: u32,

    /// Number of images found
    pub images_found: u32,
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
    pub metadata: HashMap<String, String>,
}
