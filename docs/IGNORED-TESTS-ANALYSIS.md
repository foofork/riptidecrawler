# Ignored Tests Analysis & Surgical Testing Plan

**Date:** 2025-11-07
**Total Ignored Tests:** 36
**Purpose:** Analyze feasibility of testing ignored tests with available infrastructure

---

## 🎯 Executive Summary

**Environment Availability:**
- ✅ **Redis:** Available (`/usr/bin/redis-server`)
- ✅ **Pdfium:** Available (`/usr/local/lib/libpdfium.so`)
- ❌ **Chrome/Chromium:** NOT Available (requires `snap install chromium`)

**Test Breakdown:**
- **31 tests** require Chrome/Chromium (Cannot test without installation)
- **~30 tests** in riptide-persistence require Redis (Can test ✅)
- **2 tests** in riptide-pdf require Pdfium (Can test ✅)
- **~4 tests** are performance/stress tests (Expensive, manual only)

---

## 📊 Categorized Ignored Tests

### Category 1: Redis Tests (~30 tests) - ✅ TESTABLE

**Package:** `riptide-persistence`
**File:** `tests/redis_integration_tests.rs`
**Requirement:** Redis server
**Status:** ✅ Redis available at `/usr/bin/redis-server`

**Tests Include:**
- Connection pool tests
- CRUD operations (get, set, delete, exists)
- TTL/expiration tests
- JSON serialization tests
- Batch operations
- Compression tests (if feature enabled)
- Error handling tests

**Surgical Testing Plan:**
```bash
# 1. Start Redis in background
redis-server --daemonize yes --port 6379

# 2. Run ignored tests
cargo test -p riptide-persistence --test redis_integration_tests -- --ignored

# 3. Stop Redis
redis-cli shutdown
```

**Disk Impact:** Minimal (~50MB for Redis data)
**Risk:** Low - Redis is lightweight and stable

---

### Category 2: Chrome/Browser Tests (31 tests) - ❌ NOT TESTABLE

**Packages:**
- `riptide-api`: 22 tests (resource_controls module)
- `riptide-facade`: 5 tests (browser.rs)
- `riptide-api`: 4 tests (browser_pool_integration)

**Requirement:** Chrome/Chromium browser
**Status:** ❌ NOT INSTALLED (requires `snap install chromium`)

**Why Chrome Tests Are Ignored:**
- Browser pool management and lifecycle
- Headless browser rendering
- Resource control and memory pressure
- Timeout protection
- Concurrent browser operations

**Tests in riptide-api/tests/resource_controls:**
```rust
#[ignore] // Requires Chrome/Chromium to be installed
- test_complete_resource_pipeline
- test_concurrent_operations_stress
- test_headless_browser_pool_cap
- test_memory_pressure_detection
- test_pdf_semaphore_concurrent_limit
- test_per_host_rate_limiting
- test_render_timeout_hard_cap
- test_resource_status_monitoring
... (22 total)
```

**Tests in riptide-facade/src/facades/browser.rs:**
```rust
#[ignore] // Requires browser installation
- test_extract_rendered_content
- test_extract_with_navigation
- test_extract_with_custom_config
- test_extract_single_page_app
- test_extract_with_metadata
```

**Installation Required:**
```bash
# Would need ~500MB disk space
snap install chromium

# Or build from source (HUGE - several GB)
```

**Recommendation:** Skip these in CI/automated testing. They're integration tests meant for manual verification with real browsers.

---

### Category 3: PDF Tests (2 tests) - ✅ TESTABLE

**Package:** `riptide-pdf`
**Files:** `src/memory_benchmark.rs`, `src/tests.rs`
**Requirement:** libpdfium.so
**Status:** ✅ Available at `/usr/local/lib/libpdfium.so`

**Tests:**
```rust
#[ignore] // Requires libpdfium.so - skip in CI environments
- test_memory_benchmark (memory_benchmark.rs)
- test_pdf_extraction (tests.rs)
```

