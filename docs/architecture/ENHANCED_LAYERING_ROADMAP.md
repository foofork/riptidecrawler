# Enhanced Layering Refactoring Roadmap - Riptide EventMesh
**Version:** 2.0 (Comprehensive)
**Date:** 2025-11-08
**Status:** Ready for Implementation
**Architect:** System Architecture Designer

---

## Executive Summary

This enhanced roadmap integrates **stricter architectural principles** with the existing 3-phase refactoring strategy, adding critical deduplication, ports & adapters, Redis consolidation, and enhanced validation requirements.

### Key Enhancements Over v1.0

1. **Stricter Layering**: Handlers <50 LOC (was 100), zero loops/conditionals
2. **Ports & Adapters**: All external dependencies via traits (DI at composition root)
3. **Application Layer**: Authorization, idempotency, transactional outbox, domain events
4. **Deduplication Targets**: Eliminate duplicate robots.rs, memory managers, cache implementations
5. **Redis Consolidation**: Single pooled client, scoped usage, graceful degradation
6. **Enhanced Validation**: Automated checks for HTTP/JSON leaks, oversized handlers, forbidden deps
7. **Stricter KPIs**: ≥90% facade coverage, 0 duplications, 0 violations

### Current State Analysis (Updated)

**Achievements:**
- ✅ Zero circular dependencies
- ✅ 12 domain types in facade_types.rs
- ✅ 6 facades created (5,742 LOC)
- ✅ ReliableHttpClient infrastructure exists

**Critical Issues Identified:**
- ❌ **2 duplicate robots.rs files** (riptide-fetch, riptide-spider)
- ❌ **3 duplicate memory managers** (3,213 LOC total: riptide-pool, riptide-spider, riptide-api)
- ❌ **6 crates with Redis dependencies** (should be 1-2 max)
- ❌ **3 facades with serde_json::Value** (browser, pipeline, extractor)
- ❌ **15 handlers >300 LOC** (largest: 945 LOC)
- ❌ **144 handler functions** with business logic

---

## Phase 0: Pre-Refactoring Cleanup (NEW - Week 0)

**Duration:** 3 days
**Priority:** CRITICAL (removes duplication before refactoring)

### Sprint 0.1: Deduplication & Consolidation (3 days)

#### Task 0.1.1: Split Robots.txt Implementation (1.5 days)

**Problem:** Duplicate robots.rs in riptide-fetch and riptide-spider (identical ~400 LOC each)

**Solution - Two-Part Split (Separation of Concerns):**

1. **Pure Logic → `riptide-utils/src/robots.rs`** (~200 LOC)
   - robots.txt parsing
   - Rule evaluation
   - **NO HTTP, NO async I/O** (pure functions only)

2. **HTTP/Retry → `riptide-reliability/src/robots_fetcher.rs`** (~200 LOC)
   - Circuit breaker
   - Retry logic
   - Timeout handling
   - Uses `ReliableHttpClient`

**Implementation:**

```rust
// crates/riptide-utils/src/robots.rs (NEW - Pure Parsing)
pub struct RobotsPolicy {
    rules: Vec<Rule>,
}

impl RobotsPolicy {
    /// Parse robots.txt content (pure function, no I/O)
    pub fn parse(content: &str) -> Result<Self> {
        // Parsing logic only
    }

    /// Check if URL is allowed for user agent (pure function)
    pub fn is_allowed(&self, path: &str, user_agent: &str) -> bool {
        // Rule evaluation logic
    }
}

// crates/riptide-reliability/src/robots_fetcher.rs (NEW - I/O Layer)
pub struct RobotsFetcher {
    http_client: Arc<ReliableHttpClient>,
    cache: DashMap<String, (RobotsPolicy, Instant)>,
}

impl RobotsFetcher {
    pub async fn fetch_policy(&self, domain: &str) -> Result<RobotsPolicy> {
        // Check cache first
        if let Some(cached) = self.get_cached(domain) {
            return Ok(cached);
        }

        // Fetch with circuit breaker and retry
        let content = self.http_client
            .get(&format!("https://{}/robots.txt", domain))
            .await?;

        // Parse using pure function from riptide-utils
        let policy = RobotsPolicy::parse(&content)?;

        // Cache result
        self.cache_policy(domain, policy.clone());

        Ok(policy)
    }
}
```

**Files Modified:**
```
CREATE:  crates/riptide-utils/src/robots.rs (~200 LOC - pure parsing)
CREATE:  crates/riptide-reliability/src/robots_fetcher.rs (~200 LOC - HTTP/retry)
UPDATE:  crates/riptide-utils/src/lib.rs (add pub mod robots)
UPDATE:  crates/riptide-reliability/src/lib.rs (add pub mod robots_fetcher)
UPDATE:  crates/riptide-reliability/Cargo.toml (add riptide-utils dependency)
UPDATE:  crates/riptide-fetch/Cargo.toml (add riptide-reliability + riptide-utils deps)
UPDATE:  crates/riptide-spider/Cargo.toml (add riptide-reliability + riptide-utils deps)
UPDATE:  crates/riptide-fetch/src/lib.rs (use reliability::robots_fetcher + utils::robots)
UPDATE:  crates/riptide-spider/src/lib.rs (use reliability::robots_fetcher + utils::robots)
DELETE:  crates/riptide-fetch/src/robots.rs
DELETE:  crates/riptide-spider/src/robots.rs
```

**Validation:**
```bash
# Ensure pure logic in utils
grep -r "async\|await\|http" crates/riptide-utils/src/robots.rs && echo "FAIL: I/O found in pure code" || echo "PASS"

# Ensure single source of truth (2 files: utils + reliability)
find crates -name "robots*.rs" | wc -l  # Expected: 2 (robots.rs in utils, robots_fetcher.rs in reliability)

# Ensure tests still pass
cargo test -p riptide-utils
cargo test -p riptide-reliability
cargo test -p riptide-fetch
cargo test -p riptide-spider
```

**LOC Reduction:** ~400 LOC deleted (50% reduction)
**Architectural Benefit:** Pure business logic separated from infrastructure

#### Task 0.1.2: Consolidate Memory Managers (1 day)

**Problem:** 3 duplicate memory managers (3,213 LOC total)
- `riptide-pool/src/memory_manager.rs` (WASM-specific)
- `riptide-spider/src/memory_manager.rs` (WASM-specific, near-identical)
- `riptide-api/src/resource_manager/memory_manager.rs` (HTTP-specific)

**Solution:**
```
Keep:   riptide-pool/src/memory_manager.rs (most feature-complete)
Enhance: Add HTTP resource tracking capabilities
Migrate: riptide-spider to use pool::MemoryManager
Migrate: riptide-api to use pool::MemoryManager
Delete: riptide-spider/src/memory_manager.rs
Delete: riptide-api/src/resource_manager/memory_manager.rs
```

**Implementation:**
```rust
// crates/riptide-pool/src/memory_manager.rs (ENHANCED)
pub enum ResourceType {
    WasmInstance { component: Arc<Component> },
    HttpConnection { pool_size: usize },
    BrowserSession { session_id: String },
    PdfProcessor { slots_used: usize },
}

pub struct UnifiedMemoryManager {
    wasm_pool: WasmPoolManager,      // Existing
    http_resources: HttpResourceTracker,  // NEW
    metrics: Arc<MemoryStats>,
}
```

**Files Modified:**
```
UPDATE:  crates/riptide-pool/src/memory_manager.rs (~300 LOC added)
UPDATE:  crates/riptide-pool/Cargo.toml (add feature flags)
UPDATE:  crates/riptide-spider/src/lib.rs (use pool::MemoryManager)
UPDATE:  crates/riptide-api/src/state.rs (use pool::MemoryManager)
DELETE:  crates/riptide-spider/src/memory_manager.rs (~1,100 LOC)
DELETE:  crates/riptide-api/src/resource_manager/memory_manager.rs (~800 LOC)
```

