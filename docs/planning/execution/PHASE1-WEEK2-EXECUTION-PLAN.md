# Phase 1 Week 2 - Execution Plan
**Date:** 2025-10-17
**Duration:** 5 days (Days 8-12 of Phase 1)
**Status:** 🚀 READY TO BEGIN

---

## 🎯 Executive Summary

Week 2 continues Phase 1 momentum with **parallel execution across 5 tracks** using a coordinated swarm of specialized agents.

**Week 2 Goals:**
1. ✅ Unblock baseline measurements (criterion, coverage)
2. ✅ Begin architectural refactoring (P1-A2, P1-A3)
3. ✅ Advance performance optimizations (P1-B3, P1-B4)
4. ✅ Progress spider-chrome migration (P1-C2)
5. ✅ Maintain 100% test passing, 0 build errors

---

## 📋 Week 2 Task Breakdown

### Track 1: Baseline Unblocking (Priority: HIGH)
**Agent:** QA Engineer + DevOps Engineer
**Duration:** 1-2 days
**Status:** 🟡 BLOCKED → ✅ UNBLOCKING

#### Tasks:
1. **Fix Criterion Dependency** (30 min) - P0
   - Add `criterion = "0.5"` to riptide-performance/Cargo.toml
   - Verify all 5 benchmark suites compile
   - Run smoke test: `cargo bench --no-run`

2. **Implement Per-Crate Coverage** (2 hours) - P1
   - Create script: `/scripts/measure-coverage.sh`
   - Run tarpaulin per crate: `cargo tarpaulin -p <crate>`
   - Aggregate results into HTML report
   - Target: 75-85% coverage baseline

3. **Fix 7 Environmental Test Failures** (1 hour) - P1
   - Update tests to use `std::env::temp_dir()`
   - Create test fixture directories
   - Verify: `cargo test --all` passes 100%

**Success Criteria:**
- ✅ All benchmarks compile and run
- ✅ Coverage baseline documented for all crates
- ✅ 254/254 tests passing (100%)

**Deliverables:**
- `/scripts/measure-coverage.sh`
- `/scripts/run-benchmarks.sh`
- `/docs/testing/COVERAGE-BASELINE.md`
- `/docs/testing/PERFORMANCE-BASELINE.md`

---

### Track 2: Architecture (P1-A2, P1-A3)
**Agent:** Senior Architect
**Duration:** 3-4 days
**Status:** 🟢 READY TO START

#### P1-A2: Architectural Cleanup (2 days)
**Description:** Remove dead code, consolidate duplicate logic, improve module boundaries

**Tasks:**
1. **Remove Dead Code** (4 hours)
   - Target: 340 lines identified in dead code analysis
   - Files: extraction strategies, stealth patterns, API endpoints
   - Verify: No functionality regression

2. **Consolidate Duplicate Logic** (4 hours)
   - Spider-chrome vs chromiumoxide duplicates
   - Extraction engine selection logic
   - Configuration management

3. **Improve Module Boundaries** (8 hours)
   - Extract shared utilities to riptide-types
   - Clear separation: core vs extraction vs API
   - Update dependency graph

**Success Criteria:**
- ✅ 340 lines of dead code removed
- ✅ No duplicate logic across crates
- ✅ Clear module boundaries documented
- ✅ All tests passing

#### P1-A3: Refactor riptide-core (2 days)
**Description:** Split riptide-core into 4 smaller crates

**Target Structure:**
```
riptide-core (orchestration only)
├── riptide-config (configuration)
├── riptide-engine (browser management)
├── riptide-cache (caching logic)
└── riptide-types (shared types) ✅ already exists
```

**Tasks:**
1. **Create riptide-config crate** (4 hours)
   - Move config structs from core
   - Environment variable handling
   - Validation logic

2. **Create riptide-engine crate** (6 hours)
   - Move browser pool logic
   - CDP connection management
   - Engine selection

3. **Create riptide-cache crate** (4 hours)
   - Move caching logic from core
   - Domain selection cache
   - WASM module cache

4. **Update Dependencies** (2 hours)
   - Update all Cargo.toml files
   - Fix compilation errors
   - Verify: cargo tree shows no circular deps

**Success Criteria:**
- ✅ 4 new crates created with clear responsibilities
- ✅ riptide-core reduced to <2000 lines
- ✅ 0 circular dependencies
- ✅ All tests passing

**Deliverables:**
- `/crates/riptide-config/`
- `/crates/riptide-engine/`
- `/crates/riptide-cache/`
- `/docs/architecture/ADR-005-core-refactoring.md`

---

### Track 3: Performance (P1-B3, P1-B4)
**Agent:** Performance Engineer
**Duration:** 3-4 days
**Status:** 🟢 READY TO START

#### P1-B3: Memory Pressure Validation (2 days)
**Description:** Validate browser pool memory limits work correctly under load

**Tasks:**
1. **Create Memory Load Tests** (4 hours)
   - Simulate 20 concurrent browsers
   - Monitor memory usage with tokio-console
   - Test soft limit (400MB) and hard limit (500MB)

