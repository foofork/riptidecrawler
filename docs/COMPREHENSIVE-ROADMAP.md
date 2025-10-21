# EventMesh/Riptide Completion Roadmap

**Version:** 3.0 (Consolidated)
**Date:** 2025-10-21
**Status:** 🟢 30% Complete (adjusted for CLI migration)
**Target:** 2026-05-02 (27.0 weeks total, +12 weeks for CLI migration)

---

## 📋 Executive Summary

**Mission:** Complete EventMesh/Riptide to 100% production-ready status with spider-chrome migration, comprehensive testing, and deployment readiness.

**Current Progress:** 40% complete (2.5 of 8 phases done)
- ✅ Phase 1: Compilation fixed (267 errors → 0)
- ✅ Phase 2: Spider-chrome migration complete (626/630 tests = 99.4%)
- ✅ Phase 4 Task 4.0: Global singletons implemented
- 📅 Phase 3: Ready to start (3 days)
- 🟢 Phase 4 Task 4.1: Load testing unblocked

**Timeline:** 27.0 weeks total, completing 2026-05-02 (+12 weeks for critical CLI architecture migration)

| Phase | Duration | Status | Completion |
|-------|----------|--------|------------|
| Phase 1: Compilation Fix | 1.2 weeks | ✅ Complete | 2025-10-20 |
| Phase 2: Spider-chrome Migration | 4.8 weeks | ✅ Complete | 2025-10-20 |
| Phase 3: Architecture Cleanup | 1.0 week | 📅 Ready | - |
| Phase 4: Production Validation | 1.2 weeks | 🟢 50% Done | Task 4.0: 2025-10-21 |
| Phase 5: CLI Architecture Migration | 12.0 weeks | 🔴 CRITICAL | - |
| Phase 6: Test Infrastructure | 2.4 weeks | 🔄 Pending | - |
| Phase 7: Code Quality | 1.4 weeks | 🔄 Pending | - |
| Phase 8: Documentation | 2.4 weeks | 🔄 Pending | - |
| Phase 9: Client Libraries | 1.0 week | 🔄 Pending | - |

---

## ✅ Completed Work (Phases 1-2, Task 4.0)

### Phase 1: Critical Bug Fixes ✅ Complete (2025-10-20)
- **Duration:** 1.2 weeks
- **Achieved:** 267 compilation errors fixed (255 persistence + 7 intelligence + 5 API)
- **Results:** 0 errors, <50 warnings, 626/630 tests passing (99.4%)
- **Key Files Fixed:** render/mod.rs, render/strategies.rs, persistence/sqlite tests, intelligence mocks
- **Reference:** `/docs/hive/p1-completion-report.md`

### Phase 2: Spider-Chrome Migration ✅ Complete (2025-10-20)
- **Duration:** 4.8 weeks
- **Achieved:** Full spider-chrome integration (6 core files migrated, ~5,490 lines)
- **Results:** 626/630 tests passing, browser pool optimized, CDP performance improved
- **Key Files:** engine/pool.rs, engine/cdp_pool.rs, headless/pool.rs, headless/cdp_pool.rs
- **Features Enabled:** Screenshots, PDFs, network interception
- **Note:** 162 chromiumoxide references remain - INTENTIONAL (spider-chrome exports for compatibility)
- **Reference:** `/docs/hive/phase2-completion-report.md`

### Phase 4 Task 4.0: Global Singletons ✅ Complete (2025-10-21)
- **Duration:** 35 minutes (2 days ahead of schedule)
- **Achieved:** 3 global singleton methods implemented
  - EngineSelectionCache::get_global() - `engine_cache.rs:13-16, 35-38`
  - WasmCache::get_global() - `wasm_cache.rs:188-197` (already existed)
  - PerformanceMonitor::get_global() - `performance_monitor.rs:197-200, 207-212`
- **Results:** OptimizedExecutor enabled, 10+ integration tests, 20-thread stress tests passing
- **Impact:** Unblocked Phase 4 Task 4.1 (load testing)
- **Reference:** `/docs/hive/GLOBAL-SINGLETONS-DEPLOYMENT-SUMMARY.md`

---

## 📅 Phase 3: Architecture Cleanup & Quality Baseline (5 days) - READY TO START

**Objective:** Consolidate browser crates and establish quality metrics
**Dependencies:** Phase 2 complete ✅
**Risk:** MEDIUM - Crate reorganization requires careful migration
**Timeline:** 1.0 week (5 days)
**Priority:** 🔴 CRITICAL - Eliminates 56% code duplication, unblocks Phase 4

### Task 3.0: Browser Crate Consolidation (3 days) 🔴 HIGHEST PRIORITY

**Objective:** Consolidate 4 overlapping browser crates → 2 clean crates
**Reference:** `/docs/analysis/crate-architecture-assessment.md`

