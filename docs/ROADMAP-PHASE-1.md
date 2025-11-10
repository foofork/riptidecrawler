# Phase 1: P0 Critical Blockers (Weeks 1-3)

**Duration**: 3 weeks
**Goal**: Eliminate critical architectural blockers preventing hexagonal architecture
**Key Outcomes**:
- Hybrid AppState pattern established
- 5 missing P0 ports created
- 3 critical circular dependencies broken
- ApplicationContext integrated into runtime

---

## Phase Overview

Phase 1 addresses the **7 critical blockers** identified in technical validation:

1. **AppState God Object** (2213 LOC, 40+ fields)
   - **Solution**: Hybrid `AppState { context, facades }` pattern

2. **Missing P0 Ports** (IdempotencyStore, CircuitBreaker, RateLimiter, Validator, Authorizer)
   - **Solution**: Create trait definitions in riptide-types

3. **Circular Dependencies** (AppState ↔ Facades)
   - **Solution**: Facades depend on ApplicationContext, not AppState

4. **No Init Order** (Undefined startup sequence)
   - **Solution**: ApplicationContext → Facades → AppState wrapper

5. **Undefined Runtime Validation** (Can't verify port wiring)
   - **Solution**: `ApplicationContext::validate()` method

6. **Missing Resilience Patterns** (No circuit breakers, retries)
   - **Solution**: Port traits for CircuitBreaker, RateLimiter

7. **No Rollback Testing** (Can't prove feature flags work)
   - **Solution**: Per-sprint rollback drills

---

## Sprint 1: ApplicationContext Integration (Week 1)

**Goal**: Make ApplicationContext usable in runtime without removing AppState
**Strategy**: Hybrid pattern keeps both systems working during migration

### Day 1-2: ApplicationContext Builder Pattern

#### Task 1.1: Create ApplicationContext in riptide-api
`crates/riptide-api/src/composition/mod.rs`:

```rust
use std::sync::Arc;
use riptide_types::{
    BrowserDriver, HttpClient, CacheStorage, SessionStorage,
    EventBus, Clock, Entropy, TransactionManager,
    MetricsCollector, HealthChecker,
};

/// Dependency injection container with port trait dependencies
pub struct ApplicationContext {
    // Core Infrastructure Ports
    pub browser_driver: Arc<dyn BrowserDriver>,
    pub http_client: Arc<dyn HttpClient>,
    pub cache_storage: Arc<dyn CacheStorage>,
    pub session_store: Arc<dyn SessionStorage>,
    pub event_bus: Arc<dyn EventBus>,

    // System Ports
    pub clock: Arc<dyn Clock>,
    pub entropy: Arc<dyn Entropy>,
    pub transaction_manager: Arc<dyn TransactionManager>,

    // Observability Ports
    pub metrics: Arc<dyn MetricsCollector>,
    pub health: Arc<dyn HealthChecker>,

    // Configuration
    pub config: DiConfig,
}

impl ApplicationContext {
    pub fn builder() -> ApplicationContextBuilder {
        ApplicationContextBuilder::default()
    }

    /// Validate that all required ports are wired
    pub fn validate(&self) -> Result<(), ContextError> {
        // Verify each port is initialized
        self.browser_driver.health_check().await?;
        self.http_client.health_check().await?;
        self.cache_storage.ping().await?;
        // ... validate all ports
        Ok(())
    }
}
```

**Acceptance Criteria**:
- [ ] ApplicationContext compiles
- [ ] Builder pattern allows gradual construction
- [ ] `validate()` method checks all ports
- [ ] Unit tests for builder and validate

#### Task 1.2: Create Builder Implementation
```rust
#[derive(Default)]
pub struct ApplicationContextBuilder {
    browser_driver: Option<Arc<dyn BrowserDriver>>,
    http_client: Option<Arc<dyn HttpClient>>,
    cache_storage: Option<Arc<dyn CacheStorage>>,
    // ... all other fields
}

impl ApplicationContextBuilder {
    pub fn with_browser_driver(mut self, driver: Arc<dyn BrowserDriver>) -> Self {
        self.browser_driver = Some(driver);
        self
    }

    pub fn with_http_client(mut self, client: Arc<dyn HttpClient>) -> Self {
        self.http_client = Some(client);
        self
    }

    // ... builder methods for each port

    pub fn build(self) -> Result<ApplicationContext, ContextError> {
        Ok(ApplicationContext {
            browser_driver: self.browser_driver
                .ok_or(ContextError::MissingPort("browser_driver"))?,
            http_client: self.http_client
                .ok_or(ContextError::MissingPort("http_client"))?,
            // ... construct all fields
        })
    }
}
```

**Acceptance Criteria**:
- [ ] Builder enforces required ports at compile time
- [ ] `build()` returns error for missing ports
- [ ] Fluent API allows chaining

### Day 3-4: Hybrid AppState Pattern

#### Task 1.3: Modify AppState to Wrap ApplicationContext
`crates/riptide-api/src/state.rs`:

```rust
#[cfg(feature = "new-context")]
pub struct AppState {
    /// Clean DI container with port traits
    pub context: Arc<ApplicationContext>,

    /// Facades (will be moved to FacadeRegistry in Sprint 6)
    pub facades: Arc<FacadeRegistry>,
}

#[cfg(feature = "legacy-appstate")]
pub struct AppState {
    // Keep all 40+ fields for backward compatibility
    pub http_client: Client,
    pub cache: Arc<tokio::sync::Mutex<CacheManager>>,
    // ... all existing fields
}

impl AppState {
    #[cfg(feature = "new-context")]
    pub fn new(context: ApplicationContext) -> Self {
        let facades = FacadeRegistry::new(context.clone());
        Self {
            context: Arc::new(context),
            facades: Arc::new(facades),
        }
    }

    #[cfg(feature = "legacy-appstate")]
    pub fn new(/* all 40+ fields */) -> Self {
        Self {
            http_client,
            cache,
            // ... all existing fields
        }
    }
}
```

**Acceptance Criteria**:
- [ ] Both feature modes compile
- [ ] `AppState::new()` signature differs by feature
- [ ] Tests pass in both modes

#### Task 1.4: Update main.rs Initialization
`crates/riptide-api/src/main.rs`:

```rust
async fn main() -> Result<()> {
    let config = load_config()?;

    #[cfg(feature = "new-context")]
    let state = {
        // Initialize adapters
        let browser_adapter = ChromiumBrowserAdapter::new(config.browser)?;
        let http_adapter = ReqwestHttpAdapter::new(config.http)?;
        let cache_adapter = RedisCacheAdapter::new(config.redis)?;

        // Build ApplicationContext
        let context = ApplicationContext::builder()
            .with_browser_driver(Arc::new(browser_adapter))
            .with_http_client(Arc::new(http_adapter))
            .with_cache_storage(Arc::new(cache_adapter))
            // ... wire all ports
            .build()?;

        // Validate all ports wired
        context.validate().await?;

        // Create hybrid AppState
        AppState::new(context)
    };

    #[cfg(feature = "legacy-appstate")]
    let state = {
        // Old initialization (40+ field construction)
        AppState::new(
            reqwest::Client::new(),
            Arc::new(tokio::sync::Mutex::new(CacheManager::new())),
            // ... all 40+ fields
        )
    };

    // Start server (same for both modes)
    axum::Server::bind(&addr)
        .serve(app.with_state(state))
        .await?;
}
```

**Acceptance Criteria**:
- [ ] Server starts in both modes
- [ ] `context.validate()` passes in new-context mode
- [ ] Health endpoint returns 200 in both modes

### Day 5: Testing & Documentation

#### Task 1.5: Integration Tests for Both Modes
`crates/riptide-api/tests/initialization_tests.rs`:

```rust
#[cfg(feature = "new-context")]
#[tokio::test]
async fn test_application_context_initialization() {
    let context = create_test_context().await;

    // Validate all ports wired
    assert!(context.validate().await.is_ok());

    // Verify each port is functional
    context.browser_driver.health_check().await.unwrap();
    context.http_client.health_check().await.unwrap();
}

#[cfg(feature = "legacy-appstate")]
#[tokio::test]
async fn test_legacy_appstate_initialization() {
    let state = create_test_state().await;

    // Verify all fields initialized
    assert!(state.http_client.get("http://example.com").send().await.is_ok());
}

#[tokio::test]
async fn test_server_starts_in_both_modes() {
    // Test that /health endpoint works in both feature modes
    let server = spawn_test_server().await;
    let response = reqwest::get("http://localhost:3000/health").await.unwrap();
    assert_eq!(response.status(), 200);
}
```

**Acceptance Criteria**:
- [ ] Tests pass in `new-context` mode
- [ ] Tests pass in `legacy-appstate` mode
- [ ] CI runs both test suites

#### Task 1.6: Document ADR for Hybrid Pattern
`docs/architecture/ADR-002-hybrid-appstate.md`:

```markdown
# ADR 002: Hybrid AppState Pattern

**Status**: Accepted
**Date**: 2025-11-[DATE]
**Supersedes**: ADR-001 (extends initialization order)

## Context
AppState has 40+ fields. Atomic migration is too risky. Need gradual migration path.

## Decision
Use hybrid `AppState { context, facades }` from Sprint 1-9:

1. **Sprint 1-3**: ApplicationContext created, facades still use AppState fields
2. **Sprint 4-6**: Facades migrated to use `state.context` instead of direct fields
3. **Sprint 7-9**: All facades using context, facade tests green
4. **Sprint 10**: Remove AppState wrapper, facades use ApplicationContext directly

## Consequences
### Positive
- Gradual migration reduces risk
- Feature flags enable instant rollback
- Each sprint is independently testable
- Teams can adopt at their own pace

### Negative
- Temporary complexity (two code paths)
- AppState still exists until Sprint 10
- Extra maintenance burden for 9 sprints

### Mitigation
- Feature flags tested every sprint
- Clear documentation on which mode to use when
- Timeline commitment: remove wrapper by Sprint 10

## Alternatives Considered
1. **Big Bang Migration**: Replace AppState in one sprint (REJECTED: too risky)
2. **Keep AppState Forever**: Never remove wrapper (REJECTED: perpetuates debt)
3. **Parallel Systems**: Run both in production (REJECTED: double resource cost)
```

**Acceptance Criteria**:
- [ ] ADR documents decision clearly
- [ ] Alternatives and trade-offs explained
- [ ] Timeline commitment stated

### Sprint 1 Quality Gates

**Gate 1: Builds in both modes** ✅
- `cargo build --features legacy-appstate` passes
- `cargo build --features new-context` passes

**Gate 2: Top routes run** ✅
- `/api/v1/crawl` returns 200 in both modes
- `/api/v1/extract` returns 200 in both modes
- `/health` returns 200 in both modes

**Gate 3: All ports wired** ✅
- `ApplicationContext::validate()` passes (new-context mode)
- Builder enforces required ports

**Gate 4: Tests pass** ✅
- Unit tests: 100% passing (new tests added)
- Integration tests: Both modes tested
- Coverage: ≥ Week 0 baseline (61%)

**Gate 5: Rollback works** ✅
- Feature flag flip tested: `new-context` → `legacy-appstate`
- Rollback completes in <5 minutes
- No data loss, no manual intervention

**Gate 6: Docs updated** ✅
- ADR-002 written
- Dependency matrix shows ApplicationContext
- Migration guide for teams

---

## Sprint 2: Create P0 Port Traits (Week 2)

**Goal**: Create 5 missing P0 ports needed for resilience and deduplication
**Ports**: IdempotencyStore, CircuitBreaker, RateLimiter, Validator, Authorizer

### Day 1-2: Create Port Trait Definitions

#### Task 2.1: IdempotencyStore Port
`crates/riptide-types/src/idempotency.rs`:

```rust
use async_trait::async_trait;
use std::time::Duration;

/// Port for idempotent operation tracking
#[async_trait]
pub trait IdempotencyStore: Send + Sync {
    /// Check if operation ID has been seen before
    async fn is_duplicate(&self, operation_id: &str) -> Result<bool, IdempotencyError>;

    /// Record operation ID with TTL
    async fn record(&self, operation_id: &str, ttl: Duration) -> Result<(), IdempotencyError>;

    /// Get stored result for duplicate operation (optional)
    async fn get_result(&self, operation_id: &str) -> Result<Option<Vec<u8>>, IdempotencyError>;

    /// Store result for future duplicate requests
    async fn store_result(
        &self,
        operation_id: &str,
        result: Vec<u8>,
        ttl: Duration,
    ) -> Result<(), IdempotencyError>;
}

#[derive(Debug, thiserror::Error)]
pub enum IdempotencyError {
    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}
```

**Acceptance Criteria**:
- [ ] Trait compiles
- [ ] Async-trait properly applied
- [ ] Error types defined
- [ ] Documentation explains use cases

#### Task 2.2: CircuitBreaker Port
`crates/riptide-types/src/resilience.rs`:

```rust
use async_trait::async_trait;
use std::time::Duration;

/// Port for circuit breaker pattern
#[async_trait]
pub trait CircuitBreaker: Send + Sync {
    /// Execute operation with circuit breaker protection
    async fn call<F, T>(&self, operation: F) -> Result<T, CircuitBreakerError>
    where
        F: FnOnce() -> futures::future::BoxFuture<'static, Result<T, Box<dyn std::error::Error>>> + Send,
        T: Send;

    /// Get current circuit state
    fn state(&self) -> CircuitState;

    /// Manually reset circuit
    async fn reset(&self);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,     // Normal operation
    Open,       // Failing, rejecting requests
    HalfOpen,   // Testing if service recovered
}

#[derive(Debug, thiserror::Error)]
pub enum CircuitBreakerError {
    #[error("Circuit breaker is open")]
    CircuitOpen,

    #[error("Operation failed: {0}")]
    OperationFailed(String),
}
```

**Acceptance Criteria**:
- [ ] Trait supports async operations
- [ ] Circuit states well-defined
- [ ] Generic over operation return type

#### Task 2.3: RateLimiter Port
`crates/riptide-types/src/resilience.rs`:

```rust
#[async_trait]
pub trait RateLimiter: Send + Sync {
    /// Check if request is allowed under current rate limit
    async fn check_rate_limit(&self, key: &str) -> Result<bool, RateLimitError>;

    /// Wait until request is allowed (blocking)
    async fn wait_for_slot(&self, key: &str) -> Result<(), RateLimitError>;

    /// Get remaining capacity for key
    async fn remaining(&self, key: &str) -> Result<u64, RateLimitError>;

    /// Reset rate limit for key
    async fn reset(&self, key: &str) -> Result<(), RateLimitError>;
}

#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("Rate limit exceeded for key: {0}")]
    Exceeded(String),

    #[error("Storage error: {0}")]
    Storage(String),
}
```

**Acceptance Criteria**:
- [ ] Supports both check and wait patterns
- [ ] Key-based rate limiting (per-user, per-IP, etc.)
- [ ] Remaining capacity queryable

#### Task 2.4: Validator Port
`crates/riptide-types/src/validation.rs`:

```rust
#[async_trait]
pub trait Validator: Send + Sync {
    /// Validate input against schema
    async fn validate<T>(&self, input: &T) -> Result<(), ValidationError>
    where
        T: serde::Serialize + Send + Sync;

    /// Validate and coerce input (normalize)
    async fn validate_and_coerce<T>(&self, input: T) -> Result<T, ValidationError>
    where
        T: serde::Serialize + serde::de::DeserializeOwned + Send + Sync;
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Validation failed: {0}")]
    Invalid(String),

    #[error("Missing required field: {0}")]
    MissingField(String),
}
```

**Acceptance Criteria**:
- [ ] Generic over input types
- [ ] Supports both validate and coerce patterns
- [ ] Clear error messages for validation failures

#### Task 2.5: Authorizer Port
`crates/riptide-types/src/authorization.rs`:

```rust
#[async_trait]
pub trait Authorizer: Send + Sync {
    /// Check if user has permission for resource
    async fn authorize(
        &self,
        user_id: &str,
        resource: &str,
        action: &str,
    ) -> Result<bool, AuthorizationError>;

    /// Get all permissions for user
    async fn get_permissions(&self, user_id: &str) -> Result<Vec<Permission>, AuthorizationError>;

    /// Grant permission to user
    async fn grant_permission(
        &self,
        user_id: &str,
        permission: Permission,
    ) -> Result<(), AuthorizationError>;
}

#[derive(Debug, Clone)]
pub struct Permission {
    pub resource: String,
    pub action: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AuthorizationError {
    #[error("Permission denied")]
    PermissionDenied,

    #[error("User not found: {0}")]
    UserNotFound(String),
}
```

**Acceptance Criteria**:
- [ ] RBAC-style authorization supported
- [ ] Permissions granular (resource + action)
- [ ] Extensible for ABAC in future

### Day 3-4: Wire New Ports into ApplicationContext

#### Task 2.6: Update ApplicationContext
`crates/riptide-api/src/composition/mod.rs`:

```rust
pub struct ApplicationContext {
    // ... existing ports ...

    // NEW: Resilience & Security Ports (Sprint 2)
    pub idempotency_store: Arc<dyn IdempotencyStore>,
    pub circuit_breaker: Arc<dyn CircuitBreaker>,
    pub rate_limiter: Arc<dyn RateLimiter>,
    pub validator: Arc<dyn Validator>,
    pub authorizer: Arc<dyn Authorizer>,

    // ... config ...
}

impl ApplicationContextBuilder {
    pub fn with_idempotency_store(mut self, store: Arc<dyn IdempotencyStore>) -> Self {
        self.idempotency_store = Some(store);
        self
    }

    // ... builder methods for each new port ...
}
```

**Acceptance Criteria**:
- [ ] ApplicationContext compiles with 5 new ports
- [ ] Builder enforces new required ports
- [ ] `validate()` checks new ports

#### Task 2.7: Create In-Memory Adapters (for testing)
`crates/riptide-infrastructure/src/idempotency/memory.rs`:

```rust
use std::collections::HashMap;
use std::sync::RwLock;
use riptide_types::{IdempotencyStore, IdempotencyError};

pub struct InMemoryIdempotencyStore {
    records: RwLock<HashMap<String, (Vec<u8>, std::time::Instant)>>,
}

#[async_trait]
impl IdempotencyStore for InMemoryIdempotencyStore {
    async fn is_duplicate(&self, operation_id: &str) -> Result<bool, IdempotencyError> {
        let records = self.records.read().unwrap();
        Ok(records.contains_key(operation_id))
    }

    async fn record(&self, operation_id: &str, ttl: Duration) -> Result<(), IdempotencyError> {
        let mut records = self.records.write().unwrap();
        let expiry = std::time::Instant::now() + ttl;
        records.insert(operation_id.to_string(), (vec![], expiry));
        Ok(())
    }

    // ... implement remaining methods ...
}
```

Create similar in-memory adapters for:
- `InMemoryCircuitBreaker`
- `InMemoryRateLimiter`
- `JsonSchemaValidator`
- `InMemoryAuthorizer`

**Acceptance Criteria**:
- [ ] All 5 in-memory adapters compile
- [ ] Adapters are testable (no external dependencies)
- [ ] Used in tests and development mode

### Day 5: Testing & Documentation

#### Task 2.8: Unit Tests for Each Port
`crates/riptide-types/src/idempotency/tests.rs`:

```rust
#[tokio::test]
async fn test_idempotency_store_prevents_duplicates() {
    let store = InMemoryIdempotencyStore::new();

    // First request
    assert!(!store.is_duplicate("op-123").await.unwrap());
    store.record("op-123", Duration::from_secs(60)).await.unwrap();

    // Duplicate request
    assert!(store.is_duplicate("op-123").await.unwrap());
}
```

Write comprehensive tests for:
- [ ] IdempotencyStore: duplicate detection, TTL expiry, result storage
- [ ] CircuitBreaker: state transitions, failure threshold, half-open recovery
- [ ] RateLimiter: capacity enforcement, key isolation, reset functionality
- [ ] Validator: schema validation, coercion, error messages
- [ ] Authorizer: permission checks, user permissions, grant/revoke

**Acceptance Criteria**:
- [ ] 90%+ coverage for each port
- [ ] Edge cases tested (expiry, overflow, etc.)
- [ ] Tests pass in CI

#### Task 2.9: Update Dependency Matrix
`docs/architecture/DEPENDENCY-MATRIX.md`:

```markdown
| Facade | BrowserDriver | HttpClient | CacheStorage | IdempotencyStore | CircuitBreaker | RateLimiter |
|--------|---------------|------------|--------------|------------------|----------------|-------------|
| BrowserFacade | ✅ | ✅ | ✅ | ✅ (NEW) | ✅ (NEW) | ⬜ |
| CrawlFacade | ✅ | ✅ | ⬜ | ✅ (NEW) | ✅ (NEW) | ✅ (NEW) |
| ExtractorFacade | ⬜ | ✅ | ✅ | ✅ (NEW) | ✅ (NEW) | ⬜ |
```

**Acceptance Criteria**:
- [ ] Matrix shows which facades use which ports
- [ ] New ports clearly marked
- [ ] Coverage gaps identified for Sprint 6

### Sprint 2 Quality Gates

**All 6 gates must pass** ✅

---

## Sprint 3: Break Circular Dependencies (Week 3)

**Goal**: Eliminate 8 circular dependencies between AppState and Facades
**Strategy**: Facades depend on ApplicationContext (ports), not AppState (concrete types)

### Day 1-2: Migrate Top 3 Facades to ApplicationContext

#### Task 3.1: BrowserFacade Migration
**Before** (circular dependency):
```rust
// crates/riptide-facade/src/browser.rs
pub struct BrowserFacade {
    state: Arc<AppState>,  // ❌ Circular: AppState has BrowserFacade field
}

impl BrowserFacade {
    pub async fn launch(&self) -> Result<Browser> {
        // Directly access AppState field
        self.state.browser_pool.checkout().await
    }
}
```

**After** (clean dependency):
```rust
// crates/riptide-facade/src/browser.rs
pub struct BrowserFacade {
    context: Arc<ApplicationContext>,  // ✅ Clean: depends on ports
}

impl BrowserFacade {
    pub fn new(context: Arc<ApplicationContext>) -> Self {
        Self { context }
    }

    pub async fn launch(&self) -> Result<Browser> {
        // Use port trait, not concrete type
        self.context.browser_driver.acquire().await
    }
}
```

**Acceptance Criteria**:
- [ ] BrowserFacade constructor takes ApplicationContext
- [ ] All methods use `self.context.browser_driver` (port trait)
- [ ] No direct AppState references
- [ ] Tests pass

#### Task 3.2: CrawlFacade Migration
```rust
// BEFORE
pub struct CrawlFacade {
    state: Arc<AppState>,
}

impl CrawlFacade {
    pub async fn crawl(&self, url: &str) -> Result<CrawlResult> {
        let browser = self.state.browser_pool.checkout().await?;
        let page = browser.new_page().await?;
        // ... crawling logic
    }
}

// AFTER
pub struct CrawlFacade {
    context: Arc<ApplicationContext>,
}

impl CrawlFacade {
    pub fn new(context: Arc<ApplicationContext>) -> Self {
        Self { context }
    }

    pub async fn crawl(&self, url: &str) -> Result<CrawlResult> {
        // Idempotency check (new port!)
        if self.context.idempotency_store.is_duplicate(&format!("crawl:{}", url)).await? {
            return self.context.idempotency_store.get_result(&format!("crawl:{}", url)).await;
        }

        // Rate limiting (new port!)
        self.context.rate_limiter.wait_for_slot("crawl").await?;

        // Circuit breaker (new port!)
        let result = self.context.circuit_breaker.call(|| async {
            let browser = self.context.browser_driver.acquire().await?;
            let page = browser.new_page().await?;
            // ... crawling logic
        }).await?;

        // Store result for idempotency
        self.context.idempotency_store.store_result(
            &format!("crawl:{}", url),
            serde_json::to_vec(&result)?,
            Duration::from_secs(3600),
        ).await?;

        Ok(result)
    }
}
```

**Acceptance Criteria**:
- [ ] CrawlFacade uses 5 ports (browser, idempotency, circuit breaker, rate limiter, cache)
- [ ] Idempotency prevents duplicate crawls
- [ ] Circuit breaker protects against failing sites
- [ ] Rate limiter prevents overload
- [ ] Tests cover all resilience patterns

#### Task 3.3: ExtractorFacade Migration
Similar pattern for ExtractorFacade:
- [ ] Replace `state: Arc<AppState>` with `context: Arc<ApplicationContext>`
- [ ] Use `context.http_client` instead of `state.http_client`
- [ ] Add circuit breaker for HTTP requests
- [ ] Add validator for extraction parameters
- [ ] Tests pass

### Day 3-4: Update FacadeRegistry

#### Task 3.4: Create FacadeRegistry
`crates/riptide-facade/src/registry.rs`:

```rust
/// Central registry for all facades
pub struct FacadeRegistry {
    context: Arc<ApplicationContext>,

    // Facades
    browser: Arc<BrowserFacade>,
    crawl: Arc<CrawlFacade>,
    extractor: Arc<ExtractorFacade>,
    // ... 32 more facades
}

impl FacadeRegistry {
    pub fn new(context: Arc<ApplicationContext>) -> Self {
        // Initialize all facades with shared context
        let browser = Arc::new(BrowserFacade::new(context.clone()));
        let crawl = Arc::new(CrawlFacade::new(context.clone()));
        let extractor = Arc::new(ExtractorFacade::new(context.clone()));

        Self {
            context,
            browser,
            crawl,
            extractor,
            // ...
        }
    }

    pub fn browser(&self) -> &Arc<BrowserFacade> {
        &self.browser
    }

    // ... accessor methods for each facade
}
```

**Acceptance Criteria**:
- [ ] FacadeRegistry owns all facades
- [ ] Single ApplicationContext shared by all
- [ ] Accessor methods for each facade
- [ ] No circular dependencies

#### Task 3.5: Update AppState to Use FacadeRegistry
```rust
#[cfg(feature = "new-context")]
pub struct AppState {
    pub context: Arc<ApplicationContext>,
    pub facades: Arc<FacadeRegistry>,  // ✅ Clean dependency
}

impl AppState {
    pub fn new(context: ApplicationContext) -> Self {
        let context = Arc::new(context);
        let facades = Arc::new(FacadeRegistry::new(context.clone()));

        Self { context, facades }
    }
}
```

**Acceptance Criteria**:
- [ ] AppState creates FacadeRegistry from ApplicationContext
- [ ] No circular dependencies in dependency graph
- [ ] `cargo tree` shows acyclic graph

### Day 5: Validation & Documentation

#### Task 3.6: Verify Circular Dependencies Eliminated
```bash
# Check for circular dependencies
cargo tree -p riptide-api --depth 3 | grep -E "riptide-(facade|api)" > circular-check.txt

# Count circular references (should be 0)
grep -c "riptide-api.*riptide-facade.*riptide-api" circular-check.txt
# Expected: 0
```

**Acceptance Criteria**:
- [ ] Dependency graph is acyclic
- [ ] No `riptide-api → riptide-facade → riptide-api` cycles
- [ ] Baseline 8 circular dependencies reduced to 0

#### Task 3.7: Document ADR for Facade-Context Dependency
`docs/architecture/ADR-003-facade-context-dependency.md`:

```markdown
# ADR 003: Facades Depend on ApplicationContext, Not AppState

**Status**: Accepted
**Date**: 2025-11-[DATE]

## Context
Facades previously depended on AppState, which also contained facades, creating circular dependencies.

## Decision
All facades now:
1. Take `Arc<ApplicationContext>` in constructor
2. Use port traits via `self.context.{port_name}`
3. Never reference AppState directly

FacadeRegistry:
1. Initializes all facades with shared ApplicationContext
2. Owned by AppState (one-way dependency)

## Consequences
- Circular dependencies eliminated (8 → 0)
- Facades testable in isolation (mock ports)
- Easier to add new facades (just wire ports)
- AppState can eventually be removed (Sprint 10)
```

**Acceptance Criteria**:
- [ ] ADR explains decision and rationale
- [ ] Consequences documented

### Sprint 3 Quality Gates

**All 6 gates must pass** ✅

**Additional Sprint 3 Validation**:
- [ ] Circular dependencies: 8 → 0 (cargo tree verification)
- [ ] Top 3 facades migrated to ApplicationContext
- [ ] FacadeRegistry owns all facades
- [ ] No AppState references in facades

---

## Phase 1 Success Metrics

| Metric | Week 0 Baseline | Sprint 3 Target | Actual |
|--------|-----------------|-----------------|--------|
| Circular Dependencies | 8 | 0 | ___ |
| Missing P0 Ports | 5 | 0 | ___ |
| ApplicationContext Integration | 0% | 100% | ___ |
| Hexagonal Compliance | 24% | 35%+ | ___ |

---

## Phase 1 Deliverables

- [ ] ApplicationContext created and integrated
- [ ] Hybrid AppState pattern established
- [ ] 5 P0 ports created (IdempotencyStore, CircuitBreaker, RateLimiter, Validator, Authorizer)
- [ ] 3 critical circular dependencies eliminated
- [ ] Top 3 facades migrated to ApplicationContext
- [ ] FacadeRegistry created
- [ ] Feature flags tested and validated
- [ ] ADRs 002 and 003 written
- [ ] All quality gates passed for Sprints 1-3

---

## Next: Phase 2 (Weeks 4-6)

**Goal**: Create 7 missing P1 ports and migrate 12 facades to ApplicationContext
**See**: [ROADMAP-PHASE-2.md](ROADMAP-PHASE-2.md)

---

**Status**: Ready for Sprint 1 kickoff
**Owner**: Backend Engineering Team
**Duration**: 3 weeks
