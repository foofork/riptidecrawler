# Hexagonal Architecture Migration Strategy

**Phase**: 1-2 Migration Planning
**Date**: 2025-11-11
**Status**: Complete
**Architect**: System Architecture Designer

---

## Executive Summary

This document outlines the phase-by-phase migration strategy to transition Riptide from its current architecture to a complete hexagonal (ports & adapters) architecture. The strategy emphasizes incremental changes, zero downtime, and comprehensive testing at each phase.

## Migration Principles

1. **Incremental Migration**: Small, verifiable steps with working code at each checkpoint
2. **Zero Breaking Changes**: Maintain API compatibility throughout
3. **Test-Driven**: Write tests before refactoring, verify after
4. **Rollback-Safe**: Each phase can be reverted independently
5. **Parallel Work**: Identify parallelizable tasks for efficiency

---

## Migration Phases

### Phase 2.1: Implement CircuitBreaker Port (CRITICAL PATH)

**Duration**: 2-3 hours
**Parallelizable**: No (blocks facade refactoring)
**Risk**: Low

**Tasks**:
1. ✅ Create `crates/riptide-types/src/ports/reliability.rs`
2. ✅ Define `CircuitBreaker` trait with async methods
3. ✅ Define `CircuitState` enum (Closed, Open, HalfOpen)
4. ✅ Define `CircuitPermit` RAII guard
5. ✅ Define `CircuitMetrics` struct
6. ✅ Define `CircuitBreakerConfig` struct
7. ✅ Export from `ports/mod.rs`
8. ✅ Run `cargo clippy -p riptide-types -- -D warnings`
9. ✅ Run `cargo test -p riptide-types`

**Verification**:
```bash
cargo check -p riptide-types
cargo test -p riptide-types::ports::reliability
```

**Deliverable**: CircuitBreaker port trait ready for implementation

---

### Phase 2.2: Create Adapter Implementations

**Duration**: 4-6 hours
**Parallelizable**: Yes (3 parallel tasks)
**Risk**: Low-Medium

#### Task 2.2a: Atomic CircuitBreaker Adapter (Parallel 1)

**Location**: `crates/riptide-utils/src/circuit_breaker.rs` (already exists, verify interface)

**Tasks**:
1. Verify `AtomicCircuitBreaker` implements new `CircuitBreaker` trait
2. Add missing methods if any
3. Ensure thread-safe atomic operations
4. Add unit tests for state transitions
5. Run clippy and tests

**Verification**:
```bash
cargo test -p riptide-utils circuit_breaker
```

#### Task 2.2b: StateBased CircuitBreaker Adapter (Parallel 2)

**Location**: `crates/riptide-reliability/src/circuit_breaker_pool.rs`

**Tasks**:
1. Implement `CircuitBreaker` trait for `CircuitBreakerState`
2. Add event bus integration
3. Add detailed metrics collection
4. Add unit tests with event verification
5. Run clippy and tests

**Verification**:
```bash
cargo test -p riptide-reliability circuit_breaker
```

#### Task 2.2c: InMemory CircuitBreaker Stub (Parallel 3)

**Location**: `crates/riptide-api/src/composition/stubs.rs`

**Tasks**:
1. Create `AlwaysClosedCircuitBreaker` (always allows requests)
2. Create `ManualCircuitBreaker` (externally controlled state)
3. Add to testing composition
4. Add unit tests
5. Run clippy and tests

**Verification**:
```bash
cargo test -p riptide-api composition::stubs
```

---

### Phase 2.3: Update ApplicationContext Structure

**Duration**: 2-3 hours
**Parallelizable**: No (depends on 2.1, 2.2)
**Risk**: Medium

**Tasks**:
1. ✅ Update `ApplicationContext` struct with all new ports:
   - `cache: Arc<dyn CacheStorage>` (already exists)
   - `headless_circuit_breaker: Arc<dyn CircuitBreaker>` (new)
   - `llm_circuit_breaker: Arc<dyn CircuitBreaker>` (new, feature-gated)
   - `rate_limiter: Arc<dyn RateLimiter>` (already exists)
   - `health_registry: Arc<dyn HealthRegistry>` (already exists)
   - `business_metrics: Arc<dyn BusinessMetrics>` (already exists)
   - `wasm_pool: Arc<dyn Pool<WasmInstance>>` (already exists)
   - `browser_pool: Arc<dyn Pool<BrowserSession>>` (already exists)

