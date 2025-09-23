# Session Management and Header Usage Guide

## Overview

The RipTide API provides session management capabilities for tracking requests, managing persistent state, and coordinating long-running operations. This guide covers session headers, state management, and coordination patterns.

## Session Headers

### X-Session-ID Header

The primary session identifier for request tracking and coordination:

```http
POST /crawl/stream
Content-Type: application/json
X-Session-ID: session-1640995200-abc123

{
  "urls": ["https://example.com"]
}
```

**Format**: `session-{timestamp}-{random}`

**Generation Example**:
```javascript
function generateSessionId() {
  const timestamp = Date.now();
  const random = Math.random().toString(36).substring(2, 8);
  return `session-${timestamp}-${random}`;
}

// Example: session-1640995200-abc123
```

### X-Request-ID Header

Unique identifier for individual requests within a session:

```http
POST /crawl
Content-Type: application/json
X-Session-ID: session-1640995200-abc123
X-Request-ID: req-1640995201-def456

{
  "urls": ["https://example.com"]
}
```

### X-Client-Info Header

Client identification and version information:

```http
POST /crawl
Content-Type: application/json
X-Session-ID: session-1640995200-abc123
X-Client-Info: RipTideClient/1.2.0 (JavaScript; Node.js/18.0.0)

{
  "urls": ["https://example.com"]
}
```

## Session Lifecycle

### Session Creation

Sessions are created implicitly with the first request containing a session ID:

```javascript
class RipTideSession {
  constructor() {
    this.sessionId = this.generateSessionId();
    this.requestCount = 0;
    this.startTime = Date.now();
    this.state = new Map();
  }

  generateSessionId() {
    const timestamp = Date.now();
    const random = Math.random().toString(36).substring(2, 8);
    return `session-${timestamp}-${random}`;
  }

  async makeRequest(endpoint, data) {
    this.requestCount++;

    const headers = {
      'Content-Type': 'application/json',
      'X-Session-ID': this.sessionId,
      'X-Request-ID': `req-${Date.now()}-${Math.random().toString(36).substring(2, 8)}`,
      'X-Client-Info': 'RipTideClient/1.2.0 (JavaScript; Browser)'
    };

    const response = await fetch(endpoint, {
      method: 'POST',
      headers,
      body: JSON.stringify(data)
    });

    return response;
  }
}
```

### Session State Management

The API maintains session state for coordination and optimization:

```javascript
// Session state structure
const sessionState = {
  sessionId: 'session-1640995200-abc123',
  startTime: 1640995200000,
  lastActivity: 1640995800000,
  requestCount: 15,
  totalUrls: 150,
  successfulUrls: 142,
  failedUrls: 8,
  cacheHits: 45,
  avgProcessingTime: 1250,
  currentStream: null, // Active streaming connection
  preferences: {
    concurrency: 3,
    cacheMode: 'read_write',
    extractMode: 'article'
  }
};
```

### Session Persistence

Sessions persist across requests and can be restored:

```javascript
async function restoreSession(sessionId) {
  const response = await fetch('/session/restore', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'X-Session-ID': sessionId
    },
    body: JSON.stringify({ sessionId })
  });

  if (response.ok) {
    const sessionData = await response.json();
    return sessionData;
  }

  return null;
}

// Usage
const existingSessionId = localStorage.getItem('riptide-session-id');
if (existingSessionId) {
  const restoredSession = await restoreSession(existingSessionId);
  if (restoredSession) {
    console.log('Session restored:', restoredSession);
  }
}
```

## Streaming Session Management

### NDJSON Stream Sessions

Streaming endpoints use sessions for progress tracking and reconnection:

```javascript
async function createStreamingSession(urls, options = {}) {
  const session = new RipTideSession();

  const response = await fetch('/crawl/stream', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'X-Session-ID': session.sessionId,
      'X-Buffer-Size': '512' // Optional buffer size hint
    },
    body: JSON.stringify({ urls, options })
  });

  if (!response.ok) {
    throw new Error(`Stream failed: ${response.status}`);
  }

  return { session, stream: response.body };
}

// Process streaming results
async function processStream(session, stream) {
  const reader = stream.getReader();
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
          await handleStreamEvent(session, event);
        } catch (error) {
          console.error('Failed to parse stream event:', error);
        }
      }
    }
  } finally {
    reader.releaseLock();
  }
}

async function handleStreamEvent(session, event) {
  session.state.set('lastEvent', event);

  switch (event.event) {
    case 'start':
      console.log(`Session ${session.sessionId} started with ${event.total_urls} URLs`);
      break;

    case 'progress':
      const progress = (event.completed / event.total) * 100;
      console.log(`Progress: ${progress.toFixed(1)}%`);
      break;

    case 'result':
      console.log(`Processed: ${event.url}`);
      break;

    case 'summary':
      console.log(`Session completed: ${event.successful}/${event.total_urls} successful`);
      break;

    case 'error':
      console.error(`Stream error: ${event.error.message}`);
      break;
  }
}
```

### Stream Reconnection

Handle stream interruptions with session-based reconnection:

```javascript
class ReconnectingStream {
  constructor(urls, options = {}) {
    this.urls = urls;
    this.options = options;
    this.session = new RipTideSession();
    this.processedUrls = new Set();
    this.maxReconnectAttempts = 3;
    this.reconnectDelay = 1000;
  }

  async start() {
    let attempt = 0;

    while (attempt < this.maxReconnectAttempts) {
      try {
        await this.createStream();
        return; // Successfully completed
      } catch (error) {
        attempt++;
        console.error(`Stream attempt ${attempt} failed:`, error);

        if (attempt < this.maxReconnectAttempts) {
          console.log(`Reconnecting in ${this.reconnectDelay}ms...`);
          await new Promise(resolve => setTimeout(resolve, this.reconnectDelay));
          this.reconnectDelay *= 2; // Exponential backoff
        }
      }
    }

    throw new Error('Max reconnection attempts exceeded');
  }

  async createStream() {
    // Filter out already processed URLs
    const remainingUrls = this.urls.filter(url => !this.processedUrls.has(url));

    if (remainingUrls.length === 0) {
      console.log('All URLs processed');
      return;
    }

    const response = await fetch('/crawl/stream', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-Session-ID': this.session.sessionId,
        'X-Resume-From': Array.from(this.processedUrls).join(',')
      },
      body: JSON.stringify({
        urls: remainingUrls,
        options: this.options
      })
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    await this.processStreamResponse(response.body);
  }

  async processStreamResponse(stream) {
    const reader = stream.getReader();
    const decoder = new TextDecoder();

    try {
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        const lines = decoder.decode(value).split('\n');

        for (const line of lines) {
          if (!line.trim()) continue;

          const event = JSON.parse(line);

          if (event.event === 'result' && event.url) {
            this.processedUrls.add(event.url);
          }

          await this.handleEvent(event);
        }
      }
    } finally {
      reader.releaseLock();
    }
  }

  async handleEvent(event) {
    // Handle events and update session state
    console.log('Event:', event);
  }
}

// Usage
const stream = new ReconnectingStream([
  'https://example.com/page1',
  'https://example.com/page2',
  'https://example.com/page3'
]);

await stream.start();
```

### WebSocket Session Management

WebSocket connections maintain persistent sessions:

```javascript
class WebSocketSession {
  constructor() {
    this.sessionId = this.generateSessionId();
    this.ws = null;
    this.messageQueue = [];
    this.isConnected = false;
    this.reconnectAttempts = 0;
    this.maxReconnectAttempts = 5;
  }

  async connect() {
    const wsUrl = `ws://localhost:8080/crawl/ws?session_id=${this.sessionId}`;

    return new Promise((resolve, reject) => {
      this.ws = new WebSocket(wsUrl);

      this.ws.onopen = () => {
        console.log(`WebSocket session ${this.sessionId} connected`);
        this.isConnected = true;
        this.reconnectAttempts = 0;

        // Send queued messages
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
        console.log(`WebSocket session ${this.sessionId} closed:`, event.code);
        this.isConnected = false;

        if (event.code !== 1000 && this.reconnectAttempts < this.maxReconnectAttempts) {
          this.attemptReconnect();
        }
      };

      this.ws.onerror = (error) => {
        console.error(`WebSocket session ${this.sessionId} error:`, error);
        reject(error);
      };
    });
  }

  async attemptReconnect() {
    this.reconnectAttempts++;
    const delay = Math.pow(2, this.reconnectAttempts) * 1000;

    console.log(`Reconnecting WebSocket session ${this.sessionId} in ${delay}ms (attempt ${this.reconnectAttempts})`);

    setTimeout(() => {
      this.connect().catch(error => {
        console.error('Reconnection failed:', error);
      });
    }, delay);
  }

  send(message) {
    const messageWithSession = {
      ...message,
      sessionId: this.sessionId,
      timestamp: Date.now()
    };

    if (this.isConnected && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(messageWithSession));
    } else {
      // Queue message for later delivery
      this.messageQueue.push(messageWithSession);
    }
  }

  flushMessageQueue() {
    while (this.messageQueue.length > 0 && this.isConnected) {
      const message = this.messageQueue.shift();
      this.ws.send(JSON.stringify(message));
    }
  }

  handleMessage(data) {
    if (data.sessionId !== this.sessionId) {
      console.warn('Received message for different session:', data.sessionId);
      return;
    }

    // Handle session-specific messages
    console.log('WebSocket message:', data);
  }

  generateSessionId() {
    const timestamp = Date.now();
    const random = Math.random().toString(36).substring(2, 8);
    return `ws-session-${timestamp}-${random}`;
  }
}

// Usage
const wsSession = new WebSocketSession();
await wsSession.connect();

// Send crawl request
wsSession.send({
  action: 'crawl',
  urls: ['https://example.com'],
  options: { concurrency: 3 }
});
```

## Session Analytics and Monitoring

### Session Metrics Collection

```javascript
class SessionAnalytics {
  constructor(sessionId) {
    this.sessionId = sessionId;
    this.metrics = {
      startTime: Date.now(),
      requestCount: 0,
      totalUrls: 0,
      successfulUrls: 0,
      failedUrls: 0,
      cacheHits: 0,
      totalProcessingTime: 0,
      errors: []
    };
  }

  recordRequest(urlCount) {
    this.metrics.requestCount++;
    this.metrics.totalUrls += urlCount;
  }

  recordResult(result) {
    if (result.document) {
      this.metrics.successfulUrls++;
    } else {
      this.metrics.failedUrls++;
      if (result.error) {
        this.metrics.errors.push({
          url: result.url,
          error: result.error,
          timestamp: Date.now()
        });
      }
    }

    if (result.from_cache) {
      this.metrics.cacheHits++;
    }

    this.metrics.totalProcessingTime += result.processing_time_ms || 0;
  }

  getSessionSummary() {
    const duration = Date.now() - this.metrics.startTime;
    const successRate = this.metrics.totalUrls > 0
      ? (this.metrics.successfulUrls / this.metrics.totalUrls) * 100
      : 0;
    const cacheHitRate = this.metrics.totalUrls > 0
      ? (this.metrics.cacheHits / this.metrics.totalUrls) * 100
      : 0;
    const avgProcessingTime = this.metrics.successfulUrls > 0
      ? this.metrics.totalProcessingTime / this.metrics.successfulUrls
      : 0;

    return {
      sessionId: this.sessionId,
      duration: duration,
      requestCount: this.metrics.requestCount,
      totalUrls: this.metrics.totalUrls,
      successfulUrls: this.metrics.successfulUrls,
      failedUrls: this.metrics.failedUrls,
      successRate: successRate,
      cacheHits: this.metrics.cacheHits,
      cacheHitRate: cacheHitRate,
      avgProcessingTime: avgProcessingTime,
      errorCount: this.metrics.errors.length,
      topErrors: this.getTopErrors()
    };
  }

  getTopErrors() {
    const errorCounts = {};

    this.metrics.errors.forEach(error => {
      const errorType = error.error.error_type || 'unknown';
      errorCounts[errorType] = (errorCounts[errorType] || 0) + 1;
    });

    return Object.entries(errorCounts)
      .sort((a, b) => b[1] - a[1])
      .slice(0, 5)
      .map(([type, count]) => ({ type, count }));
  }

