# RipTide API - Complete Endpoint Catalog

**Total Endpoints: 59** across 12 categories

This document provides a comprehensive overview of all RipTide API endpoints, organized by feature category.

## Table of Contents
1. [Health & Metrics (2)](#health--metrics)
2. [Core Crawling (5)](#core-crawling)
3. [Search (2)](#search)
4. [Streaming (4)](#streaming)
5. [Spider Deep Crawling (3)](#spider-deep-crawling)
6. [Extraction Strategies (2)](#extraction-strategies)
7. [PDF Processing (3)](#pdf-processing)
8. [Stealth (4)](#stealth)
9. [Table Extraction (2)](#table-extraction)
10. [LLM Providers (4)](#llm-providers)
11. [Sessions (12)](#sessions)
12. [Workers & Jobs (9)](#workers--jobs)
13. [Monitoring (6)](#monitoring)
14. [Pipeline Metrics (1)](#pipeline-metrics)

---

## Health & Metrics

### 1. GET `/healthz` - System Health Check
**Category**: Health | **Phase**: 1

Returns detailed health information including dependency status, system metrics, and uptime.

**Response**:
```json
{
  "status": "healthy",
  "version": "1.0.0",
  "timestamp": "2025-01-15T10:30:00Z",
  "uptime": 3600,
  "dependencies": {
    "redis": {"status": "healthy"},
    "extractor": {"status": "healthy"},
    "http_client": {"status": "healthy"},
    "headless_service": {"status": "healthy"},
    "spider_engine": {"status": "healthy"}
  },
  "metrics": {
    "memory_usage_bytes": 104857600,
    "active_connections": 10,
    "total_requests": 1000,
    "requests_per_second": 50.5
  }
}
```

### 2. GET `/metrics` - Prometheus Metrics
**Category**: Health | **Phase**: 1

Returns Prometheus-formatted metrics for monitoring and observability.

---

## Core Crawling

### 3. POST `/crawl` - Batch Crawl URLs
**Category**: Crawling | **Phase**: 1

Processes multiple URLs through the fetch→gate→extract pipeline with adaptive routing.

**Request**:
```json
{
  "urls": ["https://example.com", "https://example.org"],
  "options": {
    "cache_mode": "auto",
    "concurrency": 5,
    "use_spider": false,
    "chunking_config": {
      "mode": "sliding",
      "token_max": 512,
      "overlap": 50
    }
  }
}
```

**Response**: Includes results for each URL with gate decisions, quality scores, and extracted content.

### 4. POST `/render` - Headless Browser Rendering
**Category**: Crawling | **Phase**: 1

Renders JavaScript-heavy pages using headless Chrome with stealth capabilities.

**Request**:
```json
{
  "url": "https://example.com",
  "wait_time": 2000,
  "screenshot": true
}
```

---

## Search

### 5. POST `/deepsearch` - Deep Search with Content Extraction
**Category**: Search | **Phase**: 1

Performs web search using configured provider and extracts content from discovered URLs.

**Request**:
```json
{
  "query": "web scraping best practices",
  "limit": 10,
  "country": "us",
  "locale": "en",
  "include_content": true,
  "crawl_options": {
    "cache_mode": "auto"
  }
}
```

**Features**:
- Circuit breaker protection for search providers
- Configurable backends (Serper, custom)
- Automatic URL extraction and crawling
- Combined search + content results

### 6. POST `/deepsearch/stream` - Stream Search Results
**Category**: Streaming | **Phase**: 1

Streams deep search results as NDJSON for real-time processing.

---

## Streaming

### 7. POST `/crawl/stream` - Stream Crawl Results (NDJSON)
**Category**: Streaming | **Phase**: 1

Streams crawl results as newline-delimited JSON.

### 8. POST `/crawl/sse` - Stream Crawl Results (SSE)
**Category**: Streaming | **Phase**: 1

Streams crawl results using Server-Sent Events for browser compatibility.

### 9. GET `/crawl/ws` - WebSocket Crawl Stream
**Category**: Streaming | **Phase**: 1

Bidirectional WebSocket connection for real-time crawling.

### 10. (See #6) `/deepsearch/stream`

---

## Spider Deep Crawling

### 11. POST `/spider/crawl` - Deep Crawl with Spider Engine
**Category**: Spider | **Phase**: 3

Performs deep crawling with frontier management and adaptive strategies.

**Request**:
```json
{
  "seed_urls": ["https://example.com"],
  "max_depth": 3,
  "max_pages": 100,
  "strategy": "breadth_first",
  "concurrency": 5,
  "respect_robots": true
}
```

**Features**:
- Frontier-based URL queue
- Multiple strategies (BFS, DFS, Best-First)
- Adaptive stopping
- Budget controls
- Rate limiting

### 12. POST `/spider/status` - Get Spider Status
**Category**: Spider | **Phase**: 3

Returns current spider crawl state and performance metrics.

**Request**:
```json
{
  "include_metrics": true
}
```

### 13. POST `/spider/control` - Control Spider Operations
**Category**: Spider | **Phase**: 3

Controls running spider (stop, reset, pause, resume).

**Request**:
```json
{
  "action": "stop"
}
```

---

## Extraction Strategies

### 14. POST `/strategies/crawl` - Advanced Extraction Strategies
**Category**: Strategies | **Phase**: 2

Crawls with multi-strategy extraction (CSS, TREK, LLM, Regex, Auto).

**Query Parameters**:
- `strategy`: auto, trek, css_json, regex, llm
- `chunking`: sliding, fixed, sentence, topic, regex

**Request**:
```json
{
  "url": "https://example.com",
  "extraction_strategy": "auto",
  "chunking_config": {
    "mode": "sliding",
    "token_max": 512,
    "overlap": 50,
    "preserve_sentences": true
  },
  "enable_metrics": true
}
```

**Features**:
- Auto strategy selection
- Custom CSS selectors
- Regex patterns
- LLM-powered extraction
- Intelligent chunking

### 15. GET `/strategies/info` - Get Strategies Information
**Category**: Strategies | **Phase**: 2

Returns information about all available extraction strategies and their capabilities.

---

## PDF Processing

### 16. POST `/pdf/process` - Process PDF File
**Category**: PDF | **Phase**: 2

Processes base64-encoded PDF and extracts structured content.

**Request**:
```json
{
  "pdf_data": "base64_encoded_pdf_data",
  "filename": "document.pdf",
  "stream_progress": false,
  "url": "https://example.com/doc.pdf"
}
```

**Response**:
```json
{
  "success": true,
  "document": {
    "title": "Document Title",
    "content": "Extracted text...",
    "metadata": {}
  },
  "stats": {
    "processing_time_ms": 1500,
    "file_size": 1024000,
    "pages_processed": 10,
    "pages_per_second": 6.67
  }
}
```

### 17. POST `/pdf/process-stream` - Stream PDF Processing
**Category**: PDF | **Phase**: 2

Processes PDF and streams real-time progress updates as NDJSON.

**Progress Updates**:
- Stage: parsing, extracting_text, rendering, completed
- Current page / total pages
- Estimated remaining time
- Memory usage

### 18. GET `/pdf/health` - PDF Processor Health
**Category**: PDF | **Phase**: 2

Returns health status of PDF processing service.

---

## Stealth

### 19. POST `/stealth/configure` - Configure Stealth Settings
**Category**: Stealth | **Phase**: 3

Configures stealth browsing settings for bot detection evasion.

**Request**:
```json
{
  "preset": "Medium",
  "config": {
    "user_agent_rotation": true,
    "header_randomization": true,
    "timing_jitter": true
  }
}
```

**Response**: Includes effectiveness score, generated user agent, headers, and delays.

### 20. POST `/stealth/test` - Test Stealth Effectiveness
**Category**: Stealth | **Phase**: 3

Tests stealth configuration against real websites.

**Request**:
```json
{
  "urls": ["https://example.com"],
  "preset": "Medium",
  "iterations": 3
}
```

**Response**:
```json
{
  "success": true,
  "results": [...],
  "metrics": {
    "success_rate": 95.0,
    "detection_rate": 5.0,
    "avg_response_time_ms": 850.0
  },
  "recommendations": [
    "Stealth configuration is performing well!"
  ]
}
```

### 21. GET `/stealth/capabilities` - Get Stealth Capabilities
**Category**: Stealth | **Phase**: 3

Returns available stealth features and presets.

### 22. GET `/stealth/health` - Stealth Service Health
**Category**: Stealth | **Phase**: 3

Returns health status of stealth service.

---

## Table Extraction

### 23. POST `/api/v1/tables/extract` - Extract Tables from HTML
**Category**: Tables | **Phase**: 2

Extracts structured table data from HTML content.

**Request**:
```json
{
  "html_content": "<html>...</html>",
  "extract_options": {
    "include_headers": true,
    "detect_data_types": true,
    "include_nested": true,
    "max_nesting_depth": 3
  }
}
```

**Response**:
```json
{
  "tables": [
    {
      "id": "uuid-here",
      "rows": 10,
      "columns": 5,
      "headers": ["Name", "Age", "City"],
      "data": [["John", "30", "NYC"]],
      "metadata": {
        "has_headers": true,
        "data_types": ["string", "number", "string"],
        "has_complex_structure": false
      }
    }
  ],
  "total_tables": 1,
  "extraction_time_ms": 150
}
```

### 24. GET `/api/v1/tables/{id}/export` - Export Table
**Category**: Tables | **Phase**: 2

Exports previously extracted table in CSV or Markdown format.

**Query Parameters**:
- `format`: csv | markdown
- `include_headers`: boolean
- `include_metadata`: boolean (markdown only)

---

## LLM Providers

### 25. GET `/api/v1/llm/providers` - List LLM Providers
**Category**: LLM | **Phase**: 3

Returns information about all configured LLM providers.

**Query Parameters**:
- `provider_type`: Filter by type (openai, anthropic, etc.)
- `available_only`: boolean
- `include_cost`: boolean
- `include_models`: boolean

**Response**:
```json
{
  "providers": [
    {
      "name": "openai",
      "provider_type": "openai",
      "status": "available",
      "capabilities": ["text-generation", "chat", "function-calling"],
      "config_required": ["api_key", "model"],
      "available": true,
      "cost_info": {
        "input_token_cost": 0.001,
        "output_token_cost": 0.002,
        "currency": "USD"
      },
      "models": [...]
    }
  ],
  "current_provider": "openai",
  "total_providers": 3
}
```

### 26. POST `/api/v1/llm/providers/switch` - Switch LLM Provider
**Category**: LLM | **Phase**: 3

Switches between configured LLM providers with optional gradual rollout.

**Request**:
```json
{
  "provider_name": "anthropic",
  "config_updates": {},
  "gradual_rollout": true,
  "rollout_percentage": 50
}
```

### 27. GET `/api/v1/llm/config` - Get LLM Configuration
**Category**: LLM | **Phase**: 3

Returns current LLM configuration and active provider.

### 28. POST `/api/v1/llm/config` - Update LLM Configuration
**Category**: LLM | **Phase**: 3

Updates configuration for LLM providers.

**Request**:
```json
{
  "provider_configs": {
    "openai": {
      "api_key": "sk-...",
      "model": "gpt-4"
    }
  },
  "global_config": {},
  "validate": true
}
```

---

## Sessions

### 29. POST `/sessions` - Create Session
**Category**: Sessions | **Phase**: 3

Creates a new browsing session with isolated cookies and state.

**Response**:
```json
{
  "session_id": "uuid-here",
  "user_data_dir": "/path/to/session",
  "created_at": "2025-01-15T10:00:00Z",
  "expires_at": "2025-01-15T11:00:00Z"
}
```

### 30. GET `/sessions` - List Sessions
**Category**: Sessions | **Phase**: 3

Returns list of all active sessions.

**Query Parameters**:
- `include_expired`: boolean
- `limit`: integer

### 31. GET `/sessions/stats` - Session Statistics
**Category**: Sessions | **Phase**: 3

Returns statistics about all sessions.

**Response**:
```json
{
  "total_sessions": 10,
  "active_sessions": 8,
  "expired_sessions": 2
}
```

### 32. POST `/sessions/cleanup` - Cleanup Expired Sessions
**Category**: Sessions | **Phase**: 3

Removes all expired sessions and their data.

### 33. GET `/sessions/{session_id}` - Get Session Info
**Category**: Sessions | **Phase**: 3

Returns detailed information about a specific session.

### 34. DELETE `/sessions/{session_id}` - Delete Session
**Category**: Sessions | **Phase**: 3

Deletes a session and all associated data.

### 35. POST `/sessions/{session_id}/extend` - Extend Session TTL
**Category**: Sessions | **Phase**: 3

Extends the expiration time of a session.

**Request**:
```json
{
  "ttl_seconds": 3600
}
```

### 36. POST `/sessions/{session_id}/cookies` - Set Cookie
**Category**: Sessions | **Phase**: 3

Adds or updates a cookie for the session.

**Request**:
```json
{
  "domain": "example.com",
  "name": "auth_token",
  "value": "token_value",
  "path": "/",
  "expires_in_seconds": 3600,
  "secure": true,
  "http_only": true
}
```

### 37. DELETE `/sessions/{session_id}/cookies` - Clear All Cookies
**Category**: Sessions | **Phase**: 3

Removes all cookies from the session.

### 38. GET `/sessions/{session_id}/cookies/{domain}` - Get Domain Cookies
**Category**: Sessions | **Phase**: 3

Returns all cookies for a specific domain.

### 39. GET `/sessions/{session_id}/cookies/{domain}/{name}` - Get Specific Cookie
**Category**: Sessions | **Phase**: 3

Returns a specific cookie by domain and name.

### 40. DELETE `/sessions/{session_id}/cookies/{domain}/{name}` - Delete Cookie
**Category**: Sessions | **Phase**: 3

Removes a specific cookie from the session.

---

## Workers & Jobs

### 41. POST `/workers/jobs` - Submit Async Job
**Category**: Workers | **Phase**: 3

Submits a job to the worker queue for asynchronous processing.

**Request**:
```json
{
  "job_type": {
    "type": "batch_crawl",
    "urls": ["https://example.com"],
    "options": {}
  },
  "priority": "High",
  "retry_config": {
    "max_attempts": 3,
    "initial_delay_secs": 5
  },
  "scheduled_at": "2025-01-15T12:00:00Z",
  "timeout_secs": 300
}
```

**Response**:
```json
{
  "job_id": "uuid-here",
  "status": "submitted",
  "submitted_at": "2025-01-15T10:00:00Z",
  "message": "Job submitted successfully"
}
```

### 42. GET `/workers/jobs/{job_id}` - Get Job Status
**Category**: Workers | **Phase**: 3

Returns current status and metadata for a job.

**Response**:
```json
{
  "job_id": "uuid-here",
  "status": "Processing",
  "created_at": "2025-01-15T10:00:00Z",
  "started_at": "2025-01-15T10:00:05Z",
  "worker_id": "worker-1",
  "retry_count": 0,
  "processing_time_ms": 5000
}
```

### 43. GET `/workers/jobs/{job_id}/result` - Get Job Result
**Category**: Workers | **Phase**: 3

Returns the result of a completed job.

### 44. GET `/workers/stats/queue` - Queue Statistics
**Category**: Workers | **Phase**: 3

Returns statistics about the job queue.

**Response**:
```json
{
  "pending": 10,
  "processing": 5,
  "completed": 100,
  "failed": 2,
  "retry": 1,
  "delayed": 3,
  "total": 121
}
```

### 45. GET `/workers/stats/workers` - Worker Pool Stats
**Category**: Workers | **Phase**: 3

Returns statistics about the worker pool.

**Response**:
```json
{
  "total_workers": 10,
  "healthy_workers": 9,
  "total_jobs_processed": 1000,
  "total_jobs_failed": 20,
  "is_running": true
}
```

### 46. GET `/workers/metrics` - Worker Metrics
**Category**: Workers | **Phase**: 3

Returns detailed worker performance metrics.

### 47. POST `/workers/schedule` - Create Scheduled Job
**Category**: Workers | **Phase**: 3

Creates a recurring job with cron schedule.

**Request**:
```json
{
  "name": "daily-crawl",
  "cron_expression": "0 0 * * *",
  "job_template": {
    "type": "batch_crawl",
    "urls": ["https://example.com"]
  },
  "enabled": true,
  "priority": "Normal"
}
```

### 48. GET `/workers/schedule` - List Scheduled Jobs
**Category**: Workers | **Phase**: 3

Returns all scheduled jobs.

### 49. DELETE `/workers/schedule/{job_id}` - Delete Scheduled Job
**Category**: Workers | **Phase**: 3

Removes a scheduled job.

---

## Monitoring

### 50. GET `/monitoring/health-score` - System Health Score
**Category**: Monitoring | **Phase**: 1

Returns overall system health score (0-100).

**Response**:
```json
{
  "health_score": 95.5,
  "status": "excellent",
  "timestamp": "2025-01-15T10:00:00Z"
}
```

**Status Thresholds**:
- 95-100: Excellent
- 85-94: Good
- 70-84: Fair
- 50-69: Poor
- 0-49: Critical

### 51. GET `/monitoring/performance-report` - Performance Report
**Category**: Monitoring | **Phase**: 1

Returns comprehensive performance report with recommendations.

**Response**:
```json
{
  "health_score": 95.5,
  "metrics": {
    "avg_response_time_ms": 250,
    "requests_per_second": 50.5,
    "error_rate": 0.01
  },
  "summary": "System performing well...",
  "recommendations": [
    "Consider increasing cache TTL",
    "Monitor memory usage trends"
  ]
}
```

### 52. GET `/monitoring/metrics/current` - Current Metrics Snapshot
**Category**: Monitoring | **Phase**: 1

Returns current snapshot of all performance metrics.

### 53. GET `/monitoring/alerts/rules` - Alert Rule Definitions
**Category**: Monitoring | **Phase**: 1

Returns configured alert rules.

**Response**:
```json
{
  "rules": [
    {
      "name": "high_error_rate",
      "metric_name": "error_rate",
      "threshold": 0.05,
      "condition": "GreaterThan",
      "severity": "Critical",
      "enabled": true
    }
  ],
  "total": 10,
  "enabled": 8
}
```

### 54. GET `/monitoring/alerts/active` - Active Alerts
**Category**: Monitoring | **Phase**: 1

Returns currently triggered alerts.

**Response**:
```json
{
  "active_alerts": [
    "high_response_time",
    "memory_threshold_exceeded"
  ],
  "count": 2
}
```

### 55. GET `/pipeline/phases` - Pipeline Phase Metrics
**Category**: Monitoring | **Phase**: 1

Returns detailed metrics and bottleneck analysis for pipeline phases.

**Response**:
```json
{
  "overall": {
    "total_requests": 1000,
    "avg_total_time_ms": 360,
    "p50_latency_ms": 288,
    "p95_latency_ms": 648,
    "p99_latency_ms": 900
  },
  "phases": [
    {
      "name": "fetch",
      "avg_duration_ms": 150,
      "percentage_of_total": 41.7,
      "execution_count": 1000,
      "success_rate": 95.0,
      "p50_ms": 120,
      "p95_ms": 225
    },
    {
      "name": "gate",
      "avg_duration_ms": 10,
      "percentage_of_total": 2.8,
      "execution_count": 1000,
      "success_rate": 99.0,
      "p50_ms": 8,
      "p95_ms": 15
    },
    {
      "name": "wasm",
      "avg_duration_ms": 200,
      "percentage_of_total": 55.6,
      "execution_count": 1000,
      "success_rate": 97.0,
      "p50_ms": 160,
      "p95_ms": 300
    },
    {
      "name": "render",
      "avg_duration_ms": 2000,
      "percentage_of_total": 80.0,
      "execution_count": 50,
      "success_rate": 90.0,
      "p50_ms": 1600,
      "p95_ms": 3000
    }
  ],
  "bottlenecks": [
    {
      "phase": "render",
      "severity": "high",
      "description": "render phase is taking 2000ms on average",
      "recommendation": "Reduce headless rendering timeout or optimize dynamic content detection"
    }
  ],
  "success_rates": {
    "overall": 96.0,
    "by_gate_decision": {
      "raw": 60.0,
      "probes_first": 25.0,
      "headless": 5.0,
      "cached": 10.0
    },
    "cache_hit_rate": 10.0
  }
}
```

---

## Summary

### Endpoint Distribution by Phase

**Phase 1: Core Crawling & Event System (11 endpoints)**
- Crawling: 5 endpoints
- Search: 2 endpoints
- Streaming: 4 endpoints
- Health: 2 endpoints
- Monitoring: 6 endpoints
- Pipeline: 1 endpoint

**Phase 2: Advanced Extraction (14 endpoints)**
- Strategies: 2 endpoints
- PDF: 3 endpoints
- Tables: 2 endpoints

**Phase 3: Enterprise Features (34 endpoints)**
- Spider: 3 endpoints
- Stealth: 4 endpoints
- LLM: 4 endpoints
- Sessions: 12 endpoints
- Workers: 9 endpoints

### Key Features by Category

| Category | Endpoints | Key Features |
|----------|-----------|--------------|
| Health | 2 | Dependency health, Prometheus metrics |
| Crawling | 5 | Batch crawling, adaptive gate, caching |
| Search | 2 | Web search integration, content extraction |
| Streaming | 4 | NDJSON, SSE, WebSocket |
| Spider | 3 | Deep crawling, frontier management |
| Strategies | 2 | Multi-strategy extraction, chunking |
| PDF | 3 | PDF extraction, progress streaming |
| Stealth | 4 | Bot evasion, fingerprint randomization |
| Tables | 2 | Table extraction, CSV/Markdown export |
| LLM | 4 | Provider management, runtime switching |
| Sessions | 12 | Cookie persistence, TTL management |
| Workers | 9 | Async jobs, scheduling, retry logic |
| Monitoring | 6 | Health scores, alerts, performance |
| Pipeline | 1 | Phase metrics, bottleneck analysis |

---

## Next Steps

For complete request/response schemas, parameters, and examples:
- See `/docs/api/openapi.yaml` for the OpenAPI 3.0 specification
- See individual handler files in `/crates/riptide-api/src/handlers/`
- See `/docs/api/rest-api.md` for detailed usage examples
