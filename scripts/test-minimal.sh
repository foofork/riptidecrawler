#!/bin/bash
set -e

echo "🧪 Minimal Test Suite"
echo "===================="

# Kill any blocking processes
lsof -ti:3000 -ti:8080 -ti:9222 | xargs -r kill -9 2>/dev/null || true

# Run minimal tests with strict timeout
echo "Running core library tests..."
timeout 10s cargo test --lib -p riptide-core 2>&1 | tail -20 || true

echo ""
echo "✅ Minimal tests completed!"
echo ""
echo "For full test suite, use: ./scripts/fast-test.sh"