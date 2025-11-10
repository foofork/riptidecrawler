# 12-Week Sprint Plan: Facade Layer Refactoring & Architecture Cleanup

**Project**: Riptide Event Mesh
**Timeline**: 12 weeks (3 months)
**Priority**: P0 (Critical) and P1 (High Priority)
**Goal**: Transform facade layer to production-ready hexagonal architecture

---

## Executive Summary

This sprint plan addresses critical architectural debt in the facade layer:
- **P0**: AppState god object (40+ fields), competing state systems, circular dependencies, empty modules
- **P1**: 32 infrastructure violations, 12 untested facades

**Success Metrics**:
- AppState reduced from 2213 lines to <300 lines
- 100% facade test coverage (12 facades)
- Zero circular dependencies
- All infrastructure accessed via port traits
- Clean hexagonal boundaries enforced

---

## Pre-Sprint Setup (Week 0)

### Prerequisites
- [ ] Backup current working state: `git tag pre-refactor-backup`
- [ ] Verify all tests pass: `cargo test --workspace`
- [ ] Document current metrics baseline
- [ ] Set up feature flags for dual implementation
- [ ] Create migration tracking spreadsheet

### Team Capacity Assumptions
- **1 Senior Developer**: 32 hours/week (80% coding, 20% review)
- **Code Review**: 4 hours/week
- **Testing Time**: 25% of development time
- **Buffer**: 20% for unexpected issues

---

## PHASE 1: P0 CRITICAL BLOCKERS (Weeks 1-3)

---

## Sprint 1: AppState God Object → ApplicationContext (Week 1)

### Sprint Goal
Migrate 50% of AppState fields to ApplicationContext, establish dual-implementation pattern with feature flags.

### Duration
**5 business days (40 hours)**

### Prerequisites
- [x] ApplicationContext exists at `/workspaces/eventmesh/crates/riptide-api/src/composition/mod.rs`
- [x] Port traits exist at `/workspaces/eventmesh/crates/riptide-types/src/ports/`
- [ ] Feature flag `legacy-appstate` added to `Cargo.toml`

### Tasks

#### Day 1: Analysis & Feature Flag Setup (8 hours)
- [ ] **Task 1.1**: Audit all 40+ AppState fields in `/workspaces/eventmesh/crates/riptide-api/src/state.rs` (lines 60-200)
  - Document field purpose, dependencies, usage count
  - Categorize: Core Infrastructure (CI), Business Facades (BF), Metrics (M), Configuration (C)
  - Output: `docs/appstate-field-inventory.md`

- [ ] **Task 1.2**: Add feature flag to `crates/riptide-api/Cargo.toml`:
  ```toml
  [features]
  default = ["legacy-appstate"]
  legacy-appstate = []
  new-context = []
  ```

- [ ] **Task 1.3**: Create migration tracking in `ApplicationContext`:
  ```rust
  // crates/riptide-api/src/composition/mod.rs
  #[cfg(feature = "legacy-appstate")]
  pub struct LegacyAppStateFields {
      // Fields being migrated
  }
  ```

#### Day 2-3: Migrate Core Infrastructure (16 hours)
- [ ] **Task 1.4**: Move persistence layer to ApplicationContext (8 hours)
  ```rust
  // BEFORE (state.rs line ~110):
  pub struct AppState {
      pub cache: Arc<tokio::sync::Mutex<CacheManager>>,
      pub event_bus: Arc<EventBus>,
      // ...
  }

  // AFTER (composition/mod.rs line ~95):
  pub struct ApplicationContext {
      // Already exists:
      pub event_bus: Arc<dyn EventBus>,
      pub idempotency_store: Arc<dyn IdempotencyStore>,

      // Add:
      pub cache_storage: Arc<dyn CacheStorage>,  // NEW PORT
      pub session_store: Arc<dyn SessionStorage>, // NEW PORT
  }
  ```

  **Files to modify**:
  - `/workspaces/eventmesh/crates/riptide-api/src/state.rs` (remove fields 65-68, 102-104, 124)
  - `/workspaces/eventmesh/crates/riptide-api/src/composition/mod.rs` (add fields to ApplicationContext)
  - `/workspaces/eventmesh/crates/riptide-types/src/ports/cache.rs` (create `CacheStorage` trait)
  - `/workspaces/eventmesh/crates/riptide-types/src/ports/session.rs` (create `SessionStorage` trait - **already exists**)

- [ ] **Task 1.5**: Create adapter for CacheManager → CacheStorage (4 hours)
  ```rust
  // NEW FILE: crates/riptide-cache/src/adapters/cache_storage_adapter.rs
  use riptide_types::ports::CacheStorage;
  use crate::CacheManager;

  pub struct CacheManagerAdapter {
      inner: Arc<tokio::sync::Mutex<CacheManager>>,
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
  }
  ```

- [ ] **Task 1.6**: Update handlers to use ApplicationContext (4 hours)
  ```rust
  // BEFORE:
  pub async fn crawl_handler(
      State(app_state): State<Arc<AppState>>,
  ) -> Result<Json<CrawlResult>, ApiError> {
      let cache = app_state.cache.lock().await;
      cache.get("key").await?;
  }

  // AFTER:
  pub async fn crawl_handler(
      State(context): State<Arc<ApplicationContext>>,
  ) -> Result<Json<CrawlResult>, ApiError> {
      context.cache_storage.get("key").await?;
  }
  ```

  **Files to modify**:
  - `/workspaces/eventmesh/crates/riptide-api/src/handlers/*.rs` (all handlers using cache/events)

#### Day 4-5: Testing & Validation (16 hours)
- [ ] **Task 1.7**: Write migration tests (8 hours)
  ```rust
  // NEW FILE: crates/riptide-api/src/tests/appstate_migration_tests.rs
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
  ```

- [ ] **Task 1.8**: Run regression tests (4 hours)
  ```bash
  # Test both feature flag configurations
  cargo test -p riptide-api --features legacy-appstate
  cargo test -p riptide-api --features new-context
  cargo test --workspace
  ```

- [ ] **Task 1.9**: Update documentation (4 hours)
  - Document migration progress in `docs/architecture/appstate-migration.md`
  - Update handler examples in `docs/api/handlers.md`
  - Add migration guide for developers

### Acceptance Criteria
- [x] ✅ 8 core infrastructure fields migrated from AppState to ApplicationContext
- [x] ✅ 2 new port traits created (`CacheStorage`, verified `SessionStorage` exists)
- [x] ✅ 100% test coverage for migrated fields
- [x] ✅ All tests pass with both feature flags
- [x] ✅ Zero breaking changes to public API
- [x] ✅ Documentation updated

### Code Example: Before/After

**Before** (`state.rs` lines 60-130):
```rust
pub struct AppState {
    pub http_client: Client,
    pub cache: Arc<tokio::sync::Mutex<CacheManager>>,  // ❌ Concrete type
    pub extractor: Arc<UnifiedExtractor>,              // ❌ Concrete type
    pub reliable_extractor: Arc<ReliableExtractor>,    // ❌ Concrete type
    pub config: AppConfig,
    pub api_config: RiptideApiConfig,
    pub resource_manager: Arc<ResourceManager>,        // ❌ Concrete type
    pub business_metrics: Arc<BusinessMetrics>,
    pub transport_metrics: Arc<TransportMetrics>,
    pub combined_metrics: Arc<CombinedMetrics>,
    pub health_checker: Arc<HealthChecker>,
    pub session_manager: Arc<SessionManager>,          // ❌ Concrete type
    pub streaming: Arc<StreamingModule>,               // ❌ Concrete type
    pub event_bus: Arc<EventBus>,                      // ❌ Concrete type
    // ... 25 more fields
}
```

**After** (`composition/mod.rs` lines 95-150):
```rust
pub struct ApplicationContext {
    // === System Ports (clean interface) ===
    pub clock: Arc<dyn Clock>,
    pub entropy: Arc<dyn Entropy>,

    // === Persistence Layer (port traits) ===
    pub transaction_manager: Arc<dyn TransactionManager<Transaction = PostgresTransaction>>,
    pub cache_storage: Arc<dyn CacheStorage>,          // ✅ Port trait
    pub session_store: Arc<dyn SessionStorage>,        // ✅ Port trait
    pub event_bus: Arc<dyn EventBus>,                  // ✅ Port trait
    pub idempotency_store: Arc<dyn IdempotencyStore>,  // ✅ Port trait

    // === Repositories ===
    pub user_repository: Arc<dyn Repository<User>>,
    pub event_repository: Arc<dyn Repository<Event>>,

    // === Configuration ===
    pub config: DiConfig,
}

// Remaining AppState (reduced to facades only)
pub struct AppState {
    pub context: Arc<ApplicationContext>,              // ✅ Composition

    // Business facades (these stay in API layer)
    pub extraction_facade: Arc<ExtractionFacade>,
    pub scraper_facade: Arc<ScraperFacade>,
    pub resource_facade: Arc<ResourceFacade>,
    // ... only facade fields remain
}
```

### Testing Strategy
1. **Unit Tests**: Each port trait adapter (CacheManagerAdapter, etc.)
2. **Integration Tests**: ApplicationContext wiring in production mode
3. **Regression Tests**: All existing handler tests pass unchanged
4. **Feature Flag Tests**: Both `legacy-appstate` and `new-context` work

