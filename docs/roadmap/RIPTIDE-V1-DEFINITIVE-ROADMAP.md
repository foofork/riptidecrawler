# 🎯 RipTide v1.0 - THE DEFINITIVE ROADMAP
## Single Source of Truth - Validated & Corrected

**Status:** ✅ VALIDATED (95% confidence)
**Timeline:** 18 weeks to production-ready v1.0
**Validation:** 4-agent swarm verification complete
**Last Updated:** 2025-11-06

**⚠️ IMPORTANT:** This is THE roadmap. All other roadmap documents are superseded and archived.

---

## 🔴 IMMEDIATE TODO (Resume Here)

**🔥 PRIORITY:** Phase 3.1 Performance Optimization (Optional) OR Phase 4 Architecture Cleanup

**Current Status:**
- **Phase 3 COMPLETE** ✅ Comprehensive testing validated all restored endpoints
- **Test Suite:** 234/236 passing (99.2%), 72/72 facade tests passing
- **Code Quality:** 8.5/10 (Excellent), zero new architecture violations
- **Production Readiness:** 95% (functional ready, minor performance work optional)

**Findings:**
- ✅ All 6 endpoints fully functional
- ✅ Graceful degradation verified
- ✅ Error handling excellent
- ⚠️ 3 non-blocking performance regressions (health check, session middleware, cookies)

**Next Options:**
1. **Phase 3.1** - Performance optimization (1-2 days, optional)
2. **Phase 4** - Address 83 architecture violations (5-8 days)
3. **Python SDK** - Integration testing (2-3 days)

---

## 📋 Previous Completions

**✅ Phase 0 Week 0-1** (2025-11-04) - Foundation utils, 40 tests passing. See: `docs/phase0/PHASE-0-COMPLETION-REPORT.md` | Commit: `d653911`

**✅ Phase 0 Week 1.5-2** (2025-11-04) - Config system with env precedence. See: `docs/phase0/PHASE-0-WEEK-1.5-2-COMPLETION-REPORT.md`

**✅ Phase 1 Week 2.5-5.5** (2025-11-04) - Spider decoupling, 88/88 tests passing. See: completion report | Commit: `e5e8e37`

**✅ Phase 1 Week 5.5-9** (2025-11-05) - Trait composition, 21 tests passing, ~1,100 lines. See: `docs/phase1/PHASE-1-WEEK-5.5-9-COMPLETION-REPORT.md` | Commit: `e5e8e37`

**✅ Phase 1 Week 9** (2025-11-05) - CrawlFacade unification, 23/23 tests passing. See: `docs/phase1/PHASE-1-WEEK-9-FACADE-UNIFICATION-COMPLETION-REPORT.md` | Commit: `e5e8e37`

**✅ Phase 2 PyO3 Spike** (2025-11-05) - 10/10 tests, GO decision. See: `docs/phase2/PYO3-SPIKE-GO-NOGO-DECISION.md`

**✅ Week 13-14 Events Schema** (2025-11-05) - ICS + JSON-LD support. Commit: `bf26cbd`

**✅ Phase 2C.2 Handler Restoration** (2025-11-06) - All 6 disabled endpoints restored with facade integration. See: `docs/architecture/REFACTORING-PLAN.md` | Commits: `9343421`, `d638f69`, `3b6ad56`

**✅ Phase 3 Comprehensive Testing** (2025-11-06) - All endpoints validated, 99.2% test pass rate, code quality 8.5/10. See: `docs/PHASE-3-COMPLETION-REPORT.md` | Commit: `a5162c2`

---

## 🎯 Quick Reference: What to MOVE vs CREATE vs WRAP

| Task | Action | Reason |
|------|--------|--------|
| **Redis pooling** | CREATE NEW | Existing code is duplicated, needs unified API |
| **HTTP client factory** | CREATE NEW | Test setup code, not production-ready |
| **Retry logic** | REFACTOR | Extract from riptide-fetch, generalize |
| **Rate limiting** | CREATE NEW | Doesn't exist yet |
| **Secrets redaction** | CREATE NEW | Security hardening, doesn't exist |
| **Error system** | CREATE NEW | StrategyError doesn't exist |
| **Config system** | REFACTOR | Exists but needs server.yaml + precedence |
| **Robots toggle** | EXPOSE EXISTING | Already in SpiderConfig, just expose in API |
| **Spider decoupling** | CREATE NEW + MOVE | New trait, move embedded extraction code |
| **Composition traits** | CREATE NEW | Doesn't exist, enables `.and_extract()` |
| **PipelineOrchestrator** | WRAP EXISTING | 1,596 lines production code - DO NOT REBUILD |
| **Python SDK** | CREATE NEW | PyO3 bindings don't exist |
| **Events schema** | CREATE NEW | Schema-aware extraction doesn't exist |

**Golden Rule:** If code exists and works → WRAP or EXPOSE. Only CREATE NEW when truly missing.

# 🚨 START HERE - PASTE AT SESSION START

