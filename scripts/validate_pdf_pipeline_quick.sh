#!/bin/bash

# Quick PDF Pipeline Validation Script
# Lightweight validation of core PDF processing integration
# Version: 1.0.0

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
TOTAL_CHECKS=0
PASSED_CHECKS=0
FAILED_CHECKS=0

# Project root
PROJECT_ROOT="/workspaces/eventmesh"

# Log functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[✅ PASS]${NC} $1"
    ((PASSED_CHECKS++))
}

log_error() {
    echo -e "${RED}[❌ FAIL]${NC} $1"
    ((FAILED_CHECKS++))
}

check_status() {
    ((TOTAL_CHECKS++))
    if [ $1 -eq 0 ]; then
        log_success "$2"
    else
        log_error "$2"
    fi
}

echo "=============================================="
echo "    Quick PDF Pipeline Validation"
echo "=============================================="
echo ""

cd "$PROJECT_ROOT"

# 1. File Structure Check
log_info "1. Checking PDF Module Structure..."

PDF_FILES=(
    "crates/riptide-core/src/pdf/mod.rs"
    "crates/riptide-core/src/pdf/processor.rs"
    "crates/riptide-core/src/pdf/types.rs"
    "crates/riptide-core/src/pdf/errors.rs"
    "crates/riptide-api/src/routes/pdf.rs"
    "crates/riptide-api/src/handlers/pdf.rs"
)

for file in "${PDF_FILES[@]}"; do
    if [ -f "$file" ]; then
        check_status 0 "$(basename $file) exists"
    else
        check_status 1 "$(basename $file) missing"
    fi
done

# 2. Basic Compilation Check
log_info "2. Quick Compilation Check..."
if cargo check --workspace --quiet 2>/dev/null; then
    check_status 0 "Workspace compiles without errors"
else
    check_status 1 "Compilation errors found"
fi

# 3. API Integration Check
log_info "3. API Integration Check..."
if grep -q "pdf_routes" "crates/riptide-api/src/main.rs" 2>/dev/null; then
    check_status 0 "PDF routes integrated in main"
else
    check_status 1 "PDF routes not integrated"
fi

# 4. Worker Integration Check
log_info "4. Worker Integration Check..."
if grep -q -i "pdf" "crates/riptide-workers/src/processors.rs" 2>/dev/null; then
    check_status 0 "PDF processor in workers"
else
    check_status 1 "PDF processor missing from workers"
fi

# 5. Error Handling Check
log_info "5. Error Handling Check..."
UNWRAP_COUNT=$(find crates/riptide-core/src/pdf -name "*.rs" -exec grep -c "\.unwrap()" {} + 2>/dev/null | awk '{sum+=$1} END {print sum}')
if [ -z "$UNWRAP_COUNT" ] || [ "$UNWRAP_COUNT" -eq 0 ]; then
    check_status 0 "No unwrap() calls in PDF code"
else
    check_status 1 "$UNWRAP_COUNT unwrap() calls found"
fi

# 6. Test Existence Check
log_info "6. Test Coverage Check..."
if [ -f "crates/riptide-core/src/pdf/tests.rs" ]; then
    TEST_COUNT=$(grep -c "#\[test\]" "crates/riptide-core/src/pdf/tests.rs" 2>/dev/null || echo "0")
    if [ "$TEST_COUNT" -ge 3 ]; then
        check_status 0 "Test suite exists ($TEST_COUNT tests)"
    else
        check_status 1 "Insufficient tests ($TEST_COUNT found)"
    fi
else
    check_status 1 "Test file missing"
fi

echo ""
echo "=============================================="
echo "           QUICK VALIDATION REPORT"
echo "=============================================="
echo ""

echo -e "Total Checks:  ${BLUE}$TOTAL_CHECKS${NC}"
echo -e "Passed:        ${GREEN}$PASSED_CHECKS${NC}"
echo -e "Failed:        ${RED}$FAILED_CHECKS${NC}"

if [ $FAILED_CHECKS -eq 0 ]; then
    echo ""
    echo -e "${GREEN}🎉 QUICK VALIDATION PASSED! Core PDF integration looks good.${NC}"
    echo -e "${BLUE}💡 Run ./scripts/validate_pdf_pipeline.sh for comprehensive validation.${NC}"
    echo ""
    exit 0
else
    echo ""
    echo -e "${RED}❌ $FAILED_CHECKS CHECK(S) FAILED! Address issues before full validation.${NC}"
    echo ""
    exit 1
fi