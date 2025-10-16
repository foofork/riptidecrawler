#!/bin/bash

# Corrected test script for riptide extract command
RIPTIDE="/workspaces/eventmesh/target/x86_64-unknown-linux-gnu/release/riptide"
OUTPUT_DIR="/workspaces/eventmesh/eval/results"
CSV_FILE="$OUTPUT_DIR/extract_command_tests.csv"
TEMP_DIR="$OUTPUT_DIR/temp_extract"

# Create temp directory
mkdir -p "$TEMP_DIR"

# Initialize CSV with headers
echo "URL,Engine,Method,Strategy,Success,Content_Length,Time_ms,Error" > "$CSV_FILE"

# Function to test extraction
test_extraction() {
    local url="$1"
    local engine="$2"
    local method="$3"
    local extra_flags="$4"
    local test_name="$5"

    echo "Testing: $test_name"
    echo "  URL: $url"
    echo "  Engine: $engine"
    echo "  Method: $method"
    echo "  Flags: $extra_flags"

    # Prepare command
    local cmd="$RIPTIDE extract --url \"$url\" --engine $engine"
    if [ -n "$method" ]; then
        cmd="$cmd --method $method"
    fi
    if [ -n "$extra_flags" ]; then
        cmd="$cmd $extra_flags"
    fi

    # Measure execution time and capture output
    local start_time=$(date +%s%3N)
    local output_file="$TEMP_DIR/output_${engine}_${method}_$(echo $url | md5sum | cut -d' ' -f1).txt"
    local error_file="$TEMP_DIR/error_${engine}_${method}_$(echo $url | md5sum | cut -d' ' -f1).txt"

    # Run command
    eval "$cmd" > "$output_file" 2> "$error_file"
    local exit_code=$?
    local end_time=$(date +%s%3N)
    local duration=$((end_time - start_time))

    # Analyze results
    local success="false"
    local content_length=0
    local error_msg=""
    local strategy="none"

    # Extract strategy from extra_flags if present
    if [[ "$extra_flags" == *"--strategy"* ]]; then
        strategy=$(echo "$extra_flags" | grep -oP '(?<=--strategy )[^ ]+' || echo "unknown")
    fi

    if [ $exit_code -eq 0 ]; then
        success="true"
        content_length=$(wc -c < "$output_file")

        # Check if content is actually meaningful (not just empty or error)
        if [ $content_length -lt 10 ]; then
            success="false"
            error_msg="Content too short"
        fi
    else
        error_msg=$(head -n 1 "$error_file" | tr ',' ' ' | tr '"' "'" | head -c 100)
        if [ -z "$error_msg" ]; then
            error_msg="Command failed with exit code $exit_code"
        fi
    fi

    # Write to CSV (escape quotes and commas in error message)
    echo "\"$url\",$engine,$method,$strategy,$success,$content_length,$duration,\"$error_msg\"" >> "$CSV_FILE"

    echo "  Result: $success (${content_length} bytes, ${duration}ms)"
    if [ "$success" = "false" ]; then
        echo "  Error: $error_msg"
    fi
    echo ""
}

echo "=== Testing Basic Requested Combinations ==="

# Test 1: example.com with raw engine
test_extraction \
    "https://example.com" \
    "raw" \
    "" \
    "--local" \
    "Test 1: example.com with raw engine (local)"

# Test 2: Wikipedia with raw engine
test_extraction \
    "https://en.wikipedia.org/wiki/Rust_(programming_language)" \
    "raw" \
    "" \
    "--local" \
    "Test 2: Wikipedia Rust page with raw engine (local)"

# Test 3: Hacker News with raw engine
test_extraction \
    "https://news.ycombinator.com/" \
    "raw" \
    "" \
    "--local" \
    "Test 3: Hacker News with raw engine (local)"

# Test 4: Rust-lang.org with auto engine and metadata
test_extraction \
    "https://www.rust-lang.org/" \
    "auto" \
    "" \
    "--local --metadata" \
    "Test 4: Rust-lang.org with auto engine and metadata (local)"

echo "=== Testing All Available Engines ==="

# Test all engines with example.com
for engine in auto raw wasm headless; do
    test_extraction \
        "https://example.com" \
        "$engine" \
        "" \
        "--local" \
        "Test: example.com with $engine engine"
done

echo "=== Testing Different Methods ==="

# Test different methods with auto engine
for method in auto wasm css llm regex; do
    test_extraction \
        "https://example.com" \
        "auto" \
        "$method" \
        "--local" \
        "Test: example.com with auto engine and $method method"
done

echo "=== Testing Strategy Compositions ==="

