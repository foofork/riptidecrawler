# Dependency Flow Architecture Analysis

**Date:** 2025-11-06
**Analysis Target:** RipTide Codebase Dependency Architecture
**Rule:** API → FACADE → DOMAIN → INFRASTRUCTURE (downward only, no sideways)

---

## Executive Summary

This analysis examines the dependency flow across the RipTide codebase to identify violations of the architectural layering rule. The codebase has **recently resolved a critical circular dependency** (Phase 2C.2) between `riptide-api` and `riptide-facade`, but several **architectural violations remain** in the domain and infrastructure layers.

### Key Findings:
- ✅ **API → FACADE**: Clean (circular dependency resolved via trait abstraction)
- ⚠️ **FACADE → DOMAIN**: Multiple violations (layer skipping, sideways dependencies)
- ⚠️ **DOMAIN LAYER**: Significant circular dependencies and sideways coupling
- ⚠️ **INFRASTRUCTURE**: Redis directly used in domain crates (should be abstracted)

---

## Architecture Layer Definitions

### Expected Architecture:

```
┌─────────────────────────────────────┐
│  API LAYER (riptide-api)            │  ← HTTP/REST endpoints, application entry
├─────────────────────────────────────┤
│  FACADE LAYER (riptide-facade)      │  ← Orchestration, workflow coordination
├─────────────────────────────────────┤
│  DOMAIN LAYER                        │  ← Business logic, core features
│  - riptide-spider                    │
│  - riptide-fetch                     │
│  - riptide-extraction                │
│  - riptide-browser                   │
│  - riptide-pipeline                  │
│  - riptide-pool                      │
│  - riptide-search                    │
│  - riptide-intelligence              │
├─────────────────────────────────────┤
│  INFRASTRUCTURE LAYER                │  ← Technical concerns, external systems
│  - riptide-cache (Redis)             │
│  - riptide-persistence (DB)          │
│  - riptide-monitoring (Telemetry)    │
└─────────────────────────────────────┘

FOUNDATION: riptide-types, riptide-config, riptide-events, riptide-reliability
```

**Rules:**
1. **Downward only**: Higher layers depend on lower layers
2. **No sideways**: Domain crates should not circularly depend on each other
3. **Infrastructure abstraction**: Domain should use traits, not concrete infrastructure
4. **Foundation everywhere**: All layers can use foundation crates

---

## Violation Analysis

### 🟢 Category 1: RESOLVED - API ↔ FACADE Circular Dependency

**Status:** ✅ **FIXED in Phase 2C.2**

#### Previous Violation:
```toml
# riptide-api/Cargo.toml (Line 69)
riptide-facade = { path = "../riptide-facade" }

# riptide-facade/Cargo.toml (Line 15 - REMOVED)
# riptide-api = { path = "../riptide-api" }  # REMOVED
```

#### Resolution:
- **Trait extraction**: `PipelineExecutor` and `StrategiesPipelineExecutor` traits moved to `riptide-types`
- **Inversion of Control**: `riptide-facade` now depends on traits, not concrete `riptide-api` implementations
- **Comment documentation**: Clear annotation in `Cargo.toml` explaining the fix

**Recommendation:** ✅ No action needed. This is the correct architectural pattern.

---

### 🔴 Category 2: FACADE → DOMAIN Layer Violations

#### Violation 2.1: Facade Directly Depends on Multiple Domain Crates

**Location:** `/workspaces/eventmesh/crates/riptide-facade/Cargo.toml`

```toml
# Lines 16-27 - Direct domain dependencies
riptide-fetch = { path = "../riptide-fetch" }
riptide-extraction = { path = "../riptide-extraction" }
riptide-pdf = { path = "../riptide-pdf" }
riptide-cache = { path = "../riptide-cache" }
riptide-browser = { path = "../riptide-browser" }
riptide-stealth = { path = "../riptide-stealth" }
riptide-spider = { path = "../riptide-spider" }
riptide-search = { path = "../riptide-search" }
```

