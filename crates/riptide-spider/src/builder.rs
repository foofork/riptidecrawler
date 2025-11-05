//! Spider builder pattern with extractor plugin support
//!
//! This module provides a builder for creating Spider instances with
//! configurable extraction strategies. Enables spider-only mode (no extraction)
//! or spider with extraction (plugin any ContentExtractor).
//!
//! # Architecture
//!
//! The builder pattern decouples spider configuration from construction,
//! allowing flexible combination of:
//! - Spider settings (depth, budget, robots.txt)
//! - Extraction strategy (basic, no-op, ICS, JSON-LD, LLM, etc.)
//! - Session management
//! - Circuit breakers and fault tolerance
//!
//! # Examples
//!
//! ```rust,no_run
//! use riptide_spider::builder::SpiderBuilder;
//! use riptide_spider::extractor::{BasicExtractor, NoOpExtractor};
//! use riptide_spider::config::SpiderConfig;
//!
//! // Spider-only mode (no extraction)
//! let spider = SpiderBuilder::new()
//!     .with_config(SpiderConfig::default())
//!     .with_extractor(Box::new(NoOpExtractor))
//!     .build();
//!
//! // Spider with basic extraction
//! let spider = SpiderBuilder::new()
//!     .with_config(SpiderConfig::default())
//!     .with_extractor(Box::new(BasicExtractor))
//!     .build();
//! ```

use crate::config::SpiderConfig;
use crate::extractor::{BasicExtractor, ContentExtractor};
use std::sync::Arc;

/// SpiderBuilder - Construct Spider instances with extractor plugins
///
/// Provides a fluent API for building Spider instances with configurable
/// extraction strategies. Supports both spider-only mode (no extraction)
/// and spider with extraction (any ContentExtractor implementation).
///
/// # Design Pattern
///
/// Uses the Builder pattern to separate spider construction from configuration:
/// 1. Create builder: `SpiderBuilder::new()`
/// 2. Configure: `.with_config()`, `.with_extractor()`, etc.
/// 3. Build: `.build()` or `.build_raw()` (spider-only)
///
/// # Extractor Plugins
///
/// - `NoOpExtractor`: Spider-only mode (pure URL discovery)
/// - `BasicExtractor`: Simple text and link extraction
/// - `IcsExtractor`: iCalendar parsing (future)
/// - `JsonLdExtractor`: JSON-LD structured data (future)
/// - `LlmExtractor`: LLM-based extraction with schema (future)
/// - Custom: Implement `ContentExtractor` trait
#[derive(Default)]
pub struct SpiderBuilder {
    config: Option<SpiderConfig>,
    extractor: Option<Box<dyn ContentExtractor>>,
    respect_robots: Option<bool>,
    max_depth: Option<u32>,
    max_pages: Option<u32>,
}

