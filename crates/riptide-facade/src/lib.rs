//! # Riptide Facade
//!
//! Unified facade and composition layer for the Riptide web scraping framework.
//!
//! This crate provides a simplified, cohesive API surface for accessing all Riptide
//! functionality without requiring direct knowledge of the 24+ specialized crates.
//!
//! ## Features
//!
//! - **Simplified API**: Task-oriented facades for common operations
//! - **Reduced Coupling**: Abstract internal crate boundaries
//! - **Unified Error Handling**: Single error type with context preservation
//! - **Composition Patterns**: Pre-built workflows and pipelines
//! - **Type Safety**: Leverage Rust's type system for compile-time guarantees
//! - **Async-First**: Native async/await support throughout
//!
//! ## Quick Start
//!
//! ```no_run
//! use riptide_facade::{Riptide, ScrapeOptions};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Build Riptide instance with configuration
//!     let riptide = Riptide::builder()
//!         .with_default_config()
//!         .build()?;
//!
//!     // Simple scraping
//!     let doc = riptide.scraper()
//!         .fetch("https://example.com")
//!         .await?;
//!
//!     println!("Title: {}", doc.title);
//!     println!("Content length: {}", doc.text.len());
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! The facade provides domain-specific entry points:
//!
//! - **ScraperFacade**: Web page fetching and extraction
//! - **SpiderFacade**: Web crawling and spidering
//! - **BrowserFacade**: Headless browser automation
//! - **ExtractorFacade**: Content extraction strategies
//! - **IntelligenceFacade**: LLM-powered operations
//! - **SecurityFacade**: Authentication and rate limiting
//! - **MonitoringFacade**: Metrics and telemetry
//! - **CacheFacade**: Caching infrastructure
//!
//! ## Design Principles
//!
//! 1. **Layered Abstraction**: High-level APIs delegate to specialized crates
//! 2. **Trait-Based Composition**: Flexible and testable architecture
//! 3. **Builder Pattern**: Fluent configuration APIs
//! 4. **Error Context**: Rich errors without losing details
//! 5. **Feature Flags**: Optional functionality behind gates
//! 6. **Zero-Cost Abstractions**: Minimal overhead

// Re-export types from riptide-types (always available)
pub use riptide_types::{
    ContentChunk, ExtractedContent, ExtractedDoc, ExtractionQuality, ExtractionResult,
};

// Core modules (always available)
pub mod builder;
pub mod config;
pub mod error;
pub mod runtime;

// Re-export core types
pub use builder::RiptideBuilder;
pub use config::{RiptideConfig, FetchConfig, SpiderConfig};
pub use error::{Result, RiptideError};
pub use runtime::RiptideRuntime;

// Facade modules (feature-gated)
pub mod facades;

#[cfg(feature = "scraper")]
pub use facades::scraper::ScraperFacade;

#[cfg(feature = "spider")]
pub use facades::spider::SpiderFacade;

#[cfg(feature = "browser")]
pub use facades::browser::BrowserFacade;

#[cfg(feature = "extractor")]
pub use facades::extractor::ExtractorFacade;

#[cfg(feature = "intelligence")]
pub use facades::intelligence::IntelligenceFacade;

#[cfg(feature = "security")]
pub use facades::security::SecurityFacade;

#[cfg(feature = "monitoring")]
pub use facades::monitoring::MonitoringFacade;

#[cfg(feature = "cache")]
pub use facades::cache::CacheFacade;

// Trait definitions (feature-gated)
pub mod traits;

// Composition patterns (feature-gated)
#[cfg(any(feature = "scraper", feature = "spider"))]
pub mod composition;

#[cfg(any(feature = "scraper", feature = "spider"))]
pub use composition::{Pipeline, Workflow, WorkflowBuilder};

// Internal adapters (not public API)
mod adapters;

// Prelude for common imports
pub mod prelude {
    pub use crate::builder::RiptideBuilder;
    pub use crate::config::RiptideConfig;
    pub use crate::error::{Result, RiptideError};
    pub use crate::runtime::RiptideRuntime;
    pub use crate::Riptide;

    #[cfg(feature = "scraper")]
    pub use crate::facades::scraper::{ScrapeOptions, ScraperFacade};

    #[cfg(feature = "spider")]
    pub use crate::facades::spider::{CrawlBudget, CrawlResult, SpiderFacade};

    #[cfg(feature = "browser")]
    pub use crate::facades::browser::{BrowserAction, BrowserFacade, ScreenshotOptions};

    #[cfg(feature = "extractor")]
    pub use crate::facades::extractor::{ExtractionStrategy, ExtractorFacade};

    #[cfg(feature = "intelligence")]
    pub use crate::facades::intelligence::IntelligenceFacade;

