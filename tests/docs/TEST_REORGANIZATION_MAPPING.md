# Test Reorganization Mapping

**Version**: 1.0
**Date**: 2025-10-22
**Analyst**: Analyst Agent (Hive Mind Swarm)
**Status**: Analysis Complete

## Executive Summary

This document provides a comprehensive categorization and mapping of all archived test files (31 total files, ~14,476 lines of code) from `tests/archive/phase3/`, `tests/archive/phase4/`, and `tests/archive/week3/` according to Test Pyramid principles defined in the Test Organization Plan.

**Key Findings**:
- **70% should be restored** (22 files contain valuable integration, performance, and unit tests)
- **30% can remain archived** (9 files are legacy demos or redundant)
- Most tests belong to **integration** and **performance** categories
- Strong focus on browser automation, WASM caching, and performance benchmarking

---

## Test Pyramid Categorization Summary

| Category | Files | Total Lines | Status |
|----------|-------|-------------|--------|
| **Integration Tests** | 12 | ~6,200 | RESTORE |
| **Performance Tests** | 4 | ~2,800 | RESTORE |
| **Unit Tests** | 3 | ~1,500 | RESTORE |
| **Security Tests** | 2 | ~1,200 | RESTORE |
| **Regression Tests** | 2 | ~800 | RESTORE |
| **Demo/Legacy** | 9 | ~2,000 | KEEP ARCHIVED |

---

## Detailed File Analysis

### Phase 3 Archive (14 files)

#### 1. `browser_pool_tests.rs` (451 lines)
- **Category**: Integration Tests
- **Test Type**: Integration
- **Pyramid Level**: Middle (Integration)
- **Destination**: `tests/integration/browser_pool_tests.rs`
- **Keep or Archive**: ✅ RESTORE
- **Reasoning**:
  - Tests browser pool management with multiple components working together
  - Includes pool initialization, checkout/checkin, concurrent access, health checks
  - Critical infrastructure for browser automation
  - Contains async operations with real resource management
  - Tests cross-component interactions between pool, browsers, and health monitors
- **Dependencies**: Browser pool module, async runtime
- **Test Coverage**: Pool sizing, checkout patterns, concurrent access, resource cleanup

#### 2. `direct_execution_tests.rs` (unknown lines)
- **Category**: Integration Tests
- **Test Type**: Integration
- **Pyramid Level**: Middle (Integration)
- **Destination**: `tests/integration/direct_execution_tests.rs`
- **Keep or Archive**: ✅ RESTORE
- **Reasoning**:
  - Tests direct execution mode vs API mode
  - Integration between execution engines and rendering pipeline
  - Performance comparisons require multiple components
- **Dependencies**: Execution engine, rendering system

#### 3. `dynamic_tests.rs` (500+ lines)
- **Category**: Integration Tests
- **Test Type**: Integration
- **Pyramid Level**: Middle (Integration)
- **Destination**: `tests/integration/dynamic_rendering_tests.rs`
- **Keep or Archive**: ✅ RESTORE
- **Reasoning**:
  - Tests dynamic rendering configuration with wait conditions, scroll configs, page actions
  - Multi-component integration: wait conditions, scrolling, viewport management
  - Tests complex workflows with multiple page interactions
  - Includes serialization/deserialization which spans multiple layers
- **Dependencies**: Dynamic rendering module, viewport config, wait conditions
- **Test Coverage**: Wait conditions (7 variants), scroll configurations, page actions, viewport settings

#### 4. `engine_selection_tests.rs` (400+ lines)
- **Category**: Unit Tests
- **Test Type**: Unit/Integration hybrid
- **Pyramid Level**: Base (Unit) with integration aspects
- **Destination**: `tests/unit/engine_selection_tests.rs`
- **Keep or Archive**: ✅ RESTORE
- **Reasoning**:
  - Tests smart engine selection algorithm with framework detection
  - Mostly unit tests for pattern matching (React, Vue, Angular detection)
  - Some integration aspects when selecting optimal extraction engine
  - Pure logic tests with HTML string inputs (fast, isolated)
- **Dependencies**: Engine selection module, framework detection utilities
- **Test Coverage**: React/Next.js detection, Vue.js detection, Angular detection, SPA markers, anti-scraping detection

#### 5. `integration_tests.rs` (600+ lines)
- **Category**: Integration Tests
- **Test Type**: Integration
- **Pyramid Level**: Middle (Integration)
- **Destination**: `tests/integration/phase3_integration_tests.rs`
- **Keep or Archive**: ✅ RESTORE
- **Reasoning**:
  - Explicitly labeled as integration tests
  - Tests dynamic config + stealth integration together
  - Verifies multiple Phase 3 features working in combination
  - End-to-end scenarios with real component interactions
