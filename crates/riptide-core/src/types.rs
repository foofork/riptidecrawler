use serde::{Deserialize, Serialize};

// Basic extraction types for core functionality
// Full HTML processing types are in riptide-html crate

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

/// Content extraction modes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExtractionMode {
    Article,
    Full,
    Metadata,
    Custom(Vec<String>),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlOptions {
    pub concurrency: usize,
    pub cache_mode: String, // "enabled" | "bypass" | "read_through"
    pub dynamic_wait_for: Option<String>,
    pub scroll_steps: u32,
    pub token_chunk_max: usize,
    pub token_overlap: usize,
    // Phase 3 dynamic content options
    pub dynamic_config: Option<crate::dynamic::DynamicConfig>,
    pub stealth_config: Option<crate::stealth::StealthConfig>,
    #[cfg(feature = "pdf")]
    pub pdf_config: Option<riptide_pdf::PdfConfig>,
    #[cfg(not(feature = "pdf"))]
    pub pdf_config: Option<()>,
    pub render_mode: RenderMode,
    pub output_format: OutputFormat,
    // Spider deep crawling mode
    pub use_spider: Option<bool>,
    pub spider_max_depth: Option<usize>,
    pub spider_strategy: Option<String>,
    // Content chunking configuration
    pub chunking_config: Option<ChunkingConfig>,
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

fn default_true() -> bool { true }

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

impl Default for CrawlOptions {
    fn default() -> Self {
        Self {
            concurrency: 16,
            cache_mode: "read_through".to_string(),
            dynamic_wait_for: None,
            scroll_steps: 8,
            token_chunk_max: 1200,
            token_overlap: 120,
            dynamic_config: None,
            stealth_config: None,
            pdf_config: None,
            render_mode: RenderMode::default(),
            output_format: OutputFormat::default(),
            use_spider: None,
            spider_max_depth: None,
            spider_strategy: None,
            chunking_config: None,
        }
    }
}
