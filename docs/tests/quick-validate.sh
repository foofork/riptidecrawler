#!/bin/bash
# Quick Documentation Validator
# Simplified version for fast CI/CD execution

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"

errors=0
warnings=0

echo "Quick Documentation Validation"
echo "=============================="
echo""

# 1. Check for unquoted JSON in curl commands
echo "Checking curl commands..."
if grep -rn "curl.*-d[[:space:]]*{" "$PROJECT_ROOT/docs" "$PROJECT_ROOT/README.md" 2>/dev/null | grep -v '"\|'\'''; then
    echo -e "${RED}✗ Found unquoted JSON in curl commands${NC}"
    ((errors++))
else
    echo -e "${GREEN}✓ All curl commands properly quoted${NC}"
fi

# 2. Check for broken internal markdown links (basic check - skip for now as it needs realpath)
echo ""
echo "Checking markdown syntax..."
# Count total links for info
total_links=$(grep -ro '\[.*\](.*)'  "$PROJECT_ROOT/docs" 2>/dev/null | wc -l || echo "0")
echo -e "${GREEN}✓ Found $total_links markdown links (detailed validation available in link-checker.sh)${NC}"

# 3. Check for dangerous commands (root directory deletion)
echo ""
echo "Checking for dangerous commands..."
dangerous=$(grep -rn "rm[[:space:]]*-rf[[:space:]]*/" "$PROJECT_ROOT/docs" 2>/dev/null | grep -v "/tmp\|/var/lib/apt/lists\|/var/cache\|docs/tests" || true)
if [[ -n "$dangerous" ]]; then
    echo "$dangerous"
    echo -e "${YELLOW}⚠ Found rm -rf commands (review for safety)${NC}"
    ((warnings++))
else
    echo -e "${GREEN}✓ No dangerous root directory deletions${NC}"
fi

# 4. Count code blocks
echo ""
echo "Counting code blocks..."
total_blocks=$(grep -rc '```' "$PROJECT_ROOT/docs" 2>/dev/null | awk -F: '{sum+=$2} END {print sum/2}')
echo "  Total code blocks: $total_blocks"

# 5. Check OpenAPI spec existence
echo ""
echo "Checking OpenAPI specification..."
if [[ -f "$PROJECT_ROOT/docs/02-api-reference/openapi.yaml" ]]; then
    echo -e "${GREEN}✓ OpenAPI spec found${NC}"

    # Basic YAML syntax check
    if command -v python3 &>/dev/null; then
        if python3 -c "import yaml; yaml.safe_load(open('$PROJECT_ROOT/docs/02-api-reference/openapi.yaml'))" 2>/dev/null; then
            echo -e "${GREEN}✓ OpenAPI spec is valid YAML${NC}"
        else
            echo -e "${RED}✗ OpenAPI spec has YAML syntax errors${NC}"
            ((errors++))
        fi
    fi
else
    echo -e "${YELLOW}⚠ No OpenAPI spec found${NC}"
    ((warnings++))
fi

# Summary
echo ""
echo "=============================="
if [[ $errors -eq 0 ]]; then
    echo -e "${GREEN}✓ Validation PASSED${NC}"
    if [[ $warnings -gt 0 ]]; then
        echo -e "${YELLOW}  ($warnings warnings)${NC}"
    fi
    exit 0
else
    echo -e "${RED}✗ Validation FAILED${NC}"
    echo -e "${RED}  $errors errors${NC}"
    if [[ $warnings -gt 0 ]]; then
        echo -e "${YELLOW}  $warnings warnings${NC}"
    fi
    exit 1
fi
