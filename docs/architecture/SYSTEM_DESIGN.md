# System Design Documentation

## Architecture Overview

RipTide employs a **layered, modular architecture** with clear separation of concerns and multiple deployment modes.

```
┌─────────────────────────────────────────────────────────────────────┐
│                           Client Layer                               │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────┐   │
│  │   CLI    │  │ Web SDK  │  │ Python   │  │  Direct Rust     │   │
│  │ (Node.js)│  │(Browser) │  │   SDK    │  │   Integration    │   │
│  └─────┬────┘  └─────┬────┘  └─────┬────┘  └────────┬─────────┘   │
└────────┼─────────────┼─────────────┼─────────────────┼─────────────┘
         │             │             │                 │
         │    ┌────────┴─────────────┴─────────┐      │
         │    │                                 │      │
         ▼    ▼                                 ▼      ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        API Layer (Optional)                          │
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │              REST API Server (riptide-api)                   │   │
│  │  • 59 RESTful endpoints across 13 categories                │   │
│  │  • Request validation & authentication                       │   │
│  │  • Rate limiting & throttling                                │   │
│  │  • Response formatting (JSON/Stream)                         │   │
│  │  • API-First routing for CLI                                 │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                              │                                        │
└──────────────────────────────┼────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         Core Services Layer                          │
│                                                                       │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │                  Orchestration (riptide-core)                │  │
│  │  • Request routing & coordination                            │  │
│  │  • Pipeline management                                       │  │
│  │  • Worker pool management                                    │  │
│  │  • Error handling & retry logic                              │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                       │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │            Content Extraction (riptide-extraction)           │  │
│  │  • WASM-powered extraction engine                            │  │
│  │  • Multi-strategy extraction (CSS, Regex, LLM)               │  │
│  │  • HTML parsing & cleaning                                   │  │
│  │  • Markdown generation                                       │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                       │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │              Crawling Engine (riptide-core)                  │  │
│  │  • Concurrent HTTP/2 requests                                │  │
│  │  • Spider/frontier management                                │  │
│  │  • Robots.txt compliance                                     │  │
│  │  • Adaptive routing (fast/headless)                          │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                       │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │          Headless Browser (riptide-headless)                 │  │
│  │  • Chromiumoxide integration                                 │  │
│  │  • JavaScript rendering                                      │  │
│  │  • Stealth mode & anti-detection                             │  │
│  │  • Screenshot & PDF generation                               │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                       │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │              Search Integration (riptide-search)             │  │
│  │  • Multi-provider support (Serper, etc.)                     │  │
│  │  • Circuit breaker pattern                                   │  │
│  │  • Result aggregation & ranking                              │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                       │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │            Session Management (riptide-persistence)          │  │
│  │  • Cookie persistence                                        │  │
│  │  • Session isolation                                         │  │
│  │  • TTL management                                            │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                       │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │              Job Queue (riptide-workers)                     │  │
│  │  • Async task processing                                     │  │
│  │  • Redis-backed persistence                                  │  │
│  │  • Retry logic & scheduling                                  │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                       │
└───────────────────────────────┬───────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      Integration Layer                               │
│                                                                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐  │
│  │    Redis     │  │   Chromium   │  │    WASM Runtime          │  │
│  │              │  │              │  │    (Wasmtime)            │  │
│  │ • Caching    │  │ • Rendering  │  │                          │  │
│  │ • Sessions   │  │ • JavaScript │  │ • Component Model        │  │
│  │ • Job Queue  │  │ • Screenshots│  │ • High Performance       │  │
│  └──────────────┘  └──────────────┘  └──────────────────────────┘  │
│                                                                       │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │              LLM Providers (riptide-intelligence)            │  │
│  │  • OpenAI • Anthropic • Google Vertex AI                     │  │
│  │  • Runtime provider switching                                │  │
│  │  • Intelligent content extraction                            │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                       │
└─────────────────────────────────────────────────────────────────────┘
```

## Component Responsibilities

### 1. Client Layer

#### CLI (riptide-cli)
- **Purpose**: Command-line interface for all RipTide operations
- **Languages**: Node.js (TypeScript/JavaScript)
- **Key Features**:
  - Two operation modes: API-First (default) and Direct
  - Configurable output directories
  - Rich terminal UI with progress indicators
  - Configuration management

**API-First Mode:**
```
CLI → HTTP Request → REST API → Core Services → Response
```

**Direct Mode:**
```
CLI → Direct Rust FFI/Subprocess → Core Services → Response
```

#### Web SDK
- Browser-based JavaScript/TypeScript client
- REST API integration
- Real-time streaming support (SSE, WebSocket)

