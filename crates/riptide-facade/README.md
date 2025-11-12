# Riptide Facade

🎯 **Application Layer - Use Case Orchestration**

The application layer that coordinates domain objects and infrastructure adapters, providing simplified task-oriented interfaces for the Riptide web scraping framework.

## Quick Overview

**What is riptide-facade?**

The facade layer sits between your API handlers and domain logic, orchestrating complex workflows while keeping your domain pure. Think of it as the "conductor" that coordinates all the specialized components of Riptide without containing business logic itself.

**Key Responsibilities:**
- 🔄 **Use Case Orchestration**: Coordinates multi-step workflows
- 🔐 **Authorization**: Enforces access policies and tenant scoping
- ♻️ **Idempotency**: Prevents duplicate operations
- 📤 **Event Emission**: Publishes domain events reliably
- ⚡ **Transaction Management**: Ensures ACID guarantees
- 🎯 **Simplified API**: Task-oriented interfaces hiding internal complexity

**Why it exists:**

Instead of API handlers directly knowing about 24+ specialized crates, they interact with a few intuitive facades. This reduces coupling, improves testability, and makes the codebase easier to navigate.

## Architecture Role

### Hexagonal Architecture Layers

```text
┌─────────────────────────────────────────────────────────┐
│  API Layer (riptide-api)                                │
│  - HTTP handlers                                        │
│  - WebSocket endpoints                                  │
│  - REST/GraphQL interfaces                              │
└────────────────┬────────────────────────────────────────┘
                 │ calls facades
                 ▼
┌─────────────────────────────────────────────────────────┐
│  APPLICATION LAYER (riptide-facade) ◄── YOU ARE HERE    │
│  - Use case orchestration                               │
│  - Workflow coordination                                │
│  - Authorization policies                               │
│  - Transaction boundaries                               │
└────────────────┬────────────────────────────────────────┘
                 │ uses port traits
                 ▼
┌─────────────────────────────────────────────────────────┐
│  Domain Layer (riptide-types)                           │
│  - Pure business logic                                  │
│  - Domain entities                                      │
│  - Port traits (interfaces)                             │
└────────────────△────────────────────────────────────────┘
                 │ implemented by
                 │
┌─────────────────────────────────────────────────────────┐
│  Infrastructure Layer                                   │
│  - riptide-fetch (HTTP client)                          │
│  - riptide-cache (caching)                              │
│  - riptide-browser (headless browsing)                  │
│  - riptide-reliability (retries, circuit breakers)      │
└─────────────────────────────────────────────────────────┘
```

### Dependency Rules

**✅ ALLOWED Dependencies:**
- `riptide-types` - Domain types and port traits
- `riptide-config` - Configuration
- `riptide-events` - Event definitions
- `riptide-monitoring` - Observability

**❌ FORBIDDEN Dependencies:**
- Infrastructure implementations directly
- HTTP types (axum, actix-web, hyper)
- Database types (sqlx, postgres)
- SDK/client types (redis, reqwest)
- Serialization details (serde_json::Value - use typed DTOs)

### Breaking Circular Dependencies

**Critical Architectural Decision - Phase 2C.2:**

The facade layer previously had a circular dependency with `riptide-api`. This was resolved by:

1. **Extracting trait interfaces** to `riptide-types`:
   - `PipelineExecutor` trait
   - `StrategiesPipelineExecutor` trait

2. **Dependency injection** via trait objects:
   ```rust
   // ❌ OLD: Direct dependency on concrete types
   // use riptide_api::{PipelineOrchestrator, StrategiesPipelineOrchestrator};

   // ✅ NEW: Depends on traits from riptide-types
   use riptide_types::pipeline::{PipelineExecutor, StrategiesPipelineExecutor};

   pub struct CrawlFacade {
       pipeline: Arc<dyn PipelineExecutor>,
       strategies: Arc<dyn StrategiesPipelineExecutor>,
   }
   ```

3. **ApplicationContext** (in riptide-api) creates concrete implementations and injects them

This pattern is documented extensively in the code - see comments in `Cargo.toml` and `crawl_facade.rs`.

## Facade Pattern

### What Facades Provide

Facades offer **simplified, task-oriented interfaces** to complex subsystems:

```rust
// Without facade: Complex, requires knowledge of many crates
let fetch_engine = FetchEngine::new()?;
let cache = CacheManager::new(redis_client)?;
let browser = BrowserPool::new(headless_config).await?;
let extractor = ExtractionEngine::new(strategies)?;

// Check cache
if let Some(cached) = cache.get(&url).await? {
    return Ok(cached);
}

// Fetch with retries
let html = fetch_engine.fetch_with_retry(url, 3).await?;

// Extract with browser fallback
let data = if requires_js {
    let session = browser.navigate(&url).await?;
    extractor.extract_from_browser(&session).await?
} else {
    extractor.extract_from_html(&html).await?
};

// Cache result
cache.set(&url, &data, Duration::from_secs(3600)).await?;

// With facade: Simple, intuitive
let scraper = ScraperFacade::new(config).await?;
let data = scraper.fetch_html(url).await?;
```