# Test strategy compositions
test_extraction \
    "https://example.com" \
    "auto" \
    "" \
    "--local --strategy chain:wasm,css" \
    "Test: example.com with chain strategy (wasm,css)"

test_extraction \
    "https://example.com" \
    "auto" \
    "" \
    "--local --strategy parallel:all" \
    "Test: example.com with parallel strategy (all)"

test_extraction \
    "https://example.com" \
    "auto" \
    "" \
    "--local --strategy fallback:wasm,css,regex" \
    "Test: example.com with fallback strategy (wasm,css,regex)"

echo "=== Testing Complex URLs ==="

# Test Wikipedia with different engines
test_extraction \
    "https://en.wikipedia.org/wiki/Rust_(programming_language)" \
    "wasm" \
    "" \
    "--local" \
    "Test: Wikipedia with WASM engine"

test_extraction \
    "https://en.wikipedia.org/wiki/Rust_(programming_language)" \
    "auto" \
    "wasm" \
    "--local" \
    "Test: Wikipedia with auto engine and WASM method"

# Test Rust-lang.org with different engines
test_extraction \
    "https://www.rust-lang.org/" \
    "wasm" \
    "" \
    "--local" \
    "Test: Rust-lang.org with WASM engine"

test_extraction \
    "https://www.rust-lang.org/" \
    "raw" \
    "" \
    "--local" \
    "Test: Rust-lang.org with raw engine"

echo "=== Testing with Confidence Scores ==="

test_extraction \
    "https://example.com" \
    "auto" \
    "wasm" \
    "--local --show-confidence" \
    "Test: example.com with confidence scores"

test_extraction \
    "https://en.wikipedia.org/wiki/Rust_(programming_language)" \
    "wasm" \
    "" \
    "--local --show-confidence" \
    "Test: Wikipedia with confidence scores"

echo "=== All tests completed ==="
echo "Results saved to: $CSV_FILE"
echo ""
echo "=== Summary Statistics ==="

# Calculate summary statistics
total_tests=$(tail -n +2 "$CSV_FILE" | wc -l)
successful_tests=$(tail -n +2 "$CSV_FILE" | grep ",true," | wc -l)
failed_tests=$(tail -n +2 "$CSV_FILE" | grep ",false," | wc -l)
success_rate=$(echo "scale=2; ($successful_tests * 100) / $total_tests" | bc)

echo "Total tests: $total_tests"
echo "Successful: $successful_tests"
echo "Failed: $failed_tests"
echo "Success rate: ${success_rate}%"
echo ""

# Average extraction time for successful tests
if [ $successful_tests -gt 0 ]; then
    avg_time=$(tail -n +2 "$CSV_FILE" | grep ",true," | cut -d',' -f7 | awk '{sum+=$1; count++} END {if(count>0) print sum/count; else print 0}')
    echo "Average extraction time (successful): ${avg_time}ms"
fi

# Content length statistics
echo ""
echo "Content length statistics (successful extractions):"
tail -n +2 "$CSV_FILE" | grep ",true," | cut -d',' -f6 | sort -n | awk '
BEGIN {
    count=0;
    sum=0;
}
{
    arr[count++]=$1;
    sum+=$1;
}
END {
    if(count>0) {
        print "  Min: " arr[0] " bytes";
        print "  Max: " arr[count-1] " bytes";
        print "  Avg: " int(sum/count) " bytes";
        if(count % 2 == 0) {
            print "  Median: " int((arr[count/2-1]+arr[count/2])/2) " bytes";
        } else {
            print "  Median: " arr[int(count/2)] " bytes";
        }
    }
}'

echo ""
echo "=== Performance by Engine ==="
for engine in auto raw wasm headless; do
    engine_tests=$(tail -n +2 "$CSV_FILE" | grep ",$engine," | wc -l)
    if [ $engine_tests -gt 0 ]; then
        engine_success=$(tail -n +2 "$CSV_FILE" | grep ",$engine," | grep ",true," | wc -l)
        engine_rate=$(echo "scale=2; ($engine_success * 100) / $engine_tests" | bc)
        engine_avg=$(tail -n +2 "$CSV_FILE" | grep ",$engine," | grep ",true," | cut -d',' -f7 | awk '{sum+=$1; count++} END {if(count>0) print sum/count; else print 0}')
        echo "  $engine: $engine_success/$engine_tests (${engine_rate}%) - avg ${engine_avg}ms"
    fi
done

echo ""
echo "=== Error Summary ==="
tail -n +2 "$CSV_FILE" | grep ",false," | cut -d',' -f1,2,8 | head -20 | while IFS=',' read -r url engine error; do
    echo "  [$engine] $url: $error"
done

echo ""
echo "Temp files preserved in: $TEMP_DIR"
