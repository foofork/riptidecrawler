#!/bin/bash

# Simple PDF Pipeline Validation Script
# Basic validation of PDF processing integration

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

echo "=============================================="
echo "    Simple PDF Pipeline Validation"
echo "=============================================="
echo ""

cd "/workspaces/eventmesh"

PASSED=0
FAILED=0

check_file() {
    if [ -f "$1" ]; then
        echo -e "${GREEN}[✅ PASS]${NC} $2"
        PASSED=$((PASSED + 1))
    else
        echo -e "${RED}[❌ FAIL]${NC} $2 (file missing: $1)"
        FAILED=$((FAILED + 1))
    fi
}

check_content() {
    if grep -q "$2" "$1" 2>/dev/null; then
        echo -e "${GREEN}[✅ PASS]${NC} $3"
        PASSED=$((PASSED + 1))
    else
        echo -e "${RED}[❌ FAIL]${NC} $3"
        FAILED=$((FAILED + 1))
    fi
}

echo -e "${BLUE}[INFO]${NC} 1. Checking PDF Module Files..."

check_file "crates/riptide-core/src/pdf/mod.rs" "PDF module exists"
check_file "crates/riptide-core/src/pdf/processor.rs" "PDF processor exists"
check_file "crates/riptide-core/src/pdf/types.rs" "PDF types exists"
check_file "crates/riptide-core/src/pdf/errors.rs" "PDF errors exists"
check_file "crates/riptide-core/src/pdf/config.rs" "PDF config exists"
check_file "crates/riptide-core/src/pdf/metrics.rs" "PDF metrics exists"

echo ""
echo -e "${BLUE}[INFO]${NC} 2. Checking API Integration..."

check_file "crates/riptide-api/src/routes/pdf.rs" "PDF routes exist"
check_file "crates/riptide-api/src/handlers/pdf.rs" "PDF handlers exist"
check_content "crates/riptide-api/src/routes/mod.rs" "pub mod pdf" "PDF routes declared"
check_content "crates/riptide-api/src/routes/mod.rs" "pdf_routes" "PDF routes exported"

echo ""
echo -e "${BLUE}[INFO]${NC} 3. Checking Worker Integration..."

check_content "crates/riptide-workers/src/processors.rs" "pdf\|PDF" "PDF in workers"

echo ""
echo -e "${BLUE}[INFO]${NC} 4. Checking Tests..."

check_file "crates/riptide-core/src/pdf/tests.rs" "PDF tests exist"

echo ""
echo -e "${BLUE}[INFO]${NC} 5. Checking Code Quality..."

# Check for unwrap calls
UNWRAP_FILES=$(find crates/riptide-core/src/pdf -name "*.rs" -exec grep -l "\.unwrap()" {} + 2>/dev/null | wc -l)
if [ "$UNWRAP_FILES" -eq 0 ]; then
    echo -e "${GREEN}[✅ PASS]${NC} No unwrap() calls found in PDF code"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}[❌ FAIL]${NC} Found unwrap() calls in $UNWRAP_FILES PDF files"
    FAILED=$((FAILED + 1))
fi

echo ""
echo "=============================================="
echo "           VALIDATION SUMMARY"
echo "=============================================="
echo ""

TOTAL=$((PASSED + FAILED))
echo -e "Total Checks: ${BLUE}$TOTAL${NC}"
echo -e "Passed:       ${GREEN}$PASSED${NC}"
echo -e "Failed:       ${RED}$FAILED${NC}"

if [ $FAILED -eq 0 ]; then
    echo ""
    echo -e "${GREEN}🎉 ALL BASIC CHECKS PASSED!${NC}"
    echo -e "${BLUE}💡 PDF pipeline structure is in place.${NC}"
    echo ""
    exit 0
else
    echo ""
    echo -e "${RED}❌ $FAILED CHECK(S) FAILED!${NC}"
    echo ""
    exit 1
fi