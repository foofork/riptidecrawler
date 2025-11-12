# riptide-types

🎯 **Domain Layer - Pure Business Logic**

Core type definitions and port interfaces for the RipTide web scraping framework. This crate serves as the **contract catalog** for the entire system, defining domain models and backend-agnostic port traits that enable dependency inversion and hexagonal architecture.

## Quick Overview

`riptide-types` is the foundation of RipTide's domain layer. It provides:
- **30+ port trait definitions** for infrastructure concerns (persistence, events, caching, browser automation, etc.)
- **Domain types** representing business concepts (extraction results, crawl requests, configurations)
- **Error types** for consistent error handling across the system
- **Zero infrastructure dependencies** - pure domain logic only

This crate exists to **break circular dependencies** and maintain **clean architecture boundaries**. All other RipTide crates depend on `riptide-types`, but it depends on no other RipTide crates.

## Key Concepts

### 1. Port Traits (Hexagonal Architecture)

Port traits define **what** the application needs without specifying **how** it's implemented. Concrete implementations (adapters) live in infrastructure crates.

```rust
use riptide_types::ports::{Repository, EventBus, CacheStorage};

// Domain layer uses ports, never concrete implementations
async fn save_user(
    repo: &dyn Repository<User>,
    events: &dyn EventBus,
    cache: &dyn CacheStorage,
) -> Result<()> {
    // Save to database (implementation unknown)
    repo.save(&user).await?;

    // Publish domain event (message broker unknown)
    events.publish(event).await?;

    // Cache result (Redis/Memcached/in-memory unknown)
    cache.set("user:123", &user, Duration::from_secs(300)).await?;

    Ok(())
}
```

### 2. Domain Models

Core business types representing extraction and crawling concepts:

```rust
use riptide_types::{
    ExtractedContent,
    ExtractionQuality,
    CrawlRequest,
    Priority,
};

// Extracted content from a web page
let content = ExtractedContent {
    title: "Article Title".to_string(),
    content: "Full text content...".to_string(),
    summary: Some("Brief summary...".to_string()),
    url: "https://example.com".to_string(),
    metadata: HashMap::new(),
};

// Quality metrics for extraction
let quality = ExtractionQuality {
    content_quality: 0.95,
    title_quality: 0.90,
    structure_score: 0.85,
    relevance_score: Some(0.92),
};

// Crawl request with priority
let request = CrawlRequest::new(url)
    .with_priority(Priority::High)
    .with_depth(2)
    .with_parent(parent_url);
```

### 3. Configuration Types

Type-safe configuration for extraction and crawling operations:

```rust
use riptide_types::{ExtractionMode, RenderMode, OutputFormat, ChunkingConfig};

// Extraction modes
let mode = ExtractionMode::Article; // or Full, Metadata, Custom

// Rendering modes
let render = RenderMode::Dynamic; // Headless browser rendering
let render = RenderMode::Static;  // Plain HTTP fetch
let render = RenderMode::Adaptive; // Smart selection

// Output formats
let format = OutputFormat::Document; // Single document
let format = OutputFormat::Chunked;  // Split into chunks
let format = OutputFormat::NdJson;   // Newline-delimited JSON

// Chunking configuration
let chunking = ChunkingConfig {
    chunk_size: 1000,
    overlap: 100,
    strategy: ChunkingStrategy::Sentence,
};
```

## Design Principles

### Zero Infrastructure Dependencies ✅

**Why this matters:**
- **Testability**: Domain logic can be tested without databases, message brokers, or HTTP servers
- **Portability**: Swap database backends (PostgreSQL → MongoDB) without touching domain code
- **Evolution**: Business logic remains stable as infrastructure changes
- **Clarity**: Clear separation between "what we do" (domain) and "how we do it" (infrastructure)

**Verification:**
```bash
# No infrastructure dependencies in domain layer
cargo tree -p riptide-types | grep -E "(sqlx|redis|actix|hyper|axum)"
# Result: ZERO matches ✅
```

### Hexagonal Architecture Role

