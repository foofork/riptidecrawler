#!/bin/bash
# Master Validation Script
# Runs all documentation validators and generates summary report

set -eo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Find script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Validation results
declare -A results
declare -A times
total_validators=3
passed_validators=0
failed_validators=0

echo ""
echo -e "${BOLD}${BLUE}╔════════════════════════════════════════════════╗${NC}"
echo -e "${BOLD}${BLUE}║  RipTide Documentation Validation Suite      ║${NC}"
echo -e "${BOLD}${BLUE}╚════════════════════════════════════════════════╝${NC}"
echo ""
echo "Running comprehensive documentation validation..."
echo "Start time: $(date '+%Y-%m-%d %H:%M:%S')"
echo ""

# Function to run a validator and capture results
run_validator() {
    local name="$1"
    local script="$2"
    local description="$3"

    echo -e "${BOLD}┌─────────────────────────────────────────────────┐${NC}"
    echo -e "${BOLD}│ $description${NC}"
    echo -e "${BOLD}└─────────────────────────────────────────────────┘${NC}"
    echo ""

    local start_time=$(date +%s)
    local exit_code=0

    # Run the validator and capture output
    if bash "$script" 2>&1; then
        exit_code=0
    else
        exit_code=$?
    fi

    local end_time=$(date +%s)
    local duration=$((end_time - start_time))

    results["$name"]=$exit_code
    times["$name"]=$duration

    if [[ $exit_code -eq 0 ]]; then
        ((passed_validators++))
        echo -e "${GREEN}✓ $name PASSED${NC} (${duration}s)"
    else
        ((failed_validators++))
        echo -e "${RED}✗ $name FAILED${NC} (${duration}s)"
    fi

    echo ""
}

# Run all validators
run_validator "Code Examples" "$SCRIPT_DIR/code-examples-validator.sh" "Validator 1/3: Code Examples and Syntax"
run_validator "Link Checker" "$SCRIPT_DIR/link-checker.sh" "Validator 2/3: Internal Links and Anchors"
run_validator "API Endpoints" "$SCRIPT_DIR/api-endpoint-validator.sh" "Validator 3/3: API Endpoint Accuracy"

# Generate summary report
echo -e "${BOLD}${BLUE}╔════════════════════════════════════════════════╗${NC}"
echo -e "${BOLD}${BLUE}║  Validation Summary Report                    ║${NC}"
echo -e "${BOLD}${BLUE}╚════════════════════════════════════════════════╝${NC}"
echo ""

# Calculate total time
total_time=0
for time in "${times[@]}"; do
    total_time=$((total_time + time))
done

# Print summary
echo "Total Validators: $total_validators"
echo -e "${GREEN}Passed: $passed_validators${NC}"
if [[ $failed_validators -gt 0 ]]; then
    echo -e "${RED}Failed: $failed_validators${NC}"
else
    echo "Failed: 0"
fi
echo "Total Time: ${total_time}s"
echo ""

# Print individual results
echo -e "${BOLD}Individual Results:${NC}"
echo "┌─────────────────────────┬────────┬──────────┐"
echo "│ Validator               │ Status │ Duration │"
echo "├─────────────────────────┼────────┼──────────┤"

for name in "Code Examples" "Link Checker" "API Endpoints"; do
    local status_symbol
    local status_color

    if [[ ${results["$name"]} -eq 0 ]]; then
        status_symbol="✓ PASS"
        status_color="${GREEN}"
    else
        status_symbol="✗ FAIL"
        status_color="${RED}"
    fi

    printf "│ %-23s │ " "$name"
    printf "${status_color}%-6s${NC} │ " "$status_symbol"
    printf "%6ss │\n" "${times["$name"]}"
done

echo "└─────────────────────────┴────────┴──────────┘"
echo ""

# Performance check
if [[ $total_time -gt 60 ]]; then
    echo -e "${YELLOW}⚠ Warning: Total validation time exceeded 60 seconds${NC}"
    echo "  Consider optimizing validation scripts for CI/CD"
else
    echo -e "${GREEN}✓ Performance: Under 60 second target${NC}"
fi

echo ""

# CI/CD recommendations
if [[ $failed_validators -gt 0 ]]; then
    echo -e "${BOLD}Recommendations:${NC}"

    for name in "${!results[@]}"; do
        if [[ ${results["$name"]} -ne 0 ]]; then
            echo -e "${RED}  ✗ Fix issues in: $name${NC}"
            case "$name" in
                "Code Examples")
                    echo "    - Review code blocks in markdown files"
                    echo "    - Ensure proper syntax and quoting"
                    echo "    - Run: bash docs/tests/code-examples-validator.sh"
                    ;;
                "Link Checker")
                    echo "    - Fix broken internal links"
                    echo "    - Update file paths and anchors"
                    echo "    - Run: bash docs/tests/link-checker.sh"
                    ;;
                "API Endpoints")
                    echo "    - Update documentation to match OpenAPI spec"
                    echo "    - Document new endpoints"
                    echo "    - Run: bash docs/tests/api-endpoint-validator.sh"
                    ;;
            esac
            echo ""
        fi
    done
fi

# Final status
echo -e "${BOLD}End time: $(date '+%Y-%m-%d %H:%M:%S')${NC}"
echo ""

if [[ $failed_validators -eq 0 ]]; then
    echo -e "${BOLD}${GREEN}╔════════════════════════════════════════════════╗${NC}"
    echo -e "${BOLD}${GREEN}║  ✓ ALL VALIDATIONS PASSED                     ║${NC}"
    echo -e "${BOLD}${GREEN}╚════════════════════════════════════════════════╝${NC}"
    echo ""
    echo "Documentation is validated and ready for deployment!"
    exit 0
else
    echo -e "${BOLD}${RED}╔════════════════════════════════════════════════╗${NC}"
    echo -e "${BOLD}${RED}║  ✗ VALIDATION FAILED                           ║${NC}"
    echo -e "${BOLD}${RED}╚════════════════════════════════════════════════╝${NC}"
    echo ""
    echo "Please fix the issues above before deploying documentation."
    exit 1
fi
