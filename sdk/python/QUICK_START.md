# RipTide Python SDK - Quick Start Guide

## Installation

```bash
pip install -e /workspaces/eventmesh/sdk/python
```

## Basic Usage (Traditional)

```python
import asyncio
from riptide_sdk import RipTideClient

async def main():
    async with RipTideClient(base_url="http://localhost:8080") as client:
        result = await client.crawl.batch(["https://example.com"])
        print(result.to_summary())

asyncio.run(main())
```

## Modern Builder Pattern (Recommended)

```python
from riptide_sdk import RipTideClientBuilder

# Configure with fluent API
client = (RipTideClientBuilder()
    .with_base_url("http://localhost:8080")
    .with_api_key("your-api-key")
    .with_timeout(60.0)
    .with_max_connections(200)
    .with_retry_config(max_retries=3, backoff_factor=2.0)
    .build())

async with client:
    result = await client.crawl.batch(urls)
    print(result.to_markdown())  # Auto-formatted output!
```

## Output Formatters

```python
result = await client.crawl.batch(urls)

# Quick summary
print(result.to_summary())

# Detailed markdown
print(result.to_markdown())

# JSON for APIs
json_str = result.to_json(include_documents=False)

# Works with all response types
profile = await client.profiles.get("example.com")
print(profile.to_markdown())

stats = await client.engine.get_stats()
print(stats.to_summary())
```

## Parallel High-Throughput Crawling

```python
# Crawl 100 URLs efficiently
urls = [f"https://example.com/page/{i}" for i in range(100)]

results = await client.batch_crawl_parallel(
    urls,
    batch_size=10,      # 10 URLs per batch
    max_concurrent=5    # 5 batches at once
)

# Aggregate results
total_success = sum(r.successful for r in results if not isinstance(r, Exception))
```

## Enhanced Error Handling

```python
from riptide_sdk import ValidationError, APIError

try:
    result = await client.crawl.batch([])
except ValidationError as e:
    print(e)  # Includes helpful suggestion
    # Output:
    # URLs list cannot be empty
    #
    # 💡 Suggestion: The 'urls' field is required and cannot be empty.
    # 📚 Documentation: https://github.com/...

except APIError as e:
    if e.is_retryable:
        # Retry logic for 429, 500, 502, 503, 504
        await asyncio.sleep(e.backoff_time)
        # retry...
```

## Configuration Options

```python
builder = RipTideClientBuilder()

# Connection
builder.with_base_url("http://localhost:8080")
builder.with_api_key("sk_test_...")

# Timeouts & Performance
builder.with_timeout(60.0)
builder.with_max_connections(200)
builder.with_max_keepalive(20)

# Retry Logic
builder.with_retry_config(
    max_retries=3,
    backoff_factor=2.0,
    max_backoff=60.0
)

# Headers
builder.with_user_agent("MyApp/1.0")
builder.with_custom_header("X-Custom", "value")

# Security
builder.with_ssl_verification(True)
builder.with_follow_redirects(True)

client = builder.build()
```

## Advanced Usage

### Domain Profiles

```python
from riptide_sdk import ProfileConfig, StealthLevel, UAStrategy

# Create profile
config = ProfileConfig(
    stealth_level=StealthLevel.HIGH,
    rate_limit=2.0,
    ua_strategy=UAStrategy.ROTATE,
    enable_javascript=True
)

profile = await client.profiles.create("example.com", config=config)
print(profile.to_markdown())

# Use profile for crawling
result = await client.crawl.batch(urls)  # Auto-applies profile
```

### Streaming Operations

```python
# NDJSON streaming
async for item in client.streaming.crawl_ndjson(urls):
    print(f"Received: {item.url}")
    if item.document:
        print(item.document.markdown)

# Server-Sent Events
async for event in client.streaming.crawl_sse(urls):
    print(f"Event: {event.event_type}")
```

### Custom Options

```python
from riptide_sdk import CrawlOptions, CacheMode, ChunkingConfig

options = CrawlOptions(
    cache_mode=CacheMode.READ_WRITE,
    concurrency=10,
    timeout_secs=30,
    use_spider=True,
    chunking_config=ChunkingConfig(
        enabled=True,
        max_chunk_size=1000,
        overlap=100
    )
)

result = await client.crawl.batch(urls, options=options)
```

## Type Hints & IDE Support

All methods have full type hints:

```python
from typing import List
from riptide_sdk import CrawlResponse, CrawlResult

async def process_results(urls: List[str]) -> List[CrawlResult]:
    result: CrawlResponse = await client.crawl.batch(urls)
    return result.results  # IDE knows this is List[CrawlResult]
```

## Examples

See the `examples/` directory for more:
- `examples/builder_example.py` - Builder pattern usage
- `examples/formatters_example.py` - Output formatting
- `examples/basic_crawl.py` - Simple crawling
- `examples/streaming_example.py` - Streaming operations

## Quick Reference

| Task | Code |
|------|------|
| Build client | `RipTideClientBuilder().with_base_url(...).build()` |
| Batch crawl | `await client.crawl.batch(urls)` |
| Format output | `result.to_markdown()` / `.to_summary()` / `.to_json()` |
| Parallel crawl | `await client.batch_crawl_parallel(urls)` |
| Create profile | `await client.profiles.create(domain, config)` |
| Get stats | `await client.engine.get_stats()` |
| Stream results | `async for item in client.streaming.crawl_ndjson(urls)` |
| Health check | `await client.health_check()` |

## Common Patterns

### Retry with Exponential Backoff

```python
from riptide_sdk import APIError
import asyncio

async def crawl_with_retry(urls, max_retries=3):
    for attempt in range(max_retries):
        try:
            return await client.crawl.batch(urls)
        except APIError as e:
            if not e.is_retryable or attempt == max_retries - 1:
                raise
            backoff = 2 ** attempt
            await asyncio.sleep(backoff)
```

### Process Large URL Lists

```python
# Split large lists efficiently
all_urls = [...]  # 1000+ URLs

results = await client.batch_crawl_parallel(
    all_urls,
    batch_size=20,
    max_concurrent=10
)

# Filter successful results
successful = [
    r for r in results
    if not isinstance(r, Exception) and r.successful > 0
]
```

### Export to Multiple Formats

```python
result = await client.crawl.batch(urls)

# Save in different formats
with open("summary.txt", "w") as f:
    f.write(result.to_summary())

with open("report.md", "w") as f:
    f.write(result.to_markdown())

with open("data.json", "w") as f:
    f.write(result.to_json())
```

## Support

- **Documentation**: `/workspaces/eventmesh/sdk/python/README.md`
- **Examples**: `/workspaces/eventmesh/sdk/python/examples/`
- **Tests**: `/workspaces/eventmesh/sdk/python/tests/`
- **Issues**: GitHub Issues

---

**Version**: 0.1.0
**Updated**: 2025-10-28
**Status**: Production Ready ✅
