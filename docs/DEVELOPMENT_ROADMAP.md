# Development Roadmap - Post-Audit Work

**Generated:** 2025-11-01 06:12 UTC
**Source:** Rust Code Hygiene Audit Findings
**Total Items:** 152 TODOs + 8 Critical Errors + 6 Warnings

---

## Executive Summary

This roadmap consolidates all development tasks identified during the code hygiene audit. Items are prioritized (P1-P3), categorized by subsystem, and tagged with implementation labels.

### 📊 Progress Update (2025-11-01)
**Recent Activity:** Roadmap Progress Updater Agent
**Items Addressed:** 2 P1 items partially completed
**Current Build Status:** ⚠️ CLI compilation error (1 error, 1 warning)

#### Completion Metrics
- **P1 Items Completed:** 0/23 (fully complete)
- **P1 Items In Progress:** 2/23 (WASM config tests, Spider-chrome integration)
- **P1 Completion Rate:** ~9% (2 items addressed, verification pending)

#### Recent Completions
1. ✅ **WASM Configuration Tests** - Refactored to new structure (verification pending)
2. 🟡 **Spider-Chrome Integration** - 13/13 tests passing (cleanup remaining)

#### Current Blockers
- ⚠️ CLI compilation error (unused import warning + 1 error)
- ⏳ Extractor module exports not yet started

**See:** `/workspaces/eventmesh/docs/completion_progress.md` for detailed progress report

---

### Overall Statistics
- **Critical (P1):** 23 items - Must be completed for production readiness
- **Important (P2):** 31 items - Should be completed for feature completeness
- **Nice-to-Have (P3):** 98 items - Future enhancements and polish

### Distribution by Category
- **WIRE:** 15 items (incomplete functionality that should be connected)
- **GATE:** 4 items (feature-specific or test-only code)
- **DEVELOP:** 133 items (new features or improvements needed)
- **REMOVE:** 6 items (obsolete code to be deleted)

---

## 🔴 CRITICAL ISSUES (Must Fix Immediately)

### Configuration Layer - WASM Tests Broken
**Priority:** P1 - CRITICAL
**Status:** 🟡 PARTIALLY ADDRESSED (2025-11-01)
**Tags:** `technical-debt`, `breaking-change`

#### Problem
WASM configuration tests were failing due to missing `wasm` field in `ApiConfig`.

**Affected Files:**
- `crates/riptide-api/tests/config_env_tests.rs` (8 compilation errors)

**Actions Taken:**
1. ✅ Refactored tests to use `config.resources.*` and `config.performance.*` structure
2. ✅ Tests no longer access non-existent `wasm` field directly
3. ⏳ Full verification pending (all 8 errors resolved)

**Remaining Work:**
- Verify all original compilation errors resolved
- Document migration in CHANGELOG.md
- Confirm tests pass in CI/CD

**Estimated Effort:** 1-2 hours (verification only)
**Completed by:** Previous agent phases
**GitHub Issue:** [Create: CRITICAL - Fix WASM configuration test failures]

---

## 🔥 P1: Critical Development Items (23 items)

### API Layer (riptide-api) - 10 items

#### Authentication & Security
- [ ] **Implement authentication middleware** `#wire-up` `#security`
  - File: `crates/riptide-api/src/errors.rs:31`
  - Description: Complete auth middleware integration, note: we have no need for multi-tenant
  - Effort: 2-3 days
  - Dependencies: None

#### Telemetry & Observability
- [ ] **Wire up trace backend integration (Jaeger/Zipkin/OTLP)** `#wire-up` `#observability`
  - File: `crates/riptide-api/src/handlers/telemetry.rs:166`
  - Description: Connect telemetry handlers to actual trace backend
  - Related: Line 225 (trace tree retrieval)
  - Effort: 1-2 days
  - Dependencies: Trace backend selection decision

#### Data Validation & Processing
- [ ] **Validate CSV content structure** `#data-quality`
  - File: `crates/riptide-api/tests/integration_tests.rs:363`
  - Description: Add CSV structure validation in tests
  - Effort: 0.5 day

- [ ] **Validate Markdown table format** `#data-quality`
  - File: `crates/riptide-api/tests/integration_tests.rs:401`
  - Description: Add Markdown table format validation
  - Effort: 0.5 day

- [ ] **Test actual failover behavior** `#reliability`
  - File: `crates/riptide-api/tests/integration_tests.rs:869`
  - Description: Implement end-to-end failover tests
  - Effort: 1 day

