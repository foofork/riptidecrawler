# Rust API Patterns Research for RipTide
## Composable, Idiomatic Web Extraction Architecture

**Research Date:** 2025-01-04
**Focus:** Building composable, idiomatic Rust APIs for web extraction/crawling tools
**Project Context:** RipTide - High-performance web extraction framework

---

## Executive Summary

This research analyzes Rust idioms and patterns for building composable web extraction APIs by studying RipTide's existing architecture and comparable Rust projects. The goal is to identify the most idiomatic approaches for progressive API complexity while maintaining modularity.

**Key Findings:**
1. **Trait-based composition** (like `async_trait`) enables modular, pluggable extractors
2. **Builder pattern** with typestate for progressive complexity and compile-time guarantees
3. **Stream-based processing** using `tokio::stream` for async iteration
4. **Layered error handling** with `thiserror` for domain-specific errors
5. **Arc-wrapped shared state** for concurrent access patterns

---

## 1. Async Trait Patterns for Extraction APIs

### 1.1 Current RipTide Pattern

**Source:** `crates/riptide-types/src/traits.rs`, `crates/riptide-extraction/src/strategies/traits.rs`

```rust
use async_trait::async_trait;

#[async_trait]
pub trait ExtractionStrategy: Send + Sync {
    /// Extract content from HTML
    async fn extract(&self, html: &str, url: &str) -> Result<ExtractionResult>;

    /// Get strategy name
    fn name(&self) -> &str;

    /// Get strategy capabilities
    fn capabilities(&self) -> StrategyCapabilities;

    /// Check if strategy is available
    fn is_available(&self) -> bool { true }

    /// Calculate confidence score (0.0-1.0)
    fn confidence_score(&self, html: &str) -> f64 {
        // Default heuristics
        if html.contains("<article") || html.contains("<main") {
            0.8
        } else {
            0.5
        }
    }
}
```

**Strengths:**
- ✅ **Pluggable architecture**: Swap extractors without changing client code
- ✅ **Default implementations**: Sensible defaults reduce boilerplate
- ✅ **Progressive disclosure**: Simple methods first, advanced optional
- ✅ **Capability-based design**: Extractors declare what they can do

**Patterns from Ecosystem:**

**1. Tower's Service Trait (request/response pattern)**
```rust
// Inspired by tower::Service - used in axum, hyper
pub trait Service<Request> {
    type Response;
    type Error;
    type Future: Future<Output = Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;
    fn call(&mut self, req: Request) -> Self::Future;
}
```

**Application to RipTide:**
```rust
// Service-oriented extractor with backpressure
#[async_trait]
pub trait ExtractorService: Send + Sync {
    async fn ready(&self) -> Result<()>;
    async fn extract(&self, request: ExtractionRequest) -> Result<ExtractionResult>;
}
```

**2. Reqwest's Builder + Finalize Pattern**
```rust
// Reqwest pattern: builder returns self, execute() finalizes
let result = client
    .get("https://example.com")
    .header("User-Agent", "bot")
    .timeout(Duration::from_secs(10))
    .send()  // Finalizer consumes self
    .await?;
```

**Recommendation for RipTide:**
- ✅ Keep `async_trait` for trait-based extensibility
- ✅ Add `poll_ready()` for backpressure-aware extraction
- ✅ Use GATs (Generic Associated Types) when stable for zero-cost abstractions
- ✅ Consider `Service` pattern for middleware/layering

### 1.2 Trait Composition Patterns

**Source:** `crates/riptide-extraction/src/strategies/manager.rs`

**Current Pattern:**
```rust
pub struct StrategyRegistry {
    extraction_strategies: HashMap<String, Arc<dyn ExtractionStrategy>>,
    spider_strategies: HashMap<String, Arc<dyn SpiderStrategy>>,
}

impl StrategyRegistry {
    pub fn find_best_extraction(&self, html: &str) -> Option<&Arc<dyn ExtractionStrategy>> {
        self.extraction_strategies
            .values()
            .filter(|s| s.is_available())
            .max_by(|a, b| {
                a.confidence_score(html)
                    .partial_cmp(&b.confidence_score(html))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }
}
```

**Comparable Ecosystem Patterns:**

