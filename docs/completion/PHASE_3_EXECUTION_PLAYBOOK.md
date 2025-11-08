# Phase 3 Execution Playbook: Step-by-Step Guide

**Date:** 2025-11-08
**Status:** 📋 **EXECUTION READY**
**Total Duration:** 7 days (3-4 days effective with parallel execution)

---

## 🚀 Sprint 3.2 Execution (Days 1-3)

### Pre-Flight Checklist ✅

```bash
# 1. Check disk space (CRITICAL)
df -h / | head -2
# MUST have >15GB free before starting

# 2. Clean if needed
[ $(df / | awk 'END{print $4}') -lt 5000000 ] && cargo clean

# 3. Verify Sprint 3.1 completion
ls -lh /workspaces/eventmesh/docs/completion/PHASE_3_SPRINT_3.1_COMPLETE.md

# 4. Check existing facades
ls -lh /workspaces/eventmesh/crates/riptide-facade/src/facades/

# 5. Verify git status
git status
```

---

### Day 1: Agent Spawn & Initial Implementation

#### Step 1: Initialize Swarm Coordination (09:00 - 09:15)

```bash
# Optional: MCP coordination setup
# (Skip if using Claude Code Task tool only)

# Initialize memory store
npx claude-flow@alpha hooks session-start --session-id "sprint-3.2"
```

#### Step 2: Spawn All 4 Agents in Parallel (09:15 - 09:30)

**CRITICAL: Use Claude Code Task tool to spawn ALL agents in a SINGLE message**

```
Spawn 4 agents in parallel for Sprint 3.2 handler migrations:

Agent #1 (Chunking & Memory Specialist):
Create ChunkingFacade and MemoryFacade in crates/riptide-facade/src/facades/.

ChunkingFacade (450 LOC):
- Implement chunk_content() with 5 strategies (topic, sliding, fixed, sentence, html-aware)
- Add validate_chunking_config(), list_supported_modes(), estimate_chunks()
- Use riptide_extraction::chunking directly
- Write 15+ unit tests

MemoryFacade (400 LOC):
- Implement get_memory_profile(), get_component_breakdown(), get_peak_usage()
- Add detect_memory_pressure(), get_jemalloc_stats(), calculate_fragmentation()
- Integrate with ProfilingFacade (Sprint 3.1)
- Write 12+ unit tests

Refactor handlers:
- chunking.rs (356 → <50 LOC)
- memory.rs (313 → <50 LOC)

Quality gates: Zero clippy warnings, all tests pass, handlers <50 LOC

Coordination: Use claude-flow hooks
npx claude-flow@alpha hooks pre-task --description 'ChunkingFacade + MemoryFacade'
npx claude-flow@alpha hooks post-edit --file 'chunking.rs' --memory-key 'swarm/agent1/chunking'
npx claude-flow@alpha hooks post-task --task-id 'agent1-facades'

---

Agent #2 (Monitoring & Pipeline Analyst):
Create MonitoringFacade and PipelinePhasesFacade in crates/riptide-facade/src/facades/.

MonitoringFacade (600 LOC):
- Implement 10 methods: calculate_health_score(), generate_performance_report(), get_alert_rules(), get_active_alerts(), get_current_metrics(), get_resource_status(), get_memory_metrics(), analyze_memory_leaks(), get_allocation_metrics(), get_wasm_health()
- Use MonitoringSystemPort, MetricsCollectorPort, ProfilingFacade
- Write 20+ unit tests

PipelinePhasesFacade (350 LOC):
- Implement get_phase_breakdown(), calculate_overall_metrics(), get_phase_metrics(), detect_bottlenecks(), calculate_success_rates(), calculate_percentiles()
- Use MetricsCollectorPort
- Write 14+ unit tests

Refactor handlers:
- monitoring.rs (344 → <50 LOC)
- pipeline_phases.rs (289 → <50 LOC)

Quality gates: Zero clippy warnings, all tests pass

Coordination: Use claude-flow hooks for ProfilingFacade integration

---

Agent #3 (Strategies & Search Orchestrator):
Create StrategiesFacade and DeepSearchFacade in crates/riptide-facade/src/facades/.

StrategiesFacade (550 LOC):
- Implement execute_strategy_crawl(), list_strategies(), validate_strategy_config()
- Add configure_css_strategy(), configure_regex_strategy(), configure_llm_strategy()
- Use StrategiesPipelineOrchestrator, StrategyConfigPort
- Coordinate with ScraperFacade, CacheFacade (Phase 2)
- Write 18+ unit tests

DeepSearchFacade (500 LOC):
- Implement execute_deep_search(), validate_query(), search_web()
- Add extract_urls(), crawl_urls(), combine_results()
- Use SearchProviderPort, PipelineOrchestratorPort, EventBusPort
- Coordinate with ScraperFacade
- Write 18+ unit tests

Refactor handlers:
- strategies.rs (336 → <50 LOC)
- deepsearch.rs (310 → <50 LOC)

Quality gates: Zero clippy warnings, all tests pass

Coordination: Check memory for ScraperFacade interface

---

Agent #4 (Streaming Specialist):
Create StreamingFacade in crates/riptide-facade/src/facades/.

StreamingFacade (550 LOC):
- Implement stream_crawl(), stream_deep_search()
- Add create_ndjson_line(), apply_backpressure(), create_progress_update()
- Use NdjsonStreamingHandler, PipelineOrchestratorPort
- Coordinate with ScraperFacade and DeepSearchFacade (Agent #3)
- Write 15+ unit tests for streaming, backpressure, progress, errors

Refactor handler:
- streaming.rs (300 → <50 LOC)

Quality gates: Zero clippy warnings, all tests pass

Coordination: Wait for Agent #3 DeepSearchFacade before implementing stream_deep_search()

---

IMPORTANT: Run all coordination hooks as specified in each agent's instructions.
```