## Pre-Flight (30 seconds)
```bash
df -h / | head -2  # MUST have >5GB free
git branch --show-current  # Verify correct branch (see "Branches & Disk" below)
```

## Every Build
```bash
ruv-swarm build --parallel 4  # Use swarm (4x faster)
RUSTFLAGS="-D warnings" cargo clippy --all -- -D warnings  # ZERO warnings
cargo test -p [crate-changed]  # Test what you changed
```

## Golden Rules (Updated for Phase 2C Architecture)
1. **LAYERING**: API → Facade → Domain → Types (one-way dependency flow)
2. **THIN API**: Handlers only route HTTP, no business logic
3. **TYPES OWNERSHIP**: Shared types live in riptide-types (not API)
4. **CHECK FIRST**: `rg "function_name"` before creating
5. **COMMIT CLEAN**: Zero clippy warnings before pushing

## Rust Layering Decision Tree

**Where does this code belong?**

```
Is it HTTP routing/validation?
  ↓ YES → riptide-api (thin handlers)
  ↓ NO  → Continue...

Is it orchestration/workflow?
  ↓ YES → riptide-facade (use-cases)
  ↓ NO  → Continue...

Is it business logic (crawl/extract/search)?
  ↓ YES → domain crate (spider/extraction/search)
  ↓ NO  → Continue...

Is it a shared type/DTO?
  ↓ YES → riptide-types (contracts)
```

**Type Ownership Rules:**
- HTTP DTOs (request/response) → **riptide-types**
- Pure data used by 2+ layers → **riptide-types**
- Orchestrators (1,596 lines pipeline.rs) → **riptide-facade**
- API handlers → **riptide-api** (thin wrappers only)

⚠️ **NEVER:** Business logic or orchestration in API layer

## Branches & Disk
**Branch Names (use EXACTLY these):**
- **Week 0-2.5** (Phase 0: Foundation) → `main` (no PR, direct commits)
- **Week 2.5-5.5** (Spider decoupling) → `feature/phase1-spider-decoupling`
- **Week 5.5-9** (Composition traits) → `feature/phase1-composition`
- **Week 9-13** (Python SDK) → `feature/phase2-python-sdk`
- **Week 13-14** (Events schema) → `feature/phase2-events-schema`
- **Week 14-16** (Testing) → `feature/phase3-testing`
- **Week 16-18** (Docs + Launch) → `feature/phase3-launch`

**Disk:** <30GB total, >5GB free minimum (`df -h /`)
**PR:** All quality gates pass + >80% test coverage

## Agent Recovery (if lost)
```bash
git branch --show-current && df -h / | tail -1  # Where am I + disk OK?
rg "^## Week [0-9]" docs/roadmap/RIPTIDE-V1-DEFINITIVE-ROADMAP.md  # What's the plan?
```

**Remember:** REFACTOR not REWRITE. Check disk. Use swarm. Zero warnings. Update Roadmap with progress after any commits.

## 📋 File Operations Reference

**CRITICAL:** Before ANY file operation (MOVE/WRAP/EXTRACT), consult:
→ **[FILE-OPERATIONS-REFERENCE.md](./FILE-OPERATIONS-REFERENCE.md)**

**Quick lookup:**
- MOVE which files? → See reference doc
- WRAP which code? → See reference doc (pipeline.rs: 1,596 lines ❌ DO NOT MODIFY)
- EXTRACT from where? → See reference doc with exact line numbers

---

## 🎯 v1.0 Success Criteria

**Core Value Propositions:**
1. ✅ **Extract** (single URL) - `client.extract(url)` → JSON/Markdown/structured data
2. ✅ **Spider** (discover URLs) - `client.spider(url, max_depth=3)` → URL list (no extraction)
3. ✅ **Crawl** (batch process) - `client.crawl([urls])` → full pipeline (fetch + extract)
4. ✅ **Search** (via providers) - `client.search(query, provider="google")` → discovered URLs
5. ✅ **Compose** (flexible chains) - `client.spider(url).and_extract()` → chained operations
6. ✅ **Format outputs** - Convert to JSON, Markdown, iCal, CSV, or custom formats
7. ✅ **Python API** - `pip install riptidecrawler` with type hints and async support

**Extraction Strategy Modularity:**
- **Modular extraction**: ICS, JSON-LD, CSS selectors, LLM, regex, rules, browser-based
- **Adaptive selection**: Auto-select best strategy per content type
- **Output conversion**: Any extraction → JSON, Markdown, iCal, CSV, YAML

**Yes to all 7 = Ship v1.0** 🚀

**Test Coverage:** 41 test targets, 2,665+ test functions (maintain > 80%)

---

## 📊 Timeline Overview (18 Weeks + Phase 2C)

