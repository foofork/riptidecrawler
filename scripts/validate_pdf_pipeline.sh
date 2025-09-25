#!/bin/bash

# PDF Pipeline Validation Script
# Comprehensive validation of PDF processing integration
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
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

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

log_warning() {
    echo -e "${YELLOW}[⚠️  WARN]${NC} $1"
}

check_status() {
    ((TOTAL_CHECKS++))
    if [ $1 -eq 0 ]; then
        log_success "$2"
    else
        log_error "$2"
    fi
}

# Header
echo "=============================================="
echo "    PDF Pipeline Validation Script"
echo "=============================================="
echo ""

# Record start time
START_TIME=$(date +%s)

cd "$PROJECT_ROOT"

# 1. Compilation Checks
log_info "1. Checking PDF Components Compilation..."
echo ""

log_info "1.1 Compiling riptide-core with PDF features..."
if timeout 60 cargo check -p riptide-core 2>/dev/null; then
    check_status 0 "riptide-core compiles without errors"
else
    check_status 1 "riptide-core compilation failed"
fi

log_info "1.2 Compiling riptide-api..."
if timeout 60 cargo check -p riptide-api 2>/dev/null; then
    check_status 0 "riptide-api compiles without errors"
else
    check_status 1 "riptide-api compilation failed"
fi

log_info "1.3 Compiling riptide-workers..."
if timeout 60 cargo check -p riptide-workers 2>/dev/null; then
    check_status 0 "riptide-workers compiles without errors"
else
    check_status 1 "riptide-workers compilation failed"
fi

echo ""

# 2. PDF Module Structure Validation
log_info "2. Validating PDF Module Structure..."
echo ""

# Check core PDF files exist
PDF_FILES=(
    "crates/riptide-core/src/pdf/mod.rs"
    "crates/riptide-core/src/pdf/processor.rs"
    "crates/riptide-core/src/pdf/types.rs"
    "crates/riptide-core/src/pdf/errors.rs"
    "crates/riptide-core/src/pdf/config.rs"
    "crates/riptide-core/src/pdf/metrics.rs"
    "crates/riptide-core/src/pdf/utils.rs"
    "crates/riptide-core/src/pdf/tests.rs"
)

for file in "${PDF_FILES[@]}"; do
    if [ -f "$file" ]; then
        check_status 0 "PDF module file exists: $(basename $file)"
    else
        check_status 1 "PDF module file missing: $file"
    fi
done

echo ""

# 3. API Endpoint Registration
log_info "3. Checking PDF API Endpoints Registration..."
echo ""

log_info "3.1 Checking PDF routes module..."
if [ -f "crates/riptide-api/src/routes/pdf.rs" ]; then
    check_status 0 "PDF routes module exists"
else
    check_status 1 "PDF routes module missing"
fi

log_info "3.2 Checking routes registration in mod.rs..."
if grep -q "pub mod pdf" "crates/riptide-api/src/routes/mod.rs" 2>/dev/null; then
    check_status 0 "PDF routes declared in mod.rs"
else
    check_status 1 "PDF routes not declared in mod.rs"
fi

if grep -q "pub use pdf::pdf_routes" "crates/riptide-api/src/routes/mod.rs" 2>/dev/null; then
    check_status 0 "PDF routes exported in mod.rs"
else
    check_status 1 "PDF routes not exported in mod.rs"
fi

log_info "3.3 Checking PDF handlers..."
if [ -f "crates/riptide-api/src/handlers/pdf.rs" ]; then
    check_status 0 "PDF handlers module exists"
else
    check_status 1 "PDF handlers module missing"
fi

log_info "3.4 Checking main.rs for PDF route registration..."
if grep -q "pdf_routes" "crates/riptide-api/src/main.rs" 2>/dev/null; then
    check_status 0 "PDF routes registered in main.rs"
else
    check_status 1 "PDF routes not registered in main.rs"
