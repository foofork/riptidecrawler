# WebScraping Port Trait Design

**Port Trait**: `WebScraping`
**Facade**: ScraperFacade
**Action**: Create abstraction
**Location**: `/workspaces/riptidecrawler/crates/riptide-types/src/ports/scraping.rs`
**Risk Level**: Low
**Estimated Time**: 3-4 hours

---

## Rationale

The `ScraperFacade` provides web scraping functionality (fetching HTML/bytes from URLs) but doesn't duplicate any existing port trait. We need to create a proper hexagonal port trait to abstract this functionality.

---

## Current ScraperFacade Interface

**File**: `/workspaces/riptidecrawler/crates/riptide-facade/src/facades/scraper.rs`

```rust
pub struct ScraperFacade {
    config: Arc<RiptideConfig>,
    client: Arc<FetchEngine>,
}

impl ScraperFacade {
    pub async fn fetch_html(&self, url: impl AsRef<str>) -> RiptideResult<String>;
    pub async fn fetch_bytes(&self, url: impl AsRef<str>) -> RiptideResult<Vec<u8>>;
    pub fn config(&self) -> &RiptideConfig;
    pub fn client(&self) -> &FetchEngine;
}
```

**Key Operations**:
1. Fetch HTML content from URL (as String)
2. Fetch raw bytes from URL (as Vec<u8>)
3. URL validation
4. Timeout enforcement
5. Error handling

---

## Port Trait Design

### File: `/workspaces/riptidecrawler/crates/riptide-types/src/ports/scraping.rs`

```rust
//! Web Scraping Port - Hexagonal abstraction for fetching web content
//!
//! This port trait provides a backend-agnostic interface for web scraping operations,
//! enabling dependency inversion and facilitating testing with mock implementations.
//!
//! # Architecture
//!
//! ```text
//! Domain Layer (riptide-types)
//!     ↓ defines WebScraping trait
//! Application Layer (riptide-facade)
//!     ↓ adapts ScraperFacade
//! Composition Root (riptide-api)
//!     ↓ wires Arc<dyn WebScraping>
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use riptide_types::ports::{WebScraping, ScrapeOptions};
//!
//! async fn scrape(scraper: &dyn WebScraping, url: &str) -> Result<String> {
//!     let options = ScrapeOptions::default();
//!     let page = scraper.scrape_url(url, options).await?;
//!     Ok(page.html)
//! }
//! ```

use async_trait::async_trait;
use crate::error::Result as RiptideResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Web scraping port trait
///
/// Defines the interface for fetching and processing web content.
/// Implementations handle HTTP requests, retries, timeouts, and error handling.
#[async_trait]
pub trait WebScraping: Send + Sync {
    /// Fetch HTML content from a URL
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to fetch
    /// * `options` - Scraping options (timeout, headers, etc.)
    ///
    /// # Returns
    ///
    /// * `Ok(ScrapedPage)` - Successfully fetched page with metadata
    /// * `Err(_)` - Network error, timeout, or invalid URL
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let options = ScrapeOptions::default();
    /// let page = scraper.scrape_url("https://example.com", options).await?;
    /// println!("Title: {}", page.title.unwrap_or_default());
    /// ```
    async fn scrape_url(&self, url: &str, options: ScrapeOptions) -> RiptideResult<ScrapedPage>;

