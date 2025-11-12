# AppState to ApplicationContext Migration: One-Shot Implementation Plan

**Project**: Riptide Event Mesh
**Timeline**: 3 weeks (15 business days, 120 hours)
**Priority**: P0 (Critical)
**Approach**: One-shot migration replacing incremental sprint-based approach
**Goal**: Transform facade layer to production-ready hexagonal architecture in a single coordinated effort

---

## Executive Summary

This plan consolidates Sprints 1-3 from the original 12-week roadmap into a single **3-week one-shot migration**. This approach reduces context switching, simplifies feature flag management, and delivers a clean architecture faster.

**Key Problems Being Solved**:
- **P0**: AppState god object (2213 lines, 40+ fields)
- **P0**: Competing state systems (AppState vs ApplicationContext)
- **P0**: Circular dependencies between riptide-api ↔ riptide-facade
- **P1**: 32 infrastructure violations, 12 untested facades

**Success Metrics**:
- AppState reduced from 2213 lines to <200 lines (90% reduction)
- 100% facade test coverage (12 facades)
- Zero circular dependencies (`cargo tree` verification)
- All infrastructure accessed via port traits
- Clean hexagonal boundaries enforced

---

## Benefits of One-Shot Approach

### Advantages Over Incremental Sprints
1. **Reduced Context Switching**: Complete migration in single flow instead of 3 separate sprints
2. **Simpler Feature Flags**: Single migration flag instead of intermediate states
3. **Faster Time to Value**: 3 weeks vs original 3+ weeks of gradual migration
4. **Cleaner Rollback**: All-or-nothing rollback strategy (simpler than partial migrations)
5. **Better Testing**: Comprehensive end-to-end testing in single phase
6. **Less Risk**: No intermediate states that could cause production issues

### Trade-offs Accepted
- Higher upfront complexity (balanced by better coordination)
- Requires dedicated 3-week focus (acceptable for P0 critical work)
- Larger initial code review (offset by cleaner final state)

---

## Pre-Migration Setup (Week 0)

### Prerequisites
- [ ] Backup current working state: `git tag pre-appstate-migration`
- [ ] Verify all tests pass: `cargo test --workspace`
- [ ] Document current metrics baseline
- [ ] Set up feature flag infrastructure
- [ ] Create migration tracking system
- [ ] Align team on 3-week timeline

### Team Capacity Assumptions
- **1 Senior Developer**: 32 hours/week (80% coding, 20% review)
- **Code Review**: 4 hours/week
- **Testing Time**: 25% of development time
- **Buffer**: 20% for unexpected issues

---

## Phase 1: Analysis & Setup (Week 1, Days 1-2 | 16 hours)

### Goal
Complete comprehensive analysis of all migration requirements before making any code changes.

### Tasks

#### Task 1.1: Comprehensive AppState Audit (6 hours)
**Objective**: Document all 40+ AppState fields and their dependencies

```bash
# Generate field inventory
rg "pub \w+:" crates/riptide-api/src/state.rs > docs/appstate-fields.txt

# Map usage patterns
rg "app_state\." crates/riptide-api/src/handlers/ > docs/appstate-usage.txt
rg "State\(app_state\)" crates/riptide-api/src/ >> docs/appstate-usage.txt
```

**Deliverables**:
- `docs/appstate-field-inventory.md` - All fields categorized:
  - **Core Infrastructure (CI)**: cache, event_bus, session_store, etc.
  - **Business Facades (BF)**: extraction_facade, scraper_facade, etc.
  - **Metrics (M)**: business_metrics, transport_metrics, combined_metrics
  - **Configuration (C)**: config, resource_manager, performance_manager
- Field usage count and dependency graph
- Migration priority ordering

#### Task 1.2: Circular Dependency Mapping (4 hours)
**Objective**: Identify all circular dependency risks

```bash
# Generate full dependency graph
cargo tree -p riptide-api --format "{p} -> {p}" > docs/dependency-graph.txt
cargo tree -p riptide-facade --format "{p} -> {p}" >> docs/dependency-graph.txt

# Find circular references
grep -E "riptide-api.*riptide-facade|riptide-facade.*riptide-api" docs/dependency-graph.txt
```

**Deliverables**:
- `docs/architecture/circular-dependencies.md` - All coupling points documented
- Impact analysis for breaking dependencies
- Facade-by-facade migration plan

#### Task 1.3: Feature Flag Infrastructure (4 hours)
**Objective**: Set up dual-implementation support

