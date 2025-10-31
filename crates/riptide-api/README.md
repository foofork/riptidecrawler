# RipTide API

A high-performance REST API server for web crawling, content extraction, and intelligent data processing built with Rust and Axum.

## Overview

RipTide API provides a comprehensive suite of 59 REST endpoints for web scraping, content extraction, search integration, PDF processing, and system monitoring. Built on async Rust with the Axum web framework, it delivers sub-3-second render times, intelligent rate limiting, and production-ready resource management.

### Key Capabilities

- **Web Crawling & Extraction**: Multi-strategy crawling with headless browser support
- **Real-time Streaming**: NDJSON, Server-Sent Events (SSE), and WebSocket streaming
- **PDF Processing**: OCR, table extraction, and text extraction with progress tracking
- **Search Integration**: Deep search with Serper, SearxNG, or URL-based providers
- **Session Management**: Stateful crawling with cookie management and session persistence
- **Worker Queue**: Background job processing with scheduling and priority management
- **Resource Monitoring**: Comprehensive metrics with Prometheus integration
- **Memory Profiling**: jemalloc-based profiling with leak detection and bottleneck analysis

## API Endpoint Categories

The API provides 59 fully documented endpoints organized into 13 categories:

| Category | Endpoints | Description |
|----------|-----------|-------------|
| **Health** | 5 | System health checks, component health, and detailed diagnostics |
| **Crawling** | 6 | Web crawling with streaming support (NDJSON, SSE, WebSocket) |
| **Extraction** | 2 | Content extraction from HTML/JavaScript with WASM support |
| **Search** | 2 | Search engine integration and deep search capabilities |
| **Streaming** | 3 | Real-time data streaming endpoints |
| **Spider** | 3 | Deep crawling with status monitoring and control |
| **Strategies** | 2 | Advanced extraction strategies and pipeline information |
| **PDF** | 4 | PDF processing, OCR, table extraction, and progress tracking |
| **Stealth** | 3 | Anti-bot detection configuration and testing |
| **Tables** | 2 | HTML table extraction and processing |
| **LLM** | 3 | LLM provider management and integration |
| **Sessions** | 12 | Session lifecycle, cookie management, and statistics |
| **Workers** | 9 | Job submission, scheduling, metrics, and queue management |
| **Monitoring** | 8 | Health scores, performance reports, alerts, and profiling |
| **Resources** | 6 | Resource status monitoring (browser pool, memory, rate limiter) |
| **Browser** | 4 | Browser session management and action execution |
| **Telemetry** | 3 | Distributed tracing and telemetry visualization |
| **Admin** | 13 | Tenant management, cache control, and state management (feature-gated) |

**Total**: 59+ production-ready endpoints

## Quick Start

### Prerequisites

- Rust 1.75+ (2021 edition)
- Redis (for caching and session storage)
- Chromium/Chrome (for headless rendering)
- Optional: WASM runtime for advanced extraction

### Basic Setup

```bash
# Install dependencies
cargo build --release

# Set required environment variables
export REDIS_URL="redis://localhost:6379"
export WASM_PATH="./wasm/extractor.wasm"

# Start the server
cargo run --release -- --bind 0.0.0.0:8080
```

### Docker Deployment

```bash
# Build Docker image
docker build -t riptide-api .

# Run with Docker Compose
docker-compose up -d
```

### Example API Call

```bash
# Health check
curl http://localhost:8080/healthz

# Crawl a website
curl -X POST http://localhost:8080/crawl \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://example.com",
    "max_depth": 2,
    "respect_robots": true
  }'

# Extract content
curl -X POST http://localhost:8080/api/v1/extract \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://example.com",
    "selector": "article",
    "wait_for": ".content-loaded"
  }'
```

## Configuration

### Environment Variables

#### Core Settings

| Variable | Default | Description |
|----------|---------|-------------|
| `REDIS_URL` | `redis://localhost:6379` | Redis connection URL for caching |
| `WASM_PATH` | `./wasm/extractor.wasm` | Path to WASM extraction module |
| `HEADLESS_URL` | `http://localhost:9222` | Chromium DevTools Protocol endpoint |
| `MAX_CONCURRENCY` | `10` | Maximum concurrent requests |
| `CACHE_TTL` | `3600` | Cache TTL in seconds |

