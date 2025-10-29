# Spider Crawl Configuration Guide

Complete guide to crawling and extracting all events from any website.

## Quick Start

```python
from riptide_sdk import RipTideClient
from riptide_sdk.models import SpiderConfig

async with RipTideClient() as client:
    # Configure spider
    config = SpiderConfig(
        max_depth=3,              # How deep to follow links
        max_pages=100,            # Maximum pages to crawl
        respect_robots=True       # Be polite
    )

    # Start crawling with result_mode="urls" to get discovered URLs
    result = await client.spider.crawl(
        seed_urls=["https://example.com"],
        config=config,
        result_mode="urls"  # NEW: Returns discovered URLs!
    )

    # Access discovered URLs
    for url in result.result.discovered_urls:
        extraction = await client.extract.extract_markdown(url)
        # Save extraction...
```

---

## New Feature: result_mode Parameter

Control what data is returned from spider crawls:

```python
# Option 1: Statistics only (default, lightweight)
result = await client.spider.crawl(
    seed_urls=["https://example.com"],
    config=config,
    result_mode="stats"  # Returns only crawl statistics
)
print(f"Crawled {result.pages_crawled} pages")
# No URLs returned - minimal response

# Option 2: Statistics + URLs (comprehensive)
result = await client.spider.crawl(
    seed_urls=["https://example.com"],
    config=config,
    result_mode="urls"  # Returns statistics + discovered URLs
)
print(f"Discovered {len(result.result.discovered_urls)} URLs")
for url in result.result.discovered_urls:
    print(f"  • {url}")
```

### When to use each mode

**Use `result_mode="stats"` when:**
- Testing crawl configuration
- Monitoring performance
- Quick metrics checks
- You don't need the actual URLs

**Use `result_mode="urls"` when:**
- Building sitemap generators
- Content extraction pipelines
- SEO auditing
- You need the discovered URLs

**Default:** `"stats"` (for backwards compatibility)

---

## Spider Configuration Options

### Basic Settings

```python
SpiderConfig(
    # === Core Settings ===
    max_depth=3,                    # How deep to crawl (0 = seed only)
    max_pages=100,                  # Maximum total pages
    follow_links=True,              # Follow <a> tags
    respect_robots_txt=True,        # Honor robots.txt

    # === Performance ===
    max_concurrent_requests=5,      # Parallel requests
    delay_between_requests=1.0,     # Seconds between requests
    timeout_secs=30,                # Request timeout

    # === Link Filtering ===
    allowed_domains=["example.com"], # Only these domains
    url_pattern=".*events.*",       # Regex: only matching URLs
    exclude_pattern=".*admin.*",    # Regex: skip matching URLs

    # === Content Settings ===
    extract_content=True,           # Extract text content
    extract_links=True,             # Extract all links
    extract_metadata=True,          # Extract meta tags

    # === User Agent ===
    user_agent="MyBot/1.0"         # Custom user agent
)
```

---

## Common Use Cases

### 1. Crawl Event Listings

**Goal:** Get all events from a site

```python
config = SpiderConfig(
    max_depth=2,                    # Main page + event pages
    max_pages=50,                   # Reasonable limit
    concurrency=5,                  # Crawl 5 pages at a time
    delay_ms=1000,                  # 1 second delay
    respect_robots=True             # Be polite
)

# Use result_mode="urls" to get discovered URLs
result = await client.spider.crawl(
    seed_urls=["https://example.com/events"],
    config=config,
    result_mode="urls"
)

# Extract URLs containing "event" or "agenda"
event_urls = [
    url for url in result.result.discovered_urls
    if 'event' in url or 'agenda' in url
]
```

### 2. Crawl Entire Website

**Goal:** Get everything

```python
config = SpiderConfig(
    max_depth=5,                    # Deep crawl
    max_pages=500,                  # Large site
    follow_links=True,
    allowed_domains=["example.com"], # Stay on domain
    max_concurrent_requests=10,     # Fast crawling
    delay_between_requests=0.5
)
```

