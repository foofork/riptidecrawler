# Phase 5: Validation Automation Roadmap
**Version:** 2.0 (No Changes from Original)
**Date:** 2025-11-08
**Duration:** 3 days
**Status:** Ready for Implementation

---

## Phase Overview

**Goal:** Comprehensive CI/CD checks for architectural violations

**Objectives:**
- Automated validation of handler sizes (<50 LOC)
- Detection of HTTP/JSON leaks in facades
- Layer boundary enforcement via cargo-deny
- Pre-commit hooks for fast feedback
- CI/CD integration for blocking violations

**No Changes:**
This phase remains unchanged from the original ENHANCED_LAYERING_ROADMAP.md as the validation requirements apply regardless of coverage expansion.

---

## 🚨 Quality Gates (MANDATORY - Every Task)

**Zero-tolerance policy for errors/warnings. Every commit must:**

```bash
# 1. Tests pass (NO #[ignore], NO skipped tests)
cargo test -p [affected-crate]  # NOT --workspace (conserve disk)

# 2. Clippy clean (ZERO warnings)
cargo clippy -p [affected-crate] -- -D warnings

# 3. Cargo check passes
cargo check -p [affected-crate]

# 4. Full workspace ONLY for final phase validation
# Use targeted builds: cargo build -p [crate] to save disk space
```

**Commit Rules:**
- ❌ NO commits with failing tests
- ❌ NO commits with clippy warnings
- ❌ NO commits with compilation errors
- ❌ NO #[ignore] on tests without tracking issue
- ✅ Each phase MUST be fully complete before moving to next

---

## Prerequisites from Previous Phases

**Phase 4 Must Be Complete:**
- ✅ All infrastructure consolidated
- ✅ HTTP via ReliableHttpClient
- ✅ Redis via single manager
- ✅ Streaming system refactored
- ✅ Resource manager consolidated

**Phase 1-3 Must Be Complete:**
- ✅ All port traits defined
- ✅ All handlers <50 LOC
- ✅ All business logic in facades
- ✅ Zero serde_json::Value in facades

---

## Sprint 5.1: Enhanced validate_architecture.sh (Day 1)

**Duration:** 1 day
**Priority:** CRITICAL (foundation for automation)

### Enhanced Validation Script

**File:** `/workspaces/eventmesh/scripts/validate_architecture.sh` (ENHANCED)

