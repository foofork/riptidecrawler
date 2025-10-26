# Task 7.4: Release Preparation

**Agent Type:** planner
**Duration:** 1 day
**Dependencies:** Task 7.3 COMPLETE (code quality) ⚠️
**Status:** Waiting for Task 7.3

## ⚠️ CRITICAL DEPENDENCY

**DO NOT START until Task 7.3 (Code Quality) is COMPLETE.**

Check memory key `phase7/code_quality/status` for completion status.

## Objectives

1. **CHANGELOG Update**
   - Add all Phase 5 changes (engine selection)
   - Add all Phase 6 changes (testing infrastructure)
   - Add all Phase 7 changes (build infra, config, quality)
   - Format according to Keep a Changelog standard
   - Include breaking changes section

2. **Version Bumping to 2.0.0**
   - Update all Cargo.toml files (workspace + crates)
   - Update version in documentation
   - Update version in README
   - Update version in package.json (if applicable)
   - Verify semantic versioning rationale

3. **Release Notes**
   - Write user-facing release notes
   - Highlight major features
   - Document breaking changes
   - Include migration guide references
   - Add upgrade instructions

4. **Pre-Release Validation**
   - All tests passing (626/630)
   - Clippy warnings <20
   - Documentation up to date
   - No TODO comments in critical paths
   - Security audit clean

## CHANGELOG Structure

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.0] - 2025-10-23

### Added
- Phase 5: Centralized engine selection in riptide-reliability (583 LOC eliminated)
- Phase 6: CLI integration tests with assert_cmd/assert_fs (45+ tests)
- Phase 6: Unified coverage with cargo-llvm-cov (34 crates)
- Phase 6: Chaos testing framework (29+ resilience tests)
- Phase 7: Build infrastructure with sccache (25-40% faster builds)
- Phase 7: Complete environment variable support (93 variables)

### Changed
- Phase 5: Engine selection now uses shared reliability module
- Phase 7: Improved build performance with shared target-dir
- Phase 7: Enhanced configuration system with from_env() methods

### Fixed
- Phase 7: Reduced clippy warnings to <20
- Phase 7: Removed ~500 lines of dead code
- Phase 7: CLI metrics now wired to commands

### Breaking Changes
- Engine selection API changed (use riptide_reliability::engine_selection::decide())
- Configuration now requires environment variables (see .env.example)

## [1.0.0] - 2025-10-21
...
```

## Coordination Requirements

**BEFORE starting:**
```bash
# MUST check Task 7.3 completion first
npx claude-flow@alpha hooks session-restore --session-id "swarm-phase7"
# Check phase7/code_quality/status in memory

# If Task 7.3 is complete, proceed:
npx claude-flow@alpha hooks pre-task --description "Task 7.4: Release Preparation"
```

**DURING work:**
```bash
# After each major update
npx claude-flow@alpha hooks post-edit --file "[file]" --memory-key "phase7/release_prep/[step]"
npx claude-flow@alpha hooks notify --message "Release: [what was done]"
```

**AFTER completion:**
```bash
npx claude-flow@alpha hooks post-task --task-id "task-7.4-release-prep"
npx claude-flow@alpha hooks session-end --export-metrics true
```

## Deliverables

1. ✅ CHANGELOG.md updated with all changes
2. ✅ Version bumped to 2.0.0 in all Cargo.toml
3. ✅ Release notes in /docs/RELEASE-NOTES-v2.0.0.md
4. ✅ Migration guide updated (if breaking changes)
5. ✅ README.md version references updated
6. ✅ Pre-release validation checklist complete
7. ✅ Git tag ready (v2.0.0)

## Success Criteria

- ✅ CHANGELOG follows Keep a Changelog format
- ✅ All version numbers consistent (2.0.0)
- ✅ Release notes comprehensive and user-friendly
- ✅ Breaking changes documented
- ✅ All tests passing
- ✅ Ready for git tag and release

## Files to Update

Priority files:
- `/workspaces/eventmesh/CHANGELOG.md`
- `/workspaces/eventmesh/Cargo.toml` (workspace)
- `/workspaces/eventmesh/crates/*/Cargo.toml` (all crates)
- `/workspaces/eventmesh/README.md`
- `/workspaces/eventmesh/docs/RELEASE-NOTES-v2.0.0.md` (create)

## Memory Storage

Store progress at:
- `phase7/release_prep/status` - Current status
- `phase7/release_prep/changelog_done` - CHANGELOG complete
- `phase7/release_prep/version_bump_done` - Version bump complete
- `phase7/release_prep/notes_done` - Release notes complete
- `phase7/release_prep/blockers` - Any issues encountered

## Pre-Release Checklist

- [ ] Task 7.3 (Code Quality) COMPLETE
- [ ] All tests passing (626/630)
- [ ] Clippy warnings <20
- [ ] CHANGELOG updated
- [ ] Version bumped to 2.0.0
- [ ] Release notes written
- [ ] Documentation updated
- [ ] Migration guide ready
- [ ] Security audit clean
- [ ] Performance benchmarks run
- [ ] Ready for git tag
