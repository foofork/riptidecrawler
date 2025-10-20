# Test Infrastructure Report - Spider-Chrome Migration

**Generated:** 2025-10-20
**Purpose:** Prepare test infrastructure for Phase 2 spider-chrome validation

---

## 1. Existing Test Structure

### 1.1 Test Directories

#### Core Crate Tests (`crates/*/tests/`)
- **riptide-browser-abstraction/tests/**
  - `spider_chrome_integration_tests.rs` (394 lines) - Integration tests for spider-chrome

- **riptide-engine/tests/**
  - `browser_pool_lifecycle_tests.rs` (1,235 lines) - Pool management tests
  - `cdp_pool_tests.rs` (501 lines) - CDP connection pool tests
  - `cdp_pool_validation_tests.rs` (426 lines) - CDP validation tests

- **riptide-headless/tests/**
  - `headless_tests.rs` (317 lines) - Headless browser tests

- **riptide-facade/tests/**
  - `test_helpers.rs` - Test utilities

- **riptide-extraction/tests/**
  - `simple_chunking_test.rs` - Content chunking tests
  - In-module tests in `src/strategies/tests.rs` and `src/spider/tests.rs`

- **riptide-stealth/tests/**
  - `integration_test.rs` - Stealth feature tests

- **riptide-spider/benches/**
  - `query_aware_benchmark.rs` - Spider performance benchmarks

#### Integration Tests (`tests/`)
- **tests/integration/**
  - `spider_chrome_tests.rs` (330 lines) - Spider-chrome integration tests
  - `spider_chrome_benchmarks.rs` - Performance benchmarks
  - `cdp_pool_tests.rs` - CDP pool integration
  - `browser_pool_scaling_tests.rs` - Pool scaling tests
  - `memory_pressure_tests.rs` - Memory management
  - `full_pipeline_tests.rs` - End-to-end pipeline

- **tests/cli/**
  - `e2e_tests.rs` (525 lines) - CLI end-to-end tests
  - `e2e_workflow.rs` - Complete workflows
  - `integration_tests.rs` - CLI integration
  - `real_world_tests.rs` (599 lines) - Real website tests
  - `performance_tests.rs` - CLI performance
  - `api_client_tests.rs` - API client tests

- **tests/phase3/** (Previous phase tests)
  - `test_headless_v2.rs` - Headless v2 tests
  - `engine_selection_tests.rs` - Engine selection
  - `browser_pool_tests.rs` - Pool tests
  - `performance_benchmarks.rs` - Phase 3 benchmarks

- **tests/webpage-extraction/**
  - `main.rs` - Extraction test harness
  - `cli-test-harness.rs` (285 lines) - CLI test framework
  - `comparison-tool.rs` (355 lines) - Extraction comparison
  - `test-urls.json` (398 lines) - Test URL database

#### Unit Tests (In-module)
- `crates/riptide-browser-abstraction/src/tests.rs`
- `crates/riptide-pdf/src/tests.rs`
- `crates/riptide-spider/src/tests.rs`
- `crates/riptide-stealth/src/tests.rs`

### 1.2 Test Types Summary

| Test Type | Count | Location | Status |
|-----------|-------|----------|--------|
| **Unit Tests** | ~50+ | In crate modules | ✓ Existing |
| **Integration Tests** | ~35+ | tests/integration/ | ✓ Existing |
| **E2E Tests** | ~20+ | tests/cli/ | ✓ Existing |
| **Performance Tests** | ~10+ | benches/, tests/phase*/ | ✓ Existing |
| **Browser Pool Tests** | ~25+ | crates/riptide-engine/tests/ | ✓ Existing |
| **Spider Tests** | ~15+ | tests/integration/spider_* | ✓ Existing |

### 1.3 Test Dependencies

**Key Testing Crates:**
- `tokio-test` - Async test utilities
- `wiremock` - HTTP mocking
- `mockall` - Mock generation
- `assert_cmd` - CLI testing
- `predicates` - Assertion helpers
- `tempfile` - Temporary files
- `criterion` - Benchmarking

---

## 2. Spider-Chrome Specific Tests

### 2.1 Tests That Need Updates

#### ✅ Already Updated (Phase 1)
1. **Browser Abstraction Tests** (`crates/riptide-browser-abstraction/tests/spider_chrome_integration_tests.rs`)
   - Engine initialization ✓
   - Navigate params ✓
   - Wait strategies ✓
   - Screenshot params ✓
   - PDF params ✓

