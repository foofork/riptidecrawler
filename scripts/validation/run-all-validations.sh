#!/bin/bash
# Master Validation Script - Runs all Phase 0 validations

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPORT_DIR="${SCRIPT_DIR}/../../tests/validation-reports"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
MASTER_REPORT="${REPORT_DIR}/master-validation-${TIMESTAMP}.md"

echo "╔════════════════════════════════════════════════════════════╗"
echo "║  Phase 0 Cleanup - Master Validation Suite                ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# Initialize master report
cat > "$MASTER_REPORT" <<EOF
# Phase 0 Cleanup - Master Validation Report

**Date:** $(date +%Y-%m-%d\ %H:%M:%S)
**Purpose:** Comprehensive validation of all Phase 0 cleanup tasks

## Executive Summary

EOF

# Track overall status
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Function to run validation and track results
run_validation() {
    local name="$1"
    local script="$2"
    local required="$3"  # "required" or "optional"

    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Running: $name"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    if bash "$script"; then
        echo "✅ $name: PASS"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        echo "- $name: ✅ PASS" >> "$MASTER_REPORT"
    else
        echo "❌ $name: FAIL"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        echo "- $name: ❌ FAIL ($required)" >> "$MASTER_REPORT"

        if [ "$required" = "required" ]; then
            echo ""
            echo "⛔ CRITICAL FAILURE: Required validation failed!"
            echo "   Aborting remaining tests."
            return 1
        fi
    fi

    return 0
}

# Sprint 0.4 Quick Wins Validations
echo "" >> "$MASTER_REPORT"
echo "### Sprint 0.4: Quick Wins" >> "$MASTER_REPORT"
echo "" >> "$MASTER_REPORT"

run_validation "Sprint 0.4.1: Robots.txt" "$SCRIPT_DIR/validate-sprint-0.4.1.sh" "optional" || true
run_validation "Sprint 0.4.2: Circuit Breaker" "$SCRIPT_DIR/validate-sprint-0.4.2.sh" "required" || exit 1
run_validation "Sprint 0.4.3: Redis Client" "$SCRIPT_DIR/validate-sprint-0.4.3.sh" "required" || exit 1
run_validation "Sprint 0.4.4: Rate Limiter" "$SCRIPT_DIR/validate-sprint-0.4.4.sh" "required" || exit 1

# Full Workspace Validation
echo "" >> "$MASTER_REPORT"
echo "### Full Workspace Quality Gates" >> "$MASTER_REPORT"
echo "" >> "$MASTER_REPORT"

run_validation "Full Workspace" "$SCRIPT_DIR/validate-full-workspace.sh" "required" || exit 1

# Generate summary
cat >> "$MASTER_REPORT" <<EOF

## Test Results Summary

- **Total Tests:** $TOTAL_TESTS
- **Passed:** $PASSED_TESTS ($(( PASSED_TESTS * 100 / TOTAL_TESTS ))%)
- **Failed:** $FAILED_TESTS ($(( FAILED_TESTS * 100 / TOTAL_TESTS ))%)

## Final Status

EOF

if [ "$FAILED_TESTS" -eq 0 ]; then
    FINAL_STATUS="✅ ALL TESTS PASSED"
    cat >> "$MASTER_REPORT" <<EOF
**Status:** ✅ SUCCESS

All validation tests passed successfully. Phase 0 cleanup meets quality standards.
EOF
else
    FINAL_STATUS="❌ SOME TESTS FAILED"
    cat >> "$MASTER_REPORT" <<EOF
**Status:** ❌ FAILURE

$FAILED_TESTS validation test(s) failed. Review individual reports for details.

### Failed Tests
EOF

    grep "❌ FAIL" "$MASTER_REPORT" | sed 's/^/- /' >> "$MASTER_REPORT"
fi

cat >> "$MASTER_REPORT" <<EOF

## Individual Reports

All detailed validation reports are available in:
\`${REPORT_DIR}\`

---
**Validation Suite:** run-all-validations.sh
**Master Report:** $MASTER_REPORT
**Generated:** $(date +%Y-%m-%d\ %H:%M:%S)
EOF

# Print final summary
echo ""
echo "╔════════════════════════════════════════════════════════════╗"
echo "║  Validation Complete                                       ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""
echo "Results: $PASSED_TESTS/$TOTAL_TESTS passed"
echo "Status: $FINAL_STATUS"
echo ""
echo "Master Report: $MASTER_REPORT"
echo ""

# Exit with appropriate code
[ "$FAILED_TESTS" -eq 0 ] && exit 0 || exit 1
