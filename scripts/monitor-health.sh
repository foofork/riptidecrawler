#!/bin/bash
# Continuous health monitoring for CI/CD

METRICS_FILE="./metrics/health-$(date +%Y%m%d-%H%M%S).json"
mkdir -p ./metrics

echo "🏥 Running health check..."
echo ""

# Collect build metrics
echo "🔨 Checking build..."
BUILD_OUTPUT=$(cargo build --all 2>&1)
BUILD_ERRORS=$(echo "$BUILD_OUTPUT" | grep -c "error:")
BUILD_WARNINGS=$(echo "$BUILD_OUTPUT" | grep -c "warning:")

# Collect test metrics
echo "🧪 Running tests..."
TEST_OUTPUT=$(cargo test --all --lib 2>&1)
TESTS_TOTAL=$(echo "$TEST_OUTPUT" | grep "test result" | grep -oP '\d+ passed' | grep -oP '\d+' | head -1)
TESTS_PASSED=${TESTS_TOTAL:-0}
TESTS_FAILED=$(echo "$TEST_OUTPUT" | grep "test result" | grep -oP '\d+ failed' | grep -oP '\d+' | head -1)
TESTS_FAILED=${TESTS_FAILED:-0}

# Collect clippy metrics
echo "🔍 Running clippy..."
CLIPPY_OUTPUT=$(cargo clippy --all 2>&1)
CLIPPY_WARNINGS=$(echo "$CLIPPY_OUTPUT" | grep -c "warning:")
CLIPPY_ERRORS=$(echo "$CLIPPY_OUTPUT" | grep -c "error:")

# Calculate health status
if [ $BUILD_ERRORS -gt 0 ] || [ $CLIPPY_ERRORS -gt 0 ]; then
    STATUS="unhealthy"
elif [ $TESTS_FAILED -gt 0 ]; then
    STATUS="degraded"
elif [ $BUILD_WARNINGS -gt 10 ] || [ $CLIPPY_WARNINGS -gt 10 ]; then
    STATUS="warning"
else
    STATUS="healthy"
fi

# Create metrics JSON
cat > "$METRICS_FILE" <<EOF
{
  "timestamp": "$(date -Iseconds)",
  "build": {
    "errors": $BUILD_ERRORS,
    "warnings": $BUILD_WARNINGS
  },
  "tests": {
    "passed": $TESTS_PASSED,
    "failed": $TESTS_FAILED,
    "total": $((TESTS_PASSED + TESTS_FAILED))
  },
  "clippy": {
    "warnings": $CLIPPY_WARNINGS,
    "errors": $CLIPPY_ERRORS
  },
  "status": "$STATUS"
}
EOF

echo ""
echo "📊 Health Metrics:"
echo "=================="
cat "$METRICS_FILE" | jq . 2>/dev/null || cat "$METRICS_FILE"

echo ""
echo "📁 Metrics saved to: $METRICS_FILE"

# Alert on issues
if [ "$STATUS" = "unhealthy" ]; then
    echo ""
    echo "🚨 SYSTEM UNHEALTHY"
    echo "   Build errors: $BUILD_ERRORS"
    echo "   Clippy errors: $CLIPPY_ERRORS"
    exit 1
elif [ "$STATUS" = "degraded" ]; then
    echo ""
    echo "⚠️  SYSTEM DEGRADED"
    echo "   Tests failed: $TESTS_FAILED"
    exit 1
elif [ "$STATUS" = "warning" ]; then
    echo ""
    echo "⚠️  WARNINGS DETECTED"
    echo "   Build warnings: $BUILD_WARNINGS"
    echo "   Clippy warnings: $CLIPPY_WARNINGS"
fi

echo ""
echo "✅ System $STATUS"