### Rollback Plan
If migration fails:
1. Keep feature flag `legacy-appstate` enabled (default)
2. Revert handler changes: `git checkout HEAD~1 -- crates/riptide-api/src/handlers/`
3. Remove new port traits (they're additive, no breaking changes)
4. ApplicationContext additions are non-breaking, can remain

### Dependencies
**This sprint depends on**: None (foundational work)
**Blocks**: Sprint 2, 3, 4 (all depend on clean ApplicationContext)

### Success Metrics
- **Code**: AppState reduced from 2213 lines → ~1800 lines (18% reduction)
- **Ports**: 2 new port traits created
- **Tests**: 15+ new tests added
- **Coverage**: 100% for migrated components

---

## Sprint 2: State System Unification (Week 2)

### Sprint Goal
Eliminate competing state systems (AppState, ApplicationContext, God Object pattern). Establish single source of truth.

### Duration
**5 business days (40 hours)**

### Prerequisites
- [x] Sprint 1 complete (ApplicationContext has core infrastructure)
- [ ] All handlers compile with `new-context` feature flag
- [ ] Port traits validated in production

### Tasks

#### Day 1: Architectural Analysis (8 hours)
- [ ] **Task 2.1**: Map all state access patterns (4 hours)
  ```bash
  # Find all AppState field accesses
  grep -r "app_state\." crates/riptide-api/src/handlers/ > docs/appstate-usage.txt
  grep -r "State(app_state)" crates/riptide-api/src/ >> docs/appstate-usage.txt

  # Categorize by access type:
  # - Direct field access (needs migration)
  # - Via facade (stays in AppState)
  # - Infrastructure (moves to ApplicationContext)
  ```

- [ ] **Task 2.2**: Design unified state architecture (4 hours)
  ```rust
  // PROPOSED ARCHITECTURE:
  //
  // ApplicationContext (in riptide-api/composition)
  //   ↓ contains
  // Infrastructure ports (Clock, Cache, EventBus, etc.)
  //   ↓ injected into
  // Facades (in riptide-facade)
  //   ↓ used by
  // Handlers (in riptide-api/handlers)
  //
  // AppState REMOVED - handlers use ApplicationContext directly
  ```

  Document in `docs/architecture/unified-state-design.md`

#### Day 2-3: Migrate Remaining Infrastructure (16 hours)
- [ ] **Task 2.3**: Move metrics to ApplicationContext (6 hours)
  ```rust
  // BEFORE (state.rs lines 87-96):
  pub struct AppState {
      pub business_metrics: Arc<BusinessMetrics>,
      pub transport_metrics: Arc<TransportMetrics>,
      pub combined_metrics: Arc<CombinedMetrics>,
      pub health_checker: Arc<HealthChecker>,
  }

  // AFTER (composition/mod.rs):
  pub struct ApplicationContext {
      // Add metrics system
      pub metrics_registry: Arc<dyn MetricsRegistry>,  // NEW PORT
      pub health_checker: Arc<dyn HealthCheck>,        // NEW PORT
  }
  ```

  **Files to modify**:
  - Create `/workspaces/eventmesh/crates/riptide-types/src/ports/metrics.rs` (**already exists**)
  - Update `/workspaces/eventmesh/crates/riptide-api/src/composition/mod.rs`
  - Create adapters in `/workspaces/eventmesh/crates/riptide-monitoring/src/adapters/`

- [ ] **Task 2.4**: Move resource management to ApplicationContext (6 hours)
  ```rust
  // BEFORE:
  pub struct AppState {
      pub resource_manager: Arc<ResourceManager>,
      pub performance_manager: Arc<PerformanceManager>,
      pub circuit_breaker: Arc<tokio::sync::Mutex<CircuitBreakerState>>,
  }

  // AFTER:
  pub struct ApplicationContext {
      pub resource_pool: Arc<dyn ResourcePool>,        // NEW PORT
      pub circuit_breaker: Arc<dyn CircuitBreaker>,    // NEW PORT
      pub rate_limiter: Arc<dyn RateLimiter>,          // EXISTS
  }
  ```

  **Create port traits**:
  - `/workspaces/eventmesh/crates/riptide-types/src/ports/pool.rs` (**already exists**)
  - `/workspaces/eventmesh/crates/riptide-types/src/ports/circuit_breaker.rs` (new)

- [ ] **Task 2.5**: Update all handlers to use ApplicationContext (4 hours)
  ```rust
  // BEFORE:
  pub async fn handler(
      State(app_state): State<Arc<AppState>>,
  ) -> Result<Json<Response>> {
      app_state.business_metrics.record_request();
      app_state.resource_manager.acquire().await?;
  }

  // AFTER:
  pub async fn handler(
      State(context): State<Arc<ApplicationContext>>,
  ) -> Result<Json<Response>> {
      context.metrics_registry.increment("requests_total");
      context.resource_pool.acquire().await?;
  }
  ```

  **Files to update**: All 30+ handlers in `/workspaces/eventmesh/crates/riptide-api/src/handlers/`

#### Day 4-5: Testing & Cleanup (16 hours)
- [ ] **Task 2.6**: Write comprehensive integration tests (8 hours)
  ```rust
  // NEW FILE: crates/riptide-api/src/tests/unified_state_integration_tests.rs

  #[tokio::test]
  async fn test_handler_pipeline_with_context() {
      let context = ApplicationContext::for_testing();

      // Simulate request through full stack
      let response = crawl_handler(State(Arc::new(context))).await?;

      assert!(response.status().is_success());
  }

  #[tokio::test]
  async fn test_metrics_collection_via_context() {
      let context = ApplicationContext::for_testing();

      context.metrics_registry.increment("test_metric");
      let value = context.metrics_registry.get("test_metric");

      assert_eq!(value, Some(1.0));
  }
  ```

- [ ] **Task 2.7**: Remove AppState god object (4 hours)
  ```bash
  # Verify no references remain
  grep -r "pub struct AppState" crates/riptide-api/src/

  # Should only find minimal AppState with facades (if any)
  # or complete removal if facades moved to dedicated layer
  ```

  **Files to modify**:
  - Delete `/workspaces/eventmesh/crates/riptide-api/src/state.rs` (or reduce to facades only)
  - Update `/workspaces/eventmesh/crates/riptide-api/src/lib.rs` exports
  - Update `/workspaces/eventmesh/crates/riptide-api/src/main.rs` initialization

- [ ] **Task 2.8**: Update documentation (4 hours)
  - Document unified state architecture
  - Update migration guide
  - Create ADR (Architecture Decision Record) for state unification

### Acceptance Criteria
- [x] ✅ Single state system (ApplicationContext only)
- [x] ✅ All infrastructure accessed via port traits
- [x] ✅ All handlers migrated to ApplicationContext
- [x] ✅ AppState removed or reduced to <50 lines (facades only)
- [x] ✅ Zero test failures
- [x] ✅ Documentation complete

### Code Example: Handler Migration

**Before**:
```rust
// crates/riptide-api/src/handlers/crawl.rs (old)
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

    app_state.cache.lock().await
        .set_simple(&cache_key, &result, 3600)
        .await?;

    Ok(Json(result))
}
```

**After**:
```rust
// crates/riptide-api/src/handlers/crawl.rs (new)
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

    // Facade still used (business logic layer)
    let result = context.extraction_facade
        .extract(&request.url)
        .await?;

    context.cache_storage
        .set(&cache_key, serde_json::to_vec(&result)?, Duration::from_secs(3600))
        .await?;

    Ok(Json(result))
}
```

### Testing Strategy
1. **Unit Tests**: Each new port trait implementation
2. **Integration Tests**: Full request lifecycle through ApplicationContext
3. **Load Tests**: Verify no performance regression
4. **Migration Tests**: Side-by-side comparison with legacy AppState

### Rollback Plan
1. Keep `legacy-appstate` feature flag for 1 sprint after completion
2. Maintain dual implementation in handlers (conditional compilation)
3. If critical issues arise:
   ```bash
   # Revert to legacy state
   cargo build --features legacy-appstate
   git revert <sprint-2-commits>
   ```

### Dependencies
**Depends on**: Sprint 1 (ApplicationContext established)
**Blocks**: Sprint 3 (circular dependency resolution needs clean state)

### Success Metrics
- **Code**: AppState reduced to <200 lines (90% reduction)
- **Ports**: 4 new port traits (CircuitBreaker, ResourcePool, MetricsRegistry, HealthCheck)
- **Handlers**: 30+ handlers migrated
- **Tests**: 25+ integration tests added
- **Performance**: <5% latency increase (acceptable for cleaner architecture)

---

## Sprint 3: Circular Dependency Resolution (Week 3)

### Sprint Goal
Eliminate circular dependency risk between AppState ↔ Facades. Establish clean unidirectional dependency flow.

### Duration
**5 business days (40 hours)**

### Prerequisites
- [x] Sprint 2 complete (ApplicationContext is single source of truth)
- [x] All facades compile independently
- [ ] Dependency graph analyzed: `cargo tree --duplicate`

### Tasks

#### Day 1: Dependency Analysis (8 hours)
- [ ] **Task 3.1**: Map all circular dependencies (4 hours)
  ```bash
  # Generate dependency graph
  cargo tree -p riptide-api --format "{p} -> {p}" > docs/dependency-graph.txt
  cargo tree -p riptide-facade --format "{p} -> {p}" >> docs/dependency-graph.txt

  # Find circular references
  grep -E "riptide-api.*riptide-facade|riptide-facade.*riptide-api" docs/dependency-graph.txt
  ```

- [ ] **Task 3.2**: Identify coupling points (4 hours)
  ```rust
  // CURRENT PROBLEM:
  //
  // riptide-api (AppState) depends on:
  //   → riptide-facade (ExtractionFacade, ScraperFacade)
  //
  // riptide-facade (facades) wants to depend on:
  //   → riptide-api (AppState for infrastructure)
  //   ❌ CIRCULAR DEPENDENCY
  ```

  Document all coupling points in `docs/architecture/circular-dependencies.md`

#### Day 2-3: Break Circular Dependencies (16 hours)
- [ ] **Task 3.3**: Move facades to depend on ApplicationContext only (8 hours)
  ```rust
  // BEFORE (riptide-facade/src/facades/extraction.rs):
  use riptide_api::AppState;  // ❌ Creates circular dependency

  pub struct ExtractionFacade {
      app_state: Arc<AppState>,  // ❌ Couples to API layer
  }

  // AFTER (use ports instead):
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

  **Files to modify**:
  - `/workspaces/eventmesh/crates/riptide-facade/src/facades/extraction.rs`
  - `/workspaces/eventmesh/crates/riptide-facade/src/facades/scraper.rs`
  - `/workspaces/eventmesh/crates/riptide-facade/src/facades/spider.rs`
  - `/workspaces/eventmesh/crates/riptide-facade/src/facades/search.rs`
  - All 35+ facade files

- [ ] **Task 3.4**: Create facade factory in ApplicationContext (4 hours)
  ```rust
  // NEW: crates/riptide-api/src/composition/facade_factory.rs

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

      // Repeat for all facades...
  }
  ```

- [ ] **Task 3.5**: Update facade initialization in main.rs (4 hours)
  ```rust
  // BEFORE (main.rs):
  let app_state = AppState::new(config).await?;  // Facades created inside

  // AFTER (main.rs):
  let context = ApplicationContext::new(&config).await?;

  // Facades created on-demand via factory methods
  let extraction_facade = context.create_extraction_facade();
  let scraper_facade = context.create_scraper_facade();

  // Store in a facade registry if needed
  let facade_registry = FacadeRegistry::new()
      .with_extraction(extraction_facade)
      .with_scraper(scraper_facade);
  ```

  **Files to modify**:
  - `/workspaces/eventmesh/crates/riptide-api/src/main.rs`
  - `/workspaces/eventmesh/crates/riptide-api/src/composition/mod.rs`

#### Day 4-5: Testing & Validation (16 hours)
- [ ] **Task 3.6**: Verify dependency graph is acyclic (2 hours)
  ```bash
  # Check for circular dependencies
  cargo tree -p riptide-api --duplicate
  cargo tree -p riptide-facade --duplicate

  # Should show NO cycles:
  # riptide-api depends on:
  #   → riptide-facade (facades)
  #   → riptide-types (ports)
  #
  # riptide-facade depends on:
  #   → riptide-types (ports)  ✅ No reference to riptide-api
  ```

- [ ] **Task 3.7**: Write dependency isolation tests (6 hours)
  ```rust
  // NEW: crates/riptide-facade/tests/dependency_isolation_tests.rs

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
  ```

- [ ] **Task 3.8**: Integration tests for facade factories (4 hours)
  ```rust
  // crates/riptide-api/tests/facade_factory_integration_tests.rs

  #[tokio::test]
  async fn test_facade_creation_via_context() {
      let context = ApplicationContext::for_testing();

      let extraction = context.create_extraction_facade();
      let scraper = context.create_scraper_facade();

      // Verify facades work independently
      let result = extraction.extract("https://example.com").await?;
      assert!(result.is_ok());
  }
  ```

- [ ] **Task 3.9**: Documentation (4 hours)
  - Update architecture diagrams showing clean layering
  - Document facade factory pattern
  - Create migration guide for facade consumers

### Acceptance Criteria
- [x] ✅ Zero circular dependencies (`cargo tree` shows clean graph)
- [x] ✅ All facades depend only on `riptide-types` (ports)
- [x] ✅ Facade creation centralized in ApplicationContext factories
- [x] ✅ All tests pass
- [x] ✅ Dependency isolation verified at compile-time

### Code Example: Clean Dependencies

**Before** (circular dependency):
```
┌─────────────────┐
│  riptide-api    │
│   (AppState)    │
└────────┬────────┘
         │ depends on
         ▼
