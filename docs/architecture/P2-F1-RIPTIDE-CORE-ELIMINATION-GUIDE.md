# P2-F1: Riptide-Core Elimination - Migration Guide

**Status**: Day 3 Partial Complete
**Timeline**: Days 3-7 (7-day execution plan)
**Objective**: Complete elimination of riptide-core crate through systematic modular architecture

## Executive Summary

### Completed (Day 3)
- ✅ Module analysis and mapping complete
- ✅ wasm_validation moved to riptide-extraction/validation
- ✅ riptide-core updated with re-exports from modular crates
- ✅ Circular dependency issues identified and documented

### Remaining Work (Days 4-7)

#### Day 4-5: Fix Dependencies (PRIORITY)
1. **Fix riptide-reliability circular dependencies**
   - Add feature flags for PoolEvent/PoolOperation usage
   - Create PerformanceMetrics stub in riptide-types
   - Update circuit_breaker.rs lines 283-310 with `#[cfg(feature = "events")]`

2. **Update 11 dependent crates** to remove riptide-core:
   - riptide-api (Cargo.toml line 47)
   - riptide-cli (Cargo.toml line 61)
   - riptide-extraction (dev-dependencies line 53)
   - riptide-intelligence
   - riptide-pdf
   - riptide-performance
   - riptide-persistence
   - riptide-search
   - riptide-streaming
   - riptide-workers
   - riptide-headless (commented out)

#### Day 6: Core Deletion
1. Remove `crates/riptide-core` directory
2. Update root `Cargo.toml` workspace members
3. Full workspace rebuild: `cargo check --workspace`
4. Expected: 0 errors

#### Day 7: Documentation
1. Create `docs/architecture/riptide-core-migration-guide.md`
2. Update `CHANGELOG.md` with breaking changes
3. Final commit

---

## Architecture Analysis

### Module Distribution (Post-Migration)

| Original Location | New Location | Size | Purpose |
|-------------------|--------------|------|---------|
| riptide-core/circuit.rs | riptide-reliability/circuit.rs | 364 lines | Atomic circuit breaker |
| riptide-core/circuit_breaker.rs | riptide-reliability/circuit_breaker.rs | 406 lines | State-based circuit breaker |
| riptide-core/gate.rs | riptide-reliability/gate.rs | 325 lines | Decision gate logic |
| riptide-core/reliability.rs | riptide-reliability/reliability.rs | 542 lines | Reliability orchestration |
| riptide-core/wasm_validation.rs | riptide-extraction/validation/ | 293 lines | WASM component validation |
| riptide-core/component.rs | riptide-types/component.rs | 66 lines | Component metadata |
| riptide-core/conditional.rs | riptide-types/conditional.rs | 364 lines | HTTP conditional requests |
| riptide-core/error.rs | **KEEP IN CORE** | 512 lines | Error types (used by all) |
| riptide-core/types.rs | **KEEP IN CORE** | 66 lines | Re-export from riptide-types |
| riptide-core/common/ | **KEEP IN CORE** | 981 lines | Validation utilities |

**Total Eliminated**: ~3000 lines → Modular crates
**Remaining in Core**: ~1500 lines (error + common + re-exports)

---

## Detailed Migration Steps

### Step 1: Fix riptide-reliability (BLOCKER)

**File**: `crates/riptide-reliability/src/circuit_breaker.rs`

**Problem**: Lines 283-310 use PoolEvent/PoolOperation without feature flag

**Solution**:
```rust
// Lines 283-310: Wrap in feature flag
#[cfg(feature = "events")]
{
    let mut event = PoolEvent::new(
        PoolOperation::CircuitBreakerTripped,
        self.pool_id.clone(),
    );
    event.details.insert("failure_count".to_string(), failure_count.to_string());
    event_bus.publish(event).await;
}
```

**File**: `crates/riptide-reliability/src/circuit_breaker.rs` (lines 13-19)

**Update imports**:
```rust
#[cfg(feature = "events")]
use riptide_pool::PerformanceMetrics;
#[cfg(not(feature = "events"))]
use riptide_types::extracted::PerformanceMetrics; // Fallback

#[cfg(feature = "events")]
use riptide_events::{EventBus, PoolEvent, PoolOperation};
```

**Create stub**: `crates/riptide-types/src/extracted.rs`
```rust
#[derive(Debug, Default)]
pub struct PerformanceMetrics {
    // Stub for non-events builds
}
```

---

### Step 2: Update Dependent Crates (Systematic)

**Pattern for each crate**:

1. **Update Cargo.toml**:
```diff
- riptide-core = { path = "../riptide-core", features = ["api-integration"] }
+ riptide-types = { path = "../riptide-types" }
+ riptide-reliability = { path = "../riptide-reliability" }
+ riptide-extraction = { path = "../riptide-extraction" }
```

2. **Update imports** (use sed for batch):
```bash
# Example for riptide-api
cd crates/riptide-api/src
find . -name "*.rs" -exec sed -i 's/use riptide_core::types::/use riptide_types::/g' {} +
find . -name "*.rs" -exec sed -i 's/use riptide_core::reliability::/use riptide_reliability::/g' {} +
find . -name "*.rs" -exec sed -i 's/use riptide_core::wasm_validation::/use riptide_extraction::validation::/g' {} +
```

3. **Verify**:
```bash
cargo check --package riptide-api
```

4. **Commit**:
```bash
git add crates/riptide-api
git commit -m "refactor(p2-f1): Update riptide-api to use modular crates (Day 4)"
```

---

### Step 3: Batch Update Remaining Crates

**Crate-by-Crate Checklist**:

- [ ] riptide-api
  - Imports: types, reliability, pdf, stealth, events, strategies, spider, fetch, cache_key
  - Est. Time: 15 min

- [ ] riptide-cli
  - Imports: telemetry, monitoring
  - Est. Time: 10 min

- [ ] riptide-extraction (dev-dependencies only)
  - Remove: dev-dependencies line 53
  - Est. Time: 5 min

- [ ] riptide-intelligence
  - Imports: types, extraction
  - Est. Time: 10 min

- [ ] riptide-pdf
  - Imports: types
  - Est. Time: 5 min

- [ ] riptide-performance
  - Imports: types, reliability
  - Est. Time: 10 min

- [ ] riptide-persistence
  - Imports: types, cache
  - Est. Time: 10 min

- [ ] riptide-search
  - Imports: types
  - Est. Time: 5 min

- [ ] riptide-streaming
  - Imports: types, events
  - Est. Time: 10 min

- [ ] riptide-workers
  - Imports: types, reliability
  - Est. Time: 10 min

**Total Estimated Time**: 90 minutes (1.5 hours)

---

### Step 4: Delete riptide-core

**Prerequisites**:
- All 11 dependent crates updated
- `cargo check --workspace` passes
- 0 references to riptide-core in any Cargo.toml

**Commands**:
```bash
# 1. Remove from workspace
sed -i '/riptide-core/d' Cargo.toml

# 2. Delete directory
rm -rf crates/riptide-core

# 3. Verify
cargo check --workspace

# 4. Commit
git add -A
git commit -m "feat(p2-f1): Remove riptide-core crate (Day 6) ✅

BREAKING CHANGE: riptide-core eliminated. Use modular crates:
- riptide-types (shared types)
- riptide-reliability (circuit breakers, gates)
- riptide-extraction (validation)

Migration guide: docs/architecture/riptide-core-migration-guide.md"
```

---

## Breaking Changes

### For External Users

**Before** (with riptide-core):
```rust
use riptide_core::types::ExtractedDoc;
use riptide_core::reliability::ReliableExtractor;
use riptide_core::wasm_validation::WitValidator;
```

**After** (modular):
```rust
use riptide_types::ExtractedDoc;
use riptide_reliability::ReliableExtractor;
use riptide_extraction::validation::WitValidator;
```

### Cargo.toml Changes

**Before**:
```toml
[dependencies]
riptide-core = { version = "0.1.0", features = ["api-integration"] }
```

**After**:
```toml
[dependencies]
riptide-types = "0.1.0"
riptide-reliability = "0.1.0"
riptide-extraction = "0.1.0"
```

---

## Testing Strategy

### Verification Checklist

- [ ] **Build**: `cargo check --workspace` (0 errors)
- [ ] **Tests**: `cargo test --workspace` (all pass)
- [ ] **Lints**: `cargo clippy --workspace` (0 warnings)
- [ ] **Docs**: `cargo doc --workspace --no-deps` (builds)
- [ ] **Dependencies**: `cargo tree --duplicates` (0 duplicates)
- [ ] **Size**: `du -sh crates/` (reduced by ~3000 lines)

### Integration Tests

Test these workflows post-deletion:
1. API server starts: `cargo run --bin riptide-api`
2. CLI works: `cargo run --bin riptide -- --help`
3. Extraction pipeline: Test riptide-extraction crate
4. Reliability patterns: Test circuit breakers

---

## Rollback Plan

If deletion fails:

1. **Revert commit**:
```bash
git revert HEAD
```

2. **Restore riptide-core**:
```bash
git checkout main -- crates/riptide-core
```

3. **Fix issues** and retry

---

## Performance Impact

**Expected Improvements**:
- ✅ Faster builds (fewer circular dependencies)
- ✅ Clearer dependency tree
- ✅ Easier to maintain modular crates
- ✅ Better parallelization in cargo builds

**Build Time Comparison** (estimated):
- Before: ~180s (full workspace build)
- After: ~120s (40% improvement with modular structure)

---

## Risks & Mitigations

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| Circular dependencies | Medium | High | Feature flags + careful import ordering |
| Breaking external users | Low | High | Migration guide + deprecation warnings |
| Build failures | Medium | Medium | Incremental updates + thorough testing |
| Type conflicts | Low | Medium | Use re-exports in riptide-types |

---

## Timeline

| Day | Tasks | Est. Time | Status |
|-----|-------|-----------|--------|
| 3 | Module analysis, wasm_validation move | 1h | ✅ Complete |
| 4-5 | Fix riptide-reliability, update 11 crates | 3h | ⏳ Pending |
| 6 | Delete riptide-core, workspace rebuild | 1h | ⏳ Pending |
| 7 | Documentation, CHANGELOG | 1h | ⏳ Pending |

**Total**: 6 hours over 5 days

---

## Success Criteria

- [ ] riptide-core directory deleted
- [ ] 0 compilation errors in workspace
- [ ] All tests pass
- [ ] Migration guide complete
- [ ] CHANGELOG updated
- [ ] Commit history clean and atomic
- [ ] Documentation updated

---

## References

- [P2-F1 Roadmap](/workspaces/eventmesh/docs/COMPREHENSIVE-ROADMAP.md)
- [Architecture Analysis](/workspaces/eventmesh/docs/architecture/riptide-core-architecture-analysis.md)
- [Day 1-2 Progress](/workspaces/eventmesh/docs/ROADMAP-CURRENT-STATUS.md)

---

**Document Version**: 1.0
**Last Updated**: 2025-10-19
**Author**: System Architect Agent (Claude Code)
