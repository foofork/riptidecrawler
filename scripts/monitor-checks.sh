#!/bin/bash

echo "📊 Build & Test Monitor"
echo "======================"

while true; do
    clear
    echo "📊 Build & Test Monitor - $(date +%H:%M:%S)"
    echo "================================"
    echo ""

    # Check test progress
    if [ -f test-output.log ]; then
        echo "🧪 Tests:"
        tail -5 test-output.log | grep -E "Compiling|Running|test result|error" || echo "  Building..."
    fi
    echo ""

    # Check format status
    if [ -f fmt-output.log ]; then
        echo "🎨 Format Check:"
        if grep -q "Diff in" fmt-output.log; then
            echo "  ⚠️ Format issues found ($(grep -c "Diff in" fmt-output.log) files)"
        else
            echo "  ✅ Format check passed"
        fi
    fi
    echo ""

    # Check clippy status
    if [ -f clippy-output.log ]; then
        echo "📝 Clippy Lints:"
        grep -c "warning:" clippy-output.log 2>/dev/null && echo " warnings found" || echo "  Building..."
    fi
    echo ""

    # System resources
    echo "💾 Resources:"
    echo "  Memory: $(free -h | grep "^Mem:" | awk '{print $3 " / " $2}')"
    echo "  Disk: $(df -h . | tail -1 | awk '{print $3 " / " $2 " (" $5 " used)"}')"
    echo ""

    echo "Press Ctrl+C to stop monitoring"
    sleep 5
done