```bash
#!/bin/bash
set -e

FAIL_COUNT=0

echo "🔍 Enhanced Architecture Validation"
echo "===================================="

# 1. Handler size limits (<50 LOC strict)
echo ""
echo "📏 Checking handler sizes (max: 50 LOC)..."
for file in crates/riptide-api/src/handlers/*.rs; do
    lines=$(wc -l < "$file" | tr -d ' ')
    if [ "$lines" -gt 50 ]; then
        echo "❌ FAIL: $(basename "$file") has $lines lines (max: 50)"
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
done

# 2. HTTP types in facades (ZERO tolerance)
echo ""
echo "🌐 Checking for HTTP types in facades..."
if rg "actix_web::|axum::|HttpMethod|HeaderMap" crates/riptide-facade/src/facades/ >/dev/null 2>&1; then
    echo "❌ FAIL: HTTP types found in facade layer"
    rg "actix_web::|axum::|HttpMethod|HeaderMap" crates/riptide-facade/src/facades/ --files-with-matches
    FAIL_COUNT=$((FAIL_COUNT + 1))
else
    echo "✅ PASS: No HTTP types in facades"
fi

# 3. JSON in facades (ZERO tolerance)
echo ""
echo "📋 Checking for serde_json::Value in facades..."
if rg "serde_json::Value" crates/riptide-facade/src/facades/ >/dev/null 2>&1; then
    echo "❌ FAIL: Untyped JSON found in facades"
    rg "serde_json::Value" crates/riptide-facade/src/facades/ --files-with-matches
    FAIL_COUNT=$((FAIL_COUNT + 1))
else
    echo "✅ PASS: No untyped JSON in facades"
fi

# 4. Forbidden facade dependencies
echo ""
echo "📦 Checking facade dependencies..."
FORBIDDEN_DEPS=$(cargo tree -p riptide-facade --depth 1 | grep -E "axum|actix-web|tower-http|hyper" || true)
if [ -n "$FORBIDDEN_DEPS" ]; then
    echo "❌ FAIL: Forbidden dependencies in riptide-facade:"
    echo "$FORBIDDEN_DEPS"
    FAIL_COUNT=$((FAIL_COUNT + 1))
else
    echo "✅ PASS: Facade dependencies clean"
fi

# 5. Domain depends on infra (forbidden)
echo ""
echo "🏗️  Checking domain layer purity..."
DOMAIN_INFRA_DEPS=$(cargo tree -p riptide-types | grep -E "riptide-(api|facade)" || true)
if [ -n "$DOMAIN_INFRA_DEPS" ]; then
    echo "❌ FAIL: Domain depends on higher layers:"
    echo "$DOMAIN_INFRA_DEPS"
    FAIL_COUNT=$((FAIL_COUNT + 1))
else
    echo "✅ PASS: Domain layer pure"
fi

# 6. Redis dependencies (max 2 crates)
echo ""
echo "💾 Checking Redis dependency scope..."
REDIS_CRATE_COUNT=$(find crates -name "Cargo.toml" -exec grep -l "redis" {} \; | wc -l)
if [ "$REDIS_CRATE_COUNT" -gt 2 ]; then
    echo "❌ FAIL: Redis in $REDIS_CRATE_COUNT crates (max: 2)"
    find crates -name "Cargo.toml" -exec grep -l "redis" {} \;
    FAIL_COUNT=$((FAIL_COUNT + 1))
else
    echo "✅ PASS: Redis scoped to $REDIS_CRATE_COUNT crates"
fi

# 7. Duplicate files (robots, memory managers)
echo ""
echo "🔍 Checking for duplications..."
ROBOTS_COUNT=$(find crates -name "robots.rs" | wc -l)
if [ "$ROBOTS_COUNT" -gt 2 ]; then
    echo "❌ FAIL: $ROBOTS_COUNT robots.rs files found (should be 2: utils + reliability)"
    find crates -name "robots.rs"
    FAIL_COUNT=$((FAIL_COUNT + 1))
else
    echo "✅ PASS: robots.rs properly split (utils + reliability)"
fi

MEMORY_MGR_COUNT=$(find crates -name "memory_manager.rs" | wc -l)
if [ "$MEMORY_MGR_COUNT" -gt 1 ]; then
    echo "❌ FAIL: $MEMORY_MGR_COUNT memory_manager.rs files found (should be 1)"
    find crates -name "memory_manager.rs"
    FAIL_COUNT=$((FAIL_COUNT + 1))
else
    echo "✅ PASS: Single memory manager implementation"
fi

# 8. Clippy warnings (strict)
echo ""
echo "🔧 Running clippy (strict mode)..."
if ! cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tee /tmp/clippy.log; then
    echo "❌ FAIL: Clippy warnings found"
    FAIL_COUNT=$((FAIL_COUNT + 1))
else
    echo "✅ PASS: Clippy clean"
fi

# 9. Facade test coverage (≥90%)
echo ""
echo "📊 Checking facade test coverage (min: 90%)..."
if command -v cargo-llvm-cov >/dev/null 2>&1; then
    cargo llvm-cov --package riptide-facade --json --output-path /tmp/coverage.json >/dev/null 2>&1
    COVERAGE=$(jq '.data[0].totals.lines.percent' /tmp/coverage.json 2>/dev/null || echo "0")

    if (( $(echo "$COVERAGE < 90" | bc -l) )); then
        echo "❌ FAIL: Facade coverage ${COVERAGE}% (min: 90%)"
        FAIL_COUNT=$((FAIL_COUNT + 1))
    else
        echo "✅ PASS: Facade coverage ${COVERAGE}%"
    fi
else
    echo "⚠️  SKIP: cargo-llvm-cov not installed"
fi

# 10. Circular dependencies
echo ""
echo "🔄 Checking for circular dependencies..."
if cargo tree -p riptide-api 2>&1 | grep -q "cycle"; then
    echo "❌ FAIL: Circular dependency detected"
    FAIL_COUNT=$((FAIL_COUNT + 1))
else
    echo "✅ PASS: No circular dependencies"
fi

# 11. Browser abstraction leaks
echo ""
echo "🧠 Checking for browser CDP leaks..."
if rg "chromiumoxide|spider_chrome" crates/riptide-browser-abstraction >/dev/null 2>&1; then
    echo "❌ FAIL: Browser abstraction leaks concrete CDP types"
    FAIL_COUNT=$((FAIL_COUNT + 1))
else
    echo "✅ PASS: Browser abstraction clean"
fi

# Summary
echo ""
echo "===================================="
if [ $FAIL_COUNT -eq 0 ]; then
    echo "✅ ALL CHECKS PASSED!"
    exit 0
else
    echo "❌ $FAIL_COUNT CHECKS FAILED"
    exit 1
fi
```