### 3. Crawl Specific Section

**Goal:** Only get blog posts

```python
config = SpiderConfig(
    max_depth=3,
    max_pages=100,
    url_pattern=".*/blog/.*",       # Only /blog/ URLs
    exclude_pattern=".*/tag/.*",    # Skip tag pages
    follow_links=True
)
```

### 4. Crawl with Progress Tracking

**Goal:** Monitor progress in real-time

```python
async def on_progress(status):
    print(f"Crawled: {status.pages_crawled}/{status.total_pages}")

result = await client.spider.crawl_with_status_polling(
    seed_urls=["https://example.com"],
    config=config,
    poll_interval=5.0,  # Check every 5 seconds
    callback=on_progress
)
```

### 5. Gentle Crawling (Be Nice)

**Goal:** Don't overload the server

```python
config = SpiderConfig(
    max_depth=2,
    max_pages=50,
    max_concurrent_requests=2,      # Only 2 at a time
    delay_between_requests=2.0,     # 2 seconds between
    respect_robots_txt=True,
    timeout_secs=30
)
```

### 6. Fast Crawling (Aggressive)

**Goal:** Get data quickly

```python
config = SpiderConfig(
    max_depth=3,
    max_pages=200,
    max_concurrent_requests=20,     # Many concurrent
    delay_between_requests=0.1,     # Fast
    timeout_secs=10
)
```

---

## Output Formats

### 1. Extract as Markdown

```python
for page in result.pages:
    extraction = await client.extract.extract_markdown(page.url)

    with open(f"page_{idx}.md", "w") as f:
        f.write(extraction.content)
```

### 2. Extract as Article

```python
for page in result.pages:
    article = await client.extract.extract_article(page.url)

    # Get structured data
    print(f"Title: {article.title}")
    print(f"Author: {article.metadata.author}")
    print(f"Published: {article.metadata.published_date}")
    print(f"Content: {article.content}")
```

### 3. Extract as JSON

```python
for page in result.pages:
    extraction = await client.extract.extract(
        page.url,
        options=ExtractOptions(output_format="json")
    )

    # Save as JSON
    import json
    with open(f"page_{idx}.json", "w") as f:
        json.dump(extraction.to_dict(), f, indent=2)
```

---

## Advanced Patterns

### Pattern 1: Two-Phase Crawl

**First crawl to discover, then extract:**

```python
# Phase 1: Discover all URLs
discovery_config = SpiderConfig(
    max_depth=3,
    max_pages=100,
    concurrency=10
)

result = await client.spider.crawl(
    seed_urls=["https://example.com"],
    config=discovery_config,
    result_mode="urls"  # Get discovered URLs
)

# Phase 2: Extract each URL with custom options
for url in result.result.discovered_urls:
    extraction = await client.extract.extract_markdown(url)
    # Process extraction...
```

### Pattern 2: Filter and Extract

**Only extract certain types of pages:**

```python
result = await client.spider.crawl(
    seed_urls,
    config,
    result_mode="urls"  # Get discovered URLs
)

# Filter for event pages only
event_urls = [
    url for url in result.result.discovered_urls
    if 'event' in url or 'agenda' in url
]

# Extract only events
for url in event_urls:
    extraction = await client.extract.extract_markdown(url)
    # Process...
```

### Pattern 3: Parallel Extraction

**Extract multiple pages at once:**

```python
import asyncio

result = await client.spider.crawl(
    seed_urls,
    config,
    result_mode="urls"  # Get discovered URLs
)

# Create extraction tasks for first 10 URLs
tasks = [
    client.extract.extract_markdown(url)
    for url in result.result.discovered_urls[:10]
]

# Run in parallel
extractions = await asyncio.gather(*tasks, return_exceptions=True)

for extraction in extractions:
    if not isinstance(extraction, Exception):
        # Process successful extraction
        print(extraction.content)
```

---

## Performance Tips

