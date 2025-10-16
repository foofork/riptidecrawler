# Strategy Endpoints API Documentation

## Overview

The Strategy Endpoints provide advanced content extraction capabilities with multiple configurable extraction strategies and chunking modes. These endpoints enable fine-grained control over how web content is processed and structured.

## Features

- **Multiple Extraction Strategies**: Wasm (WASM), CSS/JSON selectors, Regex patterns, LLM-based extraction
- **Flexible Chunking**: Sliding windows, fixed sizes, sentence boundaries, topic-based, regex splits
- **Auto-Detection**: Automatic strategy selection based on content analysis
- **Performance Metrics**: Detailed processing statistics and quality scores
- **Caching**: Redis-backed caching with configurable TTL
- **Schema Validation**: Optional schema validation for extracted content

---

## Endpoints

### 1. POST /strategies/crawl

Process a URL using configurable extraction strategies and chunking modes.

#### Query Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `strategy` | string | `"auto"` | Extraction strategy: `"auto"`, `"wasm"`, `"css_json"`, `"regex"`, `"llm"` |
| `chunking` | string | `"sliding"` | Chunking mode: `"sliding"`, `"fixed"`, `"sentence"`, `"topic"`, `"regex"` |

#### Request Body

```json
{
  "url": "https://example.com/article",
  "extraction_strategy": "wasm",
  "enable_metrics": true,
  "validate_schema": true,
  "cache_mode": "read_write",
  "css_selectors": {
    "title": "h1.entry-title",
    "content": ".entry-content",
    "author": ".author",
    "date": "time.published"
  },
  "regex_patterns": [
    {
      "name": "email",
      "pattern": "[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}",
      "field": "emails",
      "required": false
    }
  ],
  "llm_config": {
    "enabled": false,
    "model": "claude-3-sonnet-20240229",
    "prompt_template": "Extract the main content from this webpage"
  },
  "chunking_config": {
    "mode": "sliding",
    "token_max": 512,
    "overlap": 128,
    "preserve_sentences": true,
    "deterministic": true
  }
}
```

#### Request Schema

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `url` | string | Yes | URL to crawl and process |
| `extraction_strategy` | string | No | Override default extraction strategy |
| `enable_metrics` | boolean | No | Collect performance metrics (default: `true`) |
| `validate_schema` | boolean | No | Validate extracted content schema (default: `true`) |
| `cache_mode` | string | No | Cache behavior: `"default"`, `"bypass"`, `"refresh"` |
| `css_selectors` | object | No | CSS selectors for CSS_JSON strategy |
| `regex_patterns` | array | No | Regex patterns for REGEX strategy |
| `llm_config` | object | No | LLM configuration for LLM strategy |
| `chunking_config` | object | No | Chunking configuration (deprecated - handled by riptide-html) |

##### CSS Selectors Object

```json
{
  "title": "CSS selector for title",
  "content": "CSS selector for main content",
  "author": "CSS selector for author",
  "date": "CSS selector for date"
}
```

##### Regex Pattern Object

```json
{
  "name": "pattern_name",
  "pattern": "regex pattern",
  "field": "target_field",
  "required": true|false
}
```

##### LLM Config Object

```json
{
  "enabled": true|false,
  "model": "model_identifier",
  "prompt_template": "extraction prompt"
}
```

##### Chunking Config Object (Deprecated)

Chunking is now handled by the `riptide-html` crate. This configuration is preserved for backward compatibility but has no effect.

#### Response

```json
{
  "success": true,
  "result": {
    "processed_content": {
      "content": "Extracted content text...",
      "metadata": {
        "title": "Article Title",
        "author": "John Doe",
        "published": "2024-01-15T10:00:00Z"
      },
      "chunks": [],
      "extracted_data": {}
    },
    "from_cache": false,
    "gate_decision": "raw",
    "quality_score": 0.85,
    "processing_time_ms": 1250,
    "cache_key": "riptide:strategies:v1:read_write:a7f3c2d1",
    "http_status": 200,
    "strategy_config": {
      "extraction": "Wasm",
      "enable_metrics": true,
      "validate_schema": true
    },
    "performance_metrics": {
      "extraction_time_ms": 850,
      "validation_time_ms": 120,
      "total_time_ms": 1250,
      "content_size_bytes": 45678
    }
  },
  "stats": {
    "chunks_created": 1,
    "total_processing_time_ms": 1250,
    "extraction_strategy_used": "Wasm",
    "chunking_mode_used": "None",
    "cache_hit": false,
    "quality_score": 0.85
  }
}
```