**Serde's Trait Composition (Serialize + Deserialize)**
```rust
// Multiple small traits compose into powerful abstractions
pub trait Serialize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer;
}

pub trait Deserialize<'de>: Sized {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>;
}

// Blanket implementations enable composition
impl<T: Serialize + Deserialize> SerdeFormat for T {}
```

**Recommendation:**
```rust
// Split traits into focused capabilities
pub trait Extractor: Send + Sync {
    async fn extract(&self, html: &str, url: &str) -> Result<ExtractionResult>;
}

pub trait SelfDescribing {
    fn name(&self) -> &str;
    fn capabilities(&self) -> Capabilities;
}

pub trait ConfidenceScoring {
    fn confidence_score(&self, html: &str) -> f64;
}

// Compose for full extractor
pub trait FullExtractor: Extractor + SelfDescribing + ConfidenceScoring {}

// Blanket impl for any type implementing all three
impl<T> FullExtractor for T
where
    T: Extractor + SelfDescribing + ConfidenceScoring
{}
```

---

## 2. Builder Patterns for Progressive Complexity

### 2.1 Current RipTide Pattern

**Source:** `crates/riptide-config/src/builder.rs`

```rust
pub trait ConfigBuilder<T> {
    fn build(self) -> BuilderResult<T>;
    fn validate(&self) -> BuilderResult<()>;
    fn load_from_env(&mut self) -> &mut Self;
}

pub struct DefaultConfigBuilder<T> {
    fields: HashMap<String, ConfigValue>,
    required_fields: Vec<String>,
    defaults: HashMap<String, ConfigValue>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> DefaultConfigBuilder<T> {
    pub fn require_field(&mut self, field: &str) -> &mut Self {
        self.required_fields.push(field.to_string());
        self
    }

    pub fn set_field(&mut self, field: &str, value: ConfigValue) -> &mut Self {
        self.fields.insert(field.to_string(), value);
        self
    }
}
```

**Strengths:**
- ✅ Method chaining for ergonomics
- ✅ Validation before build
- ✅ Environment variable integration
- ✅ Type-safe enum for values

**Weaknesses:**
- ❌ Runtime validation (not compile-time)
- ❌ String-based field names (typo-prone)
- ❌ HashMap overhead

### 2.2 Typestate Builder Pattern

**Source:** AWS SDK, `typed-builder`, `bon`

**AWS SDK Example:**
```rust
// Compile-time enforcement of required fields
let client = S3Client::builder()
    .region(Region::new("us-east-1"))  // Required
    .build();  // Won't compile without region

// Internal implementation uses typestate
pub struct S3ClientBuilder<R = Unset> {
    region: R,
    // ... other fields
}

impl S3ClientBuilder<Unset> {
    pub fn region(self, region: Region) -> S3ClientBuilder<Set<Region>> {
        S3ClientBuilder { region: Set(region) }
    }
}

impl S3ClientBuilder<Set<Region>> {
    pub fn build(self) -> S3Client {
        S3Client { region: self.region.0 }
    }
}
```

**Typed-Builder Macro:**
```rust
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct ExtractionConfig {
    #[builder(default)]
    pub timeout: Duration,

    pub url: String,  // Required field

    #[builder(default = 5)]
    pub max_retries: usize,
}

// Usage
let config = ExtractionConfig::builder()
    .url("https://example.com".to_string())
    .build();  // timeout and max_retries use defaults
```

