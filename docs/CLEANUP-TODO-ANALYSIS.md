# TODO and Disabled Code Analysis Report

**Generated:** 2025-11-06
**Context:** Post-circular dependency fix (commit 9343421)
**Status:** COMPREHENSIVE ANALYSIS COMPLETE

---

## Executive Summary

This report catalogs all TODOs, commented-out code, and temporarily disabled features resulting from the circular dependency refactoring between `riptide-api` and `riptide-facade`.

### Key Findings:
- **6 Handler Endpoints** temporarily returning errors
- **141+ TODO Comments** across the codebase
- **5 Major Facades** commented out in AppState
- **51+ Integration Tests** stubbed pending facade implementation
- **1 Critical Feature** (spider crawling) temporarily unavailable

---

## 🔴 CRITICAL: Disabled Handlers (PRIORITY 1)

### Handlers Returning "Facade temporarily unavailable"

#### 1. `/api/v1/extract` - Content Extraction Handler
- **File:** `crates/riptide-api/src/handlers/extract.rs:163-168`
- **Status:** Returns `SERVICE_UNAVAILABLE (503)`
- **Impact:** Core extraction functionality broken
- **Dependencies:** Requires `ExtractionFacade` re-integration
- **Lines of Code:** 169 lines total, handler at line 109-169

```rust
// Facade temporarily unavailable during refactoring
(
    StatusCode::SERVICE_UNAVAILABLE,
    Json("Facade temporarily unavailable during refactoring".to_string()),
)
    .into_response()
```

**Restoration Steps:**
1. Re-wire `state.extraction_facade` (currently exists in AppState line 148)
2. Call `extraction_facade.extract()` method
3. Convert facade result to `ExtractResponse`
4. Remove stub error at line 163-168

---

#### 2. `/api/v1/search` - Web Search Handler
- **File:** `crates/riptide-api/src/handlers/search.rs:93-98`
- **Status:** Returns `SERVICE_UNAVAILABLE (503)`
- **Impact:** Search functionality completely disabled
- **Dependencies:** Requires `SearchFacade` re-integration
- **Lines of Code:** 154 lines total, handler at line 72-99

```rust
// Facade temporarily unavailable during refactoring
(
    StatusCode::SERVICE_UNAVAILABLE,
    Json("Facade temporarily unavailable during refactoring".to_string()),
)
    .into_response()
```

**Restoration Steps:**
1. Re-wire `state.search_facade` (commented out in AppState)
2. Call `search_facade.search()` method
3. Convert results to `SearchResponse`
4. Remove stub at line 93-98

---

#### 3. `/api/v1/spider/crawl` - Spider Crawl Handler
- **File:** `crates/riptide-api/src/handlers/spider.rs:83-86`
- **Status:** Returns `ApiError::internal()`
- **Impact:** Deep crawling completely broken
- **Dependencies:** Requires `SpiderFacade` re-integration
- **Lines of Code:** 110 lines total, handler at line 68-87

```rust
// Facade temporarily unavailable during refactoring
Err::<Json<()>, ApiError>(ApiError::internal(
    "Facade temporarily unavailable during refactoring",
))
```

**Also Affects:**
- `/api/v1/spider/status` (line 90-98)
- `/api/v1/spider/control` (line 101-109)

**Restoration Steps:**
1. Uncomment `state.spider_facade` in AppState (line 155)
2. Wire SpiderFacade initialization in state.rs
3. Implement spider_crawl using facade
4. Remove stubs at lines 83-86, 94-97, 105-108

---

#### 4. `/api/v1/pdf/process` - PDF Processing Handler
- **File:** `crates/riptide-api/src/handlers/pdf.rs:151-154`
- **Status:** Returns `ApiError::internal()`
- **Impact:** PDF processing broken
- **Dependencies:** Requires facade or direct PDF integration
- **Lines of Code:** 642 lines total, handler at line 75-155

```rust
// Facade temporarily unavailable during refactoring
Err(ApiError::internal(
    "Facade temporarily unavailable during refactoring".to_string(),
))
```

**Also Affects:**
- `/api/v1/pdf/upload` (line 545-548) - Multipart upload handler

**Note:** `process_pdf_stream` (line 161-234) uses direct PDF integration, may still work

**Restoration Steps:**
1. Decide on facade vs direct integration approach
2. If facade: wire ExtractionFacade for PDF support
3. If direct: use existing `riptide_pdf` integration (already working in streaming)
4. Remove stubs at lines 151-154 and 545-548

