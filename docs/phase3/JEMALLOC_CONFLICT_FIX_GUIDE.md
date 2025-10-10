# Jemalloc Dependency Conflict - Fix Guide

**Date:** 2025-10-10
**Priority:** P0 - CRITICAL
**Status:** 🔴 **BLOCKING ALL TESTING**

---

## Problem Summary

The RipTide EventMesh project cannot be built or tested due to conflicting jemalloc library linkage:

- `riptide-api` depends on `tikv-jemallocator v0.5`
- `riptide-performance` depends on `jemalloc-ctl v0.5`
- Both link to the native `jemalloc` library, which Cargo prohibits

This blocks **ALL** integration testing, load testing, and production deployment.

---

## Recommended Solution: Option 1 (Preferred)

### Use tikv-jemallocator Ecosystem Exclusively

**Rationale:**
- `tikv-jemallocator` provides both the allocator AND control interface
- No conflicts, unified API
- Well-maintained by TiKV project
- Industry standard for Rust

### Implementation Steps

#### Step 1: Update riptide-performance/Cargo.toml

```toml
# REMOVE these lines:
# jemalloc-ctl = { version = "0.5", optional = true }

# ADD these lines:
tikv-jemalloc-ctl = { version = "0.5", optional = true, package = "jemalloc-ctl" }

# UPDATE feature definitions:
[features]
memory-profiling = ["tikv-jemalloc-ctl", "pprof", "memory-stats"]
jemalloc = ["tikv-jemalloc-ctl"]
```

#### Step 2: Update riptide-performance source code

Find all imports of `jemalloc_ctl` and update:

```rust
// OLD:
use jemalloc_ctl::{stats, epoch};

// NEW:
use tikv_jemalloc_ctl::{stats, epoch};
```

Files to update:
- `crates/riptide-performance/src/memory_profiler.rs` (if exists)
- `crates/riptide-performance/src/profiling/*.rs`
- Any file using jemalloc control interface

#### Step 3: Verify riptide-api Cargo.toml

Ensure it's using tikv-jemallocator correctly:

```toml
[dependencies]
riptide-performance = { path = "../riptide-performance" }

[target.'cfg(not(target_env = "msvc"))'.dependencies]
tikv-jemallocator = { version = "0.5", optional = true }

[features]
jemalloc = ["riptide-performance/jemalloc", "tikv-jemallocator"]
```

#### Step 4: Test the fix

```bash
# Clean build artifacts
cargo clean

# Build without jemalloc features
cargo build --workspace

# Build with jemalloc features
cargo build --workspace --features jemalloc

# Run tests
cargo test --workspace --no-fail-fast
```

### Expected Outcome

✅ Build succeeds
✅ Tests run
✅ Jemalloc memory profiling works
✅ No dependency conflicts

---

## Alternative Solution: Option 2

### Make Jemalloc Features Mutually Exclusive

**Rationale:**
- Keep current dependencies
- Use Cargo feature resolution
- More complex but preserves existing code

### Implementation Steps

#### Step 1: Update workspace Cargo.toml

Add resolver configuration:

```toml
[workspace]
resolver = "2"

[workspace.dependencies]
# Ensure only one jemalloc implementation
jemalloc-sys = "=0.5.4+5.3.0-patched"
tikv-jemalloc-sys = "=0.5.4+5.3.0-patched.1"
```

#### Step 2: Update riptide-api Cargo.toml

```toml
[dependencies]
# Remove jemalloc feature from default activation
riptide-performance = { path = "../riptide-performance", default-features = false }

[features]
# Only enable ONE jemalloc implementation
jemalloc-api = ["tikv-jemallocator"]
jemalloc-perf = ["riptide-performance/jemalloc"]

# Never enable both at once
default = []  # No jemalloc by default
```

#### Step 3: Document feature usage

