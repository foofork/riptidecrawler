#!/bin/bash
set -e

echo "🔄 Updating golden test baselines..."
cd "$(dirname "$0")/../wasm/riptide-extractor-wasm"

# Run tests with UPDATE_BASELINES flag
UPDATE_BASELINES=1 cargo test --test test_wasm_extractor -- --nocapture

echo ""
echo "✅ Baselines updated successfully!"
echo "📝 Review changes with: git diff tests/golden/snapshots/"
echo ""
echo "💡 Tip: Run 'cargo test --test test_wasm_extractor' to verify the updated baselines"