    /// Fetch raw bytes from a URL
    ///
    /// Useful for downloading binary content (images, PDFs, etc.)
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to fetch
    /// * `options` - Scraping options
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<u8>)` - Raw response bytes
    /// * `Err(_)` - Network error, timeout, or invalid URL
    async fn fetch_bytes(&self, url: &str, options: ScrapeOptions) -> RiptideResult<Vec<u8>>;

    /// Scrape multiple URLs in batch
    ///
    /// Efficiently fetches multiple URLs with concurrency control.
    ///
    /// # Arguments
    ///
    /// * `urls` - List of URLs to scrape
    /// * `options` - Scraping options applied to all URLs
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<ScrapedPage>)` - Results for all URLs (includes errors)
    /// * `Err(_)` - Fatal error that prevented batch processing
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let urls = vec!["https://example.com", "https://example.org"];
    /// let pages = scraper.scrape_batch(urls, ScrapeOptions::default()).await?;
    /// for page in pages {
    ///     println!("{}: {} bytes", page.url, page.html.len());
    /// }
    /// ```
    async fn scrape_batch(
        &self,
        urls: Vec<String>,
        options: ScrapeOptions,
    ) -> RiptideResult<Vec<ScrapedPage>>;

    /// Extract data using CSS selectors
    ///
    /// Applies CSS selectors to HTML content and extracts matching elements.
    ///
    /// # Arguments
    ///
    /// * `html` - The HTML content to parse
    /// * `selectors` - Map of selector names to CSS selector strings
    ///
    /// # Returns
    ///
    /// * `Ok(ExtractedData)` - Extracted data keyed by selector name
    /// * `Err(_)` - Parse error or selector error
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut selectors = SelectorSet::new();
    /// selectors.insert("title".to_string(), "h1".to_string());
    /// selectors.insert("links".to_string(), "a[href]".to_string());
    ///
    /// let data = scraper.extract_with_selectors(&html, selectors).await?;
    /// println!("Title: {:?}", data.get("title"));
    /// ```
    async fn extract_with_selectors(
        &self,
        html: &str,
        selectors: SelectorSet,
    ) -> RiptideResult<ExtractedData>;

    /// Check if scraper is available/healthy
    ///
    /// # Returns
    ///
    /// `true` if scraper is ready to process requests
    async fn is_available(&self) -> bool;
}

/// Options for scraping operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapeOptions {
    /// Request timeout
    #[serde(default = "default_timeout")]
    pub timeout: Duration,

    /// Custom HTTP headers
    #[serde(default)]
    pub headers: HashMap<String, String>,

    /// Follow redirects
    #[serde(default = "default_true")]
    pub follow_redirects: bool,

    /// Maximum redirects to follow
    #[serde(default = "default_max_redirects")]
    pub max_redirects: usize,

    /// User agent string
    #[serde(default = "default_user_agent")]
    pub user_agent: String,

    /// Retry on failure
    #[serde(default = "default_true")]
    pub retry_enabled: bool,

    /// Maximum retry attempts
    #[serde(default = "default_max_retries")]
    pub max_retries: usize,
}

impl Default for ScrapeOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            headers: HashMap::new(),
            follow_redirects: true,
            max_redirects: 10,
            user_agent: "Riptide/1.0".to_string(),
            retry_enabled: true,
            max_retries: 3,
        }
    }
}

/// Result of scraping a URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapedPage {
    /// Source URL
    pub url: String,

    /// HTML content
    pub html: String,

    /// HTTP status code
    pub status_code: u16,

    /// Response headers
    pub headers: HashMap<String, String>,

    /// Content type
    pub content_type: Option<String>,

    /// Final URL after redirects
    pub final_url: Option<String>,

    /// Page title (if parsed)
    pub title: Option<String>,

    /// Fetch duration in milliseconds
    pub fetch_duration_ms: u64,
}

/// CSS selector set for extraction
pub type SelectorSet = HashMap<String, String>;

/// Extracted data from selectors
pub type ExtractedData = HashMap<String, Vec<String>>;

// Default value functions for serde
fn default_timeout() -> Duration {
    Duration::from_secs(30)
}

fn default_true() -> bool {
    true
}

fn default_max_redirects() -> usize {
    10
}

fn default_user_agent() -> String {
    "Riptide/1.0".to_string()
}

