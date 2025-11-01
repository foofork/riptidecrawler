#!/bin/bash
# Extract unused/dead/unreachable code warnings from cargo JSON outputs

set -euo pipefail

OUTPUT_FILE=".unused_all.json"

# Combine all JSON files and extract unused warnings
cat .check.json .clippy.json .check_nofeat.json .check_allfeat.json .check_tests.json 2>/dev/null | \
  jq -c 'select(.reason=="compiler-message" and .message.code!=null)' | \
  jq -c '{
    code: .message.code.code,
    level: .message.level,
    file: (.message.spans[0].file_name // ""),
    line: (.message.spans[0].line_start // 0),
    column: (.message.spans[0].column_start // 0),
    msg: .message.message,
    span_text: ((.message.spans[0].text[0].text // "") | gsub("^\\s+"; "")),
    rendered: (.message.rendered // "")
  }' | \
  jq -c 'select(.code | test("^(unused_|dead_code|unused_imports|unused_must_use|unreachable_code)$"))' | \
  jq -s 'unique_by({file, line, code})' > "$OUTPUT_FILE"

# Report results
TOTAL=$(jq 'length' "$OUTPUT_FILE")
echo "✓ Extracted $TOTAL unique unused/dead code findings to $OUTPUT_FILE"

# Summary by code type
echo ""
echo "Breakdown by warning type:"
jq -r 'group_by(.code) | map({code: .[0].code, count: length}) | .[] | "  \(.code): \(.count)"' "$OUTPUT_FILE"