2. ✅ Update `ApplicationContext::new()` for production wiring
3. ✅ Update `ApplicationContext::for_testing()` with stubs
4. ✅ Update `ApplicationContextBuilder` with new ports
5. ✅ Run clippy and tests

**Verification**:
```bash
cargo check -p riptide-api
cargo test -p riptide-api composition
```

**Rollback**: Revert ApplicationContext changes, facades still work with old structure

---

### Phase 2.4: Create Facade Factory Pattern

**Duration**: 3-4 hours
**Parallelizable**: No (depends on 2.3)
**Risk**: Medium

**Tasks**:
1. ✅ Create `crates/riptide-api/src/composition/facade_factory.rs`
2. ✅ Define `FacadeFactory` trait
3. ✅ Implement `DefaultFacadeFactory`
4. ✅ Add factory methods for each facade:
   - `create_crawl_facade()`
   - `create_browser_facade()`
   - `create_scraper_facade()`
   - `create_pipeline_facade()`
   - `create_spider_facade()`
   - `create_llm_facade()` (feature-gated)
   - `create_search_facade()` (feature-gated)

5. ✅ Add to ApplicationContext:
```rust
impl ApplicationContext {
    pub fn facade_factory(&self) -> Arc<dyn FacadeFactory> {
        Arc::new(DefaultFacadeFactory::new(Arc::new(self.clone())))
    }
}
```

6. ✅ Add unit tests for factory
7. ✅ Run clippy and tests

**Verification**:
```bash
cargo test -p riptide-api facade_factory
```

**Rollback**: Remove facade_factory.rs, facades can still be constructed directly

---

### Phase 2.5: Refactor Facades to Accept Ports

**Duration**: 6-8 hours
**Parallelizable**: Yes (7 parallel tasks)
**Risk**: High (changes all facades)

#### Task 2.5a: Refactor CrawlFacade (Parallel 1)

**Location**: `crates/riptide-facade/src/facades/crawl_handler_facade.rs`

**Current Issues**:
- Direct dependency on infrastructure types
- Concrete implementations in constructor

**Changes**:
```rust
// BEFORE
pub struct CrawlFacade {
    wasm_pool: Arc<WasmInstancePool>,  // Concrete type
    cache: Arc<RedisCache>,            // Concrete type
}

// AFTER
pub struct CrawlFacade {
    wasm_pool: Arc<dyn Pool<WasmInstance>>,  // Port trait
    cache: Arc<dyn CacheStorage>,            // Port trait
    http_client: Arc<dyn HttpClient>,
    rate_limiter: Arc<dyn RateLimiter>,
    event_bus: Arc<dyn EventBus>,
    idempotency_store: Arc<dyn IdempotencyStore>,
    metrics: Arc<dyn MetricsCollector>,
}

impl CrawlFacade {
    pub fn new(
        wasm_pool: Arc<dyn Pool<WasmInstance>>,
        cache: Arc<dyn CacheStorage>,
        http_client: Arc<dyn HttpClient>,
        rate_limiter: Arc<dyn RateLimiter>,
        event_bus: Arc<dyn EventBus>,
        idempotency_store: Arc<dyn IdempotencyStore>,
        metrics: Arc<dyn MetricsCollector>,
    ) -> Self {
        Self {
            wasm_pool,
            cache,
            http_client,
            rate_limiter,
            event_bus,
            idempotency_store,
            metrics,
        }
    }
}
```

**Testing Strategy**:
1. Write integration test with real adapters
2. Write unit test with mock ports
3. Verify all existing tests pass
4. Run clippy

**Verification**:
```bash
cargo test -p riptide-facade crawl
cargo clippy -p riptide-facade -- -D warnings
```

#### Task 2.5b: Refactor BrowserFacade (Parallel 2)

