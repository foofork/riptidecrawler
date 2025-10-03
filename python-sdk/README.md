# RipTide Python Client

Official Python client for the RipTide web crawling and content extraction API.

## Installation

```bash
pip install riptide-client
```

## Quick Start

```python
from riptide_client import RipTide

# Initialize client
client = RipTide('http://localhost:8080')

# Crawl a URL
result = client.crawl(['https://example.com'])
print(result['results'][0]['document']['title'])

# Or use context manager
with RipTide('http://localhost:8080') as client:
    result = client.crawl(['https://example.com'])
    print(result)
```

## Features

- ✅ Full API coverage (59 endpoints)
- ✅ Type hints and autocompletion
- ✅ Automatic retries with exponential backoff
- ✅ Connection pooling
- ✅ Streaming support
- ✅ Session management
- ✅ Context manager support
- ✅ Comprehensive error handling

## Usage Examples

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

```python
# Stream results in real-time
for result in client.stream_crawl(['https://example.com']):
    print(f"Got: {result['url']}")
    print(f"Title: {result['document']['title']}")
```

### Search

```python
# Deep search with content extraction
results = client.search(
    query='python web scraping tutorials',
    options={
        'limit': 20,
        'include_content': True,
        'crawl_options': {
            'extract_mode': 'article'
        }
    }
)

for item in results['results']:
    print(f"{item['title']} - {item['url']}")
```

### Session Management

```python
# Create session with authentication
session = client.create_session(
    name='my-session',
    config={
        'user_agent': 'MyBot/1.0',
        'cookies': [
            {'name': 'session_token', 'value': 'abc123'}
        ]
    }
)

# Use session for crawling
result = client.crawl(
    urls=['https://protected-site.com'],
    session_id=session['id']
)

# Cleanup
client.delete_session(session['id'])
```

### Error Handling

```python
from riptide_client import RipTide, APIError, RateLimitError, TimeoutError

client = RipTide('http://localhost:8080')

try:
    result = client.crawl(['https://example.com'])
except RateLimitError:
    print("Rate limit exceeded, waiting...")
except TimeoutError:
    print("Request timed out")
except APIError as e:
    print(f"API error: {e}")
```

### Advanced Options

```python
# Custom timeout and retries
client = RipTide(
    base_url='http://localhost:8080',
    timeout=60,  # 60 seconds
    max_retries=5
)

# With API key
client = RipTide(
    base_url='http://localhost:8080',
    api_key='your-api-key'
)

# Headless rendering
result = client.render(
    url='https://spa-website.com',
    wait_time=3000,  # Wait 3 seconds
    screenshot=True
)

# Deep crawling with Spider
spider_job = client.start_spider(
    url='https://example.com',
    max_depth=3,
    max_pages=50
)
```

### Monitoring

```python
# Health check
health = client.health()
print(f"Status: {health['status']}")

# Health score
score = client.health_score()
print(f"Health score: {score['score']}/100")

# Performance metrics
report = client.performance_report()
print(report)

# Worker status
workers = client.worker_status()
print(f"Active jobs: {workers['active_jobs']}")
```

## API Reference

### Client

```python
RipTide(
    base_url: str = "http://localhost:8080",
    api_key: Optional[str] = None,
    timeout: int = 30,
    max_retries: int = 3
)
```

### Methods

#### Core

- `crawl(urls, options=None, session_id=None)` - Crawl URLs
- `stream_crawl(urls, options=None)` - Stream crawl results
- `search(query, options=None)` - Deep search
- `render(url, wait_time=2000, screenshot=False)` - Headless render

#### Sessions

- `list_sessions()` - List all sessions
- `create_session(name, config=None)` - Create session
- `get_session(session_id)` - Get session details
- `delete_session(session_id)` - Delete session

#### Monitoring

- `health()` - Health check
- `metrics()` - Prometheus metrics
- `health_score()` - Health score (0-100)
- `performance_report()` - Performance metrics
- `worker_status()` - Worker queue status

#### Spider

- `start_spider(url, max_depth=2, max_pages=10)` - Start deep crawling

#### Strategies

- `get_strategies()` - Get extraction strategies

## Development

```bash
# Clone repository
git clone https://github.com/your-org/riptide-api.git
cd riptide-api/python-sdk

# Install development dependencies
pip install -e ".[dev]"

# Run tests
pytest

# Format code
black riptide_client tests

# Type checking
mypy riptide_client

# Linting
ruff check riptide_client
```

## Testing

```python
import pytest
from riptide_client import RipTide

def test_health():
    client = RipTide('http://localhost:8080')
    health = client.health()
    assert health['status'] == 'healthy'

def test_crawl():
    client = RipTide('http://localhost:8080')
    result = client.crawl(['https://example.com'])
    assert len(result['results']) == 1
    assert 'document' in result['results'][0]
```

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](../docs/development/contributing.md)

## License

MIT License - see [LICENSE](../LICENSE) file

## Links

- [Documentation](https://github.com/your-org/riptide-api/tree/main/docs)
- [API Reference](https://github.com/your-org/riptide-api/blob/main/docs/api/ENDPOINT_CATALOG.md)
- [Issues](https://github.com/your-org/riptide-api/issues)
- [Changelog](CHANGELOG.md)

## Support

- GitHub Issues: [Report bugs](https://github.com/your-org/riptide-api/issues)
- Email: support@riptide.dev
- Discord: [Join community](https://discord.gg/riptide)
