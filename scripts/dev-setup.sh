#!/bin/bash
set -e

echo "🔧 Development Environment Setup"
echo "================================"

# Kill processes on common ports
echo "📍 Freeing up ports..."
for port in 3000 8080 9222 6379 5432 9050; do
    lsof -ti:$port | xargs -r kill -9 2>/dev/null || true
done
echo "✅ Ports cleared"

# Install sccache if not present
if ! command -v sccache &> /dev/null; then
    echo "📦 Installing sccache for faster builds..."
    cargo install sccache
    export RUSTC_WRAPPER=sccache
fi

# Set environment variables
export RUST_BACKTRACE=1
export RUST_TEST_THREADS=8
export CARGO_BUILD_JOBS=8

echo "✅ Environment ready!"
echo ""
echo "Available commands:"
echo "  ./scripts/quick-build.sh   - Fast incremental build"
echo "  ./scripts/fast-test.sh     - Run tests in parallel"
echo "  ./scripts/quality-check.sh - Run lints and formatting"
echo "  ./scripts/dev-run.sh       - Run with hot-reload"