# Design Document: Concise Status-Oriented Roadmap

**Document Type**: Architecture Design
**Date**: 2025-11-11
**Status**: Design Review
**Purpose**: High-level roadmap with concise status updates

---

## Executive Summary

This design document defines the structure and format for a **concise, status-oriented roadmap** that replaces verbose phase descriptions with actionable status indicators and deliverables. The roadmap focuses on **what** is being delivered and **when**, removing **why** explanations and detailed task lists.

---

## Design Principles

### 1. Status-First Approach

Every phase/sprint leads with a **visual status indicator**:

- ✅ **Complete**: All deliverables shipped, quality gates passed
- 🔄 **In Progress**: Active work, partially complete
- ⏸️ **Blocked**: Waiting on dependency or blocker resolution
- ⏭️ **Not Started**: Scheduled but not yet begun
- ⚠️ **At Risk**: Behind schedule or facing issues

### 2. Conciseness Over Completeness

**Remove**:
- Verbose rationale ("why we're doing this")
- Detailed task lists (belong in sprint plan)
- Historical context (put in ADRs instead)
- Architecture explanations (link to separate docs)

**Keep**:
- Current status
- Timeline (week/sprint number)
- Goal (single sentence)
- Deliverables (3-5 bullets max)
- Quality gate summary (score + link)

### 3. Deliverable-Focused

Each phase describes **outputs**, not **activities**:
- ❌ "Migrate AppState fields" (activity)
- ✅ "ApplicationContext as sole state system" (output)

### 4. Quality Gate Integration

Reference sprint plan for details, show summary:
- **Format**: `10/10 checks passing` or `8/10 (2 blockers)`
- **Link**: Direct link to sprint plan quality gate section
- **Binary**: Pass/fail, no partial credit

---

## Roadmap Structure

### Header Section

```markdown
# RipTide Crawler Refactoring Roadmap

**Project**: Hexagonal Architecture Migration
**Timeline**: 12 weeks (Weeks 1-12)
**Current Phase**: Phase 1 - Critical Blockers
**Overall Status**: 🔄 In Progress (Week 0 Complete)

---

## Overview

Transform facade layer to production-ready hexagonal architecture:
- Eliminate AppState god object (2213 LOC → 0)
- Migrate 12 facades to port-adapter pattern
- Achieve 90%+ test coverage
- Deploy to production with zero infrastructure violations

**Success Metrics**:
- Hexagonal compliance: 24% → 95%
- Test coverage: <50% → 90%+
- Circular dependencies: 8 → 0
- Infrastructure violations: 32 → 0

---
```

### Phase Format Template

```markdown
## Phase [N]: [Phase Name]

**Status**: [Status Icon] [Status Text]
**Timeline**: Week [X]-[Y] | Sprint [A]-[B]
**Goal**: [Single sentence describing outcome]

**Deliverables**:
- [Output 1]
- [Output 2]
- [Output 3]

**Quality Gate**: [Score]/[Total] checks passing → [Link to sprint plan]

**Blockers**: [None | List blockers if ⏸️]

---
```

### Example Phase (One-Shot Migration)

```markdown
## Phase 1: AppState Elimination

**Status**: ⏭️ Not Started
**Timeline**: Week 1 | Milestone 1
**Goal**: Eliminate AppState god object via atomic bulk migration

**Deliverables**:
- ApplicationContext as sole state system
- Zero AppState references in codebase (`grep` verified)
- Circular dependencies eliminated
- ADR-001: AppState elimination documented

**Quality Gate**: 10/10 checks passing → [Sprint Plan §1.5](#)

**Blockers**: None

---
```

---

## Complete Roadmap Design

### File Structure

```markdown
# RipTide Crawler Refactoring Roadmap
[Header with project overview]

## Phase 1: Critical Blockers (Weeks 1-3)
### Phase 1A: AppState Elimination (Week 1)
### Phase 1B: Infrastructure Violations Part 1 (Week 2)
### Phase 1C: Infrastructure Violations Part 2 (Week 3)

## Phase 2: Facade Testing (Weeks 4-9)
### Phase 2A: Core Facades (Weeks 4-6)
### Phase 2B: Specialized Facades (Weeks 7-9)

## Phase 3: Production Readiness (Weeks 10-12)
### Phase 3A: Integration Testing (Week 10)
### Phase 3B: Performance Validation (Week 11)
### Phase 3C: Deployment (Week 12)

## Deferred Work
[Link to ROADMAP-ADDENDUM-DEFERRED-WORK.md]

## Appendices
- Architecture Diagrams
- Quality Gates Summary
- Risk Register
```

