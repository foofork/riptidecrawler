# CLI Real-World Testing Roadmap

This document tracks the plan to implement production-ready real-world testing for the RipTide CLI.

---

# 🚨 ACTION ITEMS - ALL TODOS

## 🔴 CRITICAL - Must Fix for Production (P0)

### ✅ TODO #0: Ensure ToS-Compliant Test URLs
**Status**: ✅ COMPLETED (2025-10-13)
**Priority**: P0 - CRITICAL PREREQUISITE
**Effort**: 1 day (DONE)
**Location**: `tests/webpage-extraction/test-urls.json`

**Problem**: Original test URLs (69%) violated Terms of Service
**Impact**: Legal risk, IP bans, lawsuits from Amazon, eBay, Twitter, Reddit, NYT, etc.
**Action Taken**:
1. ✅ Migrated from 29 non-compliant URLs to 30 ToS-compliant URLs
2. ✅ Backed up original URLs to `test-urls-original-backup.json`
3. ✅ Created comprehensive migration documentation
4. ✅ All new URLs are legally safe (100% compliance)

**Documentation**:
- 📄 **Migration Table**: `/docs/testing/TEST_URLS_MIGRATION_TABLE.md`
- 📄 **Safe URLs Guide**: `/docs/testing/SAFE_TEST_URLS_GUIDE.md`
- 📄 **Test URLs**: `/tests/webpage-extraction/test-urls.json` (updated)

**Success Criteria**:
- [x] All non-compliant URLs removed (Amazon, eBay, CNN, BBC, Twitter, Reddit, etc.)
- [x] 30 safe replacements with diverse content types
- [x] Rate limits documented (ArXiv: 3s, Wikipedia: 0.2s)
- [x] Edge case coverage maintained (20+ edge cases)

---

### ☐ TODO #1: Integrate Test Harness with cargo test
**Status**: 📝 Ready to implement
**Priority**: P0 - CRITICAL BLOCKER
**Effort**: 2 days
**Location**: `tests/webpage-extraction/cli-test-harness.rs` → `tests/common/`

**Problem**: Test harness exists but is **NEVER USED** by any tests
**Impact**: Cannot run systematic tests across 30 ToS-compliant URLs, no automation possible
**Action Required**:
1. Move `cli-test-harness.rs` to `tests/common/` module
2. Create integration tests in `tests/cli/real_world_integration.rs`
3. Import and use `TestHarness` in test suite
4. Run against local API server
5. Verify with `cargo test --test real_world_integration`

**Success Criteria**:
- [ ] TestHarness accessible from all test files
- [ ] At least 1 integration test using TestHarness
- [ ] Test runs in `cargo test` workflow
- [ ] Test session results persisted to JSON

---

### ☐ TODO #2: Implement Content Validation Framework
**Status**: 📝 Ready to implement
**Priority**: P0 - CRITICAL BLOCKER
**Effort**: 3 days
**Location**: New file `tests/common/content_validator.rs`

**Problem**: Tests only check success/failure, not extraction **correctness**
**Impact**: Cannot detect when CLI returns wrong content - may ship broken features
**Action Required**:
1. Create `ContentValidator` struct with validation rules
2. Implement keyword/pattern matching
3. Add quality score thresholds
4. Add metadata completeness checks
5. Integrate with TestHarness

**Validation Rules**:
```rust
pub struct ContentValidator {
    // Validate content length (min/max)
    // Check for expected keywords
    // Verify quality score threshold
    // Validate metadata fields
    // Check extraction time performance
}
```

**Success Criteria**:
- [ ] Can validate content against expected keywords
- [ ] Can detect missing or incorrect titles
- [ ] Can flag quality score regressions
- [ ] Catches at least 1 real bug in testing

---

### ☐ TODO #3: Create Baseline Management System
**Status**: 📝 Ready to implement
**Priority**: P0 - CRITICAL BLOCKER
**Effort**: 2 days
**Location**: `test-results/baselines/` directory + loader/comparator

