# Riptide Facade Composition Layer - Architecture Design

**Status**: Design Phase
**Version**: 1.0.0
**Date**: 2025-10-18
**Author**: System Architect (Hive Mind P1-A4)

## Executive Summary

The `riptide-facade` crate provides a unified, simplified API for accessing the entire Riptide/EventMesh ecosystem. It serves as a composition layer that reduces coupling between `riptide-api` and the 24+ specialized crates, while maintaining backward compatibility and following Rust best practices.

## Table of Contents

1. [Problem Statement](#problem-statement)
2. [Goals and Non-Goals](#goals-and-non-goals)
3. [Architecture Overview](#architecture-overview)
4. [API Surface Design](#api-surface-design)
5. [Module Structure](#module-structure)
6. [Trait Unification](#trait-unification)
7. [Composition Patterns](#composition-patterns)
8. [Error Handling Strategy](#error-handling-strategy)
9. [Migration Path](#migration-path)
10. [Implementation Roadmap](#implementation-roadmap)

---

## Problem Statement

### Current State

The Riptide ecosystem consists of 24+ specialized crates:
- **Core Infrastructure**: riptide-core (orchestration, cache, circuit breakers, memory)
- **Web Scraping**: riptide-spider, riptide-fetch, riptide-extraction
- **Browser Automation**: riptide-engine, riptide-headless, riptide-browser-abstraction
- **Intelligence**: riptide-intelligence (LLM abstraction)
- **Security**: riptide-security (auth, rate limiting, PII redaction)
- **Monitoring**: riptide-monitoring (telemetry, metrics, alerts)
- **Specialized**: riptide-pdf, riptide-stealth, riptide-search, riptide-workers, etc.

### Challenges

1. **High Coupling**: `riptide-api` directly depends on 15+ crates
2. **Complex API Surface**: Users must understand which crate provides which functionality
3. **Inconsistent Patterns**: Different crates use different error types, async patterns, configuration styles
4. **Difficult Composition**: Combining features from multiple crates requires deep knowledge
5. **Breaking Changes**: Changes in one crate propagate directly to API consumers

### Impact

- Increased cognitive load for API developers
- Brittle integration points
- Difficult to test and mock
- Hard to evolve individual crates independently

---

## Goals and Non-Goals

### Goals

✅ **Simplify API Surface**: Provide cohesive, task-oriented APIs
✅ **Reduce Coupling**: Abstract internal crate boundaries
✅ **Unified Error Handling**: Single error type with context preservation
✅ **Composition Patterns**: Pre-built workflows for common use cases
✅ **Backward Compatibility**: Maintain existing functionality
✅ **Type Safety**: Leverage Rust's type system for compile-time guarantees
✅ **Async-First**: Native async/await support throughout

### Non-Goals

❌ **Replace Individual Crates**: Power users can still use crates directly
❌ **One-Size-Fits-All**: Provide escape hatches for advanced scenarios
❌ **Performance Overhead**: Zero-cost abstractions where possible
❌ **Complete Rewrite**: Wrap existing functionality, don't duplicate

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                      riptide-api                            │
│                   (HTTP API Layer)                          │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                   riptide-facade                            │
│              (Composition & Unification)                    │
├─────────────────────────────────────────────────────────────┤
│  • Unified Traits      • Error Mapping                      │
│  • Builder Patterns    • Workflow Composition               │
│  • Async Coordination  • Config Management                  │
└─────────┬───────────────────────────────────┬───────────────┘
          │                                   │
          ▼                                   ▼
┌──────────────────────┐           ┌──────────────────────────┐
│  Core Functionality  │           │  Specialized Features     │
├──────────────────────┤           ├──────────────────────────┤
│ • riptide-core       │           │ • riptide-intelligence   │
│ • riptide-spider     │           │ • riptide-security       │
│ • riptide-fetch      │           │ • riptide-monitoring     │
│ • riptide-extraction │           │ • riptide-pdf            │
│ • riptide-engine     │           │ • riptide-stealth        │
│ • riptide-types      │           │ • riptide-search         │
└──────────────────────┘           └──────────────────────────┘
```

### Design Principles

1. **Layered Abstraction**: Facade provides high-level APIs, delegates to specialized crates
2. **Trait-Based Composition**: Use traits for flexibility and testability
3. **Builder Pattern**: Fluent configuration APIs
4. **Error Context**: Rich error context without losing underlying details
5. **Feature Flags**: Optional functionality behind feature gates
6. **Zero-Cost Abstractions**: Inline delegation where possible

---

## API Surface Design

### Primary Entry Points

```rust
// Main facade entry point
pub struct Riptide {
    config: RiptideConfig,
    runtime: Arc<RiptideRuntime>,
}

impl Riptide {
    // Builder pattern for configuration
    pub fn builder() -> RiptideBuilder { }

    // Core scraping operations
    pub fn scraper(&self) -> ScraperFacade { }
    pub fn spider(&self) -> SpiderFacade { }
    pub fn browser(&self) -> BrowserFacade { }

    // Intelligence and extraction
    pub fn extractor(&self) -> ExtractorFacade { }
    pub fn intelligence(&self) -> IntelligenceFacade { }

    // Infrastructure
    pub fn security(&self) -> SecurityFacade { }
    pub fn monitoring(&self) -> MonitoringFacade { }
    pub fn cache(&self) -> CacheFacade { }
}
```

### Task-Oriented Facades

Each facade provides a cohesive API for a specific domain:

```rust
// Scraping facade - unified web scraping
pub struct ScraperFacade {
    fetch: FetchClient,
    extractor: ExtractionEngine,
    cache: CacheLayer,
    security: SecurityMiddleware,
}

impl ScraperFacade {
    // Simple scraping
    pub async fn fetch(&self, url: &str) -> Result<ExtractedDoc> { }

    // Scraping with options
    pub async fn fetch_with_options(
        &self,
        url: &str,
        options: ScrapeOptions
    ) -> Result<ExtractedDoc> { }

    // Batch scraping
    pub async fn fetch_batch(
        &self,
        urls: &[&str]
    ) -> Result<Vec<ExtractedDoc>> { }

    // Streaming scraping
    pub fn fetch_stream(
        &self,
        urls: impl Stream<Item = String>
    ) -> impl Stream<Item = Result<ExtractedDoc>> { }
}

// Spider facade - web crawling
pub struct SpiderFacade {
    spider: Spider,
    config: SpiderConfig,
}

impl SpiderFacade {
    // Simple crawl
    pub async fn crawl(&self, start_url: &str) -> Result<CrawlResult> { }

    // Crawl with budget
    pub async fn crawl_with_budget(
        &self,
        start_url: &str,
        budget: CrawlBudget
    ) -> Result<CrawlResult> { }

    // Streaming crawl
    pub fn crawl_stream(
        &self,
        start_url: &str
    ) -> impl Stream<Item = Result<ExtractedDoc>> { }

    // Query-aware crawl
    pub async fn crawl_query_aware(
        &self,
        start_url: &str,
        query: &str
    ) -> Result<CrawlResult> { }
}

// Browser facade - headless browser automation
pub struct BrowserFacade {
    pool: Arc<BrowserPool>,
    launcher: HeadlessLauncher,
}

impl BrowserFacade {
    // Execute JavaScript
    pub async fn execute_js(
        &self,
        url: &str,
        script: &str
    ) -> Result<serde_json::Value> { }

    // Screenshot
    pub async fn screenshot(
        &self,
        url: &str,
        options: ScreenshotOptions
    ) -> Result<Vec<u8>> { }

    // PDF generation
    pub async fn render_pdf(
        &self,
        url: &str,
        options: PdfOptions
    ) -> Result<Vec<u8>> { }

    // Full page interaction
    pub async fn interact(
        &self,
        url: &str,
        actions: Vec<BrowserAction>
    ) -> Result<InteractionResult> { }
}

// Extractor facade - content extraction
pub struct ExtractorFacade {
    strategies: StrategyManager,
    wasm: WasmExtractor,
}

impl ExtractorFacade {
    // CSS extraction
    pub async fn extract_css(
        &self,
        html: &str,
        selectors: CssSelectors
    ) -> Result<ExtractedContent> { }

    // Regex extraction
    pub async fn extract_regex(
        &self,
        html: &str,
        patterns: Vec<RegexPattern>
    ) -> Result<ExtractedContent> { }

    // AI-powered extraction
    pub async fn extract_intelligent(
        &self,
        html: &str,
        schema: ExtractionSchema
    ) -> Result<ExtractedContent> { }

    // Multi-strategy extraction
    pub async fn extract_auto(
        &self,
        html: &str
    ) -> Result<ExtractedContent> { }
}

// Intelligence facade - LLM operations
pub struct IntelligenceFacade {
    client: IntelligenceClient,
    registry: Arc<LlmRegistry>,
}

impl IntelligenceFacade {
    // Text completion
    pub async fn complete(
        &self,
        prompt: &str
    ) -> Result<CompletionResponse> { }

    // Embeddings
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> { }

    // Structured extraction
    pub async fn extract_structured<T: DeserializeOwned>(
        &self,
        text: &str,
        schema: &str
    ) -> Result<T> { }

    // Multi-provider fallback
    pub async fn complete_with_fallback(
        &self,
        prompt: &str,
        providers: Vec<&str>
    ) -> Result<CompletionResponse> { }
}

// Security facade - authentication, rate limiting, PII
pub struct SecurityFacade {
    middleware: SecurityMiddleware,
    api_keys: ApiKeyManager,
    pii: PiiRedactor,
}

impl SecurityFacade {
    // Validate API key
    pub async fn validate_key(&self, key: &str) -> Result<ApiKey> { }

    // Check rate limit
    pub async fn check_rate_limit(&self, key: &str) -> Result<()> { }

    // Redact PII
    pub fn redact_pii(&self, text: &str) -> String { }

    // Apply security headers
    pub fn secure_headers(&self) -> HeaderMap { }
}

// Monitoring facade - metrics, telemetry, health
pub struct MonitoringFacade {
    metrics: MetricsCollector,
    telemetry: TelemetryProvider,
}

impl MonitoringFacade {
    // Record metric
    pub fn record_metric(&self, name: &str, value: f64) { }

    // Start span
    pub fn start_span(&self, name: &str) -> Span { }

    // Health check
    pub async fn health_check(&self) -> HealthStatus { }

    // Get metrics
    pub fn get_metrics(&self) -> MetricsSnapshot { }
}
```

---

## Module Structure

```
riptide-facade/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs              # Main entry point, re-exports
│   ├── builder.rs          # RiptideBuilder pattern
│   ├── config.rs           # Unified configuration
│   ├── error.rs            # Unified error types
│   ├── runtime.rs          # Runtime coordination
│   │
│   ├── facades/            # Domain-specific facades
│   │   ├── mod.rs
│   │   ├── scraper.rs      # ScraperFacade
│   │   ├── spider.rs       # SpiderFacade
│   │   ├── browser.rs      # BrowserFacade
│   │   ├── extractor.rs    # ExtractorFacade
│   │   ├── intelligence.rs # IntelligenceFacade
│   │   ├── security.rs     # SecurityFacade
│   │   ├── monitoring.rs   # MonitoringFacade
│   │   └── cache.rs        # CacheFacade
│   │
│   ├── traits/             # Unified trait definitions
│   │   ├── mod.rs
│   │   ├── scraper.rs      # Scraper traits
│   │   ├── extractor.rs    # Extractor traits
│   │   ├── browser.rs      # Browser traits
│   │   └── provider.rs     # Provider traits
│   │
│   ├── composition/        # Workflow composition
│   │   ├── mod.rs
│   │   ├── workflows.rs    # Pre-built workflows
│   │   ├── pipeline.rs     # Pipeline builders
│   │   └── batch.rs        # Batch processing
│   │
│   ├── adapters/           # Internal crate adapters
│   │   ├── mod.rs
│   │   ├── fetch.rs        # riptide-fetch adapter
│   │   ├── spider.rs       # riptide-spider adapter
│   │   ├── engine.rs       # riptide-engine adapter
│   │   └── intelligence.rs # riptide-intelligence adapter
│   │
│   └── prelude.rs          # Common imports
│
└── tests/
    ├── integration/
    │   ├── scraping.rs
    │   ├── crawling.rs
    │   └── workflows.rs
    └── fixtures/
```

---

## Trait Unification

### Core Traits

```rust
// Unified scraper trait
#[async_trait]
pub trait Scraper: Send + Sync {
    async fn fetch(&self, url: &str) -> Result<ExtractedDoc>;
    async fn fetch_with_options(
        &self,
        url: &str,
        options: ScrapeOptions
    ) -> Result<ExtractedDoc>;
}

// Unified extractor trait
#[async_trait]
pub trait Extractor: Send + Sync {
    async fn extract(
        &self,
        html: &str,
        strategy: ExtractionStrategy
    ) -> Result<ExtractedContent>;
}

// Unified browser trait (extends riptide-browser-abstraction)
#[async_trait]
pub trait Browser: Send + Sync {
    async fn navigate(&self, url: &str) -> Result<()>;
    async fn execute_script(&self, script: &str) -> Result<serde_json::Value>;
    async fn screenshot(&self, options: ScreenshotOptions) -> Result<Vec<u8>>;
}

// Unified cache trait
#[async_trait]
pub trait Cache: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn set(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
}

// Provider trait for dependency injection
#[async_trait]
pub trait Provider<T>: Send + Sync {
    async fn provide(&self) -> Result<Arc<T>>;
    fn name(&self) -> &str;
}
```

### Trait Implementations

Adapt existing crate types to unified traits:

```rust
// Adapter for riptide-fetch
impl Scraper for FetchClient {
    async fn fetch(&self, url: &str) -> Result<ExtractedDoc> {
        let response = riptide_fetch::fetch_with_retries(url, &Default::default())
            .await
            .map_err(|e| RiptideError::FetchError(e.to_string()))?;

        Ok(ExtractedDoc::from_response(response))
    }
}

// Adapter for riptide-extraction strategies
impl Extractor for StrategyManager {
    async fn extract(
        &self,
        html: &str,
        strategy: ExtractionStrategy
    ) -> Result<ExtractedContent> {
        match strategy {
            ExtractionStrategy::Css(selectors) => {
                self.css_extract(html, selectors).await
            }
            ExtractionStrategy::Regex(patterns) => {
                self.regex_extract(html, patterns).await
            }
            ExtractionStrategy::Wasm => {
                self.wasm_extract(html).await
            }
        }
        .map_err(|e| RiptideError::ExtractionError(e.to_string()))
    }
}
```

---

## Composition Patterns

### Workflow Composition

Pre-built workflows for common use cases:

```rust
pub struct WorkflowBuilder {
    config: WorkflowConfig,
    steps: Vec<WorkflowStep>,
}

impl WorkflowBuilder {
    // Scrape and extract workflow
    pub fn scrape_and_extract() -> Self {
        Self::new()
            .add_step(WorkflowStep::Fetch)
            .add_step(WorkflowStep::Extract)
            .add_step(WorkflowStep::Cache)
    }

    // Crawl and index workflow
    pub fn crawl_and_index() -> Self {
        Self::new()
            .add_step(WorkflowStep::Spider)
            .add_step(WorkflowStep::Extract)
            .add_step(WorkflowStep::Index)
    }

    // Browser automation workflow
    pub fn browser_automation() -> Self {
        Self::new()
            .add_step(WorkflowStep::LaunchBrowser)
            .add_step(WorkflowStep::Navigate)
            .add_step(WorkflowStep::ExecuteActions)
            .add_step(WorkflowStep::Extract)
    }

    // Execute workflow
    pub async fn execute(&self, input: WorkflowInput) -> Result<WorkflowOutput> {
        let mut context = WorkflowContext::new(input);

        for step in &self.steps {
            context = step.execute(context).await?;
        }

        Ok(context.into_output())
    }
}

// Example usage
let result = Riptide::builder()
    .build()?
    .workflow()
    .scrape_and_extract()
    .with_cache()
    .with_retry(3)
    .execute(WorkflowInput::url("https://example.com"))
    .await?;
```

### Pipeline Pattern

Chainable operations:

```rust
// Pipeline builder
pub struct Pipeline<T> {
    steps: Vec<Box<dyn PipelineStep<T>>>,
}

impl Pipeline<ExtractedDoc> {
    pub fn new() -> Self { }

    pub fn fetch(mut self) -> Self { }
    pub fn extract(mut self, strategy: ExtractionStrategy) -> Self { }
    pub fn transform<F>(mut self, f: F) -> Self
    where F: Fn(ExtractedDoc) -> ExtractedDoc + 'static { }
    pub fn filter<F>(mut self, f: F) -> Self
    where F: Fn(&ExtractedDoc) -> bool + 'static { }

    pub async fn execute(self, input: String) -> Result<ExtractedDoc> { }
}

// Example usage
let result = Pipeline::new()
    .fetch()
    .extract(ExtractionStrategy::Auto)
    .filter(|doc| doc.quality_score > 0.8)
    .transform(|doc| {
        // Clean up content
        doc
    })
    .execute("https://example.com".to_string())
    .await?;
```

---

## Error Handling Strategy

### Unified Error Type

```rust
#[derive(Debug, thiserror::Error)]
pub enum RiptideError {
    // Network errors
    #[error("Fetch error: {0}")]
    FetchError(String),

    #[error("HTTP error: {status} - {message}")]
    HttpError {
        status: u16,
        message: String,
    },

    // Extraction errors
    #[error("Extraction error: {0}")]
    ExtractionError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    // Browser errors
    #[error("Browser error: {0}")]
    BrowserError(String),

    #[error("CDP error: {0}")]
    CdpError(String),

    // Intelligence errors
    #[error("LLM error: {provider} - {message}")]
    LlmError {
        provider: String,
        message: String,
    },

    // Security errors
    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Rate limit exceeded")]
    RateLimitError,

    // Configuration errors
    #[error("Configuration error: {0}")]
    ConfigError(String),

    // Validation errors
    #[error("Validation error: {field} - {message}")]
    ValidationError {
        field: String,
        message: String,
    },

    // Internal errors with context
    #[error("Internal error: {0}")]
    Internal(#[from] Box<dyn std::error::Error + Send + Sync>),
}

// Context preservation
impl RiptideError {
    pub fn with_context<S: Into<String>>(self, context: S) -> Self {
        // Wrap error with additional context
        Self::Internal(Box::new(ContextError {
            context: context.into(),
            source: Box::new(self),
        }))
    }
}

// Result type alias
pub type Result<T> = std::result::Result<T, RiptideError>;
```

### Error Mapping

Map crate-specific errors to unified error type:

```rust
// From riptide-fetch
impl From<riptide_fetch::FetchError> for RiptideError {
    fn from(err: riptide_fetch::FetchError) -> Self {
        RiptideError::FetchError(err.to_string())
    }
}

// From riptide-spider
impl From<riptide_spider::SpiderError> for RiptideError {
    fn from(err: riptide_spider::SpiderError) -> Self {
        RiptideError::Internal(Box::new(err))
    }
}

// From riptide-intelligence
impl From<riptide_intelligence::IntelligenceError> for RiptideError {
    fn from(err: riptide_intelligence::IntelligenceError) -> Self {
        match err {
            IntelligenceError::Provider(msg) =>
                RiptideError::LlmError {
                    provider: "unknown".to_string(),
                    message: msg
                },
            IntelligenceError::RateLimit { .. } =>
                RiptideError::RateLimitError,
            _ => RiptideError::Internal(Box::new(err)),
        }
    }
}
```

---

## Migration Path

### Phase 1: Foundation (Week 1-2)

1. **Create crate structure**
   - Set up Cargo.toml with all dependencies
   - Create module hierarchy
   - Define core traits and error types

2. **Implement basic facades**
   - ScraperFacade (wraps riptide-fetch + riptide-extraction)
   - SpiderFacade (wraps riptide-spider)
   - BrowserFacade (wraps riptide-engine)

3. **Builder pattern**
   - RiptideBuilder for configuration
   - Feature flag support

### Phase 2: Advanced Features (Week 3-4)

1. **Composition patterns**
   - WorkflowBuilder
   - Pipeline pattern
   - Batch processing utilities

2. **Additional facades**
   - IntelligenceFacade
   - SecurityFacade
   - MonitoringFacade
   - CacheFacade

3. **Integration testing**
   - End-to-end workflows
   - Performance benchmarks
   - Error handling scenarios

### Phase 3: API Integration (Week 5-6)

1. **Update riptide-api**
   - Replace direct crate dependencies with facade
   - Update handlers to use unified APIs
   - Add backward compatibility shims

2. **Documentation**
   - API documentation
   - Migration guide
   - Example workflows

3. **Performance optimization**
   - Identify overhead
   - Inline hot paths
   - Optimize error conversions

### Backward Compatibility Strategy

```rust
// In riptide-api: Use facade while maintaining old imports
pub use riptide_facade::{
    Riptide,
    ScraperFacade,
    SpiderFacade,
    // ... other facades
};

// Legacy re-exports (deprecated)
#[deprecated(since = "0.2.0", note = "Use riptide_facade::ScraperFacade instead")]
pub use riptide_fetch as fetch;

#[deprecated(since = "0.2.0", note = "Use riptide_facade::SpiderFacade instead")]
pub use riptide_spider as spider;
```

### Migration Examples

**Before (direct crate usage):**
```rust
// In riptide-api handler
use riptide_fetch::{fetch_with_retries, FetchConfig};
use riptide_extraction::css_extraction;
use riptide_core::cache::Cache;

async fn scrape_handler(url: String) -> Result<ExtractedDoc> {
    let config = FetchConfig::default();
    let response = fetch_with_retries(&url, &config).await?;
    let html = response.text().await?;
    let doc = css_extraction::extract_default(&html, &url).await?;
    Ok(doc)
}
```

**After (facade usage):**
```rust
// In riptide-api handler
use riptide_facade::Riptide;

async fn scrape_handler(
    riptide: &Riptide,
    url: String
) -> Result<ExtractedDoc> {
    riptide.scraper()
        .fetch_with_options(&url, ScrapeOptions::default())
        .await
}
```

---

## Implementation Roadmap

### Milestone 1: Core Facade (1-2 weeks)
- [ ] Create crate structure
- [ ] Define error types
- [ ] Implement RiptideBuilder
- [ ] Implement ScraperFacade
- [ ] Implement SpiderFacade
- [ ] Implement BrowserFacade
- [ ] Basic integration tests

### Milestone 2: Advanced Facades (2-3 weeks)
- [ ] Implement ExtractorFacade
- [ ] Implement IntelligenceFacade
- [ ] Implement SecurityFacade
- [ ] Implement MonitoringFacade
- [ ] Implement CacheFacade
- [ ] Workflow composition patterns
- [ ] Pipeline builder

### Milestone 3: API Integration (1-2 weeks)
- [ ] Update riptide-api to use facade
- [ ] Refactor handlers
- [ ] Add deprecation warnings
- [ ] Migration guide
- [ ] Performance testing

### Milestone 4: Polish & Documentation (1 week)
- [ ] API documentation
- [ ] Usage examples
- [ ] Performance optimization
- [ ] Error message improvements
- [ ] Final integration testing

**Total Estimated Timeline**: 5-8 weeks

---

## Configuration Example

```rust
// Unified configuration
#[derive(Debug, Clone)]
pub struct RiptideConfig {
    // Fetch configuration
    pub fetch: FetchConfig,

    // Spider configuration
    pub spider: SpiderConfig,

    // Browser configuration
    pub browser: BrowserConfig,

    // Intelligence configuration
    pub intelligence: IntelligenceConfig,

    // Security configuration
    pub security: SecurityConfig,

    // Monitoring configuration
    pub monitoring: MonitoringConfig,

    // Cache configuration
    pub cache: CacheConfig,
}

// Builder pattern
let riptide = Riptide::builder()
    // Fetch options
    .with_fetch(|fetch| {
        fetch
            .max_retries(3)
            .timeout(Duration::from_secs(30))
            .user_agent("RiptideBot/1.0")
    })
    // Spider options
    .with_spider(|spider| {
        spider
            .max_depth(5)
            .max_pages(1000)
            .crawl_delay(Duration::from_millis(200))
    })
    // Browser options
    .with_browser(|browser| {
        browser
            .headless(true)
            .pool_size(5)
            .enable_stealth()
    })
    // Intelligence options
    .with_intelligence(|intelligence| {
        intelligence
            .default_provider("openai")
            .enable_fallback()
            .timeout(Duration::from_secs(60))
    })
    // Security options
    .with_security(|security| {
        security
            .enable_rate_limiting()
            .enable_pii_redaction()
            .api_key_required(true)
    })
    // Monitoring options
    .with_monitoring(|monitoring| {
        monitoring
            .enable_telemetry()
            .enable_metrics()
            .otlp_endpoint("http://localhost:4317")
    })
    // Cache options
    .with_cache(|cache| {
        cache
            .enable_memory_cache()
            .memory_cache_size_mb(100)
            .enable_redis()
            .redis_url("redis://localhost:6379")
    })
    .build()?;
```

---

## Testing Strategy

### Unit Tests
- Test each facade independently
- Mock underlying crate dependencies
- Verify error mapping
- Test builder configuration

### Integration Tests
- Test workflows end-to-end
- Test facade composition
- Test error propagation
- Test async coordination

### Performance Tests
- Measure facade overhead
- Compare with direct crate usage
- Optimize hot paths
- Memory profiling

---

## Appendix: Crate Dependency Matrix

| Facade | Core Dependencies | Optional Dependencies |
|--------|-------------------|----------------------|
| ScraperFacade | riptide-fetch, riptide-extraction, riptide-types | riptide-cache, riptide-security |
| SpiderFacade | riptide-spider, riptide-fetch | riptide-cache, riptide-search |
| BrowserFacade | riptide-engine, riptide-browser-abstraction | riptide-headless, riptide-stealth |
| ExtractorFacade | riptide-extraction, riptide-types | riptide-intelligence, riptide-pdf |
| IntelligenceFacade | riptide-intelligence | - |
| SecurityFacade | riptide-security | - |
| MonitoringFacade | riptide-monitoring | - |
| CacheFacade | riptide-cache | riptide-persistence |

---

## Next Steps

1. **Review and Approval**: Get feedback from team on architecture
2. **Create riptide-facade crate**: Set up initial structure
3. **Implement Phase 1**: Core facades and builder pattern
4. **Integrate with riptide-api**: Refactor one handler as proof of concept
5. **Iterate**: Gather feedback and refine design

---

**Document Version**: 1.0.0
**Last Updated**: 2025-10-18
**Status**: Ready for Implementation
