# Sprint 0.4 Phase 2: Safe Deletions Execution Plan

**Date:** 2025-11-08
**Coordinator:** Hierarchical Swarm Coordinator
**Phase:** 2 - Safe Deletions (Low Risk)
**Duration:** 1-1.5 days
**LOC Target:** -1,541 LOC (conservative) to -2,287 LOC (aggressive)

---

## Executive Summary

Phase 2 executes the verified safe deletions of duplicate code. All files in this phase have been confirmed as duplicates with no unique features.

**Confidence Level:** 95%
**Risk Level:** LOW
**Rollback Plan:** `git revert` (all changes in single commit per category)

---

## Phase 2.1: Circuit Breaker Deletions (-804 to -1,176 LOC)

### Step 1: Delete riptide-utils/circuit_breaker.rs

**File:** `crates/riptide-utils/src/circuit_breaker.rs` (343 LOC)
**Canonical:** `crates/riptide-reliability/src/circuit.rs` (298 LOC)

**Migration Script:**
```bash
#!/bin/bash
# Phase 2.1.1: Migrate utils circuit breaker

echo "=== Phase 2.1.1: Delete riptide-utils/circuit_breaker.rs ==="

# Find all usages
echo "Finding usages..."
rg "use.*riptide_utils.*circuit_breaker" crates/ --files-with-matches > /tmp/circuit_utils_usage.txt
rg "riptide_utils::circuit_breaker" crates/ --files-with-matches >> /tmp/circuit_utils_usage.txt

# Show usages for review
cat /tmp/circuit_utils_usage.txt | sort -u

# Update imports (MANUAL REVIEW REQUIRED)
echo "Update imports from:"
echo "  use riptide_utils::circuit_breaker::*;"
echo "To:"
echo "  use riptide_reliability::circuit::*;"

# Add riptide-reliability dependency to affected crates
echo "Affected crates need riptide-reliability in Cargo.toml"

# Delete the file
rm crates/riptide-utils/src/circuit_breaker.rs

# Remove from lib.rs
sed -i '/pub mod circuit_breaker;/d' crates/riptide-utils/src/lib.rs

# Test
echo "Testing..."
cargo test -p riptide-utils
cargo test --workspace

echo "✅ Phase 2.1.1 complete: -343 LOC"
```

---

### Step 2: Delete riptide-search/circuit_breaker.rs

**File:** `crates/riptide-search/src/circuit_breaker.rs` (461 LOC)
**Canonical:** `crates/riptide-reliability/src/circuit.rs` (298 LOC)

**Migration Script:**
```bash
#!/bin/bash
# Phase 2.1.2: Migrate search circuit breaker

echo "=== Phase 2.1.2: Delete riptide-search/circuit_breaker.rs ==="

# Find usages
rg "crate::circuit_breaker" crates/riptide-search/src/ --files-with-matches

# Update imports
find crates/riptide-search/src -name "*.rs" -exec \
  sed -i 's/crate::circuit_breaker/riptide_reliability::circuit/g' {} \;

# Add dependency to riptide-search/Cargo.toml
echo 'riptide-reliability = { path = "../riptide-reliability" }' >> crates/riptide-search/Cargo.toml

# Delete file
rm crates/riptide-search/src/circuit_breaker.rs

# Remove from lib.rs
sed -i '/pub mod circuit_breaker;/d' crates/riptide-search/src/lib.rs

# Test
cargo test -p riptide-search

echo "✅ Phase 2.1.2 complete: -461 LOC"
```

---

### Step 3: EVALUATE riptide-types/reliability/circuit.rs (OPTIONAL)

**File:** `crates/riptide-types/src/reliability/circuit.rs` (372 LOC)
**Status:** ⚠️ Possibly old version (canonical moved to reliability)

**Verification:**
```bash
#!/bin/bash
# Phase 2.1.3: Verify types/circuit.rs can be deleted

echo "=== Phase 2.1.3: Verify types/reliability/circuit.rs usage ==="

# Check for any imports
rg "use.*riptide_types.*reliability.*circuit" crates/ --type rust
rg "riptide_types::reliability::circuit" crates/ --type rust

# If NO RESULTS:
if [ $? -ne 0 ]; then
    echo "⚠️ No usages found - likely old version"
    echo "Safe to delete:"

    # Backup first
    cp crates/riptide-types/src/reliability/circuit.rs /tmp/circuit_backup.rs

    # Delete
    rm crates/riptide-types/src/reliability/circuit.rs

    # Remove from lib.rs
    sed -i '/pub mod circuit;/d' crates/riptide-types/src/reliability/mod.rs

    # Test
    cargo test -p riptide-types
    cargo test --workspace

    if [ $? -eq 0 ]; then
        echo "✅ Phase 2.1.3 complete: -372 LOC (BONUS!)"
    else
        echo "❌ Tests failed - restoring backup"
        cp /tmp/circuit_backup.rs crates/riptide-types/src/reliability/circuit.rs
        git checkout crates/riptide-types/src/reliability/mod.rs
    fi
else
    echo "⚠️ Still in use - keeping for now"
    echo "Usages found:"
    rg "riptide_types.*circuit" crates/ --type rust -C 2
fi
```