### Key Facades

| Facade | Purpose | Use When |
|--------|---------|----------|
| **ScraperFacade** | Simple web page fetching | You need HTML/bytes from a single URL |
| **CrawlFacade** | Multi-page crawling workflows | Spidering websites, following links |
| **BrowserFacade** | Headless browser automation | JavaScript rendering, screenshots |
| **ExtractionFacade** | Content extraction strategies | Parsing structured data from HTML |
| **PipelineFacade** | Multi-stage workflow orchestration | Complex data processing pipelines |
| **SpiderFacade** | Intelligent crawling with budgets | Rate-limited, depth-limited crawling |
| **SearchFacade** | Content search and indexing | Full-text search across crawled content |

### When to Use Facades vs Direct Domain Access

**Use Facades when:**
- Building API endpoints (most common case)
- Implementing user-facing features
- Coordinating multiple infrastructure components
- Need transaction boundaries or idempotency

**Use Domain directly when:**
- Writing unit tests for pure business logic
- Building other facades
- Implementing infrastructure adapters

## Use Case Implementations

### Practical Example: Crawling Workflow

```rust
use riptide_facade::facades::{CrawlFacade, CrawlMode};
use riptide_types::config::CrawlOptions;

async fn crawl_website(
    facade: &CrawlFacade,
    url: &str,
) -> Result<CrawlResult, RiptideError> {
    // Configure crawl behavior
    let options = CrawlOptions {
        spider_max_depth: Some(3),
        spider_max_pages: Some(100),
        respect_robots_txt: true,
        crawl_delay_ms: Some(200),
        ..Default::default()
    };

    // Execute with automatic retry logic, rate limiting, and error handling
    let result = facade
        .crawl_single(url, options, CrawlMode::Enhanced)
        .await?;

    Ok(result)
}
```

### Error Handling and Retry Logic

```rust
use riptide_facade::facades::ScraperFacade;
use riptide_facade::RiptideError;

async fn fetch_with_retry(
    scraper: &ScraperFacade,
    url: &str,
) -> Result<String, RiptideError> {
    match scraper.fetch_html(url).await {
        Ok(html) => Ok(html),
        Err(RiptideError::Timeout { .. }) => {
            // Timeouts are retryable
            tokio::time::sleep(Duration::from_secs(5)).await;
            scraper.fetch_html(url).await
        }
        Err(RiptideError::HttpError { status, .. }) if status == 429 => {
            // Rate limited - wait and retry
            tokio::time::sleep(Duration::from_secs(60)).await;
            scraper.fetch_html(url).await
        }
        Err(e) => Err(e), // Not retryable
    }
}
```

### Event Publishing Pattern

```rust
use riptide_facade::workflows::TransactionalWorkflow;
use riptide_types::ports::DomainEvent;

async fn extract_with_events(
    workflow: &TransactionalWorkflow<_>,
    url: &str,
) -> Result<ExtractedData, RiptideError> {
    workflow.execute(
        &format!("extract:{}", url), // idempotency key
        |tx| async move {
            // 1. Perform extraction
            let data = extraction_service.extract(url).await?;

            // 2. Prepare domain event
            let event = DomainEvent::new(
                "content.extracted",
                url.to_string(),
                serde_json::to_value(&data)?,
            );

            // 3. Return result + events
            // Events are written to outbox transactionally
            Ok((data, vec![event]))
        }
    ).await
}
```

### Transaction Management

```rust
use riptide_facade::workflows::TransactionalWorkflow;

async fn create_crawl_job(
    workflow: &TransactionalWorkflow<_>,
    job: CrawlJob,
) -> Result<String, RiptideError> {
    workflow.execute(
        &format!("create-job:{}", job.id),
        |tx| async move {
            // 1. Validate job doesn't exist
            if repository.exists(&job.id, tx).await? {
                return Err(RiptideError::conflict("Job already exists"));
            }

            // 2. Save job to database
            repository.save(&job, tx).await?;

            // 3. Emit job.created event
            let event = DomainEvent::new(
                "crawl_job.created",
                job.id.clone(),
                serde_json::to_value(&job)?,
            );

            // Transaction commits atomically:
            // - Database write
            // - Event written to outbox
            // - Idempotency key set
            Ok((job.id.clone(), vec![event]))
        }
    ).await
}
```

## Working with ApplicationContext

### How Facades Receive Dependencies

Facades use **dependency injection** via trait objects, not concrete types:

```rust
use riptide_types::pipeline::{PipelineExecutor, StrategiesPipelineExecutor};
use std::sync::Arc;

pub struct CrawlFacade {
    // Depends on traits, not concrete implementations
    pipeline_orchestrator: Arc<dyn PipelineExecutor>,
    strategies_orchestrator: Arc<dyn StrategiesPipelineExecutor>,
}

impl CrawlFacade {
    pub fn new(
        pipeline: Arc<dyn PipelineExecutor>,
        strategies: Arc<dyn StrategiesPipelineExecutor>,
    ) -> Self {
        Self {
            pipeline_orchestrator: pipeline,
            strategies_orchestrator: strategies,
        }
    }
}
```