```toml
# crates/riptide-api/Cargo.toml
[features]
default = ["legacy-appstate"]
legacy-appstate = []
new-context = []
```

**Implementation**:
```rust
// crates/riptide-api/src/composition/mod.rs
#[cfg(feature = "legacy-appstate")]
pub struct LegacyAppStateFields {
    // Track fields being migrated
}

#[cfg(feature = "new-context")]
pub struct MigrationProgress {
    migrated_fields: Vec<String>,
    pending_fields: Vec<String>,
}
```

#### Task 1.4: Port Trait Requirements Analysis (2 hours)
**Objective**: Document all port traits needed

**New Ports Required**:
1. `CacheStorage` - Cache abstraction
2. `SessionStorage` - Session management (verify if exists)
3. `MetricsRegistry` - Metrics collection (verify if exists)
4. `HealthCheck` - Health monitoring
5. `ResourcePool` - Resource management (verify if exists)
6. `CircuitBreaker` - Circuit breaker pattern
7. `BrowserDriver` - Browser automation

**Deliverables**:
- `docs/architecture/port-traits-spec.md` - Trait definitions and contracts

---

## Phase 2: Core Infrastructure Migration (Week 1, Days 3-5 | 24 hours)

### Goal
Migrate all infrastructure fields from AppState to ApplicationContext using port traits.

### Tasks

#### Task 2.1: Create Missing Port Traits (6 hours)

**Create `CacheStorage` Port**:
```rust
// crates/riptide-types/src/ports/cache.rs
use async_trait::async_trait;
use std::time::Duration;

#[async_trait]
pub trait CacheStorage: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn set(&self, key: &str, value: Vec<u8>, ttl: Duration) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn exists(&self, key: &str) -> Result<bool>;
}
```

**Create `CircuitBreaker` Port**:
```rust
// crates/riptide-types/src/ports/circuit_breaker.rs
use async_trait::async_trait;

#[async_trait]
pub trait CircuitBreaker: Send + Sync {
    async fn call<F, T>(&self, operation: F) -> Result<T>
    where
        F: Future<Output = Result<T>> + Send;

    fn state(&self) -> CircuitState;
    fn reset(&self);
}

#[derive(Debug, Clone, Copy)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}
```

**Create `HealthCheck` Port**:
```rust
// crates/riptide-types/src/ports/health.rs
use async_trait::async_trait;

#[async_trait]
pub trait HealthCheck: Send + Sync {
    async fn check_health(&self) -> HealthStatus;
    async fn check_component(&self, component: &str) -> ComponentHealth;
}

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub status: ServiceStatus,
    pub components: Vec<ComponentHealth>,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone)]
pub enum ServiceStatus {
    Healthy,
    Degraded,
    Unhealthy,
}
```

#### Task 2.2: Create Infrastructure Adapters (8 hours)

**CacheManager Adapter**:
```rust
// crates/riptide-cache/src/adapters/cache_storage_adapter.rs
use riptide_types::ports::CacheStorage;
use crate::CacheManager;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct CacheManagerAdapter {
    inner: Arc<Mutex<CacheManager>>,
}

impl CacheManagerAdapter {
    pub fn new(cache_manager: Arc<Mutex<CacheManager>>) -> Self {
        Self { inner: cache_manager }
    }
}

#[async_trait::async_trait]
impl CacheStorage for CacheManagerAdapter {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let cache = self.inner.lock().await;
        cache.get(key).await
    }

    async fn set(&self, key: &str, value: Vec<u8>, ttl: Duration) -> Result<()> {
        let mut cache = self.inner.lock().await;
        cache.set_simple(key, &value, ttl.as_secs()).await
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let mut cache = self.inner.lock().await;
        cache.delete(key).await
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let cache = self.inner.lock().await;
        cache.exists(key).await
    }
}
```

**Metrics Registry Adapter** (if needed):
```rust
// crates/riptide-monitoring/src/adapters/metrics_adapter.rs
use riptide_types::ports::MetricsRegistry;

pub struct CombinedMetricsAdapter {
    business: Arc<BusinessMetrics>,
    transport: Arc<TransportMetrics>,
}

#[async_trait::async_trait]
impl MetricsRegistry for CombinedMetricsAdapter {
    fn increment(&self, metric: &str) {
        // Route to appropriate metrics system
    }

    fn gauge(&self, metric: &str, value: f64) {
        // Update gauge metric
    }
}
```

#### Task 2.3: Migrate Infrastructure to ApplicationContext (6 hours)

