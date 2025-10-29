# Spider API result_mode Parameter Documentation

## Overview

The `result_mode` parameter controls what data is returned from spider crawl operations. This feature allows users to choose between lightweight statistics or comprehensive URL discovery based on their needs.

**API Endpoint:** `POST /api/v1/spider/crawl`

**Added in:** Phase 1 (Target: Q4 2025)

---

## Quick Reference

```python
# Statistics only (lightweight, default)
result = await client.spider.crawl(
    seed_urls=["https://example.com"],
    result_mode="stats"  # default
)

# Statistics + discovered URLs (comprehensive)
result = await client.spider.crawl(
    seed_urls=["https://example.com"],
    result_mode="urls"
)
```

---

## Parameter Specification

### `result_mode`

**Type:** `string` (enum)
**Required:** No
**Default:** `"stats"`
**Values:**
- `"stats"` - Returns only crawl statistics
- `"urls"` - Returns statistics + list of discovered URLs

---

## Response Formats

### Mode: `stats` (Default)

Returns lightweight crawl statistics without URLs.

**Response Structure:**
```json
{
  "result": {
    "pages_crawled": 42,
    "pages_failed": 3,
    "duration_seconds": 5.23,
    "stop_reason": "max_pages_reached",
    "domains": ["example.com"]
  },
  "state": {
    "active": false,
    "pages_crawled": 42,
    "pages_failed": 3,
    "frontier_size": 0,
    "domains_seen": 1
  },
  "performance": {
    "pages_per_second": 8.03,
    "avg_response_time_ms": 124.5,
    "memory_usage": 1048576,
    "error_rate": 0.067
  }
}
```

**Use Cases:**
- Quick crawl metrics
- Performance monitoring
- Budget validation
- Rate limit testing

**Benefits:**
- Minimal response size
- Faster processing
- Lower memory usage
- Ideal for monitoring/analytics

---

### Mode: `urls`

Returns comprehensive data including all discovered URLs.

**Response Structure:**
```json
{
  "result": {
    "pages_crawled": 42,
    "pages_failed": 3,
    "duration_seconds": 5.23,
    "stop_reason": "max_pages_reached",
    "domains": ["example.com"],
    "discovered_urls": [
      "https://example.com",
      "https://example.com/page1",
      "https://example.com/page2",
      "https://example.com/about",
      // ... all 42 URLs
    ]
  },
  "state": { ... },
  "performance": { ... }
}
```

**Use Cases:**
- Sitemap generation
- Content extraction pipelines
- SEO auditing
- Link validation
- Archive/backup workflows

**Benefits:**
- Complete URL inventory
- Ready for downstream processing
- No need for separate URL discovery
- Single API call for discovery + extraction

---

## Usage Examples

### Python SDK

#### Example 1: Basic Statistics

```python
from riptide_sdk import RipTideClient
from riptide_sdk.models import SpiderConfig

async with RipTideClient() as client:
    # Get stats only (default)
    result = await client.spider.crawl(
        seed_urls=["https://example.com"],
        config=SpiderConfig(max_pages=50),
        # result_mode="stats" is default
    )

    print(f"Crawled {result.pages_crawled} pages")
    print(f"Duration: {result.duration_seconds:.2f}s")
    print(f"Domains: {', '.join(result.domains)}")
```

#### Example 2: URL Discovery

```python
async with RipTideClient() as client:
    # Request URLs to be returned
    result = await client.spider.crawl(
        seed_urls=["https://example.com"],
        config=SpiderConfig(
            max_depth=3,
            max_pages=100,
        ),
        result_mode="urls"  # ← Returns discovered URLs
    )

    # Access discovered URLs
    print(f"Discovered {len(result.result.discovered_urls)} URLs:")
    for url in result.result.discovered_urls:
        print(f"  • {url}")
```

#### Example 3: Crawl → Extract Pipeline

