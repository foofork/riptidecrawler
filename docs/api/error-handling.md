# Error Handling and Response Patterns

## Overview

The RipTide API implements comprehensive error handling with consistent response formats, appropriate HTTP status codes, and detailed error information to facilitate debugging and proper client error handling.

## Error Response Format

All API errors follow a consistent JSON structure:

```json
{
  "error": {
    "type": "error_type_identifier",
    "message": "Human-readable error description",
    "retryable": true,
    "status": 400
  }
}
```

### Response Fields

- **type**: Machine-readable error identifier for programmatic handling
- **message**: Human-readable description for logging and debugging
- **retryable**: Boolean indicating if the request can be safely retried
- **status**: HTTP status code for the error

## Error Types and Status Codes

### Client Errors (4xx)

#### Validation Errors (400 Bad Request)

**Type**: `validation_error`

Occurs when request data is malformed or invalid.

```json
{
  "error": {
    "type": "validation_error",
    "message": "URL cannot be empty",
    "retryable": false,
    "status": 400
  }
}
```

**Common Scenarios:**
- Empty or missing required fields
- Invalid URL formats
- Out-of-range parameter values
- Malformed JSON in request body

**Example Validation Errors:**

```json
// Empty URLs array
{
  "error": {
    "type": "validation_error",
    "message": "URLs array cannot be empty",
    "retryable": false,
    "status": 400
  }
}

// Invalid concurrency value
{
  "error": {
    "type": "validation_error",
    "message": "Concurrency must be between 1 and 10",
    "retryable": false,
    "status": 400
  }
}

// Invalid cache mode
{
  "error": {
    "type": "validation_error",
    "message": "Invalid cache_mode. Must be one of: read_only, write_only, read_write, disabled",
    "retryable": false,
    "status": 400
  }
}
```

#### Invalid URL (400 Bad Request)

**Type**: `invalid_url`

Occurs when URLs fail parsing or validation.

```json
{
  "error": {
    "type": "invalid_url",
    "message": "Invalid URL: not-a-url - relative URL without a base",
    "retryable": false,
    "status": 400
  }
}
```

#### Authentication Error (401 Unauthorized)

**Type**: `authentication_error`

Occurs when API keys are missing or invalid.

```json
{
  "error": {
    "type": "authentication_error",
    "message": "SERPER_API_KEY environment variable not set",
    "retryable": false,
    "status": 401
  }
}
```

#### Not Found (404 Not Found)

**Type**: `not_found`

Occurs when requested resources don't exist.

```json
{
  "error": {
    "type": "not_found",
    "message": "The requested endpoint was not found",
    "retryable": false,
    "status": 404
  }
}
```

#### Request Timeout (408 Request Timeout)

**Type**: `timeout_error`

Occurs when operations exceed configured timeouts.

```json
{
  "error": {
    "type": "timeout_error",
    "message": "Operation timed out: http_request - Request took longer than 30 seconds",
    "retryable": true,
    "status": 408
  }
}
```

#### Payload Too Large (413 Payload Too Large)

**Type**: `payload_too_large`

Occurs when request size exceeds limits.

```json
{
  "error": {
    "type": "payload_too_large",
    "message": "Request payload too large: Maximum 100 URLs per request",
    "retryable": false,
    "status": 413
  }
}
```

#### Rate Limited (429 Too Many Requests)

**Type**: `rate_limited`

Occurs when rate limits are exceeded.

```json
{
  "error": {
    "type": "rate_limited",
    "message": "Rate limit exceeded: 100 requests per minute",
    "retryable": true,
    "status": 429
  }
}
```

**Response Headers:**
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1640995200
Retry-After: 60
```

### Server Errors (5xx)

#### Internal Server Error (500 Internal Server Error)

**Type**: `internal_error`

Generic server-side errors.

```json
{
  "error": {
    "type": "internal_error",
    "message": "Internal server error: Unexpected database connection failure",
    "retryable": true,
    "status": 500
  }
}
```

#### Extraction Error (500 Internal Server Error)

**Type**: `extraction_error`

WASM component or content extraction failures.

```json
{
  "error": {
    "type": "extraction_error",
    "message": "Content extraction failed: Trek extraction failed: Invalid HTML structure",
    "retryable": true,
    "status": 500
  }
}
```

#### Pipeline Error (500 Internal Server Error)

**Type**: `pipeline_error`

Pipeline orchestration failures.

```json
{
  "error": {
    "type": "pipeline_error",
    "message": "Pipeline execution failed: Failed to initialize gate component",
    "retryable": true,
    "status": 500
  }
}
```

#### Routing Error (500 Internal Server Error)

**Type**: `routing_error`

Content routing/gate decision failures.

```json
{
  "error": {
    "type": "routing_error",
    "message": "Content routing failed: Unable to determine processing strategy",
    "retryable": true,
    "status": 500
  }
}
```

#### Configuration Error (500 Internal Server Error)

**Type**: `config_error`

Configuration or environment setup issues.

```json
{
  "error": {
    "type": "config_error",
    "message": "Configuration error: Invalid Redis connection string",
    "retryable": false,
    "status": 500
  }
}
```

#### Fetch Error (502 Bad Gateway)

**Type**: `fetch_error`

External HTTP request failures.

```json
{
  "error": {
    "type": "fetch_error",
    "message": "Failed to fetch content from https://example.com: Connection refused",
    "retryable": true,
    "status": 502
  }
}
```

#### Service Unavailable (503 Service Unavailable)

**Type**: `dependency_error` or `cache_error`

Dependency service failures.

```json
{
  "error": {
    "type": "dependency_error",
    "message": "Dependency unavailable: redis - Connection timeout after 5 seconds",
    "retryable": true,
    "status": 503
  }
}