#### Response Schema

| Field | Type | Description |
|-------|------|-------------|
| `success` | boolean | Indicates successful processing |
| `result` | object | Processed content and metadata |
| `result.processed_content` | object | Extracted and processed content |
| `result.from_cache` | boolean | Whether result was served from cache |
| `result.gate_decision` | string | Routing decision: `"raw"`, `"probes_first"`, `"headless"`, `"cached"`, `"pdf"` |
| `result.quality_score` | number | Content quality score (0.0-1.0) |
| `result.processing_time_ms` | integer | Total processing time in milliseconds |
| `result.cache_key` | string | Cache key used for this request |
| `result.http_status` | integer | HTTP status code from original fetch |
| `result.strategy_config` | object | Strategy configuration used |
| `result.performance_metrics` | object | Detailed performance metrics (if enabled) |
| `stats` | object | Processing statistics summary |

#### Error Responses

**400 Bad Request**
```json
{
  "error": {
    "type": "invalid_request",
    "message": "URL cannot be empty",
    "retryable": false,
    "status": 400
  }
}
```

**500 Internal Server Error**
```json
{
  "error": {
    "type": "pipeline_error",
    "message": "Strategy processing failed: extraction error",
    "retryable": true,
    "status": 500
  }
}
```

**504 Gateway Timeout**
```json
{
  "error": {
    "type": "timeout",
    "message": "Timeout fetching https://example.com",
    "retryable": true,
    "status": 504
  }
}
```

---

### 2. GET /strategies/info

Get information about available extraction strategies and chunking modes.

#### Request

No parameters required.

```bash
curl -X GET https://api.riptide.dev/strategies/info
```

#### Response

```json
{
  "extraction_strategies": [
    {
      "name": "wasm",
      "description": "Default WASM-based extraction (fastest)",
      "parameters": []
    },
    {
      "name": "css_json",
      "description": "CSS selector to JSON extraction",
      "parameters": [
        {
          "name": "selectors",
          "required": false,
          "description": "CSS selectors mapping (field -> selector)"
        }
      ]
    },
    {
      "name": "regex",
      "description": "Regex pattern extraction",
      "parameters": [
        {
          "name": "patterns",
          "required": true,
          "description": "List of regex patterns to apply"
        }
      ]
    },
    {
      "name": "llm",
      "description": "LLM-based extraction (hook-based, disabled by default)",
      "parameters": [
        {
          "name": "enabled",
          "required": true,
          "description": "Enable LLM extraction"
        },
        {
          "name": "model",
          "required": false,
          "description": "LLM model to use"
        }
      ]
    }
  ],
  "chunking_modes": [
    {
      "name": "sliding",
      "description": "Sliding windows with overlap (default)",
      "parameters": ["token_max", "overlap", "preserve_sentences"]
    },
    {
      "name": "fixed",
      "description": "Fixed character/token count",
      "parameters": ["size", "by_tokens"]
    },
    {
      "name": "sentence",
      "description": "Split by sentence boundaries (NLP)",
      "parameters": ["max_sentences"]
    },
    {
      "name": "topic",
      "description": "Split by semantic topics",
      "parameters": ["similarity_threshold"]
    },
    {
      "name": "regex",
      "description": "Split by regex pattern",
      "parameters": ["pattern", "min_chunk_size"]
    }
  ]
}
```

#### Response Schema

| Field | Type | Description |
|-------|------|-------------|
| `extraction_strategies` | array | List of available extraction strategies |
| `extraction_strategies[].name` | string | Strategy identifier |
| `extraction_strategies[].description` | string | Human-readable description |
| `extraction_strategies[].parameters` | array | Required/optional parameters |
| `chunking_modes` | array | List of available chunking modes (deprecated) |

---

## Extraction Strategies

### Wasm Strategy (Default)

High-performance WASM-based extraction using the Wasm engine.

**Characteristics:**
- Fastest extraction method
- No additional configuration required
- Automatic content detection
- Handles HTML, article content, metadata

**Usage:**
```bash
curl -X POST "https://api.riptide.dev/strategies/crawl?strategy=wasm" \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://example.com/article",
    "enable_metrics": true
  }'
```

