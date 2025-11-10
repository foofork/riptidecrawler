Note to Analyst / Architect Review

Subject: Riptide Alpha Refactor Roadmap — Final Architecture Validation

Team,

Attached is the finalized Riptide Alpha Refactor Roadmap (8-Week Plan).
This plan compresses the remaining critical architectural work needed to move Riptide from its current alpha state to a clean, testable, and reliable system.

Before execution, please perform a technical validation review focused on identifying any critical missing elements required for the system to actually function end-to-end after this refactor.

Specifically, please verify:

Runtime Completeness – confirm all essential paths (search, extraction, PDF, browser, event bus, persistence) will still function after AppState is replaced with ApplicationContext.

Ports/Adapters Coverage – ensure no external system (Redis, browser, fetch, events, metrics) remains unabstracted or missing adapter wiring.

Handler Path Validation – confirm all top-10 routes have enough coverage to actually execute after the migration (no unresolved dependency chains or missing facades).

Resilience & Reliability – validate timeouts, circuit breaker, and idempotency hooks are correctly positioned to prevent failure cascades.

Observability Minimums – confirm metrics and structured logs will provide sufficient visibility for debugging in alpha without extra tooling.

Feature-Flag Safety – ensure legacy/new-context dual-mode remains compile-safe and deployable at any point during migration.

Test Path Coverage – confirm the E2E and facade tests outlined are sufficient to catch critical regressions before release.

If any runtime, initialization, or dependency wiring issues would still prevent the system from running end-to-end after this plan, please flag them immediately so we can amend before sprint kickoff.

Goal: No surprises at runtime after this refactor.
This is the final pass to guarantee architectural soundness before moving to beta-readiness work.

Thank you for doing a full technical validation pass on this — please annotate directly in the roadmap doc or return a concise summary of findings.

— [Your Name]
Riptide Lead / Architecture Coordination


# 🚀 Riptide Alpha Refactor Roadmap (8-Week Plan)

**Project:** Riptide Event Mesh
**Stage:** Alpha (pre-production)
**Timeline:** 8 weeks (compressed from 12-week plan)
**Priority:** P0 (Critical) and P1 (High Priority)
**Goal:** Achieve a working, testable, and reliable alpha system with a clean hexagonal architecture and unified state management.

---

## Executive Summary

This roadmap delivers an **alpha-ready**, testable version of Riptide by focusing only on high-impact architectural work needed for the system to function end-to-end:

* **P0:** Eliminate the `AppState` god object and unify all infrastructure access behind clean port traits in `ApplicationContext`.
* **P1:** Implement minimal adapters and refactor the top 10 API handlers/facades to use the new architecture.
* **P1:** Add resilience (timeouts, circuit breakers, idempotency) and essential observability.

**Success Metrics**

* AppState reduced from 2213 → <300 lines
* Top 10 handlers fully migrated to clean facade usage
* 100% of external dependencies accessed via ports
* End-to-end tests passing for key routes
* Feature-flagged dual-mode builds (`legacy` vs. `new-context`)

---

## Week 0: Pre-Sprint Setup

### Prerequisites

* [ ] Create `git tag alpha-pre-refactor-backup`
* [ ] Verify baseline tests: `cargo test --workspace`
* [ ] Measure initial AppState metrics (lines, dependencies)
* [ ] Add feature flags for dual implementation (`legacy-appstate`, `new-context`)
* [ ] Create migration tracking spreadsheet (`docs/migration-tracking.xlsx`)

### Team Assumptions

* **1 Senior Developer** – 32 hrs/week (80% coding, 20% review)
* **Code Review:** 4 hrs/week
* **Testing:** 25% dev time
* **Buffer:** 20% for unknowns

---

# PHASE 1 — FOUNDATIONS (Weeks 1–2)

Refactor core state management and introduce minimal port traits + adapters.

---

## Sprint 1: ApplicationContext & Feature Flags (Week 1)

### Sprint Goal

Migrate infrastructure fields from `AppState` to a new `ApplicationContext` while keeping legacy support via feature flags.

### Duration

**5 business days (40 hrs)**

### Tasks

#### Day 1: Setup & Field Inventory (8 hrs)

* [ ] **Task 1.1:** Audit all `AppState` fields in
  `/workspaces/eventmesh/crates/riptide-api/src/state.rs` (lines 60–200)
  Categorize each as:

  * Core Infrastructure (CI)
  * Business Facade (BF)
  * Metrics (M)
  * Configuration (C)

  Output: `docs/appstate-field-inventory.md`

* [ ] **Task 1.2:** Add dual-feature configuration

  ```toml
  [features]
  default = ["legacy-appstate"]
  legacy-appstate = []
  new-context = []
  ```

