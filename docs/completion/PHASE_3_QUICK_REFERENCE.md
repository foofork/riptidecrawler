# Phase 3 Sprints 3.2-3.4: Quick Reference Guide

**Date:** 2025-11-08
**Status:** 📋 **READY TO EXECUTE**

---

## 🎯 Quick Stats

| Metric | Sprint 3.2 | Sprint 3.3 | Sprint 3.4 | Total |
|--------|------------|------------|------------|-------|
| **Agents** | 4 parallel | 1 sequential | 1 sequential | 6 agents |
| **Facades** | 7 new | 1 new | 0 | 8 facades |
| **Handler LOC Reduced** | -2,048 | -646 | Variable | -2,694+ |
| **Facade LOC Created** | +3,400 | +900 | 0 | +4,300 |
| **Tests Added** | 112+ | 20+ | 10+ | 142+ |
| **Duration** | 3 days | 2 days | 2 days | 7 days |
| **Speedup** | 3.6x | N/A | N/A | 3.6x overall |

---

## 📦 Sprint 3.2: Medium Handlers (7 Facades)

### Agent #1: Chunking & Memory
- **Facades:** ChunkingFacade (450 LOC) + MemoryFacade (400 LOC)
- **Tests:** 27+ unit tests
- **Handlers:** chunking.rs (356 → <50) + memory.rs (313 → <50)
- **Time:** 9 hours

### Agent #2: Monitoring & Pipeline
- **Facades:** MonitoringFacade (600 LOC) + PipelinePhasesFacade (350 LOC)
- **Tests:** 34+ unit tests
- **Handlers:** monitoring.rs (344 → <50) + pipeline_phases.rs (289 → <50)
- **Time:** 10 hours

### Agent #3: Strategies & Search
- **Facades:** StrategiesFacade (550 LOC) + DeepSearchFacade (500 LOC)
- **Tests:** 36+ unit tests
- **Handlers:** strategies.rs (336 → <50) + deepsearch.rs (310 → <50)
- **Time:** 12 hours

### Agent #4: Streaming
- **Facade:** StreamingFacade (550 LOC)
- **Tests:** 15+ unit tests
- **Handler:** streaming.rs (300 → <50)
- **Time:** 6 hours

**Parallel Execution:** All 4 agents work simultaneously
**Coordination:** Claude-flow hooks + memory sharing

---

## 🎨 Sprint 3.3: Render Subsystem (1 Facade)

### Agent #5: Render Architect
- **Facade:** RenderFacade (900 LOC)
- **Tests:** 20+ unit tests
- **Handlers:** render/handlers.rs (362 → <50) + render/processors.rs (334 → 0, merged)
- **Time:** 8 hours

**Consolidation:** Unifies 2 handlers into 1 comprehensive facade

---

## 🔍 Sprint 3.4: Route Audit (0 Facades, Refactoring)

### Agent #6: Route Auditor
- **Audit:** 8 route files (347 LOC total)
- **Refactor:** routes/profiles.rs (124 LOC), routes/pdf.rs (58 LOC), routes/stealth.rs (52 LOC)
- **Deliverable:** Route audit report + refactored routes
- **Time:** 6 hours

**Goal:** Ensure all route files <30 LOC with zero business logic

---

## 🚀 Execution Commands

### Sprint 3.2: Spawn 4 Agents (Parallel)

```javascript
// Single message with all agents
Task("Chunking & Memory Specialist", "Create ChunkingFacade + MemoryFacade. 27+ tests.", "coder")
Task("Monitoring & Pipeline Analyst", "Create MonitoringFacade + PipelinePhasesFacade. 34+ tests.", "analyst")
Task("Strategies & Search Orchestrator", "Create StrategiesFacade + DeepSearchFacade. 36+ tests.", "researcher")
Task("Streaming Specialist", "Create StreamingFacade. 15+ tests.", "optimizer")
```

### Sprint 3.3: Spawn 1 Agent (Sequential)

```javascript
Task("Render Subsystem Architect", "Create unified RenderFacade. 20+ tests.", "system-architect")
```

### Sprint 3.4: Spawn 1 Agent (Sequential)

```javascript
Task("Route Auditor & Refactoring Specialist", "Audit 8 route files. Generate report. Refactor as needed.", "reviewer")
```

---

## 📋 Daily Checklist

### Day 1 (Sprint 3.2 Start)
- [ ] All 4 agents spawn successfully
- [ ] Pre-task hooks executed
- [ ] Facade design documents created
- [ ] Port interfaces defined
- [ ] Initial implementation (30-40% complete)

### Day 2 (Sprint 3.2 Implementation)
- [ ] All facades 100% implemented
- [ ] 112+ unit tests written
- [ ] 7 handlers refactored to <50 LOC
- [ ] Memory coordination verified

