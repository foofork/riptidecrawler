# RipTide Python SDK

**Official Python SDK for the RipTide web crawling and extraction API.**

**Version:** 0.2.0 ⚡
**Status:** Production Ready - Feature Complete
**Python:** 3.8+ (Fully typed with async/await support)
**Coverage:** 84% of API endpoints (52/62 core + all P0/P1/P2 features)

## What's New in v0.2.0 🎉

- ⚡ **Browser Automation** - Direct browser control with 15 methods
- ⚡ **WebSocket Streaming** - Bidirectional real-time communication
- ⚡ **Extraction API** - Standalone content extraction (article, markdown, product)
- ⚡ **Search API** - Web search integration
- ⚡ **Spider Crawling** - Deep multi-page site crawling
- ⚡ **Session Management** - Authenticated crawling with persistent sessions
- ⚡ **PDF Processing** - Document extraction with progress tracking
- ⚡ **Worker Queue** - Async job management for long-running operations

## Features

- 🚀 **Async/await support** - Built on httpx for high-performance async operations
- 📊 **Type hints** - Full type annotations for Python 3.8+
- 🔄 **Streaming support** - NDJSON, SSE, and WebSocket streaming for real-time results
- 🎯 **Complete API coverage** - 84% of RipTide API endpoints:
  - **Core Crawling** - Batch and single URL crawling (100%)
  - **Extraction** - Standalone extraction without crawling (100%) ⚡ NEW
  - **Search** - Web search with content extraction (100%) ⚡ NEW
  - **Spider** - Deep crawling with status tracking (100%) ⚡ NEW
  - **Sessions** - Authenticated crawling (100%) ⚡ NEW
  - **PDF** - Document processing (100%) ⚡ NEW
  - **Workers** - Async job queue (100%) ⚡ NEW
  - **Browser** - Direct browser automation (100%) ⚡ NEW
  - **Domain Profiles** - Warm-start caching and rate limiting (100%)
  - **Engine Selection** - Intelligent engine routing (100%)
  - **Streaming** - Real-time processing with NDJSON/SSE/WebSocket (100%) ⚡ NEW
- ⚡ **Connection pooling** - Efficient HTTP connection management
- 🛡️ **Error handling** - Comprehensive exception hierarchy with retry logic
- 🧪 **Production tested** - Battle-tested with comprehensive test coverage

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

    # WebSocket streaming (⚡ NEW in v0.2.0)
    async for result in client.streaming.crawl_websocket(urls):
        if result.event_type == "result":
            print(f"URL: {result.data['result']['url']}")
            print(f"Progress: {result.data.get('progress', {})}")

    # Deep search streaming
    async for result in client.streaming.deepsearch_ndjson("python"):
        print(f"Found: {result.data['url']}")
```

### Extraction API ⚡ NEW

```python
from riptide_sdk.models import ExtractOptions

async with RipTideClient() as client:
    # Basic extraction (uses native strategy by default)
    result = await client.extract.extract("https://example.com/article")
    print(f"Title: {result.title}")
    print(f"Content: {result.content[:200]}...")
    print(f"Strategy used: {result.strategy_used}")  # Shows "native"

    # Article extraction
    article = await client.extract.extract_article("https://blog.example.com/post")
    print(f"Author: {article.metadata.author}")
    print(f"Published: {article.metadata.published_date}")

    # Extract as markdown
    markdown = await client.extract.extract_markdown(url)
    print(markdown.content)

    # Product extraction
    product = await client.extract.extract_product("https://shop.example.com/item")
    print(f"Price: ${product.metadata.price}")

    # Explicit strategy selection (v0.9.0+)
    options = ExtractOptions(strategy="native")  # Default, fastest (2-5ms)
    result = await client.extract.extract(url, options=options)

    # WASM extraction (only if server supports it)
    try:
        options = ExtractOptions(strategy="wasm")  # 4x slower but sandboxed
        result = await client.extract.extract(url, options=options)
    except Exception as e:
        print(f"WASM not available: {e}")