**Update ApplicationContext**:
```rust
// crates/riptide-api/src/composition/mod.rs

pub struct ApplicationContext {
    // ===== INFRASTRUCTURE PORTS =====

    // Storage & Caching
    pub cache_storage: Arc<dyn CacheStorage>,
    pub session_store: Arc<dyn SessionStorage>,
    pub idempotency_store: Arc<dyn IdempotencyStore>,

    // Messaging & Events
    pub event_bus: Arc<dyn EventBus>,

    // Metrics & Monitoring
    pub metrics_registry: Arc<dyn MetricsRegistry>,
    pub health_checker: Arc<dyn HealthCheck>,

    // Resource Management
    pub resource_pool: Arc<dyn ResourcePool>,
    pub circuit_breaker: Arc<dyn CircuitBreaker>,
    pub rate_limiter: Arc<dyn RateLimiter>,

    // Browser Automation (if needed)
    pub browser_driver: Arc<dyn BrowserDriver>,

    // HTTP Client
    pub http_client: Arc<dyn HttpClient>,

    // Configuration (owned, not a port)
    pub config: Arc<AppConfig>,

    // ===== FACADES (injected, not owned) =====
    // Note: Facades will be created on-demand via factory methods
}

impl ApplicationContext {
    pub async fn new(config: AppConfig) -> Result<Self> {
        let config = Arc::new(config);

        // Initialize all infrastructure adapters
        let cache_storage = Arc::new(CacheManagerAdapter::new(/* ... */));
        let event_bus = Arc::new(EventBusAdapter::new(/* ... */));
        // ... initialize all other ports

        Ok(Self {
            cache_storage,
            event_bus,
            config,
            // ... all other fields
        })
    }

    pub fn for_testing() -> Self {
        // Create with test doubles
        Self {
            cache_storage: Arc::new(InMemoryCacheStorage::new()),
            event_bus: Arc::new(MockEventBus::new()),
            // ... all test doubles
        }
    }
}
```

#### Task 2.4: Remove Infrastructure from AppState (4 hours)

**Reduce AppState**:
```rust
// crates/riptide-api/src/state.rs
// This file will be deleted or reduced to near-nothing

#[cfg(feature = "legacy-appstate")]
pub struct AppState {
    // Only keep absolute minimum during migration
    // Eventually this entire struct will be removed
}
```

**Files to Modify**:
- `/workspaces/riptidecrawler/crates/riptide-api/src/state.rs` - Remove infrastructure fields
- `/workspaces/riptidecrawler/crates/riptide-api/src/composition/mod.rs` - Add all port fields
- `/workspaces/riptidecrawler/crates/riptide-types/src/ports/*.rs` - Create new port traits

---

## Phase 3: State Unification & Handler Migration (Week 2, Days 1-3 | 24 hours)

### Goal
Migrate all handlers to use ApplicationContext and eliminate AppState entirely.

### Tasks

#### Task 3.1: Handler Migration Pattern (4 hours)

**Before (Old Pattern)**:
```rust
pub async fn crawl_handler(
    State(app_state): State<Arc<AppState>>,
    Json(request): Json<CrawlRequest>,
) -> Result<Json<CrawlResult>, ApiError> {
    // 8 different state field accesses
    app_state.business_metrics.record_request();
    let _permit = app_state.resource_manager.acquire_permit().await?;
    let cache_key = format!("crawl:{}", request.url);

    if let Some(cached) = app_state.cache.lock().await.get(&cache_key).await? {
        return Ok(Json(serde_json::from_slice(&cached)?));
    }

    let result = app_state.extraction_facade
        .extract(&request.url)
        .await?;

    Ok(Json(result))
}
```

**After (New Pattern)**:
```rust
pub async fn crawl_handler(
    State(context): State<Arc<ApplicationContext>>,
    Json(request): Json<CrawlRequest>,
) -> Result<Json<CrawlResult>, ApiError> {
    // Clean port-based access
    context.metrics_registry.increment("crawl_requests_total");
    let _permit = context.resource_pool.acquire().await?;
    let cache_key = format!("crawl:{}", request.url);

    if let Some(cached) = context.cache_storage.get(&cache_key).await? {
        return Ok(Json(serde_json::from_slice(&cached)?));
    }

    // Facade created on-demand via factory
    let extraction_facade = context.create_extraction_facade();
    let result = extraction_facade.extract(&request.url).await?;

    context.cache_storage
        .set(&cache_key, serde_json::to_vec(&result)?, Duration::from_secs(3600))
        .await?;

    Ok(Json(result))
}
```

#### Task 3.2: Migrate All Handlers (12 hours)

