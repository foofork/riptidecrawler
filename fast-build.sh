#!/bin/bash
# Fast build script for development

echo "🚀 RipTide Fast Build"
echo "===================="

# Quick check (no codegen)
if [ "$1" = "check" ]; then
    echo "Running quick check..."
    time cargo check --all
    exit 0
fi

# Single crate build
if [ -n "$1" ]; then
    echo "Building $1 only..."
    time cargo build -p $1
    exit 0
fi

# Default: build core crates only
echo "Building core crates..."
time cargo build \
    -p riptide-core \
    -p riptide-html \
    -p riptide-intelligence \
    -p riptide-api

echo ""
echo "✅ Build complete!"
echo "Tip: Use './fast-build.sh check' for even faster checking"
echo "     Use './fast-build.sh riptide-api' to build single crate"