### ApplicationContext Factory Pattern

The `ApplicationContext` (in `riptide-api`) acts as the **composition root**:

```rust
// In riptide-api/src/context.rs (conceptual example)
pub struct ApplicationContext {
    // Concrete infrastructure implementations
    fetch_engine: Arc<FetchEngine>,
    cache: Arc<CacheManager>,
    browser_pool: Arc<BrowserPool>,
    // ... other infrastructure
}

impl ApplicationContext {
    pub fn crawl_facade(&self) -> CrawlFacade {
        // Create concrete orchestrators
        let pipeline = Arc::new(PipelineOrchestrator::new(
            self.fetch_engine.clone(),
            self.cache.clone(),
        )) as Arc<dyn PipelineExecutor>;

        let strategies = Arc::new(StrategiesPipelineOrchestrator::new(
            self.browser_pool.clone(),
            self.cache.clone(),
        )) as Arc<dyn StrategiesPipelineExecutor>;

        // Inject into facade
        CrawlFacade::new(pipeline, strategies)
    }
}
```

### Testing with Mock Implementations

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use riptide_types::pipeline::{PipelineExecutor, StrategiesPipelineExecutor};

    struct MockPipelineExecutor;

    #[async_trait]
    impl PipelineExecutor for MockPipelineExecutor {
        async fn execute_single(&self, url: &str) -> Result<PipelineResult> {
            Ok(PipelineResult {
                url: url.to_string(),
                status_code: 200,
                content: "mock content".to_string(),
            })
        }

        // ... other trait methods
    }

    #[tokio::test]
    async fn test_crawl_facade_with_mocks() {
        let pipeline = Arc::new(MockPipelineExecutor) as Arc<dyn PipelineExecutor>;
        let strategies = Arc::new(MockStrategiesExecutor) as Arc<dyn StrategiesPipelineExecutor>;

        let facade = CrawlFacade::new(pipeline, strategies);

        let result = facade
            .crawl_single("https://example.com", options, CrawlMode::Standard)
            .await;

        assert!(result.is_ok());
    }
}
```

## Key Components

### Core Facades

#### **ScraperFacade**
Simple web page fetching facade.

```rust
pub struct ScraperFacade {
    config: Arc<RiptideConfig>,
    client: Arc<FetchEngine>,
}

// Key methods:
// - fetch_html(url) -> Result<String>
// - fetch_bytes(url) -> Result<Vec<u8>>
```

**Use for:** Single-page fetches, API calls, downloading resources.

#### **CrawlFacade**
Wraps production pipeline orchestrators (1,596 lines of battle-tested code).

```rust
pub struct CrawlFacade {
    pipeline_orchestrator: Arc<dyn PipelineExecutor>,
    strategies_orchestrator: Arc<dyn StrategiesPipelineExecutor>,
}

// Key methods:
// - crawl_single(url, options, mode) -> Result<CrawlResult>
// - crawl_batch(urls) -> (Vec<Option<PipelineResult>>, PipelineStats)
```

**Use for:** Multi-page crawling with depth limits, rate limiting, and robots.txt respect.

**Important:** This facade **wraps** existing production code, not rebuilds it:
- `PipelineOrchestrator`: 1,071 lines (standard mode)
- `StrategiesPipelineOrchestrator`: 525 lines (enhanced mode with browser fallbacks)

#### **BrowserFacade**
Headless browser automation facade.

```rust
pub struct BrowserFacade {
    config: Arc<RiptideConfig>,
    pool: Arc<BrowserPool>,
}

// Key methods:
// - navigate(url) -> Result<BrowserSession>
// - screenshot(url, options) -> Result<Vec<u8>>
// - execute_script(session, script) -> Result<Value>
```

**Use for:** JavaScript-heavy sites, screenshots, PDF generation, form interaction.

#### **ExtractionFacade**
Content extraction with multiple strategies.

```rust
pub struct ExtractionFacade {
    config: Arc<RiptideConfig>,
    strategies: Vec<Box<dyn ExtractionStrategy>>,
}

// Key methods:
// - extract(html, schema) -> Result<ExtractedData>
// - extract_with_strategy(html, strategy) -> Result<ExtractedData>
```

**Use for:** Structured data extraction, field mapping, schema validation.

#### **PipelineFacade**
Multi-stage workflow orchestration.

```rust
pub struct PipelineFacade {
    config: Arc<RiptideConfig>,
    cache: Arc<RwLock<PipelineCache>>,
}

