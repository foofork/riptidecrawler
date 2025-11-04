//! # Content Extractor
//!
//! Modular content extraction trait enabling pluggable extraction strategies.
//! Spider can work with ANY extractor implementation (ICS, JSON-LD, CSS, LLM, etc.)
//! or NO extractor at all (spider-only mode).
//!
//! ## Architecture
//!
//! The `ContentExtractor` trait provides a clean separation between:
//! - **Spider**: URL discovery and crawling logic
//! - **Extractor**: Content parsing and extraction strategies
//!
//! This enables:
//! - Plugin architecture for domain-specific extractors
//! - Testing with mock extractors
//! - Spider-only mode for pure URL discovery
//! - Zero-cost abstraction via trait objects
//!
//! ## Examples
//!
//! ```rust,no_run
//! use riptide_spider::extractor::{ContentExtractor, BasicExtractor, NoOpExtractor};
//! use url::Url;
//!
//! // Use basic extractor
//! let extractor = BasicExtractor;
//! let base_url = Url::parse("https://example.com").unwrap();
//! let html = r#"<a href="/page1">Link</a>"#;
//! let links = extractor.extract_links(html, &base_url);
//!
//! // Use no-op extractor for spider-only mode
//! let noop = NoOpExtractor;
//! let links = noop.extract_links(html, &base_url); // Returns empty vec
//! ```

use url::Url;

/// ContentExtractor trait enables modular, swappable extraction strategies.
///
/// All extractors must be `Send + Sync` for concurrent usage in the spider.
/// Implementations should be stateless or use interior mutability.
///
/// ## Design Principles
///
/// - **Single Responsibility**: Each extractor focuses on one extraction strategy
/// - **Performance**: Methods are synchronous for efficiency (use blocking for I/O)
/// - **Fallible**: Return `Vec` and `Option` rather than `Result` for simplicity
/// - **Debuggable**: Strategy names enable metrics and debugging
pub trait ContentExtractor: Send + Sync {
    /// Extract links from HTML content.
    ///
    /// # Arguments
    ///
    /// * `html` - Raw HTML content to parse
    /// * `base_url` - Base URL for resolving relative links
    ///
    /// # Returns
    ///
    /// Vector of absolute URLs found in the content. Invalid URLs are silently skipped.
    ///
    /// # Performance
    ///
    /// This method is called frequently during crawling. Implementations should:
    /// - Cache compiled regexes or parsers
    /// - Use streaming parsing for large documents
    /// - Avoid allocating unnecessary intermediate strings
    fn extract_links(&self, html: &str, base_url: &Url) -> Vec<Url>;

    /// Extract text content from HTML.
    ///
    /// # Arguments
    ///
    /// * `html` - Raw HTML content to parse
    ///
    /// # Returns
    ///
    /// Extracted text content, or `None` if extraction fails or content is empty.
    ///
    /// # Implementation Notes
    ///
    /// - Remove script and style tags
    /// - Decode HTML entities
    /// - Preserve whitespace semantics
    /// - Return `None` for empty content after cleaning
    fn extract_text(&self, html: &str) -> Option<String>;

    /// Strategy identifier for debugging and metrics.
    ///
    /// Used in logs, metrics, and error messages to identify which
    /// extraction strategy is being used.
    ///
    /// # Returns
    ///
    /// Static string identifying the extractor (e.g., "basic", "ics", "jsonld")
    fn strategy_name(&self) -> &'static str;
}

/// Basic HTML extractor using regex-based parsing.
///
/// This is the default extractor that provides simple, efficient extraction
/// without external dependencies. Suitable for general-purpose crawling.
///
/// ## Features
///
/// - Regex-based link extraction from `href` attributes
/// - Simple tag-stripping text extraction
/// - Zero external dependencies
/// - Fast and memory-efficient
///
/// ## Limitations
///
/// - Does not handle JavaScript-rendered content
/// - No support for complex CSS selectors
/// - Limited HTML entity decoding
/// - No DOM-aware parsing
///
/// For advanced extraction needs, use domain-specific extractors like
/// `IcsExtractor` or `JsonLdExtractor`.
#[derive(Debug, Clone, Copy, Default)]
pub struct BasicExtractor;

impl ContentExtractor for BasicExtractor {
    fn extract_links(&self, html: &str, base_url: &Url) -> Vec<Url> {
        // Regex pattern matches href="..." or href='...'
        // This is a simple implementation - Agent-2 will MOVE the production code
        // from core.rs:4-17 (extract_links_basic function)
        let link_regex = match regex::Regex::new(r#"href\s*=\s*["']([^"']+)["']"#) {
            Ok(re) => re,
            Err(_) => return Vec::new(),
        };

        let mut links = Vec::new();

        for cap in link_regex.captures_iter(html) {
            if let Some(link_str) = cap.get(1) {
                // Resolve relative URLs to absolute
                if let Ok(url) = base_url.join(link_str.as_str()) {
                    links.push(url);
                }
            }
        }

        links
    }

    fn extract_text(&self, html: &str) -> Option<String> {
        // Simple tag-stripping text extraction
        // Agent-2 will MOVE the production code from core.rs:629-647
        // (simple_text_extraction method)
        let mut text = String::new();
        let mut in_tag = false;

        for ch in html.chars() {
            match ch {
                '<' => in_tag = true,
                '>' => in_tag = false,
                c if !in_tag && !c.is_control() => text.push(c),
                _ => {}
            }
        }

        let trimmed = text.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    }

    fn strategy_name(&self) -> &'static str {
        "basic"
    }
}

