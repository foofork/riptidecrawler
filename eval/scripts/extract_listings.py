#!/usr/bin/env python3
"""
Extract listings from test URLs using riptide with raw engine.
Saves results to CSV format.
"""

import subprocess
import json
import csv
import time
import re
from html.parser import HTMLParser

# Test URLs from 30_listings.yml
TEST_URLS = [
    ("hackernews", "https://news.ycombinator.com/"),
    ("github", "https://github.com/topics/rust"),
    ("stackoverflow", "https://stackoverflow.com/questions/tagged/rust"),
    ("coolblue", "https://www.coolblue.nl/en/laptops")
]

OUTPUT_FILE = "/workspaces/eventmesh/eval/results/listings_test.csv"
RIPTIDE_BIN = "/usr/local/bin/riptide"


class HNParser(HTMLParser):
    """Parse Hacker News front page"""
    def __init__(self):
        super().__init__()
        self.items = []
        self.current_item = {}
        self.in_title = False
        self.in_subtext = False
        self.current_tag = None

    def handle_starttag(self, tag, attrs):
        attrs_dict = dict(attrs)

        # Detect item rank
        if 'class' in attrs_dict and 'rank' in attrs_dict['class']:
            self.current_item = {'rank': None}

        # Detect title link
        if tag == 'a' and 'class' in attrs_dict and 'titleline' in str(attrs):
            self.in_title = True
            if 'href' in attrs_dict:
                self.current_item['url'] = attrs_dict['href']

        # Detect score
        if 'class' in attrs_dict and 'score' in attrs_dict['class']:
            self.current_tag = 'score'

        # Detect author
        if 'class' in attrs_dict and 'hnuser' in attrs_dict['class']:
            self.current_tag = 'author'

    def handle_data(self, data):
        data = data.strip()
        if not data:
            return

        if self.in_title:
            self.current_item['title'] = data
            self.in_title = False

        if self.current_tag == 'score':
            match = re.search(r'(\d+) points?', data)
            if match:
                self.current_item['points'] = match.group(1)
            self.current_tag = None

        if self.current_tag == 'author':
            self.current_item['author'] = data
            self.current_tag = None

        # Detect rank number
        if data.endswith('.') and data[:-1].isdigit() and 'rank' not in self.current_item:
            self.current_item['rank'] = data[:-1]

        # Detect comments
        if 'comment' in data.lower():
            match = re.search(r'(\d+)', data)
            if match:
                self.current_item['comments'] = match.group(1)
                # Item complete, save it
                if 'title' in self.current_item and 'rank' in self.current_item:
                    self.items.append(self.current_item.copy())
                    self.current_item = {}


def extract_with_riptide(url, engine="raw"):
    """Extract content using riptide CLI"""
    try:
        start_time = time.time()
        result = subprocess.run(
            [RIPTIDE_BIN, "extract", "--url", url, "--engine", engine,
             "--no-wasm", "--local"],
            capture_output=True,
            text=True,
            timeout=30
        )
        end_time = time.time()

        if result.returncode == 0:
            return result.stdout, int((end_time - start_time) * 1000)
        else:
            print(f"Error extracting {url}: {result.stderr}")
            return None, 0

    except subprocess.TimeoutExpired:
        print(f"Timeout extracting {url}")
        return None, 0
    except Exception as e:
        print(f"Exception extracting {url}: {e}")
        return None, 0


def extract_hackernews(html):
    """Extract HN listings from HTML"""
    parser = HNParser()
    parser.feed(html)
    return parser.items[:10]  # Return top 10


def extract_github(html):
    """Extract GitHub repository listings"""
    items = []
    # Look for repository cards
    repo_pattern = r'<h3[^>]*>.*?<a href="/([^/"]+/[^/"]+)"[^>]*>([^<]+)</a>'
    matches = re.finditer(repo_pattern, html)

    rank = 1
    for match in matches:
        if rank > 10:
            break
        repo_path = match.group(1)
        repo_name = match.group(2).strip()

        # Try to find stars nearby
        stars_pattern = rf'{re.escape(repo_name)}.*?(\d+\.?\d*[kKmM]?)\s*stars?'
        stars_match = re.search(stars_pattern, html, re.DOTALL)
        stars = stars_match.group(1) if stars_match else "0"

        items.append({
            'rank': str(rank),
            'title': repo_name,
            'url': f"https://github.com/{repo_path}",
            'stars': stars
        })
        rank += 1

    return items


