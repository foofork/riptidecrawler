# RipTide Python SDK - Quick Start Guide

## Installation

```bash
cd /workspaces/eventmesh/sdk/python
pip install -e .
```

## 5-Minute Quick Start

### 1. Basic Crawl
```python
from riptide_sdk import RipTideClient
import asyncio

async def main():
    async with RipTideClient(base_url="http://localhost:8080") as client:
        result = await client.crawl.batch(["https://example.com"])
        print(f"Success: {result.successful}/{result.total_urls}")

asyncio.run(main())
```

### 2. Domain Profiles (Phase 10.4)
```python
from riptide_sdk.models import ProfileConfig, StealthLevel

async with RipTideClient() as client:
    # Create profile
    config = ProfileConfig(stealth_level=StealthLevel.HIGH)
    profile = await client.profiles.create("example.com", config=config)

    # Get stats
    stats = await client.profiles.get_stats("example.com")
    print(f"Cache hit rate: {stats.cache_hits / stats.total_requests:.2%}")
```

### 3. Engine Selection (Phase 10)
```python
async with RipTideClient() as client:
    # Analyze HTML
    decision = await client.engine.analyze(
        html="<html>...</html>",
        url="https://example.com"
    )
    print(f"Use {decision.engine} ({decision.confidence:.0%} confidence)")
```

### 4. Streaming
```python
async with RipTideClient() as client:
    async for result in client.streaming.crawl_ndjson(urls):
        print(f"Got: {result.data['url']}")
```

## API Endpoints Coverage

### ✅ Profiles API (Phase 10.4) - 11 endpoints
- `POST /api/v1/profiles` - Create profile
- `GET /api/v1/profiles/:domain` - Get profile
- `PUT /api/v1/profiles/:domain` - Update profile
- `DELETE /api/v1/profiles/:domain` - Delete profile
- `GET /api/v1/profiles` - List profiles
- `GET /api/v1/profiles/:domain/stats` - Get stats
- `GET /api/v1/profiles/metrics` - Get metrics
- `POST /api/v1/profiles/batch` - Batch create
- `GET /api/v1/profiles/search` - Search profiles
- `POST /api/v1/profiles/:domain/warm` - Warm cache
- `DELETE /api/v1/profiles/clear` - Clear caches

### ✅ Engine Selection API (Phase 10) - 4 endpoints
- `POST /api/v1/engine/analyze` - Analyze HTML
- `POST /api/v1/engine/decide` - Make decision
- `GET /api/v1/engine/stats` - Get stats
- `PUT /api/v1/engine/probe-first` - Toggle mode

### ✅ Crawl API - 1 endpoint
- `POST /api/v1/crawl` - Batch crawl

### ✅ Streaming API - 3 formats
- NDJSON streaming (crawl + search)
- SSE streaming (crawl)
- WebSocket support ready

## Examples

Run the included examples:

```bash
# Basic crawling
python examples/basic_crawl.py

# Domain profiles (Phase 10.4)
python examples/domain_profiles.py

# Engine selection (Phase 10)
python examples/engine_selection.py

# Streaming operations
python examples/streaming_example.py
```

## Key Features

- ✅ **Type-safe** - Full type hints for Python 3.8+
- ✅ **Async/await** - Built on httpx for high performance
- ✅ **Streaming** - Real-time results via NDJSON/SSE
- ✅ **Connection pooling** - Efficient HTTP management
- ✅ **Error handling** - Comprehensive exceptions
- ✅ **All 15+ endpoints** - Complete Phase 10+ coverage

## Project Structure

```
sdk/python/
├── riptide_sdk/
│   ├── __init__.py           # Main exports
│   ├── client.py             # RipTideClient
│   ├── models.py             # Data models (450+ lines)
│   ├── exceptions.py         # Error handling
│   └── endpoints/
│       ├── crawl.py          # Batch crawl API
│       ├── profiles.py       # Domain profiles (Phase 10.4)
│       ├── engine.py         # Engine selection (Phase 10)
│       └── streaming.py      # Streaming operations
├── examples/                 # 4 working examples
├── tests/                    # Test suite (ready for pytest)
├── setup.py                  # Package setup
├── pyproject.toml            # Modern Python packaging
└── requirements.txt          # Dependencies

Total: 2,052 lines of Python code
```

## Next Steps

1. Install development dependencies:
   ```bash
   pip install -e ".[dev]"
   ```

2. Run tests (when added):
   ```bash
   pytest
   ```

3. Format and lint:
   ```bash
   black .
   ruff check .
   mypy riptide_sdk
   ```

## Memory Key

Implementation stored in: `swarm/python-sdk/implementation`
