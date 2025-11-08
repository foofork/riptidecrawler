# Architecture Cleanup - Quick Reference Guide
## One-Page Summary for Developers

**Version**: 1.0
**Last Updated**: 2025-11-07
**Status**: 33% Complete (Phase 1.2 ✅)

---

## The 5 Phases

### Phase 1: Foundation (Week 1, 16h)
**Status**: 🟡 33% (2/6 tasks)
- [x] 1.1: Create riptide-domain ✅
- [x] 1.2: Move circuit breaker (372 lines) ✅
- [ ] 1.3: Move HTTP caching (180 lines) ← **NEXT**
- [ ] 1.4: Move error handling (100+ lines)
- [ ] 1.5: Move security/processing (40+ lines)
- [ ] 1.6: Validate & cleanup
- [ ] 1.7: Remove pipeline Redis (5 min)

**Resolves**: Issues #1 (Types purity), #5 (Pipeline Redis)

---

### Phase 2: Infrastructure (Week 2, 12h)
**Status**: ⏳ Pending
- [ ] 2.1: Extract cache warming (1,172 lines)
- [ ] 2.2: Move env access to API (14 calls)
- [ ] 2.3: Create abstraction traits

**Resolves**: Issues #6 (Cache coupling), #7 (Env access)

---

### Phase 3: Facade Detox (Week 3, 16h)
**Status**: ⏳ Pending
- [ ] 3.1: Create FetchMethod enum (1h)
- [ ] 3.2: Create typed models (4h)
- [ ] 3.3: Replace 42+ JSON usages (4h)
- [ ] 3.4: Define 11 service traits (3h)
- [ ] 3.5: Facade → traits only (4h)
- [ ] 3.6: Wire at AppState (2h)

**Resolves**: Issues #3 (HTTP leakage), #4 (Dependencies)

---

### Phase 4: Handlers (Week 4, 12h)
**Status**: ⏳ Pending
- [ ] 4.1: TableExtractionFacade (95 lines)
- [ ] 4.2: RenderFacade (138 lines)
- [ ] 4.3: ReportFacade (92 lines)
- [ ] 4.4: Validate handlers <30 lines

**Resolves**: Issue #2 (Handler complexity)

---

### Phase 5: Validation (Week 5, 8h)
**Status**: ⏳ Pending
- [ ] 5.1: Run full validation suite (1h)
- [ ] 5.2: Update documentation (3h)
- [ ] 5.3: CI/CD integration (4h)

**Delivers**: ✅ All 7 issues resolved

---

## Critical Commands

### Run After Every Change
```bash
# Architecture validation
./scripts/validate_architecture.sh

# All tests
cargo test --workspace --no-fail-fast

# No warnings
cargo clippy --all -- -D warnings

# Clean build
cargo build --workspace
```

### Check Progress
```bash
# Types LOC (target: 2,000)
tokei crates/riptide-types/src/ | grep Total

# Domain LOC (target: 859)
tokei crates/riptide-domain/src/ | grep Total

# Facade dependencies (target: 1 = types only)
cargo tree -p riptide-facade --depth 1 | grep "riptide-" | wc -l
```

### Quick Validation
```bash
# Check specific issue
./scripts/validate_architecture.sh | grep "Issue #1"

# Individual checks
grep -r "impl.*CircuitBreaker" crates/riptide-types/src  # Should be 0
grep "redis" crates/riptide-pipeline/Cargo.toml          # Should be 0
grep "riptide-" crates/riptide-facade/Cargo.toml | grep -v "types"  # Should be 0
```

---

## Common Issues

### Test Failures
**Symptom**: Tests fail after moving code
**Solution**:
```bash
# Check re-exports
grep "pub use riptide_domain" crates/riptide-types/src/

# Quick rollback
git stash
cargo test --workspace
```

### Compilation Errors
**Symptom**: Missing imports
**Solution**:
```bash
# Check workspace build
cargo check --workspace 2>&1 | grep "error\[E"

# Update imports incrementally
```

### Circular Dependencies
**Symptom**: Cyclic dependency error
**Solution**: Review dependency graph, ensure API → Facade → Domain → Types

### Performance Regression
**Symptom**: Slower build
**Solution**: Benchmark before/after, re-exports are zero-cost

---

## The 3 Golden Rules

### 1. Types Purity
**Rule**: riptide-types = data structures + traits ONLY

**Check**:
```bash
grep -rE "fn (parse|validate|hash|transform|check)" crates/riptide-types/src
# Should return <5 results (constructors/getters only)
```

**Fix**: Move business logic to riptide-domain

---

