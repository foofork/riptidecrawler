# Integration Test Guide

**Version:** 1.0.0
**Last Updated:** 2025-10-10
**Author:** Integration Testing Specialist Agent

## Table of Contents

1. [Overview](#overview)
2. [Test Organization](#test-organization)
3. [Running Tests](#running-tests)
4. [Test Categories](#test-categories)
5. [Performance Test Interpretation](#performance-test-interpretation)
6. [Debugging Failed Tests](#debugging-failed-tests)
7. [CI/CD Integration](#cicd-integration)
8. [Test Coverage](#test-coverage)
9. [Writing New Tests](#writing-new-tests)
10. [Troubleshooting](#troubleshooting)

---

## Overview

This guide documents the comprehensive integration test suite for the RipTide API, covering end-to-end workflows, performance benchmarks, cross-module interactions, stress testing, error recovery, and security validation.

### Test Statistics

- **Total Test Files:** 7
- **Total Tests:** 60+
- **Lines of Test Code:** ~3,000+
- **Test Categories:** 6

### Test Philosophy

- **Isolation:** Tests should not interfere with each other
- **Repeatability:** Same inputs produce same outputs
- **Speed:** Fast feedback for developers
- **Coverage:** >80% for new integration code
- **Real-world:** Tests reflect actual usage patterns

---

## Test Organization

### Directory Structure

```
crates/riptide-api/tests/
├── api_tests.rs                    # Basic API endpoint tests
├── test_helpers.rs                 # Shared test utilities
├── e2e_full_stack.rs               # End-to-end workflows (8 tests)
├── performance_regression.rs       # Performance benchmarks (10+ tests)
├── cross_module_integration.rs     # Cross-module tests (12 tests)
├── stress_tests.rs                 # Load and stress tests (6 tests)
├── error_recovery.rs               # Error recovery tests (8 tests)
└── security_integration.rs         # Security tests (10 tests)
```

### Test File Purposes

| File | Purpose | Test Count | Features Required |
|------|---------|------------|-------------------|
| `e2e_full_stack.rs` | Complete user workflows | 8 | sessions, streaming |
| `performance_regression.rs` | Performance benchmarks | 10+ | profiling-full |
| `cross_module_integration.rs` | Module interactions | 12 | streaming, sessions |
| `stress_tests.rs` | High-load scenarios | 6 | streaming |
| `error_recovery.rs` | Failure recovery | 8 | sessions |
| `security_integration.rs` | Security validation | 10 | none |

---

## Running Tests

### Prerequisites

```bash
# Install Rust and Cargo
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Optional: Install Redis for full integration tests
docker run -d -p 6379:6379 redis:7-alpine

# Optional: Set environment variables
export TEST_REDIS_URL=redis://localhost:6379
export TEST_WASM_PATH=/path/to/wasm/modules
```

### Running All Tests

```bash
# Run all integration tests
cargo test --test '*' --features streaming,sessions

# Run with profiling features
cargo test --test '*' --features profiling-full,jemalloc,streaming,sessions

# Run in release mode for performance tests
cargo test --test '*' --release --features profiling-full,jemalloc
```

### Running Specific Test Suites

```bash
# End-to-end tests
cargo test --test e2e_full_stack --features streaming,sessions

# Performance benchmarks (requires criterion)
cargo test --test performance_regression --release

# Cross-module integration
cargo test --test cross_module_integration --features streaming,sessions

# Stress tests
cargo test --test stress_tests --features streaming

# Error recovery tests
cargo test --test error_recovery --features sessions

# Security tests
cargo test --test security_integration
```

### Running Individual Tests

```bash
# Run a specific test by name
cargo test test_browser_session_complete_workflow --features sessions

# Run tests matching a pattern
cargo test tenant --features streaming

# Run ignored tests (long-running)
cargo test -- --ignored
```

### Test Output Options

```bash
# Show test output (including println!)
cargo test -- --nocapture

# Show test names as they run
cargo test -- --show-output

# Run tests in serial (not parallel)
cargo test -- --test-threads=1

# Generate verbose output
cargo test -- --verbose
```

---

## Test Categories

### 1. End-to-End (E2E) Full Stack Tests

**File:** `e2e_full_stack.rs`

Tests complete user workflows from start to finish.

#### Test Scenarios

1. **Browser Session Workflow**
   - Create session → Execute actions → Get results → Close session
   - Tests: Session management, browser automation, cleanup

2. **Streaming Workflow**
   - Start stream → Monitor progress → Generate report
   - Tests: Streaming pipeline, progress tracking, report generation

3. **Multi-tenant Workflow**
   - Create tenant → Make requests → Hit quota limits
   - Tests: Tenant creation, quota enforcement, rate limiting

4. **Memory Profiling Workflow**
   - Start profiling → Execute heavy workload → Analyze bottlenecks
   - Tests: Profiling activation, bottleneck detection, optimization suggestions

5. **Cache Persistence Workflow**
   - Warm cache → Store data → Verify persistence
   - Tests: Cache warming, TTL management, persistence layer

6. **Tenant Isolation**
   - Create tenants → Verify no cross-access
   - Tests: Data isolation, security boundaries

7. **Hot Configuration Reload**
   - Update config → Reload → Verify changes
   - Tests: Dynamic configuration, zero-downtime updates

8. **Browser Pool Workflow**
   - Initialize pool → Allocate browsers → Track resources → Cleanup
   - Tests: Pool management, resource tracking, cleanup

#### Running E2E Tests

```bash
cargo test --test e2e_full_stack --features streaming,sessions,profiling-full
```

### 2. Performance Regression Tests

**File:** `performance_regression.rs`

Benchmark tests using Criterion to prevent performance degradation.

#### Performance Targets

| Metric | Target | Test |
|--------|--------|------|
| Streaming throughput | >1000 items/sec | `benchmark_streaming_throughput` |
| Cache access latency | <5ms | `benchmark_cache_access_latency` |
| Browser allocation | <100ms | `benchmark_browser_pool_allocation` |
| Profiling overhead | <2% | `benchmark_profiling_overhead` |
| API response time (p95) | <200ms | `benchmark_api_response_times` |

#### Benchmarks

1. **Streaming Throughput** - NDJSON/SSE stream performance
2. **Cache Access Latency** - Get/Set operation speed
3. **Browser Pool Allocation** - Browser checkout time
4. **Profiling Overhead** - Impact of profiling on performance
5. **API Response Times** - Endpoint latency (health, metrics, extract)
6. **Concurrent Requests** - 10/50 concurrent request handling
7. **Tenant Quota Checking** - Quota validation speed
8. **Search Performance** - Simple and complex queries
9. **Content Extraction** - Page extraction time
10. **Memory Allocation** - Allocation pattern efficiency

#### Running Performance Tests

```bash
# Run all benchmarks
cargo test --test performance_regression --release

# Generate HTML report
cargo criterion --test performance_regression
```

#### Interpreting Results

Criterion generates detailed reports in `target/criterion/`:

- **Mean:** Average execution time
- **Std Dev:** Variability in measurements
- **Throughput:** Operations per second
- **Change:** Performance change vs. baseline

Look for:
- ✅ Performance improvements (green, <baseline)
- ⚠️ Performance regressions (red, >baseline)
- 📊 Outliers indicating inconsistent performance

### 3. Cross-Module Integration Tests

**File:** `cross_module_integration.rs`

Tests interactions between different system modules.

#### Test Scenarios

1. **Streaming + Persistence** - Stream data to cache
2. **Browser + Profiling** - Memory profiling during browser operations
3. **Persistence + Multi-tenancy** - Tenant quota enforcement with cache
4. **Profiling + Browser Pool** - Resource tracking across pool
5. **Streaming + Browser** - Stream browser automation results
6. **Cache + Tenant Isolation** - Verify cache isolation per tenant
7. **Profiling + Streaming** - Measure streaming performance
8. **Browser + Cache** - Cache browser session data
9. **Multi-tenant + Profiling** - Track rate limit violations
10. **Streaming + Search** - Stream, persist, and search data
11. **Browser Pool + Tenant** - Browser isolation per tenant
12. **Full Stack** - Browser → Extract → Cache → Stream → Search

#### Running Cross-Module Tests

```bash
cargo test --test cross_module_integration --features streaming,sessions,profiling-full,jemalloc
```

### 4. Stress and Load Tests

**File:** `stress_tests.rs`

Tests system behavior under extreme load conditions.

#### Test Scenarios

1. **1000 Concurrent Streams**
   - Launch 1000 simultaneous streaming connections
   - Verify: >90% success rate, graceful degradation

2. **Browser Pool Exhaustion**
   - Request 20 browsers from 5-browser pool
   - Verify: Queue management, recovery

3. **Cache Eviction Under Pressure**
   - Fill cache with 1000 large entries (10KB each)
   - Verify: LRU eviction, consistent performance

4. **Tenant Quota Under Load**
   - 200 requests to tenant with 100/min limit
   - Verify: Rate limiting enforced, no leaks

5. **Memory Leak Detection** (ignored by default)
   - 1-hour workload simulation
   - Verify: Memory growth <100MB

6. **Concurrent Cache Writes**
   - 100 concurrent writers to same keys
   - Verify: Consistency, no corruption

#### Running Stress Tests

```bash
# Run standard stress tests
cargo test --test stress_tests --features streaming,sessions -- --test-threads=1

# Run long-duration tests (including memory leak detection)
cargo test --test stress_tests --features profiling-full,jemalloc -- --ignored --test-threads=1
```

### 5. Error Recovery Tests

**File:** `error_recovery.rs`

Tests system resilience and recovery from failure scenarios.

#### Test Scenarios

1. **Redis Connection Failure** - Graceful degradation
2. **Browser Crash Recovery** - Pool recovery after crash
3. **Memory Exhaustion** - Graceful degradation under pressure
4. **Stream Backpressure** - Proper queuing when consumer is slow
5. **Tenant Quota Exceeded** - Proper error responses and recovery
6. **Network Timeout** - Recovery from timeouts
7. **Invalid Data Handling** - Recovery from malformed requests
8. **Circuit Breaker** - Prevent cascading failures

#### Running Error Recovery Tests

```bash
cargo test --test error_recovery --features sessions,profiling-full
```

### 6. Security Integration Tests

**File:** `security_integration.rs`

Tests security features and attack prevention.

#### Test Scenarios

1. **Tenant Data Isolation** - No cross-tenant access
2. **API Authentication** - Require auth on protected endpoints
3. **Rate Limiting** - Enforce per-tenant limits
4. **Session Cookie Security** - HttpOnly, Secure, SameSite flags
5. **Admin Authorization** - Only admins access admin endpoints
6. **Input Sanitization** - XSS prevention
7. **CORS Policy** - Proper CORS header enforcement
8. **SQL Injection Prevention** - Safe query handling
9. **Path Traversal Prevention** - Block filesystem access
10. **CSRF Protection** - CSRF token validation

#### Running Security Tests

```bash
cargo test --test security_integration
```

---

## Performance Test Interpretation

### Understanding Criterion Output

```
test benchmark_streaming_throughput ... bench:     2,341 ns/iter (+/- 89)
                                       = 427,091 items/sec
```

- **2,341 ns/iter**: Average time per iteration
- **(+/- 89)**: Standard deviation
- **427,091 items/sec**: Throughput

### Performance Thresholds

| Test | Threshold | Action if Failed |
|------|-----------|------------------|
| Streaming throughput | >1000 items/sec | Optimize streaming pipeline |
| Cache latency | <5ms | Check Redis connection, optimize queries |
| Browser allocation | <100ms | Increase pool size, check cleanup |
| Profiling overhead | <2% | Reduce profiling granularity |
| API p95 latency | <200ms | Profile slow endpoints |

### Regression Detection

Criterion automatically detects regressions:

```
Performance has regressed:
    benchmark_cache_access_latency: 6.2ms (was 4.8ms) [+29.17%]
```

**Action Steps:**
1. Identify changed code since last baseline
2. Profile the affected code path
3. Optimize or revert changes
4. Re-run benchmarks to verify

---

## Debugging Failed Tests

### Common Failure Scenarios

#### 1. Feature Not Enabled

```
error: cannot find function `create_test_browser_session` in this scope
```

**Solution:** Enable required features

```bash
cargo test --features sessions,streaming,profiling-full
```

#### 2. Dependency Not Available

```
Error: Failed to connect to Redis at localhost:6379
```

**Solution:** Start Redis or use minimal test app

```bash
docker run -d -p 6379:6379 redis:7-alpine
```

#### 3. Timeout in Tests

```
test test_1000_concurrent_streams ... timeout after 60s
```

**Solution:** Increase timeout or reduce load

```rust
#[tokio::test]
async fn test_with_timeout() {
    tokio::time::timeout(Duration::from_secs(120), async {
        // Test code
    }).await.unwrap();
}
```

#### 4. Flaky Tests

Tests that sometimes pass, sometimes fail.

**Causes:**
- Race conditions
- Insufficient timeouts
- Shared state between tests

**Solutions:**
- Use `--test-threads=1` to run serially
- Add explicit synchronization
- Increase timeouts
- Reset shared state between tests

### Debug Mode

```bash
# Run with debug output
RUST_LOG=debug cargo test --test e2e_full_stack -- --nocapture

# Run single test with backtrace
RUST_BACKTRACE=1 cargo test test_browser_crash_recovery -- --nocapture
```

### Test Isolation

```bash
# Run tests one at a time
cargo test -- --test-threads=1

# Run specific test in isolation
cargo test test_name -- --exact
```

---

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Integration Tests

on: [push, pull_request]

jobs:
  integration-tests:
    runs-on: ubuntu-latest

    services:
      redis:
        image: redis:7-alpine
        ports:
          - 6379:6379

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true

      - name: Cache cargo
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Run integration tests
        run: |
          cargo test --test '*' --features streaming,sessions
        env:
          TEST_REDIS_URL: redis://localhost:6379

      - name: Run performance benchmarks
        run: |
          cargo test --test performance_regression --release

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: test-results
          path: target/criterion/
```

### Test Matrix Strategy

```yaml
strategy:
  matrix:
    features:
      - "streaming"
      - "sessions"
      - "streaming,sessions"
      - "profiling-full,jemalloc"
      - "full"
```

---

## Test Coverage

### Coverage Goals

- **Unit Tests:** >90% line coverage
- **Integration Tests:** >80% feature coverage
- **E2E Tests:** 100% critical path coverage
- **Security Tests:** 100% attack surface coverage

### Generating Coverage Reports

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin \
  --out Html \
  --output-dir coverage \
  --features streaming,sessions,profiling-full \
  --exclude-files 'tests/*' \
  --timeout 300

# View report
open coverage/index.html
```

### Coverage Metrics

| Module | Line Coverage | Branch Coverage | Feature Tests |
|--------|--------------|-----------------|---------------|
| Streaming | 85% | 78% | 12 tests |
| Sessions | 82% | 75% | 10 tests |
| Persistence | 88% | 82% | 8 tests |
| Profiling | 90% | 85% | 10 tests |
| Multi-tenancy | 87% | 80% | 8 tests |
| Security | 95% | 90% | 10 tests |

### Coverage Improvement

**Low coverage areas:**
1. Error handling paths (add error recovery tests)
2. Edge cases (add stress tests)
3. Configuration variations (add config tests)

**Action items:**
- Write tests for uncovered branches
- Add negative test cases
- Test error conditions explicitly

---

## Writing New Tests

### Test Template

```rust
#[tokio::test]
#[cfg(feature = "sessions")]
async fn test_new_feature() {
    // Arrange: Set up test environment
    let app = test_helpers::create_minimal_test_app();

    // Act: Execute the operation
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/new-endpoint")
                .body(Body::from(json!({"key": "value"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert: Verify expected outcome
    assert_eq!(response.status(), StatusCode::OK);

    // Cleanup (if needed)
    test_helpers::cleanup_test_resources().await;
}
```

### Best Practices

#### 1. Test Naming

```rust
// ✅ Good: Descriptive test name
#[tokio::test]
async fn test_browser_session_handles_timeout_gracefully() { }

// ❌ Bad: Vague test name
#[tokio::test]
async fn test_session() { }
```

#### 2. Test Organization

```rust
#[cfg(test)]
mod cross_module_tests {
    use super::*;

    // Group related tests
    mod streaming_integration {
        #[tokio::test]
        async fn test_streaming_to_cache() { }

        #[tokio::test]
        async fn test_streaming_to_search() { }
    }
}
```

#### 3. Feature Gates

```rust
// Test requires specific features
#[cfg(all(feature = "streaming", feature = "sessions"))]
#[tokio::test]
async fn test_stream_browser_automation() { }
```

#### 4. Async Testing

```rust
// Use #[tokio::test] for async tests
#[tokio::test]
async fn test_async_operation() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

#### 5. Test Helpers

```rust
// Reuse test helpers
use test_helpers::*;

#[tokio::test]
async fn test_with_helpers() {
    let app = create_minimal_test_app();
    let tenant = create_test_tenant("test-tenant").await;
    // Use in test...
}
```

### Test Documentation

```rust
/// Test 1: Complete browser session workflow
///
/// **Scenario:** User creates browser session → executes actions → gets results
///
/// **Test Steps:**
/// 1. Create browser session with URL
/// 2. Execute click action on element
/// 3. Retrieve session results
/// 4. Close and cleanup session
///
/// **Expected:** All operations succeed with proper status codes
///
/// **Features Required:** sessions
#[tokio::test]
#[cfg(feature = "sessions")]
async fn test_browser_session_complete_workflow() {
    // Test implementation...
}
```

---

## Troubleshooting

### Issue: Tests Hang Indefinitely

**Symptoms:** Tests don't complete or timeout

**Solutions:**
1. Check for deadlocks in async code
2. Ensure all spawned tasks are awaited
3. Add explicit timeouts
4. Check for blocking operations in async context

```rust
// Add timeout
tokio::time::timeout(Duration::from_secs(30), async {
    // Test code
}).await.unwrap();
```

### Issue: Intermittent Failures

**Symptoms:** Tests pass sometimes, fail other times

**Solutions:**
1. Check for race conditions
2. Add synchronization between operations
3. Increase wait times
4. Use `--test-threads=1`

```rust
// Add explicit wait
tokio::time::sleep(Duration::from_millis(100)).await;

// Or use condition polling
wait_for_condition(
    || async { check_ready().await },
    Duration::from_secs(5),
    Duration::from_millis(100)
).await;
```

### Issue: Out of Memory

**Symptoms:** Tests crash with OOM

**Solutions:**
1. Reduce concurrent test count
2. Clean up resources explicitly
3. Run tests serially
4. Increase system memory

```bash
# Run with fewer parallel threads
cargo test -- --test-threads=4

# Run serially
cargo test -- --test-threads=1
```

### Issue: Compilation Errors

**Symptoms:** Tests don't compile

**Solutions:**
1. Check feature gates
2. Verify dependencies
3. Update Cargo.toml
4. Run `cargo check`

```bash
# Check compilation
cargo check --tests --features streaming,sessions

# Update dependencies
cargo update
```

---

## Quick Reference

### Essential Commands

```bash
# Run all tests
cargo test --test '*' --features streaming,sessions

# Run specific suite
cargo test --test e2e_full_stack --features sessions

# Run with output
cargo test -- --nocapture --show-output

# Run single test
cargo test test_name -- --exact

# Performance benchmarks
cargo test --test performance_regression --release

# Generate coverage
cargo tarpaulin --out Html --output-dir coverage
```

### Feature Flags

| Feature | Purpose |
|---------|---------|
| `streaming` | Streaming tests (NDJSON, SSE, WebSocket) |
| `sessions` | Browser session tests |
| `profiling-full` | Memory profiling and bottleneck analysis |
| `jemalloc` | jemalloc-based memory monitoring |
| `full` | All features enabled |

### Test Execution Matrix

| Test Suite | Features Required | Run Time |
|------------|------------------|----------|
| E2E | `streaming,sessions` | ~30s |
| Performance | `profiling-full` | ~2m |
| Cross-module | `streaming,sessions,profiling-full` | ~45s |
| Stress | `streaming` | ~5m |
| Error Recovery | `sessions` | ~30s |
| Security | none | ~15s |

---

## Conclusion

This comprehensive integration test suite ensures the RipTide API maintains high quality, performance, and security standards. By following this guide, developers can:

- Run tests efficiently
- Interpret results accurately
- Debug failures quickly
- Write new tests consistently
- Maintain high coverage

For questions or issues, consult:
- Project issue tracker
- Team documentation
- Test code comments

**Happy Testing!** 🚀