---

#### 5. `/api/v1/crawl` - Spider Mode Disabled
- **File:** `crates/riptide-api/src/handlers/crawl.rs:298-301`
- **Status:** Spider mode returns `ConfigError`
- **Impact:** Spider crawling via regular crawl endpoint disabled
- **Dependencies:** Requires SpiderFacade
- **Unreachable Code:** Lines 304-396 (93 lines of commented-out spider implementation)

```rust
// Spider facade temporarily removed during refactoring
// TODO: Re-implement spider crawling after facade unification
return Err(ApiError::ConfigError {
    message: "Spider crawling temporarily unavailable during facade refactoring.".to_string(),
});
```

**Special Note:** Has `#[allow(unreachable_code)]` annotation at line 304 with 93 lines of implementation preserved for reference

**Restoration Steps:**
1. Re-enable SpiderFacade in AppState
2. Remove `return Err` at line 299-301
3. Remove `#[allow(unreachable_code)]` at line 304
4. Test preserved implementation (lines 305-396)
5. Update to use SpiderFacade methods

---

## 🟡 COMMENTED-OUT FACADES (PRIORITY 2)

### AppState Facade Fields

**File:** `crates/riptide-api/src/state.rs`

#### 1. BrowserFacade (Line 142-145)
```rust
/// Browser facade for simplified browser automation
/// Only available when using local Chrome mode (headless_url not configured)
#[cfg(feature = "browser")]
pub browser_facade: Option<Arc<riptide_facade::BrowserFacade>>,
```
- **Status:** ACTIVE (not commented)
- **Feature Gate:** `#[cfg(feature = "browser")]`
- **Dependencies:** Requires browser feature enabled

#### 2. ExtractionFacade (Line 147-148)
```rust
/// Extraction facade for content extraction with multiple strategies
pub extraction_facade: Arc<riptide_facade::facades::ExtractionFacade>,
```
- **Status:** ACTIVE (not commented)
- **Issue:** Not initialized in `AppState::new()` due to circular dependency
- **Priority:** HIGH - needed for extract handler

#### 3. ScraperFacade (Line 150-151)
```rust
/// Scraper facade for simple HTTP operations
pub scraper_facade: Arc<riptide_facade::ScraperFacade>,
```
- **Status:** ACTIVE (not commented)
- **Issue:** Not initialized in `AppState::new()` due to circular dependency
- **Priority:** MEDIUM

#### 4. SpiderFacade (Line 153-155)
```rust
/// Spider facade for web crawling operations
#[cfg(feature = "spider")]
pub spider_facade: Option<Arc<riptide_facade::facades::SpiderFacade>>,
```
- **Status:** ACTIVE (not commented)
- **Feature Gate:** `#[cfg(feature = "spider")]`
- **Issue:** Not initialized in `AppState::new()`
- **Priority:** HIGH - needed for spider handlers

#### 5. SearchFacade (Line 157-159)
```rust
/// Search facade for web search operations
#[cfg(feature = "search")]
pub search_facade: Option<Arc<riptide_facade::facades::SearchFacade>>,
```
- **Status:** ACTIVE (not commented)
- **Feature Gate:** `#[cfg(feature = "search")]`
- **Issue:** Not initialized in `AppState::new()`
- **Priority:** HIGH - needed for search handler

### Initialization Code Comments

**File:** `crates/riptide-api/src/state.rs` (around line 1150-1200)

```rust
// Initialize facade layer for simplified API access
// REMOVED: Caused circular dependency with riptide-facade
// let facade_config = riptide_facade::RiptideConfig::default()
// ...
// Initialize browser facade only if not using headless service
// let browser_facade = if let Some(url) = &config.headless_url {
// ...
// match BrowserFacade::new(facade_config.clone()).await {
// ...
```

**Lines affected:** Approximately 50+ lines of facade initialization commented out

---

## 📋 TODO COMMENTS BY CATEGORY

### Category 1: Facade Re-integration (HIGH PRIORITY)

#### Test Utils
- **File:** `crates/riptide-test-utils/src/lib.rs:10`
- **TODO:** Add mock_server module when needed
- **Priority:** LOW - test infrastructure

#### Phase 4B Integration Tests
- **File:** `crates/riptide-api/tests/phase4b_integration_tests.rs:49`
- **TODO:** Re-enable after test_router module is implemented
- **Priority:** MEDIUM - blocked by test infrastructure