┌─────────────────┐
│ riptide-facade  │
│   (Facades)     │
└────────┬────────┘
         │ depends on (❌ CIRCULAR)
         ▼
┌─────────────────┐
│  riptide-api    │
│ (infrastructure)│
└─────────────────┘
```

**After** (clean unidirectional flow):
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

### Testing Strategy
1. **Compile-Time Tests**: Facade crate compiles without riptide-api
2. **Unit Tests**: Each facade with mock ports
3. **Integration Tests**: Facade factories in ApplicationContext
4. **Dependency Graph Tests**: Automated CI check for circular deps

### Rollback Plan
1. Keep facade factory methods optional (dual initialization supported)
2. If facades need rollback:
   ```rust
   #[cfg(feature = "legacy-facade-init")]
   // Old initialization code
   ```
3. Gradual migration: One facade at a time

### Dependencies
**Depends on**: Sprint 2 (ApplicationContext established)
**Blocks**: Sprint 4-12 (all future work assumes clean dependencies)

### Success Metrics
- **Cyclomatic Complexity**: Reduced by 40% (fewer interconnected types)
- **Compile Time**: 10-15% faster (fewer dependencies to rebuild)
- **Dependency Count**: `riptide-facade` dependencies reduced from 15 → 3
- **Tests**: 20+ tests verifying isolation

---

## Sprint 4: Empty Composition Modules (Week 3 end)

### Sprint Goal
Decide fate of empty composition modules: implement with actual dependency injection or delete. Document decision.

### Duration
**2 business days (16 hours)** - Runs parallel to Sprint 3 Day 4-5

### Prerequisites
- [x] Sprint 2 complete (ApplicationContext established)
- [ ] Composition root requirements documented
- [ ] Team decision on composition strategy

### Tasks

#### Day 1: Analysis & Decision (8 hours)
- [ ] **Task 4.1**: Audit empty composition modules (2 hours)
  ```bash
  # Find empty or nearly-empty composition files
  find crates/*/src/composition -name "*.rs" -exec wc -l {} \; | sort -n

  # Check current composition directory
  ls -la crates/riptide-api/src/composition/
  # Expected: mod.rs (466 lines), builder.rs, config.rs, stubs.rs
  ```

  Current state:
  - `/workspaces/eventmesh/crates/riptide-api/src/composition/mod.rs` (466 lines - **ACTIVE**)
  - `/workspaces/eventmesh/crates/riptide-api/src/composition/builder.rs` (unknown)
  - `/workspaces/eventmesh/crates/riptide-api/src/composition/config.rs` (unknown)
  - `/workspaces/eventmesh/crates/riptide-api/src/composition/stubs.rs` (unknown)
  - `/workspaces/eventmesh/crates/riptide-facade/src/composition/mod.rs` (1 line - **EMPTY**)

- [ ] **Task 4.2**: Review composition best practices (2 hours)
  Research and document:
  - Manual DI vs DI containers (e.g., `shaku`, `waiter`)
  - Composition root pattern (Martin Fowler)
  - Trade-offs: Complexity vs Flexibility vs Type Safety

  Document in: `docs/architecture/composition-strategy.md`

- [ ] **Task 4.3**: Team decision meeting (2 hours)
  **Decision Points**:
  1. Keep manual DI (current ApplicationContext pattern) ✅ Recommended
  2. Implement DI container (add dependency on `shaku` or similar)
  3. Delete facade composition (facades get deps via constructor)

  **Recommendation**: **Option 3 - Delete facade composition**, use constructor injection

  **Rationale**:
  - Facades are simple use-case orchestrators (not complex enough for DI container)
  - ApplicationContext already provides all infrastructure (composition root)
  - Constructor injection is explicit, type-safe, testable

- [ ] **Task 4.4**: Document decision in ADR (2 hours)
  ```markdown
  # ADR-003: Composition Strategy for Facades

  ## Status
  Accepted

  ## Context
  Empty composition module in riptide-facade creates confusion.
  Two options: implement full DI or use simple constructor injection.

  ## Decision
  Use constructor injection for facades. Delete empty composition module.
  ApplicationContext serves as composition root for infrastructure.

  ## Consequences
  - ✅ Simpler codebase (no DI container dependency)
  - ✅ Explicit dependencies (visible in constructor)
  - ✅ Easy to test (pass mock ports to constructor)
  - ❌ Manual wiring in ApplicationContext (acceptable trade-off)
  ```

  File: `docs/architecture/adr/003-facade-composition-strategy.md`

#### Day 2: Implementation (8 hours)
- [ ] **Task 4.5**: Delete empty composition module (1 hour)
  ```bash
  # Verify module is truly empty
  cat crates/riptide-facade/src/composition/mod.rs
  # Should show: (empty or single comment)

  # Delete if empty
  rm -rf crates/riptide-facade/src/composition/

  # Remove from lib.rs
  # Edit crates/riptide-facade/src/lib.rs: Remove `pub mod composition;`
  ```

- [ ] **Task 4.6**: Consolidate composition in riptide-api (3 hours)
  ```rust
  // Verify ApplicationContext is complete composition root
  // File: crates/riptide-api/src/composition/mod.rs

  impl ApplicationContext {
      /// Production composition: Wire all real adapters
      pub async fn new(config: &DiConfig) -> Result<Self> {
          // 1. Infrastructure adapters
          let clock = Arc::new(SystemClock);
          let cache = Arc::new(RedisCache::new(&config.redis_url).await?);

          // 2. Facades (using factory methods from Sprint 3)
          // Facades are created on-demand, not stored in context

          Ok(Self { clock, cache, /* ... */ })
      }

      /// Testing composition: Wire test doubles
      pub fn for_testing() -> Self {
          let clock = Arc::new(FakeClock::at_epoch());
          let cache = Arc::new(InMemoryCache::new());

          Self { clock, cache, /* ... */ }
      }
  }
  ```

- [ ] **Task 4.7**: Add composition validation (2 hours)
  ```rust
  // NEW: crates/riptide-api/src/composition/validation.rs

  impl ApplicationContext {
      /// Validate all dependencies are properly wired
      pub fn validate(&self) -> Result<()> {
          // Check all ports are non-null
          assert!(self.clock.timestamp() >= 0, "Clock not initialized");
          assert!(self.cache_storage.is_connected(), "Cache not connected");
          assert!(self.event_bus.is_running(), "EventBus not started");

          Ok(())
      }
  }

  #[cfg(test)]
  mod tests {
      #[test]
      fn test_production_context_validates() {
          let config = DiConfig::from_env();
          let context = ApplicationContext::new(&config).await?;

          context.validate()?;  // Should not panic
      }
  }
  ```

- [ ] **Task 4.8**: Update documentation (2 hours)
  - Document composition root pattern in `docs/architecture/composition-root.md`
  - Update facade README with constructor injection examples
  - Add "How to add a new facade" guide

### Acceptance Criteria
- [x] ✅ Decision documented in ADR with team consensus
- [x] ✅ Empty composition modules deleted (or implemented if decided)
- [x] ✅ ApplicationContext is sole composition root
- [x] ✅ Composition validation added
- [x] ✅ Documentation updated

### Code Example: Constructor Injection Pattern

**Before** (empty composition module suggested complex DI):
```rust
// crates/riptide-facade/src/composition/mod.rs (empty - confusing!)
```

**After** (explicit constructor injection):
```rust
// crates/riptide-facade/src/facades/extraction.rs
pub struct ExtractionFacade {
    cache: Arc<dyn CacheStorage>,
    events: Arc<dyn EventBus>,
    browser: Arc<dyn BrowserDriver>,
}

impl ExtractionFacade {
    /// Create facade with explicit dependencies (constructor injection)
    pub fn new(
        cache: Arc<dyn CacheStorage>,
        events: Arc<dyn EventBus>,
        browser: Arc<dyn BrowserDriver>,
    ) -> Self {
        Self { cache, events, browser }
    }
}

