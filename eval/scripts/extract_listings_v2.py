#!/usr/bin/env python3
"""
Extract listings from test URLs using riptide with raw engine.
Saves results to CSV format using regex parsing.
"""

import subprocess
import csv
import time
import re
from typing import List, Dict, Tuple

# Test URLs from 30_listings.yml
TEST_URLS = [
    ("hackernews", "https://news.ycombinator.com/"),
    ("github", "https://github.com/topics/rust"),
    ("stackoverflow", "https://stackoverflow.com/questions/tagged/rust"),
    ("coolblue", "https://www.coolblue.nl/en/laptops")
]

OUTPUT_FILE = "/workspaces/eventmesh/eval/results/listings_test.csv"
RIPTIDE_BIN = "/usr/local/bin/riptide"


def extract_with_riptide(url: str, engine: str = "raw") -> Tuple[str, int]:
    """Extract content using riptide CLI"""
    try:
        start_time = time.time()
        result = subprocess.run(
            [RIPTIDE_BIN, "extract", "--url", url, "--engine", engine,
             "--no-wasm", "--local"],
            capture_output=True,
            text=True,
            timeout=60
        )
        end_time = time.time()

        if result.returncode == 0:
            # Extract just the HTML content (after "Extracted Content" line)
            html = result.stdout
            if "Extracted Content" in html:
                html = html.split("Extracted Content", 1)[1]
            return html, int((end_time - start_time) * 1000)
        else:
            print(f"  Error: {result.stderr[:200]}")
            return None, 0

    except subprocess.TimeoutExpired:
        print(f"  Timeout after 60s")
        return None, 0
    except Exception as e:
        print(f"  Exception: {e}")
        return None, 0


def extract_hackernews(html: str) -> List[Dict]:
    """Extract HN listings from HTML"""
    items = []

    # Pattern to find each story row
    story_pattern = r'<tr class="athing submission"[^>]*>(.*?)</tr>\s*<tr><td colspan="2"></td><td class="subtext">(.*?)</td></tr>'
    matches = re.finditer(story_pattern, html, re.DOTALL)

    for match in matches:
        if len(items) >= 10:
            break

        title_row = match.group(1)
        meta_row = match.group(2)

        # Extract rank
        rank_match = re.search(r'<span class="rank">(\d+)\.</span>', title_row)
        rank = rank_match.group(1) if rank_match else "?"

        # Extract title and URL
        title_match = re.search(r'<span class="titleline"><a href="([^"]+)">([^<]+)</a>', title_row)
        if not title_match:
            continue

        url = title_match.group(1)
        title = title_match.group(2)

        # Extract points
        points_match = re.search(r'<span class="score"[^>]*>(\d+)\s+points?</span>', meta_row)
        points = points_match.group(1) if points_match else "0"

        # Extract author
        author_match = re.search(r'<a href="user\?id=[^"]+"[^>]*>([^<]+)</a>', meta_row)
        author = author_match.group(1) if author_match else "unknown"

        # Extract comments
        comments_match = re.search(r'>(\d+)&nbsp;comments?</a>', meta_row)
        comments = comments_match.group(1) if comments_match else "0"

        items.append({
            'rank': rank,
            'title': title,
            'url': url,
            'metadata': f"points:{points}|author:{author}|comments:{comments}"
        })

    return items


def extract_github(html: str) -> List[Dict]:
    """Extract GitHub repository listings"""
    items = []

    # Look for repository links in topics page
    repo_pattern = r'<h3[^>]*>.*?<a[^>]*href="/([^/]+/[^/"]+)"[^>]*>([^<]+)</a>'
    matches = re.finditer(repo_pattern, html)

    for match in matches:
        if len(items) >= 10:
            break

        repo_path = match.group(1).strip()
        repo_name = match.group(2).strip()

        # Try to find stars (approximate search in surrounding context)
        context_start = max(0, match.start() - 500)
        context_end = min(len(html), match.end() + 500)
        context = html[context_start:context_end]

        stars_match = re.search(r'(\d+(?:,\d+)?(?:\.\d+)?[kKmM]?)\s*(?:stars?|Star)', context, re.IGNORECASE)
        stars = stars_match.group(1) if stars_match else "0"

        items.append({
            'rank': str(len(items) + 1),
            'title': repo_name,
            'url': f"https://github.com/{repo_path}",
            'metadata': f"stars:{stars}"
        })

    return items


