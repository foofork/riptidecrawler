# RipTide Roadmap Integration Design
## Architectural Refactoring as Priority Work

**Document Version:** 1.0
**Date:** 2025-11-06
**Author:** System Architect
**Status:** READY FOR IMPLEMENTATION

---

## Executive Summary

### The Problem

The current RipTide v1.0 roadmap contains a **critical architectural debt** that was discovered during Week 9 (Facade Unification):

- **Circular dependency:** `riptide-api` ↔ `riptide-facade`
- **6 disabled handlers** returning HTTP 503 errors
- **Type ownership violation:** HTTP DTOs living in wrong crates
- **Facade blockage:** Cannot complete facade-based work without resolving architecture

This architectural issue was "fixed" by commenting out facades in `AppState`, which means **Phase 1 (Modularity & Facades) is incomplete** despite being marked "✅ COMPLETE".

### The Solution

Insert architectural refactoring as **Phase 2C** (priority work) to:

1. Break circular dependency by moving types to `riptide-types`
2. Restore 6 disabled handler implementations
3. Unblock all facade-dependent work in remaining phases

### Impact Assessment

**Timeline Impact:** +2-3 days (16-24 hours work)
**Roadmap Changes:** Insert Phase 2C between Events Schema and Pool System
**Dependencies Affected:** All facade-related work now depends on Phase 2C completion
**Risk Level:** MEDIUM (managed with incremental approach)

---

## Recommended Integration Point: **Phase 2C (Architectural Refactoring)**

### Why Phase 2C (Not Week 9.5)?

**✅ RECOMMENDED: Insert as Phase 2C**

**Rationale:**
1. **Phase 2A (Events Schema) is complete** - Clean boundary
2. **Phase 2B (Pool System) is pending** - Natural insertion point
3. **Aligns with Phase structure** - Keeps week numbering consistent
4. **Blocks facade work** - Must complete before handler implementations
5. **Clear milestone** - "Phase 2C: Architecture Fixed" is a deliverable

**❌ NOT RECOMMENDED: Insert as Week 9.5**

**Why Not:**
1. **Week 9 is already marked complete** - Would confuse history
2. **Breaks week numbering** - Week 9.5 → 10 → 11 is awkward
3. **Phase structure is clearer** - Phase 2C is more explicit
4. **No precedent** - Other phases use sub-phases (2A, 2B), not fractional weeks

---

## New Phase Structure

### Current Roadmap (Before Integration)

```
Phase 0: Foundation (Week 0-2.5) ✅ COMPLETE
Phase 1: Modularity & Facades (Week 2.5-9) ✅ COMPLETE
Phase 2: User-Facing API (Week 9-14)
  ├─ Week 9-13: Python SDK (4 weeks)
  │   ├─ Step 1: PyO3 Spike ✅ COMPLETE
  │   ├─ Step 2-5: Core Bindings, Packaging, Type Stubs, Docs ⏳ PENDING
  ├─ Week 13-14: Events Schema MVP ✅ COMPLETE
  └─ Week 14-16: (Original Phase 2C) ⏳ PENDING
Phase 3: Validation & Launch (Week 14-18)
```

### Updated Roadmap (After Integration)

```
Phase 0: Foundation (Week 0-2.5) ✅ COMPLETE
Phase 1: Modularity & Facades (Week 2.5-9) ⚠️ PARTIAL (6 handlers disabled)
Phase 2: User-Facing API (Week 9-14)
  ├─ Phase 2A: Events Schema MVP (Week 13-14) ✅ COMPLETE
  ├─ Phase 2B: Pool System (Week 15-16) ⏳ PENDING
  ├─ Phase 2C: Architectural Refactoring (2-3 days) 🔴 CRITICAL
  │   ├─ Week 2C.1: Type Migration (6-8 hours)
  │   │   └─ Break circular dependency: API ↔ Facade
  │   └─ Week 2C.2: Facade Restoration (10-16 hours)
  │       └─ Re-enable 6 disabled handlers
  └─ Phase 2D: (Original Phase 2C work) ⏳ PENDING
Phase 2E: Python SDK (Week 9-13) ⏳ IN PROGRESS
  ├─ Step 1: PyO3 Spike ✅ COMPLETE
  └─ Step 2-5: Core Bindings, Packaging, Type Stubs, Docs ⏳ PENDING
Phase 3: Validation & Launch (Week 14-18)
```

