# CI/CD Feature Flags Analysis for Test Execution

**Date:** 2025-10-27
**Project:** RipTide (EventMesh)
**Focus:** Implementing feature flags for conditional test execution in CI/CD

---

## Executive Summary

This analysis examines the feasibility and optimal approach for implementing Cargo feature flags to control test execution in CI/CD pipelines. The project has a mature CI/CD setup with parallel test execution, service dependencies (Redis), and browser-based tests requiring special handling.

### Key Findings

1. **Current CI Structure**: Well-optimized with parallel execution, service containers, and artifact caching
2. **Test Categories**: Unit, integration, and browser tests already separated into matrix jobs
3. **Dependencies**: Redis available via Docker service containers; browser tests require Chrome
4. **Workspace**: 26 crates with varying dependency requirements
5. **Feature Flag Readiness**: Project already uses feature flags extensively (e.g., `jemalloc`, `profiling-full`, `persistence`)

---

## Current CI/CD Architecture

### Workflow Files

| Workflow | Purpose | Test Execution |
|----------|---------|----------------|
| `ci.yml` | Main CI pipeline | Unit, integration, browser (matrix) |
| `api-validation.yml` | API contract testing | Dredd, Schemathesis with Redis |
| `quality-gates.yml` | Advisory checks | Clippy, security audits, coverage |
| `docker-build.yml` | Container builds | Build verification |
| `metrics.yml` | Performance tracking | Benchmark execution |

### Test Execution Matrix (ci.yml)

```yaml
strategy:
  fail-fast: false
  matrix:
    test-type:
      - unit        # cargo test --lib --bins --exclude riptide-browser
      - integration # cargo test --tests --exclude riptide-browser
      - browser     # cargo test -p riptide-browser (single-threaded)
```

**Services Available:**
- Redis 7 Alpine (all test jobs)
- Health checks configured
- Port 6379 exposed

**Execution Details:**
- **Unit tests**: `--test-threads=4` (parallel, fast)
- **Integration tests**: `--test-threads=2` (parallel, moderate)
- **Browser tests**: `--test-threads=1` (serialized, Chrome SingletonLock avoidance)

---

## Dependency Analysis

### 1. **Redis Dependencies**

**Crates Using Redis:**
- `riptide-cache` (primary consumer)
- `riptide-persistence` (sync/storage)
- `riptide-cli` (health checks)

**Usage Pattern:**
- Connection pooling via workspace `redis` crate
- Optional in most crates (graceful degradation possible)
- **CI Availability**: ✅ Always available via service container

### 2. **Browser Dependencies**

**Crates Using Browser:**
- `riptide-browser` (core browser pool)
- `riptide-headless` (CDP interactions)
- `riptide-browser-abstraction` (unified interface)
- `riptide-facade` (high-level API)

**Complexity:**
- Chrome process management
- CDP protocol communication
- Profile isolation (SingletonLock conflicts)
- **CI Availability**: ✅ Available but requires careful orchestration (already handled)

### 3. **Other External Dependencies**

| Service | Usage | CI Availability |
|---------|-------|----------------|
| PDFium | PDF extraction | ✅ System package |
| Fontconfig | Font rendering | ✅ System package |
| WASM Runtime | Wasmtime 37 | ✅ Compiled in |

---

## Current Test Organization

### Workspace Structure

26 crates divided into:
- **Core infrastructure**: 9 crates (types, config, cache, events, etc.)
- **Extraction/Processing**: 6 crates (extraction, spider, pdf, search, etc.)
- **API/CLI**: 3 crates (api, cli, workers)
- **Browser**: 3 crates (browser, headless, browser-abstraction)
- **Advanced features**: 5 crates (intelligence, stealth, reliability, etc.)

### Existing Test Patterns

1. **Unit Tests** (`#[cfg(test)]` in lib files)
   - Fast, isolated, no external dependencies
   - Currently: 353 files with unit tests

2. **Integration Tests** (`tests/` directories)
   - 30+ integration test files across workspace
   - May require Redis, file system, network

3. **Browser Tests** (riptide-browser specific)
   - Require Chrome installation
   - Single-threaded execution
   - Profile management complexity

4. **Benchmark Tests** (`#[cfg(all(feature = "benchmarks", test))]`)
   - Already feature-gated
   - Example: riptide-pdf benchmarks

---

## Feature Flag Strategy Recommendations

### Proposed Feature Flags