**Problem Identified:**
- 56% code duplication (~3,400 lines) between riptide-engine and riptide-headless
- riptide-headless is 100% re-export wrapper (no unique functionality)
- riptide-browser-abstraction adds unused abstraction (904 LOC for YAGNI)
- riptide-headless-hybrid overlaps with engine launcher

**Solution:**
```
BEFORE (4 crates, 10,520 LOC):          AFTER (2 crates, 8,000 LOC):
├── riptide-engine                      ├── riptide-browser (NEW)
├── riptide-headless                    │   ├── pool/
├── riptide-headless-hybrid             │   ├── cdp/
└── riptide-browser-abstraction   →     │   ├── launcher/
                                        │   ├── http_api/
                                        │   └── stealth/
                                        └── riptide-stealth (existing)
```

**Day 1: Scaffolding & Core Migration (8 hours)**
1. Create `riptide-browser` crate structure (1 hour)
   ```bash
   cargo new --lib crates/riptide-browser
   mkdir -p crates/riptide-browser/src/{pool,cdp,launcher,http_api}
   ```

2. Copy core implementations from riptide-engine (3 hours)
   - `pool.rs` → `riptide-browser/src/pool/mod.rs` (1,363 lines)
   - `cdp_pool.rs` → `riptide-browser/src/cdp/mod.rs` (1,630 lines)
   - `launcher.rs` → `riptide-browser/src/launcher/mod.rs` (672 lines)
   - `models.rs` → `riptide-browser/src/models.rs` (400 lines)

3. Merge hybrid launcher features (2 hours)
   - Integrate riptide-headless-hybrid/launcher.rs features
   - Consolidate stealth middleware into launcher/stealth.rs
   - Remove duplicate code

4. Move HTTP API from riptide-headless (2 hours)
   - `headless/src/cdp.rs` → `browser/src/http_api/mod.rs`
   - `headless/src/dynamic.rs` → `browser/src/launcher/dynamic.rs`

**Day 2: Remove Abstraction & Integration (8 hours)**
1. Remove browser-abstraction layer (2 hours)
   - Replace trait usage with direct spider_chrome types
   - Update imports: `BrowserEngine` → direct `Browser` usage
   - Delete `riptide-browser-abstraction` crate

2. Update Cargo.toml dependencies (1 hour)
   ```toml
   [dependencies]
   riptide-stealth = { path = "../riptide-stealth" }
   riptide-types = { path = "../riptide-types" }
   riptide-config = { path = "../riptide-config" }
   spider_chrome = { workspace = true }
   tokio = { workspace = true }
   # ... existing dependencies
   ```

3. Configure public API exports (2 hours)
   - Define clear module structure
   - Create comprehensive lib.rs with re-exports
   - Document public API surface

4. Initial compilation check (3 hours)
   ```bash
   cargo build -p riptide-browser
   cargo test -p riptide-browser --lib
   ```
   - Fix compilation errors
   - Resolve import conflicts

**Day 3: Consumer Migration & Validation (8 hours)**
1. Update riptide-api dependencies (2 hours)
   - `Cargo.toml`: `riptide-headless` → `riptide-browser`
   - Update imports across all files
   - Test compilation

2. Update riptide-cli dependencies (2 hours)
   - `Cargo.toml`: `riptide-headless` → `riptide-browser`
   - Update command implementations
   - Test compilation

3. Update riptide-facade dependencies (2 hours)
   - Remove: `riptide-headless`, `riptide-engine`, `riptide-headless-hybrid`
   - Add: `riptide-browser`
   - Simplify facade implementations

4. Full workspace validation (2 hours)
   ```bash
   cargo build --workspace
   cargo test --workspace
   cargo clippy --workspace
   ```
   - Verify 0 compilation errors
   - Ensure 626/630 tests still pass (99.4%)
   - Check for new warnings

**Post-Consolidation Tasks (Days 4-5):**

**Day 4: Deprecation & Documentation (4 hours)**
1. Mark old crates deprecated (1 hour)
   - Add `#[deprecated]` attributes to old crate lib.rs
   - Update Cargo.toml with deprecation notices
   - Create DEPRECATED.md in each old crate

2. Create migration guide (2 hours)
   - Document import path changes
   - Provide before/after examples
   - List breaking changes (none expected)

3. Update architecture documentation (1 hour)
   - Update `/docs/analysis/crate-architecture-map.md`
   - Create ADR (Architecture Decision Record)
   - Update README files

**Day 5: Testing & Cleanup (4 hours)**
1. Comprehensive testing (2 hours)
   - Unit tests: `cargo test -p riptide-browser`
   - Integration tests: `cargo test --workspace`
   - Performance benchmarks (no regression)