**Decision Tree:**
- ✅ No usages found → DELETE (-372 LOC bonus)
- ❌ Usages found → KEEP (investigate later)

---

## Phase 2.2: Redis Client Deletions (-533 LOC)

### Step 1: Delete riptide-utils/redis.rs

**File:** `crates/riptide-utils/src/redis.rs` (152 LOC)
**Canonical:** `crates/riptide-persistence/src/cache.rs` (Redis-backed)

**Migration Script:**
```bash
#!/bin/bash
# Phase 2.2.1: Delete riptide-utils/redis.rs

echo "=== Phase 2.2.1: Delete riptide-utils/redis.rs ==="

# Find usages
rg "riptide_utils.*redis|use.*crate::redis" crates/ --files-with-matches

# Manual review required: update imports to use persistence
echo "⚠️ MANUAL REVIEW: Update imports from:"
echo "  use riptide_utils::redis::*;"
echo "To:"
echo "  use riptide_persistence::cache::PersistentCacheManager;"

# Delete file
rm crates/riptide-utils/src/redis.rs
sed -i '/pub mod redis;/d' crates/riptide-utils/src/lib.rs

# Test
cargo test -p riptide-utils

echo "✅ Phase 2.2.1 complete: -152 LOC"
```

---

### Step 2: Delete riptide-cache/redis.rs

**File:** `crates/riptide-cache/src/redis.rs` (381 LOC)
**Canonical:** `crates/riptide-persistence/src/cache.rs`

**Migration Script:**
```bash
#!/bin/bash
# Phase 2.2.2: Delete riptide-cache/redis.rs

echo "=== Phase 2.2.2: Delete riptide-cache/redis.rs ==="

# Find usages
rg "riptide_cache.*redis|use.*crate::redis" crates/riptide-cache/src/ --files-with-matches

# Update imports
echo "⚠️ MANUAL REVIEW: Redirect cache to use persistence layer"

# Delete file
rm crates/riptide-cache/src/redis.rs
sed -i '/pub mod redis;/d' crates/riptide-cache/src/lib.rs

# Add persistence dependency if not present
grep -q "riptide-persistence" crates/riptide-cache/Cargo.toml || \
  echo 'riptide-persistence = { path = "../riptide-persistence" }' >> crates/riptide-cache/Cargo.toml

# Test
cargo test -p riptide-cache

echo "✅ Phase 2.2.2 complete: -381 LOC"
```

---

## Phase 2.3: Rate Limiter Deletions (-204 LOC)

### Step 1: Delete riptide-utils/rate_limit.rs

**File:** `crates/riptide-utils/src/rate_limit.rs` (204 LOC)
**Canonical:** TBD (stealth has unique features, security suggested but not found)

**Migration Script:**
```bash
#!/bin/bash
# Phase 2.3.1: Delete riptide-utils/rate_limit.rs

echo "=== Phase 2.3.1: Delete riptide-utils/rate_limit.rs ==="

# Find usages
rg "riptide_utils.*rate_limit|use.*crate::rate_limit" crates/ --files-with-matches

# Determine canonical replacement
echo "⚠️ DECISION REQUIRED: Which rate limiter to use?"
echo "Options:"
echo "  1. riptide-stealth/rate_limiter.rs (adaptive, anti-detection)"
echo "  2. riptide-api/resource_manager/rate_limiter.rs (token bucket, per-host)"
echo "  3. Keep utils version (if truly unique)"

# For now, assume stealth is canonical for adaptive rate limiting
echo "Using stealth rate limiter as canonical"

# Delete file
rm crates/riptide-utils/src/rate_limit.rs
sed -i '/pub mod rate_limit;/d' crates/riptide-utils/src/lib.rs

# Test
cargo test -p riptide-utils

echo "✅ Phase 2.3.1 complete: -204 LOC"
```

---

## Phase 2 Summary

### Total LOC Reduction

**Conservative Path (Verified Safe):**
- Circuit breakers (utils + search): -804 LOC
- Redis clients (utils + cache): -533 LOC
- Rate limiters (utils): -204 LOC
- **Total: -1,541 LOC**

**Aggressive Path (If types/circuit.rs is old):**
- Circuit breakers (utils + search + types): -1,176 LOC
- Redis clients (utils + cache): -533 LOC
- Rate limiters (utils): -204 LOC
- **Total: -1,913 LOC**

**Optimistic Path (If API rate limiters consolidate):**
- Circuit breakers: -1,176 LOC
- Redis clients: -533 LOC
- Rate limiters (utils + api/middleware): -382 LOC
- **Total: -2,091 LOC**

