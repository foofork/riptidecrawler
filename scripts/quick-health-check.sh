#!/bin/bash
# Quick health check without running full benchmarks

echo "🏥 Quick Health Check"
echo "===================="
echo ""

# Check if code compiles
echo "🔨 Checking build..."
if cargo check --all 2>&1 | grep -q "Finished"; then
    echo "✅ Build check passed"
else
    echo "❌ Build check failed"
fi

# Check test count
echo ""
echo "🧪 Checking tests..."
TEST_COUNT=$(cargo test --all --lib -- --list 2>/dev/null | grep -c "test$")
echo "   Total tests: $TEST_COUNT"

# Check for warnings
echo ""
echo "🔍 Checking for warnings..."
WARNINGS=$(cargo clippy --all 2>&1 | grep -c "warning:")
echo "   Clippy warnings: $WARNINGS"

echo ""
echo "✅ Quick health check complete"
