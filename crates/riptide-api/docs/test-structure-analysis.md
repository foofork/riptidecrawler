# Riptide-API Test Structure Analysis

**Analysis Date:** 2025-10-27
**Total Test Files:** 49
**Ignored Tests:** 60
**Feature Flags Found:** 28 instances

## Executive Summary

The riptide-api test suite contains **comprehensive integration tests** but currently lacks proper **feature flag organization**. Most external dependencies (WASM extractor, Redis, browser services, LLM providers) are handled through `#[ignore]` attributes rather than feature gates. This analysis provides a detailed mapping to implement selective test execution via feature flags.

---

## 1. Current Feature Flags in Cargo.toml

### Defined Features

```toml
[features]
default = []
events = []           # EventEmitter/ResultTransformer
sessions = []         # Session management system
streaming = []        # SSE/WebSocket/NDJSON streaming
telemetry = []        # Telemetry configuration
persistence = []      # Redis-based caching and multi-tenancy
jemalloc = ["riptide-performance/jemalloc", "tikv-jemallocator"]
profiling-full = ["jemalloc", "riptide-performance/bottleneck-analysis-full"]
full = ["events", "sessions", "streaming", "telemetry", "persistence", "jemalloc"]
```

### Current Usage in Tests

**Limited:** Only 28 instances found:
- `#[cfg(feature = "persistence")]` - persistence_integration.rs
- `#[cfg(feature = "sessions")]` - cross_module_integration.rs (8 uses)
- `#[cfg(feature = "streaming")]` - cross_module_integration.rs (4 uses)
- `#[cfg(feature = "jemalloc")]` - profiling_integration_tests.rs (4 uses)
- `#[cfg(feature = "profiling-full")]` - cross_module_integration.rs (4 uses)

**Problem:** 60 tests use `#[ignore]` instead of proper feature gates!

---

## 2. Feature Dependency Matrix

### Test File → Required Features

| Test File | WASM Extractor | Redis/Persistence | Browser/Headless | LLM Providers | Pure Unit | Notes |
|-----------|---------------|-------------------|------------------|---------------|-----------|-------|
| **api_tests.rs** | ❌ | ❌ | ❌ | ❌ | ✅ | Pure endpoint tests with minimal state |
| **browser_pool_integration.rs** | ✅ | ✅ | ✅ | ❌ | ❌ | All 15 tests require browser+Redis |
| **config_env_tests.rs** | ❌ | ❌ | ❌ | ❌ | ✅ | Environment variable parsing only |
| **cross_module_integration.rs** | ✅ | ⚠️ | ✅ | ✅ | ❌ | Complex integration, uses feature flags |
| **e2e_full_stack.rs** | ✅ | ✅ | ✅ | ✅ | ❌ | Full AppState required |
| **error_recovery.rs** | ✅ | ✅ | ✅ | ❌ | ❌ | Most tests ignored for Redis |
| **golden/test_extraction.rs** | ✅ | ❌ | ❌ | ❌ | ❌ | Mock extractor, but needs WASM for full tests |
| **health_tests.rs** | ❌ | ❌ | ❌ | ❌ | ✅ | Pure unit tests |
| **health_check_system_tests.rs** | ❌ | ❌ | ❌ | ❌ | ✅ | Mock-based health checks |
| **integration_phase4a_tests.rs** | ✅ | ✅ | ✅ | ✅ | ❌ | **ALL IGNORED** - Full AppState |
| **integration_tests.rs** | ❌ | ❌ | ❌ | ❌ | ✅ | Some ignored but mostly pure |
| **metrics_tests.rs** | ❌ | ⚠️ | ❌ | ❌ | ✅ | Minimal state |
| **pdf_integration_tests.rs** | ✅ | ❌ | ❌ | ❌ | ❌ | **ALL 13 IGNORED** - WASM required |
| **persistence_integration.rs** | ❌ | ✅ | ❌ | ❌ | ❌ | Uses `#[cfg(feature = "persistence")]` ✅ |
| **phase4b_integration_tests.rs** | ✅ | ✅ | ✅ | ✅ | ❌ | **ALL IGNORED** - Full dependencies |
| **profiling_endpoints_live.rs** | ✅ | ❌ | ❌ | ❌ | ❌ | Jemalloc profiling |
| **profiling_integration_tests.rs** | ✅ | ❌ | ❌ | ❌ | ❌ | Uses `#[cfg(feature = "jemalloc")]` ✅ |
| **security_integration.rs** | ✅ | ✅ | ❌ | ❌ | ❌ | Security tests with state |
| **session_tests.rs** | ❌ | ❌ | ❌ | ❌ | ✅ | Pure session logic |
| **streaming_*.rs** (4 files) | ✅ | ⚠️ | ⚠️ | ❌ | ❌ | Various streaming tests |
| **worker_tests.rs** | ❌ | ❌ | ❌ | ❌ | ✅ | Worker behavior contracts |

