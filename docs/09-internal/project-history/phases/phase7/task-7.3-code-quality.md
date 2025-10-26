# Task 7.3: Code Quality & Cleanup

**Agent Type:** reviewer
**Duration:** 1.2 days
**Dependencies:** Phase 6 complete ✅
**Status:** Ready to execute

## Objectives

1. **Clippy Warning Reduction**
   - Current: 12 warnings
   - Target: <20 warnings (maintain current level)
   - Fix unused imports, variables
   - Fix dead_code warnings where appropriate
   - Document intentional warnings

2. **Dead Code Removal** (~500 lines target)
   - Remove unused API methods
   - Remove unused cache utilities
   - Remove deprecated functions
   - Clean up commented code
   - Remove debug code

3. **CLI Metrics Wiring**
   - Wire CLI metrics to benchmark command
   - Wire CLI metrics to status command
   - Add metrics reporting to output
   - Test metrics collection

4. **Warning Cleanup** (114 warnings total)
   - Unused imports: ~40 warnings
   - Unused variables: ~30 warnings
   - Dead code: ~25 warnings
   - Other: ~19 warnings

## Approach

**Phase 1: Analysis** (2 hours)
```bash
cargo clippy --workspace --all-targets --all-features -- -W clippy::all > clippy-full.txt
grep "warning:" clippy-full.txt | sort | uniq -c | sort -rn
```

**Phase 2: Quick Wins** (4 hours)
- Remove unused imports (automated)
- Remove unused variables (automated)
- Remove obvious dead code

**Phase 3: Manual Review** (2 hours)
- Review complex warnings
- Document intentional code (test infrastructure)
- Fix remaining issues

**Phase 4: Validation** (1 hour)
- Run full test suite
- Verify no functionality lost
- Measure LOC reduction

## Coordination Requirements

**BEFORE starting:**
```bash
npx claude-flow@alpha hooks pre-task --description "Task 7.3: Code Quality & Cleanup"
npx claude-flow@alpha hooks session-restore --session-id "swarm-phase7"
```

**DURING work:**
```bash
# After each cleanup pass
npx claude-flow@alpha hooks post-edit --file "cleanup-[phase].txt" --memory-key "phase7/code_quality/[phase]"
npx claude-flow@alpha hooks notify --message "Quality: [what was cleaned]"
```

**AFTER completion:**
```bash
npx claude-flow@alpha hooks post-task --task-id "task-7.3-code-quality"
npx claude-flow@alpha hooks session-end --export-metrics true
```

## Deliverables

1. ✅ Clippy warnings: <20 (currently 12, maintain)
2. ✅ Dead code removed: ~500 LOC
3. ✅ CLI metrics wired to commands
4. ✅ Warning cleanup: 114 → <20
5. ✅ Code quality report in /docs/CODE-QUALITY-REPORT.md
6. ✅ All tests passing (626/630 maintained)
7. ✅ Performance validation (no regression)

## Success Criteria

- ✅ <20 clippy warnings workspace-wide
- ✅ ~500 lines of dead code removed
- ✅ CLI metrics functional in benchmark/status commands
- ✅ 100% test pass rate maintained
- ✅ No performance regression
- ✅ Build time unchanged or improved

## Files to Review

Priority files with unused code:
- `/workspaces/eventmesh/crates/riptide-api/src/**/*.rs`
- `/workspaces/eventmesh/crates/riptide-cache/src/**/*.rs`
- `/workspaces/eventmesh/crates/riptide-cli/src/**/*.rs`
- `/workspaces/eventmesh/crates/riptide-intelligence/src/**/*.rs`

## Memory Storage

Store progress at:
- `phase7/code_quality/status` - Current status
- `phase7/code_quality/warnings_before` - Initial count
- `phase7/code_quality/warnings_after` - Final count
- `phase7/code_quality/loc_removed` - Lines removed
- `phase7/code_quality/blockers` - Any issues encountered

## CRITICAL: Must Complete Before Task 7.4

Task 7.4 (Release Preparation) depends on this task being COMPLETE.
Do not proceed to release prep until code quality passes all criteria.