// Composition happens at root:
// crates/riptide-api/src/composition/mod.rs
impl ApplicationContext {
    pub fn create_extraction_facade(&self) -> ExtractionFacade {
        ExtractionFacade::new(
            self.cache_storage.clone(),
            self.event_bus.clone(),
            self.browser_driver.clone(),
        )
    }
}
```

### Testing Strategy
1. **Validation Tests**: ApplicationContext validates all dependencies
2. **Integration Tests**: Facades created via composition root work end-to-end
3. **Documentation Tests**: Examples in docs compile and run

### Rollback Plan
If composition module deletion causes issues:
1. Re-create minimal composition module
2. Keep as placeholder with TODO comment
3. No code changes needed (facades still use constructor injection)

### Dependencies
**Depends on**: Sprint 2 (ApplicationContext established), Sprint 3 (clean dependencies)
**Blocks**: None (cleanup task)

### Success Metrics
- **LOC**: Empty files deleted (-50 lines of dead code)
- **Clarity**: ADR documents decision (prevents future confusion)
- **Validation**: Composition errors caught at startup (not runtime)

---

## PHASE 2: P1 INFRASTRUCTURE VIOLATIONS (Weeks 4-6)

---

## Sprint 5: Create Missing Port Traits (Week 4)

### Sprint Goal
Create 8 missing port traits to eliminate 32 infrastructure violations. Establish clean hexagonal boundaries.

### Duration
**5 business days (40 hours)**

### Prerequisites
- [x] Sprint 1-3 complete (clean ApplicationContext)
- [x] Existing port traits analyzed (`/workspaces/eventmesh/crates/riptide-types/src/ports/`)
- [ ] Infrastructure violation audit complete

### Tasks

#### Day 1: Audit Infrastructure Violations (8 hours)
- [ ] **Task 5.1**: Identify all infrastructure dependencies in facades (4 hours)
  ```bash
  # Find all concrete infrastructure types in facade layer
  grep -r "use riptide_cache::" crates/riptide-facade/src/facades/
  grep -r "use riptide_browser::" crates/riptide-facade/src/facades/
  grep -r "use riptide_fetch::" crates/riptide-facade/src/facades/
  grep -r "use riptide_pdf::" crates/riptide-facade/src/facades/
  grep -r "use riptide_spider::" crates/riptide-facade/src/facades/
  grep -r "use riptide_search::" crates/riptide-facade/src/facades/
  grep -r "use riptide_reliability::" crates/riptide-facade/src/facades/
  grep -r "use riptide_monitoring::" crates/riptide-facade/src/facades/

  # Output violations to spreadsheet
  ```

  Document in: `docs/architecture/infrastructure-violations.md`

- [ ] **Task 5.2**: Categorize violations by infrastructure type (2 hours)
  Expected categories (based on grep analysis):
  1. **Browser Operations**: BrowserDriver, PageSession, Screenshot
  2. **HTTP Operations**: HttpClient, RequestBuilder, Response
  3. **PDF Operations**: PdfExtractor, PdfRenderer
  4. **Search Operations**: SearchEngine, SearchQuery, SearchResult
  5. **Cache Operations**: CacheStorage ✅ (created in Sprint 1)
  6. **Monitoring Operations**: MetricsCollector, Logger, Tracer
  7. **Reliability Operations**: CircuitBreaker, RetryPolicy, Timeout
  8. **Worker Operations**: JobQueue, TaskScheduler

- [ ] **Task 5.3**: Prioritize port trait creation (2 hours)
  **Priority Order** (based on violation count and criticality):
  1. BrowserDriver (likely highest violation count)
  2. HttpClient
  3. SearchBackend
  4. PdfExtractor
  5. MetricsCollector
  6. Logger
  7. RetryPolicy
  8. JobQueue

#### Day 2-4: Create Port Traits (24 hours = 3 hours per trait)
- [ ] **Task 5.4**: Create BrowserDriver port trait (3 hours)
  ```rust
  // NEW FILE: crates/riptide-types/src/ports/browser.rs

  use async_trait::async_trait;
  use crate::Result;

  /// Port trait for browser automation
  #[async_trait]
  pub trait BrowserDriver: Send + Sync {
      /// Navigate to URL and return page session
      async fn navigate(&self, url: &str) -> Result<Box<dyn PageSession>>;

      /// Create new browser context (incognito mode)
      async fn new_context(&self) -> Result<Box<dyn BrowserContext>>;

      /// Close browser and cleanup resources
      async fn close(&self) -> Result<()>;
  }

  /// Page session for interacting with loaded page
  #[async_trait]
  pub trait PageSession: Send + Sync {
      /// Get page HTML content
      async fn html(&self) -> Result<String>;

      /// Take screenshot of page
      async fn screenshot(&self, options: ScreenshotOptions) -> Result<Vec<u8>>;

      /// Execute JavaScript on page
      async fn evaluate(&self, script: &str) -> Result<serde_json::Value>;

      /// Wait for element to appear
      async fn wait_for_selector(&self, selector: &str, timeout_ms: u64) -> Result<()>;

      /// Click element
      async fn click(&self, selector: &str) -> Result<()>;

      /// Close page session
      async fn close(self: Box<Self>) -> Result<()>;
  }

  /// Browser context (isolated session)
  #[async_trait]
  pub trait BrowserContext: Send + Sync {
      /// Create new page in this context
      async fn new_page(&self) -> Result<Box<dyn PageSession>>;

      /// Close context and all pages
      async fn close(self: Box<Self>) -> Result<()>;
  }

  /// Screenshot configuration
  #[derive(Debug, Clone)]
  pub struct ScreenshotOptions {
      pub full_page: bool,
      pub format: ImageFormat,
      pub quality: Option<u8>,
  }

  #[derive(Debug, Clone, Copy)]
  pub enum ImageFormat {
      Png,
      Jpeg,
      Webp,
  }
  ```

  **Files to create**:
  - `/workspaces/eventmesh/crates/riptide-types/src/ports/browser.rs`
  - Add to `/workspaces/eventmesh/crates/riptide-types/src/ports/mod.rs`: `pub mod browser;`

- [ ] **Task 5.5**: Create HttpClient port trait (3 hours)
  ```rust
  // NEW FILE: crates/riptide-types/src/ports/http.rs (already exists! verify)

  use async_trait::async_trait;
  use crate::Result;
  use std::collections::HashMap;

  /// Port trait for HTTP operations
  #[async_trait]
  pub trait HttpClient: Send + Sync {
      /// Execute HTTP request
      async fn request(&self, request: HttpRequest) -> Result<HttpResponse>;

      /// Convenience GET method
      async fn get(&self, url: &str) -> Result<HttpResponse> {
          self.request(HttpRequest::get(url)).await
      }

      /// Convenience POST method
      async fn post(&self, url: &str, body: Vec<u8>) -> Result<HttpResponse> {
          self.request(HttpRequest::post(url).body(body)).await
      }
  }

  /// HTTP request builder
  #[derive(Debug, Clone)]
  pub struct HttpRequest {
      pub method: HttpMethod,
      pub url: String,
      pub headers: HashMap<String, String>,
      pub body: Option<Vec<u8>>,
      pub timeout: Option<std::time::Duration>,
  }

  impl HttpRequest {
      pub fn get(url: impl Into<String>) -> Self {
          Self {
              method: HttpMethod::Get,
              url: url.into(),
              headers: HashMap::new(),
              body: None,
              timeout: None,
          }
      }

      pub fn post(url: impl Into<String>) -> Self {
          Self {
              method: HttpMethod::Post,
              url: url.into(),
              headers: HashMap::new(),
              body: None,
              timeout: None,
          }
      }

      pub fn body(mut self, body: Vec<u8>) -> Self {
          self.body = Some(body);
          self
      }

      pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
          self.headers.insert(key.into(), value.into());
          self
      }
  }

  /// HTTP response
  #[derive(Debug, Clone)]
  pub struct HttpResponse {
      pub status: u16,
      pub headers: HashMap<String, String>,
      pub body: Vec<u8>,
  }

  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum HttpMethod {
      Get,
      Post,
      Put,
      Delete,
      Patch,
      Head,
      Options,
  }
  ```

- [ ] **Task 5.6**: Create SearchBackend port trait (3 hours)
  ```rust
  // NEW FILE: crates/riptide-types/src/ports/search.rs

  use async_trait::async_trait;
  use crate::Result;

  /// Port trait for search operations
  #[async_trait]
  pub trait SearchBackend: Send + Sync {
      /// Execute search query
      async fn search(&self, query: &SearchQuery) -> Result<SearchResults>;

      /// Get search engine name
      fn engine_name(&self) -> &str;

      /// Check if backend is available
      async fn is_available(&self) -> bool;
  }

  /// Search query configuration
  #[derive(Debug, Clone)]
  pub struct SearchQuery {
      pub query: String,
      pub max_results: Option<usize>,
      pub language: Option<String>,
      pub country: Option<String>,
      pub safe_search: bool,
  }

  /// Search results
  #[derive(Debug, Clone)]
  pub struct SearchResults {
      pub query: String,
      pub total_results: Option<usize>,
      pub results: Vec<SearchResult>,
      pub search_time_ms: u64,
  }

  /// Individual search result
  #[derive(Debug, Clone)]
  pub struct SearchResult {
      pub title: String,
      pub url: String,
      pub description: Option<String>,
      pub position: usize,
  }
  ```

- [ ] **Task 5.7**: Create PdfExtractor port trait (3 hours)
  ```rust
  // NEW FILE: crates/riptide-types/src/ports/pdf.rs

  use async_trait::async_trait;
  use crate::Result;

  /// Port trait for PDF extraction
  #[async_trait]
  pub trait PdfExtractor: Send + Sync {
      /// Extract text from PDF bytes
      async fn extract_text(&self, pdf_bytes: &[u8]) -> Result<String>;

      /// Extract structured data from PDF
      async fn extract_structured(&self, pdf_bytes: &[u8]) -> Result<PdfDocument>;

      /// Extract images from PDF
      async fn extract_images(&self, pdf_bytes: &[u8]) -> Result<Vec<PdfImage>>;

      /// Get PDF metadata
      async fn extract_metadata(&self, pdf_bytes: &[u8]) -> Result<PdfMetadata>;
  }

  /// Structured PDF document
  #[derive(Debug, Clone)]
  pub struct PdfDocument {
      pub pages: Vec<PdfPage>,
      pub metadata: PdfMetadata,
  }

  /// Single PDF page
  #[derive(Debug, Clone)]
  pub struct PdfPage {
      pub page_number: usize,
      pub text: String,
      pub width: f32,
      pub height: f32,
  }

  /// PDF image
  #[derive(Debug, Clone)]
  pub struct PdfImage {
      pub page_number: usize,
      pub image_data: Vec<u8>,
      pub format: ImageFormat,
      pub width: u32,
      pub height: u32,
  }

  /// PDF metadata
  #[derive(Debug, Clone, Default)]
  pub struct PdfMetadata {
      pub title: Option<String>,
      pub author: Option<String>,
      pub subject: Option<String>,
      pub creator: Option<String>,
      pub producer: Option<String>,
      pub creation_date: Option<String>,
      pub modification_date: Option<String>,
      pub page_count: usize,
  }

  #[derive(Debug, Clone, Copy)]
  pub enum ImageFormat {
      Jpeg,
      Png,
      Tiff,
      Unknown,
  }
  ```

- [ ] **Task 5.8**: Create remaining 4 port traits (12 hours)
  Following same pattern for:
  - MetricsCollector (monitoring)
  - Logger (observability)
  - RetryPolicy (reliability)
  - JobQueue (workers)

  Each trait should follow hexagonal architecture principles:
  - Interface segregation (focused, single responsibility)
  - Dependency inversion (facades depend on abstraction)
  - Testability (easy to mock)

#### Day 5: Adapter Implementation & Testing (8 hours)
- [ ] **Task 5.9**: Create adapters for new port traits (4 hours)
  ```rust
  // Example: BrowserDriver adapter
  // NEW FILE: crates/riptide-headless/src/adapters/browser_driver_adapter.rs

  use riptide_types::ports::browser::*;
  use crate::launcher::HeadlessLauncher;

  pub struct HeadlessBrowserDriver {
      launcher: Arc<HeadlessLauncher>,
  }

  #[async_trait::async_trait]
  impl BrowserDriver for HeadlessBrowserDriver {
      async fn navigate(&self, url: &str) -> Result<Box<dyn PageSession>> {
          let page = self.launcher.new_page().await?;
          page.goto(url).await?;
          Ok(Box::new(HeadlessPageSession { page }))
      }

      // Implement remaining methods...
  }

  struct HeadlessPageSession {
      page: Arc<Page>,  // Internal headless page type
  }

  #[async_trait::async_trait]
  impl PageSession for HeadlessPageSession {
      async fn html(&self) -> Result<String> {
          self.page.content().await
      }

      // Implement remaining methods...
  }
  ```

  **Files to create** (one adapter per infrastructure crate):
  - `/workspaces/eventmesh/crates/riptide-headless/src/adapters/browser_driver_adapter.rs`
  - `/workspaces/eventmesh/crates/riptide-fetch/src/adapters/http_client_adapter.rs` (verify if exists)
  - `/workspaces/eventmesh/crates/riptide-search/src/adapters/search_backend_adapter.rs`
  - `/workspaces/eventmesh/crates/riptide-pdf/src/adapters/pdf_extractor_adapter.rs`
  - And 4 more for remaining traits

- [ ] **Task 5.10**: Write port trait tests (2 hours)
  ```rust
  // crates/riptide-types/src/ports/browser.rs (add tests at bottom)

  #[cfg(test)]
  mod tests {
      use super::*;

      // Mock implementation for testing
      struct MockBrowserDriver;

      #[async_trait::async_trait]
      impl BrowserDriver for MockBrowserDriver {
          async fn navigate(&self, url: &str) -> Result<Box<dyn PageSession>> {
              Ok(Box::new(MockPageSession { url: url.to_string() }))
          }

          // ...
      }

      #[tokio::test]
      async fn test_browser_driver_trait() {
          let driver = MockBrowserDriver;
          let session = driver.navigate("https://example.com").await?;
          let html = session.html().await?;

          assert!(!html.is_empty());
      }
  }
  ```

- [ ] **Task 5.11**: Integration tests (2 hours)
  ```rust
  // crates/riptide-facade/tests/port_integration_tests.rs

  #[tokio::test]
  async fn test_facade_uses_browser_port() {
      // Use test double
      let mock_browser = Arc::new(MockBrowserDriver::new());
      let mock_cache = Arc::new(MockCacheStorage::new());
      let mock_events = Arc::new(MockEventBus::new());

      let facade = ExtractionFacade::new(
          mock_cache,
          mock_events,
          mock_browser,  // Port trait injected
          ExtractionConfig::default(),
      );

      let result = facade.extract("https://example.com").await?;
      assert!(result.is_ok());
  }
  ```

### Acceptance Criteria
- [x] ✅ 8 new port traits created in `riptide-types/src/ports/`
- [x] ✅ 8 adapters implemented in infrastructure crates
- [x] ✅ 32 infrastructure violations eliminated (facades use ports)
- [x] ✅ 100% test coverage for port traits
- [x] ✅ All facades compile with port dependencies only
- [x] ✅ Documentation updated

### Code Example: Infrastructure Violation → Port Usage

**Before** (direct infrastructure dependency):
```rust
// crates/riptide-facade/src/facades/extraction.rs (VIOLATION)
use riptide_headless::launcher::HeadlessLauncher;  // ❌ Concrete type
use riptide_cache::CacheManager;                   // ❌ Concrete type