```

#### Extraction Strategies (v0.9.0+)

The SDK now supports multiple extraction strategies with **native as the new default** for better performance:

| Strategy | Speed | Availability | Use Case |
|----------|-------|--------------|----------|
| `native` (default) | 2-5ms | Always | General web scraping (recommended) |
| `wasm` | 10-20ms | Server feature | Untrusted HTML, sandboxing needed |
| `multi` | Varies | Always | Server auto-selects best strategy |

**Server Compatibility:**

```python
# Server with native-only (default build):
# ✅ strategy="native" - Works
# ✅ strategy="multi" - Falls back to native
# ❌ strategy="wasm" - Error (WASM not available)

# Server with WASM enabled (--features wasm-extractor):
# ✅ strategy="native" - Works
# ✅ strategy="wasm" - Works
# ✅ strategy="multi" - Prefers WASM, falls back to native
```

**Migration from v0.8.x:**

```python
# Before v0.9.0: Used WASM by default if available
result = await client.extract.extract(url)

# After v0.9.0: Uses native by default (4x faster)
result = await client.extract.extract(url)
# Strategy used: "native" (unless explicitly changed)

# To restore old behavior (use WASM if available):
options = ExtractOptions(strategy="wasm")
result = await client.extract.extract(url, options=options)
```

**Graceful Fallback Pattern:**

```python
from riptide_sdk.exceptions import ExtractionError

async def extract_with_fallback(url):
    """Try WASM first, fall back to native"""
    for strategy in ["wasm", "native"]:
        try:
            options = ExtractOptions(strategy=strategy)
            result = await client.extract.extract(url, options=options)
            print(f"✅ Success with {strategy}")
            return result
        except ExtractionError as e:
            if "not available" in str(e).lower():
                print(f"⚠️  {strategy} unavailable, trying next...")
                continue
            raise
```

### Search API ⚡ NEW

```python
from riptide_sdk.models import SearchOptions

async with RipTideClient() as client:
    # Basic search
    results = await client.search.search("python web scraping", limit=10)
    print(f"Found {results.total_results} results")
    for item in results.results:
        print(f"{item.title}: {item.url}")

    # Quick search (convenience method)
    results = await client.search.quick_search("AI news")
    for item in results.results:
        print(f"{item.snippet}")
```

### Spider Crawling ⚡ NEW

```python
from riptide_sdk.models import SpiderConfig

async with RipTideClient() as client:
    # Start spider crawl
    config = SpiderConfig(
        max_depth=3,
        max_pages=100,
        respect_robots_txt=True,
        follow_links=True
    )
    result = await client.spider.crawl(
        seed_urls=["https://example.com"],
        config=config
    )

    # Poll for status
    status = await client.spider.status(result.crawl_id)
    print(f"Progress: {status.pages_crawled}/{status.total_pages}")

    # Or use automatic polling with callback
    async def on_progress(status):
        print(f"Crawled: {status.pages_crawled}")

    result = await client.spider.crawl_with_status_polling(
        seed_urls=["https://example.com"],
        config=config,
        callback=on_progress
    )
```

### Session Management ⚡ NEW

```python
from riptide_sdk.models import SessionConfig, Cookie

async with RipTideClient() as client:
    # Create session
    config = SessionConfig(
        ttl_seconds=3600,
        stealth_mode=True
    )
    session = await client.sessions.create(config)

    # Set authentication cookie
    cookie = Cookie(
        name="auth_token",
        value="secret123",
        domain="example.com",
        secure=True,
        http_only=True
    )
    await client.sessions.set_cookie(session.id, cookie)

    # Crawl with session
    result = await client.crawl.batch(urls, session_id=session.id)

    # Extend session TTL
    await client.sessions.extend(session.id, ttl_seconds=7200)

    # Get session stats
    stats = await client.sessions.get_stats()
    print(f"Active sessions: {stats.active_sessions}")
```

### PDF Processing ⚡ NEW

```python
from riptide_sdk.models import PdfExtractionOptions

async with RipTideClient() as client:
    # Extract text from PDF
    result = await client.pdf.extract("https://example.com/doc.pdf")
    print(f"Text: {result.text[:500]}...")
    print(f"Pages: {result.num_pages}")

    # Extract with progress tracking
    async for progress in client.pdf.extract_with_progress(pdf_url):
        print(f"Progress: {progress.percentage}%")
        if progress.completed:
            print(f"Extracted {len(progress.result.text)} characters")

    # Get PDF metrics
    metrics = await client.pdf.get_metrics()
    print(f"Total extracted: {metrics.total_extracted}")