#### Facade Composition Tests (14 tests)
- **File:** `crates/riptide-facade/tests/facade_composition_integration.rs`
- **Lines:** 53, 82, 105, 131, 170, 195, 229, 251
- **Status:** All stubbed with "TODO: Implement when facades are ready"
- **Count:** 8 test stubs
- **Priority:** HIGH - integration testing

#### Extractor Facade Tests (14 tests)
- **File:** `crates/riptide-facade/tests/extractor_facade_integration.rs`
- **Lines:** 13, 38, 59, 83, 115, 132, 147, 173, 202, 228, 251, 273, 294, 309
- **Status:** All stubbed with "TODO: Implement when ExtractorFacade is ready"
- **Count:** 14 test stubs
- **Priority:** HIGH

#### Browser Facade Tests (14 tests)
- **File:** `crates/riptide-facade/tests/browser_facade_integration.rs`
- **Lines:** 13, 31, 50, 68, 90, 108, 128, 147, 167, 190, 210, 235, 250, 269
- **Status:** All stubbed with "TODO: Implement when BrowserFacade is ready"
- **Count:** 14 test stubs
- **Priority:** HIGH

---

### Category 2: Feature Implementation (MEDIUM PRIORITY)

#### Strategy Composition Tests (51 tests)
**Location:** `tests/strategy-composition/`

**Files:**
1. `result_merging_tests.rs` - 7 tests (lines 15, 23, 30, 39, 46, 53, 60)
2. `best_tests.rs` - 7 tests (lines 12, 22, 29, 36, 43, 52, 59)
3. `fallback_tests.rs` - 6 tests (lines 13, 21, 28, 38, 45, 52)
4. `parallel_tests.rs` - 9 tests (lines 13, 21, 28, 35, 43, 51, 58, 65, 72)
5. `chain_tests.rs` - 8 tests (lines 13, 21, 28, 35, 43, 50, 57, 64)
6. `integration_tests.rs` - 8 tests (lines 12, 19, 26, 33, 42, 51, 60, 70)

**Status:** All use `todo!("Implement test: ...")` macros
**Priority:** MEDIUM - test coverage for future features

#### Enhanced Pipeline Tests
- **File:** `crates/riptide-api/tests/enhanced_pipeline_tests.rs`
- **Lines:** 23, 213, 221, 229
- **TODOs:**
  - Line 23: "Implement test state creation - requires full AppState initialization"
  - Line 213: "Implement end-to-end integration test"
  - Line 221: "Implement load test for 100+ RPS"
  - Line 229: "Implement 24h memory leak test"
- **Priority:** MEDIUM - performance testing

#### Python SDK
- **File:** `crates/riptide-py/src/document.rs`
- **Lines:** 141, 145
- **TODOs:**
  - Line 141: Extract title from strategies_result
  - Line 145: Calculate word_count from processed_content
- **Priority:** LOW - SDK enhancement

- **File:** `crates/riptide-py/src/riptide_class.rs`
- **Lines:** 95, 198
- **TODOs:**
  - Line 95: Use api_key for future cloud features
  - Line 198: Implement actual spider logic using riptide-spider
- **Priority:** MEDIUM - Python SDK completion

---

### Category 3: Monitoring & Profiling (LOW PRIORITY)

#### Monitoring Handlers
- **File:** `crates/riptide-api/src/handlers/monitoring.rs`
- **Lines:** 213, 240, 266
- **TODOs:**
  - Line 213: "Implement memory profiling integration"
  - Line 240: "Implement leak detection integration"
  - Line 266: "Implement allocation analysis integration"
- **Priority:** LOW - advanced monitoring features

#### Profiling Handler
- **File:** `crates/riptide-api/src/handlers/profiling.rs`
- **Lines:** 267, 568
- **Notes:** CPU profiling simplified, prompts to enable 'profiling-full' feature
- **Priority:** LOW - profiling enhancement

---

### Category 4: DTO & State Management (LOW PRIORITY)

#### DTO Conversion
- **File:** `crates/riptide-api/src/dto.rs:71`
- **TODO:** Re-implement conversion from spider::CrawlResult when spider integration is complete
- **Priority:** MEDIUM - depends on spider facade

#### State Management
- **File:** `crates/riptide-api/src/state.rs`
- **Lines:** 64, 171, 1209, 1578
- **TODOs:**
  - Line 64: Future wiring for learned extractor patterns
  - Line 171: Replace with actual PersistenceAdapter type when available
  - Lines 1209, 1578: Initialize actual persistence adapter when integrated