#### Python SDK
- Official Python client library
- Async/await support
- Type hints and comprehensive docs

### 2. API Layer

#### REST API Server (riptide-api)
- **Framework**: Axum 0.7 (Rust)
- **Runtime**: Tokio (async multi-threaded)
- **Endpoints**: 59 total across 13 categories

**Endpoint Categories:**
```
Health & Metrics (2)     → System status and Prometheus metrics
Core Crawling (5)        → URL crawling and rendering
Search (2)               → Web search integration
Streaming (4)            → Real-time data (NDJSON, SSE, WebSocket)
Spider (3)               → Deep crawling with frontier
Strategies (2)           → Multi-strategy extraction
PDF Processing (3)       → PDF extraction and tables
Stealth (4)              → Anti-detection features
Table Extraction (2)     → HTML table parsing
LLM Providers (4)        → Multi-provider management
Sessions (12)            → Cookie and session handling
Workers & Jobs (9)       → Async job processing
Monitoring (6)           → Health scores and alerts
```

**Request Flow:**
1. Request validation (headers, body, auth)
2. Rate limiting check
3. Route to appropriate handler
4. Core service invocation
5. Response formatting (JSON/Stream)
6. Error handling and logging

### 3. Core Services Layer

#### Orchestration (riptide-core)
**Responsibilities:**
- Coordinate between different services
- Manage pipeline stages (fetch → gate → extract)
- Worker pool management
- Concurrent request handling
- Circuit breaker pattern for external services

**Key Patterns:**
- **Pipeline Pattern**: Sequential stages with gating logic
- **Worker Pool**: Configurable concurrency limits
- **Circuit Breaker**: Fault tolerance for external services
- **Retry Logic**: Exponential backoff for transient failures

#### Content Extraction (riptide-extraction)
**Extraction Strategies:**
1. **Auto-Selection**: Intelligent strategy based on content type
2. **CSS Selectors**: Fast, precise extraction
3. **LLM-Powered**: Intelligent content understanding
4. **Regex Patterns**: Custom pattern matching

**Processing Pipeline:**
```
Raw HTML → Parsing → Cleaning → Extraction → Markdown → Output
```

**WASM Integration:**
- Component Model architecture
- Adaptive routing (WASM vs Native)
- High-performance extraction
- Sandbox isolation

#### Crawling Engine (riptide-core)
**Features:**
- HTTP/2 with prior knowledge
- Compression (Gzip, Brotli, Zstd)
- Robots.txt compliance
- Rate limiting per domain
- Concurrent request handling

**Adaptive Routing (Gate Phase):**
```
Content Analysis → Decision Tree → Route Selection
                                      ├→ Static (Fast)
                                      ├→ WASM (Optimal)
                                      └→ Headless (Full Render)
```

**Decision Factors:**
- Content-Type detection
- JavaScript requirement analysis
- Quality score calculation
- Performance metrics

#### Headless Browser (riptide-headless)
- **Engine**: Chromiumoxide (Chrome DevTools Protocol)
- **Features**:
  - JavaScript rendering
  - Network interception
  - Cookie management
  - Screenshot capture
  - Stealth mode (fingerprint randomization)

#### Session Management (riptide-persistence)
- Redis-backed session store
- Cookie persistence with TTL
- Session isolation
- Concurrent session handling

#### Job Queue (riptide-workers)
- Async task processing
- Redis-backed job persistence
- Retry logic with exponential backoff
- Job scheduling and prioritization
- Progress tracking

### 4. Integration Layer

#### Redis
**Use Cases:**
- **Caching**: Read-through cache with TTL
- **Sessions**: Cookie and session data
- **Job Queue**: Background task persistence
- **Rate Limiting**: Token bucket implementation

#### WASM Runtime (Wasmtime)
- Component Model support
- WASI Preview 2 interface
- High-performance execution
- Memory safety guarantees

#### LLM Providers
- **OpenAI**: GPT models for extraction
- **Anthropic**: Claude for understanding
- **Google Vertex AI**: Multi-model support
- Runtime provider switching
- Fallback mechanism

## Data Flow Diagrams

### 1. Content Extraction Flow

