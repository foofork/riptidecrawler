//! Trait definitions for Riptide components
//!
//! This module defines the core traits that enable extensibility
//! and abstraction across the Riptide framework.

use crate::error::{Result, RiptideError};
use crate::types::{BrowserConfig, ExtractionRequest, ExtractionResult, ScrapedContent, Url};
use async_trait::async_trait;
use std::future::Future;
use std::time::Duration;

/// Trait for browser implementations
///
/// This trait abstracts over different browser backends (e.g., Chromium, WebKit)
/// allowing for pluggable browser implementations.
///
/// ## Hard Timeouts
///
/// All browser operations respect the `timeout_ms` configured in `BrowserConfig`.
/// If an operation exceeds the timeout (default: 3000ms/3s), it will:
/// - Return a `RiptideError::Timeout` error
/// - Allow fallback to static content retrieval
///
/// Implementors should wrap operations with `with_timeout()` helper.
#[async_trait]
pub trait Browser: Send + Sync {
    /// Initialize the browser with the given configuration
    async fn initialize(&mut self, config: BrowserConfig) -> Result<()>;

    /// Navigate to a URL and wait for page load
    ///
    /// Respects configured timeout. On timeout, returns `RiptideError::Timeout`.
    async fn navigate(&mut self, url: &Url) -> Result<()>;

    /// Get the current page HTML
    ///
    /// Respects configured timeout. On timeout, returns `RiptideError::Timeout`.
    async fn get_html(&self) -> Result<String>;

    /// Execute JavaScript in the page context
    ///
    /// Respects configured timeout. On timeout, returns `RiptideError::Timeout`.
    async fn execute_script(&mut self, script: &str) -> Result<serde_json::Value>;

    /// Take a screenshot of the current page
    async fn screenshot(&self) -> Result<Vec<u8>>;

    /// Close the browser instance
    async fn close(&mut self) -> Result<()>;

    /// Check if the browser is still active
    fn is_active(&self) -> bool;

