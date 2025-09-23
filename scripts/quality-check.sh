#!/bin/bash
set -e

echo "🔍 Quality Check Runner"
echo "======================="

# Fast quality checks
echo "📝 Running clippy (fast mode)..."
cargo clippy --workspace --all-targets -- \
    -W clippy::all \
    -D warnings \
    --cap-lints warn

echo "🎨 Checking formatting..."
cargo fmt --all -- --check

echo "🔗 Checking dependencies..."
cargo tree --duplicates --workspace | head -20

echo "✅ Quality checks passed!"