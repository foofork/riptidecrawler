# Phase 1: Ports & Adapters Foundation Roadmap
**Version:** 2.0 (Enhanced)
**Date:** 2025-11-08
**Duration:** 3 weeks (updated from 2 weeks)
**Goal:** Define all infrastructure ports before handler refactoring

---

## Overview

Phase 1 establishes the Ports & Adapters (Hexagonal Architecture) pattern by defining port traits in the domain layer and implementing adapters in infrastructure layers. This creates a clean dependency inversion that enables testability, swappability, and maintainability.

### Objectives

1. **Define Core Ports**: Create trait definitions for all infrastructure dependencies
2. **Implement Adapters**: Provide concrete implementations for ports
3. **Establish Composition Root**: Wire dependencies via dependency injection
4. **Session Port Integration**: Add session management port (NEW)
5. **Core Infrastructure Ports**: Health, metrics, RPC ports (NEW)
6. **Document Architecture**: Clarify facade = application layer

---

## 🚨 Quality Gates (MANDATORY - Every Task)

**Zero-tolerance policy for errors/warnings. Every commit must:**

```bash
# 1. Tests pass (NO #[ignore], NO skipped tests)
cargo test -p [affected-crate]  # NOT --workspace (conserve disk)

# 2. Clippy clean (ZERO warnings)
cargo clippy -p [affected-crate] -- -D warnings

# 3. Cargo check passes
cargo check -p [affected-crate]

# 4. Full workspace ONLY for final phase validation
# Use targeted builds: cargo build -p [crate] to save disk space
```

**Commit Rules:**
- ❌ NO commits with failing tests
- ❌ NO commits with clippy warnings
- ❌ NO commits with compilation errors
- ❌ NO #[ignore] on tests without tracking issue
- ✅ Each phase MUST be fully complete before moving to next

---

### Success Criteria

- ✅ All ports defined (15+ traits)
- ✅ All adapters implemented
- ✅ ApplicationContext wires dependencies
- ✅ Zero direct infra usage in facades
- ✅ Tests use in-memory adapters
- ✅ cargo-deny enforces layer boundaries
- ✅ **All tests pass (ZERO ignored)**
- ✅ **Clippy clean (ZERO warnings)**
- ✅ **Cargo check passes**

---

## Sprint 1.1: Core Infrastructure Ports (Week 1, Days 1-5)

### Task 1.1.1: Define Repository Ports

**File:** `crates/riptide-types/src/ports/repository.rs`

```rust
use async_trait::async_trait;
use std::sync::Arc;

/// Generic repository pattern for domain entities
#[async_trait]
pub trait Repository<T>: Send + Sync {
    async fn find_by_id(&self, id: &str) -> Result<Option<T>>;
    async fn find_all(&self, filter: RepositoryFilter) -> Result<Vec<T>>;
    async fn save(&self, entity: &T) -> Result<()>;
    async fn delete(&self, id: &str) -> Result<()>;
    async fn count(&self, filter: RepositoryFilter) -> Result<usize>;
}

/// Transaction management port
#[async_trait]
pub trait TransactionManager: Send + Sync {
    type Transaction: Transaction;

    async fn begin(&self) -> Result<Self::Transaction>;
    async fn commit(&self, tx: Self::Transaction) -> Result<()>;
    async fn rollback(&self, tx: Self::Transaction) -> Result<()>;
}

#[async_trait]
pub trait Transaction: Send + Sync {
    fn id(&self) -> &str;
    async fn execute<F, R>(&mut self, f: F) -> Result<R>
    where
        F: FnOnce() -> Result<R> + Send;
}
```

**Files Created:**
```
CREATE: crates/riptide-types/src/ports/repository.rs (~150 LOC)
```

---

### Task 1.1.2: Define Event Bus Port

**File:** `crates/riptide-types/src/ports/events.rs`