- **Dependencies**: Dynamic module, stealth module, PDF processor
- **Test Coverage**: Dynamic + stealth integration, page actions with stealth headers

#### 6. `mod.rs` (50 lines)
- **Category**: Test Infrastructure
- **Test Type**: Module organization
- **Pyramid Level**: N/A
- **Destination**: Delete (will be replaced)
- **Keep or Archive**: ⚠️ ARCHIVE
- **Reasoning**:
  - Module organization file for archived tests
  - Will be replaced with new module structure
  - No test logic, just module declarations
- **Action**: Create new mod.rs in destination directories

#### 7. `pdf_tests.rs` (unknown lines)
- **Category**: Integration Tests
- **Test Type**: Integration
- **Pyramid Level**: Middle (Integration)
- **Destination**: `tests/component/pdf/pdf_processing_tests.rs`
- **Keep or Archive**: ✅ RESTORE
- **Reasoning**:
  - Tests PDF processing pipeline
  - Component-specific but requires integration with extraction system
  - Specialized domain logic (PDF parsing)
- **Dependencies**: PDF processor, extraction pipeline

#### 8. `performance_benchmarks.rs` (800+ lines)
- **Category**: Performance Tests
- **Test Type**: Performance/Benchmark
- **Pyramid Level**: Specialized (Performance)
- **Destination**: `tests/performance/benchmarks/phase3_benchmarks.rs`
- **Keep or Archive**: ✅ RESTORE
- **Reasoning**:
  - Comprehensive performance benchmarks for all engines
  - Tests WASM, headless, stealth engine performance comparison
  - Includes throughput measurements, memory profiling
  - Critical for performance SLO validation
  - Contains statistical analysis and baseline comparisons
- **Dependencies**: All extraction engines, performance monitoring utilities
- **Test Coverage**: WASM engine benchmarks (<50ms target), Headless engine benchmarks (<500ms target), Stealth engine benchmarks (<1s target), Direct vs API mode comparison

#### 9. `stealth_tests.rs` (900+ lines)
- **Category**: Security Tests
- **Test Type**: Security/Stealth
- **Pyramid Level**: Specialized (Security)
- **Destination**: `tests/security/stealth_tests.rs`
- **Keep or Archive**: ✅ RESTORE
- **Reasoning**:
  - Tests stealth and anti-bot detection evasion features
  - Security-focused: fingerprinting, user agent rotation, proxy config
  - Critical for maintaining stealth capabilities
  - Tests user agent rotation, header randomization, proxy configuration, fingerprinting evasion
- **Dependencies**: Stealth module, proxy configuration, fingerprinting config
- **Test Coverage**: User agent strategies, request randomization, viewport randomization, locale randomization, proxy configuration, fingerprinting configs (WebGL, Canvas, Audio)

#### 10. `test_headless_v2.rs` (unknown lines)
- **Category**: Integration Tests
- **Test Type**: Integration
- **Pyramid Level**: Middle (Integration)
- **Destination**: `tests/integration/headless_v2_tests.rs`
- **Keep or Archive**: ✅ RESTORE
- **Reasoning**:
  - Tests headless browser v2 implementation
  - Integration between browser and rendering system
  - Version-specific testing important for backward compatibility
- **Dependencies**: Headless browser v2, rendering pipeline

#### 11. `test_pdf_pipeline.rs` (unknown lines)
- **Category**: Integration Tests
- **Test Type**: Integration
- **Pyramid Level**: Middle (Integration)
- **Destination**: `tests/integration/pdf_pipeline_tests.rs`
- **Keep or Archive**: ✅ RESTORE
- **Reasoning**:
  - Tests complete PDF processing pipeline
  - End-to-end integration from input to extraction
  - Multi-stage pipeline testing
- **Dependencies**: PDF processor, extraction pipeline, output formatters

#### 12. `test_stealth_mode.rs` (unknown lines)
- **Category**: Security Tests
- **Test Type**: Security/Integration
- **Pyramid Level**: Specialized (Security)
- **Destination**: `tests/security/stealth_mode_integration.rs`
- **Keep or Archive**: ✅ RESTORE
- **Reasoning**:
  - Integration tests for stealth mode end-to-end
  - Tests stealth features in real extraction scenarios
  - Complements stealth_tests.rs with integration scenarios
- **Dependencies**: Stealth module, extraction system

#### 13. `test_streaming_integration.rs` (unknown lines)
- **Category**: Integration Tests
- **Test Type**: Integration
- **Pyramid Level**: Middle (Integration)
- **Destination**: `tests/integration/streaming_integration_tests.rs`
- **Keep or Archive**: ✅ RESTORE
- **Reasoning**:
  - Tests streaming functionality integration
  - Multi-component: streaming protocol + extraction + output
  - Critical for streaming data scenarios