**Validation:**
```bash
# Ensure single source of truth
rg "struct MemoryManager" crates/ | wc -l  # Expected: 1 (in riptide-pool)

# Tests pass
cargo test -p riptide-pool
cargo test -p riptide-spider
cargo test -p riptide-api
```

**LOC Reduction:** ~1,900 LOC deleted (60% reduction in memory manager code)

#### Task 0.1.3: Audit & Scope Redis Dependencies (1 day)

**Problem:** 6 crates with Redis dependencies (should be 1-2)
```
riptide-utils/Cargo.toml
riptide-cache/Cargo.toml
riptide-persistence/Cargo.toml
riptide-workers/Cargo.toml
riptide-api/Cargo.toml
riptide-performance/Cargo.toml
```

**Correct Architecture:**
```
✅ ALLOWED (Choose ONE pattern - Maximum 2 crates total):

  Pattern A (Job Queue Architecture):
    - riptide-cache       (cache, idempotency, rate limits, short locks)
    - riptide-workers     (job queues via Redis Streams)

  Pattern B (Event Outbox Architecture):
    - riptide-cache       (cache, idempotency, rate limits, short locks)
    - riptide-persistence (outbox event polling via Redis pub/sub - OPTIONAL)

❌ FORBIDDEN (Use CacheStorage trait instead):
  riptide-utils       (should use cache abstraction)
  riptide-api         (should inject CacheStorage trait)
  riptide-performance (metrics should go to TSDB, not Redis)

**Decision Point:** Choose Pattern A or B based on your event architecture.
**Constraint:** MAXIMUM 2 crates with direct Redis dependency.
```

**Implementation:**
```rust
// Define port in riptide-types
pub trait CacheStorage: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn set(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn exists(&self, key: &str) -> Result<bool>;
}

// Implementation in riptide-cache
impl CacheStorage for RedisCache {
    // Redis-specific implementation
}
```

**Files Modified:**
```
CREATE:  crates/riptide-types/src/ports/cache.rs (CacheStorage trait)
UPDATE:  crates/riptide-cache/src/lib.rs (impl CacheStorage)
UPDATE:  crates/riptide-utils/Cargo.toml (REMOVE redis dependency)
UPDATE:  crates/riptide-persistence/Cargo.toml (REMOVE redis dependency)
UPDATE:  crates/riptide-api/Cargo.toml (REMOVE redis dependency)
UPDATE:  crates/riptide-performance/Cargo.toml (REMOVE redis dependency)
```

**Validation:**
```bash
# Only 2 crates should have Redis
find crates -name "Cargo.toml" -exec grep -l "redis" {} \; | wc -l  # Expected: 2

# No direct Redis usage outside cache/workers
rg "redis::" crates/riptide-{utils,persistence,api,performance} || echo "PASS: No Redis found"
```

**Success Criteria:**
- ✅ Only riptide-cache and riptide-workers depend on Redis
- ✅ All other crates use CacheStorage trait
- ✅ Zero Redis imports outside infrastructure layer
- ✅ All tests pass after migration

**Total Phase 0 Impact:**
- **2,300+ LOC deleted** (duplications removed)
- **4 Redis dependencies removed** (from 6 to 2)
- **3 files consolidated** into single sources of truth
- **Foundation for ports & adapters** (CacheStorage trait defined)

---

## Phase 1: Ports & Adapters Foundation (NEW - Week 1-2)

**Duration:** 2 weeks
**Goal:** Define all infrastructure ports before handler refactoring

### Sprint 1.1: Core Infrastructure Ports (Week 1)

#### Define Repository Ports

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

#### Define Event Bus Port

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

#### Define Idempotency Port

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

#### Define Feature Ports

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

#### Define Infrastructure Ports

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

**Files Created in Sprint 1.1:**
```
CREATE: crates/riptide-types/src/ports/mod.rs
CREATE: crates/riptide-types/src/ports/repository.rs
CREATE: crates/riptide-types/src/ports/events.rs
CREATE: crates/riptide-types/src/ports/idempotency.rs
CREATE: crates/riptide-types/src/ports/features.rs
CREATE: crates/riptide-types/src/ports/infrastructure.rs
UPDATE: crates/riptide-types/src/lib.rs
```

**Total:** ~600 LOC (trait definitions)

#### Task 1.1.4: Document Facade Layer Architectural Rules (0.5 days)

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

### Sprint 1.2: Implement Adapters (Week 2)

#### PostgreSQL Repository Adapter

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

#### Redis Idempotency Adapter

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

#### Event Bus Adapters

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

**Files Created in Sprint 1.2:**
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

### Sprint 1.3: Composition Root (Dependency Injection)

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

**AppState Refactor:**

```rust
// crates/riptide-api/src/state.rs (REFACTORED)
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

**Success Criteria for Phase 1:**
- ✅ All ports defined in riptide-types/ports
- ✅ All adapters implement port traits
- ✅ Dependencies wired at composition root (main.rs)
- ✅ Zero direct Redis/SQL usage outside adapters
- ✅ Facades depend only on ports (traits), not concrete types
- ✅ Test context uses in-memory implementations

---

## Phase 2: Application Layer Enhancements (Week 3-4)

**Duration:** 2 weeks
**Goal:** Add authorization, idempotency, events, and transactions

### Sprint 2.1: Authorization Policies (Week 3, Days 1-2)

**File:** `crates/riptide-facade/src/authorization/mod.rs` (NEW)

```rust
use riptide_types::ports::Repository;

pub struct AuthorizationContext {
    pub user_id: String,
    pub tenant_id: String,
    pub roles: Vec<String>,
    pub permissions: HashSet<String>,
}

pub trait AuthorizationPolicy: Send + Sync {
    fn authorize(&self, ctx: &AuthorizationContext, resource: &Resource) -> Result<()>;
}

pub struct TenantScopingPolicy;

impl AuthorizationPolicy for TenantScopingPolicy {
    fn authorize(&self, ctx: &AuthorizationContext, resource: &Resource) -> Result<()> {
        if resource.tenant_id != ctx.tenant_id {
            Err(RiptideError::Unauthorized {
                reason: "Cross-tenant access denied".to_string()
            })
        } else {
            Ok(())
        }
    }
}

pub struct RbacPolicy {
    required_permission: String,
}

impl AuthorizationPolicy for RbacPolicy {
    fn authorize(&self, ctx: &AuthorizationContext, resource: &Resource) -> Result<()> {
        if ctx.permissions.contains(&self.required_permission) {
            Ok(())
        } else {
            Err(RiptideError::Forbidden {
                permission: self.required_permission.clone()
            })
        }
    }
}
```

**Facade Integration:**

```rust
// crates/riptide-facade/src/facades/extraction.rs (ENHANCED)
pub struct ExtractionFacade {
    extractor: Arc<dyn BrowserDriver>,
    cache: Arc<dyn CacheStorage>,
    event_bus: Arc<dyn EventBus>,
    authz_policies: Vec<Box<dyn AuthorizationPolicy>>,  // NEW
}

impl ExtractionFacade {
    pub async fn extract_content(
        &self,
        url: &str,
        authz_ctx: &AuthorizationContext,  // NEW
    ) -> Result<ExtractedData> {
        // 1. Authorization checks (Application layer)
        for policy in &self.authz_policies {
            policy.authorize(authz_ctx, &Resource::Url(url.to_string()))?;
        }

        // 2. Business logic
        let data = self.extractor.extract(url).await?;

        // 3. Emit domain event
        self.event_bus.publish(DomainEvent {
            event_type: "content.extracted".to_string(),
            aggregate_id: url.to_string(),
            payload: serde_json::to_value(&data)?,
            // ...
        }).await?;

        Ok(data)
    }
}
```

### Sprint 2.2: Idempotency & Transactions (Week 3, Days 3-4)

**File:** `crates/riptide-facade/src/workflows/transactional.rs` (NEW)

```rust
use riptide_types::ports::{TransactionManager, EventBus, IdempotencyStore};

