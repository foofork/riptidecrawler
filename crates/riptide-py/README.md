# Riptide Python SDK

High-performance web scraping for Python, powered by Rust.

[![PyPI version](https://badge.fury.io/py/riptidecrawler.svg)](https://badge.fury.io/py/riptidecrawler)
[![Python 3.8+](https://img.shields.io/badge/python-3.8+-blue.svg)](https://www.python.org/downloads/)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE-APACHE)

## Status

**Phase 2, Week 9-11:** Core Bindings ✅ Complete
- ✅ RipTide Python class
- ✅ Document class
- ✅ Error handling
- ✅ Type hints (.pyi)
- ✅ Pytest test suite (60+ tests)
- ✅ Performance benchmarks
- ✅ Python examples

## Features

- **🚀 Fast**: Rust-powered extraction with async runtime
- **🎯 Simple API**: Extract, spider, and crawl with 3 methods
- **📊 Rich Data**: Structured document objects with metadata
- **🔧 Type Safe**: Full type hints for IDEs and type checkers
- **⚡ Parallel**: Batch processing with concurrent execution
- **🐍 Pythonic**: Clean, intuitive Python API

## Installation

### From Source (Development)

```bash
# Install maturin
pip install maturin

# Build and install in development mode
cd crates/riptide-py
maturin develop

# Or build release wheel
maturin build --release
```

### From PyPI (Coming Soon)

```bash
pip install riptidecrawler
```

## Quick Start

```python
import riptide

# Create RipTide instance
rt = riptide.RipTide()

# Extract content from a URL
doc = rt.extract("https://example.com")
print(doc.title)
print(doc.text)
print(f"Quality: {doc.quality_score:.2f}")

# Spider to discover URLs
urls = rt.spider("https://example.com", max_depth=2)
print(f"Found {len(urls)} URLs")

# Batch crawl multiple URLs
docs = rt.crawl(urls[:10])
for doc in docs:
    print(f"{doc.url}: {doc.title}")
```

## API Reference

### RipTide Class

Main class for web scraping operations.

#### `__init__(api_key=None)`

Create a new RipTide instance.

**Parameters:**
- `api_key` (str, optional): API key for future cloud features

**Example:**
```python
rt = riptide.RipTide()
# or with API key
rt = riptide.RipTide(api_key="your-key")
```

#### `extract(url, mode="standard")`

Extract content from a single URL.

**Parameters:**
- `url` (str): URL to extract content from
- `mode` (str): Extraction mode - "standard" or "enhanced"

**Returns:**
- `Document`: Extracted content

**Raises:**
- `ValueError`: If URL is empty or mode is invalid
- `RuntimeError`: If extraction fails
- `TimeoutError`: If request times out

**Example:**
```python
# Standard mode (fast)
doc = rt.extract("https://example.com")

# Enhanced mode (multiple strategies)
doc = rt.extract("https://example.com", mode="enhanced")
```

#### `spider(url, max_depth=2, max_urls=100)`

Discover URLs by crawling a website.

**Parameters:**
- `url` (str): Starting URL
- `max_depth` (int): Maximum crawl depth (default: 2)
- `max_urls` (int): Maximum URLs to discover (default: 100)

**Returns:**
- `List[str]`: List of discovered URLs

**Raises:**
- `ValueError`: If URL is empty
- `RuntimeError`: If spider operation fails

**Example:**
```python
urls = rt.spider("https://example.com", max_depth=3, max_urls=200)
```

#### `crawl(urls, mode="standard")`

Batch process multiple URLs in parallel.

**Parameters:**
- `urls` (List[str]): List of URLs to crawl
- `mode` (str): Extraction mode - "standard" or "enhanced"

**Returns:**
- `List[Document]`: List of extracted documents

**Raises:**
- `ValueError`: If URLs list is empty or mode is invalid
- `RuntimeError`: If batch crawl fails

**Example:**
```python
urls = ["https://example.com", "https://example.org"]
docs = rt.crawl(urls)
```

#### `version()` (static method)

Get the version of the Riptide library.

**Returns:**
- `str`: Version string

**Example:**
```python
print(riptide.RipTide.version())
```

#### `is_healthy()`

Check if the instance is healthy.

**Returns:**
- `bool`: True if healthy

**Example:**
```python
if rt.is_healthy():
    print("Ready to scrape!")
```

### Document Class

Represents extracted web content.

#### Attributes

- `url` (str): Source URL
- `title` (str): Page title
- `text` (str): Extracted text content
- `html` (str | None): Raw HTML (if available)
- `quality_score` (float): Content quality (0.0-1.0)
- `word_count` (int): Number of words
- `from_cache` (bool): Whether cached
- `processing_time_ms` (int): Processing time in milliseconds

#### Methods

##### `to_dict()`

Convert document to dictionary.

**Returns:**
- `Dict[str, Any]`: Document as dictionary

**Example:**
```python
doc = rt.extract("https://example.com")
doc_dict = doc.to_dict()
print(doc_dict['title'])
```

##### `__len__()`

Get length of text content.

**Example:**
```python
doc = rt.extract("https://example.com")
print(f"Text length: {len(doc)} characters")
```

## Usage Examples

### Basic Extraction

```python
import riptide

rt = riptide.RipTide()

# Extract content
doc = rt.extract("https://example.com")

# Access properties
print(f"Title: {doc.title}")
print(f"Text: {doc.text[:200]}...")  # First 200 chars
print(f"Quality: {doc.quality_score:.2f}")
print(f"Words: {doc.word_count}")
print(f"Cached: {doc.from_cache}")
print(f"Time: {doc.processing_time_ms}ms")
```

### URL Discovery

```python
rt = riptide.RipTide()

# Spider with depth 3, max 50 URLs
urls = rt.spider("https://example.com", max_depth=3, max_urls=50)

print(f"Discovered {len(urls)} URLs:")
for url in urls:
    print(f"  - {url}")
```

### Batch Processing

```python
rt = riptide.RipTide()

# List of URLs to process
urls = [
    "https://example.com",
    "https://example.org",
    "https://example.net",
]

# Crawl all URLs
docs = rt.crawl(urls)

# Process results
for doc in docs:
    print(f"{doc.url}")
    print(f"  Title: {doc.title}")
    print(f"  Words: {doc.word_count}")
    print(f"  Quality: {doc.quality_score:.2f}")
    print()
```

### Error Handling

```python
import riptide

rt = riptide.RipTide()

try:
    doc = rt.extract("https://invalid-url")
except ValueError as e:
    print(f"Invalid input: {e}")
except RuntimeError as e:
    print(f"Runtime error: {e}")
except TimeoutError as e:
    print(f"Request timed out: {e}")
```

### Working with Document Objects

```python
rt = riptide.RipTide()
doc = rt.extract("https://example.com")

# String representations
print(repr(doc))  # Document(url='...', title='...', ...)
print(str(doc))   # https://example.com: Example Domain

# Length
print(len(doc))   # Length of text content

# Convert to dictionary
doc_dict = doc.to_dict()

# Access all fields
for key, value in doc_dict.items():
    print(f"{key}: {value}")
```

## Testing

### Run Tests

```bash
# Install pytest
pip install pytest

# Build and run tests
maturin develop && pytest tests/ -v

# Run specific test file
pytest tests/test_riptide.py -v

# Run performance benchmarks
pytest tests/test_performance.py -v -s
```

### Test Coverage

The test suite includes:
- **60+ tests** covering all functionality
- **Unit tests** for each method
- **Integration tests** for workflows
- **Error handling tests**
- **Performance benchmarks**
- **Spike compatibility tests**

## Performance

### Benchmarks

| Operation | Time | Notes |
|-----------|------|-------|
| Instance creation | <10ms | Very fast |
| Extract (cached) | <1ms | Minimal overhead |
| Extract (network) | ~100-500ms | Network dependent |
| Spider (10 URLs) | ~50-200ms | Parallel discovery |
| Batch crawl (10 URLs) | ~1-2s | Parallel processing |
| Document.to_dict() | <0.01ms | Negligible |

*Benchmarks run on standard hardware with good network connection.*

### Memory Usage

- **Efficient**: Rust memory management
- **Scalable**: Handles large batches
- **Safe**: No memory leaks

## Architecture

```
┌─────────────────────────────────┐
│     Python Application          │
│  (import riptide)               │
└────────────┬────────────────────┘
             │
┌────────────▼────────────────────┐
│     PyO3 Bindings               │
│  - RipTide class                │
│  - Document class               │
│  - Type conversions             │
└────────────┬────────────────────┘
             │
┌────────────▼────────────────────┐
│     Tokio Runtime               │
│  - Async execution              │
│  - Parallel processing          │
└────────────┬────────────────────┘
             │
┌────────────▼────────────────────┐
│     CrawlFacade                 │
│  (riptide-facade)               │
└────────────┬────────────────────┘
             │
┌────────────▼────────────────────┐
│  Production Pipeline Code       │
│  (1,640 lines)                  │
└─────────────────────────────────┘
```

## Development

### Building from Source

```bash
# Clone repository
git clone https://github.com/yourusername/riptidecrawler.git
cd riptidecrawler/crates/riptide-py

# Install development dependencies
pip install maturin pytest

# Build and install
maturin develop

# Run tests
pytest tests/ -v
```

### Project Structure

```
crates/riptide-py/
├── src/
│   ├── lib.rs              # Module exports
│   ├── riptide_class.rs    # RipTide Python class
│   ├── document.rs         # Document Python class
│   └── errors.rs           # Error handling
├── examples/
│   ├── spike_test.py       # Spike validation tests
│   └── basic_usage.py      # Usage examples
├── tests/
│   ├── test_riptide.py     # Main test suite
│   └── test_performance.py # Performance benchmarks
├── riptide.pyi             # Type hints
├── pytest.ini              # Pytest configuration
├── pyproject.toml          # Python project config
└── README.md               # This file
```

## Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Write tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../../LICENSE-MIT))

at your option.

## Links

- **Repository**: https://github.com/yourusername/riptidecrawler
- **Documentation**: https://docs.riptidecrawler.io
- **PyPI**: https://pypi.org/project/riptidecrawler/ (coming soon)
- **Issues**: https://github.com/yourusername/riptidecrawler/issues

## Acknowledgments

Built with:
- [PyO3](https://pyo3.rs/) - Rust Python bindings
- [Tokio](https://tokio.rs/) - Async runtime
- [Rust](https://www.rust-lang.org/) - Systems programming language

---

**Status**: Phase 2 Week 9-11 - Core Bindings Complete ✅
