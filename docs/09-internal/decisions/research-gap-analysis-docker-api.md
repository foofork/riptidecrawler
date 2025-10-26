# Docker/API Gap Analysis - Recent Codebase Changes Research

**Date:** 2025-10-24
**Research Period:** 2 weeks (2025-10-10 to 2025-10-24)
**Commits Analyzed:** 50+ commits across Phases 7.5, 9, 10, and 10.4
**Status:** 🔴 **CRITICAL GAPS IDENTIFIED**

---

## Executive Summary

**CRITICAL FINDING:** Docker configuration is **severely outdated** with 16 missing crates and 2 obsolete crate references. The API binary depends on 19 crates, but Docker only stubs 12 crates (10 valid + 2 obsolete).

### Impact
- **Docker Build Risk:** HIGH - Missing 13 critical dependencies
- **API Runtime Risk:** CRITICAL - Missing riptide-reliability, riptide-cache, riptide-extraction
- **Deployment Blocker:** YES - Cannot deploy v2.1.0 without fixes

---

## Timeline of Major Additions (Last 2 Weeks)

### Phase 10.4: Domain Warm-Start Caching (2025-10-24)
**Commit:** `0475c49`

**New Features:**
- ✨ Domain warm-start caching in `riptide-reliability/engine_selection.rs`
- ✨ `EngineCacheable` trait for domain profiling integration
- ✨ Integration with `riptide-intelligence/domain_profiling/profiler.rs`

**Files Changed:**
- `crates/riptide-reliability/src/engine_selection.rs` (+90 LOC)
- `crates/riptide-intelligence/src/domain_profiling/profiler.rs` (enhanced)
- `tests/integration/domain_warm_start_tests.rs` (new)

**Docker Impact:** ❌ `riptide-reliability` NOT in Docker stub sections

---

### Phase 10: Engine Selection Optimizations (2025-10-24)
**Commit:** `1b6c9c1`

**New Features:**
1. **Probe-first SPA escalation** - Try WASM before headless
2. **JSON-LD short-circuit** - Early return for complete schemas
3. **Refined content signals** - Better classification accuracy

**Major Changes:**
- ✨ Created `riptide-reliability/engine_selection.rs` (449 LOC)
- 📁 Test reorganization: unit/, integration/, chaos/, golden/, cli/, performance/
- ➕ 73 new comprehensive tests across crates

**New Test Files:**
```
crates/riptide-browser-abstraction/tests/
  ├── chromiumoxide_engine_tests.rs (563 LOC)
  ├── chromiumoxide_impl_tests.rs (735 LOC)
  ├── error_handling_tests.rs (166 LOC)
  └── factory_tests.rs (121 LOC)

crates/riptide-cli/tests/
  ├── client_tests.rs
  ├── config_tests.rs
  ├── execution_mode_tests.rs
  ├── job_manager_tests.rs
  └── job_storage_tests.rs

crates/riptide-pool/tests/
  ├── circuit_breaker_tests.rs
  ├── concurrent_access_tests.rs
  ├── error_recovery_tests.rs
  ├── health_monitor_tests.rs
  └── integration_tests.rs

crates/riptide-reliability/tests/
  └── content_signal_tests.rs
```

**Docker Impact:** ❌ None of these new test dependencies reflected in Docker

---

### Phase 9: CLI Refactoring (2025-10-23)
**Commit:** `5398d2b`

**Massive Code Migration (5,298 LOC across 5 sprints):**

#### Sprint 1: Infrastructure Migration
- ✅ PDF helpers → `riptide-pdf` (135 LOC)
- ✅ Browser pool manager removed (456 LOC dead code)

#### Sprint 2: Domain Profiling Migration
- ✅ Domain profiling → `riptide-intelligence/domain_profiling/` (1,172 LOC)
  - `profiler.rs`
  - `analyzer.rs`
  - 8 unit tests

#### Sprint 3: Schema & Reliability Migration
- ✅ Schema logic → `riptide-extraction/schema/` (1,000 LOC)
  - `comparator.rs`
  - `generator.rs`
  - `registry.rs`
  - `types.rs`
  - `validator.rs`
- ✅ Adaptive timeout → `riptide-reliability/timeout/` (539 LOC)

#### Sprint 4: WASM Features Migration
- ✅ WASM AOT cache → `riptide-cache/wasm/aot.rs` (779 LOC)
- ✅ WASM module cache → `riptide-cache/wasm/module.rs`
- ✅ Test files: `crates/riptide-cache/tests/wasm_aot_tests.rs`

#### Sprint 5: Final Integration
- ✅ Tables extraction → `riptide-extraction/tables/` (436 LOC)
  - `converter.rs`
  - `extractor.rs`
  - `parser.rs`
  - `types.rs`
  - `tests.rs`
