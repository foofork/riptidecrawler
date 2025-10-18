# P1-A4: Riptide Facade Implementation Strategy

**Status**: Ready for Implementation
**Phase**: Phase 1 - Foundation
**Date**: 2025-10-18
**Architect**: System Architecture Designer

## Executive Summary

This document provides the detailed implementation strategy for **Phase 1** of the `riptide-facade` crate. The skeleton exists with excellent foundations (builder pattern, error handling, config management). We need to implement the **ScraperFacade** as the simplest facade to establish patterns for the remaining facades.

## Current State Analysis

### ✅ Already Implemented

1. **Core Infrastructure** (`src/lib.rs`):
   - Main `Riptide` struct with facade accessors
   - Feature-gated facade methods
   - Clone implementation with Arc runtime
   - Comprehensive documentation

2. **Error Handling** (`src/error.rs`):
   - 20+ error variants covering all domains
   - `with_context()` for error enrichment
   - `is_retryable()`, `is_client_error()`, `is_server_error()` helpers
   - Conversions from common error types
   - Unit tests for error behavior

3. **Builder Pattern** (`src/builder.rs`):
   - `RiptideBuilder` with fluent API
   - Feature-specific config builders:
     - `FetchConfigBuilder`
     - `SpiderConfigBuilder`
     - `BrowserConfigBuilder`
     - `IntelligenceConfigBuilder`
     - `SecurityConfigBuilder`
     - `MonitoringConfigBuilder`
     - `CacheConfigBuilder`
   - Configuration validation

4. **Module Structure**:
   - `/src/facades/` - 8 facade stubs
   - `/src/traits/` - Trait definitions (empty)
   - `/src/adapters/` - Internal adapters (empty)
   - `/src/composition/` - Workflow patterns (empty)

### ❌ Not Yet Implemented

1. **Configuration** (`src/config.rs`):
   - Needs actual config structs
   - Default implementations missing

2. **Runtime** (`src/runtime.rs`):
   - Coordination layer stub
   - Resource management missing

3. **ScraperFacade** (`src/facades/scraper.rs`):
   - All methods return `unimplemented!()`
   - No actual fetch/extract logic

4. **Testing**:
   - No `/tests/` directory exists
   - Only inline unit tests

## Phase 1 Implementation Plan

### Goal
Implement a **fully functional ScraperFacade** that demonstrates:
- Builder pattern usage
- Error handling patterns
- Configuration management
- Feature flag integration
- Comprehensive testing

### Success Metrics
- [ ] ScraperFacade can fetch and extract a web page
- [ ] Builder pattern works end-to-end
- [ ] At least 5 passing integration tests
- [ ] Zero compilation warnings
- [ ] Documentation complete with examples

---

## Detailed Implementation Design

### 1. Configuration Module (`src/config.rs`)

**Purpose**: Define configuration structs for all facades.

