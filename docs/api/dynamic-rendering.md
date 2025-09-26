# Dynamic Rendering API Guide

## Overview

The RipTide API provides advanced dynamic rendering capabilities for JavaScript-heavy websites, single-page applications (SPAs), and complex web content that requires browser execution. This guide covers the enhanced `/render` endpoint, configuration options, and integration patterns.

## Key Features

- **Multiple Rendering Modes**: Static, Dynamic, PDF, and Adaptive
- **Stealth Capabilities**: Anti-detection measures for bot-resistant sites
- **Custom Actions**: Click, scroll, input, and navigation automation
- **Wait Conditions**: Smart waiting for content to load
- **Artifact Capture**: Screenshots and MHTML archives
- **Performance Monitoring**: Detailed timing and resource tracking

## Rendering Modes

### Static Mode

Fast HTTP-only rendering without JavaScript execution:

```json
{
  "url": "https://example.com/static-page",
  "mode": "static",
  "output_format": "markdown"
}
```

**Use Cases:**
- Traditional server-rendered pages
- Static HTML content
- Fast processing requirements
- Content that doesn't require JavaScript

**Performance:**
- Fastest mode (~100-200ms)
- Minimal resource usage
- No browser overhead

### Dynamic Mode

Full browser rendering with JavaScript execution:

```json
{
  "url": "https://app.example.com/dashboard",
  "mode": "dynamic",
  "dynamic_config": {
    "wait_conditions": [
      {
        "type": "element_visible",
        "selector": ".dashboard-loaded",
        "timeout": 10000
      }
    ],
    "actions": [
      {
        "type": "click",
        "selector": "button.load-data"
      },
      {
        "type": "wait",
        "duration": 2000
      }
    ],
    "viewport": {
      "width": 1920,
      "height": 1080
    },
    "javascript_enabled": true,
    "timeout": 30000
  }
}
```

**Use Cases:**
- Single-page applications (SPAs)
- React, Vue, Angular applications
- Content loaded via AJAX
- Interactive elements required

**Features:**
- Full JavaScript execution
- DOM manipulation support
- Network request interception
- Custom user interactions

### PDF Mode

Native PDF content extraction:

```json
{
  "url": "https://example.com/document.pdf",
  "mode": "pdf",
  "pdf_config": {
    "extract_text": true,
    "extract_metadata": true,
    "extract_images": false
  }
}
```

**Capabilities:**
- Text content extraction
- Metadata parsing (title, author, creation date)
- Page count and structure
- Optional image extraction

### Adaptive Mode (Recommended)

Intelligent mode selection based on content analysis:

```json
{
  "url": "https://example.com/unknown-page",
  "mode": "adaptive",
  "dynamic_config": {
    "wait_conditions": [
      {
        "type": "dom_content_loaded",
        "timeout": 5000
      }
    ]
  }
}
```

**Decision Logic:**
1. Check URL extension (`.pdf` → PDF mode)
2. Analyze HTML structure and JavaScript usage
3. Fallback to static for simple content
4. Use dynamic for complex applications

## Dynamic Configuration

### Wait Conditions

Define when rendering is considered complete:

#### Element Visibility
```json
{
  "type": "element_visible",
  "selector": ".content-loaded",
  "timeout": 10000
}
```

#### Element Hidden
```json
{
  "type": "element_hidden",
  "selector": ".loading-spinner",
  "timeout": 15000
}
```

#### DOM Content Loaded
```json
{
  "type": "dom_content_loaded",
  "timeout": 5000
}
```

#### Network Idle
```json
{
  "type": "network_idle",
  "timeout": 10000
}
```

#### Fixed Timeout
```json
{
  "type": "timeout",
  "timeout": 8000
}
```

### Actions

Automate user interactions during rendering:

#### Click Actions
```json
{
  "type": "click",
  "selector": "button.show-more"
}
```

#### Scroll Actions
```json
{
  "type": "scroll",
  "selector": ".infinite-scroll-container"
}
```

#### Input Actions
```json
{
  "type": "input",
  "selector": "input[name='search']",
  "value": "search query"
}
```

#### Wait Actions
```json
{
  "type": "wait",
  "duration": 3000
}
```

#### Navigation Actions
```json
{
  "type": "navigate",
  "value": "https://example.com/next-page"
}
```

### Viewport Configuration

Control browser viewport dimensions:

```json
{
  "viewport": {
    "width": 1920,
    "height": 1080
  }
}
```

**Common Presets:**
- Desktop: 1920×1080
- Laptop: 1366×768
- Tablet: 768×1024
- Mobile: 375×667

## Stealth Configuration

