# Architecture Documentation

This directory contains the essential architectural documentation for the Riptide EventMesh refactoring project.

## 📁 Essential Documents

### 1. ENHANCED_LAYERING_ROADMAP.md
**Purpose:** Master index for comprehensive refactoring plan

**Structure:** Index document with links to 6 phase-specific roadmaps

**Evolution:**
- **Original Plan (v1.0):** 8 weeks, 58 files, -1,657 LOC
- **Enhanced Plan (v2.0):** 12 weeks, 96 files, -14,383 LOC (+767% cleanup)
- **Workspace Analysis (v3.0):** 14.5 weeks, 96 files, **-34,103 LOC** (+1,958% cleanup!) ⭐

**Latest Enhancement (v3.0):**
- Comprehensive 29-crate workspace analysis
- Discovered **21,000 LOC of duplicate code**
- Phase 0 expanded with Quick Wins: -18,450 LOC in 9 days
- Total cleanup now **-34,103 LOC** (saves more than 2x the original plan)

**Phase-Specific Roadmaps** (in `/docs/roadmap/`):

#### Phase 0: Pre-Refactoring Cleanup (14.5 days) ⭐ **HIGHEST IMPACT**
📄 **PHASE_0_CLEANUP_ROADMAP.md** (v3.0 with Workspace Analysis)
- Sprint 0.1: Deduplication (robots.rs, memory managers, Redis)
- **Sprint 0.2:** Pipeline consolidation (4 files → 2)
- **Sprint 0.3:** Admin cleanup (delete admin_old.rs)
- **Sprint 0.4: Quick Wins Deduplication** ⭐ (-18,450 LOC in 9 days)
  - Delete duplicate robots.txt: -16,150 LOC
  - Consolidate circuit breakers: -900 LOC (4 → 1)
  - Consolidate Redis clients: -800 LOC (3 → 1)
  - Consolidate rate limiters: -600 LOC (4 → 1)
- Impact: **-22,020 LOC deleted** (857% improvement!)

#### Phase 1: Ports & Adapters Foundation (3 weeks)
📄 **PHASE_1_PORTS_ADAPTERS_ROADMAP.md**
- Sprint 1.1-1.3: Core ports, adapters, composition root (original)
- **NEW Sprint 1.4:** Session port definition
- **NEW Sprint 1.5:** Core infrastructure ports (health, metrics, RPC)
- Impact: +3,600 LOC added, -4,300 LOC deleted

#### Phase 2: Application Layer Enhancements (3 weeks)
📄 **PHASE_2_APPLICATION_LAYER_ROADMAP.md**
- Sprint 2.1-2.4: Authorization, idempotency, transactions, metrics
- **NEW Sprint 2.5:** Middleware refactoring (auth, validation)
- Impact: +1,630 LOC added, -1,200 LOC deleted

#### Phase 3: Handler Refactoring (3 weeks)
📄 **PHASE_3_HANDLER_REFACTORING_ROADMAP.md**
- Sprint 3.1: Large handler migrations (top 10 handlers)
- **NEW Sprint 3.2:** Medium handler migrations (7 handlers, 2,600 LOC)
- **NEW Sprint 3.3:** Render subsystem refactoring (656 LOC)
- **NEW Sprint 3.4:** Route registration audit
- Impact: -8,213 LOC deleted, +5,250 LOC added

#### Phase 4: Infrastructure Consolidation (2 weeks)
📄 **PHASE_4_INFRASTRUCTURE_ROADMAP.md**
- Sprint 4.1-4.2: HTTP client + Redis consolidation
- **NEW Sprint 4.3:** 🚨 Streaming system refactoring (5,427 LOC - CRITICAL)
- **NEW Sprint 4.4:** Resource manager consolidation (1,845 LOC)
- **NEW Sprint 4.5:** Metrics system split (1,670 LOC)
- Impact: -6,370 LOC deleted, +4,000 LOC added

#### Phase 5: Validation Automation (3 days)
📄 **PHASE_5_VALIDATION_ROADMAP.md**
- Sprint 5.1: Enhanced validation scripts
- Sprint 5.2: cargo-deny integration
- Sprint 5.3: Pre-commit hooks
- Sprint 5.4: GitHub Actions CI/CD