- ✅ Validation → `riptide-monitoring/validation/` (952 LOC)
  - `checks.rs` (13,164 bytes)
  - `types.rs` (4,708 bytes)
  - `mod.rs`

**Docker Impact:** ❌ **CRITICAL** - 5 major crates completely missing from Docker:
- `riptide-extraction` (contains tables/, schema/)
- `riptide-cache` (contains wasm/)
- `riptide-reliability` (contains timeout/, engine_selection.rs)
- `riptide-monitoring` (contains validation/)
- `riptide-intelligence` (contains domain_profiling/)

---

### Phase 7.5: CLI Cleanup (2025-10-23)
**Commit:** `0b23b67`

**New Crates Created:**
- ✨ `riptide-reliability` (brand new crate)
  - `src/engine_selection.rs` (449 LOC)
  - `src/lib.rs`
  - Tests: engine_selection_tests.rs, circuit_breaker_tests.rs, integration_tests.rs

**Removed:**
- 🗑️ `crates/riptide-cli/src/commands/engine_fallback.rs` (474 LOC deleted)
- Functionality consolidated into `riptide-reliability`

**Docker Impact:** ❌ `riptide-reliability` not added to Docker

---

## Gap Analysis: Docker vs. Reality

### Current Workspace Crates (26 Total)
```
✅ Present in Docker:
  - riptide-api
  - riptide-headless
  - riptide-intelligence
  - riptide-pdf
  - riptide-performance
  - riptide-persistence
  - riptide-search
  - riptide-stealth
  - riptide-streaming
  - riptide-workers

❌ OBSOLETE in Docker (removed from workspace):
  - riptide-core (eliminated in P2-F1 Day 6)
  - riptide-html (renamed to riptide-extraction)

🔴 MISSING from Docker (16 crates):
  1. riptide-browser-abstraction ⚠️ Phase 1 Week 3
  2. riptide-browser ⚠️ Browser facade
  3. riptide-cache 🔥 Phase 9 Sprint 4 - WASM caching
  4. riptide-cli
  5. riptide-config ⚠️ Configuration system
  6. riptide-events ⚠️ P1-A3 Phase 2A
  7. riptide-extraction 🔥 Phase 9 Sprint 5 - Tables/Schema
  8. riptide-facade ⚠️ P1-C3 - High-level API
  9. riptide-fetch ⚠️ P1-C2 - HTTP layer
  10. riptide-monitoring 🔥 Phase 9 Sprint 5 - Validation
  11. riptide-pool ⚠️ P1-A3 Phase 2B
  12. riptide-reliability 🔥 Phase 7.5 - Engine selection
  13. riptide-security ⚠️ P1-A3 - Security middleware
  14. riptide-spider ⚠️ P1-C2 - Spider engine
  15. riptide-test-utils
  16. riptide-types ⚠️ Foundation types
```

**Legend:**
- 🔥 = Added in last 2 weeks (Phases 7.5-10.4)
- ⚠️ = Critical dependency for API runtime

---

## API Dependencies Analysis

### Current API Cargo.toml Dependencies (19 crates)
```toml
[dependencies]
riptide-pdf = { path = "../riptide-pdf" }                    ✅ In Docker
riptide-stealth = { path = "../riptide-stealth" }            ✅ In Docker
riptide-extraction = { path = "../riptide-extraction" }      🔴 MISSING
riptide-types = { path = "../riptide-types" }                🔴 MISSING
riptide-reliability = { path = "../riptide-reliability" }    🔴 MISSING (NEW)
riptide-fetch = { path = "../riptide-fetch" }                🔴 MISSING
riptide-cache = { path = "../riptide-cache" }                🔴 MISSING (NEW)
riptide-spider = { path = "../riptide-spider" }              🔴 MISSING
riptide-events = { path = "../riptide-events" }              🔴 MISSING
riptide-config = { path = "../riptide-config" }              🔴 MISSING
riptide-intelligence = { path = "../riptide-intelligence" }  ✅ In Docker
riptide-workers = { path = "../riptide-workers" }            ✅ In Docker
riptide-browser = { path = "../riptide-browser" }            🔴 MISSING
riptide-headless = { path = "../riptide-headless" }          ✅ In Docker
riptide-search = { path = "../riptide-search" }              ✅ In Docker
riptide-performance = { path = "../riptide-performance" }    ✅ In Docker
riptide-persistence = { path = "../riptide-persistence" }    ✅ In Docker
riptide-monitoring = { path = "../riptide-monitoring" }      🔴 MISSING (NEW)
riptide-facade = { path = "../riptide-facade" }              🔴 MISSING
```

**Result:** 10/19 dependencies missing from Docker (53% gap rate)

---

## Priority Recommendations

### 🔴 P0: Critical - Immediate Action Required

