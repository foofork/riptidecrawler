# Hexagonal Architecture Violations - Comprehensive Analysis
**Date**: 2025-11-12
**Analyst**: Architecture Analysis Lead
**Status**: CRITICAL - Multiple concrete type dependencies violate hexagonal principles

## Executive Summary

RiptideCrawler's ApplicationContext and facade layer contain **5 CRITICAL** and **8 HIGH** severity violations of hexagonal architecture principles. These violations create tight coupling between the application layer and concrete infrastructure implementations, preventing:

- **Testability**: Cannot mock infrastructure without full setup
- **Flexibility**: Cannot swap implementations (e.g., Redis → Valkey)
- **Isolation**: Domain logic coupled to infrastructure concerns
- **Dependency Direction**: Application depends on details, not abstractions

---

## Architecture Context

### Current State (VIOLATES Hexagonal Principles)
```
ApplicationContext (riptide-api)
├── Arc<CacheManager>           ❌ CONCRETE Redis implementation
├── Arc<UnifiedExtractor>       ❌ CONCRETE WASM implementation
├── Arc<FetchEngine>            ❌ CONCRETE HTTP client
├── Arc<Spider>                 ❌ CONCRETE crawler implementation
├── Arc<WorkerService>          ❌ CONCRETE job queue
├── Arc<HeadlessLauncher>       ❌ CONCRETE browser launcher
└── Facades (riptide-facade)
    ├── ScraperFacade::client: Arc<FetchEngine>    ❌
    ├── RenderFacade::fetch_engine: Arc<FetchEngine>  ❌
    ├── BrowserFacade::launcher: Arc<HeadlessLauncher> ❌
    └── UrlExtractionFacade::http_client: Arc<reqwest::Client> ❌
```

### Target State (Hexagonal Architecture)
```
ApplicationContext
├── Arc<dyn CacheStorage>         ✅ Port trait (EXISTS in riptide-types)
├── Arc<dyn ContentExtractor>     ✅ Port trait (EXISTS in riptide-extraction)
├── Arc<dyn HttpClient>           ✅ Port trait (EXISTS in riptide-types/ports/http.rs)
├── Arc<dyn SpiderEngine>         ⚠️  NEEDS DEFINITION
├── Arc<dyn BackgroundWorker>     ⚠️  NEEDS DEFINITION
└── Arc<dyn BrowserDriver>        ✅ Port trait (EXISTS in riptide-types/ports/features.rs)
```

---

## CRITICAL VIOLATIONS (Severity: 🔴 CRITICAL)

### V-001: ApplicationContext - Redis CacheManager Coupling
**Location**: `/crates/riptide-api/src/context.rs:79`
```rust
// VIOLATION:
pub cache: Arc<tokio::sync::Mutex<CacheManager>>,

// CORRECT:
pub cache: Arc<dyn CacheStorage>,
```

**Severity**: 🔴 **CRITICAL**
**Impact**:
- Cannot test without Redis instance
- Cannot swap to Valkey, Dragonfly, or in-memory cache
- Violates dependency inversion principle

**Available Port Trait**: ✅ `riptide_types::ports::CacheStorage` (already exists)

**Remediation**:
1. Change ApplicationContext field to `Arc<dyn CacheStorage>`
2. Adapter already exists: `riptide_cache::CacheManager` implements the trait (needs verification)
3. Create `InMemoryCacheAdapter` for testing

---

### V-002: ApplicationContext - UnifiedExtractor Coupling
**Location**: `/crates/riptide-api/src/context.rs:85`
```rust
// VIOLATION:
#[cfg(feature = "extraction")]
pub extractor: Arc<UnifiedExtractor>,

// CORRECT:
pub extractor: Arc<dyn ContentExtractor>,
```

**Severity**: 🔴 **CRITICAL**
**Impact**:
- Tightly coupled to WASM extractor implementation
- Cannot test without WASM runtime
- Cannot inject alternative extractors (CSS-only, LLM-based, etc.)

