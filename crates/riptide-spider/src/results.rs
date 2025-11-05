//! # Crawl Results
//!
//! Type definitions for raw and enriched crawl results.
//!
//! The spider produces two distinct result types:
//! - **RawCrawlResult**: HTTP response data without processing
//! - **EnrichedCrawlResult**: Processed result with extracted links and text
//!
//! This separation enables:
//! - Deferred content processing for performance
//! - Pluggable extraction strategies via ContentExtractor trait
//! - Pipeline-based processing architectures
//! - Caching raw results before extraction
//!
//! ## Architecture
//!
//! ```text
//! HTTP Response → RawCrawlResult → enrich() → EnrichedCrawlResult
//!                                      ↓
//!                              ContentExtractor
//! ```
//!
//! ## Example
//!
//! ```rust,no_run
//! use riptide_spider::results::{RawCrawlResult, enrich};
//! use riptide_spider::extractor::BasicExtractor;
//! use url::Url;
//! use http::{StatusCode, HeaderMap};
//!
//! let raw = RawCrawlResult {
//!     url: Url::parse("https://example.com").unwrap(),
//!     html: "<html><body><a href='/page'>Link</a></body></html>".to_string(),
//!     status: StatusCode::OK,
//!     headers: HeaderMap::new(),
//! };
//!
//! let extractor = BasicExtractor;
//! let enriched = enrich(raw, &extractor);
//!
//! assert!(enriched.extracted_urls.len() > 0);
//! assert!(enriched.text_content.is_some());
//! ```

use crate::extractor::ContentExtractor;
use http::{HeaderMap, StatusCode};
use url::Url;

/// Raw crawl result containing unprocessed HTTP response data.
///
/// This is the direct output from the HTTP fetch layer, before any
/// content extraction or processing occurs.
///
/// ## Fields
///
/// - `url`: The URL that was fetched
/// - `html`: Raw HTML response body
/// - `status`: HTTP status code (200, 404, etc.)
/// - `headers`: HTTP response headers
///
/// ## Design Notes
///
/// This type is intentionally minimal and fast to construct. All
/// expensive processing (link extraction, text parsing) is deferred
/// to the enrichment step.
///
/// The separation allows:
/// - Immediate URL queueing without blocking on extraction
/// - Batch processing of raw results
/// - Caching raw responses before extraction
/// - Different extraction strategies on the same raw data
#[derive(Debug, Clone)]
pub struct RawCrawlResult {
    /// The URL that was successfully fetched
    pub url: Url,

    /// Raw HTML content from the HTTP response body
    pub html: String,

    /// HTTP status code from the response
    pub status: StatusCode,

    /// HTTP response headers (Content-Type, Cache-Control, etc.)
    pub headers: HeaderMap,
}

/// Enriched crawl result containing extracted and processed content.
///
/// This is the output after applying a ContentExtractor to a RawCrawlResult.
/// It contains all the raw data plus extracted links and text content.
///
/// ## Fields
///
/// - `raw`: The original raw crawl result
/// - `extracted_urls`: Links found in the HTML content
/// - `text_content`: Extracted plain text (if extraction succeeded)
///
/// ## Design Notes
///
/// This type contains the raw result by value, allowing downstream
/// consumers to access both the original response data and the
/// extracted content without additional lookups.
///
/// The `text_content` is optional because:
/// - Some extractors may not implement text extraction
/// - Some pages may have no meaningful text content
/// - Extraction may fail on malformed HTML
#[derive(Debug, Clone)]
pub struct EnrichedCrawlResult {
    /// The original raw crawl result
    pub raw: RawCrawlResult,

    /// URLs extracted from the HTML content by the ContentExtractor
    pub extracted_urls: Vec<Url>,

    /// Plain text content extracted from the HTML (if available)
    pub text_content: Option<String>,
}