```
┌─────────┐
│ Request │
└────┬────┘
     │
     ▼
┌─────────────────┐
│  API Endpoint   │
│  /extract       │
└────┬────────────┘
     │
     ▼
┌─────────────────────────────────────┐
│       Fetch Phase (HTTP)            │
│  • Make HTTP/2 request              │
│  • Check Redis cache                │
│  • Handle compression               │
└────┬────────────────────────────────┘
     │
     ▼
┌─────────────────────────────────────┐
│      Gate Phase (Decision)          │
│  • Analyze content type             │
│  • Check JS requirements            │
│  • Calculate quality score          │
│  • Select route: Static/WASM/Browser│
└────┬────────────────────────────────┘
     │
     ├─────────────┬─────────────┬─────────────┐
     ▼             ▼             ▼             ▼
┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐
│  Static  │  │   WASM   │  │ Headless │  │   LLM    │
│  Parser  │  │  Engine  │  │ Browser  │  │ Provider │
└────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘
     │             │             │             │
     └─────────────┴─────────────┴─────────────┘
                        │
                        ▼
              ┌─────────────────┐
              │  Post-Processing │
              │  • Clean HTML    │
              │  • Extract links │
              │  • Generate MD   │
              └────┬─────────────┘
                   │
                   ▼
              ┌─────────────────┐
              │  Cache & Return │
              └─────────────────┘
```

### 2. Deep Crawl Flow

```
┌─────────┐
│  Start  │
│   URL   │
└────┬────┘
     │
     ▼
┌─────────────────┐
│  Spider Engine  │
│  • Initialize   │
│  • Set limits   │
└────┬────────────┘
     │
     ▼
┌──────────────────────────────┐
│     Frontier Manager         │
│  • URL queue management      │
│  • Deduplication             │
│  • Prioritization            │
└────┬─────────────────────────┘
     │
     │ (loop until complete or limit)
     ▼
┌─────────────────────────────────┐
│    Fetch & Extract (parallel)   │
│  • Concurrent requests          │
│  • Content extraction           │
│  • Link extraction              │
└────┬────────────────────────────┘
     │
     ▼
┌─────────────────────────────────┐
│    Add Discovered URLs          │
│  • Filter by domain rules       │
│  • Check depth limits           │
│  • Add to frontier              │
└────┬────────────────────────────┘
     │
     ▼
┌─────────────────────────────────┐
│    Check Termination            │
│  • Max pages reached?           │
│  • Frontier empty?              │
│  • Timeout exceeded?            │
└────┬────────────────────────────┘
     │
     ▼
┌─────────────────┐
│  Aggregate &    │
│  Return Results │
└─────────────────┘
```

### 3. CLI Architecture Flow

#### API-First Mode (Default)
```
┌──────────────┐
│   CLI User   │
│   Command    │
└──────┬───────┘
       │
       ▼
┌──────────────────────┐
│  CLI Parser          │
│  • Parse args        │
│  • Load config       │
│  • Validate options  │
└──────┬───────────────┘
       │
       ▼
┌──────────────────────┐
│  HTTP Client         │
│  • Build request     │
│  • Add auth headers  │
│  • Set timeout       │
└──────┬───────────────┘
       │
       ▼
┌──────────────────────────────────┐
│       REST API Server            │
│  • Validate request              │
│  • Route to handler              │
│  • Process via core services     │
└──────┬───────────────────────────┘
       │
       ▼
┌──────────────────────────────────┐
│     Core Services                │
│  • Execute operation             │
│  • Generate response             │
└──────┬───────────────────────────┘
       │
       ▼
┌──────────────────────────────────┐
│     API Response                 │
│  • Format JSON                   │
│  • Stream if needed              │
└──────┬───────────────────────────┘
       │
       ▼
┌──────────────────────────────────┐
│   CLI Output Handler             │
│  • Parse response                │
│  • Format for terminal           │
│  • Save to file (if specified)   │
│  • Respect output dir config     │
└──────┬───────────────────────────┘
       │
       ▼
┌──────────────────────────────────┐
│   File System Write              │
│  • Check output dir config       │
│  • Create subdirectories         │
│  • Write with proper permissions │
└──────────────────────────────────┘
```

#### Direct Mode
```
┌──────────────┐
│   CLI User   │
│   Command    │
│  --direct    │
└──────┬───────┘
       │
       ▼
┌──────────────────────┐
│  CLI Parser          │
│  • Detect --direct   │
│  • Skip API config   │
└──────┬───────────────┘
       │
       ▼
┌──────────────────────┐
│  Rust Core Services  │
│  • Direct invocation │
│  • No HTTP overhead  │
└──────┬───────────────┘
       │
       ▼
┌──────────────────────┐
│  Output Handler      │
│  • Same as API mode  │
│  • Uses config dirs  │
└──────────────────────┘
```

## Security Considerations

### 1. API Security
- **Authentication**: API key validation
- **Rate Limiting**: Token bucket per client
- **Input Validation**: Comprehensive request validation
- **CORS**: Configurable origins
- **TLS**: Required in production