// Key methods:
// - builder() -> PipelineBuilder
// - execute(pipeline) -> Result<PipelineResult>
```

**Use for:** Complex workflows with multiple stages, error handling, caching, and retries.

### Workflow Coordinators

#### **TransactionalWorkflow**
ACID transaction coordinator with idempotency and event emission.

```rust
pub struct TransactionalWorkflow<TM: TransactionManager> {
    tx_manager: Arc<TM>,
    event_bus: Arc<dyn EventBus>,
    idempotency_store: Arc<dyn IdempotencyStore>,
    default_ttl: Duration,
}

// Key methods:
// - execute<F, T>(idempotency_key, f) -> Result<T>
//   where F: FnOnce(&Transaction) -> Future<Output = Result<(T, Vec<DomainEvent>)>>
```

**Features:**
- ✅ ACID guarantees with automatic rollback on error
- ✅ Idempotency prevention (duplicate operations blocked)
- ✅ Transactional outbox pattern for reliable event publishing
- ✅ Configurable TTL for idempotency keys

#### **BackpressureManager**
Concurrency control and resource limiting.

```rust
pub struct BackpressureManager {
    semaphore: Arc<Semaphore>,
    max_concurrent: usize,
}

// Key methods:
// - acquire() -> Result<BackpressureGuard>
// - try_acquire() -> Option<BackpressureGuard>
// - available_permits() -> usize
```

**Use for:** Limiting concurrent browser sessions, database connections, or API calls.

### Configuration and Builder

#### **RiptideBuilder**
Fluent API for facade configuration.

```rust
let scraper = Riptide::builder()
    .user_agent("MyBot/1.0")
    .timeout_secs(60)
    .max_redirects(10)
    .header("X-API-Key", "secret")
    .build_scraper()
    .await?;
```

## Documented Architectural Rules

### Circular Dependency Avoidance

**Problem Statement:**

Initially, `riptide-facade` had a circular dependency with `riptide-api`:
- `riptide-facade` needed `PipelineOrchestrator` from `riptide-api`
- `riptide-api` needed facades from `riptide-facade`

**Solution (Phase 2C.2):**

1. **Extract trait interfaces** to `riptide-types`:
   ```rust
   // In riptide-types/src/pipeline.rs
   #[async_trait]
   pub trait PipelineExecutor: Send + Sync {
       async fn execute_single(&self, url: &str) -> Result<PipelineResult>;
       async fn execute_batch(&self, urls: &[String]) -> (Vec<Option<PipelineResult>>, PipelineStats);
   }
   ```

2. **Facades depend on traits**, not concrete implementations:
   ```rust
   // In riptide-facade/src/facades/crawl_facade.rs
   use riptide_types::pipeline::PipelineExecutor; // ✅ Trait from types
   // NOT: use riptide_api::PipelineOrchestrator; // ❌ Concrete type

   pub struct CrawlFacade {
       pipeline: Arc<dyn PipelineExecutor>,
   }
   ```

3. **ApplicationContext** provides concrete implementations:
   ```rust
   // In riptide-api/src/context.rs
   impl ApplicationContext {
       pub fn crawl_facade(&self) -> CrawlFacade {
           let pipeline = Arc::new(PipelineOrchestrator::new(...))
               as Arc<dyn PipelineExecutor>;
           CrawlFacade::new(pipeline, ...)
       }
   }
   ```

**Documentation:**

This architectural decision is documented in:
- `Cargo.toml` comments (lines 11-14)
- `crawl_facade.rs` docstrings (lines 13-14, 88-107)
- Test files explaining the pattern

### Why Certain Patterns Are Used

#### Port-Based Design
**Why:** Enables testing with in-memory implementations, no infrastructure needed.

```rust
// Port trait (interface)
#[async_trait]
pub trait CacheStorage: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn set(&self, key: &str, value: Vec<u8>, ttl: Duration) -> Result<()>;
}

// Production implementation
pub struct RedisCache { ... }

// Test implementation
pub struct InMemoryCache { ... }

// Facade uses trait, works with both
pub struct ExtractionFacade {
    cache: Arc<dyn CacheStorage>,
}
```

#### Transactional Outbox Pattern
**Why:** Ensures events are published reliably, even if event bus is temporarily unavailable.

```rust
workflow.execute(key, |tx| async move {
    // 1. Write to database
    repository.save(&entity, tx).await?;

    // 2. Write event to outbox table (same transaction)
    let event = DomainEvent::new(...);

    // If event bus fails later, background worker retries from outbox
    Ok((entity, vec![event]))
}).await
```

#### Builder Pattern
**Why:** Provides discoverable, type-safe configuration with sensible defaults.

```rust
// Easy to discover available options
Riptide::builder()
    .user_agent("MyBot/1.0")  // IDE autocompletes
    .timeout_secs(60)         // Type-checked
    .build_scraper()          // Returns Result<ScraperFacade>
    .await?