**Problem**: No system to store and compare against baseline results
**Impact**: Cannot detect **regressions** in extraction quality over time
**Action Required**:
1. Define baseline JSON schema
2. Create baseline generator tool (`cargo run --bin generate-baselines`)
3. Implement baseline loader in TestHarness
4. Implement baseline comparator with diff logic
5. Create initial baselines for 10 test URLs

**Baseline Schema**:
```json
{
  "url": "https://example.com",
  "expected_title": "Page Title",
  "expected_keywords": ["keyword1", "keyword2"],
  "min_quality_score": 0.85,
  "max_extraction_time_ms": 500,
  "expected_metadata": {
    "word_count": ">100",
    "has_images": true
  }
}
```

**Success Criteria**:
- [ ] Baselines stored in `test-results/baselines/`
- [ ] Can generate baseline from actual extraction
- [ ] Can compare current result vs baseline
- [ ] Flags differences > 20% as failures

---

### ☐ TODO #4: Activate Real API Tests
**Status**: 📝 Ready to implement - Quick win!
**Priority**: P0 - CRITICAL (but fast)
**Effort**: 1 day
**Location**: `tests/cli/real_api_tests.rs`

**Problem**: 14 real-world tests exist but ALL marked `#[ignore]`
**Impact**: Real API tests must be run **manually**, not in CI
**Action Required**:
1. Remove `#[ignore]` from critical tests (Wikipedia, GitHub, confidence scoring)
2. Add test setup (ensure API server running)
3. Add proper assertions with ContentValidator
4. Add cleanup/teardown logic
5. Run locally to verify

**Success Criteria**:
- [ ] At least 5 tests active (not ignored)
- [ ] Tests pass with real API server running
- [ ] Tests use ContentValidator for assertions
- [ ] Tests run in <2 minutes

---

## 🟠 HIGH PRIORITY - Scale & Automation (P1)