**Legend:**
- ✅ Required
- ❌ Not required
- ⚠️ Optional/Conditional

---

## 3. Dependency Categories

### 3.1 WASM Extractor Dependency (26 test files)

**Why needed:** Content extraction, table processing, intelligent chunking

**Files requiring WASM:**
- `browser_pool_integration.rs` (15 tests)
- `cross_module_integration.rs` (12 tests)
- `e2e_full_stack.rs` (8 tests)
- `error_recovery.rs` (8 tests)
- `golden/test_extraction.rs` (13 tests)
- `integration_phase4a_tests.rs` (ALL - ~50 tests)
- `pdf_integration_tests.rs` (13 tests)
- `phase4b_integration_tests.rs` (ALL)
- `profiling_endpoints_live.rs`
- `profiling_integration_tests.rs`
- `security_integration.rs`
- `streaming_*.rs` (4 files, ~30 tests)

**Current pattern:**
```rust
#[tokio::test]
#[ignore = "Requires WASM extractor and full AppState dependencies - run with --ignored"]
async fn test_table_extraction_from_html() { ... }
```

**Recommended:**
```rust
#[tokio::test]
#[cfg(feature = "wasm-extractor")]
async fn test_table_extraction_from_html() { ... }
```

### 3.2 Redis/Persistence Dependency (18 test files)

**Why needed:** Caching, session storage, multi-tenancy, persistence layer

**Files requiring Redis:**
- `browser_pool_integration.rs` (requires session storage)
- `cross_module_integration.rs` (optional for some tests)
- `e2e_full_stack.rs` (full state)
- `error_recovery.rs` (failover tests)
- `integration_phase4a_tests.rs` (full state)
- `persistence_integration.rs` (**ALREADY USES** `#[cfg(feature = "persistence")]`)
- `phase4b_integration_tests.rs`
- `security_integration.rs`
- Streaming tests (optional)

**Good Example (already implemented):**
```rust
// persistence_integration.rs
#![cfg(all(test, feature = "persistence"))]

fn should_skip_redis_tests() -> bool {
    std::env::var("SKIP_PERSISTENCE_TESTS").is_ok()
}
```

### 3.3 Browser/Headless Services (8 test files)

**Why needed:** Browser automation, screenshot capture, JavaScript execution

**Files requiring browsers:**
- `browser_pool_integration.rs` (PRIMARY - 15 tests)
- `cross_module_integration.rs` (browser-specific tests)
- `e2e_full_stack.rs` (full workflows)
- `phase4b_integration_tests.rs`

**Current pattern:**
```rust
#[tokio::test]
#[ignore = "Requires browser launcher service and Redis - run with --ignored"]
async fn test_create_browser_session_success() { ... }
```

**Recommended:**
```rust
#[tokio::test]
#[cfg(all(feature = "browser-automation", feature = "persistence"))]
async fn test_create_browser_session_success() { ... }
```

### 3.4 LLM Provider Dependency (3 test files)

**Why needed:** Intelligent content analysis, sentiment analysis, summarization