### 1. Optimize Concurrent Requests

```python
# For small sites (< 100 pages)
max_concurrent_requests=3-5

# For medium sites (100-1000 pages)
max_concurrent_requests=10-15

# For large sites (1000+ pages)
max_concurrent_requests=20-50
```

### 2. Set Appropriate Delays

```python
# Public site (be respectful)
delay_between_requests=1.0-2.0

# Your own site
delay_between_requests=0.1-0.5

# API-friendly site
delay_between_requests=0.0-0.1
```

### 3. Use URL Patterns

```python
# Instead of crawling everything
max_pages=1000

# Crawl only what you need
url_pattern=".*/(events|news|blog).*"
max_pages=200  # Much faster!
```

### 4. Batch Processing

```python
# Process in batches to save memory
for i in range(0, len(result.pages), 10):
    batch = result.pages[i:i+10]

    for page in batch:
        extraction = await client.extract.extract_markdown(page.url)
        # Process and save
        # Clear from memory
```

---

## Error Handling

### Robust Crawling

```python
from riptide_sdk.exceptions import APIError, NetworkError, TimeoutError

try:
    result = await client.spider.crawl(seed_urls, config)

    for page in result.pages:
        try:
            extraction = await client.extract.extract_markdown(page.url)
            # Save extraction

        except (APIError, NetworkError, TimeoutError) as e:
            print(f"Error extracting {page.url}: {e}")
            continue  # Skip this page

except Exception as e:
    print(f"Crawl failed: {e}")
    # Retry or abort
```

---

## Complete Example

```python
#!/usr/bin/env python3
"""Complete crawl and extract example."""

import asyncio
from pathlib import Path
from riptide_sdk import RipTideClient
from riptide_sdk.models import SpiderConfig

async def crawl_and_extract_all(seed_url: str, output_dir: str):
    """Crawl entire site and extract all pages."""

    Path(output_dir).mkdir(exist_ok=True)

    async with RipTideClient() as client:
        # Configure spider
        config = SpiderConfig(
            max_depth=3,
            max_pages=100,
            follow_links=True,
            respect_robots_txt=True,
            max_concurrent_requests=5,
            delay_between_requests=1.0
        )

        # Progress callback
        async def on_progress(status):
            print(f"Progress: {status.pages_crawled} pages")

        # Crawl with progress
        result = await client.spider.crawl_with_status_polling(
            seed_urls=[seed_url],
            config=config,
            poll_interval=5.0,
            callback=on_progress
        )

        print(f"\nCrawled {len(result.pages)} pages")

        # Extract each page
        for idx, page in enumerate(result.pages, 1):
            try:
                extraction = await client.extract.extract_markdown(page.url)

                filename = f"{output_dir}/page_{idx:03d}.md"
                with open(filename, "w") as f:
                    f.write(f"# {extraction.title}\n\n")
                    f.write(f"**URL:** {extraction.url}\n\n")
                    f.write(extraction.content)

                print(f"[{idx}/{len(result.pages)}] Saved: {filename}")

            except Exception as e:
                print(f"Error on {page.url}: {e}")

        print(f"\n✅ Complete! Saved to {output_dir}/")

if __name__ == "__main__":
    asyncio.run(crawl_and_extract_all(
        seed_url="https://example.com",
        output_dir="extracted_content"
    ))
```

---

## Scripts Available

I created these ready-to-use scripts for you:

1. **`crawl_all_events.py`** - Full-featured spider crawler
2. **`extract_events.py`** - Single URL extraction
3. **`examples/spider_example.py`** - Spider API examples
4. **`examples/extract_example.py`** - Extract API examples

---

## Run the Crawler

```bash
cd /workspaces/eventmesh/sdk/python

# Run the spider crawler
PYTHONPATH=. python crawl_all_events.py

# Or single URL extraction
PYTHONPATH=. python extract_events.py "https://example.com" output.md
```

---

**Ready to crawl!** 🕷️
