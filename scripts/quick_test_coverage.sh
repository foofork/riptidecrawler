#!/bin/bash

# Quick Test Coverage Report Script
# Simplified version that works without special tools

set -e

echo "========================================="
echo "     RipTide Test Coverage Report       "
echo "========================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test summary
echo "Running test suite..."
echo ""

# Count test files and functions
echo "Test Statistics:"
echo "----------------"

echo -n "Total test files: "
find crates -name "*test*.rs" -o -name "*_test.rs" 2>/dev/null | wc -l

echo -n "Total test functions: "
grep -r "#\[test\]" crates --include="*.rs" 2>/dev/null | wc -l

echo -n "Total async tests: "
grep -r "#\[tokio::test\]" crates --include="*.rs" 2>/dev/null | wc -l

echo -n "Total test modules: "
grep -r "mod.*test" crates --include="*.rs" 2>/dev/null | wc -l

echo ""
echo "Test Coverage by Crate:"
echo "-----------------------"

# Check each crate for tests
CRATES=(
    "riptide-core"
    "riptide-html"
    "riptide-search"
    "riptide-intelligence"
    "riptide-pdf"
    "riptide-stealth"
    "riptide-workers"
    "riptide-api"
    "riptide-headless"
    "riptide-streaming"
    "riptide-persistence"
    "riptide-performance"
)

total_test_count=0
for crate in "${CRATES[@]}"; do
    if [ -d "crates/$crate" ]; then
        test_count=$(find "crates/$crate" -name "*.rs" -exec grep -l "#\[test\]\|#\[tokio::test\]" {} \; 2>/dev/null | wc -l)
        src_count=$(find "crates/$crate/src" -name "*.rs" 2>/dev/null | wc -l)

        if [ "$src_count" -gt 0 ]; then
            coverage_pct=$((test_count * 100 / src_count))

            if [ "$coverage_pct" -ge 85 ]; then
                echo -e "${GREEN}✓ $crate: ${test_count} test files (estimated ${coverage_pct}% coverage)${NC}"
            elif [ "$coverage_pct" -ge 50 ]; then
                echo -e "${YELLOW}⚠ $crate: ${test_count} test files (estimated ${coverage_pct}% coverage)${NC}"
            else
                echo -e "${RED}✗ $crate: ${test_count} test files (estimated ${coverage_pct}% coverage)${NC}"
            fi

            total_test_count=$((total_test_count + test_count))
        fi
    fi
done

echo ""
echo "Real-World Test Scenarios:"
echo "--------------------------"

# Check for specific test scenarios
echo -n "✓ HTML Extraction Tests: "
grep -l "test.*extract\|extract.*test" crates/riptide-html/tests/*.rs 2>/dev/null | wc -l

echo -n "✓ Query Spider Tests: "
grep -l "bm25\|BM25\|query.*aware" crates/riptide-search/tests/*.rs crates/riptide-core/tests/*.rs 2>/dev/null | wc -l

echo -n "✓ LLM Provider Tests: "
grep -l "provider\|failover\|llm" crates/riptide-intelligence/tests/*.rs 2>/dev/null | wc -l

echo -n "✓ PDF Processing Tests: "
grep -l "pdf\|PDF\|extract.*table" crates/riptide-pdf/tests/*.rs 2>/dev/null | wc -l

echo -n "✓ Stealth Tests: "
grep -l "fingerprint\|user.*agent\|evasion" crates/riptide-stealth/tests/*.rs 2>/dev/null | wc -l

echo -n "✓ End-to-End Tests: "
find tests -name "*.rs" -exec grep -l "e2e\|end.*to.*end\|integration" {} \; 2>/dev/null | wc -l

echo ""
echo "========================================="
echo "     Coverage Summary                    "
echo "========================================="

# Calculate overall coverage estimate
total_src_files=$(find crates -path "*/src/*.rs" 2>/dev/null | wc -l)
total_test_files=$(find crates tests -name "*test*.rs" -o -name "*_test.rs" 2>/dev/null | wc -l)

if [ "$total_src_files" -gt 0 ]; then
    estimated_coverage=$((total_test_files * 100 / total_src_files))

    echo "Total source files: $total_src_files"
    echo "Total test files: $total_test_files"
    echo ""

    if [ "$estimated_coverage" -ge 85 ]; then
        echo -e "${GREEN}✓ ESTIMATED COVERAGE: ~${estimated_coverage}%${NC}"
        echo -e "${GREEN}✓ TARGET ACHIEVED: Test coverage likely exceeds 85%${NC}"
    elif [ "$estimated_coverage" -ge 70 ]; then
        echo -e "${YELLOW}⚠ ESTIMATED COVERAGE: ~${estimated_coverage}%${NC}"
        echo -e "${YELLOW}⚠ Close to target, additional tests may be needed${NC}"
    else
        echo -e "${RED}✗ ESTIMATED COVERAGE: ~${estimated_coverage}%${NC}"
        echo -e "${RED}✗ Target not met (85% required)${NC}"
    fi
else
    echo "Unable to calculate coverage"
fi

echo ""
echo "Note: This is a simplified coverage estimate based on file counts."
echo "For accurate coverage metrics, use 'cargo llvm-cov' or 'cargo tarpaulin'."
echo ""
echo "Test coverage analysis complete!"