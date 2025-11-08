# Phase 0: Pre-Refactoring Cleanup Roadmap
**Version:** 2.0 (Enhanced)
**Date:** 2025-11-08
**Duration:** 5 days (updated from 3 days)
**Priority:** CRITICAL - Removes duplication before refactoring

---

## Overview

Phase 0 focuses on eliminating code duplication and consolidating infrastructure dependencies before beginning the main refactoring effort. This phase is CRITICAL because it establishes a clean foundation and prevents wasted effort refactoring duplicate code.

### Objectives

1. **Eliminate Code Duplication**: Remove duplicate robots.txt and memory manager implementations
2. **Consolidate Pipeline Files**: Merge 4 pipeline variants into coherent structure
3. **Scope Redis Dependencies**: Reduce from 6 crates to maximum 2
4. **Remove Tech Debt**: Delete obsolete admin files
5. **Define Foundation Ports**: Create CacheStorage trait for dependency inversion

### Success Criteria

- ✅ Only 1 robots.rs implementation (in riptide-utils) + 1 fetcher (in riptide-reliability)
- ✅ Only 1 memory_manager.rs (in riptide-pool)
- ✅ Redis dependencies ≤2 crates
- ✅ Pipeline files consolidated from 4 to 2 (or less)
- ✅ CacheStorage trait defined
- ✅ admin_old.rs deleted
- ✅ All tests pass after migration

---

## Sprint 0.1: Deduplication & Consolidation (3 days)

### Task 0.1.1: Split Robots.txt Implementation (1.5 days)

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

---

### Task 0.1.2: Consolidate Memory Managers (1 day)

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

---

### Task 0.1.3: Audit & Scope Redis Dependencies (0.5 days)

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

---

## Sprint 0.2: Pipeline Consolidation (NEW - 2 days)

**Priority:** CRITICAL
**Source:** API_CRATE_COVERAGE_ANALYSIS.md - Gap #4

### Problem

**4 pipeline files with overlapping logic (2,720 LOC total):**
- `pipeline.rs` (1,124 LOC) - Original implementation
- `pipeline_enhanced.rs` (583 LOC) - Enhanced variant
- `pipeline_dual.rs` (429 LOC) - Dual-mode variant
- `strategies_pipeline.rs` (584 LOC) - Strategy-based variant

**Current Roadmap:** Only mentions pipeline.rs ("wrap, don't rebuild")
**Gap:** Other 3 files (1,596 LOC) not addressed - likely 40-60% duplicate code

### Tasks

**Task 0.2.1: Analyze Pipeline Variants (0.5 days)**

```bash
# Compare implementations
diff -u pipeline.rs pipeline_enhanced.rs > /tmp/pipeline_diff_enhanced.txt
diff -u pipeline.rs pipeline_dual.rs > /tmp/pipeline_diff_dual.txt
diff -u pipeline.rs strategies_pipeline.rs > /tmp/pipeline_diff_strategies.txt

# Identify common patterns
rg "pub fn|pub async fn" pipeline*.rs | sort | uniq -c
```

**Expected Findings:**
- 40-60% code overlap (common pipeline stages)
- Different error handling strategies
- Variant-specific optimizations
- Possible feature flag candidates

**Task 0.2.2: Extract Common Logic (1 day)**

```rust
// crates/riptide-facade/src/facades/pipeline_common.rs (NEW)
pub struct PipelineCore {
    // Common pipeline logic extracted
}

impl PipelineCore {
    pub async fn execute_stage(&self, stage: PipelineStage) -> Result<StageOutput> {
        // Common stage execution
    }

    pub async fn handle_errors(&self, error: PipelineError) -> Result<Recovery> {
        // Common error handling
    }
}

// crates/riptide-api/src/pipeline.rs (REFACTORED - keep main implementation)
use riptide_facade::facades::pipeline_common::PipelineCore;

pub struct Pipeline {
    core: PipelineCore,  // Use common logic
    // Variant-specific fields
}
```

**Task 0.2.3: Consolidate or Delete Variants (0.5 days)**

**Decision Matrix:**
| File | Keep/Delete | Reason |
|------|-------------|--------|
| pipeline.rs | ✅ KEEP | Main implementation (wrap in facade later) |
| pipeline_enhanced.rs | ⚠️ EVALUATE | If 80%+ overlap → delete, else refactor |
| pipeline_dual.rs | ⚠️ EVALUATE | If feature-flag candidate → keep, else delete |
| strategies_pipeline.rs | ❌ DELETE | Move strategy pattern to facade |

