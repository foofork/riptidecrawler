#!/bin/bash
# Phase 4 Performance Validation Runner
#
# Usage:
#   ./scripts/run-phase4-validation.sh [iterations] [output-path]
#
# Examples:
#   ./scripts/run-phase4-validation.sh 100 /tmp/results.json
#   ./scripts/run-phase4-validation.sh  # Uses defaults

set -e

ITERATIONS=${1:-100}
OUTPUT=${2:-"/workspaces/eventmesh/phase4-results.json"}

echo "🚀 Phase 4 Performance Validation"
echo "=================================="
echo "Iterations: $ITERATIONS"
echo "Output: $OUTPUT"
echo ""

# Build the validator
echo "📦 Building validator..."
cargo build --release --bin phase4-validator -p riptide-performance

# Run the benchmarks
echo ""
echo "🔍 Running benchmarks..."
cargo run --release --bin phase4-validator -p riptide-performance -- \
  --iterations "$ITERATIONS" \
  --output "$OUTPUT"

# Check exit code
if [ $? -eq 0 ]; then
  echo ""
  echo "✅ Validation PASSED"
  echo ""
  echo "📊 Results saved to: $OUTPUT"
  echo "View results: cat $OUTPUT | jq ."
  exit 0
else
  echo ""
  echo "❌ Validation FAILED"
  exit 1
fi