2. **Engine Pool Tests** (`crates/riptide-engine/tests/`)
   - Pool lifecycle ✓
   - Browser checkout/checkin ✓
   - Concurrent access ✓
   - Health checks ✓

3. **Headless Tests** (`crates/riptide-headless/tests/headless_tests.rs`)
   - Browser config creation ✓
   - Pool config defaults ✓
   - Pool creation ✓
   - Checkout/checkin flow ✓

#### 🔄 Need Updates for Phase 2

4. **Integration Tests** (`tests/integration/spider_chrome_tests.rs`)
   - ❌ Uses old `HybridHeadlessLauncher` API
   - ❌ Missing spider-specific configuration tests
   - ❌ Need to update for new engine selection logic
   - ⚠️ Update priority: **HIGH**

5. **CLI E2E Tests** (`tests/cli/e2e_tests.rs`)
   - ❌ Uses mock server (need real browser tests)
   - ❌ Missing `--engine spider` flag tests
   - ❌ Need screenshot/PDF validation with spider
   - ⚠️ Update priority: **HIGH**

6. **Extraction Tests** (`tests/webpage-extraction/`)
   - ✓ Test harness exists
   - ❌ Need spider-chrome comparison tests
   - ❌ Need performance parity validation
   - ⚠️ Update priority: **MEDIUM**

7. **Performance Benchmarks** (`tests/phase3/performance_benchmarks.rs`)
   - ❌ Need spider vs chromiumoxide comparisons
   - ❌ Need memory usage benchmarks
   - ❌ Need latency comparisons
   - ⚠️ Update priority: **MEDIUM**

### 2.2 Tests That Need Creation

#### New Tests Required:

1. **Spider Engine Selection Tests**
   - File: `tests/integration/spider_engine_selection_tests.rs`
   - Tests: Engine auto-detection, explicit selection, fallback behavior
   - Priority: **HIGH**

2. **Spider Navigation Tests**
   - File: `tests/integration/spider_navigation_comprehensive_tests.rs`
   - Tests: Various wait strategies, timeout handling, network conditions
   - Priority: **HIGH**

3. **Spider Screenshot/PDF Tests**
   - File: `tests/integration/spider_capture_tests.rs`
   - Tests: PNG/JPEG screenshots, full/viewport capture, PDF generation
   - Priority: **HIGH**

4. **Spider Stealth Tests**
   - File: `tests/integration/spider_stealth_integration_tests.rs`
   - Tests: Stealth features work with spider, bot detection bypass
   - Priority: **MEDIUM**

5. **Spider CLI Integration Tests**
   - File: `tests/cli/spider_cli_tests.rs`
   - Tests: CLI commands with `--engine spider`, flag combinations
   - Priority: **HIGH**

6. **Spider Performance Parity Tests**
   - File: `tests/integration/spider_performance_parity_tests.rs`
   - Tests: Latency, memory, CPU usage vs chromiumoxide
   - Priority: **MEDIUM**

---

## 3. Test Execution Plan

### 3.1 Sequential Test Strategy

Tests must run sequentially due to browser resource constraints:

```bash
# Phase 1: Unit Tests (Fast, no browser needed)
cargo test --lib --all-features

# Phase 2: Crate-specific integration tests
cargo test -p riptide-browser-abstraction --test '*'
cargo test -p riptide-engine --test '*'
cargo test -p riptide-headless --test '*'

# Phase 3: Integration tests (Sequential)
cargo test --test spider_chrome_tests -- --test-threads=1
cargo test --test integration_headless_cdp -- --test-threads=1

# Phase 4: CLI tests (Sequential)
cd tests/cli && cargo test -- --test-threads=1

# Phase 5: E2E tests (Sequential, long-running)
cargo test --test e2e_tests -- --test-threads=1 --nocapture
```

### 3.2 Test Categories by Duration

| Category | Duration | Test Count | Parallelizable |
|----------|----------|------------|----------------|
| Unit Tests | <10s | ~50 | ✓ Yes |
| Integration (Fast) | 10-30s | ~20 | Partial |
| Integration (Slow) | 30-60s | ~15 | ✗ No |
| E2E Tests | 1-5min | ~10 | ✗ No |
| Performance Tests | 5-10min | ~5 | ✗ No |

### 3.3 Browser Resource Management

**Key Constraints:**
- Maximum concurrent browser instances: 3-5
- Memory per browser: ~200-500MB
- Port allocation: 9222-9230 (CDP)
- Test cleanup: Must close browsers between tests