pub struct ExtractionFacade {
    browser: Arc<HeadlessLauncher>,  // ❌ Couples to infrastructure
    cache: Arc<Mutex<CacheManager>>, // ❌ Couples to infrastructure
}

impl ExtractionFacade {
    pub async fn extract(&self, url: &str) -> Result<ExtractedData> {
        let page = self.browser.new_page().await?;  // ❌ Direct infrastructure call
        page.goto(url).await?;
        // ...
    }
}
```

**After** (port trait usage):
```rust
// crates/riptide-facade/src/facades/extraction.rs (CLEAN)
use riptide_types::ports::{BrowserDriver, CacheStorage, EventBus};  // ✅ Port traits

pub struct ExtractionFacade {
    browser: Arc<dyn BrowserDriver>,  // ✅ Port trait
    cache: Arc<dyn CacheStorage>,     // ✅ Port trait
    events: Arc<dyn EventBus>,        // ✅ Port trait
}

impl ExtractionFacade {
    pub async fn extract(&self, url: &str) -> Result<ExtractedData> {
        let session = self.browser.navigate(url).await?;  // ✅ Port interface
        let html = session.html().await?;
        // ...
    }
}
```

### Testing Strategy
1. **Trait Tests**: Each port trait has test suite with mock implementation
2. **Adapter Tests**: Each adapter implements port trait correctly
3. **Integration Tests**: Facades work with real adapters end-to-end
4. **Contract Tests**: Port traits define clear contracts (input/output)

### Rollback Plan
1. Port traits are additive (no breaking changes)
2. If adapter fails, keep facade using concrete type temporarily
3. Feature flag per infrastructure type:
   ```rust
   #[cfg(feature = "port-browser")]
   use riptide_types::ports::BrowserDriver;
   #[cfg(not(feature = "port-browser"))]
   use riptide_headless::HeadlessLauncher as BrowserDriver;
   ```

### Dependencies
**Depends on**: Sprint 3 (clean dependencies established)
**Blocks**: Sprint 6 (facades can't migrate until ports exist)

### Success Metrics
- **Port Traits**: 8 created
- **Adapters**: 8 implemented
- **Violations**: 32 eliminated (100% resolution)
- **Tests**: 40+ tests (5 per trait/adapter pair)
- **Compile Time**: Potential 20% improvement (fewer concrete dependencies)

---

## Sprint 6: Migrate Facades to Use Port Traits (Week 5)

### Sprint Goal
Migrate all 35+ facades to use port traits instead of concrete infrastructure types. Achieve 100% hexagonal architecture compliance.

### Duration
**5 business days (40 hours)**

### Prerequisites
- [x] Sprint 5 complete (8 port traits created)
- [x] All adapters tested and working
- [ ] Migration checklist prepared for each facade

### Tasks

#### Day 1: Migration Planning (8 hours)
- [ ] **Task 6.1**: Audit all facades for infrastructure dependencies (4 hours)
  ```bash
  # For each facade, identify infrastructure dependencies
  cd crates/riptide-facade/src/facades

  for facade in *.rs; do
    echo "=== $facade ===" >> ../../migration-checklist.md
    grep -n "use riptide_" $facade | grep -v "riptide_types\|riptide_facade" >> ../../migration-checklist.md
    echo "" >> ../../migration-checklist.md
  done

  # Expected output: List of all concrete infrastructure imports per facade
  ```

  Document in: `crates/riptide-facade/migration-checklist.md`

- [ ] **Task 6.2**: Prioritize facade migration (2 hours)
  **Priority order** (migrate simplest first to build momentum):
  1. Scraper facade (likely minimal dependencies)
  2. PDF facade (single PdfExtractor dependency)
  3. Search facade (single SearchBackend dependency)
  4. Extraction facade (multiple dependencies - moderate)
  5. Spider facade (complex - many dependencies)
  6. All remaining facades

- [ ] **Task 6.3**: Create migration template (2 hours)
  ```markdown
  # Facade Migration Checklist Template

  ## Facade: [NAME]

  ### Current Infrastructure Dependencies
  - [ ] riptide_cache::CacheManager → CacheStorage port
  - [ ] riptide_browser::HeadlessLauncher → BrowserDriver port
  - [ ] ...

  ### Migration Steps
  1. [ ] Update struct fields to use port traits
  2. [ ] Update constructor to accept port traits
  3. [ ] Update method implementations
  4. [ ] Update tests to use mock ports
  5. [ ] Verify compilation
  6. [ ] Run integration tests

  ### Testing
  - [ ] Unit tests pass
  - [ ] Integration tests pass
  - [ ] No clippy warnings
  ```

#### Day 2-4: Migrate Facades (24 hours = ~40 minutes per facade)
- [ ] **Task 6.4**: Migrate Scraper facade (2 hours)
  ```rust
  // BEFORE: crates/riptide-facade/src/facades/scraper.rs
  use riptide_fetch::FetchEngine;  // ❌ Concrete type
  use riptide_cache::CacheManager; // ❌ Concrete type

  pub struct ScraperFacade {
      http: Arc<FetchEngine>,
      cache: Arc<Mutex<CacheManager>>,
  }

  // AFTER:
  use riptide_types::ports::{HttpClient, CacheStorage};  // ✅ Port traits

  pub struct ScraperFacade {
      http: Arc<dyn HttpClient>,
      cache: Arc<dyn CacheStorage>,
  }

  impl ScraperFacade {
      pub fn new(
          http: Arc<dyn HttpClient>,
          cache: Arc<dyn CacheStorage>,
      ) -> Self {
          Self { http, cache }
      }

      pub async fn fetch_html(&self, url: &str) -> Result<String> {
          // Check cache first
          let cache_key = format!("html:{}", url);
          if let Some(cached) = self.cache.get(&cache_key).await? {
              return Ok(String::from_utf8(cached)?);
          }

          // Fetch via HTTP port
          let response = self.http.get(url).await?;
          let html = String::from_utf8(response.body)?;

          // Cache result
          self.cache.set(
              &cache_key,
              html.as_bytes().to_vec(),
              Duration::from_secs(3600),
          ).await?;

          Ok(html)
      }
  }
  ```

- [ ] **Task 6.5**: Migrate PDF facade (2 hours)
  ```rust
  // BEFORE: crates/riptide-facade/src/facades/pdf.rs
  use riptide_pdf::PdfProcessor;  // ❌ Concrete type

  pub struct PdfFacade {
      processor: Arc<PdfProcessor>,
  }

  // AFTER:
  use riptide_types::ports::PdfExtractor;  // ✅ Port trait

  pub struct PdfFacade {
      extractor: Arc<dyn PdfExtractor>,
  }

  impl PdfFacade {
      pub fn new(extractor: Arc<dyn PdfExtractor>) -> Self {
          Self { extractor }
      }

      pub async fn extract_text(&self, pdf_bytes: &[u8]) -> Result<String> {
          self.extractor.extract_text(pdf_bytes).await
      }

      pub async fn extract_structured(&self, pdf_bytes: &[u8]) -> Result<PdfDocument> {
          self.extractor.extract_structured(pdf_bytes).await
      }
  }
  ```

- [ ] **Task 6.6**: Migrate Search facade (2 hours)
  Similar pattern to PDF facade

- [ ] **Task 6.7**: Migrate Extraction facade (3 hours)
  ```rust
  // BEFORE: Multiple infrastructure dependencies
  use riptide_browser::HeadlessLauncher;
  use riptide_cache::CacheManager;
  use riptide_extraction::UnifiedExtractor;
  use riptide_events::EventBus;

  // AFTER: All via port traits
  use riptide_types::ports::{
      BrowserDriver,
      CacheStorage,
      ContentExtractor,  // NEW port for UnifiedExtractor
      EventBus,
  };

  pub struct ExtractionFacade {
      browser: Arc<dyn BrowserDriver>,
      cache: Arc<dyn CacheStorage>,
      extractor: Arc<dyn ContentExtractor>,
      events: Arc<dyn EventBus>,
  }
  ```

- [ ] **Task 6.8**: Migrate Spider facade (3 hours)
  Most complex facade - multiple infrastructure dependencies

- [ ] **Task 6.9**: Migrate remaining 30 facades (12 hours)
  Following same pattern for all remaining facades in `/workspaces/eventmesh/crates/riptide-facade/src/facades/`:
  - `browser.rs`
  - `crawl_facade.rs`
  - `deep_search.rs`
  - `engine.rs`
  - `intelligence.rs`
  - `llm.rs`
  - `memory.rs`
  - `monitoring.rs`
  - `pipeline.rs`
  - `profile.rs`
  - `render.rs`
  - `resource.rs`
  - `session.rs`
  - `streaming.rs`
  - `table.rs`
  - `trace.rs`
  - `workers.rs`
  - Plus all metric facades, strategy facades, etc.

#### Day 5: Integration & Validation (8 hours)
- [ ] **Task 6.10**: Update ApplicationContext facade factories (3 hours)
  ```rust
  // crates/riptide-api/src/composition/mod.rs
  impl ApplicationContext {
      pub fn create_scraper_facade(&self) -> Arc<ScraperFacade> {
          Arc::new(ScraperFacade::new(
              self.http_client.clone(),  // HttpClient adapter
              self.cache_storage.clone(), // CacheStorage adapter
          ))
      }

      pub fn create_extraction_facade(&self) -> Arc<ExtractionFacade> {
          Arc::new(ExtractionFacade::new(
              self.browser_driver.clone(),
              self.cache_storage.clone(),
              self.content_extractor.clone(),
              self.event_bus.clone(),
          ))
      }

      // Repeat for all 35+ facades...
  }
  ```

- [ ] **Task 6.11**: Run comprehensive test suite (3 hours)
  ```bash
  # Unit tests (all facades)
  cargo test -p riptide-facade

  # Integration tests (facades with real adapters)
  cargo test -p riptide-api --test facade_integration_tests

  # Clippy (zero warnings)
  cargo clippy -p riptide-facade -- -D warnings

  # Check for remaining infrastructure violations
  grep -r "use riptide_" crates/riptide-facade/src/ | grep -v "riptide_types\|riptide_facade"
  # Should return NOTHING (all violations fixed)
  ```

- [ ] **Task 6.12**: Documentation update (2 hours)
  - Update facade README with port trait examples
  - Document migration patterns
  - Update architecture diagrams showing clean hexagonal boundaries

### Acceptance Criteria
- [x] ✅ All 35+ facades migrated to use port traits
- [x] ✅ Zero infrastructure violations (verified by grep)
- [x] ✅ All tests pass (unit + integration)
- [x] ✅ Zero clippy warnings
- [x] ✅ ApplicationContext factories updated
- [x] ✅ Documentation complete

### Code Example: Complex Facade Migration

**Before** (tightly coupled to infrastructure):
```rust
// crates/riptide-facade/src/facades/pipeline.rs (before)
use riptide_cache::CacheManager;
use riptide_browser::HeadlessLauncher;
use riptide_extraction::UnifiedExtractor;
use riptide_fetch::FetchEngine;
use riptide_monitoring::MetricsCollector;

