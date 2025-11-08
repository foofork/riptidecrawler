# Phase 0 Validation - Quick Reference Guide

## 🚨 CRITICAL: Test After EVERY Change!

**NO BATCHING ALLOWED** - Validate immediately after each consolidation task.

## Quick Start

```bash
# View baseline metrics
cat tests/validation-reports/baseline-metrics.md

# Run validation after each task
./scripts/validation/validate-sprint-0.4.2.sh  # After circuit breaker consolidation
./scripts/validation/validate-sprint-0.4.3.sh  # After Redis consolidation
./scripts/validation/validate-sprint-0.4.4.sh  # After rate limiter consolidation

# Run full validation suite at end
./scripts/validation/run-all-validations.sh
```

## Validation Checklist

### Before Starting Phase 0
- [x] Baseline metrics established
- [x] Validation scripts created
- [x] Disk space checked (23GB available ✅)
- [x] Current build status verified (8m 14s, zero warnings ✅)

### Sprint 0.4.1: Robots.txt
**Status:** ✅ Already consolidated (skip)
- 1 robots.txt file in riptide-fetch
- No action needed

### Sprint 0.4.2: Circuit Breaker (HIGH PRIORITY)
**Status:** ⚠️ NEEDS CONSOLIDATION
- Currently: 17 implementations
- Target: 1 implementation in riptide-reliability
- Run: `./scripts/validation/validate-sprint-0.4.2.sh`

### Sprint 0.4.3: Redis Client (MEDIUM PRIORITY)
**Status:** ⚠️ NEEDS CONSOLIDATION
- Currently: 3 instances in utils/cache
- Target: 1-2 (persistence + optional cache)
- Run: `./scripts/validation/validate-sprint-0.4.3.sh`

### Sprint 0.4.4: Rate Limiter (HIGH PRIORITY)
**Status:** ⚠️ NEEDS CONSOLIDATION
- Currently: 12 implementations
- Target: 1 implementation in riptide-security
- Run: `./scripts/validation/validate-sprint-0.4.4.sh`

### Final Quality Gates
- [ ] Full workspace build (zero warnings)
- [ ] Clippy clean
- [ ] All tests pass
- [ ] LOC reduction verified (~6,260 lines)
- [ ] Crate reduction verified (2-3 crates)
- Run: `./scripts/validation/validate-full-workspace.sh`

## Expected Outcomes

### Baseline (Before Cleanup)
- **LOC:** 281,733 lines
- **Crates:** 29
- **Build Time:** 8m 14s
- **Warnings:** 0
- **Test Status:** Some failures (facade tests)

### Target (After Phase 0)
- **LOC:** ~275,473 lines (-6,260)
- **Crates:** 26-27 (-2 to -3)
- **Build Time:** Improved (target: <7m)
- **Warnings:** 0
- **Test Status:** All passing

## Quality Gate Thresholds

### ✅ PASS Criteria
- Build: RUSTFLAGS="-D warnings" cargo build --workspace succeeds
- Tests: All affected crate tests pass
- Consolidation: Single implementation in correct domain
- References: No broken imports

### ⚠️ WARNING Criteria
- Clippy: Minor issues found
- Tests: Non-critical test failures
- Dependencies: Slightly more than expected

### ❌ FAIL Criteria
- Build: Warnings or errors
- Tests: Critical test failures
- Consolidation: Duplicates remain
- Domain: Violations present

## Common Issues & Solutions

### Issue: Build fails with warnings
```bash
# Solution: Review build log
cat /tmp/full-build.log

# Clean and rebuild
cargo clean
RUSTFLAGS="-D warnings" cargo build --workspace
```

### Issue: Tests fail after consolidation
```bash
# Solution: Check specific test log
cat /tmp/reliability-test.log

# Run with verbose output
cargo test -p riptide-reliability -- --nocapture
```

### Issue: Duplicate implementations detected
```bash
# Solution: Search for remaining duplicates
rg "struct.*CircuitBreaker" crates/ --type rust

# Verify import paths updated
rg "use.*circuit_breaker" crates/ --type rust
```

### Issue: Disk space low
```bash
# Solution: Clean build artifacts
cargo clean
df -h /  # Verify space freed
```

## Report Locations

All validation reports saved to: `/workspaces/eventmesh/tests/validation-reports/`

- `baseline-metrics.md` - Initial state
- `sprint-0.4.X-TIMESTAMP.md` - Individual sprint results
- `full-workspace-TIMESTAMP.md` - Quality gates
- `master-validation-TIMESTAMP.md` - Executive summary

## Workflow Example

```bash
# 1. Review baseline
cat tests/validation-reports/baseline-metrics.md

# 2. Coder completes Task 0.4.2 (circuit breaker consolidation)
# ... files edited, duplicates removed ...

# 3. IMMEDIATELY validate (no waiting!)
./scripts/validation/validate-sprint-0.4.2.sh

# 4a. If PASS: Continue to Task 0.4.3
# 4b. If FAIL: Stop, fix issues, re-validate

# 5. Repeat for each task

# 6. After all tasks, run master suite
./scripts/validation/run-all-validations.sh

# 7. Review master report
cat tests/validation-reports/master-validation-*.md

# 8. Report results to hierarchical-coordinator
```

## Metrics to Track

Track these metrics for each sprint:

| Metric | Baseline | After 0.4.2 | After 0.4.3 | After 0.4.4 | Target |
|--------|----------|-------------|-------------|-------------|--------|
| LOC | 281,733 | - | - | - | ~275,473 |
| Crates | 29 | - | - | - | 26-27 |
| Build Time | 8m 14s | - | - | - | <7m |
| Warnings | 0 | 0 | 0 | 0 | 0 |
| Circuit Breakers | 17 | 1 | 1 | 1 | 1 |
| Redis Clients | 3 | 3 | 1-2 | 1-2 | 1-2 |
| Rate Limiters | 12 | 12 | 12 | 1 | 1 |

## Emergency Contacts

- **Hierarchical Coordinator:** Report all failures immediately
- **Coder Agent:** Coordinate on fixes needed
- **Architect Agent:** Consult on domain boundary questions

## Key Reminders

1. ⚡ **Test immediately** - Don't wait, don't batch
2. 🎯 **Zero warnings** - Non-negotiable quality gate
3. 📊 **Track metrics** - Document every change
4. 🚨 **Report failures** - Escalate issues immediately
5. ✅ **Fix before continuing** - Don't accumulate debt

---

**Quick Guide Version:** 1.0
**Last Updated:** 2025-11-08
**Maintained by:** Testing & Validation Specialist