```toml
# In workspace Cargo.toml
[workspace.metadata.test-features]
# Service dependency flags
redis-tests = []          # Tests requiring Redis
browser-tests = []        # Tests requiring Chrome/CDP
pdf-tests = []           # Tests requiring PDFium
network-tests = []       # Tests requiring internet access

# CI optimization flags
ci-fast = []             # Minimal tests for PR checks (excludes slow tests)
ci-full = []             # Comprehensive tests for main branch
ci-nightly = []          # Extended tests (fuzzing, property-based)

# Default CI configuration
default = ["redis-tests", "network-tests"]
```

### Test Categorization Strategy

#### Category 1: Always Run (No Feature Flag)
- Pure unit tests with zero external dependencies
- Computation logic, parsers, data structures
- **Examples**: CSS selector parsing, HTML transformation, type conversions
- **CI**: Every PR, every commit

#### Category 2: Standard Integration (Default Feature)
- Tests requiring commonly available services
- Redis-based caching tests
- File system I/O tests
- **Feature Flag**: `default` (always enabled in CI)
- **CI**: Every PR with service containers

#### Category 3: Browser Tests (Optional)
- Tests requiring Chrome/Chromium
- CDP protocol tests
- Headless rendering tests
- **Feature Flag**: `browser-tests`
- **CI**: Separate matrix job (already implemented)

#### Category 4: Extended Tests (Nightly/Release)
- Performance benchmarks
- Fuzzing tests (Schemathesis)
- Memory safety (Miri)
- Large dataset tests
- **Feature Flag**: `ci-nightly`
- **CI**: Scheduled runs, release branches

---

## Recommended CI Job Structure

### PR Checks (Fast Feedback)

```yaml
# .github/workflows/pr-checks.yml
name: PR Checks

on:
  pull_request:
    branches: [main]

jobs:
  quick-tests:
    name: Quick Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      # Fast unit tests only (no features)
      - name: Unit tests (no external deps)
        run: |
          cargo test --workspace \
            --lib \
            --exclude riptide-browser \
            --exclude riptide-headless \
            --no-default-features

      # Integration tests with Redis
      - name: Integration tests (with Redis)
        run: |
          cargo test --workspace \
            --tests \
            --exclude riptide-browser \
            --features redis-tests

    services:
      redis:
        image: redis:7-alpine
        ports: [6379:6379]

  browser-tests:
    name: Browser Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Browser tests
        run: |
          cargo test \
            -p riptide-browser \
            -p riptide-headless \
            --features browser-tests \
            -- --test-threads=1
```

### Main Branch (Comprehensive)

```yaml
# .github/workflows/main-branch.yml
name: Main Branch CI

on:
  push:
    branches: [main]

jobs:
  comprehensive-tests:
    name: Full Test Suite
    strategy:
      matrix:
        test-suite:
          - unit-integration
          - browser
          - benchmarks

    steps:
      - name: Run full test suite
        run: |
          case "${{ matrix.test-suite }}" in
            unit-integration)
              cargo test --workspace --features ci-full
              ;;
            browser)
              cargo test -p riptide-browser --features browser-tests
              ;;
            benchmarks)
              cargo bench --workspace --features benchmarks
              ;;
          esac

    services:
      redis:
        image: redis:7-alpine
        ports: [6379:6379]
```

### Nightly/Scheduled (Extended)

```yaml
# .github/workflows/nightly.yml
name: Nightly Tests

on:
  schedule:
    - cron: '0 2 * * *'  # 2 AM daily
  workflow_dispatch:

jobs:
  extended-tests:
    runs-on: ubuntu-latest
    steps:
      - name: Fuzzing tests
        run: |
          cargo test --workspace --features ci-nightly

      - name: Miri memory safety
        run: |
          cargo +nightly miri test --features ci-nightly

      - name: Property-based tests
        run: |
          cargo test --workspace --features "proptest ci-nightly"
```

---

## Implementation Plan

### Phase 1: Add Feature Flags (Non-Breaking)

**Workspace-Level** (`Cargo.toml`):
```toml
[workspace.dependencies]
# Add test feature flag documentation
# (actual flags defined per-crate)

[workspace.metadata.ci]
# CI configuration hints
default-features = ["redis-tests", "network-tests"]
pr-features = ["redis-tests"]
main-features = ["redis-tests", "browser-tests", "network-tests"]
nightly-features = ["redis-tests", "browser-tests", "network-tests", "benchmarks"]
```

**Per-Crate** (e.g., `riptide-cache/Cargo.toml`):
```toml
[features]
default = []
redis-tests = []  # Enable tests requiring Redis

[dev-dependencies]
redis = { workspace = true, optional = false }  # Always available for tests

# Example test gating:
# #[cfg(all(test, feature = "redis-tests"))]
# mod redis_integration_tests { ... }
```

### Phase 2: Annotate Tests

