#!/bin/bash
# Pre-commit hook for quality checks
# Install: ln -s ../../scripts/pre-commit.sh .git/hooks/pre-commit

set -e

echo "🔍 Running pre-commit quality checks..."
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if running in CI
if [ -n "$CI" ]; then
    echo "Running in CI environment, skipping pre-commit checks"
    exit 0
fi

# 1. Format check
echo "1️⃣ Checking code formatting..."
if cargo fmt --all --check; then
    echo -e "${GREEN}✅ Format check passed${NC}"
else
    echo -e "${RED}❌ Format check failed${NC}"
    echo "Run 'cargo fmt --all' to fix formatting"
    exit 1
fi
echo ""

# 2. Clippy check
echo "2️⃣ Running Clippy linter..."
if cargo clippy --all-targets --all-features -- -D warnings; then
    echo -e "${GREEN}✅ Clippy check passed${NC}"
else
    echo -e "${RED}❌ Clippy check failed${NC}"
    echo "Fix the warnings above before committing"
    exit 1
fi
echo ""

# 3. Quick test
echo "3️⃣ Running quick tests..."
if cargo test --lib --all-features; then
    echo -e "${GREEN}✅ Tests passed${NC}"
else
    echo -e "${RED}❌ Tests failed${NC}"
    echo "Fix the failing tests before committing"
    exit 1
fi
echo ""

# 4. File length check
echo "4️⃣ Checking file lengths..."
if [ -x "scripts/check_file_lengths.sh" ]; then
    if ./scripts/check_file_lengths.sh; then
        echo -e "${GREEN}✅ File length check passed${NC}"
    else
        echo -e "${YELLOW}⚠️  Warning: Some files exceed 600 lines${NC}"
        echo "Consider refactoring according to docs/REFACTORING_PLAN.md"
        # Don't fail the commit for this, just warn
    fi
else
    echo -e "${YELLOW}⚠️  File length check script not found${NC}"
fi
echo ""

echo -e "${GREEN}✅ All pre-commit checks passed!${NC}"
echo ""