Anti-detection measures for bot-resistant websites:

```json
{
  "stealth_config": {
    "user_agent_rotation": true,
    "header_randomization": true,
    "timing_jitter": true,
    "javascript_execution_delay": true
  }
}
```

### User Agent Rotation

Randomize user agent strings from a curated list:

```javascript
// Example rotated user agents
const userAgents = [
  "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
  "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36",
  "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36"
];
```

### Header Randomization

Add realistic browser headers:

```json
{
  "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
  "Accept-Language": "en-US,en;q=0.5",
  "Accept-Encoding": "gzip, deflate, br",
  "Connection": "keep-alive",
  "Upgrade-Insecure-Requests": "1"
}
```

### Timing Jitter

Add human-like delays to interactions:

```javascript
// Random delays between 100-500ms for actions
const delay = Math.random() * 400 + 100;
await page.waitForTimeout(delay);
```

## Complete Examples

### React SPA with Loading States

```json
{
  "url": "https://app.example.com/dashboard",
  "mode": "dynamic",
  "dynamic_config": {
    "wait_conditions": [
      {
        "type": "element_hidden",
        "selector": ".loading-spinner",
        "timeout": 15000
      },
      {
        "type": "element_visible",
        "selector": ".dashboard-content",
        "timeout": 10000
      }
    ],
    "viewport": {
      "width": 1920,
      "height": 1080
    },
    "user_agent": "Mozilla/5.0 (compatible; RipTide-RipTide/1.0)",
    "javascript_enabled": true,
    "timeout": 30000
  },
  "stealth_config": {
    "user_agent_rotation": true,
    "header_randomization": true,
    "timing_jitter": true
  },
  "output_format": "markdown",
  "capture_artifacts": true,
  "timeout": 45
}
```

### Infinite Scroll Content

```json
{
  "url": "https://social.example.com/feed",
  "mode": "dynamic",
  "dynamic_config": {
    "actions": [
      {
        "type": "scroll",
        "selector": ".feed-container"
      },
      {
        "type": "wait",
        "duration": 2000
      },
      {
        "type": "scroll",
        "selector": ".feed-container"
      },
      {
        "type": "wait",
        "duration": 2000
      }
    ],
    "wait_conditions": [
      {
        "type": "network_idle",
        "timeout": 5000
      }
    ],
    "timeout": 45000
  },
  "capture_artifacts": true
}
```

### Login-Protected Content

```json
{
  "url": "https://portal.example.com/protected",
  "mode": "dynamic",
  "dynamic_config": {
    "actions": [
      {
        "type": "input",
        "selector": "input[name='username']",
        "value": "demo@example.com"
      },
      {
        "type": "input",
        "selector": "input[name='password']",
        "value": "demo123"
      },
      {
        "type": "click",
        "selector": "button[type='submit']"
      },
      {
        "type": "wait",
        "duration": 3000
      }
    ],
    "wait_conditions": [
      {
        "type": "element_visible",
        "selector": ".protected-content",
        "timeout": 10000
      }
    ]
  },
  "stealth_config": {
    "timing_jitter": true,
    "javascript_execution_delay": true
  }
}
```

## Response Structure

The `/render` endpoint returns comprehensive results:

```json
{
  "url": "https://app.example.com/dashboard",
  "final_url": "https://app.example.com/dashboard#loaded",
  "mode": "Dynamic",
  "success": true,
  "content": {
    "url": "https://app.example.com/dashboard",
    "title": "Dashboard - Example App",
    "byline": null,
    "published_iso": null,
    "markdown": "# Dashboard\n\n## Key Metrics\n\n- Users: 1,250\n- Revenue: $45,890\n- Conversion: 3.2%",
    "text": "Dashboard. Key Metrics. Users: 1,250. Revenue: $45,890. Conversion: 3.2%",
    "links": [
      "https://app.example.com/users",
      "https://app.example.com/revenue"
    ],
    "media": [
      "https://app.example.com/chart.png"
    ],
    "language": "en",
    "reading_time": 2,
    "quality_score": 88,
    "word_count": 450,
    "categories": ["dashboard", "analytics"],
    "site_name": "Example App",
    "description": "Real-time dashboard with key business metrics"
  },
  "pdf_result": null,
  "artifacts": {
    "screenshot_url": "https://artifacts.riptide.dev/screenshots/abc123.png",
    "mhtml_url": "https://artifacts.riptide.dev/mhtml/abc123.mhtml",
    "network_activity": [
      {
        "url": "https://api.example.com/metrics",
        "method": "GET",
        "status": 200,
        "response_time_ms": 245
      }
    ]
  },
  "stats": {
    "total_time_ms": 8500,
    "dynamic_time_ms": 6200,
    "pdf_time_ms": null,
    "extraction_time_ms": 150,
    "actions_executed": 2,
    "wait_conditions_met": 2,
    "network_requests": 15,
    "page_size_bytes": 2048576
  },
  "error": null,
  "stealth_applied": [
    "user_agent_rotation",
    "header_randomization",
    "timing_jitter"
  ]
}
```

