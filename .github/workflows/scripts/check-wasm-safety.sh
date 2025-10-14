#!/bin/bash
# Check WASM bindings for required safety documentation

set -e

echo "🔍 Checking WASM bindings for safety documentation..."

# Find all bindings.rs files
BINDINGS_FILES=$(find . -name 'bindings.rs' -type f 2>/dev/null || true)

if [ -z "$BINDINGS_FILES" ]; then
  echo "⚠️  No bindings.rs files found - skipping WASM safety check"
  exit 0
fi

VIOLATIONS=0

while IFS= read -r file; do
  if [ -z "$file" ]; then
    continue
  fi

  echo "📄 Checking $file"

  # Check for the required WASM FFI safety comment
  if grep -q "SAFETY: Required for WASM component model FFI" "$file"; then
    echo "  ✅ Has required WASM FFI safety documentation"
  elif grep -q "SAFETY:" "$file"; then
    echo "  ⚠️  Has SAFETY comments but missing WASM FFI specific documentation"
    echo "     Expected: '// SAFETY: Required for WASM component model FFI'"
    VIOLATIONS=$((VIOLATIONS + 1))
  else
    echo "  ❌ Missing SAFETY documentation"
    VIOLATIONS=$((VIOLATIONS + 1))
  fi

  # Check for unsafe blocks in bindings
  UNSAFE_COUNT=$(rg -c 'unsafe' --type rust "$file" 2>/dev/null || echo "0")
  echo "  📊 Contains $UNSAFE_COUNT unsafe references"

done <<< "$BINDINGS_FILES"

echo ""
echo "📊 Summary:"
echo "  Total bindings files: $(echo "$BINDINGS_FILES" | grep -c . || echo 0)"
echo "  Violations: $VIOLATIONS"

if [ "$VIOLATIONS" -gt "0" ]; then
  echo ""
  echo "❌ Found $VIOLATIONS bindings.rs files with missing or incomplete safety documentation"
  echo "All WASM bindings must document: 'SAFETY: Required for WASM component model FFI'"
  exit 1
fi

echo "✅ All WASM bindings are properly documented"
exit 0
