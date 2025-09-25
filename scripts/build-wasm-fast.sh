#!/bin/bash

# Fast WASM build script without sccache dependency
# Optimizes WASM compilation using cargo features and parallel builds

set -e

echo "🚀 Starting Fast WASM Build (No External Dependencies)..."

# Set up parallel build environment
export CARGO_BUILD_JOBS=16
export CARGO_INCREMENTAL=1

# Ensure WASM targets are available
echo "🎯 Installing WASM targets..."
rustup target add wasm32-wasip2

# Clean and prepare
echo "🧹 Cleaning previous builds..."
cargo clean

# Pre-warm the build cache by building dependencies first
echo "📦 Pre-building dependencies..."
cd /workspaces/eventmesh

# Build workspace first to cache common dependencies
cargo check --workspace --release --lib

# Now build the WASM component with all optimizations
echo "🚀 Building optimized WASM component..."
cd wasm/riptide-extractor-wasm

# Use optimized profile and parallel build
time RUSTFLAGS="-C target-feature=+simd128 -C opt-level=s -C lto=fat -C codegen-units=1 -C panic=abort" \
cargo build \
  --target wasm32-wasip2 \
  --release \
  --jobs 16

echo "✅ Fast WASM build completed!"

# Verify output and show stats
WASM_PATH="../../target/wasm32-wasip2/release/riptide_extractor_wasm.wasm"
if [ -f "$WASM_PATH" ]; then
    echo "📦 WASM component: $WASM_PATH"
    echo "📊 Size: $(du -h "$WASM_PATH" | cut -f1)"
    echo "🔍 File details:"
    ls -lh "$WASM_PATH"
else
    echo "❌ WASM component not found at expected location"
    echo "Available files:"
    find ../../target/wasm32-wasip2/ -name "*.wasm" 2>/dev/null || echo "No .wasm files found"
fi

echo "🎉 Fast build pipeline completed!"