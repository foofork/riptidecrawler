#!/bin/bash
# RipTide Content Extraction Pipeline Example
# This script demonstrates a complete extraction workflow with error handling

set -euo pipefail  # Exit on error, undefined vars, pipe failures

# ===========================
# Configuration
# ===========================

# Load environment variables
if [ -f .env ]; then
    source .env
else
    echo "Warning: .env file not found. Using defaults."
fi

# Script configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUTPUT_BASE="${RIPTIDE_OUTPUT_DIR:-./pipeline-output}"
LOG_FILE="${OUTPUT_BASE}/pipeline.log"
ERROR_LOG="${OUTPUT_BASE}/errors.log"

# Pipeline settings
MAX_RETRIES=3
RETRY_DELAY=5
BATCH_SIZE=10

# ===========================
# Helper Functions
# ===========================

log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $*" | tee -a "$LOG_FILE"
}

error() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] ERROR: $*" | tee -a "$ERROR_LOG" >&2
}

check_dependencies() {
    log "Checking dependencies..."

    # Check if riptide CLI is available
    if ! command -v riptide &> /dev/null; then
        error "riptide CLI not found. Please install it first."
        exit 1
    fi

    # Check API server (if using API-First mode)
    if [ "${RIPTIDE_DEFAULT_MODE:-api-first}" = "api-first" ]; then
        if ! curl -sf "${RIPTIDE_API_URL:-http://localhost:8080}/healthz" > /dev/null; then
            error "API server not reachable at ${RIPTIDE_API_URL:-http://localhost:8080}"
            error "Start the API server or use --direct mode"
            exit 1
        fi
        log "API server is healthy"
    fi
}

setup_directories() {
    log "Setting up directories..."
    mkdir -p "$OUTPUT_BASE"/{extractions,processed,failed}
    mkdir -p "$(dirname "$LOG_FILE")"
}

extract_url() {
    local url="$1"
    local output_file="$2"
    local attempt=1

    while [ $attempt -le $MAX_RETRIES ]; do
        log "Extracting $url (attempt $attempt/$MAX_RETRIES)..."

        if riptide extract \
            --url "$url" \
            --output-dir "$OUTPUT_BASE/extractions" \
            -f "$output_file" \
            --metadata \
            --show-confidence; then
            log "✓ Successfully extracted: $url"
            return 0
        else
            error "✗ Failed to extract: $url (attempt $attempt)"
            attempt=$((attempt + 1))
            [ $attempt -le $MAX_RETRIES ] && sleep $RETRY_DELAY
        fi
    done

    error "✗ Failed after $MAX_RETRIES attempts: $url"
    echo "$url" >> "$OUTPUT_BASE/failed/urls.txt"
    return 1
}

process_extraction() {
    local file="$1"
    log "Processing extracted content: $file"

    # Example: Convert to different format, validate, etc.
    # This is a placeholder for your processing logic

    if [ -f "$file" ]; then
        # Example: Count words
        local word_count=$(wc -w < "$file")
        log "  - Word count: $word_count"

        # Example: Extract metadata
        if grep -q "^---" "$file"; then
            log "  - Found frontmatter metadata"
        fi

        # Move to processed directory
        mv "$file" "$OUTPUT_BASE/processed/"
        log "  - Moved to processed directory"
    else
        error "File not found: $file"
        return 1
    fi
}

batch_extract() {
    local urls_file="$1"

    if [ ! -f "$urls_file" ]; then
        error "URLs file not found: $urls_file"
        exit 1
    fi

    log "Starting batch extraction from $urls_file"

    local count=0
    local success=0
    local failed=0

    while IFS= read -r url; do
        # Skip empty lines and comments
        [[ -z "$url" || "$url" =~ ^# ]] && continue

        count=$((count + 1))
        local output_file="extraction-${count}.md"

        if extract_url "$url" "$output_file"; then
            success=$((success + 1))

            # Process the extracted content
            if [ -f "$OUTPUT_BASE/extractions/$output_file" ]; then
                process_extraction "$OUTPUT_BASE/extractions/$output_file"
            fi
        else
            failed=$((failed + 1))
        fi

        # Progress update
        if [ $((count % $BATCH_SIZE)) -eq 0 ]; then
            log "Progress: $count URLs processed ($success success, $failed failed)"
        fi
    done < "$urls_file"

    log "Batch extraction complete: $count total, $success success, $failed failed"
}

generate_report() {
    log "Generating pipeline report..."

    local report_file="$OUTPUT_BASE/report.txt"

    cat > "$report_file" << EOF
RipTide Extraction Pipeline Report
Generated: $(date)
=====================================

Configuration:
  Output Directory: $OUTPUT_BASE
  API Mode: ${RIPTIDE_DEFAULT_MODE:-api-first}
  Max Retries: $MAX_RETRIES

Statistics:
  Total Extractions: $(find "$OUTPUT_BASE/extractions" -type f -name "*.md" 2>/dev/null | wc -l)
  Processed: $(find "$OUTPUT_BASE/processed" -type f -name "*.md" 2>/dev/null | wc -l)
  Failed: $([ -f "$OUTPUT_BASE/failed/urls.txt" ] && wc -l < "$OUTPUT_BASE/failed/urls.txt" || echo 0)

Output Locations:
  Extractions: $OUTPUT_BASE/extractions/
  Processed: $OUTPUT_BASE/processed/
  Failed URLs: $OUTPUT_BASE/failed/urls.txt
  Log File: $LOG_FILE
  Error Log: $ERROR_LOG

EOF

    cat "$report_file"
    log "Report saved to: $report_file"
}

cleanup() {
    log "Cleaning up temporary files..."
    # Add cleanup logic here if needed
}

# ===========================
# Main Pipeline
# ===========================

main() {
    log "=== RipTide Extraction Pipeline Started ==="

    # Parse arguments
    if [ $# -eq 0 ]; then
        echo "Usage: $0 <urls-file>"
        echo "       $0 --url <single-url>"
        exit 1
    fi

    # Setup
    check_dependencies
    setup_directories

    # Execute based on mode
    if [ "$1" = "--url" ]; then
        # Single URL extraction
        if [ $# -lt 2 ]; then
            error "Missing URL argument"
            exit 1
        fi
        extract_url "$2" "extraction.md"
        if [ -f "$OUTPUT_BASE/extractions/extraction.md" ]; then
            process_extraction "$OUTPUT_BASE/extractions/extraction.md"
        fi
    else
        # Batch extraction from file
        batch_extract "$1"
    fi

    # Generate report
    generate_report

    # Cleanup
    cleanup

    log "=== RipTide Extraction Pipeline Completed ==="
}

# Trap errors and cleanup
trap cleanup EXIT

# Run main pipeline
main "$@"
