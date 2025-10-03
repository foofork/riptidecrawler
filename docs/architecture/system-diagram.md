# RipTide Architecture Diagram

## System Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         CLIENT LAYER                                     │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐│
│  │   Python     │  │  JavaScript  │  │     cURL     │  │     Rust     ││
│  │     SDK      │  │     Client   │  │    Command   │  │    Client    ││
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘│
│         │                 │                 │                 │         │
│         │                 │                 │                 │         │
│         └─────────────────┴─────────────────┴─────────────────┘         │
│                                    │                                     │
└────────────────────────────────────┼─────────────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         API GATEWAY (Optional)                           │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐            │
│  │  Rate Limiting │  │  API Key Auth  │  │  Load Balance  │            │
│  └────────────────┘  └────────────────┘  └────────────────┘            │
│                                                                           │
└────────────────────────────────────┬─────────────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         RIPTIDE API (Axum/Rust)                          │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  ┌────────────────────────────────────────────────────────────┐         │
│  │                    ROUTING LAYER                            │         │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  │         │
│  │  │ Crawl    │  │ Streaming│  │  Search  │  │ Sessions │  │         │
│  │  │ (5 EPs)  │  │ (4 EPs)  │  │  (2 EPs) │  │ (12 EPs) │  │         │
│  │  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘  │         │
│  │       │             │              │             │         │         │
│  │  ┌────┴─────┬───────┴──────┬───────┴─────┬───────┴─────┐ │         │
│  │  │ Workers  │  Monitoring  │  Strategies │    Spider   │ │         │
│  │  │ (9 EPs)  │   (6 EPs)    │   (2 EPs)   │   (3 EPs)   │ │         │
│  │  └──────────┴──────────────┴─────────────┴─────────────┘ │         │
│  └────────────────────────────────────────────────────────────┘         │
│                                                                           │
│  ┌────────────────────────────────────────────────────────────┐         │
│  │                    PROCESSING PIPELINE                      │         │
│  │                                                              │         │
│  │  ┌─────────┐     ┌──────────────┐     ┌─────────────┐     │         │
│  │  │  FETCH  │────▶│  GATE ROUTER │────▶│   EXTRACT   │     │         │
│  │  │ Engine  │     │   (Decision)  │     │   Engine    │     │         │
│  │  └─────────┘     └──────────────┘     └─────────────┘     │         │
│  │                          │                                  │         │
│  │                          ▼                                  │         │
│  │              ┌────────────────────────┐                    │         │
│  │              │   Decision Routing     │                    │         │
│  │              ├────────────────────────┤                    │         │
│  │              │ • Fast Path (CSS)      │                    │         │
│  │              │ • Enhanced Path (WASM) │                    │         │
│  │              │ • Headless (Browser)   │                    │         │
│  │              │ • Cached Result        │                    │         │
│  │              └────────────────────────┘                    │         │
│  └────────────────────────────────────────────────────────────┘         │
│                                                                           │
└─────┬───────────────────────────────┬─────────────────────────┬─────────┘
      │                               │                         │
      ▼                               ▼                         ▼
┌──────────────┐              ┌──────────────┐         ┌──────────────┐
│   WASM       │              │    Redis     │         │   Headless   │
│  Extractor   │              │    Cache     │         │   Service    │
├──────────────┤              ├──────────────┤         ├──────────────┤
│ • TREK       │              │ • KV Store   │         │ • Chrome     │
│ • Component  │              │ • TTL        │         │ • Stealth    │
│ • SIMD Opt   │              │ • Pub/Sub    │         │ • JS Exec    │
└──────────────┘              └──────────────┘         └──────────────┘
```

## Component Details

### 1. Client Layer

**Python SDK** (`pip install riptide-client`)
- Full API coverage
- Type hints
- Retry logic
- Session management

**JavaScript Client**
- Fetch/Axios support
- Streaming API
- Promise-based

**CLI Tools**
- cURL examples
- Quick testing
- Scripting

### 2. API Gateway (Optional)

**Kong/Tyk/AWS API Gateway**
- Rate limiting
- Authentication
- Load balancing
- Metrics collection

### 3. RipTide Core

**Routing Layer** (59 endpoints)
- Health & Metrics (2)
- Crawling (5)
- Streaming (4)
- Search (2)
- Spider (3)
- Strategies (2)
- PDF (3)
- Stealth (4)
- Tables (2)
- LLM (4)
- Sessions (12)
- Workers (9)
- Monitoring (6)
- Pipeline (1)

**Processing Pipeline**

1. **FETCH Engine**
   - HTTP client pooling
   - Concurrent requests
   - Timeout management

2. **GATE Router**
   - Content analysis
   - Path decision:
     - Fast Path: Simple HTML → CSS extraction
     - Enhanced Path: Complex content → WASM/AI
     - Headless: JS-heavy → Browser render
     - Cached: Return cached result

3. **EXTRACT Engine**
   - Multi-strategy:
     - CSS selectors
     - WASM/TREK
     - LLM-powered
     - Regex patterns
     - Auto-detection

### 4. External Services

**WASM Extractor**
- WebAssembly Component Model
- SIMD optimization
- 45ms average extraction
- Isolated execution

**Redis Cache**
- Distributed caching
- 40-60% hit rate
- TTL management
- Pub/sub for events

**Headless Service**
- Chromium-based
- Stealth mode
- Screenshot capture
- JavaScript execution

## Data Flow

### Simple Crawl Request

```
User Request
    │
    ▼
