# Architecture Roadmap Clarifications & Refinements

**Status:** ✅ Approved for Execution
**Date:** 2025-11-08
**Phase:** Pre-Phase 0 (Ready to Begin)

---

## 📋 Overview

This document captures the critical clarifications and refinements to the Enhanced Layering Roadmap based on architectural review. These rules MUST be followed during execution.

---

## 🧱 1. Domain & Dependency Rules

### Rule: Keep Domain Pure

**Domain Layer (riptide-types):**
- ✅ Contains: Pure domain types, value objects, business rules, **port traits**
- ❌ NO imports from: `riptide-api`, `riptide-facade`, or ANY infrastructure crate
- ❌ NO imports from: `riptide-reliability`, `riptide-cache`, `riptide-browser`, `riptide-pdf`, `riptide-spider`, `riptide-search`, `riptide-persistence`, `riptide-fetch`, `riptide-pool`

### CI Enforcement

```bash
# This command MUST return no matches
cargo tree -p riptide-types --invert riptide-types | \
  grep -iE 'riptide-(api|facade|reliability|cache|browser|pdf|spider|search|persistence|fetch|pool)'

# Expected: No output (exit code 1 when piped to grep)
```

**GitHub Action:** Added to `.github/workflows/architecture-validation.yml`

### Port Definitions

**Ports live in `riptide-types`:**
- Domain and facades depend ONLY on these traits
- Infrastructure implements them
- Composition root (AppState/main) wires concrete implementations

---

## 🧩 2. Facade / Application Layer

### Clarification: NO Rename Needed

**`riptide-facade` IS the Application (Use-Case) Layer**
- No crate rename required
- Add clear documentation at crate level

### Required Documentation

Add to `crates/riptide-facade/src/lib.rs`:

```rust
//! # Riptide Facade - Application Layer (Use-Cases)
//!
//! This crate contains application use-cases that orchestrate domain logic via ports.
//!
//! ## Architectural Rules
//!
//! - NO HTTP types (actix_web, hyper, etc.)
//! - NO database types (sqlx, postgres, etc.)
//! - NO serialization formats (serde_json::Value - use typed DTOs)
//! - NO SDK/client types (redis, reqwest, etc.)
//!
//! ## What Lives Here
//!
//! - Use-case orchestration (workflows, transactions)
//! - Cross-cutting concerns (retry, timeout, circuit breaker coordination)
//! - Authorization policies
//! - Idempotency management
//! - Domain event emission
//! - Transactional outbox writes
//!
//! ## Dependencies
//!
//! - ONLY `riptide-types` (for domain types and port traits)
//! - Common utilities: `riptide-config`, `riptide-events`, `riptide-monitoring`
//! - NO infrastructure crates
```

### Handler Rule Clarification

**Handlers: <50 LOC, I/O-only**
- Simple `if` statements for input validation: ✅ **ALLOWED**
- Business logic loops (`for`, `while`, `loop`): ❌ **FORBIDDEN** (belongs in facades/domain)

**Example - Allowed:**
```rust
pub async fn handle_request(req: Request) -> Result<Response> {
    // Input validation - OK
    if req.url.is_empty() {
        return Err(Error::InvalidInput("URL required"));
    }

    // Call facade - OK
    let result = facade.execute(req.into()).await?;

    // Map response - OK
    Ok(result.into())
}
```

**Example - Forbidden:**
```rust
pub async fn handle_request(req: Request) -> Result<Response> {
    // ❌ Business logic loop in handler!
    for url in req.urls {
        process_url(url).await?;
    }

    // ❌ Direct infrastructure access!
    let redis = state.redis_pool.get().await?;
    redis.set("key", "value").await?;
}
```

### CI Enforcement

```bash
# Check for transitive dependencies
cargo tree -p riptide-facade | grep -iE 'axum|actix-web|hyper|reqwest|redis|sqlx'
# Expected: No matches

# Check for HTTP types in source
find crates/riptide-facade/src -name "*.rs" -exec grep "actix_web::\|hyper::\|reqwest::" {} +
# Expected: No output

# Check for business logic loops in handlers
find crates/riptide-api/src/handlers -name "*.rs" -exec grep -n '\b\(for\|while\|loop\)\b' {} + | \
  grep -v "//"  # Exclude comments
# Expected: Minimal matches (only in validation contexts)
```

---

## 🔌 3. Ports & Adapters Scope

### Start Lean - Phase 1-2 Essential Ports Only

**Implement first:**

1. **Transaction Management**
   - `trait TransactionManager`
   - `trait Repository<T>`

2. **Event Handling**
   - `trait EventBus`
   - `trait OutboxStore`

3. **Caching & State**
   - `trait CacheStorage`
   - `trait IdempotencyStore`

4. **Testing Utilities**
   - `trait Clock` (for deterministic time)
   - `trait Entropy` (for deterministic randomness)

5. **Feature Ports** (based on current use)
   - `trait BrowserDriver`
   - `trait PdfProcessor`
   - `trait SearchEngine`

