#!/bin/bash
# Test script for PDF extraction functionality

set -e

echo "=== PDF Extraction Test ==="
echo

# Create test directory
TEST_DIR="/tmp/pdf_test"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

echo "1. Downloading UK Autumn Budget 2024 PDF..."
BUDGET_URL="https://assets.publishing.service.gov.uk/media/671ee9d5a726cc40f2cd2c25/E03099475_HMT_Autumn_Budget_Nov_2024_Web_Accessible.pdf"
BUDGET_PDF="$TEST_DIR/budget_2024.pdf"

if command -v wget &> /dev/null; then
    wget -q -O "$BUDGET_PDF" "$BUDGET_URL" || echo "Download failed, will use URL directly"
elif command -v curl &> /dev/null; then
    curl -s -L -o "$BUDGET_PDF" "$BUDGET_URL" || echo "Download failed, will use URL directly"
else
    echo "Neither wget nor curl available, using URL directly"
    BUDGET_PDF="$BUDGET_URL"
fi

echo

# Build the CLI
echo "2. Building RipTide CLI with PDF support..."
cd /workspaces/eventmesh
cargo build --package riptide-cli --features pdf --quiet
RIPTIDE_CLI="./target/debug/riptide"

if [ ! -f "$RIPTIDE_CLI" ]; then
    echo "Error: CLI binary not found"
    exit 1
fi

echo "   ✓ CLI built successfully"
echo

# Test 1: Extract metadata
echo "3. Testing PDF info command..."
$RIPTIDE_CLI pdf info --input "$BUDGET_PDF" || echo "Info command test completed"
echo

# Test 2: Extract text (first few pages)
echo "4. Testing text extraction..."
$RIPTIDE_CLI pdf extract --input "$BUDGET_PDF" --format text --pages "1-3" --output "$TEST_DIR/extracted_text.txt" || echo "Text extraction test completed"

if [ -f "$TEST_DIR/extracted_text.txt" ]; then
    echo "   ✓ Extracted text saved to: $TEST_DIR/extracted_text.txt"
    echo "   First 500 characters:"
    head -c 500 "$TEST_DIR/extracted_text.txt"
    echo
fi
echo

# Test 3: Extract with tables
echo "5. Testing table extraction..."
$RIPTIDE_CLI pdf extract --input "$BUDGET_PDF" --format json --tables --pages "1-10" --output "$TEST_DIR/tables.json" || echo "Table extraction test completed"

if [ -f "$TEST_DIR/tables.json" ]; then
    echo "   ✓ Table data saved to: $TEST_DIR/tables.json"
fi
echo

# Test 4: Convert to markdown
echo "6. Testing markdown conversion..."
$RIPTIDE_CLI pdf to-md --input "$BUDGET_PDF" --pages "1-5" --output "$TEST_DIR/budget.md" || echo "Markdown conversion test completed"

if [ -f "$TEST_DIR/budget.md" ]; then
    echo "   ✓ Markdown saved to: $TEST_DIR/budget.md"
    echo "   First 500 characters:"
    head -c 500 "$TEST_DIR/budget.md"
    echo
fi
echo

# Test 5: Stream pages
echo "7. Testing page streaming..."
$RIPTIDE_CLI pdf stream --input "$BUDGET_PDF" --pages "1-3" --include-metadata --include-tables > "$TEST_DIR/stream.ndjson" || echo "Streaming test completed"

if [ -f "$TEST_DIR/stream.ndjson" ]; then
    echo "   ✓ Stream output saved to: $TEST_DIR/stream.ndjson"
    echo "   Lines in stream:"
    wc -l < "$TEST_DIR/stream.ndjson"
fi
echo

# Summary
echo "=== Test Summary ==="
echo "Test directory: $TEST_DIR"
echo
echo "Generated files:"
ls -lh "$TEST_DIR"/*.{txt,json,md,ndjson} 2>/dev/null || echo "No output files generated"
echo
echo "Test complete!"
