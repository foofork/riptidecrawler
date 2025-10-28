//! Configuration types for extraction and crawling

use serde::{Deserialize, Serialize};

/// Content extraction modes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExtractionMode {
    Article,
    Full,
    Metadata,
    Custom(Vec<String>),
}

/// Rendering mode for content processing
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum RenderMode {
    /// Fast path: static HTML processing only
    Static,
    /// Dynamic rendering with JavaScript execution
    Dynamic,
    /// Adaptive: choose based on content analysis
    #[default]
    Adaptive,
    /// PDF processing mode
    Pdf,
    /// HTML output mode
    Html,
    /// Markdown output mode
    Markdown,
}

/// Output format for extracted content
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum OutputFormat {
    /// Standard structured document
    #[default]
    Document,
    /// NDJSON streaming format
    NdJson,
    /// Chunked content with tokens
    Chunked,
    /// Raw text only
    Text,
    /// Markdown format
    Markdown,
}

/// Content chunking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkingConfig {
    /// Chunking mode: "topic", "sliding", "fixed", "sentence", "html-aware"
    pub chunking_mode: String,
    /// Maximum chunk size in tokens
    pub chunk_size: usize,
    /// Overlap size in tokens for sliding window
    pub overlap_size: usize,
    /// Minimum chunk size in characters
    pub min_chunk_size: usize,
    /// Preserve sentence boundaries
    #[serde(default = "default_true")]
    pub preserve_sentences: bool,
    /// Topic chunking specific config
    pub topic_config: Option<TopicChunkingConfig>,
}

/// Topic chunking specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicChunkingConfig {
    /// Window size for coherence analysis
    pub window_size: usize,
    /// Number of smoothing passes
    pub smoothing_passes: usize,
    /// Enable topic chunking (opt-in)
    pub topic_chunking: bool,
}

fn default_true() -> bool {
    true
}

impl Default for ChunkingConfig {
    fn default() -> Self {
        Self {
            chunking_mode: "sliding".to_string(),
            chunk_size: 1000,
            overlap_size: 100,
            min_chunk_size: 200,
            preserve_sentences: true,
            topic_config: None,
        }
    }
}

impl Default for TopicChunkingConfig {
    fn default() -> Self {
        Self {
            window_size: 100,
            smoothing_passes: 2,
            topic_chunking: true,
        }
    }
}

/// Crawl options configuration
/// Note: This struct cannot include DynamicConfig, StealthConfig, or PdfConfig
/// as those are defined in their respective crates. Those fields should be added
/// in the crate that uses this type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CrawlOptions {
    pub concurrency: usize,
    pub cache_mode: String, // "enabled" | "bypass" | "read_through"
    pub dynamic_wait_for: Option<String>,
    pub scroll_steps: u32,
    pub token_chunk_max: usize,
    pub token_overlap: usize,
    pub render_mode: RenderMode,
    pub output_format: OutputFormat,
    // Spider deep crawling mode
    pub use_spider: Option<bool>,
    pub spider_max_depth: Option<usize>,
    pub spider_strategy: Option<String>,
    // Content chunking configuration
    pub chunking_config: Option<ChunkingConfig>,
    /// Skip content extraction and return raw HTML only
    /// When true, only HTML field is populated, text/links/metadata are empty
    pub skip_extraction: Option<bool>,
}

impl Default for CrawlOptions {
    fn default() -> Self {
        Self {
            concurrency: 16,
            cache_mode: "read_through".to_string(),
            dynamic_wait_for: None,
            scroll_steps: 8,
            token_chunk_max: 1200,
            token_overlap: 120,
            render_mode: RenderMode::default(),
            output_format: OutputFormat::default(),
            use_spider: None,
            spider_max_depth: None,
            spider_strategy: None,
            chunking_config: None,
            skip_extraction: None,
        }
    }
}