### ☐ TODO #5: Implement Regression Detection System
**Status**: 📝 Ready to implement (blocked by TODO #2, #3)
**Priority**: P1 - High
**Effort**: 3 days
**Location**: `tests/common/regression_detector.rs`

**Problem**: No automated detection of quality degradations
**Impact**: Regressions go unnoticed until users report bugs
**Action Required**:
1. Create `RegressionDetector` struct
2. Compare current vs baseline results
3. Flag quality score drops >10%
4. Flag extraction time increases >50%
5. Flag content changes >20%

**Success Criteria**:
- [ ] Detects at least 1 real regression in testing
- [ ] Generates diff report for changes
- [ ] Fails tests on regression detection

---

### ☐ TODO #6: Performance Benchmarking Framework
**Status**: 📝 Ready to implement
**Priority**: P1 - High
**Effort**: 3 days
**Location**: `benches/cli_benchmarks.rs`

**Problem**: No framework for measuring CLI performance baselines
**Impact**: Cannot detect performance regressions, no SLA validation
**Action Required**:
1. Create benchmark harness using `criterion`
2. Measure latency (P50, P95, P99)
3. Measure throughput (requests/sec)
4. Store historical performance data
5. Alert on performance regressions >20%

**Success Criteria**:
- [ ] Benchmarks for 10 test URLs
- [ ] Historical trend data persisted
- [ ] Performance regression detection working

---

### ☐ TODO #7: Expand Test URL Coverage
**Status**: 📝 Ready to implement
**Priority**: P1 - High
**Effort**: 2 days
**Location**: Use all 30 URLs from `test-urls.json`

**Problem**: Currently testing only 10 URLs, have 30 diverse ToS-compliant URLs defined
**Impact**: Missing edge cases (SPAs, heavy JS, international sites, JSON APIs)
**Action Required**:
1. Create tests for all 30 URLs
2. Create baselines for each URL
3. Document expected behavior per URL
4. Handle expected failures (404 errors) gracefully
5. Implement rate limiting (ArXiv: 3s, Wikipedia: 0.2s)

**Success Criteria**:
- [ ] Tests for all 30 URLs
- [ ] At least 27/30 URLs passing (90%+)
- [ ] Known failures documented
- [ ] Rate limits respected

---

### ☐ TODO #8: CI/CD Integration - GitHub Actions
**Status**: 📝 Ready to implement
**Priority**: P1 - High
**Effort**: 5 days
**Location**: `.github/workflows/cli-real-world-tests.yml`

**Problem**: Real-world tests not integrated with CI pipeline
**Impact**: Tests must be run manually, regressions not caught in PRs
**Action Required**:
1. Create GitHub Actions workflow
2. Start API server in CI environment
3. Run test suite on every PR
4. Upload test results as artifacts
5. Add PR comment with test summary

**Workflow Steps**:
```yaml
- Start Redis
- Start API server (background)
- Wait for health check
- Run real-world test suite
- Upload test results artifact
- Comment on PR with summary
```

**Success Criteria**:
- [ ] Workflow runs on every PR
- [ ] Test results visible in PR
- [ ] Failing tests block merge
- [ ] CI execution <10 minutes

---

## 🟡 MEDIUM PRIORITY - Reporting & UX (P2)

### ☐ TODO #9: HTML Test Report Generator
**Status**: 📝 Ready to implement
**Priority**: P2 - Medium
**Effort**: 3 days
**Location**: `tools/test-report-generator/`

**Problem**: Test results only in JSON/terminal, no rich visualization
**Impact**: Hard to analyze test trends and failures
**Action Required**:
1. Create HTML report generator binary
2. Include test summary (pass/fail rates)
3. Add quality score trends chart
4. Add performance metrics graph
5. Add failure analysis section

**Success Criteria**:
- [ ] Generate HTML report from test session JSON
- [ ] Include charts and graphs
- [ ] Accessible via CI artifacts

---

### ☐ TODO #10: Metrics Dashboard
**Status**: 📝 Ready to implement
**Priority**: P2 - Medium
**Effort**: 2 days
**Location**: `docs/test-metrics-dashboard.html`

**Problem**: No historical trend visualization
**Impact**: Cannot track quality improvements/degradations over time
**Action Required**:
1. Create time-series metrics dashboard
2. Track quality scores over time
3. Track performance metrics over time
4. Track test pass/fail rates
5. Update automatically from CI

**Success Criteria**:
- [ ] Dashboard accessible to team
- [ ] Shows last 30 days of data
- [ ] Updates automatically

---

### ☐ TODO #11: Alerting System
**Status**: 📝 Ready to implement
**Priority**: P2 - Medium
**Effort**: 2 days
**Location**: `.github/workflows/test-alerts.yml`

**Problem**: No automated alerts on test failures
**Impact**: Failures may go unnoticed
**Action Required**:
1. Set up email/Slack notifications
2. Alert on test failures
3. Alert on quality degradations >10%
4. Alert on performance regressions >20%
5. Include failure details in alert

**Success Criteria**:
- [ ] Alerts sent on failures
- [ ] Alert includes actionable info
- [ ] Alert rate <5% false positives

---

## 📊 TODO Summary

| TODO | Priority | Status | Effort | Blocker |
|------|----------|--------|--------|---------|
| **#0: ToS-Compliant URLs** | 🔴 P0 | ✅ DONE | 1 day | None |
| **#1: TestHarness Integration** | 🔴 P0 | Ready | 2 days | None |
| **#2: Content Validation** | 🔴 P0 | Ready | 3 days | None |
| **#3: Baseline Management** | 🔴 P0 | Ready | 2 days | None |
| **#4: Activate Real API Tests** | 🔴 P0 | Ready | 1 day | None |
| **#5: Regression Detection** | 🟠 P1 | Ready | 3 days | #2, #3 |
| **#6: Performance Benchmarking** | 🟠 P1 | Ready | 3 days | #1 |
| **#7: Test URL Expansion** | 🟠 P1 | Ready | 2 days | #0, #1 |
| **#8: CI/CD Integration** | 🟠 P1 | Ready | 5 days | #1-#4 |
| **#9: HTML Report Generator** | 🟡 P2 | Ready | 3 days | #8 |
| **#10: Metrics Dashboard** | 🟡 P2 | Ready | 2 days | #8 |
| **#11: Alerting System** | 🟡 P2 | Ready | 2 days | #8 |

**Total Estimated Effort**: 28 days (5.5 weeks)
**Completed**: 1 day (TODO #0 ✅)

**Critical Path**:
```
Phase 1 (P0): #1 → #2 → #3 → #4 (8 days, parallel execution possible)
Phase 2 (P1): #5, #6, #7 → #8 (13 days total, some parallel)
Phase 3 (P2): #9 → #10 → #11 (7 days, can overlap)
```

---

## 🎯 Quick Reference

**Production Blocker**: TODOs #1-#4 (Real-world testing foundation)
**Scale Blocker**: TODOs #5-#8 (Automation and CI/CD)
**Quality of Life**: TODOs #9-#11 (Reporting and alerting)

**Current Maturity**: **3/10** (Mock testing only, no real-world validation)
**After Phase 1**: **6/10** (Real-world testing enabled)
**After Phase 2**: **8/10** (Production-ready with CI/CD)
**After Phase 3**: **9/10** (Enterprise-grade)

**Architecture Status**:
- ✅ **Mock Testing**: 8/10 - Comprehensive (156 assertions)
- ❌ **Real API Testing**: 2/10 - Tests exist but ignored
- ❌ **Content Validation**: 0/10 - No validation framework
- ❌ **Baseline Management**: 0/10 - No baseline storage
- ❌ **CI/CD Integration**: 3/10 - Mocks in CI only

---

## 📈 Implementation Timeline

### Phase 1: Foundation (1-2 weeks) - P0 🔴
**Goal**: Enable automated real-world testing

**Week 1**:
- Days 1-2: TODO #1 (TestHarness Integration)
- Days 3-5: TODO #2 (Content Validation)

**Week 2**:
- Days 1-2: TODO #3 (Baseline Management)
- Day 3: TODO #4 (Activate Real API Tests)
- Days 4-5: Integration testing and fixes

**Deliverable**: Can run `cargo test` and validate real extractions

---

### Phase 2: Quality & Automation (2-3 weeks) - P1 🟠
**Goal**: Regression detection and CI/CD integration

**Week 3-4**:
- Days 1-3: TODO #5 (Regression Detection)
- Days 4-6: TODO #6 (Performance Benchmarking)
- Days 7-8: TODO #7 (Test URL Expansion)

**Week 4-5**:
- Days 1-5: TODO #8 (CI/CD Integration)
- Days 6-7: End-to-end testing and refinement

**Deliverable**: Automated testing on every PR with regression detection

---

### Phase 3: Reporting & Observability (1 week) - P2 🟡
**Goal**: Rich reporting and trend analysis

**Week 6**:
- Days 1-3: TODO #9 (HTML Report Generator)
- Days 4-5: TODO #10 (Metrics Dashboard)
- Days 6-7: TODO #11 (Alerting System)

**Deliverable**: Production-grade test reporting and observability

---

## 🏗️ CLI Testing Architecture Overview

### Current State Analysis

**What Exists** ✅:
- **Mock Integration Tests**: 20 tests using WireMock
- **Mock E2E Tests**: 14 workflow tests
- **Test Harness**: Well-designed but unused (307 lines)
- **Test URLs**: 29 diverse URLs across 13 categories
- **CLI Client**: Production-ready HTTP client
- **Test Scripts**: Basic shell scripts (limited validation)

**What's Missing** ❌:
- Real-world test execution framework
- Content validation beyond pass/fail
- Baseline result storage and comparison
- Performance benchmarking
- Regression detection
- CI/CD integration for real tests
- Rich test reporting

### Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│              RipTide CLI Testing Architecture               │
└─────────────────────────────────────────────────────────────┘

Current State (Mock-Based):
┌──────────────────────┐
│   Mock Tests         │
│   ✅ 156 assertions  │
│                      │
│ • integration_tests  │
│ • e2e_tests          │
│ • real_world_tests   │
│   (using WireMock)   │
└──────────┬───────────┘
           │
           ▼
    ┌──────────────┐
    │  CLI Binary  │
    │  (riptide)   │
    └──────────────┘

Target State (Real-World):
┌──────────────────────┐       ┌──────────────────────┐
│   Test Harness       │       │  Baseline Manager    │
│   ✅ Designed        │       │  ❌ Missing          │
│   ❌ Not Used        │       │                      │
└──────────┬───────────┘       └──────────────────────┘
           │
           ▼
    ┌──────────────────────────────────────┐
    │          CLI Binary                   │
    │          + Content Validator          │
    │          + Regression Detector        │
    └──────────┬───────────────────────────┘
               │
               ▼
    ┌──────────────────────────────────────┐
    │     API Server / Real Websites       │
    │     (29 diverse test URLs)           │
    └──────────────────────────────────────┘
               │
               ▼
    ┌──────────────────────────────────────┐
    │     Result Validation & Storage      │
    │     + Performance Metrics            │
    │     + CI/CD Integration              │
    │     + HTML Reports & Dashboards      │
    └──────────────────────────────────────┘
```

### Component Status

| Component | Current | Target | Status |
|-----------|---------|--------|--------|
| **CLI Binary** | Production-ready | Stable | ✅ Complete |
| **HTTP Client** | Production-ready | Stable | ✅ Complete |
| **Mock Tests** | 156 assertions | Maintain | ✅ Complete |
| **Test Harness** | Exists, unused | Integrated | ❌ TODO #1 |
| **Content Validator** | None | Full validation | ❌ TODO #2 |
| **Baseline Manager** | None | Storage + comparison | ❌ TODO #3 |
| **Real API Tests** | 14 (ignored) | Active in CI | ❌ TODO #4 |
| **Regression Detector** | None | Automated | ❌ TODO #5 |
| **Performance Benchmarks** | None | Criterion-based | ❌ TODO #6 |
| **CI/CD Integration** | Mocks only | Full automation | ❌ TODO #8 |
| **Reporting** | JSON only | HTML + Dashboard | ❌ TODO #9-11 |

---

## 📋 Test Data Inventory

### ✅ 30 ToS-Compliant Test URLs (Updated 2025-10-13)

**⚠️ MIGRATION COMPLETE**: All non-compliant URLs removed. See `/docs/testing/TEST_URLS_MIGRATION_TABLE.md`

#### By Category (8 categories)

1. **Documentation (6)**:
   - MDN Web Docs (CC-BY-SA 2.5+)
   - Rust Documentation (MIT/Apache 2.0)
   - Python Docs (PSF License)
   - React Docs (CC BY 4.0) - **SPA**
   - GNU Bash Manual (GNU FDL)
   - TypeScript Handbook (Apache 2.0)

2. **Educational (4)**:
   - ArXiv.org Papers (Academic) - **Rate limit: 3s**
   - Khan Academy (CC BY-NC-SA)
   - Project Gutenberg (Public Domain)
   - OpenStax Textbooks (CC BY 4.0)

3. **Wiki (1)**:
   - Wikipedia (CC BY-SA 3.0) - **Rate limit: 0.2s**

4. **News/Articles (4)**:
   - Hacker News API (JSON)
   - Ars Technica RSS Feed
   - WordPress.org Blog (GPL/CC)
   - Internet Archive (Archive.org)

5. **Government (4)**:
   - USA.gov (Public Domain)
   - Data.gov Catalog (Public Domain)
   - NASA Image Library (Public Domain)
   - EU Parliament Open Data (24 languages)

6. **Tech Blogs (3)**:
   - GitHub Blog/Changelog
   - Dev.to Articles
   - freeCodeCamp News (BSD-3-Clause)

7. **Edge Cases - Testing Sites (8)**:
   - Example.com (Simple HTML baseline)
   - HTTPBin.org (HTTP testing, status codes)
   - JSONPlaceholder (JSON API mock)
   - Quotes to Scrape (Pagination)
   - Books to Scrape (E-commerce structure)
   - ScrapeThisSite (Dynamic/AJAX)
   - WebScraper.io Test Site (Heavy JavaScript)
   - httpstat.us (404/500 errors)

**Coverage Analysis**:
- ✅ **100% ToS-Compliant** (legally safe, zero risk)
- ✅ **20+ Edge Cases**: SPA, heavy JS, AJAX, pagination, errors, JSON API, RSS, multilingual
- ✅ **Diverse Content Types**: Documentation, academic, news, blogs, government, e-commerce
- ✅ **Multiple Licenses**: Public domain, CC, open source, built-for-testing
- ✅ **International**: Wikipedia (180+ langs), EU Parliament (24 langs)
- ✅ **Error Handling**: 404/500 status codes via httpstat.us
- ✅ **Rate Limit Ready**: ArXiv (3s), Wikipedia (0.2s) documented

---

## 🔍 Risk Assessment

### ✅ ELIMINATED RISKS

1. **~~Legal/ToS Violations~~** ✅ **RESOLVED**
   - ~~**Risk**: Lawsuit from Amazon, eBay, Twitter, NYT, etc.~~
   - ~~**Probability**: 70% (very high)~~
   - ~~**Impact**: Critical - legal action, IP bans~~
   - **Mitigation**: ✅ COMPLETED - Migrated to 30 ToS-compliant URLs (TODO #0)

### 🔴 Critical Risks

2. **No Content Validation**
   - **Risk**: Shipping CLI that returns wrong content
   - **Probability**: 60% (high)
   - **Impact**: Critical - loss of user trust
   - **Mitigation**: Implement TODO #2 (3 days)

3. **No Regression Detection**
   - **Risk**: Quality degradation goes unnoticed
   - **Probability**: 40% (medium)
   - **Impact**: High - gradual quality decline
   - **Mitigation**: Implement TODO #3, #5 (5 days)

### 🟡 Medium Risks

4. **Manual Test Execution**
   - **Risk**: Tests not run consistently
   - **Probability**: 70% (high)
   - **Impact**: Medium - delayed bug detection
   - **Mitigation**: Implement TODO #8 (5 days)

5. **Test Coverage Gaps**
   - **Risk**: Edge cases not covered
   - **Probability**: 50% (medium)
   - **Impact**: Medium - production bugs
   - **Mitigation**: Implement TODO #7 (2 days)

6. **Rate Limit Violations**
   - **Risk**: IP bans from ArXiv, Wikipedia
   - **Probability**: 30% (low-medium)
   - **Impact**: Medium - test failures
   - **Mitigation**: Implement rate limiting in TODO #7 (documented)

---

## ✅ Success Criteria

### Phase 1 (Foundation) - Metrics

- [ ] 90%+ of test URLs pass automated tests
- [ ] Content validation catches ≥1 real bug
- [ ] Baselines created for 10+ URLs
- [ ] Tests run in <5 minutes
- [ ] `cargo test` includes real-world tests
- [ ] Test session results persisted to JSON

### Phase 2 (Quality & Automation) - Metrics

- [ ] Regression detection catches ≥1 quality degradation
- [ ] Performance baseline established for 20+ URLs
- [ ] Test coverage for all 30 URLs (≥27 passing, 90%+)
- [ ] Quality score tracked over time
- [ ] Real-world tests run on every PR
- [ ] <10 minute CI pipeline execution
- [ ] Test failures block merges
- [ ] Rate limits respected (ArXiv: 3s, Wikipedia: 0.2s)

### Phase 3 (Reporting) - Metrics

- [ ] HTML reports generated automatically
- [ ] Historical trend data available (30 days)
- [ ] Alerts triggered on failures (<5% false positives)
- [ ] Dashboard accessible to team
- [ ] Test results visible in PR comments

---

## 💡 Immediate Actions (This Week)

### Day 1-2: Foundation Setup

1. **Create Test Harness Module**
   ```bash
   mkdir -p tests/common
   mv tests/webpage-extraction/cli-test-harness.rs tests/common/
   # Add to tests/common/mod.rs
   ```

2. **Define Baseline Schema**
   Create `test-results/baselines/schema.json`:
   ```json
   {
     "$schema": "http://json-schema.org/draft-07/schema#",
     "type": "object",
     "required": ["url", "expected_title", "min_quality_score"],
     "properties": {
       "url": {"type": "string"},
       "expected_title": {"type": "string"},
       "expected_keywords": {"type": "array", "items": {"type": "string"}},
       "min_quality_score": {"type": "number", "minimum": 0, "maximum": 1},
       "max_extraction_time_ms": {"type": "integer"}
     }
   }
   ```

### Day 3-4: Initial Implementation

3. **Create Initial Baselines**
   ```bash
   # Run extractions for 10 test URLs
   cargo run --bin generate-baselines -- \
     --urls test-urls.json \
     --output test-results/baselines/ \
     --count 10
   ```

4. **Enable Real API Tests**
   - Remove `#[ignore]` from Wikipedia, GitHub, confidence tests
   - Add API server startup script
   - Verify tests pass locally

### Day 5: Verification

5. **Run Full Test Suite**
   ```bash
   # Start API server
   cargo run --bin riptide-api &

   # Run real-world tests
   cargo test --test real_world_integration

   # Verify baselines exist
   ls -la test-results/baselines/
   ```

---

## 📚 Related Documentation

- 📄 **Full Assessment**: `/docs/architecture/cli-testing-infrastructure-assessment.md`
- 📄 **Test Harness Code**: `/tests/webpage-extraction/cli-test-harness.rs`
- 📄 **Test URLs** ✅: `/tests/webpage-extraction/test-urls.json` (30 ToS-compliant)
- 📄 **URL Migration Table** 🆕: `/docs/testing/TEST_URLS_MIGRATION_TABLE.md`
- 📄 **Safe URLs Guide** 🆕: `/docs/testing/SAFE_TEST_URLS_GUIDE.md`
- 📄 **CLI Production Readiness**: `/docs/cli-production-readiness.md`
- 📄 **Integration Testing Guide**: `/docs/testing/INTEGRATION_TEST_GUIDE.md`

---

## 🎯 Comparison to WASM Roadmap

**Similarities**:
- Both need real-world validation
- Both need regression testing frameworks
- Both need CI/CD integration
- Both need performance benchmarking

**Differences**:
- ✅ **CLI is simpler** - No browser complexities
- ✅ **CLI is faster** - No browser startup overhead
- ✅ **Better foundation** - Test harness already exists
- ✅ **Faster to implement** - 4-5 weeks vs 6-8 weeks for WASM

**Estimated Effort**:
- WASM Roadmap: 6-8 weeks (35+ days)
- CLI Roadmap: **4-5 weeks** (28 days) ⚡ **20% faster**

---

## 📝 Notes

- ✅ **TODO #0 COMPLETED**: All test URLs are now ToS-compliant (legal risk eliminated)
- All P0 tasks can be parallelized (8 days → potentially 4-5 days with 2 developers)
- Phase 1 critical for production confidence
- Phase 2 enables scaling and automation
- Phase 3 improves developer experience
- Test harness is production-ready, just needs integration
- 30 diverse ToS-compliant test URLs provide excellent coverage
- Rate limiting must be implemented (ArXiv: 3s, Wikipedia: 0.2s)

---

**Document Version**: 1.1
**Created**: 2025-10-13
**Last Updated**: 2025-10-13 (TODO #0 completed - ToS-compliant URLs)
**Next Review**: After Phase 1 completion
**Architecture Grade**: **Current 3/10 → Target 8/10**
**Legal Status**: ✅ **100% ToS-Compliant** (30/30 URLs safe)