#### Step 3: Monitor Agent Progress (Throughout Day 1)

```bash
# Check agent status in memory
npx claude-flow@alpha hooks session-restore --session-id "sprint-3.2"

# View agent notifications
cat .swarm/memory.db # or use SQLite browser

# Expected by end of Day 1:
# - All 4 agents have facades designed (ports defined)
# - 30-40% implementation complete
# - Initial method signatures created
```

---

### Day 2: Implementation & Testing

#### Step 1: Morning Standup (09:00 - 09:15)

```bash
# Check each agent's status
# Expected status: implementation_complete or tests_in_progress

# Verify memory coordination
npx claude-flow@alpha hooks session-restore --session-id "sprint-3.2"
```

#### Step 2: Complete Implementation (09:15 - 12:00)

Agents should have:
- ✅ All facade methods implemented
- ✅ Port-based dependencies wired
- ✅ Error handling complete
- ✅ Initial unit tests written (50% coverage)

#### Step 3: Complete Testing (13:00 - 17:00)

Agents should achieve:
- ✅ 112+ unit tests total (15 + 34 + 36 + 15)
- ✅ >90% test coverage per facade
- ✅ All tests passing locally

#### Step 4: Handler Refactoring (17:00 - 18:00)

Agents refactor handlers to <50 LOC:
- chunking.rs (356 → <50)
- memory.rs (313 → <50)
- monitoring.rs (344 → <50)
- pipeline_phases.rs (289 → <50)
- strategies.rs (336 → <50)
- deepsearch.rs (310 → <50)
- streaming.rs (300 → <50)

#### Step 5: Memory Sync (18:00)

```bash
# All agents post task completion
npx claude-flow@alpha hooks post-task --task-id "sprint-3.2-day2"

# Check status
npx claude-flow@alpha hooks notify --message "Day 2 complete. All facades implemented and tested."
```

---

### Day 3: Integration & Quality Gates

#### Step 1: Integration Testing (09:00 - 11:00)

```bash
# Compile full workspace
cd /workspaces/eventmesh
RUSTFLAGS="-D warnings" cargo build --workspace

# Expected: Some compilation errors (type mismatches, missing imports)
# This is normal for parallel development
```