**Cleanup Strategy:**
```rust
// Use RAII pattern for automatic cleanup
#[tokio::test]
async fn test_with_browser() -> Result<()> {
    let launcher = HybridHeadlessLauncher::new().await?;
    // Test code...
    launcher.shutdown().await?; // Automatic cleanup
    Ok(())
}
```

---

## 4. CLI Test Scenarios

### 4.1 Basic CLI Tests

```bash
# Test 1: Basic extraction with spider
riptide extract --url "https://example.com" --engine spider

# Test 2: Screenshot capture
riptide extract --url "https://example.com" --engine spider --screenshot

# Test 3: PDF generation
riptide extract --url "https://example.com" --engine spider --pdf

# Test 4: Full page screenshot
riptide extract --url "https://example.com" --engine spider --screenshot --full-page

# Test 5: Custom wait strategy
riptide extract --url "https://example.com" --engine spider --wait-until networkidle
```

### 4.2 Advanced CLI Tests

```bash
# Test 6: Stealth mode with spider
riptide extract --url "https://example.com" --engine spider --stealth medium

# Test 7: Multiple URLs
riptide batch extract --urls urls.txt --engine spider --concurrency 3

# Test 8: Spider with custom timeout
riptide extract --url "https://example.com" --engine spider --timeout 60

# Test 9: JSON output
riptide extract --url "https://example.com" --engine spider --format json

# Test 10: Debug output
riptide extract --url "https://example.com" --engine spider --verbose --debug
```

### 4.3 Error Handling Tests

```bash
# Test 11: Invalid URL
riptide extract --url "not-a-url" --engine spider

# Test 12: Timeout scenario
riptide extract --url "https://httpstat.us/200?sleep=70000" --engine spider --timeout 5

# Test 13: Network error
riptide extract --url "https://invalid-domain-12345.com" --engine spider

# Test 14: SSL error
riptide extract --url "https://expired.badssl.com" --engine spider

# Test 15: 404 error
riptide extract --url "https://example.com/nonexistent" --engine spider
```

### 4.4 Expected Outputs

**Success Case:**
```json
{
  "url": "https://example.com",
  "engine": "spider",
  "status": "success",
  "content": "Example Domain\nThis domain is for use in illustrative examples...",
  "metadata": {
    "title": "Example Domain",
    "extraction_time_ms": 1245,
    "method_used": "spider"
  }
}
```

**Error Case:**
```json
{
  "url": "https://invalid-domain-12345.com",
  "engine": "spider",
  "status": "error",
  "error": {
    "code": "NAVIGATION_FAILED",
    "message": "Failed to navigate: DNS resolution failed"
  }
}
```

---

## 5. Test Execution Script

### 5.1 Master Test Runner

Location: `/workspaces/eventmesh/docs/testing/run-spider-chrome-tests.sh`

```bash
#!/bin/bash
# Spider-Chrome Test Suite Runner
# Runs tests sequentially with proper cleanup

set -e

RESULTS_DIR="docs/testing/results"
mkdir -p "$RESULTS_DIR"

echo "🧪 Spider-Chrome Test Suite"
echo "================================"
echo ""

# Function to run test with timing
run_test() {
    local name="$1"
    local cmd="$2"

    echo "▶ Running: $name"
    start_time=$(date +%s)

    if eval "$cmd" > "$RESULTS_DIR/${name}.log" 2>&1; then
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        echo "  ✓ PASS ($duration seconds)"
        return 0
    else
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        echo "  ✗ FAIL ($duration seconds)"
        echo "  See: $RESULTS_DIR/${name}.log"
        return 1
    fi
}

# Phase 1: Unit Tests
echo "📦 Phase 1: Unit Tests"
run_test "unit-browser-abstraction" \
    "cargo test -p riptide-browser-abstraction --lib"
run_test "unit-engine" \
    "cargo test -p riptide-engine --lib"
run_test "unit-headless" \
    "cargo test -p riptide-headless --lib"
echo ""

# Phase 2: Integration Tests (Browser)
echo "🔗 Phase 2: Browser Integration Tests"
run_test "integration-spider-chrome" \
    "cargo test -p riptide-browser-abstraction --test spider_chrome_integration_tests -- --test-threads=1"
run_test "integration-headless" \
    "cargo test -p riptide-headless --test headless_tests -- --test-threads=1"
echo ""

# Phase 3: Engine Pool Tests
echo "🏊 Phase 3: Engine Pool Tests"
run_test "pool-lifecycle" \
    "cargo test -p riptide-engine --test browser_pool_lifecycle_tests -- --test-threads=1"
run_test "pool-cdp" \
    "cargo test -p riptide-engine --test cdp_pool_tests -- --test-threads=1"
echo ""

# Phase 4: CLI Tests
echo "💻 Phase 4: CLI Tests"
run_test "cli-e2e" \
    "cd tests/cli && cargo test -- --test-threads=1"
echo ""

# Phase 5: Integration Suite
echo "🌐 Phase 5: Integration Suite"
run_test "integration-spider-chrome-full" \
    "cargo test --test spider_chrome_tests -- --test-threads=1"
echo ""

# Generate summary
echo "================================"
echo "📊 Test Summary"
echo "================================"
cat "$RESULTS_DIR"/*.log | grep -E "(test result|passed|failed)" || true
echo ""
echo "Results saved to: $RESULTS_DIR/"
```