```markdown
# Building with jemalloc allocator (for API)
cargo build --features jemalloc-api

# Building with jemalloc profiling (for performance)
cargo build --features jemalloc-perf

# NEVER do this (will fail):
cargo build --features jemalloc-api,jemalloc-perf
```

### Drawbacks
- More complex feature management
- Easy to misuse
- Documentation burden
- CI/CD complexity

---

## Alternative Solution: Option 3

### Consolidate All Jemalloc in One Crate

**Rationale:**
- Single source of truth
- Clearest dependency tree
- Simplest to understand

### Implementation Steps

#### Step 1: Remove jemalloc from riptide-api

```toml
# riptide-api/Cargo.toml

[dependencies]
riptide-performance = { path = "../riptide-performance" }
# Remove tikv-jemallocator entirely

[features]
# Remove jemalloc feature
# Remove all jemalloc references
```

#### Step 2: Consolidate in riptide-performance

```toml
# riptide-performance/Cargo.toml

[dependencies]
tikv-jemallocator = { version = "0.5", optional = true }
tikv-jemalloc-ctl = { version = "0.5", optional = true }

[features]
jemalloc = ["tikv-jemallocator", "tikv-jemalloc-ctl"]
```

#### Step 3: Set global allocator in riptide-api main.rs

```rust
// riptide-api/src/main.rs

#[cfg(all(feature = "jemalloc", not(target_env = "msvc")))]
use riptide_performance::GLOBAL_ALLOCATOR;

#[cfg(all(feature = "jemalloc", not(target_env = "msvc")))]
#[global_allocator]
static GLOBAL: &riptide_performance::GLOBAL_ALLOCATOR = &riptide_performance::GLOBAL_ALLOCATOR;
```

#### Step 4: Export allocator from riptide-performance

```rust
// riptide-performance/src/lib.rs

#[cfg(feature = "jemalloc")]
#[global_allocator]
pub static GLOBAL_ALLOCATOR: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;
```

### Drawbacks
- Moves allocator setup to library
- Unusual pattern (allocator in library vs binary)
- May confuse other consumers of riptide-performance

---

## Comparison Matrix

| Criterion | Option 1 (tikv) | Option 2 (features) | Option 3 (consolidate) |
|-----------|-----------------|---------------------|------------------------|
| **Complexity** | Low | High | Medium |
| **Maintainability** | High | Low | Medium |
| **CI/CD Impact** | None | High | Low |
| **Code Changes** | Medium | Low | High |
| **Risk** | Low | Medium | Medium |
| **Industry Standard** | Yes | No | No |
| **Recommended** | ✅ **YES** | ❌ No | 🟡 Maybe |

---

## Recommended Action Plan

### Phase 1: Immediate Fix (Option 1)
**Timeline:** 1-2 hours
**Owner:** Build/Infrastructure Team

1. Update `riptide-performance/Cargo.toml`
   - Replace `jemalloc-ctl` with `tikv-jemalloc-ctl`
   - Update features

2. Update source code imports
   - Search and replace `jemalloc_ctl` with `tikv_jemalloc_ctl`

3. Test build
   - `cargo clean && cargo build --workspace`

4. Commit fix
   ```bash
   git add -A
   git commit -m "fix: resolve jemalloc dependency conflict using tikv-jemalloc-ctl"
   ```

### Phase 2: Verification (Test Suite)
**Timeline:** 2-4 hours
**Owner:** QA Team

1. Run test baseline
   ```bash
   cargo test --workspace --no-fail-fast > test_results.log 2>&1
   ```

2. Analyze results
   - Document passing tests
   - Triage failing tests
   - Create issues for failures

3. Verify jemalloc features
   ```bash
   cargo test --workspace --features jemalloc
   ```

### Phase 3: Integration Testing
**Timeline:** 2-3 days
**Owner:** QA + Development Teams

1. Implement sprint integration tests
2. Run load tests
3. Setup soak test
4. Document results

---

## Validation Checklist