| Phase | Duration | Goal | Status |
|-------|----------|------|--------|
| **Phase 0 (Week 0-1)** | 1 week | Shared Utilities | ✅ COMPLETE (Report: docs/phase0/PHASE-0-COMPLETION-REPORT.md) |
| **Phase 0 (Week 1.5-2)** | 0.5 weeks | Configuration | ✅ CODE COMPLETE (Report: docs/phase0/PHASE-0-WEEK-1.5-2-COMPLETION-REPORT.md, verification blocked by env) |
| **Phase 0 (Week 2-2.5)** | 0.5 weeks | TDD Guide + Test Fixtures | ⏳ PENDING |
| **Phase 1** | Weeks 2.5-9 | Modularity & Facades | ✅ COMPLETE (Week 9: Facade Unification ✅ complete, Report: docs/phase1/PHASE-1-WEEK-9-FACADE-UNIFICATION-COMPLETION-REPORT.md) |
| **Phase 2C** 🔥 | 16-24 hours | **Architectural Refactoring (PRIORITY)** | 🔄 **IN PROGRESS** - BLOCKS all facade work |
| **Phase 2** | Weeks 9-14 | User-Facing API | 🚫 **BLOCKED BY Phase 2C** |
| **Phase 3** | Weeks 14-18 | Validation & Launch | ⏳ PENDING |

**Critical Path:** utils → errors → modularity → facades → **🔥 Phase 2C refactoring** → Python SDK → launch

**Key Adjustment:** +1-2 days for Phase 2C architectural fix (circular dependency resolution)

**⚠️ IMPORTANT:** Phase 2C must complete before Phase 2 Python SDK integration testing can proceed. 6 critical handlers currently disabled.

---

## 🔥 Phase 0: Critical Foundation (Weeks 0-2.5)

**✅ Week 0-1: COMPLETE** (2025-11-04) - Shared Utilities
**🔄 Week 1.5-2: IN PROGRESS** (2025-11-04) - Configuration (partial feature gates)
**⏳ Week 2-2.5: PENDING** - TDD Guide + Test Fixtures

### Week 0-1: Consolidation (5-7 days) ✅ COMPLETE (2025-11-04)
Foundation utilities consolidation - 40 tests passing, ~203 lines duplication removed.
See: `docs/phase0/PHASE-0-COMPLETION-REPORT.md` | Commit: `d653911`

#### W1.1-1.5: Error System + Health Endpoints (2-3 days) ⏳ PENDING

**Planned Work:**
- Health endpoints (`/healthz`, `/api/health/detailed`) for component monitoring
- Circuit breakers with hard timeouts (3s browser, fallback to native parser)
- StrategyError enum with 8 variants (CSS, LLM, JSON-LD, Regex, Browser, WASM, ICS, Generic)
- Error codes system with auto-conversion to ApiError

**Acceptance:** 8 error variants, health endpoints, circuit breakers, error docs in `/docs/api/ERROR-CODES.md`

#### W1.5-2: Configuration (2-3 days) ✅ CODE COMPLETE (2025-11-04)
Config system with env precedence and secrets redaction complete.
See: `docs/phase0/PHASE-0-WEEK-1.5-2-COMPLETION-REPORT.md`

#### W2-2.5: TDD Guide + Test Fixtures (2 days) ⏳ PENDING

**Status:** ⏳ PENDING
**Goal:** Optional developer tooling for deterministic testing (NOT required for CI)

**Planned Work:**
- Optional git submodule for test fixtures (Docker Compose)
- Recorded HTTP fixtures for CI (wiremock/httpmock)
- TDD London School guide with examples
- Make targets for local fixture management

**Note:** This work is deferred and optional. CI uses recorded HTTP mocks instead of live Docker services.

**Phase 0 Complete:** Foundation ready for modularity work

---

## 🔥 Phase 2C: Architectural Refactoring (PRIORITY - IN PROGRESS)

**Duration:** 16-24 hours (Week 2C.1-2C.2: 2025-11-06 to 2025-11-20)
**Status:** 🔄 ACTIVE - BLOCKS ALL FACADE WORK
**Blocking:** Phase 2 Python SDK integration, Phase 3A facades, all handler endpoints
**Documentation:** `/docs/architecture/REFACTORING-PLAN.md`, `/docs/architecture/TYPE-MIGRATION-ANALYSIS.md`

### Overview

Critical architectural fix to break circular dependency between riptide-api and riptide-facade. Previous attempt (`9343421`) created `riptide-pipeline` crate but left HTTP DTOs in wrong location, resulting in **6 disabled handlers** returning 503/500 errors.

### Root Cause

