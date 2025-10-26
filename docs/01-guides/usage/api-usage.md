# API Usage Guide

This comprehensive guide covers all RipTide Crawler API endpoints with practical examples, request/response formats, and integration patterns.

## Quick Start

```bash
# Basic crawl request
curl -X POST http://localhost:8080/crawl \
  -H "Content-Type: application/json" \
  -d '{"urls": ["https://example.com"]}'

# Search and crawl
curl -X POST http://localhost:8080/deepsearch \
  -H "Content-Type: application/json" \
  -d '{"query": "rust programming", "limit": 10}'
```

## Base URL and Authentication

### Base URL
```
http://localhost:8080  # Default local installation
https://your-domain.com/api  # Production deployment
```

### Authentication (Optional)
```bash
# API Key authentication (if enabled)
curl -H "Authorization: Bearer your-api-key" \
  http://localhost:8080/crawl

# Basic authentication (if configured)
curl -u username:password \
  http://localhost:8080/crawl
```

## Core Endpoints

### 1. Health Check

Check service status and version information.

**Endpoint:** `GET /health`

```bash
curl http://localhost:8080/health
```

**Response:**
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "timestamp": "2024-03-15T10:30:00Z",
  "services": {
    "redis": "connected",
    "headless": "available",
    "wasm_extractor": "loaded"
  },
  "metrics": {
    "uptime_seconds": 3600,
    "total_requests": 1234,
    "cache_hit_rate": 0.85
  }
}
```

### 2. Crawl URLs

Extract content from specific URLs.

**Endpoint:** `POST /crawl`

#### Basic Request
```bash
curl -X POST http://localhost:8080/crawl \
  -H "Content-Type: application/json" \
  -d '{
    "urls": [
      "https://example.com",
      "https://another-site.com/article"
    ]
  }'
```

#### Advanced Request
```bash
curl -X POST http://localhost:8080/crawl \
  -H "Content-Type: application/json" \
  -d '{
    "urls": [
      "https://dynamic-site.com/spa-page",
      "https://blog.example.com/post/123"
    ],
    "options": {
      "concurrency": 5,
      "cache_mode": "bypass",
      "dynamic_wait_for": ".main-content",
      "scroll_steps": 3,
      "force_headless": false,
      "extract_mode": "article"
    }
  }'
```

#### Request Schema
```typescript
interface CrawlRequest {
  urls: string[];                    // Array of URLs to crawl
  options?: {
    concurrency?: number;            // Concurrent requests (1-50)
    cache_mode?: "enabled" | "bypass" | "read_through";
    dynamic_wait_for?: string;       // CSS selector to wait for
    scroll_steps?: number;           // Auto-scroll steps (0-20)
    force_headless?: boolean;        // Force headless mode
    extract_mode?: "article" | "full" | "metadata";
    timeout_seconds?: number;        // Per-URL timeout
    custom_headers?: Record<string, string>;
  };
}
```

#### Response Schema
```typescript
interface CrawlResponse {
  request_id: string;
  total_urls: number;
  completed: number;
  failed: number;
  duration_ms: number;
  results: CrawlResult[];
}

interface CrawlResult {
  url: string;
  final_url: string;               // After redirects
  status: number;                  // HTTP status code
  success: boolean;
  from_cache: boolean;
  strategy_used: "fast" | "wasm" | "headless";
  duration_ms: number;
  content?: ExtractedContent;
  error?: string;
  artifacts: {
    json_path?: string;            // Path to JSON output
    markdown_path?: string;        // Path to Markdown output
    screenshot_path?: string;      // Path to screenshot (if taken)
  };
}

interface ExtractedContent {
  title?: string;
  byline?: string;
  published_date?: string;
  text: string;
  markdown: string;
  summary?: string;
  links: string[];
  images: string[];
  videos: string[];
  metadata: Record<string, any>;
  language?: string;
  word_count: number;
  reading_time_minutes: number;
}
```

#### Example Response
```json
{
  "request_id": "req_123456789",
  "total_urls": 2,
  "completed": 2,
  "failed": 0,
  "duration_ms": 3450,
  "results": [
    {
      "url": "https://example.com",
      "final_url": "https://example.com/",
      "status": 200,
      "success": true,
      "from_cache": false,
      "strategy_used": "fast",
      "duration_ms": 1200,
      "content": {
        "title": "Example Domain",
        "text": "This domain is for use in illustrative examples...",
        "markdown": "# Example Domain\n\nThis domain is for use...",
        "links": ["https://www.iana.org/domains/example"],
        "images": [],
        "videos": [],
        "metadata": {
          "og:title": "Example Domain",
          "description": "Example site description"
        },
        "language": "en",
        "word_count": 45,
        "reading_time_minutes": 1
      },
      "artifacts": {
        "json_path": "/data/artifacts/2024/03/15/example_com_content.json",
        "markdown_path": "/data/artifacts/2024/03/15/example_com_content.md"
      }
    }
  ]
}
```

### 3. Deep Search

Search for content using SERP APIs, then crawl the results.

**Endpoint:** `POST /deepsearch`

#### Basic Request
```bash
curl -X POST http://localhost:8080/deepsearch \
  -H "Content-Type: application/json" \
  -d '{
    "query": "rust programming language tutorial",
    "limit": 10
  }'
