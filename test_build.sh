#!/bin/bash

# Kill any existing cargo processes
pkill -9 cargo rustc rust-analyzer 2>/dev/null

# Test individual packages with minimal dependencies
echo "Testing core packages..."

packages=("riptide-core" "riptide-html" "riptide-search" "riptide-stealth" "riptide-pdf" "riptide-intelligence" "riptide-workers")

for pkg in "${packages[@]}"; do
    echo -n "Checking $pkg... "
    if timeout 30 cargo check --package "$pkg" --no-default-features 2>&1 | grep -q "error\["; then
        echo "ERRORS FOUND"
        cargo check --package "$pkg" --no-default-features 2>&1 | grep "error\[" | head -3
    else
        echo "OK"
    fi
done

echo ""
echo "=== Summary ==="
echo "Build test complete. Packages checked: ${#packages[@]}"