#### Resource Management

| Variable | Default | Description |
|----------|---------|-------------|
| `RIPTIDE_MAX_CONCURRENT_RENDERS` | `10` | Maximum concurrent render operations |
| `RIPTIDE_MAX_CONCURRENT_PDF` | `2` | Maximum concurrent PDF operations |
| `RIPTIDE_HEADLESS_POOL_SIZE` | `3` | Browser pool size cap |
| `RIPTIDE_RENDER_TIMEOUT` | `3` | Hard timeout for renders (seconds) |
| `RIPTIDE_RATE_LIMIT_RPS` | `1.5` | Requests per second per host |
| `RIPTIDE_RATE_LIMIT_JITTER` | `0.1` | Jitter factor (0.0-1.0) |
| `RIPTIDE_MEMORY_LIMIT_MB` | `2048` | Global memory limit (MB) |

#### Search Provider

| Variable | Default | Description |
|----------|---------|-------------|
| `SEARCH_BACKEND` | `serper` | Search provider: `serper`, `searxng`, or `none` |
| `SEARCH_TIMEOUT` | `30` | Search operation timeout (seconds) |
| `SEARCH_ENABLE_URL_PARSING` | `true` | Enable URL parsing for `none` provider |
| `SERPER_API_KEY` | - | Serper.dev API key (required for Serper) |

#### Telemetry (Optional)

| Variable | Default | Description |
|----------|---------|-------------|
| `OTEL_ENDPOINT` | - | OpenTelemetry collector endpoint |
| `RUST_LOG` | `info` | Log level: `trace`, `debug`, `info`, `warn`, `error` |

#### Authentication (Optional)

| Variable | Default | Description |
|----------|---------|-------------|
| `API_KEY` | - | API key for authentication (optional) |

### Configuration File

Create `config/application/riptide.yml` for advanced configuration:

```yaml
resources:
  max_concurrent_renders: 10
  max_concurrent_pdf: 2
  max_concurrent_wasm: 4
  global_timeout_secs: 30

performance:
  render_timeout_secs: 3
  pdf_timeout_secs: 10
  wasm_timeout_secs: 5

rate_limiting:
  enabled: true
  requests_per_second_per_host: 1.5
  jitter_factor: 0.1
  burst_capacity_per_host: 3

headless:
  max_pool_size: 3
  min_pool_size: 1
  idle_timeout_secs: 300

memory:
  global_memory_limit_mb: 2048
  pressure_threshold: 0.85
  auto_gc: true
```

## Feature Flags

RipTide API uses Cargo features for optional functionality:

| Feature | Description | Status |
|---------|-------------|--------|
| `default` | Minimal feature set | ✅ Stable |
| `events` | Event emitter and result transformers | 🚧 WIP |
| `sessions` | Session management system | ✅ Stable |
| `streaming` | SSE/WebSocket/NDJSON streaming | ✅ Stable |
| `telemetry` | OpenTelemetry integration | ✅ Stable |
| `persistence` | Multi-tenancy and advanced caching | ✅ Stable |
| `jemalloc` | Memory profiling with jemalloc allocator | ✅ Stable |
| `profiling-full` | Full profiling with flamegraphs | 🔬 Dev Only |
| `full` | All production features enabled | ✅ Stable |

### Building with Features

```bash
# Minimal build (default)
cargo build --release

# With session management
cargo build --release --features sessions

# With streaming support
cargo build --release --features streaming

# Full production build
cargo build --release --features full

# Development with full profiling
cargo build --features profiling-full
```

## Deployment Options

### Standalone Binary

```bash
# Build optimized binary
cargo build --release --features full

# Run with custom configuration
./target/release/riptide-api \
  --config config/application/riptide.yml \
  --bind 0.0.0.0:8080
```

### Docker