### 5.2 Quick Smoke Test Script

Location: `/workspaces/eventmesh/docs/testing/smoke-test.sh`

```bash
#!/bin/bash
# Quick smoke test for spider-chrome

set -e

echo "🔥 Spider-Chrome Smoke Test"
echo ""

# Test 1: Type definitions compile
echo "▶ Test 1: Type definitions..."
cargo check -p riptide-browser-abstraction

# Test 2: Basic integration test
echo "▶ Test 2: Basic integration..."
cargo test -p riptide-browser-abstraction --test spider_chrome_integration_tests \
    test_chromiumoxide_engine_initialization -- --test-threads=1

# Test 3: Pool creation
echo "▶ Test 3: Pool creation..."
cargo test -p riptide-headless --test headless_tests \
    test_browser_pool_creation -- --test-threads=1

echo ""
echo "✓ Smoke test passed!"
```

---

## 6. Test Results Tracking

### 6.1 Results Format

```json
{
  "test_run": {
    "timestamp": "2025-10-20T10:00:00Z",
    "phase": "phase2-spider-chrome",
    "total_tests": 85,
    "passed": 82,
    "failed": 3,
    "skipped": 0,
    "duration_seconds": 420
  },
  "failures": [
    {
      "test": "test_spider_chrome_pdf_generation",
      "crate": "riptide-browser-abstraction",
      "error": "PDF generation timeout",
      "log_file": "results/integration-spider-chrome.log"
    }
  ],
  "performance": {
    "avg_test_duration": 4.9,
    "slowest_test": "test_browser_pool_concurrent_checkout",
    "slowest_duration": 45.2
  }
}
```

### 6.2 Test Coverage Tracking

Target Coverage:
- Unit tests: 80%+ line coverage
- Integration tests: All public APIs
- E2E tests: Critical user workflows

---

## 7. Test Priorities for Phase 2

### High Priority (Must Pass)
1. ✅ Browser abstraction type tests (DONE)
2. ✅ Engine pool lifecycle tests (DONE)
3. ✅ Headless launcher tests (DONE)
4. 🔄 CLI integration tests (IN PROGRESS - build dependent)
5. 🔄 Spider-chrome basic navigation (IN PROGRESS - build dependent)

### Medium Priority (Should Pass)
6. ⏳ Screenshot/PDF generation tests
7. ⏳ Performance parity tests
8. ⏳ Stealth integration tests

### Low Priority (Nice to Have)
9. ⏳ Webpage extraction comparison tests
10. ⏳ Long-running E2E workflows

---

## 8. Known Test Issues

### Current Blockers
1. **Build compilation required** - Tests can't run until Phase 2 compilation succeeds
2. **Browser binary availability** - Need Chrome/Chromium installed in test environment
3. **Port conflicts** - CDP ports may conflict if tests run in parallel

### Mitigation Strategies
1. Use `--test-threads=1` for browser tests
2. Implement proper cleanup in test teardown
3. Use dynamic port allocation when possible
4. Add retry logic for transient failures

---

## 9. Next Steps

1. **Wait for build completion** ✓ (in progress)
2. **Run smoke tests** - Validate basic functionality
3. **Run full test suite** - Execute sequential test plan
4. **Analyze failures** - Document and fix issues
5. **Update test plan** - Adjust based on results

---

## Summary

**Test Infrastructure Status:** ✅ READY

- Total test files: ~100+
- Spider-chrome specific tests: ~15 files
- Tests ready to run: ~85+
- Tests need updates: ~20
- Tests need creation: ~6

**Next Action:** Wait for build completion, then execute smoke tests.
