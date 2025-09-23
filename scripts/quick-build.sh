#!/bin/bash
set -e

echo "⚡ Quick Build Runner"
echo "===================="

# Kill blocking processes
echo "🔧 Cleaning up ports..."
lsof -ti:3000 | xargs -r kill -9 2>/dev/null || true
lsof -ti:8080 | xargs -r kill -9 2>/dev/null || true
lsof -ti:9222 | xargs -r kill -9 2>/dev/null || true

# Use sccache if available
if command -v sccache &> /dev/null; then
    export RUSTC_WRAPPER=sccache
    echo "📦 Using sccache for faster builds"
fi

# Build with optimizations
echo "🏗️ Building project..."
cargo build --workspace \
    --exclude riptide-extractor-wasm \
    --jobs 8

echo "✅ Build completed!"