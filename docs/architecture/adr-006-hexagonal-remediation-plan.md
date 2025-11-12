# ADR-006: Hexagonal Architecture Remediation Plan

## Status
**PROPOSED** - Awaiting approval for sprint planning

## Context

A comprehensive architecture analysis (2025-11-12) identified **13 violations** of hexagonal architecture principles in RiptideCrawler's ApplicationContext and facade layer. These violations create tight coupling between the application layer and concrete infrastructure implementations, preventing:

1. **Test Isolation**: Unit tests require full infrastructure (Redis, WASM runtime, network)
2. **Implementation Flexibility**: Cannot swap backends (Redis → Valkey, reqwest → hyper)
3. **Dependency Direction**: Application layer depends on infrastructure details, not abstractions
4. **Domain Purity**: Business logic contaminated with infrastructure concerns

### Severity Breakdown
- **5 CRITICAL** violations (ApplicationContext concrete types)
- **5 HIGH** violations (Facade concrete types)
- **3 MEDIUM** violations (Wrapper patterns)

### Root Cause
ApplicationContext was designed before port trait infrastructure was established. Fields were added incrementally with concrete types rather than abstractions.

## Decision

**Adopt trait-based dependency injection across all ApplicationContext fields and facades**, following the hexagonal architecture pattern (Ports & Adapters).

### Architectural Principle
```
Application Layer (riptide-api)
    ↓ depends on ↓
Domain Layer (riptide-types) ← defines ports (traits)
    ↑ implemented by ↑
Infrastructure Layer (riptide-*)  ← adapters
```

**Key Changes:**
1. Replace all `Arc<ConcreteType>` with `Arc<dyn PortTrait>` in ApplicationContext
2. Define missing port traits: `SpiderEngine`, `BackgroundWorker`
3. Create adapters wrapping concrete implementations
4. Update facades to accept trait objects
5. Provide mock implementations for testing

## Detailed Remediation

### Phase 1: Port Trait Creation (Priority: HIGH)

#### 1.1 Create SpiderEngine Port Trait
**Location**: `/crates/riptide-types/src/ports/spider.rs`

```rust
use async_trait::async_trait;
use crate::error::Result;
use url::Url;
use std::time::Duration;
use std::collections::HashMap;

/// Configuration for spider crawling operations
#[derive(Debug, Clone)]
pub struct CrawlOptions {
    /// Maximum crawl depth (None = unlimited)
    pub max_depth: Option<usize>,
    /// Maximum pages to crawl (None = unlimited)
    pub max_pages: Option<usize>,
    /// Respect robots.txt rules
    pub respect_robots: bool,
    /// Per-request timeout
    pub timeout: Duration,
    /// Concurrent requests limit
    pub concurrency: usize,
    /// Delay between requests to same host
    pub delay: Duration,
}

/// Result of crawling a single page
#[derive(Debug, Clone)]
pub struct PageCrawlResult {
    pub url: Url,
    pub status_code: u16,
    pub html: String,
    pub links: Vec<Url>,
    pub metadata: HashMap<String, String>,
    pub crawl_time: Duration,
}

/// Current state of spider crawl
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrawlState {
    Idle,
    Running,
    Paused,
    Completed,
    Failed,
}

/// Spider engine port trait for web crawling
///
/// Implementations provide multi-page crawling with configurable
/// depth, concurrency, and rate limiting.
#[async_trait]
pub trait SpiderEngine: Send + Sync {
    /// Start crawling from base URL
    ///
    /// # Arguments
    /// * `url` - Starting URL for crawl
    /// * `options` - Crawl configuration
    ///
    /// # Returns
    /// Vector of crawled page results
    async fn crawl(&self, url: &Url, options: CrawlOptions) -> Result<Vec<PageCrawlResult>>;

    /// Get current crawl state
    async fn state(&self) -> CrawlState;

    /// Pause active crawl (can be resumed)
    async fn pause(&self) -> Result<()>;

    /// Resume paused crawl
    async fn resume(&self) -> Result<()>;

    /// Stop crawl and cleanup resources
    async fn stop(&self) -> Result<()>;
}
```

**Adapter Implementation**: Create `/crates/riptide-spider/src/adapter.rs`
```rust
pub struct SpiderAdapter {
    inner: Arc<Spider>,
}

#[async_trait]
impl SpiderEngine for SpiderAdapter {
    async fn crawl(&self, url: &Url, options: CrawlOptions) -> Result<Vec<PageCrawlResult>> {
        // Delegate to concrete Spider implementation
        self.inner.crawl_url(url, options).await
    }

    // ... implement other methods
}
```

