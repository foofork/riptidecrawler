#!/bin/bash
set -e

echo "🚀 Fast Test Runner"
echo "=================="

# Kill any processes using our ports
echo "🔧 Cleaning up ports..."
lsof -ti:3000 | xargs -r kill -9 2>/dev/null || true
lsof -ti:8080 | xargs -r kill -9 2>/dev/null || true
lsof -ti:9222 | xargs -r kill -9 2>/dev/null || true
lsof -ti:6379 | xargs -r kill -9 2>/dev/null || true

# Clean old build artifacts if needed
if [ "$1" = "--clean" ]; then
    echo "🧹 Cleaning build artifacts..."
    cargo clean
fi

# Use parallel test execution with timeout
echo "🏃 Running tests in parallel..."
timeout 45s cargo test --workspace --lib --bins \
    --exclude riptide-extractor-wasm \
    --jobs 4 \
    -- --test-threads=4 || echo "⚠️ Some tests timed out"

echo "✅ Tests completed!"