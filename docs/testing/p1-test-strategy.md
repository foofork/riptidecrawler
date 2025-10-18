# P1 Test Strategy - Comprehensive Testing Plan

## Executive Summary

**Current Status**: Test suite cannot complete due to compilation errors in `riptide-api` related to monitoring imports.

**Critical Findings**:
- 100+ test files exist across the workspace (est. 500+ individual tests)
- Recent crate extractions (riptide-monitoring, riptide-security, riptide-fetch) need integration validation
- Compilation errors blocking all test execution
- 6 crates depend on newly extracted monitoring functionality

**Priority Actions**:
1. Fix compilation errors in riptide-api (monitoring imports)
2. Validate extracted crate functionality with unit tests
3. Create integration tests for cross-crate dependencies
4. Establish regression test baseline

---

## 1. Current Test Coverage Analysis

### Test Distribution by Crate

| Crate | Test Files | Estimated Tests | Status |
|-------|-----------|-----------------|--------|
| riptide-api | 35+ | 200+ | ⚠️ Compilation errors |
| riptide-extraction | 20+ | 150+ | ⚠️ Blocked by API |
| riptide-persistence | 9 | 60+ | ⚠️ Blocked by API |
| riptide-streaming | 8 | 50+ | ⚠️ Blocked by API |
| riptide-core | 8 | 80+ | ⚠️ Blocked by API |
| riptide-pdf | 4 | 30+ | ⚠️ Blocked by API |
| riptide-intelligence | 3 | 25+ | ⚠️ Blocked by API |
| riptide-cli | 2 | 15+ | ⚠️ Blocked by API |
| riptide-monitoring | 0 | 0 | ❌ No tests yet |
| riptide-security | 0 | 0 | ❌ No tests yet |
| riptide-fetch | 0 | 0 | ❌ No tests yet |
| riptide-spider | 1 | 5+ | ⚠️ Blocked by API |
| Other crates | 10+ | 50+ | ⚠️ Blocked by API |
| **TOTAL** | **100+** | **665+** | **0% passing** |

### Blocking Compilation Errors

**File**: `/workspaces/eventmesh/crates/riptide-api/src/state.rs`

```rust
// Line 672: MetricsCollector import error
let metrics_collector = riptide_core::monitoring::MetricsCollector::new();
// ERROR: could not find `MetricsCollector` in `monitoring`

// Line 1244: PerformanceMetrics import error
pub metrics: riptide_core::monitoring::PerformanceMetrics,
// ERROR: not found in `riptide_core::monitoring`
```

**Root Cause**: riptide-monitoring extraction changed module structure:
- Old: `riptide_core::monitoring::{MetricsCollector, PerformanceMetrics}`
- New: `riptide_monitoring::monitoring::{collector::MetricsCollector, metrics::PerformanceMetrics}`

**Required Fix**: Update imports in riptide-api to use new crate structure.

---

## 2. Test Gaps and Priorities

### Critical Gaps (Must Fix Before P1 Completion)

#### 2.1 Extracted Crates - Zero Test Coverage

**riptide-monitoring** (Priority: CRITICAL)
- 10 source files, 0 test files
- Components: alerts, collector, error, health, metrics, reports, time_series, telemetry
- **Needed**: 50+ unit tests, 10+ integration tests

**riptide-security** (Priority: CRITICAL)
- 8 source files, 0 test files
- Components: api_keys, audit, budget, middleware, pii
- **Needed**: 40+ unit tests, 8+ integration tests

**riptide-fetch** (Priority: CRITICAL)
- 5 source files, 0 test files
- Components: fetch, circuit breaker, robots.txt, telemetry
- **Needed**: 30+ unit tests, 5+ integration tests

#### 2.2 Cross-Crate Integration Tests

**Missing Integration Tests**:
1. riptide-api → riptide-monitoring (MetricsCollector, PerformanceMetrics)
2. riptide-api → riptide-security (middleware, auth, PII)
3. riptide-api → riptide-fetch (HTTP client, circuit breaker)
4. riptide-core → riptide-monitoring (telemetry integration)
5. riptide-core → riptide-security (security middleware)
6. riptide-core → riptide-fetch (fetch operations)

**Estimated Tests Needed**: 25+ integration tests

#### 2.3 Future Crate Readiness