```

#### Advanced Request
```bash
curl -X POST http://localhost:8080/deepsearch \
  -H "Content-Type: application/json" \
  -d '{
    "query": "machine learning python 2024",
    "limit": 20,
    "country": "us",
    "locale": "en",
    "time_range": "past_year",
    "search_type": "web",
    "crawl_options": {
      "extract_mode": "article",
      "cache_mode": "read_through",
      "concurrency": 8
    },
    "filters": {
      "include_domains": ["github.com", "medium.com"],
      "exclude_domains": ["pinterest.com"],
      "min_content_length": 500
    }
  }'
```

#### Request Schema
```typescript
interface DeepSearchRequest {
  query: string;                   // Search query
  limit?: number;                  // Max results (1-100, default: 10)
  country?: string;                // Country code (us, uk, etc.)
  locale?: string;                 // Language code (en, es, etc.)
  time_range?: "past_day" | "past_week" | "past_month" | "past_year";
  search_type?: "web" | "news" | "images";
  crawl_options?: CrawlOptions;    // Same as /crawl options
  filters?: {
    include_domains?: string[];
    exclude_domains?: string[];
    min_content_length?: number;
    require_title?: boolean;
  };
}
```

#### Response Schema
```typescript
interface DeepSearchResponse {
  request_id: string;
  query: string;
  search_results: SearchResult[];
  crawl_summary: CrawlSummary;
  results: CrawlResult[];          // Same as /crawl response
}

interface SearchResult {
  title: string;
  url: string;
  snippet: string;
  position: number;
  source: string;
}

interface CrawlSummary {
  total_found: number;
  attempted_crawl: number;
  successful_crawl: number;
  failed_crawl: number;
  duration_ms: number;
}
```

### 4. Stream Crawling (Server-Sent Events)

Real-time crawling with live progress updates.

**Endpoint:** `POST /crawl/stream`

```bash
curl -X POST http://localhost:8080/crawl/stream \
  -H "Content-Type: application/json" \
  -H "Accept: text/event-stream" \
  -d '{
    "urls": ["https://site1.com", "https://site2.com"],
    "options": {"concurrency": 3}
  }'
```

**Event Stream Format:**
```
event: progress
data: {"completed": 1, "total": 10, "current_url": "https://site1.com"}

event: result
data: {"url": "https://site1.com", "status": 200, "success": true, ...}

event: complete
data: {"total_results": 10, "successful": 8, "failed": 2}
```

### 5. Batch Operations

Handle large-scale crawling operations.

**Endpoint:** `POST /batch/crawl`

```bash
curl -X POST http://localhost:8080/batch/crawl \
  -H "Content-Type: application/json" \
  -d '{
    "name": "news_crawl_2024_03",
    "urls": [
      "https://news-site-1.com",
      "https://news-site-2.com"
    ],
    "options": {
      "concurrency": 10,
      "priority": "low",
      "schedule": "immediate"
    }
  }'
```

**Check Batch Status:**
```bash
curl http://localhost:8080/batch/news_crawl_2024_03/status
```

## Advanced Usage Patterns

### 1. Content Validation and Quality Control

```bash
# Crawl with quality thresholds
curl -X POST http://localhost:8080/crawl \
  -H "Content-Type: application/json" \
  -d '{
    "urls": ["https://blog.example.com"],
    "options": {
      "extract_mode": "article",
      "quality_checks": {
        "min_word_count": 200,
        "min_paragraph_count": 3,
        "require_title": true,
        "max_link_density": 0.3
      }
    }
  }'
```

### 2. Multi-Format Output

```bash
# Request specific output formats
curl -X POST http://localhost:8080/crawl \
  -H "Content-Type: application/json" \
  -d '{
    "urls": ["https://example.com"],
    "options": {
      "output_formats": ["json", "markdown", "text", "html"],
      "include_raw_html": true,
      "generate_summary": true
    }
  }'
```

### 3. Dynamic Content Handling

```bash
# Complex SPA crawling
curl -X POST http://localhost:8080/crawl \
  -H "Content-Type: application/json" \
  -d '{
    "urls": ["https://spa-app.com/dashboard"],
    "options": {
      "force_headless": true,
      "dynamic_wait_for": ".data-loaded",
      "scroll_steps": 5,
      "custom_js": "document.querySelector(\".load-more\").click();",
      "wait_after_js": 3000,
      "screenshot": true
    }
  }'