#### Day 2–3: ApplicationContext Definition & Migration (16 hrs)

* [ ] **Task 1.3:** Create `ApplicationContext` at
  `/workspaces/eventmesh/crates/riptide-api/src/composition/mod.rs`

  ```rust
  pub struct ApplicationContext {
      pub cache_storage: Arc<dyn CacheStorage>,
      pub event_bus: Arc<dyn EventBus>,
      pub idempotency_store: Arc<dyn IdempotencyStore>,
      pub session_store: Arc<dyn SessionStorage>,
  }
  ```

* [ ] **Task 1.4:** Create adapter for CacheManager → CacheStorage
  `/workspaces/eventmesh/crates/riptide-cache/src/adapters/cache_storage_adapter.rs`

  ```rust
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

      async fn set(&self, key: &str, val: Vec<u8>, ttl: Duration) -> Result<()> {
          let mut cache = self.inner.lock().await;
          cache.set_simple(key, &val, ttl.as_secs()).await
      }
  }
  ```

* [ ] **Task 1.5:** Update handlers to use context:

  ```rust
  // BEFORE
  pub async fn crawl_handler(State(app_state): State<Arc<AppState>>) -> Result<Json<CrawlResult>> {
      app_state.cache.lock().await.get("key").await?;
  }

  // AFTER
  pub async fn crawl_handler(State(ctx): State<Arc<ApplicationContext>>) -> Result<Json<CrawlResult>> {
      ctx.cache_storage.get("key").await?;
  }
  ```

#### Day 4–5: Testing & Validation (16 hrs)

* [ ] Add migration tests:
  `/workspaces/eventmesh/crates/riptide-api/src/tests/appstate_migration_tests.rs`

  ```rust
  #[tokio::test]
  async fn test_cache_access_via_context() {
      let ctx = ApplicationContext::for_testing();
      ctx.cache_storage.set("alpha", b"value".to_vec(), Duration::from_secs(60)).await?;
      let v = ctx.cache_storage.get("alpha").await?;
      assert_eq!(v, Some(b"value".to_vec()));
  }
  ```

* [ ] Regression checks:

  ```bash
  cargo test -p riptide-api --features legacy-appstate
  cargo test -p riptide-api --features new-context
  ```

### Acceptance Criteria

* ✅ ApplicationContext defined and integrated
* ✅ 8+ core infrastructure fields migrated
* ✅ 2 new port traits: `CacheStorage`, `SessionStorage`
* ✅ Tests pass in both feature modes
* ✅ Docs updated

---

## Sprint 2: Minimal Ports & Adapter Layer (Week 2)

### Sprint Goal

Introduce the minimum required ports and adapters to support the top 10 routes.

### Duration

**5 business days (40 hrs)**

### Tasks

#### Day 1: Define Port Traits (8 hrs)

* [ ] Create/verify the following in
  `/workspaces/eventmesh/crates/riptide-types/src/ports/`:

  * `cache.rs` → `CacheStorage`
  * `http.rs` → `HttpClient`
  * `browser.rs` → `BrowserDriver`
  * `events.rs` → `EventBus`
  * `metrics.rs` → `MetricsRegistry`
  * `circuit_breaker.rs` → `CircuitBreaker`

#### Day 2–3: Create Adapters (16 hrs)

Example: BrowserDriver
`/workspaces/eventmesh/crates/riptide-headless/src/adapters/browser_driver_adapter.rs`

```rust
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
}
```

#### Day 4: Wire into ApplicationContext (8 hrs)

```rust
impl ApplicationContext {
    pub async fn new(config: &DiConfig) -> Result<Self> {
        Ok(Self {
            cache_storage: Arc::new(CacheManagerAdapter::new()),
            http_client: Arc::new(FetchClientAdapter::new()),
            browser_driver: Arc::new(HeadlessBrowserDriver::new()),
            event_bus: Arc::new(LocalEventBus::new()),
            metrics_registry: Arc::new(PrometheusMetrics::new()),
            circuit_breaker: Arc::new(SimpleCircuitBreaker::new()),
        })
    }
}
```

#### Day 5: Test Adapters (8 hrs)

* [ ] Unit tests for each adapter (happy + failure case)
* [ ] Add fake port implementations in
  `/workspaces/eventmesh/crates/riptide-types/src/ports/test_doubles.rs`

### Acceptance Criteria

* ✅ 6 minimal port traits exist
* ✅ 6 adapters implemented
* ✅ ApplicationContext compiles and wires them correctly
* ✅ All adapter tests pass

---

# PHASE 2 — TOP-LEVEL REFACTOR (Weeks 3–5)

Focus on handlers, facades, and core functionality for top 10 routes.

---

## Sprint 3: Handler Simplification & Facade Boundaries (Week 3)