┌─────────────────┐
│ POST /crawl     │
│ {               │
│   urls: [...]   │
│ }               │
└────────┬────────┘
         │
         ▼
    ┌────────┐
    │ Redis  │──────▶ Cache Hit? ──▶ Return Cached
    │ Check  │               │
    └────────┘               │ Miss
                             ▼
                        ┌─────────┐
                        │  Fetch  │
                        │  HTML   │
                        └────┬────┘
                             │
                             ▼
                        ┌─────────┐
                        │  Gate   │
                        │ Analyze │
                        └────┬────┘
                             │
                 ┌───────────┼───────────┐
                 │           │           │
            Fast Path   Enhanced   Headless
                 │           │           │
                 ▼           ▼           ▼
            ┌────────┐  ┌────────┐  ┌────────┐
            │  CSS   │  │  WASM  │  │ Chrome │
            │Extract │  │ TREK   │  │ Render │
            └───┬────┘  └───┬────┘  └───┬────┘
                │           │           │
                └───────────┼───────────┘
                            │
                            ▼
                       ┌─────────┐
                       │  Cache  │
                       │  Result │
                       └────┬────┘
                            │
                            ▼
                     Return to User
```

### Streaming Request

```
POST /crawl/stream
    │
    ▼
Open NDJSON Stream
    │
For each URL:
    │
    ├─▶ Process URL ──▶ Stream Result (NDJSON line)
    │
    ├─▶ Process URL ──▶ Stream Result (NDJSON line)
    │
    └─▶ Process URL ──▶ Stream Result (NDJSON line)
    │
Close Stream
```

## Performance Characteristics

### Throughput
- **Concurrent Requests**: 100+/sec
- **Response Time (p50)**: ≤1.5s
- **Response Time (p95)**: ≤5s

### Caching
- **Hit Rate**: 40-60%
- **Backend**: Redis 7+
- **TTL**: Configurable

### Extraction
- **WASM Speed**: ~45ms average
- **CSS Speed**: ~100ms average
- **Headless**: 2-3s average

### Reliability
- **Success Rate**: ≥99.5%
- **Circuit Breaker**: Automatic failover
- **Retry Logic**: Exponential backoff

## Scaling Strategy

### Horizontal Scaling
```
                   ┌────────────┐
                   │  Load      │
                   │  Balancer  │
                   └─────┬──────┘
                         │
         ┌───────────────┼───────────────┐
         │               │               │
    ┌────▼────┐     ┌────▼────┐     ┌────▼────┐
    │ API     │     │ API     │     │ API     │
    │ Server  │     │ Server  │     │ Server  │
    │ (1)     │     │ (2)     │     │ (N)     │
    └────┬────┘     └────┬────┘     └────┬────┘
         │               │               │
         └───────────────┼───────────────┘
                         │
                    ┌────▼────┐
                    │  Redis  │
                    │ Cluster │
                    └─────────┘
```

### Vertical Scaling
- **Optimal**: 4-8 CPU cores
- **Memory**: 8-16GB RAM per instance
- **WASM**: Benefits from multi-core

## Security Architecture

```
Request
   │
   ▼
┌──────────────┐
│ API Gateway  │
│ - Rate Limit │
│ - API Key    │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ Input Valid  │
│ - URL check  │
│ - JSON valid │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ Circuit      │
│ Breaker      │
└──────┬───────┘
       │
       ▼
  Processing
```

## Monitoring Stack

```
RipTide API
     │
     ├─▶ /metrics ──▶ Prometheus ──▶ Grafana
     │
     ├─▶ /healthz ──▶ Health Checks
     │
     └─▶ EventBus ──▶ Structured Logs ──▶ ELK Stack
```

## Deployment Options

### Docker Compose (Development)
```yaml
services:
  riptide-api:
    image: riptide/api:latest
  redis:
    image: redis:7-alpine
  playground:
    image: riptide/playground:latest
```

### Kubernetes (Production)
```
┌─────────────────────────────────────┐
│          Kubernetes Cluster         │
├─────────────────────────────────────┤
│ ┌─────────────────────────────────┐ │
│ │  Ingress (nginx/traefik)        │ │
│ └────────────┬────────────────────┘ │
│              │                       │
│ ┌────────────▼────────────────────┐ │
│ │  Service (riptide-api)          │ │
│ └────────────┬────────────────────┘ │
│              │                       │
│ ┌────────────▼────────────────────┐ │
│ │  Deployment (3 replicas)        │ │
│ │  ┌──────┐ ┌──────┐ ┌──────┐    │ │
│ │  │ Pod1 │ │ Pod2 │ │ Pod3 │    │ │
│ │  └──────┘ └──────┘ └──────┘    │ │
│ └─────────────────────────────────┘ │
│                                      │
│ ┌─────────────────────────────────┐ │
│ │  StatefulSet (Redis Cluster)    │ │
│ └─────────────────────────────────┘ │
└─────────────────────────────────────┘
```

## Next Steps

- [API Documentation](../api/README.md)
- [Deployment Guide](deployment-guide.md)
- [Performance Tuning](../api/performance.md)
- [Security Guide](../api/security.md)