- **Priority:** LOW - future features

#### Strategies Handler
- **File:** `crates/riptide-api/src/handlers/strategies.rs:304`
- **TODO:** Use css_selectors, regex_patterns, llm_config when those strategies are implemented
- **Priority:** LOW - strategy enhancement

---

### Category 5: Test Infrastructure (LOW PRIORITY)

#### Memory Leak Detection Tests
- **File:** `crates/riptide-api/tests/memory_leak_detection_tests.rs`
- **Lines:** 57, 108, 148, 194, 264, 345, 410, 458
- **Note:** "Detection algorithm too conservative - needs threshold tuning"
- **Count:** 8 instances
- **Priority:** LOW - test refinement

#### Pool Tests
- **File:** `crates/riptide-pool/tests/pending_acquisitions_test.rs:8`
- **FIXME:** Update tests to use NativeInstancePool instead of AdvancedInstancePool
- **Priority:** LOW

#### Golden Tests
- **Files:** `tests/golden/golden_test_cli.rs`, `tests/regression/golden/golden_test_cli.rs`
- **Lines:** Multiple (235, 253, 257, 300, 314, 326 in each)
- **TODOs:** Implement JSON/YAML output, single test execution, benchmarks, memory tests
- **Priority:** LOW - CLI tooling

#### Phase 0 Integration
- **File:** `tests/phase0/integration/phase0_integration_tests.rs:354`
- **TODO:** `todo!()` - incomplete test
- **Priority:** LOW

---

### Category 6: Archive & Legacy Code (LOWEST PRIORITY)

#### Phase 3 Direct Execution Tests
- **File:** `tests/archive/phase3/direct_execution_tests.rs`, `tests/phase3/direct_execution_tests.rs`
- **Lines:** 329, 334, 346, 351, 356, 361, 370 (in both files)
- **TODOs:** Replace with actual initialization, engine selection, WASM execution, etc.
- **Status:** Archived/placeholder tests
- **Priority:** LOWEST - archived code

---

## 🚫 DISABLED FEATURES & ROUTES

### Feature Gates Without Implementations

#### 1. Crawl Handler Unreachable Code
- **File:** `crates/riptide-api/src/handlers/crawl.rs:304`
- **Annotation:** `#[allow(unreachable_code)]`
- **Lines:** 304-396 (93 lines)
- **Purpose:** Preserved spider implementation for future restoration
- **Status:** Completely unreachable after line 299 `return Err(...)`

### No Other Disabled Routes Found

All other `#[cfg(feature)]` gates appear to be properly implemented when features are enabled.

---

## 📊 STATISTICS SUMMARY

### By Priority

| Priority | Count | Category |
|----------|-------|----------|
| 🔴 CRITICAL | 6 | Disabled handlers |
| 🟡 HIGH | 51 | Facade integration tests |
| 🟠 MEDIUM | 55+ | Strategy tests + Python SDK |
| 🟢 LOW | 30+ | Monitoring, profiling, test utils |
| ⚪ LOWEST | 14 | Archived tests |

### By File Type

| Type | Count | Status |
|------|-------|--------|
| Handlers | 6 | Returning errors |
| Integration Tests | 51 | Stubbed with TODO |
| Unit Tests | 51 | Using `todo!()` macro |
| State Fields | 5 | Not initialized |
| TODOs | 141+ | Scattered across codebase |

### By Restoration Effort

| Effort | Count | Description |
|--------|-------|-------------|
| **Quick Win** | 3 | Re-wire existing facades in handlers |
| **Medium** | 2 | Implement facade initialization |
| **Complex** | 1 | Spider facade full integration |
| **Long-term** | 51+ | Strategy composition tests |

---

## 🎯 RECOMMENDED RESTORATION ORDER

### Phase 1: Critical Handlers (Week 1)
1. **Extract Handler** - Re-wire ExtractionFacade (1-2 hours)
2. **Search Handler** - Re-wire SearchFacade (1-2 hours)
3. **PDF Process Handler** - Use existing PDF integration or facade (2-3 hours)

**Estimated Time:** 6-8 hours
**Impact:** Restores 60% of broken functionality

### Phase 2: Spider Integration (Week 2)
1. **SpiderFacade Initialization** - Wire into AppState (4-6 hours)
2. **Spider Handler** - Implement all 3 spider endpoints (3-4 hours)
3. **Crawl Spider Mode** - Remove unreachable code guard (1-2 hours)

