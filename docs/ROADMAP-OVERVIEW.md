# Riptide Facade Refactoring Roadmap v2.0 - Overview

## Executive Summary

**Objective**: Transform Riptide from 24% hexagonal compliance to production-ready architecture with 95%+ compliance, eliminating the AppState god object and establishing clean port-adapter boundaries.

**Timeline**: 16 weeks (4 months)
**Success Probability**: 75% (up from 30% in v1 after critical amendments)
**ROI**: $199k over 3 years (developer efficiency, reduced incidents, faster onboarding)

## Critical Success Factors

### 1. Hybrid AppState Pattern (Weeks 1-9)
- **Keep** `AppState { context: ApplicationContext, facades: ... }` wrapper until Sprint 10
- Prevents premature removal that caused v1's 30% failure probability
- Allows gradual migration without breaking existing integrations

### 2. Port-First Strategy (Week 0 & Sprint 2)
- **Audit existing ports BEFORE creating new ones**
- Stop recreating `BrowserDriver`, `HttpClient`, `CacheStorage` (already exist)
- Add only 7 genuinely missing ports in Sprint 5.5

### 3. Per-Sprint Validation
- **Runtime smoke tests** for top-3 routes every sprint
- **Rollback drills** every sprint to prove feature flags work
- **Quality gates** must pass before marking sprint complete

### 4. Testing Shifted Left
- Unit tests written concurrently with port creation (not deferred to Week 7)
- Integration tests per sprint, not batched at end
- 90% coverage maintained throughout, never dropped

## Quality Gate Philosophy

> **"A sprint is only complete when the system builds, runs, passes tests, and can roll back safely."**

No phase is marked "done" until:
- System runs end-to-end in production mode
- All core routes return correct responses
- Feature flags enable safe rollback
- Tests remain green with no coverage reduction

## Roadmap Structure

| Phase | Duration | Focus | Key Deliverables |
|-------|----------|-------|------------------|
| **Week 0** | 1 week | Baseline & Validation | Dependency map, port inventory, baseline metrics |
| **Phase 1** | Weeks 1-3 | P0 Critical Blockers | Hybrid AppState, 5 missing ports, circular deps broken |
| **Phase 2** | Weeks 4-6 | P1 Infrastructure | 7 missing ports, 12 adapters, facade migrations |
| **Phase 3** | Weeks 7-9 | P1 Testing & Validation | 12 untested facades, integration tests, resilience tests |
| **Phase 4** | Weeks 10-16 | Integration & Production | AppState removal, feature flag flip, production hardening |

## Success Metrics

### Technical Metrics
- **Hexagonal Compliance**: 24% → 95%+
- **Test Coverage**: 61% → 90%+
- **Ignored Tests**: 44 → 0
- **Infrastructure Violations**: 32 → 0
- **Circular Dependencies**: 8 → 0

### Business Metrics
- **Deployment Frequency**: Baseline → 2x (safer releases)
- **MTTR**: Baseline → -40% (better observability)
- **Onboarding Time**: Baseline → -60% (clearer architecture)
- **Developer Velocity**: Baseline → +35% (reduced coupling)

## Risk Mitigation

### High-Risk Areas
1. **AppState Migration** (Sprint 3-10)
   - Mitigation: Hybrid pattern, feature flags, per-sprint rollback drills
2. **Port Creation** (Sprint 2, 5.5)
   - Mitigation: Audit-first approach, stop recreating existing ports
3. **Facade Migration** (Sprint 6-9)
   - Mitigation: One facade per day, smoke tests after each

### Rollback Strategy
- **Feature Flags**: `legacy-appstate` (default on), `new-context` (opt-in)
- **Rollback Drills**: Every sprint tests flag flip with zero downtime
- **Rollback Budget**: <5 minutes per rollback, <2 hour to full recovery

## Team Structure

### Core Team (Full-Time)
- **Tech Lead**: Architecture decisions, code reviews, blocker resolution
- **Backend Engineer 1**: Port/adapter implementation, facade migration
- **Backend Engineer 2**: Testing infrastructure, resilience patterns
- **QA Engineer**: Smoke tests, integration tests, rollback validation

### Part-Time Contributors
- **DevOps**: CI/CD pipeline updates, feature flag infrastructure
- **Product Owner**: Stakeholder communication, sprint planning

## Documentation Deliverables

Each sprint produces:
1. **Architecture Decision Records (ADRs)** for new ports/patterns
2. **Dependency Matrix Updates** showing port wiring
3. **Migration Guides** for other teams using affected facades
4. **Runbook Updates** for new resilience patterns

## Quality Gates Overview

### 6 Mandatory Gates Per Sprint

| # | Gate | Purpose |
|---|------|---------|
| 1 | **Builds in both modes** | Verify `cargo build` succeeds for `legacy-appstate` and `new-context` |
| 2 | **Top routes run** | 3 core endpoints return expected results |
| 3 | **All ports wired** | Every facade dependency resolves through `ApplicationContext::validate()` |
| 4 | **Tests pass** | Unit, integration, smoke tests green; coverage not reduced |
| 5 | **Rollback works** | Feature flag flip restores legacy path without errors |
| 6 | **Docs updated** | Dependency matrix and ADRs current for new ports/facades |

### Per-Sprint Checklist
- [ ] Code freeze (no new features during gate verification)
- [ ] Run full quality gate suite
- [ ] Document any gate failures and remediation
- [ ] Stakeholder sign-off before proceeding to next sprint

See `ROADMAP-QUALITY-GATES.md` for detailed gate procedures.

## Amendments from v1 → v2

Key changes addressing critical blockers identified in technical validation:

1. **Week 0 Added**: Baseline capture, dependency mapping, port audit (prevents duplicate work)
2. **Hybrid AppState Pattern**: Keep wrapper until Sprint 10 (prevents premature removal)
3. **Port Audit First**: Stop recreating existing ports (saves 2 weeks, reduces conflicts)
4. **7 Missing Ports in Sprint 5.5**: Only create genuinely missing ports (not 12)
5. **Per-Sprint Smoke Tests**: Top-3 routes tested every sprint (catch issues early)
6. **16-Week Timeline**: Realistic buffer (12 weeks was overly optimistic)

See `ROADMAP-AMENDMENTS.md` for full blocker resolutions.

## Navigation

- **[Week 0: Baseline & Validation](ROADMAP-WEEK-0.md)**
- **[Phase 1: P0 Critical Blockers](ROADMAP-PHASE-1.md)** (Weeks 1-3)
- **[Phase 2: P1 Infrastructure](ROADMAP-PHASE-2.md)** (Weeks 4-6)
- **[Phase 3: P1 Testing](ROADMAP-PHASE-3.md)** (Weeks 7-9)
- **[Phase 4: Integration & Production](ROADMAP-PHASE-4.md)** (Weeks 10-16)
- **[Quality Gates](ROADMAP-QUALITY-GATES.md)**
- **[Amendments Log](ROADMAP-AMENDMENTS.md)**

## Contact & Escalation

- **Roadmap Questions**: Tech Lead
- **Blocker Escalation**: Product Owner
- **Technical Disputes**: Architecture Review Board (weekly sync)
- **Sprint Planning**: Mondays 9 AM (2-hour block)

---

**Version**: 2.0
**Last Updated**: 2025-11-10
**Status**: Ready for Week 0 kickoff