/// Enrich a raw crawl result using a content extractor.
///
/// This function transforms a RawCrawlResult into an EnrichedCrawlResult
/// by applying the provided ContentExtractor to extract links and text.
///
/// # Arguments
///
/// * `raw` - The raw crawl result to enrich
/// * `extractor` - The content extractor implementation to use
///
/// # Returns
///
/// An EnrichedCrawlResult containing the original raw data plus
/// extracted links and text content.
///
/// # Example
///
/// ```rust,no_run
/// use riptide_spider::results::{RawCrawlResult, enrich};
/// use riptide_spider::extractor::{BasicExtractor, NoOpExtractor};
/// use url::Url;
/// use http::{StatusCode, HeaderMap};
///
/// let raw = RawCrawlResult {
///     url: Url::parse("https://example.com").unwrap(),
///     html: "<html><body>Content</body></html>".to_string(),
///     status: StatusCode::OK,
///     headers: HeaderMap::new(),
/// };
///
/// // Use BasicExtractor for normal extraction
/// let enriched = enrich(raw.clone(), &BasicExtractor);
/// assert!(enriched.text_content.is_some());
///
/// // Use NoOpExtractor for spider-only mode
/// let enriched_noop = enrich(raw, &NoOpExtractor);
/// assert!(enriched_noop.extracted_urls.is_empty());
/// assert!(enriched_noop.text_content.is_none());
/// ```
///
/// # Performance
///
/// This function delegates all extraction work to the ContentExtractor.
/// Performance characteristics depend on the chosen extractor:
///
/// - **BasicExtractor**: Fast regex-based extraction, minimal allocations
/// - **NoOpExtractor**: Zero-cost, returns empty results immediately
/// - **Custom extractors**: Varies based on implementation
///
/// For high-throughput scenarios, consider:
/// - Batching enrichment operations
/// - Using parallel extraction with rayon
/// - Caching extractor state (compiled regexes, parsers)
pub fn enrich(raw: RawCrawlResult, extractor: &dyn ContentExtractor) -> EnrichedCrawlResult {
    let extracted_urls = extractor.extract_links(&raw.html, &raw.url);
    let text_content = extractor.extract_text(&raw.html);

    EnrichedCrawlResult {
        raw,
        extracted_urls,
        text_content,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extractor::{BasicExtractor, NoOpExtractor};

    fn create_raw_result(url: &str, html: &str) -> RawCrawlResult {
        RawCrawlResult {
            url: Url::parse(url).unwrap(),
            html: html.to_string(),
            status: StatusCode::OK,
            headers: HeaderMap::new(),
        }
    }

    #[test]
    fn test_raw_result_creation() {
        let raw = create_raw_result("https://example.com", "<html><body>Test</body></html>");

        assert_eq!(raw.url.as_str(), "https://example.com/");
        assert!(raw.html.contains("Test"));
        assert_eq!(raw.status, StatusCode::OK);
    }

    #[test]
    fn test_enrich_with_basic_extractor() {
        let html = r#"<html>
            <body>
                <a href="/page1">Link 1</a>
                <a href="https://example.com/page2">Link 2</a>
                <p>Some text content</p>
            </body>
        </html>"#;

        let raw = create_raw_result("https://example.com", html);
        let extractor = BasicExtractor;
        let enriched = enrich(raw, &extractor);

        // Should extract links
        assert_eq!(enriched.extracted_urls.len(), 2);
        assert_eq!(
            enriched.extracted_urls[0].as_str(),
            "https://example.com/page1"
        );
        assert_eq!(
            enriched.extracted_urls[1].as_str(),
            "https://example.com/page2"
        );

        // Should extract text
        assert!(enriched.text_content.is_some());
        let text = enriched.text_content.unwrap();
        assert!(text.contains("Link 1"));
        assert!(text.contains("Some text content"));
    }

    #[test]
    fn test_enrich_with_noop_extractor() {
        let html = r#"<html><body><a href="/page">Link</a></body></html>"#;

        let raw = create_raw_result("https://example.com", html);
        let extractor = NoOpExtractor;
        let enriched = enrich(raw, &extractor);

        // NoOp extractor should return empty results
        assert_eq!(enriched.extracted_urls.len(), 0);
        assert!(enriched.text_content.is_none());
    }

    #[test]
    fn test_enrich_preserves_raw_data() {
        let url = "https://example.com/test";
        let html = "<html><body>Test</body></html>";
        let raw = create_raw_result(url, html);

        let extractor = BasicExtractor;
        let enriched = enrich(raw, &extractor);

        // Check that raw data is preserved
        assert_eq!(enriched.raw.url.as_str(), "https://example.com/test");
        assert_eq!(enriched.raw.html, html);
        assert_eq!(enriched.raw.status, StatusCode::OK);
    }

    #[test]
    fn test_enrich_empty_html() {
        let raw = create_raw_result("https://example.com", "");
        let extractor = BasicExtractor;
        let enriched = enrich(raw, &extractor);

        assert_eq!(enriched.extracted_urls.len(), 0);
        assert!(enriched.text_content.is_none());
    }

    #[test]
    fn test_enrich_malformed_html() {
        let html = r#"<html><body><a href="/good"><a href="bad url"><p>Text"#;
        let raw = create_raw_result("https://example.com", html);
        let extractor = BasicExtractor;
        let enriched = enrich(raw, &extractor);

        // Should extract valid URLs and text despite malformed HTML
        assert!(enriched.extracted_urls.len() >= 1);
        assert!(enriched.text_content.is_some());
    }

    #[test]
    fn test_different_status_codes() {
        let mut raw = create_raw_result("https://example.com", "<html></html>");

        raw.status = StatusCode::NOT_FOUND;
        let enriched = enrich(raw.clone(), &BasicExtractor);
        assert_eq!(enriched.raw.status, StatusCode::NOT_FOUND);

        raw.status = StatusCode::INTERNAL_SERVER_ERROR;
        let enriched = enrich(raw, &BasicExtractor);
        assert_eq!(enriched.raw.status, StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_results_are_clone() {
        let raw = create_raw_result("https://example.com", "<html></html>");
        let raw_clone = raw.clone();
        assert_eq!(raw.url, raw_clone.url);

        let enriched = enrich(raw, &BasicExtractor);
        let enriched_clone = enriched.clone();
        assert_eq!(enriched.raw.url, enriched_clone.raw.url);
    }

    #[test]
    fn test_results_are_debug() {
        let raw = create_raw_result("https://example.com", "<html></html>");
        let debug_str = format!("{:?}", raw);
        assert!(debug_str.contains("RawCrawlResult"));

        let enriched = enrich(raw, &BasicExtractor);
        let debug_str = format!("{:?}", enriched);
        assert!(debug_str.contains("EnrichedCrawlResult"));
    }
}
