# Circular Dependency - Quick Decision Summary

**Date**: 2025-11-01
**Status**: 🔴 CRITICAL - Blocks all builds
**Recommendation**: ✅ Option A - Move to riptide-types

---

## The Problem

```
extraction → spider → fetch → reliability → pool → extraction
                                    ⬆️____________⬇️
                                        CYCLE!
```

**Root Cause**: Recent consolidation (commits 18c6e9c, 37fbdbf) added `riptide-reliability` dependency to `fetch` and `spider`, but reliability's default `events` feature pulls in `pool`, which depends on `extraction`.

---

## The Options

### ✅ Option A: Move CircuitBreaker to riptide-types
- **Time**: 25 minutes
- **Risk**: LOW
- **Pros**: Fast, no new crates, maintains consolidation, complete plan exists
- **Cons**: Minor semantic mismatch ("types" has behavior)
- **Recommendation**: ✅ **DO THIS**

### ⚠️ Option B: Create riptide-circuit crate
- **Time**: 90 minutes
- **Risk**: LOW-MEDIUM
- **Pros**: Perfect separation, semantic fit
- **Cons**: Crate proliferation, overkill for 365 LOC, more maintenance
- **Recommendation**: ⚠️ Overkill

### ❌ Option C: Revert consolidation (restore local copies)
- **Time**: 10 minutes
- **Risk**: NONE
- **Pros**: Immediate fix
- **Cons**: Reintroduces 1,092 LOC duplication, defeats architectural goals
- **Recommendation**: ❌ Regression

### ❌ Option D: Make native-pool non-default
- **Time**: 5 minutes
- **Risk**: MEDIUM
- **Pros**: Minimal change
- **Cons**: Breaking change, fragile, doesn't solve root cause
- **Recommendation**: ❌ Band-aid

---

## Why Option A?

1. **Circuit breaker is self-contained** - Zero riptide-* dependencies
2. **Only 2 files use it** - Minimal migration surface
3. **riptide-types is universal** - All crates already depend on it
4. **Complete plan ready** - 4-phase, 25-minute execution
5. **Maintains consolidation** - Keeps single source of truth
6. **No breaking changes** - Deprecated re-exports for backward compat
7. **Enables P1 goals** - Unblocks circuit breaker rollout (78 locations)

---

## Quick Facts

**CircuitBreaker Analysis:**
- **Location**: `/crates/riptide-reliability/src/circuit.rs`
- **Size**: 365 lines
- **Dependencies**: Only std + tokio (already in types)
- **Consumers**: Only `fetch/src/fetch.rs` and `spider/src/core.rs`
- **Features Used**: NONE - only circuit breaker module

**Recent History:**
- **Before**: 3 copies of circuit.rs (1,092 LOC duplicated)
- **Oct 31-Nov 1**: Consolidated to reliability crate
- **Issue**: Didn't account for circular dependency via events feature
- **Solution**: Move to dependency-free foundation crate

---

## Implementation (Option A)

### Files to Change (8 total)

**Create (2):**
- `/crates/riptide-types/src/reliability/circuit.rs` (copy from reliability)
- `/crates/riptide-types/src/reliability/mod.rs` (new)

**Modify (6):**
- `/crates/riptide-types/src/lib.rs` - Add module
- `/crates/riptide-fetch/Cargo.toml` - Remove reliability dep
- `/crates/riptide-fetch/src/fetch.rs` - Update import
- `/crates/riptide-spider/Cargo.toml` - Remove reliability dep
- `/crates/riptide-spider/src/core.rs` - Update import
- `/crates/riptide-reliability/src/lib.rs` - Add deprecated re-export

### Verification Commands

```bash
# Build all
cargo build --workspace

# Test all
cargo test --workspace

# Verify no cycle
cargo tree -p riptide-fetch | grep riptide-reliability  # Should be empty
cargo tree -p riptide-spider | grep riptide-reliability  # Should be empty

# Check for cycles
cargo tree -p riptide-extraction --depth 5 | grep -i cycle  # Should be empty
```

---

## Success Criteria

- [ ] ✅ `cargo build --workspace` succeeds
- [ ] ✅ `cargo test --workspace` passes
- [ ] ✅ No circular dependencies in `cargo tree`
- [ ] ✅ fetch and spider build without reliability dependency
- [ ] ✅ All circuit breaker functionality unchanged
- [ ] ✅ Backward compatibility maintained

---

## Decision

**✅ PROCEED WITH OPTION A**

**Rationale**: Fastest proper solution (25 min), maintains consolidation goals, zero risk, complete plan ready.

**Alternative Rejected**: Option C (revert) is faster but creates technical debt. Option B is over-engineered. Option D is fragile.

---

## Reference Documents

- **Full Research**: `/docs/architecture/circular_dependency_research.md` (detailed analysis)
- **Implementation Plan**: `/docs/architecture/CIRCUIT_BREAKER_REFACTORING_PLAN.md` (step-by-step)
- **Consolidation Analysis**: `/docs/architecture/circuit_breaker_consolidation.md` (background)

---

**Next Action**: Execute Option A migration (25 minutes)