**Pattern 1: Module-level gating**
```rust
// tests/redis_integration.rs
#![cfg(all(test, feature = "redis-tests"))]

mod cache_tests {
    // All tests require Redis
}
```

**Pattern 2: Individual test gating**
```rust
#[test]
fn unit_test_no_deps() {
    // Always runs
}

#[test]
#[cfg(feature = "redis-tests")]
fn integration_test_with_redis() {
    // Only with redis-tests feature
}
```

**Pattern 3: Conditional compilation for test utilities**
```rust
#[cfg(all(test, feature = "redis-tests"))]
pub mod test_helpers {
    pub fn setup_redis() -> RedisClient { ... }
}
```

### Phase 3: Update CI Workflows

**Step 1**: Keep existing `ci.yml` as-is (no breaking changes)

**Step 2**: Add new optimized workflow `pr-fast-checks.yml`
```yaml
# Fast PR checks (5-10 minutes)
- Unit tests: --no-default-features
- Integration: --features redis-tests
```

**Step 3**: Enhance `ci.yml` for main branch
```yaml
# Comprehensive tests (15-20 minutes)
- All features enabled
- Browser tests in parallel
- Performance benchmarks
```

**Step 4**: Add scheduled nightly workflow
```yaml
# Extended tests (30-60 minutes)
- Fuzzing (Schemathesis)
- Property-based testing (proptest)
- Memory safety (Miri)
```

### Phase 4: Documentation Updates

1. **Developer Guide**: How to run tests locally with features
   ```bash
   # Fast local tests
   cargo test --workspace --no-default-features

   # With Redis (requires Docker)
   docker run -d -p 6379:6379 redis:7-alpine
   cargo test --workspace --features redis-tests

   # Full suite (like CI)
   cargo test --workspace --all-features
   ```

2. **CI Documentation**: Explain test tiers and when they run

3. **Contribution Guide**: PR expectations (fast tests must pass)

---

## Cost-Benefit Analysis

### Benefits

| Benefit | Impact | Measurement |
|---------|--------|-------------|
| **Faster PR feedback** | High | 15-20 min → 5-10 min |
| **Reduced CI costs** | Medium | ~40% fewer compute minutes |
| **Better developer experience** | High | Local tests without Docker |
| **Targeted test execution** | Medium | Run only relevant tests |
| **Clearer test requirements** | High | Self-documenting dependencies |

### Costs

| Cost | Impact | Mitigation |
|------|--------|------------|
| **Initial setup time** | Medium | Incremental rollout |
| **Maintenance overhead** | Low | Automated via CI checks |
| **Documentation burden** | Low | Template examples |
| **Potential for skipped tests** | Medium | Nightly full runs |

### Risks

1. **Test Coverage Gaps**: Developers might not run full suite locally
   - **Mitigation**: Nightly runs + main branch enforcement

2. **Feature Flag Confusion**: Which flags for which tests?
   - **Mitigation**: Clear naming + documentation + CI templates

3. **CI Complexity**: More workflows to maintain
   - **Mitigation**: Start simple, iterate based on metrics

---

## Comparison: Current vs Proposed

### Current State

```
PR Check (ci.yml)
├─ Quick checks (10 min)
├─ Build matrix (15 min)
│  ├─ native
│  └─ wasm32-wasip2
├─ Test matrix (20 min)
│  ├─ unit
│  ├─ integration
│  └─ browser
├─ Size check (5 min)
└─ Validation (2 min)

Total: ~25-30 minutes
```

### Proposed State

```
PR Check (pr-fast-checks.yml)
├─ Quick checks (5 min)
├─ Fast unit tests (8 min)
└─ Integration tests (7 min)

Total: ~12-15 minutes ✅

Main Branch (ci.yml)
├─ Full test suite (20 min)
├─ Build matrix (15 min)
├─ Browser tests (10 min)
└─ Benchmarks (10 min)

Total: ~30-35 minutes

Nightly (nightly.yml)
├─ Fuzzing (20 min)
├─ Miri (15 min)
├─ Property tests (10 min)
└─ Extended benchmarks (15 min)

Total: ~60 minutes
```

---

## Migration Strategy

### Week 1: Foundation
- [x] Add feature flag definitions to workspace Cargo.toml
- [x] Document feature flag usage in TESTING.md
- [ ] Add example gated tests in 2-3 crates

### Week 2: Core Crates
- [ ] Annotate tests in core crates (types, config, cache)
- [ ] Verify local test execution with/without features
- [ ] Update developer documentation

### Week 3: CI Integration
- [ ] Create `pr-fast-checks.yml` workflow
- [ ] Test PR workflow on feature branch
- [ ] Compare timing vs existing workflow