#### 1. Remove Obsolete Crate References (Lines 20-22, 35-38, 47-51)
```dockerfile
# DELETE THESE LINES:
COPY crates/riptide-core/Cargo.toml crates/riptide-core/
COPY crates/riptide-html/Cargo.toml crates/riptide-html/

RUN sed -i '/\[\[bench\]\]/,/required-features/d' crates/riptide-core/Cargo.toml || true && \
    sed -i '/\[\[example\]\]/,/^$/d' crates/riptide-html/Cargo.toml || true

mkdir -p crates/riptide-core/src crates/riptide-html/src && \
echo "fn main() {}" > crates/riptide-core/src/lib.rs && \
echo "fn main() {}" > crates/riptide-html/src/lib.rs && \
```

**Impact:** Prevents build failures from non-existent crates

#### 2. Add Critical Missing Crates (After Line 31)
```dockerfile
# ADD THESE LINES AFTER line 31:
COPY crates/riptide-browser-abstraction/Cargo.toml crates/riptide-browser-abstraction/
COPY crates/riptide-browser/Cargo.toml crates/riptide-browser/
COPY crates/riptide-cache/Cargo.toml crates/riptide-cache/
COPY crates/riptide-config/Cargo.toml crates/riptide-config/
COPY crates/riptide-events/Cargo.toml crates/riptide-events/
COPY crates/riptide-extraction/Cargo.toml crates/riptide-extraction/
COPY crates/riptide-facade/Cargo.toml crates/riptide-facade/
COPY crates/riptide-fetch/Cargo.toml crates/riptide-fetch/
COPY crates/riptide-monitoring/Cargo.toml crates/riptide-monitoring/
COPY crates/riptide-pool/Cargo.toml crates/riptide-pool/
COPY crates/riptide-reliability/Cargo.toml crates/riptide-reliability/
COPY crates/riptide-security/Cargo.toml crates/riptide-security/
COPY crates/riptide-spider/Cargo.toml crates/riptide-spider/
COPY crates/riptide-types/Cargo.toml crates/riptide-types/
```

**Impact:** Enables dependency caching for all API dependencies

#### 3. Add Stub Source Files (After Line 60)
```dockerfile
# ADD THESE LINES AFTER line 60:
mkdir -p crates/riptide-browser-abstraction/src \
    crates/riptide-browser/src crates/riptide-cache/src \
    crates/riptide-config/src crates/riptide-events/src \
    crates/riptide-extraction/src crates/riptide-facade/src \
    crates/riptide-fetch/src crates/riptide-monitoring/src \
    crates/riptide-pool/src crates/riptide-reliability/src \
    crates/riptide-security/src crates/riptide-spider/src \
    crates/riptide-types/src && \
echo "fn main() {}" > crates/riptide-browser-abstraction/src/lib.rs && \
echo "fn main() {}" > crates/riptide-browser/src/lib.rs && \
echo "fn main() {}" > crates/riptide-cache/src/lib.rs && \
echo "fn main() {}" > crates/riptide-config/src/lib.rs && \
echo "fn main() {}" > crates/riptide-events/src/lib.rs && \
echo "fn main() {}" > crates/riptide-extraction/src/lib.rs && \
echo "fn main() {}" > crates/riptide-facade/src/lib.rs && \
echo "fn main() {}" > crates/riptide-fetch/src/lib.rs && \
echo "fn main() {}" > crates/riptide-monitoring/src/lib.rs && \
echo "fn main() {}" > crates/riptide-pool/src/lib.rs && \
echo "fn main() {}" > crates/riptide-reliability/src/lib.rs && \
echo "fn main() {}" > crates/riptide-security/src/lib.rs && \
echo "fn main() {}" > crates/riptide-spider/src/lib.rs && \
echo "fn main() {}" > crates/riptide-types/src/lib.rs
```

**Impact:** Allows `cargo build --bin riptide-api` to succeed in stub layer

---

### 🟡 P1: High Priority - Fix Within 24 Hours

#### 4. Update API Dependencies Check
Verify all 19 API dependencies are buildable in Docker layer:
```bash
# Test command:
docker build --target builder -f infra/docker/Dockerfile.api . --progress=plain
```

**Expected:** Should compile all 26 workspace crates without errors

#### 5. Add riptide-test-utils (Optional but Recommended)
```dockerfile
COPY crates/riptide-test-utils/Cargo.toml crates/riptide-test-utils/
# And corresponding stub
```

**Impact:** Enables integration test builds in CI/CD

---

### 🟢 P2: Medium Priority - Validate in Next Sprint