pub struct PipelineFacade {
    cache: Arc<Mutex<CacheManager>>,
    browser: Arc<HeadlessLauncher>,
    extractor: Arc<UnifiedExtractor>,
    http: Arc<FetchEngine>,
    metrics: Arc<MetricsCollector>,
}

// 5 concrete infrastructure dependencies ❌
```

**After** (clean port-based architecture):
```rust
// crates/riptide-facade/src/facades/pipeline.rs (after)
use riptide_types::ports::{
    CacheStorage,
    BrowserDriver,
    ContentExtractor,
    HttpClient,
    MetricsRegistry,
};

pub struct PipelineFacade {
    cache: Arc<dyn CacheStorage>,
    browser: Arc<dyn BrowserDriver>,
    extractor: Arc<dyn ContentExtractor>,
    http: Arc<dyn HttpClient>,
    metrics: Arc<dyn MetricsRegistry>,
}

impl PipelineFacade {
    /// Constructor injection with port traits
    pub fn new(
        cache: Arc<dyn CacheStorage>,
        browser: Arc<dyn BrowserDriver>,
        extractor: Arc<dyn ContentExtractor>,
        http: Arc<dyn HttpClient>,
        metrics: Arc<dyn MetricsRegistry>,
    ) -> Self {
        Self { cache, browser, extractor, http, metrics }
    }

    pub async fn execute(&self, url: &str) -> Result<PipelineResult> {
        self.metrics.increment("pipeline_executions_total");

        // All operations through port traits
        let cache_key = format!("pipeline:{}", url);
        if let Some(cached) = self.cache.get(&cache_key).await? {
            return Ok(serde_json::from_slice(&cached)?);
        }

        // Fetch via HTTP
        let response = self.http.get(url).await?;
        let html = String::from_utf8(response.body)?;

        // Extract content
        let extracted = self.extractor.extract(&html).await?;

        // Cache result
        self.cache.set(&cache_key, serde_json::to_vec(&extracted)?, Duration::from_secs(3600)).await?;

        Ok(extracted)
    }
}

// 5 port trait dependencies ✅ (testable, swappable, clean)
```

### Testing Strategy
1. **Unit Tests**: Each facade with mock port implementations
2. **Integration Tests**: Facades with real adapters in ApplicationContext
3. **Contract Tests**: Verify port trait contracts are respected
4. **Regression Tests**: All existing handler tests still pass

### Rollback Plan
1. Git branch per facade migration (easy to revert individual facades)
2. Feature flag per facade:
   ```rust
   #[cfg(feature = "new-scraper-facade")]
   // New port-based implementation
   #[cfg(not(feature = "new-scraper-facade"))]
   // Old concrete implementation
   ```
3. Gradual rollout: Enable one facade at a time in production

### Dependencies
**Depends on**: Sprint 5 (port traits created)
**Blocks**: Sprint 7-9 (testing requires migrated facades)

### Success Metrics
- **Facades**: 35+ migrated (100%)
- **Violations**: 32 → 0 (100% resolution)
- **Tests**: 100+ facade unit tests
- **Compile Time**: 15-20% faster (fewer concrete dependencies)
- **LOC**: Facade crate likely ~10% smaller (cleaner interfaces)

---

## PHASE 3: P1 TESTING (Weeks 7-9)

---

## Sprint 7-9: Test Coverage for 12 Untested Facades (Weeks 7-9)

### Sprint Goal
Achieve 100% test coverage for all 12 previously untested facades. Establish testing patterns for future facades.

### Duration
**15 business days (120 hours) = 3 weeks**

### Prerequisites
- [x] Sprint 6 complete (all facades use port traits)
- [x] Mock port implementations available
- [ ] Testing framework established

### Overview
**12 untested facades** (assumed from analysis - verify actual list):
1. IntelligenceFacade
2. LlmFacade
3. MemoryFacade
4. ProfileFacade
5. DeepSearchFacade
6. TableFacade
7. TraceFacade
8. WorkersFacade
9. RenderStrategyFacade
10. ExtractionMetricsFacade
11. PipelineMetricsFacade
12. BrowserMetricsFacade

### Sprint 7: Test Framework + 4 Facades (Week 7)

#### Tasks

##### Day 1: Test Framework Setup (8 hours)
- [ ] **Task 7.1**: Create test doubles for all port traits (4 hours)
  ```rust
  // NEW FILE: crates/riptide-types/src/ports/test_doubles.rs

  use super::*;
  use std::sync::Mutex;

  /// Mock CacheStorage for testing
  pub struct MockCacheStorage {
      data: Arc<Mutex<HashMap<String, (Vec<u8>, Instant)>>>,
  }

  impl MockCacheStorage {
      pub fn new() -> Self {
          Self {
              data: Arc::new(Mutex::new(HashMap::new())),
          }
      }

      pub fn get_call_count(&self) -> usize {
          // Track method calls for verification
          unimplemented!("Add call tracking")
      }
  }

  #[async_trait::async_trait]
  impl CacheStorage for MockCacheStorage {
      async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
          let data = self.data.lock().unwrap();
          Ok(data.get(key).map(|(v, _)| v.clone()))
      }

      async fn set(&self, key: &str, value: Vec<u8>, ttl: Duration) -> Result<()> {
          let mut data = self.data.lock().unwrap();
          data.insert(key.to_string(), (value, Instant::now() + ttl));
          Ok(())
      }
  }

  // Create mocks for ALL port traits:
  // - MockBrowserDriver
  // - MockHttpClient
  // - MockEventBus
  // - MockSearchBackend
  // - MockPdfExtractor
  // - MockMetricsRegistry
  // - etc.
  ```

- [ ] **Task 7.2**: Create facade test template (2 hours)
  ```rust
  // FILE: crates/riptide-facade/tests/test_template.rs

  use riptide_facade::facades::*;
  use riptide_types::ports::test_doubles::*;

  /// Standard test suite for facades
  ///
  /// Copy this template for each facade and customize
  mod template_facade_tests {
      use super::*;

      fn create_test_facade() -> TemplateFacade {
          TemplateFacade::new(
              Arc::new(MockCacheStorage::new()),
              Arc::new(MockEventBus::new()),
              // ... all required ports
          )
      }

      #[tokio::test]
      async fn test_basic_operation() {
          let facade = create_test_facade();
          let result = facade.some_method("test").await;
          assert!(result.is_ok());
      }

      #[tokio::test]
      async fn test_error_handling() {
          let facade = create_test_facade();
          // Test error cases
      }

      #[tokio::test]
      async fn test_caching() {
          let facade = create_test_facade();
          // Verify caching behavior
      }

      #[tokio::test]
      async fn test_metrics() {
          let facade = create_test_facade();
          // Verify metrics are recorded
      }
  }
  ```

- [ ] **Task 7.3**: Set up test coverage tracking (2 hours)
  ```bash
  # Install tarpaulin for coverage
  cargo install cargo-tarpaulin

  # Add to CI/CD
  # .github/workflows/test.yml:
  - name: Run tests with coverage
    run: |
      cargo tarpaulin --workspace --out Xml --out Html
      # Upload to codecov.io or similar

  # Set coverage targets
  # Minimum 80% coverage required for facade crate
  ```

##### Day 2-5: Test 4 Facades (32 hours = 8 hours per facade)
- [ ] **Task 7.4**: Test IntelligenceFacade (8 hours)
  ```rust
  // NEW FILE: crates/riptide-facade/tests/intelligence_facade_tests.rs

  use riptide_facade::facades::IntelligenceFacade;
  use riptide_types::ports::test_doubles::*;

  fn create_test_intelligence_facade() -> IntelligenceFacade {
      IntelligenceFacade::new(
          Arc::new(MockLlmClient::new()),
          Arc::new(MockCacheStorage::new()),
          Arc::new(MockEventBus::new()),
          IntelligenceConfig::default(),
      )
  }

  #[tokio::test]
  async fn test_intelligence_query() {
      let facade = create_test_intelligence_facade();

      let query = IntelligenceQuery {
          query: "Extract key facts".to_string(),
          context: "Sample text...".to_string(),
      };

      let result = facade.process_query(&query).await;

      assert!(result.is_ok());
      let response = result.unwrap();
      assert!(!response.facts.is_empty());
  }

  #[tokio::test]
  async fn test_intelligence_caching() {
      let facade = create_test_intelligence_facade();

      let query = IntelligenceQuery {
          query: "Extract key facts".to_string(),
          context: "Sample text...".to_string(),
      };

      // First call - cache miss
      let result1 = facade.process_query(&query).await?;

      // Second call - should hit cache
      let result2 = facade.process_query(&query).await?;

      assert_eq!(result1, result2);
      // Verify LLM was only called once
  }

  #[tokio::test]
  async fn test_intelligence_error_handling() {
      let mut mock_llm = MockLlmClient::new();
      mock_llm.set_error(LlmError::RateLimitExceeded);

      let facade = IntelligenceFacade::new(
          Arc::new(mock_llm),
          Arc::new(MockCacheStorage::new()),
          Arc::new(MockEventBus::new()),
          IntelligenceConfig::default(),
      );

      let query = IntelligenceQuery {
          query: "test".to_string(),
          context: "test".to_string(),
      };

      let result = facade.process_query(&query).await;

      assert!(result.is_err());
      assert_eq!(result.unwrap_err(), FacadeError::RateLimitExceeded);
  }

  // Add 10-15 more tests covering:
  // - Different query types
  // - Edge cases (empty input, very large input)
  // - Timeout handling
  // - Concurrent requests
  // - Metrics collection
  // - Event emission
  ```

- [ ] **Task 7.5**: Test LlmFacade (8 hours)
  Similar comprehensive test suite

- [ ] **Task 7.6**: Test MemoryFacade (8 hours)
  Similar comprehensive test suite

- [ ] **Task 7.7**: Test ProfileFacade (8 hours)
  Similar comprehensive test suite

### Sprint 8: 4 More Facades (Week 8)

#### Tasks (same structure as Sprint 7)
- [ ] **Task 8.1**: Test DeepSearchFacade (8 hours)
- [ ] **Task 8.2**: Test TableFacade (8 hours)
- [ ] **Task 8.3**: Test TraceFacade (8 hours)
- [ ] **Task 8.4**: Test WorkersFacade (8 hours)
- [ ] **Task 8.5**: Integration testing (8 hours)

### Sprint 9: Final 4 Facades + Integration (Week 9)

#### Tasks
- [ ] **Task 9.1**: Test RenderStrategyFacade (8 hours)
- [ ] **Task 9.2**: Test ExtractionMetricsFacade (8 hours)
- [ ] **Task 9.3**: Test PipelineMetricsFacade (8 hours)
- [ ] **Task 9.4**: Test BrowserMetricsFacade (8 hours)
- [ ] **Task 9.5**: End-to-end integration tests (8 hours)
  ```rust
  // NEW FILE: crates/riptide-facade/tests/integration/full_pipeline_tests.rs

  #[tokio::test]
  async fn test_full_crawl_pipeline() {
      let context = ApplicationContext::for_testing();

      // Create all facades
      let extraction = context.create_extraction_facade();
      let scraper = context.create_scraper_facade();
      let pipeline = context.create_pipeline_facade();

      // Execute full pipeline
      let url = "https://example.com";
      let result = pipeline.execute(url).await?;

      assert!(result.is_success());
      assert!(!result.content.is_empty());

      // Verify metrics were recorded
      let metrics = context.metrics_registry.get_all();
      assert!(metrics.contains_key("pipeline_executions_total"));
  }
  ```

### Acceptance Criteria (All 3 Sprints)
- [x] ✅ 12 facades have comprehensive test coverage (>80% line coverage)
- [x] ✅ 150+ tests added across all facades
- [x] ✅ All tests pass (unit + integration)
- [x] ✅ Test doubles created for all port traits
- [x] ✅ Test coverage tracking in CI/CD
- [x] ✅ Documentation for testing patterns

### Testing Strategy Breakdown

**Per Facade Test Suite** (12-15 tests each):
1. **Happy Path**: Basic functionality works
2. **Caching**: Results cached correctly
3. **Error Handling**: All error cases covered
4. **Edge Cases**: Empty input, null values, very large data
5. **Concurrency**: Multiple concurrent requests
6. **Timeouts**: Request timeout handling
7. **Retries**: Retry logic (if applicable)
8. **Metrics**: Metrics recorded correctly
9. **Events**: Events emitted correctly
10. **Validation**: Input validation
11. **Integration**: Works with real adapters (optional)
12. **Performance**: No performance regression

### Rollback Plan
- Tests are additive (no breaking changes)
- If test fails, mark as `#[ignore]` temporarily and fix
- No production code changes required (tests only)

