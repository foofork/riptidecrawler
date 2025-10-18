//! # Riptide Types
//!
//! Shared type definitions for the Riptide web scraping framework.
//!
//! This crate provides:
//! - Common data structures used across all Riptide crates
//! - Trait definitions for extensibility
//! - Error types for consistent error handling
//! - Type aliases for common patterns

// Public modules
pub mod config;
pub mod errors;
pub mod extracted;
pub mod traits;
pub mod types;

// Re-export commonly used types at the crate root
pub use config::{ChunkingConfig, ExtractionMode, OutputFormat, RenderMode, TopicChunkingConfig};
pub use errors::{Result, RiptideError};
pub use extracted::{
    BasicExtractedDoc, ComponentInfo, ContentChunk, ExtractedContent, ExtractedDoc,
    ExtractionQuality, ExtractionStats, HealthStatus,
};
pub use traits::{Browser, Extractor, Scraper};
pub use types::{
    BrowserConfig, ExtractionConfig, ExtractionRequest, ExtractionResult, ScrapedContent,
    ScrapingOptions, Url,
};

// Re-export third-party types for convenience
pub use chrono::{DateTime, Utc};
pub use serde::{Deserialize, Serialize};
pub use uuid::Uuid;