pub struct TransactionalWorkflow<T> {
    tx_manager: Arc<dyn TransactionManager>,
    event_bus: Arc<dyn EventBus>,
    idempotency_store: Arc<dyn IdempotencyStore>,
    _phantom: PhantomData<T>,
}

impl<T> TransactionalWorkflow<T> {
    pub async fn execute<F, R>(
        &self,
        idempotency_key: &str,
        workflow_fn: F,
    ) -> Result<R>
    where
        F: FnOnce(&mut dyn Transaction) -> BoxFuture<'_, Result<(R, Vec<DomainEvent>)>>,
    {
        // 1. Check idempotency
        let idem_token = self.idempotency_store
            .try_acquire(idempotency_key, Duration::from_secs(300))
            .await?;

        // 2. Begin transaction
        let mut tx = self.tx_manager.begin().await?;

        // 3. Execute workflow (returns result + events)
        let (result, events) = match workflow_fn(&mut tx).await {
            Ok(r) => r,
            Err(e) => {
                self.tx_manager.rollback(tx).await?;
                self.idempotency_store.release(idem_token).await?;
                return Err(e);
            }
        };

        // 4. Write events to outbox (transactional)
        for event in events {
            self.event_bus.publish(event).await?;
        }

        // 5. Commit transaction
        self.tx_manager.commit(tx).await?;

        // 6. Release idempotency lock
        self.idempotency_store.release(idem_token).await?;

        Ok(result)
    }
}
```

**Usage in Facade:**

```rust
// crates/riptide-facade/src/facades/profile.rs (ENHANCED)
impl ProfileFacade {
    pub async fn create_profile(
        &self,
        request: CreateProfileRequest,
        authz_ctx: &AuthorizationContext,
    ) -> Result<Profile> {
        let idempotency_key = format!("profile:create:{}", request.user_id);

        self.workflow.execute(&idempotency_key, |tx| async move {
            // Business logic within transaction
            let profile = Profile {
                id: self.context.entropy.random_id(),
                user_id: request.user_id.clone(),
                tenant_id: authz_ctx.tenant_id.clone(),
                created_at: self.context.clock.now(),
                // ...
            };

            // Save to repository (within TX)
            self.profile_repository.save(&profile).await?;

            // Emit domain event (written to outbox in same TX)
            let events = vec![DomainEvent {
                event_type: "profile.created".to_string(),
                aggregate_id: profile.id.clone(),
                payload: serde_json::to_value(&profile)?,
                // ...
            }];

            Ok((profile, events))
        }.boxed()).await
    }
}
```

### Sprint 2.3: Backpressure & Cancellation (Week 4, Days 1-2)

**File:** `crates/riptide-facade/src/workflows/backpressure.rs` (NEW)

```rust
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;

pub struct BackpressureManager {
    semaphore: Arc<Semaphore>,
    active_count: Arc<AtomicUsize>,
    max_concurrency: usize,
}

impl BackpressureManager {
    pub async fn acquire(&self, cancel_token: &CancellationToken) -> Result<BackpressureGuard> {
        tokio::select! {
            permit = self.semaphore.acquire() => {
                let permit = permit.map_err(|_| RiptideError::ResourceExhausted)?;
                self.active_count.fetch_add(1, Ordering::SeqCst);

                Ok(BackpressureGuard {
                    _permit: permit.into(),
                    active_count: self.active_count.clone(),
                })
            }
            _ = cancel_token.cancelled() => {
                Err(RiptideError::Cancelled)
            }
        }
    }

    pub fn current_load(&self) -> f64 {
        self.active_count.load(Ordering::SeqCst) as f64 / self.max_concurrency as f64
    }
}

pub struct BackpressureGuard {
    _permit: OwnedSemaphorePermit,
    active_count: Arc<AtomicUsize>,
}

impl Drop for BackpressureGuard {
    fn drop(&mut self) {
        self.active_count.fetch_sub(1, Ordering::SeqCst);
    }
}
```

### Sprint 2.4: Business Metrics (Week 4, Days 3-4)

**File:** `crates/riptide-facade/src/metrics/business.rs` (NEW)

```rust
use prometheus::{Counter, Histogram, IntGauge};

pub struct BusinessMetrics {
    // Domain-level metrics (not transport)
    profiles_created: Counter,
    extractions_completed: Counter,
    extractions_duration: Histogram,
    active_sessions: IntGauge,

    // Business SLOs
    extraction_success_rate: Counter,
    extraction_failure_rate: Counter,
}

impl BusinessMetrics {
    pub fn record_extraction_completed(&self, duration: Duration, success: bool) {
        self.extractions_completed.inc();
        self.extractions_duration.observe(duration.as_secs_f64());

        if success {
            self.extraction_success_rate.inc();
        } else {
            self.extraction_failure_rate.inc();
        }
    }
}
```

**Success Criteria for Phase 2:**
- ✅ Authorization policies enforced in all facades
- ✅ Idempotency keys at application entry points
- ✅ Transactional workflows with outbox pattern
- ✅ Backpressure + cancellation tokens
- ✅ Business metrics (not just transport metrics)
- ✅ Domain events emitted from entities, published via EventBus

---

## Phase 3: Handler Refactoring (<50 LOC Target) (Week 5-6)

**Duration:** 2 weeks
**Goal:** Ultra-thin handlers with ZERO business logic

### Stricter Handler Requirements

**<50 LOC Target:**
- ✅ Extract HTTP body/query params (5-10 LOC)
- ✅ Validate format (URL parsing, bounds checks) (5-10 LOC)
- ✅ Map DTO → Domain types (5-10 LOC)
- ✅ Call facade (1 LOC)
- ✅ Map Domain → DTO response (5-10 LOC)
- ✅ Return HTTP response (1 LOC)
- ❌ ZERO loops
- ❌ ZERO conditionals (except input validation)
- ❌ ZERO multi-step orchestration

### Handler Conditional Logic Rules

**CLARIFICATION:** Simple `if` statements for I/O validation are ALLOWED. Business logic conditionals are FORBIDDEN.

**✅ ALLOWED - Input Validation (I/O concerns):**
```rust
// Format validation
if req.url.is_empty() {
    return Err(ApiError::invalid_request("URL required"));
}

// Bounds checking
if req.size > MAX_SIZE {
    return Err(ApiError::payload_too_large(req.size, MAX_SIZE));
}

// Type validation
if !req.content_type.starts_with("application/") {
    return Err(ApiError::unsupported_media_type(req.content_type));
}
```

**❌ FORBIDDEN - Business Logic:**
```rust
// Business logic loops (belongs in facade)
for url in urls {
    process_url(url).await?;
}

// Multi-step orchestration (belongs in facade)
while condition {
    let result = complex_operation().await?;
    if result.needs_retry {
        retry_count += 1;
    }
}

// Complex conditional trees (belongs in facade)
if user.premium_tier {
    if feature_enabled(&user, "advanced_extraction") {
        // Complex business rules
    }
}
```

**Rule of Thumb:** If the conditional checks HTTP input format/bounds → ✅ ALLOWED.
If it implements business rules or orchestration → ❌ Move to facade.

**Example (BEFORE vs AFTER):**

```rust
// ❌ BEFORE: 349 LOC, business logic in handler
pub async fn process_pdf(
    State(state): State<AppState>,
    Json(req): Json<PdfRequest>,
) -> impl IntoResponse {
    // 1. Base64 decoding (20 LOC)
    let pdf_data = match base64::decode(&req.pdf_base64) {
        Ok(data) => data,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(json!({ "error": e.to_string() }))),
    };

    // 2. PDF validation (30 LOC)
    if pdf_data.len() > MAX_PDF_SIZE { /* ... */ }
    if !pdf_data.starts_with(b"%PDF") { /* ... */ }

    // 3. Resource acquisition (40 LOC)
    if state.active_pdf_processes.load(Ordering::SeqCst) >= state.config.pdf_concurrency {
        return (StatusCode::TOO_MANY_REQUESTS, /* ... */);
    }
    state.active_pdf_processes.fetch_add(1, Ordering::SeqCst);

    // 4. Progress streaming (120 LOC)
    let (tx, rx) = mpsc::channel(100);
    tokio::spawn(async move {
        // Complex PDF processing logic
        // ...
    });

    // 5. Cleanup (30 LOC)
    // ...

    // Total: 349 LOC with business logic
}