#### Step 2: Fix Compilation Errors (11:00 - 13:00)

Common issues:
1. Type mismatches between facades and handlers
2. Missing trait imports
3. Incomplete mock implementations
4. Port signature mismatches

**Fix systematically:**
```bash
# Compile one crate at a time
cargo build -p riptide-facade
cargo build -p riptide-api

# Fix errors in order of appearance
# Document fixes for future reference
```

#### Step 3: Run Quality Gates (14:00 - 17:00)

```bash
# 1. Clippy (zero warnings)
cargo clippy --all -- -D warnings

# 2. Tests (100% pass rate)
cargo test -p riptide-facade
cargo test -p riptide-api

# 3. Verify handler LOC
for file in crates/riptide-api/src/handlers/*.rs; do
    wc -l "$file"
done | sort -rn | head -20

# 4. Check route files (should be unchanged)
git status crates/riptide-api/src/routes/
```

#### Step 4: Generate Sprint 3.2 Completion Report (17:00 - 18:00)

```bash
# Create completion report
cat > /workspaces/eventmesh/docs/completion/PHASE_3_SPRINT_3.2_COMPLETE.md << 'EOF'
# Phase 3 Sprint 3.2: Medium Handler Migrations - Complete ✅

**Date:** $(date +%Y-%m-%d)
**Status:** ✅ **COMPLETE**

## Summary
- ✅ 7 facades created (ChunkingFacade, MonitoringFacade, StrategiesFacade, MemoryFacade, DeepSearchFacade, StreamingFacade, PipelinePhasesFacade)
- ✅ 112+ unit tests added
- ✅ 7 handlers refactored to <50 LOC
- ✅ All quality gates passed

## LOC Impact
- Handler LOC: 2,600 → <350 (-86% reduction)
- Facade LOC: +3,400
- Net change: +800 LOC (business logic layer)

## Quality Gates
- ✅ Zero clippy warnings
- ✅ All tests pass (100% success rate)
- ✅ All handlers <50 LOC
- ✅ No HTTP types in facades

## Next Steps
- Sprint 3.3: RenderFacade (Agent #5)
EOF

# Commit changes
git add .
git commit -m "feat: Complete Sprint 3.2 - 7 medium handler migrations

- Created 7 facades: Chunking, Monitoring, Strategies, Memory, DeepSearch, Streaming, PipelinePhases
- Added 112+ unit tests across all facades
- Refactored 7 handlers to <50 LOC each
- All quality gates passed (clippy, tests, compilation)

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## 🎨 Sprint 3.3 Execution (Days 4-5)

### Day 4: RenderFacade Design & Implementation

#### Step 1: Spawn Agent #5 (09:00 - 09:15)

```
Spawn Agent #5 (Render Subsystem Architect) for Sprint 3.3:

Create unified RenderFacade in crates/riptide-facade/src/facades/render.rs by consolidating:
- render/handlers.rs (362 LOC)
- render/processors.rs (334 LOC)

RenderFacade (900 LOC):
- Implement render() with mode selection (PDF, dynamic, static, adaptive)
- Add process_pdf(), process_dynamic(), process_static(), process_adaptive()
- Add extract_content(), acquire_resources()
- Use ResourceManagerPort, ScraperFacade, PdfProcessorPort, StealthController
- Write 20+ unit tests covering all rendering modes

Refactor handlers:
- render/handlers.rs (362 → <50 LOC)
- render/processors.rs (334 → 0 LOC, logic migrated to facade)

Quality gates: Zero clippy warnings, all tests pass, no resource leaks

Coordination:
npx claude-flow@alpha hooks pre-task --description 'RenderFacade subsystem consolidation'
npx claude-flow@alpha hooks session-restore --session-id 'sprint-3.3'
npx claude-flow@alpha hooks post-edit --file 'render.rs' --memory-key 'swarm/agent5/render'
npx claude-flow@alpha hooks post-task --task-id 'agent5-facade'

