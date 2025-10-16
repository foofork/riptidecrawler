#!/bin/bash
# Extract listings from all URLs in 30_listings.yml

RESULTS_DIR="/workspaces/eventmesh/eval/results"
OUTPUT_CSV="$RESULTS_DIR/listings_test.csv"
RIPTIDE="/usr/local/bin/riptide"

mkdir -p "$RESULTS_DIR"

# Initialize CSV with headers
echo "source,rank,title,url,metadata,extraction_time_ms" > "$OUTPUT_CSV"

echo "=== Testing Listings Extraction ==="
echo "Output: $OUTPUT_CSV"
echo ""

# Test 1: Hacker News
echo "1. Extracting from Hacker News..."
START_TIME=$(date +%s%3N)
HN_HTML=$($RIPTIDE extract --url "https://news.ycombinator.com/" --engine raw --no-wasm --local 2>/dev/null)
END_TIME=$(date +%s%3N)
HN_TIME=$((END_TIME - START_TIME))

# Parse HN listings (first 10)
echo "$HN_HTML" | grep -oP '<span class="rank">\K\d+(?=\.)' | head -10 | while read rank; do
  # Extract title and URL for this rank
  title=$(echo "$HN_HTML" | grep -A 3 "rank\">$rank\." | grep -oP '<span class="titleline"><a href="[^"]*">\K[^<]+' | head -1 | sed 's/"/\\"/g')
  link=$(echo "$HN_HTML" | grep -A 3 "rank\">$rank\." | grep -oP '<span class="titleline"><a href="\K[^"]+' | head -1)
  points=$(echo "$HN_HTML" | grep -A 10 "rank\">$rank\." | grep -oP 'score[^>]*>\K\d+(?= points)' | head -1)
  author=$(echo "$HN_HTML" | grep -A 10 "rank\">$rank\." | grep -oP 'hnuser">\K[^<]+' | head -1)
  comments=$(echo "$HN_HTML" | grep -A 10 "rank\">$rank\." | grep -oP 'item\?id=\d+">\K\d+(?=&nbsp;comments)' | head -1)

  [ -z "$comments" ] && comments="0"
  [ -z "$points" ] && points="0"

  metadata="points:$points|author:$author|comments:$comments"
  echo "hackernews,$rank,\"$title\",$link,\"$metadata\",$HN_TIME" >> "$OUTPUT_CSV"
done

echo "   Found $(grep -c '^hackernews,' "$OUTPUT_CSV") items in ${HN_TIME}ms"

# Test 2: GitHub Topics
echo "2. Extracting from GitHub Topics (Rust)..."
START_TIME=$(date +%s%3N)
GH_HTML=$($RIPTIDE extract --url "https://github.com/topics/rust" --engine raw --no-wasm --local 2>/dev/null)
END_TIME=$(date +%s%3N)
GH_TIME=$((END_TIME - START_TIME))

# Parse GitHub repository listings (first 10)
echo "$GH_HTML" | grep -oP 'topic-repositories.*?</article>' | head -10 | \
  grep -oP '<h3[^>]*>.*?</h3>' | sed 's/<[^>]*>//g' | sed 's/^ *//;s/ *$//' | head -10 | nl | while read rank title; do
  repo_url=$(echo "$GH_HTML" | grep -A 5 "$title" | grep -oP 'href="(/[^/]+/[^/"]+)"' | head -1 | sed 's/href="//;s/"$//')
  stars=$(echo "$GH_HTML" | grep -A 10 "$title" | grep -oP 'aria-label="[^"]*stars"[^>]*>\K[^<]+' | head -1 | tr -d ' ')

  [ -z "$stars" ] && stars="0"
  [ -n "$repo_url" ] && repo_url="https://github.com$repo_url"

  metadata="stars:$stars"
  echo "github,$rank,\"$title\",$repo_url,\"$metadata\",$GH_TIME" >> "$OUTPUT_CSV"
done

echo "   Found $(grep -c '^github,' "$OUTPUT_CSV") items in ${GH_TIME}ms"

# Test 3: Stack Overflow
echo "3. Extracting from Stack Overflow (Rust tag)..."
START_TIME=$(date +%s%3N)
SO_HTML=$($RIPTIDE extract --url "https://stackoverflow.com/questions/tagged/rust" --engine raw --no-wasm --local 2>/dev/null)
END_TIME=$(date +%s%3N)
SO_TIME=$((END_TIME - START_TIME))

# Parse Stack Overflow questions (first 10)
echo "$SO_HTML" | grep -oP 's-post-summary.*?</div></div></div>' | head -10 | nl | while read rank _; do
  question_id=$(echo "$SO_HTML" | grep -oP 'data-post-id="\K\d+' | sed -n "${rank}p")
  title=$(echo "$SO_HTML" | grep -oP "question-hyperlink.*?>\K[^<]+" | sed -n "${rank}p" | sed 's/"/\\"/g')
  votes=$(echo "$SO_HTML" | grep -oP 's-post-summary--stats-item-number[^>]*>\K-?\d+' | sed -n "${rank}p")
  answers=$(echo "$SO_HTML" | grep -oP 's-post-summary--stats-item-number[^>]*>\K\d+' | sed -n "$((rank * 2))p")

  [ -z "$votes" ] && votes="0"
  [ -z "$answers" ] && answers="0"
  [ -n "$question_id" ] && question_url="https://stackoverflow.com/questions/$question_id"

  metadata="votes:$votes|answers:$answers"
  echo "stackoverflow,$rank,\"$title\",$question_url,\"$metadata\",$SO_TIME" >> "$OUTPUT_CSV"
done

echo "   Found $(grep -c '^stackoverflow,' "$OUTPUT_CSV") items in ${SO_TIME}ms"

# Test 4: Coolblue Laptops
echo "4. Extracting from Coolblue (Laptops)..."
START_TIME=$(date +%s%3N)
CB_HTML=$($RIPTIDE extract --url "https://www.coolblue.nl/en/laptops" --engine raw --no-wasm --local 2>/dev/null)
END_TIME=$(date +%s%3N)
CB_TIME=$((END_TIME - START_TIME))

# Parse Coolblue product listings
echo "$CB_HTML" | grep -oP 'product-card.*?</article>' | head -10 | nl | while read rank _; do
  product_name=$(echo "$CB_HTML" | grep -oP 'product__title[^>]*>\K[^<]+' | sed -n "${rank}p" | sed 's/"/\\"/g')
  price=$(echo "$CB_HTML" | grep -oP 'sales-price[^>]*>\K[^<]+' | sed -n "${rank}p" | tr -d ' ')
  product_url=$(echo "$CB_HTML" | grep -oP 'href="(/en/p/[^"]+)"' | sed -n "${rank}p" | sed 's/href="//;s/"$//')

  [ -z "$price" ] && price="N/A"
  [ -n "$product_url" ] && product_url="https://www.coolblue.nl$product_url"

  metadata="price:$price"
  echo "coolblue,$rank,\"$product_name\",$product_url,\"$metadata\",$CB_TIME" >> "$OUTPUT_CSV"
done

echo "   Found $(grep -c '^coolblue,' "$OUTPUT_CSV") items in ${CB_TIME}ms"

echo ""
echo "=== Summary ==="
echo "Total items extracted: $(tail -n +2 "$OUTPUT_CSV" | wc -l)"
echo "Output saved to: $OUTPUT_CSV"
echo ""
echo "Sample data:"
head -20 "$OUTPUT_CSV" | column -t -s','
