# Circuit Breaker Circular Dependency Refactoring Plan

## Executive Summary

**Problem**: Circular dependency blocking all builds:
```
extraction → spider → reliability → pool → extraction (CYCLE)
```

**Root Cause**: `riptide-fetch` and `riptide-spider` recently added `riptide-reliability` dependency solely for `CircuitBreaker`, but `riptide-reliability` transitively depends back on `riptide-extraction` via the `events` feature (enabled by default).

**Solution**: Move `CircuitBreaker` to `riptide-types` - a dependency-free foundation crate.

---

## Problem Analysis

### Current Dependency Chain

```
riptide-extraction v0.9.0
  └─> riptide-spider v0.9.0
       └─> riptide-fetch v0.9.0
            └─> riptide-reliability v0.9.0 [default features = events]
                 └─> riptide-pool v0.9.0 [via events feature]
                      └─> riptide-extraction v0.9.0 [native-pool feature]
                           └─> ⚠️ CIRCULAR DEPENDENCY
```

### Feature Flag Analysis

**riptide-reliability/Cargo.toml (lines 44-55):**
```toml
[features]
default = ["events", "monitoring"]
events = ["riptide-events", "riptide-pool"]  # ⚠️ This pulls in riptide-pool
monitoring = ["riptide-monitoring"]
# reliability-patterns = DISABLED due to circular dependency
```

The `events` feature is **enabled by default**, which pulls in `riptide-pool`, which depends on `riptide-extraction` (via `native-pool` feature), completing the cycle.

### Current Circuit Breaker Usage

**Only 2 files import CircuitBreaker:**

1. **riptide-fetch/src/fetch.rs:3**
   ```rust
   use riptide_reliability::circuit::{self, CircuitBreaker, Config as CircuitConfig};
   ```
   - Creates circuit breaker for HTTP client reliability
   - Uses `guarded_call` helper function
   - **Zero dependencies** on other `riptide-reliability` modules

2. **riptide-spider/src/core.rs:1**
   ```rust
   use riptide_reliability::circuit::CircuitBreaker;
   ```
   - Optional circuit breaker integration (line 81: `circuit_breaker: Option<Arc<CircuitBreaker>>`)
   - **Zero dependencies** on other `riptide-reliability` modules

### Circuit Breaker Implementation Analysis

**File**: `riptide-reliability/src/circuit.rs` (365 lines)

**Dependencies:**
- `std::sync::atomic` - standard library
- `std::sync::Arc` - standard library
- `tokio::sync::Semaphore` - already in riptide-types workspace deps
- `anyhow` - already in riptide-types deps
- `tracing` - already in riptide-types workspace deps

**Exports:**
- `State` enum (Closed, Open, HalfOpen)
- `Config` struct (failure_threshold, open_cooldown_ms, half_open_max_in_flight)
- `Clock` trait + `RealClock` implementation
- `CircuitBreaker` struct
- `guarded_call` async helper function

**NO dependencies on:**
- ✅ Events system
- ✅ Pool system
- ✅ Monitoring system
- ✅ Any other riptide crates

**Verdict**: Circuit breaker is **100% self-contained** and can be moved without modification.

---

## Recommended Solution: Move to riptide-types

### Why riptide-types?

1. **Zero dependencies** (only std and workspace deps like tokio, serde, anyhow)
2. **Foundation crate** - already depended on by all crates
3. **Breaking the cycle** - doesn't depend on any riptide-* crates
4. **Semantic fit** - circuit breaker is a fundamental reliability primitive

### Advantages

✅ **No new crate** - avoids proliferation of micro-crates
✅ **Zero disruption** - all crates already depend on riptide-types
✅ **Clean separation** - types crate is dependency-free
✅ **Future-proof** - other reliability primitives can live here
✅ **Minimal migration** - only 2 import statements to update

### Alternative Rejected: New riptide-circuit Crate

❌ Adds dependency management overhead
❌ Creates another crate to maintain
❌ Unnecessary for single module
❌ Violates "minimal viable crates" principle

---

## Migration Plan

### Phase 1: Move Circuit Breaker Module (10 minutes)