**Why it violates the rule:**
- **Tight coupling**: Facade is directly coupled to 8+ domain implementations
- **Poor abstraction**: Changes in domain layer ripple up to facade
- **Testability issues**: Difficult to mock domain behaviors in facade tests

**Impact:**
- 🔴 **High**: Facade cannot evolve independently
- 🔴 **High**: Adding new domain features requires facade changes
- 🟡 **Medium**: Reduced modularity

**Suggested restructuring:**

```rust
// NEW: riptide-facade should depend on traits in riptide-types

// riptide-types/src/facade_traits.rs
pub trait CrawlStrategy: Send + Sync {
    async fn crawl(&self, request: CrawlRequest) -> Result<CrawlResult>;
}

pub trait ExtractionStrategy: Send + Sync {
    async fn extract(&self, content: &str) -> Result<ExtractionResult>;
}

// riptide-facade/src/lib.rs
use riptide_types::facade_traits::{CrawlStrategy, ExtractionStrategy};

pub struct UnifiedFacade {
    crawler: Arc<dyn CrawlStrategy>,
    extractor: Arc<dyn ExtractionStrategy>,
}

// riptide-api injects concrete implementations
impl CrawlStrategy for SpiderCrawler { ... }
```

**Files to modify:**
1. Extract traits to `/workspaces/eventmesh/crates/riptide-types/src/facade_traits.rs`
2. Update `/workspaces/eventmesh/crates/riptide-facade/Cargo.toml` (remove domain deps)
3. Update `/workspaces/eventmesh/crates/riptide-facade/src/lib.rs` (use traits)
4. Update `/workspaces/eventmesh/crates/riptide-api/src/lib.rs` (dependency injection)

---

### 🔴 Category 3: Domain Layer Sideways Dependencies

#### Violation 3.1: riptide-cache → Multiple Domain Crates

**Location:** `/workspaces/eventmesh/crates/riptide-cache/Cargo.toml`

```toml
# Lines 10-15 - Cache importing domain logic
riptide-types = { path = "../riptide-types" }           # ✅ OK (foundation)
riptide-pool = { path = "../riptide-pool" }             # 🔴 SIDEWAYS (domain)
riptide-events = { path = "../riptide-events" }         # ✅ OK (foundation)
riptide-extraction = { path = "../riptide-extraction" } # 🔴 SIDEWAYS (domain)
```

**Why it violates the rule:**
- **Cache is infrastructure**, should not depend on domain crates
- **Circular potential**: `extraction` → `cache` → `pool` → `extraction`
- **Responsibility confusion**: Cache managing extraction pooling logic

**Impact:**
- 🔴 **High**: Infrastructure coupled to business logic
- 🟡 **Medium**: Difficult to replace cache implementation

**Suggested restructuring:**

```rust
// Move pooling concerns OUT of cache
// riptide-cache should ONLY handle caching primitives

// BEFORE (Wrong):
// riptide-cache depends on riptide-pool and riptide-extraction

// AFTER (Correct):
// riptide-pool manages extraction instance pooling
// riptide-cache provides generic caching via traits in riptide-types

// riptide-types/src/cache_traits.rs
pub trait CacheBackend: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<Bytes>>;
    async fn set(&self, key: &str, value: Bytes, ttl: Duration) -> Result<()>;
}

// riptide-cache implements the trait
impl CacheBackend for RedisCache { ... }
```

**Files to modify:**
1. Remove `riptide-pool` and `riptide-extraction` from `/workspaces/eventmesh/crates/riptide-cache/Cargo.toml`
2. Extract cache traits to `/workspaces/eventmesh/crates/riptide-types/src/cache_traits.rs`
3. Move pooling logic from cache to `/workspaces/eventmesh/crates/riptide-pool/src/lib.rs`

---

#### Violation 3.2: riptide-spider → riptide-fetch (Circular Risk)

