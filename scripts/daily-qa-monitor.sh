#!/bin/bash
# Daily QA monitoring script for Phase 1 Week 2
# Tracks: test pass rate, build status, coverage, performance
set -e

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT_DIR="./qa-reports"
DAILY_REPORT="$REPORT_DIR/daily-report-$TIMESTAMP.md"

mkdir -p "$REPORT_DIR"

echo "🔍 RipTide Daily QA Monitor"
echo "==========================="
echo "Timestamp: $(date)"
echo ""

# Initialize report
cat > "$DAILY_REPORT" << EOF
# Daily QA Report - $(date +%Y-%m-%d)
**Generated:** $(date)
**Session:** swarm_1760709536951_i98hegexl

---

EOF

# 1. Test Monitoring (15 min target)
echo "📋 Running full test suite..."
TEST_START=$(date +%s)

if cargo test --all --lib --no-fail-fast 2>&1 | tee "$REPORT_DIR/test-output-$TIMESTAMP.log"; then
    TEST_STATUS="✅ PASS"
    TEST_PASSING=$(grep -oP '\d+ passed' "$REPORT_DIR/test-output-$TIMESTAMP.log" | grep -oP '\d+' | head -1 || echo "0")
    TEST_TOTAL=$(grep -oP 'test result:.*\d+ passed' "$REPORT_DIR/test-output-$TIMESTAMP.log" || echo "N/A")
else
    TEST_STATUS="❌ FAIL"
    TEST_PASSING="N/A"
    TEST_TOTAL="FAILED"
fi

TEST_END=$(date +%s)
TEST_DURATION=$((TEST_END - TEST_START))

echo "$TEST_STATUS - Tests completed in ${TEST_DURATION}s"

# Append to report
cat >> "$DAILY_REPORT" << EOF
## Test Suite Status

**Status:** $TEST_STATUS
**Duration:** ${TEST_DURATION}s
**Result:** $TEST_TOTAL

EOF

# 2. Build Monitoring (15 min target)
echo ""
echo "🔨 Checking build status..."
BUILD_START=$(date +%s)

if cargo build --all 2>&1 | tee "$REPORT_DIR/build-output-$TIMESTAMP.log"; then
    BUILD_STATUS="✅ PASS"
    BUILD_WARNINGS=$(grep -c "warning:" "$REPORT_DIR/build-output-$TIMESTAMP.log" || echo "0")
    BUILD_ERRORS=$(grep -c "error:" "$REPORT_DIR/build-output-$TIMESTAMP.log" || echo "0")
else
    BUILD_STATUS="❌ FAIL"
    BUILD_WARNINGS="N/A"
    BUILD_ERRORS="N/A"
fi

BUILD_END=$(date +%s)
BUILD_DURATION=$((BUILD_END - BUILD_START))

echo "$BUILD_STATUS - Build completed in ${BUILD_DURATION}s"
echo "Warnings: $BUILD_WARNINGS, Errors: $BUILD_ERRORS"

cat >> "$DAILY_REPORT" << EOF
## Build Status

**Status:** $BUILD_STATUS
**Duration:** ${BUILD_DURATION}s
**Warnings:** $BUILD_WARNINGS
**Errors:** $BUILD_ERRORS

EOF

# 3. Coverage Check (if baseline exists)
echo ""
echo "📊 Checking coverage..."

if [ -f "./coverage/baseline-coverage.txt" ]; then
    BASELINE_COVERAGE=$(cat ./coverage/baseline-coverage.txt)
    echo "Baseline coverage: $BASELINE_COVERAGE"

    cat >> "$DAILY_REPORT" << EOF
## Coverage

**Baseline:** $BASELINE_COVERAGE
**Status:** Monitoring (baseline established)

EOF
else
    echo "⚠️ No baseline coverage found - run ./scripts/measure-coverage.sh to establish baseline"

    cat >> "$DAILY_REPORT" << EOF
## Coverage

**Status:** ⚠️ No baseline - needs establishment
**Action Required:** Run \`./scripts/measure-coverage.sh\`

EOF
fi

# 4. Performance Regression Check
echo ""
echo "⚡ Checking for performance regressions..."

if [ -d "./target/criterion" ]; then
    BENCH_STATUS="✅ Monitoring"
    # Check if we have baseline
    if [ -f "./target/criterion/baseline-marker.txt" ]; then
        echo "Performance baselines are being tracked"
    else
        echo "⚠️ No performance baseline - benchmarks need to be run with --save-baseline"
    fi
else
    BENCH_STATUS="⚠️ Not configured"
    echo "No benchmark data found"
fi

cat >> "$DAILY_REPORT" << EOF
## Performance

**Status:** $BENCH_STATUS
**Note:** Run benchmarks with \`cargo bench -- --save-baseline today\` to track

EOF

# 5. Summary
echo ""
echo "========================================"
echo "📊 Daily QA Summary"
echo "========================================"

ALERT_COUNT=0
if [ "$TEST_STATUS" != "✅ PASS" ]; then ALERT_COUNT=$((ALERT_COUNT + 1)); fi
if [ "$BUILD_STATUS" != "✅ PASS" ]; then ALERT_COUNT=$((ALERT_COUNT + 1)); fi
if [ "$BUILD_ERRORS" != "0" ]; then ALERT_COUNT=$((ALERT_COUNT + 1)); fi

echo "Tests: $TEST_STATUS"
echo "Build: $BUILD_STATUS"
echo "Alerts: $ALERT_COUNT"
echo ""
echo "Report saved: $DAILY_REPORT"

cat >> "$DAILY_REPORT" << EOF

---

## Summary

- **Tests:** $TEST_STATUS (${TEST_DURATION}s)
- **Build:** $BUILD_STATUS (${BUILD_DURATION}s, $BUILD_WARNINGS warnings, $BUILD_ERRORS errors)
- **Alerts:** $ALERT_COUNT

## Recommendations

EOF

if [ $ALERT_COUNT -gt 0 ]; then
    echo "⚠️ ALERTS DETECTED - Review required"

    cat >> "$DAILY_REPORT" << EOF
- ⚠️ **Action Required:** $ALERT_COUNT issue(s) detected
- Review test failures in \`$REPORT_DIR/test-output-$TIMESTAMP.log\`
- Review build errors in \`$REPORT_DIR/build-output-$TIMESTAMP.log\`
EOF
else
    echo "✅ All systems operational"

    cat >> "$DAILY_REPORT" << EOF
- ✅ All systems operational
- Continue monitoring
EOF
fi

cat >> "$DAILY_REPORT" << EOF

---

**Next Check:** $(date -d "+1 day" +%Y-%m-%d)
EOF

# Coordination hooks
echo ""
echo "📡 Updating swarm memory..."

# Save status to memory
npx claude-flow@alpha hooks post-edit \
    --file "$DAILY_REPORT" \
    --memory-key "swarm/qa/daily-status" 2>/dev/null || echo "Warning: Hook failed (non-blocking)"

# Notify completion
npx claude-flow@alpha hooks notify \
    --message "Daily QA monitoring complete: $TEST_STATUS, $BUILD_STATUS, $ALERT_COUNT alerts" 2>/dev/null || echo "Warning: Hook failed (non-blocking)"

echo ""
echo "✅ Daily QA monitoring complete!"
echo "Report: $DAILY_REPORT"