```text
┌─────────────────────────────────────────┐
│  Domain Layer (riptide-types)           │
│  - Pure business logic                  │
│  - Port trait definitions               │
│  - NO infrastructure dependencies       │
└─────────────────────────────────────────┘
         ↑ uses              ↑ implements
         │                   │
┌────────┴──────────┐   ┌────┴──────────────┐
│ Application Layer │   │ Infrastructure    │
│ (riptide-facade)  │   │ (riptide-*)       │
│ - Workflows       │   │ - Adapters        │
└───────────────────┘   └───────────────────┘
         ↑                       ↑
         └───────┬───────────────┘
                 │ wires together
         ┌───────┴──────────┐
         │ Composition Root │
         │ (riptide-api)    │
         └──────────────────┘
```

## Port Traits Catalog

### Data Persistence Ports

#### `Repository<T>` - Generic data persistence
```rust
#[async_trait]
pub trait Repository<T>: Send + Sync {
    async fn find_by_id(&self, id: &str) -> Result<Option<T>>;
    async fn save(&self, entity: &T) -> Result<()>;
    async fn delete(&self, id: &str) -> Result<()>;
    async fn find_all(&self, filter: &RepositoryFilter) -> Result<Vec<T>>;
}
```
**Adapters:** `riptide-persistence::PostgresRepository`, in-memory test implementations

#### `TransactionManager` - ACID transaction support
```rust
#[async_trait]
pub trait TransactionManager: Send + Sync {
    async fn begin(&self) -> Result<Transaction>;
    async fn commit(&self, tx: Transaction) -> Result<()>;
    async fn rollback(&self, tx: Transaction) -> Result<()>;
}
```
**Adapters:** `riptide-persistence::PostgresTransactionManager`

#### `IdempotencyStore` - Duplicate request prevention
```rust
#[async_trait]
pub trait IdempotencyStore: Send + Sync {
    async fn check(&self, token: &IdempotencyToken) -> Result<bool>;
    async fn mark_processed(&self, token: &IdempotencyToken) -> Result<()>;
}
```
**Adapters:** `riptide-persistence::PostgresIdempotencyStore`, `riptide-cache::RedisIdempotencyStore`

### Event & Messaging Ports

#### `EventBus` - Domain event publishing
```rust
#[async_trait]
pub trait EventBus: Send + Sync {
    async fn publish(&self, event: DomainEvent) -> Result<()>;
    async fn subscribe(
        &self,
        event_type: &str,
        handler: Arc<dyn EventHandler>,
    ) -> Result<SubscriptionId>;
}
```
**Adapters:** `riptide-events::InMemoryEventBus`, `riptide-persistence::OutboxEventBus`

#### `EventHandler` - Event consumer interface
```rust
#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle(&self, event: &DomainEvent) -> Result<()>;
}
```
**Adapters:** Application-specific handlers in `riptide-facade`

### Infrastructure Abstraction Ports

#### `CacheStorage` - Generic caching
```rust
#[async_trait]
pub trait CacheStorage: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn set(&self, key: &str, value: &[u8], ttl: Duration) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn exists(&self, key: &str) -> Result<bool>;
}
```
**Adapters:** `riptide-cache::RedisCache`, `InMemoryCache` (included in this crate)

#### `Clock` - Time abstraction for testing
```rust
pub trait Clock: Send + Sync {
    fn now(&self) -> SystemTime;
    fn now_utc(&self) -> DateTime<Utc>;
}
```
**Adapters:** `SystemClock` (production), `FakeClock` (testing) - both included in this crate

#### `Entropy` - Random number generation
```rust
pub trait Entropy: Send + Sync {
    fn random_bytes(&self, count: usize) -> Vec<u8>;
    fn random_u64(&self) -> u64;
}
```
**Adapters:** `SystemEntropy` (production), `DeterministicEntropy` (testing) - both included

### Feature Capability Ports

#### `BrowserDriver` - Headless browser automation
```rust
#[async_trait]
pub trait BrowserDriver: Send + Sync {
    async fn navigate(&self, url: &str) -> Result<BrowserSession>;
    async fn execute_script(&self, session: &BrowserSession, script: &str) -> Result<ScriptResult>;
    async fn screenshot(&self, session: &BrowserSession) -> Result<Vec<u8>>;
    async fn close(&self, session: BrowserSession) -> Result<()>;
}
```
**Adapters:** `riptide-browser::ChromeDriver`, `riptide-headless::HeadlessChrome`

