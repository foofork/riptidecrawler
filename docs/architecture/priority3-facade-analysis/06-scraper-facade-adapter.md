# ScraperFacade Adapter Implementation

**Adapter**: ScraperFacadeAdapter
**Implements**: `WebScraping` trait
**Wraps**: `ScraperFacade`
**Location**: `/workspaces/riptidecrawler/crates/riptide-facade/src/adapters/scraper_adapter.rs`
**Risk Level**: Low
**Estimated Time**: 2-3 hours

---

## Purpose

Create an adapter that implements the `WebScraping` port trait by wrapping the existing `ScraperFacade`. This allows ApplicationContext to use `Arc<dyn WebScraping>` instead of the concrete `ScraperFacade` type.

---

## Implementation

### File: `/workspaces/riptidecrawler/crates/riptide-facade/src/adapters/scraper_adapter.rs`

```rust
//! ScraperFacade Adapter - Implements WebScraping trait
//!
//! This adapter wraps ScraperFacade to implement the WebScraping port trait,
//! enabling dependency inversion and hexagonal architecture compliance.

use crate::facades::ScraperFacade;
use async_trait::async_trait;
use riptide_types::error::Result as RiptideResult;
use riptide_types::ports::{ExtractedData, ScrapedPage, ScrapeOptions, SelectorSet, WebScraping};
use std::sync::Arc;
use std::time::Instant;

/// Adapter that implements WebScraping using ScraperFacade
pub struct ScraperFacadeAdapter {
    facade: Arc<ScraperFacade>,
}

impl ScraperFacadeAdapter {
    /// Create a new adapter wrapping a ScraperFacade
    pub fn new(facade: ScraperFacade) -> Arc<Self> {
        Arc::new(Self {
            facade: Arc::new(facade),
        })
    }

    /// Create from an existing Arc<ScraperFacade>
    pub fn from_arc(facade: Arc<ScraperFacade>) -> Arc<Self> {
        Arc::new(Self { facade })
    }

    /// Get reference to inner facade (for testing/debugging)
    pub fn inner(&self) -> &ScraperFacade {
        &self.facade
    }
}

#[async_trait]
impl WebScraping for ScraperFacadeAdapter {
    async fn scrape_url(&self, url: &str, options: ScrapeOptions) -> RiptideResult<ScrapedPage> {
        let start = Instant::now();

        // Fetch HTML using facade
        let html = self.facade.fetch_html(url).await?;

        let duration_ms = start.elapsed().as_millis() as u64;

        // Build ScrapedPage from result
        Ok(ScrapedPage {
            url: url.to_string(),
            html,
            status_code: 200, // ScraperFacade doesn't expose status code
            headers: Default::default(), // ScraperFacade doesn't expose headers
            content_type: Some("text/html".to_string()),
            final_url: None,
            title: None, // Will be extracted if needed
            fetch_duration_ms: duration_ms,
        })
    }

    async fn fetch_bytes(&self, url: &str, _options: ScrapeOptions) -> RiptideResult<Vec<u8>> {
        // Delegate directly to facade
        self.facade.fetch_bytes(url).await
    }

    async fn scrape_batch(
        &self,
        urls: Vec<String>,
        options: ScrapeOptions,
    ) -> RiptideResult<Vec<ScrapedPage>> {
        // Simple sequential implementation
        // Could be optimized with concurrent fetching
        let mut results = Vec::with_capacity(urls.len());

        for url in urls {
            match self.scrape_url(&url, options.clone()).await {
                Ok(page) => results.push(page),
                Err(e) => {
                    tracing::warn!("Failed to scrape {}: {}", url, e);
                    // Create error page
                    results.push(ScrapedPage {
                        url: url.clone(),
                        html: String::new(),
                        status_code: 0,
                        headers: Default::default(),
                        content_type: None,
                        final_url: None,
                        title: None,
                        fetch_duration_ms: 0,
                    });
                }
            }
        }

        Ok(results)
    }

    async fn extract_with_selectors(
        &self,
        html: &str,
        selectors: SelectorSet,
    ) -> RiptideResult<ExtractedData> {
        use scraper::{Html, Selector};

        let document = Html::parse_document(html);
        let mut extracted = ExtractedData::new();

        for (name, selector_str) in selectors {
            match Selector::parse(&selector_str) {
                Ok(selector) => {
                    let values: Vec<String> = document
                        .select(&selector)
                        .map(|el| el.text().collect::<String>())
                        .collect();

                    extracted.insert(name, values);
                }
                Err(e) => {
                    tracing::warn!("Invalid selector '{}': {}", selector_str, e);
                    extracted.insert(name, Vec::new());
                }
            }
        }

        Ok(extracted)
    }

    async fn is_available(&self) -> bool {
        // ScraperFacade is always available if constructed
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RiptideConfig;

    #[tokio::test]
    async fn test_adapter_creation() {
        let config = RiptideConfig::default();
        let facade = ScraperFacade::new(config).await.expect("Failed to create facade");
        let adapter = ScraperFacadeAdapter::new(facade);

        assert!(adapter.is_available().await);
    }

    #[tokio::test]
    async fn test_adapter_implements_trait() {
        let config = RiptideConfig::default();
        let facade = ScraperFacade::new(config).await.expect("Failed to create facade");
        let adapter = ScraperFacadeAdapter::new(facade);

        // Can be used as Arc<dyn WebScraping>
        let _scraper: Arc<dyn WebScraping> = adapter;
    }

    #[tokio::test]
    async fn test_extract_with_selectors() {
        let config = RiptideConfig::default();
        let facade = ScraperFacade::new(config).await.expect("Failed to create facade");
        let adapter = ScraperFacadeAdapter::new(facade);

        let html = r#"
            <html>
                <head><title>Test</title></head>
                <body>
                    <h1>Hello World</h1>
                    <a href="http://example.com">Link</a>
                </body>
            </html>
        "#;

        let mut selectors = SelectorSet::new();
        selectors.insert("title".to_string(), "h1".to_string());
        selectors.insert("links".to_string(), "a".to_string());

        let extracted = adapter.extract_with_selectors(html, selectors).await.unwrap();

        assert!(extracted.contains_key("title"));
        assert!(extracted.contains_key("links"));
        assert_eq!(extracted.get("title").unwrap().len(), 1);
    }
}
```