**Recommendation for RipTide:**
```rust
// Progressive complexity with typestate
pub struct ExtractorBuilder<Stage = Initial> {
    _stage: PhantomData<Stage>,
    url: Option<String>,
    html: Option<String>,
    strategy: Option<Box<dyn ExtractionStrategy>>,
    timeout: Duration,
    retries: usize,
}

// Marker types for compile-time stages
pub struct Initial;
pub struct WithUrl;
pub struct WithContent;
pub struct Ready;

impl ExtractorBuilder<Initial> {
    pub fn new() -> Self {
        Self {
            _stage: PhantomData,
            url: None,
            html: None,
            strategy: None,
            timeout: Duration::from_secs(30),
            retries: 3,
        }
    }

    pub fn url(self, url: impl Into<String>) -> ExtractorBuilder<WithUrl> {
        ExtractorBuilder {
            _stage: PhantomData,
            url: Some(url.into()),
            html: self.html,
            strategy: self.strategy,
            timeout: self.timeout,
            retries: self.retries,
        }
    }
}

impl ExtractorBuilder<WithUrl> {
    pub fn html(self, html: impl Into<String>) -> ExtractorBuilder<Ready> {
        ExtractorBuilder {
            _stage: PhantomData,
            url: self.url,
            html: Some(html.into()),
            strategy: self.strategy,
            timeout: self.timeout,
            retries: self.retries,
        }
    }
}

impl ExtractorBuilder<Ready> {
    pub async fn extract(self) -> Result<ExtractionResult> {
        // Only available when Ready
        let strategy = self.strategy.unwrap_or_else(|| {
            Box::new(NativeExtractor::default())
        });

        strategy.extract(
            &self.html.unwrap(),
            &self.url.unwrap()
        ).await
    }
}

// Usage enforces order
let result = ExtractorBuilder::new()
    .url("https://example.com")
    .html("<html>...</html>")
    .extract()
    .await?;

// This won't compile (missing html):
// ExtractorBuilder::new().url("...").extract().await?;
```

### 2.3 Telescoping Builder Pattern

**Source:** Reqwest client

```rust
// Simple case - minimal configuration
let html = reqwest::get("https://example.com")
    .await?
    .text()
    .await?;

// Advanced case - full control
let client = reqwest::Client::builder()
    .user_agent("bot")
    .timeout(Duration::from_secs(10))
    .gzip(true)
    .build()?;

let html = client
    .get("https://example.com")
    .header("Accept", "text/html")
    .send()
    .await?
    .text()
    .await?;
```

**Recommendation for RipTide:**
```rust
// Simple API for common cases
pub async fn extract_url(url: &str) -> Result<ExtractedContent> {
    Extractor::new()
        .extract_url(url)
        .await
}

// Builder for advanced cases
pub struct Extractor {
    config: ExtractionConfig,
    strategy: Box<dyn ExtractionStrategy>,
    middleware: Vec<Box<dyn Middleware>>,
}

impl Extractor {
    pub fn new() -> Self { /* defaults */ }

    pub fn builder() -> ExtractorBuilder<Initial> {
        ExtractorBuilder::new()
    }

    // Convenience method
    pub async fn extract_url(&self, url: &str) -> Result<ExtractedContent> {
        let html = fetch(url).await?;
        self.strategy.extract(&html, url).await
    }
}
```

---

## 3. Stream-Based Processing Patterns

### 3.1 Current RipTide Pattern

**Source:** `crates/riptide-streaming/src/lib.rs`

```rust
pub struct StreamingCoordinator {
    pub streams: HashMap<Uuid, StreamInfo>,
    pub progress_tracker: ProgressTracker,
}

impl StreamingCoordinator {
    pub async fn start_stream(&mut self, extraction_id: String) -> Result<Uuid> {
        let stream_id = Uuid::new_v4();
        self.streams.insert(stream_id, StreamInfo { /* ... */ });
        self.progress_tracker.start_tracking(stream_id).await?;
        Ok(stream_id)
    }

    pub async fn update_progress(&mut self, stream_id: Uuid, processed: usize) -> Result<()> {
        // Update tracking
    }
}
```

**Weaknesses:**
- ❌ Stateful coordinator (hard to compose)
- ❌ Manual progress tracking
- ❌ No natural backpressure

### 3.2 Tokio Stream Pattern

**Source:** `tokio_stream`, `futures::stream`

```rust
use tokio_stream::{Stream, StreamExt};

// Stream-based extraction
pub struct ExtractionStream {
    tasks: VecDeque<DocumentTask>,
    extractor: Arc<UnifiedExtractor>,
    config: ParallelConfig,
}

impl Stream for ExtractionStream {
    type Item = Result<ExtractionResult>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>
    ) -> Poll<Option<Self::Item>> {
        // Implement async iteration
        if let Some(task) = self.tasks.pop_front() {
            // Spawn extraction, return Pending until ready
            Poll::Ready(Some(/* ... */))
        } else {
            Poll::Ready(None)
        }
    }
}

// Usage with natural backpressure
let results: Vec<_> = extraction_stream
    .buffer_unordered(10)  // Limit concurrency
    .collect()
    .await;
```

