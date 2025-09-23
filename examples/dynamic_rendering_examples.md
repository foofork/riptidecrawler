# Dynamic Rendering Examples

This document demonstrates how to use the new dynamic rendering capabilities via the `/render` endpoint.

## Basic Dynamic Rendering

```bash
curl -X POST http://localhost:8080/render \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://github.com/rust-lang/rust",
    "mode": "dynamic",
    "dynamic_config": {
      "wait_for": {
        "Selector": {
          "selector": ".repository-content",
          "timeout": 2000000000
        }
      },
      "scroll": {
        "steps": 3,
        "step_px": 1000,
        "delay_ms": 500,
        "mode": "Stepped"
      },
      "actions": [],
      "capture_artifacts": true,
      "timeout": 3000000000,
      "viewport": {
        "width": 1920,
        "height": 1080,
        "device_scale_factor": 1.0,
        "is_mobile": false
      }
    },
    "stealth_config": {
      "preset": "Medium"
    },
    "capture_artifacts": true
  }'
```

## Adaptive Rendering (Automatic Content Analysis)

```bash
curl -X POST http://localhost:8080/render \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://medium.com/@author/some-article",
    "mode": "adaptive",
    "capture_artifacts": true,
    "stealth_config": {
      "preset": "High"
    }
  }'
```

## Advanced Actions and Interactions

```bash
curl -X POST http://localhost:8080/render \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://example-spa.com/login",
    "mode": "dynamic",
    "dynamic_config": {
      "actions": [
        {
          "Type": {
            "selector": "#username",
            "text": "testuser",
            "clear_first": true,
            "wait_after": 500000000
          }
        },
        {
          "Type": {
            "selector": "#password",
            "text": "testpass",
            "clear_first": true,
            "wait_after": 500000000
          }
        },
        {
          "Click": {
            "selector": "#login-button",
            "wait_after": 2000000000
          }
        },
        {
          "Wait": {
            "Selector": {
              "selector": ".dashboard",
              "timeout": 3000000000
            }
          }
        }
      ],
      "capture_artifacts": true,
      "timeout": 3000000000
    },
    "session_id": "persistent-session-123"
  }'
```

## Social Media Content Extraction

```bash
curl -X POST http://localhost:8080/render \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://twitter.com/rustlang/status/123456789",
    "mode": "adaptive",
    "capture_artifacts": true,
    "stealth_config": {
      "preset": "High"
    }
  }'
```

## Static Rendering (No JavaScript)

```bash
curl -X POST http://localhost:8080/render \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://docs.rs/tokio/latest/tokio/",
    "mode": "static",
    "output_format": "markdown"
  }'
```

## PDF Processing

```bash
curl -X POST http://localhost:8080/render \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://example.com/document.pdf",
    "mode": "pdf",
    "pdf_config": {
      "extract_text": true,
      "extract_images": false,
      "ocr_enabled": true
    }
  }'
```

## Response Format

All render requests return a comprehensive response:

```json
{
  "url": "https://github.com/rust-lang/rust",
  "final_url": "https://github.com/rust-lang/rust",
  "mode": "Dynamic",
  "success": true,
  "content": {
    "url": "https://github.com/rust-lang/rust",
    "title": "GitHub - rust-lang/rust: Empowering everyone to build reliable...",
    "markdown": "# Rust Programming Language\n\nA language empowering everyone...",
    "text": "Rust Programming Language. A language empowering everyone...",
    "links": [...],
    "media": [...],
    "quality_score": 85,
    "word_count": 2500
  },
  "artifacts": {
    "screenshot": "base64_encoded_screenshot",
    "mhtml": "base64_encoded_mhtml",
    "metadata": {...},
    "console_logs": [...],
    "network_activity": [...]
  },
  "stats": {
    "total_time_ms": 2850,
    "dynamic_time_ms": 2200,
    "extraction_time_ms": 650,
    "actions_executed": 3,
    "wait_conditions_met": 1,
    "network_requests": 15,
    "page_size_bytes": 187534
  },
  "stealth_applied": [
    "user_agent_rotation",
    "header_randomization",
    "timing_jitter"
  ],
  "session_info": {
    "session_id": "persistent-session-123",
    "user_data_dir": "/tmp/browser-sessions/session-123",
    "cookies_for_domain": 5,
    "state_preserved": true
  }
}
```

## Key Features

### Adaptive Content Analysis
- Automatically detects dynamic content patterns
- Applies appropriate wait conditions and scrolling strategies
- Optimizes for different site types (GitHub, Twitter, Medium, etc.)

### Browser Pool Management
- Reuses browser instances for better performance
- Automatic health monitoring and cleanup
- 3-second hard timeout cap for all operations

### Session Persistence
- Maintains browser state across requests
- Cookie and localStorage preservation
- Support for authenticated content

### Stealth Capabilities
- Anti-detection measures
- User agent rotation
- Header randomization
- Timing jitter

### Comprehensive Artifacts
- Screenshots (PNG, base64 encoded)
- MHTML archives for complete page capture
- Console logs and network activity
- Page metadata and timing information

## Environment Configuration

Set the headless browser service URL:

```bash
export HEADLESS_URL="http://localhost:9123"
export STEALTH_MODE="medium"  # none, low, medium, high
```

## Error Handling

The dynamic rendering system includes robust fallback mechanisms:

1. **Health Check Failures**: Falls back to static rendering if headless service is unavailable
2. **Timeout Enforcement**: Hard 3-second cap on all operations
3. **Graceful Degradation**: Returns partial results rather than complete failures
4. **Resource Cleanup**: Automatic browser cleanup and resource management