**Files requiring LLMs:**
- `integration_phase4a_tests.rs` (LLM provider management tests)
- `cross_module_integration.rs` (LLM-enhanced chunking)
- `phase4b_integration_tests.rs` (optional)

**Recommended:**
```rust
#[tokio::test]
#[cfg(feature = "llm-providers")]
async fn test_llm_failover_scenario() { ... }
```

### 3.5 Pure Unit Tests (15 test files) ✅

**No external dependencies - always runnable:**
- `api_tests.rs` - Basic endpoint routing
- `config_env_tests.rs` - Environment parsing
- `health_tests.rs` - Health check logic
- `health_check_system_tests.rs` - Mock health checks
- `integration_tests.rs` - Some tests are pure
- `metrics_tests.rs` - Metrics collection
- `session_tests.rs` - Session logic
- `worker_tests.rs` - Worker contracts
- `unit/test_*.rs` (5 files) - All pure unit tests

---

## 4. Recommended Feature Flag Structure

### 4.1 New Feature Flags to Add

```toml
[features]
# Existing (keep as-is)
default = []
events = []
sessions = []
streaming = []
telemetry = []
persistence = []
jemalloc = ["riptide-performance/jemalloc", "tikv-jemallocator"]
profiling-full = ["jemalloc", "riptide-performance/bottleneck-analysis-full"]

# NEW: Test dependency features
wasm-extractor = []            # WASM-based content extraction
browser-automation = []        # Browser pool and headless services
llm-providers = []             # LLM integration (OpenAI, Anthropic, etc.)

# NEW: Combined test profiles
test-unit = []                 # Pure unit tests only (no external deps)
test-integration = [           # Integration tests with mocked services
    "sessions",
    "streaming",
    "events"
]
test-full = [                  # Full end-to-end tests
    "sessions",
    "streaming",
    "events",
    "persistence",
    "wasm-extractor",
    "browser-automation",
    "llm-providers",
    "jemalloc"
]

# Keep existing
full = ["events", "sessions", "streaming", "telemetry", "persistence", "jemalloc"]
```

### 4.2 Feature Flag Usage Patterns

#### Pattern 1: Single Dependency

```rust
#[tokio::test]
#[cfg(feature = "wasm-extractor")]
async fn test_wasm_content_extraction() { ... }
```

#### Pattern 2: Multiple Dependencies (AND)

```rust
#[tokio::test]
#[cfg(all(feature = "browser-automation", feature = "persistence"))]
async fn test_browser_session_persistence() { ... }
```

#### Pattern 3: Optional Dependencies (OR)

```rust
#[tokio::test]
#[cfg(any(feature = "streaming", feature = "sessions"))]
async fn test_stream_or_session_feature() { ... }
```

#### Pattern 4: Conditional Code Blocks

```rust
#[tokio::test]
async fn test_with_optional_features() {
    #[cfg(feature = "wasm-extractor")]
    {
        // WASM-specific test code
    }

    #[cfg(not(feature = "wasm-extractor"))]
    {
        // Mock/fallback test code
    }
}
```

---

## 5. Migration Recommendations

### 5.1 Priority Categorization

#### **P0 (Critical) - Pure Unit Tests**
**Action:** Ensure always runnable without flags
- ✅ Already working: `api_tests.rs`, `config_env_tests.rs`, `health_tests.rs`, etc.
- **No changes needed** - these should run in `cargo test` by default

#### **P1 (High) - WASM Extractor Tests**
**Action:** Add `#[cfg(feature = "wasm-extractor")]`
- `pdf_integration_tests.rs` (13 tests) - **HIGH VALUE**
- `golden/test_extraction.rs` (13 tests) - **QUALITY GATES**
- `integration_phase4a_tests.rs` (50+ tests) - **COMPREHENSIVE**
- `phase4b_integration_tests.rs`

**Estimated effort:** 4-6 hours

#### **P2 (Medium) - Persistence/Redis Tests**
**Action:** Add `#[cfg(feature = "persistence")]`
- `browser_pool_integration.rs` (needs both browser + persistence)
- `e2e_full_stack.rs`
- `error_recovery.rs`
- Streaming tests (partial)