**Files to Update** (30+ handlers):
- `/workspaces/riptidecrawler/crates/riptide-api/src/handlers/*.rs` - All handler files

**Migration Checklist per Handler**:
1. Change `State(app_state): State<Arc<AppState>>` → `State(context): State<Arc<ApplicationContext>>`
2. Replace direct field access with port trait methods
3. Use facade factories instead of direct facade access
4. Update error handling for new port trait results
5. Verify no breaking changes to public API signatures

#### Task 3.3: Update Main.rs Initialization (4 hours)

**Before**:
```rust
// crates/riptide-api/src/main.rs
let app_state = AppState::new(config).await?;
let app = create_router(Arc::new(app_state));
```

**After**:
```rust
// crates/riptide-api/src/main.rs
let context = ApplicationContext::new(config).await?;
let app = create_router(Arc::new(context));
```

#### Task 3.4: Delete or Minimize AppState (4 hours)

```bash
# Verify no references remain
rg "pub struct AppState" crates/riptide-api/src/

# Should only find minimal AppState or no results
# Delete state.rs if completely unused
```

**Files to Modify/Delete**:
- Delete `/workspaces/riptidecrawler/crates/riptide-api/src/state.rs` (or reduce to <50 lines)
- Update `/workspaces/riptidecrawler/crates/riptide-api/src/lib.rs` - Remove AppState exports
- Update `/workspaces/riptidecrawler/crates/riptide-api/src/main.rs` - Use ApplicationContext

---

## Phase 4: Circular Dependency Resolution (Week 2, Days 4-5 + Week 3, Days 1-2 | 32 hours)

### Goal
Eliminate circular dependencies between riptide-api ↔ riptide-facade. Establish clean unidirectional dependency flow.

### Tasks

#### Task 4.1: Facade Dependency Refactoring (16 hours)

**Problem Statement**:
- `riptide-api` depends on `riptide-facade` (uses facades)
- `riptide-facade` wants to depend on `riptide-api` (needs infrastructure)
- This creates a circular dependency ❌

**Solution**: Make facades depend ONLY on port traits from `riptide-types`

**Before (Circular)**:
```rust
// riptide-facade/src/facades/extraction.rs
use riptide_api::AppState;  // ❌ Creates circular dependency

pub struct ExtractionFacade {
    app_state: Arc<AppState>,  // ❌ Couples to API layer
}
```

**After (Clean)**:
```rust
// riptide-facade/src/facades/extraction.rs
use riptide_types::ports::{CacheStorage, EventBus, BrowserDriver};

pub struct ExtractionFacade {
    cache: Arc<dyn CacheStorage>,       // ✅ Port trait
    events: Arc<dyn EventBus>,          // ✅ Port trait
    browser: Arc<dyn BrowserDriver>,    // ✅ Port trait
    config: ExtractionConfig,           // ✅ Pure config
}

impl ExtractionFacade {
    pub fn new(
        cache: Arc<dyn CacheStorage>,
        events: Arc<dyn EventBus>,
        browser: Arc<dyn BrowserDriver>,
        config: ExtractionConfig,
    ) -> Self {
        Self { cache, events, browser, config }
    }
}
```

**Files to Refactor** (35+ facades):
- `/workspaces/riptidecrawler/crates/riptide-facade/src/facades/extraction.rs`
- `/workspaces/riptidecrawler/crates/riptide-facade/src/facades/scraper.rs`
- `/workspaces/riptidecrawler/crates/riptide-facade/src/facades/spider.rs`
- `/workspaces/riptidecrawler/crates/riptide-facade/src/facades/search.rs`
- All other facade files

#### Task 4.2: Create Facade Factory in ApplicationContext (8 hours)

```rust
// crates/riptide-api/src/composition/facade_factory.rs

use riptide_facade::facades::*;
use super::ApplicationContext;

impl ApplicationContext {
    /// Create ExtractionFacade with all required dependencies injected
    pub fn create_extraction_facade(&self) -> Arc<ExtractionFacade> {
        Arc::new(ExtractionFacade::new(
            self.cache_storage.clone(),
            self.event_bus.clone(),
            self.browser_driver.clone(),
            ExtractionConfig::from(&self.config),
        ))
    }

    /// Create ScraperFacade with dependencies
    pub fn create_scraper_facade(&self) -> Arc<ScraperFacade> {
        Arc::new(ScraperFacade::new(
            self.http_client.clone(),
            self.cache_storage.clone(),
            ScraperConfig::from(&self.config),
        ))
    }

    /// Create SpiderFacade with dependencies
    pub fn create_spider_facade(&self) -> Arc<SpiderFacade> {
        Arc::new(SpiderFacade::new(
            self.http_client.clone(),
            self.cache_storage.clone(),
            self.event_bus.clone(),
            SpiderConfig::from(&self.config),
        ))
    }

    // Add factory methods for all 35+ facades
}
```