**Location**: `crates/riptide-facade/src/facades/browser.rs`

**Changes**:
```rust
pub struct BrowserFacade {
    browser_pool: Arc<dyn Pool<BrowserSession>>,
    headless_http_client: Arc<dyn HttpClient>,
    circuit_breaker: Arc<dyn CircuitBreaker>,  // NEW
    cache: Arc<dyn CacheStorage>,
    session_storage: Arc<dyn SessionStorage>,
    event_bus: Arc<dyn EventBus>,
    metrics: Arc<dyn MetricsCollector>,
}
```

**Circuit Breaker Integration**:
```rust
impl BrowserFacade {
    pub async fn render(&self, url: &str) -> Result<String> {
        // Try to acquire circuit breaker permit
        let permit = self.circuit_breaker.try_acquire().await
            .map_err(|_| Error::CircuitOpen("Headless service unavailable"))?;

        match self.browser_pool.acquire().await {
            Ok(session) => {
                let result = session.navigate(url).await;
                self.circuit_breaker.record_success().await;
                drop(permit);  // Release permit
                result
            }
            Err(e) => {
                self.circuit_breaker.record_failure().await;
                Err(e)
            }
        }
    }
}
```

**Verification**:
```bash
cargo test -p riptide-facade browser
```

#### Task 2.5c: Refactor ScraperFacade (Parallel 3)

**Location**: `crates/riptide-facade/src/facades/scraper.rs` (if exists)

**Changes**: Similar pattern to CrawlFacade

#### Task 2.5d: Refactor PipelineFacade (Parallel 4)

**Location**: `crates/riptide-facade/src/facades/pipeline.rs`

**Changes**: Accept other facades + ports

#### Task 2.5e: Refactor SpiderFacade (Parallel 5)

**Location**: `crates/riptide-facade/src/facades/spider.rs` (if exists)

**Changes**: Similar pattern to CrawlFacade

#### Task 2.5f: Refactor LlmFacade (Parallel 6, feature-gated)

**Location**: `crates/riptide-facade/src/facades/llm.rs`

**Changes**:
```rust
#[cfg(feature = "llm")]
pub struct LlmFacade {
    llm_pool: Arc<dyn Pool<LlmClient>>,
    circuit_breaker: Arc<dyn CircuitBreaker>,
    cache: Arc<dyn CacheStorage>,
    metrics: Arc<dyn MetricsCollector>,
}
```

#### Task 2.5g: Refactor SearchFacade (Parallel 7, feature-gated)

**Location**: `crates/riptide-facade/src/facades/search.rs`

**Changes**: Similar pattern to CrawlFacade

---

### Phase 2.6: Update API Handlers to Use Factory

**Duration**: 4-5 hours
**Parallelizable**: Yes (per-handler basis)
**Risk**: Medium-High

**Tasks**:
1. Update each handler to use facade factory:

```rust
// BEFORE
pub async fn crawl_handler(
    State(app_state): State<AppState>,
    Json(req): Json<CrawlRequest>,
) -> Result<Json<CrawlResponse>> {
    // Direct construction
    let facade = CrawlFacade::new(
        app_state.wasm_pool.clone(),
        app_state.cache.clone(),
    );
    facade.crawl(req).await
}

// AFTER
pub async fn crawl_handler(
    State(ctx): State<Arc<ApplicationContext>>,
    Json(req): Json<CrawlRequest>,
) -> Result<Json<CrawlResponse>> {
    // Use factory
    let factory = ctx.facade_factory();
    let facade = factory.create_crawl_facade();
    facade.crawl(req).await
}
```

2. Update AppState to use ApplicationContext:

```rust
// BEFORE
pub struct AppState {
    wasm_pool: Arc<WasmInstancePool>,
    cache: Arc<RedisCache>,
    // ... many fields
}

// AFTER
pub struct AppState {
    context: Arc<ApplicationContext>,
}

impl AppState {
    pub fn new(context: Arc<ApplicationContext>) -> Self {
        Self { context }
    }
}
```

