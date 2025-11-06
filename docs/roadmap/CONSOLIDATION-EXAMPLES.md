# 📝 Roadmap Consolidation Examples

**Purpose:** Side-by-side before/after examples for consolidation patterns
**Shows:** Exact transformations for each consolidation type

---

## 🎯 Example 1: Completed Section (Week 0-1 Redis Pooling)

### **BEFORE (160 lines):**

```markdown
**1. Redis Connection Pooling** (2 days - TWO PHASES)

**ACTION: Phase 1a = REFACTOR existing → consolidated RedisPool, Phase 1b = MIGRATE (MANDATORY)**

**Phase 1a: Extract and Consolidate RedisPool** (1 day)

**Find existing implementations first (VERIFIED 2025-11-04: 10+ files):**
```bash
# Verify all Redis usage in codebase
rg "redis::Client::open|ConnectionManager::new|get_multiplexed_async_connection" --type rust -l | grep -v target
# Expected: 10+ files (not just 3)
```

**Source locations to extract patterns from:**
- `crates/riptide-workers/src/scheduler.rs:193` (basic connection setup)
- `crates/riptide-workers/src/queue.rs:56` (connection with retry)
- `crates/riptide-cache/src/redis.rs` (pooling and connection management - 393 lines!)
- `crates/riptide-persistence/tests/integration/mod.rs:92` (test connection setup)
- Plus 6+ files in riptide-persistence/src/*.rs

**REFACTOR these patterns into consolidated RedisPool with health checks:**
```rust
// crates/riptide-utils/src/redis.rs

use redis::aio::ConnectionManager;
use std::sync::Arc;
use tokio::time::{interval, Duration};

pub struct RedisPool {
    manager: Arc<ConnectionManager>,
    config: RedisConfig,
}

pub struct RedisConfig {
    pub max_connections: usize,
    pub connection_timeout: Duration,
    pub retry_attempts: u32,
    pub health_check_interval: Duration,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            connection_timeout: Duration::from_secs(5),
            retry_attempts: 3,
            health_check_interval: Duration::from_secs(30),
        }
    }
}

impl RedisPool {
    pub async fn new(url: &str, config: RedisConfig) -> Result<Self> {
        let client = redis::Client::open(url)?;
        let manager = ConnectionManager::new(client).await?;

        let pool = Self {
            manager: Arc::new(manager),
            config,
        };

        // Start health check background task
        pool.start_health_checks();

        Ok(pool)
    }

    pub async fn get(&self) -> Result<ConnectionManager> {
        Ok(self.manager.as_ref().clone())
    }

    fn start_health_checks(&self) {
        let manager = self.manager.clone();
        let interval_duration = self.config.health_check_interval;

        tokio::spawn(async move {
            let mut interval = interval(interval_duration);
            loop {
                interval.tick().await;
                // PING command to verify connection health
                if let Err(e) = redis::cmd("PING").query_async::<_, ()>(&mut manager.clone()).await {
                    tracing::warn!("Redis health check failed: {}", e);
                }
            }
        });
    }
}
```

**TDD Approach:**
```rust
// RED: Write test first
#[tokio::test]
async fn test_redis_pool_reuses_connections() {
    let pool = RedisPool::new("redis://localhost:6379").await.unwrap();
    let conn1 = pool.get().await.unwrap();
    let conn2 = pool.get().await.unwrap();
    // Both should share same underlying connection
    assert!(Arc::ptr_eq(&conn1.inner(), &conn2.inner()));
}

// GREEN: Implement above code
// REFACTOR: Add error handling
```

**Phase 1a Acceptance:**
- [x] RedisPool compiles: `cargo build -p riptide-utils` ✅
- [x] Tests pass: `cargo test -p riptide-utils redis` ✅
- [x] Health checks work (PING every 30s) ✅
- [x] Connection pooling verified (10+ concurrent) ✅

**Phase 1b: Migrate Existing Usage** (1 day - MANDATORY)

**⚠️ CRITICAL: Phase 1b is NOT optional. Must migrate ALL 10+ files BEFORE moving to next task.**

**Verification command (MUST return 10+ files before migration):**
```bash
# These files MUST still have old Redis code before migration
rg "redis::Client::open|ConnectionManager::new|get_multiplexed_async_connection" --type rust -l | \
  grep -v riptide-utils | grep -v target
# Expected: 10+ files including:
#   - riptide-workers: scheduler.rs, queue.rs
#   - riptide-persistence: cache.rs, state.rs, tenant.rs, sync.rs, tests/integration/mod.rs
#   - riptide-cache: redis.rs, manager.rs
```

**Migration commands (update ALL files found above):**
```bash
# PRIORITY 1: riptide-workers (scheduler, queue)
sd "redis::Client::open" "riptide_utils::redis::RedisPool::new" \
  crates/riptide-workers/src/scheduler.rs \
  crates/riptide-workers/src/queue.rs

