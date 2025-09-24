use serde::{Deserialize, Serialize};

/// Enhanced extraction result with comprehensive metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedDoc {
    /// Source URL for context and link resolution
    pub url: String,

    /// Extracted page title
    pub title: Option<String>,

    /// Author/byline information
    pub byline: Option<String>,

    /// Publication date in ISO 8601 format
    pub published_iso: Option<String>,

    /// Content formatted as Markdown
    pub markdown: String,

    /// Plain text content with HTML tags removed
    pub text: String,

    /// List of extracted hyperlinks
    pub links: Vec<String>,

    /// List of media URLs (images, videos, audio)
    pub media: Vec<String>,

    /// Detected content language (ISO 639-1 code)
    pub language: Option<String>,

    /// Estimated reading time in minutes
    pub reading_time: Option<u32>,

    /// Content quality score (0-100, higher = better)
    pub quality_score: Option<u8>,

    /// Word count of extracted text
    pub word_count: Option<u32>,

    /// Content categories/tags if detected
    pub categories: Vec<String>,

    /// Site name/publisher if available
    pub site_name: Option<String>,

    /// Meta description from page
    pub description: Option<String>,
}

/// Content extraction modes with specific behaviors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExtractionMode {
    /// Extract article content using readability algorithms
    Article,

    /// Extract full page content including sidebars and navigation
    Full,

    /// Extract only metadata (title, description, structured data)
    Metadata,

    /// Custom extraction using provided CSS selectors
    Custom(Vec<String>),
}

/// Structured error types for better error handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExtractionError {
    /// Invalid or malformed HTML input
    InvalidHtml(String),

    /// Network-related errors during processing
    NetworkError(String),

    /// HTML parsing failures
    ParseError(String),

    /// Resource limits exceeded (memory, time, etc.)
    ResourceLimit(String),

    /// Trek-rs library errors
    ExtractorError(String),

    /// Component internal processing errors
    InternalError(String),

    /// Unsupported extraction mode
    UnsupportedMode(String),
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
    pub pdf_config: Option<crate::pdf::PdfConfig>,
    pub render_mode: RenderMode,
    pub output_format: OutputFormat,
    // Spider deep crawling mode
    pub use_spider: Option<bool>,
    pub spider_max_depth: Option<usize>,
    pub spider_strategy: Option<String>,
}

/// Rendering mode for content processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RenderMode {
    /// Fast path: static HTML processing only
    Static,
    /// Dynamic rendering with JavaScript execution
    Dynamic,
    /// Adaptive: choose based on content analysis
    Adaptive,
    /// PDF processing mode
    Pdf,
}

impl Default for RenderMode {
    fn default() -> Self {
        RenderMode::Adaptive
    }
}

/// Output format for extracted content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    /// Standard structured document
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

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Document
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
        }
    }
}