def extract_stackoverflow(html: str) -> List[Dict]:
    """Extract Stack Overflow questions"""
    items = []

    # Pattern for question summary cards
    question_pattern = r'data-post-id="(\d+)"[^>]*>.*?question-hyperlink[^>]*>([^<]+)</a>'
    matches = re.finditer(question_pattern, html, re.DOTALL)

    for match in matches:
        if len(items) >= 10:
            break

        question_id = match.group(1)
        title = match.group(2).strip()

        # Try to find votes nearby
        context_start = max(0, match.start() - 300)
        context_end = min(len(html), match.end() + 200)
        context = html[context_start:context_end]

        votes_match = re.search(r's-post-summary--stats-item-number[^>]*>(-?\d+)</span>', context)
        votes = votes_match.group(1) if votes_match else "0"

        items.append({
            'rank': str(len(items) + 1),
            'title': title,
            'url': f"https://stackoverflow.com/questions/{question_id}",
            'metadata': f"votes:{votes}"
        })

    return items


def extract_coolblue(html: str) -> List[Dict]:
    """Extract Coolblue product listings"""
    items = []

    # Look for product cards/links
    # Coolblue may have different structures, try multiple patterns
    patterns = [
        r'product[_-]?(?:title|name)[^>]*>([^<]+)</.*?(?:price|sales-price)[^>]*>([^<]+)<',
        r'href="(/en/p/[^"]+)"[^>]*>([^<]+)</a>',
    ]

    for pattern in patterns:
        matches = re.finditer(pattern, html, re.DOTALL | re.IGNORECASE)
        for match in matches:
            if len(items) >= 10:
                break

            if len(match.groups()) >= 2:
                # Try to extract product info
                if '/en/p/' in match.group(0):
                    url_part = match.group(1) if match.group(1).startswith('/') else None
                    name = match.group(2).strip() if match.group(2) else "Product"
                    price = "N/A"

                    if url_part:
                        items.append({
                            'rank': str(len(items) + 1),
                            'title': name,
                            'url': f"https://www.coolblue.nl{url_part}",
                            'metadata': f"price:{price}"
                        })

    return items


def main():
    """Main extraction logic"""
    print("=== Testing Listings Extraction with riptide (raw engine, no WASM) ===")
    print(f"Output: {OUTPUT_FILE}\n")

    # Create CSV file
    with open(OUTPUT_FILE, 'w', newline='', encoding='utf-8') as csvfile:
        writer = csv.writer(csvfile)
        writer.writerow(['source', 'rank', 'title', 'url', 'metadata', 'extraction_time_ms'])

        total_items = 0

        for source, url in TEST_URLS:
            print(f"{source:15} {url}")

            html, extract_time = extract_with_riptide(url)
            if not html:
                print(f"  ✗ Failed\n")
                continue

            # Parse based on source
            items = []
            if source == "hackernews":
                items = extract_hackernews(html)
            elif source == "github":
                items = extract_github(html)
            elif source == "stackoverflow":
                items = extract_stackoverflow(html)
            elif source == "coolblue":
                items = extract_coolblue(html)

            # Write to CSV
            for item in items:
                writer.writerow([source, item['rank'], item['title'],
                               item['url'], item['metadata'], extract_time])

            print(f"  ✓ {len(items):2d} items | {extract_time:4d}ms\n")
            total_items += len(items)

    print(f"=== Summary ===")
    print(f"Total items: {total_items}")
    print(f"CSV file: {OUTPUT_FILE}\n")

    # Display sample
    print("Sample results:")
    with open(OUTPUT_FILE, 'r', encoding='utf-8') as f:
        lines = f.readlines()
        for i, line in enumerate(lines[:6]):  # Header + 5 rows
            # Truncate long lines for display
            if len(line) > 120:
                line = line[:117] + "..."
            print(f"  {line.rstrip()}")

    if len(lines) > 6:
        print(f"  ... ({len(lines)-6} more rows)")


if __name__ == "__main__":
    main()