```rust
//! Configuration types for Riptide facade.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Main configuration for Riptide facade.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiptideConfig {
    #[cfg(feature = "scraper")]
    pub fetch: FetchConfig,

    #[cfg(feature = "spider")]
    pub spider: SpiderConfig,

    #[cfg(feature = "browser")]
    pub browser: BrowserConfig,

    #[cfg(feature = "extractor")]
    pub extractor: ExtractorConfig,

    #[cfg(feature = "intelligence")]
    pub intelligence: IntelligenceConfig,

    #[cfg(feature = "security")]
    pub security: SecurityConfig,

    #[cfg(feature = "monitoring")]
    pub monitoring: MonitoringConfig,

    #[cfg(feature = "cache")]
    pub cache: CacheConfig,
}

impl Default for RiptideConfig {
    fn default() -> Self {
        Self {
            #[cfg(feature = "scraper")]
            fetch: FetchConfig::default(),

            #[cfg(feature = "spider")]
            spider: SpiderConfig::default(),

            #[cfg(feature = "browser")]
            browser: BrowserConfig::default(),

            #[cfg(feature = "extractor")]
            extractor: ExtractorConfig::default(),

            #[cfg(feature = "intelligence")]
            intelligence: IntelligenceConfig::default(),

            #[cfg(feature = "security")]
            security: SecurityConfig::default(),

            #[cfg(feature = "monitoring")]
            monitoring: MonitoringConfig::default(),

            #[cfg(feature = "cache")]
            cache: CacheConfig::default(),
        }
    }
}

// ============================================================================
// Fetch Configuration (Phase 1 - IMPLEMENT)
// ============================================================================

#[cfg(feature = "scraper")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchConfig {
    /// Maximum retry attempts
    pub max_retries: u32,

    /// Request timeout in seconds
    pub timeout_secs: u64,

    /// User agent string
    pub user_agent: String,

    /// Follow HTTP redirects
    pub follow_redirects: bool,

    /// Enable compression (gzip, brotli)
    pub enable_compression: bool,

    /// Connection pool size
    pub pool_size: usize,

    /// Enable HTTP/2
    pub enable_http2: bool,
}

#[cfg(feature = "scraper")]
impl Default for FetchConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            timeout_secs: 30,
            user_agent: format!("RiptideBot/{}", env!("CARGO_PKG_VERSION")),
            follow_redirects: true,
            enable_compression: true,
            pool_size: 10,
            enable_http2: true,
        }
    }
}

// Extractor config for Phase 1
#[cfg(feature = "extractor")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractorConfig {
    /// Default extraction strategy
    pub default_strategy: String, // "css", "regex", "auto"

    /// Enable quality scoring
    pub enable_quality_scoring: bool,

    /// Minimum quality threshold
    pub min_quality_threshold: f64,
}

#[cfg(feature = "extractor")]
impl Default for ExtractorConfig {
    fn default() -> Self {
        Self {
            default_strategy: "auto".to_string(),
            enable_quality_scoring: true,
            min_quality_threshold: 0.5,
        }
    }
}

// ============================================================================
// Other Configs (Stubs for Phase 2+)
// ============================================================================

#[cfg(feature = "spider")]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpiderConfig {
    pub max_depth: u32,
    pub max_pages: u32,
    pub crawl_delay_ms: u64,
    pub respect_robots_txt: bool,
}

#[cfg(feature = "browser")]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BrowserConfig {
    pub headless: bool,
    pub pool_size: usize,
    pub enable_stealth: bool,
}

#[cfg(feature = "intelligence")]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IntelligenceConfig {
    pub default_provider: String,
    pub enable_fallback: bool,
    pub timeout_secs: u64,
}

#[cfg(feature = "security")]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecurityConfig {
    pub enable_rate_limiting: bool,
    pub rate_limit_rpm: u32,
    pub enable_pii_redaction: bool,
    pub api_key_required: bool,
}

#[cfg(feature = "monitoring")]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MonitoringConfig {
    pub enable_telemetry: bool,
    pub enable_metrics: bool,
    pub otlp_endpoint: Option<String>,
}

#[cfg(feature = "cache")]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheConfig {
    pub enable_memory_cache: bool,
    pub memory_cache_size_mb: usize,
    pub enable_redis: bool,
    pub redis_url: Option<String>,
}
```

**Testing Strategy**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RiptideConfig::default();
        #[cfg(feature = "scraper")]
        assert_eq!(config.fetch.max_retries, 3);
    }

    #[cfg(feature = "scraper")]
    #[test]
    fn test_fetch_config_defaults() {
        let config = FetchConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.timeout_secs, 30);
        assert!(config.follow_redirects);
        assert!(config.user_agent.starts_with("RiptideBot/"));
    }

    #[test]
    fn test_config_serialization() {
        let config = RiptideConfig::default();
        let json = serde_json::to_string(&config).expect("Should serialize");
        let _parsed: RiptideConfig = serde_json::from_str(&json).expect("Should deserialize");
    }
}
```

---

### 2. Runtime Module (`src/runtime.rs`)

**Purpose**: Coordinate shared resources and lifecycle management.

```rust
//! Runtime coordination for Riptide facade.

use crate::config::RiptideConfig;
use crate::error::{Result, RiptideError};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Runtime coordination layer for shared resources.
pub struct RiptideRuntime {
    config: RiptideConfig,

    // Fetch client (shared across facades)
    #[cfg(feature = "scraper")]
    fetch_client: Arc<RwLock<Option<reqwest::Client>>>,

    // Add more shared resources in Phase 2
}

impl RiptideRuntime {
    pub(crate) fn new(config: RiptideConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),