### Sprint Goal

Slim down the top 10 API handlers and migrate them to use facade orchestration via clean ports.

### Duration

**5 business days (40 hrs)**

### Tasks

#### Day 1–3: Refactor Handlers (24 hrs)

```rust
// BEFORE
pub async fn crawl_handler(State(app_state): State<Arc<AppState>>, Json(req): Json<CrawlRequest>) -> Result<Json<CrawlResult>> {
    let cache = app_state.cache.lock().await;
    let _ = cache.get(&req.url).await?;
    let result = app_state.extraction_facade.extract(&req.url).await?;
    Ok(Json(result))
}

// AFTER
pub async fn crawl_handler(State(ctx): State<Arc<ApplicationContext>>, Json(req): Json<CrawlRequest>) -> Result<Json<CrawlResult>> {
    ctx.metrics_registry.increment("crawl_requests_total");
    let result = ctx.create_extraction_facade().extract(&req.url).await?;
    Ok(Json(result))
}
```

* [ ] Refactor all 10 handlers under `/riptide-api/src/handlers/`
* [ ] Each under 50 lines, no concrete infra calls
* [ ] Moved orchestration into facades
* [ ] Standardize error mapping (domain → ApiError)

#### Day 4–5: Introduce Reliability Layer (16 hrs)

Add shared timeout, circuit breaker, and idempotency behavior.

```rust
let result = ctx.circuit_breaker.run(async {
    ctx.http_client.get(&url).await
}).await?;

ctx.idempotency_store.save(&request_id, &result)?;
```

### Acceptance Criteria

* ✅ 10 handlers refactored and compile under `new-context`
* ✅ Circuit breaker + timeouts implemented
* ✅ Consistent error mapping confirmed
* ✅ Handlers <50 LOC and covered by existing tests

---

## Sprint 4: End-to-End Validation (Week 4)

### Sprint Goal

Establish baseline reliability and E2E confidence across all top routes.

### Duration

**5 business days (40 hrs)**

### Tasks

#### Day 1–2: Integration Tests (16 hrs)

`/workspaces/eventmesh/crates/riptide-api/tests/e2e_tests.rs`

```rust
#[tokio::test]
async fn test_crawl_route_e2e() {
    let ctx = ApplicationContext::for_testing();
    let req = CrawlRequest { url: "https://example.com".into(), mode: CrawlMode::Default };
    let res = crawl_handler(State(Arc::new(ctx)), Json(req)).await?;
    assert!(res.status().is_success());
}
```

#### Day 3–4: Resilience Testing (16 hrs)

```rust
#[tokio::test]
async fn test_circuit_breaker_opens() {
    let breaker = SimpleCircuitBreaker::with_threshold(2);
    breaker.fail();
    breaker.fail();
    assert!(breaker.is_open());
}
```

#### Day 5: Documentation & Review (8 hrs)

* [ ] Update architecture diagrams (AppState → ApplicationContext)
* [ ] Record current E2E test coverage baseline

### Acceptance Criteria

* ✅ All 10 routes pass E2E
* ✅ Circuit breaker degradation verified
* ✅ Documentation updated

---

# PHASE 3 — CLEANUP & COMPLETION (Weeks 6–8)

Finish migration, remove legacy dependencies, and add observability + stability.

---

## Sprint 5: Port Sweep & Facade Finalization (Week 6)

### Sprint Goal

Remove remaining infrastructure dependencies from facades; migrate all to port-based architecture.

### Duration

**5 business days (40 hrs)**

### Tasks

#### Day 1: Audit Facades (8 hrs)

```bash
grep -r "use riptide_" crates/riptide-facade/src/facades/ | grep -v "riptide_types"
```

Document results in `docs/infrastructure-violations.md`.

#### Day 2–3: Refactor Facades (16 hrs)

Example: ExtractionFacade

```rust
// BEFORE
use riptide_headless::HeadlessLauncher;
pub struct ExtractionFacade {
    browser: Arc<HeadlessLauncher>,
    cache: Arc<CacheManager>,
}

// AFTER
use riptide_types::ports::{BrowserDriver, CacheStorage, EventBus};
pub struct ExtractionFacade {
    browser: Arc<dyn BrowserDriver>,
    cache: Arc<dyn CacheStorage>,
    events: Arc<dyn EventBus>,
}
```

#### Day 4: Facade Factories in Context (8 hrs)

```rust
impl ApplicationContext {
    pub fn create_extraction_facade(&self) -> Arc<ExtractionFacade> {
        Arc::new(ExtractionFacade::new(
            self.browser_driver.clone(),
            self.cache_storage.clone(),
            self.event_bus.clone(),
        ))
    }
}
```

#### Day 5: Validation Tests (8 hrs)

