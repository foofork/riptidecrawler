#!/bin/bash
# Script to check for missing doc comments on public items

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Counters
total_public_items=0
documented_items=0
undocumented_items=0

# Output file
OUTPUT_FILE="/workspaces/eventmesh/docs/missing_doc_comments.txt"
> "$OUTPUT_FILE"

echo "Checking for missing doc comments in Rust source files..."
echo "============================================================"
echo ""

# Find all .rs files in crates (excluding tests and benches)
find /workspaces/eventmesh/crates -name "*.rs" -type f \
  | grep -v "/tests/" \
  | grep -v "/benches/" \
  | grep -v "test_" \
  | sort | while read -r file; do

  # Check if file has any public items
  if grep -q "^pub " "$file" || grep -q "^[[:space:]]*pub " "$file"; then

    # Extract public items with their line numbers and check for doc comments
    grep -n "^[[:space:]]*pub " "$file" | while IFS=: read -r line_num line_content; do

      # Skip test modules and test attributes
      if echo "$line_content" | grep -q "mod tests" || echo "$line_content" | grep -q "#\[test\]"; then
        continue
      fi

      # Get the previous line to check for doc comment
      prev_line=$((line_num - 1))
      doc_comment=""
      if [ $prev_line -gt 0 ]; then
        doc_comment=$(sed -n "${prev_line}p" "$file" | grep -E "^[[:space:]]*///" || true)
      fi

      total_public_items=$((total_public_items + 1))

      # Check if doc comment exists
      if [ -z "$doc_comment" ]; then
        # Also check 2-3 lines above for multi-line doc comments
        prev2_line=$((line_num - 2))
        prev3_line=$((line_num - 3))
        doc_comment2=""
        doc_comment3=""
        if [ $prev2_line -gt 0 ]; then
          doc_comment2=$(sed -n "${prev2_line}p" "$file" | grep -E "^[[:space:]]*///" || true)
        fi
        if [ $prev3_line -gt 0 ]; then
          doc_comment3=$(sed -n "${prev3_line}p" "$file" | grep -E "^[[:space:]]*///" || true)
        fi

        if [ -z "$doc_comment2" ] && [ -z "$doc_comment3" ]; then
          undocumented_items=$((undocumented_items + 1))
          echo "$file:$line_num:$line_content" >> "$OUTPUT_FILE"
        else
          documented_items=$((documented_items + 1))
        fi
      else
        documented_items=$((documented_items + 1))
      fi
    done
  fi
done

# Calculate documentation coverage
if [ $total_public_items -gt 0 ]; then
  coverage=$((documented_items * 100 / total_public_items))
else
  coverage=100
fi

# Print summary
echo ""
echo "Documentation Coverage Summary"
echo "=============================="
echo "Total public items:     $total_public_items"
echo "Documented items:       $documented_items"
echo "Undocumented items:     $undocumented_items"
echo "Coverage:               ${coverage}%"
echo ""

if [ $undocumented_items -gt 0 ]; then
  echo -e "${YELLOW}Found $undocumented_items public items without doc comments${NC}"
  echo "See $OUTPUT_FILE for details"
  echo ""
  echo "Sample of undocumented items:"
  head -20 "$OUTPUT_FILE"
else
  echo -e "${GREEN}All public items have doc comments!${NC}"
fi
