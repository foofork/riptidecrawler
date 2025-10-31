# Phase 5: Feature Flag and Fallback Testing Guide

## Overview

Phase 5 implements comprehensive test coverage for the three-tier fallback strategy in the WASM Optional Plan:

1. **Compile-time Fallback**: Feature flags determine available extractors
2. **Runtime Fallback**: File availability triggers fallback
3. **Execution Fallback**: Error recovery with alternative extraction

## Test Structure

### Test File: `tests/extractor_fallback_tests.rs`

Contains three main test modules:

1. **fallback_tests**: Core fallback behavior tests
2. **integration_tests**: End-to-end pipeline tests
3. **performance_tests**: Throughput and latency tests

## Test Categories

### 1. Compile-Time Tests

Tests that verify feature flag behavior at compile time.

```bash
# Test native-only build (no wasm-extractor feature)
cargo test extractor_fallback_tests

# Test with WASM feature enabled
cargo test extractor_fallback_tests --features wasm-extractor
```

**Key Tests:**
- `test_level1_compile_time_feature_flags`: Verifies feature flag state
- `test_native_only_build`: Ensures native extraction works without WASM
- `test_wasm_feature_enabled`: WASM-specific compile-time checks

### 2. Runtime Fallback Tests

Tests that verify runtime fallback when WASM files are unavailable.

**Key Tests:**
- `test_level2_runtime_file_availability_fallback`: Missing WASM file → native
- `test_error_handling_wasm_unavailable`: Graceful handling of missing WASM

### 3. Execution Fallback Tests

Tests that verify execution-time error recovery.

**Key Tests:**
- `test_level3_execution_error_fallback`: WASM failure → native retry
- `test_three_tier_fallback_integration`: All levels working together

### 4. Confidence Scoring Tests

Tests that verify confidence scoring works in both modes.

**Key Tests:**
- `test_confidence_scoring_both_modes`: Scores work with/without WASM

### 5. Performance Tests

Tests that measure extraction performance.

**Key Tests:**
- `test_extraction_throughput`: Extractions per second
- `test_latency_distribution`: P50, P95, P99 latencies
- `test_performance_comparison`: Native vs WASM performance

### 6. Edge Case Tests

Tests that verify robust error handling.

**Key Tests:**
- `test_edge_cases`: Empty HTML, large HTML, special characters, invalid URLs

## CI/CD Integration

### Test Matrix

The CI workflow (`/.github/workflows/ci.yml`) includes three feature flag variations:

```yaml
matrix:
  test-type:
    - features-native   # Native-only (default)
    - features-wasm     # With wasm-extractor
    - features-all      # All features enabled
```

### Running Tests Locally

#### Native-Only Tests (Default)
```bash
# Build and test without WASM
cargo build --workspace
cargo test --workspace

# Run fallback tests specifically
cargo test extractor_fallback_tests -- --nocapture
```

#### WASM-Enabled Tests
```bash
# Build and test with WASM feature
cargo build --workspace --features wasm-extractor
cargo test --workspace --features wasm-extractor

# Run fallback tests with WASM
cargo test extractor_fallback_tests --features wasm-extractor -- --nocapture
```

#### All Features Tests
```bash
# Build and test with all features
cargo build --workspace --all-features
cargo test --workspace --all-features

# Run fallback tests with all features
cargo test extractor_fallback_tests --all-features -- --nocapture
```

## Test Assertions

### Success Criteria

All tests must pass in **BOTH** modes:
- ✅ Without `wasm-extractor` feature (native-only)
- ✅ With `wasm-extractor` feature (WASM + fallback)

### Feature Flag Behavior

#### Without `wasm-extractor` Feature:
- `MockExtractor::wasm_available()` returns `false`
- `MockExtractor::new()` always creates `Native` variant
- All extractions use native parser
- WASM_EXTRACTOR_PATH env var triggers warning (ignored gracefully)

#### With `wasm-extractor` Feature:
- `MockExtractor::wasm_available()` returns `true`
- `MockExtractor::new()` can create `Wasm` or `Native` variants
- Falls back to native if WASM file missing
- Falls back to native if WASM extraction fails

## Performance Expectations

### Native Parser (Default)
- **Throughput**: > 100 extractions/sec
- **Latency**: P95 < 10ms, P99 < 20ms
- **Memory**: < 50MB per extraction