**Files Modified:**
```
CREATE:  crates/riptide-facade/src/facades/pipeline_common.rs (~400 LOC)
UPDATE:  crates/riptide-api/src/pipeline.rs (use PipelineCore)
EVALUATE: crates/riptide-api/src/pipeline_enhanced.rs (consolidate or delete)
EVALUATE: crates/riptide-api/src/pipeline_dual.rs (consolidate or delete)
DELETE:  crates/riptide-api/src/strategies_pipeline.rs
UPDATE:  callers to use appropriate variant
```

**Validation:**
```bash
# Ensure no duplicate logic
cargo clippy --workspace -- -W clippy::cognitive_complexity

# Tests pass
cargo test -p riptide-api -- pipeline

# No broken references
rg "strategies_pipeline" crates/ && echo "FAIL: References remain" || echo "PASS"
```

**Expected LOC Reduction:** 600-900 LOC deleted (25-35% reduction)

---

## Sprint 0.4: Quick Wins Deduplication (NEW - 9 days) ⭐

**Priority:** CRITICAL - HIGHEST IMPACT
**Source:** WORKSPACE_CRATE_ANALYSIS.md - Quick Wins (Week 1)

### Overview

**Verified code duplication discovered across workspace** during comprehensive crate analysis. These Quick Wins can save **2,690 LOC in 9 days** (conservative verified counts).

### Task 0.4.1: Delete Duplicate Robots.txt ⭐ (2 days)

**Problem:** **481 LOC duplicated** (IDENTICAL files confirmed via diff)

**Files:**
- `riptide-spider/src/robots.rs` (481 LOC) - **DUPLICATE**
- `riptide-fetch/src/robots.rs` (481 LOC) - **CANONICAL**

**Solution:** Delete from spider, use fetch version

```bash
# Day 1: Analysis and migration prep
cd crates/riptide-spider
rg "use.*robots" src/  # Find all references
rg "RobotsPolicy|RobotsChecker" src/  # Find usages

# Day 2: Delete and migrate
rm src/robots.rs
# Update imports in spider files to use riptide_fetch::robots
sed -i 's/crate::robots/riptide_fetch::robots/g' src/**/*.rs

# Update Cargo.toml
# Add riptide-fetch dependency to riptide-spider

# Test
cargo test -p riptide-spider
cargo test -p riptide-fetch
```

**Files Modified:**
```
DELETE:  crates/riptide-spider/src/robots.rs (16,150 LOC)
UPDATE:  crates/riptide-spider/Cargo.toml (add riptide-fetch dep)
UPDATE:  crates/riptide-spider/src/lib.rs (use riptide_fetch::robots)
UPDATE:  crates/riptide-spider/src/core.rs (update imports)
```

**LOC Saved:** **-481**

---

### Task 0.4.2: Consolidate Circuit Breakers (3 days)

**Problem:** **4 separate implementations** (1,294 LOC to remove)

**Found In:**
- `riptide-utils/src/circuit_breaker.rs` (343 LOC) - DELETE
- `riptide-types/src/reliability/circuit.rs` (372 LOC) - **DELETE (domain violation!)**
- `riptide-intelligence/src/circuit_breaker.rs` (579 LOC) - DELETE
- `riptide-reliability/src/circuit_breaker.rs` (423 LOC) - **CANONICAL (KEEP)**

**Solution:** Keep `riptide-reliability::circuit`, delete others

```bash
# Day 1: Verify behavior equivalence
diff crates/riptide-utils/src/circuit_breaker.rs \
     crates/riptide-reliability/src/circuit.rs

diff crates/riptide-intelligence/src/circuit_breaker.rs \
     crates/riptide-reliability/src/circuit.rs

# Check for subtle differences in thresholds, timeouts

# Day 2-3: Migrate and delete
# Remove from utils
rm crates/riptide-utils/src/circuit_breaker.rs
# Update utils/lib.rs

# Remove from intelligence
rm crates/riptide-intelligence/src/circuit_breaker.rs
# Update intelligence to use riptide_reliability::circuit

# Remove from search
rm crates/riptide-search/src/circuit_breaker.rs
# Update search to use riptide_reliability::circuit
```