3. Update main.rs server initialization:

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = DiConfig::from_env()?;

    // Create application context
    let context = Arc::new(ApplicationContext::new(&config).await?);

    // Create app state
    let app_state = AppState::new(context.clone());

    // Build router
    let app = Router::new()
        .route("/crawl", post(crawl_handler))
        // ... other routes
        .with_state(context);  // Pass context directly

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```

**Verification**:
```bash
cargo build -p riptide-api
cargo test -p riptide-api
cargo run -p riptide-api --release  # Integration test
```

---

### Phase 2.7: Remove Direct Infrastructure Dependencies

**Duration**: 2-3 hours
**Parallelizable**: No (depends on all previous phases)
**Risk**: Low

**Tasks**:
1. Search for remaining direct infrastructure imports in riptide-facade:
```bash
grep -r "riptide_cache::" crates/riptide-facade/src/
grep -r "riptide_browser::" crates/riptide-facade/src/
grep -r "riptide_pool::" crates/riptide-facade/src/
```

2. Replace with port trait imports:
```rust
// REMOVE
use riptide_cache::RedisCache;
use riptide_pool::WasmInstancePool;

// ADD
use riptide_types::ports::{CacheStorage, Pool};
```

3. Update Cargo.toml dependencies:
```toml
# crates/riptide-facade/Cargo.toml

[dependencies]
riptide-types = { path = "../riptide-types" }
# REMOVE: riptide-cache, riptide-browser, riptide-pool
```

4. Run clippy to verify no infrastructure dependencies:
```bash
cargo clippy -p riptide-facade -- -D warnings
```

**Verification**:
```bash
cargo tree -p riptide-facade -i riptide-cache  # Should show: not found
cargo tree -p riptide-facade -i riptide-browser  # Should show: not found
cargo test -p riptide-facade
```

---

### Phase 2.8: Documentation & Validation

**Duration**: 2-3 hours
**Parallelizable**: Partially
**Risk**: Low

**Tasks**:
1. Update architecture documentation
2. Create ADR (Architecture Decision Record) for hexagonal migration
3. Update CLAUDE.md with new patterns
4. Run full test suite:
```bash
cargo test --workspace
```

5. Run quality gates:
```bash
./scripts/quality_gate.sh
```

6. Validate zero circular dependencies:
```bash
cargo tree -p riptide-facade -i riptide-cache  # Should be empty
cargo tree -p riptide-types -i riptide-api     # Should be empty
```

7. Performance benchmarks (if any exist)

**Verification**:
- All tests pass
- All clippy warnings resolved
- Cargo check succeeds for all crates
- Documentation builds without warnings

---

## Rollback Strategy

### Rollback Decision Tree

```
Is production deployment successful?
├─ YES → Complete migration
└─ NO  → Which phase failed?
    ├─ Phase 2.1-2.2 (Ports/Adapters)
    │   └─ Rollback: Revert port trait files, keep existing code
    │
    ├─ Phase 2.3 (ApplicationContext)
    │   └─ Rollback: Revert ApplicationContext changes, facades work with old structure
    │
    ├─ Phase 2.4 (Factory)
    │   └─ Rollback: Remove facade_factory.rs, use direct construction
    │
    ├─ Phase 2.5 (Facade Refactoring)
    │   └─ Rollback: Revert facade constructors to accept concrete types
    │
    ├─ Phase 2.6 (Handler Updates)
    │   └─ Rollback: Revert handlers to use AppState, revert main.rs
    │
    └─ Phase 2.7 (Cleanup)
        └─ Rollback: Restore infrastructure dependencies in Cargo.toml
```

### Rollback Procedure

**Step 1**: Identify failing phase
```bash
git log --oneline --graph --decorate -20
```

**Step 2**: Create rollback branch
```bash
git checkout -b rollback-phase-2.X
```

**Step 3**: Revert commits for that phase
```bash
git revert <commit-hash>..HEAD
```

**Step 4**: Verify rollback
```bash
cargo test --workspace
cargo build --release
```

**Step 5**: Deploy rollback
```bash
git push origin rollback-phase-2.X
```

### Rollback Testing Checklist

- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] No clippy warnings
- [ ] Cargo check succeeds
- [ ] API endpoints respond correctly
- [ ] Performance metrics acceptable
- [ ] Error rates within SLO

---

## Critical Path Analysis

### Critical Path (Must be sequential)

```
Phase 2.1 (CircuitBreaker Port)
    ↓