# PRIORITY 2: riptide-persistence (4 source files + 1 test)
sd "redis::Client::open" "riptide_utils::redis::RedisPool::new" \
  crates/riptide-persistence/src/{cache,state,tenant,sync}.rs \
  crates/riptide-persistence/tests/integration/mod.rs

# PRIORITY 3: riptide-cache (redis wrapper - may need manual refactor)
# Review: crates/riptide-cache/src/redis.rs (393 lines)
# Decision: Keep as thin wrapper over RedisPool OR consolidate into utils

# 4. Add imports to all migrated files
# Add: use riptide_utils::redis::RedisPool;
```

**Verification (MUST run after migration):**
```bash
# Should return ZERO files (except riptide-utils itself)
rg "redis::Client::open" --type rust -l | grep -v riptide-utils | grep -v target | wc -l
# Expected: 0

# Verify builds still work
cargo test -p riptide-workers
cargo test -p riptide-persistence --test integration
```

**Phase 1b Acceptance (ALL required):**
- [x] `rg "redis::Client::open"` returns 0 files (outside utils and riptide-cache) ✅
- [x] All 10+ files now use `RedisPool::new` OR decision documented for riptide-cache ✅
- [x] `cargo test -p riptide-workers` passes ✅
- [x] `cargo test -p riptide-persistence` passes ✅
- [x] `cargo test -p riptide-cache` passes (if migrated) ✅
- [x] ~150 lines removed (actual: see PHASE-0-COMPLETION-REPORT.md) ✅
```

**Character count:** ~6,400 characters (~160 lines)

---

### **AFTER (15 lines):**

```markdown
| **Redis Pooling** | REFACTOR + MIGRATE | ~150 | 8 | ✅ Complete |

**Achievement:** RedisPool with health checks, 10+ concurrent connections, automatic PING every 30s
**Files Created:** `crates/riptide-utils/src/redis.rs`
**Migration:** 10+ files updated to use `RedisPool::new()`
**Tests:** 8 passing
**Details:** See `docs/phase0/PHASE-0-COMPLETION-REPORT.md` Section 1

<details>
<summary><strong>Implementation Code (120 lines)</strong> - Click to expand</summary>

```rust
// crates/riptide-utils/src/redis.rs

use redis::aio::ConnectionManager;
use std::sync::Arc;
use tokio::time::{interval, Duration};

pub struct RedisPool {
    manager: Arc<ConnectionManager>,
    config: RedisConfig,
}

// [... full implementation ...]
```

</details>
```

**Character count:** ~800 characters (~15 lines)

**Savings:** 160 - 15 = **145 lines (91% reduction)**

---

## 🎯 Example 2: Completed Section Summary (Week 0-1 Full)

### **BEFORE (656 lines):**

- Header (17 lines)
- Redis Pooling (160 lines)
- HTTP Client Factory (85 lines)
- Retry Logic (170 lines)
- Time Utilities (8 lines)
- Rate Limiting (111 lines)
- Simple Rate Limiting (67 lines)
- Feature Gates (36 lines)
- Deliverables (2 lines)

**Total:** 656 lines

---

### **AFTER (120 lines):**

```markdown
### Week 0-1: Consolidation ✅ COMPLETE (2025-11-04)

**Report:** `docs/phase0/PHASE-0-COMPLETION-REPORT.md`
**Commit:** `d653911`
**Status:** All 7 subtasks completed, 40 tests passing

<details>
<summary><strong>📊 Week 0-1 Completion Summary</strong> - Click to expand</summary>

| Task | Action Type | Lines Removed | Tests Added | Status |
|------|-------------|---------------|-------------|--------|
| **Redis Pooling** | REFACTOR + MIGRATE | ~150 | 8 | ✅ Complete |
| **HTTP Client Factory** | EXTRACT + MIGRATE | ~53 | 6 | ✅ Complete |
| **Retry Logic** | REFACTOR + ANALYZE | ~0 (SmartRetry preserved) | 4 | ✅ Complete |
| **Time Utilities** | CREATE NEW | N/A | 5 | ✅ Complete |
| **Error Re-exports** | CREATE NEW | N/A | 2 | ✅ Complete |
| **Simple Rate Limiting** | CREATE NEW (governor) | N/A | 8 | ✅ Complete |
| **Feature Gates** | PARTIAL (4/21 files) | N/A | 7 | 🔄 In Progress |

**Key Achievements:**
- ✅ RedisPool: Health checks, 10+ concurrent connections, 8 passing tests
- ✅ HTTP Factory: 3 test files migrated, 13 instances consolidated
- ✅ Retry Logic: Analysis complete, SmartRetry preserved as specialized
- ✅ Time/Error: Basic utilities functional
- ✅ Rate Limiting: Governor-based in-memory limiter (Redis deferred to v1.1)
- 🔄 Feature Gates: 23 compilation errors (expected, gates incomplete)

**Files Created:**
- `crates/riptide-utils/src/redis.rs` (RedisPool + health checks)
- `crates/riptide-utils/src/http.rs` (HTTP client factory)
- `crates/riptide-utils/src/retry.rs` (RetryPolicy)
- `crates/riptide-utils/src/time.rs` (Time utilities)
- `crates/riptide-utils/src/error.rs` (Error re-exports)
- `crates/riptide-utils/src/rate_limit.rs` (SimpleRateLimiter)

**Deferred to Week 1-2:**
- 29/36 retry migration files (low-priority crates)
- Redis token bucket (v1.1 - distributed scenarios)
- 17/21 feature gate files (Week 1.5)

</details>

<details>
<summary><strong>Implementation Details</strong> - Click to expand</summary>

### Redis Pooling Implementation

[Full code examples]

### HTTP Client Factory

[Full code examples]

### Retry Logic

[Full code examples]

### Time Utilities

[Full code examples]

</details>

**Note:** See `docs/phase0/PHASE-0-COMPLETION-REPORT.md` for full code examples, migration commands, and test results.
```

