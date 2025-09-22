# RipTide Crawler - System Architecture Overview

## Project Overview

RipTide Crawler is a high-performance web crawling and content extraction system built in Rust, designed to handle large-scale web scraping operations with advanced content processing capabilities.

## Architecture Summary

RipTide follows a modular, microservices-based architecture with the following core components:

```
┌─────────────────────────────────────────────────────────────────┐
│                    RipTide Crawler Ecosystem                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────┐  │
│  │ riptide-api │────│ riptide-    │────│ riptide-extractor-  │  │
│  │ (REST API)  │    │ headless    │    │ wasm (Component)    │  │
│  │ Port: 8080  │    │ (CDP/Chrome)│    │ WASM Content Proc.  │  │
│  └─────────────┘    │ Port: 9123  │    └─────────────────────┘  │
│         │            └─────────────┘              │             │
│         │                   │                    │             │
│         └─────────┬─────────┴────────────────────┘             │
│                   │                                            │
│            ┌─────────────┐    ┌─────────────────────────────┐   │
│            │ riptide-    │    │          Redis              │   │
│            │ core        │    │     (Cache & Queue)         │   │
│            │ (Shared)    │    │      Port: 6379             │   │
│            └─────────────┘    └─────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Component Architecture

### 1. riptide-core (Shared Library)
**Purpose**: Core functionality and shared types

**Key Modules**:
- `cache.rs` - Redis-backed caching layer
- `component.rs` - WASM component integration
- `extract.rs` - Content extraction logic
- `fetch.rs` - HTTP client and fetching utilities
- `gate.rs` - Content routing and decision making
- `types.rs` - Shared data structures

**Dependencies**: Redis, WASM runtime (wasmtime), HTTP client (reqwest)

### 2. riptide-api (REST API Service)
**Purpose**: Main API gateway for crawl requests

**Endpoints**:
- `GET /healthz` - Health check
- `POST /crawl` - Batch URL crawling
- `POST /deepsearch` - Search-driven crawling

**Port**: 8080
**Architecture**: Axum-based async web server
**Features**: CORS, compression, tracing, health checks

### 3. riptide-headless (Browser Service)
**Purpose**: Chrome DevTools Protocol (CDP) browser automation

**Endpoints**:
- `POST /render` - Dynamic content rendering

**Port**: 9123
**Technology**: Chromiumoxide for CDP interaction
**Use Case**: JavaScript-heavy sites requiring browser rendering

### 4. riptide-extractor-wasm (WASM Component)
**Purpose**: High-performance content extraction using WebAssembly

**Architecture**: WASM Component Model (wit-bindgen)
**Features**:
- Article extraction
- Markdown conversion
- JSON structured output
- Token chunking

### 5. Infrastructure Services

#### Redis (Cache & Queue)
- **Purpose**: Caching crawled content and task queueing
- **Port**: 6379
- **Features**: Health checks, data persistence
- **Configuration**: `/configs/riptide.yml`

## Data Flow Architecture

```
┌─────────────┐
│   Client    │
│  Request    │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ riptide-api │ ◄── Configuration (riptide.yml)
│   Gateway   │
└──────┬──────┘
       │
       ▼
┌─────────────┐     ┌─────────────┐
│    Gate     │────▶│    Cache    │
│  (Router)   │     │   (Redis)   │
└──────┬──────┘     └─────────────┘
       │
       ├─────► Static Content ────┐
       │                         │
       └─────► Dynamic Content    │
                      │           │
                      ▼           │
              ┌─────────────┐     │
              │ riptide-    │     │
              │ headless    │     │
              │ (Chrome)    │     │
              └──────┬──────┘     │
                     │           │
                     └───────────┤
                                 │
                                 ▼
                         ┌─────────────┐
                         │ riptide-    │
                         │ extractor-  │
                         │ wasm        │
                         └──────┬──────┘
                                │
                                ▼
                         ┌─────────────┐
                         │ Extracted   │
                         │ Content     │
                         │ (MD/JSON)   │
                         └─────────────┘
```

## Configuration Management

### Primary Configuration: `/configs/riptide.yml`

**Key Sections**:
- **Search**: Serper.dev API integration
- **Crawl**: HTTP client settings, concurrency, timeouts
- **Extraction**: WASM module configuration, tokenization
- **Dynamic**: Headless browser settings, stealth mode
- **Redis**: Connection and caching configuration
- **Artifacts**: File storage settings

### Environment Variables
- `SERPER_API_KEY` - Search API key
- `RUST_LOG` - Logging level
- `REDIS_URL` - Redis connection string
- `HEADLESS_URL` - Browser service endpoint

## Deployment Architecture

### Docker Compose Services

```yaml
services:
  redis:     # Cache & queue backend
  api:       # Main REST API (riptide-api)
  headless:  # Browser service (riptide-headless)
```

**Network Architecture**:
- External ports: 8080 (API), 6379 (Redis), 9123 (Headless)
- Internal communication via Docker network
- Health checks for all services
- Resource limits for headless service

## Technology Stack

### Core Technologies
- **Language**: Rust (Edition 2021)
- **Web Framework**: Axum 0.7
- **Async Runtime**: Tokio
- **Browser Automation**: Chromiumoxide
- **WASM Runtime**: Wasmtime 26 with Component Model
- **Caching**: Redis 7
- **HTTP Client**: Reqwest 0.12
- **Serialization**: Serde

### Key Features
- **Performance**: Concurrent processing (16 default concurrency)
- **Reliability**: Health checks, timeouts, error handling
- **Scalability**: Microservices architecture
- **Security**: Stealth mode, proxy support, robots.txt compliance
- **Extensibility**: WASM-based content processing

## Security & Compliance

### Web Scraping Ethics
- **Robots.txt**: Configurable compliance (`robots_policy: obey`)
- **Rate Limiting**: Configurable delays and timeouts
- **User Agent**: Rotation and stealth capabilities
- **Proxies**: Optional proxy support

### Data Security
- **No Hardcoded Secrets**: Environment variable based configuration
- **Secure Headers**: CORS, compression layers
- **Input Validation**: Request/response type safety
- **Error Handling**: Structured error responses

## Performance Characteristics

### Optimizations
- **Concurrent Processing**: 16 default workers
- **HTTP/2**: Prior knowledge optimization
- **Compression**: Gzip, Brotli support
- **Caching**: Redis-backed read-through cache
- **WASM**: High-performance content extraction

### Scalability
- **Horizontal**: Multiple API instances behind load balancer
- **Vertical**: Configurable concurrency and resource limits
- **Storage**: Persistent Redis data volumes
- **Processing**: Stateless service design

## Monitoring & Observability

### Logging
- **Structured Logging**: JSON format via tracing-subscriber
- **Log Levels**: Configurable via `RUST_LOG`
- **Distributed Tracing**: Request correlation across services

### Health Monitoring
- **Health Endpoints**: `/healthz` on all services
- **Docker Health Checks**: Automated container monitoring
- **Dependency Checks**: Redis connectivity validation

## Future Architecture Considerations

### Planned Enhancements
1. **Worker Scaling**: Dedicated worker service (`riptide-workers`)
2. **Queue Management**: Advanced job scheduling
3. **Content Pipeline**: Enhanced WASM processing
4. **Monitoring**: Metrics and alerting
5. **API Versioning**: Backward compatibility

### Extension Points
- **Custom Extractors**: Additional WASM modules
- **Storage Backends**: Alternative to Redis
- **Authentication**: API key management
- **Rate Limiting**: Advanced throttling mechanisms