fn default_max_retries() -> usize {
    3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scrape_options_default() {
        let opts = ScrapeOptions::default();
        assert_eq!(opts.timeout, Duration::from_secs(30));
        assert!(opts.follow_redirects);
        assert_eq!(opts.max_redirects, 10);
        assert_eq!(opts.user_agent, "Riptide/1.0");
    }

    #[test]
    fn test_scrape_options_serialization() {
        let opts = ScrapeOptions::default();
        let json = serde_json::to_string(&opts).unwrap();
        let deserialized: ScrapeOptions = serde_json::from_str(&json).unwrap();
        assert_eq!(opts.timeout, deserialized.timeout);
    }

    #[test]
    fn test_scraped_page_creation() {
        let page = ScrapedPage {
            url: "https://example.com".to_string(),
            html: "<html></html>".to_string(),
            status_code: 200,
            headers: HashMap::new(),
            content_type: Some("text/html".to_string()),
            final_url: None,
            title: Some("Example".to_string()),
            fetch_duration_ms: 150,
        };

        assert_eq!(page.status_code, 200);
        assert_eq!(page.title, Some("Example".to_string()));
    }
}
```

---

## Integration with ApplicationContext

### Update context.rs

**File**: `/workspaces/riptidecrawler/crates/riptide-api/src/context.rs`

```diff
pub struct ApplicationContext {
    // ... other fields ...

-   /// ScraperFacade for web scraping operations
-   pub scraper_facade: Arc<riptide_facade::facades::ScraperFacade>,
+   /// Web scraping provider for fetching content
+   pub scraper: Arc<dyn WebScraping>,

    // ... other fields ...
}
```

---

## Benefits of Port Trait

1. ✅ **Testability** - Easy to create mock implementations
2. ✅ **Swappability** - Can swap scraping backends without changing API
3. ✅ **Hexagonal compliance** - Proper domain abstraction
4. ✅ **Type safety** - Strongly typed options and results
5. ✅ **Documentation** - Clear contract for implementations

---

## Mock Implementation for Testing

```rust
// File: crates/riptide-types/src/ports/scraping/mock.rs

#[cfg(test)]
pub struct MockWebScraper {
    responses: HashMap<String, ScrapedPage>,
}

#[cfg(test)]
impl MockWebScraper {
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
        }
    }

    pub fn with_response(mut self, url: &str, html: &str) -> Self {
        self.responses.insert(
            url.to_string(),
            ScrapedPage {
                url: url.to_string(),
                html: html.to_string(),
                status_code: 200,
                headers: HashMap::new(),
                content_type: Some("text/html".to_string()),
                final_url: None,
                title: None,
                fetch_duration_ms: 10,
            },
        );
        self
    }
}

#[cfg(test)]
#[async_trait]
impl WebScraping for MockWebScraper {
    async fn scrape_url(&self, url: &str, _options: ScrapeOptions) -> RiptideResult<ScrapedPage> {
        self.responses
            .get(url)
            .cloned()
            .ok_or_else(|| RiptideError::network(format!("URL not found: {}", url)))
    }

    async fn fetch_bytes(&self, url: &str, _options: ScrapeOptions) -> RiptideResult<Vec<u8>> {
        let page = self.scrape_url(url, ScrapeOptions::default()).await?;
        Ok(page.html.into_bytes())
    }

    async fn scrape_batch(
        &self,
        urls: Vec<String>,
        options: ScrapeOptions,
    ) -> RiptideResult<Vec<ScrapedPage>> {
        let mut results = Vec::new();
        for url in urls {
            if let Ok(page) = self.scrape_url(&url, options.clone()).await {
                results.push(page);
            }
        }
        Ok(results)
    }

    async fn extract_with_selectors(
        &self,
        _html: &str,
        _selectors: SelectorSet,
    ) -> RiptideResult<ExtractedData> {
        Ok(HashMap::new())
    }

    async fn is_available(&self) -> bool {
        true
    }
}
```

---

## Next Steps

1. Create the port trait file
2. Implement adapter (see `06-scraper-facade-adapter.md`)
3. Update ApplicationContext
4. Update all call sites
5. Run tests

---

**Status**: ✅ Design Complete - Ready for Implementation
**Dependencies**: None
**Blockers**: None