  async sendAnalytics() {
    const summary = this.getSessionSummary();

    // Send to analytics service
    try {
      await fetch('/analytics/session', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'X-Session-ID': this.sessionId
        },
        body: JSON.stringify(summary)
      });
    } catch (error) {
      console.error('Failed to send session analytics:', error);
    }
  }
}

// Integration with session
class AnalyticsEnabledSession extends RipTideSession {
  constructor() {
    super();
    this.analytics = new SessionAnalytics(this.sessionId);
  }

  async crawl(urls, options = {}) {
    this.analytics.recordRequest(urls.length);

    const response = await this.makeRequest('/crawl', { urls, options });
    const result = await response.json();

    // Record results
    result.results.forEach(urlResult => {
      this.analytics.recordResult(urlResult);
    });

    return result;
  }

  async endSession() {
    await this.analytics.sendAnalytics();
    console.log('Session summary:', this.analytics.getSessionSummary());
  }
}
```

## Session Security

### Session Token Validation

```javascript
function validateSessionId(sessionId) {
  // Session ID format: session-{timestamp}-{random}
  const pattern = /^session-\d{13}-[a-z0-9]{6}$/;

  if (!pattern.test(sessionId)) {
    return { valid: false, reason: 'Invalid format' };
  }

  // Extract timestamp
  const timestampStr = sessionId.split('-')[1];
  const timestamp = parseInt(timestampStr);
  const age = Date.now() - timestamp;

  // Session expires after 24 hours
  const maxAge = 24 * 60 * 60 * 1000;

  if (age > maxAge) {
    return { valid: false, reason: 'Session expired' };
  }

  return { valid: true };
}

// Middleware for session validation
function sessionValidationMiddleware(req, res, next) {
  const sessionId = req.headers['x-session-id'];

  if (!sessionId) {
    return res.status(400).json({
      error: {
        type: 'validation_error',
        message: 'X-Session-ID header is required',
        retryable: false,
        status: 400
      }
    });
  }

  const validation = validateSessionId(sessionId);

  if (!validation.valid) {
    return res.status(400).json({
      error: {
        type: 'validation_error',
        message: `Invalid session ID: ${validation.reason}`,
        retryable: false,
        status: 400
      }
    });
  }

  req.sessionId = sessionId;
  next();
}
```

### Rate Limiting by Session

```javascript
class SessionRateLimiter {
  constructor() {
    this.sessionLimits = new Map();
    this.cleanupInterval = setInterval(() => this.cleanup(), 60000);
  }

  checkLimit(sessionId, endpoint) {
    const key = `${sessionId}:${endpoint}`;
    const now = Date.now();
    const windowSize = 60000; // 1 minute
    const limit = this.getLimitForEndpoint(endpoint);

    if (!this.sessionLimits.has(key)) {
      this.sessionLimits.set(key, []);
    }

    const requests = this.sessionLimits.get(key);

    // Remove old requests outside the window
    const validRequests = requests.filter(time => now - time < windowSize);
    this.sessionLimits.set(key, validRequests);

    if (validRequests.length >= limit) {
      return {
        allowed: false,
        resetTime: Math.min(...validRequests) + windowSize
      };
    }

    // Add current request
    validRequests.push(now);
    this.sessionLimits.set(key, validRequests);

    return {
      allowed: true,
      remaining: limit - validRequests.length
    };
  }

  getLimitForEndpoint(endpoint) {
    const limits = {
      '/crawl': 100,
      '/deepsearch': 20,
      '/render': 50,
      '/crawl/stream': 10
    };

    return limits[endpoint] || 60;
  }

  cleanup() {
    const now = Date.now();
    const maxAge = 24 * 60 * 60 * 1000; // 24 hours

    for (const [key, requests] of this.sessionLimits.entries()) {
      const validRequests = requests.filter(time => now - time < maxAge);

      if (validRequests.length === 0) {
        this.sessionLimits.delete(key);
      } else {
        this.sessionLimits.set(key, validRequests);
      }
    }
  }
}
```

This comprehensive session management guide covers all aspects of session handling, from basic session creation to advanced features like stream reconnection and analytics collection.