**Async Stream (when stable):**
```rust
use async_stream::stream;

pub fn extract_urls(urls: Vec<String>) -> impl Stream<Item = Result<ExtractionResult>> {
    stream! {
        for url in urls {
            let html = fetch(&url).await?;
            let result = extract(&html, &url).await?;
            yield Ok(result);
        }
    }
}

// Natural async iteration
pin_mut!(stream);
while let Some(result) = stream.next().await {
    match result {
        Ok(extracted) => println!("Got: {}", extracted.title),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### 3.3 Parallel Extraction Pattern

**Source:** `crates/riptide-extraction/src/parallel.rs`

**Current Implementation:**
```rust
pub struct ParallelConfig {
    pub max_concurrent: usize,
    pub timeout_per_doc: Duration,
    pub retry_failed: bool,
    pub max_retries: usize,
    pub fail_fast: bool,
}

impl ParallelConfig {
    pub fn with_max_concurrent(mut self, max: usize) -> Self {
        self.max_concurrent = max;
        self
    }
}

pub struct ParallelExtractor {
    config: ParallelConfig,
    extractor: Arc<UnifiedExtractor>,
    semaphore: Arc<Semaphore>,
}
```

**Recommendation - Stream-based:**
```rust
use futures::stream::{self, StreamExt};

pub async fn extract_parallel(
    urls: Vec<String>,
    max_concurrent: usize,
) -> impl Stream<Item = Result<ExtractionResult>> {
    stream::iter(urls)
        .map(|url| async move {
            let html = fetch(&url).await?;
            extract(&html, &url).await
        })
        .buffer_unordered(max_concurrent)  // Built-in concurrency control
}

// Usage
let results: Vec<_> = extract_parallel(urls, 10)
    .collect()
    .await;
```

**With Progress Tracking:**
```rust
pub struct ProgressStream<S> {
    inner: S,
    total: usize,
    processed: AtomicUsize,
    callback: Arc<dyn Fn(usize, usize) + Send + Sync>,
}

impl<S: Stream> Stream for ProgressStream<S> {
    type Item = S::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.inner.poll_next(cx) {
            Poll::Ready(Some(item)) => {
                let processed = self.processed.fetch_add(1, Ordering::Relaxed) + 1;
                (self.callback)(processed, self.total);
                Poll::Ready(Some(item))
            }
            other => other,
        }
    }
}

// Extension trait for ergonomics
pub trait StreamExt: Stream {
    fn with_progress(
        self,
        total: usize,
        callback: impl Fn(usize, usize) + Send + Sync + 'static,
    ) -> ProgressStream<Self>
    where
        Self: Sized,
    {
        ProgressStream {
            inner: self,
            total,
            processed: AtomicUsize::new(0),
            callback: Arc::new(callback),
        }
    }
}

// Usage
extract_parallel(urls, 10)
    .with_progress(urls.len(), |done, total| {
        println!("Progress: {}/{}", done, total);
    })
    .collect::<Vec<_>>()
    .await;
```

---

## 4. Error Handling Strategies

### 4.1 Current RipTide Pattern

**Source:** `crates/riptide-types/src/errors.rs`

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RiptideError {
    #[error("Browser error: {0}")]
    Browser(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Extraction failed: {0}")]
    Extraction(String),
}
```

**Strengths:**
- ✅ Domain-specific error types
- ✅ Auto-conversion with `#[from]`
- ✅ Custom display messages

### 4.2 Layered Error Pattern

**Source:** `anyhow`, `color-eyre`, ecosystem practice