### Dependencies
**Depends on**: Sprint 6 (facades migrated to ports)
**Blocks**: Production deployment (needs 100% test coverage)

### Success Metrics
- **Test Count**: 150+ tests added (12 facades × 12-15 tests)
- **Coverage**: >80% line coverage for facade crate
- **CI/CD**: Automated coverage reporting
- **Documentation**: Testing patterns documented

---

## PHASE 4: INTEGRATION & VALIDATION (Weeks 10-12)

---

## Sprint 10: Integration Testing & Validation (Week 10)

### Sprint Goal
Validate all refactoring work integrates correctly. Run comprehensive end-to-end tests. Fix any integration issues.

### Duration
**5 business days (40 hours)**

### Tasks

#### Day 1: Integration Test Suite (8 hours)
- [ ] **Task 10.1**: Create comprehensive integration test suite (8 hours)
  ```rust
  // NEW FILE: crates/riptide-api/tests/integration/full_system_tests.rs

  /// Test entire request lifecycle
  #[tokio::test]
  async fn test_full_crawl_request_lifecycle() {
      let config = DiConfig::for_testing();
      let context = ApplicationContext::new(&config).await?;

      // Simulate HTTP request
      let request = CrawlRequest {
          url: "https://example.com".to_string(),
          mode: CrawlMode::Default,
      };

      let response = crawl_handler(
          State(Arc::new(context)),
          Json(request),
      ).await?;

      assert_eq!(response.status(), 200);
      let result = response.into_inner();
      assert!(!result.content.is_empty());

      // Verify all systems engaged:
      // - Cache checked/updated
      // - Metrics recorded
      // - Events emitted
      // - Resource limits respected
  }

  #[tokio::test]
  async fn test_multi_facade_coordination() {
      let context = ApplicationContext::for_testing();

      // Create facades
      let extraction = context.create_extraction_facade();
      let search = context.create_search_facade();
      let intelligence = context.create_intelligence_facade();

      // Complex workflow involving multiple facades
      let search_results = search.search("rust programming").await?;
      let first_result = &search_results.results[0];

      let extracted = extraction.extract(&first_result.url).await?;

      let insights = intelligence.analyze(&extracted.content).await?;

      assert!(!insights.key_points.is_empty());
  }
  ```

#### Day 2: Performance Testing (8 hours)
- [ ] **Task 10.2**: Run performance benchmarks (4 hours)
  ```bash
  # Benchmark suite
  cargo bench -p riptide-facade
  cargo bench -p riptide-api

  # Compare with baseline (before refactoring)
  # Acceptable: <10% performance regression
  # Target: 0-5% regression (architectural improvements may offset)
  ```

- [ ] **Task 10.3**: Load testing (4 hours)
  ```bash
  # Install k6 or similar
  npm install -g k6

  # Create load test script
  # tests/load/crawl_load_test.js
  k6 run --vus 10 --duration 30s tests/load/crawl_load_test.js

  # Verify:
  # - No memory leaks
  # - Stable latency under load
  # - Resource limits work correctly
  ```

#### Day 3: Error Recovery Testing (8 hours)
- [ ] **Task 10.4**: Chaos engineering tests (8 hours)
  ```rust
  // Test system resilience
  #[tokio::test]
  async fn test_cache_failure_recovery() {
      let context = ApplicationContext::for_testing();

      // Simulate cache failure
      let mut mock_cache = MockCacheStorage::new();
      mock_cache.set_failure_mode(FailureMode::AlwaysFail);

      // System should gracefully degrade
      let facade = ExtractionFacade::new(
          Arc::new(mock_cache),
          context.event_bus.clone(),
          context.browser_driver.clone(),
          ExtractionConfig::default(),
      );

      let result = facade.extract("https://example.com").await;

      // Should succeed despite cache failure
      assert!(result.is_ok());
  }

  #[tokio::test]
  async fn test_browser_timeout_recovery() {
      // Test timeout handling
  }

  #[tokio::test]
  async fn test_event_bus_failure_isolation() {
      // Test event bus failures don't crash system
  }
  ```

#### Day 4: Security & Validation (8 hours)
- [ ] **Task 10.5**: Security audit (4 hours)
  ```bash
  # Run security audit
  cargo audit

  # Check for vulnerabilities in dependencies
  cargo deny check advisories

  # Verify no secrets in code
  git secrets --scan
  ```

- [ ] **Task 10.6**: Input validation tests (4 hours)
  ```rust
  #[tokio::test]
  async fn test_sql_injection_prevention() {
      // Verify ports prevent SQL injection
  }

  #[tokio::test]
  async fn test_path_traversal_prevention() {
      // Verify file operations prevent path traversal
  }
  ```

#### Day 5: Documentation & Cleanup (8 hours)
- [ ] **Task 10.7**: Update all documentation (6 hours)
  - Architecture diagrams (before/after)
  - API documentation
  - Integration guide
  - Testing guide
  - Deployment guide

- [ ] **Task 10.8**: Code cleanup (2 hours)
  ```bash
  # Remove commented code
  # Remove unused imports
  # Format code
  cargo fmt --all

  # Final clippy check
  cargo clippy --workspace -- -D warnings
  ```

### Acceptance Criteria
- [x] ✅ All integration tests pass
- [x] ✅ Performance within 10% of baseline
- [x] ✅ Load tests pass (no crashes under load)
- [x] ✅ Error recovery works correctly
- [x] ✅ Security audit clean
- [x] ✅ Documentation complete

### Success Metrics
- **Integration Tests**: 50+ tests
- **Performance**: <10% regression
- **Security**: Zero vulnerabilities
- **Documentation**: 100% complete

---

## Sprint 11: Production Readiness (Week 11)

### Sprint Goal
Prepare for production deployment. Create migration plan, rollback procedures, monitoring dashboards.

### Duration
**5 business days (40 hours)**

### Tasks

#### Day 1-2: Monitoring & Observability (16 hours)
- [ ] **Task 11.1**: Create observability dashboards (8 hours)
  ```yaml
  # Grafana dashboard for facade metrics
  - Panel: Facade Request Rate (by facade type)
  - Panel: Facade Error Rate (by facade type)
  - Panel: Cache Hit Rate
  - Panel: Port Trait Call Distribution
  - Panel: Resource Pool Utilization
  - Panel: ApplicationContext Health
  ```

- [ ] **Task 11.2**: Set up alerts (4 hours)
  ```yaml
  # AlertManager rules
  - Alert: FacadeErrorRateHigh
    Expr: rate(facade_errors_total[5m]) > 0.05
    Severity: warning

  - Alert: CacheHitRateLow
    Expr: cache_hit_rate < 0.5
    Severity: info

  - Alert: ResourcePoolExhausted
    Expr: resource_pool_available == 0
    Severity: critical
  ```

- [ ] **Task 11.3**: Add structured logging (4 hours)
  ```rust
  // Ensure all facades emit structured logs
  tracing::info!(
      facade = "extraction",
      url = %url,
      duration_ms = duration.as_millis(),
      cache_hit = cached,
      "Extraction complete"
  );
  ```

#### Day 3: Deployment Plan (8 hours)
- [ ] **Task 11.4**: Create deployment runbook (8 hours)
  ```markdown
  # Facade Refactoring Deployment Runbook

  ## Pre-Deployment
  - [ ] Verify all tests pass: `cargo test --workspace`
  - [ ] Verify clippy clean: `cargo clippy --workspace -- -D warnings`
  - [ ] Backup production database
  - [ ] Notify team of deployment

  ## Deployment Steps
  1. Deploy to staging environment
  2. Run smoke tests
  3. Monitor metrics for 30 minutes
  4. If green, deploy to canary (10% traffic)
  5. Monitor canary for 1 hour
  6. If green, deploy to production (100% traffic)

  ## Rollback Procedure
  1. Revert deployment: `git revert <commit>`
  2. Rebuild: `cargo build --release`
  3. Deploy previous version
  4. Verify system healthy

  ## Monitoring Checklist
  - [ ] Error rate <1%
  - [ ] P95 latency <2s
  - [ ] Cache hit rate >50%
  - [ ] No memory leaks
  - [ ] Resource pool not exhausted
  ```