2. Compilation time validation (1 hour)
   ```bash
   cargo clean
   time cargo build --workspace
   # Target: ≤30s for browser stack (33% improvement)
   ```

3. Final cleanup (1 hour)
   - Remove commented code
   - Update CHANGELOG.md
   - Prepare for Phase 4

**Success Criteria:**
- ✅ riptide-browser crate created and functional
- ✅ All 626/630 tests still passing (99.4%)
- ✅ Workspace compiles with 0 errors
- ✅ 3 consumer crates migrated (api, cli, facade)
- ✅ Old crates deprecated with migration guide
- ✅ Compilation time ≤30s (33% improvement)
- ✅ -2,520 LOC (24% reduction)
- ✅ Zero circular dependencies

**Deliverables:**
- `riptide-browser` crate (consolidated implementation)
- Migration guide for consumers
- Architecture Decision Record (ADR)
- Updated documentation
- Deprecated old crates (kept for 1 release)

**Risk Mitigation:**
- All consumers are internal (workspace-only)
- 626 tests validate functionality
- Incremental migration (one consumer at a time)
- Keep deprecated crates for 1 release cycle

---

### Task 3.1: Quality Metrics Baseline (1 day)
- Document current state: 142 dead_code warnings (intentional), 99.4% test pass rate
- Create quality dashboard with compilation time, warning trends
- Establish baseline for future comparison

### Task 3.2: Documentation Updates (1 day)
- Update architecture docs with Phase 2 completion details
- Document spider-chrome integration patterns
- Update API documentation
- Create Phase 1-2 completion summary

**Phase 3 Success Criteria:**
- ✅ Browser crates consolidated (4 → 2)
- ✅ Quality metrics documented
- ✅ Test baseline established (626/630 = 99.4%)
- ✅ Documentation updated
- ✅ Workspace builds with 0 errors
- ✅ 24% LOC reduction achieved

**Phase 3 Deliverables:**
- riptide-browser crate (consolidated)
- Migration guide
- ADR documentation
- Quality metrics baseline report
- Updated architecture documentation

---

## 🟢 Phase 4: Production Validation (6 days) - 50% COMPLETE

**Objective:** Production readiness validation at scale
**Dependencies:** Phase 3 complete
**Risk:** MEDIUM - May discover issues
**Timeline:** 1.2 weeks (6 days)
**Status:** Task 4.0 done ✅, Task 4.1 ready 📅

### Task 4.1: Load Testing - 10,000+ Concurrent Sessions (3.6 days) 📅 READY
**Owner:** Performance Engineer
**Status:** UNBLOCKED - Global singletons implemented

**Subtasks:**
1. Test environment setup (4 hours)
   - Configure load testing infrastructure
   - Set up monitoring dashboards
   - Prepare test scenarios
   - Validate OptimizedExecutor operational

2. Execute load tests (2 days)
   - Ramp up to 10,000 concurrent sessions
   - Monitor memory usage (<2GB per 1000 sessions target)
   - Track latency (P50/P95/P99, target P95 <500ms)
   - Identify bottlenecks
   - Use OptimizedExecutor for advanced scenarios

3. Analysis and optimization (4 hours)
   - Analyze results
   - Identify issues
   - Create optimization plan if needed

**Success Criteria:**
- ✅ 10,000+ concurrent sessions handled
- ✅ Memory usage <2GB per 1000 sessions
- ✅ P95 latency <500ms
- ✅ No connection leaks
- ✅ OptimizedExecutor performance validated

### Task 4.2: Production Readiness Review (3.6 days)
**Owner:** Reviewer Agent + Security Specialist

**Subtasks:**
1. Security audit (1 day)
   - Review browser security settings
   - Validate stealth configuration
   - Check for vulnerabilities
   - OWASP top 10 validation

2. Performance benchmarking (1 day)
   - Final performance baseline
   - Throughput testing (target: 25 req/s)
   - Memory profiling (target: 420MB/hour)
   - CPU profiling

3. Error handling validation (4 hours)
   - Test error scenarios
   - Validate graceful degradation
   - Test recovery mechanisms

4. Documentation review (4 hours)
   - Final documentation check
   - Ensure completeness
   - Validate accuracy

**Success Criteria:**
- ✅ Security audit passed
- ✅ Performance meets targets
- ✅ Error handling robust
- ✅ Documentation complete

**Phase 4 Deliverables:**
- Load test results (10k+ sessions)
- Security audit report
- Performance benchmark report
- Production readiness certification

---

## 🔴 Phase 5: CLI-to-Library Architecture Migration (12 weeks) - CRITICAL

