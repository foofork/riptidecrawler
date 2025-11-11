# Design Document: One-Shot Sprint Plan for Facade Refactoring

**Document Type**: Architecture Design
**Date**: 2025-11-11
**Status**: Design Review
**Approach**: Fix-Forward One-Shot Migration

---

## Executive Summary

This design document outlines the transformation of the existing 12-week sprint plan into a fix-forward, one-shot migration model. The key change is merging Sprints 1-3 (AppState migration) into a single atomic operation that eliminates incremental state transitions.

---

## Problem Statement

### Current Sprint Plan Issues

The existing sprint plan uses a **phased migration approach**:
- **Sprint 1**: Migrate 50% of AppState fields
- **Sprint 2**: Migrate remaining fields + fix circular deps
- **Sprint 3**: Validation and cleanup

**Problems with Phased Approach**:
1. **Dual state systems**: AppState and ApplicationContext coexist during migration
2. **Feature flags complexity**: `legacy-appstate` vs `new-context` adds cognitive overhead
3. **Incremental testing burden**: Must test both systems at every phase
4. **Risk of incomplete migration**: Easy to leave AppState references behind
5. **Extended timeline**: 3 weeks for what could be 1 week of focused work

### Fix-Forward Principle

**"Either all migrated, or not migrated at all"**

A one-shot migration approach:
- Eliminates intermediate state
- Reduces testing surface (no dual implementations)
- Forces completeness (100% migration or failure)
- Simpler rollback (single git revert)
- Faster execution (1 week vs 3 weeks)

---

## Design: One-Shot Migration Milestone

### Structure Changes

**BEFORE** (Current):
```
## Sprint 1: AppState → ApplicationContext (50% migration)
## Sprint 2: AppState → ApplicationContext (50% migration + circular deps)
## Sprint 3: Validation & Cleanup
```

**AFTER** (One-Shot):
```
## Milestone 1: AppState to ApplicationContext Migration (One-Shot)
  - All 40+ fields migrated atomically
  - Circular dependencies fixed in same commit
  - Validation runs immediately after
  - Duration: 1 week (5 days)
```

### Detailed Sprint Plan Format

#### Title Block
```markdown
## Milestone 1: AppState to ApplicationContext Migration (One-Shot)

**Duration**: 1 week (5 business days, 40 hours)
**Priority**: P0 (Critical Blocker)
**Approach**: Atomic bulk migration with zero intermediate state

### Goal
Eliminate AppState god object (2213 LOC, 40+ fields) via bulk search/replace and
compilation-driven fixes. Replace with ApplicationContext as sole state system.

**Success = Binary**: Either 100% migrated or migration failed (no partial state).
```

#### Day-by-Day Tasks

```markdown
### Day 1: Analysis & Preparation (8 hours)

#### Task 1.1: AppState Field Audit (4 hours)
**File**: `/workspaces/riptidecrawler/crates/riptide-api/src/state.rs`
**Action**:
- Inventory all 40+ fields in AppState struct
- Document field purpose, type, usage locations
- Identify port traits needed (create list)

**Output**: `docs/appstate-field-inventory.md`

#### Task 1.2: Create Port Traits (4 hours)
**Location**: `/workspaces/riptidecrawler/crates/riptide-types/src/ports/`
**Action**:
- Create missing port traits for AppState fields
- Example: `CacheStorage`, `SessionStorage`, `MetricsCollector`

**Acceptance**: All AppState fields have corresponding port trait


### Day 2: Bulk Migration (8 hours)

#### Task 2.1: Bulk Search/Replace (2 hours)
**Command**:
```bash
# Find all AppState references
grep -R "AppState" crates/ --include="*.rs" > appstate-refs.txt