```rust
#[async_trait]
pub trait EventBus: Send + Sync {
    async fn publish(&self, event: DomainEvent) -> Result<()>;
    async fn subscribe<H>(&self, handler: H) -> Result<SubscriptionId>
    where
        H: EventHandler + Send + Sync + 'static;
}

pub struct DomainEvent {
    pub id: String,
    pub event_type: String,
    pub aggregate_id: String,
    pub payload: serde_json::Value,
    pub timestamp: SystemTime,
    pub metadata: HashMap<String, String>,
}

#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle(&self, event: &DomainEvent) -> Result<()>;
}
```

**Files Created:**
```
CREATE: crates/riptide-types/src/ports/events.rs (~100 LOC)
```

---

### Task 1.1.3: Define Idempotency Port

**File:** `crates/riptide-types/src/ports/idempotency.rs`

```rust
#[async_trait]
pub trait IdempotencyStore: Send + Sync {
    async fn try_acquire(&self, key: &str, ttl: Duration) -> Result<IdempotencyToken>;
    async fn release(&self, token: IdempotencyToken) -> Result<()>;
    async fn exists(&self, key: &str) -> Result<bool>;
}

pub struct IdempotencyToken {
    pub key: String,
    pub acquired_at: SystemTime,
    pub expires_at: SystemTime,
}
```

**Files Created:**
```
CREATE: crates/riptide-types/src/ports/idempotency.rs (~80 LOC)
```

---

### Task 1.1.4: Define Feature Ports

**File:** `crates/riptide-types/src/ports/features.rs`

```rust
/// Browser automation port
#[async_trait]
pub trait BrowserDriver: Send + Sync {
    async fn navigate(&self, url: &str) -> Result<BrowserSession>;
    async fn execute_script(&self, session: &BrowserSession, script: &str) -> Result<ScriptResult>;
    async fn screenshot(&self, session: &BrowserSession) -> Result<Vec<u8>>;
    async fn close(&self, session: BrowserSession) -> Result<()>;
}

/// PDF processing port
#[async_trait]
pub trait PdfProcessor: Send + Sync {
    async fn extract_text(&self, pdf_data: &[u8]) -> Result<String>;
    async fn extract_images(&self, pdf_data: &[u8]) -> Result<Vec<Vec<u8>>>;
    async fn render_page(&self, pdf_data: &[u8], page: usize) -> Result<Vec<u8>>;
}

/// Search engine port
#[async_trait]
pub trait SearchEngine: Send + Sync {
    async fn index(&self, document: SearchDocument) -> Result<()>;
    async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>>;
    async fn delete(&self, id: &str) -> Result<()>;
}
```

**Files Created:**
```
CREATE: crates/riptide-types/src/ports/features.rs (~120 LOC)
```

---

### Task 1.1.5: Define Infrastructure Ports

**File:** `crates/riptide-types/src/ports/infrastructure.rs`

```rust
/// System clock port (for testing determinism)
pub trait Clock: Send + Sync {
    fn now(&self) -> SystemTime;
    fn now_utc(&self) -> DateTime<Utc>;
}

/// Entropy source port (for testing determinism)
pub trait Entropy: Send + Sync {
    fn random_bytes(&self, len: usize) -> Vec<u8>;
    fn random_id(&self) -> String;
}

/// Already defined in Phase 0
pub use super::cache::CacheStorage;
```

**Files Created:**
```
CREATE: crates/riptide-types/src/ports/infrastructure.rs (~70 LOC)
```

---

### Task 1.1.6: Document Facade Layer Architectural Rules (0.5 days)

**Goal:** Add crate-level documentation clarifying that riptide-facade IS the Application Layer

**File:** `crates/riptide-facade/src/lib.rs`

**Add this documentation comment at the top of the file:**

