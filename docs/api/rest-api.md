# RipTide Crawler - REST API Documentation

## Base URL
```
http://localhost:8080
```

## Authentication
Currently, no authentication is required. API keys may be implemented in future versions.

## Content Types
- **Request**: `application/json`
- **Response**: `application/json`

## Endpoints

### Health Check

#### `GET /healthz`
Returns the health status of the API service.

**Response**
```json
{
  "status": "ok",
  "version": "0.1.0"
}
```

**Status Codes**
- `200 OK` - Service is healthy

---

### Batch URL Crawling

#### `POST /crawl`
Crawls a batch of URLs and extracts content from each.

**Request Body**
```json
{
  "urls": [
    "https://example.com/article1",
    "https://example.com/article2"
  ],
  "options": {
    "concurrency": 16,
    "cache_mode": "read_through",
    "dynamic_wait_for": null,
    "scroll_steps": 8,
    "token_chunk_max": 1200,
    "token_overlap": 120
  }
}
```

**Request Schema**
| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `urls` | `string[]` | Yes | - | Array of URLs to crawl |
| `options` | `object` | No | See below | Crawling configuration |
| `options.concurrency` | `number` | No | `16` | Number of concurrent workers |
| `options.cache_mode` | `string` | No | `"read_through"` | Cache strategy: `"enabled"`, `"bypass"`, `"read_through"` |
| `options.dynamic_wait_for` | `string\|null` | No | `null` | CSS selector to wait for |
| `options.scroll_steps` | `number` | No | `8` | Number of scroll steps for dynamic content |
| `options.token_chunk_max` | `number` | No | `1200` | Maximum tokens per chunk |
| `options.token_overlap` | `number` | No | `120` | Token overlap between chunks |

**Response**
```json
{
  "received": 2,
  "results": [
    {
      "url": "https://example.com/article1",
      "title": "Article Title",
      "byline": "Author Name",
      "published_iso": "2024-01-15T10:30:00Z",
      "markdown": "# Article Title\n\nContent in markdown...",
      "text": "Article Title\n\nContent in plain text...",
      "links": [
        "https://example.com/related1",
        "https://example.com/related2"
      ],
      "media": [
        "https://example.com/image1.jpg",
        "https://example.com/video1.mp4"
      ]
    }
  ]
}
```

**Response Schema**
| Field | Type | Description |
|-------|------|-------------|
| `received` | `number` | Number of URLs received for processing |
| `results` | `CrawlResult[]` | Array of extraction results |

**CrawlResult Schema**
| Field | Type | Description |
|-------|------|-------------|
| `url` | `string` | Original URL |
| `title` | `string\|null` | Extracted page title |
| `byline` | `string\|null` | Author information |
| `published_iso` | `string\|null` | Publication date in ISO format |
| `markdown` | `string` | Content in markdown format |
| `text` | `string` | Content in plain text |
| `links` | `string[]` | Extracted links |
| `media` | `string[]` | Extracted media URLs |

**Status Codes**
- `200 OK` - Crawl completed successfully
- `400 Bad Request` - Invalid request format
- `500 Internal Server Error` - Processing error

---

### Deep Search

#### `POST /deepsearch`
Performs a web search and crawls the resulting URLs.

**Request Body**
```json
{
  "query": "artificial intelligence news",
  "limit": 10,
  "options": {
    "concurrency": 16,
    "cache_mode": "read_through"
  }
}
```

**Request Schema**
| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `query` | `string` | Yes | - | Search query |
| `limit` | `number` | No | `10` | Maximum number of results |
| `options` | `object` | No | Default crawl options | Crawling configuration |

**Response**
```json
{
  "query": "artificial intelligence news",
  "enqueued": 10,
  "status": "processing"
}
```

**Response Schema**
| Field | Type | Description |
|-------|------|-------------|
| `query` | `string` | Original search query |
| `enqueued` | `number` | Number of URLs queued for processing |
| `status` | `string` | Processing status |

**Status Codes**
- `200 OK` - Search initiated successfully
- `400 Bad Request` - Invalid query or parameters
- `500 Internal Server Error` - Search service error