```dockerfile
FROM rust:1.75-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release --features full

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    chromium \
    redis-tools \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/riptide-api /usr/local/bin/
EXPOSE 8080
CMD ["riptide-api", "--bind", "0.0.0.0:8080"]
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: riptide-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: riptide-api
  template:
    metadata:
      labels:
        app: riptide-api
    spec:
      containers:
      - name: riptide-api
        image: riptide-api:latest
        ports:
        - containerPort: 8080
        env:
        - name: REDIS_URL
          value: "redis://redis-service:6379"
        - name: RIPTIDE_MAX_CONCURRENT_RENDERS
          value: "10"
        resources:
          limits:
            memory: "2Gi"
            cpu: "1000m"
          requests:
            memory: "512Mi"
            cpu: "250m"
        livenessProbe:
          httpGet:
            path: /healthz
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /healthz
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
```

### Kong Gateway Integration

RipTide API is Kong Gateway ready with built-in support for:

- API key authentication
- Rate limiting per host
- Request/response transformations
- Load balancing
- Circuit breakers

Example Kong configuration:

```yaml
services:
- name: riptide-api
  url: http://riptide-api:8080
  routes:
  - name: crawl
    paths:
    - /crawl
    methods:
    - POST
    plugins:
    - name: rate-limiting
      config:
        minute: 90
        policy: redis
    - name: key-auth
      config:
        key_names:
        - apikey
```

## Monitoring and Metrics

### Prometheus Integration

RipTide API exposes Prometheus metrics at `/metrics`:

```bash
curl http://localhost:8080/metrics
```

**Available Metrics:**

- **Request Metrics**: `http_requests_total`, `http_request_duration_seconds`
- **Resource Metrics**: `browser_pool_size`, `pdf_semaphore_available`
- **Memory Metrics**: `memory_allocated_bytes`, `memory_pressure_ratio`
- **Worker Metrics**: `worker_jobs_pending`, `worker_jobs_completed`
- **Rate Limiting**: `rate_limit_permits_available`, `rate_limit_delays_total`

### Health Monitoring

```bash
# Basic health check
curl http://localhost:8080/healthz

# Detailed health with component status
curl http://localhost:8080/api/health/detailed

# Component-specific health
curl http://localhost:8080/health/redis
curl http://localhost:8080/health/browser
curl http://localhost:8080/health/workers

# Health metrics
curl http://localhost:8080/health/metrics
```

### Performance Profiling

```bash
# Memory profiling (requires jemalloc feature)
curl http://localhost:8080/api/profiling/memory

# CPU profiling
curl http://localhost:8080/api/profiling/cpu

# Bottleneck analysis
curl http://localhost:8080/api/profiling/bottlenecks

# Allocation metrics
curl http://localhost:8080/api/profiling/allocations

# Trigger leak detection
curl -X POST http://localhost:8080/api/profiling/leak-detection

# Create heap snapshot
curl -X POST http://localhost:8080/api/profiling/snapshot
```

### Resource Monitoring

```bash
# Overall resource status
curl http://localhost:8080/resources/status

# Browser pool status
curl http://localhost:8080/resources/browser-pool

# Rate limiter status
curl http://localhost:8080/resources/rate-limiter

# Memory status
curl http://localhost:8080/resources/memory

# Performance metrics
curl http://localhost:8080/resources/performance

# PDF semaphore status
curl http://localhost:8080/resources/pdf/semaphore
```

### OpenTelemetry Tracing

Enable distributed tracing with OpenTelemetry:

```bash
export OTEL_ENDPOINT="http://jaeger:4317"
cargo run --release --features telemetry
```

View traces:

```bash
# Telemetry status
curl http://localhost:8080/api/telemetry/status

# List traces
curl http://localhost:8080/api/telemetry/traces

# Get trace tree
curl http://localhost:8080/api/telemetry/traces/{trace_id}
```

## Testing

### Unit Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test handlers::crawl

# Run with features
cargo test --features full
```

### Integration Tests

```bash
# Run integration tests
cargo test --test '*'

# Test with Redis and browser
docker-compose up -d redis chromium
cargo test --features full -- --test-threads=1
```

### Load Testing

```bash
# Using Apache Bench
ab -n 1000 -c 10 \
  -H "Content-Type: application/json" \
  -p request.json \
  http://localhost:8080/crawl