**Step 1.1**: Copy circuit.rs to riptide-types
```bash
# Create reliability subdirectory
mkdir -p /workspaces/eventmesh/crates/riptide-types/src/reliability

# Copy circuit breaker
cp /workspaces/eventmesh/crates/riptide-reliability/src/circuit.rs \
   /workspaces/eventmesh/crates/riptide-types/src/reliability/circuit.rs
```

**Step 1.2**: Update riptide-types/src/lib.rs
```rust
// Add new module
pub mod reliability;

// Re-export circuit breaker types
pub use reliability::circuit::{
    guarded_call, CircuitBreaker, Clock, Config as CircuitConfig, RealClock, State as CircuitState,
};
```

**Step 1.3**: Verify riptide-types builds
```bash
cd /workspaces/eventmesh/crates/riptide-types
cargo build
cargo test
```

### Phase 2: Update Consumers (5 minutes)

**Step 2.1**: Update riptide-fetch/src/fetch.rs
```rust
# Change line 3:
- use riptide_reliability::circuit::{self, CircuitBreaker, Config as CircuitConfig};
+ use riptide_types::reliability::circuit::{self, CircuitBreaker, Config as CircuitConfig};

# Also update line 30 (re-export):
- pub use riptide_reliability::circuit::State as CircuitState;
+ pub use riptide_types::reliability::circuit::State as CircuitState;
```

**Step 2.2**: Update riptide-spider/src/core.rs
```rust
# Change line 1:
- use riptide_reliability::circuit::CircuitBreaker;
+ use riptide_types::reliability::circuit::CircuitBreaker;
```

**Step 2.3**: Update riptide-fetch/Cargo.toml
```toml
# Remove riptide-reliability dependency (line 11):
- riptide-reliability = { path = "../riptide-reliability" }
```

**Step 2.4**: Update riptide-spider/Cargo.toml
```toml
# Remove riptide-reliability dependency (line 12):
- riptide-reliability = { path = "../riptide-reliability" }
```

### Phase 3: Clean Up riptide-reliability (5 minutes)

**Step 3.1**: Keep circuit.rs for backward compatibility (deprecated)
```rust
// In riptide-reliability/src/circuit.rs - add deprecation notice
#[deprecated(
    since = "0.10.0",
    note = "CircuitBreaker has moved to riptide-types::reliability::circuit. This re-export will be removed in 1.0.0"
)]
pub use riptide_types::reliability::circuit::*;
```

**Step 3.2**: Update riptide-reliability/src/lib.rs
```rust
// Update re-exports to point to riptide-types
pub use riptide_types::reliability::circuit::{
    CircuitBreaker as AtomicCircuitBreaker, Clock, Config as CircuitConfig, State,
};
```

**Step 3.3**: Update riptide-reliability documentation
```rust
// Update lib.rs documentation (lines 19-42) to mention new location
//! ### 1. Atomic Circuit Breaker (`circuit`)
//!
//! **NOTE**: Circuit breaker has moved to `riptide-types::reliability::circuit`.
//! This re-export is provided for backward compatibility.
```

### Phase 4: Verification (5 minutes)

**Step 4.1**: Build all crates
```bash
cargo build --workspace
```

**Step 4.2**: Run all tests
```bash
cargo test --workspace
```

**Step 4.3**: Check for circular dependency
```bash
cargo tree -p riptide-fetch | grep -E "(riptide-reliability|riptide-pool|riptide-extraction)"
cargo tree -p riptide-spider | grep -E "(riptide-reliability|riptide-pool|riptide-extraction)"
```

**Step 4.4**: Verify existing consumers still work
```bash
# Test files that use circuit breaker
cargo test -p riptide-fetch --test '*'
cargo test -p riptide-spider --test '*'
cargo test -p riptide-reliability --test circuit_breaker_tests
```

---

## Files to Update

### New Files (1)
- [ ] `/workspaces/eventmesh/crates/riptide-types/src/reliability/circuit.rs` (copy from reliability)
- [ ] `/workspaces/eventmesh/crates/riptide-types/src/reliability/mod.rs` (new module file)

