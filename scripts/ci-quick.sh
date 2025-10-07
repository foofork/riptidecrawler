#!/bin/bash
# Quick CI Check - Fast subset for rapid iteration
# Run before every commit for instant feedback

set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

echo "🚀 Quick CI Check (30s-1min)"
echo ""

# Fast checks only
cargo fmt --all --check && echo -e "${GREEN}✓${NC} Formatting" || (echo -e "${RED}✗${NC} Formatting" && exit 1)
cargo clippy --workspace --lib --bins --quiet && echo -e "${GREEN}✓${NC} Clippy" || (echo -e "${RED}✗${NC} Clippy" && exit 1)
cargo test --workspace --lib --quiet && echo -e "${GREEN}✓${NC} Unit Tests" || (echo -e "${RED}✗${NC} Unit Tests" && exit 1)

echo -e "\n${GREEN}✅ Quick checks passed! Safe to commit.${NC}"
echo -e "💡 Run ${GREEN}./scripts/ci-local.sh${NC} before pushing for full validation"