## Performance Considerations

### Timeout Configuration

Set appropriate timeouts based on content complexity:

```json
{
  "timeout": 30,  // Overall request timeout (seconds)
  "dynamic_config": {
    "timeout": 25000,  // Dynamic rendering timeout (milliseconds)
    "wait_conditions": [
      {
        "type": "element_visible",
        "selector": ".content",
        "timeout": 10000  // Individual condition timeout
      }
    ]
  }
}
```

### Resource Optimization

Minimize resource usage:

```json
{
  "dynamic_config": {
    "viewport": {
      "width": 1366,   // Smaller viewport = faster rendering
      "height": 768
    },
    "javascript_enabled": true,
    "timeout": 20000   // Shorter timeout for simple content
  },
  "capture_artifacts": false  // Disable if not needed
}
```

### Concurrent Requests

The API supports concurrent rendering with resource limits:

- Maximum concurrent renders: 5 (configurable)
- Queue timeout: 60 seconds
- Resource monitoring and scaling

## Error Handling

### Common Error Scenarios

#### Timeout Errors
```json
{
  "error": {
    "type": "timeout_error",
    "message": "Dynamic rendering timed out after 30000ms",
    "retryable": true,
    "status": 408
  }
}
```

#### Navigation Errors
```json
{
  "error": {
    "type": "fetch_error",
    "message": "Failed to navigate to URL: ERR_NAME_NOT_RESOLVED",
    "retryable": false,
    "status": 502
  }
}
```

#### Selector Errors
```json
{
  "error": {
    "type": "extraction_error",
    "message": "Element not found: .non-existent-selector",
    "retryable": true,
    "status": 500
  }
}
```

### Retry Strategies

Implement intelligent retry logic:

```javascript
async function renderWithRetry(request, maxRetries = 3) {
  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      const response = await fetch('/render', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(request)
      });

      if (response.ok) {
        return await response.json();
      }

      const error = await response.json();

      // Don't retry non-retryable errors
      if (!error.error.retryable) {
        throw new Error(error.error.message);
      }

      // Exponential backoff
      const delay = Math.pow(2, attempt) * 1000;
      await new Promise(resolve => setTimeout(resolve, delay));

    } catch (err) {
      if (attempt === maxRetries) throw err;
    }
  }
}
```

## Security Considerations

### Input Validation

Always validate and sanitize input:

```javascript
// Validate URL format
function isValidUrl(url) {
  try {
    new URL(url);
    return true;
  } catch {
    return false;
  }
}

// Sanitize selectors
function sanitizeSelector(selector) {
  // Remove potentially dangerous characters
  return selector.replace(/[<>'"]/g, '');
}
```

### Rate Limiting

The API implements rate limiting:

- Per-IP limits: 100 requests/hour
- Per-endpoint limits: 20 render requests/minute
- Dynamic scaling based on resource usage

### Content Security

- No execution of client-provided JavaScript
- Sandboxed browser environments
- Network request filtering
- Resource usage monitoring

## Monitoring and Debugging

### Performance Metrics

Track rendering performance:

```javascript
// Example metrics collection
const metrics = {
  total_time_ms: response.stats.total_time_ms,
  dynamic_time_ms: response.stats.dynamic_time_ms,
  success_rate: response.success ? 1 : 0,
  actions_executed: response.stats.actions_executed,
  network_requests: response.stats.network_requests
};

// Send to monitoring system
analytics.track('render_performance', metrics);
```

### Debug Information

Use artifacts for debugging:

```json
{
  "capture_artifacts": true
}
```

**Artifacts include:**
- Screenshots of the rendered page
- MHTML archives with full page state
- Network activity logs
- Console logs and errors

### Health Monitoring

Monitor rendering health:

```javascript
// Health check for rendering service
async function checkRenderHealth() {
  const testRequest = {
    url: "https://httpbin.org/html",
    mode: "static",
    timeout: 10
  };

  try {
    const response = await renderWithRetry(testRequest, 1);
    return response.success;
  } catch {
    return false;
  }
}
```

This guide provides comprehensive coverage of the dynamic rendering capabilities, from basic usage to advanced optimization and debugging techniques.