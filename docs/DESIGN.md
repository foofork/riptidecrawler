# RipTide Design Documentation

> **Version**: 2.0.0
> **Last Updated**: 2025-10-24
> **Status**: Production Ready

---

## Table of Contents

1. [Design Philosophy](#design-philosophy)
2. [Design Patterns](#design-patterns)
3. [Module Design](#module-design)
4. [API Design](#api-design)
5. [Data Models](#data-models)
6. [Error Handling](#error-handling)
7. [Configuration Design](#configuration-design)
8. [Testing Strategy](#testing-strategy)
9. [Performance Design](#performance-design)

---

## Design Philosophy

### Core Principles

#### 1. Simplicity Over Complexity

**Guideline**: Choose the simplest solution that meets requirements

**Examples**:
- Builder pattern over complex constructors
- Facade layer for common use cases
- Sensible defaults with override capability

```rust
// Simple: Builder with defaults
let scraper = Riptide::builder()
    .build_scraper()
    .await?;

// Advanced: Full configuration
let scraper = Riptide::builder()
    .user_agent("Custom/1.0")
    .timeout_secs(30)
    .max_concurrency(20)
    .cache_ttl_hours(24)
    .build_scraper()
    .await?;
```

#### 2. Composability

**Guideline**: Build complex functionality from simple, reusable components

**Crate Relationships**:
```
riptide-facade (high-level)
    ↓ composes
riptide-spider, riptide-fetch, riptide-extraction (mid-level)
    ↓ use
riptide-types, riptide-config, riptide-cache (low-level)
```

**Benefits**:
- Testable in isolation
- Reusable across contexts
- Clear dependency graph
- Easier to understand

#### 3. Type Safety

**Guideline**: Use Rust's type system to prevent errors at compile time

**Examples**:
```rust
// Type-safe URLs
pub struct Url(url::Url);  // Not String

// Type-safe configuration
pub struct CrawlConfig {
    concurrency: NonZeroUsize,  // Can't be zero
    timeout: Duration,          // Not raw seconds
    max_depth: Option<u32>,     // Explicit optionality
}

// Type-safe states
pub enum CrawlState {
    Pending,
    Running { start_time: Instant },
    Completed { result: CrawlResult },
    Failed { error: CrawlError },
}
```

#### 4. Performance by Default

**Guideline**: Optimize common paths without sacrificing correctness

**Strategies**:
- Zero-copy parsing where possible
- Lazy evaluation
- Connection pooling
- Async I/O throughout
- WASM for CPU-intensive tasks

---

## Design Patterns

### 1. Facade Pattern

**Purpose**: Provide simplified interface to complex subsystems

**Implementation**: `riptide-facade`

```rust
pub struct ScraperFacade {
    fetch_engine: Arc<FetchEngine>,
    cache_manager: Arc<CacheManager>,
    wasm_extractor: Arc<WasmExtractor>,
    strategy_manager: Arc<StrategyManager>,
}

impl ScraperFacade {
    // Simple interface hiding complex coordination
    pub async fn fetch_html(&self, url: &str) -> Result<String> {
        // 1. Check cache
        if let Some(cached) = self.cache_manager.get(url).await? {
            return Ok(cached.html);
        }

        // 2. Fetch
        let response = self.fetch_engine.fetch(url).await?;

        // 3. Cache
        self.cache_manager.set(url, &response).await?;

        Ok(response.html)
    }
}
```

**Benefits**:
- Reduces learning curve
- Encapsulates complexity
- Provides sensible defaults
- Easy to mock for testing

### 2. Strategy Pattern

**Purpose**: Select algorithm at runtime based on context

**Implementation**: `riptide-extraction/strategies`

```rust
pub trait ExtractionStrategy: Send + Sync {
    fn name(&self) -> &str;
    fn can_handle(&self, content: &Content) -> bool;
    async fn extract(&self, content: &Content) -> Result<ExtractedDoc>;
}

pub struct StrategyManager {
    strategies: Vec<Box<dyn ExtractionStrategy>>,
    fallback: Box<dyn ExtractionStrategy>,
}

impl StrategyManager {
    pub async fn select_and_extract(&self, content: &Content)
        -> Result<ExtractedDoc> {
        // Find first matching strategy
        for strategy in &self.strategies {
            if strategy.can_handle(content) {
                return strategy.extract(content).await;
            }
        }

        // Use fallback
        self.fallback.extract(content).await
    }
}
```

**Strategies**:
- `WasmExtractor` - High-performance, sandboxed
- `CssExtractor` - CSS selector-based
- `RegexExtractor` - Pattern matching
- `LlmExtractor` - AI-powered (via riptide-intelligence)

### 3. Builder Pattern

**Purpose**: Construct complex objects with optional parameters

**Implementation**: `riptide-facade/builder`

```rust
pub struct RiptideBuilder {
    user_agent: Option<String>,
    timeout: Option<Duration>,
    concurrency: Option<usize>,
    cache_ttl: Option<Duration>,
    redis_url: Option<String>,
    enable_stealth: bool,
}

impl RiptideBuilder {
    pub fn user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = Some(ua.into());
        self
    }

    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.timeout = Some(Duration::from_secs(secs));
        self
    }

    pub async fn build_scraper(self) -> Result<ScraperFacade> {
        // Apply defaults
        let config = CrawlConfig {
            user_agent: self.user_agent
                .unwrap_or_else(|| "RipTide/2.0".to_string()),
            timeout: self.timeout
                .unwrap_or(Duration::from_secs(30)),
            // ...
        };

        ScraperFacade::new(config).await
    }
}
```

### 4. Repository Pattern

**Purpose**: Abstract data access from business logic

**Implementation**: `riptide-cache`, `riptide-persistence`

```rust
#[async_trait]
pub trait CrawlRepository: Send + Sync {
    async fn get(&self, url: &Url) -> Result<Option<CrawlResult>>;
    async fn set(&self, url: &Url, result: &CrawlResult) -> Result<()>;
    async fn delete(&self, url: &Url) -> Result<()>;
    async fn exists(&self, url: &Url) -> Result<bool>;
}

// Redis implementation
pub struct RedisCrawlRepository {
    client: redis::Client,
    ttl: Duration,
}

#[async_trait]
impl CrawlRepository for RedisCrawlRepository {
    async fn get(&self, url: &Url) -> Result<Option<CrawlResult>> {
        let key = self.make_key(url);
        let mut conn = self.client.get_async_connection().await?;

        let value: Option<String> = conn.get(&key).await?;
        match value {
            Some(json) => Ok(Some(serde_json::from_str(&json)?)),
            None => Ok(None),
        }
    }
    // ...
}
```

### 5. Circuit Breaker Pattern

**Purpose**: Prevent cascading failures in distributed systems

**Implementation**: `riptide-spider/circuit`

```rust
pub struct CircuitBreaker {
    failure_threshold: u32,
    timeout: Duration,
    state: Arc<RwLock<CircuitState>>,
}

enum CircuitState {
    Closed { consecutive_failures: u32 },
    Open { opened_at: Instant },
    HalfOpen,
}

impl CircuitBreaker {
    pub async fn call<F, T>(&self, f: F) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        // Check state
        let state = self.state.read().await;
        match *state {
            CircuitState::Open { opened_at } => {
                if opened_at.elapsed() > self.timeout {
                    // Try to recover
                    drop(state);
                    self.transition_to_half_open().await;
                } else {
                    return Err(Error::CircuitOpen);
                }
            }
            _ => {}
        }
        drop(state);

        // Execute
        match f.await {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(e) => {
                self.on_failure().await;
                Err(e)
            }
        }
    }
}
```

### 6. Observer Pattern

**Purpose**: Notify multiple components of state changes

**Implementation**: `riptide-events`

```rust
pub struct EventBus {
    subscribers: Arc<RwLock<HashMap<EventType, Vec<Subscriber>>>>,
}

pub struct Subscriber {
    id: Uuid,
    callback: Box<dyn Fn(SystemEvent) + Send + Sync>,
}

impl EventBus {
    pub async fn publish(&self, event: SystemEvent) {
        let event_type = event.event_type();
        let subscribers = self.subscribers.read().await;

        if let Some(subs) = subscribers.get(&event_type) {
            for sub in subs {
                (sub.callback)(event.clone());
            }
        }
    }

    pub async fn subscribe<F>(&self, event_type: EventType, callback: F)
        -> Uuid
    where
        F: Fn(SystemEvent) + Send + Sync + 'static,
    {
        let id = Uuid::new_v4();
        let subscriber = Subscriber {
            id,
            callback: Box::new(callback),
        };

        let mut subs = self.subscribers.write().await;
        subs.entry(event_type)
            .or_insert_with(Vec::new)
            .push(subscriber);

        id
    }
}
```

---

## Module Design

### Crate Organization Principles

#### 1. Single Responsibility

Each crate has one clear purpose:

- `riptide-fetch` → HTTP client operations
- `riptide-spider` → Crawling logic
- `riptide-extraction` → Content parsing
- `riptide-cache` → Caching layer

#### 2. Dependency Direction

```
High-level modules depend on low-level modules (never vice versa)

riptide-api
    ↓
riptide-facade
    ↓
riptide-spider, riptide-fetch, riptide-extraction
    ↓
riptide-types, riptide-config, riptide-cache
```

#### 3. Interface Segregation

Export only what's necessary:

```rust
// riptide-extraction/lib.rs
pub use css_extraction::{extract, default_selectors};  // Public API
pub use processor::HtmlProcessor;

// Internal modules not exposed
mod internal_utils;  // Private
```

### Module Structure Template

```
crate-name/
├── src/
│   ├── lib.rs           # Public API, re-exports
│   ├── config.rs        # Configuration types
│   ├── error.rs         # Error types
│   ├── types.rs         # Domain types
│   ├── client.rs        # Main client/facade
│   ├── internal/        # Private implementation details
│   │   ├── mod.rs
│   │   ├── foo.rs
│   │   └── bar.rs
│   └── tests/           # Integration tests
│       └── mod.rs
├── Cargo.toml
└── README.md
```

---

## API Design

### RESTful Endpoint Design

#### Naming Conventions

```
Resource-based:
  GET    /crawl/{id}           # Get crawl status
  POST   /crawl                # Start crawl
  DELETE /crawl/{id}           # Cancel crawl

Action-based (when needed):
  POST /deepsearch             # Search-driven crawl
  POST /browser/screenshot     # Capture screenshot
  POST /extract                # Extract from HTML
```

#### Request/Response Design

**Request Structure**:
```rust
#[derive(Deserialize, Validate)]
pub struct CrawlRequest {
    #[validate(length(min = 1, max = 1000))]
    pub urls: Vec<String>,

    #[serde(default)]
    pub options: CrawlOptions,
}

#[derive(Deserialize, Default)]
pub struct CrawlOptions {
    #[serde(default = "default_concurrency")]
    pub concurrency: usize,

    #[serde(default)]
    pub follow_links: bool,

    #[serde(default)]
    pub max_depth: Option<u32>,
}

fn default_concurrency() -> usize { 16 }
```

**Response Structure**:
```rust
#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ErrorDetails>,
    pub metadata: ResponseMetadata,
}

#[derive(Serialize)]
pub struct ResponseMetadata {
    pub request_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub duration_ms: u64,
}
```

#### Error Responses

```rust
#[derive(Serialize)]
pub struct ErrorDetails {
    pub code: String,           // Machine-readable
    pub message: String,        // Human-readable
    pub details: Option<Value>, // Additional context
}

// Example
{
    "success": false,
    "error": {
        "code": "INVALID_URL",
        "message": "The provided URL is not valid",
        "details": {
            "url": "ht!tp://invalid",
            "reason": "Invalid scheme"
        }
    }
}
```

### Versioning Strategy

**URL Versioning**:
```
/v1/crawl
/v2/crawl  (future)
```

**Header Versioning** (alternative):
```
Accept: application/vnd.riptide.v1+json
```

---

## Data Models

### Core Types

#### URL Handling

```rust
use url::Url;

/// Type-safe URL wrapper
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RiptideUrl(Url);

impl RiptideUrl {
    /// Normalize URL for deduplication
    pub fn canonical(&self) -> Self {
        let mut url = self.0.clone();

        // Remove fragment
        url.set_fragment(None);

        // Normalize path
        if url.path().ends_with('/') && url.path() != "/" {
            url.set_path(url.path().trim_end_matches('/'));
        }

        // Sort query parameters
        let mut params: Vec<_> = url.query_pairs().collect();
        params.sort();
        url.set_query(Some(&params.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&")));

        Self(url)
    }
}
```

#### Extracted Content

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedDoc {
    pub url: String,
    pub title: Option<String>,
    pub content: String,
    pub markdown: Option<String>,

    #[serde(default)]
    pub metadata: Metadata,

    #[serde(default)]
    pub links: Vec<Link>,

    #[serde(default)]
    pub media: Vec<Media>,

    #[serde(default)]
    pub tables: Vec<Table>,

    pub extracted_at: DateTime<Utc>,
    pub strategy_used: String,

    #[serde(default)]
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Metadata {
    pub description: Option<String>,
    pub author: Option<String>,
    pub published_date: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
    pub language: Option<String>,

    #[serde(flatten)]
    pub custom: HashMap<String, Value>,
}
```

#### Crawl Result

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlResult {
    pub url: String,
    pub status: CrawlStatus,
    pub extracted: Option<ExtractedDoc>,
    pub error: Option<String>,

    pub http_status: Option<u16>,
    pub duration_ms: u64,

    #[serde(default)]
    pub redirects: Vec<String>,

    pub crawled_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CrawlStatus {
    Success,
    PartialSuccess,  // Fetched but extraction failed
    Failed,
    Skipped,         // Already crawled, cached
    Blocked,         // robots.txt, rate limit
}
```

### Configuration Models

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct RiptideConfig {
    pub server: ServerConfig,
    pub crawl: CrawlConfig,
    pub extraction: ExtractionConfig,
    pub cache: CacheConfig,
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CrawlConfig {
    #[serde(default = "default_concurrency")]
    pub concurrency: usize,

    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    #[serde(default)]
    pub user_agent: String,

    #[serde(default = "default_true")]
    pub follow_redirects: bool,

    #[serde(default)]
    pub max_redirects: Option<u32>,

    #[serde(default)]
    pub proxy: Option<ProxyConfig>,
}
```

---

## Error Handling

### Error Type Hierarchy

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RiptideError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("URL parsing error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("Extraction failed: {0}")]
    Extraction(String),

    #[error("Cache error: {0}")]
    Cache(#[from] CacheError),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Browser error: {0}")]
    Browser(String),

    #[error("Timeout after {0:?}")]
    Timeout(Duration),

    #[error("Rate limit exceeded")]
    RateLimit,

    #[error("Circuit breaker open")]
    CircuitOpen,
}

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Redis connection failed: {0}")]
    Connection(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Key not found: {0}")]
    NotFound(String),
}
```

### Error Context

```rust
use anyhow::{Context, Result};

pub async fn fetch_and_extract(url: &str) -> Result<ExtractedDoc> {
    let response = fetch(url)
        .await
        .context("Failed to fetch URL")?;

    let doc = extract(&response.body)
        .await
        .with_context(|| format!("Extraction failed for {}", url))?;

    Ok(doc)
}
```

### Error Recovery

```rust
pub async fn fetch_with_retry(url: &str, config: &RetryConfig)
    -> Result<Response> {
    let mut attempts = 0;
    let mut delay = config.initial_delay;

    loop {
        match fetch_once(url).await {
            Ok(response) => return Ok(response),
            Err(e) if attempts < config.max_retries => {
                attempts += 1;
                tracing::warn!(
                    "Fetch failed (attempt {}/{}): {}",
                    attempts, config.max_retries, e
                );

                // Exponential backoff with jitter
                let jitter = if config.jitter {
                    fastrand::u64(0..delay.as_millis() as u64 / 4)
                } else {
                    0
                };
                tokio::time::sleep(delay + Duration::from_millis(jitter)).await;

                delay = (delay * config.backoff_multiplier as u32)
                    .min(config.max_delay);
            }
            Err(e) => return Err(e),
        }
    }
}
```

---

## Configuration Design

### Layered Configuration

```
Priority (highest to lowest):
1. CLI arguments          (--timeout 60)
2. Environment variables  (RIPTIDE_TIMEOUT=60)
3. Config file           (timeout_secs: 60)
4. Code defaults         (Duration::from_secs(30))
```

### Configuration Loading

```rust
pub fn load_config() -> Result<RiptideConfig> {
    // 1. Load from file
    let mut config: RiptideConfig = if let Ok(path) = env::var("RIPTIDE_CONFIG") {
        let contents = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config: {}", path))?;
        serde_yaml::from_str(&contents)?
    } else {
        RiptideConfig::default()
    };

    // 2. Override with environment variables
    if let Ok(timeout) = env::var("RIPTIDE_TIMEOUT") {
        config.crawl.timeout_secs = timeout.parse()?;
    }

    // 3. Validate
    config.validate()?;

    Ok(config)
}
```

---

## Testing Strategy

### Test Pyramid

```
        ┌────────┐
        │  E2E   │  (5% - Full system)
        ├────────┤
        │Integration│ (15% - Multiple modules)
        ├──────────┤
        │   Unit    │ (80% - Individual functions)
        └──────────┘
```

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_canonicalization() {
        let url1 = RiptideUrl::parse("https://example.com/?b=2&a=1").unwrap();
        let url2 = RiptideUrl::parse("https://example.com/?a=1&b=2").unwrap();

        assert_eq!(url1.canonical(), url2.canonical());
    }

    #[tokio::test]
    async fn test_fetch_success() {
        let config = FetchConfig::default();
        let engine = FetchEngine::new(config);

        let result = engine.fetch("https://httpbin.org/get").await;
        assert!(result.is_ok());
    }
}
```

### Integration Tests

```rust
// tests/integration_test.rs
use riptide_facade::Riptide;

#[tokio::test]
async fn test_full_crawl_pipeline() {
    let scraper = Riptide::builder()
        .build_scraper()
        .await
        .unwrap();

    let result = scraper
        .fetch_and_extract("https://example.com")
        .await
        .unwrap();

    assert!(result.title.is_some());
    assert!(!result.content.is_empty());
}
```

### Mock Testing

```rust
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

#[tokio::test]
async fn test_with_mock_server() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/test"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string("<html><body>Test</body></html>"))
        .mount(&mock_server)
        .await;

    let url = format!("{}/test", mock_server.uri());
    let result = fetch(&url).await.unwrap();

    assert_eq!(result.status, 200);
}
```

---

## Performance Design

### Async Design Patterns

```rust
// ✅ Good: Stream processing
use futures::stream::{self, StreamExt};

pub async fn crawl_batch(urls: Vec<Url>) -> Vec<Result<CrawlResult>> {
    stream::iter(urls)
        .map(|url| crawl_one(url))
        .buffer_unordered(16)  // Concurrent limit
        .collect()
        .await
}

// ❌ Bad: Sequential processing
pub async fn crawl_batch_slow(urls: Vec<Url>) -> Vec<Result<CrawlResult>> {
    let mut results = Vec::new();
    for url in urls {
        results.push(crawl_one(url).await);
    }
    results
}
```

### Memory Optimization

```rust
// Use Arc for shared data
pub struct Spider {
    config: Arc<SpiderConfig>,          // Shared
    frontier: Arc<FrontierManager>,     // Shared
    state: Arc<RwLock<CrawlState>>,     // Mutable shared
}

// Avoid cloning large data
pub async fn extract(html: &str) -> Result<ExtractedDoc> {
    // Work with references
    let doc = parse_html(html)?;
    Ok(doc)
}
```

### Caching Strategy

```rust
pub struct CacheManager {
    redis: redis::Client,
    local: DashMap<String, CachedValue>,  // L1 cache
}

impl CacheManager {
    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        // Check L1 (local)
        if let Some(value) = self.local.get(key) {
            if !value.is_expired() {
                return Ok(Some(value.data.clone()));
            }
        }

        // Check L2 (Redis)
        let mut conn = self.redis.get_async_connection().await?;
        let value: Option<String> = conn.get(key).await?;

        // Populate L1
        if let Some(ref data) = value {
            self.local.insert(key.to_string(), CachedValue {
                data: data.clone(),
                expires_at: Instant::now() + Duration::from_secs(60),
            });
        }

        Ok(value)
    }
}
```

---

## Conclusion

RipTide's design emphasizes:

✅ **Simplicity**: Easy to understand and use
✅ **Composability**: Build complex features from simple parts
✅ **Type Safety**: Catch errors at compile time
✅ **Performance**: Optimized for common cases
✅ **Reliability**: Comprehensive error handling
✅ **Testability**: Modular design enables thorough testing

For implementation details, see `ARCHITECTURE.md` and individual crate documentation.