```

### Trade-offs and Design Decisions

| Decision | Pro | Con |
|----------|-----|-----|
| **Trait-based ports** | ✅ Testable without infrastructure<br>✅ Swappable implementations | ❌ Slight runtime overhead (vtable dispatch)<br>❌ More verbose than concrete types |
| **Arc-wrapped dependencies** | ✅ Thread-safe sharing<br>✅ Cheap cloning | ❌ Reference counting overhead<br>❌ Can mask ownership issues |
| **Async-first API** | ✅ Non-blocking I/O<br>✅ Scalable | ❌ More complex error handling<br>❌ Requires Tokio runtime |
| **Wrapping production code** | ✅ No duplication<br>✅ Proven reliability | ❌ Must keep traits in sync<br>❌ Less flexibility for facade API |

## Testing Strategy

### Contract Testing with Test Doubles

**Pattern:** Test facades with mock port implementations, no real infrastructure.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    struct MockCache {
        data: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    }

    #[async_trait]
    impl CacheStorage for MockCache {
        async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
            Ok(self.data.lock().unwrap().get(key).cloned())
        }

        async fn set(&self, key: &str, value: Vec<u8>, _ttl: Duration) -> Result<()> {
            self.data.lock().unwrap().insert(key.to_string(), value);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_extraction_uses_cache() {
        let cache = Arc::new(MockCache::new());
        let facade = ExtractionFacade::new(cache.clone()).await?;

        // First call misses cache
        let result1 = facade.extract("https://example.com").await?;

        // Second call hits cache
        let result2 = facade.extract("https://example.com").await?;

        assert_eq!(result1, result2);
        assert_eq!(cache.data.lock().unwrap().len(), 1);
    }
}
```

### Integration Testing with Real Adapters

**Pattern:** Test facades with real infrastructure in `tests/` directory.

```rust
// tests/crawl_facade_integration_tests.rs
use riptide_api::context::ApplicationContext;
use riptide_facade::facades::{CrawlFacade, CrawlMode};

#[tokio::test]
async fn test_crawl_facade_with_real_orchestrators() {
    // Create ApplicationContext with real infrastructure
    let context = ApplicationContext::new_for_testing().await?;

    // Get facade with real orchestrator implementations
    let facade = context.crawl_facade();

    // Test against real (or wiremock) HTTP server
    let result = facade
        .crawl_single("http://localhost:8080/test", options, CrawlMode::Standard)
        .await?;

    assert!(matches!(result, CrawlResult::Standard(_)));
}
```

### Example Test Patterns

#### Testing Idempotency

```rust
#[tokio::test]
async fn test_transactional_workflow_idempotency() {
    let workflow = TransactionalWorkflow::new(
        Arc::new(MockTransactionManager::new()),
        Arc::new(MockEventBus::new()),
        Arc::new(InMemoryIdempotencyStore::new()),
    );

    let key = "test-operation-123";

    // First execution succeeds
    let result1 = workflow.execute(key, |_tx| async move {
        Ok(("success", vec![]))
    }).await?;

    // Second execution with same key is prevented
    let result2 = workflow.execute(key, |_tx| async move {
        panic!("Should not execute again!");
    }).await;

    assert!(result2.is_err()); // Idempotency violation
}
```

#### Testing Transaction Rollback

```rust
#[tokio::test]
async fn test_transaction_rollback_on_error() {
    let tx_manager = Arc::new(MockTransactionManager::new());
    let workflow = TransactionalWorkflow::new(
        tx_manager.clone(),
        Arc::new(MockEventBus::new()),
        Arc::new(InMemoryIdempotencyStore::new()),
    );

    let result = workflow.execute("key", |_tx| async move {
        // Simulate error after partial work
        Err(RiptideError::extraction("Failed"))
    }).await;

    assert!(result.is_err());
    assert_eq!(tx_manager.rollback_count(), 1);
    assert_eq!(tx_manager.commit_count(), 0);
}
```

### Reference to tests/README.md

For comprehensive testing documentation, see [`tests/README.md`](tests/README.md) which covers:
- Test organization and structure
- Mock implementations and test helpers
- Integration test patterns
- Performance benchmarking
- CI/CD integration

## Common Workflows

### Step-by-Step Guide: Initiating a Crawl with Extraction

**Scenario:** Crawl a website and extract product data.