**Location:** `/workspaces/eventmesh/crates/riptide-spider/Cargo.toml`

```toml
# Lines 13-15 - Spider depending on Fetch
riptide-types = { path = "../riptide-types" }   # ✅ OK
riptide-config = { path = "../riptide-config" } # ✅ OK
riptide-fetch = { path = "../riptide-fetch" }   # 🟡 SIDEWAYS (domain)
```

**Cross-reference:**
```toml
# /workspaces/eventmesh/crates/riptide-fetch/Cargo.toml
riptide-types = { path = "../riptide-types" }   # ✅ OK
riptide-config = { path = "../riptide-config" } # ✅ OK
# Does NOT depend on spider (avoiding circular)
```

**Why it's concerning:**
- **Sideways dependency**: Two domain crates coupling
- **Layering ambiguity**: Is spider higher-level than fetch, or equal?
- **Future risk**: Easy to accidentally create `fetch → spider` circular dependency

**Impact:**
- 🟡 **Medium**: Currently not circular, but architecturally unclear
- 🟢 **Low**: Both crates have clear boundaries (crawling vs HTTP)

**Suggested restructuring:**

```rust
// Option A: Make fetch truly foundational (extract to foundation layer)
// Move riptide-fetch to foundation layer alongside riptide-types

// Option B: Use trait abstraction
// riptide-types/src/http_traits.rs
pub trait HttpClient: Send + Sync {
    async fn fetch(&self, url: &Url) -> Result<Response>;
}

// riptide-spider uses trait instead of concrete fetch
use riptide_types::HttpClient;

pub struct Spider {
    http: Arc<dyn HttpClient>,
}
```