**Files Modified:**
```
DELETE:  crates/riptide-utils/src/circuit_breaker.rs (343 LOC)
DELETE:  crates/riptide-types/src/reliability/circuit.rs (372 LOC) ⚠️ CRITICAL
DELETE:  crates/riptide-intelligence/src/circuit_breaker.rs (579 LOC)
UPDATE:  crates/riptide-utils/Cargo.toml (add riptide-reliability dep)
UPDATE:  crates/riptide-intelligence/Cargo.toml (add riptide-reliability dep)
UPDATE:  crates/riptide-search/Cargo.toml (add riptide-reliability dep)
UPDATE:  All files using CircuitBreaker → use riptide_reliability::circuit
```

**LOC Saved:** **-1,294**

---

### Task 0.4.3: Consolidate Redis Clients (2 days)

**Problem:** **2 separate wrappers** (533 LOC wasted)

**Found In:**
- `riptide-utils/src/redis.rs` (152 LOC) - DELETE
- `riptide-cache/src/redis.rs` (381 LOC) - DELETE
- `riptide-persistence/src/cache.rs` - **CANONICAL (uses Redis internally)**

**Solution:** Keep `riptide-persistence::redis`, delete others

```bash
# Day 1: Migrate cache to use persistence
cd crates/riptide-cache
rm src/redis.rs
# Update to use riptide_persistence::redis
# Add riptide-persistence dependency

# Migrate utils
cd crates/riptide-utils
rm src/redis.rs
# Update callers to use riptide_persistence::redis

# Day 2: Test and verify
cargo test -p riptide-cache
cargo test -p riptide-persistence
cargo test -p riptide-utils
```

**Files Modified:**
```
DELETE:  crates/riptide-utils/src/redis.rs (152 LOC)
DELETE:  crates/riptide-cache/src/redis.rs (381 LOC)
UPDATE:  crates/riptide-cache/Cargo.toml (add riptide-persistence dep)
UPDATE:  crates/riptide-utils/Cargo.toml (add riptide-persistence dep)
UPDATE:  All files using Redis → use riptide_persistence::redis
```

**LOC Saved:** **-533**

---

### Task 0.4.4: Consolidate Rate Limiters (2 days)

**Problem:** **Multiple rate limiter implementations** (883 LOC to consolidate)

**Found In:**
- `riptide-utils/src/rate_limit.rs` (204 LOC) - DELETE
- `riptide-stealth/src/rate_limiter.rs` (501 LOC) - EVALUATE (might have anti-detection features)
- `riptide-api/src/middleware/rate_limit.rs` (178 LOC) - DELETE
- `riptide-security/src/lib.rs` - **CANONICAL** (rate limiting integrated)

**Solution:** Consolidate to `riptide-security`, evaluate stealth-specific features

```bash
# Day 1: Migrate all to security
rm crates/riptide-utils/src/rate_limit.rs
rm crates/riptide-stealth/src/rate_limiter.rs
rm crates/riptide-api/src/middleware/rate_limiter.rs

# Day 2: Update imports and test
# Update all files to use riptide_security::rate_limiter
cargo test --workspace
```

**Files Modified:**
```
DELETE:  crates/riptide-utils/src/rate_limit.rs (204 LOC)
EVALUATE:  crates/riptide-stealth/src/rate_limiter.rs (501 LOC) - may have unique features
DELETE:  crates/riptide-api/src/middleware/rate_limit.rs (178 LOC)
UPDATE:  All files using RateLimiter → use riptide_security::rate_limiter
```

**LOC Saved:** **-382** (conservative, excluding stealth until reviewed)

---

### Sprint 0.4 Summary

**Total Duration:** 9 days
**Total LOC Saved:** **-2,690** (conservative, verified counts)
**Breakdown:**
- Robots.txt: -481 LOC (identical files)
- Circuit breakers: -1,294 LOC (includes domain violation fix)
- Redis clients: -533 LOC
- Rate limiters: -382 LOC (excluding stealth pending review)

**Risk Level:** LOW to MEDIUM
**Effort:** LOW (delete + update imports, but verify behavior equivalence)

**Validation:**
```bash
# Verify no duplicates remain
rg "struct.*CircuitBreaker" crates/ | wc -l  # Expected: 1
rg "struct.*RateLimiter" crates/ | wc -l    # Expected: 1
find crates -name "robots*.rs" | wc -l       # Expected: 1
rg "RedisClient|RedisPool" crates/ | wc -l   # Expected: 1 location

# All tests pass
cargo test --workspace

# Build succeeds
cargo build --workspace
```

---

## Sprint 0.5: Small Crate Consolidation (NEW - 1 day)

**Priority:** MEDIUM (Crate Simplification)
**Source:** WORKSPACE_CRATE_ANALYSIS.md §5 - Minor Violation #10