### WASM Parser (When Enabled)
- **Throughput**: ~25-50 extractions/sec (2-4x slower)
- **Latency**: P95 < 40ms, P99 < 80ms
- **Memory**: < 100MB per extraction (sandboxed)

## Test Data

### Good HTML Example
```html
<html>
    <head><title>Quality Article</title></head>
    <body>
        <article>
            <h1>Main Heading</h1>
            <p>Substantial paragraph with quality content.</p>
        </article>
    </body>
</html>
```
**Expected**: Confidence > 0.7, successful extraction

### Poor HTML Example
```html
<html><body><div>minimal</div></body></html>
```
**Expected**: Confidence < 0.7, still extracts but lower quality

### Bad HTML Example
```html
<html><body><<bad>>corrupted content</body></html>
```
**Expected**: Triggers execution fallback, still succeeds

## Debugging Failed Tests

### Test Fails in Native-Only Mode

1. Check feature flags:
   ```bash
   cargo test extractor_fallback_tests -- --nocapture
   ```

2. Verify no WASM dependencies are required:
   ```bash
   cargo tree -p riptide-extraction | grep -i wasm
   ```

3. Check compile-time feature detection:
   ```rust
   assert!(!MockExtractor::wasm_available());
   ```

### Test Fails in WASM Mode

1. Verify feature is enabled:
   ```bash
   cargo test extractor_fallback_tests --features wasm-extractor -- --nocapture
   ```

2. Check fallback behavior:
   - Should create `Native` variant when WASM file missing
   - Should retry with native on execution errors

3. Verify feature flag propagation:
   ```bash
   cargo build --features wasm-extractor -v 2>&1 | grep "wasm-extractor"
   ```

## Mock Implementation Notes

The current tests use **mock implementations** (`MockExtractor`) to test the three-tier fallback logic **before** the actual `UnifiedExtractor` is implemented in Phase 2.

### Why Mocks?

- ✅ Tests can be written and verified **now**
- ✅ Validates fallback logic independently
- ✅ Provides clear specification for Phase 2 implementation
- ✅ Tests will continue to work when mocks are replaced with real implementation

### Replacing Mocks

When Phase 2's `UnifiedExtractor` is implemented:

1. Remove `MockExtractor` and `MockExtractedContent`
2. Import actual types:
   ```rust
   use riptide_extraction::UnifiedExtractor;
   use riptide_types::ExtractedContent;
   ```
3. Update test assertions to match actual API
4. All test logic remains the same!

## Coverage Goals

### Test Coverage Metrics

- **Unit Tests**: 11 tests covering all fallback levels
- **Integration Tests**: 3 end-to-end workflow tests
- **Performance Tests**: 2 throughput/latency tests
- **Total**: 16 comprehensive tests

### Feature Coverage

- ✅ Compile-time feature flags
- ✅ Runtime file availability fallback
- ✅ Execution error recovery
- ✅ Confidence scoring (both modes)
- ✅ Performance characteristics
- ✅ Edge cases and error handling
- ✅ Three-tier fallback integration

## Next Steps (Phase 2)

After tests are passing:

1. Implement `UnifiedExtractor` enum in `crates/riptide-extraction/src/unified_extractor.rs`
2. Replace mock types with actual implementations
3. Update `AppState` to use `UnifiedExtractor`
4. All tests should continue to pass!

## Troubleshooting

### Common Issues

**Issue**: Test fails with "feature not enabled"
```
Solution: Check Cargo.toml features and rebuild:
cargo clean
cargo build --features wasm-extractor
cargo test --features wasm-extractor
```

**Issue**: Tests timeout
```
Solution: Increase timeout or reduce iterations:
cargo test -- --test-threads=1
```

**Issue**: Flaky test results
```
Solution: Run tests sequentially:
cargo test extractor_fallback_tests -- --test-threads=1 --nocapture
```

## References

- [WASM Optional Comprehensive Plan](../WASM_OPTIONAL_COMPREHENSIVE_PLAN.md)
- [Phase 5 Specification](../WASM_OPTIONAL_COMPREHENSIVE_PLAN.md#phase-5-tests-1-hour)
- [CI Workflow](../../.github/workflows/ci.yml)
