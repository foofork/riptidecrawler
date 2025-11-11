# Circular Dependency Resolution

## Status: RESOLVED (Test-Only Dependency Accepted)

**Date**: 2025-11-11
**Phase**: 2 - AppState Elimination & Hexagonal Architecture

---

## Overview

This document describes the resolution of circular dependencies between `riptide-api` and `riptide-facade` during the transition from AppState to ApplicationContext.

---

## Production Dependencies

### ✅ CLEAN - No Circular Dependencies

**Production dependency graph:**
```
riptide-api → riptide-facade (one-way)
```

- `riptide-api` depends on `riptide-facade` for hexagonal architecture facades
- `riptide-facade` does NOT depend on `riptide-api` in production builds
- All production code follows clean dependency hierarchy

### Verification Command
```bash
# Production dependencies: Should show NO riptide-api
cargo tree -p riptide-facade --no-dev-dependencies | grep riptide-api
```

**Expected result**: No output (no circular dependency)

---

## Test Dependencies

### ⚠️ ACCEPTED - Test-Only Circular Dependency

**Test dependency graph:**
```
riptide-facade (dev-dependencies) → riptide-api (test utilities)
```

- `riptide-facade` includes `riptide-api` in `dev-dependencies` for test utilities
- This is acceptable because:
  - Test code does not ship to production
  - Test utilities provide mock ApplicationContext for facade testing
  - Common pattern in Rust for test helper dependencies

### Test Utilities Location
```
/crates/riptide-facade/tests/common/mod.rs
```

Contains:
- Mock ApplicationContext builders
- Test fixtures for facades
- Integration test helpers

### Verification Command
```bash
# Including dev-dependencies: Should show ONE test dependency
cargo tree -p riptide-facade | grep riptide-api
```

**Expected result**: One line showing dev-dependency on riptide-api

---

## Decision Rationale

### Why This Is Acceptable

1. **Production Isolation**: Test dependencies do not affect production binaries
2. **Rust Guarantees**: `dev-dependencies` are only compiled for tests, benches, and examples
3. **Industry Standard**: Common pattern in Rust ecosystem for test utilities
4. **No Runtime Impact**: Zero impact on deployment artifacts

### Examples from Rust Ecosystem
- `tokio` test utilities in `dev-dependencies` of higher-level crates
- `serde` test helpers used by format-specific crates
- `axum` test utilities used by middleware crates

---

## Future Improvement (Optional)

If complete elimination of test-only circular dependency is desired:

### Option: Extract Test Utilities Crate

**New structure:**
```
riptide-test-utils (new crate)
  ├── Mock ApplicationContext builders
  ├── Test fixtures
  └── Integration test helpers

riptide-api
  └── dev-dependencies: riptide-test-utils

riptide-facade
  └── dev-dependencies: riptide-test-utils
```

**Effort estimate**: 2-3 hours
**Priority**: Low (non-blocking for production deployment)

---

## Validation Results

### Production Build
```bash
cargo build --release -p riptide-api
# ✅ Clean build, no circular dependencies
```

### Test Build
```bash
cargo test -p riptide-facade
# ✅ All tests pass with test utilities
```

### Dependency Analysis
```bash
cargo tree -p riptide-facade --no-dev-dependencies | grep riptide-api
# ✅ No results (clean production dependencies)

cargo tree -p riptide-facade | grep riptide-api
# ✅ One dev-dependency line (test utilities only)
```

---

## Conclusion

**Status**: ✅ **ACCEPTED AND NON-BLOCKING**

The test-only circular dependency between `riptide-facade` and `riptide-api` is:
- Isolated to test code only
- Does not affect production deployments
- Follows common Rust ecosystem patterns
- Properly verified and documented

**Production deployment**: **CLEARED FOR RELEASE**

---

## References

- Phase 2 Implementation: [Phase 2 Tasks](../phase2/)
- ApplicationContext Design: [ADR-001](./ADR-001-appstate-elimination.md)
- Hexagonal Architecture: [Architecture Overview](./README.md)