**Surgical Testing Plan:**
```bash
# Set library path if needed
export LD_LIBRARY_PATH=/usr/local/lib:$LD_LIBRARY_PATH

# Run PDF tests
cargo test -p riptide-pdf --lib -- --ignored
```

**Disk Impact:** None (library already installed)
**Risk:** Low - single library dependency

---

### Category 4: Performance/Stress Tests (4 tests) - ⚠️ MANUAL ONLY

**Package:** `riptide-api`
**Files:** `tests/enhanced_pipeline_tests.rs`, `tests/stress_tests.rs`, `tests/pipeline_integration_test.rs`

**Tests:**
```rust
#[ignore] // Expensive test, run manually
- test_full_integration_with_all_features (enhanced_pipeline_tests.rs)
- test_high_volume_concurrent_requests (enhanced_pipeline_tests.rs)
- test_long_running_pipeline_stability (enhanced_pipeline_tests.rs)
- test_extreme_concurrent_stress (stress_tests.rs)
- test_batch_processing_performance (pipeline_integration_test.rs)
- test_concurrent_pipeline_stress (pipeline_integration_test.rs)
```

**Why Ignored:**
- Long-running (minutes to hours)
- High resource usage (CPU, memory, network)
- Meant for load testing/benchmarking
- Not suitable for CI/automated testing

**Recommendation:** Keep ignored. Run manually when performance validation needed.

---

### Category 5: Network/Scraper Tests (1 test) - ⚠️ FLAKY

**Package:** `riptide-facade`
**File:** `tests/scraper_facade_integration.rs`

**Test:**
```rust
#[ignore] // Requires network access
- test_real_world_scraping
```

**Why Ignored:**
- Depends on external websites
- Flaky (sites go down, change structure)
- Network-dependent timing
- Not deterministic

**Recommendation:** Keep ignored for CI. Useful for manual validation.

---

## 🎯 Surgical Testing Plan

### Phase 1: Redis Tests ✅ (Recommended)

**Preparation:**
```bash
# Check disk space (need ~100MB)
df -h / | tail -1

# Start Redis
redis-server --daemonize yes --port 6379 --maxmemory 100mb

# Verify Redis is running
redis-cli ping  # Should return "PONG"
```

**Execution:**
```bash
# Run all Redis integration tests
timeout 300 cargo test -p riptide-persistence --test redis_integration_tests -- --ignored --test-threads=1

# Monitor disk during test
watch -n 1 'df -h / | tail -1'
```

**Cleanup:**
```bash
# Stop Redis
redis-cli shutdown

# Clear Redis data if needed
rm -rf /var/lib/redis/*
```

**Expected Outcome:**
- ~30 tests should pass
- Validates Redis integration works correctly
- Verifies connection pooling, TTL, serialization

**Disk Impact:** +50-100MB temporary
**Time:** ~2-5 minutes

---

### Phase 2: PDF Tests ✅ (Recommended)

**Preparation:**
```bash
# Verify pdfium is accessible
ls -lh /usr/local/lib/libpdfium.so

# Set library path
export LD_LIBRARY_PATH=/usr/local/lib:$LD_LIBRARY_PATH
```

**Execution:**
```bash
# Run PDF ignored tests
cargo test -p riptide-pdf --lib -- --ignored --nocapture
```

**Expected Outcome:**
- 2 tests should pass
- Validates PDF extraction with pdfium works
- Memory benchmark provides performance baseline

**Disk Impact:** None
**Time:** <1 minute

---

### Phase 3: Chrome Tests ❌ (NOT Recommended)

**Why Skip:**
1. **Disk Cost:** ~500MB+ for Chromium installation
2. **Current Usage:** Already at 52% disk (31GB/63GB)
3. **Value:** Browser integration already validated via unit tests
4. **Risk:** May trigger additional dependencies/downloads

**Alternative:**
- These tests validate browser pool behavior
- Core browser functionality is tested via unit tests (which pass)
- Integration tests are more for load/stress testing
- Safe to skip for now