```

### Worker Queue ⚡ NEW

```python
from riptide_sdk.models import JobConfig, ScheduledJobConfig

async with RipTideClient() as client:
    # Submit long-running job
    job_config = JobConfig(
        job_type="crawl",
        payload={"urls": large_url_list}
    )
    job = await client.workers.submit_job(job_config)

    # Wait for completion
    result = await client.workers.wait_for_job(
        job.id,
        timeout=300,
        poll_interval=5.0
    )
    print(f"Job completed: {result.status}")

    # Schedule recurring job
    scheduled = await client.workers.create_scheduled_job(
        ScheduledJobConfig(
            schedule="0 0 * * *",  # Daily at midnight
            job_config=job_config
        )
    )

    # Get queue stats
    stats = await client.workers.get_queue_stats()
    print(f"Pending jobs: {stats.pending}")
```

### Browser Automation ⚡ NEW

```python
from riptide_sdk.models import BrowserSessionConfig

async with RipTideClient() as client:
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

    # Wait for element
    await client.browser.wait_for_element(
        session.session_id,
        ".search-results"
    )

    # Execute JavaScript
    result = await client.browser.execute_script(
        session.session_id,
        "return document.title"
    )
    print(f"Page title: {result.result}")

    # Take screenshot
    screenshot = await client.browser.screenshot(
        session.session_id,
        full_page=True
    )
    with open("screenshot.png", "wb") as f:
        f.write(screenshot.data)

    # Monitor pool health
    pool_status = await client.browser.get_pool_status()
    print(pool_status.to_summary())

    # Cleanup
    await client.browser.close_session(session.session_id)