# Using wrk
wrk -t4 -c100 -d30s \
  -s post.lua \
  http://localhost:8080/api/v1/extract
```

### Health Checks

```bash
# Automated health check script
#!/bin/bash
while true; do
  curl -f http://localhost:8080/healthz || exit 1
  sleep 30
done
```

## OpenAPI Documentation

RipTide API provides comprehensive OpenAPI 3.0 documentation:

### Viewing Documentation

1. **Swagger UI** (if enabled):
   ```
   http://localhost:8080/swagger-ui
   ```

2. **OpenAPI JSON Spec**:
   ```
   http://localhost:8080/openapi.json
   ```

3. **ReDoc** (alternative UI):
   ```
   http://localhost:8080/redoc
   ```

### Generating Client SDKs

Use the OpenAPI spec to generate client libraries:

```bash
# Install OpenAPI Generator
npm install -g @openapitools/openapi-generator-cli

# Generate TypeScript client
openapi-generator-cli generate \
  -i http://localhost:8080/openapi.json \
  -g typescript-axios \
  -o ./clients/typescript

# Generate Python client
openapi-generator-cli generate \
  -i http://localhost:8080/openapi.json \
  -g python \
  -o ./clients/python

# Generate Go client
openapi-generator-cli generate \
  -i http://localhost:8080/openapi.json \
  -g go \
  -o ./clients/go
```

## Integration Examples

### Basic Web Scraping

```python
import requests

# Initialize session
session_response = requests.post(
    "http://localhost:8080/sessions",
    json={"ttl_seconds": 3600}
)
session_id = session_response.json()["session_id"]

# Crawl with session
crawl_response = requests.post(
    "http://localhost:8080/crawl",
    json={
        "url": "https://example.com",
        "max_depth": 2,
        "respect_robots": true
    },
    headers={"X-Session-ID": session_id}
)

print(crawl_response.json())
```

### Streaming Extraction

```javascript
// Using Server-Sent Events
const eventSource = new EventSource(
  'http://localhost:8080/crawl/sse',
  {
    method: 'POST',
    body: JSON.stringify({
      url: 'https://example.com',
      max_depth: 3
    })
  }
);

eventSource.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Received:', data);
};

eventSource.onerror = (error) => {
  console.error('Stream error:', error);
  eventSource.close();
};
```

### PDF Processing

```bash
# Extract text from PDF
curl -X POST http://localhost:8080/pdf/extract \
  -F "file=@document.pdf" \
  -F "options={\"ocr\":true,\"language\":\"eng\"}"

# Extract tables from PDF
curl -X POST http://localhost:8080/api/v1/tables/extract \
  -F "file=@report.pdf" \
  -F "format=json"
```

### Worker Queue

```go
package main

import (
    "bytes"
    "encoding/json"
    "net/http"
)

func main() {
    // Submit background job
    job := map[string]interface{}{
        "type": "crawl",
        "payload": map[string]string{
            "url": "https://example.com",
            "depth": "5",
        },
        "priority": "high",
    }

    body, _ := json.Marshal(job)
    resp, _ := http.Post(
        "http://localhost:8080/workers/jobs",
        "application/json",
        bytes.NewBuffer(body),
    )

    var result map[string]string
    json.NewDecoder(resp.Body).Decode(&result)
    jobID := result["job_id"]

    // Check job status
    statusResp, _ := http.Get(
        "http://localhost:8080/workers/jobs/" + jobID,
    )

    // Get job result when complete
    resultResp, _ := http.Get(
        "http://localhost:8080/workers/jobs/" + jobID + "/result",
    )
}
```

### Deep Search Integration

```typescript
interface DeepSearchRequest {
  query: string;
  num_results?: number;
  search_backend?: 'serper' | 'searxng' | 'none';
  max_depth?: number;
}

async function deepSearch(query: string): Promise<any> {
  const response = await fetch('http://localhost:8080/deepsearch', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      query,
      num_results: 10,
      search_backend: 'serper',
      max_depth: 2
    } as DeepSearchRequest)
  });

  return await response.json();
}