```rust
use riptide_facade::facades::{CrawlFacade, ExtractionFacade, CrawlMode};
use riptide_facade::dto::{StructuredData, FieldSpec, FieldType};
use riptide_types::config::CrawlOptions;

async fn crawl_and_extract_products(
    crawl_facade: &CrawlFacade,
    extract_facade: &ExtractionFacade,
    base_url: &str,
) -> Result<Vec<Product>, RiptideError> {
    // Step 1: Configure crawl options
    let crawl_opts = CrawlOptions {
        spider_max_depth: Some(2),
        spider_max_pages: Some(50),
        respect_robots_txt: true,
        crawl_delay_ms: Some(500),
        allowed_domains: vec![base_url.to_string()],
        ..Default::default()
    };

    // Step 2: Execute crawl
    let crawl_result = crawl_facade
        .crawl_single(base_url, crawl_opts, CrawlMode::Enhanced)
        .await?;

    // Step 3: Extract URLs from crawl result
    let product_urls = match crawl_result {
        CrawlResult::Enhanced(result) => {
            result.urls.iter()
                .filter(|url| url.contains("/products/"))
                .cloned()
                .collect::<Vec<_>>()
        }
        CrawlResult::Standard(result) => {
            result.links.clone()
        }
    };

    // Step 4: Define extraction schema
    let schema = Schema {
        fields: vec![
            FieldSpec {
                name: "title".to_string(),
                field_type: FieldType::Text,
                selector: "h1.product-title".to_string(),
                required: true,
            },
            FieldSpec {
                name: "price".to_string(),
                field_type: FieldType::Number,
                selector: ".price .amount".to_string(),
                required: true,
            },
            FieldSpec {
                name: "description".to_string(),
                field_type: FieldType::Text,
                selector: ".product-description".to_string(),
                required: false,
            },
        ],
    };

    // Step 5: Extract data from each product page
    let mut products = Vec::new();
    for url in product_urls {
        match extract_facade.extract(&url, schema.clone()).await {
            Ok(data) => {
                let product = Product {
                    url: url.clone(),
                    title: data.get_field("title")?,
                    price: data.get_field("price")?,
                    description: data.get_field("description").ok(),
                };
                products.push(product);
            }
            Err(e) => {
                tracing::warn!("Failed to extract {}: {}", url, e);
                // Continue with other products
            }
        }
    }

    Ok(products)
}
```

### Step-by-Step Guide: Handling Errors and Retries

**Scenario:** Fetch data with exponential backoff retry logic.

```rust
use riptide_facade::facades::ScraperFacade;
use riptide_facade::RiptideError;
use std::time::Duration;
use tokio::time::sleep;

async fn fetch_with_exponential_backoff(
    scraper: &ScraperFacade,
    url: &str,
    max_retries: u32,
) -> Result<String, RiptideError> {
    let mut attempt = 0;
    let mut delay = Duration::from_secs(1);

    loop {
        match scraper.fetch_html(url).await {
            Ok(html) => return Ok(html),

            Err(e) if attempt >= max_retries => {
                return Err(e);
            }

            Err(RiptideError::Timeout { .. }) |
            Err(RiptideError::HttpError { status: 429, .. }) |
            Err(RiptideError::HttpError { status: 503, .. }) => {
                // Retryable errors
                tracing::warn!(
                    "Attempt {} failed, retrying in {:?}: {}",
                    attempt + 1,
                    delay,
                    e
                );

                sleep(delay).await;

                // Exponential backoff: 1s, 2s, 4s, 8s, ...
                delay *= 2;
                attempt += 1;
            }

            Err(e) => {
                // Non-retryable error
                tracing::error!("Non-retryable error: {}", e);
                return Err(e);
            }
        }
    }
}
```

### Step-by-Step Guide: Working with Events

**Scenario:** Emit events when crawl milestones are reached.

```rust
use riptide_facade::workflows::TransactionalWorkflow;
use riptide_types::ports::DomainEvent;

async fn crawl_with_progress_events(
    workflow: &TransactionalWorkflow<_>,
    crawl_facade: &CrawlFacade,
    url: &str,
) -> Result<CrawlResult, RiptideError> {
    // Step 1: Emit crawl.started event
    workflow.execute(
        &format!("crawl:start:{}", url),
        |_tx| async move {
            let event = DomainEvent::new(
                "crawl.started",
                url.to_string(),
                serde_json::json!({ "url": url, "timestamp": Utc::now() }),
            );
            Ok(((), vec![event]))
        }
    ).await?;

    // Step 2: Perform crawl
    let result = crawl_facade
        .crawl_single(url, options, CrawlMode::Standard)
        .await;

    // Step 3: Emit appropriate completion event
    match &result {
        Ok(crawl_result) => {
            workflow.execute(
                &format!("crawl:complete:{}", url),
                |_tx| async move {
                    let event = DomainEvent::new(
                        "crawl.completed",
                        url.to_string(),
                        serde_json::json!({
                            "url": url,
                            "pages_crawled": crawl_result.pages_count(),
                            "timestamp": Utc::now(),
                        }),
                    );
                    Ok(((), vec![event]))
                }
            ).await?;
        }
        Err(e) => {
            workflow.execute(
                &format!("crawl:failed:{}", url),
                |_tx| async move {
                    let event = DomainEvent::new(
                        "crawl.failed",
                        url.to_string(),
                        serde_json::json!({
                            "url": url,
                            "error": e.to_string(),
                            "timestamp": Utc::now(),
                        }),
                    );
                    Ok(((), vec![event]))
                }
            ).await?;
        }
    }

    result
}
```

### Step-by-Step Guide: Transaction Management

**Scenario:** Create a crawl job with transactional guarantees.

