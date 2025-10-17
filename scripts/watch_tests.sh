#!/bin/bash
# Watch mode for tests during development
# Automatically re-runs tests when files change

echo "🔍 EventMesh Test Watch Mode"
echo "Watching for file changes..."
echo ""

# Check if cargo-watch is installed
if ! command -v cargo-watch &> /dev/null; then
    echo "📦 Installing cargo-watch..."
    cargo install cargo-watch
fi

# Run tests on file changes
cargo watch \
    --clear \
    --watch crates/ \
    --ignore "*.md" \
    --ignore "target/" \
    --exec "test --all" \
    --exec "clippy --all -- -D warnings"
