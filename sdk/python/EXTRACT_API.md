# Extract API - Python SDK Implementation

## Overview

The Extract API provides single-URL content extraction with multi-strategy support (CSS, WASM, hybrid fallback) and comprehensive metadata extraction.

## Installation

The Extract API is included in the RipTide Python SDK:

```python
from riptide_sdk import RipTideClient
from riptide_sdk.models import ExtractOptions, ExtractionResult
```

## API Endpoint

**POST** `/api/v1/extract`

## Quick Start

### Basic Extraction

```python
import asyncio
from riptide_sdk import RipTideClient

async def main():
    async with RipTideClient(base_url="http://localhost:8080") as client:
        result = await client.extract.extract("https://example.com")

        print(f"Title: {result.title}")
        print(f"Quality: {result.quality_score:.2f}")
        print(f"Strategy: {result.strategy_used}")
        print(f"Content: {result.content[:200]}...")

asyncio.run(main())
```

### Extraction with Custom Options

```python
from riptide_sdk.models import ExtractOptions

options = ExtractOptions(
    strategy="wasm",           # Force WASM parser
    quality_threshold=0.8,     # Higher quality threshold
    timeout_ms=15000           # 15 second timeout
)

result = await client.extract.extract(
    "https://example.com/article",
    mode="article",
    options=options
)

# Get formatted summary
print(result.to_summary())
```

## Extraction Modes

### Standard Mode (default)
General purpose extraction for any webpage.

```python
result = await client.extract.extract("https://example.com")
```

### Article Mode
Optimized for news articles and blog posts. Extracts author, publish date, and article-specific metadata.

```python
result = await client.extract.extract_article("https://example.com/blog/post")

print(f"Author: {result.metadata.author}")
print(f"Published: {result.metadata.publish_date}")
```

### Markdown Mode
Extracts content formatted as Markdown.

```python
result = await client.extract.extract_markdown("https://example.com/docs")

# Content is returned in Markdown format
print(result.content)
```

### Product Mode
Optimized for e-commerce product pages.

```python
result = await client.extract.extract_product("https://shop.example.com/product/123")
```

## Extraction Strategies

The `strategy` option controls which parser to use:

- **`"multi"`** (default): Tries WASM → CSS → Fallback chain for best quality
- **`"wasm"`**: Uses WASM parser only
- **`"css"`**: Uses CSS selector-based extraction
- **`"auto"`**: Same as "multi"

```python
options = ExtractOptions(strategy="wasm")
result = await client.extract.extract(url, options=options)
```

## Models

### ExtractOptions

Configuration for extraction requests.

```python
@dataclass
class ExtractOptions:
    strategy: str = "multi"              # Extraction strategy
    quality_threshold: float = 0.7       # Quality threshold (0.0-1.0)
    timeout_ms: int = 30000              # Timeout in milliseconds
```

### ExtractionResult

Result of content extraction.

```python
@dataclass
class ExtractionResult:
    url: str                             # Source URL
    title: Optional[str]                 # Page title
    content: str                         # Extracted content
    metadata: ContentMetadata            # Content metadata
    strategy_used: str                   # Strategy that succeeded
    quality_score: float                 # Quality score (0.0-1.0)
    extraction_time_ms: int              # Extraction time
    parser_metadata: Optional[ParserMetadata]  # Parser observability

    def to_summary(self) -> str:
        """Generate human-readable summary"""
```

### ContentMetadata

Metadata about extracted content.

```python
@dataclass
class ContentMetadata:
    author: Optional[str]                # Article author
    publish_date: Optional[str]          # Publication date
    word_count: int                      # Word count
    language: Optional[str]              # Content language
```

### ParserMetadata

Parser observability information (optional).

```python
@dataclass
class ParserMetadata:
    parser_used: str                     # Parser that succeeded
    confidence_score: float              # Parser confidence
    fallback_occurred: bool              # Whether fallback was used
    parse_time_ms: int                   # Parse time
    extraction_path: Optional[str]       # Extraction path taken
    primary_error: Optional[str]         # Primary error if any
```

## Advanced Usage

### Batch Extraction (Parallel)

```python
urls = [
    "https://example.com/page1",
    "https://example.com/page2",
    "https://example.com/page3",
]

# Execute in parallel using asyncio.gather()
tasks = [client.extract.extract(url) for url in urls]
results = await asyncio.gather(*tasks, return_exceptions=True)

for result in results:
    if isinstance(result, Exception):
        print(f"Failed: {result}")
    else:
        print(f"Success: {result.title} ({result.quality_score:.2f})")
```

### Error Handling

```python
from riptide_sdk.exceptions import APIError, ValidationError

try:
    result = await client.extract.extract("https://example.com")
except ValidationError as e:
    print(f"Invalid URL: {e}")
except APIError as e:
    print(f"API Error ({e.status_code}): {e.message}")
```

### Parser Observability

```python
result = await client.extract.extract("https://example.com")

if result.parser_metadata:
    print(f"Parser: {result.parser_metadata.parser_used}")
    print(f"Confidence: {result.parser_metadata.confidence_score:.2f}")
    print(f"Fallback: {result.parser_metadata.fallback_occurred}")
    print(f"Parse Time: {result.parser_metadata.parse_time_ms}ms")
```

## API Reference

### ExtractAPI Class

#### `extract(url, mode="standard", options=None) -> ExtractionResult`

Extract content from a URL using multi-strategy extraction.

**Parameters:**
- `url` (str): URL to extract content from
- `mode` (str): Extraction mode - "standard", "article", "product", or "markdown"
- `options` (ExtractOptions, optional): Extraction configuration

**Returns:** `ExtractionResult`

**Raises:**
- `ValidationError`: If URL is invalid
- `APIError`: If the API returns an error

#### `extract_article(url, options=None) -> ExtractionResult`

Extract content in article mode (convenience method).

#### `extract_markdown(url, options=None) -> ExtractionResult`

Extract content as Markdown (convenience method).

#### `extract_product(url, options=None) -> ExtractionResult`

Extract content in product mode (convenience method).

## Examples

See `/workspaces/eventmesh/sdk/python/examples/extract_example.py` for comprehensive examples including:

- Basic extraction
- Extraction with custom options
- Article mode extraction
- Markdown extraction
- Product mode extraction
- Batch parallel extraction
- Parser observability

## Implementation Details

**Files:**
- `/workspaces/eventmesh/sdk/python/riptide_sdk/endpoints/extract.py` - API implementation
- `/workspaces/eventmesh/sdk/python/riptide_sdk/models.py` - Data models

**Rust API Handler:**
- `/workspaces/eventmesh/crates/riptide-api/src/handlers/extract.rs`

**Features:**
- ✅ Async/await support
- ✅ Type hints throughout
- ✅ Comprehensive error handling
- ✅ Rich metadata extraction
- ✅ Parser observability
- ✅ Multiple extraction strategies
- ✅ Quality scoring
- ✅ Production-ready

## Testing

```bash
cd /workspaces/eventmesh/sdk/python
python3 examples/extract_example.py
```

## Support

For issues or questions:
- Check the main SDK documentation
- Review example code
- Examine the Rust API handler for contract details