// ✅ AFTER: 35 LOC, I/O only
pub async fn process_pdf(
    State(state): State<AppState>,
    AuthContext(authz): AuthContext,  // Middleware extracts
    Json(req): Json<PdfRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // 1. Format validation (HTTP concern)
    let pdf_data = base64::decode(&req.pdf_base64)
        .map_err(|e| ApiError::invalid_request("Invalid base64", e))?;

    // 2. Input bounds check
    if pdf_data.len() > MAX_PDF_SIZE {
        return Err(ApiError::payload_too_large(pdf_data.len(), MAX_PDF_SIZE));
    }

    // 3. Map DTO → Domain
    let options = PdfProcessingOptions {
        extract_images: req.extract_images.unwrap_or(true),
        page_range: req.pages,
        // ...
    };

    // 4. Call facade (all business logic)
    let stream = state.pdf_facade
        .process_pdf_stream(pdf_data, options, &authz)
        .await?;

    // 5. Return HTTP stream
    Ok(Sse::new(stream))
}
// Total: 35 LOC, zero business logic
```

### Sprint 3.1: Large Handler Migrations (Week 5)

**Priority Targets (Top 10):**

| Handler | Current LOC | Target LOC | Reduction |
|---------|-------------|------------|-----------|
| trace_backend.rs | 945 | 40 | -96% |
| llm.rs | 863 | 45 | -95% |
| browser.rs | 695 | 35 | -95% |
| profiling.rs | 646 | 30 | -95% |
| workers.rs | 639 | 35 | -95% |
| profiles.rs | 582 | 30 | -95% |
| engine_selection.rs | 500 | 30 | -94% |
| sessions.rs | 450 | 25 | -94% |
| tables.rs | 363 | 30 | -92% |
| pdf.rs | 349 | 35 | -90% |

**Implementation Pattern:**

```rust
// Step 1: Create facade (if not exists)
crates/riptide-facade/src/facades/trace.rs (NEW)

pub struct TraceFacade {
    telemetry_backend: Arc<dyn TelemetryBackend>,
    tx_manager: Arc<dyn TransactionManager>,
    event_bus: Arc<dyn EventBus>,
    authz_policies: Vec<Box<dyn AuthorizationPolicy>>,
}

impl TraceFacade {
    pub async fn submit_trace(
        &self,
        trace_data: TraceData,
        authz_ctx: &AuthorizationContext,
    ) -> Result<TraceId> {
        // 1. Authorization
        self.authorize(authz_ctx, &trace_data)?;

        // 2. Idempotency
        let idem_key = format!("trace:{}", trace_data.trace_id);

        // 3. Transactional workflow
        self.workflow.execute(&idem_key, |tx| async {
            // Business logic: validate, transform, store
            let trace_id = self.store_trace(&trace_data, tx).await?;

            // Emit event
            let event = DomainEvent {
                event_type: "trace.submitted".to_string(),
                aggregate_id: trace_id.clone(),
                // ...
            };

            Ok((trace_id, vec![event]))
        }).await
    }
}

// Step 2: Refactor handler to <50 LOC
crates/riptide-api/src/handlers/trace_backend.rs (REFACTORED)

pub async fn submit_trace(
    State(state): State<AppState>,
    AuthContext(authz): AuthContext,
    Json(req): Json<SubmitTraceRequest>,
) -> Result<Json<TraceResponse>, ApiError> {
    // Validate format
    let trace_data = TraceData::try_from(req)?;

    // Call facade
    let trace_id = state.trace_facade
        .submit_trace(trace_data, &authz)
        .await?;

    // Return response
    Ok(Json(TraceResponse { trace_id }))
}
// Total: 15 LOC
```

**Files to Create:**
```
CREATE: crates/riptide-facade/src/facades/trace.rs (~800 LOC)
CREATE: crates/riptide-facade/src/facades/llm.rs (~750 LOC)
CREATE: crates/riptide-facade/src/facades/profiling.rs (~550 LOC)
CREATE: crates/riptide-facade/src/facades/workers.rs (~500 LOC)
CREATE: crates/riptide-facade/src/facades/engine.rs (~400 LOC)
```

**Files to Refactor:**
```
UPDATE: crates/riptide-api/src/handlers/trace_backend.rs (945 → 40 LOC)
UPDATE: crates/riptide-api/src/handlers/llm.rs (863 → 45 LOC)
UPDATE: crates/riptide-api/src/handlers/browser.rs (695 → 35 LOC)
UPDATE: crates/riptide-api/src/handlers/profiling.rs (646 → 30 LOC)
UPDATE: crates/riptide-api/src/handlers/workers.rs (639 → 35 LOC)
UPDATE: crates/riptide-api/src/handlers/profiles.rs (582 → 30 LOC)
UPDATE: crates/riptide-api/src/handlers/engine_selection.rs (500 → 30 LOC)
UPDATE: crates/riptide-api/src/handlers/sessions.rs (450 → 25 LOC)
UPDATE: crates/riptide-api/src/handlers/tables.rs (363 → 30 LOC)
UPDATE: crates/riptide-api/src/handlers/pdf.rs (349 → 35 LOC)
```

**LOC Impact:**
- **5,532 LOC moved** to facades
- **Handlers reduced** to 375 LOC total (from 5,907)
- **93.6% handler LOC reduction**

### Sprint 3.2: Domain Type Migration (Week 6)

**Remove serde_json::Value from facades (35 instances)**

Already covered in existing roadmap Sprints 2.1-2.3 (pipeline, browser, extractor).

**Additional Requirement:** Ensure NO serde_json::Value in ANY facade after completion.

**Validation:**
```bash
rg "serde_json::Value" crates/riptide-facade/src/facades/ || echo "PASS: No JSON found"
```

---

## Phase 4: Infrastructure Consolidation (Week 7)

**Duration:** 1 week
**Goal:** Centralize HTTP clients, caching, Redis in reliability layer

### Sprint 4.1: HTTP Client Consolidation

Already covered in existing roadmap Sprint 3.1, but enhanced with:

**Additional Requirements:**
- ✅ ALL HTTP clients MUST use ReliableHttpClient
- ✅ Circuit breakers per endpoint type (6 presets)
- ✅ Retry with exponential backoff + jitter
- ✅ Request/response interceptors for telemetry

**Validation:**
```bash
# No direct reqwest usage outside reliability
rg "reqwest::Client::new" crates/riptide-{facade,api,spider} && echo "FAIL" || echo "PASS"
```

### Sprint 4.2: Redis Consolidation (NEW)

**Goal:** Single pooled Redis client, scoped usage, versioned keys

**Redis Usage Scope (Whitelist):**
- ✅ Cache: short-lived data (TTL 1 hour - 1 day)
- ✅ Idempotency keys: request deduplication (TTL 5 minutes)
- ✅ Rate limits: per-user/tenant quotas (TTL 1 minute)
- ✅ Short-lived locks: resource coordination (TTL 10 seconds)
- ❌ NOT for: primary persistence (use PostgreSQL)
- ❌ NOT for: event bus (use transactional outbox)
- ❌ NOT for: long-lived state (use DB)

**Implementation:**

```rust
// crates/riptide-cache/src/redis_manager.rs (NEW)
pub struct RedisManager {
    pool: deadpool_redis::Pool,
    key_prefix: String,
    version: String,
}

impl RedisManager {
    pub fn versioned_key(&self, namespace: &str, key: &str) -> String {
        format!("{}:{}:{}:{}", self.key_prefix, self.version, namespace, key)
    }

