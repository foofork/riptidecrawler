# Dead Code Analysis - Quick Reference

**Generated:** 2025-10-21 | **Researcher Agent:** swarm-1761028289463-tpian51aa

## Critical Decision Matrix

| Action | Count | Files | Phase | Decision |
|--------|-------|-------|-------|----------|
| **KEEP ALL** | 85 | Pool infrastructure | Phase 3+ | ✅ Required for browser pool scaling |
| **REVIVE NOW** | 6 | CLI benchmarks + tests | Phase 3 | 🚨 Blockers for performance validation |
| **REVIVE P3** | 3 | Tracing + health | Phase 3 | ⚡ Critical for distributed debugging |
| **REVIVE P4** | 10 | Streaming + profiling | Phase 4 | 📅 Deferred to observability phase |
| **EVALUATE** | 1 | Test feature flags | Any | 🔍 Review http-mock usage |

---

## Phase 3 Blockers (Immediate Action Required)

### 🚨 1. CLI Benchmark Commands (CRITICAL)
**Files:**
- `/workspaces/eventmesh/crates/riptide-cli/src/main.rs` (lines 20, 71, 173)
- `/workspaces/eventmesh/crates/riptide-cli/src/commands/mod.rs` (line 27)

**Status:** Completely disabled with `TODO(chromiumoxide-migration)` markers

**Impact:** Cannot run performance benchmarks or load tests

**Action:** Create spider-chrome equivalents before Phase 3 optimization work

---

### 🚨 2. Browser Pool Lifecycle Tests (HIGH)
**Files:**
- `/workspaces/eventmesh/crates/riptide-engine/tests/browser_pool_lifecycle_tests.rs` (lines 374, 1230)

**Status:** Commented out `#[tokio::test]` attributes

**Impact:** Pool lifecycle untested, potential regressions

**Action:** Investigate why tests were disabled and re-enable

---

### ⚡ 3. Distributed Tracing (P1)
**Files:**
- `/workspaces/eventmesh/crates/riptide-api/src/handlers/telemetry.rs` (lines 166, 225)

**Status:** `TODO(P1): Wire up to actual trace backend`

**Impact:** Cannot debug distributed browser pool operations

**Action:** Integrate Jaeger/Zipkin/OTLP before Phase 3 distributed scaling

---

## What to KEEP (Do Not Remove)

### Browser Pool Infrastructure (85 items)
All `#[allow(dead_code)]` in these files are **REQUIRED** for Phase 3+:

```
✅ riptide-headless/src/pool.rs - BrowserPoolConfig, BrowserStats, PoolEvent
✅ riptide-engine/src/pool.rs - PoolStats, shutdown(), get_stats()
✅ riptide-engine/src/cdp_pool.rs - ConnectionWaiter, SessionAffinityManager
```

**Reason:** Phase 3 adaptive pool scaling and Phase 4 resource optimization depend on these structures.

---

## File-Level Breakdown

### High-Priority Files (Review First)

| File | Dead Code | TODOs | Status |
|------|-----------|-------|--------|
| `riptide-engine/src/pool.rs` | 30+ | 0 | ✅ Keep all (Phase 3 infrastructure) |
| `riptide-headless/src/pool.rs` | 30+ | 0 | ✅ Keep all (Phase 3 infrastructure) |
| `riptide-engine/src/cdp_pool.rs` | 25+ | 0 | ✅ Keep all (P1-B4 multiplexing) |
| `riptide-cli/src/main.rs` | 0 | 3 | 🚨 Revive benchmarks NOW |
| `riptide-cli/src/commands/mod.rs` | 0 | 2 | 🚨 Revive benchmarks NOW |
| `riptide-api/src/handlers/telemetry.rs` | 0 | 3 | ⚡ Wire tracing in Phase 3 |
| `riptide-api/src/handlers/monitoring.rs` | 0 | 3 | 📅 Memory profiling in Phase 4 |

---

## TODO Priority Breakdown

### P0 (Critical - Phase 3 Blockers)
- [ ] Re-enable CLI benchmark command (riptide-cli/src/main.rs:71)
- [ ] Re-enable CLI load test command (riptide-cli/src/main.rs:173)
- [ ] Uncomment benchmark module (riptide-cli/src/commands/mod.rs:27)
- [ ] Re-enable browser pool tests (riptide-engine/tests:374,1230)
- [ ] Re-enable cache warming test (riptide-cache/src/warming_integration.rs:263)

### P1 (High - Phase 3 Required)
- [ ] Wire up distributed tracing backend (riptide-api/src/handlers/telemetry.rs:166)
- [ ] Implement trace tree retrieval (riptide-api/src/handlers/telemetry.rs:225)
- [ ] Implement spider health check (riptide-api/src/health.rs:179)
- [ ] Add dynamic version from Cargo.toml (riptide-api/src/health.rs:40)

### P2 (Medium - Phase 4 Deferred)
- [ ] Implement memory profiling integration (riptide-api/src/handlers/monitoring.rs:217)
- [ ] Implement leak detection (riptide-api/src/handlers/monitoring.rs:244)
- [ ] Implement allocation analysis (riptide-api/src/handlers/monitoring.rs:270)
- [ ] Activate streaming infrastructure routes (7 files in riptide-api/src/streaming/)

---

## Commented Code Hotspots

### Critical (Re-enable Immediately)
1. **CLI Benchmarks** - 3 command handlers commented out
2. **Pool Tests** - 2 lifecycle tests disabled
3. **Cache Tests** - 1 warming test disabled

### Low Priority (Evaluate)
4. **Test Utilities** - 1 feature flag commented (`http-mock`)
5. **Browser Abstraction** - 1 spider feature flag commented

---

## Statistics by Crate

| Crate | Dead Code | TODOs | Priority |
|-------|-----------|-------|----------|
| riptide-engine | 55 | 1 | CRITICAL |
| riptide-headless | 30 | 0 | CRITICAL |
| riptide-api | 0 | 18 | HIGH |
| riptide-cli | 0 | 5 | HIGH |
| riptide-facade | 12 | 0 | MEDIUM |
| riptide-spider | 8 | 0 | MEDIUM |
| riptide-performance | 5 | 0 | MEDIUM |
| riptide-test-utils | 15 | 0 | LOW |

---

## Coordination Notes for Analyst

**Research Findings Stored In:**
- File: `/workspaces/eventmesh/docs/hive/dead-code-analysis.md`
- Memory: `hive/research/dead-code-findings`
- Metrics: `/tmp/research_metrics.json`

**Key Insights:**
1. **85% of dead code is intentional** - Pool infrastructure for Phase 3+
2. **4 migration blockers** - Benchmark/load test commands disabled
3. **Phase markers are excellent** - Code is well-documented for cleanup
4. **No zombie code detected** - All dead_code has clear "future use" comments
5. **Test regression risk** - 5 tests commented out, unknown reason

**Recommended Analyst Actions:**
1. Confirm pool infrastructure retention with architect
2. Create tickets for 4 chromiumoxide migration items
3. Investigate test regression root cause
4. Plan Phase 3 activation sequence
5. Document Phase 4 cleanup strategy

---

**Full Report:** `/workspaces/eventmesh/docs/hive/dead-code-analysis.md`
