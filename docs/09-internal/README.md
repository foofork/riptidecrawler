# Internal Documentation

This directory contains internal documentation, historical records, and maintainer-specific materials.

## 📂 Directory Structure

### Project History (`project-history/`)

Comprehensive archive of RipTide's development journey, organized by document type.

#### Completed Phases (`project-history/phases/`)

Documentation from completed development phases:

- **`phase2-completion/`** (4 reports) - Phase 2: Application Layer
  - PHASE_2_COMPLETE_SUMMARY.md
  - PHASE_2_COORDINATOR_REPORT.md
  - PHASE_2_PRODUCTION_READY.md
  - PHASE_2_VALIDATION_REPORT.md

- **`phase3-4-completion/`** (2 reports) - Phase 3-4: Handler Refactoring
  - PHASE3-4_DELIVERABLE.md
  - PHASE3-4_APPSTATE_ELIMINATION_SUMMARY.md

- **`phase5-completion/`** (1 report) - Phase 5: Quality Validation
  - phase5_quality_validation_report.md

#### Migration Reports (`project-history/migration/`)

Records of major architectural migrations:

- **`appstate-migration/`** (4 reports) - AppState→ApplicationContext migration
  - MIGRATION_COMPLETE_SUMMARY.md
  - MIGRATION_FINAL_REPORT.md
  - MIGRATION_STATUS_ACTUAL.md
  - MIGRATION_VALIDATION_REPORT.md

- **`handler-refactoring/`** (1 report) - Handler layer refactoring
  - HANDLER_MIGRATION_COMPLETE.md

- **`coordination/`** (1 report) - Migration coordination
  - migration-coordination-status.md

#### Sprint Planning (`project-history/sprints/`)

Sprint planning documents and coordination summaries:

- **`coordination/`** (1 document) - Sprint coordination
  - COORDINATION-EXECUTIVE-SUMMARY.md

- **`facade-refactoring/`** (1 document) - Facade refactoring sprint
  - sprint-plan-facade-refactoring.md

- **`appstate-migration/`** (1 document) - One-shot migration sprint
  - design-sprint-plan-one-shot.md

- **`decisions/`** (1 document) - Go/no-go decisions
  - GO-NO-GO-DECISION.md

#### Design Artifacts (`project-history/design/`)

Historical design documents and validation checklists:

- **`roadmap-evolution/`** (2 documents) - Roadmap evolution history
  - design-roadmap-concise.md
  - DESIGN-SUMMARY-ROADMAP-RESTRUCTURE.md

- **`validation/`** (1 document) - Design validation
  - design-validation-checklist.md

- **`facade-refactoring/`** (1 document) - Facade design audits
  - facade_verification_audit.md

#### Quality & Status Reports (`project-history/reports/`)

Historical quality assessments and status reports:

- **`quality/`** (5 reports) - Quality and validation reports
  - FINAL_ACTUAL_STATUS.md
  - FINAL_VALIDATION_REPORT.md
  - CRITICAL_FIX_STATUS.md
  - quality_baseline_report.md
  - code_review_report.md

- **Architecture health report** (1 report)
  - architecture-health-report-2025-11-12.md (798 lines, first analysis)

#### Architecture Planning (`project-history/architecture-planning/`)

Historical architecture planning documents (superseded by current docs):

- **`README.md`** - Old architecture directory index
- **`ADR-001-appstate-elimination.md`** - AppState elimination decision record
- **`ARCHITECTURE_DELIVERABLES.md`** - Historical deliverables
- **`CIRCULAR_DEPENDENCY_RESOLUTION.md`** - Dependency resolution strategies
- **`application-context-design.md`** - ApplicationContext design
- **`migration-strategy.md`** - Migration planning
- **`port-trait-specifications.md`** - Port trait specifications

**Note:** Current architecture documentation is in `/docs/04-architecture/`, especially the comprehensive [HEXAGONAL_ARCHITECTURE.md](../04-architecture/HEXAGONAL_ARCHITECTURE.md).

#### Roadmap Snapshots (`project-history/roadmap-snapshots/`)

Historical versions of the roadmap:

- **`ROADMAP-2025-11-11.md`** - Roadmap snapshot from November 11, 2025

**Current roadmap:** [docs/roadmap/README.md](../roadmap/README.md)

#### Analysis & Instructions (`project-history/analysis/`)

Internal analysis and agent instructions:

- **`instructions/`** (2 files) - Internal instructions
  - analysisinstructions.md
  - hygieneinstructions.md

### Configuration (`configuration/`)

Internal configuration and development settings.

---

## 📋 Finding Historical Documents

### By Type

| Document Type | Location | Count |
|---------------|----------|-------|
| Phase Reports | `project-history/phases/` | 7 |
| Migration Reports | `project-history/migration/` | 6 |
| Sprint Planning | `project-history/sprints/` | 4 |
| Design Artifacts | `project-history/design/` | 4 |
| Quality Reports | `project-history/reports/quality/` | 5 |
| Architecture Planning | `project-history/architecture-planning/` | 7 |
| Roadmap Snapshots | `project-history/roadmap-snapshots/` | 1 |
| Analysis | `project-history/analysis/` | 2 |
| **Total** | | **36** |

### By Date

Most documents are from 2024-2025 during major refactoring phases.

### By Phase

- **Phase 0** - Initial cleanup and foundation
- **Phase 1** - Ports and adapters implementation
- **Phase 2** - Application layer (riptide-facade)
- **Phase 3-4** - Handler refactoring and AppState elimination
- **Phase 5** - Quality validation and production readiness

---

## 🎯 Common Questions

**Q: Where are the phase completion reports?**
A: See `project-history/phases/` - organized by phase number.

**Q: How was the AppState migration done?**
A: See `project-history/migration/appstate-migration/` - 4 detailed reports document the entire migration.

**Q: What's the current architecture?**
A: See [/docs/04-architecture/HEXAGONAL_ARCHITECTURE.md](../04-architecture/HEXAGONAL_ARCHITECTURE.md) - comprehensive current documentation.

**Q: Where are old roadmaps?**
A: See `project-history/roadmap-snapshots/` - timestamped historical versions.

**Q: What lessons were learned?**
A: Check phase completion reports and migration final reports - they contain "Lessons Learned" sections.

---

## 📅 Archive Organization

**Archived:** 2025-11-12
**Strategy:** Preserve all historical documents with complete audit trail
**Organization:** By document type (phases, migration, sprints, design, reports)
**Reason:** 97% root directory cleanup (35 files → 1 file in docs/)

**Cleanup Plan:** [project-history/DOCUMENTATION_CLEANUP_PLAN-2025-11-12.md](./project-history/DOCUMENTATION_CLEANUP_PLAN-2025-11-12.md)

---

## 🔗 Related Documentation

- **[Current Documentation](../README.md)** - Main documentation index
- **[Architecture](../04-architecture/README.md)** - Current architecture documentation
- **[Roadmap](../roadmap/README.md)** - Current v1.0 roadmap
- **[Development](../05-development/README.md)** - Contributing and development

---

**For current project status and active development, see the main [documentation](../README.md).**