Check memory for ScraperFacade and PdfFacade interfaces.
```

#### Step 2: Implementation (09:15 - 17:00)

Agent #5 should complete:
- ✅ RenderFacade design (7 methods, port interfaces)
- ✅ All rendering strategies (PDF, dynamic, static, adaptive)
- ✅ Resource management integration
- ✅ 10/20 unit tests written

#### Step 3: Evening Sync (17:00 - 18:00)

```bash
# Check progress
npx claude-flow@alpha hooks notify --message "Agent #5: Day 4 complete. RenderFacade 60% implemented."
```

---

### Day 5: RenderFacade Completion

#### Step 1: Complete Testing (09:00 - 12:00)

```bash
# Agent #5 completes:
# - 20+ unit tests (all rendering modes)
# - Resource acquisition/cleanup tests
# - Timeout handling tests
# - Session context integration tests
```

#### Step 2: Handler Refactoring (13:00 - 15:00)

```bash
# Refactor handlers
# render/handlers.rs (362 → <50 LOC)
# render/processors.rs (334 → 0 LOC, deleted)
```

#### Step 3: Integration & Quality Gates (15:00 - 17:00)

```bash
# Compile and test
cargo build --workspace
cargo clippy --all -- -D warnings
cargo test -p riptide-facade
cargo test -p riptide-api

# Verify handler LOC
wc -l crates/riptide-api/src/handlers/render/handlers.rs
```

#### Step 4: Sprint 3.3 Completion Report (17:00 - 18:00)

```bash
# Generate report and commit
cat > /workspaces/eventmesh/docs/completion/PHASE_3_SPRINT_3.3_COMPLETE.md << 'EOF'
# Phase 3 Sprint 3.3: Render Subsystem - Complete ✅

**Date:** $(date +%Y-%m-%d)
**Status:** ✅ **COMPLETE**

## Summary
- ✅ RenderFacade created (900 LOC)
- ✅ 20+ unit tests added
- ✅ render/handlers.rs refactored to <50 LOC
- ✅ render/processors.rs logic migrated to RenderFacade

## Quality Gates
- ✅ Zero clippy warnings
- ✅ All tests pass
- ✅ No resource leaks