    /// Get the configured timeout duration
    fn timeout(&self) -> Duration {
        Duration::from_millis(3000) // Default 3s
    }
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

/// Helper function to enforce timeout on async operations
///
/// Wraps an async operation with a timeout. Returns `RiptideError::Timeout`
/// if the operation exceeds the specified duration.
///
/// # Example
///
/// ```ignore
/// let result = with_timeout(
///     Duration::from_millis(3000),
///     expensive_operation()
/// ).await?;
/// ```
pub async fn with_timeout<F, T>(timeout: Duration, future: F) -> Result<T>
where
    F: Future<Output = Result<T>>,
{
    match tokio::time::timeout(timeout, future).await {
        Ok(result) => result,
        Err(_) => Err(RiptideError::Timeout(timeout.as_millis() as u64)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    // Mock implementation for testing
    struct MockBrowser {
        active: bool,
        timeout: Duration,
    }

    #[async_trait]
    impl Browser for MockBrowser {
        async fn initialize(&mut self, config: BrowserConfig) -> Result<()> {
            self.active = true;
            self.timeout = Duration::from_millis(config.timeout_ms);
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

        fn timeout(&self) -> Duration {
            self.timeout
        }
    }

    // Mock browser that simulates slow operations
    struct SlowBrowser {
        active: bool,
        timeout: Duration,
        delay_ms: u64,
    }

    #[async_trait]
    impl Browser for SlowBrowser {
        async fn initialize(&mut self, config: BrowserConfig) -> Result<()> {
            self.active = true;
            self.timeout = Duration::from_millis(config.timeout_ms);
            Ok(())
        }

        async fn navigate(&mut self, _url: &Url) -> Result<()> {
            with_timeout(self.timeout, async {
                sleep(Duration::from_millis(self.delay_ms)).await;
                Ok(())
            })
            .await
        }

        async fn get_html(&self) -> Result<String> {
            with_timeout(self.timeout, async {
                sleep(Duration::from_millis(self.delay_ms)).await;
                Ok("<html><body>Slow response</body></html>".to_string())
            })
            .await
        }

        async fn execute_script(&mut self, _script: &str) -> Result<serde_json::Value> {
            with_timeout(self.timeout, async {
                sleep(Duration::from_millis(self.delay_ms)).await;
                Ok(serde_json::json!({"result": "success"}))
            })
            .await
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

        fn timeout(&self) -> Duration {
            self.timeout
        }
    }

    #[tokio::test]
    async fn test_mock_browser() {
        let mut browser = MockBrowser {
            active: false,
            timeout: Duration::from_millis(3000),
        };
        assert!(!browser.is_active());

        browser.initialize(BrowserConfig::default()).await.unwrap();
        assert!(browser.is_active());

        browser.close().await.unwrap();
        assert!(!browser.is_active());
    }

    #[tokio::test]
    async fn test_browser_config_default_timeout() {
        let config = BrowserConfig::default();
        assert_eq!(
            config.timeout_ms, 3000,
            "Default timeout should be 3 seconds"
        );
    }

    #[tokio::test]
    async fn test_browser_timeout_configured() {
        let mut browser = MockBrowser {
            active: false,
            timeout: Duration::from_millis(5000),
        };

        let config = BrowserConfig {
            timeout_ms: 5000,
            ..Default::default()
        };

        browser.initialize(config).await.unwrap();
        assert_eq!(browser.timeout().as_millis(), 5000);
    }

    #[tokio::test]
    async fn test_with_timeout_success() {
        let result = with_timeout(Duration::from_millis(100), async {
            sleep(Duration::from_millis(10)).await;
            Ok::<String, RiptideError>("success".to_string())
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[tokio::test]
    async fn test_with_timeout_expires() {
        let result = with_timeout(Duration::from_millis(50), async {
            sleep(Duration::from_millis(200)).await;
            Ok::<String, RiptideError>("should not reach".to_string())
        })
        .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            RiptideError::Timeout(ms) => assert_eq!(ms, 50),
            _ => panic!("Expected Timeout error"),
        }
    }

    #[tokio::test]
    async fn test_slow_navigate_timeout() {
        let config = BrowserConfig {
            timeout_ms: 100,
            ..Default::default()
        };

        let mut browser = SlowBrowser {
            active: false,
            timeout: Duration::from_millis(100),
            delay_ms: 500, // Simulate slow navigation
        };

        browser.initialize(config).await.unwrap();

        let url = Url::parse("https://example.com").unwrap();
        let result = browser.navigate(&url).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            RiptideError::Timeout(ms) => assert_eq!(ms, 100),
            _ => panic!("Expected Timeout error"),
        }
    }

    #[tokio::test]
    async fn test_slow_get_html_timeout() {
        let config = BrowserConfig {
            timeout_ms: 100,
            ..Default::default()
        };

        let mut browser = SlowBrowser {
            active: false,
            timeout: Duration::from_millis(100),
            delay_ms: 500,
        };

        browser.initialize(config).await.unwrap();

        let result = browser.get_html().await;

        assert!(result.is_err());
        match result.unwrap_err() {
            RiptideError::Timeout(ms) => assert_eq!(ms, 100),
            _ => panic!("Expected Timeout error"),
        }
    }

    #[tokio::test]
    async fn test_slow_execute_script_timeout() {
        let config = BrowserConfig {
            timeout_ms: 100,
            ..Default::default()
        };

        let mut browser = SlowBrowser {
            active: false,
            timeout: Duration::from_millis(100),
            delay_ms: 500,
        };

        browser.initialize(config).await.unwrap();

        let result = browser.execute_script("console.log('test')").await;

        assert!(result.is_err());
        match result.unwrap_err() {
            RiptideError::Timeout(ms) => assert_eq!(ms, 100),
            _ => panic!("Expected Timeout error"),
        }
    }

    #[tokio::test]
    async fn test_fast_operations_succeed() {
        let config = BrowserConfig {
            timeout_ms: 1000,
            ..Default::default()
        };

        let mut browser = SlowBrowser {
            active: false,
            timeout: Duration::from_millis(1000),
            delay_ms: 50, // Fast enough
        };

        browser.initialize(config).await.unwrap();

        let url = Url::parse("https://example.com").unwrap();
        assert!(browser.navigate(&url).await.is_ok());
        assert!(browser.get_html().await.is_ok());
        assert!(browser.execute_script("test").await.is_ok());
    }

    #[tokio::test]
    async fn test_timeout_error_is_retryable() {
        let error = RiptideError::Timeout(3000);
        assert!(error.is_retryable(), "Timeout errors should be retryable");
    }

    #[tokio::test]
    async fn test_fallback_static_content_pattern() {
        // Simulate timeout scenario with fallback
        let config = BrowserConfig {
            timeout_ms: 100,
            ..Default::default()
        };

        let mut browser = SlowBrowser {
            active: false,
            timeout: Duration::from_millis(100),
            delay_ms: 500,
        };

        browser.initialize(config).await.unwrap();

        // Try browser operation
        let result = browser.get_html().await;

        // On timeout, fallback to static content
        let content = match result {
            Ok(html) => html,
            Err(RiptideError::Timeout(_)) => {
                // Fallback: use static HTTP request instead
                "<html><body>Static fallback content</body></html>".to_string()
            }
            Err(e) => panic!("Unexpected error: {}", e),
        };

        assert!(content.contains("Static fallback"));
    }
}
