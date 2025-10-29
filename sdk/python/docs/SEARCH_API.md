# Search API Documentation

## Overview

The Search API endpoint provides search functionality using the RipTide search infrastructure with support for multiple providers (Serper, SearXNG, None).

## Installation

```bash
pip install riptide-sdk
```

## Quick Start

```python
import asyncio
from riptide_sdk import RipTideClient

async def main():
    async with RipTideClient(base_url="http://localhost:8080") as client:
        result = await client.search.search("rust web scraping")

        print(f"Found {result.total_results} results")
        for item in result.results:
            print(f"{item.position}. {item.title}")
            print(f"   {item.url}")

asyncio.run(main())
```

## API Reference

### SearchAPI

#### `search(query: str, limit: int = 10, options: Optional[SearchOptions] = None) -> SearchResponse`

Perform a search query using configured search providers.

**Parameters:**
- `query` (str): Search query string (required)
- `limit` (int): Number of results to return (1-50, default: 10)
- `options` (SearchOptions): Optional search configuration

**Returns:**
- `SearchResponse`: Search results with metadata

**Raises:**
- `ValidationError`: If query is empty or limit is invalid
- `APIError`: If the API returns an error

**Example:**
```python
result = await client.search.search("python tutorial", limit=20)
```

#### `quick_search(query: str, country: str = "us", language: str = "en") -> SearchResponse`

Convenience method for quick searches with common parameters.

**Parameters:**
- `query` (str): Search query string (required)
- `country` (str): Country code (default: "us")
- `language` (str): Language code (default: "en")

**Returns:**
- `SearchResponse`: Search results

**Example:**
```python
result = await client.search.quick_search("golang frameworks")
```

## Models

### SearchOptions

Configuration options for search operations.

**Attributes:**
- `country` (str): Country code (default: "us")
- `language` (str): Language code (default: "en")
- `provider` (Optional[str]): Force specific provider ("serper", "searxng", "none")

**Example:**
```python
from riptide_sdk import SearchOptions

options = SearchOptions(
    country="uk",
    language="en",
    provider="serper"
)
```

### SearchResponse

Search results with metadata.

**Attributes:**
- `query` (str): The search query
- `results` (List[SearchResultItem]): List of search results
- `total_results` (int): Total number of results
- `provider_used` (str): Name of the provider used
- `search_time_ms` (int): Search time in milliseconds

### SearchResultItem

Individual search result.

**Attributes:**
- `title` (str): Result title
- `url` (str): Result URL
- `snippet` (str): Result snippet/description
- `position` (int): Position in results (1-based)

## Usage Examples

### Basic Search

```python
async with RipTideClient(base_url="http://localhost:8080") as client:
    result = await client.search.search("rust web scraping")

    for item in result.results:
        print(f"{item.title}: {item.url}")
```

### Search with Options

```python
from riptide_sdk import SearchOptions

async with RipTideClient(base_url="http://localhost:8080") as client:
    options = SearchOptions(
        country="uk",
        language="en"
    )

    result = await client.search.search(
        query="python machine learning",
        limit=20,
        options=options
    )

    print(f"Provider: {result.provider_used}")
    print(f"Search time: {result.search_time_ms}ms")
```

### Force Specific Provider

```python
from riptide_sdk import SearchOptions

async with RipTideClient(base_url="http://localhost:8080") as client:
    options = SearchOptions(provider="serper")

    result = await client.search.search(
        query="golang frameworks",
        options=options
    )

    print(f"Using provider: {result.provider_used}")
```

### Extract URLs from Results

```python
async with RipTideClient(base_url="http://localhost:8080") as client:
    result = await client.search.quick_search("python tutorial")

    urls = [item.url for item in result.results]
    print(f"Found {len(urls)} URLs")
```

## Error Handling

```python
from riptide_sdk import ValidationError, APIError

async with RipTideClient(base_url="http://localhost:8080") as client:
    try:
        result = await client.search.search("test query")
    except ValidationError as e:
        print(f"Validation error: {e}")
    except APIError as e:
        print(f"API error: {e.message} (status: {e.status_code})")
```

## API Endpoint

**Endpoint:** `GET /api/v1/search`

**Query Parameters:**
- `q` (string, required): Search query
- `limit` (integer, optional): Number of results (1-50, default: 10)
- `country` (string, optional): Country code (default: "us")
- `language` (string, optional): Language code (default: "en")
- `provider` (string, optional): Force specific provider

**Response:**
```json
{
  "query": "rust web scraping",
  "results": [
    {
      "title": "Rust Web Scraping",
      "url": "https://example.com/rust",
      "snippet": "Learn web scraping with Rust",
      "position": 1
    }
  ],
  "total_results": 10,
  "provider_used": "Serper",
  "search_time_ms": 150
}
```

## Configuration

The Search API requires proper backend configuration:

### Environment Variables

- `RIPTIDE_SEARCH_BACKEND` or `SEARCH_BACKEND`: Provider type ("serper", "searxng", "none")
- `SERPER_API_KEY`: API key for Serper provider
- `SEARXNG_BASE_URL`: Base URL for SearXNG provider
- `RIPTIDE_SEARCH_TIMEOUT_SECS`: Request timeout (default: 30)

### Provider Support

1. **Serper** - Google search via Serper.dev API
   - Requires `SERPER_API_KEY` environment variable
   - Get API key from https://serper.dev

2. **SearXNG** - Self-hosted meta-search engine
   - Requires `SEARXNG_BASE_URL` environment variable
   - Privacy-focused alternative

3. **None** - URL parsing only
   - No external API required
   - Limited functionality

## See Also

- [Python SDK Documentation](../README.md)
- [Examples](../examples/search_example.py)
- [API Reference](../../docs/API.md)