#### `PdfProcessor` - PDF text/image extraction
```rust
#[async_trait]
pub trait PdfProcessor: Send + Sync {
    async fn extract_text(&self, pdf_bytes: &[u8]) -> Result<String>;
    async fn extract_images(&self, pdf_bytes: &[u8]) -> Result<Vec<Vec<u8>>>;
    async fn get_metadata(&self, pdf_bytes: &[u8]) -> Result<PdfMetadata>;
}
```
**Adapters:** `riptide-browser::PdfiumProcessor`

#### `SearchEngine` - Full-text search indexing
```rust
#[async_trait]
pub trait SearchEngine: Send + Sync {
    async fn index(&self, document: &SearchDocument) -> Result<()>;
    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>>;
    async fn delete(&self, id: &str) -> Result<()>;
}
```
**Adapters:** `riptide-search::MeiliSearchEngine`, `riptide-search::TantivyEngine`

### Session & State Ports

#### `SessionStorage` - User session management
```rust
#[async_trait]
pub trait SessionStorage: Send + Sync {
    async fn get(&self, session_id: &str) -> Result<Option<Session>>;
    async fn save(&self, session: &Session) -> Result<()>;
    async fn delete(&self, session_id: &str) -> Result<()>;
}
```
**Adapters:** `riptide-cache::RedisSessionStorage`, in-memory implementations

#### `Pool<T>` - Resource pooling
```rust
#[async_trait]
pub trait Pool<T>: Send + Sync {
    async fn acquire(&self) -> Result<PooledResource<T>>;
    async fn health(&self) -> PoolHealth;
    async fn stats(&self) -> PoolStats;
}
```
**Adapters:** `riptide-pool::GenericPool`

### Reliability & Monitoring Ports

#### `CircuitBreaker` - Circuit breaker pattern
```rust
#[async_trait]
pub trait CircuitBreaker: Send + Sync {
    async fn call<F, T>(&self, operation: F) -> Result<T>
    where
        F: Future<Output = Result<T>> + Send;

    fn state(&self) -> CircuitState;
    fn stats(&self) -> CircuitBreakerStats;
}
```
**Adapters:** `riptide-reliability::AdaptiveCircuitBreaker`

#### `RateLimiter` - Rate limiting
```rust
#[async_trait]
pub trait RateLimiter: Send + Sync {
    async fn acquire(&self, key: &str, cost: u32) -> Result<()>;
    async fn check(&self, key: &str, cost: u32) -> Result<bool>;
}
```
**Adapters:** `riptide-reliability::TokenBucketRateLimiter`

#### `HealthCheck` - Service health monitoring
```rust
#[async_trait]
pub trait HealthCheck: Send + Sync {
    async fn check(&self) -> Result<HealthStatus>;
    fn name(&self) -> &str;
}
```
**Adapters:** Component-specific health checks in infrastructure crates

#### `MetricsCollector` - Business metrics tracking
```rust
pub trait MetricsCollector: Send + Sync {
    fn increment_counter(&self, name: &str, value: u64);
    fn record_histogram(&self, name: &str, value: f64);
    fn set_gauge(&self, name: &str, value: f64);
}
```
**Adapters:** `riptide-monitoring::PrometheusCollector`

### Streaming & Real-time Ports

#### `StreamProcessor` - Real-time data streaming
```rust
#[async_trait]
pub trait StreamProcessor: Send + Sync {
    async fn process(&self, event: StreamEvent) -> Result<ProcessedResult>;
    async fn on_complete(&self, summary: StreamCompletionSummary) -> Result<()>;
}
```
**Adapters:** `riptide-api::StreamingHandler`

#### `StreamingTransport` - SSE/WebSocket transport
```rust
#[async_trait]
pub trait StreamingTransport: Send + Sync {
    async fn send(&self, event: StreamEvent) -> Result<()>;
    async fn close(&self) -> Result<()>;
}
```
**Adapters:** `riptide-api::SseTransport`, `riptide-api::WebSocketTransport`

#### `HttpClient` - HTTP request abstraction
```rust
#[async_trait]
pub trait HttpClient: Send + Sync {
    async fn request(&self, req: HttpRequest) -> Result<HttpResponse>;
}
```
**Adapters:** `riptide-fetch::ReqwestClient`

## Usage Examples

### Working with Port Traits

