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
//! - **CSS/Regex extraction**: `riptide-extraction`
//! - **LLM extraction**: `riptide-intelligence`
//! - **Content chunking**: `riptide-extraction`
//! - **Search providers**: `riptide-search`

// Core infrastructure modules
// pub mod ai_processor;  // P1-A3 Phase 2D: Moved to riptide-intelligence crate
// pub mod cache;  // P1-A3 Phase 2C: Moved to riptide-cache crate
// pub mod cache_key;  // P1-A3 Phase 2C: Moved to riptide-cache crate
// pub mod cache_warming;  // P1-A3 Phase 2C: Moved to riptide-cache crate
// pub mod cache_warming_integration;  // P1-A3 Phase 2C: Moved to riptide-cache crate
// P2-F1 Day 3: Circuit breaker patterns moved to riptide-reliability
pub mod circuit {
    //! Circuit breaker module - MOVED
    //!
    //! This module re-exports types from the `riptide-reliability` crate for backward compatibility.
    //!
    //! **NOTICE**: The circuit breaker functionality has been extracted to its own crate.
    //! Please migrate to using `riptide-reliability` crate directly:
    //!
    //! ```rust
    //! // Old (still works):
    //! use riptide_core::circuit::*;
    //!
    //! // New (recommended):
    //! use riptide_reliability::circuit::*;
    //! ```
    pub use riptide_reliability::circuit::*;
}

pub mod circuit_breaker {
    //! Circuit breaker state module - MOVED
    //!
    //! This module re-exports types from the `riptide-reliability` crate for backward compatibility.
    //!
    //! **NOTICE**: The circuit breaker state functionality has been extracted to its own crate.
    //! Please migrate to using `riptide-reliability` crate directly:
    //!
    //! ```rust
    //! // Old (still works):
    //! use riptide_core::circuit_breaker::*;
    //!
    //! // New (recommended):
    //! use riptide_reliability::circuit_breaker::*;
    //! ```
    pub use riptide_reliability::circuit_breaker::*;
}
pub mod common;
// P2-F1 Day 3: Component types moved to riptide-types
pub mod component {
    //! Component module - MOVED
    //!
    //! This module re-exports types from the `riptide-types` crate for backward compatibility.
    //!
    //! **NOTICE**: The component types have been extracted to riptide-types.
    //! Please migrate to using `riptide-types` crate directly:
    //!
    //! ```rust
    //! // Old (still works):
    //! use riptide_core::component::*;
    //!
    //! // New (recommended):
    //! use riptide_types::component::*;
    //! ```
    pub use riptide_types::component::*;
}