### CI/CD Integration

**File:** `.github/workflows/architecture_validation.yml` (NEW)

```yaml
name: Architecture Validation

on:
  pull_request:
    branches: [main]
  push:
    branches: [main]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy

      - name: Install cargo-llvm-cov
        run: cargo install cargo-llvm-cov

      - name: Run Architecture Validation
        run: ./scripts/validate_architecture.sh

      - name: Upload Coverage
        if: always()
        uses: codecov/codecov-action@v3
        with:
          files: /tmp/coverage.json
```

### Files Created (Sprint 5.1)

```
CREATE: scripts/validate_architecture.sh (~200 LOC)
CREATE: .github/workflows/architecture_validation.yml (~30 LOC)
UPDATE: README.md (add validation documentation)
```

### Validation (Sprint 5.1)

```bash
# Script is executable
[ -x scripts/validate_architecture.sh ] && echo "PASS" || echo "FAIL"

# Script runs successfully on clean codebase
./scripts/validate_architecture.sh && echo "PASS" || echo "FAIL"

# CI workflow syntax is valid
yamllint .github/workflows/architecture_validation.yml && echo "PASS" || echo "FAIL"
```

---

## Sprint 5.2: cargo-deny Integration (Day 2)

**Duration:** 1 day
**Priority:** CRITICAL (compile-time enforcement)

### Configure cargo-deny Rules

**File:** `deny.toml` (ALREADY EXISTS - REFERENCE CONFIGURATION)

The deny.toml file contains layer boundary enforcement rules:

```toml
# ===================================================================
# CLEAN ARCHITECTURE LAYER BOUNDARY ENFORCEMENT
# ===================================================================
# Enforces separation of concerns and dependency inversion principle
# Based on: API → Application → Domain ←→ Infrastructure pattern

[[bans.deny]]
# Domain layer (riptide-types) must remain pure
name = "riptide-api"
wrappers = ["riptide-types"]
reason = "Domain layer cannot depend on API layer (violates dependency inversion)"

[[bans.deny]]
name = "riptide-facade"
wrappers = ["riptide-types"]
reason = "Domain layer cannot depend on Application layer (violates dependency inversion)"

[[bans.deny]]
name = "riptide-reliability"
wrappers = ["riptide-types"]
reason = "Domain layer cannot depend on Infrastructure (use ports/traits instead)"

[[bans.deny]]
name = "redis"
wrappers = ["riptide-facade"]
reason = "Application layer cannot depend on Redis directly (use CacheStorage port)"

[[bans.deny]]
name = "axum"
wrappers = ["riptide-facade"]
reason = "Application layer cannot depend on HTTP frameworks (violates ports & adapters)"

[[bans.deny]]
name = "sqlx"
wrappers = ["riptide-facade"]
reason = "Application layer cannot depend on databases directly (use Repository port)"

[[bans.deny]]
name = "reqwest"
wrappers = ["riptide-facade"]
reason = "Application layer cannot depend on HTTP clients directly (use ports)"
```

### Installation and Setup

**Install cargo-deny:**
```bash
# Install cargo-deny (one-time)
cargo install cargo-deny

# Verify installation
cargo deny --version
```

**Add cargo-deny to CI Pipeline:**

**File:** `.github/workflows/architecture_validation.yml` (APPEND)

```yaml
    - name: Install cargo-deny
      run: cargo install cargo-deny

    - name: Check layer boundaries
      run: |
        cargo deny check bans
        cargo deny check advisories
        cargo deny check licenses
```

### Local Usage

```bash
# Check all rules
cargo deny check

# Check only layer boundaries
cargo deny check bans

# Output detailed information
cargo deny check --show-stats
```

### Files Modified (Sprint 5.2)

```
VERIFY: deny.toml (already exists with correct rules)
UPDATE: .github/workflows/architecture_validation.yml (add cargo-deny step)
UPDATE: README.md (document cargo-deny usage)
```

### Validation (Sprint 5.2)