**Objective:** Relocate 9,270 LOC of business logic from CLI to library crates (66% → 12%)
**Dependencies:** Phase 4 complete
**Risk:** HIGH - Major architectural refactoring affecting all consumers
**Timeline:** 12.0 weeks (60 days)
**Priority:** 🔴 CRITICAL - Blocks library-first usage, Python bindings, WASM modules, performance parity
**Reference:** `/docs/hive/CLI-RELOCATION-EXECUTIVE-SUMMARY.md`

### Problem Statement

**Current State:** CLI contains 13,782 LOC with 66% business logic (9,100+ LOC)
**Industry Standard:** Best-in-class Rust CLIs maintain <15% business logic
**Riptide Status:** 5-11x worse than standards (ripgrep: 8%, cargo: 12%, fd: 6%)

**Critical Issues:**
- ❌ Cannot use Riptide as library without CLI dependency
- ❌ API server must duplicate orchestration logic (2-10x performance gap)
- ❌ No Python bindings possible
- ❌ No WASM modules possible
- ❌ Engine selection logic duplicated (60+ lines in 2 places)
- ❌ 8 global singletons in CLI that should be in libraries

**Impact:** Blocks third-party integration, creates performance inequality, prevents multi-interface support

### Phase 5.1: Critical Infrastructure Extraction (4 weeks) - P0

**Objective:** Extract 8 critical modules (3,050 LOC) to new `riptide-optimization` crate + facade
**Timeline:** Weeks 1-4
**Effort:** 9 days with 1-2 engineers

#### Week 1: Create riptide-optimization Crate + Caching (5 days)

**Day 1-2: Scaffolding & Cache Migration**
- [ ] Create `riptide-optimization` crate structure
  ```bash
  cargo new --lib crates/riptide-optimization
  mkdir -p crates/riptide-optimization/src/{engine,wasm,timeout,metrics}
  ```
- [ ] Move `engine_cache.rs` (211 LOC) → `optimization/src/engine/cache.rs`
- [ ] Move `wasm_cache.rs` (282 LOC) → `optimization/src/wasm/cache.rs`
- [ ] Move `wasm_aot_cache.rs` (497 LOC) → `optimization/src/wasm/aot.rs`
- [ ] Configure Cargo.toml dependencies
- [ ] Build and test: `cargo build -p riptide-optimization`

**Day 3-4: Timeout & Monitoring**
- [ ] Move `adaptive_timeout.rs` (536 LOC) → `optimization/src/timeout/adaptive.rs`
- [ ] Move `performance_monitor.rs` (256 LOC) → `optimization/src/metrics/performance.rs`
- [ ] Create `OptimizationManager` unified API
- [ ] Integration testing

**Day 5: Browser Pool Migration**
- [ ] Move `browser_pool_manager.rs` (384 LOC) → `riptide-browser/src/pool/manager.rs`
- [ ] Update `riptide-browser` exports
- [ ] Compilation validation

#### Week 2: Engine Selection Consolidation (5 days)

**Day 6-7: Merge Duplicate Logic**
- [ ] Audit duplicate engine selection (2 locations, 60+ lines each)
- [ ] Create unified `EngineSelector` in `facade/src/engine/selection.rs`
- [ ] Merge logic from:
  - `cli/commands/engine_fallback.rs::analyze_content_for_engine()`
  - `cli/commands/extract.rs::Engine::gate_decision()`
- [ ] Add confidence scoring and caching integration
- [ ] Unit tests for engine selection heuristics

**Day 8-9: Engine Selection Integration**
- [ ] Update `optimized_executor.rs` to use new `EngineSelector`
- [ ] Remove duplicate code from CLI
- [ ] Integration tests across all engine types
- [ ] Validate performance (no regression)

**Day 10: Week 2 Validation**
- [ ] Run full test suite: `cargo test --workspace`
- [ ] Performance benchmarks
- [ ] Update documentation

#### Week 3-4: ExecutorFacade Creation (10 days)

**Day 11-12: Create ExecutorFacade**
- [ ] Create `facade/src/facades/executor.rs`
- [ ] Implement `ExecutorFacade` with `OptimizationManager` integration
- [ ] Move core orchestration logic from `cli/commands/optimized_executor.rs`
- [ ] Implement `.extract()` and `.render()` methods

**Day 13-14: Extraction Orchestration**
- [ ] Implement WASM extraction in facade
- [ ] Implement headless extraction in facade
- [ ] Implement raw HTTP extraction in facade
- [ ] Add adaptive timeout integration
- [ ] Add engine caching integration

**Day 15-16: Rendering Orchestration**
- [ ] Implement browser pool checkout/checkin
- [ ] Implement stealth configuration
- [ ] Implement screenshot/PDF capture
- [ ] Add performance monitoring

**Day 17-18: CLI Integration & Testing**
- [ ] Update `cli/commands/extract.rs` to use `ExecutorFacade`
- [ ] Update `cli/commands/render.rs` to use `ExecutorFacade`
- [ ] Simplify CLI commands to ~150 LOC each
- [ ] Comprehensive integration testing

