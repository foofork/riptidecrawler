#!/bin/bash
# Unused Dependency Audit Script
# Generated: 2025-11-01
# Context: P2-F1 Day 6 - Post riptide-core elimination

set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔍 RipTide Unused Dependency Audit"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
TOTAL_CHECKS=0
LIKELY_UNUSED=0
NEEDS_REVIEW=0

check_dependency() {
    local dep_name=$1
    local search_pattern=$2
    local confidence=$3

    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))

    echo -e "${BLUE}[$TOTAL_CHECKS]${NC} Checking: ${YELLOW}$dep_name${NC}"

    # Search for usage in source files
    usage_count=$(grep -r "$search_pattern" crates --include="*.rs" 2>/dev/null | grep -v "^Binary\|\.wasm\|//.*" | wc -l)

    if [ "$usage_count" -eq 0 ]; then
        echo -e "  ${RED}✗ No usage found${NC} (Confidence: $confidence%)"
        if [ "$confidence" -ge 75 ]; then
            LIKELY_UNUSED=$((LIKELY_UNUSED + 1))
            echo "  → Recommendation: REMOVE"
        else
            NEEDS_REVIEW=$((NEEDS_REVIEW + 1))
            echo "  → Recommendation: REVIEW"
        fi
    else
        echo -e "  ${GREEN}✓ Found $usage_count usage(s)${NC}"
        echo "  → Status: ACTIVELY USED"
    fi
    echo ""
}

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📦 HIGH CONFIDENCE - Likely Unused"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

check_dependency "tiktoken-rs" "use tiktoken_rs" 90
check_dependency "spider (workspace)" "^use spider::" 85
check_dependency "governor" "use governor" 75

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔍 MEDIUM CONFIDENCE - Conditionally Used"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

check_dependency "csv" "use csv::" 60
check_dependency "xml" "use xml::" 50
check_dependency "robotstxt" "use robotstxt" 40

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ LOW CONFIDENCE - Likely Still Used"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

check_dependency "wasmtime" "use wasmtime::" 20
check_dependency "prometheus" "use prometheus" 10
check_dependency "redis" "use.*redis" 5

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 SUMMARY"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Total checks performed: $TOTAL_CHECKS"
echo -e "${RED}Likely unused (≥75% confidence): $LIKELY_UNUSED${NC}"
echo -e "${YELLOW}Needs review (<75% confidence): $NEEDS_REVIEW${NC}"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔧 ADDITIONAL CHECKS"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check for duplicate dependencies across crates
echo "Checking for duplicate dependencies across crates..."
echo ""

deps_to_check=("robotstxt" "xml" "scraper" "reqwest")

for dep in "${deps_to_check[@]}"; do
    count=$(grep -r "\"$dep\"" crates --include="Cargo.toml" | wc -l)
    if [ "$count" -gt 1 ]; then
        echo -e "${YELLOW}⚠️  $dep appears in $count crates${NC}"
        grep -r "\"$dep\"" crates --include="Cargo.toml" | sed 's/^/  /'
        echo ""
    fi
done

# Check for stale riptide-core references
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🧹 CLEANUP RECOMMENDATIONS"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

core_refs=$(grep -r "use riptide_core" crates --include="*.rs" 2>/dev/null | wc -l)
if [ "$core_refs" -gt 0 ]; then
    echo -e "${YELLOW}⚠️  Found $core_refs stale riptide_core references in source files${NC}"
    echo "  → These should be updated to use specialized crates"
fi

doc_refs=$(grep -r "riptide-core" docs --include="*.md" 2>/dev/null | wc -l)
if [ "$doc_refs" -gt 0 ]; then
    echo -e "${YELLOW}⚠️  Found $doc_refs riptide-core references in documentation${NC}"
    echo "  → Documentation should be updated to reflect new architecture"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📝 NEXT STEPS"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "1. Review high-confidence unused dependencies (≥75%)"
echo "2. Run 'cargo +nightly udeps' for automated detection"
echo "3. Test removal with: cargo test --all-features"
echo "4. Update feature flags to mark optional deps properly"
echo "5. Consolidate duplicate dependencies where possible"
echo ""
echo "For detailed analysis, see: docs/unused-deps-analysis.md"
echo ""

echo "✅ Audit complete!"
echo ""