The circular dependency stems from **incorrect type ownership**:
1. **HTTP request/response DTOs live in riptide-api** (should be in riptide-types)
2. **riptide-facade depends on riptide-api** for these types (wrong direction)
3. **riptide-api tries to use facades** (correct direction, but blocked by #2)

**Result:** Facades commented out in AppState (state.rs:142-165), handlers return errors

### Solution: Two-Phase Type Migration + Facade Restoration

Implement proper **Rust layering** with one-way dependency flow:

```
riptide-api (HTTP handlers - thin)
    ↓ depends on
riptide-facade (orchestration/use-cases)
    ↓ depends on
[spider, extraction, search, pdf] (domain logic)
    ↓ depends on
riptide-types (shared contracts) ← NO UPWARD DEPENDENCIES
```

**Key Principle:** Dependencies flow **DOWN ONLY**. API → Application → Domain → Types

### Week 2C.1: Type Migration (6-8 hours)

**Goal:** Break circular dependency by moving HTTP DTOs to foundation layer

#### Files to Create/Modify

**New File:**
- `crates/riptide-types/src/http_types.rs` (~300 lines)

**Modified:**
- `crates/riptide-types/src/lib.rs` (add http_types export)
- `crates/riptide-facade/Cargo.toml` (remove riptide-api dependency)
- `crates/riptide-facade/src/*.rs` (update imports to use riptide-types)
- `crates/riptide-api/src/handlers/*.rs` (update imports to use riptide-types)

#### Types to Move (11 core DTOs, ~300-400 lines)

**Extract Endpoint:**
- `ExtractRequest` (extract.rs:14-23)
- `ExtractResponse` (extract.rs:67-79)
- `ExtractOptions` (extract.rs:30-42)
- `ContentMetadata` (extract.rs:82+)
- `ParserMetadata` (extract.rs:77+)

**Search Endpoint:**
- `SearchRequest` (search.rs)
- `SearchResponse` (search.rs)

**Spider Endpoint:**
- `SpiderResultStats` (dto.rs:26-43)
- `SpiderResultUrls` (dto.rs:45-67)
- `CrawledPage` (dto.rs:74-100)
- `ResultMode` (dto.rs:4-23, enum)

#### Task Checklist

- [ ] Create `crates/riptide-types/src/http_types.rs` with 11 DTO types
- [ ] Add `pub mod http_types; pub use http_types::*;` to riptide-types/lib.rs
- [ ] Remove `riptide-api = { path = "../riptide-api" }` from riptide-facade/Cargo.toml
- [ ] Update imports in riptide-facade: `use riptide_types::http_types::{ExtractRequest, ...}`
- [ ] Update imports in riptide-api handlers: `use riptide_types::http_types::{...}`
- [ ] Verify: `cargo tree -p riptide-facade | grep riptide-api` (should be empty)
- [ ] Build: `RUSTFLAGS="-D warnings" cargo build --workspace`
- [ ] Test: `cargo test -p riptide-types`

**Success Criteria:**
- ✅ No circular dependency in cargo tree
- ✅ All types in riptide-types/http_types.rs
- ✅ Zero clippy warnings
- ✅ All tests pass

### Week 2C.2: Facade Restoration (10-16 hours)

**Goal:** Re-enable 6 disabled handlers by restoring facades in AppState

#### Files to Modify

**AppState Initialization:**
- `crates/riptide-api/src/state.rs:142-165` (uncomment facades)
- `crates/riptide-api/src/state.rs:980+` (initialize facades)
- `crates/riptide-api/Cargo.toml` (restore riptide-facade dependency)

**Handler Restoration (6 files):**
- `crates/riptide-api/src/handlers/extract.rs:163-168`
- `crates/riptide-api/src/handlers/search.rs:93-98`
- `crates/riptide-api/src/handlers/spider.rs:83-86, 94-97, 105-108` (3 handlers)
- `crates/riptide-api/src/handlers/pdf.rs:151-154`
- `crates/riptide-api/src/handlers/crawl.rs:298-301` (remove unreachable guard)

#### Task Checklist

**State.rs Changes:**
- [ ] Uncomment facade fields in AppState struct (lines 142-165)
- [ ] Initialize ExtractionFacade in AppState::new() (~line 980)
- [ ] Initialize ScraperFacade in AppState::new()
- [ ] Initialize SpiderFacade (feature-gated) in AppState::new()
- [ ] Initialize SearchFacade (feature-gated) in AppState::new()
- [ ] Initialize BrowserFacade (feature-gated) in AppState::new()

**Handler Restoration:**
- [ ] Restore extract handler: replace 503 stub with facade call
- [ ] Restore search handler: replace 503 stub with facade call
- [ ] Restore spider_crawl handler: replace 500 stub with facade call
- [ ] Restore spider_status handler: replace 500 stub with facade call
- [ ] Restore spider_control handler: replace 500 stub with facade call
- [ ] Restore pdf_process handler: replace 500 stub with facade call
- [ ] Remove unreachable code guard in crawl handler (line 298)

**Testing:**
- [ ] Build: `cargo build --workspace`
- [ ] Tests: `cargo test -p riptide-api`
- [ ] Clippy: `cargo clippy --all -- -D warnings`
- [ ] Smoke test: `curl -X POST http://localhost:3000/api/v1/extract -d '{"url":"https://example.com"}'`

**Success Criteria:**
- ✅ All 6 handlers return real data (not 503/500)
- ✅ Facades initialized in AppState
- ✅ cargo test -p riptide-api passes
- ✅ Manual API smoke test works
- ✅ Zero clippy warnings

### Dependency Graph Transformation

**BEFORE (Circular - BROKEN):**
```
riptide-api  ←──────────┐ 🔴 CIRCULAR!
    ↓                    │
riptide-facade ──────────┘
    ↓
riptide-pipeline (partial fix)
    ↓
riptide-types
```

**AFTER (Acyclic - CORRECT):**
```
riptide-api (HTTP handlers - thin layer)
    ↓ uses
riptide-facade (orchestration/business logic)
    ↓ uses
[spider, extraction, search, pdf] (domain engines)
    ↓ uses
riptide-types (pure data contracts)
    ⚠️ NO UPWARD DEPENDENCIES
```

### Rust Layering Best Practices Applied

1. **API Layer (riptide-api):** Thin HTTP handlers, routing only
2. **Application Layer (riptide-facade):** Orchestration, workflows, composition
3. **Domain Layer:** Business logic in specialized crates (spider, extraction, etc.)
4. **Data Contracts (riptide-types):** Shared types, errors, configs - NO business logic

**Dependency Flow:** ONE-WAY DOWN ✅

### Architectural Best Practice Notes

**For All New Handlers (Post-Refactor):**
- ✅ Keep handlers thin (HTTP concerns only: validation, serialization, auth)
- ✅ Business logic belongs in facades (riptide-facade layer)
- ✅ Domain logic belongs in specialized crates (riptide-spider, riptide-extraction)
- ✅ Shared types belong in riptide-types (contracts, errors, DTOs)
- ❌ NEVER import upward (types can't depend on domain, domain can't depend on facade, facade can't depend on API)

**Type Ownership Decision Tree:**
```
Is it pure data (no behavior)?
├─ YES → Used by 2+ crates?
│        ├─ YES → riptide-types ✅
│        └─ NO → Keep in current crate
└─ NO → Contains business logic?
         ├─ YES → Orchestration? → riptide-facade
         │        └─ Domain? → Domain crate (spider, extraction, etc.)
         └─ NO → HTTP/transport? → riptide-api
```

### Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Type mismatches after move | HIGH | Incremental compilation per phase |
| Facade init failures | HIGH | Graceful fallbacks, detailed logging |
| Test breakage | MEDIUM | Update tests per phase, preserve coverage |
| Import path errors | LOW | Compiler catches, IDE auto-fix |

### Files Modified Summary

**Phase 1 (Type Migration) - ~300 lines moved:**
- `crates/riptide-types/src/http_types.rs` (NEW, 300 lines)
- `crates/riptide-types/src/lib.rs` (exports)
- `crates/riptide-facade/Cargo.toml` (remove api dep)
- `crates/riptide-facade/src/*.rs` (import updates)
- `crates/riptide-api/src/handlers/*.rs` (import updates)

**Phase 2 (Restoration) - 6 handlers fixed:**
- `crates/riptide-api/Cargo.toml` (restore facade dep)
- `crates/riptide-api/src/state.rs` (uncomment + initialize)
- `crates/riptide-api/src/handlers/extract.rs` (restore)
- `crates/riptide-api/src/handlers/search.rs` (restore)
- `crates/riptide-api/src/handlers/spider.rs` (restore 3 handlers)
- `crates/riptide-api/src/handlers/pdf.rs` (restore)
- `crates/riptide-api/src/handlers/crawl.rs` (remove guard)

### Documentation

- **Refactoring Plan:** `/docs/architecture/REFACTORING-PLAN.md`
- **Type Analysis:** `/docs/architecture/TYPE-MIGRATION-ANALYSIS.md` (342 types analyzed)
- **Completion Report:** TBD after Phase 2C.2

### Commits

- **Phase 1:** TBD (type migration complete)
- **Phase 2:** TBD (facade restoration complete)

---

## 🧩 Phase 1: Modularity & Composition (Weeks 2.5-9)

### Week 2.5-5.5: Decouple Spider from Extraction (3 weeks) ✅ COMPLETE (2025-11-04)
Spider decoupling complete - 88/88 tests passing (22 unit + 66 integration), ~200 lines removed.
See: completion report | Commit: `e5e8e37`

### Week 5.5-9: Trait-Based Composition (3.5 weeks) ✅ COMPLETE (2025-11-05)
Trait composition with fluent API - 21 tests passing, ~1,100 lines added.
See: `docs/phase1/PHASE-1-WEEK-5.5-9-COMPLETION-REPORT.md` | Commit: `e5e8e37`

### Week 9: Facade Unification (1 week) ✅ COMPLETE (2025-11-05)
CrawlFacade wraps 1,640 lines of production code - 23/23 tests passing.
See: `docs/phase1/PHASE-1-WEEK-9-FACADE-UNIFICATION-COMPLETION-REPORT.md` | Commit: `e5e8e37`

---

## 🚀 Phase 2: User-Facing API (Weeks 9-14)

### Python SDK (Weeks 9-13) - PyO3 Bindings

**Step 1: PyO3 Spike** ✅ COMPLETE (2025-11-05) - 10/10 tests, GO decision.
See: `docs/phase2/PYO3-SPIKE-GO-NOGO-DECISION.md`

**Steps 2-5:** Python SDK Core Bindings, Packaging, Type Stubs, Documentation 🚫 **BLOCKED BY Phase 2C**

**Blocking Reason:** Python SDK integration tests require working handler endpoints (extract, search, spider). Phase 2C must complete before integration testing can proceed.

**Prerequisites:**
- ✅ Phase 2C.1 complete (type migration)
- ✅ Phase 2C.2 complete (facades functional)
- ✅ All 6 handlers returning real data

### Week 13-14: Events Schema MVP ✅ COMPLETE (2025-11-05)
Schema-aware extraction with ICS + JSON-LD support. Commit: `bf26cbd`

---

## 🧪 Phase 3: Validation & Launch (Weeks 14-18)

**Status:** ⏳ NOT STARTED

**Planned Work:**
- Comprehensive testing (Week 14-16)
- Documentation and launch preparation (Week 16-18)
- Performance optimization
- Security hardening
- Production readiness validation

---

## 🎯 Success Metrics

**Week 18 Launch Criteria:**

**User Experience (7 Core Value Propositions):**
- [ ] Time to first extraction < 5 minutes
- [ ] **Extract**: `client.extract(url)` works in 1 line
- [ ] **Spider**: `client.spider(url)` discovers URLs independently

### Week 5.5-9: Trait-Based Composition (3.5 weeks) ⏳ PLANNED

**Planned Work:**
- Spider, Extractor, Chainable traits with `.and_extract()` fluent API
- BoxStream-based composition with partial success pattern
- Extraction DTO boundary (Document type decoupled from internals)
- Error handling: extraction errors yield Result::Err but stream continues

**Performance:** BoxStream adds ~100ns overhead (acceptable for I/O-bound ops)

**CRITICAL: Extraction DTO Boundary** - Public Document type decoupled from internal extraction models via ToDto mapper trait. Enables internal evolution without breaking SDK users.

**Usage Examples:** See completion report for Python examples (7 value propositions) and Rust low-level API.

**Acceptance:** Core traits, `.and_extract()` composition, partial success pattern, DTO boundary, performance benchmarks.

### Week 9: Facade Unification (1 week) ⏳ PLANNED

**ACTION:** WRAP existing 1,596 lines (pipeline.rs: 1,071 + strategies_pipeline.rs: 525) with thin CrawlFacade. DO NOT REBUILD!

**Acceptance:** Facade delegates to production orchestrators, both modes work, tests pass.

---

## ✨ Phase 2: User-Facing API (Weeks 9-14)

### Week 9-13: Python SDK (4-5 weeks)

**⚠️ ADJUSTED: +1-2 weeks from original estimate**
**Reason:** Async runtime complexity underestimated

**Step 1: PyO3 Spike** (Week 9, 2 days)

**Test async runtime integration:**
```rust
// Test if tokio runtime works with PyO3
use pyo3::prelude::*;
use tokio::runtime::Runtime;

#[pyfunction]
fn test_async() -> PyResult<String> {
    let rt = Runtime::new()?;
    rt.block_on(async {
        // Test basic async
        Ok("success".to_string())
    })
}
```

**Acceptance:**
- [x] Async runtime works in PyO3 ✅
- [x] No deadlocks or panics ✅
- [x] Go/no-go decision on Python SDK approach ✅ **GO**

**✅ Step 1 COMPLETE** (2025-11-05)
**Report:** `docs/phase2/PYO3-SPIKE-GO-NOGO-DECISION.md`
**Decision:** GO - Proceed with Python SDK (95% confidence)
**Tests:** 10/10 passing (100% success rate)

**Steps 2-5:** Core Bindings (PyO3 + tokio runtime), Python Packaging (maturin + PyPI), Type Stubs (.pyi), Documentation

**Acceptance:** `pip install riptidecrawler` works, all 3 modes functional, type hints, 5+ examples, PyPI published

### Week 13-14: Events Schema MVP + Output Formats (1-2 weeks) ⏳ PLANNED

**Planned Work:**
- Event schema with `schema_version: "v1"` field (Location, Organizer, metadata)
- 8 extraction strategies: ICS, JSON-LD, CSS, Regex, Rules, LLM (OpenAI only), Browser, WASM
- Adaptive strategy auto-selection based on content
- Output formats: JSON + Markdown (v1.0), CSV/iCal/YAML deferred to v1.1
- SchemaAdapter trait stub (full impl in v1.1)

**Acceptance:** Schema defined, 8 strategies, auto-selection, JSON/Markdown formats, >80% extraction accuracy, 10+ event sites tested

---

## 🚀 Phase 3: Validation & Launch (Weeks 14-18)

### Week 14-16: Testing (2-3 weeks) ⏳ PLANNED

**Strategy:** Fast CI with recorded fixtures (wiremock/httpmock), optional nightly E2E with live Docker

**Planned Tests:**
- 35 new integration tests (recorded fixtures)
- 20 golden tests with recorded HTML
- 5 performance tests
- Nightly E2E workflow (optional, doesn't block PRs)

**Acceptance:** 41+ test targets pass, CI <10 min, >80% coverage, recorded fixtures for robots/retry/timeouts/streaming

### Week 16-17: Documentation (1-2 weeks)

**Create:**
- Getting started guide (5 minutes)
- API reference (auto-generated)
- 10 examples
- Migration guide from crawl4ai
- Error handling guide

### Week 17-18: Beta & Launch (1-2 weeks)

**Beta testing:**
- 10 beta testers
- Real-world use cases
- Feedback collection

**Launch deliverables:**
- Docker image < 500MB
- Deployment guide
- Release notes
- Blog post

---

## 📦 Post-Launch Steps (Week 18+)

### Immediate (Day of Launch)
- [ ] **Tag release**: `git tag v1.0.0 && git push origin v1.0.0`
- [ ] **Build Docker image**: `docker build -t riptide:1.0.0 . && docker push`
- [ ] **Publish crates**: `cargo publish -p riptide` (if public)
- [ ] **Update docs site**: Deploy documentation to production
- [ ] **Announce**: Blog post, Twitter, Reddit, HN (if appropriate)

### Week 18-19 (Monitoring Period)
- [ ] **Monitor production metrics**: Error rates, latency, memory usage
- [ ] **Triage critical bugs**: Fix P0/P1 issues immediately
- [ ] **User feedback loop**: GitHub issues, support channels
- [ ] **Update README**: Add production deployment examples
- [ ] **Create v1.0.1 hotfix branch** if needed


---

## 🎯 Success Metrics

**Week 18 Launch Criteria:**

**User Experience (7 Core Value Propositions):**
- [ ] Time to first extraction < 5 minutes
- [ ] **Extract**: `client.extract(url)` works in 1 line
- [ ] **Spider**: `client.spider(url)` discovers URLs independently
- [ ] **Crawl**: `client.crawl([urls])` batch processes independently
- [ ] **Search**: `client.search(query)` discovers via providers
- [ ] **Compose**: `client.spider(url).and_extract()` chains flexibly
- [ ] **Format Outputs**: Convert to JSON, Markdown, iCal, CSV, YAML
- [ ] **Modular Extraction**: 8 strategies (ICS, JSON-LD, CSS, Regex, Rules, LLM, Browser, WASM)
- [ ] Adaptive strategy auto-selection works
- [ ] Events schema accuracy > 80%
- [ ] Python SDK fully functional with type hints

**Technical Quality:**
- [ ] 41 test targets + 35 new tests passing
- [ ] 80%+ test coverage maintained
- [ ] Zero code duplication (~2,580 lines removed)
- [ ] 100% facade usage
- [ ] Performance within 10% baseline

---

## 📊 v1.0 vs v1.1 Scope

### ✅ v1.0 - Must Have (18 weeks)

**User Features (7 Core Value Propositions):**
- [x] **Extract**: `client.extract(url)` - Single URL extraction
- [x] **Spider**: `client.spider(url)` - URL discovery only (no extraction)
- [x] **Crawl**: `client.crawl([urls])` - Batch processing (full pipeline)
- [x] **Search**: `client.search(query)` - Provider-based URL discovery
- [x] **Compose**: `client.spider(url).and_extract()` - Flexible chaining
- [x] **Format Outputs**: JSON, Markdown, iCal, CSV, YAML conversion
- [x] **Python SDK**: Full API with type hints

**Extraction Modularity:**
- [x] 8 extraction strategies: ICS, JSON-LD, CSS, Regex, Rules, LLM, Browser, WASM
- [x] Adaptive auto-selection: Best strategy per content type
- [x] Strategy registry: Swappable, extensible architecture

**Technical:**
- [x] 100% facade usage
- [x] Zero code duplication
- [x] Error codes: 50+ defined
- [x] 80%+ test coverage

### ❌ v1.1 - Deferred (Post-18 weeks)

**Deferred Features:**
- [ ] Full pipeline automation
- [ ] Multi-schema support
- [ ] Schema auto-detection
- [ ] Advanced streaming
- [ ] Multi-tenancy

---

## 🔧 Critical Path

```
Week 0: utils → Week 1: errors → Week 2.5-5.5: modularity →
Week 5.5-9: composition → **🔥 Phase 2C: arch refactor (16-24h)** →
Week 9-13: Python SDK → Week 14-18: validation
```

**Checkpoints:**
- Week 2.5: Foundation complete ✅
- Week 5.5: Spider decoupled ✅
- Week 9: Composition works ✅
- **Phase 2C.1:** Type migration complete (6-8 hours) ⏳
- **Phase 2C.2:** Facades restored, 6 handlers working (10-16 hours) ⏳
- Week 13: Python SDK works (blocked until Phase 2C complete)
- Week 18: Launch ready

**⚠️ BLOCKER:** Phase 2C must complete before Python SDK integration testing can proceed.

---

## 🚨 Risk Mitigation

**Risk 1: PyO3 Async Complexity**
- **Probability:** MEDIUM
- **Impact:** HIGH
- **Mitigation:** Week 9 spike, 2-day go/no-go decision

**Risk 2: Spider Decoupling**
- **Probability:** LOW
- **Impact:** MEDIUM
- **Mitigation:** 3 weeks allocated (was 1.5)

**Risk 3: Timeline Slip**
- **Probability:** MEDIUM (38% chance)
- **Impact:** HIGH
- **Mitigation:** +2 weeks buffer, weekly checkpoints

---

## ✅ Validation Status

**This roadmap has been:**
- ✅ Validated by 4-agent swarm
- ✅ 98% codebase alignment verified
- ✅ Timeline adjusted to realistic 18 weeks
- ✅ All syntax errors corrected
- ✅ All file paths verified
- ✅ All line counts verified (within 2 lines!)
- ✅ All effort estimates validated

**Confidence:** 95% (exceptional for 18-week project)

**Validation reports:**
- `/docs/roadmap/VALIDATION-SYNTHESIS.md`
- `/docs/validation/architecture-validation.md`
- `/docs/validation/codebase-alignment-verification.md`
- `/docs/validation/timeline-validation.md`
- `/docs/validation/completeness-review.md`

---

---

---

## 📋 v1.1 Planning (Post-Launch Priorities)

These are **important but safe to defer** after v1.0 ships:

### 1. **Extraction Model Decoupling** (v1.1)
- **Issue:** Extraction models have 9 dependents, high fanout
- **Fix:** Split `riptide-extraction` into:
  - `riptide-extraction-core` (traits, base types)
  - `riptide-extraction-strategies` (ICS, JSON-LD, CSS, etc.)
  - `riptide-extraction-wasm` (custom extractors)
- **Benefit:** Faster builds, clearer boundaries

### 2. **Feature Flag Matrix & CI Coverage** (v1.1)
- **Issue:** 45+ feature flags across 13 crates, no documented matrix
- **Fix:** Document blessed feature combinations, add CI matrix
- **Benefit:** Prevents "works-on-my-flagset" failures

### 3. **Config Consolidation** (v1.1)
- **Issue:** 150+ env vars, scattered docs
- **Fix:** Group configs by service, standardize naming, improve docs
- **Benefit:** Lower user support load

### 4. **Test-Time Optimization** (v1.1)
- **Issue:** 1,500+ tests are long-running
- **Fix:** Add "fast test" profile, parallelize slow tests
- **Benefit:** Faster iteration velocity

### 5. **Event Schema v2** (v1.1+)
- **Foundation:** v1.0 includes `schema_version: "v1"` string field
- **v1.1:** Implement SchemaAdapter trait + actual v2 schema migration
- **Benefit:** Non-breaking evolution of event models

### 6. **Additional Output Formats** (v1.1)
- **Deferred from v1.0:** CSV, iCal, YAML formats
- **Reason:** JSON + Markdown sufficient for launch
- **Benefit:** Keeps DTO surface small, adds later based on user demand

### 7. **Additional LLM Providers** (v1.1)
- **Deferred from v1.0:** Azure OpenAI, AWS Bedrock, Anthropic
- **v1.0 ships with:** OpenAI only
- **Benefit:** Reduces integration complexity, validates architecture first

### 8. **Advanced Streaming** (v1.1)
- **Deferred from v1.0:** Full SSE/WebSocket/templated reports
- **v1.0 ships with:** Basic NDJSON streaming
- **Benefit:** Reduces API surface during stabilization

### 9. **Redis Distributed Rate Limiting** (v1.1)
- **Deferred from v1.0:** Redis Lua token bucket
- **v1.0 ships with:** Simple governor-based in-memory limiter
- **Benefit:** Sufficient for single-instance deployments

### 10. **Browser Crate Consolidation** (v1.1)
- **Deferred from v1.0:** Merge duplicate browser impls
- **Reason:** Not on critical path, can consolidate after API stabilizes
- **Benefit:** Wait until public API is proven before internal refactor

---

## 🚨 Critical v1.0 Additions Summary

**Added to roadmap based on codebase analysis:**

1. ✅ **Event schema versioning** (Week 13-14)
   - `SchemaVersion` enum
   - `SchemaAdapter` trait for v1→v2 path
   - Prevents multi-crate churn on future schema changes

2. ✅ **Extraction DTO boundary** (Week 5.5-9)
   - Public `Document` DTO decoupled from internals
   - `ToDto` mapper trait
   - Allows internal evolution without breaking SDK users

**Why critical:** Both address **high-coupling hotspots** that become exponentially harder to fix post-launch. Small additions now (~1 day each), massive future insurance.

---

**This is THE roadmap. Follow this document. It is detailed, explicit, and verified.**

**Ready to execute Week 0.** 🚀
