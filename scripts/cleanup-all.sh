#!/bin/bash

echo "🧹 Complete System Cleanup"
echo "=========================="

# Kill stuck processes
echo "📍 Killing stuck processes..."
pkill -f "npm exec ruv-swarm" 2>/dev/null || true
pkill -f "cargo" 2>/dev/null || true
pkill -f "rustc" 2>/dev/null || true
pkill -f "chromium" 2>/dev/null || true

# Free ports
echo "📍 Freeing ports..."
for port in 3000 8080 9222 6379; do
    lsof -ti:$port | xargs -r kill -9 2>/dev/null || true
done

# Clean Docker if needed
if docker ps -q 2>/dev/null; then
    echo "🐳 Stopping Docker containers..."
    docker stop $(docker ps -q) 2>/dev/null || true
fi

# Clean build artifacts (optional)
read -p "Clean build artifacts (726MB)? [y/N] " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "🗑️ Cleaning build artifacts..."
    cargo clean
    rm -rf target/
fi

# Clean temp files
echo "🗑️ Cleaning temp files..."
find /tmp -name "rust*" -o -name "cargo*" -exec rm -rf {} + 2>/dev/null || true

echo ""
echo "✅ Cleanup complete!"
echo ""
echo "Current resource usage:"
df -h . | tail -1
free -h | grep "^Mem:"