# RipTide Architecture Documentation

> **Version**: 2.0.0
> **Status**: Production Ready (90%+ Complete)
> **Last Updated**: 2025-10-24

---

## Table of Contents

1. [System Overview](#system-overview)
2. [Architectural Principles](#architectural-principles)
3. [Component Architecture](#component-architecture)
4. [Data Flow](#data-flow)
5. [Technology Stack](#technology-stack)
6. [Deployment Architecture](#deployment-architecture)
7. [Security Architecture](#security-architecture)
8. [Performance & Scalability](#performance--scalability)
9. [Extension Points](#extension-points)

---

## System Overview

RipTide is a high-performance web crawling and content extraction platform built with Rust, designed for enterprise-scale web scraping operations with advanced content processing capabilities.

### Core Characteristics

- **Language**: Rust 1.75+ (Edition 2021)
- **Architecture**: Modular microservices with 27 specialized crates
- **Performance**: WASM-optimized extraction with concurrent processing
- **Deployment**: Docker-based with Kubernetes-ready design
- **API**: RESTful with 59 endpoints across 13 categories

### High-Level Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         RipTide Platform                                │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌────────────────┐                                                    │
│  │   Client SDKs  │  (CLI, riptide-sdk, REST API)                     │
│  └────────┬───────┘                                                    │
│           │                                                             │
│           ▼                                                             │
│  ┌────────────────────────────────────────────────────────────────┐   │
│  │                    API Gateway (riptide-api)                   │   │
│  │    REST API (59 endpoints) + WebSocket + SSE                   │   │
│  └────────┬───────────────────────────────────────────────────────┘   │
│           │                                                             │
│           ▼                                                             │
│  ┌────────────────────────────────────────────────────────────────┐   │
│  │              Facade Layer (riptide-facade)                     │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌────────────┐        │   │
│  │  │ Browser  │ │Extraction│ │  Spider  │ │  Pipeline  │        │   │
│  │  │ Facade   │ │  Facade  │ │  Facade  │ │   Facade   │        │   │
│  │  └──────────┘ └──────────┘ └──────────┘ └────────────┘        │   │
│  └────────┬───────────────────────────────────────────────────────┘   │
│           │                                                             │
│           ▼                                                             │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │                    Core Services Layer                          │  │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐            │  │
│  │  │ Spider Engine│ │Fetch Engine  │ │Browser Pool  │            │  │
│  │  │ (riptide-    │ │(riptide-     │ │(riptide-     │            │  │
│  │  │  spider)     │ │  fetch)      │ │  pool)       │            │  │
│  │  └──────────────┘ └──────────────┘ └──────────────┘            │  │
│  │                                                                  │  │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐            │  │
│  │  │ Extraction   │ │ Intelligence │ │   Security   │            │  │
│  │  │ (riptide-    │ │ (riptide-    │ │ (riptide-    │            │  │
│  │  │ extraction)  │ │intelligence) │ │ security)    │            │  │
│  │  └──────────────┘ └──────────────┘ └──────────────┘            │  │
│  │                                                                  │  │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐            │  │
│  │  │  Monitoring  │ │   Streaming  │ │  Persistence │            │  │
│  │  │ (riptide-    │ │ (riptide-    │ │ (riptide-    │            │  │
│  │  │ monitoring)  │ │ streaming)   │ │ persistence) │            │  │
│  │  └──────────────┘ └──────────────┘ └──────────────┘            │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│           │                                                             │
│           ▼                                                             │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │               External Integrations                             │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐           │  │
│  │  │  Redis   │ │ Chromium │ │   WASM   │ │   LLMs   │           │  │
│  │  │  Cache   │ │  (CDP)   │ │Component │ │ (OpenAI, │           │  │
│  │  │          │ │          │ │  Model   │ │Anthropic)│           │  │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘           │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Architectural Principles

### 1. Modularity

**Workspace-Based Design**: 27 specialized crates organized by responsibility:

```
Core (3)              → riptide-types, riptide-config, riptide-test-utils
API & Interface (3)   → riptide-api, riptide-cli, riptide-facade
Extraction (4)        → riptide-extraction, riptide-extractor-wasm,
                        riptide-intelligence, riptide-pdf
Browser (4)           → riptide-browser-abstraction, riptide-headless,
                        riptide-browser, riptide-stealth
Network & Data (5)    → riptide-fetch, riptide-spider, riptide-search,
                        riptide-cache, riptide-persistence
Infrastructure (5)    → riptide-monitoring, riptide-performance,
                        riptide-events, riptide-workers, riptide-streaming
Security & Utils (3)  → riptide-security, riptide-pool, riptide-reliability
```

**Benefits**:
- Clear separation of concerns
- Independent versioning and testing
- Reduced compilation times
- Easy feature toggling

### 2. Performance-First

**WASM Component Model**:
- High-performance content extraction
- Sandboxed execution environment
- Adaptive routing (fast path → slow path)
- Zero-copy data transfer where possible

**Concurrent Processing**:
- Tokio async runtime
- Configurable concurrency (default: 16)
- Per-host rate limiting
- Circuit breaker pattern

### 3. Reliability

**Multi-Layer Error Handling**:
```rust
API Layer     → HTTP status codes + structured errors
Service Layer → Result<T, Error> with context
Domain Layer  → Custom error types with recovery strategies
```

**Fault Tolerance**:
- Circuit breakers for external services
- Exponential backoff with jitter
- Health checks with dependency validation
- Graceful degradation

### 4. Extensibility

**Plugin Architecture**:
- Custom extraction strategies
- LLM provider abstraction
- Configurable content pipelines
- WASM-based extensibility

---

## Component Architecture

### Layer 1: API & Interface

#### riptide-api

**Purpose**: Main REST API gateway

**Key Modules**:
- `handlers/` - Request handlers for 59 endpoints
- `routes/` - Route definitions organized by category
- `middleware/` - Security, CORS, compression, tracing
- `state/` - Application state management
- `streaming/` - SSE and WebSocket support

**Responsibilities**:
- Request validation and deserialization
- Authentication and authorization
- Rate limiting and quota management
- Response serialization and streaming
- Health monitoring

**Technology**: Axum 0.7, Tower middleware, Tokio runtime

#### riptide-cli

**Purpose**: Command-line interface for local operations

**Features**:
- URL crawling and batch processing
- Configuration file generation
- Health checks and status monitoring
- Artifact management
- Development utilities

**Technology**: Clap 4.x with derive macros

#### riptide-facade

**Purpose**: High-level composition layer providing simplified APIs

**Design Pattern**: Facade + Builder

**Components**:
```rust
ScraperFacade     → Simple HTTP fetching with extraction
BrowserFacade     → Headless browser automation
SpiderFacade      → Deep crawling with frontier management
PipelineFacade    → Multi-stage content processing
SearchFacade      → Search-driven discovery
```

**Example Flow**:
```
Client Request
    ↓
RiptideBuilder → Configure components
    ↓
ScraperFacade.fetch() → Coordinate services
    ↓
    ├─→ FetchEngine (HTTP)
    ├─→ CacheManager (Redis)
    ├─→ WasmExtractor (Processing)
    └─→ StrategyManager (Routing)
```

---

### Layer 2: Core Services

#### riptide-spider

**Purpose**: Deep web crawling with intelligent frontier management

**Architecture**:
```
Spider
  ├─→ FrontierManager     (Priority queue, visited tracking)
  ├─→ StrategyEngine      (Crawl strategy selection)
  ├─→ BudgetManager       (Resource limits, quotas)
  ├─→ AdaptiveStopEngine  (Intelligent termination)
  ├─→ RobotsManager       (robots.txt compliance)
  ├─→ SessionManager      (Cookie/auth persistence)
  ├─→ CircuitBreaker      (Fault tolerance)
  └─→ MemoryManager       (Resource monitoring)
```

**Key Features**:
- **URL Frontier**: Priority-based queue with depth tracking
- **Politeness**: Per-host delays and robots.txt compliance
- **Deduplication**: Canonical URL normalization
- **Session Support**: Cookie jar and authentication
- **Query-Aware**: Content relevance scoring

**Concurrency Model**:
- Global semaphore for total concurrency
- Per-host semaphores for politeness
- Async task spawning with Tokio
- Backpressure with bounded channels

#### riptide-fetch

**Purpose**: HTTP client abstraction with retry logic

**Features**:
- HTTP/2 with prior knowledge
- Compression (gzip, brotli, deflate)
- Cookie management
- Proxy support
- Redirect handling
- Timeout configuration

**Retry Strategy**:
```rust
pub struct RetryConfig {
    max_retries: usize,        // Default: 3
    initial_delay: Duration,   // Default: 1s
    max_delay: Duration,       // Default: 60s
    backoff_multiplier: f64,   // Default: 2.0
    jitter: bool,              // Default: true
}
```

#### riptide-extraction

**Purpose**: HTML parsing and content extraction

**Extraction Strategies**:
```
┌────────────────────────────────────┐
│      StrategyManager               │
│   (Auto-selects best strategy)     │
└─────────────┬──────────────────────┘
              │
    ┌─────────┴─────────────────────────────┐
    │                                       │
    ▼                                       ▼
┌─────────────┐  ┌──────────────┐  ┌──────────────┐
│    WASM     │  │     CSS      │  │    Regex     │
│  Extractor  │  │  Selectors   │  │   Patterns   │
└─────────────┘  └──────────────┘  └──────────────┘
    │                    │                  │
    └────────────────────┴──────────────────┘
                         │
                         ▼
               ┌──────────────────┐
               │ ExtractedContent │
               └──────────────────┘
```

**Strategy Selection Logic**:
1. Check cache for strategy hints
2. Analyze content complexity
3. Route to WASM (fast path) or CSS/Regex (fallback)
4. Store performance metrics for learning

**Components**:
- `html_parser.rs` - DOM parsing with scraper
- `css_extraction.rs` - CSS selector-based extraction
- `regex_extraction.rs` - Pattern-based extraction
- `wasm_extraction.rs` - WASM Component Model integration
- `enhanced_extractor.rs` - Multi-strategy coordinator
- `table_extraction/` - Structured table data
- `chunking/` - Content segmentation

#### riptide-intelligence

**Purpose**: LLM integration for advanced extraction

**Provider Abstraction**:
```rust
pub trait LLMProvider {
    async fn extract(&self, request: ExtractionRequest)
        -> Result<LLMResponse>;
    fn supports_streaming(&self) -> bool;
    fn max_tokens(&self) -> usize;
}
```

**Supported Providers**:
- OpenAI (GPT-4, GPT-3.5)
- Anthropic (Claude 3)
- Google Vertex AI (Gemini)
- Azure OpenAI
- AWS Bedrock
- Local models (Ollama)

**Features**:
- Provider failover
- Response caching
- Token usage tracking
- Cost estimation
- Schema-guided extraction

#### riptide-headless

**Purpose**: Headless browser automation via Chrome DevTools Protocol

**Architecture**:
```
BrowserPool
  ├─→ InstanceManager (Lifecycle management)
  ├─→ SessionRegistry (Session isolation)
  ├─→ ResourceLimits  (Memory/CPU caps)
  └─→ CDPMultiplexer  (Connection pooling)
      │
      └─→ spider_chrome (CDP client)
```

**Dynamic Content Handling**:
- JavaScript execution
- AJAX wait strategies
- Network idle detection
- DOM mutation observers
- Screenshot capture

**Stealth Features** (via riptide-stealth):
- User-Agent rotation
- WebRTC leak prevention
- Canvas fingerprint randomization
- Timezone spoofing
- Header consistency

#### riptide-monitoring

**Purpose**: Observability and telemetry

**Components**:
```
Monitoring Stack
  ├─→ Metrics Collection  (Prometheus format)
  ├─→ Distributed Tracing (OpenTelemetry)
  ├─→ Health Scoring      (Component health)
  ├─→ Bottleneck Analysis (Performance)
  └─→ Resource Tracking   (Memory/CPU)
```

**Metrics Categories**:
- **Request**: Total, success rate, latency percentiles
- **Crawl**: Pages/sec, queue depth, error rate
- **Extraction**: Strategy distribution, confidence scores
- **Cache**: Hit rate, evictions, memory usage
- **Browser**: Active sessions, memory per instance

**Health Score Calculation**:
```rust
health_score =
    0.4 * dependency_health +     // Redis, Browser availability
    0.3 * error_rate_health +     // Last 1000 requests
    0.2 * performance_health +    // Latency percentiles
    0.1 * resource_health         // Memory/CPU usage
```

#### riptide-streaming

**Purpose**: Real-time data streaming protocols

**Protocols**:
- **NDJSON**: Newline-delimited JSON for bulk streaming
- **SSE**: Server-Sent Events for real-time updates
- **WebSocket**: Bi-directional communication

**Use Cases**:
- Live crawl progress updates
- Real-time extraction results
- Health monitoring dashboards
- Job queue status

---

### Layer 3: Infrastructure

#### riptide-cache

**Purpose**: Redis-backed caching with TTL management

**Cache Strategies**:
```
Read-Through:  Cache → Miss → Fetch → Store → Return
Write-Through: Write → Store → Cache → Return
Write-Behind:  Write → Queue → Async Cache Update
```

**Cache Keys**:
```
crawl:{url_hash}        → CrawlResult (TTL: 24h)
extract:{url_hash}      → ExtractedContent (TTL: 12h)
strategy:{domain}       → StrategyHint (TTL: 1h)
session:{session_id}    → SessionData (TTL: 30m)
robots:{domain}         → RobotsTxt (TTL: 24h)
```

#### riptide-persistence

**Purpose**: Long-term data storage abstraction

**Storage Backends**:
- Local filesystem (artifacts, logs)
- S3-compatible object storage
- Database integration (PostgreSQL, SQLite)

**Data Types**:
- Extracted content archives
- Crawl metadata and statistics
- Configuration snapshots
- Performance metrics history

#### riptide-events

**Purpose**: Event-driven architecture support

**Event Types**:
```rust
pub enum SystemEvent {
    CrawlStarted { request_id: Uuid, urls: Vec<Url> },
    PageCrawled { url: Url, success: bool, duration: Duration },
    ExtractionComplete { url: Url, strategy: String, confidence: f64 },
    BrowserLaunched { instance_id: Uuid },
    CacheEviction { key: String, reason: EvictionReason },
    HealthScoreUpdated { score: f64, components: HashMap<String, f64> },
}
```

**Event Flow**:
```
Component → Emit Event → Event Bus → Subscribers
                            │
                            ├─→ Monitoring Dashboard
                            ├─→ Metrics Collector
                            ├─→ Audit Logger
                            └─→ WebSocket Broadcaster
```

---

## Data Flow

### Primary Crawl Flow

```
1. Client Request
   ↓
2. API Gateway (riptide-api)
   - Validate request
   - Check authentication
   - Create request ID
   ↓
3. Facade Layer
   - Select appropriate facade
   - Configure pipeline
   ↓
4. Cache Check (riptide-cache)
   - Check for cached result
   - Return if fresh → END
   ↓
5. URL Frontier (riptide-spider)
   - Enqueue URLs
   - Apply prioritization
   - Check robots.txt
   ↓
6. Fetch Phase (riptide-fetch)
   - HTTP request with retries
   - Handle redirects
   - Decompress response
   ↓
7. Content Gate Decision
   ├─→ Static Content (HTML, JSON, XML)
   │   └─→ Fast Path → WASM Extractor
   │
   └─→ Dynamic Content (JavaScript, AJAX)
       └─→ Slow Path → Browser Pool
           - Render in Chromium
           - Wait for network idle
           - Execute scripts
           - Extract DOM
   ↓
8. Extraction Phase (riptide-extraction)
   - Strategy selection
   - Content parsing
   - Structure extraction
   - Confidence scoring
   ↓
9. Intelligence Enhancement (riptide-intelligence)
   - LLM-based extraction (optional)
   - Schema validation
   - Data enrichment
   ↓
10. Post-Processing
    - Chunking (if enabled)
    - Format conversion
    - Metadata attachment
    ↓
11. Cache Update (riptide-cache)
    - Store result with TTL
    - Update strategy hints
    ↓
12. Response Streaming
    - NDJSON, SSE, or JSON
    - Include metadata
    - Track metrics
    ↓
13. Client Receives Result
```

### Deep Search Flow

```
1. POST /deepsearch
   - query: "rust web scraping"
   - limit: 10
   ↓
2. Search Phase (riptide-search)
   - Query Serper.dev API
   - Parse search results
   - Extract URLs + snippets
   ↓
3. Spider Initialization
   - Create frontier with search URLs
   - Set priority based on relevance
   - Configure depth limits
   ↓
4. Parallel Crawling
   ┌─────┬─────┬─────┬─────┐
   │ URL1│ URL2│ URL3│ URL4│ (Concurrent workers)
   └──┬──┴──┬──┴──┬──┴──┬──┘
      ↓     ↓     ↓     ↓
    Fetch → Extract → Score
   ↓
5. Relevance Scoring
   - Query-aware content analysis
   - TF-IDF scoring
   - Link graph analysis
   ↓
6. Adaptive Stopping
   - Check budget (time, pages, bytes)
   - Evaluate quality threshold
   - Monitor diminishing returns
   ↓
7. Result Aggregation
   - Sort by relevance
   - Apply limit
   - Format response
   ↓
8. Stream Results (NDJSON/SSE)
```

---

## Technology Stack

### Core Technologies

| Component | Technology | Version | Purpose |
|-----------|-----------|---------|---------|
| Language | Rust | 1.75+ | Performance, safety, concurrency |
| Web Framework | Axum | 0.7 | REST API, middleware |
| Async Runtime | Tokio | 1.x | Async I/O, task scheduling |
| HTTP Client | Reqwest | 0.12 | HTTP requests, retries |
| Browser Automation | spider_chrome | 2.37 | CDP client for Chromium |
| WASM Runtime | Wasmtime | 37 | WASM Component Model |
| Caching | Redis | 7+ | Distributed cache |
| HTML Parsing | Scraper | 0.20 | CSS selector parsing |
| Serialization | Serde | 1.x | JSON, YAML, TOML |

### Supporting Libraries

**Observability**:
- `tracing` - Structured logging
- `tracing-subscriber` - Log output formatting
- `opentelemetry` - Distributed tracing
- `sysinfo` - System resource monitoring

**Error Handling**:
- `anyhow` - Flexible error handling
- `thiserror` - Custom error types

**Utilities**:
- `dashmap` - Concurrent HashMap
- `governor` - Rate limiting
- `uuid` - Unique identifiers
- `chrono` - Date/time handling

---

## Deployment Architecture

### Docker Compose Stack

```yaml
services:
  redis:
    image: redis:7-alpine
    ports: ["6379:6379"]
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s

  riptide-api:
    build: .
    ports: ["8080:8080"]
    depends_on:
      - redis
    environment:
      - REDIS_URL=redis://redis:6379
      - RUST_LOG=info
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/healthz"]
```

### Kubernetes Deployment (Production)

**Architecture**:
```
┌─────────────────────────────────────────┐
│         Load Balancer (Ingress)         │
└─────────────┬───────────────────────────┘
              │
    ┌─────────┴─────────┐
    │                   │
    ▼                   ▼
┌─────────┐       ┌─────────┐
│  API    │       │  API    │  (Replicas: 3+)
│  Pod 1  │       │  Pod 2  │
└────┬────┘       └────┬────┘
     │                 │
     └────────┬────────┘
              │
    ┌─────────┴─────────────┐
    │                       │
    ▼                       ▼
┌──────────┐         ┌──────────┐
│  Redis   │         │ Browser  │
│ Cluster  │         │  Pool    │
│(StatefulSet)       │(DaemonSet)
└──────────┘         └──────────┘
```

**Scaling Strategy**:
- **Horizontal**: API pods scale based on CPU/memory
- **Vertical**: Browser pool scales with node capacity
- **Cache**: Redis Cluster for distributed caching

---

## Security Architecture

### Authentication & Authorization

**API Key Management**:
```
Request → Header: X-API-Key → Validate → Budget Check → Allow/Deny
```

**Budget Enforcement**:
- Request count limits (per day/month)
- Byte transfer limits
- Concurrent request limits
- Rate limiting per API key

### Data Security

**PII Handling**:
- Automatic PII detection (emails, phone numbers, SSNs)
- Configurable redaction
- Audit logging

**Content Security**:
- Input sanitization (XSS prevention)
- URL validation (SSRF prevention)
- File size limits
- MIME type validation

### Network Security

**Stealth Features**:
- User-Agent rotation
- Fingerprint randomization
- Proxy support (HTTP, SOCKS5)
- Cookie jar isolation

**Compliance**:
- robots.txt enforcement
- Configurable delays
- User-Agent identification
- Respect for rate limits

---

## Performance & Scalability

### Performance Characteristics

**Benchmarks** (24-core, 64GB RAM):
- **Throughput**: 1000+ pages/minute (static content)
- **Latency**: p50=150ms, p95=800ms, p99=2s
- **Concurrency**: 100+ parallel crawls
- **Memory**: ~500MB base + 200MB per browser instance

### Optimization Strategies

**Compilation**:
```toml
[profile.release]
opt-level = 3            # Maximum optimization
lto = "thin"             # Link-time optimization
codegen-units = 1        # Better optimization, slower compile
```

**WASM**:
```toml
[profile.wasm]
opt-level = "s"          # Optimize for size
lto = "fat"              # Full LTO
strip = true             # Remove debug symbols
```

**Runtime**:
- Connection pooling (HTTP keep-alive)
- DNS caching
- Zero-copy parsing where possible
- Lazy initialization

### Scalability Patterns

**Horizontal Scaling**:
- Stateless API design
- Shared cache layer
- Distributed queue (future)

**Resource Management**:
- Browser pool with limits
- Per-host concurrency limits
- Memory-based circuit breakers
- Adaptive request throttling

---

## Extension Points

### Custom Extraction Strategies

```rust
pub trait ExtractionStrategy {
    fn name(&self) -> &str;
    fn can_handle(&self, content: &Content) -> bool;
    async fn extract(&self, content: &Content) -> Result<ExtractedDoc>;
    fn metrics(&self) -> PerformanceMetrics;
}

// Register custom strategy
StrategyManager::register(Box::new(MyCustomStrategy));
```

### LLM Provider Integration

```rust
pub struct CustomLLMProvider {
    api_key: String,
    endpoint: Url,
}

#[async_trait]
impl LLMProvider for CustomLLMProvider {
    async fn extract(&self, request: ExtractionRequest)
        -> Result<LLMResponse> {
        // Custom implementation
    }
}
```

### Pipeline Middleware

```rust
pub trait PipelineMiddleware {
    async fn before_fetch(&self, url: &Url) -> Result<()>;
    async fn after_extract(&self, doc: &mut ExtractedDoc) -> Result<()>;
}
```

---

## Future Architecture

### Planned Enhancements (Phase 2)

1. **Distributed Crawling**: Coordinator-worker architecture
2. **GraphQL API**: Flexible querying of crawl results
3. **Analytics Dashboard**: Real-time visualization
4. **Message Queue**: RabbitMQ/Kafka for job distribution
5. **Database Integration**: PostgreSQL for metadata storage
6. **Search Index**: Elasticsearch for full-text search

### Migration Paths

**Current State** (Phase 1):
```
API → Services → Redis/Browser
```

**Future State** (Phase 2):
```
API Gateway → Load Balancer → [API Pods]
                                    ↓
Message Queue → [Worker Pods] → Coordinator
                                    ↓
                          [Browser Cluster]
                                    ↓
                    [Cache] [DB] [Search Index]
```

---

## Conclusion

RipTide's architecture is designed for:

✅ **Performance**: WASM optimization, concurrent processing
✅ **Reliability**: Multi-layer error handling, fault tolerance
✅ **Scalability**: Modular design, horizontal scaling
✅ **Maintainability**: Clear separation of concerns, comprehensive testing
✅ **Extensibility**: Plugin architecture, provider abstraction

The modular crate structure enables independent evolution of components while maintaining a cohesive system design.

For detailed component documentation, see individual crate README files in `/crates/`.