```bash
# cargo-deny installed
command -v cargo-deny >/dev/null && echo "PASS" || echo "FAIL"

# No layer boundary violations
cargo deny check bans && echo "PASS" || echo "FAIL"

# CI workflow includes cargo-deny
grep "cargo deny" .github/workflows/architecture_validation.yml && echo "PASS" || echo "FAIL"
```

### Acceptance Criteria

- ✅ cargo-deny installed in CI/CD pipeline
- ✅ All layer boundary violations detected and prevented
- ✅ Zero warnings from `cargo deny check bans`
- ✅ Documentation updated with cargo-deny usage

### Success Metrics

- 🎯 Domain layer purity: 100% (zero forbidden dependencies)
- 🎯 Facade isolation: 100% (no HTTP/database types)
- 🎯 Redis scope: ≤2 crates
- 🎯 CI build time increase: <30 seconds

---

## Sprint 5.3: Pre-commit Hook Installation (Day 3)

**Duration:** 1 day
**Priority:** MEDIUM (developer convenience)

### Create Pre-commit Hook Script

**File:** `.git/hooks/pre-commit` (NEW - chmod +x required)

```bash
#!/bin/bash
# Architecture validation pre-commit hook
# Prevents commits that violate clean architecture rules

set -e

echo "🔍 Running architecture validation..."

# 1. Quick checks (fast fail)
echo "  → Checking for HTTP types in facades..."
if rg -n 'actix_web::|hyper::|axum::' crates/riptide-facade/src/ 2>/dev/null; then
    echo "❌ FAIL: HTTP types found in facade layer"
    echo "Fix: Remove HTTP framework dependencies from riptide-facade"
    exit 1
fi

echo "  → Checking for serde_json::Value in facades..."
if rg -n 'serde_json::Value' crates/riptide-facade/src/ 2>/dev/null; then
    echo "❌ FAIL: serde_json::Value found in facade (use typed DTOs)"
    echo "Fix: Replace with domain types from riptide-types"
    exit 1
fi

# 2. Run cargo-deny (if installed)
if command -v cargo-deny &> /dev/null; then
    echo "  → Checking layer boundaries with cargo-deny..."
    if ! cargo deny check bans --quiet 2>/dev/null; then
        echo "❌ FAIL: Layer boundary violations detected"
        echo "Fix: Run 'cargo deny check bans' for details"
        exit 1
    fi
else
    echo "  ⚠️  SKIP: cargo-deny not installed (run 'cargo install cargo-deny')"
fi

# 3. Run clippy (quick check)
echo "  → Running clippy..."
if ! cargo clippy --workspace --all-targets --quiet -- -D warnings 2>/dev/null; then
    echo "❌ FAIL: Clippy warnings detected"
    echo "Fix: Run 'cargo clippy --workspace --all-targets' for details"
    exit 1
fi

# 4. Check for common anti-patterns
echo "  → Checking for anti-patterns in handlers..."
HANDLER_LOOPS=$(find crates/riptide-api/src/handlers -name "*.rs" -exec grep -Hn '\bfor\b.*{' {} + | grep -v "//" | wc -l)
if [ "$HANDLER_LOOPS" -gt 0 ]; then
    echo "⚠️  WARNING: Found $HANDLER_LOOPS loop(s) in handlers (should be in facades)"
    echo "This is a warning - commit allowed but review recommended"
fi

echo "✅ All pre-commit checks passed!"
exit 0
```

### Install Pre-commit Hook

```bash
# Make script executable
chmod +x .git/hooks/pre-commit

# Verify it runs
.git/hooks/pre-commit
```

### Document Bypass Procedure

**File:** `docs/architecture/PRE_COMMIT_HOOK.md` (NEW)

```markdown
# Pre-commit Hook Documentation

## Purpose
Prevents architectural violations from being committed to the repository.

## Checks Performed
1. HTTP types in facade layer
2. serde_json::Value usage in facades
3. Layer boundary violations (via cargo-deny)
4. Clippy warnings
5. Business logic loops in handlers

## Bypass Procedure
**ONLY in emergencies (e.g., critical hotfix):**

```bash
# Option 1: Skip hooks for single commit
git commit --no-verify -m "hotfix: critical bug fix"

