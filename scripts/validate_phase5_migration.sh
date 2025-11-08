#!/bin/bash
# Phase 5 Migration Validation Script
# Validates API layer separation and facade migration

set -e

echo "🔍 Validating Phase 5 Migration..."
echo ""

FAILED=0

# 1. Check orchestration logic removed from API
echo "📋 Step 1: Checking API handlers for orchestration logic..."
if grep -r "execute_single\|execute_batch\|analyze_content\|process_pdf_content\|extract_with_headless" crates/riptide-api/src/handlers/ 2>/dev/null; then
    echo "❌ FAIL: Orchestration logic still found in API handlers"
    FAILED=1
else
    echo "✅ PASS: No orchestration logic in API handlers"
fi
echo ""

# 2. Check PipelineOrchestrator only as thin wrapper
echo "📋 Step 2: Checking PipelineOrchestrator is thin wrapper..."
API_PIPELINE_LOC=$(wc -l < crates/riptide-api/src/pipeline.rs 2>/dev/null || echo 0)
if [ "$API_PIPELINE_LOC" -gt 100 ]; then
    echo "⚠️  WARNING: pipeline.rs still has $API_PIPELINE_LOC lines (target: <100)"
    FAILED=1
else
    echo "✅ PASS: pipeline.rs is thin ($API_PIPELINE_LOC lines)"
fi
echo ""

# 3. Check correct dependency direction
echo "📋 Step 3: Checking dependency direction..."
if cargo tree -p riptide-facade --edges normal 2>/dev/null | grep -q "riptide-api v"; then
    echo "❌ FAIL: Circular dependency detected (Facade → API)"
    FAILED=1
else
    echo "✅ PASS: No circular dependency"
fi
echo ""

# 4. Check API depends on facade
echo "📋 Step 4: Checking API → Facade dependency exists..."
if ! cargo tree -p riptide-api --edges normal 2>/dev/null | grep -q "riptide-facade v"; then
    echo "❌ FAIL: API should depend on Facade"
    FAILED=1
else
    echo "✅ PASS: API depends on Facade correctly"
fi
echo ""

# 5. Check API LOC reduction
echo "📋 Step 5: Checking API LOC reduction..."
API_LOC=$(find crates/riptide-api/src -name "*.rs" -exec wc -l {} + 2>/dev/null | tail -1 | awk '{print $1}')
echo "   Current API LOC: $API_LOC"
if [ "$API_LOC" -gt 30000 ]; then
    echo "⚠️  WARNING: API LOC still high (target: <30,000)"
else
    echo "✅ PASS: API LOC reduced successfully"
fi
echo ""

# 6. Check facade has orchestration logic
echo "📋 Step 6: Checking facade has orchestration logic..."
if [ -f "crates/riptide-facade/src/facades/crawl_facade.rs" ]; then
    CRAWL_FACADE_LOC=$(wc -l < crates/riptide-facade/src/facades/crawl_facade.rs)
    echo "   CrawlFacade LOC: $CRAWL_FACADE_LOC"
    if [ "$CRAWL_FACADE_LOC" -lt 500 ]; then
        echo "⚠️  WARNING: CrawlFacade seems incomplete ($CRAWL_FACADE_LOC lines, expected ~1,100)"
    else
        echo "✅ PASS: CrawlFacade has orchestration logic"
    fi
else
    echo "❌ FAIL: crawl_facade.rs not found"
    FAILED=1
fi
echo ""

# 7. Check no JSON blobs in facade public API
echo "📋 Step 7: Checking no serde_json::Value in facade public API..."
JSON_VIOLATIONS=$(grep -r "pub fn.*serde_json::Value" crates/riptide-facade/src/ 2>/dev/null | wc -l)
if [ "$JSON_VIOLATIONS" -gt 0 ]; then
    echo "⚠️  WARNING: Found $JSON_VIOLATIONS public functions returning JSON blobs"
    grep -r "pub fn.*serde_json::Value" crates/riptide-facade/src/ 2>/dev/null || true
else
    echo "✅ PASS: No JSON blobs in facade public API"
fi
echo ""

# 8. Run compilation check
echo "📋 Step 8: Running compilation check..."
if cargo check --workspace --all-features > /dev/null 2>&1; then
    echo "✅ PASS: Workspace compiles successfully"
else
    echo "❌ FAIL: Compilation errors detected"
    FAILED=1
fi
echo ""

# 9. Run tests
echo "📋 Step 9: Running workspace tests..."
if cargo test --workspace --all-features > /dev/null 2>&1; then
    echo "✅ PASS: All tests pass"
else
    echo "❌ FAIL: Test failures detected"
    FAILED=1
fi
echo ""

# 10. Check clippy warnings
echo "📋 Step 10: Running clippy..."
if cargo clippy --workspace -- -D warnings > /dev/null 2>&1; then
    echo "✅ PASS: No clippy warnings"
else
    echo "⚠️  WARNING: Clippy warnings detected"
fi
echo ""

# Summary
echo "════════════════════════════════════════"
if [ $FAILED -eq 0 ]; then
    echo "✅ Phase 5 Migration Validation: PASSED"
    echo "════════════════════════════════════════"
    exit 0
else
    echo "❌ Phase 5 Migration Validation: FAILED"
    echo "════════════════════════════════════════"
    echo ""
    echo "Please fix the issues above before proceeding."
    exit 1
fi
