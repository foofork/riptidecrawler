#!/bin/bash
# Start RipTide API Server with WASM enabled

set -e

echo "🚀 Starting RipTide API Server..."

# Check Redis
if ! redis-cli ping > /dev/null 2>&1; then
    echo "❌ Redis not running. Starting Redis..."
    docker start riptide-redis || docker run -d --name riptide-redis -p 6379:6379 redis:alpine
    sleep 2
fi

echo "✅ Redis is running"

# Check WASM module
if [ ! -f "/opt/riptide/wasm/riptide_extractor_wasm.wasm" ]; then
    echo "⚠️  WASM module not found at /opt/riptide/wasm/"
    echo "   Server will run without WASM extraction"
fi

# Set environment variables
export RIPTIDE_WASM_PATH=/opt/riptide/wasm/riptide_extractor_wasm.wasm
export RIPTIDE_ENABLE_WASM=true
export RUST_LOG=info,cranelift=warn,wasmtime=warn
export RIPTIDE_REDIS_URL=redis://localhost:6379

# Start server
echo "🌊 Starting RipTide API on http://localhost:8080"
echo "   Press Ctrl+C to stop"
echo ""

exec target/x86_64-unknown-linux-gnu/release/riptide-api \
    --bind 127.0.0.1:8080