# Bulk replace (using ripgrep + sed)
rg "State<Arc<AppState>>" crates/ -l | xargs sed -i 's/State<Arc<AppState>>/State<Arc<ApplicationContext>>/g'
rg "State(app_state)" crates/ -l | xargs sed -i 's/State(app_state)/State(context)/g'
rg "app_state\." crates/ -l | xargs sed -i 's/app_state\./context./g'
```

**Verification**: Run `grep -R "AppState" crates/` and verify only trait/type definitions remain

#### Task 2.2: Delete/Alias state.rs (1 hour)
**File**: `/workspaces/riptidecrawler/crates/riptide-api/src/state.rs`
**Options**:
- **Option A (Delete)**: Remove file entirely, update imports
- **Option B (Alias)**: Create type alias then deprecate
  ```rust
  #[deprecated(note = "Use ApplicationContext instead")]
  pub type AppState = ApplicationContext;
  ```

**Recommendation**: Option A (clean break)

#### Task 2.3: Fix Compilation Errors (5 hours)
**Action**: Run `cargo check --workspace` and fix all errors
**Expected Errors**:
- Missing method calls (AppState methods → ApplicationContext methods)
- Field access patterns changed
- Handler signatures need updating

**Approach**: Iterative compilation fixes until `cargo check` passes


### Day 3: Facade Migration (8 hours)

#### Task 3.1: Update Facade Factories (4 hours)
**Files**: All facade constructors in `crates/riptide-facade/src/facades/*.rs`
**Before**:
```rust
pub fn new(app_state: Arc<AppState>) -> Self {
    Self { state: app_state }
}
```
**After**:
```rust
pub fn new(context: Arc<ApplicationContext>) -> Self {
    Self { context }
}
```

#### Task 3.2: Update Facade Method Implementations (4 hours)
**Action**: Update all facade methods to use `self.context` instead of `self.state`
**Files**: 12 facades across `crates/riptide-facade/src/facades/`


### Day 4: Validation & Testing (8 hours)

#### Task 4.1: Implement ApplicationContext::validate() (3 hours)
**File**: `/workspaces/riptidecrawler/crates/riptide-api/src/composition/mod.rs`
**Implementation**:
```rust
impl ApplicationContext {
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Check all required ports are present
        assert!(self.cache_storage.is_some(), "CacheStorage required");
        assert!(self.event_bus.is_some(), "EventBus required");
        // ... validate all 40+ fields

        // Check no circular dependencies (runtime check)
        // Check configuration validity
        Ok(())
    }
}
```

#### Task 4.2: Verify Circular Dependencies (2 hours)
**Command**:
```bash
cargo tree --workspace --duplicates > deps-check.txt
# Review output for cycles
```

**Action**: Fix any circular dependencies found

#### Task 4.3: Run Test Suite (3 hours)
**Commands**:
```bash
cargo test -p riptide-api
cargo test -p riptide-facade
cargo test --workspace --lib
```

**Acceptance**: All tests pass (0 failures)


### Day 5: Cleanup & Documentation (8 hours)

#### Task 5.1: Remove Empty Modules (1 hour)
**Action**: Find and delete empty composition modules
```bash
find crates/ -name "*.rs" -size 0 -delete
```

#### Task 5.2: Run Quality Gate (2 hours)
**Command**:
```bash
./scripts/quality_gate.sh
```

**Must Pass**:
- Formatting check
- Clippy with zero warnings
- All unit tests
- Compilation check

#### Task 5.3: Final Verification (2 hours)
**Checklist**:
```bash
# 1. No AppState references remain
grep -R "\bAppState\b" crates/ | grep -v "^Binary" | wc -l
# Expected output: 0

# 2. All handlers use ApplicationContext
grep -R "State<Arc<ApplicationContext>>" crates/riptide-api/src/handlers/ | wc -l
# Expected: All handler files

# 3. state.rs is deleted
test ! -f crates/riptide-api/src/state.rs && echo "✅ Deleted"
```

#### Task 5.4: Documentation & ADR (3 hours)
**Files to Create/Update**:
1. `docs/architecture/ADR-001-appstate-elimination.md` (NEW)
2. `docs/architecture/appstate-migration-complete.md` (NEW)
3. Update `docs/architecture/hexagonal-architecture.md`

**ADR Template**:
```markdown
# ADR-001: AppState Elimination via One-Shot Migration

## Status
Accepted

## Context
AppState god object (2213 LOC, 40+ fields) violated hexagonal architecture
principles and created circular dependencies.

## Decision
Perform atomic bulk migration: AppState → ApplicationContext in single sprint.

## Consequences
- Single state system (ApplicationContext)
- Zero feature flags
- Simpler testing (no dual implementations)
- Faster migration (1 week vs 3 weeks)

## Implementation
- Bulk search/replace all references
- Delete state.rs
- Fix compilation errors
- Validate in ApplicationContext::validate()
```
```

---

### One-Shot Migration Quality Gate

**Binary Pass/Fail - All 10 Must Pass**:

```markdown
### One-Shot Migration Quality Gate

- [ ] **1. Bulk Search/Replace**: `AppState` → `ApplicationContext` complete in all files
- [ ] **2. File Deletion**: `crates/riptide-api/src/state.rs` deleted (not aliased)
- [ ] **3. Handler Compilation**: All handlers in `crates/riptide-api/src/handlers/` compile using ApplicationContext
- [ ] **4. Facade Compilation**: All 12 facades compile and instantiate with ApplicationContext
- [ ] **5. Validation Function**: `ApplicationContext::validate()` implemented and passes all checks
- [ ] **6. Circular Dependencies**: `cargo tree --workspace --duplicates` shows zero cycles
- [ ] **7. Empty Modules**: All empty composition modules removed from codebase
- [ ] **8. Quality Script**: `./scripts/quality_gate.sh` passes (exit code 0)
- [ ] **9. Zero References**: `grep -R "\bAppState\b" crates/` returns 0 matches (excluding binaries)
- [ ] **10. Documentation**: ADR-001 created, migration guide updated

**Scoring**: 10/10 = PASS | <10 = FAIL (no partial credit)
```

---

### Acceptance Criteria

**Must achieve ALL of the following**:

1. ✅ **Single State System**: ApplicationContext is sole state container
2. ✅ **Zero AppState**: No references to AppState in codebase (except deprecation notices)
3. ✅ **All Handlers Migrated**: 100% of handlers use `State<Arc<ApplicationContext>>`
4. ✅ **All Facades Migrated**: 12/12 facades instantiate with ApplicationContext
5. ✅ **Port-Based Access**: All infrastructure accessed via port traits (no concrete types)
6. ✅ **Circular Deps Fixed**: `cargo tree --duplicates` shows 0 circular dependencies
7. ✅ **Quality Gate Passed**: `./scripts/quality_gate.sh` exits 0 (zero warnings)
8. ✅ **Test Coverage**: All existing tests pass, no regressions introduced
9. ✅ **Documentation Complete**: ADR written, migration guide updated
10. ✅ **Binary Outcome**: Either 100% complete or migration rolled back (no partial state)

---

### Mitigation Strategies

1. **Create Baseline Tag**:
   ```bash
   git tag -a pre-migration-baseline -m "Pre-AppState migration baseline"
   ```

2. **Commit Frequently**:
   - After each day's work
   - Before and after bulk search/replace
   - After compilation fixes

3. **Parallel Branch**:
   - Work on `feature/appstate-one-shot-migration` branch
   - Keep `main` stable for rollback reference

4. **Incremental Verification**:
   - Run `cargo check` after every file batch (10 files)
   - Run `cargo test` after each major change
   - Monitor compilation time for explosions

---

## Effort Estimate

### Time Breakdown

| Day | Tasks | Hours | Risk |
|-----|-------|-------|------|
| Day 1 | Analysis + Port Traits | 8h | Low |
| Day 2 | Bulk Migration + Compilation | 8h | Medium |
| Day 3 | Facade Updates | 8h | Low |
| Day 4 | Validation + Testing | 8h | Medium |
| Day 5 | Cleanup + Documentation | 8h | Low |
| **Total** | **Full Migration** | **40h** | **Medium** |

### Comparison to Phased Approach

| Approach | Duration | Complexity | Risk | Testing Burden |
|----------|----------|------------|------|----------------|
| **Phased** (Current) | 3 weeks (120h) | High (dual state) | High (incomplete) | 3x (both systems) |
| **One-Shot** (Proposed) | 1 week (40h) | Medium (single state) | Medium (atomic) | 1x (single system) |
| **Savings** | **-66% time** | **-50% complexity** | **-33% risk** | **-66% testing** |

---

## Remaining Sprints (Unchanged)

The following sprints remain **EXACTLY AS CURRENTLY DOCUMENTED**:

- **Sprint 4**: Empty Composition Modules (P0)
- **Sprint 5**: Infrastructure Violations Part 1 (P0)
- **Sprint 6**: Infrastructure Violations Part 2 (P0)
- **Sprint 7**: CrawlerFacade Testing (P1)
- **Sprint 8**: ExtractorFacade Testing (P1)
- **Sprint 9**: SearchFacade Testing (P1)
- **Sprint 10**: BrowserFacade Testing (P1)
- **Sprint 11**: StreamingFacade Testing (P1)
- **Sprint 12**: Remaining Facades Testing (P1)

**No changes to Sprints 4-12 content, format, or timeline**.

---

## Implementation File Location

**Target File**: `/workspaces/riptidecrawler/docs/sprint-plan-facade-refactoring.md`

### Changes to Apply

1. **Replace Section**: "## Sprint 1" through "## Sprint 3"
2. **With**: New "## Milestone 1: AppState to ApplicationContext Migration (One-Shot)"
3. **Keep Unchanged**: All content before Sprint 1, all content after Sprint 3
4. **Update Table of Contents**: Reflect merged milestone
5. **Update Executive Summary**: Show 1-week milestone instead of 3 sprints

---

## Success Metrics

### Quantitative Metrics

- **Migration Completeness**: 100% (binary, no partial state)
- **AppState References**: 0 (verified via grep)
- **Quality Gate Score**: 10/10 (all checks pass)
- **Test Pass Rate**: 100% (zero regressions)
- **Compilation Errors**: 0 (clean build)
- **Circular Dependencies**: 0 (verified via cargo tree)
- **Time to Complete**: ≤5 business days

### Qualitative Metrics

- **Code Clarity**: Single state system, no feature flags
- **Developer Confidence**: Binary outcome (success or rollback)
- **Architecture Compliance**: 100% hexagonal (no state violations)
- **Documentation Quality**: Complete ADR with decision rationale

---

## Dependencies

### Prerequisites

**Must exist before migration**:
- ✅ ApplicationContext struct defined
- ✅ Port traits infrastructure in place
- ✅ Test infrastructure working
- ✅ Quality gate script functional

### Blockers

**Migration cannot proceed if**:
- ApplicationContext missing required fields
- Port traits incomplete (need 40+ traits)
- Test suite has >10% failures
- Circular dependencies pre-exist in ApplicationContext

---

## Conclusion

### Summary

The one-shot migration approach offers:
- **66% time savings** (1 week vs 3 weeks)
- **Simpler implementation** (no dual state systems)
- **Binary outcome** (success or rollback, no partial state)
- **Reduced risk** (atomic commit, easier rollback)

### Recommendation

**Adopt one-shot migration approach** for Sprints 1-3 consolidation.

### Next Steps

1. **Review this design** with team/stakeholders
2. **Update sprint plan file** with merged milestone
3. **Create baseline git tag** before starting migration
4. **Execute migration** following day-by-day plan
5. **Validate with quality gate** before merging to main

---

**Design Status**: ✅ Ready for Implementation
**Risk Level**: Medium (mitigated with rollback strategy)
**Expected Benefit**: High (faster, simpler, cleaner)
