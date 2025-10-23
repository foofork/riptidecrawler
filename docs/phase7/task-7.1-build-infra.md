# Task 7.1: Build Infrastructure Optimization

**Agent Type:** cicd-engineer
**Duration:** 2.4 days
**Dependencies:** Phase 6 complete ✅
**Status:** Ready to execute

## Objectives

1. **sccache Integration** (10GB cap)
   - Install and configure sccache for the 24-crate workspace
   - Set RUSTC_WRAPPER environment variable
   - Configure 10GB cache size limit
   - Measure build time improvements

2. **Shared target-dir**
   - Configure workspace to use shared target directory
   - Update .cargo/config.toml with shared-target setting
   - Avoid redundant builds across crates
   - Measure disk space savings

3. **Cargo Sweep Integration**
   - Add cargo-sweep to CI workflow
   - Add sweep commands to Codespaces cleanup
   - Configure automatic cleanup of old build artifacts
   - Set disk usage thresholds

4. **Metrics Collection**
   - Baseline: Full clean build time
   - Baseline: Incremental build time
   - After sccache: Full build time (expect ~30-40% improvement)
   - After sccache: Incremental build time
   - Disk usage before/after sweep

## Coordination Requirements

**BEFORE starting:**
```bash
npx claude-flow@alpha hooks pre-task --description "Task 7.1: Build Infrastructure Optimization"
npx claude-flow@alpha hooks session-restore --session-id "swarm-phase7"
```

**DURING work:**
```bash
# After each configuration change
npx claude-flow@alpha hooks post-edit --file "[file]" --memory-key "phase7/build_infra/[step]"
npx claude-flow@alpha hooks notify --message "Build Infra: [what was done]"
```

**AFTER completion:**
```bash
npx claude-flow@alpha hooks post-task --task-id "task-7.1-build-infra"
npx claude-flow@alpha hooks session-end --export-metrics true
```

## Deliverables

1. ✅ sccache configured and operational
2. ✅ Shared target-dir configured
3. ✅ Cargo sweep integrated in CI
4. ✅ Build time metrics report (before/after)
5. ✅ Documentation in /docs/BUILD-OPTIMIZATION.md
6. ✅ Updated .cargo/config.toml
7. ✅ Updated .github/workflows/ci.yml

## Success Criteria

- ✅ Clean build time reduced by 25-40%
- ✅ Incremental build time reduced by 40-60%
- ✅ Disk usage controlled via sweep
- ✅ All workspace crates build successfully
- ✅ No regression in test pass rate
- ✅ CI pipeline integration complete

## Memory Storage

Store progress at:
- `phase7/build_infra/status` - Current status
- `phase7/build_infra/metrics` - Build time measurements
- `phase7/build_infra/blockers` - Any issues encountered
