# RipTide Python SDK

Official Python SDK for the RipTide web crawling and extraction API.

## Features

- 🚀 **Async/await support** - Built on httpx for high-performance async operations
- 📊 **Type hints** - Full type annotations for Python 3.8+
- 🔄 **Streaming support** - NDJSON and SSE streaming for real-time results
- 🎯 **All Phase 10+ endpoints** - Complete API coverage including:
  - Domain Profiles (Phase 10.4) - Warm-start caching
  - Engine Selection (Phase 10) - Intelligent engine selection
  - Batch crawling with caching
  - Streaming operations
- ⚡ **Connection pooling** - Efficient HTTP connection management
- 🛡️ **Error handling** - Comprehensive exception hierarchy

## Installation

```bash
pip install riptide-sdk
```

Or install from source:

```bash
cd /workspaces/eventmesh/sdk/python
pip install -e .
```

## Quick Start

```python
from riptide_sdk import RipTideClient
import asyncio

async def main():
    async with RipTideClient(base_url="http://localhost:8080") as client:
        # Batch crawl
        result = await client.crawl.batch([
            "https://example.com",
            "https://example.org"
        ])

        print(f"Successful: {result.successful}/{result.total_urls}")
        for item in result.results:
            print(f"  {item.url}: {item.status}")

asyncio.run(main())
```

## Usage Examples

### Batch Crawling

```python
from riptide_sdk import RipTideClient, CrawlOptions, CacheMode

async with RipTideClient() as client:
    # Basic crawl
    result = await client.crawl.batch(["https://example.com"])

    # With options
    options = CrawlOptions(
        cache_mode=CacheMode.READ_WRITE,
        concurrency=10,
        use_spider=False
    )
    result = await client.crawl.batch(urls, options=options)

    # Single URL
    result = await client.crawl.single("https://example.com")
```

### Domain Profiles (Phase 10.4)

```python
from riptide_sdk.models import ProfileConfig, StealthLevel

async with RipTideClient() as client:
    # Create profile
    config = ProfileConfig(
        stealth_level=StealthLevel.HIGH,
        rate_limit=2.0,
        respect_robots_txt=True
    )
    profile = await client.profiles.create(
        "example.com",
        config=config
    )

    # Get profile stats
    stats = await client.profiles.get_stats("example.com")
    print(f"Cache hit rate: {stats.cache_hits / stats.total_requests:.2%}")

    # Warm cache
    await client.profiles.warm_cache("example.com", "https://example.com/page")

    # List all profiles
    profiles = await client.profiles.list()

    # Search profiles
    results = await client.profiles.search("example")
```

### Engine Selection (Phase 10)

```python
async with RipTideClient() as client:
    # Analyze HTML
    decision = await client.engine.analyze(
        html="<html>...</html>",
        url="https://example.com"
    )
    print(f"Recommended: {decision.engine} ({decision.confidence:.2%})")

    # Get engine stats
    stats = await client.engine.get_stats()
    print(f"Total decisions: {stats.total_decisions}")
    print(f"Raw: {stats.raw_count}, Headless: {stats.headless_count}")

    # Toggle probe-first mode
    await client.engine.toggle_probe_first(True)
```

### Streaming

```python
async with RipTideClient() as client:
    # NDJSON streaming
    async for result in client.streaming.crawl_ndjson(urls):
        print(f"Got result: {result.data['url']}")

    # SSE streaming
    async for event in client.streaming.crawl_sse(urls):
        if event.event_type == "result":
            print(event.data)

    # Deep search streaming
    async for result in client.streaming.deepsearch_ndjson("python"):
        print(f"Found: {result.data['url']}")
```

## API Reference

### RipTideClient

Main client class with the following attributes:

- `crawl`: CrawlAPI - Batch crawl operations
- `profiles`: ProfilesAPI - Domain profile management (Phase 10.4)
- `engine`: EngineSelectionAPI - Engine selection API (Phase 10)
- `streaming`: StreamingAPI - Streaming operations

### CrawlAPI

- `batch(urls, options)` - Batch crawl multiple URLs
- `single(url, options)` - Crawl a single URL

### ProfilesAPI (Phase 10.4)

- `create(domain, config, metadata)` - Create domain profile
- `get(domain)` - Get profile
- `update(domain, config, metadata)` - Update profile
- `delete(domain)` - Delete profile
- `list(filter_query)` - List all profiles
- `get_stats(domain)` - Get profile statistics
- `get_metrics()` - Get aggregated metrics
- `batch_create(profiles)` - Batch create profiles
- `search(query)` - Search profiles
- `warm_cache(domain, url)` - Warm engine cache
- `clear_all_caches()` - Clear all caches

### EngineSelectionAPI (Phase 10)

- `analyze(html, url)` - Analyze HTML and recommend engine
- `decide(html, url, flags)` - Make engine decision with flags
- `get_stats()` - Get engine usage statistics
- `toggle_probe_first(enabled)` - Toggle probe-first mode

### StreamingAPI

- `crawl_ndjson(urls, options)` - Stream crawl results (NDJSON)
- `deepsearch_ndjson(query, limit, options)` - Stream search results (NDJSON)
- `crawl_sse(urls, options)` - Stream crawl results (SSE)

## Error Handling

```python
from riptide_sdk import (
    RipTideClient,
    ValidationError,
    APIError,
    NetworkError,
    TimeoutError
)

async with RipTideClient() as client:
    try:
        result = await client.crawl.batch([])
    except ValidationError as e:
        print(f"Validation error: {e}")
    except APIError as e:
        print(f"API error [{e.status_code}]: {e.message}")
        if e.is_retryable:
            print("This error can be retried")
    except NetworkError as e:
        print(f"Network error: {e}")
    except TimeoutError as e:
        print(f"Request timed out: {e}")
```

## Configuration

```python
from riptide_sdk import RipTideClient

client = RipTideClient(
    base_url="http://localhost:8080",  # API base URL
    api_key="your-api-key",            # Optional API key
    timeout=30.0,                      # Request timeout (seconds)
    max_connections=100,               # Max concurrent connections
)
```

## Development

```bash
# Install development dependencies
pip install -e ".[dev]"

# Run tests
pytest

# Format code
black .

# Type checking
mypy riptide_sdk

# Linting
ruff check .
```

## Requirements

- Python 3.8+
- httpx >= 0.25.0

## License

MIT License - see LICENSE file for details