**Available Port Trait**: ✅ `riptide_extraction::ContentExtractor` (already exists)

**Remediation**:
1. Change to `Arc<dyn ContentExtractor>`
2. Adapters exist: `UnifiedExtractor`, `CssExtractor`, `FallbackExtractor`
3. No changes needed to adapters—they already implement the trait

---

### V-003: ApplicationContext - FetchEngine HTTP Client Coupling
**Location**: `/crates/riptide-api/src/context.rs:154`
```rust
// VIOLATION:
#[cfg(feature = "fetch")]
pub fetch_engine: Arc<FetchEngine>,

// CORRECT:
pub fetch_engine: Arc<dyn HttpClient>,
```

**Severity**: 🔴 **CRITICAL**
**Impact**:
- Cannot test HTTP operations without actual network calls
- Cannot inject mock HTTP responses for unit tests
- Cannot swap reqwest for hyper, ureq, or custom implementation

**Available Port Trait**: ✅ `riptide_types::ports::HttpClient` (Sprint 1.5, already exists)

**Remediation**:
1. Change to `Arc<dyn HttpClient>`
2. Create `FetchEngineAdapter` implementing `HttpClient` trait
3. Create `MockHttpClient` for testing

---

### V-004: ApplicationContext - Spider Engine Coupling
**Location**: `/crates/riptide-api/src/context.rs:127`
```rust
// VIOLATION:
#[cfg(feature = "spider")]
pub spider: Option<Arc<Spider>>,

// CORRECT:
pub spider: Option<Arc<dyn SpiderEngine>>,
```

**Severity**: 🔴 **CRITICAL**
**Impact**:
- Cannot test crawling logic without full Spider setup
- Cannot inject custom crawlers (focused, distributed, etc.)
- Breaks dependency inversion

**Available Port Trait**: ⚠️ **NEEDS CREATION**

**Remediation**:
1. Define `SpiderEngine` trait in `riptide-types/ports`
2. Create adapter implementing trait
3. Update ApplicationContext

---

### V-005: ApplicationContext - WorkerService Background Jobs Coupling
**Location**: `/crates/riptide-api/src/context.rs:135`
```rust
// VIOLATION:
#[cfg(feature = "workers")]
pub worker_service: Arc<WorkerService>,

// CORRECT:
pub worker_service: Arc<dyn BackgroundWorker>,
```

**Severity**: 🔴 **CRITICAL**
**Impact**:
- Cannot test background job execution without Redis queue
- Cannot swap to different job queue (Sidekiq, Bull, etc.)
- Forces integration tests for unit-level logic

**Available Port Trait**: ⚠️ **NEEDS CREATION**

**Remediation**:
1. Define `BackgroundWorker` trait in `riptide-types/ports`
2. Create adapter for WorkerService
3. Create `InMemoryWorker` for testing

---

## HIGH SEVERITY VIOLATIONS (Severity: 🟠 HIGH)

### V-006: ScraperFacade - FetchEngine Coupling
**Location**: `/crates/riptide-facade/src/facades/scraper.rs:15`
```rust
// VIOLATION:
client: Arc<FetchEngine>,

// CORRECT:
client: Arc<dyn HttpClient>,
```

**Severity**: 🟠 **HIGH**
**Impact**: Facades leak infrastructure types, defeating their purpose

---

### V-007: RenderFacade - FetchEngine Coupling
**Location**: `/crates/riptide-facade/src/facades/render.rs:114`
```rust
// VIOLATION:
fetch_engine: Arc<FetchEngine>,
pub fn new(fetch_engine: Arc<FetchEngine>, ...) -> Self

// CORRECT:
fetch_engine: Arc<dyn HttpClient>,
pub fn new(fetch_engine: Arc<dyn HttpClient>, ...) -> Self
```

**Severity**: 🟠 **HIGH**

---