**Status:** ✅ Ready for execution (100% compliant, all gaps addressed)

### 2. WORKSPACE_CRATE_ANALYSIS.md ⭐ **NEW in v3.0**
**Purpose:** Comprehensive analysis of all 29 crates for deduplication and proper layering

**Location:** `/reports/WORKSPACE_CRATE_ANALYSIS.md`

**Key Discoveries:**
- **Massive Code Duplication:** 21,000 LOC wasted across workspace
  - Robots.txt duplicated: 16,150 LOC (spider + fetch)
  - 4 circuit breaker implementations: 900 LOC
  - 3 Redis client wrappers: 800 LOC
  - 4 rate limiter implementations: 600 LOC
- **Bloated API Crate:** 75,370 LOC (should be <10,000)
- **Browser Abstraction Failure:** 3 crates with overlapping responsibilities
- **10 Architectural Violations:** Domain depending on infrastructure

**Quick Wins (Week 1):**
- Delete duplicate robots.txt: -16,150 LOC (2 days)
- Consolidate circuit breakers: -900 LOC (3 days)
- Consolidate Redis clients: -800 LOC (2 days)
- Consolidate rate limiters: -600 LOC (2 days)
- **Total: -18,450 LOC in 9 days with LOW RISK**

**Impact:**
- Added Sprint 0.4 to Phase 0 (+9 days, -18,450 LOC)
- Increased total cleanup: -14,383 → -34,103 LOC (+137%)
- Identified 4 phases of workspace cleanup (63 days total)

### 3. API_CRATE_COVERAGE_ANALYSIS.md
**Purpose:** Comprehensive gap analysis that identified missing modules

**Key Findings:**
- Original roadmap covered only 21.7% of handler files
- Identified 83 uncovered files (~18,100 LOC)
- Found 6 critical gaps (streaming, middleware, sessions, etc.)
- Recommended 10 new sprints across all phases

**Impact:** Drove roadmap expansion from 8 weeks → 12 weeks for complete coverage

### 4. ROADMAP_CLARIFICATIONS.md
**Purpose:** Architectural requirements and clarifications

**Contents:**
- Domain & dependency rules
- Facade = Application layer documentation
- Ports & adapters scope
- Robots.rs split strategy
- Redis consolidation rules
- CI validation enhancements
- Execution guidance
- Acceptance criteria

**Status:** ✅ Complete reference for architectural decisions

## 🛠️ Supporting Infrastructure

### Scripts
- **scripts/validate_architecture_enhanced.sh**
  - 12 automated validation checks
  - Domain layer purity verification
  - Handler size limits (<50 LOC)
  - HTTP/JSON leak detection
  - Redis scope validation
  - Duplication detection
  - Clippy strict mode
  - Test coverage checks (≥90%)

### CI/CD
- **.github/workflows/architecture-validation.yml**
  - Automated PR validation
  - Layer boundary enforcement
  - Build and test gates
  - Coverage reporting

### Dependency Enforcement
- **deny.toml**
  - Compile-time layer boundary enforcement
  - Domain cannot depend on API/Facade/Infrastructure
  - Facades cannot depend on HTTP frameworks or databases
  - Redis scoped to specific crates

## 🎯 Architectural Principles

### Layering Rules
```
API Layer (riptide-api)
  ↓ calls
Application Layer (riptide-facade)
  ↓ uses
Domain Layer (riptide-types)
  ↑↓ defines ports
Infrastructure Layer (riptide-*)
```

### Handler Rules
- **Size:** <50 LOC (strict target)
- **ALLOWED:** Simple `if` for input validation
- **FORBIDDEN:** Business logic loops, complex conditionals
- **Pattern:** Validate → Map → Call Facade → Map Response

### Facade Rules
- **NO** HTTP types (actix_web, axum, hyper)
- **NO** database types (sqlx, postgres)
- **NO** serde_json::Value (use typed DTOs)
- **YES** Use-case orchestration
- **YES** Port trait dependencies only

### Domain Rules
- **Pure domain types only**
- **Port trait definitions**
- **NO dependencies on higher layers**
- **NO infrastructure dependencies**

