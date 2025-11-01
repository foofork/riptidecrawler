//! Extractor trait definitions
//!
//! This module provides trait abstractions for various extraction strategies,
//! enabling dependency injection and breaking circular dependencies.

use crate::ExtractedDoc;
use anyhow::Result;

/// WASM extractor trait for dependency injection
pub trait WasmExtractor: Send + Sync {
    fn extract(&self, html: &[u8], url: &str, mode: &str) -> Result<ExtractedDoc>;
}

/// Native HTML parser trait for dependency injection
///
/// This trait abstracts HTML parsing functionality to break circular dependencies
/// between `riptide-reliability` and `riptide-extraction`.
///
/// # Implementation
///
/// The primary implementation is `NativeHtmlParser` in `riptide-extraction`,
/// which uses the `scraper` crate for robust HTML parsing.
///
/// # Usage in Reliability Patterns
///
/// ```rust,ignore
/// use riptide_types::extractors::HtmlParser;
///
/// fn extract_with_parser(parser: &dyn HtmlParser, html: &str, url: &str) -> Result<ExtractedDoc> {
///     parser.parse_html(html, url)
/// }
/// ```
pub trait HtmlParser: Send + Sync {
    /// Parse HTML and extract structured document
    ///
    /// # Arguments
    /// * `html` - Raw HTML string (can be from headless or direct fetch)
    /// * `url` - Source URL for link resolution and metadata
    ///
    /// # Returns
    /// * `Ok(ExtractedDoc)` - Successfully parsed and extracted document
    /// * `Err(_)` - Parsing or extraction failed
    fn parse_html(&self, html: &str, url: &str) -> Result<ExtractedDoc>;

    /// Parse HTML with quality-based fallback strategies
    ///
    /// This method tries multiple extraction approaches if the initial
    /// parse produces low-quality results.
    ///
    /// # Arguments
    /// * `html` - Raw HTML string
    /// * `url` - Source URL
    ///
    /// # Returns
    /// * `Ok(ExtractedDoc)` - Best available extraction result
    /// * `Err(_)` - All extraction strategies failed
    fn parse_with_fallbacks(&self, html: &str, url: &str) -> Result<ExtractedDoc> {
        // Default implementation: just use primary parser
        self.parse_html(html, url)
    }
}
