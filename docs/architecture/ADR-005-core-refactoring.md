# ADR-005: Riptide Core Refactoring - Module Boundary Clarification

**Status:** IN PROGRESS
**Date:** 2025-10-17
**Authors:** Architecture Team (Swarm: swarm_1760709536951_i98hegexl)
**Phase:** P1-A2, P1-A3 (Week 2)

---

## Context

The `riptide-core` crate has grown to **39,604 lines** across 80+ source files, resulting in:

### Problems Identified

1. **Unclear Module Boundaries**
   - Configuration management mixed with orchestration logic
   - Browser pool management in wrong crate (riptide-headless)
   - Caching logic scattered across riptide-core and riptide-cli
   - 10+ internal dependencies creating tight coupling

2. **Excessive Crate Size**
   - `riptide-core/src/memory_manager.rs`: 1,107 lines
   - `riptide-core/src/spider/core.rs`: 1,013 lines
   - `riptide-core/src/instance_pool/pool.rs`: 1,005 lines
   - Total core: ~2,500 lines just in top 3 files

3. **Circular Dependency Risk**
   - `riptide-core` ↔ `riptide-extraction` potential circular ref
   - Multiple crates all depending directly on core
   - Shared types not properly isolated

### Current Architecture

```
riptide-core (39,604 lines)
├── Configuration (mixed in)
├── Browser Pool (should be separate)
├── Cache Management (partially in CLI)
├── Spider Logic
├── Memory Management
├── Events & Monitoring
└── Strategy Composition
```

### Dependency Graph (Before Refactoring)

```
All crates → riptide-core (central bottleneck)
            ↓
    riptide-types (1,200 lines - underutilized)
```

**Problems:**
- **10+ crates** depend directly on riptide-core
- riptide-core has **8+ dependencies** on other riptide crates
- No clear layering or separation of concerns
- Changes to core impact all crates

---

## Decision

### Phase 1: Extract Configuration (riptide-config)

**Rationale:** Configuration should be separate from orchestration logic. This allows:
- Independent configuration updates without rebuilding core
- Clear configuration API
- Better testability
- Reusability across crates

**Extracted Components:**
- `riptide-core/src/common/config_builder.rs` → `riptide-config/src/builder.rs`
- `riptide-core/src/common/validation.rs` → `riptide-config/src/validation.rs`
- Environment variable handling
- Configuration validation and type conversion

**Size:** ~1,200 lines

### Phase 2: Extract Browser Engine (riptide-engine)

**Rationale:** Browser pool management is currently split between riptide-headless and riptide-core. Consolidating into a dedicated crate:
- Centralizes browser lifecycle management
- Enables spider-chrome integration (Phase 1 Week 2 goal)
- Reduces riptide-core complexity
- Clear API for browser operations

**Extracted Components:**
- `riptide-headless/src/pool.rs` (1,324 lines) → `riptide-engine/src/pool.rs`
- `riptide-core/src/instance_pool/` → `riptide-engine/src/instance_pool.rs`
- CDP connection management
- Engine selection logic (chromiumoxide vs spider-chrome)
- Browser health monitoring

**Size:** ~2,500 lines

### Phase 3: Extract Caching (riptide-cache)

**Rationale:** Caching logic is currently scattered across multiple crates. Consolidation provides:
- Unified caching interface
- Better cache strategy management
- Easier testing and optimization
- Clear cache invalidation patterns

**Extracted Components:**
- `riptide-core/src/cache.rs` (381 lines) → `riptide-cache/src/core.rs`
- `riptide-core/src/cache_key.rs` (313 lines) → `riptide-cache/src/key.rs`
- `riptide-core/src/cache_warming.rs` (881 lines) → `riptide-cache/src/warming.rs`
- `riptide-core/src/integrated_cache.rs` (402 lines) → `riptide-cache/src/integrated.rs`
- `riptide-cli/src/cache.rs` → `riptide-cache/src/domain_selection.rs`

**Size:** ~2,200 lines

### Phase 4: Slim Down riptide-core

**After Refactoring:**
```
riptide-core (target: <15,000 lines)
├── Orchestration & Workflows
├── Strategy Composition
├── Spider Logic
├── Memory Management
├── Events & Monitoring
└── Circuit Breakers & Reliability
```

**Dependencies (After):**
```
riptide-core → riptide-types
            → riptide-config
            → riptide-engine
            → riptide-cache
```

---

## New Dependency Graph

### Target Architecture

```
┌─────────────────────────────────────────────┐
│              riptide-types                  │
│    (Shared types, traits, interfaces)      │
└────────────┬────────────────────────────────┘
             ↑ (no dependencies)
             │
    ┌────────┴─────────┬──────────┬───────────┐
    │                  │          │           │
┌───┴────┐     ┌──────┴───┐   ┌──┴────┐   ┌──┴────┐
│ config │     │  engine  │   │ cache │   │ core  │
│        │     │          │   │       │   │       │
│ 1.2K   │     │  2.5K    │   │ 2.2K  │   │ 15K   │
└────────┘     └──────────┘   └───────┘   └───┬───┘
                                               │
                    ┌──────────────────────────┘
                    │
         ┌──────────┴─────────┬──────────┬─────────┐
         │                    │          │         │
    ┌────┴────┐        ┌─────┴─────┐   │         │
    │ extract │        │ search    │   │         │
    └─────────┘        └───────────┘   │         │
                                        │         │
                                   ┌────┴────┐   │
                                   │   API   │   │
                                   └─────────┘   │
                                                 └─→ (other crates)
```

