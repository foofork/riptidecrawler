# Streaming API Documentation

## Overview

The RipTide API provides comprehensive streaming capabilities for real-time processing of web crawling and content extraction operations. The streaming infrastructure supports multiple protocols with features including dynamic buffer management, backpressure handling, and connection health monitoring.

## Supported Protocols

### NDJSON (Newline Delimited JSON)
- **Endpoint**: `/crawl/stream`, `/deepsearch/stream`
- **Content-Type**: `application/x-ndjson`
- **Best For**: Batch processing, programmatic consumption
- **Buffer Size**: 256 events (configurable)

### Server-Sent Events (SSE)
- **Endpoint**: `/crawl/sse`
- **Content-Type**: `text/event-stream`
- **Best For**: Browser integration, automatic reconnection
- **Keep-Alive**: 30-second intervals

### WebSocket
- **Endpoint**: `/crawl/ws`
- **Protocol**: WebSocket (bidirectional)
- **Best For**: Real-time interaction, control messages
- **Ping Interval**: 30-second heartbeat

## NDJSON Streaming

### Basic Usage

```bash
curl -X POST 'http://localhost:8080/crawl/stream' \
  -H 'Content-Type: application/json' \
  -H 'X-Session-ID: session-1640995200-abc123' \
  -H 'X-Buffer-Size: 512' \
  -d '{
    "urls": [
      "https://example.com/page1",
      "https://example.com/page2",
      "https://example.com/page3"
    ],
    "options": {
      "concurrency": 3,
      "cache_mode": "read_write"
    }
  }' \
  --no-buffer
```

### JavaScript Implementation

```javascript
async function streamCrawlResults(urls, options = {}) {
  const sessionId = `session-${Date.now()}-${Math.random().toString(36).substring(2, 8)}`;

  const response = await fetch('/crawl/stream', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'X-Session-ID': sessionId,
      'X-Buffer-Size': '256'
    },
    body: JSON.stringify({ urls, options })
  });

  if (!response.ok) {
    throw new Error(`Stream failed: ${response.status}`);
  }

  const reader = response.body.getReader();
  const decoder = new TextDecoder();

  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;

      const chunk = decoder.decode(value, { stream: true });
      const lines = chunk.split('\n');

      for (const line of lines) {
        if (!line.trim()) continue;

        try {
          const event = JSON.parse(line);
          await handleStreamEvent(event);
        } catch (error) {
          console.error('Failed to parse event:', error, 'Line:', line);
        }
      }
    }
  } finally {
    reader.releaseLock();
  }
}

async function handleStreamEvent(event) {
  switch (event.event) {
    case 'start':
      console.log(`Starting crawl of ${event.total_urls} URLs`);
      updateProgressBar(0, event.total_urls);
      break;

    case 'progress':
      console.log(`Progress: ${event.completed}/${event.total}`);
      updateProgressBar(event.completed, event.total);
      break;

    case 'result':
      console.log(`Processed: ${event.url} (${event.status})`);
      displayResult(event);
      break;

    case 'summary':
      console.log(`Completed: ${event.successful}/${event.total_urls} successful`);
      displaySummary(event);
      break;

    case 'error':
      console.error(`Stream error: ${event.error.message}`);
      handleStreamError(event.error);
      break;

    case 'ping':
      console.debug('Received ping from server');
      break;

    default:
      console.warn('Unknown event type:', event.event);
  }
}
```

### Python Implementation

```python
import requests
import json
import time

def stream_crawl_results(urls, options=None):
    session_id = f"session-{int(time.time())}-{random.randint(100000, 999999)}"

    headers = {
        'Content-Type': 'application/json',
        'X-Session-ID': session_id,
        'X-Buffer-Size': '256'
    }

    payload = {
        'urls': urls,
        'options': options or {}
    }

    response = requests.post(
        'http://localhost:8080/crawl/stream',
        headers=headers,
        json=payload,
        stream=True
    )

    response.raise_for_status()

    for line in response.iter_lines(decode_unicode=True):
        if not line.strip():
            continue

        try:
            event = json.loads(line)
            handle_stream_event(event)
        except json.JSONDecodeError as e:
            print(f"Failed to parse event: {e}")
            print(f"Line: {line}")

def handle_stream_event(event):
    event_type = event.get('event')

    if event_type == 'start':
        print(f"Starting crawl of {event['total_urls']} URLs")
    elif event_type == 'progress':
        progress = (event['completed'] / event['total']) * 100
        print(f"Progress: {progress:.1f}% ({event['completed']}/{event['total']})")
    elif event_type == 'result':
        print(f"Processed: {event['url']} - Status: {event['status']}")
        if event.get('document'):
            print(f"  Title: {event['document'].get('title', 'N/A')}")
    elif event_type == 'summary':
        print(f"Completed: {event['successful']}/{event['total_urls']} successful")
        print(f"Total time: {event['total_time_ms']}ms")
    elif event_type == 'error':
        print(f"Stream error: {event['error']['message']}")

# Usage
urls = [
    'https://example.com/page1',
    'https://example.com/page2',
    'https://example.com/page3'
]

options = {
    'concurrency': 3,
    'cache_mode': 'read_write',
    'extract_mode': 'article'
}

stream_crawl_results(urls, options)
```