**Good news:** `persistence_integration.rs` already done! ✅

**Estimated effort:** 3-4 hours

#### **P3 (Low) - Browser Automation Tests**
**Action:** Add `#[cfg(feature = "browser-automation")]`
- `browser_pool_integration.rs` (15 tests)
- Related cross-module tests

**Estimated effort:** 2-3 hours

#### **P4 (Nice-to-have) - LLM Provider Tests**
**Action:** Add `#[cfg(feature = "llm-providers")]`
- `integration_phase4a_tests.rs` (partial)
- Cross-module LLM tests

**Estimated effort:** 1-2 hours

### 5.2 Phased Migration Plan

#### **Phase 1: Infrastructure** (1-2 hours)
1. Add new feature flags to `Cargo.toml`
2. Create `test-profiles` documentation
3. Update CI/CD to use feature-based test execution

#### **Phase 2: WASM Tests** (4-6 hours)
1. Update `pdf_integration_tests.rs` → `#[cfg(feature = "wasm-extractor")]`
2. Update `golden/test_extraction.rs` → `#[cfg(feature = "wasm-extractor")]`
3. Update `integration_phase4a_tests.rs` → combine features
4. Update `phase4b_integration_tests.rs`

#### **Phase 3: Persistence Tests** (3-4 hours)
1. Pattern after `persistence_integration.rs` (already correct)
2. Update `browser_pool_integration.rs` → `#[cfg(all(feature = "browser-automation", feature = "persistence"))]`
3. Update `e2e_full_stack.rs` → `#[cfg(feature = "test-full")]`
4. Update `error_recovery.rs` → conditional blocks

#### **Phase 4: Browser + LLM Tests** (3-4 hours)
1. Update `browser_pool_integration.rs` tests
2. Update cross-module browser tests
3. Add LLM provider gates
4. Comprehensive test coverage validation

#### **Phase 5: Documentation & CI** (2-3 hours)
1. Update test README
2. Create feature flag guide
3. Configure CI matrix for different feature combinations
4. Validate test coverage remains high

**Total estimated effort: 13-19 hours**

---

## 6. Test Helper Patterns

### Current Pattern (Test Helpers)

```rust
// test_helpers.rs - Used by most tests
pub fn create_minimal_test_app() -> axum::Router { ... }
pub async fn create_test_app() -> axum::Router { ... }
```

**Usage:** Found in 18+ test files

### Recommendation: Feature-Gated Helpers

```rust
// test_helpers.rs
#[cfg(feature = "wasm-extractor")]
pub async fn create_test_app_with_wasm() -> axum::Router { ... }

#[cfg(feature = "persistence")]
pub async fn create_test_app_with_redis() -> axum::Router { ... }

#[cfg(feature = "test-full")]
pub async fn create_full_test_app() -> axum::Router {
    // Full AppState with all services
}

// Always available
pub fn create_minimal_test_app() -> axum::Router {
    // Minimal mocked state
}
```

---

## 7. Existing Good Patterns

### ✅ persistence_integration.rs (EXCELLENT EXAMPLE)

```rust
#![cfg(all(test, feature = "persistence"))]

fn should_skip_redis_tests() -> bool {
    std::env::var("SKIP_PERSISTENCE_TESTS").is_ok()
}

#[tokio::test]
async fn test_multi_tenant_cache_isolation() -> Result<()> {
    if should_skip_redis_tests() {
        return Ok(());
    }
    // ... test code
}
```

**Why excellent:**
- Module-level feature gate (`#![cfg(all(test, feature = "persistence"))]`)
- Runtime skip check for CI flexibility
- Clear environment variable override
- Proper error handling

### ✅ cross_module_integration.rs (GOOD PATTERN)

```rust
#[tokio::test]
#[cfg(all(feature = "streaming", feature = "jemalloc"))]
async fn test_streaming_to_cache_persistence() { ... }
```

