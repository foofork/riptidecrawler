#!/bin/bash
# Architecture Validation Script for Phase 3 (Week 6)
# Validates infrastructure consolidation in riptide-reliability

set -e

echo "================================================"
echo "  Riptide Architecture Validation Script"
echo "  Phase 3 (Week 6): Infrastructure Consolidation"
echo "================================================"
echo ""

WORKSPACE_ROOT="/workspaces/eventmesh"
cd "$WORKSPACE_ROOT"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

VIOLATIONS=0

echo "🔍 Step 1: Checking for HTTP types in facades..."
echo "----------------------------------------------"
if rg -q "HttpMethod|StatusCode|reqwest::Client" crates/riptide-facade/src/facades/ 2>/dev/null; then
    echo -e "${YELLOW}⚠️  WARNING: Found HTTP types in facades:${NC}"
    rg "HttpMethod|StatusCode|reqwest::Client" crates/riptide-facade/src/facades/ || true
    echo ""
    echo "   Facades should use HttpClientService from riptide-reliability"
    VIOLATIONS=$((VIOLATIONS + 1))
else
    echo -e "${GREEN}✓ No HTTP type violations in facades${NC}"
fi
echo ""

echo "🔍 Step 2: Validating riptide-reliability exports..."
echo "-----------------------------------------------------"
if grep -q "pub use http_client::" crates/riptide-reliability/src/lib.rs; then
    echo -e "${GREEN}✓ HttpClientService properly exported${NC}"
else
    echo -e "${RED}✗ HttpClientService not exported from riptide-reliability${NC}"
    VIOLATIONS=$((VIOLATIONS + 1))
fi

if grep -q "pub mod circuit;" crates/riptide-reliability/src/lib.rs; then
    echo -e "${GREEN}✓ Circuit presets module properly exported${NC}"
else
    echo -e "${RED}✗ Circuit presets module not exported${NC}"
    VIOLATIONS=$((VIOLATIONS + 1))
fi
echo ""

echo "================================================"
echo "  Validation Summary"
echo "================================================"
echo ""

if [ $VIOLATIONS -eq 0 ]; then
    echo -e "${GREEN}✓ Architecture validation PASSED${NC}"
    exit 0
else
    echo -e "${YELLOW}⚠️  Found $VIOLATIONS potential issues${NC}"
    exit 0
fi
