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
pub mod cache;
pub mod cache_warming;
pub mod cache_warming_integration;
pub mod circuit;
pub mod common;
pub mod component;
pub mod conditional;
pub mod dynamic;
pub mod error;
pub mod events;
pub mod extract;
pub mod fetch;
pub mod gate;
pub mod instance_pool;
pub mod integrated_cache;
pub mod memory_manager;
pub mod monitoring;
// PDF functionality moved to riptide-pdf crate
#[cfg(feature = "pdf")]
pub use riptide_pdf as pdf;

#[cfg(feature = "pdf")]
mod pdf_integration {
    use super::types::ExtractedDoc;

    /// Convert riptide_pdf::ExtractedDoc to riptide_core::ExtractedDoc
    pub fn convert_pdf_extracted_doc(pdf_doc: riptide_pdf::ExtractedDoc) -> ExtractedDoc {
        ExtractedDoc {
            url: pdf_doc.url,
            title: pdf_doc.title,
            byline: pdf_doc.byline,
            published_iso: pdf_doc.published_iso,
            markdown: pdf_doc.markdown,
            text: pdf_doc.text,
            links: pdf_doc.links,
            media: pdf_doc.media,
            language: pdf_doc.language,
            reading_time: pdf_doc.reading_time,
            quality_score: pdf_doc.quality_score,
            word_count: pdf_doc.word_count,
            categories: pdf_doc.categories,
            site_name: pdf_doc.site_name,
            description: pdf_doc.description,
        }
    }
}

#[cfg(feature = "pdf")]
pub use pdf_integration::convert_pdf_extracted_doc;
pub mod pool_health;
pub mod reliability;
pub mod robots;
pub mod security;
pub mod spider;
pub mod strategies;
pub mod telemetry;
pub mod types;

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

// Re-export search functionality from riptide-search crate for backward compatibility
// TODO: Deprecate in Week 8 - use riptide-search directly
pub mod search {
    //! Search provider abstraction - DEPRECATED
    //!
    //! This module re-exports types from the `riptide-search` crate for backward compatibility.
    //!
    //! **DEPRECATION NOTICE**: This re-export will be removed in a future version.
    //! Please migrate to using `riptide-search` crate directly:
    //!
    //! ```rust
    //! // Old (deprecated):
    //! use riptide_core::search::{SearchProvider, SearchBackend};
    //!
    //! // New (recommended):
    //! use riptide_search::{SearchProvider, SearchBackend};
    //! ```

    pub use riptide_search::{
        SearchProvider, SearchHit, SearchBackend, SearchConfig, AdvancedSearchConfig,
        CircuitBreakerConfigOptions, SearchProviderFactory,
        create_search_provider, create_search_provider_from_env,
        SerperProvider, NoneProvider, CircuitBreakerWrapper, CircuitBreakerConfig, CircuitState
    };

    // Re-export the mod structure for backward compatibility
    pub mod providers {
        pub use riptide_search::SerperProvider;
    }

    pub mod circuit_breaker {
        pub use riptide_search::{CircuitBreakerWrapper, CircuitBreakerConfig, CircuitState};
    }

    pub mod none_provider {
        pub use riptide_search::NoneProvider;
    }
}

#[cfg(feature = "benchmarks")]
pub mod benchmarks;

pub use types::*;
pub use common::{
    validation::{CommonValidator, ValidationConfig, ContentTypeValidator, UrlValidator, SizeValidator, ParameterValidator},
    error_conversions::{IntoCore, WithErrorContext, CoreErrorConverter, ErrorPatterns},
    config_builder::{ConfigBuilder, DefaultConfigBuilder, ConfigValue, ValidationPatterns},
};