**Example Response:**
```json
{
  "success": true,
  "result": {
    "processed_content": {
      "content": "Full article text extracted...",
      "metadata": {
        "title": "Article Title",
        "published": "2024-01-15"
      }
    },
    "quality_score": 0.92,
    "processing_time_ms": 850
  },
  "stats": {
    "extraction_strategy_used": "Wasm",
    "quality_score": 0.92
  }
}
```

---

### CSS/JSON Strategy

Extract content using custom CSS selectors mapped to JSON fields.

**Characteristics:**
- Precise field extraction
- Customizable selectors
- Handles structured content
- Fallback to default selectors

**Usage:**
```bash
curl -X POST "https://api.riptide.dev/strategies/crawl?strategy=css_json" \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://example.com/blog/post",
    "css_selectors": {
      "title": "h1.post-title",
      "content": "article.post-content",
      "author": ".author-name",
      "date": "time.published-date",
      "tags": ".tag-list > .tag"
    }
  }'
```

**Example Response:**
```json
{
  "success": true,
  "result": {
    "processed_content": {
      "content": "Article content...",
      "metadata": {
        "title": "How to Use CSS Selectors",
        "author": "Jane Developer",
        "date": "2024-01-15",
        "tags": ["CSS", "Web Development", "Tutorial"]
      }
    },
    "quality_score": 0.88
  }
}
```

**Built-in Selector Templates:**

GitHub selectors:
```json
{
  "title": "h1.entry-title, .js-issue-title, .repository-content h1",
  "content": ".entry-content, .markdown-body, .comment-body",
  "author": ".author, .commit-author, .discussion-item-header a",
  "date": "time, .commit-date, relative-time"
}
```

Blog selectors:
```json
{
  "title": "h1, .entry-title, .post-title, [data-testid='storyTitle']",
  "content": ".entry-content, .post-content, .story-content, article",
  "author": ".author, .byline, .writer, [data-testid='authorName']",
  "date": "time, .date, .published, [data-testid='storyPublishDate']"
}
```

---

### Regex Strategy

Extract content using regular expression patterns.

**Characteristics:**
- Pattern-based extraction
- Multiple patterns support
- Field mapping
- Required/optional patterns

**Usage:**
```bash
curl -X POST "https://api.riptide.dev/strategies/crawl?strategy=regex" \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://example.com/contact",
    "regex_patterns": [
      {
        "name": "emails",
        "pattern": "[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}",
        "field": "contact_emails",
        "required": false
      },
      {
        "name": "phones",
        "pattern": "\\+?[1-9]\\d{1,14}",
        "field": "phone_numbers",
        "required": false
      },
      {
        "name": "dates",
        "pattern": "\\d{4}-\\d{2}-\\d{2}",
        "field": "dates",
        "required": false
      }
    ]
  }'
```

**Example Response:**
```json
{
  "success": true,
  "result": {
    "processed_content": {
      "content": "Contact page content...",
      "extracted_data": {
        "contact_emails": ["info@example.com", "support@example.com"],
        "phone_numbers": ["+1-555-0123", "+1-555-0124"],
        "dates": ["2024-01-15", "2024-02-01"]
      }
    }
  }
}
```

---

### LLM Strategy

Extract content using Large Language Model processing (requires hook integration).

**Characteristics:**
- AI-powered extraction
- Natural language understanding
- Flexible prompt templates
- Hook-based integration (disabled by default)

**Usage:**
```bash
curl -X POST "https://api.riptide.dev/strategies/crawl?strategy=llm" \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://example.com/research-paper",
    "llm_config": {
      "enabled": true,
      "model": "claude-3-sonnet-20240229",
      "prompt_template": "Extract the abstract, methodology, and conclusions from this research paper"
    }
  }'
```

**Note:** LLM strategy requires hook-based implementation and is disabled by default. Enable via application configuration.

---

### Auto Strategy

Automatically select the best extraction strategy based on content analysis.

**Characteristics:**
- Intelligent strategy selection
- URL pattern recognition
- Content type detection
- Optimized for performance

**Usage:**
```bash
curl -X POST "https://api.riptide.dev/strategies/crawl?strategy=auto" \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://example.com/unknown-page",
    "enable_metrics": true
  }'
```