### Build System
- [ ] `cargo clean` completes
- [ ] `cargo build --workspace` succeeds
- [ ] `cargo build --workspace --release` succeeds
- [ ] `cargo build --features jemalloc` succeeds
- [ ] No jemalloc-sys conflict errors

### Test Execution
- [ ] `cargo test --workspace` runs
- [ ] Tests complete within reasonable time (<10 min)
- [ ] Test results are reportable
- [ ] No test hangs or crashes

### Feature Verification
- [ ] Jemalloc allocator activates when enabled
- [ ] Memory profiling works
- [ ] Performance metrics accurate
- [ ] No runtime errors with jemalloc

### Integration
- [ ] All crates build together
- [ ] No circular dependencies
- [ ] Feature flags work correctly
- [ ] Documentation updated

---

## Rollback Plan

If Option 1 fails:

```bash
# Revert changes
git reset --hard HEAD~1

# Try Option 3 instead
# Follow Option 3 implementation steps
```

If all options fail:

```bash
# Temporary workaround: Disable jemalloc entirely
# Remove from both Cargo.toml files
# Document as tech debt
# Create issue for proper fix
```

---

## Testing After Fix

### Quick Smoke Test
```bash
# Should complete in <5 minutes
cargo test -p riptide-api --lib test_health_endpoint
cargo test -p riptide-performance --lib
```

### Full Test Suite
```bash
# Should complete in <15 minutes
cargo test --workspace --no-fail-fast | tee test_results.log

# Analyze results
grep -E "test result:" test_results.log
```

### With Jemalloc Enabled
```bash
# Verify jemalloc features work
cargo test --workspace --features jemalloc --no-fail-fast
```

---

## Documentation Updates Required

After fix is implemented:

1. Update `/docs/phase3/INTEGRATION_TEST_COMPREHENSIVE_REPORT.md`
   - Change status from BLOCKED to IN_PROGRESS
   - Update build system status to ✅

2. Update `/README.md`
   - Document jemalloc feature usage
   - Update build instructions

3. Create `/docs/technical/JEMALLOC_SETUP.md`
   - Document allocator configuration
   - Explain feature flags
   - Provide troubleshooting guide

4. Update CI/CD configuration
   - Add jemalloc feature tests
   - Verify no regressions

---

## Success Criteria

### Must Have
- ✅ Project builds without errors
- ✅ Tests can execute
- ✅ No jemalloc conflicts

### Should Have
- ✅ All existing tests still pass
- ✅ Jemalloc features work correctly
- ✅ Documentation updated

### Nice to Have
- ✅ Performance improved or unchanged
- ✅ Memory profiling enhanced
- ✅ CI/CD pipelines green

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Fix breaks existing code | Low | High | Comprehensive testing |
| Performance regression | Low | Medium | Benchmark before/after |
| New dependency issues | Low | High | Lock file verification |
| Test failures | Medium | High | Baseline documentation |

---

## Support and Escalation

**Primary Contact:** Build/Infrastructure Team
**Escalation Path:** Tech Lead → Engineering Manager → CTO
**Timeframe:** Fix must be completed within 24 hours (P0)

**Communication Plan:**
- Status updates every 2 hours
- Immediate notification if blocked
- Post-fix verification report

---

**Document Version:** 1.0
**Last Updated:** 2025-10-10
**Status:** ACTIVE - REQUIRES IMMEDIATE ACTION
**Priority:** P0 - CRITICAL BLOCKER

---

## Quick Reference Commands

```bash
# Option 1 Implementation
cd /workspaces/eventmesh
# Edit crates/riptide-performance/Cargo.toml (see Step 1)
# Update source files (see Step 2)
cargo clean
cargo build --workspace
cargo test --workspace

# Verification
cargo build --workspace --release
cargo test --workspace --features jemalloc
cargo clippy --workspace

# Documentation
git add -A
git commit -m "fix: resolve jemalloc dependency conflict"
git push origin main
```

---

**END OF FIX GUIDE**