```rust
//! # Riptide Facade - Application Layer (Use-Cases)
//!
//! This crate contains application use-cases that orchestrate domain logic via ports.
//!
//! ## Architectural Rules
//!
//! **FORBIDDEN in this crate:**
//! - ❌ NO HTTP types (actix_web, hyper, axum, etc.)
//! - ❌ NO database types (sqlx, postgres, etc.)
//! - ❌ NO serialization formats (serde_json::Value - use typed DTOs)
//! - ❌ NO SDK/client types (redis, reqwest, etc.)
//! - ❌ NO infrastructure implementations
//!
//! ## What Lives Here
//!
//! **ALLOWED in this crate:**
//! - ✅ Use-case orchestration (workflows, transactions)
//! - ✅ Cross-cutting concerns (retry coordination, timeout management)
//! - ✅ Authorization policies (tenant scoping, RBAC)
//! - ✅ Idempotency management
//! - ✅ Domain event emission
//! - ✅ Transactional outbox writes
//! - ✅ Backpressure and cancellation token management
//! - ✅ Business metrics collection
//!
//! ## Dependencies
//!
//! This crate ONLY depends on:
//! - `riptide-types` (for domain types and port traits)
//! - Common utilities: `riptide-config`, `riptide-events`, `riptide-monitoring`
//! - NO infrastructure crates (riptide-reliability, riptide-cache, riptide-browser, etc.)
//!
//! ## Layer Boundary
//!
//! ```text
//! API Layer (riptide-api)
//!       ↓ calls
//! APPLICATION LAYER (riptide-facade) ← YOU ARE HERE
//!       ↓ uses ports (traits)
//! Domain Layer (riptide-types)
//!       ↑ implemented by
//! Infrastructure Layer (riptide-reliability, riptide-cache, etc.)
//! ```
//!
//! Infrastructure implementations are injected via dependency injection at the
//! composition root (`ApplicationContext` in riptide-api).
```

**Validation:**
```bash
# Ensure documentation exists
head -40 crates/riptide-facade/src/lib.rs | grep "Application Layer"

# Verify no HTTP/database types in facade crate
cargo tree -p riptide-facade | grep -iE 'axum|actix|redis|sqlx' \
  && echo "FAIL: Forbidden dependencies found" \
  || echo "PASS: No forbidden dependencies"
```

**Success Criteria:**
- ✅ Documentation comment added to lib.rs
- ✅ Team understands facade = application layer (no rename needed)
- ✅ Clear rules documented for what belongs in this crate
- ✅ CI enforces these rules via cargo-deny (added in Phase 5)

**Files Modified:**
```
UPDATE: crates/riptide-facade/src/lib.rs (add architecture documentation)
```

---

### Task 1.1.7: Domain Purity - Remove CircuitBreaker from Types (0.5 days) ⚠️ CRITICAL

**Problem:** `riptide-types` (domain layer) contains infrastructure (CircuitBreaker, tokio dependency)

**Violation Found:**
- `crates/riptide-types/src/reliability/circuit.rs` (372 LOC) - Infrastructure in domain!
- Domain depends on `tokio::sync` - architectural violation
- **Source:** WORKSPACE_CRATE_ANALYSIS.md §2 - Critical Violation #1

**Solution:**
```bash
# Move circuit breaker to infrastructure layer
mv crates/riptide-types/src/reliability/circuit.rs \
   crates/riptide-reliability/src/circuit_breaker.rs

# Update riptide-types to remove infrastructure
# Remove tokio dependency from types/Cargo.toml
```

**Domain Purity Validation:**
```bash
# 1. Verify no infrastructure dependencies in domain
cargo tree -p riptide-types --depth 1 | grep -iE 'tokio|redis|axum|hyper' \
  && echo "❌ FAIL: Infrastructure found in domain" \
  || echo "✅ PASS: Domain is pure"

# 2. Verify CircuitBreaker only in reliability
find crates -name "circuit*.rs" | grep -v riptide-reliability \
  && echo "❌ FAIL: CircuitBreaker found outside reliability" \
  || echo "✅ PASS: Single source of truth"

# 3. Enforce with cargo-deny
cargo deny check bans
```

**Files Modified:**
```
MOVE:   crates/riptide-types/src/reliability/circuit.rs →
        crates/riptide-reliability/src/circuit_breaker.rs