// P2-F1 Day 3: Conditional request types moved to riptide-types
pub mod conditional {
    //! Conditional module - MOVED
    //!
    //! This module re-exports types from the `riptide-types` crate for backward compatibility.
    //!
    //! **NOTICE**: The conditional request types have been extracted to riptide-types.
    //! Please migrate to using `riptide-types` crate directly:
    //!
    //! ```rust
    //! // Old (still works):
    //! use riptide_core::conditional::*;
    //!
    //! // New (recommended):
    //! use riptide_types::conditional::*;
    //! ```
    pub use riptide_types::conditional::*;
}
// pub mod confidence;  // P1-A3 Phase 2D: Moved to riptide-extraction crate
// pub mod confidence_integration;  // P1-A3 Phase 2D: Moved to riptide-extraction crate
// pub mod dynamic;  // P1-A3 Phase 2D: Moved to riptide-headless crate
// pub mod enhanced_extractor; // Temporarily disabled
pub mod error;
// pub mod events;  // P1-A3 Phase 2A: Moved to riptide-events crate
// pub mod events_pool_integration;  // P1-A3 Phase 2D: Moved to riptide-pool crate
// pub mod fetch;  // P1-C2: Moved to riptide-fetch crate
// P2-F1 Day 3: Gate decision logic moved to riptide-reliability
pub mod gate {
    //! Gate module - MOVED
    //!
    //! This module re-exports types from the `riptide-reliability` crate for backward compatibility.
    //!
    //! **NOTICE**: The gate decision functionality has been extracted to its own crate.
    //! Please migrate to using `riptide-reliability` crate directly:
    //!
    //! ```rust
    //! // Old (still works):
    //! use riptide_core::gate::*;
    //!
    //! // New (recommended):
    //! use riptide_reliability::gate::*;
    //! ```
    pub use riptide_reliability::gate::*;
}
// pub mod html_parser;  // P1-C2: Moved to riptide-extraction crate
// pub mod instance_pool;  // P1-A3 Phase 2B: Moved to riptide-pool crate
// pub mod integrated_cache;  // P1-A3 Phase 2C: Moved to riptide-cache crate
// pub mod memory_manager;  // P1-A3 Phase 2D: Moved to riptide-pool crate
// pub mod monitoring;  // P1-A3: Moved to riptide-monitoring crate
// P2-F1 Day 3: wasm_validation moved to riptide-extraction
pub mod wasm_validation {
    //! WASM validation module - MOVED
    //!
    //! This module re-exports types from the `riptide-extraction` crate for backward compatibility.
    //!
    //! **NOTICE**: The WASM validation functionality has been extracted to its own location.
    //! Please migrate to using `riptide-extraction` crate directly:
    //!
    //! ```rust
    //! // Old (still works):
    //! use riptide_core::wasm_validation::*;
    //!
    //! // New (recommended):
    //! use riptide_extraction::validation::*;
    //! ```
    pub use riptide_extraction::validation::*;
}

// PDF functionality moved to riptide-pdf crate
#[cfg(feature = "pdf")]
pub use riptide_pdf as pdf;

// pub mod pool_health;  // P1-A3 Phase 2B: Moved to riptide-pool crate
// P2-F1 Day 3: Reliability orchestration moved to riptide-reliability
pub mod reliability {
    //! Reliability module - MOVED
    //!
    //! This module re-exports types from the `riptide-reliability` crate for backward compatibility.
    //!
    //! **NOTICE**: The reliability orchestration functionality has been extracted to its own crate.
    //! Please migrate to using `riptide-reliability` crate directly:
    //!
    //! ```rust
    //! // Old (still works):
    //! use riptide_core::reliability::*;
    //!
    //! // New (recommended):
    //! use riptide_reliability::reliability::*;
    //! ```
    pub use riptide_reliability::reliability::*;
}
// pub mod robots;  // P1-A3 Phase 2D: Already moved to riptide-fetch crate (duplicate)
// pub mod security;  // P1-A3: Moved to riptide-security crate
// pub mod spider;  // P1-C2: Moved to riptide-spider crate
// pub mod strategies;  // P1-C2: Moved to riptide-extraction crate
// pub mod strategy_composition;  // P1-A3 Phase 2D: Moved to riptide-extraction crate
// pub mod telemetry;  // P1-A3: Moved to riptide-monitoring crate
pub mod types;

// P1-C2: Re-export extracted modules for backward compatibility
pub mod fetch {
    //! Fetch module - MOVED
    //!
    //! This module re-exports types from the `riptide-fetch` crate for backward compatibility.
    //!
    //! **NOTICE**: The fetch functionality has been extracted to its own crate.
    //! Please migrate to using `riptide-fetch` crate directly:
    //!
    //! ```rust
    //! // Old (still works):
    //! use riptide_core::fetch::*;
    //!
    //! // New (recommended):
    //! use riptide_fetch::*;
    //! ```
    pub use riptide_fetch::*;
}