## 🚀 Getting Started

### 1. Read the Master Index
```bash
# Review the 12-week plan overview
cat docs/architecture/ENHANCED_LAYERING_ROADMAP.md
```

### 2. Review Phase-Specific Roadmaps
```bash
# Read phases in order
cat docs/roadmap/PHASE_0_CLEANUP_ROADMAP.md
cat docs/roadmap/PHASE_1_PORTS_ADAPTERS_ROADMAP.md
cat docs/roadmap/PHASE_2_APPLICATION_LAYER_ROADMAP.md
cat docs/roadmap/PHASE_3_HANDLER_REFACTORING_ROADMAP.md
cat docs/roadmap/PHASE_4_INFRASTRUCTURE_ROADMAP.md
cat docs/roadmap/PHASE_5_VALIDATION_ROADMAP.md
```

### 3. Understand the Requirements
```bash
# Review architectural clarifications
cat docs/architecture/ROADMAP_CLARIFICATIONS.md

# Review gap analysis that drove the enhancements
cat docs/architecture/API_CRATE_COVERAGE_ANALYSIS.md
```

### 4. Run Validation
```bash
# Ensure current state is understood
./scripts/validate_architecture_enhanced.sh
```

### 5. Begin Phase 0
```bash
# Start with Sprint 0.1: Deduplication
# See docs/roadmap/PHASE_0_CLEANUP_ROADMAP.md
```

## 📊 Success Metrics

| Metric | Target | Validation |
|--------|--------|------------|
| Handler LOC | <50 | Script check |
| Domain purity | 100% | `cargo tree` |
| Facade coverage | ≥90% | `cargo tarpaulin` |
| Redis scope | ≤2 crates | Script check |
| Clippy warnings | 0 | `-D warnings` |
| Build warnings | 0 | `-D warnings` |

## 🔍 Validation Commands

```bash
# Layer boundary enforcement
cargo deny check bans

# Domain purity check
cargo tree -p riptide-types --invert riptide-types | \
  grep -iE 'riptide-(api|facade|reliability|cache)'
# Expected: No matches

# Handler size check
find crates/riptide-api/src/handlers -name "*.rs" -exec wc -l {} + | \
  awk '$1 > 50 {print}'
# Expected: No output

# Full validation suite
./scripts/validate_architecture_enhanced.sh
```

## 📝 Notes

- All interim analysis documents have been removed
- Only essential execution documents remain
- Team should reference ENHANCED_LAYERING_ROADMAP.md as primary guide
- ROADMAP_CLARIFICATIONS.md provides detailed architectural rules
- Validation automation ensures compliance at each phase

## 📂 File Structure

```
docs/
├── architecture/
│   ├── ENHANCED_LAYERING_ROADMAP.md (INDEX - start here!)
│   ├── API_CRATE_COVERAGE_ANALYSIS.md (gap analysis)
│   ├── ROADMAP_CLARIFICATIONS.md (requirements)
│   └── README.md (this file)
└── roadmap/
    ├── PHASE_0_CLEANUP_ROADMAP.md (5 days)
    ├── PHASE_1_PORTS_ADAPTERS_ROADMAP.md (3 weeks)
    ├── PHASE_2_APPLICATION_LAYER_ROADMAP.md (3 weeks)
    ├── PHASE_3_HANDLER_REFACTORING_ROADMAP.md (3 weeks)
    ├── PHASE_4_INFRASTRUCTURE_ROADMAP.md (2 weeks)
    └── PHASE_5_VALIDATION_ROADMAP.md (3 days)
```

---

**Last Updated:** 2025-11-08
**Status:** ✅ Ready for Team Execution - **START WITH PHASE 0 QUICK WINS!**
**Version:** 3.0 (with Workspace Crate Analysis)
**Compliance:** 100%
**Coverage:** 96 files (78.3% of handlers) + 29 crates analyzed
**Timeline:** 14.5 weeks (enhanced from 8 weeks)
**LOC Cleanup:** **-34,103 net** (+1,958% improvement!) ⭐

**Quick Wins Available:** -18,450 LOC in 9 days (Phase 0 Sprint 0.4)