DELETE: crates/riptide-types/src/reliability/ (directory)
UPDATE: crates/riptide-types/Cargo.toml (remove tokio dependency)
UPDATE: crates/riptide-types/src/lib.rs (remove reliability module)
UPDATE: All files using riptide_types::reliability::circuit → riptide_reliability::circuit_breaker
```

**Success Criteria:**
- ✅ Zero infrastructure dependencies in riptide-types
- ✅ All CircuitBreaker usage via riptide-reliability only
- ✅ cargo-deny passes (no tokio in domain)
- ✅ Tests pass after migration

**Impact:**
- **LOC Moved:** 372 LOC from domain to infrastructure
- **Domain Purity:** 100% (critical architectural fix)
- **Preparation:** Enables proper port/adapter pattern in later sprints

**References:**
- WORKSPACE_CRATE_ANALYSIS.md §4 - Critical Violation #1
- PHASE_0_CLEANUP_ROADMAP.md - Sprint 0.4.2 (consolidates other circuit breakers)

---

### Sprint 1.1 Summary

**Files Created:**
```
CREATE: crates/riptide-types/src/ports/mod.rs
CREATE: crates/riptide-types/src/ports/repository.rs
CREATE: crates/riptide-types/src/ports/events.rs
CREATE: crates/riptide-types/src/ports/idempotency.rs
CREATE: crates/riptide-types/src/ports/features.rs
CREATE: crates/riptide-types/src/ports/infrastructure.rs
UPDATE: crates/riptide-types/src/lib.rs
UPDATE: crates/riptide-facade/src/lib.rs (documentation)
```

**Total:** ~600 LOC (trait definitions)

---

## Sprint 1.2: Implement Adapters (Week 2, Days 1-5)

### Task 1.2.1: PostgreSQL Repository Adapter

**File:** `crates/riptide-persistence/src/adapters/postgres_repository.rs`

```rust
use riptide_types::ports::Repository;
use sqlx::PgPool;

pub struct PostgresRepository<T> {
    pool: Arc<PgPool>,
    _phantom: PhantomData<T>,
}

#[async_trait]
impl<T> Repository<T> for PostgresRepository<T>
where
    T: Send + Sync + Serialize + DeserializeOwned,
{
    async fn find_by_id(&self, id: &str) -> Result<Option<T>> {
        // Anti-corruption: SQL -> Domain types
    }

    async fn save(&self, entity: &T) -> Result<()> {
        // Anti-corruption: Domain types -> SQL
    }
}
```

**Files Created:**
```
CREATE: crates/riptide-persistence/src/adapters/postgres_repository.rs (~300 LOC)
```

---

### Task 1.2.2: Redis Idempotency Adapter

**File:** `crates/riptide-cache/src/adapters/redis_idempotency.rs`

```rust
use riptide_types::ports::IdempotencyStore;

pub struct RedisIdempotencyStore {
    client: Arc<redis::Client>,
    pool: deadpool_redis::Pool,
}

#[async_trait]
impl IdempotencyStore for RedisIdempotencyStore {
    async fn try_acquire(&self, key: &str, ttl: Duration) -> Result<IdempotencyToken> {
        let versioned_key = format!("idempotency:v1:{}", key);

        // Atomic SET NX EX
        let acquired: bool = self.pool
            .get().await?
            .set_nx_ex(&versioned_key, "locked", ttl.as_secs() as usize)
            .await?;

        if acquired {
            Ok(IdempotencyToken {
                key: versioned_key,
                acquired_at: SystemTime::now(),
                expires_at: SystemTime::now() + ttl,
            })
        } else {
            Err(RiptideError::DuplicateRequest { key: key.to_string() })
        }
    }