---

## Full Roadmap Content

### Complete ROADMAP.md Design

```markdown
# RipTide Crawler Refactoring Roadmap

**Project**: Hexagonal Architecture Migration
**Timeline**: 12 weeks (3 months)
**Start Date**: [TBD]
**Current Phase**: Phase 1 - Critical Blockers
**Overall Status**: 🔄 In Progress (Week 0 Baseline Complete)

---

## Overview

Transform the RipTide Crawler facade layer from a tightly-coupled monolith to a production-ready hexagonal architecture. This refactoring addresses critical architectural debt while maintaining zero downtime.

**Baseline (Week 0)**:
- Hexagonal compliance: 24%
- Test coverage: <50%
- Circular dependencies: 8
- Infrastructure violations: 32
- AppState LOC: 2,213 (40+ fields)

**Target (Week 12)**:
- Hexagonal compliance: 95%+
- Test coverage: 90%+
- Circular dependencies: 0
- Infrastructure violations: 0
- AppState LOC: 0 (eliminated)

---

## Phase 1: Critical Blockers (Weeks 1-3)

**Goal**: Eliminate god objects, circular dependencies, and infrastructure violations.

---

### Phase 1A: AppState Elimination (Week 1)

**Status**: ⏭️ Not Started
**Timeline**: Week 1 (5 business days)
**Sprint**: Milestone 1 - One-Shot Migration

**Goal**: Eliminate AppState god object via atomic bulk migration to ApplicationContext.

**Deliverables**:
- ApplicationContext as sole state system
- Zero AppState references (`grep -R "\bAppState\b" crates/` returns 0)
- All handlers migrated to `State<Arc<ApplicationContext>>`
- All 12 facades instantiate with ApplicationContext
- Circular dependencies eliminated (verified via `cargo tree`)
- ADR-001: AppState elimination documented

**Quality Gate**: 10/10 checks passing
- [View Quality Gate](./sprint-plan-facade-refactoring.md#one-shot-migration-quality-gate)

**Blockers**: None

---

### Phase 1B: Infrastructure Violations Part 1 (Week 2)

**Status**: ⏭️ Not Started
**Timeline**: Week 2 (5 business days)
**Sprint**: Sprint 5

**Goal**: Fix 16/32 infrastructure violations via port-adapter pattern.

**Deliverables**:
- 8 new port traits created (HttpClient, CacheStorage, etc.)
- 8 adapters implemented for existing infrastructure
- 50% reduction in direct infrastructure coupling
- Zero compilation warnings (`cargo clippy -- -D warnings`)

**Quality Gate**: 8/8 checks passing
- [View Quality Gate](./sprint-plan-facade-refactoring.md#sprint-5-quality-gate)

**Blockers**: Depends on Phase 1A completion

---

### Phase 1C: Infrastructure Violations Part 2 (Week 3)

**Status**: ⏭️ Not Started
**Timeline**: Week 3 (5 business days)
**Sprint**: Sprint 6

**Goal**: Eliminate remaining 16/32 infrastructure violations.

**Deliverables**:
- 8 additional port traits created
- 8 additional adapters implemented
- 100% infrastructure accessed via ports
- Zero direct infrastructure imports in domain layer

**Quality Gate**: 8/8 checks passing
- [View Quality Gate](./sprint-plan-facade-refactoring.md#sprint-6-quality-gate)

**Blockers**: Depends on Phase 1B completion

---

## Phase 2: Facade Testing (Weeks 4-9)

**Goal**: Achieve 90%+ test coverage across all 12 facades.

---

### Phase 2A: Core Facades (Weeks 4-6)

**Status**: ⏭️ Not Started
**Timeline**: Weeks 4-6 (15 business days)
**Sprints**: Sprints 7-9

**Goal**: Test core crawling, extraction, and search facades.

**Deliverables**:
- CrawlerFacade: 90%+ coverage (unit + integration)
- ExtractorFacade: 90%+ coverage
- SearchFacade: 90%+ coverage
- 100+ new tests written
- Zero failing tests (`cargo test --workspace`)

**Quality Gate**: 3/3 facade quality gates passing
- [View Quality Gate](./sprint-plan-facade-refactoring.md#phase-2a-quality-gate)

**Blockers**: Depends on Phase 1 completion

---

### Phase 2B: Specialized Facades (Weeks 7-9)

**Status**: ⏭️ Not Started
**Timeline**: Weeks 7-9 (15 business days)
**Sprints**: Sprints 10-12

**Goal**: Test browser, streaming, and remaining facades.

**Deliverables**:
- BrowserFacade: 90%+ coverage
- StreamingFacade: 90%+ coverage
- Remaining 7 facades: 90%+ coverage each
- 150+ additional tests written
- Comprehensive integration test suite

**Quality Gate**: 9/9 facade quality gates passing
- [View Quality Gate](./sprint-plan-facade-refactoring.md#phase-2b-quality-gate)

**Blockers**: Depends on Phase 2A completion

---

## Phase 3: Production Readiness (Weeks 10-12)

**Goal**: Validate production readiness and deploy to production.

---

### Phase 3A: Integration Testing (Week 10)

**Status**: ⏭️ Not Started
**Timeline**: Week 10 (5 business days)
**Sprint**: Sprint 13

**Goal**: End-to-end integration testing across all components.

**Deliverables**:
- 20+ integration test scenarios
- Cross-facade integration validated
- Error handling and retry logic tested
- Performance baseline captured

**Quality Gate**: 5/5 checks passing
- [View Quality Gate](./sprint-plan-facade-refactoring.md#sprint-13-quality-gate)

**Blockers**: Depends on Phase 2 completion

---

### Phase 3B: Performance Validation (Week 11)

**Status**: ⏭️ Not Started
**Timeline**: Week 11 (5 business days)
**Sprint**: Sprint 14

**Goal**: Validate no performance regression from refactoring.

**Deliverables**:
- Performance benchmarks vs baseline
- <10% overhead from port abstraction
- Load testing (1000+ concurrent requests)
- Memory profiling (zero leaks)

**Quality Gate**: 4/4 checks passing
- [View Quality Gate](./sprint-plan-facade-refactoring.md#sprint-14-quality-gate)

**Blockers**: Depends on Phase 3A completion

---

### Phase 3C: Production Deployment (Week 12)

**Status**: ⏭️ Not Started
**Timeline**: Week 12 (5 business days)
**Sprint**: Sprint 15

**Goal**: Deploy refactored system to production with zero downtime.

**Deliverables**:
- Blue-green deployment executed
- Feature flag cleanup
- Production monitoring validated
- Rollback plan tested
- Post-deployment verification complete

**Quality Gate**: 6/6 checks passing
- [View Quality Gate](./sprint-plan-facade-refactoring.md#sprint-15-quality-gate)

**Blockers**: Depends on Phase 3B completion

---

## Deferred Work

Additional enhancements identified during analysis but **not included** in 12-week roadmap:

**Browser Enhancements** (4-6 weeks):
- Multi-browser support (Firefox, WebKit)
- Browser pooling optimizations
- Fix 3 failing CDP tests

**Extraction Enhancements** (3-5 weeks):
- Markdown extraction consolidation
- JSON format standardization
- Image download support
- Non-browser extraction modes

**Streaming Enhancements** (3-5 weeks):
- Event-driven patterns
- WebSocket API
- Real-time progress streaming

**State System Investigation** (1-2 weeks):
- RiptideRuntime analysis
- State system unification plan

**Performance Optimizations** (2-3 weeks):
- Facade profiling and optimization
- Database query optimization
- Cache tuning

**Full Details**: [ROADMAP-ADDENDUM-DEFERRED-WORK.md](./ROADMAP-ADDENDUM-DEFERRED-WORK.md)

**Timeline**: Phases 5-7 (Weeks 17-24) - Post-production deployment

---

## Appendices

### A. Architecture Diagrams

- [Hexagonal Architecture Overview](./architecture/hexagonal-architecture.md)
- [Port-Adapter Pattern](./architecture/port-adapter-pattern.md)
- [ApplicationContext Design](./architecture/application-context.md)

### B. Quality Gates Summary

| Phase | Quality Gate | Checks | Status |
|-------|--------------|--------|--------|
| Phase 1A | One-Shot Migration | 10/10 | ⏭️ Not Started |
| Phase 1B | Infrastructure Part 1 | 8/8 | ⏭️ Not Started |
| Phase 1C | Infrastructure Part 2 | 8/8 | ⏭️ Not Started |
| Phase 2A | Core Facades | 3/3 | ⏭️ Not Started |
| Phase 2B | Specialized Facades | 9/9 | ⏭️ Not Started |
| Phase 3A | Integration Testing | 5/5 | ⏭️ Not Started |
| Phase 3B | Performance | 4/4 | ⏭️ Not Started |
| Phase 3C | Deployment | 6/6 | ⏭️ Not Started |

**Total Quality Checks**: 53

### C. Risk Register

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| AppState migration incomplete | High | Low | One-shot atomic migration, rollback plan |
| Circular dependencies reintroduced | High | Medium | cargo tree validation in quality gate |
| Test coverage < 90% | Medium | Low | Quality gate blocks merge if coverage fails |
| Performance regression | Medium | Low | Benchmarking in Phase 3B |
| Production deployment issues | High | Low | Blue-green deployment, feature flags |

### D. Success Criteria

**Must achieve ALL of the following**:

1. ✅ **Hexagonal Compliance**: 95%+ (measured via architecture lint rules)
2. ✅ **Test Coverage**: 90%+ across all facades
3. ✅ **Zero AppState**: No AppState references in codebase
4. ✅ **Zero Circular Deps**: cargo tree --duplicates shows 0
5. ✅ **Zero Infrastructure Violations**: All infrastructure via ports
6. ✅ **Zero Warnings**: cargo clippy -- -D warnings passes
7. ✅ **Production Deployed**: Deployed with zero downtime
8. ✅ **Performance**: <10% overhead vs baseline

**Scoring**: 8/8 = SUCCESS | <8 = FAIL

---

**Roadmap Version**: 2.0
**Last Updated**: 2025-11-11
**Status**: ✅ Design Complete, Ready for Implementation
**Related Documents**:
- [Sprint Plan (Detailed)](./sprint-plan-facade-refactoring.md)
- [Deferred Work](./ROADMAP-ADDENDUM-DEFERRED-WORK.md)
- [Week 0 Baseline](./BASELINE_SUMMARY.md)
```

