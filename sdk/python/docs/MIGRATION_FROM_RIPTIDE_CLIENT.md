# Migration Guide: riptide-client → riptide-sdk

This guide helps you migrate from the legacy `riptide-client` package to the new official `riptide-sdk`.

## Table of Contents

1. [Why Migrate](#why-migrate)
2. [Installation Changes](#installation-changes)
3. [Import Changes](#import-changes)
4. [Breaking Changes](#breaking-changes)
5. [Feature Mapping](#feature-mapping)
6. [Code Examples](#code-examples)
7. [Migration Checklist](#migration-checklist)
8. [Troubleshooting](#troubleshooting)

---

## Why Migrate

### Benefits of riptide-sdk

The new `riptide-sdk` is a complete rewrite with significant improvements over `riptide-client`:

| Feature | riptide-client | riptide-sdk |
|---------|----------------|-------------|
| **API Coverage** | ~30% (basic endpoints) | **84%** (52/62 endpoints) |
| **Async/Await** | Limited support | Full async/await with httpx |
| **Type Hints** | Minimal | Complete type annotations |
| **Streaming** | Basic NDJSON only | NDJSON, SSE, WebSocket |
| **Error Handling** | Basic exceptions | Comprehensive exception hierarchy |
| **Connection Pooling** | None | Built-in connection pooling |
| **New Features** | None | Browser automation, sessions, workers, PDF processing, search |
| **Python Support** | 3.7+ | 3.8+ with modern typing |
| **Documentation** | Limited | Comprehensive with examples |
| **Maintenance** | Deprecated | Active development |

### Performance Improvements

- **Connection Pooling**: Up to 100 concurrent connections with keep-alive
- **Async Architecture**: Built on modern httpx library for better performance
- **Streaming**: Real-time processing with multiple streaming protocols
- **Memory Efficiency**: Optimized response handling and parsing

### New Features Not in riptide-client

1. **Browser Automation** (15 methods) - Direct browser control with stealth presets
2. **WebSocket Streaming** - Bidirectional real-time communication
3. **Extraction API** - Standalone content extraction (article, markdown, product)
4. **Search API** - Web search integration with content extraction
5. **Spider Crawling** - Deep multi-page site crawling with status tracking
6. **Session Management** - Persistent cookies and authentication
7. **PDF Processing** - Document extraction with progress tracking
8. **Worker Queue** - Async job management for long-running operations
9. **Domain Profiles** - Advanced caching and rate limiting
10. **Engine Selection** - Intelligent engine routing

---

## Installation Changes

### Old Installation (riptide-client)

```bash
# Legacy package
pip uninstall riptide-client

# or
pip uninstall riptide-client
```

### New Installation (riptide-sdk)

```bash
# Install riptide-sdk
pip install riptide-sdk

# Or from source
cd /path/to/eventmesh/sdk/python
pip install -e .

# With development dependencies
pip install -e ".[dev]"
```

### Dependencies

**riptide-client** dependencies:
- requests (synchronous HTTP)
- Limited type hints support

**riptide-sdk** dependencies:
- httpx >= 0.25.0 (async HTTP with HTTP/2 support)
- typing-extensions >= 4.0.0 (for Python < 3.10)
- websockets >= 12.0 (optional, for WebSocket streaming)

---

## Import Changes

### Basic Imports

```python
# OLD (riptide-client)
from riptide_client import RipTideClient
from riptide_client.models import CrawlOptions

# NEW (riptide-sdk)
from riptide_sdk import RipTideClient, CrawlOptions
```

### All Available Imports

```python
# NEW (riptide-sdk)
from riptide_sdk import (
    # Client
    RipTideClient,
    RipTideClientBuilder,

    # Crawl models
    CrawlResult,
    CrawlResponse,
    CrawlOptions,

    # Extraction
    ExtractOptions,
    ExtractionResult,
    ContentMetadata,
    ParserMetadata,

    # Search
    SearchOptions,
    SearchResponse,
    SearchResultItem,

    # Sessions
    Session,
    SessionConfig,
    SessionStats,
    Cookie,
    SetCookieRequest,

    # Spider
    SpiderConfig,
    SpiderResult,
    SpiderStatus,
    SpiderControlResponse,

    # PDF
    PdfExtractionOptions,
    PdfExtractionResult,
    PdfJobStatus,
    PdfMetrics,
    PdfStreamProgress,

    # Workers
    Job,
    JobConfig,
    JobResult,
    JobType,
    JobPriority,
    JobStatus,
    QueueStats,
    WorkerStats,
    ScheduledJob,
    ScheduledJobConfig,
    JobListItem,
    JobListResponse,

    # Domain Profiles
    DomainProfile,
    ProfileConfig,

    # Engine Selection
    EngineStats,

    # Streaming
    StreamingResult,

    # Enums
    CacheMode,
    StealthLevel,
    UAStrategy,
    ResultMode,

    # Exceptions
    RipTideError,
    ValidationError,
    APIError,
    NetworkError,
    TimeoutError,
    ConfigError,
    StreamingError,

    # Formatters
    format_crawl_response,
    format_domain_profile,
    format_engine_stats,
)
```

---

## Breaking Changes

### 1. Synchronous → Asynchronous

**MAJOR CHANGE**: All methods are now async and require async/await.

```python
# OLD (riptide-client) - Synchronous
client = RipTideClient("http://localhost:8080")
result = client.crawl(["https://example.com"])  # Blocks thread

# NEW (riptide-sdk) - Asynchronous
async with RipTideClient("http://localhost:8080") as client:
    result = await client.crawl.batch(["https://example.com"])  # Non-blocking
```

### 2. Client Initialization

```python
# OLD (riptide-client)
client = RipTideClient(
    base_url="http://localhost:8080",
    api_key="your-key"
)

# NEW (riptide-sdk) - Same interface, but use as context manager
async with RipTideClient(
    base_url="http://localhost:8080",
    api_key="your-key",
    timeout=30.0,              # NEW: configurable timeout
    max_connections=100        # NEW: connection pooling
) as client:
    # Use client
    pass

# NEW (riptide-sdk) - Alternative: Fluent builder pattern
from riptide_sdk import RipTideClientBuilder

client = (RipTideClientBuilder()
    .with_base_url("http://localhost:8080")
    .with_api_key("your-key")
    .with_timeout(60.0)
    .with_max_connections(200)
    .with_retry_config(max_retries=3, backoff_factor=2.0)
    .build())
```

### 3. Method Name Changes

| Old Method (riptide-client) | New Method (riptide-sdk) | Notes |
|-----------------------------|--------------------------|-------|
| `client.crawl(urls)` | `client.crawl.batch(urls)` | Now under `crawl` namespace |
| `client.crawl_single(url)` | `client.crawl.single(url)` | Consistent naming |
| `client.stream_crawl(urls)` | `client.streaming.crawl_ndjson(urls)` | Multiple streaming options |
| N/A | `client.streaming.crawl_sse(urls)` | NEW: Server-Sent Events |
| N/A | `client.streaming.crawl_websocket(urls)` | NEW: WebSocket streaming |

### 4. Response Object Changes

```python
# OLD (riptide-client)
result = client.crawl(urls)
print(result["successful"])  # Dictionary access
print(result["results"][0]["url"])

# NEW (riptide-sdk)
result = await client.crawl.batch(urls)
print(result.successful)  # Typed attributes
print(result.results[0].url)  # Fully typed objects

# NEW: Rich formatting methods
print(result.to_summary())    # Human-readable summary
print(result.to_markdown())   # Markdown format
print(result.to_dict())       # Dictionary (if needed)
```

### 5. Options/Configuration

```python
# OLD (riptide-client)
result = client.crawl(
    urls,
    cache_mode="read_write",
    concurrency=10
)

# NEW (riptide-sdk)
from riptide_sdk import CrawlOptions, CacheMode

options = CrawlOptions(
    cache_mode=CacheMode.READ_WRITE,  # Typed enum
    concurrency=10,
    use_spider=False
)
result = await client.crawl.batch(urls, options=options)
```

### 6. Error Handling

```python
# OLD (riptide-client)
try:
    result = client.crawl(urls)
except Exception as e:
    print(f"Error: {e}")

# NEW (riptide-sdk)
from riptide_sdk import ValidationError, APIError, NetworkError, TimeoutError

try:
    result = await client.crawl.batch(urls)
except ValidationError as e:
    print(f"Invalid input: {e}")
except APIError as e:
    print(f"API error [{e.status_code}]: {e.message}")
    if e.is_retryable:
        # Retry logic
        pass
except NetworkError as e:
    print(f"Network error: {e}")
except TimeoutError as e:
    print(f"Request timed out: {e}")
```

---

## Feature Mapping

### Core Crawling

| Feature | riptide-client | riptide-sdk |
|---------|----------------|-------------|
| Batch crawl | `client.crawl(urls)` | `client.crawl.batch(urls, options)` |
| Single URL | `client.crawl_single(url)` | `client.crawl.single(url, options)` |
| Parallel crawl | Manual implementation | `client.batch_crawl_parallel(urls, batch_size=10, max_concurrent=5)` |
| Options | Dict parameters | Typed `CrawlOptions` object |
| Cache control | String "cache_mode" | `CacheMode` enum (READ_ONLY, WRITE_ONLY, READ_WRITE, BYPASS) |

### Streaming

| Feature | riptide-client | riptide-sdk |
|---------|----------------|-------------|
| NDJSON stream | `client.stream_crawl(urls)` | `client.streaming.crawl_ndjson(urls)` |
| SSE stream | Not available | `client.streaming.crawl_sse(urls)` |
| WebSocket stream | Not available | `client.streaming.crawl_websocket(urls)` |
| Search stream | Not available | `client.streaming.deepsearch_ndjson(query)` |

### New Features (Not in riptide-client)

These features are exclusive to riptide-sdk:

1. **Content Extraction** - `client.extract.*`
2. **Web Search** - `client.search.*`
3. **Spider Crawling** - `client.spider.*`
4. **Session Management** - `client.sessions.*`
5. **PDF Processing** - `client.pdf.*`
6. **Worker Queue** - `client.workers.*`
7. **Browser Automation** - `client.browser.*`
8. **Domain Profiles** - `client.profiles.*`
9. **Engine Selection** - `client.engine.*`

### Deprecated Features

None - all riptide-client features are supported in riptide-sdk with improved interfaces.

---

## Code Examples

### Example 1: Simple Crawl

```python
# OLD (riptide-client)
from riptide_client import RipTideClient

client = RipTideClient("http://localhost:8080")
result = client.crawl(["https://example.com"])
print(f"Success: {result['successful']}")

for item in result['results']:
    print(item['url'], item['status'])
```

```python
# NEW (riptide-sdk)
import asyncio
from riptide_sdk import RipTideClient

async def main():
    async with RipTideClient("http://localhost:8080") as client:
        result = await client.crawl.batch(["https://example.com"])
        print(f"Success: {result.successful}")

        for item in result.results:
            print(item.url, item.status)

asyncio.run(main())
```

### Example 2: Batch Crawling with Options

```python
# OLD (riptide-client)
from riptide_client import RipTideClient

client = RipTideClient("http://localhost:8080")
result = client.crawl(
    urls=["https://example.com", "https://example.org"],
    cache_mode="read_write",
    concurrency=10
)
```

```python
# NEW (riptide-sdk)
import asyncio
from riptide_sdk import RipTideClient, CrawlOptions, CacheMode

async def main():
    async with RipTideClient("http://localhost:8080") as client:
        options = CrawlOptions(
            cache_mode=CacheMode.READ_WRITE,
            concurrency=10,
            use_spider=False
        )
        result = await client.crawl.batch(
            urls=["https://example.com", "https://example.org"],
            options=options
        )

asyncio.run(main())
```

### Example 3: Streaming Results

```python
# OLD (riptide-client)
from riptide_client import RipTideClient

client = RipTideClient("http://localhost:8080")
for result in client.stream_crawl(urls):
    print(result['url'], result['status'])
```

```python
# NEW (riptide-sdk)
import asyncio
from riptide_sdk import RipTideClient

async def main():
    async with RipTideClient("http://localhost:8080") as client:
        async for result in client.streaming.crawl_ndjson(urls):
            data = result.data
            print(data['url'], data['status'])

asyncio.run(main())
```

### Example 4: Error Handling

```python
# OLD (riptide-client)
from riptide_client import RipTideClient

client = RipTideClient("http://localhost:8080")
try:
    result = client.crawl([])
except Exception as e:
    print(f"Error: {e}")
```

```python
# NEW (riptide-sdk)
import asyncio
from riptide_sdk import (
    RipTideClient,
    ValidationError,
    APIError,
    NetworkError,
    TimeoutError
)

async def main():
    async with RipTideClient("http://localhost:8080") as client:
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

asyncio.run(main())
```

### Example 5: Session Management (NEW)

```python
# OLD (riptide-client)
# Not available

# NEW (riptide-sdk)
import asyncio
from riptide_sdk import RipTideClient, SessionConfig, SetCookieRequest

async def main():
    async with RipTideClient("http://localhost:8080") as client:
        # Create session
        config = SessionConfig(ttl_seconds=3600, stealth_mode=True)
        session = await client.sessions.create(config)

        # Set authentication cookie
        cookie = SetCookieRequest(
            domain="example.com",
            name="auth_token",
            value="secret123",
            secure=True,
            http_only=True
        )
        await client.sessions.set_cookie(session.session_id, cookie)

        # Crawl with session
        result = await client.crawl.batch(urls, session_id=session.session_id)

        # Cleanup
        await client.sessions.delete(session.session_id)

asyncio.run(main())
```

### Example 6: Content Extraction (NEW)

```python
# OLD (riptide-client)
# Not available - only full crawling

# NEW (riptide-sdk)
import asyncio
from riptide_sdk import RipTideClient

async def main():
    async with RipTideClient("http://localhost:8080") as client:
        # Extract article content
        article = await client.extract.extract_article("https://blog.example.com/post")
        print(f"Title: {article.title}")
        print(f"Author: {article.metadata.author}")
        print(f"Published: {article.metadata.published_date}")
        print(f"Content: {article.content[:200]}...")

        # Extract as markdown
        markdown = await client.extract.extract_markdown(url)
        print(markdown.content)

        # Extract product information
        product = await client.extract.extract_product("https://shop.example.com/item")
        print(f"Price: ${product.metadata.price}")

asyncio.run(main())
```

### Example 7: Spider Crawling (NEW)

```python
# OLD (riptide-client)
# Not available - only single-level crawling

# NEW (riptide-sdk)
import asyncio
from riptide_sdk import RipTideClient, SpiderConfig

async def main():
    async with RipTideClient("http://localhost:8080") as client:
        # Configure spider
        config = SpiderConfig(
            max_depth=3,
            max_pages=100,
            respect_robots_txt=True,
            follow_links=True
        )

        # Start crawl with automatic polling
        async def on_progress(status):
            print(f"Crawled: {status.pages_crawled}/{status.total_pages}")

        result = await client.spider.crawl_with_status_polling(
            seed_urls=["https://example.com"],
            config=config,
            callback=on_progress
        )

        print(f"Completed: {result.pages_crawled} pages")

asyncio.run(main())
```

### Example 8: WebSocket Streaming (NEW)

```python
# OLD (riptide-client)
# Not available

# NEW (riptide-sdk)
import asyncio
from riptide_sdk import RipTideClient

async def main():
    async with RipTideClient("http://localhost:8080") as client:
        # Real-time WebSocket streaming
        async for result in client.streaming.crawl_websocket(urls):
            if result.event_type == "result":
                print(f"URL: {result.data['result']['url']}")
                print(f"Status: {result.data['result']['status']}")
            elif result.event_type == "progress":
                progress = result.data.get('progress', {})
                print(f"Progress: {progress.get('completed')}/{progress.get('total')}")

asyncio.run(main())
```

### Example 9: Browser Automation (NEW)

```python
# OLD (riptide-client)
# Not available

# NEW (riptide-sdk)
import asyncio
from riptide_sdk import RipTideClient, BrowserSessionConfig

async def main():
    async with RipTideClient("http://localhost:8080") as client:
        # Create browser session with stealth
        config = BrowserSessionConfig(
            stealth_preset="medium",
            initial_url="https://example.com",
            timeout_secs=600
        )
        session = await client.browser.create_session(config)

        # Navigate and interact
        await client.browser.navigate(session.session_id, "https://github.com")
        await client.browser.type_text(session.session_id, "#search", "python")
        await client.browser.click(session.session_id, "button[type='submit']")

        # Wait for results
        await client.browser.wait_for_element(session.session_id, ".search-results")

        # Take screenshot
        screenshot = await client.browser.screenshot(session.session_id, full_page=True)
        with open("screenshot.png", "wb") as f:
            f.write(screenshot.data)

        # Cleanup
        await client.browser.close_session(session.session_id)

asyncio.run(main())
```

---

## Migration Checklist

### Phase 1: Preparation

- [ ] Review current riptide-client usage in your codebase
- [ ] Identify which endpoints you're using
- [ ] Check if you need any new features (sessions, extraction, spider, etc.)
- [ ] Plan async/await refactoring if your code is synchronous

### Phase 2: Installation

- [ ] Install riptide-sdk: `pip install riptide-sdk`
- [ ] Keep riptide-client installed temporarily for gradual migration
- [ ] Verify Python version is 3.8 or higher
- [ ] Install websockets if using WebSocket streaming: `pip install websockets>=12.0`

### Phase 3: Code Changes

- [ ] Update imports from `riptide_client` to `riptide_sdk`
- [ ] Convert client initialization to use context manager
- [ ] Add `async`/`await` to all crawl operations
- [ ] Update method calls:
  - [ ] `client.crawl()` → `client.crawl.batch()`
  - [ ] `client.crawl_single()` → `client.crawl.single()`
  - [ ] `client.stream_crawl()` → `client.streaming.crawl_ndjson()`
- [ ] Update options from dict to typed objects (CrawlOptions, etc.)
- [ ] Update response access from dict to typed objects
- [ ] Add proper exception handling with typed exceptions

### Phase 4: Testing

- [ ] Test basic crawl operations
- [ ] Test streaming if used
- [ ] Test error handling scenarios
- [ ] Verify performance is acceptable
- [ ] Test with your production workload

### Phase 5: Cleanup

- [ ] Remove riptide-client dependency
- [ ] Update documentation
- [ ] Update CI/CD pipelines
- [ ] Train team on new SDK

### Optional: Leverage New Features

- [ ] Add session management for authenticated crawling
- [ ] Use extraction API for content-only operations
- [ ] Implement spider crawling for deep site exploration
- [ ] Add browser automation for JavaScript-heavy sites
- [ ] Use worker queue for long-running operations
- [ ] Add domain profiles for advanced caching

---

## Troubleshooting

### Problem: "Cannot use sync client in async context"

```python
# WRONG
def my_function():
    client = RipTideClient()
    result = await client.crawl.batch(urls)  # Error!

# CORRECT
async def my_function():
    async with RipTideClient() as client:
        result = await client.crawl.batch(urls)
```

### Problem: "Event loop is already running"

```python
# WRONG (in Jupyter notebook or existing async context)
asyncio.run(main())

# CORRECT (in Jupyter)
await main()

# CORRECT (in existing event loop)
loop = asyncio.get_event_loop()
result = loop.run_until_complete(main())
```

### Problem: "Missing type hints"

```python
# Install typing-extensions for Python < 3.10
pip install typing-extensions>=4.0.0
```

### Problem: "WebSocket connection failed"

```python
# Install websockets package
pip install websockets>=12.0

# Or use NDJSON/SSE streaming instead
async for result in client.streaming.crawl_ndjson(urls):
    pass
```

### Problem: "Timeout errors"

```python
# Increase timeout
client = RipTideClient(
    base_url="http://localhost:8080",
    timeout=60.0  # Increase from default 30s
)
```

### Problem: "Too many concurrent connections"

```python
# Limit concurrency in options
options = CrawlOptions(concurrency=5)  # Reduce from default

# Or limit max connections in client
client = RipTideClient(
    base_url="http://localhost:8080",
    max_connections=50  # Reduce from default 100
)
```

### Problem: "API key not working"

```python
# Ensure proper API key format
client = RipTideClient(
    base_url="http://localhost:8080",
    api_key="your-api-key"  # Will be sent as "Bearer your-api-key"
)

# Check headers are being sent
print(client._client.headers)
```

### Problem: "Migration path for large codebases"

For large codebases, consider a wrapper approach during migration:

```python
# Create a compatibility wrapper
class RipTideClientCompat:
    def __init__(self, base_url, api_key=None):
        self.base_url = base_url
        self.api_key = api_key
        self._client = None

    async def __aenter__(self):
        from riptide_sdk import RipTideClient
        self._client = RipTideClient(self.base_url, api_key=self.api_key)
        await self._client.__aenter__()
        return self

    async def __aexit__(self, *args):
        await self._client.__aexit__(*args)

    async def crawl(self, urls, **kwargs):
        """Compatibility method that mimics old API"""
        from riptide_sdk import CrawlOptions, CacheMode

        # Map old parameters to new options
        cache_mode = kwargs.pop("cache_mode", "read_write")
        cache_mode_enum = getattr(CacheMode, cache_mode.upper().replace("-", "_"))

        options = CrawlOptions(
            cache_mode=cache_mode_enum,
            concurrency=kwargs.pop("concurrency", 10),
            **kwargs
        )

        result = await self._client.crawl.batch(urls, options=options)

        # Optionally convert to dict for compatibility
        return result.to_dict() if hasattr(result, 'to_dict') else result

# Use in migration
async with RipTideClientCompat("http://localhost:8080") as client:
    result = await client.crawl(urls)  # Old-style call
```

---

## Additional Resources

- **SDK Documentation**: `/workspaces/eventmesh/sdk/python/README.md`
- **Quick Start Guide**: `/workspaces/eventmesh/sdk/python/QUICK_START.md`
- **API Coverage Report**: `/workspaces/eventmesh/sdk/python/FINAL_COVERAGE_REPORT.md`
- **Code Examples**: `/workspaces/eventmesh/sdk/python/examples/`
- **Browser API Guide**: `/workspaces/eventmesh/sdk/python/docs/BROWSER_API.md`
- **WebSocket Streaming Guide**: `/workspaces/eventmesh/sdk/python/docs/WEBSOCKET_STREAMING.md`

---

## Support

If you encounter issues during migration:

1. Check the examples directory for similar use cases
2. Review the comprehensive README.md for API details
3. Open an issue on GitHub with migration-specific questions
4. Include code snippets showing old (riptide-client) and attempted new (riptide-sdk) code

---

## Summary

**Key Takeaways:**

1. All operations are now async/await - this is the biggest change
2. Use context managers (`async with`) for proper resource cleanup
3. Methods are organized into namespaces (crawl, streaming, extract, etc.)
4. Typed objects replace dictionaries for better IDE support
5. Comprehensive exception handling with specific error types
6. Many new features available (sessions, extraction, spider, browser, etc.)

**Migration Time Estimates:**

- **Small project** (< 10 crawl calls): 30 minutes
- **Medium project** (10-50 crawl calls): 2-4 hours
- **Large project** (50+ crawl calls): 1-2 days
- **Enterprise project** (complex workflows): 3-5 days

The migration effort is worth it for the improved type safety, performance, error handling, and access to 84% API coverage vs 30% in the legacy client.