**Benefits:**
- ✅ **Clear layering**: types → config/engine/cache → core → features → api
- ✅ **No circular dependencies**: enforced by crate boundaries
- ✅ **Smaller compilation units**: faster rebuilds (4 crates vs 1 monolith)
- ✅ **Better testability**: isolated testing per crate
- ✅ **Clearer ownership**: each crate has single responsibility

---

## Implementation Plan

### Week 1: Preparation & Types Crate

**Tasks:**
1. ✅ Audit riptide-types crate (already exists, 1,200 lines)
2. ✅ Identify shared types to extract from core
3. ✅ Document dependency graph
4. [ ] Create branch: `refactor/p1-a2-a3-core-split`

**Deliverables:**
- This ADR document
- Dependency graph visualization
- Implementation checklist

### Week 2: Create New Crates (Days 1-3)

**Day 1: riptide-config**
```bash
cargo new --lib crates/riptide-config
```
- [ ] Extract config_builder.rs
- [ ] Extract validation.rs
- [ ] Setup Cargo.toml dependencies
- [ ] Run: `cargo build -p riptide-config`

**Day 2: riptide-engine**
```bash
cargo new --lib crates/riptide-engine
```
- [ ] Move pool.rs from riptide-headless
- [ ] Move instance_pool from riptide-core
- [ ] Add spider-chrome integration prep
- [ ] Run: `cargo build -p riptide-engine`

**Day 3: riptide-cache**
```bash
cargo new --lib crates/riptide-cache
```
- [ ] Extract all cache modules from core
- [ ] Extract domain selection cache from CLI
- [ ] Unified cache interface
- [ ] Run: `cargo build -p riptide-cache`

### Week 2: Update Dependencies (Days 4-5)

**Day 4: Update Imports**
- [ ] Update riptide-core imports (use new crates)
- [ ] Update riptide-api imports
- [ ] Update riptide-cli imports
- [ ] Update riptide-extraction imports
- [ ] Fix all compilation errors

**Day 5: Verification**
- [ ] Run: `cargo build --all`
- [ ] Run: `cargo test --all`
- [ ] Run: `cargo tree | grep riptide` (verify no circular deps)
- [ ] Performance regression tests

---

## Success Criteria

### Code Metrics

| Metric | Before | Target | Success |
|--------|--------|--------|---------|
| **riptide-core size** | 39,604 lines | <15,000 lines | -62% |
| **Core dependencies** | 10+ crates | 4-5 crates | -50%+ |
| **Circular dependencies** | Risk present | 0 | ✅ |
| **Build time (core only)** | ~5s | ~2s | -60% |
| **Test coverage** | ~80% | >85% | +5% |

### Architectural Goals

- [x] **Single Responsibility**: Each crate has one clear purpose
- [x] **Clear Layering**: types → config/engine/cache → core → features
- [x] **No Circular Deps**: Enforced by crate boundaries
- [x] **Spider-Chrome Ready**: riptide-engine enables Phase 1 Week 2 goal
- [ ] **All Tests Pass**: 254/254 tests passing (currently 247/254)

### Integration Checkpoints

1. **Checkpoint 1** (Day 3): All 3 new crates compile independently
2. **Checkpoint 2** (Day 4): riptide-core compiles with new dependencies
3. **Checkpoint 3** (Day 5): All integration tests pass
4. **Checkpoint 4** (Day 5): Performance benchmarks show no regression

---

## Risks & Mitigation

### Risk 1: Breaking Changes in API

**Probability:** Medium
**Impact:** High
**Mitigation:**
- Incremental refactoring (one crate at a time)
- Re-export types from riptide-core for backward compatibility
- Comprehensive test coverage before changes
- Rollback plan: keep branch until full verification

### Risk 2: Performance Regression

**Probability:** Low
**Impact:** Medium
**Mitigation:**
- Benchmark before/after refactoring
- Profile critical paths
- Monitor CI/CD performance metrics
- Accept <5% performance impact for better architecture

### Risk 3: Test Failures

**Probability:** High (expected)
**Impact:** Medium
**Mitigation:**
- Fix tests as they break (expected)
- Run tests frequently during refactoring
- Isolate test failures per crate
- Budget 20% extra time for test fixes

### Risk 4: Merge Conflicts

**Probability:** Medium (active codebase)
**Impact:** Low
**Mitigation:**
- Short-lived branch (5 days max)
- Daily merges from main
- Clear communication with team
- Use atomic commits

---

## Alternatives Considered

### Alternative 1: Keep Monolithic Core

**Pros:**
- No refactoring effort
- No risk of breakage

**Cons:**
- Technical debt continues to grow
- Compilation times increase
- Harder to understand and maintain
- Blocks spider-chrome integration