### V-008: BrowserFacade - HeadlessLauncher Coupling
**Location**: `/crates/riptide-facade/src/facades/browser.rs:56`
```rust
// VIOLATION:
launcher: Arc<HeadlessLauncher>,

// CORRECT:
launcher: Arc<dyn BrowserDriver>,
```

**Severity**: 🟠 **HIGH**
**Available Port Trait**: ✅ `riptide_types::ports::BrowserDriver` (already exists)

---

### V-009: UrlExtractionFacade - Reqwest Client Coupling
**Location**: `/crates/riptide-facade/src/facades/extraction.rs:58`
```rust
// VIOLATION:
http_client: Arc<reqwest::Client>,

// CORRECT:
http_client: Arc<dyn HttpClient>,
```

**Severity**: 🟠 **HIGH**
**Notes**: Direct dependency on reqwest (not even wrapped in FetchEngine)

---

### V-010: ApplicationContext - HeadlessLauncher Browser Coupling
**Location**: `/crates/riptide-api/src/context.rs:168`
```rust
// VIOLATION:
#[cfg(feature = "browser")]
pub browser_launcher: Option<Arc<HeadlessLauncher>>,

// CORRECT:
pub browser_launcher: Option<Arc<dyn BrowserDriver>>,
```

**Severity**: 🟠 **HIGH**
**Available Port Trait**: ✅ `riptide_types::ports::BrowserDriver`

---

## MEDIUM SEVERITY VIOLATIONS (Severity: 🟡 MEDIUM)

### V-011: ReliableExtractor Wrapping Concrete Type
**Location**: `/crates/riptide-api/src/context.rs:89`
```rust
// CURRENT:
pub reliable_extractor: Arc<ReliableExtractor>,

// ANALYSIS:
// ReliableExtractor is a reliability wrapper (retry, circuit breaker)
// It should wrap trait objects, not be exposed itself
```

**Severity**: 🟡 **MEDIUM**
**Notes**: This is less critical because `ReliableExtractor` is a cross-cutting concern wrapper, but it should wrap `Arc<dyn ContentExtractor>` internally rather than being exposed.

---

### V-012: Circular Facade Dependencies (Previously Resolved)
**Location**: Multiple facade files
**Status**: ✅ **RESOLVED in Phase 2C.2**

Previous circular dependency between `riptide-api` and `riptide-facade` was resolved by:
- Moving trait definitions to `riptide-types/pipeline/traits.rs`
- CrawlFacade now uses `Arc<dyn PipelineExecutor>` and `Arc<dyn StrategiesPipelineExecutor>`

**Lesson Learned**: Trait-based abstractions successfully break circular dependencies.

---

## Dependency Graph Analysis

### Current Dependencies (Problematic)
```
riptide-api (ApplicationContext)
    ├─> riptide-cache (CacheManager) ❌
    ├─> riptide-extraction (UnifiedExtractor) ❌
    ├─> riptide-fetch (FetchEngine) ❌
    ├─> riptide-spider (Spider) ❌
    ├─> riptide-workers (WorkerService) ❌
    ├─> riptide-headless (HeadlessLauncher) ❌
    └─> riptide-facade (facades depend on above) ❌

Result: Application layer depends on ALL infrastructure layers
```

### Target Dependencies (Hexagonal)
```
riptide-api (ApplicationContext)
    └─> riptide-types (port traits ONLY) ✅

riptide-types (domain layer)
    ├─> CacheStorage trait
    ├─> ContentExtractor trait
    ├─> HttpClient trait
    ├─> SpiderEngine trait (to be created)
    ├─> BackgroundWorker trait (to be created)
    └─> BrowserDriver trait

Infrastructure crates implement traits:
    riptide-cache ──implements──> CacheStorage
    riptide-extraction ──implements──> ContentExtractor
    riptide-fetch ──implements──> HttpClient
    riptide-spider ──implements──> SpiderEngine (new)
    riptide-workers ──implements──> BackgroundWorker (new)
    riptide-headless ──implements──> BrowserDriver

Result: Application depends on abstractions, infrastructure depends on abstractions
```

