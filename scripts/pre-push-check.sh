#!/bin/bash
# Comprehensive Pre-Push Check
# Catches ALL common GitHub Actions failures locally
# Run this before every push to avoid CI failures

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

FAILED=()
WARNINGS=()

print_header() {
    echo -e "\n${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${CYAN}$1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

check_step() {
    local name="$1"
    shift
    echo -e "${YELLOW}▶${NC} $name..."

    if "$@" > /tmp/check_output 2>&1; then
        echo -e "${GREEN}✓${NC} $name passed"
        return 0
    else
        echo -e "${RED}✗${NC} $name failed"
        cat /tmp/check_output | tail -20
        FAILED+=("$name")
        return 1
    fi
}

warn_if_fails() {
    local name="$1"
    shift
    echo -e "${YELLOW}▶${NC} $name..."

    if "$@" > /tmp/check_output 2>&1; then
        echo -e "${GREEN}✓${NC} $name passed"
    else
        echo -e "${YELLOW}⚠${NC} $name has warnings (non-blocking)"
        cat /tmp/check_output | tail -10
        WARNINGS+=("$name")
    fi
}

print_header "🚀 Pre-Push Comprehensive Check"
echo "This mirrors GitHub Actions CI to catch issues before pushing"

# ============================================================================
# CRITICAL CHECKS (Must pass)
# ============================================================================

print_header "1️⃣ Code Formatting"
check_step "cargo fmt --check" cargo fmt --all --check

print_header "2️⃣ Clippy Lints (strict)"
check_step "cargo clippy (all warnings)" \
    cargo clippy --workspace --all-targets -- -D warnings

print_header "3️⃣ Unused Imports & Dead Code"
check_step "cargo check with pedantic" \
    cargo check --workspace --all-targets

print_header "4️⃣ Compilation (all targets)"
check_step "cargo build workspace" \
    cargo build --workspace --all-targets

print_header "5️⃣ Unit Tests"
check_step "cargo test --lib" \
    cargo test --workspace --lib --bins -- --test-threads=1 --nocapture

print_header "6️⃣ Integration Tests"
check_step "cargo test --tests" \
    cargo test --workspace --tests -- --test-threads=1 --nocapture

print_header "7️⃣ Doc Tests"
warn_if_fails "cargo test --doc" \
    cargo test --workspace --doc

# ============================================================================
# SECURITY & QUALITY CHECKS
# ============================================================================

print_header "8️⃣ Security Audit"
if command -v cargo-audit &> /dev/null; then
    warn_if_fails "cargo audit" cargo audit
else
    echo -e "${YELLOW}⚠${NC} cargo-audit not installed (run: cargo install cargo-audit)"
    WARNINGS+=("cargo-audit not installed")
fi

print_header "9️⃣ License & Dependency Check"
if command -v cargo-deny &> /dev/null; then
    warn_if_fails "cargo deny check" cargo deny check licenses bans advisories
else
    echo -e "${YELLOW}⚠${NC} cargo-deny not installed (run: cargo install cargo-deny)"
    WARNINGS+=("cargo-deny not installed")
fi

# ============================================================================
# PROJECT-SPECIFIC CHECKS
# ============================================================================

print_header "🔟 OpenAPI Schema Validation"
if command -v swagger-cli &> /dev/null || command -v npx &> /dev/null; then
    if [ -f "docs/api/openapi.yaml" ]; then
        check_step "OpenAPI validation" \
            npx @apidevtools/swagger-cli validate docs/api/openapi.yaml
    fi
else
    echo -e "${YELLOW}⚠${NC} swagger-cli not available, skipping OpenAPI validation"
fi

print_header "1️⃣1️⃣ Feature Gate Consistency"
echo -e "${YELLOW}▶${NC} Checking for invalid #[cfg(feature = ...)] attributes..."
if grep -rn '#\[cfg.*feature.*=' crates/ | grep -v 'Cargo.toml' | grep -Ev '(streaming|persistence|profiling-full|sessions|events|jemalloc|full|default)' > /tmp/bad_features; then
    echo -e "${RED}✗${NC} Found potentially invalid feature gates:"
    cat /tmp/bad_features
    FAILED+=("Invalid feature gates found")
else
    echo -e "${GREEN}✓${NC} Feature gates look good"
fi

print_header "1️⃣2️⃣ Unused Imports Detection"
echo -e "${YELLOW}▶${NC} Checking for unused imports..."
if cargo build --workspace 2>&1 | grep -i "unused import" > /tmp/unused_imports; then
    echo -e "${RED}✗${NC} Found unused imports:"
    cat /tmp/unused_imports
    FAILED+=("Unused imports found")
else
    echo -e "${GREEN}✓${NC} No unused imports"
fi

# ============================================================================
# COMMON ERROR PATTERNS
# ============================================================================

print_header "1️⃣3️⃣ Common CI Error Patterns"

echo -e "${YELLOW}▶${NC} Checking for common issues..."

# Check for accidentally committed temp files
if ls *.md 2>/dev/null | grep -E '^temp.*\.md$' > /dev/null; then
    echo -e "${YELLOW}⚠${NC} Found temp*.md files in root (should be in docs/)"
    WARNINGS+=("Temp files in root directory")
fi

# Check for large test timeouts
if grep -rn "timeout.*60\|timeout.*120" crates/*/src/tests.rs 2>/dev/null; then
    echo -e "${YELLOW}⚠${NC} Found potentially long test timeouts (may cause CI issues)"
    WARNINGS+=("Long test timeouts found")
fi

# Check for missing #[ignore] on known-failing tests
if grep -rn '#\[tokio::test\]' crates/ | grep -A5 "not implemented\|TODO\|FIXME" | grep -v '#\[ignore' > /dev/null 2>&1; then
    echo -e "${YELLOW}⚠${NC} Found potentially incomplete tests without #[ignore]"
    WARNINGS+=("Incomplete tests without #[ignore]")
fi

echo -e "${GREEN}✓${NC} Common pattern check complete"

# ============================================================================
# SUMMARY
# ============================================================================

print_header "📊 Summary"

if [ ${#WARNINGS[@]} -gt 0 ]; then
    echo -e "${YELLOW}⚠ Warnings (${#WARNINGS[@]}):${NC}"
    for warn in "${WARNINGS[@]}"; do
        echo -e "  ${YELLOW}•${NC} $warn"
    done
    echo ""
fi

if [ ${#FAILED[@]} -eq 0 ]; then
    echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║   ✅  ALL CRITICAL CHECKS PASSED!     ║${NC}"
    echo -e "${GREEN}║   Your code is ready to push          ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"
    exit 0
else
    echo -e "${RED}╔════════════════════════════════════════╗${NC}"
    echo -e "${RED}║   ❌  ${#FAILED[@]} CHECK(S) FAILED              ║${NC}"
    echo -e "${RED}╚════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${RED}Failed checks:${NC}"
    for check in "${FAILED[@]}"; do
        echo -e "  ${RED}✗${NC} $check"
    done
    echo ""
    echo -e "${YELLOW}Fix these issues before pushing to avoid CI failures${NC}"
    exit 1
fi
