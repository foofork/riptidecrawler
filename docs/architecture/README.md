# Architecture Documentation

This directory contains the essential architectural documentation for the Riptide EventMesh refactoring project.

## 📁 Essential Documents

### 1. ENHANCED_LAYERING_ROADMAP.md
**Purpose:** Complete 8-week refactoring roadmap for team execution

**Contents:**
- Phase 0: Pre-Refactoring Cleanup (Week 0)
  - robots.rs split strategy (utils + reliability)
  - Memory manager consolidation
  - Redis scope definition (≤2 crates)

- Phase 1: Ports & Adapters Foundation (Weeks 1-2)
  - 10+ port trait definitions
  - Adapter implementations
  - ApplicationContext composition root

- Phase 2: Application Layer Enhancements (Weeks 3-4)
  - Authorization policies
  - Idempotency & transactions
  - Transactional outbox pattern

- Phase 3: Handler Refactoring (Weeks 5-6)
  - All handlers <50 LOC
  - 5 new facades
  - Domain type migrations

- Phase 4: Infrastructure Consolidation (Week 7)
  - Unified HTTP client
  - Redis manager
  - Circuit breakers

- Phase 5: Validation Automation (Week 8)
  - Enhanced validation scripts
  - cargo-deny integration
  - Pre-commit hooks
  - GitHub Actions CI/CD

**Status:** ✅ Ready for execution (100% compliant with requirements)

### 2. ROADMAP_CLARIFICATIONS.md
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

### 1. Read the Roadmap
```bash
# Review the complete 8-week plan
cat docs/architecture/ENHANCED_LAYERING_ROADMAP.md
```

### 2. Understand the Requirements
```bash
# Review architectural clarifications
cat docs/architecture/ROADMAP_CLARIFICATIONS.md
```

### 3. Run Validation
```bash
# Ensure current state is understood
./scripts/validate_architecture_enhanced.sh
```

### 4. Begin Phase 0
```bash
# Start with Task 0.1.1: Robots.rs Split
# See ENHANCED_LAYERING_ROADMAP.md lines 48-145
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

---

**Last Updated:** 2025-11-08
**Status:** ✅ Ready for Team Execution
**Compliance:** 100%