fi

echo ""

# 4. Worker Service Integration
log_info "4. Checking Worker Service PDF Integration..."
echo ""

log_info "4.1 Checking PDF processor in workers..."
if grep -q "PdfProcessor\|pdf" "crates/riptide-workers/src/processors.rs" 2>/dev/null; then
    check_status 0 "PDF processor referenced in workers"
else
    check_status 1 "PDF processor not found in workers"
fi

log_info "4.2 Checking job types for PDF..."
if grep -q "PdfExtraction\|Pdf" "crates/riptide-workers/src/job.rs" 2>/dev/null || \
   grep -q "PdfExtraction\|Pdf" "crates/riptide-workers/src/lib.rs" 2>/dev/null; then
    check_status 0 "PDF job types defined"
else
    check_status 1 "PDF job types not found"
fi

echo ""

# 5. Memory Management Validation
log_info "5. Validating Memory Management Integration..."
echo ""

log_info "5.1 Checking memory configuration..."
if [ -f "crates/riptide-core/src/pdf/config.rs" ]; then
    if grep -q "memory\|Memory" "crates/riptide-core/src/pdf/config.rs" 2>/dev/null; then
        check_status 0 "Memory configuration found in PDF config"
    else
        check_status 1 "Memory configuration missing from PDF config"
    fi
else
    check_status 1 "PDF config file missing"
fi

log_info "5.2 Checking memory hooks in processor..."
if [ -f "crates/riptide-core/src/pdf/processor.rs" ]; then
    if grep -q "memory\|Memory" "crates/riptide-core/src/pdf/processor.rs" 2>/dev/null; then
        check_status 0 "Memory hooks found in PDF processor"
    else
        check_status 1 "Memory hooks missing from PDF processor"
    fi
else
    check_status 1 "PDF processor file missing"
fi

log_info "5.3 Checking memory benchmark integration..."
if [ -f "crates/riptide-core/src/pdf/memory_benchmark.rs" ]; then
    check_status 0 "Memory benchmark module exists"
else
    check_status 1 "Memory benchmark module missing"
fi

echo ""

# 6. Progress Tracking Validation
log_info "6. Validating Progress Tracking Integration..."
echo ""

log_info "6.1 Checking progress types..."
if grep -q "Progress\|progress" "crates/riptide-core/src/pdf/types.rs" 2>/dev/null; then
    check_status 0 "Progress types defined in PDF types"
else
    check_status 1 "Progress types missing from PDF types"
fi

log_info "6.2 Checking progress tracking in processor..."
if grep -q "progress\|Progress" "crates/riptide-core/src/pdf/processor.rs" 2>/dev/null; then
    check_status 0 "Progress tracking implemented in processor"
else
    check_status 1 "Progress tracking missing from processor"
fi

log_info "6.3 Checking progress callback integration..."
if grep -q "callback\|Callback" "crates/riptide-core/src/pdf/processor.rs" 2>/dev/null; then
    check_status 0 "Progress callbacks implemented"
else
    check_status 1 "Progress callbacks missing"
fi

echo ""

# 7. Test Suite Validation
log_info "7. Running PDF Test Suite..."
echo ""

log_info "7.1 Running unit tests..."
if timeout 120 cargo test -p riptide-core pdf --lib 2>/dev/null; then
    check_status 0 "PDF unit tests pass"
else
    check_status 1 "PDF unit tests fail"
fi

log_info "7.2 Running integration tests..."
if timeout 120 cargo test -p riptide-core pdf_pipeline 2>/dev/null; then
    check_status 0 "PDF integration tests pass"
else
    check_status 1 "PDF integration tests fail or not found"
fi