### Day 3 (Sprint 3.2 Quality Gates)
- [ ] All facades compile without errors
- [ ] Zero clippy warnings
- [ ] All tests pass (100% success rate)
- [ ] Handler LOC targets met
- [ ] Sprint 3.2 completion report generated

### Day 4 (Sprint 3.3 Start)
- [ ] Agent #5 spawns successfully
- [ ] RenderFacade design complete
- [ ] 7 methods implemented
- [ ] 10/20 tests written

### Day 5 (Sprint 3.3 Completion)
- [ ] RenderFacade 100% complete
- [ ] 20+ unit tests pass
- [ ] render/handlers.rs refactored to <50 LOC
- [ ] render/processors.rs logic migrated
- [ ] Sprint 3.3 completion report generated

### Day 6 (Sprint 3.4 Audit)
- [ ] All 8 route files audited
- [ ] Audit report generated with findings
- [ ] High-risk files refactored (profiles.rs, pdf.rs, stealth.rs)
- [ ] All routes <30 LOC

### Day 7 (Final Quality Gates)
- [ ] Full workspace compilation successful
- [ ] All clippy warnings resolved
- [ ] All tests pass (riptide-facade + riptide-api)
- [ ] Handler LOC verification complete
- [ ] Phase 3 completion report generated

---

## 🔧 Quality Gates (Run Daily)

```bash
# Check disk space (CRITICAL - need 15GB+ free)
df -h / | head -2

# Compile with zero warnings
RUSTFLAGS="-D warnings" cargo build --workspace

# Run clippy
cargo clippy --all -- -D warnings

# Run tests
cargo test -p riptide-facade
cargo test -p riptide-api

# Verify handler LOC
for file in crates/riptide-api/src/handlers/*.rs; do
    wc -l "$file"
done | sort -rn | head -20
```

---

## 📊 Success Metrics

### Per-Sprint Targets

| Sprint | Facades | Handler Reduction | Test Coverage | Duration | Status |
|--------|---------|-------------------|---------------|----------|--------|
| 3.2 | 7 | 2,600 → <350 LOC | 112+ tests | 3 days | Planned |
| 3.3 | 1 | 696 → <50 LOC | 20+ tests | 2 days | Planned |
| 3.4 | 0 | Routes <30 LOC | 10+ tests | 2 days | Planned |

### Phase 3 Overall Targets

| Metric | Baseline | Target | Expected | Status |
|--------|----------|--------|----------|--------|
| **Total Handlers Migrated** | 19 | 19 | 17 (3.2+3.3) | On track |
| **Facades Created** | 8 | 16 | 15 (Sprint 3.1+3.2+3.3) | On track |
| **Handler LOC** | 8,803 | <950 | ~1,000 | On track |
| **Facade LOC** | 2,844 | ~12,000 | ~12,071 | On track |
| **Unit Tests** | 70 | 200+ | 212+ | On track |
| **Handler Avg LOC** | 463 | <50 | ~53 | Close |

---

## 🧠 Memory Coordination

### Memory Keys (All Agents Use These)

```bash
# Store facade interface
npx claude-flow@alpha hooks post-edit --file "[facade].rs" --memory-key "swarm/agent[N]/[facade]/interface"

# Store status update
npx claude-flow@alpha hooks notify --message "[Agent N]: [status update]"

# Restore session context
npx claude-flow@alpha hooks session-restore --session-id "sprint-3.[2|3|4]"

# Complete task
npx claude-flow@alpha hooks post-task --task-id "agent[N]-facade"
```

### Status Values
- `started` → `design_complete` → `implementation_complete` → `tests_complete` → `refactoring_complete` → `quality_gates_passed` → `done`

---

## 🎯 Agent Dependencies

### Sprint 3.2 Dependencies

| Agent | Depends On | Reason |
|-------|------------|--------|
| Agent #1 | ProfilingFacade (Sprint 3.1) | Memory metrics integration |
| Agent #2 | ProfilingFacade (Sprint 3.1) | Memory profiling integration |
| Agent #3 | ScraperFacade (Phase 2), CacheFacade (Phase 2) | URL crawling and caching |
| Agent #4 | DeepSearchFacade (Agent #3) | stream_deep_search() method |

**Coordination:** Agent #4 waits for Agent #3 to complete DeepSearchFacade

### Sprint 3.3 Dependencies

| Agent | Depends On | Reason |
|-------|------------|--------|
| Agent #5 | ScraperFacade (Phase 2), PdfFacade (Sprint 3.1) | URL fetching and PDF processing |

### Sprint 3.4 Dependencies

| Agent | Depends On | Reason |
|-------|------------|--------|
| Agent #6 | All facades (Sprint 3.1-3.3) | Route audit references all facades |

---

## 📁 File Structure

### Facades Created (Sprint 3.2-3.3)