**riptide-facade** (P1-A4 - Not Yet Created)
- Will need: 20+ unit tests, 15+ integration tests
- Test plan: Validate facade pattern hides internal complexity
- Key tests: Simplified API surface, backward compatibility

**riptide-headless-hybrid** (P1-A5 - Not Yet Created)
- Will need: 30+ unit tests, 20+ integration tests
- Test plan: Spider + Chrome integration validation
- Key tests: Browser pool lifecycle, CDP protocol, hybrid switching

---

## 3. Integration Test Strategy

### 3.1 Cross-Crate Dependency Testing

**Test Matrix**:

| Consumer | Provider | Test Coverage Goal | Status |
|----------|----------|-------------------|--------|
| riptide-api | riptide-monitoring | 90%+ | ❌ Not started |
| riptide-api | riptide-security | 85%+ | ❌ Not started |
| riptide-api | riptide-fetch | 85%+ | ❌ Not started |
| riptide-core | riptide-monitoring | 80%+ | ❌ Not started |
| riptide-core | riptide-security | 80%+ | ❌ Not started |
| riptide-core | riptide-fetch | 80%+ | ❌ Not started |

### 3.2 Facade Pattern Integration Tests

**Objective**: Validate riptide-facade simplifies riptide-core complexity

**Test Scenarios**:
1. **Simplified Initialization**: Facade provides 1-line setup vs multi-step core setup
2. **Unified Interface**: Single facade method replaces multiple core method calls
3. **Error Handling**: Facade translates internal errors to user-friendly messages
4. **Configuration**: Facade accepts simple config, translates to complex core config
5. **Backward Compatibility**: Facade maintains API stability as core evolves

**Test Template**: `/workspaces/eventmesh/docs/testing/templates/integration-facade-test.rs`

### 3.3 Spider-Chrome Hybrid Tests

**Objective**: Validate seamless switching between spider_rs and chrome_headless

**Test Scenarios**:
1. **Spider-Only Mode**: Fast crawling without JavaScript
2. **Chrome-Only Mode**: Full JavaScript execution and rendering
3. **Hybrid Switching**: Automatic fallback spider → chrome on JS detection
4. **Resource Management**: Browser pool lifecycle and cleanup
5. **CDP Protocol**: Chrome DevTools Protocol integration
6. **Performance**: Hybrid mode should be faster than always-chrome

**Test Template**: `/workspaces/eventmesh/docs/testing/templates/integration-hybrid-test.rs`

---

## 4. Migration Test Strategy

### 4.1 Pre-Migration Baseline

**Goal**: Capture current behavior before spider-chrome integration

**Tests to Create**:
```rust
// tests/migration/baseline_spider_behavior.rs
- test_spider_crawl_performance_baseline()
- test_spider_resource_usage_baseline()
- test_spider_success_rate_baseline()
- test_spider_error_handling_baseline()
```

### 4.2 Post-Migration Validation

**Goal**: Verify no regressions after chrome integration

**Tests to Create**:
```rust
// tests/migration/post_migration_validation.rs
- test_hybrid_maintains_spider_performance()
- test_chrome_fallback_does_not_degrade_non_js()
- test_resource_usage_within_baseline()
- test_error_handling_compatibility()
```

### 4.3 Migration Safety Tests

**Golden File Testing**: Snapshot current outputs, validate equivalence post-migration

```rust
// tests/migration/golden_tests.rs
#[test]
fn test_crawl_output_matches_golden() {
    let input = load_test_html("fixtures/sample.html");
    let output = crawl(input);
    assert_json_snapshot!(output);
}
```

---

## 5. Test Templates and Utilities

### 5.1 Unit Test Template

**File**: `/workspaces/eventmesh/docs/testing/templates/unit-test-template.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        // Arrange
        let input = create_test_input();

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected_output());
    }

    #[tokio::test]
    async fn test_async_functionality() {
        // Arrange
        let mock_service = MockService::new();

        // Act
        let result = async_function(&mock_service).await;

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_handling() {
        // Arrange
        let invalid_input = create_invalid_input();

        // Act
        let result = function_under_test(invalid_input);

        // Assert
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Expected error message");
    }
}
```

### 5.2 Integration Test Template

**File**: `/workspaces/eventmesh/docs/testing/templates/integration-test-template.rs`

