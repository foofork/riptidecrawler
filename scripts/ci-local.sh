#!/bin/bash
# Local CI Mirror - Run the same checks as GitHub Actions locally
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

FAILED_CHECKS=()

print_header() {
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

run_check() {
    local name="$1"
    shift
    echo -e "\n${YELLOW}▶ Running: $name${NC}"

    if "$@"; then
        echo -e "${GREEN}✓ $name passed${NC}"
        return 0
    else
        echo -e "${RED}✗ $name failed${NC}"
        FAILED_CHECKS+=("$name")
        return 1
    fi
}

# Check for required tools
check_tool() {
    if ! command -v "$1" &> /dev/null; then
        echo -e "${YELLOW}⚠ $1 not found. Installing...${NC}"
        cargo install "$2"
    fi
}

print_header "🔧 Checking Required Tools"
check_tool cargo-deny cargo-deny
check_tool cargo-audit cargo-audit
# cargo-bloat is optional, skip for now

print_header "🎨 Code Formatting Check"
run_check "cargo fmt" cargo fmt --all --check || true

print_header "📋 Clippy Lints"
run_check "cargo clippy" cargo clippy --workspace --all-targets -- -D warnings || true

print_header "🔒 Security Audit"
run_check "cargo audit" cargo audit --deny warnings || true

print_header "📜 License Check"
run_check "cargo deny licenses" cargo deny check licenses || true

print_header "🚫 Dependency Bans"
run_check "cargo deny bans" cargo deny check bans || true

print_header "⚠️  Advisory Check"
run_check "cargo deny advisories" cargo deny check advisories || true

print_header "🔨 Build Check (Native)"
run_check "cargo build workspace" cargo build --workspace --all-targets || true

print_header "🧪 Unit Tests"
run_check "cargo test lib" cargo test --workspace --lib --bins -- --nocapture || true

print_header "🔗 Integration Tests"
run_check "cargo test integration" cargo test --workspace --tests -- --nocapture || true

# Optional: WASM build (slower)
if [[ "${SKIP_WASM:-0}" != "1" ]]; then
    print_header "🌐 WASM Build"
    run_check "cargo build wasm" bash -c "cd wasm/riptide-extractor-wasm && cargo build --target wasm32-wasip2" || true
fi

# Summary
echo -e "\n${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
if [ ${#FAILED_CHECKS[@]} -eq 0 ]; then
    echo -e "${GREEN}✅ All checks passed!${NC}"
    echo -e "${GREEN}Your code is ready to push to CI${NC}"
    exit 0
else
    echo -e "${RED}❌ ${#FAILED_CHECKS[@]} check(s) failed:${NC}"
    for check in "${FAILED_CHECKS[@]}"; do
        echo -e "${RED}  - $check${NC}"
    done
    echo -e "\n${YELLOW}Fix these issues before pushing to avoid CI failures${NC}"
    exit 1
fi