    async fn release(&self, token: IdempotencyToken) -> Result<()> {
        // Safe unlock with Lua script
        let script = r#"
            if redis.call("get", KEYS[1]) then
                return redis.call("del", KEYS[1])
            else
                return 0
            end
        "#;

        self.pool.get().await?
            .eval(script, &[&token.key], &[])
            .await?;
        Ok(())
    }
}
```

**Files Created:**
```
CREATE: crates/riptide-cache/src/adapters/redis_idempotency.rs (~200 LOC)
```

---

### Task 1.2.3: Event Bus Adapters

**File:** `crates/riptide-persistence/src/adapters/outbox_event_bus.rs`

```rust
use riptide_types::ports::{EventBus, TransactionManager};

/// Transactional Outbox pattern implementation
pub struct OutboxEventBus {
    tx_manager: Arc<dyn TransactionManager>,
    pool: Arc<PgPool>,
}

#[async_trait]
impl EventBus for OutboxEventBus {
    async fn publish(&self, event: DomainEvent) -> Result<()> {
        // Write to outbox table in same transaction as business data
        let event_json = serde_json::to_string(&event)?;

        sqlx::query!(
            "INSERT INTO event_outbox (event_id, event_type, payload, created_at)
             VALUES ($1, $2, $3, NOW())",
            event.id,
            event.event_type,
            event_json
        )
        .execute(&*self.pool)
        .await?;

        Ok(())
    }
}

/// Background worker to publish outbox events
pub struct OutboxPublisher {
    pool: Arc<PgPool>,
    transport: Arc<dyn EventTransport>,
}

impl OutboxPublisher {
    pub async fn run(&self) {
        // Poll outbox table, publish to message bus, mark as published
    }
}
```

**Files Created:**
```
CREATE: crates/riptide-persistence/src/adapters/outbox_event_bus.rs (~400 LOC)
CREATE: crates/riptide-persistence/src/adapters/postgres_transaction.rs (~300 LOC)
```

---

### Sprint 1.2 Summary

**Files Created:**
```
CREATE: crates/riptide-persistence/src/adapters/mod.rs
CREATE: crates/riptide-persistence/src/adapters/postgres_repository.rs
CREATE: crates/riptide-persistence/src/adapters/postgres_transaction.rs
CREATE: crates/riptide-persistence/src/adapters/outbox_event_bus.rs
CREATE: crates/riptide-cache/src/adapters/redis_idempotency.rs
CREATE: crates/riptide-cache/src/adapters/redis_cache.rs (impl CacheStorage)
UPDATE: crates/riptide-browser/src/lib.rs (impl BrowserDriver)
UPDATE: crates/riptide-pdf/src/lib.rs (impl PdfProcessor)
UPDATE: crates/riptide-search/src/lib.rs (impl SearchEngine)
```

**Total:** ~1,200 LOC (adapter implementations)

---

## Sprint 1.3: Composition Root (Week 2-3, Days 6-10)

### Task 1.3.1: Create Application Context

**File:** `crates/riptide-api/src/composition.rs` (NEW)

```rust
use riptide_types::ports::*;
use std::sync::Arc;

/// Application configuration and wiring
pub struct ApplicationContext {
    // Repositories
    pub session_repository: Arc<dyn Repository<Session>>,
    pub profile_repository: Arc<dyn Repository<Profile>>,

    // Infrastructure
    pub tx_manager: Arc<dyn TransactionManager>,
    pub event_bus: Arc<dyn EventBus>,
    pub idempotency_store: Arc<dyn IdempotencyStore>,
    pub cache: Arc<dyn CacheStorage>,

    // Features
    pub browser_driver: Arc<dyn BrowserDriver>,
    pub pdf_processor: Arc<dyn PdfProcessor>,
    pub search_engine: Arc<dyn SearchEngine>,

    // System
    pub clock: Arc<dyn Clock>,
    pub entropy: Arc<dyn Entropy>,
}

