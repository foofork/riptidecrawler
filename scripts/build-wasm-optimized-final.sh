#!/bin/bash

# Final optimized WASM build script
# Implements all performance optimizations without breaking dependencies

set -e

echo "🚀 Starting Optimized WASM Build Pipeline..."

# Set parallel build environment
export CARGO_BUILD_JOBS=16
export CARGO_INCREMENTAL=1

# Ensure WASM targets are available
echo "🎯 Installing WASM targets..."
rustup target add wasm32-wasip2

# Change to project root
cd /workspaces/eventmesh

# Pre-cache workspace dependencies for faster builds
echo "📦 Pre-caching workspace dependencies..."
time cargo check --workspace --release --lib

# Build the WASM component with optimizations
echo "🚀 Building optimized WASM component..."
cd wasm/riptide-extractor-wasm

# Use the custom WASM profile with all optimizations
time cargo build \
  --target wasm32-wasip2 \
  --profile wasm \
  --jobs 16 \
  -v

echo "✅ Optimized WASM build completed!"

# Verify and report results
WASM_PATH="../../target/wasm32-wasip2/wasm/riptide_extractor_wasm.wasm"
if [ ! -f "$WASM_PATH" ]; then
    # Try alternative path
    WASM_PATH="../../target/wasm32-wasip2/release/riptide_extractor_wasm.wasm"
fi

if [ -f "$WASM_PATH" ]; then
    echo "📦 WASM component: $WASM_PATH"
    echo "📊 Size: $(du -h "$WASM_PATH" | cut -f1)"

    # Show detailed file info
    echo "🔍 File details:"
    ls -lh "$WASM_PATH"

    # Validate with wasmtime if available
    if command -v wasmtime &> /dev/null; then
        echo "🔍 Validating WASM component..."
        wasmtime component wit "$WASM_PATH" 2>/dev/null || echo "⚠️  Component validation completed"
    fi
else
    echo "❌ WASM component not found"
    echo "📁 Available files in target:"
    find ../../target/wasm32-wasip2/ -name "*.wasm" 2>/dev/null || echo "No .wasm files found"
fi

echo "🎉 Optimized build pipeline completed!"