#!/bin/bash

# Test news article URL extraction
# Using riptide with --engine raw --no-wasm

RIPTIDE="/usr/local/bin/riptide"
OUTPUT_CSV="/workspaces/eventmesh/eval/results/news_test.csv"

# Test URLs (skipping blocked Reuters special report)
declare -a URLS=(
    "https://www.reuters.com/graphics/WW2-ANNIVERSARY/CHINA-PARADE/zdvxkgybypx/"
    "https://www.reuters.com/graphics/UKRAINE-CRISIS/DRONES/dwpkeyjwkpm/"
    "https://nos.nl/nieuws/tech"
    "https://nos.nl/nieuwsuur/artikel/2585550-europa-ontwaakt-in-de-ai-race-maar-druppel-op-gloeiende-plaat"
)

declare -a NAMES=(
    "Reuters Graphics — China war tech on parade"
    "Reuters Graphics — Ukraine drones"
    "NOS Tech — Tech news hub"
    "NOS — Europa ontwaakt in de AI-race"
)

declare -a TYPES=(
    "article"
    "article"
    "listing"
    "article"
)

# Create CSV header
echo "name,url,type,status,urls_extracted,execution_time_ms,error" > "$OUTPUT_CSV"

echo "Testing URL extraction on news articles..."
echo "============================================"

# Test each URL
for i in "${!URLS[@]}"; do
    url="${URLS[$i]}"
    name="${NAMES[$i]}"
    type="${TYPES[$i]}"

    echo ""
    echo "Testing: $name"
    echo "URL: $url"
    echo "Type: $type"

    # Run riptide with timing
    start_time=$(date +%s%3N)

    # Extract raw HTML content (no WASM due to version issues)
    html_output=$("$RIPTIDE" extract --url "$url" --engine raw --no-wasm --local 2>&1)
    exit_code=$?

    end_time=$(date +%s%3N)
    execution_time=$((end_time - start_time))

    if [ $exit_code -eq 0 ]; then
        # Extract URLs from HTML using regex
        # Look for href="..." and src="..." attributes
        urls=$(echo "$html_output" | grep -oP '(?:href|src)="(https?://[^"]+)"' | sed 's/.*"\(.*\)"/\1/' | sort -u)
        url_count=$(echo "$urls" | grep -c "^http" || echo "0")

        status="success"
        error=""

        echo "✓ Success: Extracted $url_count URLs in ${execution_time}ms"
        echo "Sample URLs:"
        echo "$urls" | head -5
    else
        url_count=0
        status="failed"
        error=$(echo "$output" | head -1 | tr ',' ';')

        echo "✗ Failed: $error"
    fi

    # Append to CSV (escape name for CSV)
    name_escaped=$(echo "$name" | sed 's/"/""/g')
    echo "\"$name_escaped\",\"$url\",\"$type\",\"$status\",$url_count,$execution_time,\"$error\"" >> "$OUTPUT_CSV"
done

echo ""
echo "============================================"
echo "Results saved to: $OUTPUT_CSV"
echo ""
echo "Summary:"
cat "$OUTPUT_CSV"