**Day 19-20: Phase 5.1 Validation**
- [ ] All tests passing: `cargo test --workspace`
- [ ] Performance benchmarks (within 5% of baseline)
- [ ] CLI commands still functional (backward compatibility)
- [ ] Documentation updates

**Phase 5.1 Success Criteria:**
- ✅ `riptide-optimization` crate created (2,050 LOC)
- ✅ Browser pool moved to `riptide-browser` (384 LOC)
- ✅ Engine selection consolidated in facade (640 LOC)
- ✅ `ExecutorFacade` provides unified API (600 LOC)
- ✅ All modules have >70% test coverage
- ✅ CLI commands still functional
- ✅ 8 global singletons now in library crates

---

### Phase 5.2: Core Business Logic Extraction (4 weeks) - P1

**Objective:** Extract core workflows (3,650 LOC) to eliminate CLI/API duplication
**Timeline:** Weeks 5-8
**Effort:** 8 days with 1-2 engineers

#### Week 5-6: Extraction & Rendering Workflows (10 days)

**Day 21-22: ExtractionFacade**
- [ ] Create `facade/src/facades/extraction.rs`
- [ ] Extract logic from `cli/commands/extract.rs` (680 LOC of business logic)
- [ ] Implement `.extract_local()`, `.extract_headless()`, `.extract_direct()`
- [ ] Integration with `ExecutorFacade`
- [ ] Unit tests

**Day 23-24: RenderingFacade**
- [ ] Create `facade/src/facades/rendering.rs`
- [ ] Extract logic from `cli/commands/render.rs` (730 LOC of business logic)
- [ ] Implement `.render()`, `.capture_screenshot()`, `.generate_pdf()`
- [ ] Browser pool integration
- [ ] Unit tests

**Day 25-26: CLI Command Simplification**
- [ ] Refactor `extract.rs`: 972 LOC → ~150 LOC (84% reduction)
- [ ] Refactor `render.rs`: 980 LOC → ~150 LOC (85% reduction)
- [ ] Pure arg parsing and output formatting only
- [ ] All business logic via facade
- [ ] Smoke tests

**Day 27-28: API Integration**
- [ ] Update API server to use `ExtractionFacade`
- [ ] Update API server to use `RenderingFacade`
- [ ] Remove duplicated orchestration logic
- [ ] Integration tests
- [ ] Performance validation (2-10x speedup expected)

**Day 29-30: Week 5-6 Validation**
- [ ] All tests passing
- [ ] CLI/API use same logic (guaranteed consistency)
- [ ] Performance benchmarks
- [ ] Documentation

#### Week 7: Intelligence Modules (5 days)

**Day 31-32: Domain Profiling**
- [ ] Create `intelligence/src/domain/profile.rs`
- [ ] Extract from `cli/commands/domain.rs` (820 LOC of business logic)
- [ ] Implement `DomainProfile`, `SiteBaseline`, `DriftDetector`
- [ ] Unit tests with mock data
- [ ] Integration tests

**Day 33-34: Schema Management**
- [ ] Create `intelligence/src/schema/definition.rs`
- [ ] Extract from `cli/commands/schema.rs` (720 LOC of business logic)
- [ ] Implement `SchemaDefinition`, `SchemaValidator`, `SchemaApplier`
- [ ] Unit tests
- [ ] Integration tests

**Day 35: Week 7 Validation**
- [ ] All tests passing
- [ ] Intelligence modules testable in isolation
- [ ] Documentation updated

#### Week 8: Session Management + Phase 5.2 Wrap-up (5 days)

**Day 36-37: Session Management**
- [ ] Create `core/src/session/manager.rs`
- [ ] Extract from `cli/commands/session.rs` (700 LOC of business logic)
- [ ] Implement session creation, cookie mgmt, auth handling
- [ ] Unit tests
- [ ] Integration tests

**Day 38-39: Integration Testing**
- [ ] Full workspace test suite
- [ ] Cross-module integration tests
- [ ] Performance benchmarks (no regressions)
- [ ] Load testing (10k sessions)

**Day 40: Phase 5.2 Validation**
- [ ] CLI commands reduced by ~3,000 LOC total
- [ ] API server uses facade (no duplication)
- [ ] All tests passing (>80% coverage target)
- [ ] Documentation complete

**Phase 5.2 Success Criteria:**
- ✅ Extraction workflows in facade (680 LOC)
- ✅ Rendering workflows in facade (730 LOC)
- ✅ Domain profiling in intelligence (820 LOC)
- ✅ Schema management in intelligence (720 LOC)
- ✅ Session management in core (700 LOC)
- ✅ CLI commands reduced by 3,650 LOC
- ✅ API server performance parity achieved
- ✅ No code duplication between CLI/API