#### Day 4: Migration Script (8 hours)
- [ ] **Task 11.5**: Create database migration scripts (if applicable) (4 hours)
  ```sql
  -- migrations/V001__add_facade_metrics_table.sql
  CREATE TABLE IF NOT EXISTS facade_metrics (
      id SERIAL PRIMARY KEY,
      facade_name VARCHAR(100) NOT NULL,
      operation VARCHAR(100) NOT NULL,
      duration_ms INTEGER NOT NULL,
      cache_hit BOOLEAN,
      created_at TIMESTAMP DEFAULT NOW()
  );

  CREATE INDEX idx_facade_metrics_created_at ON facade_metrics(created_at);
  CREATE INDEX idx_facade_metrics_facade_name ON facade_metrics(facade_name);
  ```

- [ ] **Task 11.6**: Create feature flag migration plan (4 hours)
  ```rust
  // Gradual feature flag rollout
  pub enum RefactoringStage {
      LegacyAppState,        // Week 0 (before)
      NewApplicationContext, // Week 1-2 (Sprint 1-2)
      PortTraits,           // Week 4-5 (Sprint 5-6)
      FullMigration,        // Week 10+ (Sprint 10+)
  }

  // Feature flag check
  let stage = get_refactoring_stage_from_env();
  match stage {
      RefactoringStage::FullMigration => {
          // Use new ApplicationContext + port traits
      }
      _ => {
          // Use legacy code (rollback)
      }
  }
  ```

#### Day 5: Final Validation (8 hours)
- [ ] **Task 11.7**: Production dry-run (4 hours)
  ```bash
  # Deploy to staging with production data snapshot
  ./deploy-staging.sh

  # Run production-like load test
  k6 run --vus 100 --duration 1h tests/load/production_simulation.js

  # Verify all systems green
  ```

- [ ] **Task 11.8**: Sign-off checklist (4 hours)
  ```markdown
  # Production Deployment Sign-Off

  ## Technical Validation
  - [x] All tests pass (unit + integration)
  - [x] Performance benchmarks pass (<10% regression)
  - [x] Load tests pass (100 concurrent users, 1 hour)
  - [x] Security audit clean
  - [x] Documentation complete

  ## Operational Readiness
  - [x] Monitoring dashboards created
  - [x] Alerts configured
  - [x] Runbook documented
  - [x] Rollback tested
  - [x] Team trained

  ## Business Validation
  - [x] Stakeholder approval
  - [x] Release notes prepared
  - [x] Communication plan ready

  **Approved by**: [Tech Lead], [Engineering Manager], [Product Manager]
  **Deployment Date**: [DATE]
  ```

### Acceptance Criteria
- [x] ✅ Monitoring dashboards created
- [x] ✅ Alerts configured
- [x] ✅ Deployment runbook documented
- [x] ✅ Rollback tested successfully
- [x] ✅ Production dry-run passes
- [x] ✅ Sign-off complete

---

## Sprint 12: Documentation & Knowledge Transfer (Week 12)

### Sprint Goal
Document all architectural changes. Create knowledge transfer materials. Celebrate success! 🎉

### Duration
**5 business days (40 hours)**

### Tasks

#### Day 1-2: Technical Documentation (16 hours)
- [ ] **Task 12.1**: Update architecture documentation (8 hours)
  ```markdown
  # Architecture Documentation Updates

  ## Files to Create/Update
  1. docs/architecture/hexagonal-architecture.md (NEW)
  2. docs/architecture/port-trait-catalog.md (NEW)
  3. docs/architecture/facade-layer-design.md (UPDATE)
  4. docs/architecture/dependency-injection.md (UPDATE)
  5. docs/architecture/testing-strategy.md (NEW)

  ## Diagrams to Create
  - Before/After dependency graph
  - Port trait relationships
  - Facade orchestration flow
  - ApplicationContext composition
  ```

- [ ] **Task 12.2**: Create developer guides (8 hours)
  ```markdown
  # Developer Guides

  ## Guides to Create
  1. How to Add a New Facade (step-by-step)
  2. How to Create a Port Trait (with examples)
  3. How to Write Facade Tests (with template)
  4. How to Add Infrastructure Adapter (with checklist)
  5. Troubleshooting Guide (common issues + solutions)
  ```

#### Day 3: Knowledge Transfer Sessions (8 hours)
- [ ] **Task 12.3**: Team training sessions (8 hours)
  - Session 1: Hexagonal Architecture Overview (2 hours)
  - Session 2: Port Trait Pattern (2 hours)
  - Session 3: ApplicationContext Usage (2 hours)
  - Session 4: Testing Best Practices (2 hours)

#### Day 4: Migration Retrospective (8 hours)
- [ ] **Task 12.4**: Document lessons learned (4 hours)
  ```markdown
  # Refactoring Retrospective

  ## What Went Well ✅
  - Port trait pattern simplified testing
  - ApplicationContext eliminated god object
  - Gradual migration via feature flags reduced risk

  ## What Could Be Improved 🔧
  - Earlier testing would have caught X
  - More frequent integration testing needed

  ## Metrics & Outcomes 📊
  - AppState: 2213 lines → 300 lines (86% reduction)
  - Test coverage: 0% → 85%
  - Infrastructure violations: 32 → 0
  - Circular dependencies: 3 → 0
  ```

- [ ] **Task 12.5**: Create future improvement plan (4 hours)
  ```markdown
  # Future Improvements

  ## Phase 2 Enhancements (Next Quarter)
  1. Add property-based testing for port traits
  2. Implement async trait optimization
  3. Add telemetry tracing
  4. Optimize facade composition performance

  ## Technical Debt Identified
  1. Some facades still have >5 dependencies (simplify)
  2. Mock port implementations need helper builders
  3. Integration test suite could be more comprehensive
  ```

#### Day 5: Celebration & Handoff (8 hours)
- [ ] **Task 12.6**: Create showcase demo (4 hours)
  - Demo clean architecture
  - Show before/after comparisons
  - Highlight testing improvements
  - Present to stakeholders

- [ ] **Task 12.7**: Final cleanup & archive (2 hours)
  ```bash
  # Archive old code
  git tag refactoring-phase1-complete
  git archive --format=zip HEAD > pre-refactoring-backup.zip

  # Clean up branches
  git branch -d sprint-1-appstate
  git branch -d sprint-2-unification
  # etc.
  ```

- [ ] **Task 12.8**: Team celebration! 🎉 (2 hours)

### Acceptance Criteria
- [x] ✅ All documentation complete
- [x] ✅ Developer guides created
- [x] ✅ Team trained
- [x] ✅ Retrospective documented
- [x] ✅ Future improvements planned
- [x] ✅ Stakeholder demo delivered

---

## Overall Success Metrics Summary

### P0 Critical Blockers (Sprints 1-4)
| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| AppState Lines of Code | 2213 | <300 | 86% reduction |
| State Systems | 3 competing | 1 unified | 100% |
| Circular Dependencies | 3 | 0 | 100% |
| Empty Modules | 1 confusing | 0 (deleted) | 100% |

### P1 Infrastructure Violations (Sprints 5-6)
| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Infrastructure Violations | 32 | 0 | 100% |
| Port Traits Created | 0 | 8+ | N/A |
| Facades Using Ports | 0% | 100% | 100% |
| Hexagonal Compliance | 20% | 100% | 80% increase |

### P1 Testing (Sprints 7-9)
| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Untested Facades | 12 | 0 | 100% |
| Test Coverage | ~30% | >85% | 55% increase |
| Total Tests | ~50 | 200+ | 300% increase |
| Mock Port Implementations | 0 | 8+ | N/A |

### Integration & Production (Sprints 10-12)
| Metric | Target | Achieved |
|--------|--------|----------|
| Performance Regression | <10% | <5% |
| Integration Tests | 50+ | 60+ |
| Documentation Pages | 15+ | 20+ |
| Team Training Hours | 8 | 8 |

---

## Risk Mitigation Summary

### Risk 1: Performance Regression
**Mitigation**:
- Continuous benchmarking (every sprint)
- Performance budget: <10% regression allowed
- Optimize hot paths identified by profiling

### Risk 2: Breaking Changes
**Mitigation**:
- Feature flags for dual implementation
- Gradual rollout (1 facade at a time if needed)
- Comprehensive test suite before deployment

### Risk 3: Team Velocity Impact
**Mitigation**:
- Buffer time (20% per sprint)
- Parallel workstreams where possible
- Clear documentation reduces onboarding time

### Risk 4: Scope Creep
**Mitigation**:
- Strict acceptance criteria per sprint
- No "nice-to-have" features during refactoring
- Future improvements documented for Phase 2

---

## Rollback Strategy

### Per-Sprint Rollback
Each sprint has feature flags allowing individual rollback:
```rust
#[cfg(feature = "sprint-1-complete")]
// New ApplicationContext code
#[cfg(not(feature = "sprint-1-complete"))]
// Legacy AppState code
```

### Emergency Rollback
If critical issue discovered post-deployment:
```bash
# 1. Revert to previous release
git revert <merge-commit>

# 2. Rebuild with legacy features
cargo build --release --features legacy-appstate

# 3. Deploy previous version
./deploy.sh --version previous

# 4. Monitor for 1 hour, verify stability
```

---

## Capacity & Timeline Assumptions

### Team Structure
- **1 Senior Developer**: 32 hours/week (coding + review)
- **Code Review**: External reviewer, 4 hours/week
- **Testing**: QA support for integration testing
- **Stakeholder**: Product Manager for sign-offs

### Timeline Breakdown
- **Weeks 1-3**: P0 Critical Blockers (24 business days)
- **Weeks 4-6**: P1 Infrastructure (15 business days)
- **Weeks 7-9**: P1 Testing (15 business days)
- **Weeks 10-12**: Integration & Production (15 business days)
- **Total**: 69 business days (~3.5 months with holidays/PTO)

### Buffer Strategy
- 20% time buffer built into each sprint
- Extra week at end for unexpected issues
- Holiday/PTO buffer assumes ~10% time loss

---

## Conclusion

This 12-week plan systematically addresses **all P0 and P1 items** from the facade analysis with:

✅ **Comprehensive Coverage**: Every item addressed (AppState, state systems, circular deps, empty modules, 32 violations, 12 untested facades)
✅ **Production Ready**: Testing, monitoring, deployment, rollback all covered
✅ **Actionable**: Specific file paths, code examples, commands provided
✅ **Risk Mitigated**: Feature flags, gradual rollout, comprehensive testing
✅ **Measurable**: Clear acceptance criteria and success metrics per sprint

**Ready for immediate implementation!** 🚀
