#!/bin/bash
# Test PDF extraction on accessible URLs from 40_tables_pdfs.yml
# Saves results to eval/results/pdf_test.csv

set -e

RIPTIDE="/workspaces/eventmesh/target/x86_64-unknown-linux-gnu/release/riptide"
RESULTS_DIR="/workspaces/eventmesh/eval/results"
OUTPUT_CSV="${RESULTS_DIR}/pdf_test.csv"
TEMP_DIR="${RESULTS_DIR}/pdf_temp"

# Create directories
mkdir -p "$RESULTS_DIR" "$TEMP_DIR"

# Initialize CSV with headers
echo "url,name,status,tables_extracted,text_length,error_message,extraction_time" > "$OUTPUT_CSV"

# Test URLs (excluding OECD which is blocked)
declare -a URLS=(
  "https://assets.publishing.service.gov.uk/media/6722120210b0d582ee8c48c0/Autumn_Budget_2024__print_.pdf|UK Autumn Budget 2024 (print)"
  "https://assets.publishing.service.gov.uk/media/6721d2c54da1c0d41942a8d2/Policy_Costing_Document_-_Autumn_Budget_2024.pdf|UK Autumn Budget 2024 — Policy Costings"
  "https://hilversum.nl/sites/default/files/documents/Vereiste%20informatie%20in%20activiteitenplan%2C%20begroting%20en%20dekkingsplan%20.pdf|Hilversum — Vereiste info"
)

echo "========================================"
echo "PDF Extraction Test"
echo "========================================"
echo ""

# Process each URL
for item in "${URLS[@]}"; do
  IFS='|' read -r url name <<< "$item"

  echo "----------------------------------------"
  echo "Testing: $name"
  echo "URL: $url"
  echo "----------------------------------------"

  # Variables for results
  status="FAILED"
  tables_count=0
  text_length=0
  error_msg=""
  start_time=$(date +%s)

  # Create unique output file
  output_file="${TEMP_DIR}/$(echo "$name" | tr ' ' '_' | tr -cd '[:alnum:]_-').json"

  # Try extraction with tables flag
  if timeout 60 "$RIPTIDE" pdf extract \
    --input "$url" \
    --format json \
    --tables \
    --output "$TEMP_DIR" > "$output_file" 2>&1; then

    status="SUCCESS"

    # Parse JSON to count tables and text length
    if [ -f "$output_file" ]; then
      # Count tables in JSON output
      tables_count=$(grep -o '"tables"' "$output_file" | wc -l || echo 0)

      # Get text length
      text_length=$(wc -c < "$output_file" || echo 0)

      echo "✓ Extraction successful"
      echo "  Tables found: $tables_count"
      echo "  Output size: $text_length bytes"
    fi
  else
    error_msg="Extraction failed or timed out"
    echo "✗ $error_msg"

    # Try to capture error details
    if [ -f "$output_file" ]; then
      error_msg=$(head -n 5 "$output_file" | tr '\n' ' ' | tr ',' ';')
    fi
  fi

  # Calculate extraction time
  end_time=$(date +%s)
  extraction_time=$((end_time - start_time))

  # Escape name and error for CSV
  name_escaped=$(echo "$name" | sed 's/"/""/g')
  error_escaped=$(echo "$error_msg" | sed 's/"/""/g')

  # Append to CSV
  echo "\"$url\",\"$name_escaped\",\"$status\",$tables_count,$text_length,\"$error_escaped\",$extraction_time" >> "$OUTPUT_CSV"

  echo ""
done

echo "========================================"
echo "Test Summary"
echo "========================================"
echo ""

# Generate summary
total_tests=$(tail -n +2 "$OUTPUT_CSV" | wc -l)
successful=$(grep -c ",\"SUCCESS\"," "$OUTPUT_CSV" || echo 0)
failed=$((total_tests - successful))

echo "Total PDFs tested: $total_tests"
echo "Successful: $successful"
echo "Failed: $failed"
echo ""
echo "Results saved to: $OUTPUT_CSV"
echo ""

# Display results table
echo "Detailed Results:"
echo "----------------"
column -t -s ',' "$OUTPUT_CSV" || cat "$OUTPUT_CSV"

# Cleanup temp files (optional)
# rm -rf "$TEMP_DIR"

exit 0