{
  "error": {
    "type": "cache_error",
    "message": "Cache operation failed: Redis server is down",
    "retryable": true,
    "status": 503
  }
}
```

## Partial Success Handling

For batch operations, the API may return partial success with detailed per-item results:

```json
{
  "total_urls": 3,
  "successful": 2,
  "failed": 1,
  "results": [
    {
      "url": "https://example.com/page1",
      "status": 200,
      "document": { /* extracted content */ },
      "error": null
    },
    {
      "url": "https://example.com/page2",
      "status": 200,
      "document": { /* extracted content */ },
      "error": null
    },
    {
      "url": "https://invalid-domain.invalid",
      "status": 0,
      "document": null,
      "error": {
        "error_type": "fetch_error",
        "message": "Failed to resolve domain: invalid-domain.invalid",
        "retryable": true
      }
    }
  ]
}
```

## Client Error Handling Patterns

### Basic Error Handling

```javascript
async function crawlUrls(urls, options = {}) {
  try {
    const response = await fetch('/crawl', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ urls, options })
    });

    const data = await response.json();

    if (!response.ok) {
      throw new ApiError(data.error);
    }

    return data;
  } catch (error) {
    console.error('Crawl request failed:', error);
    throw error;
  }
}

class ApiError extends Error {
  constructor(errorData) {
    super(errorData.message);
    this.type = errorData.type;
    this.retryable = errorData.retryable;
    this.status = errorData.status;
  }
}
```

### Retry Logic with Exponential Backoff

```javascript
async function crawlWithRetry(urls, options = {}, maxRetries = 3) {
  let lastError;

  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      return await crawlUrls(urls, options);
    } catch (error) {
      lastError = error;

      // Don't retry non-retryable errors
      if (error instanceof ApiError && !error.retryable) {
        throw error;
      }

      // Don't retry on last attempt
      if (attempt === maxRetries) {
        break;
      }

      // Exponential backoff: 1s, 2s, 4s, 8s...
      const delay = Math.pow(2, attempt - 1) * 1000;

      console.log(`Attempt ${attempt} failed, retrying in ${delay}ms...`);
      await new Promise(resolve => setTimeout(resolve, delay));
    }
  }

  throw lastError;
}
```

### Rate Limit Handling

```javascript
async function crawlWithRateLimit(urls, options = {}) {
  try {
    return await crawlUrls(urls, options);
  } catch (error) {
    if (error instanceof ApiError && error.type === 'rate_limited') {
      // Extract retry delay from error message or use default
      const retryAfter = extractRetryAfter(error.message) || 60;

      console.log(`Rate limited, waiting ${retryAfter} seconds...`);
      await new Promise(resolve => setTimeout(resolve, retryAfter * 1000));

      // Retry once after rate limit reset
      return await crawlUrls(urls, options);
    }

    throw error;
  }
}

function extractRetryAfter(message) {
  const match = message.match(/wait (\d+) seconds/);
  return match ? parseInt(match[1]) : null;
}
```

### Circuit Breaker Pattern

```javascript
class CircuitBreaker {
  constructor(threshold = 5, timeout = 60000) {
    this.threshold = threshold;
    this.timeout = timeout;
    this.failureCount = 0;
    this.lastFailureTime = null;
    this.state = 'CLOSED'; // CLOSED, OPEN, HALF_OPEN
  }

  async execute(operation) {
    if (this.state === 'OPEN') {
      if (Date.now() - this.lastFailureTime < this.timeout) {
        throw new Error('Circuit breaker is OPEN');
      }
      this.state = 'HALF_OPEN';
    }

    try {
      const result = await operation();
      this.onSuccess();
      return result;
    } catch (error) {
      this.onFailure();
      throw error;
    }
  }

  onSuccess() {
    this.failureCount = 0;
    this.state = 'CLOSED';
  }

  onFailure() {
    this.failureCount++;
    this.lastFailureTime = Date.now();

    if (this.failureCount >= this.threshold) {
      this.state = 'OPEN';
    }
  }
}

// Usage
const breaker = new CircuitBreaker(5, 60000);

