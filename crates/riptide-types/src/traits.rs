//! Trait definitions for Riptide components
//!
//! This module defines the core traits that enable extensibility
//! and abstraction across the Riptide framework.

use crate::error::Result;
use crate::types::{BrowserConfig, ExtractionRequest, ExtractionResult, ScrapedContent, Url};
use async_trait::async_trait;

/// Trait for browser implementations
///
/// This trait abstracts over different browser backends (e.g., Chromium, WebKit)
/// allowing for pluggable browser implementations.
#[async_trait]
pub trait Browser: Send + Sync {
    /// Initialize the browser with the given configuration
    async fn initialize(&mut self, config: BrowserConfig) -> Result<()>;

    /// Navigate to a URL and wait for page load
    async fn navigate(&mut self, url: &Url) -> Result<()>;

    /// Get the current page HTML
    async fn get_html(&self) -> Result<String>;

    /// Execute JavaScript in the page context
    async fn execute_script(&mut self, script: &str) -> Result<serde_json::Value>;

    /// Take a screenshot of the current page
    async fn screenshot(&self) -> Result<Vec<u8>>;

    /// Close the browser instance
    async fn close(&mut self) -> Result<()>;

    /// Check if the browser is still active
    fn is_active(&self) -> bool;
}

/// Trait for content extractors
///
/// Extractors parse HTML/DOM content and extract structured data
/// according to specified rules and selectors.
#[async_trait]
pub trait Extractor: Send + Sync {
    /// Extract content from HTML according to the request configuration
    async fn extract(&self, html: &str, request: &ExtractionRequest) -> Result<ScrapedContent>;

    /// Validate that the extractor can handle the given request
    fn can_handle(&self, request: &ExtractionRequest) -> bool;

    /// Get the name/type of this extractor
    fn name(&self) -> &str;
}

/// Trait for complete scraping implementations
///
/// A scraper combines browser automation and content extraction
/// to provide end-to-end scraping functionality.
#[async_trait]
pub trait Scraper: Send + Sync {
    /// Perform a complete scraping operation for the given request
    async fn scrape(&mut self, request: ExtractionRequest) -> Result<ExtractionResult>;

    /// Check if the scraper is ready to accept requests
    fn is_ready(&self) -> bool;

    /// Get scraper health status
    async fn health_check(&self) -> Result<bool>;
}

/// Trait for caching implementations
#[async_trait]
pub trait Cache: Send + Sync {
    /// Store content in cache
    async fn set(&self, key: &str, value: &[u8], ttl_seconds: u64) -> Result<()>;

    /// Retrieve content from cache
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;

    /// Remove content from cache
    async fn delete(&self, key: &str) -> Result<()>;

    /// Check if key exists in cache
    async fn exists(&self, key: &str) -> Result<bool>;

    /// Clear all cache entries
    async fn clear(&self) -> Result<()>;
}

/// Trait for storage backends
#[async_trait]
pub trait Storage: Send + Sync {
    /// Store extraction result
    async fn store_result(&self, result: &ExtractionResult) -> Result<()>;

    /// Retrieve extraction result by ID
    async fn get_result(&self, request_id: &uuid::Uuid) -> Result<Option<ExtractionResult>>;

    /// List results with pagination
    async fn list_results(&self, limit: usize, offset: usize) -> Result<Vec<ExtractionResult>>;

    /// Delete result by ID
    async fn delete_result(&self, request_id: &uuid::Uuid) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock implementation for testing
    struct MockBrowser {
        active: bool,
    }

    #[async_trait]
    impl Browser for MockBrowser {
        async fn initialize(&mut self, _config: BrowserConfig) -> Result<()> {
            self.active = true;
            Ok(())
        }

        async fn navigate(&mut self, _url: &Url) -> Result<()> {
            Ok(())
        }

        async fn get_html(&self) -> Result<String> {
            Ok("<html><body>Mock</body></html>".to_string())
        }

        async fn execute_script(&mut self, _script: &str) -> Result<serde_json::Value> {
            Ok(serde_json::json!({}))
        }

        async fn screenshot(&self) -> Result<Vec<u8>> {
            Ok(vec![])
        }

        async fn close(&mut self) -> Result<()> {
            self.active = false;
            Ok(())
        }

        fn is_active(&self) -> bool {
            self.active
        }
    }

    #[tokio::test]
    async fn test_mock_browser() {
        let mut browser = MockBrowser { active: false };
        assert!(!browser.is_active());

        browser.initialize(BrowserConfig::default()).await.unwrap();
        assert!(browser.is_active());

        browser.close().await.unwrap();
        assert!(!browser.is_active());
    }
}