**New File**:
- `/workspaces/riptidecrawler/crates/riptide-api/src/composition/facade_factory.rs`

#### Task 4.3: Verify Dependency Graph is Acyclic (4 hours)

```bash
# Check for circular dependencies
cargo tree -p riptide-api --duplicate
cargo tree -p riptide-facade --duplicate

# Verify clean dependency flow
cargo tree -p riptide-facade | grep riptide-api
# Should return NOTHING (no riptide-api dependency)

# Verify facades only depend on types
cargo tree -p riptide-facade | grep riptide-types
# Should show riptide-types as only riptide-* dependency
```

**Expected Dependency Flow**:
```
┌─────────────────┐
│  riptide-types  │  ← Domain layer (ports, domain types)
│    (Ports)      │
└────────┬────────┘
         │ implements
         ▼
┌─────────────────┐
│ riptide-facade  │  ← Application layer (use-cases)
│   (Facades)     │  ✅ Depends ONLY on ports
└────────┬────────┘
         │ uses
         ▼
┌─────────────────┐
│  riptide-api    │  ← Infrastructure layer (DI, HTTP)
│ (AppContext +   │  ✅ Wires facades with adapters
│  Handlers)      │
└─────────────────┘
         ▲
         │ implements
┌─────────────────┐
│ Infrastructure  │  ← Adapters (Redis, Postgres, etc.)
│   Adapters      │
└─────────────────┘
```

#### Task 4.4: Update Documentation (4 hours)

**Documents to Create/Update**:
- `docs/architecture/hexagonal-architecture.md` - Clean layering diagram
- `docs/architecture/facade-factory-pattern.md` - Factory pattern usage
- `docs/migration-guide.md` - How to consume facades post-migration

---

## Phase 5: Comprehensive Testing & Validation (Week 3, Days 3-5 | 24 hours)

### Goal
Ensure 100% test coverage and validate all migration objectives met.

### Tasks

#### Task 5.1: Migration Tests (8 hours)

```rust
// crates/riptide-api/src/tests/appstate_migration_tests.rs

#[tokio::test]
async fn test_cache_access_via_context() {
    let ctx = ApplicationContext::for_testing();

    ctx.cache_storage.set("test", b"value".to_vec(), Duration::from_secs(60)).await?;
    let value = ctx.cache_storage.get("test").await?;

    assert_eq!(value, Some(b"value".to_vec()));
}

#[tokio::test]
async fn test_event_bus_integration() {
    let ctx = ApplicationContext::for_testing();

    let event = BaseEvent::new("test.event", "test", EventSeverity::Info);
    ctx.event_bus.emit(event).await?;

    // Verify event was published
}

#[tokio::test]
async fn test_metrics_collection() {
    let ctx = ApplicationContext::for_testing();

    ctx.metrics_registry.increment("test_metric");
    let value = ctx.metrics_registry.get("test_metric");

    assert_eq!(value, Some(1.0));
}

#[tokio::test]
async fn test_resource_pool_acquisition() {
    let ctx = ApplicationContext::for_testing();

    let permit = ctx.resource_pool.acquire().await?;
    assert!(permit.is_valid());
}
```

**Expected Test Count**: 15+ migration tests

#### Task 5.2: Dependency Isolation Tests (6 hours)

```rust
// crates/riptide-facade/tests/dependency_isolation_tests.rs

#[test]
fn test_facade_has_no_api_dependency() {
    // This test ensures riptide-facade does NOT depend on riptide-api
    // If this test compiles, we're good (compile-time verification)

    use riptide_facade::facades::ExtractionFacade;
    use riptide_types::ports::*;

    // Should compile without riptide_api in scope
    let facade = ExtractionFacade::new(
        test_doubles::MockCacheStorage::new(),
        test_doubles::MockEventBus::new(),
        test_doubles::MockBrowserDriver::new(),
        ExtractionConfig::default(),
    );

    assert!(facade.is_initialized());
}

#[test]
fn test_no_circular_dependency_at_compile_time() {
    // If this file compiles, circular dependencies are broken
    // because we're importing facade without importing api
    use riptide_facade::facades::*;

    // This would fail to compile if circular deps exist
    let _ = std::marker::PhantomData::<ExtractionFacade>;
}
```