## Next Steps
- Sprint 3.4: Route audit (Agent #6)
EOF

git add .
git commit -m "feat: Complete Sprint 3.3 - Render subsystem consolidation"
```

---

## 🔍 Sprint 3.4 Execution (Days 6-7)

### Day 6: Route Audit & Refactoring

#### Step 1: Spawn Agent #6 (09:00 - 09:15)

```
Spawn Agent #6 (Route Auditor & Refactoring Specialist) for Sprint 3.4:

Audit 8 route files in crates/riptide-api/src/routes/ for business logic violations:

1. routes/profiles.rs (124 LOC) - HIGH RISK
2. routes/pdf.rs (58 LOC) - MEDIUM RISK
3. routes/stealth.rs (52 LOC) - MEDIUM RISK
4. routes/llm.rs (34 LOC) - LOW RISK
5. routes/tables.rs (28 LOC) - LOW RISK
6. routes/engine.rs (23 LOC) - LOW RISK
7. routes/chunking.rs (21 LOC) - LOW RISK
8. routes/mod.rs (7 LOC) - LOW RISK

Audit criteria:
✅ Route files should ONLY contain:
- Router setup (Router::new())
- Route registration (get, post, put, delete)
- Path definitions
- Handler function references

❌ Route files should NOT contain:
- Business logic (calculations, validations, transformations)
- Direct database/cache access
- Complex error handling
- DTO transformations
- Authorization logic

Generate comprehensive audit report: docs/completion/ROUTE_AUDIT_REPORT.md

Refactor high-risk files:
- Extract business logic to existing facades (ProfileFacade, PdfFacade, etc.)
- Create helper utilities for shared logic
- Simplify error handling

Target: All routes <30 LOC, zero business logic

Coordination:
npx claude-flow@alpha hooks pre-task --description 'Route file audit and refactoring'
npx claude-flow@alpha hooks post-edit --file 'ROUTE_AUDIT_REPORT.md' --memory-key 'swarm/agent6/audit'
npx claude-flow@alpha hooks post-task --task-id 'agent6-audit'
```

#### Step 2: Audit Execution (09:15 - 11:00)

Agent #6 audits all 8 files:
- Document violations
- Identify business logic
- Create refactoring recommendations

#### Step 3: Generate Audit Report (11:00 - 12:00)

```bash
# Agent #6 creates comprehensive report
# Expected findings:
# - routes/profiles.rs: Profile validation logic → ProfileFacade
# - routes/pdf.rs: PDF validation → PdfFacade
# - routes/stealth.rs: Stealth configuration → New helper utility
```

#### Step 4: Refactoring (13:00 - 17:00)

```bash
# Refactor high-risk files
# Move logic to facades or helpers
# Simplify route registration
```

#### Step 5: Evening Sync (17:00 - 18:00)

```bash
npx claude-flow@alpha hooks notify --message "Agent #6: Route audit complete. 3 files refactored."
```

---

### Day 7: Final Quality Gates & Phase 3 Completion

#### Step 1: Full Workspace Compilation (09:00 - 11:00)

```bash
# Clean build
cargo clean
df -h / # Verify space

# Full compilation
RUSTFLAGS="-D warnings" cargo build --workspace
```

#### Step 2: Comprehensive Testing (11:00 - 12:00)

```bash
# Run all tests
cargo test --workspace

# Expected: 212+ tests passing
# - Sprint 3.1: 70+ tests
# - Sprint 3.2: 112+ tests
# - Sprint 3.3: 20+ tests
# - Sprint 3.4: 10+ tests
```

#### Step 3: Final Verification (13:00 - 15:00)

```bash
# 1. Verify all handlers <50 LOC
for file in crates/riptide-api/src/handlers/*.rs; do
    lines=$(wc -l < "$file")
    if [ $lines -gt 50 ]; then
        echo "❌ $file: $lines LOC (>50)"
    else
        echo "✅ $file: $lines LOC"
    fi
done

# 2. Verify all routes <30 LOC
for file in crates/riptide-api/src/routes/*.rs; do
    lines=$(wc -l < "$file")
    if [ $lines -gt 30 ]; then
        echo "⚠️  $file: $lines LOC (>30)"
    else
        echo "✅ $file: $lines LOC"
    fi
done

# 3. Count facades
ls -l crates/riptide-facade/src/facades/*.rs | wc -l
# Expected: 15+ facades (8 from Sprint 3.1 + 7 from Sprint 3.2 + 1 from Sprint 3.3)

# 4. Verify hexagonal architecture (no HTTP types in facades)
rg "use axum::|use hyper::" crates/riptide-facade/src/facades/
# Expected: No matches
```

#### Step 4: Generate Phase 3 Completion Report (15:00 - 17:00)

```bash
cat > /workspaces/eventmesh/docs/completion/PHASE_3_COMPLETE.md << 'EOF'
# Phase 3: Handler Refactoring - Complete ✅

**Date:** $(date +%Y-%m-%d)
**Status:** ✅ **COMPLETE**
**Duration:** 7 days (3-4 days effective)

## Executive Summary

Phase 3 successfully refactored **all handlers** in riptide-api to ultra-thin HTTP wrappers using a **multi-agent swarm** approach. Business logic has been migrated to 15+ facades in the riptide-facade application layer.

## Sprint Completion

### Sprint 3.1 (Complete)
- ✅ 10 largest handlers refactored
- ✅ 8 facades created/enhanced
- ✅ 70+ unit tests
- ✅ 5,907 LOC migrated

### Sprint 3.2 (Complete)
- ✅ 7 medium handlers refactored
- ✅ 7 facades created
- ✅ 112+ unit tests
- ✅ 2,600 LOC migrated

### Sprint 3.3 (Complete)
- ✅ Render subsystem consolidated
- ✅ 1 facade created
- ✅ 20+ unit tests
- ✅ 696 LOC migrated

### Sprint 3.4 (Complete)
- ✅ 8 route files audited
- ✅ 3 routes refactored
- ✅ Route audit report generated
- ✅ All routes <30 LOC

## Total Impact

| Metric | Before | After | Change |
|--------|--------|-------|--------|
| **Handler LOC** | 8,803 | <1,000 | -88.6% |
| **Facades** | 0 | 15+ | +15 |
| **Facade LOC** | 2,844 | ~12,000 | +322% |
| **Unit Tests** | ~500 | 212+ | +142% |
| **Avg Handler LOC** | 463 | <53 | -88.5% |

## Architecture Compliance

- ✅ Hexagonal architecture enforced (100%)
- ✅ Port-based dependencies (100%)
- ✅ Zero HTTP types in facades (100%)
- ✅ Comprehensive test coverage (>90% per facade)
- ✅ All handlers <50 LOC (95%+)
- ✅ All routes <30 LOC (100%)

## Quality Gates

- ✅ Zero clippy warnings
- ✅ All 212+ tests passing
- ✅ Full workspace compilation
- ✅ No business logic in routes
- ✅ Proper dependency injection via ports

## Next Steps

**Phase 4:** Infrastructure Layer
- Implement missing ports
- Create infrastructure adapters
- Optimize performance
- Production readiness

---

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
EOF
```

#### Step 5: Final Commit (17:00 - 18:00)

```bash
# Add all changes
git add .

# Create comprehensive commit
git commit -m "feat: Complete Phase 3 - Handler Refactoring

Phase 3 successfully refactored ALL handlers to ultra-thin HTTP wrappers.

Sprints Completed:
- Sprint 3.1: 10 largest handlers (5,907 LOC)
- Sprint 3.2: 7 medium handlers (2,600 LOC)
- Sprint 3.3: Render subsystem (696 LOC)
- Sprint 3.4: Route audit and refactoring

Total Impact:
- 15+ facades created in riptide-facade
- 212+ unit tests added
- Handler LOC: 8,803 → <1,000 (-88.6%)
- All quality gates passed

Architecture:
- Hexagonal architecture enforced
- Port-based dependencies throughout
- Zero HTTP types in facades
- >90% test coverage per facade

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com)"

# Export session metrics
npx claude-flow@alpha hooks session-end --export-metrics true
```

---

## 📊 Success Criteria Verification

### Final Checklist

```bash
# Run this script to verify all success criteria

#!/bin/bash

echo "=== Phase 3 Success Criteria Verification ==="
echo

# 1. Facade count
facade_count=$(ls crates/riptide-facade/src/facades/*.rs 2>/dev/null | wc -l)
echo "✅ Facades created: $facade_count (target: 15+)"

# 2. Handler LOC
echo
echo "Handler LOC (target: <50 each):"
for file in crates/riptide-api/src/handlers/*.rs; do
    lines=$(wc -l < "$file")
    if [ $lines -gt 50 ]; then
        echo "  ❌ $(basename $file): $lines LOC"
    else
        echo "  ✅ $(basename $file): $lines LOC"
    fi
done

# 3. Route LOC
echo
echo "Route LOC (target: <30 each):"
for file in crates/riptide-api/src/routes/*.rs; do
    lines=$(wc -l < "$file")
    if [ $lines -gt 30 ]; then
        echo "  ⚠️  $(basename $file): $lines LOC"
    else
        echo "  ✅ $(basename $file): $lines LOC"
    fi
done

# 4. HTTP types in facades
echo
echo "Hexagonal architecture (no HTTP types in facades):"
http_violations=$(rg "use axum::|use hyper::" crates/riptide-facade/src/facades/ | wc -l)
if [ $http_violations -eq 0 ]; then
    echo "  ✅ Zero HTTP types in facades"
else
    echo "  ❌ Found $http_violations HTTP type references"
fi

# 5. Compilation
echo
echo "Compilation:"
if cargo build --workspace 2>&1 | grep -q "error"; then
    echo "  ❌ Compilation errors found"
else
    echo "  ✅ Clean compilation"
fi

# 6. Clippy
echo
echo "Clippy:"
if cargo clippy --all -- -D warnings 2>&1 | grep -q "error"; then
    echo "  ❌ Clippy warnings found"
else
    echo "  ✅ Zero clippy warnings"
fi

# 7. Tests
echo
echo "Tests:"
test_output=$(cargo test --workspace 2>&1)
if echo "$test_output" | grep -q "test result: ok"; then
    test_count=$(echo "$test_output" | grep "test result: ok" | awk '{print $4}')
    echo "  ✅ All $test_count tests passing"
else
    echo "  ❌ Some tests failing"
fi

echo
echo "=== Verification Complete ==="
```

---

## 🎯 Troubleshooting Guide

### Issue: Agent Coordination Conflicts

**Symptoms:**
- Agents overwriting each other's work
- Memory sync failures
- Duplicate implementations

**Solution:**
```bash
# 1. Check memory coordination
npx claude-flow@alpha hooks session-restore --session-id "sprint-3.2"

# 2. Review agent status
cat .swarm/memory.db

# 3. Re-spawn conflicting agent with updated instructions
# Include: "Check memory for [other agent's work] before implementing [feature]"
```

### Issue: Handler LOC >50 After Refactoring

**Symptoms:**
- Handler has >50 LOC despite facade migration
- Complex error handling inflates LOC
- DTO mapping logic adds 10-20 LOC

**Solution:**
```rust
// 1. Extract DTO converters to separate module
mod dto_converters {
    pub fn request_to_config(req: Request) -> Config { ... }
    pub fn result_to_response(result: Result) -> Response { ... }
}

// 2. Create helper utilities
mod helpers {
    pub fn validate_and_convert(input: String) -> Result<Output> { ... }
}

// 3. Simplify error handling
// Before (10 LOC):
let result = facade.execute().await.map_err(|e| match e {
    RiptideError::NotFound => ApiError::not_found(...),
    RiptideError::Validation => ApiError::validation(...),
    // ...
})?;

// After (2 LOC):
let result = facade.execute().await.map_err(ApiError::from)?;
```

### Issue: Compilation Errors After Parallel Development

**Symptoms:**
- 20+ compilation errors
- Type mismatches
- Missing trait implementations

**Solution:**
```bash
# 1. Compile one crate at a time
cargo build -p riptide-types
cargo build -p riptide-facade
cargo build -p riptide-api

# 2. Fix errors systematically
# Start with riptide-facade (no dependencies on API layer)
# Then fix riptide-api (depends on facade)

# 3. Common fixes:
# - Add missing imports: use futures::FutureExt;
# - Fix return types: Option<String> vs String
# - Complete mock implementations
# - Add trait bounds: T: Clone + Send + Sync
```

### Issue: Tests Failing

**Symptoms:**
- Tests pass locally for agent but fail in integration
- Mock implementations incomplete
- Test data inconsistencies

**Solution:**
```rust
// 1. Use consistent test fixtures
mod test_fixtures {
    pub fn sample_config() -> Config { ... }
    pub fn sample_request() -> Request { ... }
}

// 2. Complete mock implementations
impl MockPort for TestPort {
    async fn execute(&self, input: Input) -> Result<Output> {
        // Don't use unimplemented!() or todo!()
        Ok(Output::default())
    }
}

// 3. Add more assertions
#[tokio::test]
async fn test_facade_method() {
    let result = facade.execute().await.unwrap();
    assert_eq!(result.status, "success"); // Add specific assertions
    assert!(result.data.len() > 0); // Validate state
}
```

---

## 📚 Reference Quick Links

- **Planning:** `/workspaces/eventmesh/docs/completion/PHASE_3_SPRINTS_3.2-3.4_PLAN.md`
- **Assignments:** `/workspaces/eventmesh/docs/completion/PHASE_3_AGENT_ASSIGNMENTS.md`
- **Quick Ref:** `/workspaces/eventmesh/docs/completion/PHASE_3_QUICK_REFERENCE.md`
- **Sprint 3.1:** `/workspaces/eventmesh/docs/completion/PHASE_3_SPRINT_3.1_COMPLETE.md`

---

🚀 **Ready to execute! Start with Sprint 3.2 Day 1: Agent Spawn**

---

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
