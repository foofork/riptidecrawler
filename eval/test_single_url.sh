#!/bin/bash

# Simple single URL test for RipTide CLI

URL="${1:-https://en.wikipedia.org/wiki/Web_scraping}"
ENGINE="${2:-auto}"
METHOD="${3:-auto}"

echo "Testing RipTide extraction:"
echo "URL: $URL"
echo "Engine: $ENGINE"
echo "Method: $METHOD"
echo ""

# Create results directory
mkdir -p eval/results/test

# Run extraction
echo "Running extraction..."
cargo run --release --bin riptide -- extract \
    --url "$URL" \
    --engine "$ENGINE" \
    --method "$METHOD" \
    --local \
    2>&1 | tee eval/results/test/single_test.log

echo ""
echo "Test complete. Check eval/results/test/single_test.log for output."