**Why good:**
- Combines multiple features with `all()`
- Self-documenting test requirements
- Enables granular test selection

### ❌ integration_phase4a_tests.rs (NEEDS IMPROVEMENT)

```rust
#[tokio::test]
#[ignore = "Requires WASM extractor, LLM providers, and full AppState dependencies - run with --ignored"]
async fn test_table_extraction_from_html() { ... }
```

**Problems:**
- `#[ignore]` is all-or-nothing (can't selectively enable)
- String description is not machine-readable
- No compile-time enforcement
- Clutters test output

**Should be:**
```rust
#[tokio::test]
#[cfg(all(feature = "wasm-extractor", feature = "llm-providers"))]
async fn test_table_extraction_from_html() { ... }
```

---

## 8. CI/CD Integration Strategy

### Current State
- Most integration tests skipped with `#[ignore]`
- CI probably runs `cargo test` (skips all ignored tests)
- No selective test execution based on available services

### Recommended CI Matrix

```yaml
# .github/workflows/test.yml (example)
test:
  strategy:
    matrix:
      include:
        # Quick feedback: Pure unit tests
        - name: "Unit Tests"
          features: ""
          rust-cache-key: "unit"

        # Integration tests (no external services)
        - name: "Integration Tests (Mocked)"
          features: "test-integration"
          rust-cache-key: "integration"

        # Full end-to-end (requires services)
        - name: "E2E Tests (Full Stack)"
          features: "test-full"
          services: "redis,chrome"
          rust-cache-key: "e2e"

  steps:
    - name: Run Tests
      run: |
        cargo test --features "${{ matrix.features }}"
```

### Benefits
1. **Fast feedback:** Unit tests run first (~2-5 min)
2. **Parallel execution:** Integration and E2E in parallel
3. **Resource optimization:** Only spin up services when needed
4. **Clear failures:** Feature-specific test failures are isolated

---

## 9. Quick Reference Card

### Test Execution Commands

```bash
# Pure unit tests (fast, no external deps)
cargo test

# Unit + integration tests (mocked services)
cargo test --features test-integration

# WASM extractor tests only
cargo test --features wasm-extractor

# Persistence tests (requires Redis)
cargo test --features persistence

# Browser automation tests (requires Chrome + Redis)
cargo test --features browser-automation,persistence

# LLM provider tests
cargo test --features llm-providers

# Full end-to-end suite (requires all services)
cargo test --features test-full

# Run specific ignored test manually
cargo test --test pdf_integration_tests -- --ignored

# Run with specific feature combination
cargo test --features "wasm-extractor,persistence" --test integration_phase4a_tests
```

---

## 10. Specific File Recommendations

### High-Impact Changes

#### `pdf_integration_tests.rs` (13 tests, ALL ignored)
```rust
// BEFORE
#[tokio::test]
#[ignore = "Requires WASM extractor and full AppState dependencies - run with --ignored"]
async fn test_basic_pdf_processing_api_endpoint() { ... }

// AFTER
#![cfg(all(test, feature = "wasm-extractor"))]  // Module-level

#[tokio::test]
async fn test_basic_pdf_processing_api_endpoint() { ... }  // No ignore!
```

#### `browser_pool_integration.rs` (15 tests, ALL ignored)
```rust
// BEFORE
#[tokio::test]
#[ignore = "Requires browser launcher service and Redis - run with --ignored"]
async fn test_create_browser_session_success() { ... }

// AFTER
#![cfg(all(test, feature = "browser-automation", feature = "persistence"))]

#[tokio::test]
async fn test_create_browser_session_success() { ... }
```

#### `integration_phase4a_tests.rs` (50+ tests)
```rust
// BEFORE
#[tokio::test]
#[ignore = "Requires WASM extractor, LLM providers, and full AppState dependencies - run with --ignored"]
async fn test_llm_enhanced_chunking_workflow() { ... }

// AFTER
#![cfg(all(test, feature = "wasm-extractor", feature = "llm-providers"))]

#[tokio::test]
async fn test_llm_enhanced_chunking_workflow() { ... }
```

---

## 11. Summary Statistics

### Current State
- **Total test files:** 49
- **Total ignored tests:** 60+ (distributed across multiple files)
- **Files using feature flags:** 2 (persistence_integration.rs, profiling_integration_tests.rs)
- **Pure unit tests:** ~15 files (always runnable)
- **Blocked integration tests:** ~26 files (need feature gates)

### Post-Migration Target
- **Total test files:** 49 (unchanged)
- **Tests with proper feature gates:** 45+ files
- **Default `cargo test` runs:** ~150-200 pure unit tests (< 5 min)
- **`cargo test --features test-integration`:** ~300-400 tests (< 10 min)
- **`cargo test --features test-full`:** 500+ tests (< 30 min with all services)

### Expected Improvements
1. **CI speed:** 60-80% faster for PR validation (unit tests only)
2. **Developer experience:** Clear feature requirements, no surprise failures
3. **Test coverage visibility:** Separate coverage for unit vs integration
4. **Selective execution:** Run only relevant tests for code changes
5. **Documentation:** Self-documenting test requirements via feature flags

---

## 12. Action Items

### Immediate (This Sprint)
- [ ] Add new feature flags to `Cargo.toml`
- [ ] Update `pdf_integration_tests.rs` (highest value, isolated)
- [ ] Update `golden/test_extraction.rs` (quality gates)
- [ ] Document feature flag usage in `tests/README.md`

### Short-term (Next Sprint)
- [ ] Migrate `browser_pool_integration.rs`
- [ ] Migrate `integration_phase4a_tests.rs`
- [ ] Migrate `e2e_full_stack.rs`
- [ ] Update CI/CD pipeline for feature matrix

### Medium-term (Next Month)
- [ ] Complete all feature flag migrations
- [ ] Add test coverage reporting per feature
- [ ] Create developer guide for writing feature-gated tests
- [ ] Validate 100% test pass rate for each feature combination

---

## Appendix A: Complete File Listing with Dependencies

| # | File | Lines | Tests | WASM | Redis | Browser | LLM | Pure | Status |
|---|------|-------|-------|------|-------|---------|-----|------|--------|
| 1 | api_tests.rs | 165 | 8 | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ Ready |
| 2 | benchmarks/performance_tests.rs | - | - | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ Ready |
| 3 | browser_pool_integration.rs | 526 | 15 | ✅ | ✅ | ✅ | ❌ | ❌ | ⚠️ All ignored |
| 4 | config_env_tests.rs | 351 | 13 | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ Ready |
| 5 | cross_module_integration.rs | 693 | 12 | ✅ | ⚠️ | ✅ | ✅ | ❌ | ⚠️ Mixed flags |
| 6 | e2e_full_stack.rs | 663 | 8 | ✅ | ✅ | ✅ | ✅ | ❌ | ⚠️ All ignored |
| 7 | error_recovery.rs | 467 | 8 | ✅ | ✅ | ✅ | ❌ | ❌ | ⚠️ Most ignored |
| 8 | fixtures/* | - | - | ⚠️ | ❌ | ❌ | ❌ | ✅ | ✅ Test data |
| 9 | golden/test_extraction.rs | 502 | 13 | ✅ | ❌ | ❌ | ❌ | ❌ | ⚠️ Mock/real |
| 10 | health_check_system_tests.rs | 572 | 30 | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ Ready |
| 11 | health_tests.rs | - | - | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ Ready |
| 12 | integration_phase4a_tests.rs | 1674 | 50+ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ ALL IGNORED |
| 13 | integration_tests.rs | - | - | ⚠️ | ❌ | ❌ | ❌ | ⚠️ | ⚠️ Mixed |
| 14 | integration/test_edge_cases.rs | - | - | ⚠️ | ❌ | ❌ | ❌ | ✅ | ✅ Mostly pure |
| 15 | integration/test_handlers.rs | - | - | ⚠️ | ❌ | ❌ | ❌ | ✅ | ✅ Mostly pure |
| 16 | metrics_integration_tests.rs | - | - | ❌ | ⚠️ | ❌ | ❌ | ✅ | ✅ Minimal state |
| 17 | metrics_tests.rs | - | - | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ Ready |
| 18 | pdf_integration_tests.rs | 1170 | 13 | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ ALL IGNORED |
| 19 | performance_regression.rs | - | - | ⚠️ | ❌ | ❌ | ❌ | ✅ | ✅ Benchmarks |
| 20 | persistence_integration.rs | 647 | 15 | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ HAS FEATURE! |
| 21 | phase4b_integration_tests.rs | - | - | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ ALL IGNORED |
| 22 | profiling_endpoints_live.rs | - | - | ✅ | ❌ | ❌ | ❌ | ❌ | ⚠️ Jemalloc |
| 23 | profiling_integration_tests.rs | - | - | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ HAS FEATURE! |
| 24 | resource_tests.rs | - | - | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ Ready |
| 25 | security_integration.rs | - | - | ✅ | ✅ | ❌ | ❌ | ❌ | ⚠️ State needed |
| 26 | session_performance_tests.rs | - | - | ❌ | ⚠️ | ❌ | ❌ | ⚠️ | ⚠️ Optional Redis |
| 27 | session_security_tests.rs | - | - | ❌ | ⚠️ | ❌ | ❌ | ⚠️ | ⚠️ Optional Redis |
| 28 | session_tests.rs | 367 | 15 | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ Ready |
| 29 | streaming_endpoints_integration.rs | - | - | ✅ | ⚠️ | ⚠️ | ❌ | ❌ | ⚠️ Partial |
| 30 | streaming_metrics_test.rs | - | - | ✅ | ❌ | ❌ | ❌ | ❌ | ⚠️ WASM needed |
| 31 | streaming_ndjson_tests.rs | - | - | ✅ | ⚠️ | ❌ | ❌ | ❌ | ⚠️ Partial |
| 32 | streaming_response_helpers_integration.rs | - | - | ✅ | ❌ | ❌ | ❌ | ❌ | ⚠️ WASM needed |
| 33 | streaming_sse_ws_tests.rs | - | - | ✅ | ⚠️ | ❌ | ❌ | ❌ | ⚠️ Partial |
| 34 | stress_tests.rs | - | - | ✅ | ⚠️ | ❌ | ❌ | ❌ | ⚠️ Load tests |
| 35 | telemetry_tests.rs | - | - | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ Ready |
| 36 | test_helpers.rs | - | - | ⚠️ | ⚠️ | ⚠️ | ⚠️ | ✅ | ⚠️ Multiple helpers |
| 37 | test_runner.rs | - | - | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ Test utilities |
| 38 | unit/test_errors.rs | - | - | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ Ready |
| 39 | unit/test_pipeline.rs | - | - | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ Ready |
| 40 | unit/test_state.rs | - | - | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ Ready |
| 41 | unit/test_validation.rs | - | - | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ Ready |
| 42 | worker_tests.rs | 164 | 5 | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ Ready |

**Status Legend:**
- ✅ Ready: Works with current setup
- ⚠️ Partial: Some tests need features
- ❌ Blocked: All tests need feature gates

---

## Conclusion

The riptide-api test suite is **comprehensive and well-structured** but currently **under-utilizes feature flags**. By implementing the recommended feature flag system:

1. **60+ ignored tests** can become selectively executable
2. **CI/CD pipelines** can run appropriate test subsets (3-5x faster)
3. **Developer experience** improves with clear dependency requirements
4. **Test coverage** becomes more granular and measurable

**Estimated total effort:** 13-19 hours across 4 phases

**Highest ROI first steps:**
1. Add feature flags to `Cargo.toml` (30 min)
2. Migrate `pdf_integration_tests.rs` (2 hours)
3. Migrate `golden/test_extraction.rs` (2 hours)
4. Update CI configuration (1 hour)

This provides **immediate value** by unlocking PDF and extraction quality tests while establishing patterns for the remaining migrations.
