#!/bin/bash

# Pre-compile dependencies for faster subsequent builds
# This script should be run once to cache common dependencies

set -e

echo "🔧 Pre-building dependencies for faster WASM builds..."

cd /workspaces/eventmesh

# Build all workspace libraries to cache dependencies
echo "📦 Building workspace libraries..."
cargo build --workspace --lib --release

# Pre-compile common WASM dependencies
echo "🎯 Pre-compiling WASM target dependencies..."
cargo build --target wasm32-wasip2 --lib --release || true

echo "✅ Dependency caching completed!"
echo "💡 Subsequent WASM builds should be significantly faster."