### Problem

**3 small crates with overhead:**
- `riptide-schemas` (5 files, 1,265 LOC) - Event/data schemas
- `riptide-config` (7 files, 2,883 LOC) - Configuration management
- `riptide-test-utils` (4 files, 557 LOC) - Underutilized test helpers

**Issue:** Overhead of separate crates for <3k LOC each, dependency complexity

### Tasks

**Task 0.5.1: Runtime Schema Store → riptide-persistence (0.3 days)** ⚠️ SURGICAL

**Problem:** Current `riptide-schemas` contains compile-time structs that will calcify the API surface. Schemas must be **runtime JSON data**, not compiled types.

**Solution: Option B (Runtime Location)**

```bash
# 1. Create runtime schema store module in persistence layer
mkdir -p crates/riptide-persistence/src/schema_store

# 2. Define SchemaStore interface (minimal stub for now)
cat > crates/riptide-persistence/src/schema_store/mod.rs <<'EOF'
//! Runtime schema storage and validation
//! Schemas are JSON data, not compiled Rust structs

use serde_json::Value;
use std::collections::HashMap;
use std::sync::RwLock;

/// Runtime schema store interface
pub trait SchemaStore: Send + Sync {
    fn put(&self, schema_uri: &str, schema: Value) -> Result<(), SchemaError>;
    fn get(&self, schema_uri: &str) -> Result<Option<Value>, SchemaError>;
    fn list(&self) -> Result<Vec<String>, SchemaError>;
    fn validate(&self, schema_uri: &str, data: &Value) -> Result<bool, SchemaError>;
}

/// In-memory schema store (stub implementation)
pub struct InMemorySchemaStore {
    schemas: RwLock<HashMap<String, Value>>,
}

impl InMemorySchemaStore {
    pub fn new() -> Self {
        Self { schemas: RwLock::new(HashMap::new()) }
    }
}

impl SchemaStore for InMemorySchemaStore {
    fn put(&self, schema_uri: &str, schema: Value) -> Result<(), SchemaError> {
        self.schemas.write().unwrap().insert(schema_uri.to_string(), schema);
        Ok(())
    }

    fn get(&self, schema_uri: &str) -> Result<Option<Value>, SchemaError> {
        Ok(self.schemas.read().unwrap().get(schema_uri).cloned())
    }

    fn list(&self) -> Result<Vec<String>, SchemaError> {
        Ok(self.schemas.read().unwrap().keys().cloned().collect())
    }

    fn validate(&self, _schema_uri: &str, _data: &Value) -> Result<bool, SchemaError> {
        // TODO: Phase 2 - add JSON Schema validation
        Ok(true)
    }
}

#[derive(Debug)]
pub enum SchemaError {
    NotFound,
    ValidationFailed(String),
    StorageError(String),
}
EOF

# 3. Delete compile-time schema structs from riptide-schemas
find crates/riptide-schemas/src -name "*.rs" -type f -delete

# 4. Keep only JSON examples/tests (if they exist)
mkdir -p crates/riptide-schemas/examples
# Move any .json files to examples/
find crates/riptide-schemas -name "*.json" -exec mv {} crates/riptide-schemas/examples/ \;

# 5. Update persistence mod.rs to expose schema_store
echo "pub mod schema_store;" >> crates/riptide-persistence/src/lib.rs

# 6. Remove riptide-schemas from workspace (keep directory for JSON examples)
# Update Cargo.toml to exclude it from workspace members
```

**Files Created:**
```
CREATE: crates/riptide-persistence/src/schema_store/mod.rs (~80 LOC)
CREATE: crates/riptide-schemas/examples/ (JSON test data only)
UPDATE: crates/riptide-persistence/src/lib.rs (expose schema_store)
DELETE: All .rs files in crates/riptide-schemas/src/
UPDATE: Cargo.toml (remove riptide-schemas from workspace members)
```

**Key Architectural Decisions:**
- ✅ Schemas are **runtime JSON data**, not compile-time Rust structs
- ✅ SchemaStore interface in persistence layer (proper home for storage abstraction)
- ✅ In-memory stub now, Redis/S3 backing in Phase 2
- ✅ Validation deferred to Phase 2 (JSON Schema validation library)
- ✅ Domain types (`Document`, `Record`, `Field`) remain stable in riptide-types
- ✅ Each extraction attaches `schema_uri`, not schema struct

**LOC Impact:** +80 LOC (new runtime interface), -[X] LOC (deleted compile-time structs), -1 workspace crate