# Option 2: Temporarily disable hook
mv .git/hooks/pre-commit .git/hooks/pre-commit.disabled
git commit -m "your message"
mv .git/hooks/pre-commit.disabled .git/hooks/pre-commit
```

**NOTE:** All bypassed commits MUST be fixed in follow-up PR within 24 hours.

## Installation
```bash
chmod +x .git/hooks/pre-commit
```

## Troubleshooting
- **Hook too slow:** Comment out clippy check for faster commits
- **False positives:** Whitelist patterns in the hook script
- **Not running:** Check file is executable (`ls -la .git/hooks/pre-commit`)
```

### Add Hook Installation to Setup Documentation

**File:** `README.md` (UPDATE - add to Development Setup section)

```markdown
### Development Setup

1. Install Rust toolchain
2. Install dependencies:
   ```bash
   cargo install cargo-deny cargo-llvm-cov
   ```
3. Install pre-commit hook:
   ```bash
   chmod +x .git/hooks/pre-commit
   ```
4. Verify setup:
   ```bash
   ./scripts/validate_architecture.sh
   ```
```

### Files Created (Sprint 5.3)

```
CREATE: .git/hooks/pre-commit (~80 LOC)
CREATE: docs/architecture/PRE_COMMIT_HOOK.md (~50 LOC)
UPDATE: README.md (add pre-commit hook installation)
```

### Validation (Sprint 5.3)

```bash
# Pre-commit hook installed and executable
[ -x .git/hooks/pre-commit ] && echo "PASS" || echo "FAIL"

# Hook runs in <10 seconds
time .git/hooks/pre-commit

# Documentation exists
[ -f docs/architecture/PRE_COMMIT_HOOK.md ] && echo "PASS" || echo "FAIL"
```

### Acceptance Criteria

- ✅ Pre-commit hook installed and executable
- ✅ All checks run in <10 seconds
- ✅ Documentation explains bypass procedure
- ✅ Zero false positives on clean codebase

### Success Metrics

- 🎯 Hook run time: <10 seconds
- 🎯 False positive rate: 0%
- 🎯 Architectural violation prevention: 100%
- 🎯 Developer adoption rate: 100% (team consensus)

**Notes:**
- Hook is local only (not enforced in CI - that's what GitHub Actions is for)
- Optional but recommended for all developers
- Can be customized per developer workflow preferences

---

## Success Criteria for Phase 5

### Quantitative Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Validation script exists** | ✅ | `[ -f scripts/validate_architecture.sh ]` |
| **Script passes on clean codebase** | ✅ | `./scripts/validate_architecture.sh` |
| **cargo-deny installed in CI** | ✅ | `grep "cargo deny" .github/workflows/architecture_validation.yml` |
| **Pre-commit hook installed** | ✅ | `[ -x .git/hooks/pre-commit ]` |
| **All docs updated** | ✅ | `grep "validate_architecture" README.md` |

### Qualitative Checks

- [ ] validate_architecture.sh runs in CI/CD
- [ ] cargo-deny enforces layer boundaries
- [ ] Pre-commit hook prevents violations
- [ ] Documentation complete
- [ ] Team trained on bypass procedures

### Final Validation

```bash
# Run complete validation
./scripts/validate_architecture.sh

# Verify CI integration
cat .github/workflows/architecture_validation.yml

# Test pre-commit hook
.git/hooks/pre-commit

# Verify cargo-deny
cargo deny check
```

---

## LOC Impact Summary

| Item | LOC |
|------|-----|
| validate_architecture.sh | +200 |
| architecture_validation.yml | +30 |
| pre-commit hook | +80 |
| PRE_COMMIT_HOOK.md | +50 |
| README updates | +20 |
| **Total** | **+380 LOC** |

---

## Timeline

**Day 1:** Sprint 5.1 (Enhanced validation script + CI integration)
**Day 2:** Sprint 5.2 (cargo-deny integration)
**Day 3:** Sprint 5.3 (Pre-commit hook installation)

**Total:** 3 days

---

## Related Documents

- [PHASE_4_INFRASTRUCTURE_ROADMAP.md](./PHASE_4_INFRASTRUCTURE_ROADMAP.md) (prerequisite)
- [ENHANCED_LAYERING_ROADMAP_INDEX.md](./ENHANCED_LAYERING_ROADMAP_INDEX.md) (roadmap index)
- [API_CRATE_COVERAGE_ANALYSIS.md](../architecture/API_CRATE_COVERAGE_ANALYSIS.md)

---

**Document Version:** 2.0
**Status:** ✅ Ready for Implementation
**Next Review:** After Phase 5 completion (final validation)
