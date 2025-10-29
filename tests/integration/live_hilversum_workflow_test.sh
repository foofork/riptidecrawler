#!/bin/bash
# Integration Test: Live Hilversum Use Case
#
# Workflow:
# 1. Use spider to discover URLs from livehilversum.nl
# 2. Extract content from each discovered URL
# 3. Verify end-to-end functionality

set -euo pipefail

# Configuration
API_BASE_URL="${API_BASE_URL:-http://localhost:8080}"
RESULTS_DIR="tests/integration/results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT_FILE="$RESULTS_DIR/live_hilversum_workflow_$TIMESTAMP.json"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Setup
mkdir -p "$RESULTS_DIR"

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_step() {
    echo -e "${YELLOW}[STEP]${NC} $1"
}

# ============================================================================
# Main Workflow
# ============================================================================

log_info "Starting Live Hilversum Workflow Integration Test"
log_info "Target: livehilversum.nl"
log_info "API: $API_BASE_URL"
echo ""

# Check API availability
log_step "1/3 Checking API availability..."
if ! curl -s -f "$API_BASE_URL/healthz" > /dev/null 2>&1; then
    log_error "API not available at $API_BASE_URL"
    exit 1
fi
log_success "API is available"
echo ""

# Step 1: Discover URLs using spider
log_step "2/3 Discovering URLs from livehilversum.nl..."

SPIDER_RESPONSE=$(curl -s -X POST "$API_BASE_URL/spider/crawl" \
    -H "Content-Type: application/json" \
    -d '{
        "seed_urls": ["https://livehilversum.nl"],
        "max_pages": 10,
        "max_depth": 2,
        "strategy": "breadth_first",
        "result_mode": "urls",
        "respect_robots": true
    }')

# Parse spider response
if ! echo "$SPIDER_RESPONSE" | jq -e '.result.discovered_urls' > /dev/null 2>&1; then
    log_error "Spider failed or returned invalid response"
    echo "$SPIDER_RESPONSE" | jq '.' || echo "$SPIDER_RESPONSE"
    exit 1
fi

DISCOVERED_URLS=$(echo "$SPIDER_RESPONSE" | jq -r '.result.discovered_urls[]')
URL_COUNT=$(echo "$SPIDER_RESPONSE" | jq '.result.discovered_urls | length')
PAGES_CRAWLED=$(echo "$SPIDER_RESPONSE" | jq '.result.pages_crawled')

log_success "Discovered $URL_COUNT URLs (crawled $PAGES_CRAWLED pages)"

# Display discovered URLs
log_info "Discovered URLs:"
echo "$DISCOVERED_URLS" | head -10 | while read -r url; do
    echo "  - $url"
done

if [ "$URL_COUNT" -gt 10 ]; then
    echo "  ... and $((URL_COUNT - 10)) more"
fi
echo ""

# Step 2: Extract content from discovered URLs
log_step "3/3 Extracting content from discovered URLs..."

EXTRACTED_COUNT=0
FAILED_COUNT=0
EXTRACTION_RESULTS="[]"

# Process URLs in batches of 5 for efficiency
URLS_ARRAY=$(echo "$DISCOVERED_URLS" | jq -R . | jq -s .)

EXTRACT_RESPONSE=$(curl -s -X POST "$API_BASE_URL/crawl" \
    -H "Content-Type: application/json" \
    -d "{
        \"urls\": $URLS_ARRAY,
        \"options\": {
            \"include_markdown\": true,
            \"include_metadata\": true
        }
    }")

# Parse extraction results
if echo "$EXTRACT_RESPONSE" | jq -e '.successful' > /dev/null 2>&1; then
    EXTRACTED_COUNT=$(echo "$EXTRACT_RESPONSE" | jq '.successful')
    FAILED_COUNT=$(echo "$EXTRACT_RESPONSE" | jq '.failed')
    EXTRACTION_RESULTS=$(echo "$EXTRACT_RESPONSE" | jq '.results')

    log_success "Extracted content from $EXTRACTED_COUNT URLs"
    if [ "$FAILED_COUNT" -gt 0 ]; then
        log_info "Failed to extract $FAILED_COUNT URLs"
    fi
else
    log_error "Extraction failed"
    echo "$EXTRACT_RESPONSE" | jq '.' || echo "$EXTRACT_RESPONSE"
    exit 1
fi

echo ""

# ============================================================================
# Generate Report
# ============================================================================

log_info "Generating workflow report..."

cat > "$REPORT_FILE" <<EOF
{
  "test_name": "Live Hilversum Workflow Integration Test",
  "timestamp": "$TIMESTAMP",
  "target_site": "livehilversum.nl",
  "workflow_steps": {
    "step_1_spider": {
      "status": "success",
      "urls_discovered": $URL_COUNT,
      "pages_crawled": $PAGES_CRAWLED,
      "strategy": "breadth_first",
      "max_pages": 10,
      "max_depth": 2
    },
    "step_2_extraction": {
      "status": "success",
      "urls_processed": $URL_COUNT,
      "successful": $EXTRACTED_COUNT,
      "failed": $FAILED_COUNT,
      "success_rate": $(awk "BEGIN {printf \"%.2f\", ($EXTRACTED_COUNT / $URL_COUNT) * 100}")
    }
  },
  "summary": {
    "workflow_status": "success",
    "total_urls_discovered": $URL_COUNT,
    "total_content_extracted": $EXTRACTED_COUNT,
    "overall_success_rate": $(awk "BEGIN {printf \"%.2f\", ($EXTRACTED_COUNT / $URL_COUNT) * 100}")
  }
}
EOF

# Display summary
echo ""
echo "=========================================="
echo "Workflow Summary"
echo "=========================================="
echo "Target Site:      livehilversum.nl"
echo "URLs Discovered:  $URL_COUNT"
echo "Pages Crawled:    $PAGES_CRAWLED"
echo "Content Extracted: $EXTRACTED_COUNT"
echo "Failed Extractions: $FAILED_COUNT"
echo "Success Rate:     $(awk "BEGIN {printf \"%.1f%%\", ($EXTRACTED_COUNT / $URL_COUNT) * 100}")"
echo ""
echo "Report: $REPORT_FILE"
echo "=========================================="

# Verify workflow success
if [ "$EXTRACTED_COUNT" -gt 0 ]; then
    log_success "✓ Workflow completed successfully!"
    log_success "✓ Spider discovered URLs"
    log_success "✓ Extraction processed content"
    log_success "✓ End-to-end integration working"
    exit 0
else
    log_error "✗ Workflow failed - no content extracted"
    exit 1
fi