### Week 4: Rollout
- [ ] Enable fast PR checks for all PRs
- [ ] Monitor CI metrics (timing, cost, failures)
- [ ] Iterate based on feedback

### Week 5: Optimization
- [ ] Add nightly workflow for extended tests
- [ ] Fine-tune feature combinations
- [ ] Document best practices from learnings

---

## Recommendations Summary

### ✅ DO

1. **Implement feature flags incrementally**: Start with 2-3 crates, expand gradually
2. **Keep existing CI**: Don't break current workflows during transition
3. **Use clear naming**: `redis-tests`, `browser-tests` (self-explanatory)
4. **Leverage existing structure**: CI already has test matrix and services
5. **Default to comprehensive**: Main branch and releases get full test suite
6. **Document extensively**: Make it obvious which tests need which flags

### ✅ CONSIDER

1. **Separate PR and main workflows**: Different speed/coverage tradeoffs
2. **Matrix builds by feature**: Run different feature combinations in parallel
3. **Conditional services**: Only start Redis when `redis-tests` enabled
4. **Local testing scripts**: Provide `scripts/test-fast.sh`, `scripts/test-full.sh`
5. **CI cost monitoring**: Track compute minutes before/after implementation

### ❌ AVOID

1. **Don't over-granularize**: Too many feature flags = confusion
2. **Don't skip browser tests on main**: They're critical for riptide-browser crate
3. **Don't make feature flags optional dependencies**: Keep deps, gate tests only
4. **Don't remove nightly/comprehensive tests**: Balance speed with coverage
5. **Don't force features in dev-dependencies**: Keep tests as optional as possible

---

## Appendix: Example Implementation

### Example: riptide-cache Crate

**Cargo.toml**:
```toml
[features]
default = []
redis-tests = []

[dev-dependencies]
redis = { workspace = true }
tokio = { workspace = true }
```

**lib.rs**:
```rust
// Unit tests (always run)
#[cfg(test)]
mod tests {
    #[test]
    fn test_cache_key_parsing() {
        // Pure logic, no Redis
    }
}

// Integration tests (require Redis)
#[cfg(all(test, feature = "redis-tests"))]
mod redis_integration {
    use super::*;

    #[tokio::test]
    async fn test_redis_connection() {
        let client = redis::Client::open("redis://localhost:6379").unwrap();
        // ... Redis-specific tests
    }
}
```

**tests/cache_integration.rs**:
```rust
#![cfg(feature = "redis-tests")]

use riptide_cache::*;

#[tokio::test]
async fn test_distributed_cache() {
    // Requires Redis service
}
```

### Example: CI Workflow Snippet

```yaml
name: PR Fast Checks

on:
  pull_request:
    branches: [main]

jobs:
  fast-tests:
    runs-on: ubuntu-latest
    timeout-minutes: 15

    # Conditional services
    services:
      redis:
        image: redis:7-alpine
        ports: [6379:6379]
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s

    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      # Fast unit tests (no features)
      - name: Unit Tests (isolated)
        run: |
          cargo test --workspace \
            --lib \
            --no-default-features \
            --exclude riptide-browser \
            --exclude riptide-headless

      # Integration tests (with Redis)
      - name: Integration Tests (Redis)
        env:
          REDIS_URL: redis://localhost:6379
        run: |
          cargo test --workspace \
            --tests \
            --features redis-tests \
            --exclude riptide-browser \
            --exclude riptide-headless
```

---

## Conclusion

**Recommendation**: ✅ **PROCEED with feature flag implementation**

### Why?

1. **High ROI**: 40-50% faster PR checks with minimal complexity
2. **Low Risk**: Incremental rollout, existing CI remains intact
3. **Clear Pattern**: Project already uses feature flags extensively
4. **Infrastructure Ready**: CI has service containers and matrix builds
5. **Developer Benefits**: Easier local testing without full dependency stack

### Next Steps

1. Create feature flag definitions in workspace Cargo.toml
2. Annotate tests in 2-3 pilot crates (riptide-cache, riptide-extraction)
3. Build `pr-fast-checks.yml` workflow (parallel to existing CI)
4. Measure and compare CI timing/costs
5. Iterate and expand to remaining crates

### Success Metrics

- **PR feedback time**: Target <15 minutes (from ~25 minutes)
- **CI cost reduction**: Target 30-40% fewer compute minutes on PRs
- **Developer satisfaction**: Easier local testing workflow
- **Test coverage**: Maintain >80% coverage (nightly comprehensive runs)

---

**Document Version**: 1.0
**Last Updated**: 2025-10-27
**Author**: CI/CD Pipeline Engineer (Claude)