```
crates/riptide-facade/src/facades/
├── chunking.rs (450 LOC) - Agent #1
├── memory.rs (400 LOC) - Agent #1
├── monitoring.rs (600 LOC) - Agent #2
├── pipeline_phases.rs (350 LOC) - Agent #2
├── strategies.rs (550 LOC) - Agent #3
├── deepsearch.rs (500 LOC) - Agent #3
├── streaming.rs (550 LOC) - Agent #4
└── render.rs (900 LOC) - Agent #5
```

### Handlers Refactored (Sprint 3.2-3.3)

```
crates/riptide-api/src/handlers/
├── chunking.rs (356 → <50 LOC)
├── memory.rs (313 → <50 LOC)
├── monitoring.rs (344 → <50 LOC)
├── pipeline_phases.rs (289 → <50 LOC)
├── strategies.rs (336 → <50 LOC)
├── deepsearch.rs (310 → <50 LOC)
├── streaming.rs (300 → <50 LOC)
└── render/
    ├── handlers.rs (362 → <50 LOC)
    └── processors.rs (334 → 0 LOC, merged into RenderFacade)
```

### Routes Audited (Sprint 3.4)

```
crates/riptide-api/src/routes/
├── profiles.rs (124 LOC) - HIGH RISK
├── pdf.rs (58 LOC) - MEDIUM RISK
├── stealth.rs (52 LOC) - MEDIUM RISK
├── llm.rs (34 LOC) - LOW RISK
├── tables.rs (28 LOC) - LOW RISK
├── engine.rs (23 LOC) - LOW RISK
├── chunking.rs (21 LOC) - LOW RISK
└── mod.rs (7 LOC) - LOW RISK
```

---

## ⚠️ Common Pitfalls & Solutions

### Pitfall #1: Missing Port Interfaces
**Solution:** Create mock ports temporarily, implement in Phase 4

### Pitfall #2: Handler LOC >50 After Refactoring
**Solution:** Extract DTO converters and helper utilities to separate files

### Pitfall #3: Compilation Errors After Parallel Development
**Solution:** Day 3 dedicated to integration testing and error fixes

### Pitfall #4: Test Failures Blocking Progress
**Solution:** TDD approach - write tests before implementation

### Pitfall #5: Agent Coordination Conflicts
**Solution:** Memory-based coordination via claude-flow hooks

---

## 📞 Communication Protocol

### Daily Standups (Async)
- **When:** 09:00 each day
- **How:** Store update in memory
- **Format:**
  ```bash
  npx claude-flow@alpha hooks notify --message "[Agent N]: Yesterday: [X]. Today: [Y]. Blockers: [Z]."
  ```

### Mid-Sprint Review (Async)
- **When:** Day 2 at 17:00
- **How:** Generate progress report
- **Content:** Facades completed, tests passing, issues encountered

### Sprint Completion (Report)
- **When:** End of Day 3, 5, 7
- **How:** Generate comprehensive completion report
- **Content:** All deliverables, quality gates, LOC impact, lessons learned

---

## 🎓 Lessons from Sprint 3.1

### What Worked ✅
1. Multi-agent swarm execution (5x faster)
2. Hexagonal architecture enforcement via ports
3. Port-based design prevented infrastructure coupling
4. In-memory mocks for fast unit testing

### What to Improve ⚠️
1. Better integration testing before parallel merge
2. Complete mock implementations upfront
3. Type mismatch prevention via stricter interfaces
4. Extract DTO converters earlier for handler LOC target

### Applied to Sprint 3.2-3.4 ✅
1. Day 3 dedicated to integration testing
2. Mock port templates provided in agent instructions
3. Stricter type annotations in facade specifications
4. Helper utility extraction as standard practice

---

## 📚 Reference Documents

1. **PHASE_3_SPRINTS_3.2-3.4_PLAN.md** - Detailed execution plan (48 pages)
2. **PHASE_3_AGENT_ASSIGNMENTS.md** - Agent responsibilities and instructions (35 pages)
3. **PHASE_3_QUICK_REFERENCE.md** - This document (quick lookup)
4. **PHASE_3_SPRINT_3.1_COMPLETE.md** - Sprint 3.1 completion report (lessons learned)

---

## ✅ Final Checklist (Phase 3 Complete)

- [ ] All 15 facades created (8 from Sprint 3.1 + 7 from Sprint 3.2 + 1 from Sprint 3.3)
- [ ] 212+ unit tests written and passing
- [ ] All handlers <50 LOC (target: 100% compliance)
- [ ] All route files <30 LOC (zero business logic)
- [ ] Zero clippy warnings
- [ ] Full workspace compiles successfully
- [ ] Hexagonal architecture maintained (no HTTP in facades)
- [ ] Phase 3 completion report generated

---

🚀 **Ready to execute! Start with Sprint 3.2 by spawning 4 agents in parallel.**

---

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