// Usage
deepSearch('rust web scraping').then(results => {
  console.log('Search results:', results);
});
```

## Architecture

### Core Components

- **Handlers**: Request processing and business logic
- **State Management**: Shared application state with Arc<AppState>
- **Resource Manager**: Semaphores and guards for resource control
- **Pipeline System**: Multi-stage processing with transformations
- **Streaming Engine**: Real-time data delivery (NDJSON, SSE, WebSocket)
- **Session Manager**: Stateful crawling with cookie persistence
- **Worker Service**: Background job queue with Redis backend
- **Metrics System**: Prometheus integration with custom metrics

### Request Flow

```
Client Request
    ↓
Auth Middleware (API key validation)
    ↓
Rate Limit Middleware (per-host limiting)
    ↓
Payload Limit Layer (50MB cap)
    ↓
Session Layer (optional)
    ↓
Handler (business logic)
    ↓
Resource Guards (semaphores)
    ↓
Pipeline Processing
    ↓
Response (JSON/Stream)
```

### Technology Stack

- **Web Framework**: Axum (async/await with Tokio)
- **Serialization**: Serde JSON
- **Browser Automation**: chromiumoxide (Chrome DevTools Protocol)
- **PDF Processing**: riptide-pdf (OCR with Tesseract)
- **Search**: riptide-search (Serper/SearxNG integration)
- **Memory Allocator**: jemalloc (optional, for profiling)
- **Metrics**: Prometheus with axum-prometheus
- **Tracing**: OpenTelemetry with tracing-opentelemetry
- **Caching**: Redis with connection pooling
- **WASM Runtime**: Wasmtime for extraction modules

## Performance Characteristics

### Benchmarks

- **Render Time**: < 3 seconds (hard timeout enforced)
- **Rate Limiting**: 1.5 RPS per host with ±10% jitter
- **Throughput**: 100+ requests/second (with proper resource tuning)
- **Memory**: < 2GB under normal load
- **Browser Pool**: Maximum 3 concurrent browser instances
- **PDF Semaphore**: 2 concurrent operations maximum

### Optimization Tips

1. **Enable jemalloc**: Reduces memory fragmentation
   ```bash
   cargo build --release --features jemalloc
   ```

2. **Tune resource limits**: Adjust based on available hardware
   ```bash
   export RIPTIDE_MAX_CONCURRENT_RENDERS=20
   export RIPTIDE_HEADLESS_POOL_SIZE=5
   ```

3. **Use connection pooling**: Reuse Redis connections
4. **Enable compression**: Reduce bandwidth with gzip/brotli
5. **Configure rate limiting**: Balance speed vs. politeness
6. **Monitor memory pressure**: Enable auto-GC for long-running instances

## Troubleshooting

### Common Issues

**Browser won't start:**
```bash
# Check Chromium is installed
which chromium-browser

# Test browser connectivity
curl http://localhost:9222/json/version
```

**Redis connection fails:**
```bash
# Verify Redis is running
redis-cli ping

# Check connection string
echo $REDIS_URL
```

**High memory usage:**
```bash
# Enable jemalloc profiling
cargo build --features jemalloc

# Check memory metrics
curl http://localhost:8080/resources/memory
```

**Rate limiting too aggressive:**
```bash
# Increase RPS limit
export RIPTIDE_RATE_LIMIT_RPS=5.0

# Disable rate limiting temporarily
export RIPTIDE_RATE_LIMITING_ENABLED=false
```

### Debug Mode

```bash
# Enable verbose logging
export RUST_LOG=riptide_api=debug,riptide_core=debug

# Run with backtrace
export RUST_BACKTRACE=full
cargo run
```

## License

Apache-2.0

## Contributing

Contributions are welcome! Please see the main EventMesh repository for contribution guidelines.

## Related Crates

- **riptide-core**: Core crawling and extraction engine
- **riptide-pdf**: PDF processing with OCR
- **riptide-search**: Search engine integration
- **riptide-stealth**: Anti-bot detection evasion
- **riptide-workers**: Background job processing
- **riptide-persistence**: Multi-tenant data persistence
- **riptide-performance**: Memory profiling and bottleneck analysis

## Support

For issues, questions, or feature requests, please file an issue in the EventMesh repository.