```rust
use riptide_monitoring::monitoring::collector::MetricsCollector;
use riptide_security::middleware::SecurityMiddleware;
use riptide_api::AppState;

#[tokio::test]
async fn test_cross_crate_integration() {
    // Arrange: Setup all crates
    let metrics = MetricsCollector::new();
    let security = SecurityMiddleware::new();
    let state = AppState::new(metrics, security);

    // Act: Exercise integration
    let result = state.process_request(test_request()).await;

    // Assert: Validate cross-crate behavior
    assert!(result.is_ok());
    assert!(metrics.get_counter("requests").unwrap() > 0);
    assert!(security.audit_log_contains("request_processed"));
}
```

### 5.3 Performance Benchmark Template

**File**: `/workspaces/eventmesh/docs/testing/templates/benchmark-template.rs`

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_crawl_performance(c: &mut Criterion) {
    c.bench_function("spider_crawl_100_pages", |b| {
        b.iter(|| {
            let crawler = Spider::new();
            crawler.crawl(black_box("http://example.com"));
        });
    });
}

criterion_group!(benches, benchmark_crawl_performance);
criterion_main!(benches);
```

---

## 6. Regression Test Suite

### 6.1 Critical Path Tests

**Always-Pass Tests** (Must pass before any commit):

1. **Core Functionality**
   - `cargo test -p riptide-core --lib` - Core library tests
   - `cargo test -p riptide-api --lib` - API handler tests
   - `cargo test -p riptide-spider` - Spider integration tests

2. **Extracted Crates**
   - `cargo test -p riptide-monitoring` - Monitoring functionality
   - `cargo test -p riptide-security` - Security middleware
   - `cargo test -p riptide-fetch` - Fetch operations

3. **Integration Suite**
   - `cargo test --workspace --test integration_tests` - Cross-crate tests
   - `cargo test --workspace --test e2e_full_stack` - End-to-end tests

### 6.2 Performance Regression Tests

**Baseline Metrics** (To be established after compilation fixes):

```yaml
performance_baselines:
  spider_crawl_100_urls:
    max_duration_ms: 5000
    max_memory_mb: 100

  chrome_render_10_pages:
    max_duration_ms: 15000
    max_memory_mb: 500

  hybrid_crawl_50_urls:
    max_duration_ms: 8000
    max_memory_mb: 200
```

**Test Implementation**:
```rust
#[test]
fn test_no_performance_regression() {
    let baseline = load_baseline_metrics();
    let current = measure_current_performance();

    assert!(current.duration <= baseline.max_duration * 1.1); // 10% tolerance
    assert!(current.memory <= baseline.max_memory * 1.1);
}
```

### 6.3 Golden File Regression Tests

**Purpose**: Detect unexpected output changes

```rust
#[test]
fn test_extraction_output_unchanged() {
    let html = include_str!("fixtures/sample.html");
    let result = extract_content(html);

    insta::assert_json_snapshot!(result);
}
```

**Tools**: Use `insta` crate for snapshot testing

---

## 7. CI/CD Test Automation Strategy

### 7.1 GitHub Actions Workflow

**File**: `.github/workflows/test.yml`

```yaml
name: Test Suite

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Cache dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Run unit tests
        run: cargo test --workspace --lib

      - name: Run integration tests
        run: cargo test --workspace --test "*"

      - name: Run doc tests
        run: cargo test --workspace --doc

      - name: Check code coverage
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --workspace --out Xml --output-dir coverage

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: coverage/cobertura.xml
```

### 7.2 Pre-Commit Hooks

**File**: `.git/hooks/pre-commit`

```bash
#!/bin/bash
set -e

echo "Running pre-commit tests..."

# Fast smoke tests only
cargo test --workspace --lib -- --test-threads=1 --quiet

# Check formatting
cargo fmt -- --check

# Run clippy
cargo clippy --workspace -- -D warnings

