#!/bin/bash

# Test script for riptide extract command
RIPTIDE="/workspaces/eventmesh/target/x86_64-unknown-linux-gnu/release/riptide"
OUTPUT_DIR="/workspaces/eventmesh/eval/results"
CSV_FILE="$OUTPUT_DIR/extract_command_tests.csv"
TEMP_DIR="$OUTPUT_DIR/temp_extract"

# Create temp directory
mkdir -p "$TEMP_DIR"

# Initialize CSV with headers
echo "URL,Engine,Success,Content_Length,Time_ms,Error" > "$CSV_FILE"

# Function to test extraction
test_extraction() {
    local url="$1"
    local engine="$2"
    local extra_flags="$3"
    local test_name="$4"

    echo "Testing: $test_name"
    echo "  URL: $url"
    echo "  Engine: $engine"
    echo "  Flags: $extra_flags"

    # Prepare command
    local cmd="$RIPTIDE extract --url \"$url\" --engine $engine --local $extra_flags"

    # Measure execution time and capture output
    local start_time=$(date +%s%3N)
    local output_file="$TEMP_DIR/output_${engine}_$(echo $url | md5sum | cut -d' ' -f1).txt"
    local error_file="$TEMP_DIR/error_${engine}_$(echo $url | md5sum | cut -d' ' -f1).txt"

    # Run command
    eval "$cmd" > "$output_file" 2> "$error_file"
    local exit_code=$?
    local end_time=$(date +%s%3N)
    local duration=$((end_time - start_time))

    # Analyze results
    local success="false"
    local content_length=0
    local error_msg=""

    if [ $exit_code -eq 0 ]; then
        success="true"
        content_length=$(wc -c < "$output_file")

        # Check if content is actually meaningful (not just empty or error)
        if [ $content_length -lt 10 ]; then
            success="false"
            error_msg="Content too short"
        fi
    else
        error_msg=$(head -n 1 "$error_file" | tr ',' ' ' | tr '"' "'")
        if [ -z "$error_msg" ]; then
            error_msg="Command failed with exit code $exit_code"
        fi
    fi

    # Write to CSV (escape quotes and commas in error message)
    echo "\"$url\",$engine,$success,$content_length,$duration,\"$error_msg\"" >> "$CSV_FILE"

    echo "  Result: $success (${content_length} bytes, ${duration}ms)"
    if [ "$success" = "false" ]; then
        echo "  Error: $error_msg"
    fi
    echo ""
}

# Test 1: example.com with raw engine
test_extraction \
    "https://example.com" \
    "raw" \
    "" \
    "Test 1: example.com with raw engine"

# Test 2: Wikipedia with raw engine
test_extraction \
    "https://en.wikipedia.org/wiki/Rust_(programming_language)" \
    "raw" \
    "" \
    "Test 2: Wikipedia Rust page with raw engine"

# Test 3: Hacker News with raw engine
test_extraction \
    "https://news.ycombinator.com/" \
    "raw" \
    "" \
    "Test 3: Hacker News with raw engine"

# Test 4: Rust-lang.org with auto engine and metadata
test_extraction \
    "https://www.rust-lang.org/" \
    "auto" \
    "--metadata" \
    "Test 4: Rust-lang.org with auto engine and metadata"

# Additional test variations for robustness
echo "=== Running additional test variations ==="

# Test with readability engine
test_extraction \
    "https://en.wikipedia.org/wiki/Rust_(programming_language)" \
    "readability" \
    "" \
    "Test 5: Wikipedia with readability engine"

# Test with trafilatura engine
test_extraction \
    "https://www.rust-lang.org/" \
    "trafilatura" \
    "" \
    "Test 6: Rust-lang.org with trafilatura engine"

# Test with markdown output
test_extraction \
    "https://example.com" \
    "raw" \
    "--format markdown" \
    "Test 7: example.com with markdown format"

# Test with JSON output
test_extraction \
    "https://example.com" \
    "raw" \
    "--format json" \
    "Test 8: example.com with JSON format"

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
    avg_time=$(tail -n +2 "$CSV_FILE" | grep ",true," | cut -d',' -f5 | awk '{sum+=$1; count++} END {if(count>0) print sum/count; else print 0}')
    echo "Average extraction time (successful): ${avg_time}ms"
fi

# Content length statistics
echo ""
echo "Content length statistics (successful extractions):"
tail -n +2 "$CSV_FILE" | grep ",true," | cut -d',' -f4 | sort -n | awk '
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
        print "  Avg: " sum/count " bytes";
        if(count % 2 == 0) {
            print "  Median: " (arr[count/2-1]+arr[count/2])/2 " bytes";
        } else {
            print "  Median: " arr[int(count/2)] " bytes";
        }
    }
}'

echo ""
echo "=== Error Summary ==="
tail -n +2 "$CSV_FILE" | grep ",false," | cut -d',' -f1,6 | while IFS=',' read -r url error; do
    echo "  $url: $error"
done

# Cleanup temp files (keep outputs for inspection if needed)
# rm -rf "$TEMP_DIR"
echo ""
echo "Temp files preserved in: $TEMP_DIR"