2. **Validate Memory Eviction** (4 hours)
   - Test browser eviction at 400MB
   - Test browser termination at 500MB
   - Verify pool recovery

3. **V8 Heap Stats Integration** (4 hours)
   - Implement V8 heap stats collection
   - Add metrics export
   - Dashboard integration

4. **Load Testing** (4 hours)
   - Use `/scripts/load-test-pool.sh`
   - Target: 20 browsers, 1000 page loads
   - Verify: Memory stays under 500MB

**Success Criteria:**
- ✅ Memory limits enforced correctly
- ✅ Browser eviction works at 400MB threshold
- ✅ Pool recovery after OOM events
- ✅ V8 heap stats exported to metrics

#### P1-B4: CDP Connection Multiplexing (2 days)
**Description:** Optimize Chrome DevTools Protocol connection management

**Tasks:**
1. **Implement Connection Pool** (6 hours)
   - Create CDP connection pool
   - Reuse connections across requests
   - Implement connection health checks

2. **Batch CDP Commands** (4 hours)
   - Group related CDP commands
   - Send in single batch
   - Reduce round-trip latency

3. **Performance Testing** (2 hours)
   - Measure baseline: 1 connection per request
   - Measure optimized: connection pooling + batching
   - Target: 30% latency reduction

4. **Documentation** (2 hours)
   - Update architecture docs
   - Performance tuning guide
   - Troubleshooting guide

**Success Criteria:**
- ✅ CDP connection pool implemented
- ✅ Command batching reduces round-trips by 50%
- ✅ 30% latency reduction measured
- ✅ No connection leaks under load

**Deliverables:**
- `/crates/riptide-headless/src/cdp_pool.rs`
- `/docs/performance/CDP-OPTIMIZATION.md`
- `/tests/integration/memory_pressure_tests.rs`

---

### Track 4: Integration (P1-C2)
**Agent:** Backend Developer #1
**Duration:** 3-4 days
**Status:** 🟢 READY TO START

#### P1-C2: Spider-Chrome Migration Phase 1 (3-4 days)
**Description:** Migrate 20% of browser automation to spider-chrome

**Scope:**
- ✅ Basic page rendering (no JS execution)
- ✅ Screenshot capture
- ✅ PDF generation
- ❌ Complex JS (deferred to Phase 2)

**Tasks:**
1. **Integrate Real spider-chrome** (4 hours)
   - Replace placeholder with actual spider-chrome
   - Update riptide-headless-hybrid dependencies
   - Re-enable in workspace

2. **Implement Basic Automation** (8 hours)
   - Page navigation
   - HTML capture
   - Screenshot capture
   - PDF generation

3. **Testing & Validation** (6 hours)
   - Create integration tests
   - Compare spider-chrome vs chromiumoxide
   - Verify stealth preservation
   - Performance benchmarking

4. **Fallback Logic** (4 hours)
   - Implement spider-chrome → chromiumoxide fallback
   - Error handling
   - Metrics tracking

5. **Documentation** (2 hours)
   - Migration guide
   - Performance comparison
   - Known limitations

**Success Criteria:**
- ✅ spider-chrome handles 20% of page loads
- ✅ Performance parity with chromiumoxide
- ✅ Fallback works correctly on errors
- ✅ All integration tests passing

**Deliverables:**
- `/crates/riptide-headless-hybrid/` (re-enabled)
- `/docs/integration/SPIDER-CHROME-PHASE1.md`
- `/tests/integration/spider_chrome_tests.rs`

---

### Track 5: Quality Assurance
**Agent:** QA Engineer
**Duration:** Continuous (throughout Week 2)
**Status:** 🟢 MONITORING

#### Continuous Tasks:
1. **Test Monitoring** (daily)
   - Run: `cargo test --all`
   - Verify: 100% pass rate maintained
   - Alert: On any test failure

2. **Build Monitoring** (daily)
   - Run: `cargo build --all`
   - Verify: 0 errors, 0 warnings
   - Alert: On build failures

3. **Coverage Tracking** (after baseline)
   - Run: `./scripts/measure-coverage.sh`
   - Verify: Coverage doesn't drop
   - Target: 75-85% maintained

4. **Performance Regression** (after baseline)
   - Run: `./scripts/run-benchmarks.sh`
   - Compare: Against baseline
   - Alert: If performance degrades >10%

**Success Criteria:**
- ✅ 100% test pass rate all week
- ✅ 0 build errors all week
- ✅ Coverage maintained or improved
- ✅ No performance regressions

---

## 👥 Swarm Configuration

### Agent Assignments

| Agent | Role | Track | Workload |
|-------|------|-------|----------|
| **Agent 1** | Senior Architect | Architecture | P1-A2, P1-A3 |
| **Agent 2** | Performance Engineer | Performance | P1-B3, P1-B4 |
| **Agent 3** | Backend Dev #1 | Integration | P1-C2 |
| **Agent 4** | QA Engineer | Baseline + QA | Criterion, Coverage, Monitoring |
| **Agent 5** | DevOps Engineer | Baseline + CI | Benchmarks, Scripts, Automation |