def extract_stackoverflow(html):
    """Extract Stack Overflow questions"""
    items = []
    # Look for question summaries
    question_pattern = r'data-post-id="(\d+)".*?question-hyperlink[^>]*>([^<]+)</a>'
    matches = re.finditer(question_pattern, html, re.DOTALL)

    rank = 1
    for match in matches:
        if rank > 10:
            break
        question_id = match.group(1)
        title = match.group(2).strip()

        items.append({
            'rank': str(rank),
            'title': title,
            'url': f"https://stackoverflow.com/questions/{question_id}",
            'votes': "0"  # Would need more complex parsing
        })
        rank += 1

    return items


def extract_coolblue(html):
    """Extract Coolblue product listings"""
    items = []
    # Look for product names and prices
    product_pattern = r'product[_-]?title[^>]*>([^<]+)</.*?price[^>]*>([^<]+)<'
    matches = re.finditer(product_pattern, html, re.DOTALL)

    rank = 1
    for match in matches:
        if rank > 10:
            break
        name = match.group(1).strip()
        price = match.group(2).strip()

        items.append({
            'rank': str(rank),
            'title': name,
            'url': "https://www.coolblue.nl/en/laptops",  # Would need link extraction
            'price': price
        })
        rank += 1

    return items


def main():
    """Main extraction logic"""
    print("=== Testing Listings Extraction ===")
    print(f"Output: {OUTPUT_FILE}\n")

    # Create CSV file
    with open(OUTPUT_FILE, 'w', newline='', encoding='utf-8') as csvfile:
        writer = csv.writer(csvfile)
        writer.writerow(['source', 'rank', 'title', 'url', 'metadata', 'extraction_time_ms'])

        total_items = 0

        for source, url in TEST_URLS:
            print(f"Extracting from {source} ({url})...")

            html, extract_time = extract_with_riptide(url)
            if not html:
                print(f"  ✗ Failed to extract content")
                continue

            # Parse based on source
            items = []
            if source == "hackernews":
                items = extract_hackernews(html)
                for item in items:
                    metadata = f"points:{item.get('points', '0')}|author:{item.get('author', 'unknown')}|comments:{item.get('comments', '0')}"
                    writer.writerow([source, item.get('rank', '?'), item.get('title', ''),
                                   item.get('url', ''), metadata, extract_time])

            elif source == "github":
                items = extract_github(html)
                for item in items:
                    metadata = f"stars:{item.get('stars', '0')}"
                    writer.writerow([source, item['rank'], item['title'],
                                   item['url'], metadata, extract_time])

            elif source == "stackoverflow":
                items = extract_stackoverflow(html)
                for item in items:
                    metadata = f"votes:{item.get('votes', '0')}"
                    writer.writerow([source, item['rank'], item['title'],
                                   item['url'], metadata, extract_time])

            elif source == "coolblue":
                items = extract_coolblue(html)
                for item in items:
                    metadata = f"price:{item.get('price', 'N/A')}"
                    writer.writerow([source, item['rank'], item['title'],
                                   item['url'], metadata, extract_time])

            print(f"  ✓ Found {len(items)} items in {extract_time}ms")
            total_items += len(items)

    print(f"\n=== Summary ===")
    print(f"Total items extracted: {total_items}")
    print(f"Output saved to: {OUTPUT_FILE}")

    # Show sample
    print("\nSample data:")
    with open(OUTPUT_FILE, 'r', encoding='utf-8') as f:
        lines = f.readlines()[:11]  # Header + 10 rows
        for line in lines:
            print(line.rstrip())


if __name__ == "__main__":
    main()
