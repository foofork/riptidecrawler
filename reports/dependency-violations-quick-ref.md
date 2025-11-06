# Dependency Violations - Quick Reference Card

**Generated:** 2025-11-06
**For:** RipTide Development Team

---

## 🚨 Active Violations (5 Total)

### 1. ✅ RESOLVED: API ↔ Facade Circular Dependency
**Status:** Fixed in Phase 2C.2
**Solution:** Trait extraction to `riptide-types`

---

### 2. 🔴 CRITICAL: Facade → 8+ Domain Dependencies

**File:** `/workspaces/eventmesh/crates/riptide-facade/Cargo.toml`

**Problem:**
```toml
riptide-fetch = { path = "../riptide-fetch" }
riptide-extraction = { path = "../riptide-extraction" }
riptide-pdf = { path = "../riptide-pdf" }
riptide-cache = { path = "../riptide-cache" }
riptide-browser = { path = "../riptide-browser" }
riptide-stealth = { path = "../riptide-stealth" }
riptide-spider = { path = "../riptide-spider" }
riptide-search = { path = "../riptide-search" }
```

**Why it's bad:**
- Tight coupling to concrete implementations
- Cannot test facade in isolation
- Cannot swap implementations

**Fix:**
```rust
// BEFORE (wrong)
use riptide_spider::Spider;
use riptide_extraction::Extractor;

pub struct Facade {
    spider: Spider,
    extractor: Extractor,
}

// AFTER (correct)
use riptide_types::facade_traits::{CrawlStrategy, ExtractionStrategy};

pub struct Facade {
    crawler: Arc<dyn CrawlStrategy>,
    extractor: Arc<dyn ExtractionStrategy>,
}
```

---

### 3. 🔴 CRITICAL: Cache → Domain Circular Dependency

**File:** `/workspaces/eventmesh/crates/riptide-cache/Cargo.toml`

**Problem:**
```toml
riptide-pool = { path = "../riptide-pool" }             # Domain crate
riptide-extraction = { path = "../riptide-extraction" } # Domain crate
```

**Why it's bad:**
- Infrastructure importing business logic
- Circular dependency: `cache → extraction → cache`
- Cannot replace cache implementation

**Fix:**
Move pooling logic OUT of cache:
```rust
// riptide-cache should ONLY provide caching
// riptide-pool should manage extraction instance pooling

// REMOVE from riptide-cache/Cargo.toml:
# riptide-pool
# riptide-extraction

// ADD to riptide-pool/src/lib.rs:
// Pool management logic that was in cache
```

---

### 4. 🟡 MEDIUM: Spider → Fetch Sideways Dependency

**File:** `/workspaces/eventmesh/crates/riptide-spider/Cargo.toml`

**Problem:**
```toml
riptide-fetch = { path = "../riptide-fetch" }  # Sideways domain dependency
```

**Why it's concerning:**
- Two domain crates coupling
- Easy to accidentally create circular dependency
- Layering ambiguity

**Fix Option A:** Make fetch foundational
```toml
# Move riptide-fetch to foundation layer
# It's just an HTTP client, should be available to all domain crates
```

**Fix Option B:** Use trait abstraction
```rust
use riptide_types::HttpClient;

pub struct Spider {
    http: Arc<dyn HttpClient>,
}
```

---

### 5. 🔴 CRITICAL: Pipeline → Infrastructure Direct Dependency

**File:** `/workspaces/eventmesh/crates/riptide-pipeline/Cargo.toml`

**Problem:**
```toml
redis = { workspace = true }  # Direct infrastructure dependency
```

**Why it's bad:**
- Domain tied to specific database
- Cannot test without Redis running
- Cannot swap to alternative (DragonflyDB, Memcached)

**Fix:**
```rust
// BEFORE (wrong)
use redis::Client as RedisClient;

pub struct Pipeline {
    redis: RedisClient,
}

// AFTER (correct)
use riptide_types::KeyValueStore;

pub struct Pipeline {
    storage: Arc<dyn KeyValueStore>,
}

// riptide-cache implements KeyValueStore
impl KeyValueStore for RedisCache { ... }
```

---

## 📐 Architecture Rule

```
API → FACADE → DOMAIN → INFRASTRUCTURE
 ↓      ↓        ↓          ↓
     FOUNDATION (types, config, events)
```

**Rules:**
1. ✅ Dependencies flow **downward** only
2. ❌ No **sideways** dependencies in domain
3. ✅ Domain uses **traits**, not concrete infrastructure
4. ✅ All layers can use **foundation** crates

---

## 🔧 Quick Fixes

### When adding a dependency, ask:

#### ❓ "Should I add this to Cargo.toml?"

```bash
# ✅ YES if:
- It's a foundation crate (riptide-types, riptide-events)
- It's an external workspace dependency (tokio, serde)
- You're implementing a trait from riptide-types

# ❌ NO if:
- It's a domain crate in facade layer
- It's infrastructure (redis, postgres) in domain
- It creates a circular dependency
```

#### ❓ "Should I use a trait or concrete type?"

```rust
// ✅ USE TRAIT when:
pub struct Facade {
    crawler: Arc<dyn CrawlStrategy>,  // Interface
}

// ❌ USE CONCRETE when:
pub struct Facade {
    crawler: Spider,  // Tight coupling
}
```

#### ❓ "Where should this trait be defined?"

```bash
# ✅ TRAITS go in riptide-types:
/workspaces/eventmesh/crates/riptide-types/src/
├── facade_traits.rs       # Facade layer traits
├── storage_traits.rs      # Infrastructure traits
├── http_traits.rs         # HTTP client traits
└── pipeline_traits.rs     # Pipeline orchestration traits

# ❌ NOT in the implementation crate
```

---

## 🧪 Testing for Violations

### Before committing, run:

```bash
# Check for circular dependencies
cargo tree --workspace --duplicates

# Check specific crate dependencies
cargo tree -p riptide-facade --depth 2
cargo tree -p riptide-pipeline --depth 2

# Ensure no warnings
RUSTFLAGS="-D warnings" cargo clippy --workspace

# Run full test suite
cargo test --workspace
```

### Red flags to watch for:

```toml
# 🚨 RED FLAG 1: Domain crate importing another domain crate
[dependencies]
riptide-spider = { path = "../riptide-spider" }  # If you're in riptide-fetch

# 🚨 RED FLAG 2: Infrastructure in domain
[dependencies]
redis = { workspace = true }  # If you're in riptide-pipeline

# 🚨 RED FLAG 3: Concrete types in facade
[dependencies]
riptide-extraction = { path = "../riptide-extraction" }  # If you're in riptide-facade
```

---

## 📋 Checklist for New Features

When adding a new feature:

- [ ] Traits defined in `riptide-types`
- [ ] Domain crate implements trait
- [ ] Infrastructure crate implements trait (if needed)
- [ ] Facade uses trait object (`Arc<dyn Trait>`)
- [ ] API injects concrete implementation
- [ ] No sideways domain dependencies
- [ ] No infrastructure in domain
- [ ] `cargo tree` shows clean dependency flow
- [ ] `cargo clippy` passes with no warnings
- [ ] Unit tests with mocks pass
- [ ] Integration tests with real implementations pass

---

## 🎯 Priority Actions

### This Week:
1. Extract traits to `riptide-types` (Phase 1)
2. Fix `riptide-cache` domain dependencies (Violation 3)
3. Remove Redis from `riptide-pipeline` (Violation 5)

### Next Week:
4. Refactor `riptide-facade` to use traits (Violation 2)
5. Resolve `riptide-spider` → `riptide-fetch` coupling (Violation 4)

### Following Week:
6. Update API with dependency injection
7. Full integration testing

---

## 📞 Need Help?

**Architecture questions:**
- Review: `/workspaces/eventmesh/reports/dependency-flow-analysis.md`
- Diagrams: `/workspaces/eventmesh/reports/dependency-graph.mermaid`

**Example trait implementations:**
```rust
// See Phase 2C.2 fix in riptide-api for reference:
// - PipelineExecutor trait extraction
// - Trait-based dependency injection
```

**Testing:**
```bash
# Test with mocks
cargo test -p riptide-facade --features mock-domain

# Test with real implementations
cargo test -p riptide-api --features full
```

---

## 🎨 Visual Summary

```
Current (5 violations):
┌─────────┐
│   API   │──┐
└─────────┘  │
             ▼
┌─────────┐  🔴 Direct coupling
│ FACADE  │────▶ 8+ domain crates
└─────────┘

┌─────────┐  🔴 Sideways
│  Cache  │────▶ Pool + Extraction
└─────────┘

┌─────────┐  🔴 Infrastructure
│Pipeline │────▶ Redis (direct)
└─────────┘

Target (0 violations):
┌─────────┐
│   API   │ injects
└─────────┘
     │
     ▼
┌─────────┐
│ FACADE  │ uses traits
└─────────┘
     │
     ▼
┌─────────┐
│  TYPES  │ defines all traits
└─────────┘
     ▲
     │ implements
     │
[Domain + Infrastructure]
```

---

**Remember:** When in doubt, use a trait! 🎭