async function resilientCrawl(urls, options) {
  return await breaker.execute(() => crawlUrls(urls, options));
}
```

## Streaming Error Handling

### NDJSON Stream Errors

```javascript
async function handleNdjsonStream(response) {
  const reader = response.body.getReader();
  const decoder = new TextDecoder();

  try {
    while (true) {
      const { done, value } = await reader.read();

      if (done) break;

      const lines = decoder.decode(value).split('\n');

      for (const line of lines) {
        if (!line.trim()) continue;

        try {
          const event = JSON.parse(line);

          if (event.error) {
            console.error('Stream error:', event.error);
            // Handle individual errors
            continue;
          }

          handleStreamEvent(event);
        } catch (parseError) {
          console.error('Failed to parse stream event:', parseError);
        }
      }
    }
  } finally {
    reader.releaseLock();
  }
}
```

### WebSocket Error Handling

```javascript
function createWebSocketConnection(url) {
  const ws = new WebSocket(url);

  ws.onopen = () => {
    console.log('WebSocket connected');
  };

  ws.onmessage = (event) => {
    try {
      const data = JSON.parse(event.data);

      if (data.error) {
        console.error('WebSocket error:', data.error);
        handleWebSocketError(data.error);
        return;
      }

      handleWebSocketMessage(data);
    } catch (error) {
      console.error('Failed to parse WebSocket message:', error);
    }
  };

  ws.onerror = (error) => {
    console.error('WebSocket connection error:', error);
  };

  ws.onclose = (event) => {
    console.log('WebSocket closed:', event.code, event.reason);

    if (event.code !== 1000) {
      // Unexpected close, attempt reconnection
      setTimeout(() => createWebSocketConnection(url), 5000);
    }
  };

  return ws;
}
```

## Error Monitoring and Alerting

### Error Tracking

```javascript
function trackError(error, context = {}) {
  const errorData = {
    type: error.type || 'unknown',
    message: error.message,
    stack: error.stack,
    retryable: error.retryable,
    status: error.status,
    timestamp: new Date().toISOString(),
    context
  };

  // Send to monitoring service
  analytics.track('api_error', errorData);

  // Log locally
  console.error('API Error:', errorData);
}

// Usage in error handlers
try {
  const result = await crawlUrls(urls, options);
} catch (error) {
  trackError(error, {
    operation: 'crawl',
    urls: urls.length,
    options
  });
  throw error;
}
```

### Health Check Integration

```javascript
async function performHealthCheck() {
  try {
    const response = await fetch('/healthz');
    const health = await response.json();

    if (health.status === 'unhealthy') {
      // Alert on unhealthy dependencies
      alerting.send('API dependencies unhealthy', {
        dependencies: health.dependencies,
        timestamp: health.timestamp
      });
    }

    return health;
  } catch (error) {
    // Alert on health check failure
    alerting.send('Health check failed', {
      error: error.message,
      timestamp: new Date().toISOString()
    });

    throw error;
  }
}
```

## Best Practices

### 1. Error Classification

Always check the error type to determine appropriate handling:

```javascript
function handleApiError(error) {
  switch (error.type) {
    case 'validation_error':
    case 'invalid_url':
      // Fix request and don't retry
      logUserError(error);
      break;

    case 'rate_limited':
      // Implement backoff and retry
      return scheduleRetry(error);

    case 'timeout_error':
    case 'fetch_error':
    case 'dependency_error':
      // Retry with exponential backoff
      return retryWithBackoff(error);

    case 'config_error':
      // Don't retry, alert operations
      alertOps(error);
      break;

    default:
      // Unknown error, log and alert
      logError(error);
      alertOps(error);
  }
}
```

### 2. Graceful Degradation

Implement fallbacks for non-critical operations:

```javascript
async function crawlWithFallback(urls, options = {}) {
  try {
    return await crawlUrls(urls, options);
  } catch (error) {
    if (error.type === 'extraction_error') {
      // Fallback to simplified extraction
      return await crawlUrls(urls, {
        ...options,
        extract_mode: 'metadata'
      });
    }

    throw error;
  }
}
```

### 3. User-Friendly Error Messages

Transform technical errors into user-friendly messages:

```javascript
function getUserFriendlyMessage(error) {
  const errorMessages = {
    'validation_error': 'Please check your request parameters and try again.',
    'invalid_url': 'One or more URLs are invalid. Please verify the URLs and try again.',
    'rate_limited': 'Too many requests. Please wait a moment and try again.',
    'timeout_error': 'The request timed out. Please try again with a smaller batch.',
    'fetch_error': 'Unable to access the requested website. Please check the URL.',
    'dependency_error': 'Our service is temporarily unavailable. Please try again later.',
    'extraction_error': 'Unable to extract content from the website. Please try again.'
  };

  return errorMessages[error.type] || 'An unexpected error occurred. Please try again.';
}
```

This comprehensive error handling guide ensures robust client applications and proper error resolution workflows.