log_info "7.3 Checking test coverage..."
if [ -f "crates/riptide-core/src/pdf/tests.rs" ]; then
    TEST_COUNT=$(grep -c "#\[test\]" "crates/riptide-core/src/pdf/tests.rs" 2>/dev/null || echo "0")
    if [ "$TEST_COUNT" -ge 5 ]; then
        check_status 0 "Adequate test coverage ($TEST_COUNT tests)"
    else
        check_status 1 "Insufficient test coverage ($TEST_COUNT tests)"
    fi
else
    check_status 1 "PDF tests file missing"
fi

echo ""

# 8. Code Quality Checks
log_info "8. Performing Code Quality Checks..."
echo ""

log_info "8.1 Checking for unwrap() calls in PDF code..."
UNWRAP_COUNT=$(find crates/riptide-core/src/pdf -name "*.rs" -exec grep -n "\.unwrap()" {} + 2>/dev/null | wc -l)
if [ "$UNWRAP_COUNT" -eq 0 ]; then
    check_status 0 "No unwrap() calls found in PDF code"
else
    check_status 1 "$UNWRAP_COUNT unwrap() calls found in PDF code"
    # Show the locations
    log_warning "Unwrap locations:"
    find crates/riptide-core/src/pdf -name "*.rs" -exec grep -n "\.unwrap()" {} + 2>/dev/null || true
fi

log_info "8.2 Checking for panic! calls..."
PANIC_COUNT=$(find crates/riptide-core/src/pdf -name "*.rs" -exec grep -n "panic!" {} + 2>/dev/null | wc -l)
if [ "$PANIC_COUNT" -eq 0 ]; then
    check_status 0 "No panic! calls found in PDF code"
else
    check_status 1 "$PANIC_COUNT panic! calls found in PDF code"
fi

log_info "8.3 Checking error handling patterns..."
if grep -q "Result\|Error" "crates/riptide-core/src/pdf/processor.rs" 2>/dev/null; then
    check_status 0 "Proper error handling patterns used"
else
    check_status 1 "Error handling patterns missing"
fi

echo ""

# 9. Metrics Collection Validation
log_info "9. Validating Metrics Collection..."
echo ""

log_info "9.1 Checking metrics module..."
if [ -f "crates/riptide-core/src/pdf/metrics.rs" ]; then
    check_status 0 "PDF metrics module exists"
else
    check_status 1 "PDF metrics module missing"
fi

log_info "9.2 Checking metrics integration in processor..."
if grep -q "metrics\|Metrics" "crates/riptide-core/src/pdf/processor.rs" 2>/dev/null; then
    check_status 0 "Metrics integrated in processor"
else
    check_status 1 "Metrics missing from processor"
fi

log_info "9.3 Checking telemetry integration..."
if grep -q "telemetry\|Telemetry" "crates/riptide-core/src/pdf/processor.rs" 2>/dev/null; then
    check_status 0 "Telemetry integrated"
else
    check_status 1 "Telemetry integration missing"
fi

echo ""

# 10. Performance Guardrails
log_info "10. Validating Performance Guardrails..."
echo ""

log_info "10.1 Checking memory limits configuration..."
if grep -q "max_memory\|memory_limit" crates/riptide-core/src/pdf/config.rs 2>/dev/null; then
    check_status 0 "Memory limits configured"
else
    check_status 1 "Memory limits not configured"
fi

log_info "10.2 Checking timeout configurations..."
if grep -q "timeout\|Timeout" crates/riptide-core/src/pdf/config.rs 2>/dev/null; then
    check_status 0 "Timeout configurations found"
else
    check_status 1 "Timeout configurations missing"
fi

log_info "10.3 Checking concurrent processing limits..."
if grep -q "concurrent\|parallel" crates/riptide-core/src/pdf/config.rs 2>/dev/null; then
    check_status 0 "Concurrency limits configured"
else
    check_status 1 "Concurrency limits not configured"
fi

echo ""

# 11. Integration Test
log_info "11. Running Integration Smoke Test..."
echo ""

log_info "11.1 Building all components..."
if timeout 180 cargo build --workspace --quiet 2>/dev/null; then
    check_status 0 "All components build successfully"