**Key Changes:**
1. **Phase 1 status:** Changed from "✅ COMPLETE" to "⚠️ PARTIAL (6 handlers disabled)"
2. **Phase 2 structure:** Renamed to 2A, 2B, **2C (NEW)**, 2D, 2E
3. **Phase 2C inserted:** Between Events Schema (2A) and Pool System (2B)
4. **Python SDK:** Moved to Phase 2E (was inline, now separate phase)

---

## Phase 2C Detailed Plan

### Phase 2C.1: Type Migration (6-8 hours)

**Goal:** Break circular dependency by moving shared types to `riptide-types`

**Steps:**
1. Create `crates/riptide-types/src/http_types.rs` (30 min)
2. Move 47 DTO types from `riptide-api` → `riptide-types` (1 hour)
   - `ExtractRequest`, `ExtractResponse`, `ExtractOptions`
   - `SearchRequest`, `SearchResponse`
   - `SpiderResultStats`, `SpiderResultUrls`, `CrawledPage`
   - `ResultMode`, `FieldFilter`, `ErrorInfo`
   - All other handler request/response types
3. Remove `riptide-api` dependency from `riptide-facade/Cargo.toml` (5 min)
4. Update `riptide-facade` imports to use `riptide-types` (2 hours)
5. Add `riptide-facade` dependency to `riptide-api/Cargo.toml` (5 min)
6. Update `riptide-api` handlers to import from `riptide-types` (2 hours)
7. Validation: `cargo tree` shows NO circular dependency (30 min)

**Acceptance Criteria:**
- ✅ `cargo tree -p riptide-facade --depth 2 | grep riptide-api` returns EMPTY
- ✅ All types moved: ~300-400 lines from `riptide-api` → `riptide-types`
- ✅ `RUSTFLAGS="-D warnings" cargo build --workspace` succeeds
- ✅ Zero clippy warnings

**Risk:** MEDIUM
**Rollback:** `git reset --hard HEAD` (Phase 2C.1 is atomic)

### Phase 2C.2: Facade Restoration (10-16 hours)

**Goal:** Re-enable 6 disabled handlers by initializing facades in `AppState`

**Steps:**
1. Restore facade initialization in `AppState::new()` (2 hours)
   - Initialize `ExtractionFacade`, `ScraperFacade`
   - Initialize `SpiderFacade` (feature-gated)
   - Initialize `SearchFacade` (feature-gated)
   - Initialize `BrowserFacade` (feature-gated)
2. Restore handler implementations (8-12 hours)
   - `/api/v1/extract` (1 hour)
   - `/api/v1/search` (1 hour)
   - `/api/v1/spider/crawl` (2 hours)
   - `/api/v1/spider/status` (1 hour)
   - `/api/v1/spider/control` (1 hour)
   - `/api/v1/pdf/process` (1 hour)
   - Remove unreachable code guard in `/api/v1/crawl` (30 min)
3. Testing & validation (2 hours)
   - Unit tests for each handler
   - Integration tests
   - Manual smoke tests

**Acceptance Criteria:**
- ✅ All 6 handlers return real data (not 503/500 errors)
- ✅ Facades initialized successfully in `AppState`
- ✅ `cargo test -p riptide-api` passes
- ✅ Manual API smoke test: `curl http://localhost:3000/api/v1/extract` returns 200

**Risk:** MEDIUM (facade initialization may fail)
**Rollback:** Revert handler changes, facades remain commented out (system continues with 503 errors)