## Stream Event Types

### Start Event

Sent when streaming begins:

```json
{
  "event": "start",
  "total_urls": 100,
  "session_id": "session-1640995200-abc123",
  "timestamp": "2024-01-15T10:30:00Z",
  "options": {
    "concurrency": 3,
    "cache_mode": "read_write"
  }
}
```

### Progress Event

Sent periodically during processing:

```json
{
  "event": "progress",
  "completed": 25,
  "total": 100,
  "url": "https://example.com/current-page",
  "timestamp": "2024-01-15T10:30:15Z",
  "phase": "extraction",
  "estimated_completion": "2024-01-15T10:32:00Z"
}
```

### Result Event

Sent for each completed URL:

```json
{
  "event": "result",
  "url": "https://example.com/page1",
  "status": 200,
  "from_cache": false,
  "gate_decision": "raw",
  "quality_score": 0.85,
  "processing_time_ms": 1250,
  "document": {
    "url": "https://example.com/page1",
    "title": "Page Title",
    "markdown": "# Page Title\n\nContent...",
    "text": "Page Title. Content...",
    "word_count": 1200,
    "reading_time": 5,
    "quality_score": 85
  },
  "error": null,
  "cache_key": "crawl:v1:example.com:page1:7d2a8c9b",
  "timestamp": "2024-01-15T10:30:10Z"
}
```

### Summary Event

Sent when streaming completes:

```json
{
  "event": "summary",
  "total_urls": 100,
  "successful": 95,
  "failed": 5,
  "from_cache": 35,
  "total_time_ms": 45000,
  "avg_processing_time_ms": 450,
  "cache_hit_rate": 0.35,
  "gate_decisions": {
    "raw": 60,
    "probes_first": 25,
    "headless": 10,
    "cached": 35
  },
  "timestamp": "2024-01-15T10:32:00Z"
}
```

### Error Event

Sent when errors occur:

```json
{
  "event": "error",
  "url": "https://example.com/failed-page",
  "error": {
    "type": "fetch_error",
    "message": "Connection timeout after 30 seconds",
    "retryable": true,
    "timestamp": "2024-01-15T10:30:45Z"
  }
}
```

### Ping Event

Keep-alive heartbeat:

```json
{
  "event": "ping",
  "timestamp": "2024-01-15T10:31:00Z",
  "session_id": "session-1640995200-abc123"
}
```

## Deep Search Streaming

### NDJSON Deep Search Stream

The `/deepsearch/stream` endpoint provides streaming for web search operations:

```javascript
async function streamDeepSearch(query, options = {}) {
  const sessionId = `search-${Date.now()}-${Math.random().toString(36).substring(2, 8)}`;

  const response = await fetch('/deepsearch/stream', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'X-Session-ID': sessionId
    },
    body: JSON.stringify({
      query,
      limit: options.limit || 10,
      include_content: options.include_content !== false,
      crawl_options: options.crawl_options || {}
    })
  });

  const reader = response.body.getReader();
  const decoder = new TextDecoder();

  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;

      const chunk = decoder.decode(value, { stream: true });
      const lines = chunk.split('\n');

      for (const line of lines) {
        if (!line.trim()) continue;

        try {
          const event = JSON.parse(line);
          await handleDeepSearchEvent(event);
        } catch (error) {
          console.error('Failed to parse event:', error);
        }
      }
    }
  } finally {
    reader.releaseLock();
  }
}

async function handleDeepSearchEvent(event) {
  switch (event.event) {
    case 'search_start':
      console.log(`Starting search for: "${event.query}"`);
      break;

    case 'search_results':
      console.log(`Found ${event.urls_found} URLs from search`);
      displaySearchResults(event.results);
      break;

    case 'crawl_start':
      console.log(`Starting content extraction for ${event.total_urls} URLs`);
      break;

    case 'search_result':
      console.log(`Extracted content for: ${event.url}`);
      displaySearchResult(event);
      break;

    case 'search_summary':
      console.log(`Search completed: ${event.urls_crawled}/${event.urls_found} URLs processed`);
      break;

    default:
      // Handle standard crawl events
      await handleStreamEvent(event);
  }
}
```