#### 1.2 Create BackgroundWorker Port Trait
**Location**: `/crates/riptide-types/src/ports/worker.rs`

```rust
use async_trait::async_trait;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Job priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Background job definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// Unique job identifier
    pub id: String,
    /// Job type identifier (for routing to handlers)
    pub job_type: String,
    /// JSON payload
    pub payload: serde_json::Value,
    /// Job priority
    pub priority: JobPriority,
    /// Maximum retry attempts
    pub max_retries: u32,
    /// Job timeout
    pub timeout: Duration,
}

/// Job execution status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed { error: String, retries: u32 },
    Cancelled,
}

/// Worker health metrics
#[derive(Debug, Clone)]
pub struct WorkerHealth {
    pub active_workers: usize,
    pub pending_jobs: usize,
    pub processing_jobs: usize,
    pub failed_jobs_last_hour: usize,
}

/// Background worker port trait
///
/// Provides asynchronous job processing with priorities,
/// retries, and health monitoring.
#[async_trait]
pub trait BackgroundWorker: Send + Sync {
    /// Enqueue a job for background processing
    ///
    /// # Arguments
    /// * `job` - Job to enqueue
    ///
    /// # Returns
    /// Job ID for tracking
    async fn enqueue(&self, job: Job) -> Result<String>;

    /// Check status of a specific job
    async fn status(&self, job_id: &str) -> Result<JobStatus>;

    /// Cancel a pending or running job
    async fn cancel(&self, job_id: &str) -> Result<()>;

    /// Get worker pool health metrics
    async fn health(&self) -> WorkerHealth;
}
```

**Adapter Implementation**: Create `/crates/riptide-workers/src/adapter.rs`
```rust
pub struct WorkerServiceAdapter {
    inner: Arc<WorkerService>,
}

#[async_trait]
impl BackgroundWorker for WorkerServiceAdapter {
    async fn enqueue(&self, job: Job) -> Result<String> {
        // Convert port Job to internal JobMessage and delegate
        self.inner.enqueue_job(job.into()).await
    }

    // ... implement other methods
}
```

### Phase 2: ApplicationContext Refactoring

#### 2.1 Update ApplicationContext Fields
**Location**: `/crates/riptide-api/src/context.rs`

```rust
// BEFORE:
pub struct ApplicationContext {
    pub http_client: Client,
    pub cache: Arc<tokio::sync::Mutex<CacheManager>>,           // ❌ Concrete
    pub extractor: Arc<UnifiedExtractor>,                        // ❌ Concrete
    pub fetch_engine: Arc<FetchEngine>,                          // ❌ Concrete
    pub spider: Option<Arc<Spider>>,                             // ❌ Concrete
    pub worker_service: Arc<WorkerService>,                      // ❌ Concrete
    pub browser_launcher: Option<Arc<HeadlessLauncher>>,        // ❌ Concrete
    // ... other fields
}

// AFTER:
pub struct ApplicationContext {
    pub http_client: Arc<dyn HttpClient>,                       // ✅ Port trait
    pub cache: Arc<dyn CacheStorage>,                           // ✅ Port trait
    pub extractor: Arc<dyn ContentExtractor>,                   // ✅ Port trait
    pub spider: Option<Arc<dyn SpiderEngine>>,                  // ✅ Port trait
    pub worker_service: Arc<dyn BackgroundWorker>,              // ✅ Port trait
    pub browser_launcher: Option<Arc<dyn BrowserDriver>>,       // ✅ Port trait
    // ... other fields remain unchanged
}
```

#### 2.2 Update Constructor to Accept Trait Objects
```rust
impl ApplicationContext {
    pub async fn new_base(
        config: AppConfig,
        api_config: RiptideApiConfig,
        health_checker: Arc<HealthChecker>,
        telemetry: Option<Arc<TelemetrySystem>>,
        // NEW: Accept trait objects for dependency injection
        cache: Arc<dyn CacheStorage>,
        extractor: Arc<dyn ContentExtractor>,
        http_client: Arc<dyn HttpClient>,
        spider: Option<Arc<dyn SpiderEngine>>,
        worker: Arc<dyn BackgroundWorker>,
        browser: Option<Arc<dyn BrowserDriver>>,
    ) -> Result<Self> {
        // ... rest of initialization

        Ok(Self {
            http_client,
            cache,
            extractor,
            spider,
            worker_service: worker,
            browser_launcher: browser,
            // ... other fields
        })
    }
}
```