#### Task 5.3: Facade Factory Integration Tests (4 hours)

```rust
// crates/riptide-api/tests/facade_factory_integration_tests.rs

#[tokio::test]
async fn test_facade_creation_via_context() {
    let context = ApplicationContext::for_testing();

    let extraction = context.create_extraction_facade();
    let scraper = context.create_scraper_facade();

    // Verify facades work independently
    let result = extraction.extract("https://example.com").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_all_facades_creatable() {
    let context = ApplicationContext::for_testing();

    // Test all 35+ facade factory methods
    let _extraction = context.create_extraction_facade();
    let _scraper = context.create_scraper_facade();
    let _spider = context.create_spider_facade();
    // ... test all facades
}
```

#### Task 5.4: Handler Integration Tests (4 hours)

```rust
// crates/riptide-api/tests/handler_integration_tests.rs

#[tokio::test]
async fn test_handler_pipeline_with_context() {
    let context = ApplicationContext::for_testing();

    // Simulate request through full stack
    let response = crawl_handler(State(Arc::new(context))).await?;

    assert!(response.status().is_success());
}

#[tokio::test]
async fn test_all_handlers_compile_with_context() {
    let context = Arc::new(ApplicationContext::for_testing());

    // Verify all handlers accept ApplicationContext
    let _crawl = crawl_handler(State(context.clone()));
    let _scrape = scrape_handler(State(context.clone()));
    // ... test all 30+ handlers
}
```

#### Task 5.5: Regression Testing (2 hours)

```bash
# Test entire workspace
cargo test --workspace

# Test with feature flags
cargo test -p riptide-api --features legacy-appstate
cargo test -p riptide-api --features new-context

# Performance regression tests
cargo bench --workspace
```

**Expected Results**:
- All tests pass ✅
- <5% latency increase (acceptable for cleaner architecture)
- No breaking changes to public APIs

---

## Quality Gates

### Phase 1: Analysis & Setup
**Gates**:
- [ ] All 40+ AppState fields documented and categorized
- [ ] Circular dependencies mapped and documented
- [ ] Feature flag infrastructure compiles
- [ ] All port trait specifications documented
- [ ] Migration plan approved by team

**Verification**:
```bash
# Documentation exists
ls -la docs/appstate-field-inventory.md
ls -la docs/architecture/circular-dependencies.md
ls -la docs/architecture/port-traits-spec.md

# Feature flags compile
cargo check --features legacy-appstate
cargo check --features new-context
```

### Phase 2: Core Infrastructure Migration
**Gates**:
- [ ] All port traits created and compile
- [ ] All infrastructure adapters created
- [ ] ApplicationContext has all infrastructure ports
- [ ] No compilation errors
- [ ] Unit tests for all adapters pass

**Verification**:
```bash
# Compile check
cargo check -p riptide-types
cargo check -p riptide-api

# Test adapters
cargo test -p riptide-cache
cargo test -p riptide-monitoring

# Verify port traits exist
rg "pub trait CacheStorage" crates/riptide-types/src/ports/
rg "pub trait CircuitBreaker" crates/riptide-types/src/ports/
```

### Phase 3: State Unification & Handler Migration
**Gates**:
- [ ] All 30+ handlers migrated to ApplicationContext
- [ ] AppState reduced to <200 lines or deleted
- [ ] All handler tests pass
- [ ] No breaking changes to public APIs
- [ ] Zero clippy warnings

**Verification**:
```bash
# Handler tests
cargo test -p riptide-api --test handler_tests

# Clippy clean
cargo clippy -p riptide-api -- -D warnings

# Verify AppState minimized
wc -l crates/riptide-api/src/state.rs  # Should be <200 or not exist

# Verify no public API breaks
cargo build --release -p riptide-api
```

### Phase 4: Circular Dependency Resolution
**Gates**:
- [ ] Zero circular dependencies (cargo tree verification)
- [ ] All facades depend ONLY on riptide-types
- [ ] Facade factories created for all 35+ facades
- [ ] Dependency isolation tests pass
- [ ] Documentation updated

**Verification**:
```bash
# Verify no circular dependencies
cargo tree -p riptide-api --duplicate | grep -i duplicate  # Should be empty
cargo tree -p riptide-facade --duplicate | grep -i duplicate  # Should be empty

# Verify facades don't import riptide-api
rg "use riptide_api" crates/riptide-facade/src/  # Should return NOTHING

# Verify facade tests compile independently
cargo test -p riptide-facade  # Should pass without riptide-api
```

