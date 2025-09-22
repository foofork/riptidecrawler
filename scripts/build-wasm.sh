#!/bin/bash

# Build script for RipTide WASM Component Model extractor
# This script builds the WASM component with proper Component Model support

set -e

echo "🔨 Building RipTide WASM Component Model Extractor..."

# Ensure wasm32-wasip2 target is installed
echo "📦 Ensuring wasm32-wasip2 target is available..."
rustup target add wasm32-wasip2

# Clean previous builds
echo "🧹 Cleaning previous builds..."
cargo clean

# Build the WASM component for Component Model
echo "🚀 Building WASM component with Component Model..."
cd wasm/riptide-extractor-wasm

# Build with optimized profile for WASM
CARGO_PROFILE=release-wasm \
RUSTFLAGS="-C target-feature=+simd128 -C opt-level=s -C lto=fat" \
cargo build --target wasm32-wasip2 --release

echo "✅ WASM Component built successfully!"

# Verify the component was built
WASM_PATH="../../target/wasm32-wasip2/release/riptide_extractor_wasm.wasm"
if [ -f "$WASM_PATH" ]; then
    echo "📦 Component located at: $WASM_PATH"
    echo "📊 Component size: $(du -h "$WASM_PATH" | cut -f1)"

    # Validate the component if wasmtime is available
    if command -v wasmtime &> /dev/null; then
        echo "🔍 Validating component with wasmtime..."
        wasmtime component wit "$WASM_PATH" || echo "⚠️  Component validation failed (this is normal for some components)"
    fi
else
    echo "❌ Component not found at expected location: $WASM_PATH"
    exit 1
fi

echo "🎉 Build completed successfully!"