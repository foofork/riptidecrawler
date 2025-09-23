#!/bin/bash
set -e

echo "⚡ Quick Test Runner (Unit tests only)"
echo "===================================="

# Run only unit tests for core crates
echo "🧪 Running unit tests..."
cargo test --lib -p riptide-core -p riptide-api \
    --jobs 4 \
    -- --test-threads=4

echo "✅ Unit tests completed!"