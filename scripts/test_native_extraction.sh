#!/bin/bash
# Test script for native extraction verification
# Run this after resolving disk space issues

set -e

echo "=================================================="
echo "Native-First Extraction Test Suite"
echo "=================================================="

# Check disk space first
echo ""
echo "1. Checking disk space..."
df -h /workspaces/eventmesh/target || true

# Clean if needed
echo ""
echo "2. Cleaning build artifacts..."
cargo clean || true

echo ""
echo "3. Running native-first tests..."
echo "=================================================="

# Run the new test suite
echo ""
echo "Running all native-first tests..."
cargo test -p riptide-extraction --test native_first_tests -- --nocapture

echo ""
echo "=================================================="
echo "4. Running existing extraction tests..."
echo "=================================================="

# Run lib tests
cargo test -p riptide-extraction --lib

echo ""
echo "=================================================="
echo "5. Checking for regressions in dependent crates..."
echo "=================================================="

# Run API tests
echo ""
echo "Testing riptide-api..."
cargo test -p riptide-api --lib

# Run CLI tests
echo ""
echo "Testing riptide-cli..."
cargo test -p riptide-cli --lib

echo ""
echo "=================================================="
echo "6. Performance benchmark (optional)..."
echo "=================================================="

# Run performance test specifically
cargo test -p riptide-extraction --test native_first_tests test_native_extraction_performance -- --nocapture

echo ""
echo "=================================================="
echo "✅ All tests completed!"
echo "=================================================="