### Deep Search Event Types

#### Search Start Event

```json
{
  "event": "search_start",
  "query": "machine learning best practices",
  "limit": 10,
  "timestamp": "2024-01-15T10:30:00Z"
}
```

#### Search Results Event

```json
{
  "event": "search_results",
  "query": "machine learning best practices",
  "urls_found": 10,
  "results": [
    {
      "url": "https://example.com/ml-guide",
      "rank": 1,
      "search_title": "ML Best Practices Guide",
      "search_snippet": "Comprehensive guide to ML..."
    }
  ],
  "timestamp": "2024-01-15T10:30:05Z"
}
```

#### Search Result Event

```json
{
  "event": "search_result",
  "url": "https://example.com/ml-guide",
  "rank": 1,
  "search_title": "ML Best Practices Guide",
  "search_snippet": "Comprehensive guide to ML...",
  "content": {
    "title": "Machine Learning Best Practices",
    "markdown": "# ML Best Practices\n\n...",
    "word_count": 2500
  },
  "crawl_result": {
    "status": 200,
    "processing_time_ms": 1800,
    "quality_score": 0.92
  },
  "timestamp": "2024-01-15T10:30:12Z"
}
```

## Server-Sent Events (SSE)

### Browser Implementation

```html
<!DOCTYPE html>
<html>
<head>
    <title>RipTide SSE Example</title>
</head>
<body>
    <div id="status"></div>
    <div id="progress"></div>
    <div id="results"></div>

    <script>
    async function startSSECrawl(urls, options = {}) {
        const sessionId = `session-${Date.now()}-${Math.random().toString(36).substring(2, 8)}`;

        // First, initiate the crawl
        const response = await fetch('/crawl/sse', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-Session-ID': sessionId
            },
            body: JSON.stringify({ urls, options })
        });

        if (!response.ok) {
            throw new Error(`SSE failed: ${response.status}`);
        }

        // Create EventSource for the stream
        const eventSource = new EventSource(`/crawl/sse?session_id=${sessionId}`);

        eventSource.onopen = () => {
            console.log('SSE connection opened');
            document.getElementById('status').textContent = 'Connected';
        };

        eventSource.addEventListener('start', (event) => {
            const data = JSON.parse(event.data);
            console.log('Crawl started:', data);
            document.getElementById('status').textContent = `Processing ${data.total_urls} URLs`;
        });

        eventSource.addEventListener('progress', (event) => {
            const data = JSON.parse(event.data);
            const progress = (data.completed / data.total) * 100;
            document.getElementById('progress').innerHTML = `
                <div style="width: 100%; background: #f0f0f0;">
                    <div style="width: ${progress}%; background: #4caf50; height: 20px;"></div>
                </div>
                <p>Progress: ${data.completed}/${data.total} (${progress.toFixed(1)}%)</p>
            `;
        });

        eventSource.addEventListener('result', (event) => {
            const data = JSON.parse(event.data);
            const resultDiv = document.createElement('div');
            resultDiv.innerHTML = `
                <div style="border: 1px solid #ccc; margin: 10px; padding: 10px;">
                    <h3>${data.document?.title || data.url}</h3>
                    <p><strong>URL:</strong> ${data.url}</p>
                    <p><strong>Status:</strong> ${data.status}</p>
                    <p><strong>Processing Time:</strong> ${data.processing_time_ms}ms</p>
                    <p><strong>Quality Score:</strong> ${(data.quality_score * 100).toFixed(1)}%</p>
                    ${data.document?.text ? `<p><strong>Preview:</strong> ${data.document.text.substring(0, 200)}...</p>` : ''}
                </div>
            `;
            document.getElementById('results').appendChild(resultDiv);
        });

        eventSource.addEventListener('summary', (event) => {
            const data = JSON.parse(event.data);
            document.getElementById('status').textContent =
                `Completed: ${data.successful}/${data.total_urls} successful (${data.total_time_ms}ms)`;
            eventSource.close();
        });

        eventSource.addEventListener('error', (event) => {
            const data = JSON.parse(event.data);
            console.error('SSE error:', data);
            document.getElementById('status').textContent = `Error: ${data.error.message}`;
        });

        eventSource.onerror = (error) => {
            console.error('EventSource failed:', error);
            document.getElementById('status').textContent = 'Connection failed';
            eventSource.close();
        };

        return eventSource;
    }

    // Example usage
    const urls = [
        'https://example.com/page1',
        'https://example.com/page2',
        'https://example.com/page3'
    ];

    const options = {
        concurrency: 3,
        cache_mode: 'read_write'
    };

    startSSECrawl(urls, options).catch(console.error);
    </script>
</body>
</html>
```