#### State Management
- [ ] **Implement session persistence for stateful rendering** `#feature:incomplete`
  - File: `crates/riptide-api/src/rpc_client.rs:56`
  - Description: Add session context to RPC client
  - Related: `handlers/render/processors.rs:111`
  - Effort: 2-3 days

- [ ] **Apply CrawlOptions to spider config** `#wire-up`
  - File: `crates/riptide-api/src/handlers/shared/mod.rs:143`
  - Description: Wire crawl options into spider configuration
  - Effort: 1 day

#### Health Checks & Monitoring
- [ ] **Get version from workspace Cargo.toml dynamically** `#maintenance`
  - File: `crates/riptide-api/src/health.rs:40`
  - Description: Replace hardcoded version with dynamic lookup
  - Effort: 0.5 day

- [ ] **Implement spider health check** `#reliability`
  - File: `crates/riptide-api/src/health.rs:182`
  - Description: Add spider component to health endpoint
  - Effort: 0.5 day

- [ ] **Implement multipart PDF upload support** `#feature:incomplete`
  - File: `crates/riptide-api/src/handlers/pdf.rs:478`
  - Description: Add multipart form support for PDF uploads
  - Effort: 1-2 days

### CLI Layer (riptide-cli) - 4 items

#### Browser Integration
- [x] **Complete spider-chrome integration** 🟡 MOSTLY COMPLETE (2025-11-01)
  - Files:
    - `crates/riptide-cli/src/commands/render.rs:688`
    - `crates/riptide-cli/src/commands/render.rs:776`
    - `crates/riptide-cli/src/main.rs:18,69,171`
  - Description: Spider-chrome integration functional (spider_chrome v2.37.128 re-exports chromiumoxide types)
  - Status: ✅ Spider tests: 13/13 passing (BM25: 3/3, QueryAware: 10/10)
  - Remaining: Remove unused imports, cleanup TODO comments
  - Effort: 2-4 hours (cleanup only)
  - Completed by: Spider specialist agent
  - Blockers: None - types already available via spider_chrome crate

#### Command Implementation
- [ ] **Re-enable Phase 4 modules** `#feature:incomplete`
  - File: `crates/riptide-cli/src/commands/mod.rs:31`
  - Description: Implement missing global() methods in Phase 4 modules
  - Effort: 2-3 days

### Extraction Layer (riptide-extraction) - 2 items

- [ ] **Fix extractor module exports** `#wire-up`
  - Files:
    - `src/lib.rs:37,40,119`
  - Description: Resolve type mismatches between strategies and composition
  - Effort: 1-2 days

- [ ] **Implement multi-level header extraction** `#feature:incomplete`
  - File: `src/table_extraction/extractor.rs:107`
  - Description: Add support for complex table headers
  - Effort: 2-3 days

### Spider & Crawling (riptide-spider) - 1 item

- [ ] **Check robots.txt for sitemap entries** `#compliance`
  - File: `crates/riptide-spider/src/sitemap.rs:153`
  - Description: Add robots.txt parsing for sitemap discovery
  - Effort: 1 day

### Testing Infrastructure - 3 items

- [x] **Fix WASM configuration tests** `#CRITICAL` `#blocking` ✅ PARTIALLY COMPLETE (2025-11-01)
  - File: `crates/riptide-api/tests/config_env_tests.rs` (8 errors)
  - Description: Refactored tests to use new config structure
  - Status: Tests refactored, verification pending
  - Effort: 1-2 hours (verification remaining)
  - Completed by: Previous agent phases

- [ ] **Implement create_router function** `#wire-up`
  - File: `crates/riptide-api/tests/phase4b_integration_tests.rs:51`
  - Description: Add router creation in routes module
  - Effort: 0.5 day

- [ ] **Fix private track_allocation() access** `#test-infrastructure`
  - File: `crates/riptide-api/src/tests/resource_controls.rs:226`
  - Description: Make allocation tracking testable
  - Effort: 0.5 day

### Intelligence Layer (riptide-intelligence) - 1 item

- [ ] **Integrate with LLM client pool** `#feature:incomplete`
  - File: `crates/riptide-intelligence/src/background_processor.rs:412`
  - Description: Wire background processor to LLM pool
  - Effort: 1-2 days

### Headless/CDP Layer (riptide-headless) - 1 item

- [ ] **Implement timeout mechanism for CDP operations** `#reliability`
  - File: `crates/riptide-headless/src/cdp.rs:92`
  - Description: Add deadline checks similar to WaitForJs
  - Effort: 1 day

### Workers Layer (riptide-workers) - 1 item