**Recommendation:**
```rust
// Library errors - precise, actionable
#[derive(Error, Debug)]
pub enum ExtractionError {
    #[error("Invalid HTML structure: {0}")]
    InvalidHtml(String),

    #[error("Strategy '{strategy}' not available")]
    StrategyUnavailable { strategy: String },

    #[error("Timeout after {timeout:?}")]
    Timeout { timeout: Duration },

    #[error("Network error")]
    Network(#[from] reqwest::Error),
}

// Application layer - use anyhow for flexibility
pub async fn extract_with_context(url: &str) -> anyhow::Result<ExtractionResult> {
    let html = fetch(url)
        .await
        .context("Failed to fetch URL")?;

    extract(&html, url)
        .await
        .with_context(|| format!("Failed to extract content from {}", url))
}

// Public API - return Result<T, E>
pub trait Extractor {
    type Error;
    async fn extract(&self, html: &str) -> Result<ExtractionResult, Self::Error>;
}

// Internal - use anyhow for convenience
async fn internal_logic() -> anyhow::Result<()> {
    // Can use ? with different error types
    let html = fetch().await?;
    let result = parse(&html).await?;
    Ok(())
}
```

### 4.3 Fallible Iteration Pattern

**Source:** Iterator ecosystem

```rust
// Don't use this - loses errors:
let results: Vec<_> = urls.iter()
    .filter_map(|url| extract(url).await.ok())
    .collect();

// Use this - preserves all errors:
let results: Result<Vec<_>, _> = urls.iter()
    .map(|url| extract(url).await)
    .collect();  // collect() on Result<Iterator> short-circuits

// Or process errors individually:
let (successes, failures): (Vec<_>, Vec<_>) =
    stream::iter(urls)
        .then(|url| extract(url))
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .partition_result();  // From itertools
```

---

## 5. Modularity and Independent Usage

### 5.1 Current RipTide Architecture

**Source:** `Cargo.toml` workspace structure

```toml
[workspace]
members = [
    "crates/riptide-types",      # Shared types
    "crates/riptide-extraction", # Extraction logic
    "crates/riptide-spider",     # Crawling engine
    "crates/riptide-fetch",      # HTTP layer
    "crates/riptide-streaming",  # Streaming results
    "crates/riptide-api",        # HTTP API
    # ... 20+ crates
]
```

**Dependency Graph:**
```
riptide-types (bottom layer)
    ↑
    ├── riptide-extraction
    ├── riptide-spider
    └── riptide-fetch
        ↑
        └── riptide-api (top layer)
```

**Strengths:**
- ✅ Clear separation of concerns
- ✅ Bottom-up dependencies
- ✅ Can use extraction without spider

**Pattern from Tokio Ecosystem:**
```toml
[dependencies]
# Use only what you need
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }

# Optional features for progressive complexity
riptide = { version = "0.9", features = ["extraction"] }
riptide = { version = "0.9", features = ["extraction", "spider", "streaming"] }
```

### 5.2 Feature Flag Strategy

**Recommendation:**
```toml
# crates/riptide/Cargo.toml
[features]
default = ["extraction"]

# Core features (independent)
extraction = ["scraper", "lol_html"]
spider = ["riptide-spider"]
streaming = ["tokio-stream", "riptide-streaming"]
intelligence = ["riptide-intelligence"]

# Convenience bundles
full = ["extraction", "spider", "streaming", "intelligence"]
minimal = ["extraction"]
```

**Conditional Compilation:**
```rust
#[cfg(feature = "spider")]
pub use riptide_spider::Spider;

#[cfg(feature = "streaming")]
pub mod streaming {
    pub use riptide_streaming::*;
}

// Always available
pub mod extraction {
    pub use riptide_extraction::*;
}
```

### 5.3 Facade Pattern for Simplicity

**Source:** `crates/riptide-facade/src/facades/extractor.rs`

```rust
// High-level facade for common cases
pub struct RipTide {
    extractor: Arc<UnifiedExtractor>,
    spider: Option<Arc<Spider>>,
}

impl RipTide {
    pub fn new() -> Self { /* ... */ }

    // Simple API
    pub async fn extract_url(&self, url: &str) -> Result<ExtractedContent> {
        let html = fetch(url).await?;
        self.extractor.extract(&html, url).await
    }

    // Access to underlying components
    pub fn extractor(&self) -> &UnifiedExtractor {
        &self.extractor
    }

    #[cfg(feature = "spider")]
    pub fn spider(&self) -> Option<&Spider> {
        self.spider.as_ref().map(|s| s.as_ref())
    }
}
```

---

## 6. Comparable Rust Projects Analysis

### 6.1 Scraper Crate