**Expand later** as new facades require them.

### Outbox Pattern Clarification

**Outbox MUST be a port (EventBus interface)**
- PostgreSQL outbox table: just one adapter implementation
- Allow for future: Kafka, RabbitMQ, in-memory (for tests)

```rust
// Port definition (in riptide-types)
#[async_trait]
pub trait EventBus: Send + Sync {
    async fn publish(&self, events: Vec<DomainEvent>) -> Result<()>;
    async fn subscribe(&self, handler: Box<dyn EventHandler>) -> Result<()>;
}

// Postgres adapter (in riptide-persistence)
pub struct PostgresEventBus {
    pool: PgPool,
}

#[async_trait]
impl EventBus for PostgresEventBus {
    async fn publish(&self, events: Vec<DomainEvent>) -> Result<()> {
        // Write to outbox table within transaction
        // Background worker polls and publishes
    }
}

// In-memory adapter (for tests)
pub struct InMemoryEventBus {
    events: Arc<Mutex<Vec<DomainEvent>>>,
}
```

---

## ⚙️ 4. Infrastructure / Robots / Redis

### Robots.rs Split Strategy

**Split pure parsing/policy from HTTP/retry logic:**

1. **Pure logic → `riptide-utils/src/robots.rs`**
   - `robots.txt` parsing
   - Rule evaluation
   - NO HTTP, NO async I/O

2. **HTTP fetch/retry → `riptide-reliability`**
   - Circuit breaker
   - Retry logic
   - Timeout handling
   - Uses `ReliableHttpClient`

```rust
// riptide-utils (pure)
pub struct RobotsPolicy {
    rules: Vec<Rule>,
}

impl RobotsPolicy {
    pub fn parse(content: &str) -> Result<Self> { /* ... */ }
    pub fn is_allowed(&self, path: &str, user_agent: &str) -> bool { /* ... */ }
}

// riptide-reliability (I/O)
pub struct RobotsFetcher {
    http_client: Arc<ReliableHttpClient>,
}

impl RobotsFetcher {
    pub async fn fetch_policy(&self, domain: &str) -> Result<RobotsPolicy> {
        let content = self.http_client.get(&format!("https://{}/robots.txt", domain)).await?;
        RobotsPolicy::parse(&content)
    }
}
```

### Redis Consolidation Rules

**Single pooled client in `riptide-cache`:**

```rust
// riptide-cache/src/redis_manager.rs
pub struct RedisManager {
    pool: deadpool_redis::Pool,
}

impl RedisManager {
    // Allowed use cases only
    pub async fn cache_get(&self, key: &str) -> Result<Option<String>> { /* ... */ }
    pub async fn cache_set(&self, key: &str, value: &str, ttl: Duration) -> Result<()> { /* ... */ }
    pub async fn idempotency_check(&self, key: &str, ttl: Duration) -> Result<bool> { /* ... */ }
    pub async fn rate_limit(&self, key: &str, limit: u32, window: Duration) -> Result<bool> { /* ... */ }
    pub async fn acquire_lock(&self, key: &str, ttl: Duration) -> Result<Option<String>> { /* ... */ }
    pub async fn release_lock(&self, key: &str, token: &str) -> Result<bool> { /* ... */ }
}
```

**Usage Limits:**
- ✅ Cache (1h-1d TTL)
- ✅ Idempotency keys (5min TTL)
- ✅ Rate limits (1min window)
- ✅ Short locks (10sec TTL)
- ❌ Primary persistence
- ❌ Core event bus
- ❌ Long-term state

**CI Enforcement:**

```bash
# Count crates with Redis dependency
find crates -name Cargo.toml -exec grep -l redis {} \; | wc -l
# Expected: ≤2 (riptide-cache, maybe riptide-persistence)

# Check Redis usage in source code
rg -n '\bredis::' crates/ | grep -v 'riptide-\(cache\|workers\|persistence\)'
# Expected: No matches
```

---

## 🧪 5. CI / Validation Enhancements

### Enhanced `validate_architecture.sh`

**Implemented in `/workspaces/eventmesh/scripts/validate_architecture_enhanced.sh`**

**12 Automated Checks:**

1. ✅ Domain layer purity (transitive deps)
2. ✅ Facade dependencies (only riptide-types)
3. ✅ Handler sizes (<50 LOC target)
4. ✅ Handler business logic (loop detection)
5. ✅ Redis scope (≤2 crates)
6. ✅ Deduplication (robots.rs, memory_manager.rs)
7. ✅ Build with zero warnings (`RUSTFLAGS="-D warnings"`)
8. ✅ Clippy strict mode (`-D warnings`)
9. ✅ Circular dependencies (cargo tree)
10. ✅ Test coverage (facades ≥90%)
11. ✅ JSON/HTTP leaks in ports
12. ✅ Performance baseline

**cargo-deny Configuration**

**Created: `/workspaces/eventmesh/deny.toml`**