**Auto-Detection Logic:**
- GitHub domains → Wasm with GitHub selectors
- Blog domains → Wasm with blog selectors
- PDF content → PDF extraction pipeline
- SPA detection → Headless rendering
- Default → Wasm extraction

---

## Chunking Modes (Deprecated)

**Note:** Chunking functionality has been moved to the `riptide-html` crate. The chunking configuration in these endpoints is preserved for backward compatibility but has no effect on processing.

For chunking capabilities, use the `riptide-html` library directly:

```rust
use riptide_html::chunking::{ChunkingConfig, ChunkingMode};

let config = ChunkingConfig {
    mode: ChunkingMode::Sliding {
        token_max: 512,
        overlap: 128,
        preserve_sentences: true,
    },
    deterministic: true,
};
```

---

## Complete Examples

### Example 1: Simple Article Extraction (Wasm)

```bash
curl -X POST "https://api.riptide.dev/strategies/crawl?strategy=wasm" \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://blog.example.com/web-scraping-guide",
    "cache_mode": "read_write",
    "enable_metrics": true
  }'
```

### Example 2: Structured Data Extraction (CSS/JSON)

```bash
curl -X POST "https://api.riptide.dev/strategies/crawl?strategy=css_json" \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://example.com/product/123",
    "css_selectors": {
      "title": "h1.product-title",
      "price": ".price-display",
      "description": ".product-description",
      "availability": ".stock-status",
      "images": ".product-image img"
    },
    "cache_mode": "refresh"
  }'
```

### Example 3: Pattern Extraction (Regex)

```bash
curl -X POST "https://api.riptide.dev/strategies/crawl?strategy=regex" \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://example.com/events",
    "regex_patterns": [
      {
        "name": "event_dates",
        "pattern": "(January|February|March|April|May|June|July|August|September|October|November|December)\\s+\\d{1,2},\\s+\\d{4}",
        "field": "dates",
        "required": true
      },
      {
        "name": "event_times",
        "pattern": "\\d{1,2}:\\d{2}\\s*(AM|PM)",
        "field": "times",
        "required": false
      }
    ]
  }'
```

### Example 4: Auto-Detection with Metrics

```bash
curl -X POST "https://api.riptide.dev/strategies/crawl?strategy=auto" \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://github.com/user/repo/issues/42",
    "enable_metrics": true,
    "validate_schema": true,
    "cache_mode": "read_write"
  }'
```

**Response with Metrics:**
```json
{
  "success": true,
  "result": {
    "processed_content": {
      "content": "Issue description and comments...",
      "metadata": {
        "title": "Fix authentication bug",
        "author": "developer123",
        "date": "2024-01-15T14:30:00Z"
      }
    },
    "gate_decision": "raw",
    "quality_score": 0.94,
    "processing_time_ms": 1150,
    "performance_metrics": {
      "extraction_time_ms": 820,
      "validation_time_ms": 95,
      "cache_lookup_ms": 12,
      "total_time_ms": 1150,
      "content_size_bytes": 23456,
      "quality_checks_passed": 8,
      "quality_checks_failed": 0
    }
  },
  "stats": {
    "chunks_created": 1,
    "total_processing_time_ms": 1150,
    "extraction_strategy_used": "Wasm",
    "chunking_mode_used": "None",
    "cache_hit": false,
    "quality_score": 0.94
  }
}
```

---

## Performance Considerations

### Caching

All strategies support caching for improved performance:

- `cache_mode: "default"` - Read and write to cache
- `cache_mode: "bypass"` - Skip cache completely
- `cache_mode: "refresh"` - Force fresh extraction, update cache

Cache keys include:
- URL
- Cache mode
- Extraction strategy
- Strategy-specific configuration

### Strategy Performance Comparison

| Strategy | Avg Time | Use Case | Quality |
|----------|----------|----------|---------|
| Wasm | ~850ms | Articles, general content | High |
| CSS/JSON | ~950ms | Structured data, specific fields | Very High |
| Regex | ~780ms | Pattern matching, data extraction | Medium |
| LLM | ~3500ms | Complex understanding, reasoning | Very High |
| Auto | ~900ms | Unknown content types | High |

### Optimization Tips

