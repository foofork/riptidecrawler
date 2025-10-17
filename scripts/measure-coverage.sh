#!/bin/bash
# Per-crate coverage measurement to avoid timeouts
# Part of Phase 1 Week 2 baseline measurement
set -e

CRATES=(
    "riptide-core"
    "riptide-extraction"
    "riptide-api"
    "riptide-headless"
    "riptide-cli"
    "riptide-types"
    "riptide-stealth"
    "riptide-pdf"
    "riptide-spider"
    "riptide-persistence"
    "riptide-workers"
    "riptide-performance"
    "riptide-intelligence"
)

OUTPUT_DIR="./coverage"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT_DIR="$OUTPUT_DIR/report_$TIMESTAMP"
mkdir -p "$REPORT_DIR"

echo "📊 Measuring coverage for ${#CRATES[@]} crates..."
echo "Output directory: $REPORT_DIR"
echo ""

# Summary file
SUMMARY_FILE="$REPORT_DIR/coverage-summary.txt"
echo "Coverage Baseline Report - $(date)" > "$SUMMARY_FILE"
echo "======================================" >> "$SUMMARY_FILE"
echo "" >> "$SUMMARY_FILE"

# Track overall stats
TOTAL_CRATES=0
SUCCESSFUL_CRATES=0
FAILED_CRATES=0

for crate in "${CRATES[@]}"; do
    TOTAL_CRATES=$((TOTAL_CRATES + 1))
    echo "[$TOTAL_CRATES/${#CRATES[@]}] 🔍 Running coverage for $crate..."

    CRATE_OUTPUT_DIR="$REPORT_DIR/$crate"
    mkdir -p "$CRATE_OUTPUT_DIR"

    # Run tarpaulin with timeout
    if cargo tarpaulin \
        -p "$crate" \
        --out Html \
        --out Stdout \
        --output-dir "$CRATE_OUTPUT_DIR" \
        --timeout 300 \
        --skip-clean \
        --exclude-files 'tests/*' 'benches/*' \
        2>&1 | tee "$CRATE_OUTPUT_DIR/coverage.log"; then

        SUCCESSFUL_CRATES=$((SUCCESSFUL_CRATES + 1))

        # Extract coverage percentage from log
        COVERAGE=$(grep -oP '\d+\.\d+%' "$CRATE_OUTPUT_DIR/coverage.log" | tail -1 || echo "N/A")

        echo "✅ $crate coverage complete: $COVERAGE"
        echo "$crate: $COVERAGE" >> "$SUMMARY_FILE"
    else
        FAILED_CRATES=$((FAILED_CRATES + 1))
        echo "❌ $crate coverage failed"
        echo "$crate: FAILED" >> "$SUMMARY_FILE"
    fi

    echo ""
done

# Write summary
echo "" >> "$SUMMARY_FILE"
echo "Summary:" >> "$SUMMARY_FILE"
echo "--------" >> "$SUMMARY_FILE"
echo "Total crates: $TOTAL_CRATES" >> "$SUMMARY_FILE"
echo "Successful: $SUCCESSFUL_CRATES" >> "$SUMMARY_FILE"
echo "Failed: $FAILED_CRATES" >> "$SUMMARY_FILE"

# Print summary
echo "========================================"
echo "📋 Coverage Measurement Complete"
echo "========================================"
echo "Total crates: $TOTAL_CRATES"
echo "Successful: $SUCCESSFUL_CRATES"
echo "Failed: $FAILED_CRATES"
echo ""
echo "Reports saved to: $REPORT_DIR"
echo "Summary: $SUMMARY_FILE"
echo ""

# Display coverage summary
if [ -f "$SUMMARY_FILE" ]; then
    echo "Coverage Results:"
    cat "$SUMMARY_FILE"
fi

# Create a quick index.html for easy navigation
INDEX_FILE="$REPORT_DIR/index.html"
cat > "$INDEX_FILE" << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>RipTide Coverage Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        h1 { color: #333; }
        .crate { margin: 20px 0; padding: 15px; border: 1px solid #ddd; }
        .success { background-color: #d4edda; }
        .failed { background-color: #f8d7da; }
        a { color: #007bff; text-decoration: none; }
        a:hover { text-decoration: underline; }
    </style>
</head>
<body>
    <h1>RipTide Coverage Report</h1>
    <p>Generated: $(date)</p>
EOF

for crate in "${CRATES[@]}"; do
    if [ -f "$REPORT_DIR/$crate/index.html" ]; then
        echo "    <div class='crate success'>" >> "$INDEX_FILE"
        echo "        <h3>$crate</h3>" >> "$INDEX_FILE"
        echo "        <a href='$crate/index.html'>View Coverage Report</a>" >> "$INDEX_FILE"
        echo "    </div>" >> "$INDEX_FILE"
    elif [ -d "$REPORT_DIR/$crate" ]; then
        echo "    <div class='crate failed'>" >> "$INDEX_FILE"
        echo "        <h3>$crate</h3>" >> "$INDEX_FILE"
        echo "        <p>Coverage generation failed - <a href='$crate/coverage.log'>View Log</a></p>" >> "$INDEX_FILE"
        echo "    </div>" >> "$INDEX_FILE"
    fi
done

cat >> "$INDEX_FILE" << 'EOF'
</body>
</html>
EOF

echo "Index page: $INDEX_FILE"
echo ""
echo "✅ Coverage measurement complete!"