Enforces layer boundaries at dependency level:
- Domain cannot depend on HTTP frameworks
- Facades cannot depend on databases
- Redis scoped to allowed crates
- Centralized HTTP clients

**Run with:**
```bash
cargo install cargo-deny  # One-time
cargo deny check          # In CI and pre-commit
```

### Maintain ≥90% Coverage on Facades

```bash
# Install tarpaulin (one-time)
cargo install cargo-tarpaulin

# Generate coverage for facades
cargo tarpaulin --out Html --output-dir coverage \
  --packages riptide-facade \
  --target-dir target/tarpaulin

# Open coverage/index.html to view
# Fail CI if <90%
```

### Clippy Clean Workspace

```bash
# Must pass with zero warnings
cargo clippy --workspace --all-targets -- -D warnings
```

---

## 🧠 6. Execution Guidance

### Phase 0: Exactly as Defined

**DO NOT add new abstractions until Phase 0 passes all validations:**

1. ✅ Consolidate robots.rs
2. ✅ Consolidate memory_manager.rs
3. ✅ Scope Redis to ≤2 crates
4. ✅ Run `validate_architecture_enhanced.sh`
5. ✅ All checks GREEN

### Feature Flags per New Facade

**Default OFF in production:**

```toml
# crates/riptide-facade/Cargo.toml
[features]
default = []
new-trace-facade = []
new-llm-facade = []
new-profiling-facade = []

# Enable in development/testing
experimental = ["new-trace-facade", "new-llm-facade", "new-profiling-facade"]
```

**In code:**
```rust
#[cfg(feature = "new-trace-facade")]
pub mod trace;

// Composition root
#[cfg(feature = "new-trace-facade")]
let trace_facade = TraceFacade::new(deps);
#[cfg(not(feature = "new-trace-facade"))]
let trace_facade = LegacyTraceHandler::new(deps);
```

### Performance Baseline Capture

**Before refactoring:**

```bash
# Install criterion (one-time)
cargo install cargo-criterion

# Run benchmarks and save baseline
cargo bench --workspace -- --save-baseline main

# Store in git
git add benches/baseline_metrics.json
git commit -m "chore: capture performance baseline before refactor"
```

**After each phase:**

```bash
# Run benchmarks against baseline
cargo bench --workspace -- --baseline main

# Fail if regression >10%
# (criterion will show comparison automatically)
```

### Cache as Optional (Graceful Degradation)

**All Redis usage must have fallback:**

```rust
impl CacheStorage for RedisCacheAdapter {
    async fn get(&self, key: &str) -> Result<Option<String>> {
        match self.redis.get(key).await {
            Ok(value) => Ok(Some(value)),
            Err(e) => {
                warn!("Redis cache miss (degraded): {}", e);
                // Fallback to database or recompute
                Ok(None)
            }
        }
    }
}
```

---

## ✅ Acceptance Summary

### Criteria Checklist

| Check | Target | Validation Method |
|-------|--------|-------------------|
| **Handlers** | <50 LOC, no loops | Script + manual review |
| **Domain deps** | None outside riptide-types | `cargo tree` + CI |
| **Facade deps** | Only riptide-types | `cargo tree` + CI |
| **Redis crates** | ≤2 | Script + CI |
| **JSON/HTTP in facades** | 0 | `grep` + CI |
| **Facade coverage** | ≥90% | `cargo tarpaulin` |
| **Clippy warnings** | 0 | `cargo clippy -D warnings` |
| **Circular deps** | 0 | `cargo tree --duplicates` |

### Automated Validation

**Scripts:**
- `/workspaces/eventmesh/scripts/validate_architecture_enhanced.sh` (comprehensive)
- `.github/workflows/architecture-validation.yml` (CI/CD)
- `/workspaces/eventmesh/deny.toml` (cargo-deny config)

**Pre-commit hook:**
```bash
# .git/hooks/pre-commit
#!/bin/bash
./scripts/validate_architecture_enhanced.sh
if [ $? -ne 0 ]; then
    echo "Architecture validation failed. Fix issues before committing."
    exit 1
fi
```

---

## 📘 Final Recommendation

### ✅ APPROVED: Proceed with Phase 0 Immediately

**Phase 0 - Deduplication & Redis Scope (Week 0)**
1. Consolidate robots.rs to riptide-utils (Day 1)
2. Consolidate memory_manager.rs to riptide-pool (Day 2)
3. Scope Redis to ≤2 crates (Day 3)
4. Run validation & fix issues (Days 4-5)

**Before merging Phase 1 PRs:**
1. ✅ Update `validate_architecture_enhanced.sh` (DONE)
2. ✅ Add GitHub Actions workflow (DONE)
3. ✅ Configure cargo-deny (DONE)
4. ✅ Capture performance baseline
5. ✅ All Phase 0 validations GREEN

### No Structural Redesign Required

The roadmap is **solid and approved** with the clarifications above applied.

---

**Status:** ✅ **Ready for Implementation**
**Next Step:** Begin Phase 0 - Day 1 (Consolidate robots.rs)