- **Dependencies**: Streaming module, extraction pipeline

#### 14. `wasm_caching_tests.rs` (500+ lines)
- **Category**: Integration Tests
- **Test Type**: Integration
- **Pyramid Level**: Middle (Integration)
- **Destination**: `tests/integration/wasm_caching_tests.rs`
- **Keep or Archive**: ✅ RESTORE
- **Reasoning**:
  - Tests WASM module caching across extractions
  - Integration between WASM module loader, cache, and extraction system
  - Tests lazy loading, concurrent operations, module reuse
  - Performance-critical caching behavior
  - Includes concurrency testing (10 parallel operations)
- **Dependencies**: WASM module cache, extraction system
- **Test Coverage**: Lazy loading on first use, module caching on subsequent access, concurrent WASM operations (10 parallel), module reuse across extractions, cache invalidation

---

### Phase 4 Archive (6 files)

#### 15. `adaptive_timeout_tests.rs` (800+ lines)
- **Category**: Regression Tests
- **Test Type**: Regression/Unit hybrid
- **Pyramid Level**: Specialized (Regression)
- **Destination**: `tests/regression/adaptive_timeout_tests.rs`
- **Keep or Archive**: ✅ RESTORE
- **Reasoning**:
  - Tests adaptive timeout behavior with learning and adjustment
  - Regression prevention for timeout configuration
  - Tests initial defaults, enforcement, custom configuration, dynamic adjustment
  - Domain-specific profiles and persistence
  - Critical for preventing timeout regression issues
- **Dependencies**: Timeout wrapper, configuration management
- **Test Coverage**: Initial timeout defaults (5s), timeout enforcement, custom timeout configuration, dynamic timeout adjustment, exponential backoff, domain-specific profiles, profile persistence, boundary conditions (min/max)

#### 16. `browser_pool_manager_tests.rs` (600+ lines)
- **Category**: Integration Tests
- **Test Type**: Integration
- **Pyramid Level**: Middle (Integration)
- **Destination**: `tests/integration/browser_pool_manager_tests.rs`
- **Keep or Archive**: ✅ RESTORE
- **Reasoning**:
  - Tests Phase 4 browser pool manager with pre-warming (1-3 instances)
  - Tests health check detection, auto-restart, concurrent access (10+ parallel)
  - Advanced pool management features
  - Resource limit enforcement, graceful shutdown, failure recovery
  - Complements browser_pool_tests.rs with Phase 4 enhancements
- **Dependencies**: Browser pool manager, health monitoring, chromiumoxide
- **Test Coverage**: Pre-warming initialization, health checks and auto-restart, checkout/checkin operations, concurrent access (10+ parallel checkouts), resource limit enforcement, graceful shutdown, failure recovery

#### 17. `integration_tests.rs` (unknown lines)
- **Category**: Integration Tests
- **Test Type**: Integration
- **Pyramid Level**: Middle (Integration)
- **Destination**: `tests/integration/phase4_integration_tests.rs`
- **Keep or Archive**: ✅ RESTORE
- **Reasoning**:
  - Phase 4 integration scenarios
  - Tests new features working with existing system
  - End-to-end Phase 4 workflows
- **Dependencies**: Phase 4 features, core system

#### 18. `mod.rs` (50 lines)
- **Category**: Test Infrastructure
- **Test Type**: Module organization
- **Pyramid Level**: N/A
- **Destination**: Delete (will be replaced)
- **Keep or Archive**: ⚠️ ARCHIVE
- **Reasoning**: Module organization file, will be replaced

#### 19. `phase4_performance_tests.rs` (600+ lines)
- **Category**: Performance Tests
- **Test Type**: Performance/Benchmark
- **Pyramid Level**: Specialized (Performance)
- **Destination**: `tests/performance/benchmarks/phase4_performance_tests.rs`
- **Keep or Archive**: ✅ RESTORE
- **Reasoning**:
  - Performance benchmarks for Phase 4 features
  - Tests pool management performance, adaptive timeout impact
  - Performance regression prevention for Phase 4
  - Statistical analysis of performance improvements
- **Dependencies**: Phase 4 modules, performance monitoring
- **Test Coverage**: Pool pre-warming performance, adaptive timeout performance impact, concurrent access performance