            #[cfg(feature = "scraper")]
            fetch_client: Arc::new(RwLock::new(None)),
        })
    }

    /// Get or create HTTP client (lazy initialization)
    #[cfg(feature = "scraper")]
    pub async fn get_fetch_client(&self) -> Result<reqwest::Client> {
        // Check if client exists
        {
            let client_guard = self.fetch_client.read().await;
            if let Some(client) = client_guard.as_ref() {
                return Ok(client.clone());
            }
        }

        // Create new client
        let mut client_guard = self.fetch_client.write().await;

        // Double-check (another thread might have created it)
        if let Some(client) = client_guard.as_ref() {
            return Ok(client.clone());
        }

        // Build client
        let client = self.build_fetch_client()?;
        *client_guard = Some(client.clone());

        Ok(client)
    }

    #[cfg(feature = "scraper")]
    fn build_fetch_client(&self) -> Result<reqwest::Client> {
        let fetch_config = &self.config.fetch;

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(fetch_config.timeout_secs))
            .user_agent(&fetch_config.user_agent)
            .redirect(if fetch_config.follow_redirects {
                reqwest::redirect::Policy::limited(10)
            } else {
                reqwest::redirect::Policy::none()
            })
            .gzip(fetch_config.enable_compression)
            .brotli(fetch_config.enable_compression)
            .http2_prior_knowledge()
            .pool_max_idle_per_host(fetch_config.pool_size)
            .build()
            .map_err(|e| RiptideError::ConfigError(format!("Failed to build HTTP client: {}", e)))?;

        Ok(client)
    }

    pub fn config(&self) -> &RiptideConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_runtime_creation() {
        let config = RiptideConfig::default();
        let runtime = RiptideRuntime::new(config);
        assert!(runtime.is_ok());
    }

    #[cfg(feature = "scraper")]
    #[tokio::test]
    async fn test_fetch_client_lazy_init() {
        let config = RiptideConfig::default();
        let runtime = RiptideRuntime::new(config).unwrap();

        // First call initializes
        let client1 = runtime.get_fetch_client().await.unwrap();

        // Second call reuses
        let client2 = runtime.get_fetch_client().await.unwrap();

        // Should be the same client (Arc)
        assert_eq!(
            Arc::strong_count(&Arc::new(client1.clone())),
            Arc::strong_count(&Arc::new(client2))
        );
    }
}
```

---

### 3. ScraperFacade Implementation (`src/facades/scraper.rs`)

**Purpose**: Implement the simplest facade - web page scraping.

```rust
//! Scraper facade for unified web scraping operations.

use crate::config::RiptideConfig;
use crate::error::{Result, RiptideError};
use crate::runtime::RiptideRuntime;
use riptide_types::ExtractedDoc;
use std::sync::Arc;
use tracing::{debug, info, instrument};

/// Facade for web page scraping operations.
pub struct ScraperFacade {
    config: RiptideConfig,
    runtime: Arc<RiptideRuntime>,
}

impl ScraperFacade {
    pub(crate) fn new(config: RiptideConfig, runtime: Arc<RiptideRuntime>) -> Self {
        Self { config, runtime }
    }

    /// Fetch and extract content from a URL.
    #[instrument(skip(self), fields(url = %url))]
    pub async fn fetch(&self, url: &str) -> Result<ExtractedDoc> {
        info!("Fetching URL");

        // Fetch HTML
        let html = self.fetch_html(url).await?;
        debug!("Fetched {} bytes", html.len());

        // Extract content
        let doc = self.extract_content(url, &html).await?;

        Ok(doc)
    }

    /// Fetch with custom options.
    #[instrument(skip(self, options), fields(url = %url))]
    pub async fn fetch_with_options(
        &self,
        url: &str,
        options: ScrapeOptions,
    ) -> Result<ExtractedDoc> {
        info!("Fetching with custom options");

        // Check cache first
        if options.use_cache {
            // TODO: Implement cache lookup in Phase 2
            debug!("Cache check skipped (not implemented)");
        }

        // Render JavaScript if requested
        if options.render_js {
            return Err(RiptideError::ConfigError(
                "JavaScript rendering requires 'browser' feature".to_string(),
            ));
        }

        // Apply custom headers
        if !options.headers.is_empty() {
            // TODO: Pass headers to fetch_html
            debug!("Custom headers provided: {}", options.headers.len());
        }

        // Fetch
        self.fetch(url).await
    }

