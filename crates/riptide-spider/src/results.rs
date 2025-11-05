//! Spider result types - separated raw and enriched results
//!
//! This module separates spider results into:
//! - `RawCrawlResult`: Pure spider results (no extraction)
//! - `EnrichedCrawlResult`: Raw results + extracted content
//!
//! This separation enables:
//! - Spider-only mode (pure URL discovery)
//! - Flexible extraction (plugin any extractor)
//! - Clear boundaries (spider != extraction)

use crate::extractor::ContentExtractor;
use http::HeaderMap;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

/// RawCrawlResult - Spider result without extraction
///
/// Contains only the raw HTTP response data from spidering.
/// No link extraction, no text extraction - pure spider output.
///
/// # Use Cases
/// - Spider-only mode (URL discovery without extraction)
/// - Deferring extraction to later pipeline stages
/// - Streaming raw results for parallel processing
/// - Storage before extraction (e.g., distributed crawling)
///
/// # Examples
///
/// ```
/// use riptide_spider::results::RawCrawlResult;
/// use url::Url;
///
/// let result = RawCrawlResult {
///     url: Url::parse("https://example.com").unwrap(),
///     html: "<html>...</html>".to_string(),
///     status: http::StatusCode::OK,
///     headers: http::HeaderMap::new(),
///     content_type: Some("text/html".to_string()),
///     content_size: 1024,
///     processing_time: std::time::Duration::from_millis(150),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawCrawlResult {
    /// The URL that was crawled
    pub url: Url,

    /// Raw HTML content
    pub html: String,

    /// HTTP status code
    #[serde(with = "http_serde::status_code")]
    pub status: StatusCode,

    /// HTTP response headers
    #[serde(with = "http_serde::header_map")]
    pub headers: HeaderMap,

    /// Content-Type header value
    pub content_type: Option<String>,

    /// Size of content in bytes
    pub content_size: usize,

    /// Time spent processing this request
    pub processing_time: Duration,
}

impl RawCrawlResult {
    /// Create a new raw result from HTTP response
    pub fn new(
        url: Url,
        html: String,
        status: StatusCode,
        headers: HeaderMap,
        processing_time: Duration,
    ) -> Self {
        let content_size = html.len();
        let content_type = headers
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        Self {
            url,
            html,
            status,
            headers,
            content_type,
            content_size,
            processing_time,
        }
    }

    /// Check if the response was successful (2xx status)
    pub fn is_success(&self) -> bool {
        self.status.is_success()
    }

    /// Check if this is an HTML document
    pub fn is_html(&self) -> bool {
        self.content_type
            .as_ref()
            .map(|ct| ct.contains("text/html"))
            .unwrap_or(false)
    }
}

/// EnrichedCrawlResult - Raw result + extracted content
///
/// Combines raw spider results with extracted links and text.
/// Created by applying a ContentExtractor to a RawCrawlResult.
///
/// # Examples
///
/// ```
/// use riptide_spider::results::{RawCrawlResult, enrich};
/// use riptide_spider::extractor::BasicExtractor;
/// use url::Url;
/// use http::StatusCode;
///
/// let raw = RawCrawlResult {
///     url: Url::parse("https://example.com").unwrap(),
///     html: "<html><a href='/page'>Link</a><p>Text</p></html>".to_string(),
///     status: StatusCode::OK,
///     headers: http::HeaderMap::new(),
///     content_type: Some("text/html".to_string()),
///     content_size: 1024,
///     processing_time: std::time::Duration::from_millis(150),
/// };
///
/// let extractor = BasicExtractor;
/// let enriched = enrich(raw, &extractor);
///
/// assert!(enriched.extracted_urls.len() > 0);
/// assert!(enriched.text_content.is_some());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedCrawlResult {
    /// Raw spider result (no extraction)
    pub raw: RawCrawlResult,

    /// URLs extracted from the content
    pub extracted_urls: Vec<Url>,

    /// Text content extracted from HTML
    pub text_content: Option<String>,

    /// Extraction strategy used
    pub extraction_strategy: String,
}

impl EnrichedCrawlResult {
    /// Get the URL from the raw result
    pub fn url(&self) -> &Url {
        &self.raw.url
    }

