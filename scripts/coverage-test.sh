#!/bin/bash
# Quick coverage test script for cargo-llvm-cov
# Tests coverage setup across workspace crates

set -e

echo "🧪 Testing cargo-llvm-cov setup across workspace..."
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check installation
echo -n "1️⃣  Checking cargo-llvm-cov installation... "
if command -v cargo-llvm-cov &> /dev/null; then
    VERSION=$(cargo llvm-cov --version | head -1)
    echo -e "${GREEN}✓ ${VERSION}${NC}"
else
    echo -e "${RED}✗ Not installed${NC}"
    echo "Run: cargo install cargo-llvm-cov --locked"
    exit 1
fi

# Check LLVM tools
echo -n "2️⃣  Checking llvm-tools-preview... "
if rustup component list | grep -q "llvm-tools.*installed"; then
    echo -e "${GREEN}✓ Installed${NC}"
else
    echo -e "${RED}✗ Not installed${NC}"
    echo "Run: rustup component add llvm-tools-preview"
    exit 1
fi

# Count workspace members
echo -n "3️⃣  Counting workspace crates... "
CRATE_COUNT=$(grep -A 50 '^\[workspace\]' Cargo.toml | grep -c 'crates/' || true)
echo -e "${GREEN}${CRATE_COUNT} crates found${NC}"

# Test quick coverage (without running)
echo "4️⃣  Testing coverage command (dry-run)..."
if cargo llvm-cov --workspace --all-features --no-run 2>&1 | grep -q "Finished"; then
    echo -e "   ${GREEN}✓ Coverage build successful${NC}"
else
    echo -e "   ${YELLOW}⚠ Coverage build had issues (may need dependencies)${NC}"
fi

# Check configuration files
echo "5️⃣  Checking configuration files..."
CONFIG_FILES=(
    ".cargo/config.toml"
    ".codecov.yml"
    "Makefile"
    "tests/docs/coverage-guide.md"
)

for file in "${CONFIG_FILES[@]}"; do
    if [ -f "$file" ]; then
        echo -e "   ${GREEN}✓${NC} $file"
    else
        echo -e "   ${RED}✗${NC} $file (missing)"
    fi
done

# Check workflow updates
echo "6️⃣  Checking CI workflow updates..."
WORKFLOW_FILES=(
    ".github/workflows/baseline-check.yml"
    ".github/workflows/refactoring-quality.yml"
)

for file in "${WORKFLOW_FILES[@]}"; do
    if grep -q "cargo-llvm-cov" "$file" 2>/dev/null; then
        echo -e "   ${GREEN}✓${NC} $file (uses cargo-llvm-cov)"
    else
        echo -e "   ${YELLOW}⚠${NC} $file (may still use tarpaulin)"
    fi
done

# Check for old tarpaulin config
echo "7️⃣  Checking for legacy Tarpaulin config..."
if [ -f "tarpaulin.toml" ] || [ -f ".tarpaulin.toml" ]; then
    echo -e "   ${YELLOW}⚠ Found tarpaulin.toml (consider removing)${NC}"
else
    echo -e "   ${GREEN}✓ No legacy tarpaulin config${NC}"
fi

# Test Makefile targets
echo "8️⃣  Verifying Makefile coverage targets..."
MAKE_TARGETS=(
    "coverage"
    "coverage-html"
    "coverage-lcov"
    "coverage-json"
    "coverage-open"
    "coverage-report"
)

for target in "${MAKE_TARGETS[@]}"; do
    if grep -q "^${target}:" Makefile 2>/dev/null; then
        echo -e "   ${GREEN}✓${NC} make ${target}"
    else
        echo -e "   ${RED}✗${NC} make ${target} (missing)"
    fi
done

echo ""
echo -e "${GREEN}✅ Coverage setup verification complete!${NC}"
echo ""
echo "Next steps:"
echo "  1. Run: make coverage-html"
echo "  2. Open: target/llvm-cov/html/index.html"
echo "  3. Check coverage percentage meets 80% baseline"
echo ""
echo "Quick commands:"
echo "  make coverage         - Generate lcov.info"
echo "  make coverage-open    - Generate and open HTML report"
echo "  make coverage-report  - Generate all formats (HTML, LCOV, JSON)"
