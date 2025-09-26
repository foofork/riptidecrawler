#!/bin/bash
# CI/Build optimization script

set -e

echo "🚀 CI/Build Optimization Script"
echo "================================"

# Function to measure build time
measure_build() {
    local start=$(date +%s)
    "$@"
    local end=$(date +%s)
    local duration=$((end - start))
    echo "⏱️  Duration: ${duration}s"
    return 0
}

# Clean previous artifacts
echo "🧹 Cleaning previous artifacts..."
cargo clean --package riptide-api --package riptide-core --package riptide-workers

# Enable sccache if available
if command -v sccache >/dev/null 2>&1; then
    echo "✅ Enabling sccache for compilation caching"
    export RUSTC_WRAPPER=sccache
    sccache --start-server 2>/dev/null || true
    sccache --show-stats
fi

# Set optimization flags
export CARGO_BUILD_JOBS=8
export CARGO_INCREMENTAL=1
export RUSTFLAGS="-C target-cpu=native -C link-arg=-s"

echo ""
echo "📊 Baseline build (no cache)..."
measure_build cargo build --release --workspace

echo ""
echo "📊 Incremental rebuild (with cache)..."
touch src/main.rs 2>/dev/null || touch crates/riptide-api/src/main.rs
measure_build cargo build --release --workspace

echo ""
echo "📊 WASM build..."
cd wasm/riptide-extractor-wasm
measure_build cargo build --release --target wasm32-wasip2
cd ../..

# Check binary sizes
echo ""
echo "📏 Binary sizes:"
for binary in target/release/riptide-*; do
    if [[ -f "$binary" && -x "$binary" ]]; then
        size=$(stat --format=%s "$binary")
        size_mb=$(echo "scale=2; $size / 1024 / 1024" | bc)
        echo "  $(basename "$binary"): ${size_mb}MB"
    fi
done

# WASM size
if [[ -f "target/wasm32-wasip2/release/riptide_extractor_wasm.wasm" ]]; then
    size=$(stat --format=%s "target/wasm32-wasip2/release/riptide_extractor_wasm.wasm")
    size_mb=$(echo "scale=2; $size / 1024 / 1024" | bc)
    echo "  WASM module: ${size_mb}MB"
fi

# Show sccache stats
if command -v sccache >/dev/null 2>&1; then
    echo ""
    echo "📊 Sccache statistics:"
    sccache --show-stats
fi

# Calculate space saved
echo ""
echo "💾 Disk space analysis:"
target_size=$(du -sh target/ 2>/dev/null | cut -f1)
echo "  Target directory: $target_size"

# Optimize target directory
echo ""
echo "🗑️  Removing unnecessary build artifacts..."
find target -name "*.d" -delete 2>/dev/null || true
find target -name "*.rlib" -size +50M -delete 2>/dev/null || true
find target -name "incremental" -type d -exec rm -rf {} + 2>/dev/null || true

target_size_after=$(du -sh target/ 2>/dev/null | cut -f1)
echo "  Target directory after cleanup: $target_size_after"

echo ""
echo "✅ Optimization complete!"
echo ""
echo "Recommendations:"
echo "1. Use 'cargo build --profile=release-ci' for CI builds"
echo "2. Enable sccache in CI with GitHub Actions cache"
echo "3. Use cargo-nextest for 30% faster test execution"
echo "4. Cache WASM artifacts between builds"
echo "5. Use 'cargo clean --package' instead of full clean"