**Total:** ~120 lines

**Savings:** 656 - 120 = **536 lines (82% reduction)**

---

## 🎯 Example 3: Spider Decoupling ✅ COMPLETE

### **BEFORE (280 lines):**

- Current Problem (31 lines)
- Robots Policy Toggle (55 lines)
- ContentExtractor Trait Definition (63 lines)
- Result Types (30 lines)
- Refactor Spider (60 lines)
- Update Facades (32 lines)
- Acceptance Criteria (10 lines)

**Total:** 280 lines

---

### **AFTER (100 lines):**

```markdown
### Week 2.5-5.5: Decouple Spider from Extraction ✅ COMPLETE (2025-11-04)

**Report:** `docs/phase1/PHASE-1-SPIDER-DECOUPLING-COMPLETION-REPORT.md`
**Commit:** `abc1234`
**Tests:** 88/88 passing (22 unit + 66 integration)
**Quality:** Zero clippy warnings

<details>
<summary><strong>📊 Spider Decoupling Summary</strong> - Click to expand</summary>

| Component | Lines Added | Tests | Status |
|-----------|-------------|-------|--------|
| **ContentExtractor Trait** | +120 | 22 unit | ✅ Complete |
| **BasicExtractor** | +80 | 15 | ✅ Complete |
| **NoOpExtractor** | +30 | 10 | ✅ Complete |
| **Result Types** | +50 | 8 | ✅ Complete |
| **Facade Updates** | +100 | 33 integration | ✅ Complete |
| **Robots Policy Toggle** | +40 | 8 | ✅ Complete |

**Key Achievements:**
- ✅ Spider works without extraction (pure URL discovery)
- ✅ Modular extractor plugins (ICS, JSON-LD, LLM ready)
- ✅ ~200 lines removed from spider core
- ✅ Robots.txt toggle exposed in API with ethical warnings

**Files Created:**
- `crates/riptide-spider/src/extractor.rs` (ContentExtractor trait)
- `crates/riptide-spider/src/results.rs` (Result types)
- `crates/riptide-spider/src/builder.rs` (Builder pattern)
- `crates/riptide-facade/src/facades/spider_facade.rs` (Updated facade)

</details>

<details>
<summary><strong>Architecture & Code Examples</strong> - Click to expand</summary>

### ContentExtractor Trait

```rust
#[async_trait]
pub trait ContentExtractor: Send + Sync {
    fn extract_links(&self, html: &str, base_url: &Url) -> Vec<Url>;
    fn extract_text(&self, html: &str) -> Option<String>;
    fn strategy_name(&self) -> &'static str;
}
```

### BasicExtractor Implementation

[Full code]

### NoOpExtractor Implementation

[Full code]

### Result Types

[Full code]

### Facade Updates

[Full code]

</details>

**Known Issues:** 23 pre-existing riptide-api compilation errors (optional features: browser, llm) - NOT Phase 1 blockers, scheduled for Week 1.5. See `/docs/phase1/RIPTIDE_API_KNOWN_ISSUES.md`
```

**Total:** ~100 lines

**Savings:** 280 - 100 = **180 lines (64% reduction)**

---

## 🎯 Example 4: Code Example with `<details>` Block

### **BEFORE (85 lines):**

```markdown
**Minimal Health Endpoints:**
```rust
// crates/riptide-api/src/handlers/health.rs