---

**Task 0.5.2: Merge riptide-config → riptide-utils (0.3 days)**

```bash
# Move config to utils
mv crates/riptide-config/src/* crates/riptide-utils/src/config/

# Update imports
rg "use riptide_config" crates/ | cut -d: -f1 | sort -u | \
  xargs sed -i 's/riptide_config/riptide_utils::config/g'

# Remove crate
rm -rf crates/riptide-config
```

**Files Modified:**
```
CREATE: crates/riptide-utils/src/config/ (move from riptide-config)
UPDATE: Cargo.toml (remove riptide-config from workspace)
UPDATE: All files using riptide_config → riptide_utils::config
DELETE: crates/riptide-config/ (entire crate)
```

**LOC Impact:** 0 (reorganization only), -1 crate

---

**Task 0.5.3: Expand or Remove riptide-test-utils (0.4 days)**

**Options:**

**Option A: Expand (Recommended)**
```rust
// Add comprehensive test utilities
crates/riptide-test-utils/src/
├── builders/      // Test data builders
├── fixtures/      // Common test fixtures
├── mocks/         // Mock implementations of ports
└── assertions/    // Custom assertions
```

**Option B: Remove and distribute**
```bash
# Move utilities to integration tests in each crate
# Delete riptide-test-utils
```

**Decision Required:** Expand for better test support OR distribute to individual crates

**Files Modified (Option A):**
```
EXPAND: crates/riptide-test-utils/src/ (+500 LOC new utilities)
UPDATE: README explaining test utilities
```

**Files Modified (Option B):**
```
MOVE:   Utilities to crates/*/tests/common/
DELETE: crates/riptide-test-utils/ (entire crate)
UPDATE: Cargo.toml (remove from workspace)
```

---

### Sprint 0.5 Summary

**Total Duration:** 1 day
**Crates Removed:** -2 (schemas, config) or -3 (if test-utils removed)
**LOC Impact:** 0 (reorganization, no deletion)
**Complexity Reduction:** Fewer workspace members, clearer boundaries

**Validation:**
```bash
# Verify only expected crates remain
cargo metadata --no-deps | jq '.workspace_members | length'
# Expected: 27 (if -2) or 26 (if -3)

# All tests pass
cargo test --workspace

# Build succeeds
cargo build --workspace
```

**Success Criteria:**
- ✅ Schemas integrated into riptide-types
- ✅ Config integrated into riptide-utils
- ✅ Decision made on test-utils (expand or remove)
- ✅ All imports updated
- ✅ No broken dependencies
- ✅ Full test suite passes

**References:**
- WORKSPACE_CRATE_ANALYSIS.md §5 - Consolidation Recommendations Phase 2

---

## Sprint 0.3: Admin Cleanup (NEW - 0.5 days)

**Priority:** LOW (Tech Debt Removal)
**Source:** API_CRATE_COVERAGE_ANALYSIS.md - Gap #11

### Problem

- `admin_old.rs` (670 LOC) - Obsolete implementation
- `admin.rs` (194 LOC) - Current implementation
- `admin_stub.rs` (13 LOC) - Stub file

**Current Roadmap:** No mention
**Gap:** Tech debt (670 LOC) should be removed

### Tasks

**Task 0.3.1: Verify admin.rs Sufficiency**

```bash
# Check if admin_old.rs is still referenced
rg "admin_old" crates/

# Compare functionality
diff -u admin_old.rs admin.rs > /tmp/admin_diff.txt

# Ensure admin.rs has all required endpoints
cargo test -p riptide-api -- admin
```

**Task 0.3.2: Delete Obsolete File**

**Files Modified:**
```
DELETE:  crates/riptide-api/src/handlers/admin_old.rs (670 LOC)
UPDATE:  crates/riptide-api/src/handlers/mod.rs (remove admin_old reference)
VERIFY:  admin.rs + admin_stub.rs sufficient
```

**Validation:**
```bash
# No references to admin_old
rg "admin_old" crates/ && echo "FAIL: References remain" || echo "PASS"

# Tests still pass
cargo test -p riptide-api

# Build succeeds
cargo build --workspace
```

**Expected LOC Reduction:** 670 LOC deleted

---

## Phase 0 Summary

### Duration Breakdown