**Pattern:** Simple, focused API
```rust
use scraper::{Html, Selector};

let html = Html::parse_document(html_str);
let selector = Selector::parse("div.article").unwrap();

for element in html.select(&selector) {
    println!("{}", element.text().collect::<String>());
}
```

**Lessons:**
- ✅ Single-purpose types (Html, Selector)
- ✅ Iterator-based traversal
- ✅ Panic on parse errors (simplified API)

### 6.2 Reqwest Crate

**Pattern:** Builder + async execution
```rust
let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(10))
    .build()?;

let response = client
    .get("https://example.com")
    .header("User-Agent", "bot")
    .send()
    .await?;
```

**Lessons:**
- ✅ Two-stage configuration (client + request)
- ✅ Sensible defaults
- ✅ Method chaining for optional config
- ✅ Finalize with `send()` or `execute()`

### 6.3 Actix-Web / Axum

**Pattern:** Middleware/layer composition
```rust
// Axum layers
let app = Router::new()
    .route("/extract", post(extract_handler))
    .layer(TimeoutLayer::new(Duration::from_secs(30)))
    .layer(CompressionLayer::new())
    .layer(TraceLayer::new_for_http());
```

**Lesson for RipTide:**
```rust
pub struct Extractor {
    strategy: Box<dyn ExtractionStrategy>,
    middleware: Vec<Box<dyn Middleware>>,
}

#[async_trait]
pub trait Middleware: Send + Sync {
    async fn process(
        &self,
        html: &str,
        next: &dyn Fn(&str) -> BoxFuture<'_, Result<ExtractionResult>>,
    ) -> Result<ExtractionResult>;
}

// Usage
let extractor = Extractor::builder()
    .strategy(NativeExtractor::new())
    .middleware(TimeoutMiddleware::new(Duration::from_secs(30)))
    .middleware(RetryMiddleware::new(3))
    .build();
```

### 6.4 AWS SDK

**Pattern:** Typestate + fluent builders
```rust
let client = S3Client::from_conf(config);

let result = client
    .get_object()
    .bucket("my-bucket")
    .key("file.txt")
    .send()  // Compile error if bucket/key missing
    .await?;
```

**Lessons:**
- ✅ Required fields enforced at compile-time
- ✅ Fluent API hides complexity
- ✅ Generated code (reduces manual maintenance)

---

## 7. Recommended Architecture for RipTide

### 7.1 Three-Tier API Design

```rust
// TIER 1: Simple functions (one-liners)
pub async fn extract_url(url: &str) -> Result<ExtractedContent> {
    let html = fetch(url).await?;
    extract_html(&html, url).await
}

pub async fn extract_html(html: &str, url: &str) -> Result<ExtractedContent> {
    UnifiedExtractor::default()
        .extract(html, url)
        .await
}

// TIER 2: Builder for common customization
pub struct ExtractorBuilder { /* ... */ }

let result = Extractor::builder()
    .strategy(StrategyType::Native)
    .timeout(Duration::from_secs(30))
    .build()?
    .extract_url("https://example.com")
    .await?;

// TIER 3: Full control with traits
struct CustomExtractor;

#[async_trait]
impl ExtractionStrategy for CustomExtractor {
    async fn extract(&self, html: &str, url: &str) -> Result<ExtractionResult> {
        // Custom logic
    }

    fn name(&self) -> &str { "custom" }
    fn capabilities(&self) -> StrategyCapabilities { /* ... */ }
}

let registry = StrategyRegistry::builder()
    .with_extraction(Arc::new(CustomExtractor))
    .build();
```

### 7.2 Stream-First Architecture

```rust
// Primary API returns streams
pub fn extract_urls(urls: Vec<String>) -> impl Stream<Item = Result<ExtractionResult>> {
    stream::iter(urls)
        .map(|url| extract_url(url))
        .buffer_unordered(10)
}

// Convenience collectors
impl<S: Stream<Item = Result<T>>> StreamExt for S {
    async fn collect_ok(self) -> Vec<T> {
        self.filter_map(|r| r.ok()).collect().await
    }

    async fn try_collect(self) -> Result<Vec<T>> {
        self.collect::<Vec<_>>()
            .await
            .into_iter()
            .collect()
    }
}

// Usage
let all_results = extract_urls(urls).try_collect().await?;
let successes = extract_urls(urls).collect_ok().await;
```

