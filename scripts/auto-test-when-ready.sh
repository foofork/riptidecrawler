#!/bin/bash

# Automatic Test Runner - Waits for services and runs professional tests

echo "🤖 Auto-Test Monitor Starting..."
echo "   Waiting for RipTide services to be ready..."
echo ""

# Wait for docker-compose to finish
max_wait=1800  # 30 minutes max
elapsed=0

while [ $elapsed -lt $max_wait ]; do
    if curl -s -f http://localhost:8080/health > /dev/null 2>&1; then
        echo "✅ API is ready!"
        echo ""
        echo "🧪 Running Professional Scraping Test Suite..."
        echo ""

        # Run the professional test suite
        /workspaces/eventmesh/tests/professional-scraping-test.sh

        echo ""
        echo "📊 Test complete! Report available at:"
        echo "   docs/professional-scraping-report.md"
        echo ""

        exit 0
    fi

    # Progress indicator every 30 seconds
    if [ $((elapsed % 30)) -eq 0 ] && [ $elapsed -gt 0 ]; then
        echo "⏳ Still building... (${elapsed}s elapsed)"
    fi

    sleep 5
    elapsed=$((elapsed + 5))
done

echo "❌ Timeout waiting for services to start"
echo "   Check docker-compose logs for details"
exit 1