### Phase 5: Comprehensive Testing & Validation
**Gates**:
- [ ] 60+ new tests added (15 migration + 20 isolation + 25 integration)
- [ ] 100% test coverage for migrated components
- [ ] All workspace tests pass
- [ ] Performance regression <5%
- [ ] Zero clippy warnings workspace-wide

**Verification**:
```bash
# Full workspace test
cargo test --workspace

# Clippy clean workspace-wide
cargo clippy --workspace -- -D warnings

# Test count verification
cargo test --workspace -- --list | wc -l  # Should increase by 60+

# Performance check
cargo bench --workspace
```

### One-Shot Migration Quality Gate

- [ ] Bulk search/replace AppState → ApplicationContext complete
- [ ] crates/riptide-api/src/state.rs deleted or aliased and removed
- [ ] All handlers compile using ApplicationContext
- [ ] All facades compile and run using ApplicationContext factories
- [ ] ApplicationContext::validate() implemented and passes
- [ ] Circular deps clean: `cargo tree --workspace --duplicates`
- [ ] Empty composition modules removed
- [ ] `./scripts/quality_gate.sh` fully ran and passed
- [ ] `grep -R \bAppState\b crates/` returns 0
- [ ] Docs/ADR updated: AppState elimination

**Final Verification Commands**:
```bash
# Complete quality check
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo build --release

# Dependency verification
cargo tree --workspace --duplicates  # Must be empty
cargo tree -p riptide-facade | grep riptide-api  # Must be empty

# AppState elimination check
grep -R \bAppState\b crates/  # Must return 0 matches

# Test coverage check
cargo tarpaulin --workspace --out Html

# Performance benchmark
cargo bench --workspace > benchmark-results.txt

# Quality gate script
./scripts/quality_gate.sh
```

**If ANY gate fails**: STOP, fix the issue, and re-verify before proceeding.

---

## Acceptance Criteria

### Sprint 1 Original Criteria (Preserved)
- [x] ✅ 8 core infrastructure fields migrated from AppState to ApplicationContext
- [x] ✅ 2 new port traits created (CacheStorage, SessionStorage)
- [x] ✅ 100% test coverage for migrated fields
- [x] ✅ All tests pass with both feature flags
- [x] ✅ Zero breaking changes to public API
- [x] ✅ Documentation updated

### Sprint 2 Original Criteria (Preserved)
- [x] ✅ Single state system (ApplicationContext only)
- [x] ✅ All infrastructure accessed via port traits
- [x] ✅ All handlers migrated to ApplicationContext
- [x] ✅ AppState removed or reduced to <50 lines
- [x] ✅ Zero test failures
- [x] ✅ Documentation complete

### Sprint 3 Original Criteria (Preserved)
- [x] ✅ Zero circular dependencies (cargo tree shows clean graph)
- [x] ✅ All facades depend only on riptide-types (ports)
- [x] ✅ Facade creation centralized in ApplicationContext factories
- [x] ✅ All tests pass
- [x] ✅ Dependency isolation verified at compile-time

### One-Shot Migration Criteria (Consolidated)
- [ ] **Code Reduction**: AppState reduced from 2213 lines → <200 lines (90% reduction)
- [ ] **Port Traits**: 6+ new port traits created (CacheStorage, CircuitBreaker, HealthCheck, etc.)
- [ ] **Handler Migration**: 30+ handlers migrated to ApplicationContext
- [ ] **Facade Migration**: 35+ facades refactored to use only port traits
- [ ] **Tests**: 60+ new tests added (migration, isolation, integration)
- [ ] **Coverage**: 100% test coverage for all migrated components
- [ ] **Dependencies**: Zero circular dependencies verified via cargo tree
- [ ] **Performance**: <5% latency increase (acceptable for cleaner architecture)
- [ ] **Quality**: Zero clippy warnings workspace-wide
- [ ] **Documentation**: All architecture docs updated

---

## Rollback Plan

### All-or-Nothing Rollback Strategy

**Advantages of One-Shot Rollback**:
- Single feature flag controls entire migration
- No partial states to manage
- Simpler decision: "Keep new" or "Revert all"

### If Critical Issues Arise

**Step 1: Immediate Rollback**
```bash
# Revert to legacy state
cargo build --features legacy-appstate
git revert <one-shot-migration-commit-range>

# Or use backup tag
git reset --hard pre-appstate-migration
```