## WebSocket Streaming

### JavaScript WebSocket Client

```javascript
class RipTideWebSocketClient {
    constructor(baseUrl = 'ws://localhost:8080') {
        this.baseUrl = baseUrl;
        this.ws = null;
        this.sessionId = null;
        this.messageQueue = [];
        this.isConnected = false;
        this.reconnectAttempts = 0;
        this.maxReconnectAttempts = 5;
        this.eventHandlers = {};
    }

    async connect() {
        this.sessionId = `ws-session-${Date.now()}-${Math.random().toString(36).substring(2, 8)}`;
        const wsUrl = `${this.baseUrl}/crawl/ws?session_id=${this.sessionId}`;

        return new Promise((resolve, reject) => {
            this.ws = new WebSocket(wsUrl);

            this.ws.onopen = () => {
                console.log(`WebSocket connected: ${this.sessionId}`);
                this.isConnected = true;
                this.reconnectAttempts = 0;
                this.flushMessageQueue();
                resolve();
            };

            this.ws.onmessage = (event) => {
                try {
                    const data = JSON.parse(event.data);
                    this.handleMessage(data);
                } catch (error) {
                    console.error('Failed to parse WebSocket message:', error);
                }
            };

            this.ws.onclose = (event) => {
                console.log(`WebSocket closed: ${event.code} - ${event.reason}`);
                this.isConnected = false;

                if (event.code !== 1000 && this.reconnectAttempts < this.maxReconnectAttempts) {
                    this.attemptReconnect();
                }
            };

            this.ws.onerror = (error) => {
                console.error('WebSocket error:', error);
                reject(error);
            };
        });
    }

    attemptReconnect() {
        this.reconnectAttempts++;
        const delay = Math.pow(2, this.reconnectAttempts) * 1000;

        console.log(`Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts})`);

        setTimeout(() => {
            this.connect().catch(error => {
                console.error('Reconnection failed:', error);
            });
        }, delay);
    }

    send(message) {
        const messageWithMeta = {
            ...message,
            sessionId: this.sessionId,
            timestamp: Date.now()
        };

        if (this.isConnected && this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify(messageWithMeta));
        } else {
            this.messageQueue.push(messageWithMeta);
        }
    }

    flushMessageQueue() {
        while (this.messageQueue.length > 0 && this.isConnected) {
            const message = this.messageQueue.shift();
            this.ws.send(JSON.stringify(message));
        }
    }

    handleMessage(data) {
        if (data.sessionId && data.sessionId !== this.sessionId) {
            console.warn('Received message for different session:', data.sessionId);
            return;
        }

        const eventType = data.event || data.action;
        if (this.eventHandlers[eventType]) {
            this.eventHandlers[eventType](data);
        } else if (this.eventHandlers['*']) {
            this.eventHandlers['*'](data);
        }
    }

    on(event, handler) {
        this.eventHandlers[event] = handler;
    }

    off(event) {
        delete this.eventHandlers[event];
    }

    async crawl(urls, options = {}) {
        return new Promise((resolve, reject) => {
            const requestId = `req-${Date.now()}-${Math.random().toString(36).substring(2, 8)}`;
            const results = [];
            let summary = null;

            // Set up event handlers for this request
            this.on('result', (data) => {
                if (data.requestId === requestId) {
                    results.push(data);
                }
            });

            this.on('summary', (data) => {
                if (data.requestId === requestId) {
                    summary = data;
                    resolve({ results, summary });
                }
            });

            this.on('error', (data) => {
                if (data.requestId === requestId) {
                    reject(new Error(data.error.message));
                }
            });

            // Send crawl request
            this.send({
                action: 'crawl',
                requestId,
                urls,
                options
            });
        });
    }

    disconnect() {
        if (this.ws) {
            this.ws.close(1000, 'Client disconnect');
        }
    }
}

// Usage example
async function webSocketExample() {
    const client = new RipTideWebSocketClient();

    // Set up event handlers
    client.on('start', (data) => {
        console.log(`Crawl started: ${data.total_urls} URLs`);
    });

    client.on('progress', (data) => {
        const progress = (data.completed / data.total) * 100;
        console.log(`Progress: ${progress.toFixed(1)}%`);
    });

    client.on('result', (data) => {
        console.log(`Processed: ${data.url} (${data.status})`);
    });

    client.on('ping', (data) => {
        console.log('Received ping from server');
    });

    try {
        await client.connect();

        const result = await client.crawl([
            'https://example.com/page1',
            'https://example.com/page2',
            'https://example.com/page3'
        ], {
            concurrency: 3,
            cache_mode: 'read_write'
        });

        console.log('Crawl completed:', result.summary);
    } catch (error) {
        console.error('WebSocket crawl failed:', error);
    } finally {
        client.disconnect();
    }
}
```