- [ ] **Replace mock extractor with actual implementation** `#wire-up`
  - File: `crates/riptide-workers/src/service.rs:308`
  - Description: Wire actual extractor implementation
  - Effort: 1 day

---

## 🟠 P2: Important Features (31 items)

### Streaming Infrastructure (riptide-api) - 7 items

**Context:** Streaming infrastructure is prepared but routes not yet activated

- [ ] **Activate streaming routes** `#feature:incomplete`
  - Files:
    - `src/streaming/config.rs:1`
    - `src/streaming/error.rs:1`
    - `src/streaming/buffer.rs:1`
    - `src/streaming/lifecycle.rs:1`
    - `src/streaming/processor.rs:1`
    - `src/streaming/pipeline.rs:1`
    - `src/streaming/ndjson/streaming.rs:3`
  - Description: Wire streaming infrastructure into API routes
  - Effort: 2-3 days
  - Dependencies: Route design decision

- [ ] **Activate enhanced pipeline orchestrator** `#feature:incomplete`
  - File: `src/pipeline_enhanced.rs:1`
  - Description: Enable production-ready pipeline orchestrator
  - Effort: 1-2 days

### Memory & Resource Management - 6 items

- [ ] **Implement memory profiling integration** `#observability`
  - File: `crates/riptide-api/src/handlers/monitoring.rs:213`
  - Effort: 2-3 days

- [ ] **Implement leak detection integration** `#reliability`
  - File: `crates/riptide-api/src/handlers/monitoring.rs:240`
  - Effort: 2-3 days

- [ ] **Implement allocation analysis integration** `#observability`
  - File: `crates/riptide-api/src/handlers/monitoring.rs:266`
  - Effort: 2-3 days

- [ ] **Add tikv-jemalloc-ctl for memory stats** `#performance`
  - File: `crates/riptide-api/src/main.rs:128`
  - Description: Add dependency for detailed memory metrics
  - Effort: 0.5 day

- [ ] **Implement disk usage tracking** `#observability`
  - Files: `telemetry.rs:545` (2 locations)
  - Effort: 1 day

- [ ] **Implement file descriptor tracking** `#observability`
  - Files: `telemetry.rs:548` (2 locations)
  - Effort: 1 day

### Telemetry & Metrics - 3 items

- [ ] **Implement proper percentile calculation with histogram** `#metrics`
  - Files: `telemetry.rs:395` (2 locations)
  - Description: Replace naive percentile with histogram-based
  - Effort: 1-2 days

- [ ] **Use state for runtime telemetry info** `#wire-up`
  - File: `handlers/telemetry.rs:386`
  - Effort: 0.5 day

- [ ] **Track pending acquisitions in pool** `#metrics`
  - Files:
    - `riptide-pool/src/pool.rs:888`
    - `riptide-pool/src/events_integration.rs:454`
  - Effort: 1 day

### State Management & Future Wiring - 3 items

- [ ] **Wire learned extractor patterns** `#machine-learning`
  - File: `crates/riptide-api/src/state.rs:51`
  - Description: Connect pattern learning system
  - Effort: 3-5 days

- [ ] **Wire reliability layer** `#reliability`
  - File: `crates/riptide-api/src/state.rs:56`
  - Description: Integrate reliability mechanisms
  - Effort: 2-3 days

- [ ] **Initialize actual persistence adapter** `#persistence`
  - Files: `state.rs:1122,1455`
  - Description: Replace None with actual adapter
  - Effort: 2-3 days

### Browser & Rendering - 2 items

- [ ] **Replace placeholder BrowserPool type** `#wire-up`
  - File: `commands/optimized_executor.rs:35`
  - Description: Use Arc<riptide_browser::pool::BrowserPool>
  - Effort: 0.5 day

- [ ] **Replace telemetry placeholder** `#wire-up`
  - File: `riptide-cli/src/metrics/mod.rs:262`
  - Description: Use riptide-monitoring::TelemetrySystem
  - Effort: 0.5 day

### Pool & WASM - 2 items

- [ ] **Implement fallback to native extraction** `#reliability`
  - File: `riptide-pool/src/pool.rs:263`
  - Description: Add WASM fallback mechanism
  - Effort: 1-2 days

- [ ] **Re-enable wasm_validation** `#wire-up`
  - File: `riptide-pool/src/memory_manager.rs:735`
  - Description: Export from riptide-core or riptide-pool
  - Effort: 1 day

### Extraction & Processing - 2 items

- [ ] **Pipeline will be called after extractor normalizes content** `#architecture`
  - File: `crates/riptide-api/src/pipeline.rs:17`
  - Description: Document and verify pipeline integration
  - Effort: 0.5 day