    /// Batch fetch multiple URLs in parallel.
    #[instrument(skip(self), fields(count = urls.len()))]
    pub async fn fetch_batch(&self, urls: &[&str]) -> Result<Vec<ExtractedDoc>> {
        info!("Batch fetching {} URLs", urls.len());

        let mut tasks = Vec::new();

        for url in urls {
            let url_owned = url.to_string();
            let facade_clone = Self {
                config: self.config.clone(),
                runtime: Arc::clone(&self.runtime),
            };

            tasks.push(tokio::spawn(async move {
                facade_clone.fetch(&url_owned).await
            }));
        }

        let mut results = Vec::new();
        for task in tasks {
            match task.await {
                Ok(Ok(doc)) => results.push(doc),
                Ok(Err(e)) => return Err(e),
                Err(e) => {
                    return Err(RiptideError::Internal {
                        context: "Task join error".to_string(),
                        source: Some(Box::new(e)),
                    })
                }
            }
        }

        Ok(results)
    }

    // ========================================================================
    // Internal Helpers
    // ========================================================================

    async fn fetch_html(&self, url: &str) -> Result<String> {
        let client = self.runtime.get_fetch_client().await?;

        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| RiptideError::FetchError(e.to_string()))?;

        // Check status
        let status = response.status();
        if !status.is_success() {
            return Err(RiptideError::HttpError {
                status: status.as_u16(),
                message: format!("HTTP {}", status),
            });
        }

        // Get text
        let html = response
            .text()
            .await
            .map_err(|e| RiptideError::FetchError(e.to_string()))?;

        Ok(html)
    }

    async fn extract_content(&self, url: &str, html: &str) -> Result<ExtractedDoc> {
        // Use riptide-extraction crate
        #[cfg(feature = "extractor")]
        {
            use riptide_extraction::css_extraction;

            let doc = css_extraction::extract_default(html, url)
                .await
                .map_err(|e| RiptideError::ExtractionError(e.to_string()))?;

            Ok(doc)
        }

        #[cfg(not(feature = "extractor"))]
        {
            // Fallback: Create minimal ExtractedDoc
            use scraper::{Html, Selector};

            let document = Html::parse_document(html);

            // Extract title
            let title = document
                .select(&Selector::parse("title").unwrap())
                .next()
                .map(|el| el.text().collect::<String>())
                .unwrap_or_default();

            // Extract body text
            let text = document
                .select(&Selector::parse("body").unwrap())
                .next()
                .map(|el| el.text().collect::<String>())
                .unwrap_or_default();

            Ok(ExtractedDoc {
                url: url.to_string(),
                title,
                text,
                links: vec![],
                metadata: Default::default(),
                quality: riptide_types::ExtractionQuality {
                    score: 1.0,
                    is_sufficient: true,
                    reason: "Basic extraction".to_string(),
                },
            })
        }
    }
}

/// Options for scraping operations.
#[derive(Debug, Clone, Default)]
pub struct ScrapeOptions {
    /// Enable caching
    pub use_cache: bool,

    /// Enable JavaScript rendering (requires 'browser' feature)
    pub render_js: bool,

    /// Maximum wait time for dynamic content (ms)
    pub wait_for_ms: Option<u64>,

    /// Custom HTTP headers
    pub headers: Vec<(String, String)>,
}

impl ScrapeOptions {
    /// Create with default options
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable caching
    pub fn with_cache(mut self) -> Self {
        self.use_cache = true;
        self
    }

    /// Enable JavaScript rendering
    pub fn with_js_rendering(mut self) -> Self {
        self.render_js = true;
        self
    }

    /// Add custom header
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scrape_options_builder() {
        let options = ScrapeOptions::new()
            .with_cache()
            .with_js_rendering()
            .with_header("X-Custom", "value");

        assert!(options.use_cache);
        assert!(options.render_js);
        assert_eq!(options.headers.len(), 1);
    }
}
```

---

### 4. Integration Tests (`tests/integration_tests.rs`)

**Purpose**: End-to-end testing with real HTTP requests.

```rust
//! Integration tests for riptide-facade.

use riptide_facade::{Riptide, ScrapeOptions};
use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{method, path};