* Compile-time dependency isolation test:

  ```rust
  use riptide_facade::facades::ExtractionFacade;
  use riptide_types::ports::*;
  ```

  Must compile **without** importing `riptide-api`.

### Acceptance Criteria

* ✅ All facades depend only on port traits
* ✅ Zero infrastructure violations
* ✅ All handlers use factory-created facades
* ✅ Tests pass for facades and handlers

---

## Sprint 6: Targeted Test Coverage (Week 7)

### Sprint Goal

Add practical unit and integration tests for migrated components (not full 100% coverage yet).

### Duration

**5 business days (40 hrs)**

### Tasks

* [ ] Add tests for 10 migrated facades (happy path, error path, cache hit/miss)
* [ ] Add adapter failure simulations (`MockCacheStorage`, `MockHttpClient`)
* [ ] Dual-feature CI test run (`legacy` + `new-context`)
* [ ] Verify no regressions introduced in `cargo clippy -- -D warnings`

### Acceptance Criteria

* ✅ Facade tests added for all top routes
* ✅ Dual-mode CI build passes
* ✅ Clippy and cargo test clean

---

## Sprint 7: Observability & Runbook (Week 8)

### Sprint Goal

Add basic metrics, structured logging, and a simple alpha runbook.

### Duration

**5 business days (40 hrs)**

### Tasks

#### Day 1–2: Structured Logging

```rust
tracing::info!(
    facade = "extraction",
    url = %url,
    duration_ms = duration.as_millis(),
    cache_hit = hit,
    "Extraction complete"
);
```

#### Day 3: Basic Metrics

* [ ] Add metrics for `requests_total`, `error_total`, `cache_hit_ratio`
* [ ] Expose via `MetricsRegistry` port

#### Day 4: Runbook

Create `docs/operations/alpha-runbook.md`:

```markdown
## Flip Features
cargo run --features new-context
## Rollback
cargo run --features legacy-appstate
## Verify Health
curl /metrics
```

#### Day 5: Cleanup & Review

* [ ] Confirm legacy path works (for rollback)
* [ ] Archive `state.rs` (final removal after beta)

### Acceptance Criteria

* ✅ Structured logs emitted for all routes
* ✅ Metrics accessible via port layer
* ✅ Runbook documented
* ✅ Dual-feature flip tested manually

---

# ✅ Alpha Completion Summary

### Deliverables

| Area                          | Result                                                       |
| ----------------------------- | ------------------------------------------------------------ |
| AppState → ApplicationContext | ✅ Unified                                                    |
| Handlers (Top 10)             | ✅ <50 LOC, facade-driven                                     |
| Facades                       | ✅ Port-based, no infra deps                                  |
| Ports & Adapters              | ✅ 6 minimal (cache, http, browser, events, metrics, breaker) |
| E2E Tests                     | ✅ Passing under both flags                                   |
| Observability                 | ✅ Basic metrics + structured logs                            |
| Reliability                   | ✅ Timeouts, circuit breaker, idempotency, rate limits        |
| Docs                          | ✅ Updated + runbook complete                                 |

### Architecture Diagram

```
┌─────────────────┐
│ riptide-types   │ ← Domain layer (Ports & Types)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ riptide-facade  │ ← Application layer (Use cases)
│  (Facades)      │ ✅ depends only on ports
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ riptide-api     │ ← Interface layer (Handlers, Context)
│ (DI, HTTP)      │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Infrastructure  │ ← Adapters (Redis, Browser, HTTP)
└─────────────────┘
```

---

# Rollback Plan

```bash
# Revert to legacy mode
cargo build --features legacy-appstate

# Undo new-context migration if broken
git revert <migration-commit>

# Restore original state.rs
git checkout pre-refactor-backup -- crates/riptide-api/src/state.rs
```

---

# Success Metrics

| Metric                     | Before  | After                | Improvement   |
| -------------------------- | ------- | -------------------- | ------------- |
| AppState LOC               | 2213    | <300                 | 86% reduction |
| Top Routes (cleaned)       | 0       | 10                   | ✅             |
| Infra Violations           | 32      | 0                    | ✅             |
| Facades Using Ports        | 0%      | 100%                 | ✅             |
| E2E Tests Passing          | partial | full                 | ✅             |
| Circuit Breaker & Timeouts | none    | implemented          | ✅             |
| Observability              | none    | basic logs + metrics | ✅             |

---

# Future Phase (Post-Alpha)

1. Expand ports/adapters to full system coverage
2. Add full test suite (100+ cases)
3. Build CI coverage tracking + load tests
4. Extend observability (Grafana/alerts)
5. Remove legacy flag after beta validation

---

**Result:**
Riptide Alpha is now **functional, reliable, and testable** — ready to
