#!/bin/bash
# Safety audit script for checking unsafe code blocks

set -e

echo "🔍 Checking for unsafe blocks without SAFETY documentation..."

# Find all Rust files with unsafe blocks, excluding bindings.rs and tests
FILES_WITH_UNSAFE=$(rg -l 'unsafe\s*\{' --type rust \
  --glob '!*/bindings.rs' \
  --glob '!**/bindings.rs' \
  --glob '!*/tests/*' \
  --glob '!**/tests/*' \
  2>/dev/null || true)

if [ -z "$FILES_WITH_UNSAFE" ]; then
  echo "✅ No unsafe blocks found in production code"
  exit 0
fi

VIOLATIONS=0
TOTAL_UNSAFE=0

for file in $FILES_WITH_UNSAFE; do
  echo "📄 Checking $file"

  # Get line numbers of unsafe blocks
  UNSAFE_LINES=$(rg 'unsafe\s*\{' "$file" -n --only-matching --no-heading | cut -d: -f1)

  for line in $UNSAFE_LINES; do
    TOTAL_UNSAFE=$((TOTAL_UNSAFE + 1))

    # Check 3 lines before the unsafe block for SAFETY comment
    START_LINE=$((line - 3))
    if [ $START_LINE -lt 1 ]; then
      START_LINE=1
    fi

    SAFETY_COMMENT=$(sed -n "${START_LINE},${line}p" "$file" | grep -c '//.*SAFETY:' || echo "0")

    if [ "$SAFETY_COMMENT" -eq "0" ]; then
      echo "  ❌ Line $line: Missing SAFETY documentation"
      VIOLATIONS=$((VIOLATIONS + 1))
    else
      echo "  ✅ Line $line: Documented"
    fi
  done
done

echo ""
echo "📊 Summary:"
echo "  Total unsafe blocks: $TOTAL_UNSAFE"
echo "  Violations: $VIOLATIONS"

if [ "$VIOLATIONS" -gt "0" ]; then
  echo ""
  echo "❌ Found $VIOLATIONS undocumented unsafe blocks"
  echo "All unsafe blocks must have a '// SAFETY:' comment explaining why they are safe"
  exit 1
fi

echo "✅ All unsafe blocks are properly documented"
exit 0