#### 6. Validate Benchmark Removal Logic (Lines 35-38)
Current logic removes benchmarks from:
- ✅ riptide-performance (valid - has benches/)
- ❌ riptide-core (obsolete - crate doesn't exist)
- ❌ riptide-persistence (verify if needed)

**Action:** Update to only target crates that actually have benchmarks

#### 7. Add Missing CLI Crate (If Building CLI Binary)
If Docker should also build `riptide-cli` binary:
```dockerfile
COPY crates/riptide-cli/Cargo.toml crates/riptide-cli/
# And corresponding stub
```

---

## External Dependencies Analysis

### New External Dependencies (Last 2 Weeks)

#### From Phase 9:
- ✅ Already in workspace: All migrations used existing workspace deps

#### From Phase 10:
- ✅ No new external dependencies (used existing workspace deps)

#### From Phase 7.5:
- ✅ `dirs = "5.0"` added to `riptide-reliability/Cargo.toml`
  - **Status:** Should be handled by workspace dependency resolution
  - **Docker Impact:** None (handled by Cargo.lock)

**Conclusion:** No missing external dependencies in Docker

---

## Test Infrastructure Additions

### New Test Directories (Phase 10)
```
tests/
├── unit/               ✅ New - organized unit tests
├── integration/        ✅ New - integration tests
├── chaos/             ✅ New - failure injection tests
├── golden/            ✅ New - golden file tests
├── cli/               ✅ New - CLI tests
└── performance/       ✅ New - performance benchmarks
```

**Docker Impact:** ❌ None - tests not included in production Docker image (correct)

### Crate-Level Test Additions
- `riptide-browser-abstraction/tests/` - 7 new test files (2,585 LOC)
- `riptide-cli/tests/` - 6 new test files (1,200+ LOC)
- `riptide-extraction/tests/` - 2 new test files
- `riptide-pool/tests/` - 7 new test files
- `riptide-reliability/tests/` - 3 new test files
- `riptide-cache/tests/` - 2 new test files

**Docker Impact:** ❌ None - but validates that missing crates ARE critical

---

## Build Command Changes Needed

### Current Docker Build Command (Line 65)
```dockerfile
cargo build --profile ci --bin riptide-api
```

**Status:** ✅ Correct - targets only API binary

### Required Updates
None to build command, but dependency stubs must be complete.

---

## Memory Key Storage

**Key:** `swarm/feature-analysis`

**Contents:**
```json
{
  "research_period": "2025-10-10 to 2025-10-24 (2 weeks)",
  "commits_analyzed": 50,
  "critical_gaps": {
    "docker_obsolete_crates": ["riptide-core", "riptide-html"],
    "docker_missing_crates": 16,
    "api_dependency_gap": 10,
    "recent_additions": [
      "riptide-reliability (Phase 7.5)",
      "riptide-cache/wasm (Phase 9)",
      "riptide-extraction/tables (Phase 9)",
      "riptide-monitoring/validation (Phase 9)"
    ]
  },
  "priority_actions": {
    "p0_remove_obsolete": 2,
    "p0_add_critical": 14,
    "p1_validate_build": 1,
    "p2_optimize": 2
  }
}
```

---

## Validation Checklist

Before deploying v2.1.0, verify:

- [ ] **P0-1:** Remove `riptide-core` references from Docker (3 locations)
- [ ] **P0-2:** Remove `riptide-html` references from Docker (3 locations)
- [ ] **P0-3:** Add 14 missing crate Cargo.toml copies
- [ ] **P0-4:** Add 14 missing crate stub directories
- [ ] **P1-5:** Test Docker build: `docker build --target builder -f infra/docker/Dockerfile.api .`
- [ ] **P1-6:** Verify all 19 API dependencies compile
- [ ] **P2-7:** Update benchmark removal logic
- [ ] **P2-8:** Document crate additions in Dockerfile comments

---

## Conclusion

**Status:** 🔴 **DEPLOYMENT BLOCKER**

The Docker configuration is critically outdated with:
- 2 obsolete crate references causing potential build failures
- 16 missing crates (62% of workspace)
- 10 missing API dependencies (53% of API deps)

**Recent additions in Phases 7.5-10.4 make this worse:**
- `riptide-reliability` - Brand new crate (Phase 7.5)
- `riptide-cache/wasm` - Major feature addition (Phase 9)
- `riptide-extraction/tables` - New functionality (Phase 9)
- `riptide-monitoring/validation` - Critical validation logic (Phase 9)

**Immediate Actions Required:**
1. Remove obsolete crate references (5 min)
2. Add 14 missing crate stubs (10 min)
3. Test Docker build (5 min)
4. Deploy to staging for validation (30 min)

**Total Effort:** ~1 hour to unblock v2.1.0 deployment

---

**Report Generated:** 2025-10-24 by Research Agent
**Coordination Key:** `swarm/feature-analysis`
**Next Steps:** Forward to Planner → Coder → Tester for remediation