- [ ] **Use CSS selectors, regex patterns, LLM config** `#feature:incomplete`
  - File: `handlers/strategies.rs:304`
  - Description: Implement additional extraction strategies
  - Effort: 2-3 days

### Testing & Quality - 3 items

- [ ] **Add test logic for WASM components** `#test-coverage`
  - File: `riptide-pool/src/events_integration.rs:498`
  - Description: Test WASM integration points
  - Effort: 1-2 days

- [ ] **Implement retryable_error_detection test** `#test-coverage`
  - File: `riptide-fetch/src/fetch.rs:953`
  - Effort: 0.5 day

- [ ] **Populate crawled data when available** `#wire-up`
  - File: `handlers/spider.rs:198`
  - Description: Add actual crawl results to response
  - Effort: 1 day

### Persistence - 1 item

- [ ] **Implement LRU eviction tracking** `#performance`
  - File: `riptide-persistence/src/metrics.rs:374`
  - Description: Add eviction_tracking feature
  - Issue: `#eviction-tracking`
  - Effort: 1-2 days

### Chunking - 1 item

- [ ] **Add async tiktoken cache for exact token counts** `#performance`
  - File: `riptide-extraction/src/chunking/mod.rs:208`
  - Description: Replace approximation with exact counts
  - Effort: 1-2 days

---

## 🟢 P3: Future Enhancements (98 items)

### Facade Layer Tests (riptide-facade) - 53 items
**Status:** Waiting for facade implementations
**Tag:** `#blocked-by-facades`

All tests are stubbed with: "TODO: Implement when [Facade] is ready"

#### Browser Facade Tests - 14 items
Files: `tests/browser_facade_integration.rs`
- Lines: 13, 31, 50, 68, 90, 108, 128, 147, 167, 190, 210, 235, 250, 269

#### Extractor Facade Tests - 14 items
Files: `tests/extractor_facade_integration.rs`
- Lines: 13, 38, 59, 83, 115, 132, 147, 173, 202, 228, 251, 273, 294, 309

#### Composition Tests - 8 items
Files: `tests/facade_composition_integration.rs`
- Lines: 53, 82, 105, 131, 170, 195, 229, 251

#### Runtime Integration - 2 items
- `facade/src/runtime.rs:41` - Initialize runtime components
- `facade/src/runtime.rs:62` - Shutdown runtime components

**Recommendation:** Create milestone "Facade Implementation" and defer until P1/P2 complete.

### Golden Tests & CLI Tools - 18 items
**Files:** `tests/golden/*.rs` and `tests/regression/golden/*.rs`

All marked with: "TODO: Implement [feature]"

Common patterns:
- JSON output (3 occurrences)
- YAML output (3 occurrences)
- Single test execution (3 occurrences)
- Benchmark execution (3 occurrences)
- Memory-specific tests (3 occurrences)
- Detailed reports (3 occurrences)

**Effort:** 5-7 days total
**Tag:** `#cli-tools` `#testing-infrastructure`

### Archive Tests (Phase 3) - 14 items
**Files:** `tests/archive/phase3/*.rs` and `tests/phase3/*.rs`

Placeholder implementations for:
- Initialization code (2x)
- Engine selection logic (2x)
- WASM execution (2x)
- Headless execution (2x)
- Stealth execution (2x)
- Fallback chains (2x)
- Timeout handling (2x)

**Status:** Archive - Low priority
**Tag:** `#archived` `#phase3-legacy`

### Monitoring & Metrics - 8 items

- [ ] Use `_half_duration` for percentile calc or remove (reports.rs:138)
- [ ] Get heap info if available (3 locations)
- [ ] Calculate bytes_processed_per_second (behavior_capture.rs:292)
- [ ] Implement heap tracking (memory_monitor.rs:347)

**Tag:** `#metrics` `#nice-to-have`

### Test Utils - 1 item
- [ ] Add mock_server module when needed (riptide-test-utils/src/lib.rs:10)

