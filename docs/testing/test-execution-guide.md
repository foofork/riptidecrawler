# Hexser Integration Testing - Quick Execution Guide

**Last Updated**: 2025-10-12
**Related Document**: [hexser-integration-test-strategy.md](./hexser-integration-test-strategy.md)

---

## Quick Start

### Prerequisites

```bash
# Ensure Rust toolchain is installed
rustc --version

# Install testing dependencies
cargo install cargo-tarpaulin  # Coverage
cargo install cargo-criterion  # Benchmarking
cargo install cargo-watch      # Auto-testing
```

### Run All Tests

```bash
# Run complete test suite
cargo test --workspace

# Run with coverage
cargo tarpaulin --out Html --output-dir ./coverage

# Run benchmarks
cargo bench --workspace
```

---

## Phase-by-Phase Execution

### Phase 1: Compatibility Tests

```bash
# HTML parsing compatibility
cargo test --package riptide-html hexser_html_compatibility

# CSS selector compatibility
cargo test --package riptide-html hexser_css_compatibility

# Unicode and special characters
cargo test --package riptide-html test_unicode_content
cargo test --package riptide-html test_special_characters
```

**Expected Results**:
- ✅ All HTML5, HTML4, XHTML tests pass
- ✅ Malformed HTML handled gracefully
- ✅ Unicode content preserved
- ✅ All selector types work

### Phase 2: Migration Validation

```bash
# Extraction parity tests
cargo test --package riptide-html extraction_parity_tests

# API contract validation
cargo test --package riptide-html api_contract_tests

# Link and metadata extraction
cargo test --package riptide-html test_link_extraction_parity
cargo test --package riptide-html test_metadata_extraction_parity
```

**Expected Results**:
- ✅ Extraction quality ≥85% similarity
- ✅ All API contracts maintained
- ✅ Link extraction comprehensive
- ✅ Metadata complete

### Phase 3: Performance Benchmarking

```bash
# Run performance benchmarks
cargo bench --package riptide-html --bench hexser_benchmarks

# Run stress tests
cargo test --package riptide-html stress_tests -- --test-threads=1 --nocapture

# Memory profiling
cargo test --package riptide-html test_memory_stability_long_running
```

**Expected Results**:
- ✅ Speed within 110% baseline
- ✅ Memory within 120% baseline
- ✅ No memory leaks
- ✅ Concurrent extractions work

### Phase 4: Integration Testing

```bash
# E2E workflow tests
cargo test --package riptide-html e2e_integration_tests

# Cross-module integration
cargo test --package riptide-html cross_module_integration

# API integration
cargo test --package riptide-api -- hexser
```

**Expected Results**:
- ✅ REST endpoints work
- ✅ Streaming functional
- ✅ WASM fallback works
- ✅ Cache/worker integration works

### Phase 5: Regression Testing

```bash
# Golden tests
cargo test --package riptide-html golden_tests --features golden-tests

# Behavioral regression tests
cargo test --package riptide-html behavioral_regression_tests

# Edge case tests
cargo test --package riptide-html test_no_regression_in_edge_cases
```

**Expected Results**:
- ✅ All golden tests pass
- ✅ No behavioral changes
- ✅ Edge cases handled
- ✅ Error handling unchanged

---

## Continuous Integration

### GitHub Actions Workflow

Tests run automatically on:
- Every push to main
- Pull requests
- Manual workflow dispatch

### CI Test Stages

1. **Compatibility**: Fast tests (5-10 min)
2. **Migration**: Validation tests (10-15 min)
3. **Performance**: Benchmarks (15-20 min)
4. **Integration**: E2E tests (15-20 min)
5. **Regression**: Golden tests (10-15 min)

Total CI time: ~60-80 minutes

---

## Test Reporting

### Coverage Report

```bash
# Generate HTML coverage report
cargo tarpaulin --out Html --output-dir ./coverage

# Open in browser
open ./coverage/index.html
```

**Target**: ≥80% code coverage

### Benchmark Comparison

```bash
# Save baseline
cargo bench --bench hexser_benchmarks -- --save-baseline main

# Compare with baseline
cargo bench --bench hexser_benchmarks -- --baseline main
```

### Test Summary

```bash
# Run tests with summary output
cargo test --workspace -- --format=json | tee test-results.json

# Generate summary
cargo test --workspace -- --format=pretty > test-summary.txt
```

---

## Validation Checkpoints

### Phase 1 Gate (Compatibility)
- [ ] HTML5 parsing: 100% pass
- [ ] CSS selectors: 95%+ pass
- [ ] Unicode handling: verified
- [ ] No panics on malformed HTML

**Blocker**: Extraction quality <85%

### Phase 2 Gate (Migration)
- [ ] Parity tests: ≥85% similarity
- [ ] API contracts: 100% pass
- [ ] Link extraction: complete
- [ ] Metadata: verified

**Blocker**: API contracts broken

### Phase 3 Gate (Performance)
- [ ] Speed: within 110% baseline
- [ ] Memory: within 120% baseline
- [ ] No memory leaks
- [ ] Concurrency: works

**Blocker**: Performance >120% baseline

### Phase 4 Gate (Integration)
- [ ] E2E workflows: pass
- [ ] Cross-module: verified
- [ ] Streaming: functional
- [ ] Worker pool: works

**Blocker**: Critical workflows fail

### Phase 5 Gate (Regression)
- [ ] Golden tests: 100% pass
- [ ] No behavioral changes
- [ ] Edge cases: handled
- [ ] Error handling: unchanged

**Blocker**: Golden tests fail

---

## Troubleshooting

### Common Issues

#### Tests Fail Due to Missing Fixtures

```bash
# Ensure test fixtures exist
ls tests/fixtures/
ls tests/hexser_integration/fixtures/

# If missing, generate test data
cargo test --package riptide-html generate_test_fixtures
```

#### Benchmark Results Vary

```bash
# Run with more samples
cargo bench --bench hexser_benchmarks -- --sample-size 100

# Ensure system is idle
# Close other applications
# Run benchmarks multiple times for consistency
```

#### Coverage Report Issues

```bash
# Clean and rebuild
cargo clean
cargo build

# Run coverage with verbose output
cargo tarpaulin --out Html --output-dir ./coverage --verbose
```

#### Memory Tests Timeout

```bash
# Increase timeout
cargo test test_memory_stability_long_running -- --ignored --test-threads=1

# Or run with custom timeout
RUST_TEST_TIMEOUT=600 cargo test stress_tests
```

---

## Performance Targets Quick Reference

| Test Type | Current Baseline | Hexser Target | Acceptance Threshold |
|-----------|-----------------|---------------|---------------------|
| Small HTML (1KB) | ~2ms | ≤2ms | ≤2.2ms (110%) |
| Medium HTML (50KB) | ~15ms | ≤15ms | ≤16.5ms (110%) |
| Large HTML (500KB) | ~150ms | ≤150ms | ≤165ms (110%) |
| XLarge HTML (5MB) | ~1.5s | ≤1.5s | ≤1.65s (110%) |
| Memory (5MB) | ~50MB | ≤50MB | ≤60MB (120%) |
| Simple Selector | ~0.5ms | ≤0.5ms | ≤0.55ms (110%) |
| Complex Selector | ~5ms | ≤5ms | ≤5.5ms (110%) |

---

## Contact & Support

For questions or issues with testing:
- Check: `/workspaces/eventmesh/docs/testing/hexser-integration-test-strategy.md`
- Review collective memory: `hive/testing/*` namespace
- Consult Hive Mind swarm coordination

---

**Document Version**: 1.0
**Maintenance**: Update after each test phase completion