## Buffer Management and Backpressure

### Buffer Configuration

The streaming API implements dynamic buffer management with configurable sizes:

```http
POST /crawl/stream
Content-Type: application/json
X-Session-ID: session-1640995200-abc123
X-Buffer-Size: 512
X-Buffer-Timeout: 5000

{
  "urls": ["https://example.com"],
  "options": {
    "concurrency": 3
  }
}
```

**Buffer Headers:**
- `X-Buffer-Size`: Maximum events per buffer (64-1024, default: 256)
- `X-Buffer-Timeout`: Buffer flush timeout in milliseconds (1000-10000, default: 2000)

### Backpressure Handling

The API implements several backpressure strategies:

1. **Dynamic Buffer Sizing**: Adjusts buffer size based on processing speed
2. **Flow Control**: Pauses processing when buffers are full
3. **Connection Throttling**: Limits concurrent streaming connections
4. **Graceful Degradation**: Drops non-essential events under load

```javascript
// Client-side backpressure detection
class BackpressureDetector {
    constructor() {
        this.eventTimestamps = [];
        this.bufferWarningThreshold = 0.8;
        this.bufferCriticalThreshold = 0.95;
    }

    recordEvent(event) {
        this.eventTimestamps.push(Date.now());

        // Keep only last 100 events for analysis
        if (this.eventTimestamps.length > 100) {
            this.eventTimestamps.shift();
        }

        this.checkBackpressure(event);
    }

    checkBackpressure(event) {
        if (event.buffer_info) {
            const usage = event.buffer_info.used / event.buffer_info.capacity;

            if (usage > this.bufferCriticalThreshold) {
                console.warn('Critical backpressure detected, consider reducing request rate');
                this.onBackpressure('critical', event.buffer_info);
            } else if (usage > this.bufferWarningThreshold) {
                console.warn('Buffer pressure detected');
                this.onBackpressure('warning', event.buffer_info);
            }
        }

        // Detect event processing delays
        if (this.eventTimestamps.length >= 10) {
            const recent = this.eventTimestamps.slice(-10);
            const intervals = recent.slice(1).map((time, i) => time - recent[i]);
            const avgInterval = intervals.reduce((a, b) => a + b, 0) / intervals.length;

            if (avgInterval > 5000) { // Events arriving slower than every 5 seconds
                console.warn('Slow event processing detected');
                this.onBackpressure('slow', { avgInterval });
            }
        }
    }

    onBackpressure(level, info) {
        // Implement backpressure handling strategies
        switch (level) {
            case 'critical':
                // Pause new requests, increase buffer size
                break;
            case 'warning':
                // Reduce request concurrency
                break;
            case 'slow':
                // Check network/processing issues
                break;
        }
    }
}
```

## Error Handling and Recovery

### Stream Reconnection