echo "✓ All pre-commit tests passed"
```

### 7.3 Test Coverage Requirements

**Coverage Targets**:
- **Overall**: 80% line coverage minimum
- **Extracted Crates**: 90% line coverage (new code)
- **Critical Paths**: 95% branch coverage

**Enforcement**:
```bash
# Block PR merge if coverage drops below threshold
cargo tarpaulin --workspace --fail-under 80
```

---

## 8. Test Execution Plan

### Phase 1: Fix Compilation (Week 1)
- [ ] Fix riptide-api monitoring imports
- [ ] Verify all crates compile
- [ ] Run existing test suite
- [ ] Document current pass rate

### Phase 2: Extracted Crate Tests (Week 1-2)
- [ ] Write 50+ tests for riptide-monitoring
- [ ] Write 40+ tests for riptide-security
- [ ] Write 30+ tests for riptide-fetch
- [ ] Achieve 90%+ coverage on new crates

### Phase 3: Integration Tests (Week 2)
- [ ] Write 25+ cross-crate integration tests
- [ ] Test monitoring integration with API
- [ ] Test security middleware with API
- [ ] Test fetch operations with core

### Phase 4: Facade Tests (Week 2-3)
- [ ] Create riptide-facade crate
- [ ] Write 20+ unit tests for facade
- [ ] Write 15+ integration tests for facade
- [ ] Validate simplified API

### Phase 5: Hybrid Tests (Week 3)
- [ ] Write migration baseline tests
- [ ] Integrate spider-chrome
- [ ] Write 30+ hybrid integration tests
- [ ] Validate performance benchmarks

### Phase 6: Regression Suite (Week 4)
- [ ] Establish performance baselines
- [ ] Create golden file snapshots
- [ ] Setup CI/CD automation
- [ ] Document test procedures

---

## 9. Success Criteria

### P1 Completion Requirements

✅ **100% Test Pass Rate**
- All existing tests passing
- No compilation errors
- No ignored tests without justification

✅ **Extracted Crate Coverage**
- riptide-monitoring: 90%+ coverage
- riptide-security: 90%+ coverage
- riptide-fetch: 90%+ coverage

✅ **Integration Test Coverage**
- 25+ cross-crate integration tests passing
- All critical paths covered
- Error scenarios tested

✅ **Performance Validation**
- No regressions vs baseline
- Hybrid mode faster than always-chrome
- Memory usage within limits

✅ **CI/CD Automation**
- Tests run on every PR
- Coverage reports generated
- Pre-commit hooks active

---

## 10. Risk Assessment

### High Risk Areas

1. **Compilation Errors**: Blocking all test execution
   - **Mitigation**: Fix monitoring imports immediately

2. **Test Execution Time**: 500+ tests may timeout
   - **Mitigation**: Parallelize tests, use `cargo nextest`

3. **Flaky Tests**: Integration tests may be unstable
   - **Mitigation**: Use retries, mock external dependencies

4. **Coverage Blind Spots**: Complex async code hard to test
   - **Mitigation**: Integration tests + manual verification

### Medium Risk Areas

1. **Test Data Management**: Fixtures may become stale
   - **Mitigation**: Version control fixtures, document updates

2. **Performance Test Variance**: CI environment inconsistent
   - **Mitigation**: Use relative metrics, generous tolerances

---

## 11. Tools and Dependencies

### Testing Frameworks
- `tokio-test` - Async test utilities
- `mockall` - Mock object framework
- `wiremock` - HTTP mocking
- `criterion` - Performance benchmarking
- `insta` - Snapshot testing
- `cargo-nextest` - Fast test runner
- `cargo-tarpaulin` - Code coverage

### CI/CD Tools
- GitHub Actions - Test automation
- Codecov - Coverage reporting
- cargo-audit - Security scanning

---

## 12. Next Actions

### Immediate (This Week)
1. Fix riptide-api compilation errors (CODER agent)
2. Run full test suite and capture baseline
3. Create test files for extracted crates
4. Write first 20 unit tests

### Short Term (Next 2 Weeks)
1. Complete extracted crate test coverage
2. Write integration tests
3. Setup CI/CD automation
4. Document test procedures

### Long Term (Next Month)
1. Create facade and hybrid test suites
2. Establish performance baselines
3. Implement golden file testing
4. Achieve 80%+ overall coverage

---

## Appendix A: Test File Inventory

See separate file: `/workspaces/eventmesh/docs/testing/test-inventory.md`

## Appendix B: Performance Baselines

See separate file: `/workspaces/eventmesh/docs/testing/performance-baselines.md`

## Appendix C: Test Data Fixtures

See separate file: `/workspaces/eventmesh/docs/testing/test-fixtures.md`

---

**Document Version**: 1.0
**Last Updated**: 2025-10-18
**Author**: Tester Agent (Hive Mind)
**Status**: ✅ Comprehensive strategy complete, ready for execution
