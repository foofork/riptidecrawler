# Phase 0 Validation Scripts

## Overview

This directory contains comprehensive validation scripts for Phase 0 Cleanup to ensure zero regressions during code consolidation.

## Validation Philosophy

**TEST AFTER EVERY CHANGE** - No batching allowed!

Each consolidation task must be validated immediately after completion to catch regressions early.

## Scripts

### Individual Sprint Validators

#### `validate-sprint-0.4.1.sh` - Robots.txt Consolidation
Validates:
- Single robots.txt file in riptide-fetch
- No duplicate implementations
- Tests pass for riptide-spider and riptide-fetch
- No old import patterns
- Zero-warning build

#### `validate-sprint-0.4.2.sh` - Circuit Breaker Consolidation
Validates:
- Single CircuitBreaker implementation
- Located in riptide-reliability (correct domain)
- No domain violations (removed from riptide-types)
- All affected crates test successfully
- No broken references
- Zero-warning build

#### `validate-sprint-0.4.3.sh` - Redis Client Consolidation
Validates:
- No duplicate Redis clients in utils/cache
- Caching functionality works correctly
- Redis dependency count ≤2
- Zero-warning build

#### `validate-sprint-0.4.4.sh` - Rate Limiter Consolidation
Validates:
- Single RateLimiter implementation
- Located in riptide-security
- Security features preserved
- Stealth features preserved (if kept)
- Zero-warning build

### Full Workspace Validator

#### `validate-full-workspace.sh` - Quality Gates
Comprehensive validation including:
- Disk space check (>5GB required)
- Full workspace build with zero warnings
- Clippy checks (zero warnings)
- Complete test suite
- LOC reduction metrics
- Crate count reduction

### Master Suite

#### `run-all-validations.sh` - Master Validation Suite
Runs all validations in sequence:
1. Sprint 0.4.1 (optional - may already be consolidated)
2. Sprint 0.4.2 (required)
3. Sprint 0.4.3 (required)
4. Sprint 0.4.4 (required)
5. Full workspace validation (required)

Generates master report with executive summary.

## Usage

### Run Individual Validation
```bash
cd /workspaces/eventmesh
./scripts/validation/validate-sprint-0.4.2.sh
```

### Run Full Validation Suite
```bash
cd /workspaces/eventmesh
./scripts/validation/run-all-validations.sh
```

### Run After Each Change
```bash
# After completing Task 0.4.2
./scripts/validation/validate-sprint-0.4.2.sh

# After completing Task 0.4.3
./scripts/validation/validate-sprint-0.4.3.sh

# After completing all tasks
./scripts/validation/run-all-validations.sh
```

## Reports

All validation reports are saved to:
```
/workspaces/eventmesh/tests/validation-reports/
```

### Report Types

- `baseline-metrics.md` - Initial state before cleanup
- `sprint-0.4.X-TIMESTAMP.md` - Individual sprint results
- `full-workspace-TIMESTAMP.md` - Quality gate results
- `master-validation-TIMESTAMP.md` - Executive summary

## Quality Gates

### Required Gates (Must Pass)
- ✅ Build with RUSTFLAGS="-D warnings" (zero warnings)
- ✅ All affected crate tests pass
- ✅ Disk space >5GB
- ✅ No duplicate implementations
- ✅ Correct domain boundaries

### Warning Gates (Should Pass)
- ⚠️ Clippy clean
- ⚠️ Full test suite passes
- ⚠️ No broken references

### Info Gates (Track Progress)
- ℹ️ LOC reduction
- ℹ️ Crate count reduction
- ℹ️ Build time improvement

## Metrics Tracking

Each validation tracks:
- Tests passing: X/Y
- LOC deleted: cumulative
- Build warnings: count
- Crates removed: count
- Build time: before/after

## Regression Testing

Run full integration tests after:
- Each sprint completion
- Any breaking change
- Before final sign-off

## Failure Handling

If validation fails:
1. Review the detailed report
2. Check log files in /tmp/
3. Fix issues immediately
4. Re-run validation
5. Report to hierarchical-coordinator

## Example Workflow

```bash
# 1. Establish baseline (before changes)
cat tests/validation-reports/baseline-metrics.md

# 2. Make changes for Task 0.4.2
# ... edit files, consolidate circuit breaker ...

# 3. Validate immediately
./scripts/validation/validate-sprint-0.4.2.sh

# 4. If pass, continue to next task
# If fail, fix and re-validate

# 5. After all sprints, run full validation
./scripts/validation/run-all-validations.sh

# 6. Review master report
cat tests/validation-reports/master-validation-*.md
```

## Best Practices

1. **Never Skip Validation** - Test after every file deletion
2. **Fix Failures Immediately** - Don't accumulate technical debt
3. **Review Reports** - Understand what changed and why
4. **Track Metrics** - Monitor LOC reduction and build time
5. **Document Issues** - Report all failures to coordinator

## Troubleshooting

### Build Failures
```bash
# Check log
cat /tmp/build.log

# Try clean build
cargo clean
cargo build --workspace
```

### Test Failures
```bash
# Check specific test log
cat /tmp/reliability-test.log

# Run test with verbose output
cargo test -p riptide-reliability -- --nocapture
```

### Disk Space Issues
```bash
# Check space
df -h /

# Clean if needed
cargo clean
```

## Integration with Phase 0 Plan

These validators align with the Comprehensive Refactoring Roadmap v3.1:
- **Sprint 0.4**: Quick Wins (highest priority)
- **Sprint 0.1-0.3**: Systematic cleanup
- **Quality Gates**: Zero-warning, zero-regression guarantee

---

**Maintained by:** Testing & Validation Specialist (tester-agent)
**Version:** 1.0
**Last Updated:** 2025-11-08
