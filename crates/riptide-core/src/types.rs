use serde::{Deserialize, Serialize};

// Re-export shared types from riptide-types
pub use riptide_types::{
    BasicExtractedDoc, ComponentInfo, ExtractedContent, ExtractedDoc, ExtractionQuality,
    ExtractionStats, HealthStatus,
};

// Kept for backward compatibility - these will be migrated later
// Basic extraction types for core functionality
// Full HTML processing types are in riptide-extraction crate

// Re-export config types from riptide-types
pub use riptide_types::{ExtractionMode, OutputFormat, RenderMode};

// Re-export chunking types from riptide-types
pub use riptide_types::{ChunkingConfig, TopicChunkingConfig};

// CrawlOptions extended with riptide-core specific fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlOptions {
    pub concurrency: usize,
    pub cache_mode: String, // "enabled" | "bypass" | "read_through"
    pub dynamic_wait_for: Option<String>,
    pub scroll_steps: u32,
    pub token_chunk_max: usize,
    pub token_overlap: usize,
    // Phase 3 dynamic content options (riptide-core specific)
    // TODO: Re-add after resolving circular dependency with riptide-headless
    // pub dynamic_config: Option<crate::dynamic::DynamicConfig>,
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

impl Default for CrawlOptions {
    fn default() -> Self {
        Self {
            concurrency: 16,
            cache_mode: "read_through".to_string(),
            dynamic_wait_for: None,
            scroll_steps: 8,
            token_chunk_max: 1200,
            token_overlap: 120,
            // dynamic_config: None,  // Removed due to circular dependency
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