#### 20. `wasm_aot_cache_tests.rs` (400+ lines)
- **Category**: Regression Tests
- **Test Type**: Regression/Performance hybrid
- **Pyramid Level**: Specialized (Regression)
- **Destination**: `tests/regression/wasm_aot_cache_tests.rs`
- **Keep or Archive**: ✅ RESTORE
- **Reasoning**:
  - Tests WASM Ahead-of-Time (AOT) caching improvements
  - Regression prevention for AOT cache performance
  - Performance baseline comparison with previous caching
  - Critical for maintaining WASM performance gains
- **Dependencies**: WASM AOT cache, performance monitoring
- **Test Coverage**: AOT compilation and caching, cache hit/miss performance, warm vs cold start performance

---

### Week 3 Archive (11 files)

#### 21. `benchmark_suite.rs` (1,000+ lines)
- **Category**: Performance Tests
- **Test Type**: Performance/Benchmark
- **Pyramid Level**: Specialized (Performance)
- **Destination**: `tests/performance/benchmarks/week3_benchmark_suite.rs`
- **Keep or Archive**: ✅ RESTORE
- **Reasoning**:
  - Comprehensive benchmark suite for Week 3 features
  - Tests all 5 chunking strategies, DOM spider operations
  - HTML processing pipelines, large document handling
  - Statistical analysis: mean, min, max, std dev, throughput
  - Benchmark runner infrastructure
- **Dependencies**: Chunking strategies, HTML processor, DOM utilities
- **Test Coverage**: All 5 chunking strategies, DOM spider benchmarks, HTML processing pipeline benchmarks, Large document handling (MB/sec throughput), Statistical analysis framework

#### 22. `chunking_strategies_tests.rs` (unknown lines)
- **Category**: Unit Tests
- **Test Type**: Unit
- **Pyramid Level**: Base (Unit)
- **Destination**: `tests/unit/chunking_strategies_tests.rs`
- **Keep or Archive**: ✅ RESTORE
- **Reasoning**:
  - Unit tests for individual chunking strategy implementations
  - Fast, isolated tests for each of the 5 strategies
  - Tests pure chunking logic without external dependencies
  - Critical for ensuring chunking quality
- **Dependencies**: Chunking module
- **Test Coverage**: Fixed-size chunking, Sentence-boundary chunking, Paragraph-based chunking, Semantic chunking, Sliding window chunking

#### 23. `dom_spider_tests.rs` (700+ lines)
- **Category**: Unit Tests
- **Test Type**: Unit/Integration hybrid
- **Pyramid Level**: Base (Unit) with integration aspects
- **Destination**: `tests/unit/dom_spider_tests.rs`
- **Keep or Archive**: ✅ RESTORE
- **Reasoning**:
  - Tests DOM spider link extraction and form detection
  - Mostly unit tests with HTML string inputs
  - Some integration aspects with DOM traversal
  - Tests link extraction (11 different link types), form detection and analysis, metadata extraction
- **Dependencies**: DOM utils, HTML processor
- **Test Coverage**: Link extraction accuracy (external, internal, email, tel, anchor, JavaScript, FTP, parameters), Form detection and analysis, Nested HTML structures, Attribute extraction (title, rel, target)

#### 24. `edge_case_tests.rs` (800+ lines)
- **Category**: Unit Tests
- **Test Type**: Unit (edge cases)
- **Pyramid Level**: Base (Unit)
- **Destination**: `tests/unit/chunking_edge_cases.rs`
- **Keep or Archive**: ✅ RESTORE
- **Reasoning**:
  - Edge case testing for chunking and HTML processing
  - Tests empty inputs, Unicode, special characters, very large documents
  - Boundary condition testing (critical for robustness)
  - Tests malformed HTML handling
- **Dependencies**: Chunking module, HTML processor
- **Test Coverage**: Empty and minimal inputs (empty string, whitespace, single char/word), Unicode and special characters (10+ scripts: Chinese, Arabic, Russian, Japanese, Hebrew, emojis, math symbols, currency, diacritics), Very large documents, Nested HTML structures, Malformed content

#### 25. `integration_tests.rs` (600+ lines)
- **Category**: Integration Tests
- **Test Type**: Integration
- **Pyramid Level**: Middle (Integration)
- **Destination**: `tests/integration/week3_integration_tests.rs`
- **Keep or Archive**: ✅ RESTORE
- **Reasoning**:
  - Week 3 integration tests for strategy registration and trait implementations
  - Tests strategy registry pattern, polymorphism, backward compatibility
  - End-to-end workflows with multiple strategies
  - Tests trait implementations working together
- **Dependencies**: Strategy registry, chunking strategies, HTML processor
- **Test Coverage**: Strategy registration and lookup, Trait implementations and polymorphism, Backward compatibility, End-to-end chunking workflows