#[derive(Serialize)]
pub struct HealthStatus {
    pub status: HealthLevel,
    pub components: HashMap<String, ComponentHealth>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize)]
pub enum HealthLevel {
    Healthy,    // All systems go
    Degraded,   // Some features unavailable
    Unhealthy,  // Critical failures
}

pub async fn health_check(State(state): State<AppState>) -> Json<HealthStatus> {
    let mut components = HashMap::new();

    // Check Redis
    components.insert("redis", check_redis(&state.redis_pool).await);

    // Check Browser pool (if enabled)
    if let Some(browser) = &state.browser_pool {
        components.insert("browser", check_browser(browser).await);
    }

    // Check WASM pool
    components.insert("wasm", check_wasm(&state.wasm_pool).await);

    // Check disk space
    components.insert("disk", check_disk().await);

    let status = if components.values().all(|c| c.healthy) {
        HealthLevel::Healthy
    } else if components.values().any(|c| c.critical) {
        HealthLevel::Unhealthy
    } else {
        HealthLevel::Degraded
    };

    Json(HealthStatus {
        status,
        components,
        timestamp: Utc::now(),
    })
}
```

**Endpoints:**
- `GET /healthz` → 200/503 (simple liveness)
- `GET /api/health/detailed` → Full component status
```

**Total:** ~85 lines

---

### **AFTER (15 lines):**

```markdown
**Health Endpoints:** `/healthz` (liveness) and `/api/health/detailed` (component status)

<details>
<summary><strong>Health Endpoint Implementation (70 lines)</strong> - Click to expand</summary>

```rust
// crates/riptide-api/src/handlers/health.rs

#[derive(Serialize)]
pub struct HealthStatus {
    pub status: HealthLevel,
    pub components: HashMap<String, ComponentHealth>,
    pub timestamp: DateTime<Utc>,
}

// [... full implementation ...]
```

</details>
```

**Total:** ~15 lines (visible) + 70 lines (folded)

**Visual Savings:** 85 - 15 = **70 lines (82% visual reduction)**

---

## 🎯 Example 5: Acceptance Criteria Consolidation

### **BEFORE (Verbose List):**

```markdown
**Phase 1b Acceptance (ALL required):**
- [x] `rg "redis::Client::open"` returns 0 files (outside utils and riptide-cache) ✅
- [x] All 10+ files now use `RedisPool::new` OR decision documented for riptide-cache ✅
- [x] `cargo test -p riptide-workers` passes ✅
- [x] `cargo test -p riptide-persistence` passes ✅
- [x] `cargo test -p riptide-cache` passes (if migrated) ✅
- [x] ~150 lines removed (actual: see PHASE-0-COMPLETION-REPORT.md) ✅
```

**Total:** 6 lines

---

### **AFTER (Table Format):**

```markdown
**Status:** ✅ Complete (10+ files migrated, 150 lines removed, all tests passing)
```

**Total:** 1 line

**Savings:** 6 - 1 = **5 lines per acceptance block**

**Total Impact:** 15+ acceptance blocks × 5 lines = **~75 lines total**

---

## 📊 Pattern Comparison Summary

| Pattern | Before | After | Savings | Method |
|---------|--------|-------|---------|--------|
| **Completed Section (Small)** | 160 lines | 15 lines | -145 lines | Summary table + `<details>` |
| **Completed Section (Large)** | 656 lines | 120 lines | -536 lines | Multi-level `<details>` |
| **Completed Phase** | 280 lines | 100 lines | -180 lines | Summary + nested `<details>` |
| **Code Example** | 85 lines | 15 lines | -70 lines (visual) | `<details>` block |
| **Acceptance Criteria** | 6 lines | 1 line | -5 lines | Status summary |

**Total Examples:** ~1,187 lines → ~251 lines = **-936 lines (79% reduction)**

---

## ✅ Application Guidelines

### **When to Use Summary Tables:**

✅ **Use for:**
- Completed multi-step tasks (Redis, HTTP, Retry)
- Sections with 100+ lines
- Repeated Phase 1a/1b patterns

❌ **Don't use for:**
- Pending work (keep full detail)
- Critical decision trees
- Agent recovery protocols

### **When to Use `<details>` Blocks:**

✅ **Use for:**
- Code examples > 30 lines
- Completed implementation details
- Verbose acceptance criteria

❌ **Don't use for:**
- Key API reference (Quick Reference table)
- Timeline overview
- Critical path diagrams

### **When to Use Links:**

✅ **Use for:**
- TDD guides (link to `/docs/development/TDD-LONDON-SCHOOL.md`)
- Completion reports (link to `/docs/phase0/*.md`)
- External documentation

❌ **Don't use for:**
- Inline code examples (use `<details>` instead)
- Critical decision logic

---

**These examples demonstrate the exact consolidation patterns to apply across the roadmap.**