### 7.3 Middleware Pattern

```rust
pub trait ExtractorMiddleware: Send + Sync {
    async fn process(
        &self,
        request: ExtractionRequest,
        next: Next<'_>,
    ) -> Result<ExtractionResult>;
}

pub struct Next<'a> {
    inner: &'a dyn Fn(ExtractionRequest) -> BoxFuture<'a, Result<ExtractionResult>>,
}

impl<'a> Next<'a> {
    pub async fn run(self, req: ExtractionRequest) -> Result<ExtractionResult> {
        (self.inner)(req).await
    }
}

// Middleware implementations
pub struct RetryMiddleware { max_retries: usize }
pub struct TimeoutMiddleware { timeout: Duration }
pub struct CacheMiddleware { cache: Arc<dyn Cache> }

// Builder with middleware
let extractor = Extractor::builder()
    .layer(RetryMiddleware::new(3))
    .layer(TimeoutMiddleware::new(Duration::from_secs(30)))
    .layer(CacheMiddleware::new(cache))
    .build();
```

---

## 8. Code Examples

### 8.1 Complete Example: Simple to Advanced

```rust
// ========================================
// SIMPLE: Just extract a URL
// ========================================
use riptide::extract_url;

let content = extract_url("https://example.com").await?;
println!("Title: {}", content.title);

// ========================================
// INTERMEDIATE: Configure extraction
// ========================================
use riptide::Extractor;

let extractor = Extractor::builder()
    .timeout(Duration::from_secs(30))
    .retries(3)
    .strategy(StrategyType::Native)
    .build()?;

let content = extractor.extract_url("https://example.com").await?;

// ========================================
// ADVANCED: Custom strategy + middleware
// ========================================
use riptide::{ExtractionStrategy, ExtractorBuilder};

struct MyCustomStrategy;

#[async_trait]
impl ExtractionStrategy for MyCustomStrategy {
    async fn extract(&self, html: &str, url: &str) -> Result<ExtractionResult> {
        // Custom extraction logic
        Ok(ExtractionResult {
            content: ExtractedContent {
                title: "Custom".to_string(),
                // ...
            },
            quality: ExtractionQuality::high(),
            performance: None,
            metadata: HashMap::new(),
        })
    }

    fn name(&self) -> &str { "custom" }
    fn capabilities(&self) -> StrategyCapabilities {
        StrategyCapabilities {
            strategy_type: "custom".to_string(),
            supported_content_types: vec!["text/html".to_string()],
            performance_tier: PerformanceTier::Fast,
            resource_requirements: ResourceRequirements {
                memory_tier: ResourceTier::Low,
                cpu_tier: ResourceTier::Low,
                requires_network: false,
                external_dependencies: vec![],
            },
        }
    }
}

let extractor = ExtractorBuilder::new()
    .strategy(Box::new(MyCustomStrategy))
    .middleware(TimeoutMiddleware::new(Duration::from_secs(30)))
    .middleware(RetryMiddleware::new(3))
    .build();

// ========================================
// STREAMING: Batch processing
// ========================================
use riptide::stream::extract_urls;
use futures::StreamExt;

let urls = vec![
    "https://example.com",
    "https://example.org",
    "https://example.net",
];

let results: Vec<_> = extract_urls(urls)
    .with_progress(|done, total| {
        println!("Progress: {}/{}", done, total);
    })
    .buffer_unordered(10)
    .collect()
    .await;
```

### 8.2 Spider Integration Example

