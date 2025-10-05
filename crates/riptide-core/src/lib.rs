//! # Riptide Core
//!
//! Core infrastructure for the Riptide web scraping framework.
//! This crate provides essential components for pipeline orchestration,
//! resource management, and system reliability.
//!
//! ## Core Components
//!
//! - **Pipeline Orchestration**: Component-based processing pipeline
//! - **Cache Infrastructure**: Multi-level caching with warming strategies
//! - **Circuit Breakers**: Fault tolerance and resilience patterns
//! - **Instance Pooling**: Resource pooling and lifecycle management
//! - **Memory Management**: Advanced memory allocation and cleanup
//! - **Event Bus**: Pub/sub messaging system
//! - **Telemetry**: Metrics collection and monitoring
//! - **Security**: Authentication, rate limiting, and safety
//! - **Provider Traits**: Abstraction layer for external services
//!
//! ## Feature Separation
//!
//! Specific implementations have been moved to dedicated crates:
//! - **CSS/Regex extraction**: `riptide-html`
//! - **LLM extraction**: `riptide-intelligence`
//! - **Content chunking**: `riptide-html`
//! - **Search providers**: `riptide-search`

// Core infrastructure modules
pub mod ai_processor;
pub mod cache;
pub mod cache_warming;
pub mod cache_warming_integration;
pub mod circuit;
pub mod circuit_breaker;
pub mod common;
pub mod component;
pub mod conditional;
pub mod dynamic;
pub mod error;
pub mod events;
pub mod fetch;
pub mod gate;
pub mod instance_pool;
pub mod integrated_cache;
pub mod memory_manager;
pub mod monitoring;

// PDF functionality moved to riptide-pdf crate
#[cfg(feature = "pdf")]
pub use riptide_pdf as pdf;

pub mod pool_health;
pub mod reliability;
pub mod robots;
pub mod security;
pub mod spider;
pub mod strategies;
pub mod telemetry;
pub mod types;

#[cfg(test)]
mod fetch_engine_tests;

// Re-export stealth functionality from riptide-stealth crate for backward compatibility
pub mod stealth {
    //! Stealth module for anti-detection measures - MOVED
    //!
    //! This module re-exports types from the `riptide-stealth` crate for backward compatibility.
    //!
    //! **NOTICE**: The stealth functionality has been extracted to its own crate.
    //! Please migrate to using `riptide-stealth` crate directly:
    //!
    //! ```rust
    //! // Old (still works):
    //! use riptide_core::stealth::{StealthController, StealthPreset};
    //!
    //! // New (recommended):
    //! use riptide_stealth::{StealthController, StealthPreset};
    //! ```

    pub use riptide_stealth::*;
}

// Search functionality has been fully moved to riptide-search crate
// Use `riptide-search` directly instead of these re-exports

#[cfg(feature = "benchmarks")]
pub mod benchmarks;

pub use common::{
    config_builder::{ConfigBuilder, ConfigValue, DefaultConfigBuilder, ValidationPatterns},
    error_conversions::{CoreErrorConverter, ErrorPatterns, IntoCore, WithErrorContext},
    validation::{
        CommonValidator, ContentTypeValidator, ParameterValidator, SizeValidator, UrlValidator,
        ValidationConfig,
    },
};
pub use types::*;

// Re-export core functionality
pub use reliability::{ExtractionMode, ReliabilityConfig, ReliabilityMetrics, ReliableExtractor};

// Create extract module alias for backward compatibility
pub mod extract {
    pub use crate::reliability::WasmExtractor;
    // Export the concrete implementation that can be instantiated
    pub use crate::component::CmExtractor;
}

// Add PDF conversion function for backward compatibility
#[cfg(feature = "pdf")]
pub fn convert_pdf_extracted_doc(doc: riptide_pdf::types::ExtractedDoc) -> ExtractedDoc {
    // Convert from riptide-pdf's ExtractedDoc to riptide-core's ExtractedDoc
    ExtractedDoc {
        url: doc.url,
        title: doc.title,
        text: doc.text,
        quality_score: doc.quality_score,
        links: doc.links,
        byline: doc.byline,
        published_iso: doc.published_iso,
        markdown: Some(doc.markdown),
        media: doc.media,
        language: doc.language,
        reading_time: doc.reading_time,
        word_count: doc.word_count,
        categories: doc.categories,
        site_name: doc.site_name,
        description: doc.description,
    }
}

// Fallback for when PDF feature is not enabled
#[cfg(not(feature = "pdf"))]
pub fn convert_pdf_extracted_doc(doc: ExtractedDoc) -> ExtractedDoc {
    // Simple pass-through when PDF crate is not available
    doc
}