pub mod spider {
    //! Spider module - MOVED
    //!
    //! This module re-exports types from the `riptide-spider` crate for backward compatibility.
    //!
    //! **NOTICE**: The spider functionality has been extracted to its own crate.
    //! Please migrate to using `riptide-spider` crate directly:
    //!
    //! ```rust
    //! // Old (still works):
    //! use riptide_core::spider::*;
    //!
    //! // New (recommended):
    //! use riptide_spider::*;
    //! ```
    pub use riptide_spider::*;
}

pub mod html_parser {
    //! HTML parser module - MOVED
    //!
    //! This module re-exports types from the `riptide-extraction` crate for backward compatibility.
    //!
    //! **NOTICE**: The HTML parser functionality has been moved to riptide-extraction.
    //! Please migrate to using `riptide-extraction` crate directly:
    //!
    //! ```rust
    //! // Old (still works):
    //! use riptide_core::html_parser::*;
    //!
    //! // New (recommended):
    //! use riptide_extraction::html_parser::*;
    //! ```
    pub use riptide_extraction::html_parser::*;
}

pub mod strategies {
    //! Strategies module - MOVED
    //!
    //! This module re-exports types from the `riptide-extraction` crate for backward compatibility.
    //!
    //! **NOTICE**: The strategies functionality has been moved to riptide-extraction.
    //! Please migrate to using `riptide-extraction` crate directly:
    //!
    //! ```rust
    //! // Old (still works):
    //! use riptide_core::strategies::*;
    //!
    //! // New (recommended):
    //! use riptide_extraction::strategies::*;
    //! ```
    pub use riptide_extraction::strategies::*;
}

pub mod cache {
    //! Cache module - MOVED
    //!
    //! This module re-exports types from the `riptide-cache` crate for backward compatibility.
    //!
    //! **NOTICE**: The cache functionality has been extracted to its own crate.
    //! Please migrate to using `riptide-cache` crate directly:
    //!
    //! ```rust
    //! // Old (still works):
    //! use riptide_core::cache::*;
    //!
    //! // New (recommended):
    //! use riptide_cache::redis::*;
    //! ```
    pub use riptide_cache::redis::*;
}

pub mod cache_key {
    //! Cache key module - MOVED
    //!
    //! This module re-exports types from the `riptide-cache` crate for backward compatibility.
    //!
    //! **NOTICE**: The cache key functionality has been extracted to its own crate.
    //! Please migrate to using `riptide-cache` crate directly:
    //!
    //! ```rust
    //! // Old (still works):
    //! use riptide_core::cache_key::*;
    //!
    //! // New (recommended):
    //! use riptide_cache::key::*;
    //! ```
    pub use riptide_cache::key::*;
}

pub mod cache_warming {
    //! Cache warming module - MOVED
    //!
    //! This module re-exports types from the `riptide-cache` crate for backward compatibility.
    //!
    //! **NOTICE**: The cache warming functionality has been extracted to its own crate.
    //! Please migrate to using `riptide-cache` crate directly:
    //!
    //! ```rust
    //! // Old (still works):
    //! use riptide_core::cache_warming::*;
    //!
    //! // New (recommended):
    //! use riptide_cache::warming::*;
    //! ```
    pub use riptide_cache::warming::*;
}

pub mod cache_warming_integration {
    //! Cache warming integration module - MOVED
    //!
    //! This module re-exports types from the `riptide-cache` crate for backward compatibility.
    //!
    //! **NOTICE**: The cache warming integration functionality has been extracted to its own crate.
    //! Please migrate to using `riptide-cache` crate directly:
    //!
    //! ```rust
    //! // Old (still works):
    //! use riptide_core::cache_warming_integration::*;
    //!
    //! // New (recommended):
    //! use riptide_cache::warming_integration::*;
    //! ```
    pub use riptide_cache::warming_integration::*;
}

pub mod integrated_cache {
    //! Integrated cache module - MOVED (Temporarily disabled)
    //!
    //! **NOTICE**: This module has been moved to riptide-cache but is currently disabled
    //! due to circular dependencies. It will be re-enabled after refactoring.
    //!
    //! For now, use the individual cache modules directly from riptide-cache.
}

