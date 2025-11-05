#!/usr/bin/env bash
# Build script for riptide Python package

set -e

echo "Building riptide Python package..."

# Check if maturin is installed
if ! command -v maturin &> /dev/null; then
    echo "Error: maturin is not installed"
    echo "Install with: pip install maturin"
    exit 1
fi

# Build mode (default: release)
MODE="${1:-release}"

if [ "$MODE" = "dev" ]; then
    echo "Building in development mode..."
    maturin develop
elif [ "$MODE" = "release" ]; then
    echo "Building release wheel..."
    maturin build --release
else
    echo "Unknown mode: $MODE"
    echo "Usage: $0 [dev|release]"
    exit 1
fi

echo "Build complete!"
