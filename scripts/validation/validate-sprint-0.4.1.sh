#!/bin/bash
# Validation Script for Sprint 0.4.1: Robots.txt Consolidation

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPORT_DIR="${SCRIPT_DIR}/../../tests/validation-reports"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT_FILE="${REPORT_DIR}/sprint-0.4.1-${TIMESTAMP}.md"

echo "=== Sprint 0.4.1 Validation: Robots.txt Consolidation ==="
echo ""

# Initialize report
cat > "$REPORT_FILE" <<EOF
# Sprint 0.4.1 Validation Report

**Task:** Robots.txt Consolidation
**Date:** $(date +%Y-%m-%d\ %H:%M:%S)
**Status:** IN PROGRESS

## Validation Steps

EOF

echo "Step 1: Verify no duplicate robots.txt files..."
ROBOTS_COUNT=$(find crates -name "robots*.rs" -type f | wc -l)
echo "Found $ROBOTS_COUNT robots.txt file(s)"

cat >> "$REPORT_FILE" <<EOF
### 1. File Count Verification
- **Expected:** 1 file (in riptide-fetch)
- **Actual:** $ROBOTS_COUNT file(s)
- **Status:** $([ "$ROBOTS_COUNT" -eq 1 ] && echo "✅ PASS" || echo "❌ FAIL")

EOF

if [ "$ROBOTS_COUNT" -ne 1 ]; then
    echo "❌ FAIL: Expected 1 robots.txt file, found $ROBOTS_COUNT"
    find crates -name "robots*.rs" -type f >> "$REPORT_FILE"
    exit 1
fi

echo "Step 2: Verify file is in correct location..."
ROBOTS_FILE=$(find crates -name "robots*.rs" -type f)
EXPECTED_LOCATION="crates/riptide-fetch/src/robots.rs"

cat >> "$REPORT_FILE" <<EOF
### 2. Location Verification
- **Expected:** $EXPECTED_LOCATION
- **Actual:** $ROBOTS_FILE
- **Status:** $([ "$ROBOTS_FILE" = "$EXPECTED_LOCATION" ] && echo "✅ PASS" || echo "❌ FAIL")

EOF

if [ "$ROBOTS_FILE" != "$EXPECTED_LOCATION" ]; then
    echo "❌ FAIL: robots.rs not in expected location"
    exit 1
fi

echo "Step 3: Test affected crates..."
cargo test -p riptide-spider --no-fail-fast 2>&1 | tee -a /tmp/spider-test.log
SPIDER_STATUS=$?

cargo test -p riptide-fetch --no-fail-fast 2>&1 | tee -a /tmp/fetch-test.log
FETCH_STATUS=$?

cat >> "$REPORT_FILE" <<EOF
### 3. Affected Crate Tests
- **riptide-spider:** $([ "$SPIDER_STATUS" -eq 0 ] && echo "✅ PASS" || echo "❌ FAIL")
- **riptide-fetch:** $([ "$FETCH_STATUS" -eq 0 ] && echo "✅ PASS" || echo "❌ FAIL")

EOF

echo "Step 4: Check for old import patterns..."
OLD_IMPORTS=$(rg "crate::robots" crates/riptide-spider/ --type rust 2>/dev/null | wc -l)

cat >> "$REPORT_FILE" <<EOF
### 4. Import Pattern Verification
- **Old crate::robots imports:** $OLD_IMPORTS
- **Status:** $([ "$OLD_IMPORTS" -eq 0 ] && echo "✅ PASS" || echo "⚠️ WARNING")

EOF

echo "Step 5: Build with zero warnings..."
RUSTFLAGS="-D warnings" cargo build -p riptide-spider -p riptide-fetch 2>&1 | tee -a /tmp/build.log
BUILD_STATUS=$?

cat >> "$REPORT_FILE" <<EOF
### 5. Zero-Warning Build
- **Build Status:** $([ "$BUILD_STATUS" -eq 0 ] && echo "✅ PASS" || echo "❌ FAIL")

EOF

# Final status
OVERALL_STATUS="PASS"
if [ "$ROBOTS_COUNT" -ne 1 ] || [ "$SPIDER_STATUS" -ne 0 ] || [ "$FETCH_STATUS" -ne 0 ] || [ "$BUILD_STATUS" -ne 0 ]; then
    OVERALL_STATUS="FAIL"
fi

cat >> "$REPORT_FILE" <<EOF

## Overall Result

**Status:** $([ "$OVERALL_STATUS" = "PASS" ] && echo "✅ PASS" || echo "❌ FAIL")

$([ "$OVERALL_STATUS" = "FAIL" ] && echo "### Issues Detected

Review logs:
- Spider tests: /tmp/spider-test.log
- Fetch tests: /tmp/fetch-test.log
- Build log: /tmp/build.log
" || echo "All validation checks passed successfully.")

---
**Validator:** validate-sprint-0.4.1.sh
**Report:** $REPORT_FILE
EOF

echo ""
echo "=== Validation Complete ==="
echo "Report: $REPORT_FILE"
echo "Overall Status: $OVERALL_STATUS"

[ "$OVERALL_STATUS" = "PASS" ] && exit 0 || exit 1