pub mod memory_manager {
    //! Memory manager module - MOVED
    //!
    //! This module re-exports types from the `riptide-pool` crate for backward compatibility.
    //!
    //! **NOTICE**: The memory management functionality has been extracted to its own crate.
    //! Please migrate to using `riptide-pool` crate directly:
    //!
    //! ```rust
    //! // Old (still works):
    //! use riptide_core::memory_manager::*;
    //!
    //! // New (recommended):
    //! use riptide_pool::memory_manager::*;
    //! ```
    pub use riptide_pool::memory_manager::*;
}

pub mod events_pool_integration {
    //! Events pool integration module - MOVED
    //!
    //! This module re-exports types from the `riptide-pool` crate for backward compatibility.
    //!
    //! **NOTICE**: The events integration functionality has been extracted to its own crate.
    //! Please migrate to using `riptide-pool` crate directly:
    //!
    //! ```rust
    //! // Old (still works):
    //! use riptide_core::events_pool_integration::*;
    //!
    //! // New (recommended):
    //! use riptide_pool::events_integration::*;
    //! ```
    pub use riptide_pool::events_integration::*;
}

pub mod confidence {
    //! Confidence scoring module - MOVED
    //!
    //! This module re-exports types from the `riptide-extraction` crate for backward compatibility.
    //!
    //! **NOTICE**: The confidence scoring functionality has been extracted to its own crate.
    //! Please migrate to using `riptide-extraction` crate directly:
    //!
    //! ```rust
    //! // Old (still works):
    //! use riptide_core::confidence::*;
    //!
    //! // New (recommended):
    //! use riptide_extraction::confidence::*;
    //! ```
    pub use riptide_extraction::confidence::*;
}

// TODO: Re-enable after fixing type conflicts in riptide-extraction
// pub mod confidence_integration {
//     pub use riptide_extraction::confidence_integration::*;
// }

// TODO: Re-enable after fixing type conflicts in riptide-extraction
// pub mod strategy_composition {
//     pub use riptide_extraction::composition::*;
// }

// TODO: Re-enable after resolving circular dependencies
// pub mod ai_processor {
//     //! AI background processor module - MOVED to riptide-intelligence
//     //! Circular dependency prevents re-export. Use riptide-intelligence directly.
//     pub use riptide_intelligence::background_processor::*;
// }

// TODO: Re-enable after resolving circular dependencies
// pub mod dynamic {
//     //! Dynamic content handling module - MOVED to riptide-headless
//     //! Circular dependency prevents re-export. Use riptide-headless directly.
//     pub use riptide_headless::dynamic::*;
// }

pub mod robots {
    //! Robots.txt handling module - MOVED
    //!
    //! This module re-exports types from the `riptide-fetch` crate for backward compatibility.
    //!
    //! **NOTICE**: The robots.txt functionality has been extracted to its own crate.
    //! Please migrate to using `riptide-fetch` crate directly:
    //!
    //! ```rust
    //! // Old (still works):
    //! use riptide_core::robots::*;
    //!
    //! // New (recommended):
    //! use riptide_fetch::robots::*;
    //! ```
    pub use riptide_fetch::robots::*;
}

pub mod security {
    //! Security module - MOVED
    //!
    //! This module re-exports types from the `riptide-security` crate for backward compatibility.
    //!
    //! **NOTICE**: The security functionality has been extracted to its own crate.
    //! Please migrate to using `riptide-security` crate directly:
    //!
    //! ```rust
    //! // Old (still works):
    //! use riptide_core::security::*;
    //!
    //! // New (recommended):
    //! use riptide_security::*;
    //! ```
    pub use riptide_security::*;
}