impl ApplicationContext {
    pub async fn new(config: &Config) -> Result<Self> {
        // Wire up dependencies based on configuration
        let pool = PgPoolOptions::new()
            .max_connections(config.db.max_connections)
            .connect(&config.db.url)
            .await?;

        let redis_pool = deadpool_redis::Config {
            url: Some(config.redis.url.clone()),
            pool: Some(deadpool_redis::PoolConfig::new(config.redis.pool_size)),
            ..Default::default()
        }
        .create_pool(Some(Runtime::Tokio1))?;

        Ok(Self {
            session_repository: Arc::new(PostgresRepository::new(pool.clone())),
            profile_repository: Arc::new(PostgresRepository::new(pool.clone())),

            tx_manager: Arc::new(PostgresTransactionManager::new(pool.clone())),
            event_bus: Arc::new(OutboxEventBus::new(pool.clone())),
            idempotency_store: Arc::new(RedisIdempotencyStore::new(redis_pool.clone())),
            cache: Arc::new(RedisCache::new(redis_pool.clone())),

            browser_driver: Arc::new(ChromeDriver::new(&config.browser)?),
            pdf_processor: Arc::new(PdfiumProcessor::new(&config.pdf)?),
            search_engine: Arc::new(MeiliSearchEngine::new(&config.search)?),

            clock: Arc::new(SystemClock::default()),
            entropy: Arc::new(SystemEntropy::default()),
        })
    }

    pub fn for_testing() -> Self {
        // In-memory implementations for tests
        Self {
            session_repository: Arc::new(InMemoryRepository::new()),
            event_bus: Arc::new(InMemoryEventBus::new()),
            idempotency_store: Arc::new(InMemoryIdempotencyStore::new()),
            cache: Arc::new(InMemoryCache::new()),
            clock: Arc::new(FakeClock::new()),
            entropy: Arc::new(DeterministicEntropy::new()),
            // ...
        }
    }
}
```

---

### Task 1.3.2: Refactor AppState

**File:** `crates/riptide-api/src/state.rs` (REFACTORED)

```rust
pub struct AppState {
    pub context: Arc<ApplicationContext>,

    // Facades (orchestration layer)
    pub extraction_facade: Arc<ExtractionFacade>,
    pub browser_facade: Arc<BrowserFacade>,
    pub pdf_facade: Arc<PdfFacade>,
    pub search_facade: Arc<SearchFacade>,
    pub spider_facade: Arc<SpiderFacade>,
    // ... other facades
}

impl AppState {
    pub async fn new(config: Config) -> Result<Self> {
        let context = Arc::new(ApplicationContext::new(&config).await?);

        Ok(Self {
            context: context.clone(),
            extraction_facade: Arc::new(ExtractionFacade::new(
                context.browser_driver.clone(),
                context.cache.clone(),
                context.event_bus.clone(),
            )),
            browser_facade: Arc::new(BrowserFacade::new(
                context.browser_driver.clone(),
                context.idempotency_store.clone(),
            )),
            // Inject dependencies into facades
        })
    }
}
```

---

## Sprint 1.4: Session Port Definition (NEW - Week 3, Days 11-12)

**Priority:** HIGH
**Source:** API_CRATE_COVERAGE_ANALYSIS.md - Gap #3

### Problem

- `sessions/storage.rs` (541 LOC) has direct DB access
- `sessions/manager.rs` (503 LOC) has business logic in API layer
- `sessions/middleware.rs` (507 LOC) has mixed concerns
- **No port pattern** - tightly coupled to PostgreSQL

### Tasks

**Task 1.4.1: Define Session Port**

**File:** `crates/riptide-types/src/ports/session.rs` (NEW)

```rust
#[async_trait]
pub trait SessionStorage: Send + Sync {
    async fn get_session(&self, id: &str) -> Result<Option<Session>>;
    async fn save_session(&self, session: &Session) -> Result<()>;
    async fn delete_session(&self, id: &str) -> Result<()>;
    async fn list_sessions(&self, filter: SessionFilter) -> Result<Vec<Session>>;
    async fn cleanup_expired(&self) -> Result<usize>;
}