    pub async fn set_with_ttl(&self, namespace: &str, key: &str, value: &[u8], ttl: Duration) -> Result<()> {
        let versioned_key = self.versioned_key(namespace, key);

        self.pool.get().await?
            .set_ex(&versioned_key, value, ttl.as_secs() as usize)
            .await
            .map_err(|e| {
                // Treat Redis as optional - log but don't fail
                warn!("Redis set failed (degraded mode): {}", e);
                RiptideError::CacheDegraded
            })
    }

    pub async fn get(&self, namespace: &str, key: &str) -> Result<Option<Vec<u8>>> {
        let versioned_key = self.versioned_key(namespace, key);

        self.pool.get().await?
            .get(&versioned_key)
            .await
            .map_err(|e| {
                warn!("Redis get failed (degraded mode): {}", e);
                Ok(None)  // Graceful degradation: return cache miss
            })?
    }

    pub async fn atomic_lock(&self, key: &str, ttl: Duration) -> Result<RedisLock> {
        let lock_key = self.versioned_key("locks", key);
        let lock_value = Uuid::new_v4().to_string();

        let acquired: bool = self.pool.get().await?
            .set_nx_ex(&lock_key, &lock_value, ttl.as_secs() as usize)
            .await?;

        if acquired {
            Ok(RedisLock {
                key: lock_key,
                value: lock_value,
                pool: self.pool.clone(),
            })
        } else {
            Err(RiptideError::LockContention { key: key.to_string() })
        }
    }
}

pub struct RedisLock {
    key: String,
    value: String,
    pool: deadpool_redis::Pool,
}

impl Drop for RedisLock {
    fn drop(&mut self) {
        // Safe unlock with Lua script (check value before delete)
        let script = r#"
            if redis.call("get", KEYS[1]) == ARGV[1] then
                return redis.call("del", KEYS[1])
            else
                return 0
            end
        "#;

        let pool = self.pool.clone();
        let key = self.key.clone();
        let value = self.value.clone();

        tokio::spawn(async move {
            let _ = pool.get().await
                .and_then(|conn| conn.eval(script, &[&key], &[&value]));
        });
    }
}
```

**Key Namespaces:**
```
cache:v1:sessions:*        (session data, TTL 1 hour)
cache:v1:extractions:*     (extraction results, TTL 6 hours)
idempotency:v1:*           (request keys, TTL 5 minutes)
ratelimit:v1:user:*        (user quotas, TTL 1 minute)
locks:v1:*                 (short locks, TTL 10 seconds)
```

**Migration:**
```
REMOVE: Direct redis::Client usage from all facades
UPDATE: All cache access via CacheStorage trait
UPDATE: All idempotency via IdempotencyStore trait
UPDATE: All rate limiting via centralized middleware
```

**Validation:**
```bash
# No direct redis usage outside cache crate
rg "redis::" crates/riptide-{facade,api,persistence,utils,performance} && echo "FAIL" || echo "PASS"

# Only 2 crates depend on redis
find crates -name "Cargo.toml" -exec grep -l "redis" {} \; | wc -l  # Expected: 2
```

---

## Phase 5: Enhanced Validation Automation (Week 8)

**Duration:** 3 days
**Goal:** Comprehensive CI/CD checks for architectural violations

### Enhanced validate_architecture.sh

**File:** `/workspaces/eventmesh/scripts/validate_architecture.sh` (ENHANCED)

```bash
#!/bin/bash
set -e

FAIL_COUNT=0

echo "🔍 Enhanced Architecture Validation"
echo "===================================="