```rust
use riptide_types::ports::{Repository, EventBus, TransactionManager};
use riptide_types::{ExtractedContent, DomainEvent};

// Domain layer function - works with ANY implementation
async fn save_extraction(
    content: &ExtractedContent,
    repo: &dyn Repository<ExtractedContent>,
    events: &dyn EventBus,
    tx_manager: &dyn TransactionManager,
) -> Result<()> {
    // Start transaction
    let mut tx = tx_manager.begin().await?;

    // Save content
    repo.save(content).await?;

    // Publish event
    let event = DomainEvent::new(
        "content.extracted",
        &content.url,
        serde_json::to_value(content)?,
    );
    events.publish(event).await?;

    // Commit transaction
    tx_manager.commit(tx).await?;

    Ok(())
}
```

### Using Domain Types

```rust
use riptide_types::{
    ExtractedContent, ExtractionQuality, ExtractionStats,
    CrawlRequest, Priority,
};

// Create extraction result
let content = ExtractedContent {
    title: "How to Build Web Scrapers".to_string(),
    content: "Full article text...".to_string(),
    summary: Some("Learn web scraping basics...".to_string()),
    url: "https://example.com/article".to_string(),
    metadata: HashMap::new(),
};

// Track quality metrics
let quality = ExtractionQuality {
    content_quality: 0.92,
    title_quality: 0.88,
    structure_score: 0.90,
    relevance_score: Some(0.85),
};

// Track performance
let stats = ExtractionStats {
    extraction_time_ms: 245,
    memory_used_bytes: 1_048_576,
    nodes_processed: 1520,
};

// Create crawl requests
let high_priority = CrawlRequest::new(url)
    .with_priority(Priority::High)
    .with_depth(1)
    .with_score(0.95);
```

### Configuration Types

```rust
use riptide_types::{
    ExtractionMode, RenderMode, OutputFormat,
    ChunkingConfig, CircuitBreakerConfig, RetryConfig,
};

// Extraction configuration
let mode = ExtractionMode::Article;
let render = RenderMode::Adaptive;
let format = OutputFormat::Chunked;

// Chunking for LLM processing
let chunking = ChunkingConfig {
    chunk_size: 2000,
    overlap: 200,
    strategy: ChunkingStrategy::Semantic,
};

// Reliability configuration
let circuit_breaker = CircuitBreakerConfig {
    failure_threshold: 5,
    timeout: Duration::from_secs(30),
    reset_timeout: Duration::from_secs(60),
};

let retry = RetryConfig {
    max_retries: 3,
    initial_delay: Duration::from_millis(100),
    max_delay: Duration::from_secs(10),
    backoff_multiplier: 2.0,
};
```

## Testing

### Pure Domain Logic - No Mocks Needed

Because domain types are pure data structures, most domain logic can be tested without mocking:

```rust
use riptide_types::{ExtractedContent, ExtractionQuality};

#[test]
fn test_extraction_quality_scoring() {
    let quality = ExtractionQuality {
        content_quality: 0.9,
        title_quality: 0.8,
        structure_score: 0.85,
        relevance_score: Some(0.9),
    };

    // Pure function - no infrastructure needed
    let overall = quality.overall_score();
    assert!(overall > 0.85);
}

#[test]
fn test_crawl_request_priority() {
    let high = CrawlRequest::new(url).with_priority(Priority::High);
    let low = CrawlRequest::new(url).with_priority(Priority::Low);

    // Pure comparison - no database needed
    assert!(high.priority > low.priority);
}
```

### Testing with Port Trait Mocks

For testing code that uses port traits, create simple in-memory mocks:

```rust
use riptide_types::ports::Repository;
use std::collections::HashMap;
use std::sync::RwLock;

// Simple in-memory mock
struct InMemoryRepo<T> {
    store: RwLock<HashMap<String, T>>,
}

#[async_trait]
impl<T: Clone + Send + Sync> Repository<T> for InMemoryRepo<T> {
    async fn find_by_id(&self, id: &str) -> Result<Option<T>> {
        Ok(self.store.read().unwrap().get(id).cloned())
    }

    async fn save(&self, entity: &T) -> Result<()> {
        self.store.write().unwrap().insert(entity.id(), entity.clone());
        Ok(())
    }

    // ... other methods
}

// Use in tests
#[tokio::test]
async fn test_save_user() {
    let repo = InMemoryRepo::new();

    // Test without real database
    repo.save(&user).await.unwrap();
    let found = repo.find_by_id("user-123").await.unwrap();

    assert_eq!(found, Some(user));
}
```