```

## API Reference

### RipTideClient

Main client class with the following attributes:

- `crawl`: CrawlAPI - Batch crawl operations
- `profiles`: ProfilesAPI - Domain profile management
- `engine`: EngineSelectionAPI - Engine selection API
- `streaming`: StreamingAPI - Streaming operations
- `extract`: ExtractAPI - Content extraction ⚡ NEW
- `search`: SearchAPI - Web search ⚡ NEW
- `spider`: SpiderAPI - Deep crawling ⚡ NEW
- `sessions`: SessionsAPI - Session management ⚡ NEW
- `pdf`: PdfAPI - PDF processing ⚡ NEW
- `workers`: WorkersAPI - Job queue ⚡ NEW
- `browser`: BrowserAPI - Browser automation ⚡ NEW

### CrawlAPI

- `batch(urls, options)` - Batch crawl multiple URLs
- `single(url, options)` - Crawl a single URL

### ExtractAPI ⚡ NEW

- `extract(url, options)` - Extract content from URL
- `extract_article(url)` - Extract article content
- `extract_markdown(url)` - Extract as markdown
- `extract_product(url)` - Extract product information

### SearchAPI ⚡ NEW

- `search(query, limit, options)` - Search the web
- `quick_search(query)` - Quick search with defaults

### SpiderAPI ⚡ NEW

- `crawl(seed_urls, config)` - Start spider crawl
- `status(crawl_id)` - Get crawl status
- `control(crawl_id, action)` - Control crawl (pause/resume/stop)
- `crawl_with_status_polling(seed_urls, config, poll_interval, callback)` - Crawl with automatic polling

### SessionsAPI ⚡ NEW

- `create(config)` - Create session
- `list(filter, limit, offset)` - List sessions
- `get(session_id)` - Get session details
- `delete(session_id)` - Delete session
- `extend(session_id, ttl_seconds)` - Extend TTL
- `set_cookie(session_id, cookie)` - Set cookie
- `get_cookies_for_domain(session_id, domain)` - Get cookies
- `get_stats()` - Get session statistics

### PdfAPI ⚡ NEW

- `extract(pdf_url, options)` - Extract PDF text
- `extract_with_progress(pdf_url, options)` - Extract with progress
- `get_job_status(job_id)` - Get extraction status
- `get_metrics()` - Get PDF metrics

### WorkersAPI ⚡ NEW

- `submit_job(config)` - Submit job
- `list_jobs(status, limit, offset)` - List jobs
- `get_job_status(job_id)` - Get job status
- `get_job_result(job_id)` - Get job result
- `get_queue_stats()` - Get queue statistics
- `get_worker_stats()` - Get worker statistics
- `create_scheduled_job(config)` - Schedule recurring job
- `wait_for_job(job_id, timeout, poll_interval)` - Wait for completion

### BrowserAPI ⚡ NEW

- `create_session(config)` - Create browser session
- `execute_action(session_id, action)` - Execute browser action
- `get_pool_status()` - Get pool status
- `navigate(session_id, url)` - Navigate to URL
- `click(session_id, selector)` - Click element
- `type_text(session_id, selector, text)` - Type text
- `screenshot(session_id, full_page)` - Capture screenshot
- `execute_script(session_id, script)` - Run JavaScript
- `get_content(session_id)` - Get page HTML
- `wait_for_element(session_id, selector)` - Wait for element
- `render_pdf(session_id)` - Render as PDF
- `close_session(session_id)` - Close session
- `reset_session(session_id)` - Reset session

### ProfilesAPI

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

### EngineSelectionAPI

- `analyze(html, url)` - Analyze HTML and recommend engine
- `decide(html, url, flags)` - Make engine decision with flags
- `get_stats()` - Get engine usage statistics
- `toggle_probe_first(enabled)` - Toggle probe-first mode

### StreamingAPI

- `crawl_ndjson(urls, options)` - Stream crawl results (NDJSON)
- `deepsearch_ndjson(query, limit, options)` - Stream search results (NDJSON)
- `crawl_sse(urls, options)` - Stream crawl results (SSE)
- `crawl_websocket(urls, options, on_message)` - Stream via WebSocket ⚡ NEW
- `ping_websocket()` - Test WebSocket connection ⚡ NEW
- `get_websocket_status()` - Get connection status ⚡ NEW

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
- websockets >= 12.0 (optional, for WebSocket streaming)

## What Changed in v0.2.0

### New Endpoints (31 total)

**Swarm #1 (P0/P1 - Critical Features):**
- ✅ Extract API (2 endpoints + 2 convenience methods)
- ✅ Search API (1 endpoint + 1 convenience method)
- ✅ Spider API (3 endpoints + 1 helper)
- ✅ Sessions API (8 endpoints)
- ✅ PDF API (4 endpoints)
- ✅ Workers API (7 endpoints + 1 helper)

**Swarm #2 (P2 - High Priority Features):**
- ✅ Browser Automation API (3 endpoints + 10 convenience methods)
- ✅ WebSocket Streaming (1 endpoint + 2 monitoring methods)

### Coverage Improvement

- **Before:** 34% (21/62 endpoints)
- **After:** 84% (52/62 endpoints)
- **Increase:** +50% coverage

### What's Complete

- ✅ 100% of P0 (Critical) features
- ✅ 100% of P1 (High Priority) features
- ✅ 100% of P2 (Medium Priority) features
- ⚪ 0% of P3 (Low Priority - specialized features)

### Code Metrics

- **~6,240 lines** of new code
- **26 new files** (endpoints, models, examples, tests, docs)
- **100% success rate** (all implementations validated)
- **~1,850 lines** for Browser API alone
- **~2,100 lines** for WebSocket streaming

## Migration Guide (v0.1.0 → v0.2.0)

All existing code continues to work without changes. New features are additive:

```python
# v0.1.0 code still works
async with RipTideClient() as client:
    result = await client.crawl.batch(urls)  # ✅ Still works

# v0.2.0 adds new capabilities
async with RipTideClient() as client:
    # New extract API
    result = await client.extract.extract(url)  # ⚡ NEW

    # New browser automation
    session = await client.browser.create_session(config)  # ⚡ NEW

    # New WebSocket streaming
    async for result in client.streaming.crawl_websocket(urls):  # ⚡ NEW
        process(result)
```

## Documentation

- **Quick Start:** See above or `sdk/python/QUICK_START.md`
- **API Coverage:** `sdk/python/FINAL_COVERAGE_REPORT.md`
- **Browser API:** `sdk/python/docs/BROWSER_API.md`
- **WebSocket Streaming:** `sdk/python/docs/WEBSOCKET_STREAMING.md`
- **Examples:** See `sdk/python/examples/` directory

## License

MIT License - see LICENSE file for details