pub struct SessionFilter {
    pub user_id: Option<String>,
    pub tenant_id: Option<String>,
    pub active_only: bool,
}
```

**Task 1.4.2: Implement PostgreSQL Adapter**

**File:** `crates/riptide-persistence/src/adapters/postgres_session_storage.rs` (NEW)

```rust
use riptide_types::ports::SessionStorage;

pub struct PostgresSessionStorage {
    pool: Arc<PgPool>,
}

#[async_trait]
impl SessionStorage for PostgresSessionStorage {
    async fn get_session(&self, id: &str) -> Result<Option<Session>> {
        // Anti-corruption: SQL -> Domain Session type
    }

    async fn save_session(&self, session: &Session) -> Result<()> {
        // Anti-corruption: Domain Session -> SQL
    }
}
```

**Task 1.4.3: Move Session Manager to Facade**

**File:** `crates/riptide-facade/src/facades/session.rs` (NEW - move from API)

```rust
pub struct SessionFacade {
    storage: Arc<dyn SessionStorage>,
    idempotency: Arc<dyn IdempotencyStore>,
    event_bus: Arc<dyn EventBus>,
}

impl SessionFacade {
    pub async fn create_session(
        &self,
        user_id: &str,
        authz_ctx: &AuthorizationContext,
    ) -> Result<Session> {
        // Business logic for session creation
        // Moved from sessions/manager.rs
    }
}
```

**Files Modified:**
```
CREATE:  crates/riptide-types/src/ports/session.rs (~100 LOC)
CREATE:  crates/riptide-persistence/src/adapters/postgres_session_storage.rs (~250 LOC)
CREATE:  crates/riptide-facade/src/facades/session.rs (~450 LOC - moved from API)
UPDATE:  crates/riptide-api/src/sessions/storage.rs (delete implementation, use port)
UPDATE:  crates/riptide-api/src/sessions/middleware.rs (use SessionStorage trait)
DELETE:  crates/riptide-api/src/sessions/manager.rs (moved to facade)
UPDATE:  crates/riptide-api/src/composition.rs (add session_storage port)
```

**LOC Impact:** +800 (ports + adapters), -1,400 (move to facade)

---

## Sprint 1.5: Core Infrastructure Ports (NEW - Week 3, Days 13-15)

**Priority:** MEDIUM
**Source:** API_CRATE_COVERAGE_ANALYSIS.md - Gap #9

### Problem

- `health.rs` (952 LOC) separate health system - direct service checks
- `metrics.rs` (1,670 LOC) business + transport metrics mixed
- `rpc_*.rs` (1,144 LOC) not using ports pattern
- `persistence_adapter.rs` (485 LOC) should use Repository trait

### Tasks

**Task 1.5.1: Define Health Check Port**

**File:** `crates/riptide-types/src/ports/health.rs` (NEW)

```rust
#[async_trait]
pub trait HealthCheck: Send + Sync {
    async fn check(&self) -> Result<HealthStatus>;
    fn name(&self) -> &str;
}

pub enum HealthStatus {
    Healthy,
    Degraded { reason: String },
    Unhealthy { error: String },
}

#[async_trait]
pub trait HealthRegistry: Send + Sync {
    async fn register(&self, check: Arc<dyn HealthCheck>);
    async fn check_all(&self) -> Vec<(String, HealthStatus)>;
}
```

**Task 1.5.2: Define Metrics Port**

**File:** `crates/riptide-types/src/ports/metrics.rs` (NEW)

```rust
pub trait MetricsCollector: Send + Sync {
    fn record_counter(&self, name: &str, value: u64, tags: &[(&str, &str)]);
    fn record_histogram(&self, name: &str, value: f64, tags: &[(&str, &str)]);
    fn record_gauge(&self, name: &str, value: f64, tags: &[(&str, &str)]);
}