### Using Included Test Implementations

This crate includes test-friendly implementations for common ports:

```rust
use riptide_types::ports::{FakeClock, DeterministicEntropy, InMemoryCache};

#[tokio::test]
async fn test_time_dependent_logic() {
    let clock = FakeClock::new();

    // Control time in tests
    clock.set(DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")?);

    let timestamp = clock.now_utc();
    assert_eq!(timestamp.year(), 2024);
}

#[tokio::test]
async fn test_random_dependent_logic() {
    let entropy = DeterministicEntropy::new(12345);

    // Deterministic "random" values for testing
    let value = entropy.random_u64();
    assert_eq!(value, 12345); // Predictable!
}

#[tokio::test]
async fn test_caching_logic() {
    let cache = InMemoryCache::new();

    // Test caching without Redis
    cache.set("key", b"value", Duration::from_secs(60)).await?;
    let result = cache.get("key").await?;

    assert_eq!(result, Some(b"value".to_vec()));
}
```

## Dependencies

### Core Dependencies (Minimal)
- **serde** / **serde_json** - Serialization for domain types
- **async-trait** - Async trait definitions for ports
- **thiserror** / **anyhow** - Error handling
- **tokio** - Async runtime primitives (sync, time only - no I/O)
- **tracing** - Logging infrastructure
- **url** - URL type for crawling
- **chrono** - DateTime handling
- **uuid** - Unique identifiers

### No Infrastructure Dependencies ✅
```toml
# ❌ NOT present in riptide-types
sqlx, diesel, postgres         # No database
redis, memcached              # No cache backends
actix-web, hyper, axum        # No HTTP servers
reqwest, ureq                 # No HTTP clients
headless_chrome, fantoccini   # No browser automation
```

## Common Patterns

### Idiomatic Port Usage

✅ **DO:** Accept port traits via dependency injection
```rust
async fn extract_and_save(
    url: &str,
    repo: &dyn Repository<ExtractedContent>,  // Port trait
    events: &dyn EventBus,                    // Port trait
) -> Result<()> {
    // Implementation
}
```

❌ **DON'T:** Use concrete infrastructure types in domain
```rust
async fn extract_and_save(
    url: &str,
    repo: &PostgresRepository,  // ❌ Concrete implementation
    events: &KafkaEventBus,     // ❌ Concrete implementation
) -> Result<()> {
    // Tightly coupled to infrastructure
}
```

### Anti-Patterns to Avoid

❌ **DON'T:** Let domain types depend on infrastructure
```rust
use sqlx::PgPool;  // ❌ Database dependency in domain

pub struct ExtractedContent {
    pub title: String,
    pub pool: PgPool,  // ❌ Infrastructure leaking into domain
}
```

✅ **DO:** Keep domain types pure
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedContent {
    pub title: String,
    pub content: String,
    // Pure data - no infrastructure
}
```

❌ **DON'T:** Implement infrastructure logic in domain layer
```rust
// ❌ In riptide-types crate
impl Repository<User> for PostgresRepository {
    async fn save(&self, user: &User) -> Result<()> {
        // ❌ SQL queries in domain layer
        sqlx::query!("INSERT INTO users ...").execute(&self.pool).await?;
        Ok(())
    }
}
```

✅ **DO:** Define port traits in domain, implement in infrastructure
```rust
// ✅ In riptide-types crate
#[async_trait]
pub trait Repository<T>: Send + Sync {
    async fn save(&self, entity: &T) -> Result<()>;
    // Abstract interface - no implementation
}