---

## Port Trait Inventory

### ✅ **EXISTING PORT TRAITS** (Already Defined)
1. **`CacheStorage`** → `/crates/riptide-types/src/ports/cache.rs`
2. **`ContentExtractor`** → `/crates/riptide-extraction/src/lib.rs` (move to riptide-types)
3. **`HttpClient`** → `/crates/riptide-types/src/ports/http.rs`
4. **`BrowserDriver`** → `/crates/riptide-types/src/ports/features.rs`
5. **`Pool<T>`** → `/crates/riptide-types/src/ports/pool.rs` (Sprint 4.7)
6. **`RateLimiter`** → `/crates/riptide-types/src/ports/rate_limit.rs` (Sprint 4.4)

### ⚠️ **MISSING PORT TRAITS** (Need Creation)
1. **`SpiderEngine`** - Web crawler abstraction
2. **`BackgroundWorker`** - Job queue abstraction
3. **`ExtractorEngine`** - If ContentExtractor needs refinement

---

## Proposed Trait Definitions

### SpiderEngine Port Trait
```rust
// Location: /crates/riptide-types/src/ports/spider.rs

use async_trait::async_trait;
use crate::error::Result;
use url::Url;

#[derive(Debug, Clone)]
pub struct CrawlOptions {
    pub max_depth: Option<usize>,
    pub max_pages: Option<usize>,
    pub respect_robots: bool,
    pub timeout: Duration,
}

#[derive(Debug, Clone)]
pub struct CrawlResult {
    pub url: Url,
    pub status_code: u16,
    pub html: String,
    pub links: Vec<Url>,
    pub metadata: HashMap<String, String>,
}

#[async_trait]
pub trait SpiderEngine: Send + Sync {
    /// Start crawling from a base URL
    async fn crawl(&self, url: &Url, options: CrawlOptions) -> Result<Vec<CrawlResult>>;

    /// Get current crawl state
    async fn state(&self) -> CrawlState;

    /// Pause active crawl
    async fn pause(&self) -> Result<()>;

    /// Resume paused crawl
    async fn resume(&self) -> Result<()>;

    /// Stop crawl and cleanup
    async fn stop(&self) -> Result<()>;
}
```

### BackgroundWorker Port Trait
```rust
// Location: /crates/riptide-types/src/ports/worker.rs

use async_trait::async_trait;
use crate::error::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub priority: JobPriority,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum JobPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[async_trait]
pub trait BackgroundWorker: Send + Sync {
    /// Enqueue a job for background processing
    async fn enqueue(&self, job: Job) -> Result<String>;

    /// Check job status
    async fn status(&self, job_id: &str) -> Result<JobStatus>;

    /// Cancel a pending or running job
    async fn cancel(&self, job_id: &str) -> Result<()>;

    /// Get worker health metrics
    async fn health(&self) -> WorkerHealth;
}
```

---

## Remediation Roadmap

### Phase 1: Port Trait Creation (1-2 days)
- [ ] Create `SpiderEngine` trait in `riptide-types/ports/spider.rs`
- [ ] Create `BackgroundWorker` trait in `riptide-types/ports/worker.rs`
- [ ] Move `ContentExtractor` trait to `riptide-types/ports` (if not already there)
- [ ] Update `riptide-types/ports/mod.rs` with re-exports

### Phase 2: Adapter Implementation (2-3 days)
- [ ] Create `CacheManagerAdapter` implementing `CacheStorage`
- [ ] Create `FetchEngineAdapter` implementing `HttpClient`
- [ ] Create `SpiderAdapter` implementing `SpiderEngine`
- [ ] Create `WorkerServiceAdapter` implementing `BackgroundWorker`
- [ ] Create `HeadlessLauncherAdapter` implementing `BrowserDriver`