### 2. Handler Simplicity
**Rule**: Each handler <30 lines (validate → facade → DTO)

**Check**:
```bash
find crates/riptide-api/src/handlers -name "*.rs" -exec wc -l {} +
# All should be <30 lines
```

**Fix**: Extract orchestration to facades

---

### 3. Facade Trait-Only
**Rule**: Facade depends on riptide-types ONLY

**Check**:
```bash
grep "riptide-" crates/riptide-facade/Cargo.toml | grep -v "types"
# Should return 0 results
```

**Fix**: Define traits, inject implementations

---

## Success Criteria

### All Green When:
```bash
./scripts/validate_architecture.sh

# Expected:
# ✅ ARCHITECTURE VALIDATION PASSED
# Passed: 28
# Warnings: 0
# Failed: 0
```

### Specific Checks:
- [ ] riptide-types: ~2,000 lines (currently 2,892)
- [ ] riptide-domain: ~859 lines (currently 475)
- [ ] All handlers: <30 lines each
- [ ] Facade deps: 1 (riptide-types only)
- [ ] Zero business logic in types
- [ ] Zero HTTP types in facade
- [ ] Zero env::var in domain

---

## Progress Tracking

| Metric | Current | Target | Progress |
|--------|---------|--------|----------|
| Types LOC | 2,892 | 2,000 | 🟡 71% |
| Domain LOC | 475 | 859 | 🟡 55% |
| Issues Resolved | 2/7 | 7/7 | 🟡 29% |
| Phase Progress | 1.2/5.3 | 5.3/5.3 | 🟡 23% |

**Overall**: 33% complete (Phase 1.2 done, 1.3 next)

---

## What to Do Next

### This Week (Week 1)
1. ⏳ **Phase 1.3**: Move HTTP caching (180 lines, 3 hours)
   - Plan: `/workspaces/eventmesh/reports/PHASE_1_3_EXECUTION_PLAN.md`
2. ⏳ **Phase 1.4**: Move error handling (100+ lines, 3 hours)
3. ⏳ **Phase 1.5**: Move security/processing (40+ lines, 2 hours)
4. ⏳ **Phase 1.6**: Validate (2 hours)
5. ⏳ **Phase 1.7**: Remove pipeline Redis (5 minutes)

**Target**: Complete Phase 1 by end of Week 1

---

## When Things Go Wrong

### Rollback Strategy
```bash
# Stash uncommitted changes
git stash

# Revert specific commit
git revert <commit-sha>

# Check tests pass
cargo test --workspace

# Re-apply if safe
git stash pop
```

### Emergency Contacts
- **Detailed Roadmap**: `/workspaces/eventmesh/docs/roadmap/ARCHITECTURE_CLEANUP_DEFINITIVE_ROADMAP.md`
- **Phase 1.3 Plan**: `/workspaces/eventmesh/reports/PHASE_1_3_EXECUTION_PLAN.md`
- **Validation Script**: `/workspaces/eventmesh/scripts/validate_architecture.sh`
- **Hive Mind Analysis**: `/workspaces/eventmesh/reports/HIVE_MIND_CONSENSUS_DECISION.md`

---

## Key Files to Watch

### riptide-types
- **Before**: 3,250 lines
- **Now**: 2,892 lines (-358, -11%)
- **Target**: 2,000 lines (-1,250, -38%)
- **Status**: 🟡 71% to target

### riptide-domain
- **Before**: 0 lines (didn't exist)
- **Now**: 475 lines (43% populated)
- **Target**: 859 lines (100% populated)
- **Status**: 🟡 55% complete

### riptide-facade
- **Dependencies Now**: 11 concrete crates
- **Dependencies Target**: 1 (riptide-types only)
- **Status**: ⏳ Week 3 (Phase 3)

---

## The Big Picture

```
API Layer (riptide-api)
    ↓ depends on
Facade Layer (riptide-facade) → ONLY depends on traits in riptide-types
    ↓ orchestrates
Domain Layer (riptide-domain) → Business logic, pure functions
    ↓ uses
Types Layer (riptide-types) → Data structures + traits ONLY
```

**Clean Architecture Achieved When**:
- ✅ Dependencies point inward (outer → inner)
- ✅ Inner layers know nothing about outer layers
- ✅ Business logic isolated in domain
- ✅ Infrastructure swappable via traits
- ✅ Handlers are thin (validate → delegate → serialize)

---

**Keep This Page Handy! Pin it to your workspace.**

*For full details, see: `/workspaces/eventmesh/docs/roadmap/ARCHITECTURE_CLEANUP_DEFINITIVE_ROADMAP.md`*

**FOR THE HIVE! 🐝**