// ✅ In riptide-persistence crate
impl Repository<User> for PostgresRepository {
    async fn save(&self, user: &User) -> Result<()> {
        // SQL implementation here
    }
}
```

### Best Practices

1. **Port First**: Define port traits before building infrastructure
2. **Interface Segregation**: Small, focused port traits over large interfaces
3. **Async by Default**: All I/O-bound ports use async/await
4. **Error Handling**: Use `RiptideError` for domain errors
5. **Serializable**: Domain types implement `Serialize`/`Deserialize`
6. **Clone When Cheap**: Domain types are `Clone` for convenience
7. **Send + Sync**: All types are thread-safe by default

## Integration Points

### How Other Layers Use This Crate

**Application Layer (riptide-facade):**
```rust
use riptide_types::{ExtractedContent, ExtractionMode};
use riptide_types::ports::{Repository, EventBus};

// Facades orchestrate domain logic using port traits
pub struct ExtractionFacade {
    repo: Arc<dyn Repository<ExtractedContent>>,
    events: Arc<dyn EventBus>,
}
```

**Infrastructure Layer (riptide-persistence, riptide-cache, etc.):**
```rust
use riptide_types::ports::Repository;
use sqlx::PgPool;

// Adapters implement port traits with real infrastructure
pub struct PostgresRepository {
    pool: PgPool,
}

#[async_trait]
impl Repository<ExtractedContent> for PostgresRepository {
    async fn save(&self, content: &ExtractedContent) -> Result<()> {
        // Postgres implementation
    }
}
```

**API Layer (riptide-api):**
```rust
use riptide_types::ports::*;
use riptide_persistence::PostgresRepository;
use riptide_cache::RedisCache;

// Composition root wires everything together
pub struct ApplicationContext {
    repo: Arc<PostgresRepository>,
    cache: Arc<RedisCache>,
    // ... other adapters
}

impl ApplicationContext {
    pub fn repo(&self) -> Arc<dyn Repository<ExtractedContent>> {
        self.repo.clone()
    }

    pub fn cache(&self) -> Arc<dyn CacheStorage> {
        self.cache.clone()
    }
}
```

### Related Crates

- **Domain Layer:**
  - `riptide-spider` - Crawling algorithms (uses types from this crate)
  - `riptide-extraction` - Content extraction (uses types from this crate)
  - `riptide-search` - Search domain logic (uses types from this crate)

- **Application Layer:**
  - `riptide-facade` - Workflows and orchestration (uses port traits)

- **Infrastructure Layer (Port Implementations):**
  - `riptide-persistence` - PostgreSQL repositories
  - `riptide-cache` - Redis caching
  - `riptide-fetch` - HTTP client
  - `riptide-browser` - Browser automation
  - `riptide-events` - Event bus implementations
  - `riptide-reliability` - Circuit breakers, retry logic
  - `riptide-monitoring` - Metrics and telemetry

- **Composition Root:**
  - `riptide-api` - HTTP API and dependency injection
  - `riptide-cli` - Command-line interface

## Module Structure

```
src/
├── lib.rs                  # Public API and re-exports
├── component.rs            # Component metadata
├── conditional.rs          # HTTP conditional requests (ETag, If-Modified-Since)
├── config.rs              # Configuration types (modes, formats, chunking)
├── error.rs               # Error types and Result aliases
├── extracted.rs           # Extraction result types
├── extractors.rs          # Extractor configuration
├── http_types.rs          # HTTP-specific types
├── pipeline.rs            # Pipeline execution types
├── reliability.rs         # Reliability configuration
├── secrets.rs             # Secret handling
├── traits.rs              # Core trait definitions (Browser, Extractor, Scraper)
├── types.rs               # Core domain types
└── ports/                 # Port trait definitions
    ├── mod.rs            # Port catalog and documentation
    ├── cache.rs          # CacheStorage port
    ├── circuit_breaker.rs # CircuitBreaker port
    ├── events.rs         # EventBus port
    ├── features.rs       # BrowserDriver, PdfProcessor, SearchEngine ports
    ├── health.rs         # HealthCheck port
    ├── http.rs           # HttpClient port
    ├── idempotency.rs    # IdempotencyStore port
    ├── infrastructure.rs # Clock, Entropy ports
    ├── memory_cache.rs   # InMemoryCache implementation
    ├── metrics.rs        # MetricsCollector port
    ├── pool.rs           # Pool port
    ├── rate_limit.rs     # RateLimiter port
    ├── repository.rs     # Repository port
    ├── session.rs        # SessionStorage port
    └── streaming.rs      # StreamProcessor, StreamingTransport ports
```

## Version

Current version: **0.9.0**

## License

Apache-2.0