```

### 4. Bulk URL Processing

```bash
# Process large URL lists
curl -X POST http://localhost:8080/crawl \
  -H "Content-Type: application/json" \
  -d @urls.json

# urls.json:
{
  "urls": [
    "https://site1.com/page1",
    "https://site1.com/page2",
    "..."
  ],
  "options": {
    "concurrency": 20,
    "batch_size": 100,
    "respect_rate_limits": true
  }
}
```

## Error Handling

### HTTP Status Codes

- `200` - Success
- `400` - Bad Request (invalid parameters)
- `401` - Unauthorized (missing/invalid API key)
- `429` - Too Many Requests (rate limited)
- `500` - Internal Server Error
- `503` - Service Unavailable

### Error Response Format

```json
{
  "error": {
    "code": "INVALID_URL",
    "message": "One or more URLs are invalid",
    "details": {
      "invalid_urls": ["not-a-url", "ftp://unsupported.com"]
    },
    "request_id": "req_123456789"
  }
}
```

### Common Error Codes

- `INVALID_URL` - Malformed or unsupported URL
- `RATE_LIMITED` - Too many requests
- `TIMEOUT` - Request timed out
- `EXTRACTION_FAILED` - Content extraction failed
- `HEADLESS_UNAVAILABLE` - Headless service not available
- `CACHE_ERROR` - Redis cache error
- `QUOTA_EXCEEDED` - API quota exceeded

## Rate Limiting

### Default Limits
- 100 requests per minute per IP
- 1000 requests per hour per API key
- 50 concurrent requests per client

### Rate Limit Headers
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 85
X-RateLimit-Reset: 1647345600
Retry-After: 60
```

### Handling Rate Limits
```javascript
async function crawlWithRetry(urls, options = {}) {
  try {
    const response = await fetch('/crawl', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ urls, options })
    });

    if (response.status === 429) {
      const retryAfter = response.headers.get('Retry-After');
      await new Promise(resolve =>
        setTimeout(resolve, parseInt(retryAfter) * 1000)
      );
      return crawlWithRetry(urls, options);
    }

    return await response.json();
  } catch (error) {
    console.error('Crawl failed:', error);
    throw error;
  }
}
```

## Client Libraries and SDKs

### JavaScript/TypeScript

```javascript
import { RipTideClient } from 'riptide-client-js';

const client = new RipTideClient({
  baseUrl: 'http://localhost:8080',
  apiKey: 'your-api-key',
  timeout: 30000
});

// Simple crawl
const result = await client.crawl(['https://example.com']);

// Advanced crawl
const result = await client.crawl(
  ['https://spa-site.com'],
  {
    forceHeadless: true,
    waitFor: '.content-loaded',
    scrollSteps: 3
  }
);

// Deep search
const searchResults = await client.deepSearch('rust programming', {
  limit: 20,
  country: 'us'
});
```

### Python

```python
from riptide_client import RipTideClient

client = RipTideClient(
    base_url='http://localhost:8080',
    api_key='your-api-key'
)

# Simple crawl
result = client.crawl(['https://example.com'])

# Advanced crawl with options
result = client.crawl(
    ['https://dynamic-site.com'],
    options={
        'force_headless': True,
        'dynamic_wait_for': '.main-content',
        'scroll_steps': 5
    }
)

# Stream crawling
for event in client.crawl_stream(['https://site1.com', 'https://site2.com']):
    if event.type == 'result':
        print(f"Crawled: {event.data['url']}")
```

### cURL Scripts

```bash
#!/bin/bash
# crawl_batch.sh

API_BASE="http://localhost:8080"
API_KEY="your-api-key"

crawl_urls() {
    local urls_json=$(printf '%s\n' "$@" | jq -R . | jq -s .)

    curl -X POST "${API_BASE}/crawl" \
        -H "Authorization: Bearer ${API_KEY}" \
        -H "Content-Type: application/json" \
        -d "{\"urls\": ${urls_json}}" \
        | jq '.'
}

# Usage: ./crawl_batch.sh url1 url2 url3
crawl_urls "$@"
```

## Integration Examples

### 1. News Aggregation

```javascript
// Automated news crawling
const newsSearch = await client.deepSearch('technology news today', {
  limit: 50,
  country: 'us',
  timeRange: 'past_day',
  filters: {
    includeDomains: ['techcrunch.com', 'arstechnica.com', 'theverge.com'],
    minContentLength: 500
  }
});

// Process results
for (const result of newsSearch.results) {
  if (result.success && result.content) {
    await storeArticle({
      title: result.content.title,
      content: result.content.text,
      url: result.url,
      publishedDate: result.content.published_date,
      source: new URL(result.url).hostname
    });
  }
}
```

