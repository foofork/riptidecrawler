//! Mock implementations for testing
//!
//! Provides mock Spider and Extractor implementations for unit testing.

use async_trait::async_trait;
use futures::stream::{self, BoxStream};
use url::Url;

use crate::dto::Document;
use crate::error::{RiptideError, RiptideResult};
use crate::traits::{Content, ExtractOpts, Extractor, Spider, SpiderOpts};

/// Mock spider for testing
///
/// Returns a predetermined list of URLs for testing composition.
pub struct MockSpider {
    urls: Vec<String>,
}

impl MockSpider {
    /// Create a new mock spider with the given URLs
    pub fn new(urls: Vec<String>) -> Self {
        Self { urls }
    }

    /// Create a mock spider with default test URLs
    pub fn with_test_urls() -> Self {
        Self::new(vec![
            "https://example.com/page1".to_string(),
            "https://example.com/page2".to_string(),
            "https://example.com/page3".to_string(),
        ])
    }
}

#[async_trait]
impl Spider for MockSpider {
    async fn crawl(
        &self,
        _url: &str,
        _opts: SpiderOpts,
    ) -> RiptideResult<BoxStream<'static, RiptideResult<Url>>> {
        let urls: Vec<RiptideResult<Url>> = self
            .urls
            .iter()
            .map(|url_str| {
                Url::parse(url_str).map_err(|e| RiptideError::UrlParseError {
                    url: url_str.clone(),
                    source: e,
                })
            })
            .collect();

        Ok(Box::pin(stream::iter(urls)))
    }
}

/// Mock extractor for testing
///
/// Returns documents with predictable content based on the URL.
pub struct MockExtractor {
    should_fail: bool,
}

impl MockExtractor {
    /// Create a new mock extractor
    pub fn new() -> Self {
        Self { should_fail: false }
    }

    /// Create a mock extractor that fails on extraction
    pub fn with_failures() -> Self {
        Self { should_fail: true }
    }
}

impl Default for MockExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Extractor for MockExtractor {
    async fn extract(&self, content: Content, _opts: ExtractOpts) -> RiptideResult<Document> {
        if self.should_fail {
            return Err(RiptideError::ExtractionError {
                message: "Mock extraction failure".to_string(),
            });
        }

        let url = match content {
            Content::Url(url) => url,
            Content::Html(_) => "https://example.com/html".to_string(),
            Content::Text(_) => "https://example.com/text".to_string(),
        };

        Ok(Document::new(
            url.clone(),
            format!("Title for {}", url),
            format!("Extracted content from {}", url),
        ))
    }
}

/// Mock spider that fails during crawling
pub struct FailingMockSpider;

#[async_trait]
impl Spider for FailingMockSpider {
    async fn crawl(
        &self,
        _url: &str,
        _opts: SpiderOpts,
    ) -> RiptideResult<BoxStream<'static, RiptideResult<Url>>> {
        Err(RiptideError::FetchError {
            url: "https://example.com".to_string(),
            message: "Mock spider failure".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;

    #[tokio::test]
    async fn test_mock_spider() {
        let spider = MockSpider::with_test_urls();
        let stream = spider
            .crawl("https://example.com", SpiderOpts::default())
            .await
            .unwrap();

        let urls: Vec<_> = stream.collect().await;
        assert_eq!(urls.len(), 3);
    }

    #[tokio::test]
    async fn test_mock_extractor() {
        let extractor = MockExtractor::new();
        let doc = extractor
            .extract(
                Content::Url("https://example.com".to_string()),
                ExtractOpts::default(),
            )
            .await
            .unwrap();

        assert!(doc.title.contains("example.com"));
        assert!(doc.content.contains("Extracted content"));
    }

    #[tokio::test]
    async fn test_failing_extractor() {
        let extractor = MockExtractor::with_failures();
        let result = extractor
            .extract(
                Content::Url("https://example.com".to_string()),
                ExtractOpts::default(),
            )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_failing_spider() {
        let spider = FailingMockSpider;
        let result = spider
            .crawl("https://example.com", SpiderOpts::default())
            .await;

        assert!(result.is_err());
    }
}