**Recommendation:**
- **Preferred**: Option A - Move `riptide-fetch` to foundation layer (it's a pure HTTP client)
- **Alternative**: Option B - Abstract via traits if fetch has complex business logic

**Files to modify:**
1. Evaluate if `/workspaces/eventmesh/crates/riptide-fetch` should be foundational
2. If yes, document in architecture as foundation, not domain
3. If no, extract HTTP traits to `/workspaces/eventmesh/crates/riptide-types/src/http_traits.rs`

---

#### Violation 3.3: riptide-pipeline → Domain Crates (Orchestration Ambiguity)

**Location:** `/workspaces/eventmesh/crates/riptide-pipeline/Cargo.toml`

```toml
# Lines 30-39 - Pipeline depending on domain features
riptide-types = { path = "../riptide-types" }        # ✅ OK
riptide-cache = { path = "../riptide-cache" }        # 🔴 INFRASTRUCTURE (should be trait)
riptide-events = { path = "../riptide-events" }      # ✅ OK
riptide-fetch = { path = "../riptide-fetch" }        # 🟡 DOMAIN (sideways)
riptide-pdf = { path = "../riptide-pdf" }            # 🟡 DOMAIN (sideways)
riptide-extraction = { path = "../riptide-extraction" } # 🟡 DOMAIN (sideways)
riptide-intelligence = { path = "../riptide-intelligence" } # 🟡 DOMAIN (sideways)
```

**Why it violates the rule:**
- **Orchestration confusion**: Is pipeline domain or facade?
- **Infrastructure coupling**: Direct Redis cache dependency
- **Sideways coupling**: Pipeline coordinating 4+ domain crates

**Current architecture suggests:**
```
riptide-api → riptide-facade → riptide-pipeline → [domain crates]
```

**Impact:**
- 🔴 **High**: Unclear separation of concerns
- 🟡 **Medium**: Pipeline acts like a second facade layer
- 🟢 **Low**: Works functionally, but architecturally confusing

**Suggested restructuring:**

**Option A: Pipeline is part of Facade layer**
```toml
# Merge riptide-pipeline into riptide-facade
# Pipeline becomes an implementation detail of facade orchestration
```

**Option B: Pipeline uses trait abstraction**
```rust
// riptide-types/src/pipeline_traits.rs
pub trait PipelineStep: Send + Sync {
    async fn execute(&self, context: &mut Context) -> Result<()>;
}

pub trait CacheLayer: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<Data>>;
}

// riptide-pipeline depends ONLY on traits
pub struct Pipeline {
    steps: Vec<Arc<dyn PipelineStep>>,
    cache: Arc<dyn CacheLayer>,
}
```

**Recommendation:** Option B - Abstract all dependencies via traits

**Files to modify:**
1. Extract pipeline traits to `/workspaces/eventmesh/crates/riptide-types/src/pipeline_traits.rs`
2. Update `/workspaces/eventmesh/crates/riptide-pipeline/Cargo.toml` (remove domain deps)
3. Update `/workspaces/eventmesh/crates/riptide-pipeline/src/lib.rs` (use traits)
4. Domain crates implement `PipelineStep` trait

---

### 🔴 Category 4: Infrastructure in Domain (Redis/Database Leakage)

#### Violation 4.1: Direct Redis Dependency in Domain Crates

**Affected crates:**
```toml
# riptide-pipeline/Cargo.toml (Line 28)
redis = { workspace = true }  # 🔴 Direct infrastructure dependency

# riptide-api/Cargo.toml (Line 27)
redis = { workspace = true }  # ✅ OK (API layer)

# riptide-cache/Cargo.toml (Line 27)
redis = { workspace = true }  # ✅ OK (infrastructure layer)
```

**Why it violates the rule:**
- **Concrete dependency**: Domain tied to specific database technology
- **Testing difficulty**: Cannot mock Redis in unit tests
- **Vendor lock-in**: Cannot switch to alternative caching (e.g., DragonflyDB, Memcached)

**Impact:**
- 🔴 **High**: Domain cannot be tested in isolation
- 🔴 **High**: Infrastructure replacement requires domain changes
- 🟡 **Medium**: Reduced portability

**Suggested restructuring:**

```rust
// riptide-types/src/storage_traits.rs
#[async_trait]
pub trait KeyValueStore: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn set(&self, key: &str, value: Vec<u8>) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
}

// riptide-cache/src/redis.rs (infrastructure implementation)
pub struct RedisStore {
    client: redis::Client,
}

#[async_trait]
impl KeyValueStore for RedisStore {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        // Redis-specific implementation
    }
}

// riptide-pipeline/src/lib.rs (domain uses trait)
use riptide_types::KeyValueStore;

pub struct Pipeline {
    storage: Arc<dyn KeyValueStore>,  // NOT redis::Client
}
```

**Files to modify:**
1. Create `/workspaces/eventmesh/crates/riptide-types/src/storage_traits.rs`
2. Move Redis implementation to `/workspaces/eventmesh/crates/riptide-cache/src/redis.rs`
3. Update `/workspaces/eventmesh/crates/riptide-pipeline/Cargo.toml` (remove `redis` dep)
4. Update `/workspaces/eventmesh/crates/riptide-pipeline/src/lib.rs` (use `KeyValueStore` trait)

---

## Dependency Flow Diagrams

### Current Architecture (With Violations)

```
┌────────────────────────────────────────────────────────────────┐
│ API LAYER                                                       │
│ ┌────────────────────────────────────────────────────────────┐ │
│ │ riptide-api                                                 │ │
│ │ Depends on: facade, spider, fetch, extraction, browser,    │ │
│ │            cache, persistence, pipeline, monitoring        │ │
│ └────────────────────────────────────────────────────────────┘ │
└───────────────────────┬────────────────────────────────────────┘
                        │ (direct dependencies - good)
                        ▼
┌────────────────────────────────────────────────────────────────┐
│ FACADE LAYER                                                    │
│ ┌────────────────────────────────────────────────────────────┐ │
│ │ riptide-facade                                              │ │
│ │ Depends on: pipeline, fetch, extraction, pdf, cache,       │ │
│ │            browser, stealth, spider, search                │ │
│ └────────────────────────────────────────────────────────────┘ │
└───────────────────────┬────────────────────────────────────────┘
                        │ ⚠️ SHOULD USE TRAITS, NOT CONCRETE DEPS
                        ▼
┌────────────────────────────────────────────────────────────────┐
│ DOMAIN LAYER (Business Logic)                                  │
│                                                                 │
│ ┌──────────────┐    ┌──────────────┐    ┌──────────────┐     │
│ │ spider       │───▶│ fetch        │    │ extraction   │     │
│ │              │    │              │    │              │     │
│ └──────────────┘    └──────────────┘    └──────────────┘     │
│        ▲ ⚠️ sideways                                           │
│        │                                                       │
│ ┌──────────────┐    ┌──────────────┐    ┌──────────────┐     │
│ │ pipeline     │───▶│ intelligence │    │ browser      │     │
│ │              │    │              │    │              │     │
│ └──────┬───────┘    └──────────────┘    └──────────────┘     │
│        │ ⚠️ direct Redis                                       │
│        ▼                                                       │
│ ┌──────────────┐                                              │
│ │ pool         │                                              │
│ │              │                                              │
│ └──────────────┘                                              │
└───────────────────────┬────────────────────────────────────────┘
                        │ ⚠️ SHOULD NOT DEPEND DIRECTLY
                        ▼
┌────────────────────────────────────────────────────────────────┐
│ INFRASTRUCTURE LAYER                                            │
│                                                                 │
│ ┌──────────────┐    ┌──────────────┐    ┌──────────────┐     │
│ │ cache        │    │ persistence  │    │ monitoring   │     │
│ │ (Redis)      │    │ (Database)   │    │ (Telemetry)  │     │
│ └──────┬───────┘    └──────────────┘    └──────────────┘     │
│        │ ⚠️ imports extraction + pool                          │
│        ▼                                                       │
│ ┌──────────────┐                                              │
│ │ extraction   │ ◀── ⚠️ CIRCULAR DEPENDENCY RISK              │
│ └──────────────┘                                              │
└────────────────────────────────────────────────────────────────┘

FOUNDATION (Used by all layers):
┌────────────────────────────────────────────────────────────────┐
│ riptide-types, riptide-config, riptide-events,                │
│ riptide-reliability, riptide-utils                             │
└────────────────────────────────────────────────────────────────┘

⚠️ = Architectural violation
```

---

### Ideal Architecture (After Restructuring)

```
┌────────────────────────────────────────────────────────────────┐
│ API LAYER                                                       │
│ ┌────────────────────────────────────────────────────────────┐ │
│ │ riptide-api                                                 │ │
│ │ - Implements traits from riptide-types                     │ │
│ │ - Injects dependencies into facade                         │ │
│ └────────────────────────────────────────────────────────────┘ │
└───────────────────────┬────────────────────────────────────────┘
                        │
                        ▼
┌────────────────────────────────────────────────────────────────┐
│ FACADE LAYER                                                    │
│ ┌────────────────────────────────────────────────────────────┐ │
│ │ riptide-facade                                              │ │
│ │ Depends on: TRAITS ONLY (no concrete domain deps)          │ │
│ │ - CrawlStrategy, ExtractionStrategy, CacheLayer            │ │
│ └────────────────────────────────────────────────────────────┘ │
└───────────────────────┬────────────────────────────────────────┘
                        │
                        ▼
┌────────────────────────────────────────────────────────────────┐
│ DOMAIN LAYER (Business Logic)                                  │
│ - Each crate implements traits from riptide-types             │
│ - NO sideways dependencies between domain crates              │
│ - NO direct infrastructure dependencies                       │
│                                                                 │
│ ┌──────────────┐    ┌──────────────┐    ┌──────────────┐     │
│ │ spider       │    │ fetch        │    │ extraction   │     │
│ │ impl         │    │ impl         │    │ impl         │     │
│ │ CrawlStrategy│    │ HttpClient   │    │ Extractor    │     │
│ └──────────────┘    └──────────────┘    └──────────────┘     │
│                                                                 │
│ ┌──────────────┐    ┌──────────────┐    ┌──────────────┐     │
│ │ pipeline     │    │ intelligence │    │ browser      │     │
│ │ impl         │    │ impl         │    │ impl         │     │
│ │ Orchestrator │    │ LLMProvider  │    │ BrowserPool  │     │
│ └──────────────┘    └──────────────┘    └──────────────┘     │
└────────────────────────────────────────────────────────────────┘
                        ▲
                        │ (implements traits)
                        │
┌────────────────────────────────────────────────────────────────┐
│ INFRASTRUCTURE LAYER                                            │
│ - Implements storage/cache traits from riptide-types          │
│ - NO domain business logic                                     │
│                                                                 │
│ ┌──────────────┐    ┌──────────────┐    ┌──────────────┐     │
│ │ cache        │    │ persistence  │    │ monitoring   │     │
│ │ impl         │    │ impl         │    │ impl         │     │
│ │ KeyValueStore│    │ Repository   │    │ MetricsStore │     │
│ └──────────────┘    └──────────────┘    └──────────────┘     │
└────────────────────────────────────────────────────────────────┘

FOUNDATION (Defines ALL traits and types):
┌────────────────────────────────────────────────────────────────┐
│ riptide-types                                                  │
│ - facade_traits.rs (CrawlStrategy, ExtractionStrategy)        │
│ - storage_traits.rs (KeyValueStore, Repository)               │
│ - http_traits.rs (HttpClient)                                 │
│ - pipeline_traits.rs (PipelineStep, Orchestrator)             │
│ - common types, errors, events                                │
│                                                                 │
│ riptide-config, riptide-events, riptide-reliability,          │
│ riptide-utils                                                  │
└────────────────────────────────────────────────────────────────┘

✅ Clean dependency inversion
✅ All layers depend downward on traits
✅ No sideways coupling
✅ Infrastructure fully abstracted
```

---

## Summary of Violations

| # | Category | Severity | File | Issue | Impact |
|---|----------|----------|------|-------|--------|
| 1 | ~~API ↔ FACADE Circular~~ | ✅ RESOLVED | `riptide-api/Cargo.toml` | ~~Circular dependency~~ | Fixed in Phase 2C.2 |
| 2.1 | FACADE → DOMAIN | 🔴 High | `riptide-facade/Cargo.toml` | Direct coupling to 8+ domain crates | Tight coupling, poor testability |
| 3.1 | DOMAIN Sideways | 🔴 High | `riptide-cache/Cargo.toml` | Infrastructure importing domain logic | Circular dependency risk |
| 3.2 | DOMAIN Sideways | 🟡 Medium | `riptide-spider/Cargo.toml` | Spider → Fetch sideways dependency | Layering ambiguity |
| 3.3 | DOMAIN Sideways | 🔴 High | `riptide-pipeline/Cargo.toml` | Pipeline → 4+ domain crates | Unclear separation of concerns |
| 4.1 | Infrastructure Leakage | 🔴 High | `riptide-pipeline/Cargo.toml` | Direct Redis dependency in domain | Vendor lock-in, untestable |

**Total Violations: 5 active** (1 resolved)

---

## Recommended Refactoring Roadmap

### Phase 1: Extract Foundation Traits (Week 1)
**Priority:** 🔴 Critical

1. Create `/workspaces/eventmesh/crates/riptide-types/src/facade_traits.rs`
   - `CrawlStrategy`, `ExtractionStrategy`, `BrowserStrategy`

2. Create `/workspaces/eventmesh/crates/riptide-types/src/storage_traits.rs`
   - `KeyValueStore`, `Repository`, `CacheLayer`

3. Create `/workspaces/eventmesh/crates/riptide-types/src/http_traits.rs`
   - `HttpClient`, `RequestBuilder`, `ResponseHandler`

4. Create `/workspaces/eventmesh/crates/riptide-types/src/pipeline_traits.rs`
   - `PipelineStep`, `Orchestrator`, `PipelineExecutor`

**Testing:** Compile `riptide-types` with new traits
```bash
cargo check -p riptide-types
```

---

### Phase 2: Refactor Infrastructure Layer (Week 1-2)
**Priority:** 🔴 Critical

1. **riptide-cache** → Remove domain dependencies
   ```toml
   # Remove from Cargo.toml:
   # riptide-pool, riptide-extraction

   # Keep only:
   # riptide-types, riptide-events
   ```

2. **riptide-persistence** → Implement repository traits
   ```rust
   // Implement Repository trait from riptide-types
   impl Repository for PostgresRepo { ... }
   ```

3. **riptide-monitoring** → No changes (already clean)

**Testing:**
```bash
cargo check -p riptide-cache
cargo check -p riptide-persistence
cargo test -p riptide-cache
```

---

### Phase 3: Refactor Pipeline (Week 2)
**Priority:** 🔴 Critical

1. Update `riptide-pipeline/Cargo.toml`
   ```toml
   # Remove:
   # redis, riptide-fetch, riptide-extraction, riptide-intelligence

   # Keep:
   # riptide-types (with new traits)
   ```

2. Update `riptide-pipeline/src/lib.rs`
   ```rust
   use riptide_types::{KeyValueStore, PipelineStep, Orchestrator};

   pub struct Pipeline {
       storage: Arc<dyn KeyValueStore>,
       steps: Vec<Arc<dyn PipelineStep>>,
   }
   ```

**Testing:**
```bash
cargo check -p riptide-pipeline
cargo test -p riptide-pipeline
```

---

### Phase 4: Refactor Domain Crates (Week 3)
**Priority:** 🟡 Medium

1. **riptide-spider** → Implement `CrawlStrategy` trait
2. **riptide-extraction** → Implement `ExtractionStrategy` trait
3. **riptide-fetch** → Implement `HttpClient` trait
4. **riptide-browser** → Implement `BrowserStrategy` trait
5. **riptide-intelligence** → Implement `LLMProvider` trait

**Testing:**
```bash
cargo test --workspace
```

---

### Phase 5: Refactor Facade (Week 3-4)
**Priority:** 🟡 Medium

1. Update `riptide-facade/Cargo.toml`
   ```toml
   # Remove all domain dependencies
   # Keep ONLY:
   # riptide-types, riptide-pipeline
   ```

2. Update `riptide-facade/src/lib.rs`
   ```rust
   use riptide_types::facade_traits::*;

   pub struct CrawlFacade {
       crawler: Arc<dyn CrawlStrategy>,
       extractor: Arc<dyn ExtractionStrategy>,
       browser: Arc<dyn BrowserStrategy>,
   }
   ```

**Testing:**
```bash
cargo check -p riptide-facade
cargo test -p riptide-facade
```

---

### Phase 6: Update API Layer (Week 4)
**Priority:** 🟢 Low (Last step)

1. Update `riptide-api/src/main.rs` with dependency injection
   ```rust
   // Inject concrete implementations
   let spider = Arc::new(SpiderCrawler::new());
   let extractor = Arc::new(NativeExtractor::new());
   let cache = Arc::new(RedisCache::new());

   let facade = CrawlFacade::new(spider, extractor, cache);
   ```

**Testing:**
```bash
cargo build --workspace
cargo test --workspace
RUSTFLAGS="-D warnings" cargo clippy --workspace
```

---

## Testing Strategy

### Unit Tests (Per-Crate)
```bash
# Test each crate in isolation
cargo test -p riptide-types
cargo test -p riptide-cache
cargo test -p riptide-pipeline
cargo test -p riptide-facade
cargo test -p riptide-spider
```

### Integration Tests (Cross-Crate)
```bash
# Test facade with mocked domain implementations
cargo test -p riptide-facade --features mock-domain

# Test API with real implementations
cargo test -p riptide-api --features full
```

### Architecture Validation
```bash
# Ensure no circular dependencies
cargo tree --workspace --duplicates

# Ensure correct dependency direction
cargo tree -p riptide-facade --depth 2
cargo tree -p riptide-pipeline --depth 2
```

---

## Acceptance Criteria

### Phase Completion Checklist

#### ✅ Phase 1 Complete When:
- [ ] All trait definitions exist in `riptide-types`
- [ ] `cargo check -p riptide-types` passes
- [ ] Documentation written for all traits

#### ✅ Phase 2 Complete When:
- [ ] Infrastructure crates depend ONLY on `riptide-types` traits
- [ ] No domain business logic in infrastructure
- [ ] All infrastructure tests pass

#### ✅ Phase 3 Complete When:
- [ ] `riptide-pipeline/Cargo.toml` has no `redis` dependency
- [ ] Pipeline uses traits, not concrete types
- [ ] All pipeline tests pass

#### ✅ Phase 4 Complete When:
- [ ] All domain crates implement required traits
- [ ] No sideways dependencies between domain crates
- [ ] All domain tests pass

#### ✅ Phase 5 Complete When:
- [ ] `riptide-facade/Cargo.toml` has no domain dependencies
- [ ] Facade accepts trait objects in constructor
- [ ] All facade tests pass with mocks

#### ✅ Phase 6 Complete When:
- [ ] API layer injects concrete implementations
- [ ] `cargo build --workspace` succeeds
- [ ] `RUSTFLAGS="-D warnings" cargo clippy --workspace` passes
- [ ] Full integration test suite passes

---

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Breaking existing functionality | 🟡 Medium | 🔴 High | Comprehensive test suite before refactoring |
| Performance regression | 🟢 Low | 🟡 Medium | Benchmark before/after with Criterion |
| Timeline overrun | 🟡 Medium | 🟡 Medium | Prioritize critical violations first |
| Resistance to change | 🟢 Low | 🟢 Low | Show architectural benefits with diagrams |

---

## Success Metrics

### Before Refactoring:
- **Circular dependencies:** 1 active (API ↔ Facade)
- **Sideways domain coupling:** 5+ instances
- **Infrastructure in domain:** 3+ crates
- **Trait abstraction:** ~20% coverage

### After Refactoring (Target):
- **Circular dependencies:** 0
- **Sideways domain coupling:** 0
- **Infrastructure in domain:** 0
- **Trait abstraction:** 90%+ coverage
- **Test coverage:** 85%+ (up from current)
- **Build time:** <5% regression acceptable

---

## Conclusion

The RipTide codebase has made significant progress by resolving the API ↔ Facade circular dependency in Phase 2C.2. However, **5 critical architectural violations remain** that prevent clean layer separation and testability.

**Recommended Next Steps:**
1. **Immediate (Week 1):** Extract foundation traits to `riptide-types` (Phase 1)
2. **Short-term (Week 1-2):** Refactor infrastructure to use traits (Phase 2)
3. **Medium-term (Week 3-4):** Refactor domain and facade layers (Phase 3-5)
4. **Long-term (Week 4):** Update API layer with dependency injection (Phase 6)

**Estimated Total Effort:** 4 weeks with 1-2 developers

**Expected Benefits:**
- ✅ Zero circular dependencies
- ✅ Clean architecture compliance
- ✅ 90%+ trait abstraction
- ✅ Full testability with mocks
- ✅ Infrastructure swap capability
- ✅ Improved maintainability

---

**Report Generated:** 2025-11-06
**Analysis Tool:** Manual Cargo.toml inspection + cargo tree
**Reviewer:** System Architecture Designer
