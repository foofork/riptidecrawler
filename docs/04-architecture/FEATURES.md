# RipTide Feature Reference

Complete guide to all RipTide features, how to enable/disable them, and usage examples.

---

## Table of Contents

1. [Core Features (Always Available)](#core-features-always-available)
2. [Optional Features (Toggle via Environment)](#optional-features-toggle-via-environment)
3. [Enterprise Features (Not Implemented)](#enterprise-features-not-implemented)
4. [Feature Flags and Configuration](#feature-flags-and-configuration)

---

## Core Features (Always Available)

These features are always enabled and require no configuration (except dependencies like Redis).

### 1. Web Scraping Engine

**Description**: Extract content from web pages using multiple strategies.

**Strategies**:
- **Native Parser** (default): Pure Rust with `scraper` crate - fastest (2-5ms)
- **WASM Parser** (optional): Sandboxed extraction with WebAssembly
- **LLM-Based** (optional): AI-powered extraction with multiple providers
- **Regex** (optional): Pattern-based extraction

**Configuration**:
```bash
# No configuration required - works out of the box
# Optional: Enable WASM extraction
cargo build --features wasm-extractor
WASM_EXTRACTOR_PATH=./target/wasm32-wasip2/release/riptide_extractor_wasm.wasm
```

**API Usage**:
```bash
curl -X POST http://localhost:8080/crawl \
  -H "Content-Type: application/json" \
  -d '{
    "urls": ["https://example.com"],
    "options": {
      "extraction_strategy": "native",
      "selectors": {
        "title": "h1",
        "content": "article p"
      }
    }
  }'
```

**Python SDK**:
```python
from riptide_sdk import RipTideClient

async with RipTideClient("http://localhost:8080") as client:
    result = await client.crawl.single("https://example.com")
    print(result.title, result.content)
```

---

### 2. Redis/DragonflyDB Caching

**Description**: Fast, persistent caching for extracted content.

**Features**:
- Sub-5ms cache access times
- LZ4/Zstd compression
- TTL management
- Automatic eviction (LRU)

**Configuration**:
```bash
# Redis connection
REDIS_URL=redis://localhost:6379

# Cache settings
CACHE_DEFAULT_TTL_SECONDS=86400    # 24 hours
CACHE_ENABLE_COMPRESSION=true
CACHE_COMPRESSION_ALGORITHM=lz4    # lz4, zstd, or none
CACHE_MAX_ENTRY_SIZE_BYTES=20971520  # 20MB
```

**API Usage**:
```bash
# Use cache (default)
curl -X POST http://localhost:8080/crawl \
  -d '{"urls": ["https://example.com"], "options": {"cache_mode": "default"}}'

# Bypass cache
curl -X POST http://localhost:8080/crawl \
  -d '{"urls": ["https://example.com"], "options": {"cache_mode": "bypass"}}'

# Refresh cache
curl -X POST http://localhost:8080/crawl \
  -d '{"urls": ["https://example.com"], "options": {"cache_mode": "refresh"}}'
```

**How It Works**:
1. **Cache Hit**: Return cached content instantly (<5ms)
2. **Cache Miss**: Fetch + extract + cache content
3. **Cache Refresh**: Re-fetch even if cached

---

### 3. Authentication (Optional, Disabled by Default)

**Description**: Simple API key authentication for production deployments.

**Status**: Disabled by default for easier development.

**How to Enable**:
```bash
# Enable authentication
REQUIRE_AUTH=true

# Set API keys (comma-separated)
API_KEYS=your_secret_key_1,your_secret_key_2
```

**API Usage**:
```bash
# With API key in header
curl -X POST http://localhost:8080/crawl \
  -H "X-API-Key: your_secret_key_1" \
  -d '{"urls": ["https://example.com"]}'

# Or with Bearer token
curl -X POST http://localhost:8080/crawl \
  -H "Authorization: Bearer your_secret_key_1" \
  -d '{"urls": ["https://example.com"]}'
```

**Why Disabled by Default?**
- **Developer Experience**: Get started immediately without auth overhead
- **Local Development**: No secrets management during development
- **Gradual Security**: Enable auth when deploying to production

**Production Recommendation**: Always enable authentication in production environments.

---

### 4. Session Management

**Description**: Persistent browser sessions with cookie and localStorage support.

**Features**:
- Isolated browser contexts
- Cookie persistence
- localStorage/sessionStorage persistence
- Session artifact storage

**Configuration**:
```bash
# Session timeout (default: 30 minutes)
STATE_SESSION_TIMEOUT_SECONDS=1800

# Sessions storage directory
RIPTIDE_SESSIONS_DIR=./riptide-output/sessions
```

**API Usage**:
```bash
# Create session and store artifacts
curl -X POST http://localhost:8080/crawl \
  -d '{
    "urls": ["https://example.com"],
    "options": {
      "use_session": true,
      "session_id": "my-session-123",
      "store_cookies": true,
      "store_local_storage": true,
      "capture_screenshot": true,
      "capture_html": true
    }
  }'

# Reuse existing session
curl -X POST http://localhost:8080/crawl \
  -d '{
    "urls": ["https://example.com/dashboard"],
    "options": {
      "use_session": true,
      "session_id": "my-session-123"
    }
  }'
```

---

### 5. Configuration Hot-Reload

**Description**: Update configuration without restarting the service.

**Configuration**:
```bash
# Enable hot reload
STATE_ENABLE_HOT_RELOAD=true

# Watch paths (comma-separated)
STATE_CONFIG_WATCH_PATHS=./config,/etc/riptide
```

**How It Works**:
1. File system watcher monitors config files
2. On change, reload configuration in-memory
3. No service restart required
4. Graceful transition to new settings

**Supported Configurations**:
- LLM provider settings
- Rate limits
- Resource limits
- Cache settings

---

### 6. Deep Spider Crawling

**Description**: Recursive web crawling with depth and breadth control.

**How to Enable**:
```bash
# Enable spider functionality globally
SPIDER_ENABLE=true
```

**API Usage**:
```bash
# Spider crawl with depth limit
curl -X POST http://localhost:8080/spider/crawl \
  -d '{
    "base_url": "https://docs.example.com",
    "max_depth": 3,
    "max_pages": 100,
    "concurrency": 5,
    "respect_robots": true
  }'

# Spider with URL patterns
curl -X POST http://localhost:8080/spider/crawl \
  -d '{
    "base_url": "https://example.com",
    "url_patterns": {
      "include": ["^https://example.com/docs/.*"],
      "exclude": ["^https://example.com/admin/.*"]
    },
    "max_depth": 5
  }'
```

**Python SDK**:
```python
async with RipTideClient("http://localhost:8080") as client:
    async for page in client.spider.crawl_stream(
        base_url="https://docs.example.com",
        max_depth=3,
        max_pages=100
    ):
        print(f"Crawled: {page.url} - {page.status}")
```

**Features**:
- Breadth-first or depth-first crawling
- Robots.txt respect
- URL pattern filtering
- Sitemap integration
- Query-aware relevance scoring

---

### 7. Real-Time Streaming

**Description**: Stream results in real-time as pages are processed.

**Protocols**:
- **NDJSON** (Newline-Delimited JSON)
- **Server-Sent Events** (SSE)
- **WebSocket**

**API Usage**:

**NDJSON Stream**:
```bash
curl -X POST http://localhost:8080/crawl/stream \
  -d '{
    "urls": ["https://example.com", "https://example.org"],
    "options": {"format": "ndjson"}
  }'
```

**Server-Sent Events**:
```javascript
const eventSource = new EventSource('http://localhost:8080/crawl/stream');

eventSource.onmessage = (event) => {
  const result = JSON.parse(event.data);
  console.log('Crawled:', result.url, result.status);
};

eventSource.addEventListener('complete', () => {
  console.log('Crawl complete');
  eventSource.close();
});
```

**WebSocket**:
```javascript
const ws = new WebSocket('ws://localhost:8080/crawl/ws');

ws.onmessage = (event) => {
  const result = JSON.parse(event.data);
  console.log('Crawled:', result.url);
};

ws.send(JSON.stringify({
  urls: ["https://example.com"],
  options: {}
}));
```

**Python SDK**:
```python
async for result in client.crawl.stream_ndjson(
    urls=["https://example.com", "https://example.org"]
):
    print(f"Crawled: {result.url} - {result.status}")
```

**Features**:
- Backpressure control
- Connection lifecycle management
- Automatic reconnection
- Buffering and flow control

---

### 8. LLM/AI Integration

**Description**: AI-powered content extraction using multiple LLM providers.

**Supported Providers**:
1. OpenAI (GPT-3.5, GPT-4)
2. Anthropic (Claude 2, Claude 3)
3. Google Vertex AI (PaLM 2, Gemini)
4. Azure OpenAI
5. AWS Bedrock
6. Ollama (local models)
7. LocalAI (local models)
8. OpenRouter (multi-provider)

**Configuration**:
```bash
# OpenAI
OPENAI_API_KEY=sk-...
OPENAI_BASE_URL=https://api.openai.com/v1  # Optional

# Anthropic
ANTHROPIC_API_KEY=sk-ant-...

# Azure OpenAI
AZURE_OPENAI_KEY=your_key
AZURE_OPENAI_ENDPOINT=https://your-resource.openai.azure.com

# Ollama (local)
OLLAMA_BASE_URL=http://localhost:11434
```

**API Usage**:
```bash
curl -X POST http://localhost:8080/crawl \
  -d '{
    "urls": ["https://example.com"],
    "options": {
      "extraction_strategy": "llm",
      "llm_provider": "openai",
      "llm_model": "gpt-4",
      "llm_prompt": "Extract the main product features from this page"
    }
  }'
```

**Python SDK**:
```python
result = await client.crawl.single(
    "https://example.com",
    extraction_strategy="llm",
    llm_provider="anthropic",
    llm_model="claude-3-opus-20240229",
    llm_prompt="Summarize this article in 3 bullet points"
)
```

**Features**:
- Automatic provider failover
- Circuit breakers
- Token usage tracking
- Cost monitoring
- Retry policies

---

## Optional Features (Toggle via Environment)

### 1. WASM Extraction (Security-Critical)

**Description**: Sandboxed content extraction using WebAssembly.

**When to Use**:
- Extracting content from untrusted sources
- Multi-tenant deployments
- Compliance requirements for isolation
- Security-critical applications

**How to Enable**:
```bash
# Build WASM module
cargo build --features wasm-extractor
cargo build --target wasm32-wasip2 -p riptide-extractor-wasm

# Configure path
WASM_EXTRACTOR_PATH=./target/wasm32-wasip2/release/riptide_extractor_wasm.wasm
```

**API Usage**:
```bash
curl -X POST http://localhost:8080/crawl \
  -d '{
    "urls": ["https://untrusted-source.com"],
    "options": {
      "extraction_strategy": "wasm"
    }
  }'
```

**Performance Trade-off**:
- **Native**: 2-5ms, no isolation
- **WASM**: 5-10ms (40% slower), full sandboxing

---

### 2. Headless Browser Rendering

**Description**: Execute JavaScript and render dynamic content using Chrome.

**Deployment Modes**:

**A. Docker Compose (Recommended)**:
```yaml
services:
  riptide-api:
    image: riptide/api:0.9.0
    environment:
      - HEADLESS_URL=http://riptide-headless:9123

  riptide-headless:
    image: zenika/alpine-chrome:latest
    ports: ["9123:9123"]
```

**B. Local Chrome**:
```bash
# Don't set HEADLESS_URL
# HEADLESS_URL= (empty or unset)

# RipTide will launch Chrome locally
RIPTIDE_HEADLESS_MAX_POOL_SIZE=3
```

**C. WASM-Only (No Chrome)**:
```bash
# Set HEADLESS_URL to empty string
HEADLESS_URL=

# Or use docker-compose.lite.yml
docker-compose -f docker-compose.lite.yml up
```

**API Usage**:
```bash
# Headless rendering for SPA
curl -X POST http://localhost:8080/crawl \
  -d '{
    "urls": ["https://spa-app.example.com"],
    "options": {
      "use_headless": true,
      "wait_for": "networkidle",
      "timeout": 10000
    }
  }'

# With JavaScript execution
curl -X POST http://localhost:8080/crawl \
  -d '{
    "urls": ["https://dynamic-content.com"],
    "options": {
      "use_headless": true,
      "execute_javascript": "document.querySelector(\".load-more\").click()"
    }
  }'
```

**Configuration**:
```bash
# Browser pool settings
RIPTIDE_HEADLESS_MAX_POOL_SIZE=3
RIPTIDE_HEADLESS_MIN_POOL_SIZE=1
RIPTIDE_HEADLESS_IDLE_TIMEOUT_SECS=300

# Performance tuning
RIPTIDE_RENDER_TIMEOUT_SECS=3
RIPTIDE_HEADLESS_MAX_PAGES_PER_BROWSER=10
```

---

### 3. Enhanced Pipeline

**Description**: Detailed phase timing and metrics for extraction pipeline.

**How to Enable**:
```bash
# Enable enhanced pipeline
ENHANCED_PIPELINE_ENABLE=true

# Enable metrics collection
ENHANCED_PIPELINE_METRICS=true

# Enable debug logging (optional)
ENHANCED_PIPELINE_DEBUG=true
```

**Features**:
- Phase-by-phase timing (fetch, parse, extract, render)
- Performance bottleneck detection
- Detailed error reporting
- Quality scoring per phase

**API Response**:
```json
{
  "url": "https://example.com",
  "content": "...",
  "pipeline_metrics": {
    "fetch_time_ms": 145,
    "parse_time_ms": 8,
    "extract_time_ms": 12,
    "total_time_ms": 165,
    "cache_hit": false,
    "extraction_strategy": "native"
  }
}
```

---

### 4. Search Integration

**Description**: Web search integration with multiple backends.

**Backends**:

**A. None (Default, No Dependencies)**:
```bash
RIPTIDE_SEARCH_BACKEND=none
RIPTIDE_SEARCH_ENABLE_URL_PARSING=true
```
- Parses URLs directly from query
- No external API required
- Best for: Development, direct URL extraction

**B. Serper (Google Search)**:
```bash
RIPTIDE_SEARCH_BACKEND=serper
SERPER_API_KEY=your_serper_key  # Get from https://serper.dev
```
- Real Google search results
- 5,000 free searches/month
- Best for: Production

**C. SearXNG (Self-Hosted)**:
```bash
RIPTIDE_SEARCH_BACKEND=searxng
SEARXNG_BASE_URL=http://localhost:8888
```
- Privacy-focused meta-search
- Self-hosted, free
- Best for: Privacy-conscious deployments

**API Usage**:
```bash
# Deep search with content extraction
curl -X POST http://localhost:8080/deepsearch \
  -d '{
    "query": "rust web scraping",
    "limit": 10,
    "include_content": true
  }'
```

**Python SDK**:
```python
results = await client.search.deep(
    query="machine learning frameworks",
    limit=20,
    include_content=True
)
for item in results:
    print(f"{item.title}: {item.url}")
```

---

### 5. Telemetry & Observability

**Description**: OpenTelemetry integration for distributed tracing and metrics.

**Configuration**:
```bash
# Enable telemetry
TELEMETRY_ENABLED=true
TELEMETRY_SERVICE_NAME=riptide-api

# OTLP endpoint
TELEMETRY_OTLP_ENDPOINT=http://localhost:4317

# Exporter type
TELEMETRY_EXPORTER_TYPE=otlp  # or "stdout" for development

# Sampling
TELEMETRY_SAMPLING_RATIO=1.0  # 100% sampling
```

**Metrics Exported**:
- Request latency (p50, p95, p99)
- Error rates
- Cache hit ratios
- Resource usage (CPU, memory)
- Browser pool utilization

**Integration Example** (Jaeger):
```yaml
services:
  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "16686:16686"  # UI
      - "4317:4317"    # OTLP

  riptide-api:
    environment:
      - TELEMETRY_ENABLED=true
      - TELEMETRY_OTLP_ENDPOINT=http://jaeger:4317
```

---

## Enterprise Features (Not Implemented)

These features are defined but not implemented. They require the `persistence` feature flag and return "not implemented" errors.

### 1. Multi-Tenancy

**Status**: Code exists, not enabled.

**Code Location**: `/workspaces/eventmesh/crates/riptide-persistence/src/tenant.rs`

**Features** (if enabled):
- Tenant isolation (logical or strong)
- Resource quotas per tenant
- Access control policies
- Security boundaries
- Billing tracking
- Usage monitoring

**Environment Variables** (not active):
```bash
TENANT_ENABLED=true
TENANT_ISOLATION_LEVEL=strong
TENANT_MAX_TENANTS=1000
TENANT_ENABLE_ENCRYPTION=true
```

**Why Not Implemented?**
- Most users run single-instance deployments
- Adds significant complexity
- Enterprise feature for commercial deployments
- Can be enabled via feature flag when needed

---

### 2. Billing and Quotas

**Status**: Infrastructure exists, not exposed.

**Features** (if enabled):
- Operation counting
- Data transfer tracking
- Storage usage monitoring
- Billing periods and history
- Quota enforcement

**Why Not Implemented?**
- Commercial/SaaS feature
- Self-hosted users don't need billing
- Can be enabled for commercial deployments

---

### 3. Admin Management API

**Status**: Stub endpoints that return "not implemented".

**Endpoints**:
```bash
# All return HTTP 500 with error message:
# "Persistence layer not yet fully integrated - this endpoint is under development"

POST   /admin/tenants           # Create tenant
GET    /admin/tenants           # List tenants
GET    /admin/tenants/:id       # Get tenant
PUT    /admin/tenants/:id       # Update tenant
DELETE /admin/tenants/:id       # Delete tenant
GET    /admin/tenants/:id/usage # Usage stats
POST   /admin/cache/warm        # Warm cache
POST   /admin/cache/invalidate  # Invalidate cache
GET    /admin/cache/stats       # Cache stats
POST   /admin/state/reload      # Reload config
POST   /admin/state/checkpoint  # Create checkpoint
```

**Why Stubs?**
- API signatures defined for testing
- Easy to implement when persistence is integrated
- Maintains API compatibility

---

## Feature Flags and Configuration

### Build-Time Feature Flags

**Defined in Cargo.toml**:
```toml
[features]
default = []
wasm-extractor = ["wasmtime", "wasmtime-wasi"]
persistence = []  # Enterprise features
```

**Usage**:
```bash
# Build with WASM support
cargo build --features wasm-extractor

# Build with all features
cargo build --all-features

# Build without WASM (default)
cargo build
```

### Runtime Feature Toggles

**Environment Variables**:

**Core Features**:
- `REQUIRE_AUTH`: Enable authentication (default: false)
- `SPIDER_ENABLE`: Enable spider crawling (default: false)
- `ENHANCED_PIPELINE_ENABLE`: Enhanced metrics (default: true)
- `TELEMETRY_ENABLED`: OpenTelemetry (default: true)

**Optional Features**:
- `WASM_EXTRACTOR_PATH`: Path to WASM module (empty = native only)
- `HEADLESS_URL`: Headless service URL (empty = WASM only)
- `SEARCH_BACKEND`: Search provider (none, serper, searxng)

**Enterprise Features** (not implemented):
- `TENANT_ENABLED`: Multi-tenancy (default: true, but not functional)
- `TENANT_ENABLE_BILLING`: Billing tracking (default: true, but not functional)

### Configuration Precedence

1. **Environment Variables** (highest priority)
2. **Configuration Files** (`.env`)
3. **Default Values** (in code)

**Example**:
```bash
# .env file
SPIDER_ENABLE=true

# Override at runtime
SPIDER_ENABLE=false cargo run

# Or via Docker
docker run -e SPIDER_ENABLE=false riptide/api
```

---

## Summary

**Always Available** (7 features):
- ✅ Web scraping engine (native parser)
- ✅ Redis caching
- ✅ Session management
- ✅ Configuration hot-reload
- ✅ Real-time streaming
- ✅ LLM integration
- ✅ Performance monitoring

**Optional** (6 features):
- 🔧 Authentication (disable/enable)
- 🔧 WASM extraction (build flag + env)
- 🔧 Headless rendering (Docker service or local)
- 🔧 Deep spider crawling (env toggle)
- 🔧 Search integration (backend selection)
- 🔧 Telemetry (enable/disable)

**Enterprise** (3 feature sets):
- ❌ Multi-tenancy (code exists, not enabled)
- ❌ Billing & quotas (infrastructure only)
- ❌ Admin API (stub endpoints)

---

**Last Updated**: 2024-10-31
**Version**: 0.9.0
**Documentation**: See `/docs/ARCHITECTURE.md` for design decisions