### 2. Content Monitoring

```javascript
// Monitor website changes
const monitorSite = async (url) => {
  const result = await client.crawl([url], {
    cacheMode: 'bypass'  // Always fetch fresh content
  });

  if (result.results[0].success) {
    const content = result.results[0].content;
    const hash = generateHash(content.text);

    if (hash !== getLastKnownHash(url)) {
      await notifyContentChange(url, content);
      saveContentHash(url, hash);
    }
  }
};

// Run every hour
setInterval(() => monitorSite('https://important-site.com'), 3600000);
```

### 3. SEO Analysis

```javascript
// Comprehensive site analysis
const analyzeSite = async (domain) => {
  // Get sitemap URLs
  const sitemapUrls = await extractSitemapUrls(`${domain}/sitemap.xml`);

  // Crawl all pages
  const results = await client.crawl(sitemapUrls, {
    concurrency: 10,
    extractMode: 'full'
  });

  // Analyze results
  const analysis = results.results.map(result => ({
    url: result.url,
    title: result.content?.title,
    wordCount: result.content?.word_count,
    hasMetaDescription: !!result.content?.metadata?.description,
    internalLinks: result.content?.links?.filter(link =>
      link.includes(domain)
    ).length,
    readingTime: result.content?.reading_time_minutes
  }));

  return generateSEOReport(analysis);
};
```

## Performance Optimization

### 1. Optimal Concurrency

```javascript
// Determine optimal concurrency based on target site
const optimizeConcurrency = (domain) => {
  const knownSlowSites = ['heavy-site.com', 'slow-server.org'];
  const knownFastSites = ['cdn-site.com', 'optimized.com'];

  if (knownSlowSites.some(site => domain.includes(site))) {
    return 2;  // Low concurrency for slow sites
  } else if (knownFastSites.some(site => domain.includes(site))) {
    return 20; // High concurrency for fast sites
  }

  return 8; // Default moderate concurrency
};
```

### 2. Intelligent Caching

```javascript
// Smart cache strategy
const crawlWithSmartCache = async (urls) => {
  const cacheMode = urls.some(url =>
    url.includes('/news/') || url.includes('/blog/')
  ) ? 'bypass' : 'read_through';

  return await client.crawl(urls, { cacheMode });
};
```

### 3. Progressive Enhancement

```javascript
// Try fast extraction first, fallback to headless if needed
const extractWithFallback = async (url) => {
  // First attempt: fast extraction
  let result = await client.crawl([url], {
    forceHeadless: false,
    timeout: 10
  });

  // If content seems insufficient, try headless
  if (result.results[0].content?.word_count < 100) {
    result = await client.crawl([url], {
      forceHeadless: true,
      dynamicWaitFor: 'body',
      scrollSteps: 3
    });
  }

  return result;
};
```

## Monitoring and Debugging

### 1. Request Tracking

```javascript
// Add request ID tracking
const crawlWithTracking = async (urls, options = {}) => {
  const requestId = generateRequestId();

  console.log(`Starting crawl ${requestId} for ${urls.length} URLs`);

  const result = await client.crawl(urls, {
    ...options,
    metadata: { requestId }
  });

  console.log(`Completed crawl ${requestId}: ${result.completed}/${result.total_urls} successful`);

  return result;
};
```

### 2. Error Analytics

```javascript
// Collect error statistics
const errorStats = new Map();

const trackErrors = (result) => {
  result.results.forEach(item => {
    if (!item.success) {
      const domain = new URL(item.url).hostname;
      const stats = errorStats.get(domain) || { count: 0, errors: [] };
      stats.count++;
      stats.errors.push(item.error);
      errorStats.set(domain, stats);
    }
  });
};

// Periodic error reporting
setInterval(() => {
  console.log('Error statistics:', Object.fromEntries(errorStats));
}, 300000); // Every 5 minutes
```

## Best Practices

### 1. Respectful Crawling

- Use appropriate delays between requests
- Respect robots.txt files
- Implement exponential backoff for retries
- Monitor target site performance

### 2. Content Quality

- Set minimum content length thresholds
- Validate extracted data before storage
- Implement content deduplication
- Handle different languages appropriately

### 3. Resource Management

- Use appropriate concurrency limits
- Implement request timeouts
- Monitor memory usage
- Clean up temporary files

### 4. Error Recovery

- Implement retry logic with backoff
- Handle partial failures gracefully
- Log errors for debugging
- Provide fallback mechanisms

This comprehensive API guide should help you integrate RipTide Crawler effectively into your applications. For more specific use cases or advanced configurations, refer to the [Configuration Guide](configuration.md) and [Troubleshooting Guide](troubleshooting.md).