---

### Phase 5.3: Utilities & Features Extraction (2 weeks) - P2

**Objective:** Consolidate remaining features (2,030 LOC) for code quality
**Timeline:** Weeks 9-10
**Effort:** 5 days with 1 engineer

#### Week 9: Jobs & Workers (5 days)

**Day 41-42: Job Orchestration**
- [ ] Create `facade/src/jobs/orchestrator.rs`
- [ ] Extract from `cli/commands/job.rs` (350 LOC)
- [ ] Extract from `cli/commands/job_local.rs` (450 LOC)
- [ ] Implement `JobOrchestrator`, `LocalJobQueue`
- [ ] Unit tests

**Day 43: PDF & Table Extraction**
- [ ] Move `cli/commands/pdf.rs` → `extraction/src/pdf/extractor.rs` (420 LOC)
- [ ] Move `cli/commands/tables.rs` → `extraction/src/tables/parser.rs` (310 LOC)
- [ ] Update imports and dependencies
- [ ] Integration tests

**Day 44: Metrics & Stealth**
- [ ] Move `cli/commands/metrics.rs` → `monitoring/src/metrics/collector.rs` (320 LOC)
- [ ] Consolidate `cli/commands/stealth.rs` with `riptide-stealth` crate (180 LOC)
- [ ] Create unified APIs
- [ ] Integration tests

**Day 45: Week 9 Validation**
- [ ] All tests passing
- [ ] CLI commands simplified
- [ ] Documentation updated

#### Week 10: Final Utilities (2 days)

**Day 46-47: Remaining Extractions**
- [ ] Extract config validation to `riptide-config/validation.rs` (100 LOC)
- [ ] Extract system checks to `monitoring/src/health/` (200 LOC)
- [ ] Extract search logic to `riptide-search/` (100 LOC)
- [ ] Cleanup and validation

**Phase 5.3 Success Criteria:**
- ✅ Job orchestration in facade (800 LOC)
- ✅ PDF extraction in extraction crate (420 LOC)
- ✅ Table parsing in extraction crate (310 LOC)
- ✅ Metrics consolidated in monitoring (320 LOC)
- ✅ Stealth config unified (180 LOC)
- ✅ Config validation in config crate (100 LOC)
- ✅ System checks in monitoring (200 LOC)

---

### Phase 5.4: CLI Finalization & Validation (2 weeks) - P3

**Objective:** Reduce CLI to pure presentation layer, validate architecture
**Timeline:** Weeks 11-12
**Effort:** 2.4 days with 1 engineer

#### Week 11: Final Refactoring (5 days)

**Day 48: Dependency Cleanup**
- [ ] Update `cli/Cargo.toml` - REMOVE all direct library imports
- [ ] Keep ONLY: `riptide-facade`, `clap`, `colored`, `indicatif`, `comfy-table`
- [ ] Validate dependency tree: `cargo tree -p riptide-cli`
- [ ] Should show exactly 1 riptide dependency (facade)

**Day 49: Command Simplification**
- [ ] Audit all remaining CLI commands
- [ ] Ensure all use facade APIs only
- [ ] Remove any lingering direct library imports
- [ ] Target: Each command <200 LOC

**Day 50-51: Documentation**
- [ ] Create migration guide (CLI v1 → v2)
- [ ] Document facade API usage
- [ ] Update architecture diagrams
- [ ] Create before/after examples

**Day 52: Week 11 Validation**
- [ ] CLI LOC check: `tokei crates/riptide-cli` (target: <5,000 LOC)
- [ ] Dependency audit
- [ ] All commands working identically

#### Week 12: Comprehensive Validation (5 days)

**Day 53-54: Testing Blitz**
- [ ] Full test suite: `cargo test --workspace`
- [ ] CLI smoke tests: `./scripts/cli-smoke-tests.sh`
- [ ] Integration tests
- [ ] Performance benchmarks (within 5% of baseline)
- [ ] Load testing (10k sessions)

**Day 55: Quality Checks**
- [ ] Run clippy: `cargo clippy --workspace -- -D warnings`
- [ ] Check for circular dependencies
- [ ] Code coverage report (target: >80% in libraries)
- [ ] Security audit

**Day 56-57: Final Validation & Metrics**
- [ ] CLI LOC: 13,782 → <5,000 (67% reduction) ✅
- [ ] CLI business logic: 66% → <15% ✅
- [ ] Library code: ~4,700 → ~32,500 LOC ✅
- [ ] Test coverage: >80% in library crates ✅
- [ ] Performance: Within 5% of baseline ✅
- [ ] All consumers work: CLI, API, workers ✅

**Day 58-60: Documentation & Release**
- [ ] Complete migration guide
- [ ] Update CHANGELOG.md
- [ ] Create release notes (v2.0.0)
- [ ] Publish documentation
- [ ] Tag release: `git tag v2.0.0-cli-refactor`

