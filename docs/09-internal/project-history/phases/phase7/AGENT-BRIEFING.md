# Phase 7 Agent Briefing - Parallel Execution

**Coordinator:** System Architecture Designer
**Session:** swarm-phase7
**Execution Mode:** 4 Parallel Agents via Claude Code Task Tool
**Start Time:** 2025-10-23T08:10:00Z

---

## 🚀 Execution Plan

According to CLAUDE.md guidelines:
- ✅ Use Claude Code's Task tool for parallel agent execution
- ✅ All 4 agents spawn in ONE message
- ✅ Each agent gets full instructions
- ✅ Coordination via hooks and memory

---

## Agent 1: Build Infrastructure Engineer (Task 7.1)

**Instructions:**
```
You are a CI/CD Engineer responsible for Task 7.1: Build Infrastructure Optimization.

READ YOUR TASK FILE FIRST:
/workspaces/eventmesh/docs/phase7/task-7.1-build-infra.md

COORDINATION HOOKS (REQUIRED):
1. BEFORE starting:
   npx claude-flow@alpha hooks pre-task --description "Task 7.1: Build Infrastructure"
   npx claude-flow@alpha hooks session-restore --session-id "swarm-phase7"

2. DURING work (after each major step):
   npx claude-flow@alpha hooks post-edit --file "[file]" --memory-key "phase7/build_infra/[step]"
   npx claude-flow@alpha hooks notify --message "Build Infra: [progress update]"

3. AFTER completion:
   npx claude-flow@alpha hooks post-task --task-id "task-7.1-build-infra"

YOUR OBJECTIVES:
1. Install and configure sccache (10GB cap) for 24-crate workspace
2. Configure shared target-dir in .cargo/config.toml
3. Integrate cargo-sweep in CI and Codespaces cleanup
4. Measure build time improvements (baseline vs optimized)
5. Create /workspaces/eventmesh/docs/BUILD-OPTIMIZATION.md

SUCCESS CRITERIA:
- Clean build time reduced 25-40%
- Incremental build time reduced 40-60%
- All workspace crates build successfully
- Documentation complete

MEMORY UPDATES:
Store progress at: phase7/build_infra/status
Store metrics at: phase7/build_infra/metrics

DELIVERABLES:
- Updated .cargo/config.toml
- Updated .github/workflows/ci.yml
- BUILD-OPTIMIZATION.md with before/after metrics
- Build time report

Start immediately after reading this briefing!
```

---

## Agent 2: Backend Configuration Developer (Task 7.2)

**Instructions:**
```
You are a Backend Developer responsible for Task 7.2: Configuration System Completion.

READ YOUR TASK FILE FIRST:
/workspaces/eventmesh/docs/phase7/task-7.2-config-system.md

COORDINATION HOOKS (REQUIRED):
1. BEFORE starting:
   npx claude-flow@alpha hooks pre-task --description "Task 7.2: Configuration System"
   npx claude-flow@alpha hooks session-restore --session-id "swarm-phase7"

2. DURING work (after each crate):
   npx claude-flow@alpha hooks post-edit --file "crates/[name]/src/config.rs" --memory-key "phase7/config_system/[crate]"
   npx claude-flow@alpha hooks notify --message "Config: Completed [crate] env vars"

3. AFTER completion:
   npx claude-flow@alpha hooks post-task --task-id "task-7.2-config-system"

YOUR OBJECTIVES:
1. Add 45 env vars to riptide-api with from_env() method
2. Add 36 env vars to riptide-persistence with from_env() method
3. Create from_env() for riptide-pool (12 env vars)
4. Update .env.example with ALL 93 variables
5. Create /workspaces/eventmesh/docs/CONFIGURATION.md

SUCCESS CRITERIA:
- 100% env variable support (93/93)
- All from_env() methods tested
- No hardcoded values in source
- Comprehensive .env.example
- All tests passing

MEMORY UPDATES:
Store progress at: phase7/config_system/status
Store per-crate at: phase7/config_system/[api|persistence|pool]_progress

DELIVERABLES:
- riptide-api/src/config.rs (45 vars)
- riptide-persistence/src/config.rs (36 vars)
- riptide-pool/src/config.rs (12 vars)
- .env.example (93 total variables)
- CONFIGURATION.md documentation

Start immediately after reading this briefing!
```

---

## Agent 3: Code Quality Reviewer (Task 7.3)

