#!/bin/bash
# Browser pool load testing

API_URL="${RIPTIDE_API_URL:-http://localhost:8080}"
NUM_BROWSERS="${1:-20}"
NUM_REQUESTS="${2:-1000}"
CONCURRENCY="${3:-5}"

echo "🔧 Load Testing Browser Pool"
echo "   API: $API_URL"
echo "   Browsers: $NUM_BROWSERS"
echo "   Requests: $NUM_REQUESTS"
echo "   Concurrency: $CONCURRENCY"
echo ""

# Start API server if not running
if ! curl -s "$API_URL/health" > /dev/null 2>&1; then
    echo "⚠️  API server not running at $API_URL"
    echo "   Please start the API server manually:"
    echo "   cargo run --release --bin riptide-api"
    echo ""
    read -p "Continue with load test anyway? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Function to run a single request
run_request() {
    local id=$1
    local start_time=$(date +%s%N)

    response=$(curl -s -w "\n%{http_code}" "$API_URL/extract?url=https://example.com" 2>/dev/null)
    http_code=$(echo "$response" | tail -n1)

    local end_time=$(date +%s%N)
    local duration=$(( (end_time - start_time) / 1000000 ))

    echo "$id,$http_code,$duration" >> /tmp/load-test-results.csv
}

# Initialize results file
echo "request_id,http_code,latency_ms" > /tmp/load-test-results.csv

# Check if hey is available
if command -v hey &> /dev/null; then
    echo "📊 Using 'hey' for load testing..."
    hey -n "$NUM_REQUESTS" -c "$CONCURRENCY" -q 10 "$API_URL/extract?url=https://example.com"
else
    echo "📊 Using curl for load testing (consider installing 'hey' for better metrics)..."
    echo "   Install: go install github.com/rakyll/hey@latest"
    echo ""

    # Run requests with concurrency control
    for i in $(seq 1 "$NUM_REQUESTS"); do
        run_request "$i" &

        # Control concurrency
        if [ $((i % CONCURRENCY)) -eq 0 ]; then
            wait
            echo "   Progress: $i/$NUM_REQUESTS requests completed"
        fi
    done
    wait

    echo ""
    echo "📊 Load Test Results:"
    echo "===================="

    # Calculate statistics from results
    if [ -f /tmp/load-test-results.csv ]; then
        awk -F',' 'NR>1 {
            sum+=$3;
            count++;
            if($3>max) max=$3;
            if(min=="" || $3<min) min=$3;
            if($2==200) success++;
        } END {
            print "   Total Requests: " count;
            print "   Successful (200): " success " (" int(success/count*100) "%)";
            print "   Avg Latency: " int(sum/count) " ms";
            print "   Min Latency: " min " ms";
            print "   Max Latency: " max " ms";
        }' /tmp/load-test-results.csv

        rm /tmp/load-test-results.csv
    fi
fi

echo ""
echo "📊 Memory Usage:"
ps aux | grep -E "riptide|chrome|chromium" | grep -v grep | awk '{sum+=$6} END {print "   Total: " sum/1024 " MB"}'

echo ""
echo "✅ Load test complete"