**Phase 5.4 Success Criteria:**
- ✅ CLI reduced to 4,500 LOC (67% reduction)
- ✅ CLI has exactly 1 library dependency (facade)
- ✅ All commands work identically (backward compatible)
- ✅ Test coverage >80% across facade and library crates
- ✅ Documentation updated with migration guide
- ✅ Performance within 5% of baseline
- ✅ No circular dependencies
- ✅ Security audit passed

---

### Phase 5 Overall Deliverables

**New Crates:**
- `riptide-optimization` - Performance optimization modules (2,050 LOC)

**Enhanced Crates:**
- `riptide-browser` - Browser pool manager added (384 LOC)
- `riptide-facade` - Executor, extraction, rendering facades (3,500+ LOC)
- `riptide-intelligence` - Domain profiling, schema management (1,540 LOC)
- `riptide-core` - Session management (700 LOC)
- `riptide-extraction` - PDF, tables (730 LOC)
- `riptide-monitoring` - Metrics, health checks (520 LOC)

**CLI Transformation:**
- Before: 13,782 LOC (66% business logic)
- After: 4,500 LOC (12% business logic)
- Reduction: 9,270 LOC relocated to libraries (67% reduction)

**Benefits Achieved:**
- ✅ Library-first architecture
- ✅ Python bindings enabled
- ✅ WASM modules enabled
- ✅ API performance parity (2-10x speedup)
- ✅ Zero code duplication
- ✅ 2.5x development velocity
- ✅ 80%+ test coverage

**Documentation:**
- CLI v1 → v2 migration guide
- Facade API reference
- Architecture Decision Records (ADRs)
- Before/after code examples
- Performance benchmarks

---

## 🔄 Phase 6: Test Infrastructure (12 days)

**Objective:** Complete testing improvements
**Dependencies:** Phase 5 complete
**Timeline:** 2.4 weeks

### Tasks:

#### 6.1: Test Consolidation (3.6 days)
- Analyze 217 test files, identify duplicates
- Consolidate to 120 files (45% reduction)
- Maintain coverage, ensure all tests pass

#### 6.2: Performance Regression Suite (2.4 days)
- Create performance baselines
- Add performance tests to CI
- Configure regression detection and alerting

#### 6.3: Chaos Testing (6 days)
- Create failure injection framework
- Test network failures, resource exhaustion
- Validate recovery mechanisms
- Document failure modes

**Deliverables:**
- Test consolidation complete (45% reduction)
- Performance regression suite in CI
- Chaos testing framework
- Failure mode documentation

---

## 🔄 Phase 7: Code Quality & CLI Metrics (7 days)

**Objective:** Final code cleanup and CLI metrics revival
**Dependencies:** Phase 6 complete
**Timeline:** 1.4 weeks

### Tasks:

#### 7.1: CLI Metrics Module Revival (1 day) 🆕
**Priority:** P1 - Based on Hive Mind analysis
- Wire metrics to CLI commands (benchmark, status)
- Clean up 114 warnings (unused imports, variables)
- Create integration tests for metrics collection
- Update CLI documentation

#### 7.2: Configuration System Enhancement (2.4 days)
**Priority:** HIGH - Production requirement
- Add missing env vars to riptide-api (45 fields)
- Add missing env vars to riptide-persistence (36 fields)
- Create from_env() for riptide-pool (12 fields)
- Update .env.example with all variables

**Alternative:** Migrate to `config` crate pattern (like riptide-streaming) for better maintainability

#### 7.3: Configuration Documentation (1.2 days)
- Create ENVIRONMENT_VARIABLES.md (100+ variables)
- Update .env.example with examples
- Create configuration guide (hierarchy, security best practices)

#### 7.4: Final Code Cleanup (1.2 days)
- Remove unused API methods, cache utilities
- Target: <20 clippy warnings, ~500 lines removed

#### 7.5: Release Preparation (1.2 days)
- Update CHANGELOG with all changes (chromiumoxide cleanup, singletons, etc.)
- Version bumping to 2.0.0
- Prepare release notes

**Deliverables:**
- CLI metrics operational
- 100% env variable support
- Environment variables documented
- Code cleanup complete (<20 warnings)
- Release prepared (v2.0.0)

---

## 🔄 Phase 8: Documentation & Deployment (12 days)

**Objective:** Complete user-facing documentation
**Dependencies:** Phase 7 complete
**Timeline:** 2.4 weeks

### Tasks:

#### 8.1: User Migration Guide (6 days)
- Document breaking changes (P1 to P2)
- Create import path migration table
- Step-by-step upgrade checklist with examples
- Deprecation timeline and support schedule