---

## Key Design Decisions

### 1. Status Indicators

**Decision**: Use emoji status indicators for visual clarity
**Rationale**: Faster scanning, language-agnostic, mobile-friendly
**Alternatives Considered**: Text-only (rejected: less visual), icons (rejected: markdown support)

### 2. Quality Gate Format

**Decision**: Show score summary in roadmap, link to sprint plan for details
**Rationale**: Balance between overview (roadmap) and detail (sprint plan)
**Format**: `10/10 checks passing → [Link]`

### 3. Deliverables Over Activities

**Decision**: Focus on outputs, not tasks
**Rationale**: Roadmap readers care about *what* is delivered, not *how*
**Example**: "Zero AppState references" (output) vs "Migrate AppState" (task)

### 4. Timeline Granularity

**Decision**: Show both week number and sprint number
**Rationale**: Different stakeholders use different mental models
**Format**: `Week 1 | Milestone 1` or `Weeks 4-6 | Sprints 7-9`

### 5. Deferred Work Placement

**Decision**: Include in roadmap as separate section with link to addendum
**Rationale**: Acknowledge deferred work exists without cluttering roadmap
**Alternative Rejected**: Omit entirely (creates "where's the rest?" questions)

---

## Comparison: Before vs After

### Before (Verbose)

```markdown
## Sprint 1: AppState God Object Removal

### Background
The AppState struct has grown to 2213 lines of code with over 40 fields...
[500 words of explanation]

### Why This Matters
God objects violate single responsibility principle...
[300 words of rationale]

### Detailed Tasks
1. Analyze AppState fields (4 hours)
   - Document each field
   - Create spreadsheet
   - [20 more sub-tasks]
...

[2000+ words total]
```