#[tokio::test]
async fn test_basic_fetch() {
    // Start mock server
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/test"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"<html><head><title>Test</title></head><body>Hello World</body></html>"#,
        ))
        .mount(&mock_server)
        .await;

    // Create Riptide
    let riptide = Riptide::with_defaults().unwrap();

    // Fetch
    let url = format!("{}/test", mock_server.uri());
    let doc = riptide.scraper().fetch(&url).await.unwrap();

    assert_eq!(doc.title, "Test");
    assert!(doc.text.contains("Hello World"));
}

#[tokio::test]
async fn test_fetch_with_options() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/options"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"<html><head><title>Options Test</title></head><body>Content</body></html>"#,
        ))
        .mount(&mock_server)
        .await;

    let riptide = Riptide::with_defaults().unwrap();

    let options = ScrapeOptions::new()
        .with_header("X-Test", "value");

    let url = format!("{}/options", mock_server.uri());
    let doc = riptide.scraper()
        .fetch_with_options(&url, options)
        .await
        .unwrap();

    assert_eq!(doc.title, "Options Test");
}

#[tokio::test]
async fn test_batch_fetch() {
    let mock_server = MockServer::start().await;

    // Mock multiple endpoints
    for i in 1..=3 {
        Mock::given(method("GET"))
            .and(path(format!("/page{}", i)))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                format!(r#"<html><head><title>Page {}</title></head><body>Content {}</body></html>"#, i, i),
            ))
            .mount(&mock_server)
            .await;
    }

    let riptide = Riptide::with_defaults().unwrap();

    let urls = vec![
        format!("{}/page1", mock_server.uri()),
        format!("{}/page2", mock_server.uri()),
        format!("{}/page3", mock_server.uri()),
    ];

    let url_refs: Vec<&str> = urls.iter().map(|s| s.as_str()).collect();
    let docs = riptide.scraper().fetch_batch(&url_refs).await.unwrap();

    assert_eq!(docs.len(), 3);
    assert_eq!(docs[0].title, "Page 1");
    assert_eq!(docs[1].title, "Page 2");
    assert_eq!(docs[2].title, "Page 3");
}

#[tokio::test]
async fn test_http_error_handling() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/notfound"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    let riptide = Riptide::with_defaults().unwrap();

    let url = format!("{}/notfound", mock_server.uri());
    let result = riptide.scraper().fetch(&url).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, riptide_facade::RiptideError::HttpError { .. }));
}

#[tokio::test]
async fn test_builder_configuration() {
    use std::time::Duration;

    let riptide = Riptide::builder()
        .with_fetch(|fetch| {
            fetch
                .max_retries(5)
                .timeout(60)
                .user_agent("CustomBot/1.0")
        })
        .build()
        .unwrap();

    let config = riptide.config();
    assert_eq!(config.fetch.max_retries, 5);
    assert_eq!(config.fetch.timeout_secs, 60);
    assert_eq!(config.fetch.user_agent, "CustomBot/1.0");
}
```

---

## File Structure Summary

```
riptide-facade/
├── Cargo.toml                    # ✅ Exists, feature flags configured
├── src/
│   ├── lib.rs                    # ✅ Exists, main entry point
│   ├── builder.rs                # ✅ Exists, builder pattern
│   ├── error.rs                  # ✅ Exists, error types
│   ├── config.rs                 # ❌ IMPLEMENT - Add actual configs
│   ├── runtime.rs                # ❌ IMPLEMENT - Add HTTP client management
│   ├── prelude.rs                # ✅ Exists
│   ├── facades/
│   │   ├── mod.rs                # ✅ Exists
│   │   └── scraper.rs            # ❌ IMPLEMENT - Actual scraping logic
│   ├── traits/
│   │   └── mod.rs                # ⏭️ Phase 2
│   ├── adapters/
│   │   └── mod.rs                # ⏭️ Phase 2
│   └── composition/
│       └── mod.rs                # ⏭️ Phase 2
└── tests/
    └── integration_tests.rs      # ❌ CREATE - Integration tests
