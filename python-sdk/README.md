# RipTide Python Client

[![PyPI version](https://badge.fury.io/py/riptide-client.svg)](https://badge.fury.io/py/riptide-client)
[![Python versions](https://img.shields.io/pypi/pyversions/riptide-client.svg)](https://pypi.org/project/riptide-client/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Downloads](https://pepy.tech/badge/riptide-client)](https://pepy.tech/project/riptide-client)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/your-org/riptide-api)

Official Python client for the RipTide web crawling and content extraction API. Enterprise-grade web scraping powered by WASM-accelerated extraction, adaptive routing, and intelligent content processing.

## ✨ Key Features

- 🚀 **Full API Coverage** - 59 endpoints across 13 categories
- ⚡ **WASM-Powered Extraction** - High-performance content extraction using WebAssembly
- 🔄 **Automatic Retries** - Built-in exponential backoff with configurable retry logic
- 🏊 **Connection Pooling** - Efficient HTTP connection management
- 📡 **Streaming Support** - Real-time results via NDJSON, SSE, and WebSocket
- 🎯 **Type Safety** - Full type hints and autocompletion support
- 🍪 **Session Management** - Persistent cookies and authentication state
- 🎭 **Stealth Crawling** - Bot detection evasion with fingerprint randomization
- 🔍 **Deep Search** - Web search integration with content extraction
- 🕷️ **Spider Engine** - Recursive deep crawling with frontier management
- 📄 **PDF Processing** - Extract structured content from PDF documents
- 📊 **Table Extraction** - Parse HTML tables into structured data
- 🎛️ **Multiple Strategies** - Auto, TREK, CSS, Regex, LLM-based extraction
- 🧩 **Intelligent Chunking** - Sliding, fixed, sentence, topic-based content chunking
- 🔐 **Context Manager** - Clean resource management with `with` statement
- ⚙️ **Highly Configurable** - Fine-tune timeouts, concurrency, caching, and more
- 📈 **Built-in Monitoring** - Health checks, metrics, and performance reporting

## 📦 Installation

```bash
pip install riptide-client
```

**Requirements:**
- Python 3.8+
- requests >= 2.31.0
- typing-extensions >= 4.5.0 (for Python < 3.10)

## 🚀 Quick Start

```python
from riptide_client import RipTide

# Initialize client
client = RipTide('http://localhost:8080')

# Crawl a URL
result = client.crawl(['https://example.com'])
print(result['results'][0]['document']['title'])

# Or use context manager for automatic cleanup
with RipTide('http://localhost:8080') as client:
    result = client.crawl(['https://example.com'])
    print(result)
```

## 📚 API Coverage

Full coverage of 59 RipTide API endpoints across 13 categories:

| Category | Endpoints | Key Capabilities |
|----------|-----------|------------------|
| **Core Crawling** | 5 | Batch crawling, adaptive gate routing, caching strategies |
| **Streaming** | 4 | NDJSON streams, Server-Sent Events, WebSocket connections |
| **Search** | 2 | Deep search with provider integration, automatic URL extraction |
| **Spider** | 3 | Deep recursive crawling, frontier management, budget controls |
| **Extraction Strategies** | 2 | Auto/TREK/CSS/Regex/LLM extraction, intelligent chunking |
| **PDF Processing** | 3 | PDF extraction, progress streaming, structured output |
| **Table Extraction** | 2 | HTML table parsing, CSV/Markdown export |
| **Stealth** | 4 | Bot evasion, fingerprint randomization, effectiveness testing |
| **Sessions** | 12 | Cookie persistence, authentication state, TTL management |
| **Workers & Jobs** | 9 | Async job processing, scheduling, retry logic, priority queues |
| **LLM Providers** | 4 | Provider management, runtime switching, gradual rollout |
| **Monitoring** | 6 | Health scores, alerts, performance metrics, bottleneck analysis |
| **Health & Metrics** | 2 | System health checks, Prometheus metrics |

## 📖 Usage Examples

### Basic Crawling

```python
from riptide_client import RipTide

client = RipTide('http://localhost:8080')

# Single URL
result = client.crawl(['https://example.com'])

# Multiple URLs with options
result = client.crawl(
    urls=['https://example.com', 'https://example.org'],
    options={
        'concurrency': 5,
        'cache_mode': 'read_write',
        'extract_mode': 'article'
    }
)

for item in result['results']:
    print(f"Title: {item['document']['title']}")
    print(f"URL: {item['url']}")
    print("---")
```

### Streaming

Stream results in real-time for large-scale crawling operations:

```python
# Stream crawl results (NDJSON)
for result in client.stream_crawl(['https://example.com']):
    print(f"Got: {result['url']}")
    print(f"Title: {result['document']['title']}")
    print(f"Content length: {len(result['document']['content'])}")
```

### Deep Search

Perform web searches and automatically extract content from discovered URLs:

```python
# Deep search with content extraction
results = client.search(
    query='python web scraping tutorials',
    options={
        'limit': 20,
        'include_content': True,
        'crawl_options': {
            'extract_mode': 'article',
            'cache_mode': 'auto'
        }
    }
)

for item in results['results']:
    print(f"{item['title']} - {item['url']}")
    if 'document' in item:
        print(f"Content preview: {item['document']['content'][:200]}...")
```

### Spider Deep Crawling

Recursively crawl websites with frontier-based URL management:

```python
# Start deep crawling with Spider engine
spider_job = client.start_spider(
    url='https://example.com',
    max_depth=3,          # Crawl up to 3 levels deep
    max_pages=50          # Stop after 50 pages
)

print(f"Spider job started: {spider_job['job_id']}")
print(f"Initial status: {spider_job['status']}")

# The spider will crawl discovered links up to max_depth
# Results are processed asynchronously
```

### PDF Processing

Extract structured content from PDF documents:

```python
import base64

# Read PDF file
with open('document.pdf', 'rb') as f:
    pdf_data = base64.b64encode(f.read()).decode('utf-8')

# Process PDF
result = client.pdf_process(
    pdf_data=pdf_data,
    filename='document.pdf'
)

print(f"Title: {result['document']['title']}")
print(f"Pages: {result['stats']['pages_processed']}")
print(f"Processing time: {result['stats']['processing_time_ms']}ms")
print(f"Content: {result['document']['content'][:500]}...")
```

### Session Management

Create persistent sessions for authenticated crawling:

```python
# Create session with authentication
session = client.create_session(
    name='my-session',
    config={
        'user_agent': 'MyBot/1.0',
        'cookies': [
            {
                'name': 'session_token',
                'value': 'abc123',
                'domain': 'example.com'
            }
        ]
    }
)

print(f"Session ID: {session['id']}")

# Use session for crawling protected content
result = client.crawl(
    urls=['https://protected-site.com/dashboard'],
    session_id=session['id']
)

# Extend session lifetime
client.extend_session(session['id'], ttl_seconds=7200)

# Cleanup when done
client.delete_session(session['id'])
```

### Worker Queue (Async Jobs)

Submit jobs for asynchronous processing:

```python
# Submit batch crawl job to worker queue
job = client.submit_job(
    job_type={
        'type': 'batch_crawl',
        'urls': ['https://example.com/page1', 'https://example.com/page2'],
        'options': {'concurrency': 10}
    },
    priority='High',
    retry_config={
        'max_attempts': 3,
        'initial_delay_secs': 5
    }
)

print(f"Job submitted: {job['job_id']}")

# Check job status
status = client.get_job_status(job['job_id'])
print(f"Status: {status['status']}")

# Get results when completed
if status['status'] == 'Completed':
    result = client.get_job_result(job['job_id'])
    print(f"Result: {result}")
```

### Error Handling

Comprehensive error handling with specific exceptions:

```python
from riptide_client import RipTide, APIError, RateLimitError, TimeoutError

client = RipTide('http://localhost:8080')

try:
    result = client.crawl(['https://example.com'])
except RateLimitError:
    print("Rate limit exceeded, waiting...")
    import time
    time.sleep(60)  # Wait before retrying
except TimeoutError as e:
    print(f"Request timed out: {e}")
except APIError as e:
    print(f"API error: {e}")
except Exception as e:
    print(f"Unexpected error: {e}")
```

### Advanced Options

Fine-tune client behavior with advanced configuration:

```python
# Custom timeout and retries
client = RipTide(
    base_url='http://localhost:8080',
    timeout=60,           # 60 seconds timeout
    max_retries=5         # Retry up to 5 times
)

# With API key authentication
client = RipTide(
    base_url='https://api.riptide.dev',
    api_key='your-api-key-here'
)

# Headless rendering for JavaScript-heavy sites
result = client.render(
    url='https://spa-website.com',
    wait_time=3000,       # Wait 3 seconds for JS to load
    screenshot=True       # Capture screenshot
)

if result.get('screenshot'):
    import base64
    screenshot_data = base64.b64decode(result['screenshot'])
    with open('screenshot.png', 'wb') as f:
        f.write(screenshot_data)
```

### Monitoring

Monitor system health and performance:

```python
# Health check
health = client.health()
print(f"Status: {health['status']}")
print(f"Dependencies: {health['dependencies']}")
print(f"Uptime: {health['uptime']}s")

# Health score (0-100)
score = client.health_score()
print(f"Health score: {score['score']}/100")
print(f"Status: {score['status']}")  # excellent, good, fair, poor, critical

# Performance metrics
report = client.performance_report()
print(f"Avg response time: {report['metrics']['avg_response_time_ms']}ms")
print(f"Requests/sec: {report['metrics']['requests_per_second']}")
print(f"Error rate: {report['metrics']['error_rate']}")
print(f"Recommendations: {report['recommendations']}")

# Worker status
workers = client.worker_status()
print(f"Total workers: {workers['total_workers']}")
print(f"Healthy workers: {workers['healthy_workers']}")
print(f"Jobs processed: {workers['total_jobs_processed']}")

# Prometheus metrics (raw format)
metrics = client.metrics()
print(metrics)  # Prometheus-formatted metrics
```

## 🔧 Performance Tips

### Connection Pooling

The client automatically manages connection pooling for optimal performance:

```python
# Connection pool is automatically configured
# Default: persistent connections with HTTP keep-alive
# Adjust via requests.Session configuration if needed

client = RipTide('http://localhost:8080')
# Reuse client instance across multiple requests for best performance
```

### Retry Configuration

Fine-tune retry behavior for different scenarios:

```python
from riptide_client import RipTide

# Aggressive retries for critical operations
client = RipTide(
    base_url='http://localhost:8080',
    max_retries=5,        # Retry up to 5 times
    timeout=90            # Longer timeout for slow sites
)

# The client uses exponential backoff:
# - 1st retry: ~1 second delay
# - 2nd retry: ~2 seconds delay
# - 3rd retry: ~4 seconds delay
# Automatically retries on: 429, 500, 502, 503, 504
```

### Batch Processing

Process multiple URLs efficiently:

```python
# Batch crawling with controlled concurrency
result = client.crawl(
    urls=['https://site1.com', 'https://site2.com', 'https://site3.com'],
    options={
        'concurrency': 10,     # Process 10 URLs concurrently
        'cache_mode': 'auto',  # Smart caching
        'timeout_ms': 30000    # 30 second timeout per URL
    }
)

# For very large batches, use streaming to avoid memory issues
for result in client.stream_crawl(large_url_list, options={'concurrency': 20}):
    process_result(result)  # Process each result as it arrives
```

### Caching Strategies

Optimize performance with intelligent caching:

```python
# Cache modes:
# - 'auto': Smart caching based on content type
# - 'read_only': Only read from cache, don't write
# - 'write_only': Write to cache, don't read
# - 'read_write': Full caching (default)
# - 'bypass': Skip cache entirely

result = client.crawl(
    urls=['https://example.com'],
    options={
        'cache_mode': 'read_write',    # Full caching
        'cache_ttl': 3600              # 1 hour cache TTL
    }
)
```

## 🔍 Troubleshooting

### Connection Errors

**Problem:** `ConnectionError: Failed to connect to API`

**Solutions:**
- Verify RipTide API is running: `curl http://localhost:8080/healthz`
- Check firewall settings
- Verify correct base URL and port
- Test network connectivity

### Timeout Issues

**Problem:** `TimeoutError: Request timed out`

**Solutions:**
```python
# Increase timeout for slow websites
client = RipTide(base_url='http://localhost:8080', timeout=120)

# Or adjust per-request
result = client.crawl(
    urls=['https://slow-site.com'],
    options={'timeout_ms': 90000}  # 90 seconds
)
```

### Rate Limiting

**Problem:** `RateLimitError: Rate limit exceeded`

**Solutions:**
```python
from time import sleep
from riptide_client import RateLimitError

try:
    result = client.crawl(urls)
except RateLimitError:
    sleep(60)  # Wait 1 minute
    result = client.crawl(urls)  # Retry

# Or reduce concurrency
result = client.crawl(urls, options={'concurrency': 2})
```

### Memory Issues with Large Crawls

**Problem:** High memory usage with large URL lists

**Solution:** Use streaming instead of batch crawling:
```python
# Instead of:
# result = client.crawl(huge_url_list)  # ❌ Loads all in memory

# Use streaming:
for result in client.stream_crawl(huge_url_list):  # ✅ Process incrementally
    save_to_database(result)
```

### Import Errors

**Problem:** `ModuleNotFoundError: No module named 'riptide_client'`

**Solutions:**
```bash
# Verify installation
pip list | grep riptide-client

# Reinstall if needed
pip install --upgrade riptide-client

# Check Python version
python --version  # Must be 3.8+
```

## ❓ FAQ

### Q: Is async/await supported?

**A:** The current version (1.0.0) uses synchronous requests. Async support is planned for v2.0. For now, use threading or multiprocessing for concurrent operations:

```python
from concurrent.futures import ThreadPoolExecutor

def crawl_url(url):
    return client.crawl([url])

with ThreadPoolExecutor(max_workers=10) as executor:
    results = executor.map(crawl_url, url_list)
```

### Q: How do I handle JavaScript-heavy websites?

**A:** Use the `render()` method for JavaScript-rendered content:

```python
result = client.render(
    url='https://spa-site.com',
    wait_time=5000,  # Wait 5 seconds for JS execution
    screenshot=True
)
```

### Q: Can I use custom headers?

**A:** Yes, configure via sessions:

```python
session = client.create_session(
    name='custom-headers',
    config={
        'user_agent': 'Mozilla/5.0 ...',
        'headers': {
            'Accept-Language': 'en-US',
            'Referer': 'https://example.com'
        }
    }
)
client.crawl(urls, session_id=session['id'])
```

### Q: How do I extract specific data?

**A:** Use extraction strategies:

```python
# CSS selectors
result = client.strategies_crawl(
    url='https://example.com',
    strategy='css_json',
    config={
        'selectors': {
            'title': 'h1.title',
            'price': 'span.price',
            'description': 'div.description'
        }
    }
)

# Or use auto strategy with LLM-powered extraction
result = client.strategies_crawl(
    url='https://example.com',
    strategy='auto'
)
```

### Q: Is there type stub support?

**A:** Yes! Full type hints are included in the package. Use with mypy for type checking:

```bash
pip install mypy
mypy your_script.py
```

### Q: How do I contribute?

**A:** See [CONTRIBUTING.md](../docs/development/contributing.md) for guidelines. We welcome:
- Bug reports and fixes
- Feature requests and implementations
- Documentation improvements
- Test coverage enhancements

## 📋 API Reference

### Client Initialization

```python
RipTide(
    base_url: str = "http://localhost:8080",
    api_key: Optional[str] = None,
    timeout: int = 30,
    max_retries: int = 3
)
```

**Parameters:**
- `base_url` - RipTide API base URL
- `api_key` - Optional API key for authentication
- `timeout` - Request timeout in seconds (default: 30)
- `max_retries` - Maximum retry attempts (default: 3)

### Core Methods

#### Crawling

- `crawl(urls, options=None, session_id=None)` - Batch crawl URLs
- `stream_crawl(urls, options=None)` - Stream crawl results (NDJSON)
- `render(url, wait_time=2000, screenshot=False)` - Headless browser render

#### Search

- `search(query, options=None)` - Deep search with content extraction

#### Spider

- `start_spider(url, max_depth=2, max_pages=10)` - Start deep crawling

#### Sessions

- `list_sessions()` - List all sessions
- `create_session(name, config=None)` - Create new session
- `get_session(session_id)` - Get session details
- `delete_session(session_id)` - Delete session
- `extend_session(session_id, ttl_seconds)` - Extend session TTL

#### Workers

- `submit_job(job_type, priority='Normal', retry_config=None)` - Submit async job
- `get_job_status(job_id)` - Get job status
- `get_job_result(job_id)` - Get job result
- `list_jobs(status=None)` - List jobs

#### Monitoring

- `health()` - System health check
- `metrics()` - Prometheus metrics (raw format)
- `health_score()` - Overall health score (0-100)
- `performance_report()` - Performance metrics with recommendations
- `worker_status()` - Worker queue statistics

#### Strategies

- `get_strategies()` - List extraction strategies
- `strategies_crawl(url, strategy, config=None)` - Advanced extraction

#### PDF

- `pdf_process(pdf_data, filename=None)` - Process PDF document

#### Tables

- `extract_tables(html_content, options=None)` - Extract HTML tables

## 🛠️ Development

### Setup Development Environment

```bash
# Clone repository
git clone https://github.com/your-org/riptide-api.git
cd riptide-api/python-sdk

# Create virtual environment
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install development dependencies
pip install -e ".[dev]"
```

### Run Tests

```bash
# Run all tests
pytest

# With coverage report
pytest --cov=riptide_client --cov-report=html

# Run specific test file
pytest tests/test_client.py

# Run with verbose output
pytest -v
```

### Code Quality

```bash
# Format code with Black
black riptide_client tests

# Type checking with mypy
mypy riptide_client

# Linting with Ruff
ruff check riptide_client

# Fix auto-fixable issues
ruff check --fix riptide_client
```

### Build Package

```bash
# Install build tools
pip install build twine

# Build distribution
python -m build

# Check distribution
twine check dist/*
```

## 📦 Publishing to PyPI

```bash
# Test on TestPyPI first
twine upload --repository testpypi dist/*

# Install from TestPyPI to verify
pip install --index-url https://test.pypi.org/simple/ riptide-client

# Upload to PyPI
twine upload dist/*
```

**Note:** Requires PyPI credentials. Configure in `~/.pypirc` or use environment variables.

## 📝 Testing

### Example Test

```python
import pytest
from riptide_client import RipTide

def test_health():
    """Test health endpoint."""
    client = RipTide('http://localhost:8080')
    health = client.health()
    assert health['status'] == 'healthy'
    assert 'version' in health
    assert 'dependencies' in health

def test_crawl():
    """Test basic crawling."""
    client = RipTide('http://localhost:8080')
    result = client.crawl(['https://example.com'])
    assert len(result['results']) == 1
    assert 'document' in result['results'][0]
    assert 'title' in result['results'][0]['document']

def test_context_manager():
    """Test context manager usage."""
    with RipTide('http://localhost:8080') as client:
        health = client.health()
        assert health['status'] == 'healthy'
    # Session automatically closed

@pytest.fixture
def client():
    """Reusable client fixture."""
    return RipTide('http://localhost:8080')

def test_with_fixture(client):
    """Test using fixture."""
    result = client.health()
    assert result['status'] == 'healthy'
```

### Running Integration Tests

```bash
# Ensure RipTide API is running
docker-compose up -d

# Run tests
pytest tests/

# Run only integration tests
pytest tests/ -m integration

# Stop services
docker-compose down
```

## 📚 Examples Gallery

Explore complete examples in the [examples directory](./examples/):

- **[basic_crawling.py](./examples/basic_crawling.py)** - Simple crawling examples
- **[streaming.py](./examples/streaming.py)** - Real-time streaming patterns
- **[sessions.py](./examples/sessions.py)** - Session management and authentication
- **[spider_crawl.py](./examples/spider_crawl.py)** - Deep recursive crawling
- **[pdf_processing.py](./examples/pdf_processing.py)** - PDF extraction workflows
- **[worker_queue.py](./examples/worker_queue.py)** - Async job processing
- **[monitoring.py](./examples/monitoring.py)** - Health checks and metrics
- **[error_handling.py](./examples/error_handling.py)** - Comprehensive error handling

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](../docs/development/contributing.md) for guidelines.

**Areas for contribution:**
- Async/await support
- Additional examples and documentation
- Test coverage improvements
- Bug fixes and optimizations
- New feature implementations

## 📄 License

MIT License - see [LICENSE](../LICENSE) file for details.

## 🔗 Links

- **Documentation**: [https://github.com/your-org/riptide-api/tree/main/docs](https://github.com/your-org/riptide-api/tree/main/docs)
- **API Reference**: [Endpoint Catalog](https://github.com/your-org/riptide-api/blob/main/docs/api/ENDPOINT_CATALOG.md)
- **Issues**: [Report bugs](https://github.com/your-org/riptide-api/issues)
- **Changelog**: [CHANGELOG.md](./CHANGELOG.md)
- **PyPI Package**: [https://pypi.org/project/riptide-client/](https://pypi.org/project/riptide-client/)

## 💬 Support

- **GitHub Issues**: [Report bugs or request features](https://github.com/your-org/riptide-api/issues)
- **Email**: support@riptide.dev
- **Discord**: [Join our community](https://discord.gg/riptide)
- **Stack Overflow**: Tag questions with `riptide-client`

## 🙏 Acknowledgments

Built with:
- [Requests](https://requests.readthedocs.io/) - HTTP library
- [urllib3](https://urllib3.readthedocs.io/) - Connection pooling
- [pytest](https://pytest.org/) - Testing framework

---

**Made with ❤️ by the RipTide Team**
