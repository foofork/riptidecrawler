#!/usr/bin/env bash
set -euo pipefail

echo "🔧 Bootstrapping RipTide development environment..."

# Install Rust toolchain and targets
echo "📦 Installing Rust toolchain..."
rustup toolchain install stable
rustup target add wasm32-wasi
rustup component add rustfmt clippy

# Install optional tools
echo "🛠️  Installing optional tools..."
command -v just >/dev/null 2>&1 || cargo install just
command -v cargo-deny >/dev/null 2>&1 || cargo install cargo-deny

# Create necessary directories
echo "📁 Creating directories..."
mkdir -p tests/{e2e/fixtures,golden/{urls,expected}}
mkdir -p /tmp/riptide/artifacts

echo "✅ Bootstrap complete!"
echo ""
echo "Next steps:"
echo "  1. Set SERPER_API_KEY in .env file"
echo "  2. Run: ./scripts/build_all.sh"
echo "  3. Run: docker compose -f infra/docker/docker-compose.yml up"