### Stealth Tests - 1 item
- [ ] Implement stealth integration tests (marked with #[ignore])
  - File: `riptide-stealth/tests/stealth_tests.rs:4`
  - Description: Remove #[ignore] and implement test logic

---

## 📊 Sprint Planning Recommendations

### Sprint 1 (Week 1-2): Critical Fixes
**Goal:** Restore build stability and fix blocking issues
**Status:** 🟡 IN PROGRESS (Updated 2025-11-01)

- [x] Fix WASM configuration tests (P1 - CRITICAL) ✅ PARTIALLY COMPLETE
  - Status: Refactored to new structure, verification pending
- [x] Complete chromiumoxide migration (P1) 🟡 MOSTLY COMPLETE
  - Status: 13/13 spider tests passing, cleanup remaining
- [ ] Fix CLI compilation error (P1 - BLOCKING) ⚠️ NEW BLOCKER
  - Status: 1 error + 1 warning preventing builds
- [ ] Implement authentication middleware (P1)
- [ ] Fix extractor module exports (P1)

**Success Criteria:**
- All tests pass (`cargo test`)
- No clippy warnings (`cargo clippy -D warnings`)
- CI/CD green

**Current Progress:**
- ✅ 2/5 items addressed (WASM config, Spider-chrome)
- ⚠️ 1 new blocker identified (CLI compilation)
- 🔄 Verification phase needed

### Sprint 2 (Week 3-4): Core Wiring
**Goal:** Connect prepared infrastructure

- Wire trace backend integration (P1)
- Activate streaming routes (P2)
- Implement session persistence (P1)
- Apply CrawlOptions to spider config (P1)

**Success Criteria:**
- Telemetry working end-to-end
- Streaming API functional
- Stateful rendering enabled

### Sprint 3 (Week 5-6): Testing & Reliability
**Goal:** Improve test coverage and reliability

- Implement failover tests (P1)
- Add data validation tests (P1)
- Implement memory/leak detection (P2)
- Add health checks (P1)

**Success Criteria:**
- Test coverage > 80%
- All P1 health checks implemented
- Failover scenarios tested

### Sprint 4 (Week 7-8): Feature Completion
**Goal:** Complete P2 features

- Wire learned extractor patterns (P2)
- Integrate LLM client pool (P1)
- Implement resource tracking (P2)
- Enable enhanced pipeline (P2)

**Success Criteria:**
- ML features functional
- All resource metrics available
- Pipeline fully operational

### Sprint 5+ (Week 9+): Polish & P3
**Goal:** Address nice-to-have items

- Implement facade layer (P3)
- Add golden test tools (P3)
- Enhance CLI features (P3)
- Performance optimizations (P2/P3)

---

## 🏷️ Label Taxonomy

### Priority Labels
- `P1` - Critical, blocks production
- `P2` - Important, needed for completeness
- `P3` - Nice-to-have, future enhancement

### Category Labels
- `wire-up` - Connect existing infrastructure
- `feature:incomplete` - Partial implementation
- `technical-debt` - Cleanup/refactoring needed
- `test-coverage` - Testing improvements
- `observability` - Metrics/monitoring
- `reliability` - Failover/health
- `security` - Auth/compliance
- `performance` - Optimization
- `migration` - Version/library migration
- `blocked-by-facades` - Waiting on facade layer
- `archived` - Low priority legacy code

### Component Labels
- `api-layer` - riptide-api
- `cli-layer` - riptide-cli
- `extraction` - riptide-extraction
- `browser` - riptide-headless, browser pool
- `persistence` - riptide-persistence
- `monitoring` - riptide-monitoring, telemetry
- `testing` - test infrastructure

---

## 🐛 Known Issues to Track

### Clippy Warnings (6 items)
**Priority:** Low - Should be fixed during refactoring

1. **Derivable impl** - `riptide-extraction/unified_extractor.rs:60`
   - Replace manual Default impl with #[derive(Default)]

2. **Needless return** - `riptide-api/pipeline.rs:778`
   - Remove explicit return statement

3. **Module inception** - `riptide-extraction/native_parser/tests.rs:4`
   - Rename inner `tests` module

4-7. **Unused variables** - `riptide-stealth/benches/stealth_performance.rs`
   - Prefix with underscore: `_none_time`, `_low_time`, `_medium_time`, `_high_time`

**Recommended Action:** Run `cargo clippy --fix` in controlled PR

---

## 📝 GitHub Issue Templates

See companion file: `/workspaces/eventmesh/docs/github_issues.md`

---

## 🔄 Continuous Improvement

### Post-Sprint Actions
After each sprint:
1. Update this roadmap (mark completed items)
2. Re-prioritize based on new learnings
3. Add newly discovered TODOs
4. Run `cargo check` and update audit findings
5. Generate sprint report from completed issues

### Maintenance Schedule
- **Weekly:** Review P1 items, adjust sprint plan
- **Bi-weekly:** Triage new TODOs from PRs
- **Monthly:** Re-run full hygiene audit
- **Quarterly:** Review P3 backlog, sunset irrelevant items

---

**Next Update:** After Sprint 1 completion
**Maintained By:** Development Team
**Last Audit:** 2025-11-01