#### 2.3 Create Production Factory Method
```rust
impl ApplicationContext {
    /// Create ApplicationContext with production adapters
    pub async fn new_production(
        config: AppConfig,
        api_config: RiptideApiConfig,
    ) -> Result<Self> {
        // Create concrete implementations
        let cache_mgr = CacheManager::new(&config.redis_url).await?;
        let cache: Arc<dyn CacheStorage> = Arc::new(CacheManagerAdapter::new(cache_mgr));

        let unified_ext = UnifiedExtractor::new(Some(&config.wasm_path)).await?;
        let extractor: Arc<dyn ContentExtractor> = Arc::new(unified_ext);

        let fetch_eng = FetchEngine::new()?;
        let http_client: Arc<dyn HttpClient> = Arc::new(FetchEngineAdapter::new(fetch_eng));

        let spider = if let Some(cfg) = config.spider_config {
            let spider_impl = Spider::new(cfg).await?;
            Some(Arc::new(SpiderAdapter::new(spider_impl)) as Arc<dyn SpiderEngine>)
        } else {
            None
        };

        let worker_svc = WorkerService::new(config.worker_config).await?;
        let worker: Arc<dyn BackgroundWorker> = Arc::new(WorkerServiceAdapter::new(worker_svc));

        let browser = if config.headless_url.is_none() {
            let launcher = HeadlessLauncher::new().await?;
            Some(Arc::new(launcher) as Arc<dyn BrowserDriver>)
        } else {
            None
        };

        // Delegate to new_base with adapters
        Self::new_base(
            config,
            api_config,
            Arc::new(HealthChecker::new()),
            None,
            cache,
            extractor,
            http_client,
            spider,
            worker,
            browser,
        ).await
    }
}
```

### Phase 3: Facade Layer Updates

#### 3.1 ScraperFacade Refactoring
```rust
// BEFORE:
pub struct ScraperFacade {
    config: Arc<RiptideConfig>,
    client: Arc<FetchEngine>,  // ❌ Concrete
}

impl ScraperFacade {
    pub async fn new(config: RiptideConfig) -> RiptideResult<Self> {
        let client = Arc::new(FetchEngine::new()?);
        Ok(Self { config: Arc::new(config), client })
    }
}

// AFTER:
pub struct ScraperFacade {
    config: Arc<RiptideConfig>,
    client: Arc<dyn HttpClient>,  // ✅ Port trait
}

impl ScraperFacade {
    pub fn new(config: RiptideConfig, client: Arc<dyn HttpClient>) -> Self {
        Self { config: Arc::new(config), client }
    }

    // Factory method for production use
    pub async fn new_default(config: RiptideConfig) -> RiptideResult<Self> {
        let fetch_engine = FetchEngine::new()?;
        let client: Arc<dyn HttpClient> = Arc::new(FetchEngineAdapter::new(fetch_engine));
        Ok(Self::new(config, client))
    }
}
```

### Phase 4: Testing Infrastructure

#### 4.1 Mock Implementations
```rust
// /crates/riptide-types/src/ports/mocks/cache.rs
pub struct MockCacheStorage {
    data: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

#[async_trait]
impl CacheStorage for MockCacheStorage {
    async fn get(&self, key: &str) -> RiptideResult<Option<Vec<u8>>> {
        Ok(self.data.read().await.get(key).cloned())
    }

    async fn set(&self, key: &str, value: &[u8], _ttl: Option<Duration>) -> RiptideResult<()> {
        self.data.write().await.insert(key.to_string(), value.to_vec());
        Ok(())
    }

    // ... other methods
}

// /crates/riptide-types/src/ports/mocks/http.rs
pub struct MockHttpClient {
    responses: Arc<RwLock<HashMap<String, HttpResponse>>>,
}

impl MockHttpClient {
    pub fn new() -> Self {
        Self { responses: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub async fn set_response(&self, url: &str, response: HttpResponse) {
        self.responses.write().await.insert(url.to_string(), response);
    }
}

#[async_trait]
impl HttpClient for MockHttpClient {
    async fn get(&self, url: &str) -> Result<HttpResponse> {
        self.responses.read().await
            .get(url)
            .cloned()
            .ok_or_else(|| RiptideError::Other("URL not mocked".into()))
    }

    // ... other methods
}
```

#### 4.2 Test Helper
```rust
// /crates/riptide-api/src/testing.rs
impl ApplicationContext {
    /// Create test context with mock implementations
    pub fn new_test() -> Self {
        let cache: Arc<dyn CacheStorage> = Arc::new(MockCacheStorage::new());
        let extractor: Arc<dyn ContentExtractor> = Arc::new(MockExtractor::new());
        let http: Arc<dyn HttpClient> = Arc::new(MockHttpClient::new());
        let spider = None; // Optional in tests
        let worker: Arc<dyn BackgroundWorker> = Arc::new(MockWorker::new());
        let browser = None; // Optional in tests

        Self::new_base(
            AppConfig::default(),
            RiptideApiConfig::default(),
            Arc::new(HealthChecker::new()),
            None,
            cache,
            extractor,
            http,
            spider,
            worker,
            browser,
        )
        .await
        .expect("Test context creation failed")
    }
}
```