### 2. Content Security
- **Sandbox Isolation**: WASM components in isolated environment
- **Resource Limits**: Memory and CPU caps
- **Timeout Protection**: Request and operation timeouts
- **Injection Prevention**: HTML sanitization

### 3. Session Security
- **Isolation**: Separate browser contexts
- **TTL Management**: Automatic session expiration
- **Secure Storage**: Encrypted Redis connection
- **Cookie Security**: HttpOnly and Secure flags

## Performance Optimizations

### 1. Caching Strategy
```
Request → Check Memory Cache → Check Redis → Fetch → Cache → Return
                 ↓ Hit              ↓ Hit       ↓
              Return            Return      Update Caches
```

### 2. Concurrent Processing
- Worker pool with configurable size
- HTTP/2 multiplexing
- Parallel extraction strategies
- Async job processing

### 3. Resource Management
- Connection pooling (Redis, HTTP)
- Browser instance reuse
- WASM instance pooling
- Memory limits per operation

## Deployment Architectures

### 1. Single Server (Development)
```
┌────────────────────────────────┐
│      Single Host               │
│  ┌──────────────────────────┐ │
│  │   API Server + Workers   │ │
│  └──────────────────────────┘ │
│  ┌──────────────────────────┐ │
│  │   Redis                  │ │
│  └──────────────────────────┘ │
│  ┌──────────────────────────┐ │
│  │   Chromium               │ │
│  └──────────────────────────┘ │
└────────────────────────────────┘
```

### 2. Horizontal Scaling (Production)
```
      ┌─────────────┐
      │ Load        │
      │ Balancer    │
      └──────┬──────┘
             │
    ┌────────┴────────┬────────────┐
    ▼                 ▼            ▼
┌─────────┐      ┌─────────┐   ┌─────────┐
│ API     │      │ API     │   │ API     │
│ Server  │      │ Server  │   │ Server  │
│ Instance│      │ Instance│   │ Instance│
└────┬────┘      └────┬────┘   └────┬────┘
     │                │             │
     └────────┬───────┴─────────────┘
              ▼
     ┌────────────────┐
     │ Redis Cluster  │
     │ (Shared Cache) │
     └────────────────┘
              │
              ▼
     ┌────────────────┐
     │ Headless Pool  │
     │ (Chromium)     │
     └────────────────┘
```

### 3. Microservices (Enterprise)
```
┌─────────────────────────────────────────────┐
│           API Gateway (Kong)                │
└────┬────────────────────────────────────────┘
     │
     ├──────────┬──────────┬─────────────┐
     ▼          ▼          ▼             ▼
┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐
│Crawling │ │Extract  │ │ Search  │ │ Session │
│ Service │ │ Service │ │ Service │ │ Service │
└────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘
     │           │           │           │
     └───────────┴───────────┴───────────┘
                      │
              ┌───────┴────────┐
              │                │
         ┌────▼────┐     ┌────▼────┐
         │  Redis  │     │ Message │
         │ Cluster │     │  Queue  │
         └─────────┘     └─────────┘
```

## Monitoring and Observability

### 1. Metrics Collection
- **Prometheus**: Time-series metrics
- **Custom Metrics**: Pipeline performance
- **Health Checks**: Component availability

### 2. Logging
- Structured JSON logging
- Log levels: DEBUG, INFO, WARN, ERROR
- Request/response logging
- Error stack traces

### 3. Tracing
- Distributed tracing support
- Request ID propagation
- Performance profiling
- Bottleneck identification

## Technology Stack Summary

| Layer | Component | Technology |
|-------|-----------|------------|
| **Client** | CLI | Node.js, TypeScript |
| | Web SDK | JavaScript/TypeScript |
| | Python SDK | Python 3.8+ |
| **API** | REST Server | Axum (Rust) |
| | Runtime | Tokio |
| **Core** | Orchestration | Rust 2021 |
| | Extraction | Rust + WASM |
| | Crawling | Reqwest, HTTP/2 |
| | Headless | Chromiumoxide |
| **Integration** | Cache | Redis 7 |
| | WASM | Wasmtime 34 |
| | Browser | Chromium |
| | LLM | OpenAI, Anthropic, Google |

## Next Steps

- See [Configuration Guide](../configuration/OUTPUT_DIRECTORIES.md) for customization
- Review [Migration Guide](../guides/MIGRATION_GUIDE.md) for upgrades
- Check [Rollout Plan](../ROLLOUT_PLAN.md) for implementation timeline
- Read [Architecture Refactor Summary](../ARCHITECTURE_REFACTOR_SUMMARY.md)