**Topology:** Mesh (peer-to-peer coordination)
**Max Agents:** 5
**Coordination:** Memory sharing via claude-flow hooks

---

## 📅 Timeline

### Day 8 (Today, 2025-10-17)
- ✅ Week 2 plan created
- ✅ Swarm initialized
- ⏳ Baseline unblocking begins
- ⏳ Architecture work begins

### Day 9
- ✅ Criterion fixed, benchmarks running
- ✅ Coverage baseline complete
- ⏳ P1-A2 architectural cleanup in progress
- ⏳ P1-B3 memory tests in progress

### Day 10
- ✅ 100% tests passing
- ✅ P1-A2 complete
- ⏳ P1-A3 core refactoring begins
- ⏳ P1-B4 CDP optimization begins

### Day 11
- ✅ P1-A3 core refactoring complete
- ✅ P1-B3 memory validation complete
- ⏳ P1-B4 CDP optimization complete
- ⏳ P1-C2 spider-chrome integration in progress

### Day 12 (End of Week 2)
- ✅ All Week 2 tasks complete
- ✅ Week 2 completion report published
- ✅ Week 3 plan ready

---

## 🎯 Success Metrics

### Week 2 Exit Criteria

| Metric | Target | Status |
|--------|--------|--------|
| **Baseline Complete** | 100% | ⏳ Pending |
| **Tests Passing** | 254/254 (100%) | 🟡 97.2% |
| **Coverage** | 75-85% | ⏳ Pending |
| **P1-A2 Complete** | 100% | ⏳ Pending |
| **P1-A3 Complete** | 100% | ⏳ Pending |
| **P1-B3 Complete** | 100% | ⏳ Pending |
| **P1-B4 Complete** | 100% | ⏳ Pending |
| **P1-C2 Complete** | 100% | ⏳ Pending |
| **Build Errors** | 0 | ✅ 0 |
| **Circular Deps** | 0 | ✅ 0 |

### Performance Targets
- Memory pressure validation: <500MB under load
- CDP optimization: 30% latency reduction
- Test suite: <2 minutes execution time
- Coverage generation: <5 minutes per crate

---

## 🚀 Execution Strategy

### 1. Swarm Initialization (Now)
```bash
# Initialize mesh topology swarm
npx claude-flow@alpha swarm init mesh --max-agents 5

# Spawn specialized agents
npx claude-flow@alpha agent spawn architect --capabilities "refactoring,design"
npx claude-flow@alpha agent spawn performance --capabilities "optimization,profiling"
npx claude-flow@alpha agent spawn backend --capabilities "integration,spider-chrome"
npx claude-flow@alpha agent spawn qa --capabilities "testing,coverage"
npx claude-flow@alpha agent spawn devops --capabilities "automation,ci-cd"
```

### 2. Task Orchestration (Parallel)
- Use Claude Code's Task tool to spawn agents concurrently
- Each agent executes their track independently
- Memory coordination via hooks
- Daily sync checkpoints

### 3. Progress Tracking (Continuous)
- TodoWrite updates every 2 hours
- Memory snapshots every 4 hours
- Build/test checks every commit
- Performance regression checks daily

---

## 📚 References

**Planning Documents:**
- `/workspaces/eventmesh/docs/COMPREHENSIVE-ROADMAP.md`
- `/workspaces/eventmesh/docs/PHASE1-WEEK1-COMPLETION-REPORT.md`
- `/workspaces/eventmesh/docs/testing/BASELINE-METRICS-REPORT.md`

**Technical Docs:**
- `/workspaces/eventmesh/docs/architecture/` (4 ADRs)
- `/workspaces/eventmesh/docs/performance/` (quick wins guide)
- `/workspaces/eventmesh/docs/testing/` (test strategy)

**Artifacts from Week 1:**
- 47 files created
- 22 files modified
- 3 build errors fixed
- 4x browser pool capacity increase
- 5x faster failure detection

---

## ⚠️ Risks & Mitigation

### Risk 1: Coverage Generation Timeouts
**Impact:** Medium
**Probability:** High (already observed)
**Mitigation:**
- Per-crate coverage instead of workspace
- Incremental measurement
- Parallel execution

### Risk 2: Spider-Chrome Integration Complexity
**Impact:** High
**Probability:** Medium
**Mitigation:**
- Start with basic features only
- Implement fallback to chromiumoxide
- Phased rollout (20% → 50% → 100%)

### Risk 3: Core Refactoring Breaks Tests
**Impact:** High
**Probability:** Medium
**Mitigation:**
- TDD approach: tests first
- Incremental refactoring
- Continuous test monitoring

---

**Status:** 🚀 **READY TO EXECUTE**
**Next Action:** Initialize swarm and begin parallel execution

**Plan Created:** 2025-10-17
**Estimated Completion:** 2025-10-21 (5 days)
**Confidence:** High (Week 1 precedent)