## Consequences

### Positive
1. **✅ Test Isolation**: Unit tests no longer require Redis, WASM, or network
2. **✅ Flexibility**: Can swap implementations without code changes
3. **✅ Domain Purity**: Application layer depends only on abstractions
4. **✅ Proper Dependency Direction**: Infrastructure → Domain (not Domain → Infrastructure)
5. **✅ Maintainability**: Changes to infrastructure don't ripple through application
6. **✅ Testability**: Mock any component for unit tests

### Negative
1. **⚠️ Complexity**: Additional adapter layer for each infrastructure component
2. **⚠️ Boilerplate**: More code for trait implementations and adapters
3. **⚠️ Migration Effort**: 11-17 days to complete full remediation
4. **⚠️ Learning Curve**: Team must understand trait objects and dynamic dispatch

### Risks & Mitigation
| Risk | Impact | Mitigation |
|------|--------|------------|
| Performance overhead (dynamic dispatch) | Low | Profile hot paths; trait objects have minimal overhead |
| Existing tests break | Medium | Update incrementally; maintain compatibility layer |
| Adapter bugs | Medium | Comprehensive adapter unit tests; integration tests unchanged |
| Team resistance | Low | Show benefits via improved test speed and flexibility |

## Implementation Plan

### Sprint 1: Foundation (Days 1-5)
- [ ] Create `SpiderEngine` port trait
- [ ] Create `BackgroundWorker` port trait
- [ ] Create adapter interfaces for all concrete types
- [ ] Update `riptide-types/ports/mod.rs` with re-exports

### Sprint 2: ApplicationContext (Days 6-10)
- [ ] Implement all adapters (CacheManager, FetchEngine, Spider, Worker, Headless)
- [ ] Update ApplicationContext fields to trait objects
- [ ] Create `new_production()` factory method
- [ ] Create `new_test()` factory method with mocks

### Sprint 3: Facades & Testing (Days 11-17)
- [ ] Update ScraperFacade, RenderFacade, BrowserFacade
- [ ] Create mock implementations for all traits
- [ ] Migrate unit tests to use mocks
- [ ] Update integration tests to use production factory
- [ ] Documentation and ADR finalization

### Success Criteria
- [ ] All `Arc<ConcreteType>` replaced with `Arc<dyn PortTrait>`
- [ ] Zero compilation errors across all feature flags
- [ ] All tests pass with same coverage (95%+)
- [ ] Unit tests run without Redis/network (30% faster)
- [ ] Documentation updated with new patterns

## Alternatives Considered

### Alternative 1: Keep Concrete Types (Status Quo)
**Rejected**: Continues violations, prevents testing and flexibility

### Alternative 2: Partial Remediation (Only Critical Violations)
**Rejected**: Inconsistent architecture, technical debt remains

### Alternative 3: Generic Type Parameters Instead of Trait Objects
```rust
pub struct ApplicationContext<C, E, H>
where
    C: CacheStorage,
    E: ContentExtractor,
    H: HttpClient,
{
    cache: Arc<C>,
    extractor: Arc<E>,
    http_client: Arc<H>,
}
```

**Rejected**:
- Explosion of generic parameters (9+ type parameters)
- Handler functions become unreadable with generics
- Cannot store in Arc without type erasure
- Trait objects are the idiomatic Rust solution for dependency injection

## References

- **Analysis Document**: `/docs/architecture/hexagonal-violations-analysis.md`
- **Port Traits**: `/crates/riptide-types/src/ports/`
- **ApplicationContext**: `/crates/riptide-api/src/context.rs`
- **Previous ADR**: ADR-002 - Circular dependency resolution (Phase 2C.2)
- **Hexagonal Architecture**: Alistair Cockburn's Ports & Adapters pattern
- **Rust Trait Objects**: [The Rust Book - Trait Objects](https://doc.rust-lang.org/book/ch17-02-trait-objects.html)

## Approval

**Proposed By**: Architecture Analysis Lead
**Date**: 2025-11-12
**Status**: AWAITING APPROVAL

**Required Approvals**:
- [ ] Tech Lead
- [ ] Product Owner (timeline confirmation)
- [ ] DevOps (infrastructure impact review)

---

**Next Steps After Approval**:
1. Create epic in project management tool
2. Break down into sprintable stories
3. Assign to team members based on expertise
4. Begin Sprint 1 implementation