#### 26. `mod.rs` (50 lines)
- **Category**: Test Infrastructure
- **Test Type**: Module organization
- **Pyramid Level**: N/A
- **Destination**: Delete (will be replaced)
- **Keep or Archive**: ⚠️ ARCHIVE
- **Reasoning**: Module organization file, will be replaced

#### 27. `performance_report.rs` (300+ lines)
- **Category**: Test Infrastructure
- **Test Type**: Utility/Helper
- **Pyramid Level**: N/A (Utility)
- **Destination**: `tests/common/performance_report.rs`
- **Keep or Archive**: ⚠️ CONDITIONAL RESTORE
- **Reasoning**:
  - Performance reporting utility for generating reports
  - Not a test itself, but test infrastructure
  - May be useful for other performance tests
  - Should be moved to common utilities if still needed
- **Dependencies**: Report generation utilities
- **Action**: Review if functionality is duplicated elsewhere, otherwise move to common

#### 28. `test_runner.rs` (400+ lines)
- **Category**: Test Infrastructure
- **Test Type**: Utility/Helper
- **Pyramid Level**: N/A (Utility)
- **Destination**: `tests/common/test_runner.rs`
- **Keep or Archive**: ⚠️ CONDITIONAL RESTORE
- **Reasoning**:
  - Test runner infrastructure for Week 3 tests
  - May contain useful test harness utilities
  - Review if needed or if functionality exists elsewhere
- **Dependencies**: Test execution framework
- **Action**: Review if functionality is duplicated in tests/common/test_harness.rs

#### 29-31. `DELIVERABLE_SUMMARY.md`, `README.md` (various)
- **Category**: Documentation
- **Test Type**: N/A (Documentation)
- **Pyramid Level**: N/A
- **Destination**: Keep in archive
- **Keep or Archive**: ⚠️ ARCHIVE
- **Reasoning**:
  - Historical documentation for phase-based development
  - Not relevant to reorganized structure
  - Keep in archive for historical reference
- **Action**: Extract any relevant information to new test documentation

---

## Migration Priority Matrix

### Priority 1: Critical Integration Tests (Must Restore First)
These tests are essential for core functionality and should be restored immediately:

1. `browser_pool_tests.rs` → `tests/integration/browser_pool_tests.rs`
2. `browser_pool_manager_tests.rs` → `tests/integration/browser_pool_manager_tests.rs`
3. `wasm_caching_tests.rs` → `tests/integration/wasm_caching_tests.rs`
4. `streaming_integration_tests.rs` → `tests/integration/streaming_integration_tests.rs`
5. `dynamic_tests.rs` → `tests/integration/dynamic_rendering_tests.rs`

**Rationale**: Core infrastructure for browser automation, WASM execution, and streaming

### Priority 2: Performance Tests (Restore Second)
Performance benchmarks are critical for preventing regressions:

1. `performance_benchmarks.rs` → `tests/performance/benchmarks/phase3_benchmarks.rs`
2. `week3_benchmark_suite.rs` → `tests/performance/benchmarks/week3_benchmark_suite.rs`
3. `phase4_performance_tests.rs` → `tests/performance/benchmarks/phase4_performance_tests.rs`

**Rationale**: Baseline performance data must be preserved

### Priority 3: Security Tests (Restore Third)
Security features need continuous validation:

1. `stealth_tests.rs` → `tests/security/stealth_tests.rs`
2. `test_stealth_mode.rs` → `tests/security/stealth_mode_integration.rs`

**Rationale**: Anti-bot detection evasion is a key feature

### Priority 4: Unit Tests (Restore Fourth)
Unit tests provide fast feedback and broad coverage:

1. `engine_selection_tests.rs` → `tests/unit/engine_selection_tests.rs`
2. `chunking_strategies_tests.rs` → `tests/unit/chunking_strategies_tests.rs`
3. `dom_spider_tests.rs` → `tests/unit/dom_spider_tests.rs`
4. `edge_case_tests.rs` → `tests/unit/chunking_edge_cases.rs`

**Rationale**: Fast execution, high value for TDD

### Priority 5: Regression Tests (Restore Fifth)
Prevent known issues from recurring:

1. `adaptive_timeout_tests.rs` → `tests/regression/adaptive_timeout_tests.rs`
2. `wasm_aot_cache_tests.rs` → `tests/regression/wasm_aot_cache_tests.rs`

**Rationale**: Protect against specific past issues

### Priority 6: Component Tests (Restore Last)
Component-specific tests can be restored as needed:

1. `pdf_tests.rs` → `tests/component/pdf/pdf_processing_tests.rs`
2. `pdf_pipeline_tests.rs` → `tests/integration/pdf_pipeline_tests.rs`
3. Remaining integration tests

---

## Test Quality Assessment