**Decision:** ❌ Rejected - technical debt must be addressed

### Alternative 2: Extract Only Configuration

**Pros:**
- Smaller scope
- Lower risk
- Faster implementation

**Cons:**
- Doesn't solve browser pool issues
- Misses opportunity for spider-chrome prep
- Partial solution only

**Decision:** ❌ Rejected - insufficient impact

### Alternative 3: Extract All Modules (10+ crates)

**Pros:**
- Maximum separation of concerns
- Smallest possible crates

**Cons:**
- Over-engineering
- Too many dependencies to manage
- Diminishing returns
- Slower compilation (more crate boundaries)

**Decision:** ❌ Rejected - too complex for benefit gained

### Alternative 4: Current Plan (3 New Crates)

**Pros:**
- Balanced approach
- Clear boundaries
- Manageable scope (5 days)
- Enables spider-chrome integration
- Significant debt reduction

**Cons:**
- Requires careful planning
- Some breakage expected

**Decision:** ✅ **ACCEPTED**

---

## References

### Related Documents

- [PHASE1-WEEK2-EXECUTION-PLAN.md](/workspaces/eventmesh/docs/PHASE1-WEEK2-EXECUTION-PLAN.md)
- [COMPREHENSIVE-ROADMAP.md](/workspaces/eventmesh/docs/COMPREHENSIVE-ROADMAP.md)
- [DEAD_CODE_TO_LIVE_CODE_ROADMAP.md](/workspaces/eventmesh/docs/roadmaps/DEAD_CODE_TO_LIVE_CODE_ROADMAP.md)

### Architecture Decisions

- **ADR-001**: Initial crate structure (historical)
- **ADR-002**: Extraction strategy separation
- **ADR-003**: Spider integration approach
- **ADR-004**: Performance optimization strategy
- **ADR-005**: **(This Document)** Core refactoring

### Implementation Tracking

- **Branch:** `refactor/p1-a2-a3-core-split`
- **Jira Epic:** RIPTIDE-123 (if applicable)
- **Swarm Session:** swarm_1760709536951_i98hegexl
- **Memory Key:** `swarm/arch/adr-005`

---

## Appendix A: File Migration Manifest

### riptide-config (1,200 lines)

**From riptide-core:**
- `src/common/config_builder.rs` (537 lines) → `src/builder.rs`
- `src/common/validation.rs` (584 lines) → `src/validation.rs`
- `src/common/error_conversions.rs` (partial) → `src/error.rs`

### riptide-engine (2,500 lines)

**From riptide-headless:**
- `src/pool.rs` (1,324 lines) → `src/pool.rs`
- `src/models.rs` (partial) → `src/models.rs`
- `src/launcher.rs` (partial) → `src/launcher.rs`

**From riptide-core:**
- `src/instance_pool/pool.rs` (1,005 lines) → `src/instance_pool.rs`
- `src/instance_pool/health.rs` (239 lines) → `src/health.rs`
- `src/instance_pool/memory.rs` (78 lines) → `src/memory.rs`
- `src/pool_health.rs` (792 lines) → `src/monitoring.rs`

### riptide-cache (2,200 lines)

**From riptide-core:**
- `src/cache.rs` (381 lines) → `src/core.rs`
- `src/cache_key.rs` (313 lines) → `src/key.rs`
- `src/cache_warming.rs` (881 lines) → `src/warming.rs`
- `src/cache_warming_integration.rs` (278 lines) → `src/integration.rs`
- `src/integrated_cache.rs` (402 lines) → `src/integrated.rs`

**From riptide-cli:**
- `src/cache.rs` (partial) → `src/domain_selection.rs`

---

## Appendix B: Updated Cargo.toml Manifests

### riptide-config/Cargo.toml

```toml
[package]
name = "riptide-config"
version = "0.1.0"
edition = "2021"
license = "Apache-2.0"

[dependencies]
riptide-types = { path = "../riptide-types" }
anyhow = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
regex = { workspace = true }
url = { workspace = true }
```

### riptide-engine/Cargo.toml

```toml
[package]
name = "riptide-engine"
version = "0.1.0"
edition = "2021"
license = "Apache-2.0"

[dependencies]
riptide-types = { path = "../riptide-types" }
riptide-config = { path = "../riptide-config" }
spider_chrome = { workspace = true }
tokio = { workspace = true }
tracing = { workspace = true }
serde = { workspace = true }
anyhow = { workspace = true }
async-trait = { workspace = true }
dashmap = { workspace = true }
sysinfo = { workspace = true }
```

### riptide-cache/Cargo.toml

```toml
[package]
name = "riptide-cache"
version = "0.1.0"
edition = "2021"
license = "Apache-2.0"

[dependencies]
riptide-types = { path = "../riptide-types" }
riptide-config = { path = "../riptide-config" }
redis = { workspace = true }
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
anyhow = { workspace = true }
sha2 = "0.10"
hex = "0.4"
```

---

**Status:** 🚧 IN PROGRESS
**Next Review:** 2025-10-19 (End of Week 2 Day 2)
**Last Updated:** 2025-10-17