pub mod monitoring {
    //! Monitoring module - MOVED
    //!
    //! This module re-exports types from the `riptide-monitoring` crate for backward compatibility.
    //!
    //! **NOTICE**: The monitoring functionality has been extracted to its own crate.
    //! Please migrate to using `riptide-monitoring` crate directly:
    //!
    //! ```rust
    //! // Old (still works):
    //! use riptide_core::monitoring::*;
    //!
    //! // New (recommended):
    //! use riptide_monitoring::monitoring::*;
    //! ```
    pub use riptide_monitoring::monitoring::*;
}

pub mod telemetry {
    //! Telemetry module - MOVED
    //!
    //! This module re-exports types from the `riptide-monitoring` crate for backward compatibility.
    //!
    //! **NOTICE**: The telemetry functionality has been extracted to its own crate.
    //! Please migrate to using `riptide-monitoring` crate directly:
    //!
    //! ```rust
    //! // Old (still works):
    //! use riptide_core::telemetry::*;
    //!
    //! // New (recommended):
    //! use riptide_monitoring::telemetry::*;
    //! ```
    pub use riptide_monitoring::telemetry::*;
}

pub mod events {
    //! Events module - MOVED
    //!
    //! This module re-exports types from the `riptide-events` crate for backward compatibility.
    //!
    //! **NOTICE**: The event system functionality has been extracted to its own crate.
    //! Please migrate to using `riptide-events` crate directly:
    //!
    //! ```rust
    //! // Old (still works):
    //! use riptide_core::events::*;
    //!
    //! // New (recommended):
    //! use riptide_events::*;
    //! ```
    pub use riptide_events::*;
}

pub mod instance_pool {
    //! Instance pool module - MOVED
    //!
    //! This module re-exports types from the `riptide-pool` crate for backward compatibility.
    //!
    //! **NOTICE**: The instance pool functionality has been extracted to its own crate.
    //! Please migrate to using `riptide-pool` crate directly:
    //!
    //! ```rust
    //! // Old (still works):
    //! use riptide_core::instance_pool::*;
    //!
    //! // New (recommended):
    //! use riptide_pool::*;
    //! ```
    pub use riptide_pool::*;
}

pub mod pool_health {
    //! Pool health module - MOVED
    //!
    //! This module re-exports types from the `riptide-pool` crate for backward compatibility.
    //!
    //! **NOTICE**: The pool health functionality has been extracted to its own crate.
    //! Please migrate to using `riptide-pool` crate directly:
    //!
    //! ```rust
    //! // Old (still works):
    //! use riptide_core::pool_health::*;
    //!
    //! // New (recommended):
    //! use riptide_pool::{PoolHealthMonitor, PoolHealthStatus, HealthLevel};
    //! ```
    pub use riptide_pool::{
        HealthLevel, HealthTrend, MemoryHealthStats, MemoryPressureLevel, PoolHealthMonitor,
        PoolHealthStatus,
    };
}

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

// Re-export from riptide-config and common
pub use common::{
    // Config and validation are now from riptide-config (re-exported through common)
    BuilderError,
    BuilderResult,
    CommonValidator,
    ConfigBuilder,
    ConfigValidator,
    ContentTypeValidator,
    // Error conversions remain in common
    CoreErrorConverter,
    DefaultConfigBuilder,
    ErrorPatterns,
    IntoCore,
    ParameterValidator,
    SizeValidator,
    UrlValidator,
    ValidationConfig,
    ValidationResult,
    WithErrorContext,
};

// Alias for backward compatibility
pub use riptide_config::ConfigValue;
pub use types::*;

// Re-export core functionality from modular crates (P2-F1 Day 3)
pub use component::{ComponentId, ComponentMeta};
pub use conditional::{CacheValidation, ConditionalRequest, ConditionalResponse};
pub use reliability::{ExtractionMode, ReliabilityConfig, ReliabilityMetrics, ReliableExtractor};
pub use wasm_validation::{ComponentMetadata, TypeSignature, ValidationReport, WitValidator};

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
        markdown: doc.markdown,
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