impl SpiderBuilder {
    /// Create a new SpiderBuilder with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Configure the spider with a SpiderConfig
    ///
    /// # Arguments
    /// * `config` - Spider configuration (depth, budget, delays, etc.)
    pub fn with_config(mut self, config: SpiderConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Set the content extractor plugin
    ///
    /// # Arguments
    /// * `extractor` - Any ContentExtractor implementation (boxed trait object)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use riptide_spider::builder::SpiderBuilder;
    /// use riptide_spider::extractor::{BasicExtractor, NoOpExtractor};
    ///
    /// // With extraction
    /// let builder = SpiderBuilder::new()
    ///     .with_extractor(Box::new(BasicExtractor));
    ///
    /// // Without extraction (spider-only)
    /// let builder = SpiderBuilder::new()
    ///     .with_extractor(Box::new(NoOpExtractor));
    /// ```
    pub fn with_extractor(mut self, extractor: Box<dyn ContentExtractor>) -> Self {
        self.extractor = Some(extractor);
        self
    }

    /// Set whether to respect robots.txt
    ///
    /// # Arguments
    /// * `respect` - True to respect robots.txt, false to ignore
    ///
    /// # Default
    /// `true` - Respect robots.txt by default (ethical crawling)
    pub fn respect_robots(mut self, respect: bool) -> Self {
        self.respect_robots = Some(respect);
        self
    }

    /// Set maximum crawl depth
    ///
    /// # Arguments
    /// * `depth` - Maximum depth from seed URL (0 = seed only, 1 = seed + 1 hop, etc.)
    pub fn max_depth(mut self, depth: u32) -> Self {
        self.max_depth = Some(depth);
        self
    }

    /// Set maximum pages to crawl
    ///
    /// # Arguments
    /// * `pages` - Maximum number of pages to crawl
    pub fn max_pages(mut self, pages: u32) -> Self {
        self.max_pages = Some(pages);
        self
    }

    /// Build a spider instance
    ///
    /// Creates a Spider with the configured settings. If no extractor is set,
    /// defaults to BasicExtractor.
    ///
    /// # Returns
    /// Configured Spider instance (note: actual Spider construction requires
    /// integration with existing Spider::new() - this is a demonstration)
    ///
    /// # Future Integration
    ///
    /// This builder will replace direct Spider::new() calls:
    ///
    /// ```rust,ignore
    /// // Current (tightly coupled)
    /// let spider = Spider::new(config).await?;
    ///
    /// // Future (with builder + plugin)
    /// let spider = SpiderBuilder::new()
    ///     .with_config(config)
    ///     .with_extractor(Box::new(BasicExtractor))
    ///     .build()
    ///     .await?;
    /// ```
    pub fn build(self) -> BuiltSpider {
        let config = self.config.unwrap_or_default();
        let extractor = self
            .extractor
            .unwrap_or_else(|| Box::new(BasicExtractor));

        BuiltSpider {
            config,
            extractor: Arc::new(extractor),
            respect_robots: self.respect_robots.unwrap_or(true),
        }
    }

    /// Build a spider for spider-only mode (no extraction)
    ///
    /// Shorthand for `.with_extractor(Box::new(NoOpExtractor)).build()`
    pub fn build_raw(self) -> BuiltSpider {
        let config = self.config.unwrap_or_default();
        let extractor = Box::new(crate::extractor::NoOpExtractor) as Box<dyn ContentExtractor>;

        BuiltSpider {
            config,
            extractor: Arc::new(extractor),
            respect_robots: self.respect_robots.unwrap_or(true),
        }
    }
}

/// BuiltSpider - Result of SpiderBuilder
///
/// Represents a configured spider ready for crawling. Contains the selected
/// extractor plugin and configuration.
///
/// # Note
///
/// This is a transitional type during the decoupling process. Eventually,
/// this will be integrated directly into the Spider struct in core.rs.
pub struct BuiltSpider {
    pub config: SpiderConfig,
    pub extractor: Arc<Box<dyn ContentExtractor>>,
    pub respect_robots: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extractor::{BasicExtractor, NoOpExtractor};

    #[test]
    fn test_builder_with_basic_extractor() {
        let built = SpiderBuilder::new()
            .with_extractor(Box::new(BasicExtractor))
            .build();

        assert_eq!(built.extractor.strategy_name(), "basic");
        assert!(built.respect_robots);
    }

    #[test]
    fn test_builder_raw_spider() {
        let built = SpiderBuilder::new().build_raw();

        assert_eq!(built.extractor.strategy_name(), "noop");
        assert!(built.respect_robots);
    }

    #[test]
    fn test_builder_respect_robots() {
        let built = SpiderBuilder::new().respect_robots(false).build();

        assert!(!built.respect_robots);
    }

    #[test]
    fn test_builder_defaults() {
        let built = SpiderBuilder::new().build();

        // Should default to BasicExtractor
        assert_eq!(built.extractor.strategy_name(), "basic");
        assert!(built.respect_robots);
    }
}