---

## Module Registration

### File: `/workspaces/riptidecrawler/crates/riptide-facade/src/adapters/mod.rs`

```rust
//! Adapters for implementing port traits

pub mod scraper_adapter;
pub mod search_adapter;
pub mod engine_adapter;

pub use scraper_adapter::ScraperFacadeAdapter;
pub use search_adapter::SearchFacadeAdapter;
pub use engine_adapter::EngineFacadeAdapter;
```

Or create the file if it doesn't exist:

```rust
//! Facade adapters implementing port traits
//!
//! This module contains adapters that wrap facades to implement
//! hexagonal port traits from riptide-types.

pub mod scraper_adapter;

pub use scraper_adapter::ScraperFacadeAdapter;
```

---

## Usage in ApplicationContext

### Update: `/workspaces/riptidecrawler/crates/riptide-api/src/context.rs`

```rust
use riptide_facade::adapters::ScraperFacadeAdapter;
use riptide_types::ports::WebScraping;

// In ApplicationContext::new():
let scraper_facade = ScraperFacade::new(config.clone()).await?;
let scraper: Arc<dyn WebScraping> = ScraperFacadeAdapter::new(scraper_facade);

// In struct:
pub struct ApplicationContext {
    // ... other fields ...
    pub scraper: Arc<dyn WebScraping>,
    // ... other fields ...
}
```

---

## Testing Strategy

### Unit Tests

Test the adapter in isolation:

```rust
#[cfg(test)]
mod adapter_tests {
    use super::*;

    #[tokio::test]
    async fn test_scrape_url_success() {
        // This would need a mock HTTP server or mock facade
        // For now, test adapter creation and trait compliance
        let config = RiptideConfig::default();
        let facade = ScraperFacade::new(config).await.unwrap();
        let adapter = ScraperFacadeAdapter::new(facade);

        assert!(adapter.is_available().await);
    }
}
```

### Integration Tests

Test through ApplicationContext:

```rust
#[tokio::test]
async fn test_context_uses_web_scraping_trait() {
    let context = ApplicationContext::new_for_test().await.unwrap();

    // Verify trait usage
    let scraper: &dyn WebScraping = &*context.scraper;
    assert!(scraper.is_available().await);
}
```

---

## Migration Checklist

- [ ] Create `/workspaces/riptidecrawler/crates/riptide-facade/src/adapters/` directory
- [ ] Create `scraper_adapter.rs` with adapter implementation
- [ ] Create or update `mod.rs` to export adapter
- [ ] Add `scraper` dependency to `riptide-facade/Cargo.toml` for CSS selector parsing
- [ ] Update ApplicationContext to use adapter
- [ ] Update all call sites to use trait methods
- [ ] Run tests: `cargo test -p riptide-facade`
- [ ] Run tests: `cargo test -p riptide-api`
- [ ] Verify no regressions: `cargo clippy -p riptide-facade -- -D warnings`

---

## Dependencies

Add to `/workspaces/riptidecrawler/crates/riptide-facade/Cargo.toml`:

```toml
[dependencies]
scraper = "0.17" # For CSS selector parsing in extract_with_selectors
```

---

## Benefits

1. ✅ **Hexagonal compliance** - ApplicationContext uses trait, not concrete type
2. ✅ **Testability** - Can inject mock implementations
3. ✅ **Swappability** - Can replace facade with different implementation
4. ✅ **Zero breaking changes** - Existing facade continues to work

---

**Status**: ✅ Design Complete - Ready for Implementation
**Time Estimate**: 2-3 hours
**Complexity**: Low
**Blockers**: Requires `WebScraping` trait to be created first