1. **Use Wasm for articles**: Fastest and most reliable for article content
2. **Cache aggressively**: Enable caching for repeated URLs
3. **Pre-warm cache**: Use background jobs to cache frequently accessed content
4. **Batch requests**: Process multiple URLs in parallel
5. **Monitor metrics**: Enable metrics to identify bottlenecks

---

## Error Handling

All strategy endpoints return consistent error responses:

### Error Types

| Type | Status | Retryable | Description |
|------|--------|-----------|-------------|
| `invalid_request` | 400 | No | Invalid request parameters |
| `invalid_url` | 400 | No | Malformed or invalid URL |
| `fetch_error` | 502 | Yes | Failed to fetch URL |
| `timeout` | 504 | Yes | Request timeout exceeded |
| `pipeline_error` | 500 | Maybe | Processing pipeline error |
| `cache_error` | 500 | Yes | Cache operation failed |

### Retry Strategy

For retryable errors:
1. Exponential backoff: 1s, 2s, 4s, 8s
2. Maximum 3 retry attempts
3. Consider using `cache_mode: "refresh"` on retry
4. Check `/healthz` endpoint before retry

---

## Rate Limiting

Strategy endpoints are subject to rate limiting:

- **Default**: 100 requests/minute
- **Burst**: 20 requests
- **Headers**: `X-RateLimit-*` included in responses

### Rate Limit Headers

```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 85
X-RateLimit-Reset: 1705320000
```

---

## Best Practices

### 1. Choose the Right Strategy

- **Wasm**: General web content, articles, blogs
- **CSS/JSON**: E-commerce, structured data, specific fields
- **Regex**: Contacts, dates, numbers, patterns
- **Auto**: Unknown content types, mixed sources

### 2. Enable Metrics for Optimization

```json
{
  "enable_metrics": true,
  "validate_schema": true
}
```

Monitor `performance_metrics` to identify bottlenecks.

### 3. Use Appropriate Cache Modes

- Development: `"bypass"`
- Production: `"read_write"`
- Data refresh: `"refresh"`

### 4. Handle Errors Gracefully

Always check `success` field and handle errors:

```javascript
if (response.success) {
  // Process content
  const content = response.result.processed_content;
} else {
  // Handle error
  console.error(response.error.message);
  if (response.error.retryable) {
    // Implement retry logic
  }
}
```

### 5. Validate Quality Scores

Check `quality_score` before processing:

```javascript
if (response.result.quality_score < 0.7) {
  // Content quality may be low
  // Consider fallback strategy
}
```

---

## Integration Examples

### Node.js

```javascript
const axios = require('axios');

async function extractWithStrategies(url, strategy = 'auto') {
  try {
    const response = await axios.post(
      `https://api.riptide.dev/strategies/crawl?strategy=${strategy}`,
      {
        url: url,
        enable_metrics: true,
        cache_mode: 'read_write'
      }
    );

    if (response.data.success) {
      return response.data.result.processed_content;
    } else {
      throw new Error(response.data.error.message);
    }
  } catch (error) {
    console.error('Extraction failed:', error.message);
    throw error;
  }
}

// Usage
extractWithStrategies('https://example.com/article', 'wasm')
  .then(content => console.log(content))
  .catch(err => console.error(err));
```

### Python

```python
import requests

def extract_with_strategies(url, strategy='auto'):
    response = requests.post(
        f'https://api.riptide.dev/strategies/crawl?strategy={strategy}',
        json={
            'url': url,
            'enable_metrics': True,
            'cache_mode': 'read_write'
        }
    )

    data = response.json()

    if data['success']:
        return data['result']['processed_content']
    else:
        raise Exception(data['error']['message'])

# Usage
content = extract_with_strategies('https://example.com/article', 'wasm')
print(content)
```

### cURL

```bash
#!/bin/bash

URL="https://example.com/article"
STRATEGY="wasm"

curl -X POST "https://api.riptide.dev/strategies/crawl?strategy=${STRATEGY}" \
  -H "Content-Type: application/json" \
  -d "{
    \"url\": \"${URL}\",
    \"enable_metrics\": true,
    \"cache_mode\": \"read_write\"
  }" | jq '.result.processed_content'
```

---

## Related Documentation

- [REST API Overview](./rest-api.md)
- [Error Handling Guide](./error-handling.md)
- [Performance Optimization](./performance.md)
- [Streaming APIs](./streaming.md)
- [OpenAPI Specification](./openapi.yaml)