pub trait BusinessMetrics: Send + Sync {
    fn record_extraction_completed(&self, duration: Duration, success: bool);
    fn record_profile_created(&self, tenant_id: &str);
    fn record_session_started(&self, user_id: &str);
}
```

**Task 1.5.3: Define RPC Client Port**

**File:** `crates/riptide-types/src/ports/rpc.rs` (NEW)

```rust
#[async_trait]
pub trait RpcClient: Send + Sync {
    async fn call<Req, Resp>(&self, method: &str, request: Req) -> Result<Resp>
    where
        Req: Serialize + Send,
        Resp: DeserializeOwned;
}
```

**Files Modified:**
```
CREATE:  crates/riptide-types/src/ports/health.rs (~100 LOC)
CREATE:  crates/riptide-types/src/ports/metrics.rs (~150 LOC)
CREATE:  crates/riptide-types/src/ports/rpc.rs (~80 LOC)
UPDATE:  crates/riptide-api/src/health.rs (impl HealthRegistry)
UPDATE:  crates/riptide-api/src/metrics.rs (split business/transport)
UPDATE:  crates/riptide-api/src/rpc_client.rs (impl RpcClient)
UPDATE:  crates/riptide-api/src/persistence_adapter.rs (use Repository<T>)
```

**LOC Impact:** +400 (ports), -2,900 (refactor)

---

## Phase 1 Summary

### Duration Breakdown

| Sprint | Duration | LOC Added | LOC Deleted |
|--------|----------|-----------|-------------|
| 1.1: Core Ports | 5 days | 600 | 0 |
| 1.2: Adapters | 5 days | 1,200 | 0 |
| 1.3: Composition Root | 5 days | 600 | 0 |
| 1.4: Session Port | 2 days | 800 | 1,400 |
| 1.5: Core Infrastructure | 3 days | 400 | 2,900 |
| **Total** | **20 days (3 weeks)** | **3,600** | **4,300** |

### Total Impact

**Original Plan (Sprints 1.1-1.3 only):**
- Duration: 2 weeks
- LOC Impact: +1,800 LOC added

**Enhanced Plan (All Sprints):**
- **Duration:** 3 weeks (50% increase)
- **LOC Impact:** +3,600 added, -4,300 deleted (net -700 LOC)
- **Ports Defined:** 15+ trait definitions
- **Adapters Implemented:** 12+ implementations

### Success Criteria

- ✅ All ports defined (15+ traits)
- ✅ All adapters implemented
- ✅ ApplicationContext wires dependencies
- ✅ Zero direct infra usage in facades
- ✅ Tests use in-memory adapters
- ✅ Session management ported
- ✅ Health checks ported
- ✅ Metrics system ported
- ✅ RPC client ported

### Risk Mitigation

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Port interfaces too complex | MEDIUM | MEDIUM | Start simple, iterate based on usage |
| Adapter implementation bugs | HIGH | MEDIUM | Comprehensive unit tests per adapter |
| DI performance overhead | LOW | LOW | Benchmark ApplicationContext creation |
| Session migration breaks auth | MEDIUM | HIGH | Feature flag for gradual rollout |

---

## Dependencies

**Requires Completion Of:**
- [PHASE_0_CLEANUP_ROADMAP.md](./PHASE_0_CLEANUP_ROADMAP.md) - Must complete deduplication first

**Enables:**
- [PHASE_2_APPLICATION_LAYER_ROADMAP.md](./PHASE_2_APPLICATION_LAYER_ROADMAP.md) - Authorization, idempotency, events

---

## Document Status

**Version:** 2.0 (Enhanced with API Coverage Analysis)
**Status:** ✅ Ready for Implementation
**Date:** 2025-11-08
**Dependencies:** Phase 0 complete
**Next Review:** After Sprint 1.3 completion

**Related Documents:**
- [API_CRATE_COVERAGE_ANALYSIS.md](../architecture/API_CRATE_COVERAGE_ANALYSIS.md)
- [PHASE_2_APPLICATION_LAYER_ROADMAP.md](./PHASE_2_APPLICATION_LAYER_ROADMAP.md) (next phase)