```rust
use riptide_facade::workflows::TransactionalWorkflow;
use uuid::Uuid;

async fn create_crawl_job_transactionally(
    workflow: &TransactionalWorkflow<_>,
    repository: &CrawlJobRepository,
    url: &str,
    user_id: &str,
) -> Result<String, RiptideError> {
    let job_id = Uuid::new_v4().to_string();
    let idempotency_key = format!("create-job:{}:{}", user_id, url);

    workflow.execute(
        &idempotency_key,
        |tx| async move {
            // Step 1: Check authorization (pure function, no I/O)
            if !user_can_create_job(user_id) {
                return Err(RiptideError::unauthorized("User cannot create jobs"));
            }

            // Step 2: Validate URL
            let parsed_url = Url::parse(url)
                .map_err(|e| RiptideError::validation(format!("Invalid URL: {}", e)))?;

            // Step 3: Create job entity
            let job = CrawlJob {
                id: job_id.clone(),
                user_id: user_id.to_string(),
                url: parsed_url.to_string(),
                status: JobStatus::Pending,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            // Step 4: Save to database (within transaction)
            repository.save(&job, tx).await?;

            // Step 5: Prepare domain event
            let event = DomainEvent::new(
                "crawl_job.created",
                job_id.clone(),
                serde_json::to_value(&job)
                    .map_err(|e| RiptideError::serialization(e.to_string()))?,
            );

            // Transaction commits atomically:
            // ✅ Database write
            // ✅ Event written to outbox
            // ✅ Idempotency key set
            Ok((job_id.clone(), vec![event]))
        }
    ).await
}
```

## Dependencies

### Domain Crates (Pure Contracts)

These crates define **what** the system does, not **how**:

| Crate | Purpose | Examples |
|-------|---------|----------|
| **riptide-types** | Core domain types and port traits | `CrawlOptions`, `PipelineExecutor`, `CacheStorage` |
| **riptide-config** | Configuration types | `RiptideConfig`, `FetchConfig`, `BrowserConfig` |
| **riptide-events** | Domain event definitions | `DomainEvent`, `EventMetadata` |

**Why these dependencies?**
- Facades orchestrate domain operations, so they need domain types
- Port traits enable dependency injection and testing
- Configuration is cross-cutting

### Infrastructure Crates (Implementations)

These crates define **how** operations are performed:

| Crate | Purpose | Used By Facade |
|-------|---------|----------------|
| **riptide-fetch** | HTTP client implementation | `ScraperFacade`, `PipelineFacade` |
| **riptide-browser** | Headless browser pool | `BrowserFacade` |
| **riptide-cache** | Caching layer | `ExtractionFacade`, `PipelineFacade` |
| **riptide-extraction** | Content extraction engine | `ExtractionFacade` |
| **riptide-spider** | Web crawling engine | `SpiderFacade`, `CrawlFacade` |
| **riptide-monitoring** | Metrics and tracing | All facades |
| **riptide-reliability** | Retries, circuit breakers | `ScraperFacade`, `PipelineFacade` |
| **riptide-intelligence** | LLM operations (optional) | `IntelligenceFacade` |
| **riptide-workers** | Background job processing (optional) | `WorkersFacade` |

**Note:** Facades depend on these crates **for implementations**, but accept dependencies as **trait objects** to maintain testability.

## Extension Points

### How to Add New Use Cases

**Pattern:** Create a new facade that orchestrates existing domain logic.

**Example:** Adding a `ReportingFacade` for analytics.

```rust
// 1. Define the facade in src/facades/reporting.rs
use riptide_types::ports::{Repository, Analytics};
use std::sync::Arc;

pub struct ReportingFacade {
    repository: Arc<dyn Repository>,
    analytics: Arc<dyn Analytics>,
}

impl ReportingFacade {
    pub fn new(
        repository: Arc<dyn Repository>,
        analytics: Arc<dyn Analytics>,
    ) -> Self {
        Self { repository, analytics }
    }

    pub async fn generate_crawl_report(
        &self,
        job_id: &str,
    ) -> Result<CrawlReport, RiptideError> {
        // Orchestrate: fetch data, compute metrics, format report
        let job = self.repository.get_job(job_id).await?;
        let metrics = self.analytics.compute_metrics(&job).await?;

        Ok(CrawlReport {
            job_id: job_id.to_string(),
            metrics,
            generated_at: Utc::now(),
        })
    }
}
```

```rust
// 2. Export from src/facades/mod.rs
pub mod reporting;
pub use reporting::ReportingFacade;
```

```rust
// 3. Add to src/lib.rs public API
pub use facades::ReportingFacade;
```

```rust
// 4. Add factory method in ApplicationContext (riptide-api)
impl ApplicationContext {
    pub fn reporting_facade(&self) -> ReportingFacade {
        ReportingFacade::new(
            self.repository.clone(),
            self.analytics.clone(),
        )
    }
}
```

### When to Add to Facade vs Domain

**Add to Facade when:**
- ✅ Orchestrating multiple domain operations
- ✅ Coordinating infrastructure components
- ✅ Managing transactions or idempotency
- ✅ Emitting events
- ✅ Enforcing authorization policies