```javascript
class ReconnectingStreamClient {
    constructor(maxRetries = 3, baseDelay = 1000) {
        this.maxRetries = maxRetries;
        this.baseDelay = baseDelay;
        this.currentRetry = 0;
    }

    async streamWithRetry(endpoint, payload, options = {}) {
        while (this.currentRetry <= this.maxRetries) {
            try {
                await this.createStream(endpoint, payload, options);
                this.currentRetry = 0; // Reset on success
                return;
            } catch (error) {
                console.error(`Stream attempt ${this.currentRetry + 1} failed:`, error);

                if (this.currentRetry >= this.maxRetries) {
                    throw new Error(`Max retries (${this.maxRetries}) exceeded`);
                }

                const delay = this.baseDelay * Math.pow(2, this.currentRetry);
                console.log(`Retrying in ${delay}ms...`);

                await new Promise(resolve => setTimeout(resolve, delay));
                this.currentRetry++;
            }
        }
    }

    async createStream(endpoint, payload, options) {
        const response = await fetch(endpoint, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-Session-ID': options.sessionId || this.generateSessionId(),
                ...options.headers
            },
            body: JSON.stringify(payload)
        });

        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }

        await this.processStream(response.body);
    }

    async processStream(stream) {
        const reader = stream.getReader();
        const decoder = new TextDecoder();

        try {
            while (true) {
                const { done, value } = await reader.read();
                if (done) break;

                const chunk = decoder.decode(value, { stream: true });
                this.processChunk(chunk);
            }
        } finally {
            reader.releaseLock();
        }
    }

    processChunk(chunk) {
        const lines = chunk.split('\n');

        for (const line of lines) {
            if (!line.trim()) continue;

            try {
                const event = JSON.parse(line);
                this.handleEvent(event);
            } catch (error) {
                console.error('Failed to parse event:', error);
            }
        }
    }

    handleEvent(event) {
        // Override in subclass
        console.log('Event:', event);
    }

    generateSessionId() {
        const timestamp = Date.now();
        const random = Math.random().toString(36).substring(2, 8);
        return `session-${timestamp}-${random}`;
    }
}
```

## Performance Monitoring

### Stream Metrics Collection

```javascript
class StreamMetrics {
    constructor() {
        this.metrics = {
            startTime: null,
            endTime: null,
            totalEvents: 0,
            eventsByType: {},
            processingTimes: [],
            errors: [],
            bufferStats: {
                maxUsage: 0,
                avgUsage: 0,
                flushes: 0
            }
        };
    }

    recordEvent(event) {
        if (!this.metrics.startTime) {
            this.metrics.startTime = Date.now();
        }

        this.metrics.totalEvents++;

        const eventType = event.event || 'unknown';
        this.metrics.eventsByType[eventType] = (this.metrics.eventsByType[eventType] || 0) + 1;

        if (event.processing_time_ms) {
            this.metrics.processingTimes.push(event.processing_time_ms);
        }

        if (event.error) {
            this.metrics.errors.push({
                type: event.error.type,
                message: event.error.message,
                timestamp: Date.now()
            });
        }

        if (event.buffer_info) {
            const usage = event.buffer_info.used / event.buffer_info.capacity;
            this.metrics.bufferStats.maxUsage = Math.max(this.metrics.bufferStats.maxUsage, usage);
            this.metrics.bufferStats.flushes++;
        }

        if (event.event === 'summary') {
            this.metrics.endTime = Date.now();
        }
    }

    getMetrics() {
        const duration = (this.metrics.endTime || Date.now()) - (this.metrics.startTime || Date.now());
        const avgProcessingTime = this.metrics.processingTimes.length > 0
            ? this.metrics.processingTimes.reduce((a, b) => a + b, 0) / this.metrics.processingTimes.length
            : 0;

        return {
            duration,
            totalEvents: this.metrics.totalEvents,
            eventsPerSecond: this.metrics.totalEvents / (duration / 1000),
            eventsByType: this.metrics.eventsByType,
            avgProcessingTime,
            errorRate: this.metrics.errors.length / this.metrics.totalEvents,
            errors: this.metrics.errors,
            bufferStats: this.metrics.bufferStats
        };
    }

    reset() {
        this.metrics = {
            startTime: null,
            endTime: null,
            totalEvents: 0,
            eventsByType: {},
            processingTimes: [],
            errors: [],
            bufferStats: {
                maxUsage: 0,
                avgUsage: 0,
                flushes: 0
            }
        };
    }
}

// Usage with stream processing
const metrics = new StreamMetrics();

async function monitoredStreamProcessing(urls, options) {
    try {
        await streamCrawlResults(urls, options);
    } finally {
        const results = metrics.getMetrics();
        console.log('Stream performance metrics:', results);

        // Send to monitoring system
        sendMetricsToMonitoring(results);
    }
}

// Override event handler to collect metrics
const originalHandleStreamEvent = handleStreamEvent;
handleStreamEvent = function(event) {
    metrics.recordEvent(event);
    return originalHandleStreamEvent(event);
};
```

This comprehensive streaming documentation covers all protocols, implementation patterns, error handling, and performance monitoring for the RipTide API streaming capabilities.