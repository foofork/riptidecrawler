#!/usr/bin/env bash
# Pre-commit hook for cleaning old build artifacts with cargo-sweep
# This helps maintain a clean workspace by removing artifacts older than 7 days
#
# Installation:
#   ln -s ../../scripts/pre-commit-sweep.sh .git/hooks/pre-commit
#
# Or add to existing pre-commit hook:
#   bash scripts/pre-commit-sweep.sh

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🧹 Running cargo sweep to clean old build artifacts...${NC}"

# Check if cargo-sweep is installed
if ! command -v cargo-sweep &> /dev/null; then
    echo -e "${YELLOW}⚠️  cargo-sweep not found. Installing...${NC}"
    cargo install cargo-sweep --locked
fi

# Timestamp current build before cleaning
cargo sweep --stamp

# Clean artifacts older than 7 days
if cargo sweep --time 7 2>&1 | grep -q "Cleaned"; then
    echo -e "${GREEN}✅ Removed old build artifacts (>7 days old)${NC}"
else
    echo -e "${GREEN}✅ No old artifacts to clean${NC}"
fi

# Optional: Show cache size
if command -v du &> /dev/null; then
    TARGET_SIZE=$(du -sh target 2>/dev/null | cut -f1 || echo "unknown")
    echo -e "${GREEN}📦 Current target/ size: ${TARGET_SIZE}${NC}"
fi

exit 0