**Add to Domain when:**
- ✅ Pure business logic (no I/O)
- ✅ Validation rules
- ✅ Calculations and transformations
- ✅ Entity behavior
- ✅ Value objects

**Example:** Where does URL normalization belong?

```rust
// ❌ WRONG: Business logic in facade
impl CrawlFacade {
    pub async fn crawl(&self, url: &str) -> Result<CrawlResult> {
        // URL normalization is business logic, doesn't belong here
        let normalized = url.trim().to_lowercase();
        // ...
    }
}

// ✅ CORRECT: Business logic in domain
// In riptide-types/src/domain/url.rs
impl Url {
    pub fn normalize(input: &str) -> Result<Url, ValidationError> {
        let trimmed = input.trim();
        // ... normalization logic
    }
}

// Facade uses domain logic
impl CrawlFacade {
    pub async fn crawl(&self, url: &str) -> Result<CrawlResult> {
        let normalized_url = Url::normalize(url)?;
        // ... orchestration
    }
}
```

### Guidelines for Keeping Logic Organized

**Checklist for adding new code:**

1. **Is it pure logic?** → Domain (riptide-types)
2. **Does it do I/O?** → Infrastructure crate
3. **Does it coordinate multiple operations?** → Facade (this crate)
4. **Does it handle HTTP requests?** → API layer (riptide-api)

**File organization:**

```
src/
  facades/
    mod.rs              # Re-exports
    scraper.rs          # Simple facades (~100-200 lines)
    crawl_facade.rs     # Wrapper facades (~200-300 lines)
    pipeline.rs         # Complex facades (~400-500 lines)
  workflows/
    transactional.rs    # Transaction coordination
    backpressure.rs     # Resource management
  authorization/
    policies.rs         # Authorization logic
  dto/
    document.rs         # Data transfer objects
    mapper.rs           # Domain ↔ DTO conversions
  metrics/
    business.rs         # Business metrics collection
```

**Testing organization:**

```
tests/
  common/
    mod.rs              # Shared test helpers
  scraper_facade_integration.rs
  crawl_facade_integration_tests.rs
  browser_facade_integration.rs
  composition_tests.rs
```

## Feature Flags

Optional functionality can be enabled via Cargo features:

```toml
[dependencies]
riptide-facade = { version = "0.9", features = ["llm", "workers"] }
```

### Available Features

| Feature | Description | Enables |
|---------|-------------|---------|
| `default` | Standard scraping and crawling | `llm` |
| `llm` | LLM-powered intelligence operations | `IntelligenceFacade`, `ProfileFacade` |
| `workers` | Background job processing | `WorkersFacade` |
| `wasm-extractor` | WASM-based content extraction | Experimental extraction engine |

## Performance Characteristics

| Operation | Typical Latency | Notes |
|-----------|----------------|-------|
| `ScraperFacade::fetch_html()` | 200-500ms | Network-bound |
| `CrawlFacade::crawl_single()` | 5-60s | Depends on depth/page count |
| `BrowserFacade::screenshot()` | 1-3s | Headless browser overhead |
| `ExtractionFacade::extract()` | 10-50ms | CPU-bound parsing |
| `TransactionalWorkflow::execute()` | +5-10ms | Idempotency check overhead |

**Optimization tips:**
- Use `CrawlMode::Standard` for simple crawls (faster than `Enhanced`)
- Enable caching to avoid redundant fetches
- Batch operations when possible (`crawl_batch`)
- Use backpressure management for resource-intensive operations

## Development Status

**Current Version:** 0.9.0

**Stability:**
- ✅ Core facades (Scraper, Browser, Extractor): **Stable**
- ✅ CrawlFacade: **Stable** (wraps production orchestrators)
- ✅ Transactional workflows: **Stable**
- ⚠️ Intelligence facades: **Beta** (LLM integration evolving)
- ⚠️ Workers facade: **Beta** (background job patterns maturing)

## Contributing

When contributing to riptide-facade:

1. **Follow hexagonal architecture**: Keep facades free of infrastructure details
2. **Use trait-based ports**: Depend on interfaces, not implementations
3. **Document architectural decisions**: Explain why patterns are used
4. **Add tests**: Both unit tests (mocks) and integration tests (real adapters)
5. **Update this README**: Keep documentation in sync with code

See the main [CONTRIBUTING.md](../../CONTRIBUTING.md) for general guidelines.

## License

Apache-2.0

---

## Quick Reference

**Key Files:**
- `src/lib.rs` - Public API and architectural rules
- `src/facades/mod.rs` - All available facades
- `src/workflows/transactional.rs` - ACID transaction pattern
- `src/builder.rs` - Configuration builder
- `Cargo.toml` - Dependencies and feature flags

**Related Documentation:**
- [Hexagonal Architecture Guide](../../docs/architecture/hexagonal-architecture.md)
- [Testing Guide](tests/README.md)
- [API Documentation](../../docs/api/README.md)

**Need Help?**
- Check inline code documentation (rich examples)
- Review integration tests for usage patterns
- See [FAQ](../../docs/FAQ.md)