/// No-op extractor for spider-only mode (pure URL discovery).
///
/// This extractor intentionally does nothing, allowing the spider to
/// discover and queue URLs without extracting any content.
///
/// ## Use Cases
///
/// - **Sitemap Generation**: Discover all URLs on a site
/// - **Link Validation**: Check for broken links without processing content
/// - **Performance Testing**: Benchmark spider performance without extraction overhead
/// - **Dry Runs**: Test crawl configuration before full extraction
///
/// ## Example
///
/// ```rust,ignore
/// use riptide_spider::Spider;
/// use riptide_spider::extractor::NoOpExtractor;
///
/// // Spider discovers URLs but doesn't extract content
/// // (Note: builder() API will be added in Week 4)
/// let spider = Spider::new(SpiderOpts {
///     extractor: Some(Box::new(NoOpExtractor)),
///     ..Default::default()
/// });
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct NoOpExtractor;

impl ContentExtractor for NoOpExtractor {
    fn extract_links(&self, _html: &str, _base_url: &Url) -> Vec<Url> {
        // Intentionally return empty - spider-only mode
        Vec::new()
    }

    fn extract_text(&self, _html: &str) -> Option<String> {
        // Intentionally return None - spider-only mode
        None
    }

    fn strategy_name(&self) -> &'static str {
        "noop"
    }
}

// ============================================================================
// FUTURE EXTRACTORS (Placeholder for Phase 1 continuation)
// ============================================================================
//
// These will be implemented in subsequent weeks as the plugin architecture
// matures. Each represents a specialized extraction strategy.

/// ICS calendar event extractor (future implementation).
///
/// Extracts iCalendar (ICS) events from HTML using microformats, hCalendar,
/// or embedded JSON-LD calendar data.
///
/// ## Roadmap
///
/// - Week 3.5: Basic ICS extraction
/// - Week 4.0: Microformat support
/// - Week 4.5: JSON-LD calendar support
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Default)]
pub struct IcsExtractor;

// Implementation will be added by Agent-3 in Week 3.5
// impl ContentExtractor for IcsExtractor { ... }

/// JSON-LD structured data extractor (future implementation).
///
/// Extracts structured data from JSON-LD scripts embedded in HTML.
/// Supports schema.org vocabularies for events, articles, products, etc.
///
/// ## Roadmap
///
/// - Week 4.0: Basic JSON-LD parsing
/// - Week 4.5: Schema.org validation
/// - Week 5.0: Multi-vocabulary support
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Default)]
pub struct JsonLdExtractor;

// Implementation will be added in Week 4.0
// impl ContentExtractor for JsonLdExtractor { ... }

/// LLM-powered extractor with custom schemas (future implementation).
///
/// Uses large language models to extract structured data according to
/// user-defined schemas. Enables zero-shot extraction for novel domains.
///
/// ## Roadmap
///
/// - Week 5.0: Schema definition framework
/// - Week 5.5: LLM integration (OpenAI, Anthropic)
/// - Week 6.0: Local model support (ONNX, llama.cpp)
#[allow(dead_code)]
#[derive(Debug)]
pub struct LlmExtractor {
    /// JSON schema defining extraction structure
    #[allow(dead_code)]
    schema: String,
}

// Implementation will be added in Week 5.0
// impl ContentExtractor for LlmExtractor { ... }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_extractor_links() {
        let extractor = BasicExtractor;
        let base_url = Url::parse("https://example.com").unwrap();
        let html = r#"<a href="/page1">Link 1</a><a href="https://other.com">External</a>"#;

        let links = extractor.extract_links(html, &base_url);

        assert_eq!(links.len(), 2);
        assert_eq!(links[0].as_str(), "https://example.com/page1");
        assert_eq!(links[1].as_str(), "https://other.com/");
    }

    #[test]
    fn test_basic_extractor_text() {
        let extractor = BasicExtractor;
        let html = r#"<html><body><p>Hello World</p></body></html>"#;

        let text = extractor.extract_text(html);

        assert!(text.is_some());
        assert!(text.unwrap().contains("Hello World"));
    }

    #[test]
    fn test_basic_extractor_empty_text() {
        let extractor = BasicExtractor;
        let html = r#"<html><body></body></html>"#;

        let text = extractor.extract_text(html);

        assert!(text.is_none());
    }

    #[test]
    fn test_noop_extractor() {
        let extractor = NoOpExtractor;
        let base_url = Url::parse("https://example.com").unwrap();
        let html = r#"<a href="/page1">Link</a>"#;

        let links = extractor.extract_links(html, &base_url);
        let text = extractor.extract_text(html);

        assert_eq!(links.len(), 0);
        assert!(text.is_none());
        assert_eq!(extractor.strategy_name(), "noop");
    }

    #[test]
    fn test_basic_extractor_strategy_name() {
        let extractor = BasicExtractor;
        assert_eq!(extractor.strategy_name(), "basic");
    }

    #[test]
    fn test_extractors_are_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<BasicExtractor>();
        assert_send_sync::<NoOpExtractor>();
    }

    #[test]
    fn test_basic_extractor_relative_links() {
        let extractor = BasicExtractor;
        let base_url = Url::parse("https://example.com/path/").unwrap();
        let html = r#"<a href="../other">Relative</a>"#;

        let links = extractor.extract_links(html, &base_url);

        assert_eq!(links.len(), 1);
        assert_eq!(links[0].as_str(), "https://example.com/other");
    }

    #[test]
    fn test_basic_extractor_malformed_html() {
        let extractor = BasicExtractor;
        let base_url = Url::parse("https://example.com").unwrap();
        let html = r#"<a href="/good"><a href="bad url"><a href="/also-good">"#;

        let links = extractor.extract_links(html, &base_url);

        // Should extract valid URLs and skip invalid ones
        assert!(links.len() >= 2);
    }
}