```rust
use riptide::spider::{Spider, SpiderConfig};

// Simple crawl
let spider = Spider::new(SpiderConfig::default()).await?;
let results = spider.crawl(vec![seed_url]).await?;

// Advanced configuration
let config = SpiderConfig::builder()
    .max_depth(3)
    .max_pages(1000)
    .respect_robots_txt(true)
    .concurrent_requests(10)
    .build();

let spider = Spider::new(config).await?;

// Stream results as they come
let crawl_stream = spider.crawl_stream(vec![seed_url]);

pin_mut!(crawl_stream);
while let Some(result) = crawl_stream.next().await {
    match result {
        Ok(page) => println!("Crawled: {}", page.url),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

---

## 9. Key Recommendations

### 9.1 Immediate Actions

1. **Adopt Typestate Builder Pattern**
   - Use `typed-builder` or manual typestate for compile-time safety
   - Eliminates runtime validation errors
   - Better IDE autocomplete

2. **Stream-First API**
   - Primary APIs return `impl Stream<Item = Result<T>>`
   - Provides natural backpressure
   - Composable with standard stream operators

3. **Trait Composition**
   - Split large traits into focused capabilities
   - Use blanket implementations for composition
   - Easier to test and extend

4. **Middleware Architecture**
   - Tower-like middleware for cross-cutting concerns
   - Retry, timeout, caching as middleware
   - Composable, testable, reusable

### 9.2 Long-Term Architecture

1. **Three-Tier API**
   - Simple functions for 80% use cases
   - Builder for 15% customization
   - Traits for 5% advanced users

2. **Feature-Flag Modularity**
   - Independent crates with optional integration
   - Pay-for-what-you-use compilation
   - Clear dependency graph

3. **Error Handling**
   - `thiserror` for library errors (precise, actionable)
   - `anyhow` for application errors (flexible context)
   - Fallible iteration patterns (preserve all errors)

---

## 10. Trade-offs and Considerations

### 10.1 Async Trait Overhead

**Issue:** `async_trait` macro has small runtime cost (heap allocation per call)

**Alternatives:**
- Use GATs when stable (zero-cost)
- Use `impl Future` in trait return types (nightly)
- Accept small cost for better ergonomics

**Recommendation:** Keep `async_trait` until GATs stabilize. The ergonomics win outweighs the small performance cost for I/O-bound operations.

### 10.2 Builder Pattern Complexity

**Issue:** Typestate builders create large amount of types

**Alternatives:**
- Use runtime validation with good error messages
- Use procedural macro (`typed-builder`, `bon`)
- Hybrid: typestate for required fields, runtime for optional

**Recommendation:** Use `typed-builder` macro for best of both worlds.

### 10.3 Stream vs Iterator

**Issue:** Streams are less familiar than iterators

**Alternatives:**
- Provide `collect()` helpers for common cases
- Document stream patterns clearly
- Offer both APIs (stream primary, iterator secondary)

**Recommendation:** Stream-first with convenience collectors. Most users will just use `collect()` or `try_collect()`.

---

## 11. Conclusion

The most Rust-idiomatic way to build RipTide's composable web extraction API is:

1. **Trait-based extensibility** with `async_trait` for pluggable extractors
2. **Typestate builder pattern** for compile-time safety and progressive complexity
3. **Stream-based processing** with `tokio::stream` for natural async iteration
4. **Layered error handling** using `thiserror` + `anyhow` appropriately
5. **Middleware composition** like Tower/Axum for cross-cutting concerns
6. **Three-tier API design** (simple functions → builder → traits)
7. **Feature flags** for independent module usage

This approach balances:
- ✅ **Simplicity** for common cases (one-liner functions)
- ✅ **Power** for advanced users (trait system, middleware)
- ✅ **Safety** with compile-time guarantees (typestate)
- ✅ **Performance** with zero-cost abstractions where possible
- ✅ **Composability** through streams and trait composition

---

## 12. References

### Ecosystem Projects Analyzed
- **Reqwest**: Builder pattern, async HTTP client
- **Scraper**: Simple, focused HTML parsing
- **Tokio**: Stream patterns, async runtime
- **Axum/Tower**: Middleware composition
- **AWS SDK**: Typestate builders
- **Serde**: Trait composition patterns

### Further Reading
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Zero Cost Abstractions](https://blog.rust-lang.org/2015/05/11/traits.html)
- [Builder Pattern in Rust](https://doc.rust-lang.org/1.0.0/style/ownership/builders.html)
- [Async Rust Patterns](https://rust-lang.github.io/async-book/)
- [Stream Processing in Rust](https://tokio.rs/tokio/tutorial/streams)

---

**Research Conducted By:** Research Agent
**Project:** RipTide Web Extraction Framework
**Last Updated:** 2025-01-04