# 1. Handler size limits (<50 LOC strict)
echo ""
echo "📏 Checking handler sizes (max: 50 LOC)..."
for file in crates/riptide-api/src/handlers/*.rs; do
    lines=$(wc -l < "$file" | tr -d ' ')
    if [ "$lines" -gt 50 ]; then
        echo "❌ FAIL: $(basename "$file") has $lines lines (max: 50)"
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
done

# 2. HTTP types in facades (ZERO tolerance)
echo ""
echo "🌐 Checking for HTTP types in facades..."
if rg "actix_web::|axum::|HttpMethod|HeaderMap" crates/riptide-facade/src/facades/ >/dev/null 2>&1; then
    echo "❌ FAIL: HTTP types found in facade layer"
    rg "actix_web::|axum::|HttpMethod|HeaderMap" crates/riptide-facade/src/facades/ --files-with-matches
    FAIL_COUNT=$((FAIL_COUNT + 1))
else
    echo "✅ PASS: No HTTP types in facades"
fi

# 3. JSON in facades (ZERO tolerance)
echo ""
echo "📋 Checking for serde_json::Value in facades..."
if rg "serde_json::Value" crates/riptide-facade/src/facades/ >/dev/null 2>&1; then
    echo "❌ FAIL: Untyped JSON found in facades"
    rg "serde_json::Value" crates/riptide-facade/src/facades/ --files-with-matches
    FAIL_COUNT=$((FAIL_COUNT + 1))
else
    echo "✅ PASS: No untyped JSON in facades"
fi

# 4. Forbidden facade dependencies
echo ""
echo "📦 Checking facade dependencies..."
FORBIDDEN_DEPS=$(cargo tree -p riptide-facade --depth 1 | grep -E "axum|actix-web|tower-http|hyper" || true)
if [ -n "$FORBIDDEN_DEPS" ]; then
    echo "❌ FAIL: Forbidden dependencies in riptide-facade:"
    echo "$FORBIDDEN_DEPS"
    FAIL_COUNT=$((FAIL_COUNT + 1))
else
    echo "✅ PASS: Facade dependencies clean"
fi

# 5. Domain depends on infra (forbidden)
echo ""
echo "🏗️  Checking domain layer purity..."
DOMAIN_INFRA_DEPS=$(cargo tree -p riptide-types | grep -E "riptide-(api|facade)" || true)
if [ -n "$DOMAIN_INFRA_DEPS" ]; then
    echo "❌ FAIL: Domain depends on higher layers:"
    echo "$DOMAIN_INFRA_DEPS"
    FAIL_COUNT=$((FAIL_COUNT + 1))
else
    echo "✅ PASS: Domain layer pure"
fi

# 6. Redis dependencies (max 2 crates)
echo ""
echo "💾 Checking Redis dependency scope..."
REDIS_CRATE_COUNT=$(find crates -name "Cargo.toml" -exec grep -l "redis" {} \; | wc -l)
if [ "$REDIS_CRATE_COUNT" -gt 2 ]; then
    echo "❌ FAIL: Redis in $REDIS_CRATE_COUNT crates (max: 2)"
    find crates -name "Cargo.toml" -exec grep -l "redis" {} \;
    FAIL_COUNT=$((FAIL_COUNT + 1))
else
    echo "✅ PASS: Redis scoped to $REDIS_CRATE_COUNT crates"
fi

# 7. Duplicate files (robots, memory managers)
echo ""
echo "🔍 Checking for duplications..."
ROBOTS_COUNT=$(find crates -name "robots.rs" | wc -l)
if [ "$ROBOTS_COUNT" -gt 1 ]; then
    echo "❌ FAIL: $ROBOTS_COUNT robots.rs files found (should be 1)"
    find crates -name "robots.rs"
    FAIL_COUNT=$((FAIL_COUNT + 1))
else
    echo "✅ PASS: Single robots.rs implementation"
fi

MEMORY_MGR_COUNT=$(find crates -name "memory_manager.rs" | wc -l)
if [ "$MEMORY_MGR_COUNT" -gt 1 ]; then
    echo "❌ FAIL: $MEMORY_MGR_COUNT memory_manager.rs files found (should be 1)"
    find crates -name "memory_manager.rs"
    FAIL_COUNT=$((FAIL_COUNT + 1))
else
    echo "✅ PASS: Single memory manager implementation"
fi

# 8. Clippy warnings (strict)
echo ""
echo "🔧 Running clippy (strict mode)..."
if ! cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tee /tmp/clippy.log; then
    echo "❌ FAIL: Clippy warnings found"
    FAIL_COUNT=$((FAIL_COUNT + 1))
else
    echo "✅ PASS: Clippy clean"
fi

# 9. Facade test coverage (≥90%)
echo ""
echo "📊 Checking facade test coverage (min: 90%)..."
if command -v cargo-llvm-cov >/dev/null 2>&1; then
    cargo llvm-cov --package riptide-facade --json --output-path /tmp/coverage.json >/dev/null 2>&1
    COVERAGE=$(jq '.data[0].totals.lines.percent' /tmp/coverage.json 2>/dev/null || echo "0")

    if (( $(echo "$COVERAGE < 90" | bc -l) )); then
        echo "❌ FAIL: Facade coverage ${COVERAGE}% (min: 90%)"
        FAIL_COUNT=$((FAIL_COUNT + 1))
    else
        echo "✅ PASS: Facade coverage ${COVERAGE}%"
    fi
else
    echo "⚠️  SKIP: cargo-llvm-cov not installed"
fi

# 10. Circular dependencies
echo ""
echo "🔄 Checking for circular dependencies..."
if cargo tree -p riptide-api 2>&1 | grep -q "cycle"; then
    echo "❌ FAIL: Circular dependency detected"
    FAIL_COUNT=$((FAIL_COUNT + 1))
else
    echo "✅ PASS: No circular dependencies"
fi

# Summary
echo ""
echo "===================================="
if [ $FAIL_COUNT -eq 0 ]; then
    echo "✅ ALL CHECKS PASSED!"
    exit 0
else
    echo "❌ $FAIL_COUNT CHECKS FAILED"
    exit 1
fi
```

**CI/CD Integration:**

**File:** `.github/workflows/architecture_validation.yml` (NEW)

```yaml
name: Architecture Validation

on:
  pull_request:
    branches: [main]
  push:
    branches: [main]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy

      - name: Install cargo-llvm-cov
        run: cargo install cargo-llvm-cov

      - name: Run Architecture Validation
        run: ./scripts/validate_architecture.sh

      - name: Upload Coverage
        if: always()
        uses: codecov/codecov-action@v3
        with:
          files: /tmp/coverage.json
```

---

### Sprint 5.2: cargo-deny Integration (Day 2)

**Goal:** Enforce layer boundaries at compile-time using cargo-deny

**Tasks:**

**Task 5.2.1: Configure cargo-deny Rules**

**File:** `deny.toml` (ALREADY CREATED - Reference existing configuration)

The deny.toml file has been enhanced with layer boundary enforcement rules:

```toml
# ===================================================================
# CLEAN ARCHITECTURE LAYER BOUNDARY ENFORCEMENT
# ===================================================================
# Enforces separation of concerns and dependency inversion principle
# Based on: API → Application → Domain ←→ Infrastructure pattern

[[bans.deny]]
# Domain layer (riptide-types) must remain pure
name = "riptide-api"
wrappers = ["riptide-types"]
reason = "Domain layer cannot depend on API layer (violates dependency inversion)"

[[bans.deny]]
name = "riptide-facade"
wrappers = ["riptide-types"]
reason = "Domain layer cannot depend on Application layer (violates dependency inversion)"

[[bans.deny]]
name = "riptide-reliability"
wrappers = ["riptide-types"]
reason = "Domain layer cannot depend on Infrastructure (use ports/traits instead)"

[[bans.deny]]
name = "redis"
wrappers = ["riptide-facade"]
reason = "Application layer cannot depend on Redis directly (use CacheStorage port)"

[[bans.deny]]
name = "axum"
wrappers = ["riptide-facade"]
reason = "Application layer cannot depend on HTTP frameworks (violates ports & adapters)"

[[bans.deny]]
name = "sqlx"
wrappers = ["riptide-facade"]
reason = "Application layer cannot depend on databases directly (use Repository port)"

[[bans.deny]]
name = "reqwest"
wrappers = ["riptide-facade"]
reason = "Application layer cannot depend on HTTP clients directly (use ports)"
```

**Task 5.2.2: Install cargo-deny**

```bash
# Install cargo-deny (one-time)
cargo install cargo-deny

# Verify installation
cargo deny --version
```

**Task 5.2.3: Add cargo-deny to CI Pipeline**

**File:** `.github/workflows/architecture_validation.yml` (APPEND)

```yaml
    - name: Install cargo-deny
      run: cargo install cargo-deny

    - name: Check layer boundaries
      run: |
        cargo deny check bans
        cargo deny check advisories
        cargo deny check licenses
```

**Task 5.2.4: Run cargo-deny Locally**

```bash
# Check all rules
cargo deny check

# Check only layer boundaries
cargo deny check bans

# Output detailed information
cargo deny check --show-stats
```

**Acceptance Criteria:**
- ✅ cargo-deny installed in CI/CD pipeline
- ✅ All layer boundary violations detected and prevented
- ✅ Zero warnings from `cargo deny check bans`
- ✅ Documentation updated with cargo-deny usage

**Success Metrics:**
- 🎯 Domain layer purity: 100% (zero forbidden dependencies)
- 🎯 Facade isolation: 100% (no HTTP/database types)
- 🎯 Redis scope: ≤2 crates
- 🎯 CI build time increase: <30 seconds

---

### Sprint 5.3: Pre-commit Hook Installation (Day 3)

**Goal:** Prevent architectural violations before commit

**Tasks:**

**Task 5.3.1: Create Pre-commit Hook Script**

**File:** `.git/hooks/pre-commit` (NEW - chmod +x required)

```bash
#!/bin/bash
# Architecture validation pre-commit hook
# Prevents commits that violate clean architecture rules

set -e

echo "🔍 Running architecture validation..."

# 1. Quick checks (fast fail)
echo "  → Checking for HTTP types in facades..."
if rg -n 'actix_web::|hyper::|axum::' crates/riptide-facade/src/ 2>/dev/null; then
    echo "❌ FAIL: HTTP types found in facade layer"
    echo "Fix: Remove HTTP framework dependencies from riptide-facade"
    exit 1
fi

echo "  → Checking for serde_json::Value in facades..."
if rg -n 'serde_json::Value' crates/riptide-facade/src/ 2>/dev/null; then
    echo "❌ FAIL: serde_json::Value found in facade (use typed DTOs)"
    echo "Fix: Replace with domain types from riptide-types"
    exit 1
fi

# 2. Run cargo-deny (if installed)
if command -v cargo-deny &> /dev/null; then
    echo "  → Checking layer boundaries with cargo-deny..."
    if ! cargo deny check bans --quiet 2>/dev/null; then
        echo "❌ FAIL: Layer boundary violations detected"
        echo "Fix: Run 'cargo deny check bans' for details"
        exit 1
    fi
else
    echo "  ⚠️  SKIP: cargo-deny not installed (run 'cargo install cargo-deny')"
fi

# 3. Run clippy (quick check)
echo "  → Running clippy..."
if ! cargo clippy --workspace --all-targets --quiet -- -D warnings 2>/dev/null; then
    echo "❌ FAIL: Clippy warnings detected"
    echo "Fix: Run 'cargo clippy --workspace --all-targets' for details"
    exit 1
fi

# 4. Check for common anti-patterns
echo "  → Checking for anti-patterns in handlers..."
HANDLER_LOOPS=$(find crates/riptide-api/src/handlers -name "*.rs" -exec grep -Hn '\bfor\b.*{' {} + | grep -v "//" | wc -l)
if [ "$HANDLER_LOOPS" -gt 0 ]; then
    echo "⚠️  WARNING: Found $HANDLER_LOOPS loop(s) in handlers (should be in facades)"
    echo "This is a warning - commit allowed but review recommended"
fi

echo "✅ All pre-commit checks passed!"
exit 0
```

**Task 5.3.2: Install Pre-commit Hook**

```bash
# Make script executable
chmod +x .git/hooks/pre-commit

# Verify it runs
.git/hooks/pre-commit
```

**Task 5.3.3: Document Bypass Procedure**

**File:** `docs/architecture/PRE_COMMIT_HOOK.md` (NEW)

```markdown
# Pre-commit Hook Documentation

## Purpose
Prevents architectural violations from being committed to the repository.

## Checks Performed
1. HTTP types in facade layer
2. serde_json::Value usage in facades
3. Layer boundary violations (via cargo-deny)
4. Clippy warnings
5. Business logic loops in handlers

## Bypass Procedure
**ONLY in emergencies (e.g., critical hotfix):**

```bash
# Option 1: Skip hooks for single commit
git commit --no-verify -m "hotfix: critical bug fix"

# Option 2: Temporarily disable hook
mv .git/hooks/pre-commit .git/hooks/pre-commit.disabled
git commit -m "your message"
mv .git/hooks/pre-commit.disabled .git/hooks/pre-commit
```

**NOTE:** All bypassed commits MUST be fixed in follow-up PR within 24 hours.

## Installation
```bash
chmod +x .git/hooks/pre-commit
```

## Troubleshooting
- **Hook too slow:** Comment out clippy check for faster commits
- **False positives:** Whitelist patterns in the hook script
- **Not running:** Check file is executable (`ls -la .git/hooks/pre-commit`)
```

**Task 5.3.4: Add Hook Installation to Setup Documentation**

**File:** `README.md` (UPDATE - add to Development Setup section)

```markdown
### Development Setup

1. Install Rust toolchain
2. Install dependencies:
   ```bash
   cargo install cargo-deny cargo-llvm-cov
   ```
3. Install pre-commit hook:
   ```bash
   chmod +x .git/hooks/pre-commit
   ```
4. Verify setup:
   ```bash
   ./scripts/validate_architecture.sh
   ```
```

**Acceptance Criteria:**
- ✅ Pre-commit hook installed and executable
- ✅ All checks run in <10 seconds
- ✅ Documentation explains bypass procedure
- ✅ Zero false positives on clean codebase

**Success Metrics:**
- 🎯 Hook run time: <10 seconds
- 🎯 False positive rate: 0%
- 🎯 Architectural violation prevention: 100%
- 🎯 Developer adoption rate: 100% (team consensus)

**Notes:**
- Hook is local only (not enforced in CI - that's what GitHub Actions is for)
- Optional but recommended for all developers
- Can be customized per developer workflow preferences

---

## Feature Flag Strategy (Incremental Rollout)

### Implementation

**File:** `crates/riptide-api/src/feature_flags.rs` (NEW)

```rust
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct FeatureFlags {
    flags: HashMap<String, bool>,
}

impl FeatureFlags {
    pub fn from_env() -> Self {
        let mut flags = HashMap::new();

        // Read from environment
        flags.insert("use_new_extraction_facade".to_string(),
                    std::env::var("FF_NEW_EXTRACTION").is_ok());
        flags.insert("use_new_browser_facade".to_string(),
                    std::env::var("FF_NEW_BROWSER").is_ok());
        flags.insert("use_ports_and_adapters".to_string(),
                    std::env::var("FF_PORTS_ADAPTERS").is_ok());

        Self { flags }
    }

    pub fn is_enabled(&self, flag: &str) -> bool {
        self.flags.get(flag).copied().unwrap_or(false)
    }
}
```

**Usage in Handlers:**

```rust
// crates/riptide-api/src/handlers/extract.rs
pub async fn extract_content(
    State(state): State<AppState>,
    Json(req): Json<ExtractRequest>,
) -> Result<Json<ExtractResponse>, ApiError> {
    if state.feature_flags.is_enabled("use_new_extraction_facade") {
        // NEW: Use facade with ports & adapters
        let result = state.extraction_facade
            .extract_content(&req.url, &authz_ctx)
            .await?;
        Ok(Json(result.into()))
    } else {
        // OLD: Keep legacy path until validated
        legacy_extract_content(state, req).await
    }
}
```

**Rollout Plan:**
```
Week 5: Enable FF_NEW_EXTRACTION in staging (10% traffic)
Week 6: Enable FF_NEW_EXTRACTION in production (50% traffic)
Week 7: Enable FF_NEW_EXTRACTION in production (100%)
Week 8: Remove legacy code, delete feature flag
```

---

## Rollback Triggers & Procedures

### Automatic Rollback Triggers

**CRITICAL - Halt Immediately If:**

1. **Test Failure Rate >5%**
   ```bash
   # In CI/CD
   PASS_RATE=$(cargo test --workspace --no-fail-fast 2>&1 | grep "test result:" | awk '{print $4}')
   if [ "$PASS_RATE" -lt 95 ]; then
       echo "ROLLBACK: Test pass rate ${PASS_RATE}% < 95%"
       exit 1
   fi
   ```

2. **Performance Regression >10%**
   ```bash
   # Benchmark comparison
   LATENCY_CHANGE=$(cargo bench --bench handler_latency | grep "change:" | awk '{print $2}')
   if (( $(echo "$LATENCY_CHANGE > 10" | bc -l) )); then
       echo "ROLLBACK: Latency increased ${LATENCY_CHANGE}%"
       exit 1
   fi
   ```

3. **Production Error Rate Spike**
   - Error rate >2% (from baseline <0.5%)
   - 5xx errors >100/min
   - Circuit breakers open >50% of endpoints

4. **Memory Leak Detected**
   ```bash
   # Memory growth >20% over 1 hour
   MEMORY_GROWTH=$(calculate_memory_growth)
   if [ "$MEMORY_GROWTH" -gt 20 ]; then
       echo "ROLLBACK: Memory leak detected"
       exit 1
   fi
   ```

### Manual Rollback Procedure

**Step 1: Disable Feature Flag**
```bash
# In production
kubectl set env deployment/riptide-api FF_NEW_EXTRACTION=false
kubectl rollout status deployment/riptide-api
```

**Step 2: Verify Rollback**
```bash
# Check error rate drops
curl https://api.riptide.io/metrics | jq '.error_rate'

# Check latency returns to baseline
curl https://api.riptide.io/metrics | jq '.p99_latency'
```

**Step 3: Investigate Root Cause**
```bash
# Export logs
kubectl logs -l app=riptide-api --since=1h > /tmp/incident.log

# Analyze failure patterns
rg "ERROR|PANIC" /tmp/incident.log | sort | uniq -c
```

**Step 4: Fix & Re-Deploy**
```bash
# Fix code
git checkout fix/rollback-issue

# Validate locally
cargo test --workspace
cargo bench --bench handler_latency

# Re-enable feature flag (gradual)
# 10% → 50% → 100% over 3 days
```

---

## Comprehensive KPIs (Stricter Targets)

### Quantitative Metrics

| Metric | Baseline | Target | Measurement |
|--------|----------|--------|-------------|
| **Handler LOC (avg)** | 145 | **<50** | `find handlers -name "*.rs" -exec wc -l {} \; \| awk '{sum+=$1; n++} END {print sum/n}'` |
| **Handler LOC (max)** | 945 | **<50** | `find handlers -name "*.rs" -exec wc -l {} \; \| sort -rn \| head -1` |
| **Handlers with loops** | 45 | **0** | `rg "for\|while\|loop" handlers/ \| wc -l` |
| **Facades >1000 LOC** | 1 | **0** | `find facades -name "*.rs" -exec wc -l {} \; \| awk '$1 > 1000'` |
| **HTTP types in facades** | 3 | **0** | `rg "actix_web::\|axum::" facades/ \| wc -l` |
| **JSON in facades** | 35 | **0** | `rg "serde_json::Value" facades/ \| wc -l` |
| **Facade test coverage** | 60% | **≥90%** | `cargo llvm-cov -p riptide-facade` |
| **Duplicate files** | 5 | **0** | `find crates -name "robots.rs" -o -name "memory_manager.rs" \| wc -l` |
| **Redis dependencies** | 6 | **≤2** | `find crates -name "Cargo.toml" -exec grep -l redis {} \; \| wc -l` |
| **Clippy warnings** | 12 | **0** | `cargo clippy --workspace -- -D warnings 2>&1 \| grep "warning:" \| wc -l` |
| **Circular dependencies** | 0 | **0** | `cargo tree \| grep -c "cycle"` |

### Qualitative Metrics

- ✅ **Maintainability:** New handler added in <5 minutes with <50 LOC
- ✅ **Testability:** Facades unit testable without HTTP mocking
- ✅ **Extensibility:** New feature added via port trait implementation
- ✅ **Type Safety:** 100% compile-time enforcement of layer boundaries
- ✅ **Operability:** Zero-downtime deployments with feature flags
- ✅ **Observability:** Business metrics (not just transport) in Prometheus

---

## Timeline Summary

### Phase 0: Pre-Refactoring Cleanup (Week 0)
- **Duration:** 3 days
- **LOC Impact:** -2,300 (duplications)
- **Deliverables:** robots.rs split (utils + reliability), single memory manager, scoped Redis

### Phase 1: Ports & Adapters (Weeks 1-2)
- **Duration:** 2 weeks
- **LOC Impact:** +1,800 (ports + adapters)
- **Deliverables:** All port traits, adapters, composition root

### Phase 2: Application Layer (Weeks 3-4)
- **Duration:** 2 weeks
- **LOC Impact:** +1,500 (authz, idempotency, events, transactions)
- **Deliverables:** Authorization policies, transactional workflows, outbox pattern

### Phase 3: Handler Refactoring (Weeks 5-6)
- **Duration:** 2 weeks
- **LOC Impact:** -5,157 (handlers), +3,000 (facades)
- **Deliverables:** All handlers <50 LOC, 5 new facades, domain types

### Phase 4: Infrastructure Consolidation (Week 7)
- **Duration:** 1 week
- **LOC Impact:** -800 (scattered infra)
- **Deliverables:** Unified HTTP client, Redis manager, circuit breakers

### Phase 5: Validation Automation (Week 8)
- **Duration:** 3 days
- **LOC Impact:** +300 (validation scripts)
- **Deliverables:** Enhanced validate_architecture.sh, CI/CD integration

**Total Duration:** 8 weeks (56 days)
**Net LOC Impact:** -2,657 LOC deleted, +6,600 LOC added (structured)
**Quality Impact:** 0 violations, ≥90% coverage, <50 LOC handlers

---

## Success Definition (Final Checklist)

### Phase 0 Complete When:
- [ ] robots.rs split completed (robots.rs in utils + robots_fetcher.rs in reliability)
- [ ] Single memory_manager.rs in riptide-pool
- [ ] Redis dependencies ≤2 crates
- [ ] CacheStorage trait defined
- [ ] All tests pass

### Phase 1 Complete When:
- [ ] All ports defined (10+ traits)
- [ ] All adapters implemented
- [ ] ApplicationContext wires dependencies
- [ ] Zero direct infra usage in facades
- [ ] Tests use in-memory adapters

### Phase 2 Complete When:
- [ ] Authorization policies enforced
- [ ] Idempotency at all entry points
- [ ] Transactional outbox working
- [ ] Domain events emitted
- [ ] Business metrics instrumented

### Phase 3 Complete When:
- [ ] All handlers <50 LOC
- [ ] Zero business logic in handlers
- [ ] 10 facades created/enhanced
- [ ] Zero serde_json::Value in facades
- [ ] ≥90% facade coverage

### Phase 4 Complete When:
- [ ] All HTTP via ReliableHttpClient
- [ ] Redis via single manager
- [ ] Circuit breakers configured
- [ ] Graceful degradation tested

### Phase 5 Complete When:
- [ ] validate_architecture.sh passes
- [ ] CI/CD integrated
- [ ] Feature flags working
- [ ] Rollback procedures documented
- [ ] Team trained

---

## Appendices

### A. File Structure (Post-Refactoring)

```
crates/
  riptide-types/
    src/
      ports/              # NEW
        mod.rs
        repository.rs
        events.rs
        idempotency.rs
        features.rs
        infrastructure.rs
        cache.rs
      pipeline/
        facade_types.rs   # EXISTING (12 types)

  riptide-facade/
    src/
      facades/
        extraction.rs     # ENHANCED (ports injected)
        browser.rs        # ENHANCED
        pipeline.rs       # ENHANCED (no JSON)
        spider.rs         # ENHANCED
        search.rs         # ENHANCED
        pdf.rs            # ENHANCED
        profile.rs        # ENHANCED
        table.rs          # ENHANCED
        trace.rs          # NEW
        llm.rs            # NEW
        profiling.rs      # NEW
        workers.rs        # NEW
        engine.rs         # NEW
      authorization/      # NEW
        mod.rs
        policies.rs
      workflows/          # NEW
        transactional.rs
        backpressure.rs
      metrics/            # NEW
        business.rs

  riptide-reliability/
    src/
      http_client.rs      # EXISTING (enhanced)
      circuit.rs          # EXISTING
      robots.rs           # NEW (consolidated)

  riptide-cache/
    src/
      adapters/           # NEW
        redis_cache.rs    # impl CacheStorage
        redis_idempotency.rs # impl IdempotencyStore
      redis_manager.rs    # NEW (single client)

  riptide-persistence/
    src/
      adapters/           # NEW
        postgres_repository.rs
        postgres_transaction.rs
        outbox_event_bus.rs
      migrations/         # SQL migrations
        001_create_event_outbox.sql

  riptide-api/
    src/
      handlers/
        *.rs              # ALL <50 LOC
      composition.rs      # NEW (DI root)
      feature_flags.rs    # NEW
      state.rs            # REFACTORED

scripts/
  validate_architecture.sh  # ENHANCED

.github/
  workflows/
    architecture_validation.yml  # NEW
```

### B. Migration Checklist (Per Facade)

For each facade migration, follow this checklist:

- [ ] 1. Define port traits in riptide-types/ports
- [ ] 2. Implement adapters in infrastructure crates
- [ ] 3. Inject dependencies via ApplicationContext
- [ ] 4. Add authorization policies
- [ ] 5. Wrap in transactional workflow
- [ ] 6. Add idempotency keys
- [ ] 7. Emit domain events
- [ ] 8. Add business metrics
- [ ] 9. Refactor handler to <50 LOC
- [ ] 10. Write unit tests (≥90% coverage)
- [ ] 11. Write integration tests
- [ ] 12. Add feature flag
- [ ] 13. Deploy to staging (10% traffic)
- [ ] 14. Monitor for 24 hours
- [ ] 15. Increase to 50% traffic
- [ ] 16. Monitor for 48 hours
- [ ] 17. Increase to 100% traffic
- [ ] 18. Remove legacy code after 1 week
- [ ] 19. Delete feature flag
- [ ] 20. Update documentation

---

## Document Status

**Version:** 2.0
**Status:** ✅ Ready for Implementation
**Author:** System Architecture Designer
**Date:** 2025-11-08
**Next Review:** After Phase 0 completion (Week 0 complete)

**Related Documents:**
- [PORTS_AND_ADAPTERS_STRATEGY.md](./PORTS_AND_ADAPTERS_STRATEGY.md)
- [DEDUPLICATION_PLAN.md](./DEDUPLICATION_PLAN.md)
- [REDIS_CONSOLIDATION_GUIDE.md](./REDIS_CONSOLIDATION_GUIDE.md)
- [VALIDATION_ENHANCEMENT_SPEC.md](./VALIDATION_ENHANCEMENT_SPEC.md)

---

**Next Steps:**
1. Review this roadmap with engineering team
2. Begin Phase 0 Sprint 0.1 (Deduplication)
3. Daily standups to track progress
4. Weekly reviews against KPIs
5. Adjust timeline based on actual velocity
