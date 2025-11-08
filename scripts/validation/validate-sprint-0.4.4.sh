#!/bin/bash
# Validation Script for Sprint 0.4.4: Rate Limiter Consolidation

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPORT_DIR="${SCRIPT_DIR}/../../tests/validation-reports"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT_FILE="${REPORT_DIR}/sprint-0.4.4-${TIMESTAMP}.md"

echo "=== Sprint 0.4.4 Validation: Rate Limiter Consolidation ==="
echo ""

# Initialize report
cat > "$REPORT_FILE" <<EOF
# Sprint 0.4.4 Validation Report

**Task:** Rate Limiter Consolidation
**Date:** $(date +%Y-%m-%d\ %H:%M:%S)
**Status:** IN PROGRESS

## Validation Steps

EOF

echo "Step 1: Verify single implementation..."
RL_COUNT=$(rg "struct.*RateLimiter" crates/ --type rust | wc -l)
echo "Found $RL_COUNT RateLimiter struct definitions"

cat >> "$REPORT_FILE" <<EOF
### 1. Single Implementation Verification
- **Expected:** 1 implementation (in riptide-security)
- **Actual:** $RL_COUNT implementation(s)
- **Status:** $([ "$RL_COUNT" -eq 1 ] && echo "✅ PASS" || echo "❌ FAIL")

EOF

if [ "$RL_COUNT" -ne 1 ]; then
    echo "❌ FAIL: Expected 1 RateLimiter implementation, found $RL_COUNT"
    echo "" >> "$REPORT_FILE"
    echo "**Locations Found:**" >> "$REPORT_FILE"
    echo "\`\`\`" >> "$REPORT_FILE"
    rg "struct.*RateLimiter" crates/ --type rust >> "$REPORT_FILE" 2>&1 || true
    echo "\`\`\`" >> "$REPORT_FILE"
    exit 1
fi

echo "Step 2: Verify RateLimiter is in riptide-security..."
RL_LOCATION=$(rg "struct.*RateLimiter" crates/ --type rust -l | head -1)
EXPECTED_CRATE="crates/riptide-security"

cat >> "$REPORT_FILE" <<EOF
### 2. Location Verification
- **Expected Crate:** $EXPECTED_CRATE
- **Actual Location:** $RL_LOCATION
- **Status:** $(echo "$RL_LOCATION" | grep -q "$EXPECTED_CRATE" && echo "✅ PASS" || echo "❌ FAIL")

EOF

if ! echo "$RL_LOCATION" | grep -q "$EXPECTED_CRATE"; then
    echo "❌ FAIL: RateLimiter not in riptide-security"
    exit 1
fi

echo "Step 3: Test security features..."
SECURITY_STATUS=0
cargo test -p riptide-security --no-fail-fast 2>&1 | tee -a /tmp/security-test.log || SECURITY_STATUS=$?

cat >> "$REPORT_FILE" <<EOF
### 3. Security Feature Tests
- **riptide-security:** $([ "$SECURITY_STATUS" -eq 0 ] && echo "✅ PASS" || echo "❌ FAIL")

EOF

echo "Step 4: Check stealth features preserved (if kept)..."
if [ -d "crates/riptide-stealth" ]; then
    STEALTH_STATUS=0
    cargo test -p riptide-stealth -- rate 2>&1 | tee -a /tmp/stealth-test.log || STEALTH_STATUS=$?

    cat >> "$REPORT_FILE" <<EOF
### 4. Stealth Feature Tests
- **riptide-stealth rate tests:** $([ "$STEALTH_STATUS" -eq 0 ] && echo "✅ PASS" || echo "❌ FAIL")

EOF
else
    cat >> "$REPORT_FILE" <<EOF
### 4. Stealth Feature Tests
- **Status:** ⏭️ SKIPPED (riptide-stealth not found)

EOF
fi

echo "Step 5: Verify no duplicate implementations..."
DUPLICATE_CHECK=$(rg "impl.*RateLimiter" crates/ --type rust | wc -l)

cat >> "$REPORT_FILE" <<EOF
### 5. Duplicate Implementation Check
- **RateLimiter implementations:** $DUPLICATE_CHECK
- **Status:** ℹ️ INFO

EOF

echo "Step 6: Build with zero warnings..."
RUSTFLAGS="-D warnings" cargo build -p riptide-security 2>&1 | tee -a /tmp/build.log
BUILD_STATUS=$?

cat >> "$REPORT_FILE" <<EOF
### 6. Zero-Warning Build
- **Build Status:** $([ "$BUILD_STATUS" -eq 0 ] && echo "✅ PASS" || echo "❌ FAIL")

EOF

# Final status
OVERALL_STATUS="PASS"
if [ "$RL_COUNT" -ne 1 ] || [ "$SECURITY_STATUS" -ne 0 ] || [ "$BUILD_STATUS" -ne 0 ]; then
    OVERALL_STATUS="FAIL"
elif [ -d "crates/riptide-stealth" ] && [ "$STEALTH_STATUS" -ne 0 ]; then
    OVERALL_STATUS="FAIL"
fi

cat >> "$REPORT_FILE" <<EOF

## Overall Result

**Status:** $([ "$OVERALL_STATUS" = "PASS" ] && echo "✅ PASS" || echo "❌ FAIL")

$([ "$OVERALL_STATUS" = "FAIL" ] && echo "### Issues Detected

Review logs:
- Security tests: /tmp/security-test.log
- Stealth tests: /tmp/stealth-test.log (if applicable)
- Build log: /tmp/build.log
" || echo "All validation checks passed successfully.")

---
**Validator:** validate-sprint-0.4.4.sh
**Report:** $REPORT_FILE
EOF

echo ""
echo "=== Validation Complete ==="
echo "Report: $REPORT_FILE"
echo "Overall Status: $OVERALL_STATUS"

[ "$OVERALL_STATUS" = "PASS" ] && exit 0 || exit 1
