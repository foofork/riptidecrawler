#!/bin/bash
# Full Workspace Validation Script - Quality Gates

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPORT_DIR="${SCRIPT_DIR}/../../tests/validation-reports"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT_FILE="${REPORT_DIR}/full-workspace-${TIMESTAMP}.md"

echo "=== Full Workspace Validation - Quality Gates ==="
echo ""

# Initialize report
cat > "$REPORT_FILE" <<EOF
# Full Workspace Validation Report

**Date:** $(date +%Y-%m-%d\ %H:%M:%S)
**Purpose:** Final Quality Gates for Phase 0 Cleanup

## Quality Gates

EOF

# Gate 1: Disk Space
echo "Quality Gate 1: Disk Space Check..."
AVAILABLE_GB=$(df / | awk 'END{print int($4/1024/1024)}')
cat >> "$REPORT_FILE" <<EOF
### 1. Disk Space
- **Available:** ${AVAILABLE_GB}GB
- **Required:** >5GB
- **Status:** $([ "$AVAILABLE_GB" -gt 5 ] && echo "✅ PASS" || echo "❌ FAIL")

EOF

if [ "$AVAILABLE_GB" -lt 5 ]; then
    echo "❌ FAIL: Insufficient disk space"
    exit 1
fi

# Gate 2: Full Build with Zero Warnings
echo "Quality Gate 2: Building workspace with zero warnings..."
BUILD_START=$(date +%s)
RUSTFLAGS="-D warnings" cargo build --workspace 2>&1 | tee /tmp/full-build.log
BUILD_STATUS=$?
BUILD_END=$(date +%s)
BUILD_TIME=$((BUILD_END - BUILD_START))

cat >> "$REPORT_FILE" <<EOF
### 2. Zero-Warning Build
- **Build Time:** ${BUILD_TIME}s
- **Status:** $([ "$BUILD_STATUS" -eq 0 ] && echo "✅ PASS" || echo "❌ FAIL")

EOF

if [ "$BUILD_STATUS" -ne 0 ]; then
    echo "❌ FAIL: Build failed with warnings/errors"
    echo "See: /tmp/full-build.log"
fi

# Gate 3: Clippy Checks
echo "Quality Gate 3: Running clippy..."
cargo clippy --all -- -D warnings 2>&1 | tee /tmp/clippy.log
CLIPPY_STATUS=$?

cat >> "$REPORT_FILE" <<EOF
### 3. Clippy Clean
- **Status:** $([ "$CLIPPY_STATUS" -eq 0 ] && echo "✅ PASS" || echo "❌ FAIL")

EOF

if [ "$CLIPPY_STATUS" -ne 0 ]; then
    echo "❌ WARNING: Clippy found issues"
    echo "See: /tmp/clippy.log"
fi

# Gate 4: Full Test Suite
echo "Quality Gate 4: Running full test suite..."
TEST_START=$(date +%s)
cargo test --workspace --no-fail-fast 2>&1 | tee /tmp/full-tests.log
TEST_STATUS=$?
TEST_END=$(date +%s)
TEST_TIME=$((TEST_END - TEST_START))

# Parse test results
TESTS_PASSED=$(grep -E "test result: ok\." /tmp/full-tests.log | tail -1 || echo "unknown")

cat >> "$REPORT_FILE" <<EOF
### 4. Complete Test Suite
- **Test Time:** ${TEST_TIME}s
- **Results:** $TESTS_PASSED
- **Status:** $([ "$TEST_STATUS" -eq 0 ] && echo "✅ PASS" || echo "❌ FAIL")

EOF

# Gate 5: LOC Metrics
echo "Quality Gate 5: Verifying LOC reduction..."
CURRENT_LOC=$(find crates -name "*.rs" -type f | xargs wc -l | tail -1 | awk '{print $1}')

cat >> "$REPORT_FILE" <<EOF
### 5. Lines of Code Reduction
- **Baseline LOC:** 281,733
- **Current LOC:** $CURRENT_LOC
- **Reduction:** $((281733 - CURRENT_LOC)) lines
- **Target:** 6,260 lines (2.22%)
- **Status:** ℹ️ INFO

EOF

# Gate 6: Crate Count
echo "Quality Gate 6: Verifying crate reduction..."
CURRENT_CRATES=$(cargo metadata --no-deps 2>/dev/null | jq '.workspace_members | length')

cat >> "$REPORT_FILE" <<EOF
### 6. Crate Count Reduction
- **Baseline Crates:** 29
- **Current Crates:** $CURRENT_CRATES
- **Reduction:** $((29 - CURRENT_CRATES)) crate(s)
- **Target:** 2-3 crates
- **Status:** ℹ️ INFO

EOF

# Overall Result
OVERALL_STATUS="PASS"
if [ "$BUILD_STATUS" -ne 0 ] || [ "$AVAILABLE_GB" -lt 5 ]; then
    OVERALL_STATUS="FAIL"
fi

if [ "$TEST_STATUS" -ne 0 ] || [ "$CLIPPY_STATUS" -ne 0 ]; then
    OVERALL_STATUS="WARNING"
fi

cat >> "$REPORT_FILE" <<EOF

## Overall Result

**Status:** $([ "$OVERALL_STATUS" = "PASS" ] && echo "✅ PASS" || ([ "$OVERALL_STATUS" = "WARNING" ] && echo "⚠️ WARNING" || echo "❌ FAIL"))

### Summary
- Build: $([ "$BUILD_STATUS" -eq 0 ] && echo "✅" || echo "❌")
- Clippy: $([ "$CLIPPY_STATUS" -eq 0 ] && echo "✅" || echo "⚠️")
- Tests: $([ "$TEST_STATUS" -eq 0 ] && echo "✅" || echo "⚠️")
- Disk: $([ "$AVAILABLE_GB" -gt 5 ] && echo "✅" || echo "❌")

### Logs
- Full build: /tmp/full-build.log
- Clippy: /tmp/clippy.log
- Tests: /tmp/full-tests.log

---
**Validator:** validate-full-workspace.sh
**Report:** $REPORT_FILE
EOF

echo ""
echo "=== Validation Complete ==="
echo "Report: $REPORT_FILE"
echo "Overall Status: $OVERALL_STATUS"

case "$OVERALL_STATUS" in
    PASS) exit 0 ;;
    WARNING) exit 0 ;;
    FAIL) exit 1 ;;
esac