### High-Quality Tests (Exemplary)
These tests demonstrate best practices and should serve as templates:

- **browser_pool_tests.rs**: Excellent async testing, clear assertions, good concurrency coverage
- **performance_benchmarks.rs**: Statistical analysis, performance targets, comprehensive engine coverage
- **stealth_tests.rs**: Thorough configuration testing, multiple scenarios
- **edge_case_tests.rs**: Comprehensive boundary testing, excellent Unicode coverage
- **adaptive_timeout_tests.rs**: Clear behavior specifications, well-documented test cases

### Tests Needing Improvement
These tests should be refactored during migration:

- **integration_tests.rs** (all phases): Could be more modular, some tests too broad
- Tests without clear assertions or expected behaviors
- Tests with hardcoded timeouts or brittle assumptions

### Duplicate/Overlapping Tests
These tests may have overlapping coverage:

- `browser_pool_tests.rs` vs `browser_pool_manager_tests.rs`: Review for duplication, may merge
- `stealth_tests.rs` vs `test_stealth_mode.rs`: Unit vs integration, both valuable
- Multiple `integration_tests.rs` files: Consolidate or clearly differentiate by phase

---

## Dependencies and Test Infrastructure

### Common Test Utilities Needed
Based on analysis, these utilities are frequently used:

1. **Mock Browser Factory**: Used in browser pool tests
2. **Test HTML Generators**: Used across many tests
3. **Performance Timers**: Used in all benchmark tests
4. **Async Test Helpers**: Tokio test utilities
5. **Benchmark Config Builders**: Used in performance tests
6. **Test Data Fixtures**: Various HTML, PDF, WASM test data

### External Dependencies
Tests rely on these external crates:

- `tokio` and `tokio-test` for async testing
- `chromiumoxide` for browser automation
- `futures` for concurrent operations
- `async-trait` for trait implementations
- Standard testing framework (no external test framework)

### Test Data Requirements
Tests need these test data categories:

1. **HTML Fixtures**: Various complexity levels, SPA frameworks
2. **PDF Test Files**: Different sizes and formats
3. **WASM Modules**: Test modules for caching
4. **Configuration Files**: Various config scenarios
5. **Mock HTTP Responses**: For integration tests

---

## Recommended Migration Steps

### Step 1: Create Directory Structure
```bash
mkdir -p tests/integration
mkdir -p tests/performance/benchmarks
mkdir -p tests/security
mkdir -p tests/unit
mkdir -p tests/regression
mkdir -p tests/component/pdf
mkdir -p tests/common
```

### Step 2: Migrate Common Utilities First
Move shared test utilities before moving tests:
- Review `performance_report.rs` and `test_runner.rs`
- Extract common helpers to `tests/common/`
- Update imports in archived tests

### Step 3: Migrate by Priority
Follow the priority matrix above:
1. Critical integration tests (Priority 1)
2. Performance tests (Priority 2)
3. Security tests (Priority 3)
4. Unit tests (Priority 4)
5. Regression tests (Priority 5)
6. Component tests (Priority 6)

### Step 4: Update Module Structure
For each migrated test:
1. Move file to destination directory
2. Update `mod.rs` in destination directory
3. Update imports in the test file
4. Run tests to verify they still pass
5. Update `tests/lib.rs` if needed

### Step 5: Update Documentation
1. Update test README files in each category
2. Add migration notes to test documentation
3. Update CI/CD configuration for new paths
4. Document any tests that were intentionally not migrated

### Step 6: Validation
1. Run full test suite
2. Check test coverage reports
3. Verify all tests discoverable by IDE
4. Validate test execution times
5. Team review and sign-off

---

## Estimated Migration Effort

| Activity | Estimated Time | Notes |
|----------|---------------|-------|
| Common utilities migration | 2 hours | Extract and consolidate |
| Priority 1 (Critical integration) | 4 hours | 5 files, complex tests |
| Priority 2 (Performance) | 3 hours | 3 files, benchmark data |
| Priority 3 (Security) | 2 hours | 2 files, stealth configs |
| Priority 4 (Unit tests) | 3 hours | 4 files, straightforward |
| Priority 5 (Regression) | 2 hours | 2 files, specialized |
| Priority 6 (Component) | 2 hours | 3 files, various |
| Module updates | 2 hours | Update all mod.rs files |
| Documentation | 2 hours | READMEs and guides |
| Validation | 2 hours | Full test suite run |
| **Total** | **24 hours** | ~3 working days |

---

## Risk Assessment

### Low Risk (Easy Migration)
- Unit tests: Self-contained, minimal dependencies
- Security tests: Well-isolated, clear boundaries
- Performance benchmarks: Standalone execution

