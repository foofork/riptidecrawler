#!/bin/bash
set -e

echo "🚀 Development Runner with Hot Reload"
echo "===================================="

# Kill existing processes
./scripts/dev-setup.sh

# Use cargo watch for hot reload if available
if command -v cargo-watch &> /dev/null; then
    echo "🔄 Starting with hot-reload..."
    cargo watch -x "run --bin riptide-api" -w crates/
else
    echo "📦 Installing cargo-watch for hot-reload..."
    cargo install cargo-watch
    cargo watch -x "run --bin riptide-api" -w crates/
fi