---

## Dependency Graph & Blocking Analysis

### Before Phase 2C (Current State)

```
Phase 1 ✅ (CLAIMED COMPLETE)
  └─ Week 9: Facade Unification
      └─ 6 handlers DISABLED (503 errors)
          └─ BLOCKS: All facade-dependent work

Phase 2A ✅ COMPLETE (Events Schema)
  └─ No facade dependency

Phase 2B ⏳ PENDING (Pool System)
  └─ May need facades → BLOCKED

Phase 2E ⏳ IN PROGRESS (Python SDK)
  └─ Needs facades for API → BLOCKED

Phase 3 ⏳ PENDING (Testing & Launch)
  └─ Needs working handlers → BLOCKED
```

### After Phase 2C (Target State)

```
Phase 2C ✅ COMPLETE (Architecture Fixed)
  └─ Circular dependency broken
  └─ 6 handlers restored

Phase 2B ⏳ UNBLOCKED (Pool System)
  └─ Can now use facades safely

Phase 2E ⏳ UNBLOCKED (Python SDK)
  └─ Can bind to working API

Phase 3 ⏳ UNBLOCKED (Testing & Launch)
  └─ All handlers functional
```

### Newly Blocked Items (Until Phase 2C Complete)

| Item | Original Phase | Why Blocked | Unblocked After |
|------|---------------|-------------|-----------------|
| Pool System | Phase 2B | May need facade integration | Phase 2C |
| Python SDK Bindings | Phase 2E | Needs working `/extract` endpoint | Phase 2C |
| Handler Integration Tests | Phase 3 | 6 handlers return 503 | Phase 2C |
| Performance Testing (facades) | Phase 3 | Cannot benchmark disabled handlers | Phase 2C |
| API Documentation | Phase 3 | Cannot document broken endpoints | Phase 2C |

---

## Best Practice Alignment

### Rust Layering Rules

**Target Architecture:**
```
riptide-api (thin HTTP layer)
    ↓ depends on
riptide-facade (orchestration layer)
    ↓ depends on
[domain crates] (spider, extraction, search, pdf)
    ↓ depends on
riptide-types (pure contracts)
```

**Rules:**
1. ✅ Dependencies flow DOWN ONLY
2. ✅ No upward dependencies
3. ✅ HTTP DTOs live in `riptide-types` (not `riptide-api`)
4. ✅ Handlers are THIN (delegate to facades)
5. ✅ Facades contain business logic
6. ✅ Domain crates are single-purpose
7. ✅ Types crate has NO business logic

### Phase 2C Enforcement

Phase 2C will **enforce** these rules by:

1. **Moving HTTP DTOs to riptide-types** (Phase 2C.1)
   - Violations: `ExtractRequest` in `riptide-api/src/handlers/extract.rs`
   - Fix: Move to `riptide-types/src/http_types.rs`

2. **Breaking circular dependency** (Phase 2C.1)
   - Violation: `riptide-facade` → `riptide-api` import
   - Fix: Remove `riptide-api` from `riptide-facade/Cargo.toml`

3. **Thin handlers** (Phase 2C.2)
   - Violation: Handlers contain business logic stubs
   - Fix: Delegate to facades

### Roadmap Audit: Best Practice Violations

| Phase | Violation | Severity | Fix |
|-------|-----------|----------|-----|
| Phase 1 Week 9 | Facades commented out | HIGH | Phase 2C.2 |
| Phase 1 Week 9 | Handlers return 503 | HIGH | Phase 2C.2 |
| Phase 1 Week 5.5-9 | DTOs in wrong crate | HIGH | Phase 2C.1 |
| Phase 2E | Python SDK binds to broken API | MEDIUM | Wait for Phase 2C |
| Phase 3 | Tests cannot cover disabled handlers | HIGH | Wait for Phase 2C |

