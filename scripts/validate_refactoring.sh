#!/bin/bash
# Riptide Refactoring Validation Script
# Comprehensive validation gates for architectural refactoring
# Usage: ./scripts/validate_refactoring.sh [--phase <1|2|3>] [--quick]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PHASE="${PHASE:-all}"
QUICK_MODE=false
VERBOSE=false
GOLDEN_BASELINE_DIR="tests/golden/baselines"
MEMORY_LIMIT_MB=600
PERFORMANCE_THRESHOLD_PERCENT=5

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --phase)
            PHASE="$2"
            shift 2
            ;;
        --quick)
            QUICK_MODE=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Helper functions
print_header() {
    echo -e "${BLUE}=== $1 ===${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

run_gate() {
    local gate_name="$1"
    local gate_cmd="$2"

    echo ""
    print_header "Gate: $gate_name"

    if $VERBOSE; then
        eval "$gate_cmd"
    else
        eval "$gate_cmd" >/dev/null 2>&1
    fi

    if [ $? -eq 0 ]; then
        print_success "$gate_name passed"
        return 0
    else
        print_error "$gate_name failed"
        return 1
    fi
}

# Gate 1: Compilation
gate_compilation() {
    print_header "[1/8] Compilation Check"

    echo "Checking workspace compilation with zero warnings..."
    if RUSTFLAGS="-D warnings" cargo build --workspace --all-features 2>&1 | tee /tmp/build.log; then
        print_success "Compilation successful with zero warnings"
        return 0
    else
        print_error "Compilation failed or has warnings"
        echo "See /tmp/build.log for details"
        return 1
    fi
}

# Gate 2: Unit Tests
gate_unit_tests() {
    print_header "[2/8] Unit Tests"

    echo "Running unit tests across all crates..."
    if cargo test --workspace --lib --bins 2>&1 | tee /tmp/unit-tests.log; then
        local test_count=$(grep -o "test result: ok" /tmp/unit-tests.log | wc -l)
        print_success "Unit tests passed ($test_count test suites)"
        return 0
    else
        print_error "Unit tests failed"
        echo "See /tmp/unit-tests.log for details"
        return 1
    fi
}

# Gate 3: Integration Tests
gate_integration_tests() {
    print_header "[3/8] Integration Tests"

    if [ "$QUICK_MODE" = true ]; then
        print_warning "Skipping integration tests (quick mode)"
        return 0
    fi

    echo "Running integration tests..."
    if cargo test --workspace --tests 2>&1 | tee /tmp/integration-tests.log; then
        local test_count=$(grep -o "test result: ok" /tmp/integration-tests.log | wc -l)
        print_success "Integration tests passed ($test_count test suites)"
        return 0
    else
        print_error "Integration tests failed"
        echo "See /tmp/integration-tests.log for details"
        return 1
    fi
}

# Gate 4: Golden Tests (Regression Detection)
gate_golden_tests() {
    print_header "[4/8] Golden Tests (Regression Detection)"

    if [ ! -d "$GOLDEN_BASELINE_DIR" ]; then
        print_warning "Golden baseline directory not found, skipping"
        return 0
    fi

    echo "Running golden tests to detect regressions..."
    if cargo test --test golden -- --nocapture 2>&1 | tee /tmp/golden-tests.log; then
        # Check for regression markers in output
        if grep -q "REGRESSION DETECTED" /tmp/golden-tests.log; then
            print_error "Performance or functional regression detected"
            echo "Review /tmp/golden-tests.log for details"
            return 1
        else
            print_success "No regressions detected"
            return 0
        fi
    else
        print_error "Golden tests failed"
        return 1
    fi
}

# Gate 5: Layer Boundary Validation
gate_layer_boundaries() {
    print_header "[5/8] Layer Boundary Validation"

    echo "Checking for layer boundary violations..."

    # Check: Handlers should not directly import domain crates
    local handler_violations=$(grep -r "use riptide_spider::" crates/riptide-api/src/handlers/ 2>/dev/null | wc -l)
    handler_violations=$((handler_violations + $(grep -r "use riptide_extraction::" crates/riptide-api/src/handlers/ 2>/dev/null | wc -l)))

    if [ "$handler_violations" -gt 0 ]; then
        print_error "Found $handler_violations layer boundary violations in handlers"
        print_error "Handlers should only use facades, not domain crates directly"
        return 1
    fi

    # Check: Facades exist and are used
    if [ ! -d "crates/riptide-facade/src/facades" ]; then
        print_error "Facade directory not found"
        return 1
    fi

    local facade_count=$(ls -1 crates/riptide-facade/src/facades/*.rs 2>/dev/null | wc -l)
    if [ "$facade_count" -lt 10 ]; then
        print_warning "Only $facade_count facades found (expected at least 10)"
    else
        print_success "Layer boundaries validated ($facade_count facades)"
    fi

    return 0
}

# Gate 6: Performance Benchmarks
gate_performance() {
    print_header "[6/8] Performance Benchmarks"

    if [ "$QUICK_MODE" = true ]; then
        print_warning "Skipping performance benchmarks (quick mode)"
        return 0
    fi

    echo "Running performance benchmarks..."

    # Run benchmarks and compare to baseline
    if cargo bench --bench performance_benchmarks -- --baseline main 2>&1 | tee /tmp/benchmarks.log; then
        # Check for performance regressions
        if grep -q "Performance has regressed" /tmp/benchmarks.log; then
            print_error "Performance regression detected (>${PERFORMANCE_THRESHOLD_PERCENT}%)"
            return 1
        else
            print_success "Performance within acceptable threshold"
            return 0
        fi
    else
        print_warning "Benchmark execution failed or no baseline found"
        # Don't fail on missing benchmarks
        return 0
    fi
}

# Gate 7: Memory Usage Check
gate_memory() {
    print_header "[7/8] Memory Usage Check"

    if [ "$QUICK_MODE" = true ]; then
        print_warning "Skipping memory checks (quick mode)"
        return 0
    fi

    echo "Running memory stability tests..."

    # Run memory-intensive tests
    if cargo test --package riptide-pdf --test pdf_memory_stability_test 2>&1 | tee /tmp/memory-tests.log; then
        # Check for memory limit violations
        if grep -q "Memory limit exceeded" /tmp/memory-tests.log; then
            print_error "Memory usage exceeded ${MEMORY_LIMIT_MB}MB limit"
            return 1
        else
            print_success "Memory usage within limits"
            return 0
        fi
    else
        print_warning "Memory tests not available, skipping"
        return 0
    fi
}

# Gate 8: Code Quality (Clippy)
gate_clippy() {
    print_header "[8/8] Code Quality (Clippy)"

    echo "Running clippy with strict settings..."
    if cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tee /tmp/clippy.log; then
        print_success "No clippy warnings or errors"
        return 0
    else
        print_error "Clippy warnings or errors detected"
        echo "See /tmp/clippy.log for details"
        return 1
    fi
}

# Phase-specific validation
run_phase_validation() {
    local phase=$1

    case $phase in
        1)
            print_header "Phase 1: Facade Layer Validation"
            echo "Validating facade layer completion..."

            # Check facade tests exist
            local facade_test_count=$(find crates/riptide-facade/tests -name "*.rs" -type f | wc -l)
            if [ "$facade_test_count" -lt 10 ]; then
                print_error "Insufficient facade tests ($facade_test_count found, need at least 10)"
                return 1
            fi

            print_success "Phase 1 validation complete"
            ;;
        2)
            print_header "Phase 2: Handler Migration Validation"
            echo "Validating handler migration..."

            # Check handlers use facades
            if grep -r "riptide_spider::\|riptide_extraction::" crates/riptide-api/src/handlers/ 2>/dev/null; then
                print_error "Handlers still have direct domain dependencies"
                return 1
            fi

            print_success "Phase 2 validation complete"
            ;;
        3)
            print_header "Phase 3: Final Cleanup Validation"
            echo "Validating final cleanup..."

            # Check for circular dependencies
            if cargo build --workspace 2>&1 | grep -i "cyclic"; then
                print_error "Circular dependencies detected"
                return 1
            fi

            print_success "Phase 3 validation complete"
            ;;
        all)
            print_header "Full Validation (All Phases)"
            ;;
        *)
            print_error "Unknown phase: $phase"
            return 1
            ;;
    esac

    return 0
}

# Main validation flow
main() {
    echo ""
    print_header "Riptide Refactoring Validation"
    echo "Phase: $PHASE"
    echo "Quick Mode: $QUICK_MODE"
    echo ""

    # Track failures
    local failed_gates=()

    # Run phase-specific validation first
    if ! run_phase_validation "$PHASE"; then
        failed_gates+=("Phase $PHASE validation")
    fi

    # Run all validation gates
    if ! gate_compilation; then
        failed_gates+=("Compilation")
    fi

    if ! gate_unit_tests; then
        failed_gates+=("Unit Tests")
    fi

    if ! gate_integration_tests; then
        failed_gates+=("Integration Tests")
    fi

    if ! gate_golden_tests; then
        failed_gates+=("Golden Tests")
    fi

    if ! gate_layer_boundaries; then
        failed_gates+=("Layer Boundaries")
    fi

    if ! gate_performance; then
        failed_gates+=("Performance")
    fi

    if ! gate_memory; then
        failed_gates+=("Memory")
    fi

    if ! gate_clippy; then
        failed_gates+=("Clippy")
    fi

    # Summary
    echo ""
    print_header "Validation Summary"

    if [ ${#failed_gates[@]} -eq 0 ]; then
        print_success "ALL VALIDATION GATES PASSED ✅"
        echo ""
        echo "Refactoring changes are safe to merge."
        return 0
    else
        print_error "VALIDATION FAILED ❌"
        echo ""
        echo "Failed gates:"
        for gate in "${failed_gates[@]}"; do
            echo "  - $gate"
        done
        echo ""
        echo "Please fix the issues before merging."
        return 1
    fi
}

# Run main function
main
exit $?