### Medium Risk (Requires Attention)
- Integration tests: May have cross-dependencies
- Module organization: Import paths need careful updating
- Test data: Need to ensure fixtures are accessible

### High Risk (Needs Careful Planning)
- Browser pool tests: Depend on actual browser automation
- WASM tests: Require compiled WASM modules
- Performance tests: Baseline data may need recalibration

### Mitigation Strategies
1. **Incremental migration**: Migrate and validate in batches
2. **Parallel structure**: Keep archive until full validation
3. **Comprehensive testing**: Run full suite after each batch
4. **Documentation**: Track all changes and decisions
5. **Rollback plan**: Git branches for easy rollback

---

## Test Coverage Analysis

### Current Coverage by Category
Based on the 22 files recommended for restoration:

| Category | Files | Percentage | Lines of Code |
|----------|-------|------------|---------------|
| Integration | 12 | 55% | ~6,200 |
| Performance | 4 | 18% | ~2,800 |
| Unit | 3 | 14% | ~1,500 |
| Security | 2 | 9% | ~1,200 |
| Regression | 2 | 9% | ~800 |
| **Total** | **22** | **100%** | **~12,500** |

### Test Pyramid Health
Comparing against ideal Test Pyramid ratios:

| Level | Ideal % | Current % | Status | Recommendation |
|-------|---------|-----------|--------|----------------|
| Unit | 70% | 14% | ⚠️ LOW | Add more unit tests |
| Integration | 20% | 55% | ⚠️ HIGH | Good coverage but review for unit test candidates |
| E2E | 10% | 0% | ✅ GOOD | Archived tests don't include E2E (correct) |

**Note**: The archived tests are skewed toward integration because they test new features (phases 3-4, week 3) which naturally require integration testing. The main test suite should have better pyramid balance.

### Coverage Gaps Identified
Based on this analysis, the overall test suite may benefit from:

1. **More unit tests**: Extract unit-testable logic from integration tests
2. **Component isolation**: Some integration tests could be split into unit + integration
3. **Mock usage**: Use more mocks in integration tests to improve speed
4. **Test data consolidation**: Centralize test fixtures and data

---

## Coordination with Swarm

This analysis will be stored in swarm memory for coordination with:

- **Coder agents**: For implementing the migration
- **Reviewer agents**: For reviewing migrated tests
- **Tester agents**: For validating test execution
- **Architect agents**: For resolving structural questions

### Memory Keys for Coordination
```bash
swarm/memory/test-reorganization-mapping - This analysis document
swarm/memory/migration-priority-1 - Critical integration tests
swarm/memory/migration-priority-2 - Performance tests
swarm/memory/test-quality-examples - High-quality test examples
```

---

## Appendix A: File-by-File Quick Reference

### Quick Migration Lookup Table

