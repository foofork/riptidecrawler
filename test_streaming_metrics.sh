#!/bin/bash

# Test script to verify streaming metrics are working in the /metrics endpoint

echo "🚀 Testing EventMesh Streaming Metrics Integration"
echo "=================================================="

# Check if the API server is running
if ! curl -s http://localhost:8080/healthz > /dev/null; then
    echo "❌ API server not running on localhost:8080"
    echo "   Start the server first: cargo run --bin riptide-api"
    exit 1
fi

echo "✅ API server is running"

# Test the /metrics endpoint
echo ""
echo "📊 Fetching metrics from /metrics endpoint..."
echo "============================================="

METRICS_OUTPUT=$(curl -s http://localhost:8080/metrics)

if [ $? -ne 0 ]; then
    echo "❌ Failed to fetch metrics"
    exit 1
fi

echo "✅ Successfully fetched metrics"
echo ""

# Check for streaming metrics
echo "🔍 Checking for streaming metrics..."
echo "===================================="

# List of streaming metrics we expect to find
EXPECTED_METRICS=(
    "riptide_streaming_active_connections"
    "riptide_streaming_total_connections"
    "riptide_streaming_messages_sent_total"
    "riptide_streaming_messages_dropped_total"
    "riptide_streaming_error_rate"
    "riptide_streaming_memory_usage_bytes"
    "riptide_streaming_connection_duration_seconds"
)

FOUND_COUNT=0

for metric in "${EXPECTED_METRICS[@]}"; do
    if echo "$METRICS_OUTPUT" | grep -q "^$metric"; then
        echo "✅ Found: $metric"
        ((FOUND_COUNT++))

        # Show the actual metric value
        echo "   $(echo "$METRICS_OUTPUT" | grep "^$metric" | head -1)"
    else
        echo "❌ Missing: $metric"
    fi
done

echo ""
echo "📈 Summary"
echo "=========="
echo "Found $FOUND_COUNT out of ${#EXPECTED_METRICS[@]} expected streaming metrics"

if [ $FOUND_COUNT -eq ${#EXPECTED_METRICS[@]} ]; then
    echo "🎉 SUCCESS: All streaming metrics are properly exposed!"
else
    echo "⚠️  WARNING: Some streaming metrics are missing"
fi

# Test a render operation to generate metrics
echo ""
echo "🎯 Testing render operation to generate metrics..."
echo "================================================="

TEST_RESPONSE=$(curl -s -X POST http://localhost:8080/render \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com", "mode": "static"}')

if [ $? -eq 0 ]; then
    echo "✅ Render operation completed"

    # Wait a moment for metrics to update
    sleep 1

    # Fetch metrics again to see if they updated
    echo ""
    echo "📊 Checking updated metrics after render operation..."
    UPDATED_METRICS=$(curl -s http://localhost:8080/metrics)

    # Look for render-related metrics
    if echo "$UPDATED_METRICS" | grep -q "riptide_render_phase_duration_seconds"; then
        echo "✅ Render metrics updated successfully"
        echo "   $(echo "$UPDATED_METRICS" | grep "riptide_render_phase_duration_seconds" | head -1)"
    else
        echo "⚠️  Render metrics not found or not updated"
    fi
else
    echo "❌ Render operation failed"
fi

echo ""
echo "🏁 Test complete!"