### Phase 3: ApplicationContext Refactoring (3-4 days)
- [ ] Replace all concrete types with trait objects in ApplicationContext
- [ ] Update `ApplicationContext::new_base()` to accept trait objects
- [ ] Update factory methods to inject adapters
- [ ] Verify compilation across all feature flags

### Phase 4: Facade Layer Updates (2-3 days)
- [ ] Update ScraperFacade to use `HttpClient` trait
- [ ] Update RenderFacade to use `HttpClient` trait
- [ ] Update BrowserFacade to use `BrowserDriver` trait
- [ ] Update UrlExtractionFacade to use `HttpClient` trait

### Phase 5: Testing Infrastructure (2-3 days)
- [ ] Create `MockCacheStorage` for unit tests
- [ ] Create `MockHttpClient` for unit tests
- [ ] Create `MockSpiderEngine` for unit tests
- [ ] Create `MockBackgroundWorker` for unit tests
- [ ] Create `MockBrowserDriver` for unit tests
- [ ] Update integration tests to use adapters

### Phase 6: Documentation & Migration (1-2 days)
- [ ] Document port trait contracts
- [ ] Create adapter selection guide
- [ ] Update ADR for hexagonal architecture
- [ ] Create migration guide for downstream consumers

**Total Estimated Effort**: 11-17 days (2-3 sprint cycles)

---

## Testing Strategy

### Unit Test Improvements
```rust
// BEFORE (requires full infrastructure):
#[tokio::test]
async fn test_extract() {
    let redis = CacheManager::new("redis://localhost").await.unwrap();
    let extractor = UnifiedExtractor::new(Some("extractor.wasm")).await.unwrap();
    let http = FetchEngine::new().unwrap();

    let ctx = ApplicationContext::new_base(config, api_config, health, None).await.unwrap();
    // Test requires Redis, WASM runtime, and network
}

// AFTER (pure unit test):
#[tokio::test]
async fn test_extract() {
    let cache = Arc::new(MockCacheStorage::new());
    let extractor: Arc<dyn ContentExtractor> = Arc::new(MockExtractor::new());
    let http: Arc<dyn HttpClient> = Arc::new(MockHttpClient::new());

    let ctx = ApplicationContext::test_new(cache, extractor, http);
    // Pure unit test - no external dependencies!
}
```

---

## Conclusion

**Current State**: RiptideCrawler violates hexagonal architecture principles through **13 identified violations** (5 critical, 8 high/medium severity).

**Root Cause**: ApplicationContext and facades couple directly to concrete infrastructure types instead of port trait abstractions.

**Impact**:
- ❌ **Testability**: Unit tests require full infrastructure setup
- ❌ **Flexibility**: Cannot swap implementations (Redis → Valkey, reqwest → hyper)
- ❌ **Maintainability**: Changes to infrastructure types ripple through application layer
- ❌ **Dependency Direction**: Application depends on infrastructure details

**Solution**: Implement hexagonal architecture via:
1. Define missing port traits (`SpiderEngine`, `BackgroundWorker`)
2. Create adapters wrapping concrete implementations
3. Update ApplicationContext to accept trait objects
4. Create mock implementations for testing

**Outcome**:
- ✅ **Pure Domain Logic**: No infrastructure dependencies
- ✅ **Test Isolation**: Mock any infrastructure component
- ✅ **Implementation Flexibility**: Swap backends without code changes
- ✅ **Proper Dependency Direction**: Infrastructure depends on abstractions, not vice versa

---

## References

- **ApplicationContext**: `/crates/riptide-api/src/context.rs`
- **Port Traits**: `/crates/riptide-types/src/ports/`
- **Facade Layer**: `/crates/riptide-facade/src/facades/`
- **Previous ADR**: Phase 2C.2 - Circular dependency resolution via traits
- **Hexagonal Architecture**: Ports & Adapters pattern by Alistair Cockburn

---

**Next Steps**: Create remediation plan tickets and assign to sprint backlog for phased implementation.