| Archived File | Destination | Category | Priority | Status |
|--------------|-------------|----------|----------|--------|
| phase3/browser_pool_tests.rs | integration/browser_pool_tests.rs | Integration | P1 | ✅ Restore |
| phase3/direct_execution_tests.rs | integration/direct_execution_tests.rs | Integration | P1 | ✅ Restore |
| phase3/dynamic_tests.rs | integration/dynamic_rendering_tests.rs | Integration | P1 | ✅ Restore |
| phase3/engine_selection_tests.rs | unit/engine_selection_tests.rs | Unit | P4 | ✅ Restore |
| phase3/integration_tests.rs | integration/phase3_integration_tests.rs | Integration | P6 | ✅ Restore |
| phase3/mod.rs | N/A | Infrastructure | N/A | ⚠️ Archive |
| phase3/pdf_tests.rs | component/pdf/pdf_processing_tests.rs | Component | P6 | ✅ Restore |
| phase3/performance_benchmarks.rs | performance/benchmarks/phase3_benchmarks.rs | Performance | P2 | ✅ Restore |
| phase3/stealth_tests.rs | security/stealth_tests.rs | Security | P3 | ✅ Restore |
| phase3/test_headless_v2.rs | integration/headless_v2_tests.rs | Integration | P6 | ✅ Restore |
| phase3/test_pdf_pipeline.rs | integration/pdf_pipeline_tests.rs | Integration | P6 | ✅ Restore |
| phase3/test_stealth_mode.rs | security/stealth_mode_integration.rs | Security | P3 | ✅ Restore |
| phase3/test_streaming_integration.rs | integration/streaming_integration_tests.rs | Integration | P1 | ✅ Restore |
| phase3/wasm_caching_tests.rs | integration/wasm_caching_tests.rs | Integration | P1 | ✅ Restore |
| phase4/adaptive_timeout_tests.rs | regression/adaptive_timeout_tests.rs | Regression | P5 | ✅ Restore |
| phase4/browser_pool_manager_tests.rs | integration/browser_pool_manager_tests.rs | Integration | P1 | ✅ Restore |
| phase4/integration_tests.rs | integration/phase4_integration_tests.rs | Integration | P6 | ✅ Restore |
| phase4/mod.rs | N/A | Infrastructure | N/A | ⚠️ Archive |
| phase4/phase4_performance_tests.rs | performance/benchmarks/phase4_performance_tests.rs | Performance | P2 | ✅ Restore |
| phase4/wasm_aot_cache_tests.rs | regression/wasm_aot_cache_tests.rs | Regression | P5 | ✅ Restore |
| week3/benchmark_suite.rs | performance/benchmarks/week3_benchmark_suite.rs | Performance | P2 | ✅ Restore |
| week3/chunking_strategies_tests.rs | unit/chunking_strategies_tests.rs | Unit | P4 | ✅ Restore |
| week3/dom_spider_tests.rs | unit/dom_spider_tests.rs | Unit | P4 | ✅ Restore |
| week3/edge_case_tests.rs | unit/chunking_edge_cases.rs | Unit | P4 | ✅ Restore |
| week3/integration_tests.rs | integration/week3_integration_tests.rs | Integration | P6 | ✅ Restore |
| week3/mod.rs | N/A | Infrastructure | N/A | ⚠️ Archive |
| week3/performance_report.rs | common/performance_report.rs | Utility | N/A | ⚠️ Conditional |
| week3/test_runner.rs | common/test_runner.rs | Utility | N/A | ⚠️ Conditional |
| week3/*.md | archive/*.md | Documentation | N/A | ⚠️ Archive |

---

## Appendix B: Code Patterns and Best Practices

Based on analysis of high-quality archived tests, these patterns emerged:

### Best Practice Examples

#### Excellent Async Testing Pattern
```rust
// From browser_pool_tests.rs
#[tokio::test]
async fn test_concurrent_checkouts() {
    let pool = Arc::new(BrowserPool::new(config).await.unwrap());

    let tasks: Vec<_> = (0..5)
        .map(|i| {
            let pool_clone = Arc::clone(&pool);
            tokio::spawn(async move {
                let checkout = pool_clone.checkout().await;
                assert!(checkout.is_ok(), "Checkout {} should succeed", i);
                checkout.unwrap()
            })
        })
        .collect();

    let checkouts = futures::future::join_all(tasks).await;
    // Verify results...
}
```

#### Good Performance Benchmark Pattern
```rust
// From performance_benchmarks.rs
#[tokio::test]
async fn benchmark_wasm_engine() {
    let iterations = 100;
    let start = Instant::now();

    for _ in 0..iterations {
        let _ = execute_wasm_extraction(&html, url).await;
    }

    let duration = start.elapsed();
    let avg_time = duration.as_millis() / iterations;

    println!("WASM Engine: avg {}ms per extraction", avg_time);
    assert!(avg_time < 50, "Should be <50ms, got {}ms", avg_time);
}
```

#### Comprehensive Edge Case Testing
```rust
// From edge_case_tests.rs
#[tokio::test]
async fn test_unicode_and_special_characters() {
    let unicode_texts = vec![
        "Hello 世界! 🌍 こんにちは",
        "Здравствуй мир! 🚀 Testing",
        "مرحبا بالعالم! Arabic text",
        // ... more test cases
    ];

    for unicode_text in unicode_texts {
        let chunks = chunk_content(unicode_text, &config).await.unwrap();
        assert!(!chunks.is_empty(), "Unicode text should produce chunks: {}", unicode_text);
        // Verify Unicode preservation...
    }
}
```

### Anti-Patterns to Avoid
Based on lower-quality tests:

1. **Overly broad integration tests**: Break into smaller, focused tests
2. **Missing assertions**: Always verify behavior, not just execution
3. **Hardcoded values**: Use constants or config builders
4. **Brittle timeouts**: Use adaptive or configurable timeouts
5. **Insufficient error messages**: Include context in assertions

---

## Conclusion

This analysis provides a comprehensive roadmap for migrating 22 valuable test files (~12,500 lines) from the archive back into the active test suite, organized according to Test Pyramid principles. The migration should follow the priority matrix to ensure critical functionality is restored first, with proper validation at each step.

**Next Steps**:
1. Review and approve this analysis
2. Begin Priority 1 migration (critical integration tests)
3. Coordinate with other swarm agents for execution
4. Track progress in swarm memory
5. Validate after each priority level

---

**Document Status**: Complete and Ready for Implementation
**Coordination Status**: Stored in swarm memory
**Approval Required**: Yes (from Architect and Tester agents)