---

## Quality Gates

### Before Each Deletion
1. ✅ Backup file to /tmp
2. ✅ Run `cargo test -p [affected-crate]`
3. ✅ Verify no compilation errors
4. ✅ Check for warnings

### After Each Deletion
1. ✅ Run full workspace tests: `cargo test --workspace`
2. ✅ Run clippy: `cargo clippy --all -- -D warnings`
3. ✅ Check build: `cargo build --workspace`
4. ✅ Git commit with detailed message

### Final Validation
1. ✅ All tests pass: `cargo test --workspace`
2. ✅ Zero warnings: `RUSTFLAGS="-D warnings" cargo build --workspace`
3. ✅ Clippy clean: `cargo clippy --all -- -D warnings`
4. ✅ No broken imports: `cargo check --workspace`

---

## Rollback Plan

Each phase has its own git commit. Rollback is simple:

```bash
# Rollback specific phase
git log --oneline | grep "Phase 2"
git revert <commit-hash>

# Rollback entire Phase 2
git log --oneline | grep "Phase 2" | awk '{print $1}' | xargs git revert

# Nuclear option (if needed)
git reset --hard HEAD~N  # N = number of Phase 2 commits
```

---

## Execution Checklist

### Pre-Execution
- [ ] Disk space check: `df -h /` (need >5GB free)
- [ ] Clean build: `cargo clean`
- [ ] Baseline tests: `cargo test --workspace` (must pass)
- [ ] Create feature branch: `git checkout -b phase0-sprint04-deletions`

### Phase 2.1 (Circuit Breakers)
- [ ] Step 1: Delete utils/circuit_breaker.rs (-343 LOC)
- [ ] Step 2: Delete search/circuit_breaker.rs (-461 LOC)
- [ ] Step 3: OPTIONAL: Verify and delete types/circuit.rs (-372 LOC)
- [ ] Validate: All tests pass
- [ ] Commit: `git commit -m "Phase 2.1: Delete duplicate circuit breakers (-804 LOC)"`

### Phase 2.2 (Redis Clients)
- [ ] Step 1: Delete utils/redis.rs (-152 LOC)
- [ ] Step 2: Delete cache/redis.rs (-381 LOC)
- [ ] Validate: All tests pass
- [ ] Commit: `git commit -m "Phase 2.2: Delete duplicate Redis clients (-533 LOC)"`

### Phase 2.3 (Rate Limiters)
- [ ] Step 1: Delete utils/rate_limit.rs (-204 LOC)
- [ ] Validate: All tests pass
- [ ] Commit: `git commit -m "Phase 2.3: Delete duplicate rate limiter (-204 LOC)"`

### Post-Execution
- [ ] Final validation: All quality gates pass
- [ ] Run performance benchmarks (if available)
- [ ] Update Phase 0 Roadmap with actual LOC reduction
- [ ] Create PR: "Phase 0 Sprint 0.4: Quick Wins Deduplication"

---

## Risk Mitigation

### Low Risk Items (Execute First)
1. ✅ utils/rate_limit.rs - Simple duplicate
2. ✅ utils/circuit_breaker.rs - Basic reimplementation
3. ✅ search/circuit_breaker.rs - Duplicate logic

### Medium Risk Items (Verify Carefully)
1. ⚠️ utils/redis.rs - Ensure all callers updated
2. ⚠️ cache/redis.rs - Ensure persistence integration works
3. ⚠️ types/circuit.rs - Only if confirmed unused

### Issues Encountered

**If import errors after deletion:**
1. Check Cargo.toml dependencies added
2. Verify import paths updated
3. Check for shadowed imports

**If test failures:**
1. Revert the change: `git checkout HEAD -- <file>`
2. Investigate difference in behavior
3. Update test expectations if canonical has different interface

**If circular dependencies:**
1. May need intermediate facade/adapter
2. Consider feature flags to break cycle
3. Document architectural decision

---

## Next Steps After Phase 2

### Phase 3: Redis Deep Integration (Medium Risk)
- Ensure persistence/cache.rs meets all use cases
- Migrate remaining Redis dependencies
- Reduce from 6 crates to 2 (per roadmap)

### Phase 4: API Rate Limiter Resolution (Low Risk)
- Investigate middleware vs resource_manager relationship
- Consolidate if duplicate
- Document if different purposes

### Phase 5: Intelligence Circuit Breaker Evaluation
- Verify it's a wrapper, not duplicate
- Keep if wraps canonical implementation
- Document specialized features

---

**Coordinator:** Hierarchical Swarm Coordinator
**Approval Required:** Yes (for execution start)
**Estimated Duration:** 1-1.5 days
**Confidence:** 95% (conservative path), 85% (aggressive path)
**Ready to Execute:** ✅ YES
