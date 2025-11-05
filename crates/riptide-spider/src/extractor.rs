//! Content extraction traits and implementations for spider
//!
//! This module provides modular, swappable extraction strategies that decouple
//! spider (URL discovery) from extraction (content parsing). Spider can work
//! with ANY extractor implementation or NO extractor at all (spider-only mode).

use async_trait::async_trait;
use url::Url;

/// ContentExtractor trait enables modular, swappable extraction strategies.
///
/// Spider can work with ANY extractor implementation (ICS, JSON-LD, CSS, LLM, etc.)
/// or NO extractor at all (spider-only mode).
///
/// # Examples
///
/// ```
/// use riptide_spider::extractor::{ContentExtractor, BasicExtractor, NoOpExtractor};
///
/// // Basic extraction
/// let extractor = BasicExtractor;
/// let links = extractor.extract_links("<a href='/page'>link</a>", &base_url);
/// let text = extractor.extract_text("<p>Hello world</p>");
///
/// // No-op for spider-only mode
/// let noop = NoOpExtractor;
/// assert_eq!(noop.extract_links("<html>...</html>", &base_url).len(), 0);
/// ```
#[async_trait]
pub trait ContentExtractor: Send + Sync {
    /// Extract links from HTML content
    ///
    /// # Arguments
    /// * `html` - Raw HTML content
    /// * `base_url` - Base URL for resolving relative links
    ///
    /// # Returns
    /// Vector of absolute URLs found in the content
    fn extract_links(&self, html: &str, base_url: &Url) -> Vec<Url>;

    /// Extract text content from HTML
    ///
    /// # Arguments
    /// * `html` - Raw HTML content
    ///
    /// # Returns
    /// Extracted text content, or None if extraction fails
    fn extract_text(&self, html: &str) -> Option<String>;

    /// Strategy identifier for debugging and metrics
    fn strategy_name(&self) -> &'static str;
}

/// BasicExtractor - Default implementation using simple text extraction
///
/// Extracted from riptide-spider/src/core.rs:620-647 (simple_text_extraction).
/// Provides basic link and text extraction without external dependencies.
///
/// # Implementation Details
/// - Link extraction: Parses <a href="..."> tags
/// - Text extraction: Strips HTML tags, preserves text content
/// - No DOM parsing: Fast, lightweight, suitable for most use cases
#[derive(Debug, Clone, Default)]
pub struct BasicExtractor;

impl ContentExtractor for BasicExtractor {
    fn extract_links(&self, html: &str, base_url: &Url) -> Vec<Url> {
        let mut links = Vec::new();

        // Simple regex-free link extraction
        let mut chars = html.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '<' {
                // Check if this is an <a> tag
                let tag_start: String = chars.clone().take(2).collect();
                if tag_start.to_lowercase() == "a " || tag_start.to_lowercase() == "a>" {
                    // Find href attribute
                    let remaining: String = chars.clone().take_while(|&c| c != '>').collect();
                    if let Some(href_start) = remaining.find("href=\"") {
                        let href_offset = href_start + 6;
                        let href_str = &remaining[href_offset..];
                        if let Some(href_end) = href_str.find('"') {
                            let href = &href_str[..href_end];
                            // Resolve relative URLs
                            if let Ok(url) = base_url.join(href) {
                                links.push(url);
                            }
                        }
                    }
                }
            }
        }

        links
    }

    fn extract_text(&self, html: &str) -> Option<String> {
        // MOVED FROM: crates/riptide-spider/src/core.rs:629-647
        // Original: SpiderCore::simple_text_extraction()
        let mut text = String::new();
        let mut in_tag = false;

        for char in html.chars() {
            match char {
                '<' => in_tag = true,
                '>' => in_tag = false,
                c if !in_tag && !c.is_control() => text.push(c),
                _ => {}
            }
        }

        if text.trim().is_empty() {
            None
        } else {
            Some(text)
        }
    }

    fn strategy_name(&self) -> &'static str {
        "basic"
    }
}

/// NoOpExtractor - No-op implementation for spider-only usage
///
/// Used when you only want URL discovery without any content extraction.
/// Returns empty results for all extraction operations.
///
/// # Use Cases
/// - Pure URL discovery (sitemap generation)
/// - Link graph analysis
/// - Site structure mapping
/// - Robots.txt validation
///
/// # Examples
///
/// ```
/// use riptide_spider::extractor::{ContentExtractor, NoOpExtractor};
/// use url::Url;
///
/// let extractor = NoOpExtractor;
/// let base_url = Url::parse("https://example.com").unwrap();
///
/// // No links extracted
/// assert_eq!(extractor.extract_links("<html>...</html>", &base_url).len(), 0);
///
/// // No text extracted
/// assert_eq!(extractor.extract_text("<p>Hello</p>"), None);
/// ```
#[derive(Debug, Clone, Default)]
pub struct NoOpExtractor;

impl ContentExtractor for NoOpExtractor {
    fn extract_links(&self, _html: &str, _base_url: &Url) -> Vec<Url> {
        Vec::new() // Don't extract anything
    }

    fn extract_text(&self, _html: &str) -> Option<String> {
        None // Don't extract anything
    }

    fn strategy_name(&self) -> &'static str {
        "noop"
    }
}

// Future extractors can be added here:
// - IcsExtractor (iCalendar parsing)
// - JsonLdExtractor (JSON-LD structured data)
// - LlmExtractor (LLM-based with schema)
// - BrowserExtractor (headless browser rendering)
// - WasmExtractor (custom WASM modules)

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_extractor_text() {
        let extractor = BasicExtractor;
        let html = "<html><body><p>Hello world</p></body></html>";
        let text = extractor.extract_text(html);

        assert!(text.is_some());
        let text = text.unwrap();
        assert!(text.contains("Hello world"));
        assert!(!text.contains("<p>"));
    }

    #[test]
    fn test_basic_extractor_empty() {
        let extractor = BasicExtractor;
        let html = "<html><body></body></html>";
        let text = extractor.extract_text(html);

        assert!(text.is_none());
    }

    #[test]
    fn test_basic_extractor_links() {
        let extractor = BasicExtractor;
        let base_url = Url::parse("https://example.com").unwrap();
        let html = r#"<a href="/page1">Link 1</a><a href="https://other.com/page2">Link 2</a>"#;

        let links = extractor.extract_links(html, &base_url);

        assert_eq!(links.len(), 2);
        assert_eq!(links[0].as_str(), "https://example.com/page1");
        assert_eq!(links[1].as_str(), "https://other.com/page2");
    }

    #[test]
    fn test_noop_extractor() {
        let extractor = NoOpExtractor;
        let base_url = Url::parse("https://example.com").unwrap();

        // Should extract nothing
        let links = extractor.extract_links("<a href='/page'>Link</a>", &base_url);
        assert_eq!(links.len(), 0);

        let text = extractor.extract_text("<p>Hello</p>");
        assert!(text.is_none());

        assert_eq!(extractor.strategy_name(), "noop");
    }

    #[test]
    fn test_strategy_names() {
        assert_eq!(BasicExtractor.strategy_name(), "basic");
        assert_eq!(NoOpExtractor.strategy_name(), "noop");
    }
}
