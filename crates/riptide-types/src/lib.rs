//! # RipTide Types
//!
//! Shared types and traits for the RipTide extraction system.
//! This crate provides common type definitions used across multiple RipTide crates,
//! particularly to break circular dependencies between riptide-core and riptide-extraction.
//!
//! ## Organization
//!
//! - `traits`: Core trait definitions for strategies (extraction, spider, chunking)
//! - `extracted`: Extracted content types and quality metrics
//! - `config`: Configuration types for various extraction modes
//! - `errors`: Error types and result aliases

pub mod config;
pub mod errors;
pub mod extracted;
pub mod traits;

// Re-export commonly used types
pub use config::{
    ChunkingConfig, CrawlOptions, ExtractionMode, OutputFormat, RenderMode, TopicChunkingConfig,
};
pub use errors::{CoreError, CoreResult};
pub use extracted::{
    BasicExtractedDoc, ComponentInfo, ExtractedContent, ExtractedDoc, ExtractionQuality,
    ExtractionStats, HealthStatus,
};
pub use traits::{
    CrawlRequest, CrawlResult, ExtractionResult, ExtractionStrategy, PerformanceTier, Priority,
    ResourceRequirements, ResourceTier, SpiderStrategy, StrategyCapabilities,
};