#### 8.2: Deployment Guides (4 days)
- Kubernetes manifests and Helm chart
- Cloud platform guides (AWS, GCP, Azure)
- Database migration scripts
- Monitoring setup (Prometheus, Grafana)

#### 8.3: Final Validation (2 days)
- Deploy to staging environment
- Run complete test suite
- Performance smoke test
- Create production deployment runbook

**Deliverables:**
- User migration guide (P1-to-P2)
- Deployment guides (K8s, cloud platforms)
- Database migration scripts
- Production deployment approved

---

## 🔄 Phase 9: Client Libraries Validation (5 days)

**Objective:** Validate all client libraries and tooling
**Dependencies:** Phase 8 complete (can parallelize)
**Timeline:** 1.0 week

### Tasks:

#### 9.1: Rust CLI Validation (1.2 days)
- Test all CLI commands, API integration
- Update for P2 API changes
- Update documentation

#### 9.2: Node.js CLI Validation (1.2 days)
- Test all 15 commands (crawl, search, health, stream, etc.)
- Update for P2 API changes
- Prepare NPM package for publish

#### 9.3: Python SDK Validation (2.4 days)
**Priority:** CRITICAL (published to PyPI)
- Verify all 59 endpoints across 13 categories
- Test against updated P2 API
- Run all 8 example scripts
- Prepare PyPI package v2.0.0

#### 9.4: WASM Component Validation (1.2 days)
- Rebuild WASM binary
- Performance benchmarking
- Test in browser and Node.js
- Memory usage profiling

#### 9.5: Cross-Component Integration (1.2 days)
- End-to-end workflow testing
- Create version compatibility matrix
- Consolidate documentation

**Deliverables:**
- Rust CLI validated
- Node.js CLI ready for NPM
- Python SDK ready for PyPI (v2.0.0)
- WASM component validated
- Cross-component integration verified
- Unified documentation published

---

## 📊 Success Metrics

| Metric | Baseline | Target | Current |
|--------|----------|--------|---------|
| Compilation Errors | 267 | 0 | ✅ 0 |
| Test Pass Rate | BLOCKED | 100% | ✅ 99.4% (626/630) |
| Clippy Warnings | 200+ | <20 | 142 (dead_code) |
| Test Coverage | Unknown | 80% | TBD |
| Concurrent Sessions | ~500 | 10,000+ | TBD (Phase 4.1) |
| P95 Latency | Unknown | <500ms | TBD |
| Throughput | 10 req/s | 25 req/s | TBD |

---

## 🎯 Current Priorities

### This Week:
1. **Phase 3 Task 3.0:** Browser crate consolidation (3 days) 🔴 HIGHEST PRIORITY
2. **Phase 3 Tasks 3.1-3.2:** Quality baseline + docs (2 days)

### Next 2 Weeks:
1. Complete Phase 4 load testing and security audit
2. Begin Phase 5 test infrastructure work

### Milestones:
- **M5:** 10k Sessions Validated - Week 7
- **M6:** 80% Coverage - Week 9
- **M7:** Documentation Complete - Week 12
- **M8:** Client Libraries Validated - Week 13
- **M9:** PRODUCTION READY - 2026-02-05 ✅

---

## 📚 Reference Documents

### Key Documentation:
- `/docs/ROADMAP-UPDATE-2025-10-21.md` - Latest timeline changes
- `/docs/hive/HIVE-MIND-SESSION-COMPLETE.md` - Dead code analysis + singletons
- `/docs/hive/GLOBAL-SINGLETONS-DEPLOYMENT-SUMMARY.md` - Task 4.0 details
- `/docs/hive/phase2-completion-report.md` - Spider-chrome migration
- `/docs/hive/HIVE-MIND-DEAD-CODE-EXECUTIVE-SUMMARY.md` - 476 dead_code markers analyzed

### Key Findings:
- 90% of dead code is intentional architecture (Phase 3+ features)
- 162 chromiumoxide references are INTENTIONAL (spider-chrome compatibility)
- CLI metrics module complete, just needs wiring
- WasmCache::get_global() already existed (saved 2-3 hours)

---

## 🚀 Next Actions

### Today (Phase 3 Start):
1. **🔴 Create riptide-browser crate scaffolding**
2. **🔴 Copy core implementations from riptide-engine**
3. **🔴 Begin merging hybrid launcher features**

### This Week (5 days):
1. Day 1: Browser crate scaffolding + core migration
2. Day 2: Remove abstraction layer + integration
3. Day 3: Update consumer crates + validation
4. Days 4-5: Quality baseline + documentation updates

---

**END OF ROADMAP**

**Last Updated:** 2025-10-21
**Next Review:** End of Phase 3
**Status:** 🟢 40% Complete, On Track
**Target:** 2026-02-05 (15.0 weeks)