    #[cfg(feature = "security")]
    pub use crate::facades::security::SecurityFacade;

    #[cfg(feature = "monitoring")]
    pub use crate::facades::monitoring::MonitoringFacade;

    #[cfg(feature = "cache")]
    pub use crate::facades::cache::CacheFacade;
}

use std::sync::Arc;

/// Main entry point for the Riptide facade.
///
/// Provides unified access to all Riptide functionality through
/// domain-specific facades.
///
/// # Example
///
/// ```no_run
/// use riptide_facade::Riptide;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let riptide = Riptide::builder()
///         .with_default_config()
///         .build()?;
///
///     // Use facades
///     let doc = riptide.scraper().fetch("https://example.com").await?;
///
///     Ok(())
/// }
/// ```
pub struct Riptide {
    config: RiptideConfig,
    runtime: Arc<RiptideRuntime>,
}

impl Riptide {
    /// Create a new builder for configuring Riptide.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use riptide_facade::Riptide;
    ///
    /// let riptide = Riptide::builder()
    ///     .with_default_config()
    ///     .build()?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn builder() -> RiptideBuilder {
        RiptideBuilder::new()
    }

    /// Create a Riptide instance with default configuration.
    ///
    /// Convenience method for simple use cases.
    pub fn with_defaults() -> Result<Self> {
        Self::builder().with_default_config().build()
    }

    /// Get the scraper facade for web page fetching.
    ///
    /// Requires the `scraper` feature.
    #[cfg(feature = "scraper")]
    pub fn scraper(&self) -> ScraperFacade {
        ScraperFacade::new(self.config.clone(), Arc::clone(&self.runtime))
    }

    /// Get the spider facade for web crawling.
    ///
    /// Requires the `spider` feature.
    #[cfg(feature = "spider")]
    pub fn spider(&self) -> SpiderFacade {
        SpiderFacade::new(self.config.clone(), Arc::clone(&self.runtime))
    }

    /// Get the browser facade for headless browser automation.
    ///
    /// Requires the `browser` feature.
    #[cfg(feature = "browser")]
    pub fn browser(&self) -> BrowserFacade {
        BrowserFacade::new(self.config.clone(), Arc::clone(&self.runtime))
    }

    /// Get the extractor facade for content extraction.
    ///
    /// Requires the `extractor` feature.
    #[cfg(feature = "extractor")]
    pub fn extractor(&self) -> ExtractorFacade {
        ExtractorFacade::new(self.config.clone(), Arc::clone(&self.runtime))
    }

    /// Get the intelligence facade for LLM operations.
    ///
    /// Requires the `intelligence` feature.
    #[cfg(feature = "intelligence")]
    pub fn intelligence(&self) -> IntelligenceFacade {
        IntelligenceFacade::new(self.config.clone(), Arc::clone(&self.runtime))
    }

    /// Get the security facade for authentication and rate limiting.
    ///
    /// Requires the `security` feature.
    #[cfg(feature = "security")]
    pub fn security(&self) -> SecurityFacade {
        SecurityFacade::new(self.config.clone(), Arc::clone(&self.runtime))
    }

    /// Get the monitoring facade for metrics and telemetry.
    ///
    /// Requires the `monitoring` feature.
    #[cfg(feature = "monitoring")]
    pub fn monitoring(&self) -> MonitoringFacade {
        MonitoringFacade::new(self.config.clone(), Arc::clone(&self.runtime))
    }

    /// Get the cache facade for caching infrastructure.
    ///
    /// Requires the `cache` feature.
    #[cfg(feature = "cache")]
    pub fn cache(&self) -> CacheFacade {
        CacheFacade::new(self.config.clone(), Arc::clone(&self.runtime))
    }

    /// Get access to the underlying runtime.
    ///
    /// Useful for advanced scenarios that need direct runtime access.
    pub fn runtime(&self) -> &Arc<RiptideRuntime> {
        &self.runtime
    }

    /// Get the current configuration.
    pub fn config(&self) -> &RiptideConfig {
        &self.config
    }

    /// Create a workflow builder for composing operations.
    ///
    /// Requires either `scraper` or `spider` feature.
    #[cfg(any(feature = "scraper", feature = "spider"))]
    pub fn workflow(&self) -> WorkflowBuilder {
        WorkflowBuilder::new(self.clone())
    }
}

// Clone implementation for convenience
impl Clone for Riptide {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            runtime: Arc::clone(&self.runtime),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_creation() {
        let _builder = Riptide::builder();
        // Builder should be created successfully
    }

    #[cfg(feature = "scraper")]
    #[tokio::test]
    async fn test_with_defaults() {
        let riptide = Riptide::with_defaults();
        assert!(riptide.is_ok(), "Should create with defaults");
    }
}
