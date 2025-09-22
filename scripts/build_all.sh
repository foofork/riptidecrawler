#!/usr/bin/env bash
set -euo pipefail

echo "🚀 Building RipTide Crawler..."

# Build all Rust binaries
echo "📦 Building Rust binaries..."
cargo build --release

# Build WASM component
echo "🌐 Building WASM component..."
rustup target add wasm32-wasip2 2>/dev/null || true
cd wasm/riptide-extractor-wasm
cargo build --release --target wasm32-wasip2
cd ../..

echo "✅ Build complete!"
echo ""
echo "Binaries:"
echo "  - API: target/release/riptide-api"
echo "  - Headless: target/release/riptide-headless"
echo "  - Workers: target/release/riptide-workers"
echo "  - WASM: target/wasm32-wasip2/release/riptide-extractor-wasm.wasm"