### After (Concise)

```markdown
## Phase 1A: AppState Elimination

**Status**: ⏭️ Not Started
**Timeline**: Week 1
**Goal**: Eliminate AppState god object via atomic bulk migration

**Deliverables**:
- ApplicationContext as sole state system
- Zero AppState references (grep verified)
- Circular dependencies eliminated
- ADR-001 documented

**Quality Gate**: 10/10 checks passing → [Sprint Plan](#)

**Blockers**: None

---

[~150 words total]
```

**Reduction**: 93% fewer words, same critical information

---

## Validation Checklist

**Before publishing roadmap, verify**:

- [ ] All phases have status indicator (✅🔄⏸️⏭️⚠️)
- [ ] All phases have timeline (week/sprint)
- [ ] All phases have single-sentence goal
- [ ] All phases have 3-5 deliverables max
- [ ] All quality gates show score + link
- [ ] No verbose explanations (move to ADRs)
- [ ] No detailed task lists (move to sprint plan)
- [ ] Deferred work section includes link to addendum
- [ ] Appendices include diagrams, risk register, success criteria
- [ ] Overall status reflects current reality

---

## Implementation Steps

### 1. Create New ROADMAP.md

**Location**: `/workspaces/riptidecrawler/docs/ROADMAP.md`
**Action**: Write new file using template above
**Content**: Full roadmap design from this document