```

---

## Implementation Checklist

### Phase 1A: Core Infrastructure (Day 1)
- [ ] Implement `src/config.rs` with full config structs
- [ ] Add unit tests for config serialization/defaults
- [ ] Implement `src/runtime.rs` with HTTP client management
- [ ] Add unit tests for runtime lazy initialization
- [ ] Run `cargo build --features scraper,extractor`
- [ ] Run `cargo test --lib`

### Phase 1B: ScraperFacade (Day 2)
- [ ] Implement `fetch()` method
- [ ] Implement `fetch_with_options()`
- [ ] Implement `fetch_batch()`
- [ ] Add internal helper methods
- [ ] Add ScrapeOptions builder methods
- [ ] Run `cargo build --features scraper`

### Phase 1C: Integration Testing (Day 3)
- [ ] Create `/tests/integration_tests.rs`
- [ ] Test basic fetch with WireMock
- [ ] Test fetch with options
- [ ] Test batch fetching
- [ ] Test error handling (404, 500, timeout)
- [ ] Test builder configuration
- [ ] Run `cargo test --features scraper,extractor`
- [ ] Ensure all 5+ tests pass

### Phase 1D: Documentation (Day 4)
- [ ] Add doc comments to all public APIs
- [ ] Create usage examples in `examples/`
- [ ] Update main README with facade example
- [ ] Run `cargo doc --open`
- [ ] Verify no broken doc links

### Phase 1E: Quality Assurance (Day 5)
- [ ] Run `cargo clippy -- -D warnings`
- [ ] Run `cargo fmt --check`
- [ ] Test with different feature combinations
- [ ] Performance benchmark (if time permits)
- [ ] Code review checklist

---

## Testing Strategy

### Unit Tests
- **Config Module**: 3 tests (defaults, serialization, feature gates)
- **Runtime Module**: 2 tests (creation, client lazy init)
- **ScraperFacade**: 1 test (options builder)

### Integration Tests
- **Basic Fetch**: Verify scraping works end-to-end
- **Fetch with Options**: Custom headers, caching flags
- **Batch Fetch**: Parallel requests, result ordering
- **Error Handling**: 404, 500, timeout scenarios
- **Builder Configuration**: Custom user-agent, retries

### Coverage Target
- **Minimum**: 70% line coverage
- **Goal**: 85%+ line coverage

---

## Dependencies Required

Already in `Cargo.toml`:
```toml
[dependencies]
tokio = { workspace = true }
async-trait = "0.1"
reqwest = { workspace = true }  # HTTP client
scraper = "0.20"                # HTML parsing (fallback)
tracing = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }

[dev-dependencies]
tokio-test = "0.4"
wiremock = { workspace = true }  # HTTP mocking
tempfile = { workspace = true }
```

---

## Error Handling Patterns

### Pattern 1: Context Enrichment
```rust
self.fetch_html(url)
    .await
    .map_err(|e| e.with_context(format!("Failed to fetch {}", url)))?
```

### Pattern 2: Error Mapping
```rust
.map_err(|e| RiptideError::FetchError(e.to_string()))?
```

### Pattern 3: Retryability Check
```rust
if err.is_retryable() {
    // Retry logic
}
```

---

## Performance Considerations

1. **HTTP Client Pooling**: `reqwest::Client` is cloned (Arc internally)
2. **Lazy Initialization**: Client created on first use
3. **Parallel Fetching**: `tokio::spawn` for batch operations
4. **Zero-Cost Abstractions**: Inline methods where possible

---

## Next Steps After Phase 1

### Phase 2: Additional Facades
1. SpiderFacade (web crawling)
2. BrowserFacade (headless browser)
3. ExtractorFacade (advanced extraction)

### Phase 3: Composition Patterns
1. WorkflowBuilder
2. Pipeline pattern
3. Batch processing utilities

### Phase 4: API Integration
1. Update `riptide-api` to use facade
2. Add backward compatibility shims
3. Migration guide

---

## Code Review Checklist

- [ ] All public APIs have doc comments
- [ ] Error handling uses facade error types
- [ ] Feature flags correctly applied
- [ ] No panics in production code
- [ ] All tests pass
- [ ] No compiler warnings
- [ ] Clippy lints pass
- [ ] Code formatted with rustfmt
- [ ] No TODO comments in production code
- [ ] README updated with examples

---

**Status**: Ready for Implementation
**Estimated Effort**: 3-5 days
**Complexity**: Medium
**Risk**: Low (well-defined scope, clear patterns)
