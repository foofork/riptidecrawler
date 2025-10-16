#!/bin/bash

# Proper extraction test using riptide CLI for full HTML content
# This actually extracts the full page content, not just metadata

RIPTIDE="./target/x86_64-unknown-linux-gnu/release/riptide"
RESULTS_DIR="eval/results/proper_extraction_$(date +%Y%m%d_%H%M%S)"

mkdir -p "$RESULTS_DIR"

echo "====================================="
echo "RipTide Proper Extraction Test"
echo "====================================="
echo ""

# CSV header
echo "URL,Engine,Content_Length,Extraction_Time_ms,Success,Error" > "$RESULTS_DIR/extraction_results.csv"

# Test URLs
URLS=(
    "https://en.wikipedia.org/wiki/Web_scraping"
    "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Introduction"
    "https://www.reuters.com/graphics/UKRAINE-CRISIS/DRONES/dwpkeyjwkpm/"
    "https://news.ycombinator.com/"
    "https://example.com"
)

for url in "${URLS[@]}"; do
    echo "Testing: $url"
    echo "----------------------------------------"

    # Create safe filename
    safe_name=$(echo "$url" | sed 's/[^a-zA-Z0-9]/_/g')
    output_file="$RESULTS_DIR/${safe_name}.html"

    # Run extraction with riptide CLI
    start_time=$(date +%s%N)

    if $RIPTIDE extract --url "$url" --engine raw --local -f "$output_file" 2>&1 | tee "$RESULTS_DIR/${safe_name}.log"; then
        end_time=$(date +%s%N)
        duration=$(( (end_time - start_time) / 1000000 ))

        # Get file size
        if [ -f "$output_file" ]; then
            content_length=$(wc -c < "$output_file")
            echo "✓ Extracted $content_length bytes in ${duration}ms"
            echo "\"$url\",\"raw\",$content_length,$duration,true,\"\"" >> "$RESULTS_DIR/extraction_results.csv"

            # Show first 500 chars of content
            echo "Preview of extracted content:"
            head -c 500 "$output_file"
            echo ""
        else
            echo "✗ No output file created"
            echo "\"$url\",\"raw\",0,$duration,false,\"No output file\"" >> "$RESULTS_DIR/extraction_results.csv"
        fi
    else
        echo "✗ Extraction failed"
        echo "\"$url\",\"raw\",0,0,false,\"Command failed\"" >> "$RESULTS_DIR/extraction_results.csv"
    fi

    echo ""
done

echo "====================================="
echo "Test Summary"
echo "====================================="
echo "Results saved to: $RESULTS_DIR"
echo ""
echo "CSV results:"
cat "$RESULTS_DIR/extraction_results.csv"