**Note**: Requires `SERPER_API_KEY` environment variable to be set.

---

## Error Responses

All errors follow a consistent format:

```json
{
  "error": {
    "code": "INVALID_REQUEST",
    "message": "The request body is invalid",
    "details": "Additional error context"
  }
}
```

### Common Error Codes
- `INVALID_REQUEST` - Malformed request
- `URL_FETCH_FAILED` - Failed to fetch URL
- `EXTRACTION_FAILED` - Content extraction error
- `SEARCH_API_ERROR` - Search service unavailable
- `RATE_LIMITED` - Too many requests
- `INTERNAL_ERROR` - Server error

## Rate Limiting

Current implementation does not enforce rate limiting at the API level. Rate limiting is handled at the crawler level through:
- Configurable concurrency limits
- Timeout settings
- Respectful crawling delays

## Content Processing

### Static Content Flow
1. **Fetch** - HTTP GET request with headers
2. **Gate** - Content type and size validation
3. **Extract** - WASM-based content processing
4. **Cache** - Store results in Redis

### Dynamic Content Flow
1. **Fetch** - Initial HTTP request
2. **Gate** - Detect JavaScript requirements
3. **Render** - Send to headless browser service
4. **Extract** - Process rendered HTML
5. **Cache** - Store results

### WASM Extraction

Content extraction is performed by a WebAssembly component with the following capabilities:
- **Article Extraction** - Main content identification
- **Metadata Parsing** - Title, author, date extraction
- **Link Discovery** - Internal and external link extraction
- **Media Detection** - Image and video URL extraction
- **Format Conversion** - HTML to Markdown and plain text
- **Tokenization** - Content chunking for downstream processing

## Configuration

The API behavior is controlled by `/configs/riptide.yml`:

```yaml
crawl:
  concurrency: 16
  max_redirects: 5
  timeout_ms: 20000
  user_agent_mode: rotate
  robots_policy: obey
  cache: read_through
  max_response_mb: 20

extraction:
  mode: "article"
  produce_markdown: true
  produce_json: true
  token_chunk_max: 1200
  token_overlap: 120
```

## SDKs and Integration

### cURL Examples

**Health Check**
```bash
curl -X GET http://localhost:8080/healthz
```

**Crawl URLs**
```bash
curl -X POST http://localhost:8080/crawl \
  -H "Content-Type: application/json" \
  -d '{
    "urls": ["https://example.com"],
    "options": {
      "concurrency": 8
    }
  }'
```

**Deep Search**
```bash
curl -X POST http://localhost:8080/deepsearch \
  -H "Content-Type: application/json" \
  -d '{
    "query": "machine learning tutorials",
    "limit": 5
  }'
```

### JavaScript/Node.js
```javascript
const response = await fetch('http://localhost:8080/crawl', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    urls: ['https://example.com'],
    options: {
      concurrency: 8
    }
  })
});

const results = await response.json();
```

### Python
```python
import requests

response = requests.post('http://localhost:8080/crawl', json={
    'urls': ['https://example.com'],
    'options': {
        'concurrency': 8
    }
})

results = response.json()
```

## Monitoring and Debugging

### Health Monitoring
Regular health checks should be performed:
```bash
curl -f http://localhost:8080/healthz
```

### Request Tracing
All requests are traced with correlation IDs. Check application logs for detailed request information.

### Debugging Failed Requests
1. Check URL accessibility
2. Verify content type compatibility
3. Review extraction configuration
4. Monitor Redis connectivity
5. Validate headless service availability

## Performance Considerations

### Optimization Tips
1. **Batch Requests** - Group multiple URLs in single crawl request
2. **Cache Strategy** - Use `read_through` for optimal performance
3. **Concurrency** - Adjust based on target server capacity
4. **Content Size** - Monitor `max_response_mb` setting
5. **Dynamic Content** - Only use when necessary (slower than static)

### Monitoring Metrics
- Request latency
- Crawl success rate
- Cache hit ratio
- Memory usage
- CPU utilization

## Changelog

### v0.1.0
- Initial API implementation
- Basic crawl and deepsearch endpoints
- WASM-based content extraction
- Redis caching integration
- Docker deployment support