### 2. Archive Old Roadmap Files

**Files to Archive**:
- `ROADMAP-OVERVIEW.md` → `archive/ROADMAP-OVERVIEW-v1.md`
- `ROADMAP-PHASE-1.md` → `archive/ROADMAP-PHASE-1-v1.md`
- `ROADMAP-PHASE-2.md` → `archive/ROADMAP-PHASE-2-v1.md`
- `ROADMAP-PHASE-3.md` → `archive/ROADMAP-PHASE-3-v1.md`
- `ROADMAP-PHASE-4.md` → `archive/ROADMAP-PHASE-4-v1.md`
- `ROADMAP-QUALITY-GATES.md` → Merge into sprint plan
- `ROADMAP-WEEK-0.md` → Keep as `BASELINE_SUMMARY.md`

**Rationale**: Preserve history, reduce clutter

### 3. Update Sprint Plan

**File**: `/workspaces/riptidecrawler/docs/sprint-plan-facade-refactoring.md`
**Action**: Apply one-shot migration design from companion document
**Changes**: Merge Sprints 1-3 into Milestone 1

### 4. Update Cross-References

**Files to Update**:
- `README.md`: Update roadmap link
- `CLAUDE.md`: Update roadmap reference
- `docs/architecture/*.md`: Update roadmap links

---

## Success Metrics

### Readability Metrics

**Before (Old Roadmap)**:
- Total words: ~15,000
- Reading time: ~60 minutes
- Sections: 20+
- Files: 7

**After (New Roadmap)**:
- Total words: ~3,000
- Reading time: ~12 minutes
- Sections: 8 phases + appendices
- Files: 1

**Improvement**: 80% shorter, 80% faster to read

### Stakeholder Feedback

**Questions to Ask**:
1. Can you understand current project status in <5 minutes?
2. Can you identify blockers and risks immediately?
3. Do you need to read sprint plan for overview?
4. Is the deliverable-focused format clear?

**Success = 4/4 "Yes" responses**

---

## Conclusion

### Summary

The concise roadmap design:
- **Status-first**: Visual indicators for quick scanning
- **Deliverable-focused**: Outputs, not activities
- **Quality-aware**: Quality gates integrated
- **Reference-friendly**: Links to detailed sprint plan
- **80% shorter**: Faster to read and understand

### Recommendation

**Implement concise roadmap format** as defined in this document.

### Next Steps

1. **Review design** with team/stakeholders
2. **Create new ROADMAP.md** using template
3. **Archive old roadmap files** to preserve history
4. **Update cross-references** in README, CLAUDE.md
5. **Publish and communicate** new roadmap structure

---

**Design Status**: ✅ Ready for Implementation
**Expected Benefit**: High (80% shorter, 5x faster to read)
**Risk**: Low (preserves all critical information)
