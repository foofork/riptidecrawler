#!/bin/bash
# Collect all quality metrics for EventMesh

set -e

echo "============================================"
echo "EventMesh Quality Metrics Report"
echo "Generated: $(date)"
echo "============================================"
echo ""

# Build Status
echo "📦 BUILD STATUS"
echo "----------------------------------------"
if cargo build --all > /dev/null 2>&1; then
    echo "✅ Build: SUCCESS"
else
    echo "❌ Build: FAILED"
    cargo build --all 2>&1 | grep "error\[E[0-9]*\]" | head -5
fi
echo ""

# Test Status
echo "🧪 TEST STATUS"
echo "----------------------------------------"
if cargo test --all --no-fail-fast > /tmp/test_results.txt 2>&1; then
    echo "✅ Tests: ALL PASSING"
    grep "test result:" /tmp/test_results.txt
else
    echo "❌ Tests: FAILURES DETECTED"
    grep "test result:" /tmp/test_results.txt
    echo ""
    echo "Failed tests:"
    grep "FAILED" /tmp/test_results.txt | head -10
fi
echo ""

# Test Count
echo "📊 TEST INVENTORY"
echo "----------------------------------------"
echo "Test files: $(find . -name "*test*.rs" -o -path "*/tests/*.rs" | wc -l)"
echo "Unit tests: $(grep -r "#\[test\]" --include="*.rs" crates/ | wc -l)"
echo "Async tests: $(grep -r "#\[tokio::test\]" --include="*.rs" crates/ | wc -l)"
echo ""

# Coverage (if tarpaulin installed)
echo "📈 CODE COVERAGE"
echo "----------------------------------------"
if command -v cargo-tarpaulin &> /dev/null; then
    cargo tarpaulin --all --out Stdout --skip-clean 2>/dev/null | grep -E "Coverage|%" | head -5
else
    echo "⚠️  cargo-tarpaulin not installed"
    echo "   Install: cargo install cargo-tarpaulin"
fi
echo ""

# Clippy Warnings
echo "⚠️  CLIPPY ANALYSIS"
echo "----------------------------------------"
warning_count=$(cargo clippy --all -- -W clippy::all 2>&1 | grep -c "warning:" || true)
echo "Total warnings: $warning_count"
if [ "$warning_count" -lt 50 ]; then
    echo "✅ Below target (<50)"
elif [ "$warning_count" -lt 100 ]; then
    echo "⚠️  Above target (50-100)"
else
    echo "❌ Too many warnings (>100)"
fi
echo ""

# Code Statistics
echo "📝 CODE STATISTICS"
echo "----------------------------------------"
echo "Total Rust files: $(find crates/ -name "*.rs" | wc -l)"
echo "Total lines of code: $(find crates/ -name "*.rs" | xargs wc -l | tail -1 | awk '{print $1}')"
echo ""

# Build Time
echo "⏱️  BUILD PERFORMANCE"
echo "----------------------------------------"
echo "Measuring build time..."
time_output=$(time cargo build --all 2>&1 | grep "real")
echo "Build time: $time_output"
echo ""

# Summary
echo "============================================"
echo "SUMMARY"
echo "============================================"
if [ "$warning_count" -lt 50 ] && cargo test --all > /dev/null 2>&1; then
    echo "✅ Overall Status: HEALTHY"
else
    echo "⚠️  Overall Status: NEEDS ATTENTION"
fi
echo ""
echo "Next steps:"
echo "  1. Review failed tests (if any)"
echo "  2. Address clippy warnings"
echo "  3. Maintain coverage >90%"
echo "  4. Optimize slow tests"
echo "============================================"