    /// Get the HTML from the raw result
    pub fn html(&self) -> &str {
        &self.raw.html
    }

    /// Get the status code
    pub fn status(&self) -> StatusCode {
        self.raw.status
    }

    /// Check if extraction found any links
    pub fn has_links(&self) -> bool {
        !self.extracted_urls.is_empty()
    }

    /// Check if extraction found any text
    pub fn has_text(&self) -> bool {
        self.text_content.is_some()
    }
}

/// Enrich a raw result by applying an extractor
///
/// Takes a `RawCrawlResult` and applies a `ContentExtractor` to produce
/// an `EnrichedCrawlResult` with extracted links and text.
///
/// # Arguments
/// * `raw` - Raw spider result (no extraction)
/// * `extractor` - Content extractor implementation
///
/// # Returns
/// Enriched result with extracted content
///
/// # Examples
///
/// ```
/// use riptide_spider::results::{RawCrawlResult, enrich};
/// use riptide_spider::extractor::{BasicExtractor, NoOpExtractor};
///
/// let raw: RawCrawlResult = /* ... */;
///
/// // With extraction
/// let enriched = enrich(raw.clone(), &BasicExtractor);
/// assert!(enriched.has_links());
///
/// // Without extraction (spider-only)
/// let noop = enrich(raw, &NoOpExtractor);
/// assert!(!noop.has_links());
/// ```
pub fn enrich(raw: RawCrawlResult, extractor: &dyn ContentExtractor) -> EnrichedCrawlResult {
    let extracted_urls = extractor.extract_links(&raw.html, &raw.url);
    let text_content = extractor.extract_text(&raw.html);
    let extraction_strategy = extractor.strategy_name().to_string();

    EnrichedCrawlResult {
        raw,
        extracted_urls,
        text_content,
        extraction_strategy,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extractor::{BasicExtractor, NoOpExtractor};

    fn create_test_raw_result() -> RawCrawlResult {
        RawCrawlResult {
            url: Url::parse("https://example.com").unwrap(),
            html: r#"<html><a href="/page1">Link 1</a><p>Hello world</p></html>"#.to_string(),
            status: StatusCode::OK,
            headers: HeaderMap::new(),
            content_type: Some("text/html".to_string()),
            content_size: 100,
            processing_time: Duration::from_millis(50),
        }
    }

    #[test]
    fn test_raw_result_creation() {
        let result = create_test_raw_result();

        assert_eq!(result.url.as_str(), "https://example.com/");
        assert!(result.is_success());
        assert!(result.is_html());
        assert_eq!(result.content_size, 100);
    }

    #[test]
    fn test_enrich_with_basic_extractor() {
        let raw = create_test_raw_result();
        let extractor = BasicExtractor;

        let enriched = enrich(raw, &extractor);

        // Should have extracted links
        assert!(enriched.has_links());
        assert_eq!(enriched.extracted_urls.len(), 1);
        assert_eq!(
            enriched.extracted_urls[0].as_str(),
            "https://example.com/page1"
        );

        // Should have extracted text
        assert!(enriched.has_text());
        let text = enriched.text_content.as_ref().unwrap();
        assert!(text.contains("Hello world"));

        // Strategy name
        assert_eq!(enriched.extraction_strategy, "basic");
    }

    #[test]
    fn test_enrich_with_noop_extractor() {
        let raw = create_test_raw_result();
        let extractor = NoOpExtractor;

        let enriched = enrich(raw, &extractor);

        // Should NOT have extracted anything
        assert!(!enriched.has_links());
        assert_eq!(enriched.extracted_urls.len(), 0);

        assert!(!enriched.has_text());
        assert!(enriched.text_content.is_none());

        // Strategy name
        assert_eq!(enriched.extraction_strategy, "noop");
    }

    #[test]
    fn test_enriched_accessors() {
        let raw = create_test_raw_result();
        let enriched = enrich(raw, &BasicExtractor);

        assert_eq!(enriched.url().as_str(), "https://example.com/");
        assert!(enriched.html().contains("<html>"));
        assert_eq!(enriched.status(), StatusCode::OK);
    }
}