**If Needed Later:**
```bash
# Only if disk space allows (>10GB free)
snap install chromium  # ~500MB
cargo test -p riptide-api --lib tests::resource_controls -- --ignored
```

---

## 📊 Recommended Action

### ✅ DO TEST (Safe & Valuable):

1. **Redis Tests** (~30 tests)
   - Low resource cost
   - High value (validates critical caching infrastructure)
   - Redis is lightweight and stable
   - Can test ALL riptide-persistence integration

2. **PDF Tests** (2 tests)
   - No additional installation needed
   - Validates pdfium integration
   - Quick to run

### ❌ DON'T TEST (High Cost or Low Value):

1. **Chrome Tests** (31 tests)
   - Requires 500MB+ installation
   - Core browser logic already tested
   - Integration tests are redundant for validation
   - Better suited for manual/production testing

2. **Performance/Stress Tests** (4 tests)
   - Long-running
   - Resource-intensive
   - Meant for manual benchmarking
   - Not blockers for deployment

3. **Network Tests** (1 test)
   - Flaky (external dependency)
   - Not deterministic
   - Better as manual smoke test

---

## 🎯 Expected Results from Surgical Testing

**If we test Redis + PDF:**
- **Tests Run:** ~32 additional tests
- **Expected Pass Rate:** 100% (both are stable, deterministic)
- **New Total:** 981/981 tests passing (including 32 previously ignored)
- **Time Required:** ~5-10 minutes
- **Disk Cost:** +50-100MB temporary (cleaned up after)
- **Risk:** Very low

**Project Status After:**
- ✅ All deterministic tests validated
- ✅ All infrastructure dependencies verified
- ⚠️ 31 Chrome tests remain ignored (appropriate for CI)
- ⚠️ 4 performance tests remain ignored (appropriate)
- ⚠️ 1 network test remains ignored (flaky)

---

## 💡 Recommendations

### For CI/Production:

**DO:**
- ✅ Test Redis integration (validates critical caching)
- ✅ Test PDF extraction (validates document processing)
- ✅ Keep Chrome tests ignored (too expensive for CI)
- ✅ Keep performance tests ignored (manual only)

**DON'T:**
- ❌ Install Chrome just for tests (diminishing returns)
- ❌ Run stress tests in CI (resource waste)
- ❌ Rely on network tests (flaky)

### For This Session:

**Surgical Test Order:**
1. **Phase 1:** Start Redis → Test riptide-persistence → Stop Redis
2. **Phase 2:** Test riptide-pdf (pdfium already available)
3. **Phase 3:** Document which tests remain ignored and why

**Disk Safety:**
- Current: 52% used (29GB free) ✅
- After Redis tests: ~52% (Redis uses <100MB)
- Safe threshold: >5GB free
- We have 29GB free - very safe ✅

---

## 🎯 Decision Matrix

| Test Category | Count | Available? | Test It? | Why/Why Not |
|--------------|-------|------------|----------|-------------|
| Redis Integration | ~30 | ✅ Yes | ✅ YES | Low cost, high value, validates critical path |
| PDF Processing | 2 | ✅ Yes | ✅ YES | Already installed, quick validation |
| Chrome/Browser | 31 | ❌ No | ❌ NO | 500MB cost, redundant with unit tests |
| Performance/Stress | 4 | N/A | ❌ NO | Too expensive, manual benchmarking only |
| Network Scraping | 1 | ⚠️ Flaky | ❌ NO | External dependency, non-deterministic |

---

## Next Steps

1. **Approve surgical testing plan**
2. **Run Phase 1: Redis tests** (~30 tests, 5 minutes)
3. **Run Phase 2: PDF tests** (2 tests, 1 minute)
4. **Document final results**
5. **Update comprehensive test report**

**Total Additional Coverage:** +32 tests (from 949 → 981)
**Total Time:** ~10 minutes
**Disk Risk:** Very low (50-100MB temporary)
**Value:** High (validates all available infrastructure)