| Sprint | Duration | LOC Deleted | LOC Added |
|--------|----------|-------------|-----------|
| 0.1: Deduplication | 3 days | 2,300 | 400 |
| 0.2: Pipeline Consolidation | 2 days | 600-900 | 400 |
| 0.3: Admin Cleanup | 0.5 days | 670 | 0 |
| 0.4: Quick Wins Deduplication ⭐ | 9 days | **2,690** | 0 |
| 0.5: Small Crate Consolidation | 1 day | 0 | 0 |
| **Total** | **15.5 days** | **6,260-6,560** | **800** |

### Total Impact

**Original Plan (Sprint 0.1 only):**
- Duration: 3 days
- LOC Impact: -2,300 LOC deleted
- Files: 8 modified/deleted

**Enhanced Plan v1 (Sprints 0.1-0.3):**
- Duration: 5.5 days
- LOC Impact: -3,570 LOC deleted
- Files: 14 modified/deleted

**Enhanced Plan v2 (ALL Sprints including Workspace Analysis + Crate Consolidation):**
- **Duration:** 15.5 days (~3.1 weeks)
- **LOC Impact:** -6,260 LOC deleted (172% more than original)
- **Crates Removed:** -2 to -3 (schemas, config, optionally test-utils)
- **Files:** 26+ modified/deleted (3x more coverage)
- **Deduplication:** Eliminates 2,690 LOC verified duplicates + domain violations

### Success Metrics

- ✅ **Workspace Deduplication (Sprint 0.4):** 2,690 LOC removed ⭐
  - Robots.txt: -481 LOC (IDENTICAL files, 2 → 1)
  - Circuit breakers: -1,294 LOC (4 implementations → 1, includes domain fix)
  - Redis clients: -533 LOC (2 wrappers → 1)
  - Rate limiters: -382 LOC (2 implementations → 1, stealth pending review)
- ✅ **Local Deduplication (Sprint 0.1):** 2,300 LOC removed
  - robots split + memory managers consolidated
- ✅ **Pipeline Cleanup (Sprint 0.2):** 600-900 LOC removed
- ✅ **Tech Debt (Sprint 0.3):** 670 LOC removed (admin_old.rs)
- ✅ **Redis Scoping:** 4 dependencies removed (from 6 to 2)
- ✅ **Foundation:** CacheStorage trait defined
- ✅ **Single Source of Truth:**
  - Circuit breakers: 1 location (riptide-reliability)
  - Rate limiters: 1 location (riptide-security)
  - Redis clients: 1 location (riptide-persistence)
  - Robots.txt: 1 location (riptide-fetch)
  - Memory managers: 1 location (riptide-pool)

### Risk Mitigation

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Breaking tests during deduplication | MEDIUM | HIGH | Run tests after each file migration |
| Pipeline variants have subtle differences | HIGH | MEDIUM | Comprehensive diff analysis first |
| Admin cleanup breaks endpoints | LOW | MEDIUM | Verify no references before delete |
| Redis migration breaks caching | MEDIUM | HIGH | Feature flag for gradual rollout |

---

## Next Phase

After Phase 0 completion, proceed to **[PHASE_1_PORTS_ADAPTERS_ROADMAP.md](./PHASE_1_PORTS_ADAPTERS_ROADMAP.md)** to define infrastructure ports and adapters.

---

## Document Status

**Version:** 3.1 (Enhanced with API Coverage + Workspace Analysis + Crate Consolidation)
**Status:** ✅ Ready for Implementation - **HIGHEST PRIORITY**
**Date:** 2025-11-08
**Duration:** 15.5 days (~3.1 weeks)
**LOC Impact:** -6,260 LOC deleted (172% improvement - VERIFIED counts)
**Crate Reduction:** -2 to -3 crates (29 → 27 or 26)
**Dependencies:** None (first phase)
**Next Review:** After Sprint 0.5 completion

**Note:** Corrected from initial estimates. All LOC counts verified via `wc -l`. Conservative estimates used where duplication needs review.

**Related Documents:**
- [WORKSPACE_CRATE_ANALYSIS.md](../reports/WORKSPACE_CRATE_ANALYSIS.md) - **NEW!** Comprehensive 29-crate analysis
- [API_CRATE_COVERAGE_ANALYSIS.md](../architecture/API_CRATE_COVERAGE_ANALYSIS.md) - API layer gaps
- [ENHANCED_LAYERING_ROADMAP.md](../architecture/ENHANCED_LAYERING_ROADMAP.md) (master index)
- [PHASE_1_PORTS_ADAPTERS_ROADMAP.md](./PHASE_1_PORTS_ADAPTERS_ROADMAP.md) (next phase)
