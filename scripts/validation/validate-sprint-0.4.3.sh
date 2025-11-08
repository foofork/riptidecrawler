#!/bin/bash
# Validation Script for Sprint 0.4.3: Redis Client Consolidation

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPORT_DIR="${SCRIPT_DIR}/../../tests/validation-reports"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT_FILE="${REPORT_DIR}/sprint-0.4.3-${TIMESTAMP}.md"

echo "=== Sprint 0.4.3 Validation: Redis Client Consolidation ==="
echo ""

# Initialize report
cat > "$REPORT_FILE" <<EOF
# Sprint 0.4.3 Validation Report

**Task:** Redis Client Consolidation
**Date:** $(date +%Y-%m-%d\ %H:%M:%S)
**Status:** IN PROGRESS

## Validation Steps

EOF

echo "Step 1: Verify consolidation (no old clients in utils/cache)..."
OLD_REDIS=$(rg "RedisClient|RedisPool" crates/riptide-utils/src/ crates/riptide-cache/src/ --type rust 2>/dev/null | wc -l)
echo "Found $OLD_REDIS RedisClient/RedisPool references in utils/cache"

cat >> "$REPORT_FILE" <<EOF
### 1. Consolidation Verification
- **Old clients in utils/cache:** $OLD_REDIS
- **Expected:** 0
- **Status:** $([ "$OLD_REDIS" -eq 0 ] && echo "✅ PASS" || echo "❌ FAIL")

EOF

if [ "$OLD_REDIS" -gt 0 ]; then
    echo "❌ FAIL: Old Redis clients still exist in utils/cache"
    echo "" >> "$REPORT_FILE"
    echo "**Locations Found:**" >> "$REPORT_FILE"
    echo "\`\`\`" >> "$REPORT_FILE"
    rg "RedisClient|RedisPool" crates/riptide-utils/src/ crates/riptide-cache/src/ --type rust >> "$REPORT_FILE" 2>&1 || true
    echo "\`\`\`" >> "$REPORT_FILE"
fi

echo "Step 2: Test caching functionality..."
PERSISTENCE_STATUS=0
CACHE_STATUS=0

cargo test -p riptide-persistence --no-fail-fast 2>&1 | tee -a /tmp/persistence-test.log || PERSISTENCE_STATUS=$?
cargo test -p riptide-cache --no-fail-fast 2>&1 | tee -a /tmp/cache-test.log || CACHE_STATUS=$?

cat >> "$REPORT_FILE" <<EOF
### 2. Caching Tests
- **riptide-persistence:** $([ "$PERSISTENCE_STATUS" -eq 0 ] && echo "✅ PASS" || echo "❌ FAIL")
- **riptide-cache:** $([ "$CACHE_STATUS" -eq 0 ] && echo "✅ PASS" || echo "❌ FAIL")

EOF

echo "Step 3: Verify dependency count..."
REDIS_DEPS=$(find crates -name "Cargo.toml" -exec grep -l "redis" {} \; | wc -l)
echo "Found $REDIS_DEPS crates with redis dependency"

cat >> "$REPORT_FILE" <<EOF
### 3. Dependency Count
- **Crates with redis dependency:** $REDIS_DEPS
- **Expected:** ≤2 (persistence + optional cache)
- **Status:** $([ "$REDIS_DEPS" -le 2 ] && echo "✅ PASS" || echo "⚠️ WARNING")

EOF

if [ "$REDIS_DEPS" -gt 2 ]; then
    echo "" >> "$REPORT_FILE"
    echo "**Crates with redis dependency:**" >> "$REPORT_FILE"
    echo "\`\`\`" >> "$REPORT_FILE"
    find crates -name "Cargo.toml" -exec grep -l "redis" {} \; >> "$REPORT_FILE"
    echo "\`\`\`" >> "$REPORT_FILE"
fi

echo "Step 4: Check for proper imports..."
IMPORT_COUNT=$(rg "use.*redis" crates/ --type rust | wc -l)

cat >> "$REPORT_FILE" <<EOF
### 4. Import Analysis
- **Total redis imports:** $IMPORT_COUNT
- **Status:** ℹ️ INFO

EOF

echo "Step 5: Build affected crates..."
RUSTFLAGS="-D warnings" cargo build -p riptide-persistence -p riptide-cache 2>&1 | tee -a /tmp/build.log
BUILD_STATUS=$?

cat >> "$REPORT_FILE" <<EOF
### 5. Zero-Warning Build
- **Build Status:** $([ "$BUILD_STATUS" -eq 0 ] && echo "✅ PASS" || echo "❌ FAIL")

EOF

# Final status
OVERALL_STATUS="PASS"
if [ "$OLD_REDIS" -gt 0 ] || [ "$PERSISTENCE_STATUS" -ne 0 ] || [ "$CACHE_STATUS" -ne 0 ] || [ "$BUILD_STATUS" -ne 0 ]; then
    OVERALL_STATUS="FAIL"
fi

cat >> "$REPORT_FILE" <<EOF

## Overall Result

**Status:** $([ "$OVERALL_STATUS" = "PASS" ] && echo "✅ PASS" || echo "❌ FAIL")

$([ "$OVERALL_STATUS" = "FAIL" ] && echo "### Issues Detected

Review logs:
- Persistence tests: /tmp/persistence-test.log
- Cache tests: /tmp/cache-test.log
- Build log: /tmp/build.log
" || echo "All validation checks passed successfully.")

---
**Validator:** validate-sprint-0.4.3.sh
**Report:** $REPORT_FILE
EOF

echo ""
echo "=== Validation Complete ==="
echo "Report: $REPORT_FILE"
echo "Overall Status: $OVERALL_STATUS"

[ "$OVERALL_STATUS" = "PASS" ] && exit 0 || exit 1
