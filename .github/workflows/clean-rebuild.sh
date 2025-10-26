#!/bin/bash
# Clean rebuild script to fix SIGILL errors from stale build artifacts
# Removes all compiled artifacts and rebuilds from scratch with baseline CPU target

set -e

echo "🧹 Cleaning all build artifacts..."
cargo clean

echo "🗑️  Removing target directory..."
rm -rf target/

echo "🔨 Rebuilding with baseline x86-64-v2 CPU target..."
cargo build --workspace --all-targets

echo "✅ Clean rebuild complete - SIGILL errors should be resolved"