Phase 2.2 (Adapters) - Can parallelize within phase
    ↓
Phase 2.3 (ApplicationContext)
    ↓
Phase 2.4 (Facade Factory)
    ↓
Phase 2.5 (Facade Refactoring) - Can parallelize within phase
    ↓
Phase 2.6 (Handler Updates) - Can parallelize within phase
    ↓
Phase 2.7 (Cleanup)
    ↓
Phase 2.8 (Validation)
```

**Total Duration**: ~24-32 hours of development time
**With Parallelization**: ~18-24 hours of calendar time

---

## Testing Strategy

### Test Pyramid

```
          /\
         /  \  E2E Tests (Few)
        /────\
       /      \  Integration Tests (Some)
      /────────\
     /          \  Unit Tests (Many)
    /────────────\
```

### Testing at Each Phase

**Phase 2.1-2.2**: Unit tests for port traits and adapters
- Test state transitions
- Test error handling
- Test thread safety

**Phase 2.3**: Unit tests for ApplicationContext
- Test wiring correctness
- Test feature flags
- Test builder pattern

**Phase 2.4**: Unit tests for facade factory
- Test factory methods create correct types
- Test port injection

**Phase 2.5**: Unit + Integration tests for facades
- Unit tests with mock ports
- Integration tests with real adapters
- Verify existing tests still pass

**Phase 2.6**: Integration + E2E tests for handlers
- Test HTTP endpoints
- Test error responses
- Test metrics collection

**Phase 2.7-2.8**: Full system validation
- Smoke tests
- Load tests
- Performance regression tests

---

## Success Criteria

### Phase Completion Checklist

Each phase must meet these criteria before proceeding:

- [ ] All new code has unit tests
- [ ] All tests pass (including existing tests)
- [ ] Zero clippy warnings (`cargo clippy -- -D warnings`)
- [ ] Cargo check succeeds for all affected crates
- [ ] Documentation updated
- [ ] Code reviewed
- [ ] Git commit with descriptive message
- [ ] Branch pushed to remote

### Final Migration Success Criteria

- [ ] Zero circular dependencies verified
- [ ] All facades use port traits only
- [ ] ApplicationContext wires all dependencies
- [ ] Facade factory pattern implemented
- [ ] All tests pass (unit, integration, E2E)
- [ ] Performance benchmarks pass (no regression)
- [ ] Documentation complete
- [ ] ADR created
- [ ] Production deployment successful

---

## Risk Mitigation

### High-Risk Areas

1. **Facade Refactoring (Phase 2.5)**
   - Risk: Breaking existing functionality
   - Mitigation: Test-driven refactoring, maintain backward compatibility
   - Fallback: Keep old constructors as deprecated methods

2. **Handler Updates (Phase 2.6)**
   - Risk: Breaking API contracts
   - Mitigation: Comprehensive integration tests, gradual rollout
   - Fallback: Blue-green deployment

3. **Performance Regression**
   - Risk: Arc overhead, trait object dispatch
   - Mitigation: Benchmark before/after, profile hot paths
   - Fallback: Optimize hot paths with inline hints

### Monitoring During Migration

**Metrics to Track**:
- Request latency (p50, p95, p99)
- Error rates
- Memory usage
- CPU usage
- Pool utilization
- Circuit breaker states

**Alert Thresholds**:
- Latency increase > 10%
- Error rate increase > 5%
- Memory usage increase > 20%

---

## Next Steps

1. **Review & Approve**: Get architecture review approval
2. **Schedule Work**: Break into sprints (2-3 sprints)
3. **Kickoff Phase 2.1**: Implement CircuitBreaker port
4. **Execute Migration**: Follow phase-by-phase plan
5. **Validate & Deploy**: Run quality gates, deploy to production
