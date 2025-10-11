#!/usr/bin/env bash
set -euo pipefail

# Verification script to check test infrastructure setup

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_DIR="$(dirname "$SCRIPT_DIR")"
PROJECT_ROOT="$(cd "$TEST_DIR/../.." && pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[OK]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

echo "================================"
echo "Test Infrastructure Verification"
echo "================================"
echo ""

ERRORS=0
WARNINGS=0

# Check directory structure
log_info "Checking directory structure..."

REQUIRED_DIRS=(
    "$TEST_DIR"
    "$TEST_DIR/scripts"
)

REQUIRED_FILES=(
    "$TEST_DIR/test-urls.json"
    "$TEST_DIR/cli-test-harness.rs"
    "$TEST_DIR/comparison-tool.rs"
    "$TEST_DIR/lib.rs"
    "$TEST_DIR/main.rs"
    "$TEST_DIR/Cargo.toml"
    "$TEST_DIR/README.md"
    "$TEST_DIR/scripts/run-all-tests.sh"
    "$TEST_DIR/scripts/compare-results.sh"
    "$TEST_DIR/scripts/quick-test.sh"
)

for dir in "${REQUIRED_DIRS[@]}"; do
    if [[ -d "$dir" ]]; then
        log_success "Directory exists: $(basename "$dir")"
    else
        log_error "Missing directory: $dir"
        ERRORS=$((ERRORS + 1))
    fi
done

echo ""
log_info "Checking required files..."

for file in "${REQUIRED_FILES[@]}"; do
    if [[ -f "$file" ]]; then
        log_success "File exists: $(basename "$file")"
    else
        log_error "Missing file: $file"
        ERRORS=$((ERRORS + 1))
    fi
done

# Check scripts are executable
echo ""
log_info "Checking script permissions..."

SCRIPTS=(
    "$TEST_DIR/scripts/run-all-tests.sh"
    "$TEST_DIR/scripts/compare-results.sh"
    "$TEST_DIR/scripts/quick-test.sh"
)

for script in "${SCRIPTS[@]}"; do
    if [[ -x "$script" ]]; then
        log_success "Executable: $(basename "$script")"
    else
        log_warning "Not executable: $script"
        log_info "Run: chmod +x $script"
        WARNINGS=$((WARNINGS + 1))
    fi
done

# Check test URLs
echo ""
log_info "Validating test URLs..."

if [[ -f "$TEST_DIR/test-urls.json" ]]; then
    if jq empty "$TEST_DIR/test-urls.json" 2>/dev/null; then
        URL_COUNT=$(jq '.test_urls | length' "$TEST_DIR/test-urls.json")
        log_success "Valid JSON with $URL_COUNT test URLs"

        # Check URL structure
        CATEGORIES=$(jq -r '.test_urls[].category' "$TEST_DIR/test-urls.json" | sort -u | wc -l)
        log_info "  Categories: $CATEGORIES"

        # List categories
        log_info "  URL categories:"
        jq -r '.test_urls[].category' "$TEST_DIR/test-urls.json" | sort -u | while read -r cat; do
            count=$(jq "[.test_urls[] | select(.category == \"$cat\")] | length" "$TEST_DIR/test-urls.json")
            echo "    - $cat: $count URLs"
        done
    else
        log_error "Invalid JSON in test-urls.json"
        ERRORS=$((ERRORS + 1))
    fi
else
    log_error "test-urls.json not found"
    ERRORS=$((ERRORS + 1))
fi

# Check dependencies
echo ""
log_info "Checking dependencies..."

# jq
if command -v jq &> /dev/null; then
    log_success "jq: $(jq --version)"
else
    log_error "jq not found - required for JSON processing"
    log_info "Install: sudo apt-get install jq (Ubuntu) or brew install jq (macOS)"
    ERRORS=$((ERRORS + 1))
fi

# bc
if command -v bc &> /dev/null; then
    log_success "bc: available"
else
    log_warning "bc not found - needed for percentage calculations"
    WARNINGS=$((WARNINGS + 1))
fi

# cargo
if command -v cargo &> /dev/null; then
    log_success "cargo: $(cargo --version | head -n1)"
else
    log_error "cargo not found - required for building Rust components"
    ERRORS=$((ERRORS + 1))
fi

# Check project binary
echo ""
log_info "Checking eventmesh CLI binary..."

BINARY_RELEASE="$PROJECT_ROOT/target/release/eventmesh-cli"
BINARY_DEBUG="$PROJECT_ROOT/target/debug/eventmesh-cli"

if [[ -f "$BINARY_RELEASE" ]]; then
    log_success "Release binary found: $BINARY_RELEASE"
elif [[ -f "$BINARY_DEBUG" ]]; then
    log_warning "Debug binary found: $BINARY_DEBUG"
    log_info "Consider building release version: cargo build --release"
    WARNINGS=$((WARNINGS + 1))
else
    log_warning "No binary found - will be built on first run"
    WARNINGS=$((WARNINGS + 1))
fi

# Create output directories
echo ""
log_info "Creating output directories..."

mkdir -p "$TEST_DIR/results" "$TEST_DIR/logs"
log_success "Created: results/ and logs/"

# Summary
echo ""
echo "================================"
echo "Verification Summary"
echo "================================"

if [[ $ERRORS -eq 0 ]] && [[ $WARNINGS -eq 0 ]]; then
    log_success "✨ All checks passed! Test infrastructure is ready."
    echo ""
    echo "Quick Start:"
    echo "  cd $TEST_DIR/scripts"
    echo "  ./quick-test.sh              # Quick test (3 URLs)"
    echo "  ./run-all-tests.sh run       # Full test suite (30 URLs)"
    echo "  ./run-all-tests.sh report    # View latest results"
    echo ""
elif [[ $ERRORS -eq 0 ]]; then
    log_warning "Setup complete with $WARNINGS warnings"
    echo ""
    echo "Test infrastructure is functional but consider addressing warnings."
    echo ""
else
    log_error "Setup incomplete: $ERRORS errors, $WARNINGS warnings"
    echo ""
    echo "Please fix errors before running tests."
    exit 1
fi

# Show example usage
echo "Example Usage:"
echo ""
echo "1. Quick test (3 URLs, 2 methods):"
echo "   $ ./scripts/quick-test.sh"
echo ""
echo "2. Full test suite (30 URLs, 6 methods):"
echo "   $ ./scripts/run-all-tests.sh run"
echo ""
echo "3. Compare methods:"
echo "   $ ./scripts/compare-results.sh methods"
echo ""
echo "4. Compare two sessions:"
echo "   $ ./scripts/compare-results.sh sessions session1.json session2.json"
echo ""
echo "5. Using Rust CLI:"
echo "   $ cargo run --bin webpage-extraction-tests -- run --methods jina,playwright"
echo ""
echo "6. View available sessions:"
echo "   $ cargo run --bin webpage-extraction-tests -- list"
echo ""

echo "Documentation:"
echo "  README: $TEST_DIR/README.md"
echo ""