**Summary:** Phase 1 claimed "✅ COMPLETE" but has **3 HIGH severity violations**. Phase 2C will fix all 3.

---

## Migration Path for Incomplete Work

### For Developers Working on Phase 2B (Pool System)

**Before Phase 2C:**
- ❌ Cannot use facades (they're commented out)
- ❌ Cannot test handler integration
- ⚠️ Must use direct domain crate calls (bypassing facade layer)

**After Phase 2C:**
- ✅ Use `ExtractionFacade`, `SpiderFacade`, etc.
- ✅ Handlers work end-to-end
- ✅ Follow standard layering: handler → facade → domain

**Action:** WAIT for Phase 2C completion before facade-dependent work.

### For Developers Working on Phase 2E (Python SDK)

**Before Phase 2C:**
- ❌ `/api/v1/extract` returns 503
- ❌ Cannot test PyO3 bindings
- ⚠️ Must mock API responses

**After Phase 2C:**
- ✅ All endpoints functional
- ✅ Can test real API calls
- ✅ Can generate accurate type stubs

**Action:** Complete PyO3 spike (already done), WAIT for Phase 2C before Step 2 (Core Bindings).

### For Developers Working on Phase 3 (Testing & Launch)

**Before Phase 2C:**
- ❌ 6 handlers untestable (503 errors)
- ❌ Integration tests fail or are skipped
- ❌ Cannot measure performance

**After Phase 2C:**
- ✅ All handlers testable
- ✅ Integration tests pass
- ✅ Performance benchmarks accurate

**Action:** Write test skeletons now, enable after Phase 2C.

---

## Timeline Impact Analysis

### Original Timeline (18 weeks)

```
Phase 0: Week 0-2.5 (2.5 weeks) ✅
Phase 1: Week 2.5-9 (6.5 weeks) ✅
Phase 2: Week 9-14 (5 weeks) ⏳
Phase 3: Week 14-18 (4 weeks) ⏳
Total: 18 weeks
```

### Updated Timeline (18 weeks + 2-3 days)

```
Phase 0: Week 0-2.5 (2.5 weeks) ✅
Phase 1: Week 2.5-9 (6.5 weeks) ⚠️ PARTIAL
Phase 2A: Week 13-14 (2 weeks) ✅
Phase 2C: 2-3 days 🔴 NEW
Phase 2B: Week 15-16 (2 weeks) ⏳
Phase 2D: TBD ⏳
Phase 2E: Week 9-13 (4 weeks) ⏳
Phase 3: Week 14-18 (4 weeks) ⏳
Total: 18 weeks + 2-3 days
```

**Timeline Impact:**
- **Added time:** 2-3 days (16-24 hours)
- **Critical path:** Phase 2C is on critical path (blocks everything)
- **Delivery impact:** Minimal (2-3 days out of 18 weeks = 1.2% slip)
- **Risk mitigation:** Small addition now prevents massive refactoring later

**Recommendation:** Accept 2-3 day slip. Alternative (skipping Phase 2C) results in **incomplete v1.0 with broken handlers**.

---

## Risk Assessment

### Phase 2C Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Type migration breaks compilation | LOW | HIGH | Incremental moves, test after each |
| Facade initialization fails | MEDIUM | HIGH | Graceful fallbacks, detailed logs |
| Handler tests fail | MEDIUM | MEDIUM | Fix as discovered, rollback if critical |
| Integration with existing code | LOW | HIGH | Careful import updates, IDE assist |

### Rollback Strategy

**Phase 2C.1 Fails:**
```bash
git reset --hard HEAD  # Revert type movement
# System returns to circular dependency state
```

**Phase 2C.2 Fails:**
```bash
git checkout HEAD -- crates/riptide-api/src/handlers/
# Facades remain commented out, handlers return 503 (current state)
```

**Nuclear Option:**
```bash
git revert <phase-2c-commit>
# Full rollback, system continues with architectural debt
```

### Acceptance of Risk

**Phase 2C is LOWER risk than alternatives:**

1. **Alternative 1:** Skip Phase 2C, ship with 6 broken handlers
   - ❌ Customer impact: Major features broken
   - ❌ Technical debt: Grows exponentially
   - ❌ Team velocity: Slows future work

2. **Alternative 2:** Fix later (post-v1.0)
   - ❌ Breaking change: Requires API version bump
   - ❌ Increased scope: More code depends on broken structure
   - ❌ Higher cost: 2-3 days now vs. 2-3 weeks later

3. **Phase 2C (RECOMMENDED):** Fix now
   - ✅ Clean architecture before v1.0
   - ✅ All handlers functional
   - ✅ Minimal timeline impact (2-3 days)

---

## Implementation Checklist

### Pre-Phase 2C
- [ ] Read this design document
- [ ] Read [REFACTORING-PLAN.md](./REFACTORING-PLAN.md)
- [ ] Read [TYPE-MIGRATION-ANALYSIS.md](./TYPE-MIGRATION-ANALYSIS.md)
- [ ] Ensure clean git working tree
- [ ] Create branch: `git checkout -b phase-2c-architecture-refactoring`
- [ ] Disk space check: `df -h / | head -2` (need >5GB)

### Phase 2C.1: Type Migration (6-8 hours)
- [ ] Create `http_types.rs` in riptide-types
- [ ] Move Extract types (1 hour)
- [ ] Move Search types (30 min)
- [ ] Move Spider types (30 min)
- [ ] Move other handler DTOs (2 hours)
- [ ] Update riptide-types lib.rs exports (15 min)
- [ ] Remove `riptide-api` dep from riptide-facade (5 min)
- [ ] Update riptide-facade imports (1 hour)
- [ ] Add `riptide-facade` dep to riptide-api (5 min)
- [ ] Update riptide-api handler imports (1 hour)
- [ ] Validate: `cargo tree -p riptide-api -i riptide-facade` (15 min)
- [ ] Validate: `RUSTFLAGS="-D warnings" cargo build --workspace` (30 min)
- [ ] Commit: `git commit -m "refactor(phase-2c): Move HTTP types to riptide-types (Phase 2C.1)"`

### Phase 2C.2: Facade Restoration (10-16 hours)
- [ ] Restore facade initialization in AppState::new() (2 hours)
  - [ ] ExtractionFacade
  - [ ] ScraperFacade
  - [ ] SpiderFacade (feature-gated)
  - [ ] SearchFacade (feature-gated)
  - [ ] BrowserFacade (feature-gated)
- [ ] Restore extract handler (1 hour)
- [ ] Restore search handler (1 hour)
- [ ] Restore spider_crawl handler (2 hours)
- [ ] Restore spider_status handler (1 hour)
- [ ] Restore spider_control handler (1 hour)
- [ ] Restore pdf_process handler (1 hour)
- [ ] Remove unreachable code guard in crawl handler (30 min)
- [ ] Run tests: `cargo test -p riptide-api` (1 hour)
- [ ] Manual smoke test: `/api/v1/extract` (15 min)
- [ ] Commit: `git commit -m "feat(phase-2c): Restore facade handlers (Phase 2C.2)"`

### Post-Phase 2C
- [ ] Update `RIPTIDE-V1-DEFINITIVE-ROADMAP.md` with Phase 2C
- [ ] Mark Phase 1 as "⚠️ PARTIAL → ✅ COMPLETE (after Phase 2C)"
- [ ] Update `ARCHITECTURE.md` with type ownership rules
- [ ] Run full test suite: `cargo test --workspace`
- [ ] Run clippy: `cargo clippy --all -- -D warnings`
- [ ] Create PR: `gh pr create --title "Phase 2C: Architectural Refactoring"`
- [ ] Memory hooks: `npx claude-flow@alpha memory store "swarm/architect/phase-2c-complete" "$(date)"`

---

## Communication Plan

### To Development Team

**Subject:** Phase 2C Insertion: 2-3 Day Architecture Fix

**Body:**
> Team,
>
> We've discovered that Phase 1 (Facade Unification) has a critical issue:
> - Circular dependency between riptide-api ↔ riptide-facade
> - 6 handlers disabled (returning 503 errors)
> - This blocks all facade-dependent work in Phase 2 & 3
>
> **Action:** Inserting **Phase 2C (2-3 days)** to fix architecture before continuing.
>
> **What This Means:**
> - Phase 2B (Pool System) and beyond: WAIT for Phase 2C completion
> - Phase 2E (Python SDK): Continue PyO3 spike, pause Core Bindings until Phase 2C done
> - Phase 3 (Testing): Write test skeletons, enable after Phase 2C
>
> **Timeline Impact:** +2-3 days (acceptable vs. shipping broken handlers)
>
> **Documentation:** See [ROADMAP-INTEGRATION-DESIGN.md](./docs/architecture/ROADMAP-INTEGRATION-DESIGN.md)
>
> Questions? Ask in #riptide-architecture

### To Project Manager

**Subject:** Timeline Adjustment: Phase 2C Insertion (+2-3 days)

**Body:**
> PM,
>
> **Issue:** Phase 1 completion report missed that 6 handlers are disabled due to circular dependency.
>
> **Fix:** Insert Phase 2C (2-3 days) to break circular dependency and restore handlers.
>
> **Timeline Impact:**
> - Original: 18 weeks
> - Updated: 18 weeks + 2-3 days (1.2% increase)
>
> **Risk Mitigation:**
> - Small fix now prevents 2-3 week refactoring post-v1.0
> - Shipping with broken handlers is NOT acceptable for v1.0
>
> **Approval Requested:** Accept 2-3 day slip to fix architecture correctly.

### To Stakeholders

**Subject:** v1.0 Architecture Quality Gate

**Message:**
> During Phase 1 completion, we identified that 6 API handlers are non-functional due to architectural debt. We're adding a 2-3 day "Phase 2C" to fix this before v1.0 launch.
>
> This is the right time to fix—delaying until post-v1.0 would require breaking changes and 10x more effort.
>
> Impact: Minimal (2-3 days out of 18 weeks). Benefit: All v1.0 handlers functional.

---

## Success Criteria

### Phase 2C Complete When:
- [ ] No circular dependencies: `cargo tree` confirms acyclic graph
- [ ] All 6 handlers functional: `/extract`, `/search`, `/spider/*`, `/pdf/process`
- [ ] All tests pass: `cargo test --workspace`
- [ ] Zero clippy warnings: `cargo clippy --all -- -D warnings`
- [ ] Manual smoke tests pass
- [ ] Roadmap updated with Phase 2C
- [ ] Architecture docs updated

### Roadmap Coherence Maintained When:
- [ ] Phase numbering clear (2A, 2B, 2C, 2D, 2E)
- [ ] Dependencies accurately tracked
- [ ] Completed work history preserved
- [ ] Best practice violations documented and fixed

---

## Related Documentation

- [REFACTORING-PLAN.md](./REFACTORING-PLAN.md) - Detailed refactoring steps
- [TYPE-MIGRATION-ANALYSIS.md](./TYPE-MIGRATION-ANALYSIS.md) - Type inventory & analysis
- [RIPTIDE-V1-DEFINITIVE-ROADMAP.md](../roadmap/RIPTIDE-V1-DEFINITIVE-ROADMAP.md) - Main roadmap (to be updated)
- [ARCHITECTURE.md](./ARCHITECTURE.md) - Architecture documentation (to be updated)

---

## Appendix A: Comparison of Integration Options

| Criteria | Phase 2C (RECOMMENDED) | Week 9.5 (Alternative) |
|----------|------------------------|------------------------|
| Roadmap Clarity | ✅ Clear phase structure | ❌ Fractional week is awkward |
| History Preservation | ✅ Week 9 stays "complete" | ❌ Retroactive edit confusing |
| Precedent | ✅ Phases 2A, 2B exist | ❌ No fractional weeks elsewhere |
| Milestone Clarity | ✅ "Phase 2C Done" is explicit | ⚠️ "Week 9.5 Done" unclear |
| Dependency Tracking | ✅ "Blocked until Phase 2C" | ⚠️ "Blocked until Week 9.5" less clear |
| Team Communication | ✅ "Working on Phase 2C" | ⚠️ "Working on Week 9.5" confusing |

**Winner:** Phase 2C by large margin.

---

## Appendix B: Quick Reference

### Key Files to Update

| File | Purpose | Phase |
|------|---------|-------|
| `crates/riptide-types/src/http_types.rs` | **NEW** - HTTP DTOs | 2C.1 |
| `crates/riptide-types/src/lib.rs` | Export http_types | 2C.1 |
| `crates/riptide-facade/Cargo.toml` | Remove riptide-api dep | 2C.1 |
| `crates/riptide-api/Cargo.toml` | Add riptide-facade dep | 2C.1 |
| `crates/riptide-api/src/state.rs` | Initialize facades (line 980+) | 2C.2 |
| `crates/riptide-api/src/handlers/extract.rs` | Restore handler (line 163) | 2C.2 |
| `crates/riptide-api/src/handlers/search.rs` | Restore handler (line 93) | 2C.2 |
| `crates/riptide-api/src/handlers/spider.rs` | Restore 3 handlers | 2C.2 |
| `crates/riptide-api/src/handlers/pdf.rs` | Restore handler (line 151) | 2C.2 |
| `crates/riptide-api/src/handlers/crawl.rs` | Remove unreachable guard | 2C.2 |

### Critical Commands

```bash
# Verify circular dependency is GONE (should be empty)
cargo tree -p riptide-facade --depth 2 | grep riptide-api

# Build with zero warnings
RUSTFLAGS="-D warnings" cargo build --workspace

# Run Phase 2C tests
cargo test -p riptide-types
cargo test -p riptide-facade
cargo test -p riptide-api

# Manual smoke test
cargo run -p riptide-api --bin riptide-api &
sleep 5
curl -X POST http://localhost:3000/api/v1/extract \
  -H 'Content-Type: application/json' \
  -d '{"url": "https://example.com"}'
# Should return 200 with content, not 503
```

---

## Summary for Coordination

**To Coder Agents:**
- Implement Phase 2C in 2 parts: Type Migration (2C.1) + Facade Restoration (2C.2)
- Follow checklists in this document
- Test incrementally
- Commit after each phase

**To Reviewer Agents:**
- Validate no circular dependencies after 2C.1
- Verify all 6 handlers functional after 2C.2
- Check clippy warnings = 0
- Confirm tests pass

**To Project Manager:**
- Phase 2C is CRITICAL (unblocks all facade work)
- Timeline impact: +2-3 days (acceptable)
- Alternative (skip) is NOT acceptable (ships broken handlers)

**Path to Integration Design Document:**
`/workspaces/eventmesh/docs/architecture/ROADMAP-INTEGRATION-DESIGN.md`

---

**RECOMMENDATION:**
- ✅ **Insert Phase 2C** between Phase 2A (Events Schema) and Phase 2B (Pool System)
- ✅ **Mark as CRITICAL PRIORITY** (blocks all facade work)
- ✅ **Timeline:** 2-3 days (16-24 hours)
- ✅ **Newly Blocked:** Phase 2B, Phase 2E (Step 2+), Phase 3 testing

**DELIVERABLE READY:** This document provides concrete roadmap update instructions.

---

**Document End** • Ready for execution • All decisions justified • Risk assessed • Timeline impact analyzed
