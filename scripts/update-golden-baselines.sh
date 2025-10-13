#!/bin/bash
set -e

echo "🔄 Updating golden test baselines..."
cd "$(dirname "$0")/../wasm/riptide-extractor-wasm"

# Run golden tests with UPDATE_BASELINES flag
UPDATE_BASELINES=1 cargo test --test mod golden::tests::test_all_golden_tests -- --nocapture

echo ""
echo "✅ Baselines updated successfully!"
echo "📝 Review changes with: git diff tests/golden/snapshots/"
echo ""
echo "💡 Tip: Run 'cargo test --test mod golden::tests' to verify the updated baselines"