**Step 2: Verify Rollback**
```bash
# Verify all tests pass with legacy state
cargo test --workspace --features legacy-appstate

# Verify production systems work
cargo run --release --features legacy-appstate
```

**Step 3: Root Cause Analysis**
- Document what failed
- Determine if issue is fixable quickly (<4 hours)
- If fixable: Apply fix and re-test
- If not: Stay on legacy, plan fix for next iteration

### Feature Flag Removal Timeline

**Week 4 (Post-Migration)**:
- Monitor production with new-context for 1 week
- Keep legacy-appstate flag as fallback
- No code removal yet

**Week 5+**:
- If no issues: Remove legacy-appstate flag
- Delete old AppState code
- Remove feature flag infrastructure

---

## Success Metrics

### Code Quality Metrics
| Metric | Before | After | Change |
|--------|--------|-------|--------|
| AppState LOC | 2213 | <200 | -90% |
| Circular Dependencies | 3+ | 0 | -100% |
| Facade Dependencies | 15 | 3 | -80% |
| Port Traits | 10 | 16+ | +60% |
| Test Coverage | 70% | 100% | +30% |
| Cyclomatic Complexity | High | -40% | -40% |

### Performance Metrics
| Metric | Target | Acceptable Range |
|--------|--------|------------------|
| Handler Latency | <5% increase | 0-10% increase |
| Compile Time | -10% | -5% to -15% |
| Test Run Time | <10% increase | 0-15% increase |

### Timeline Metrics
| Metric | Original Plan | One-Shot Plan | Improvement |
|--------|---------------|---------------|-------------|
| Total Duration | 3+ weeks | 3 weeks | Faster delivery |
| Context Switches | 3 sprints | 1 migration | -66% |
| Feature Flags | 3+ flags | 1 flag | -66% complexity |
| Integration Points | 9 (3x3) | 1 | -88% complexity |

---

## Dependencies & Blockers

### Phase Dependencies
- **Phase 1** → No dependencies (foundational work)
- **Phase 2** → Depends on Phase 1 (analysis must be complete)
- **Phase 3** → Depends on Phase 2 (infrastructure must be migrated)
- **Phase 4** → Depends on Phase 3 (handlers must use ApplicationContext)
- **Phase 5** → Depends on Phase 4 (all code changes complete)

### External Dependencies
- [ ] Team availability (3 weeks dedicated time)
- [ ] Code review capacity (4 hours/week)
- [ ] CI/CD infrastructure (for automated tests)
- [ ] Production monitoring (for performance validation)

### Potential Blockers
1. **Insufficient Test Coverage**: If existing tests <50%, budget extra time for test creation
2. **Unknown Dependencies**: Hidden AppState usage might emerge during migration
3. **Performance Issues**: If >5% regression, may need optimization iteration
4. **Team Capacity**: If interrupted, buffer may be insufficient

**Mitigation**: 20% buffer built into timeline, daily standups to catch issues early

---

## Post-Migration Checklist

### Week 4: Monitoring & Stabilization
- [ ] Monitor production metrics for 7 days
- [ ] Verify no performance regressions
- [ ] Address any edge cases discovered
- [ ] Update runbooks and operational docs

### Week 5: Cleanup & Documentation
- [ ] Remove feature flags (if stable)
- [ ] Delete legacy AppState code
- [ ] Final documentation review
- [ ] Knowledge transfer to team

### Week 6: Retrospective
- [ ] Team retrospective on one-shot approach
- [ ] Document lessons learned
- [ ] Identify improvements for future migrations
- [ ] Celebrate successful migration! 🎉

---

## Remaining Sprints (4-12)

**Note**: Sprints 4-12 from the original plan are preserved as-is and will proceed after this one-shot migration completes. These include:

- **Sprint 4**: Empty Composition Modules (Week 3 end)
- **Sprint 5**: Create Missing Port Traits (Week 4)
- **Sprint 6**: Migrate Facades to Use Port Traits (Week 5)
- **Sprint 7**: Facade Testing & Coverage (Week 6)
- **Sprint 8**: Infrastructure Cleanup (Week 7)
- **Sprint 9**: Handler Refactoring (Week 8)
- **Sprint 10**: Integration Testing (Week 9)
- **Sprint 11**: Performance Optimization (Week 10)
- **Sprint 12**: Final Validation & Documentation (Week 11-12)

For details on these remaining sprints, refer to the original 12-week sprint plan.

---

**Document Version**: 1.0
**Last Updated**: 2025-11-11
**Migration Status**: Not Started
**Timeline**: 3 weeks from start date
**Next Review**: After Phase 1 completion