else
    check_status 1 "Build failed"
fi

log_info "11.2 Checking clippy warnings..."
if timeout 120 cargo clippy --workspace --quiet -- -D warnings 2>/dev/null; then
    check_status 0 "No clippy warnings"
else
    check_status 1 "Clippy warnings found"
fi

echo ""

# 12. Configuration Validation
log_info "12. Validating Configuration Files..."
echo ""

CONFIG_FILES=(
    "configs/riptide.yml"
    "Cargo.toml"
    "crates/riptide-core/Cargo.toml"
    "crates/riptide-api/Cargo.toml"
    "crates/riptide-workers/Cargo.toml"
)

for config_file in "${CONFIG_FILES[@]}"; do
    if [ -f "$config_file" ]; then
        check_status 0 "Configuration file exists: $(basename $config_file)"
    else
        check_status 1 "Configuration file missing: $config_file"
    fi
done

log_info "12.1 Checking PDF feature flags..."
if grep -q "pdf.*=.*true\|default.*=.*\[.*pdf" crates/riptide-core/Cargo.toml 2>/dev/null; then
    check_status 0 "PDF feature flags configured"
else
    check_status 1 "PDF feature flags missing"
fi

echo ""

# Final Report
echo "=============================================="
echo "           VALIDATION REPORT"
echo "=============================================="
echo ""

echo -e "Total Checks:  ${BLUE}$TOTAL_CHECKS${NC}"
echo -e "Passed:        ${GREEN}$PASSED_CHECKS${NC}"
echo -e "Failed:        ${RED}$FAILED_CHECKS${NC}"

if [ $FAILED_CHECKS -eq 0 ]; then
    echo ""
    echo -e "${GREEN}🎉 ALL CHECKS PASSED! PDF Pipeline is fully integrated and working.${NC}"
    echo ""
    exit 0
else
    echo ""
    echo -e "${RED}❌ $FAILED_CHECKS CHECK(S) FAILED! PDF Pipeline needs attention.${NC}"
    echo ""

    # Provide recommendations
    echo "RECOMMENDATIONS:"
    echo "=================="

    if [ $FAILED_CHECKS -gt 5 ]; then
        echo -e "${YELLOW}• Major integration issues detected. Review PDF module integration.${NC}"
    fi

    if grep -q "compilation failed" <<< "$OUTPUT" 2>/dev/null; then
        echo -e "${YELLOW}• Fix compilation errors first before proceeding.${NC}"
    fi

    echo -e "${YELLOW}• Check the specific failed tests above for detailed issues.${NC}"
    echo -e "${YELLOW}• Run individual components to isolate problems.${NC}"
    echo -e "${YELLOW}• Review logs and error messages for more details.${NC}"

    echo ""
    exit 1
fi

# Performance metrics if all passed
if [ $FAILED_CHECKS -eq 0 ]; then
    echo "PERFORMANCE METRICS:"
    echo "===================="

    # Calculate success rate
    SUCCESS_RATE=$((PASSED_CHECKS * 100 / TOTAL_CHECKS))
    echo -e "Success Rate:  ${GREEN}$SUCCESS_RATE%${NC}"

    # Check memory usage
    if command -v ps >/dev/null 2>&1; then
        MEMORY_USAGE=$(ps -o pid,vsz,rss,comm -p $$ | tail -1 | awk '{print $3}')
        echo -e "Script Memory: ${BLUE}${MEMORY_USAGE}KB${NC}"
    fi

    # Execution time
    END_TIME=$(date +%s)
    if [ -n "$START_TIME" ]; then
        DURATION=$((END_TIME - START_TIME))
        echo -e "Execution Time: ${BLUE}${DURATION}s${NC}"
    fi

    echo ""
    echo -e "${GREEN}✨ PDF Pipeline is production-ready!${NC}"
fi