**Estimated Time:** 10-12 hours
**Impact:** Restores remaining 40% of broken functionality

### Phase 3: Test Coverage (Week 3-4)
1. **Facade Integration Tests** - Implement 51 facade tests (20-30 hours)
2. **Strategy Composition Tests** - Implement 51 strategy tests (20-30 hours)
3. **Enhanced Pipeline Tests** - Complete 4 performance tests (8-10 hours)

**Estimated Time:** 50-70 hours
**Impact:** Full test coverage for facades

### Phase 4: Feature Enhancements (Month 2)
1. **Python SDK** - Complete TODOs (5-8 hours)
2. **Monitoring Integration** - Wire profiling/leak detection (10-15 hours)
3. **Strategy Enhancements** - CSS/regex/LLM strategies (15-20 hours)

**Estimated Time:** 30-43 hours
**Impact:** Feature parity with pre-refactor state

---

## 🔧 TECHNICAL DETAILS

### Circular Dependency Resolution

**Commit:** 9343421a8bee7ba675cfbfb816d4868552a05b1b
**Date:** 2025-11-06
**Author:** foofork

**Problem Solved:**
- Circular dependency: `riptide-api ↔ riptide-facade` → **RESOLVED ✅**
- Now: `riptide-api → riptide-facade` (one-way)

**Changes Made:**
1. Created `riptide-pipeline` crate with shared type definitions
2. Removed `riptide-facade` dependency from `riptide-api`
3. Commented out facade fields in AppState (temporary measure)
4. Fixed all handler compilation errors with stub implementations
5. Achieved ZERO clippy warnings (45 warnings fixed)
6. Fixed type mismatches in facade implementations

**Files Modified:** 46 files across 3 crates

**Quality Gates Achieved:**
- ✅ Circular dependency broken (verified with `cargo tree`)
- ✅ ZERO clippy warnings in riptide-api
- ✅ All crates compile successfully
- ✅ riptide-pipeline tests pass (2/2)

---

## 🔍 CODE PATTERNS TO SEARCH

### Finding Stub Handlers
```bash
rg "temporarily unavailable|Facade temporarily" crates/riptide-api/src/handlers/
```

### Finding TODOs
```bash
rg -i "TODO|FIXME|HACK" --type rust -n
```

### Finding Commented Facades
```bash
rg "// pub.*facade" crates/riptide-api/src/state.rs
```

### Finding Unreachable Code
```bash
rg "#\[allow\(unreachable_code\)\]" --type rust -n
```

### Finding Test Stubs
```bash
rg "todo!\(\"Implement" tests/ crates/ -n
```

---

## 📝 NOTES FOR DEVELOPERS

### When Re-integrating Facades:

1. **Check AppState Definition** - Facades are still defined in `state.rs` but not initialized
2. **No Circular Dependencies** - Use `riptide-pipeline` for shared types, not direct facade imports in AppState
3. **Feature Gates** - Respect `#[cfg(feature)]` annotations (browser, spider, search, etc.)
4. **Handler Restoration** - Most handlers have preserved error handling and validation logic
5. **Test Coverage** - 51+ integration tests waiting for facade implementation
6. **Unreachable Code** - Check for `#[allow(unreachable_code)]` annotations before restoring

### Facade Initialization Pattern:

```rust
// DON'T: Causes circular dependency
use riptide_facade::{BrowserFacade, SearchFacade};

// DO: Use facades as fields, initialize separately
pub struct AppState {
    pub search_facade: Option<Arc<SearchFacade>>,
    // ...
}

// Initialize in separate initialization function
async fn initialize_facades(config: &AppConfig) -> Result<FacadeLayer> {
    // Facade initialization logic here
}
```

---

## 🎬 CONCLUSION

The circular dependency refactoring successfully broke the `riptide-api ↔ riptide-facade` cycle, but left 6 critical handlers temporarily disabled. The codebase is now in a healthy architectural state with clear separation of concerns, but requires restoration work to bring features back online.

**Immediate Action Required:**
- Phase 1 restoration (3 handlers) should be prioritized this week
- Spider integration can follow in Phase 2
- Test coverage can be addressed incrementally

**Long-term:**
- Strategy composition tests are optional nice-to-haves
- Monitoring enhancements can be deferred
- Archive code can remain as-is

---

**Report Generated By:** Analyst Agent (TODO and Disabled Code Analysis)
**Coordination:** Claude Code SPARC Methodology
**Next Steps:** Store findings in swarm memory, proceed with handler restoration