### Modified Files (7)

**riptide-types (2 files):**
- [ ] `/workspaces/eventmesh/crates/riptide-types/src/lib.rs` - Add module and re-exports
- [ ] `/workspaces/eventmesh/crates/riptide-types/src/reliability/mod.rs` - Module entry point

**riptide-fetch (2 files):**
- [ ] `/workspaces/eventmesh/crates/riptide-fetch/Cargo.toml` - Remove riptide-reliability dependency
- [ ] `/workspaces/eventmesh/crates/riptide-fetch/src/fetch.rs` - Update imports (lines 3, 30)

**riptide-spider (2 files):**
- [ ] `/workspaces/eventmesh/crates/riptide-spider/Cargo.toml` - Remove riptide-reliability dependency
- [ ] `/workspaces/eventmesh/crates/riptide-spider/src/core.rs` - Update import (line 1)

**riptide-reliability (1 file):**
- [ ] `/workspaces/eventmesh/crates/riptide-reliability/src/lib.rs` - Update re-exports and docs

---

## Alignment with P1 Roadmap

**P1 Item**: "Circuit Breaker Pattern - 78 Implementations"

This refactoring **enables** the circuit breaker rollout by:

1. ✅ **Unblocks builds** - Removes circular dependency
2. ✅ **Centralizes implementation** - Single source in riptide-types
3. ✅ **Improves discoverability** - Circuit breaker in foundation types crate
4. ✅ **Enables wider adoption** - All crates can now safely use circuit breaker
5. ✅ **Maintains compatibility** - Deprecated re-exports for gradual migration

### Future Circuit Breaker Expansion

After this refactoring, circuit breakers can be added to:
- HTTP client calls (already in riptide-fetch)
- Database operations
- External API calls
- WASM execution
- Headless browser operations
- Cache operations

All without circular dependency concerns.

---

## Risk Assessment

### Low Risk
- Circuit breaker module is **self-contained** (no internal deps)
- Only **2 files** import it
- **All dependencies** already in riptide-types
- **100% test coverage** exists
- **Backward compatibility** maintained via re-exports

### Mitigation
1. Keep deprecated re-exports in riptide-reliability (6 month sunset)
2. Add comprehensive tests to riptide-types
3. Update all documentation references
4. Run full test suite before/after

---

## Timeline

| Phase | Duration | Tasks |
|-------|----------|-------|
| Phase 1: Move module | 10 min | Copy circuit.rs, update lib.rs, verify build |
| Phase 2: Update consumers | 5 min | Update 2 imports, remove 2 deps |
| Phase 3: Clean up | 5 min | Add deprecations, update docs |
| Phase 4: Verification | 5 min | Build, test, verify no cycles |
| **Total** | **25 min** | **4 phases, 8 files** |

---

## Success Criteria

- [ ] `cargo build --workspace` succeeds
- [ ] `cargo test --workspace` passes
- [ ] No circular dependencies in `cargo tree`
- [ ] riptide-fetch builds without riptide-reliability
- [ ] riptide-spider builds without riptide-reliability
- [ ] All circuit breaker tests pass
- [ ] Documentation updated
- [ ] Backward compatibility maintained

---

## Post-Migration Cleanup (Future)

After 6 months (v1.0.0):
1. Remove deprecated circuit module from riptide-reliability
2. Update all internal references to use riptide-types directly
3. Remove backward compatibility re-exports
4. Update all documentation and examples

---

## Conclusion

Moving `CircuitBreaker` to `riptide-types` is the **optimal solution** because:

1. ✅ **Breaks the cycle** - riptide-types has zero riptide-* dependencies
2. ✅ **Minimal changes** - Only 2 import statements + 2 Cargo.toml entries
3. ✅ **Self-contained** - Circuit breaker has no external dependencies
4. ✅ **Future-proof** - Enables P1 roadmap item (78 implementations)
5. ✅ **Low risk** - Full test coverage + backward compatibility
6. ✅ **Fast execution** - 25 minutes total migration time

This refactoring **unblocks all builds immediately** while setting up a solid foundation for circuit breaker proliferation across the codebase.