```python
async with RipTideClient() as client:
    # Step 1: Discover all URLs
    crawl_result = await client.spider.crawl(
        seed_urls=["https://blog.example.com"],
        config=SpiderConfig(
            max_depth=2,
            url_pattern=".*article.*",  # Only articles
        ),
        result_mode="urls"
    )

    # Step 2: Extract content from each URL
    articles = []
    for url in crawl_result.result.discovered_urls:
        extraction = await client.extract.extract_markdown(url)
        articles.append({
            'url': url,
            'title': extraction.title,
            'content': extraction.content,
        })

    print(f"Extracted {len(articles)} articles")
```

---

### cURL

#### Statistics Mode

```bash
curl -X POST http://localhost:8080/api/v1/spider/crawl \
  -H "Content-Type: application/json" \
  -d '{
    "seed_urls": ["https://example.com"],
    "max_pages": 50,
    "result_mode": "stats"
  }'
```

#### URLs Mode

```bash
curl -X POST http://localhost:8080/api/v1/spider/crawl \
  -H "Content-Type: application/json" \
  -d '{
    "seed_urls": ["https://example.com"],
    "max_pages": 50,
    "max_depth": 3,
    "result_mode": "urls"
  }' | jq '.result.discovered_urls'
```

---

### JavaScript/Node.js

```javascript
const response = await fetch('http://localhost:8080/api/v1/spider/crawl', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    seed_urls: ['https://example.com'],
    max_pages: 50,
    result_mode: 'urls'  // Request URLs
  })
});

const result = await response.json();
const urls = result.result.discovered_urls;

console.log(`Discovered ${urls.length} URLs`);
urls.forEach(url => console.log(`  • ${url}`));
```

---

### Rust

```rust
use riptide_sdk::RipTideClient;
use riptide_sdk::models::{SpiderConfig, ResultMode};

let client = RipTideClient::new("http://localhost:8080");

let result = client.spider.crawl(
    vec!["https://example.com".to_string()],
    SpiderConfig {
        max_pages: Some(50),
        max_depth: Some(3),
        result_mode: Some(ResultMode::Urls),  // Request URLs
        ..Default::default()
    }
).await?;

if let Some(urls) = result.result.discovered_urls {
    println!("Discovered {} URLs:", urls.len());
    for url in urls {
        println!("  • {}", url);
    }
}
```

---

## Migration Guide

### For Existing Users

If you're currently using the spider API, no changes are required! The default behavior remains unchanged.

**Current code:**
```python
result = await client.spider.crawl(seed_urls=["https://example.com"])
# Returns statistics only
```

**To get URLs (new feature):**
```python
result = await client.spider.crawl(
    seed_urls=["https://example.com"],
    result_mode="urls"  # ← Add this parameter
)
# Now returns statistics + discovered_urls
```

### Comparison with Industry Standards

This feature aligns RipTide with industry-standard crawlers:

| Feature | Scrapy | Firecrawl | RipTide (Phase 1) |
|---------|--------|-----------|-------------------|
| Returns URLs | ✅ Yes | ✅ Yes | ✅ Yes (`urls` mode) |
| Returns content | ✅ Yes | ✅ Yes | ⏳ Phase 2 |
| Lightweight mode | ❌ No | ❌ No | ✅ Yes (`stats` mode) |

---

## Performance Considerations

### Response Size

**Stats mode:**
- Response size: ~500 bytes
- Processing time: <10ms
- Memory: Minimal

**URLs mode:**
- Response size: ~50KB (1000 URLs @ 50 bytes avg)
- Processing time: <50ms
- Memory: ~100KB per 1000 URLs

### Recommendations

Use `stats` mode when:
- ✅ You only need metrics/analytics
- ✅ Testing crawl configuration
- ✅ Monitoring performance
- ✅ Budget validation

Use `urls` mode when:
- ✅ Building sitemap generators
- ✅ Content extraction pipelines
- ✅ SEO auditing tools
- ✅ Archive/backup systems

---

## Best Practices

### 1. Start with Stats Mode

Test your crawl configuration with `stats` mode first:

```python
# Test configuration
test_result = await client.spider.crawl(
    seed_urls=["https://example.com"],
    config=SpiderConfig(max_pages=10),
    result_mode="stats"  # Quick test
)

if test_result.pages_crawled > 0:
    # Config looks good, run full crawl
    full_result = await client.spider.crawl(
        seed_urls=["https://example.com"],
        config=SpiderConfig(max_pages=1000),
        result_mode="urls"  # Get all URLs
    )
```

### 2. Handle Large Result Sets

For large crawls, consider pagination or chunking:

```python
# Crawl in chunks
urls_per_batch = 100
total_crawled = 0

while total_crawled < target_pages:
    result = await client.spider.crawl(
        seed_urls=seed_urls,
        config=SpiderConfig(max_pages=urls_per_batch),
        result_mode="urls"
    )

    # Process this batch
    process_urls(result.result.discovered_urls)
    total_crawled += result.pages_crawled
```

### 3. Error Handling

```python
from riptide_sdk.exceptions import APIError, ValidationError

try:
    result = await client.spider.crawl(
        seed_urls=urls,
        result_mode="urls"
    )

    if not result.result.discovered_urls:
        print("Warning: No URLs discovered")

except ValidationError as e:
    print(f"Invalid configuration: {e}")
except APIError as e:
    print(f"API error: {e.message}")
```

---

## Error Responses

### Invalid result_mode

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid result_mode. Must be 'stats' or 'urls'",
    "details": {
      "field": "result_mode",
      "provided": "invalid_value"
    }
  }
}
```

**HTTP Status:** `400 Bad Request`

---

## Future Enhancements (Phase 2)

The following features are planned for future releases:

### 1. `pages` Mode (Phase 2)

Return full page data including content:

```json
{
  "result_mode": "pages",
  "pages": [
    {
      "url": "https://example.com",
      "status_code": 200,
      "depth": 0,
      "title": "Example Domain",
      "content": "Full extracted content...",
      "links": ["https://example.com/page1", ...]
    }
  ]
}
```

### 2. Streaming Mode (Phase 2)

Real-time URL discovery via SSE/WebSocket:

```python
async for url in client.spider.crawl_stream(seed_urls):
    # Process URLs as they're discovered
    print(f"Found: {url}")
```

### 3. Storage Mode (Phase 2)

Auto-store discovered URLs to database/storage:

```python
result = await client.spider.crawl(
    seed_urls=urls,
    result_mode="store",
    storage_backend="redis"  # or "postgres", "s3"
)
# URLs stored automatically, not returned in response
```

---

## API Reference

### Request Schema

```typescript
{
  seed_urls: string[],           // Required: 1-50 URLs
  result_mode?: "stats" | "urls", // Optional: default "stats"
  max_depth?: number,            // Optional: 0-10
  max_pages?: number,            // Optional: 1-10000
  strategy?: "breadth_first" | "depth_first" | "best_first",
  concurrency?: number,          // Optional: 1-50
  delay_ms?: number,             // Optional: 0-5000
  respect_robots?: boolean,      // Optional: default true
  // ... other spider config options
}
```

### Response Schema (URLs mode)

```typescript
{
  result: {
    pages_crawled: number,
    pages_failed: number,
    duration_seconds: number,
    stop_reason: string,
    domains: string[],
    discovered_urls?: string[]  // Only in "urls" mode
  },
  state: {
    active: boolean,
    pages_crawled: number,
    pages_failed: number,
    frontier_size: number,
    domains_seen: number
  },
  performance: {
    pages_per_second: number,
    avg_response_time_ms: number,
    memory_usage: number,
    error_rate: number
  }
}
```

---

## Support

- **Documentation:** `/docs/API_SPIDER_RESULT_MODE.md`
- **Examples:** `/sdk/python/crawl_all_events.py`
- **Configuration Guide:** `/sdk/python/SPIDER_CONFIGURATION_GUIDE.md`
- **User Expectations:** `/sdk/python/SPIDER_USER_EXPECTATIONS.md`

---

**Last Updated:** 2025-10-29
**Version:** Phase 1 (Target)
**Status:** 🟡 Planned Implementation
