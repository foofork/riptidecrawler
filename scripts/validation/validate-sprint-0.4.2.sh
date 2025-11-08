#!/bin/bash
# Validation Script for Sprint 0.4.2: Circuit Breaker Consolidation

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPORT_DIR="${SCRIPT_DIR}/../../tests/validation-reports"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT_FILE="${REPORT_DIR}/sprint-0.4.2-${TIMESTAMP}.md"

echo "=== Sprint 0.4.2 Validation: Circuit Breaker Consolidation ==="
echo ""

# Initialize report
cat > "$REPORT_FILE" <<EOF
# Sprint 0.4.2 Validation Report

**Task:** Circuit Breaker Consolidation
**Date:** $(date +%Y-%m-%d\ %H:%M:%S)
**Status:** IN PROGRESS

## Validation Steps

EOF

echo "Step 1: Verify single source of truth..."
CB_COUNT=$(rg "struct.*CircuitBreaker" crates/ --type rust | wc -l)
echo "Found $CB_COUNT CircuitBreaker struct definitions"

cat >> "$REPORT_FILE" <<EOF
### 1. Single Source of Truth Verification
- **Expected:** 1 implementation (in riptide-reliability)
- **Actual:** $CB_COUNT implementation(s)
- **Status:** $([ "$CB_COUNT" -eq 1 ] && echo "✅ PASS" || echo "❌ FAIL")

EOF

if [ "$CB_COUNT" -ne 1 ]; then
    echo "❌ FAIL: Expected 1 CircuitBreaker implementation, found $CB_COUNT"
    echo "" >> "$REPORT_FILE"
    echo "**Locations Found:**" >> "$REPORT_FILE"
    echo "\`\`\`" >> "$REPORT_FILE"
    rg "struct.*CircuitBreaker" crates/ --type rust >> "$REPORT_FILE" 2>&1 || true
    echo "\`\`\`" >> "$REPORT_FILE"
    exit 1
fi

echo "Step 2: Verify CircuitBreaker is in riptide-reliability..."
CB_LOCATION=$(rg "struct.*CircuitBreaker" crates/ --type rust -l | head -1)
EXPECTED_CRATE="crates/riptide-reliability"

cat >> "$REPORT_FILE" <<EOF
### 2. Location Verification
- **Expected Crate:** $EXPECTED_CRATE
- **Actual Location:** $CB_LOCATION
- **Status:** $(echo "$CB_LOCATION" | grep -q "$EXPECTED_CRATE" && echo "✅ PASS" || echo "❌ FAIL")

EOF

if ! echo "$CB_LOCATION" | grep -q "$EXPECTED_CRATE"; then
    echo "❌ FAIL: CircuitBreaker not in riptide-reliability"
    exit 1
fi

echo "Step 3: Check domain violation fixed..."
if [ -f "crates/riptide-types/src/reliability/circuit.rs" ]; then
    echo "❌ FAIL: Domain violation remains - circuit.rs still in riptide-types"
    cat >> "$REPORT_FILE" <<EOF
### 3. Domain Violation Check
- **Status:** ❌ FAIL
- **Issue:** crates/riptide-types/src/reliability/circuit.rs still exists

EOF
    exit 1
else
    cat >> "$REPORT_FILE" <<EOF
### 3. Domain Violation Check
- **Status:** ✅ PASS
- **Result:** No circuit breaker implementation in riptide-types

EOF
fi

echo "Step 4: Test all affected crates..."
RELIABILITY_STATUS=0
UTILS_STATUS=0
INTELLIGENCE_STATUS=0

cargo test -p riptide-reliability --no-fail-fast 2>&1 | tee -a /tmp/reliability-test.log || RELIABILITY_STATUS=$?
cargo test -p riptide-utils --no-fail-fast 2>&1 | tee -a /tmp/utils-test.log || UTILS_STATUS=$?
cargo test -p riptide-intelligence --no-fail-fast 2>&1 | tee -a /tmp/intelligence-test.log || INTELLIGENCE_STATUS=$?

cat >> "$REPORT_FILE" <<EOF
### 4. Affected Crate Tests
- **riptide-reliability:** $([ "$RELIABILITY_STATUS" -eq 0 ] && echo "✅ PASS" || echo "❌ FAIL")
- **riptide-utils:** $([ "$UTILS_STATUS" -eq 0 ] && echo "✅ PASS" || echo "❌ FAIL")
- **riptide-intelligence:** $([ "$INTELLIGENCE_STATUS" -eq 0 ] && echo "✅ PASS" || echo "❌ FAIL")

EOF

echo "Step 5: Verify no broken references..."
BROKEN_REFS=$(rg "use.*circuit_breaker" crates/ --type rust | grep -v "riptide_reliability" | wc -l)

cat >> "$REPORT_FILE" <<EOF
### 5. Reference Verification
- **Broken references (not using riptide_reliability):** $BROKEN_REFS
- **Status:** $([ "$BROKEN_REFS" -eq 0 ] && echo "✅ PASS" || echo "⚠️ WARNING")

EOF

if [ "$BROKEN_REFS" -gt 0 ]; then
    echo "⚠️ WARNING: Found $BROKEN_REFS references not using riptide_reliability"
    echo "" >> "$REPORT_FILE"
    echo "**References Found:**" >> "$REPORT_FILE"
    echo "\`\`\`" >> "$REPORT_FILE"
    rg "use.*circuit_breaker" crates/ --type rust | grep -v "riptide_reliability" >> "$REPORT_FILE" 2>&1 || true
    echo "\`\`\`" >> "$REPORT_FILE"
fi

echo "Step 6: Build with zero warnings..."
RUSTFLAGS="-D warnings" cargo build -p riptide-reliability -p riptide-utils -p riptide-intelligence 2>&1 | tee -a /tmp/build.log
BUILD_STATUS=$?

cat >> "$REPORT_FILE" <<EOF
### 6. Zero-Warning Build
- **Build Status:** $([ "$BUILD_STATUS" -eq 0 ] && echo "✅ PASS" || echo "❌ FAIL")

EOF

# Final status
OVERALL_STATUS="PASS"
if [ "$CB_COUNT" -ne 1 ] || [ "$RELIABILITY_STATUS" -ne 0 ] || [ "$UTILS_STATUS" -ne 0 ] || [ "$INTELLIGENCE_STATUS" -ne 0 ] || [ "$BUILD_STATUS" -ne 0 ]; then
    OVERALL_STATUS="FAIL"
fi

cat >> "$REPORT_FILE" <<EOF

## Overall Result

**Status:** $([ "$OVERALL_STATUS" = "PASS" ] && echo "✅ PASS" || echo "❌ FAIL")

$([ "$OVERALL_STATUS" = "FAIL" ] && echo "### Issues Detected

Review logs:
- Reliability tests: /tmp/reliability-test.log
- Utils tests: /tmp/utils-test.log
- Intelligence tests: /tmp/intelligence-test.log
- Build log: /tmp/build.log
" || echo "All validation checks passed successfully.")

---
**Validator:** validate-sprint-0.4.2.sh
**Report:** $REPORT_FILE
EOF

echo ""
echo "=== Validation Complete ==="
echo "Report: $REPORT_FILE"
echo "Overall Status: $OVERALL_STATUS"

[ "$OVERALL_STATUS" = "PASS" ] && exit 0 || exit 1
