#!/bin/bash
# RipTide API Development Server Startup Script
# Starts the API server with development mode authentication disabled

set -e

echo "🚀 Starting RipTide API in Development Mode..."
echo ""
echo "📋 Configuration:"
echo "   • Authentication: DISABLED (REQUIRE_AUTH=false)"
echo "   • Bind Address: 0.0.0.0:8080"
echo "   • Environment: Development"
echo ""

# Check if server is already running
if pgrep -f "riptide-api" > /dev/null; then
    echo "⚠️  API server already running"
    echo ""
    read -p "Kill existing server and restart? (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "🛑 Stopping existing server..."
        pkill -f "riptide-api"
        sleep 2
    else
        echo "Exiting..."
        exit 0
    fi
fi

# Ensure we're in project root
cd "$(dirname "$0")/.."

# Check if binary exists
if [ ! -f "target/release/riptide-api" ] && [ ! -f "target/x86_64-unknown-linux-gnu/release/riptide-api" ]; then
    echo "⚠️  Binary not found. Building..."
    cargo build --release --bin riptide-api
fi

# Start server with dev mode
echo "✅ Starting server..."
REQUIRE_AUTH=false cargo run --release --bin riptide-api --bind 0.0.0.0:8080 &

# Wait for server to start
sleep 3

# Test if server is responding
echo ""
echo "🔍 Testing server..."
if curl -s http://localhost:8080/api/v1/health > /dev/null; then
    echo "✅ Server is running and healthy!"
    echo ""
    echo "📚 Available endpoints:"
    echo "   • Health: http://localhost:8080/api/v1/health"
    echo "   • Tables: http://localhost:8080/api/v1/tables/extract"
    echo "   • Search: http://localhost:8080/api/v1/search?q=query"
    echo "   • Crawl: http://localhost:8080/crawl"
    echo ""
    echo "🔓 Authentication disabled - no API key required"
else
    echo "❌ Server failed to start or not responding"
    exit 1
fi