**Instructions:**
```
You are a Code Quality Reviewer responsible for Task 7.3: Code Quality & Cleanup.

READ YOUR TASK FILE FIRST:
/workspaces/eventmesh/docs/phase7/task-7.3-code-quality.md

COORDINATION HOOKS (REQUIRED):
1. BEFORE starting:
   npx claude-flow@alpha hooks pre-task --description "Task 7.3: Code Quality"
   npx claude-flow@alpha hooks session-restore --session-id "swarm-phase7"

2. DURING work (after each cleanup phase):
   npx claude-flow@alpha hooks post-edit --file "cleanup-phase-[N].txt" --memory-key "phase7/code_quality/phase[N]"
   npx claude-flow@alpha hooks notify --message "Quality: [what was cleaned]"

3. AFTER completion:
   npx claude-flow@alpha hooks post-task --task-id "task-7.3-code-quality"

YOUR OBJECTIVES:
1. Maintain <20 clippy warnings (currently 12, don't increase)
2. Remove ~500 LOC of dead code (unused methods, cache utilities)
3. Wire CLI metrics to benchmark and status commands
4. Clean up 114 warnings (unused imports, variables, dead_code)

APPROACH:
Phase 1: Run cargo clippy and analyze warnings (2 hours)
Phase 2: Remove obvious unused code (4 hours)
Phase 3: Manual review of complex warnings (2 hours)
Phase 4: Validation - all tests must pass (1 hour)

SUCCESS CRITERIA:
- Clippy warnings: <20 workspace-wide
- Dead code removed: ~500 LOC
- CLI metrics functional
- 626/630 tests passing (maintained)
- No performance regression

MEMORY UPDATES:
Store progress at: phase7/code_quality/status
Store warnings at: phase7/code_quality/warnings_before and warnings_after
Store LOC at: phase7/code_quality/loc_removed

CRITICAL: Task 7.4 (Release Prep) cannot start until you mark status as "complete"!

DELIVERABLES:
- CODE-QUALITY-REPORT.md with before/after analysis
- ~500 LOC removed from unused code
- CLI metrics wired to commands
- All tests passing

Start immediately after reading this briefing!
```

---

## Agent 4: Release Planning Coordinator (Task 7.4)

**Instructions:**
```
You are a Release Planner responsible for Task 7.4: Release Preparation.

READ YOUR TASK FILE FIRST:
/workspaces/eventmesh/docs/phase7/task-7.4-release-prep.md

⚠️ CRITICAL DEPENDENCY: DO NOT START until Task 7.3 is COMPLETE!

DEPENDENCY CHECK (REQUIRED):
1. BEFORE starting:
   npx claude-flow@alpha hooks session-restore --session-id "swarm-phase7"
   # CHECK: phase7/code_quality/status MUST be "complete"

2. If Task 7.3 NOT complete:
   - Wait and check every 5 minutes
   - Monitor phase7/code_quality/status in memory
   - DO NOT proceed until confirmed complete

3. Once Task 7.3 complete:
   npx claude-flow@alpha hooks pre-task --description "Task 7.4: Release Preparation"

COORDINATION HOOKS:
1. DURING work (after major updates):
   npx claude-flow@alpha hooks post-edit --file "[file]" --memory-key "phase7/release_prep/[step]"
   npx claude-flow@alpha hooks notify --message "Release: [progress]"

2. AFTER completion:
   npx claude-flow@alpha hooks post-task --task-id "task-7.4-release-prep"

YOUR OBJECTIVES:
1. Update CHANGELOG.md (Keep a Changelog format)
   - Add all Phase 5 changes (engine selection)
   - Add all Phase 6 changes (testing infrastructure)
   - Add all Phase 7 changes (build, config, quality)
   - Include breaking changes section

2. Version bump to 2.0.0
   - Update workspace Cargo.toml
   - Update all crate Cargo.toml files
   - Update README.md version references

3. Create release notes
   - Write /workspaces/eventmesh/docs/RELEASE-NOTES-v2.0.0.md
   - User-facing highlights
   - Breaking changes documentation
   - Migration guide references

4. Pre-release validation
   - Verify all tests passing
   - Verify clippy warnings <20
   - Verify documentation up to date

SUCCESS CRITERIA:
- CHANGELOG follows Keep a Changelog format
- All version numbers = 2.0.0
- Release notes comprehensive
- Breaking changes documented
- All validation passed

MEMORY UPDATES:
Store progress at: phase7/release_prep/status
Store completion at: phase7/release_prep/[changelog|version|notes]_done

DELIVERABLES:
- CHANGELOG.md updated
- Version 2.0.0 in all Cargo.toml
- RELEASE-NOTES-v2.0.0.md created
- Pre-release validation checklist complete

WAIT for Task 7.3 before starting! Check memory every 5 minutes.
```

---

## 📊 Coordination Protocol

### Progress Monitoring (Coordinator)
Every 5 minutes, check:
```bash
npx claude-flow@alpha hooks session-restore --session-id "swarm-phase7"
```

Monitor these memory keys:
- `phase7/build_infra/status`
- `phase7/config_system/status`
- `phase7/code_quality/status`
- `phase7/release_prep/status`

### Quality Gates
Each agent MUST:
1. ✅ Read their task file
2. ✅ Run coordination hooks
3. ✅ Update memory with progress
4. ✅ Create deliverables
5. ✅ Mark status complete

### Conflict Resolution
- Tasks 7.1-7.3: Run in parallel (no file conflicts)
- Task 7.4: Sequential after 7.3 (intentional dependency)

---

## 🎯 Final Phase 7 Deliverables

After all 4 agents complete:

1. **PHASE7-COMPLETION-REPORT.md**
   - Executive summary
   - All task results
   - Metrics comparison
   - Quality validation

2. **Updated COMPREHENSIVE-ROADMAP.md**
   - Mark Phase 7 complete
   - Update metrics
   - Update timeline

3. **Final Metrics JSON**
   - Build improvements
   - Configuration coverage
   - Code quality metrics
   - Version information

---

**Briefing Created:** 2025-10-23T08:10:00Z
**Ready for Agent Spawning:** ✅
**Execution Mode:** Parallel (Tasks 7.1-7.3), Sequential (Task 7.4 after 7.3)
