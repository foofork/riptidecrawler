# Core Concepts

Understanding RipTide's architecture and terminology.

## 🧩 Core Components

### 1. Extractors

**What**: Rules that define how to extract content from HTML.

**Types**:
- **CSS Selectors** - Fast, static extraction (`h1, .article p`)
- **Regex Patterns** - Pattern matching (`\d{3}-\d{3}-\d{4}`)
- **WASM Components** - Custom WebAssembly extractors
- **LLM-Based** - AI-powered content understanding

**Example**:
```json
{
  "type": "css",
  "selector": "article h1",
  "field": "title"
}
```

### 2. Extraction Strategies

**What**: The approach used to extract content.

**Available Strategies**:
- **Static Extraction** - Fast, no JavaScript execution
- **Headless Browser** - Full browser rendering
- **Hybrid** - Static first, browser fallback
- **WASM-Optimized** - WebAssembly acceleration

**RipTide automatically selects the best strategy based on the target site.**

### 3. Dual-Path Pipeline

```
Request → Engine Selection → Static Extraction
                          ↓ (if fails)
                     Browser Rendering → Success
```

**Benefits**:
- Fast for static sites (90% of requests)
- Reliable for dynamic sites (automatic fallback)
- Cost-effective (browser only when needed)

### 4. Streaming Protocols

**NDJSON** (Newline-Delimited JSON):
```bash
curl http://localhost:8080/crawl/stream
```

**Server-Sent Events (SSE)**:
```javascript
const events = new EventSource('/crawl/sse');
events.onmessage = (e) => console.log(e.data);
```

**WebSocket**:
```javascript
const ws = new WebSocket('ws://localhost:8080/crawl/ws');
ws.onmessage = (msg) => processResult(msg.data);
```

### 5. Worker Queue

**What**: Background job processing system.

**Job Types**:
- **SingleCrawl** - Individual URL extraction
- **BatchCrawl** - Multiple URLs
- **PdfExtraction** - PDF processing
- **Maintenance** - System cleanup

**Priority Levels**: Low, Medium, High, Critical

### 6. Browser Pool

**What**: Managed pool of headless browser instances.

**Features**:
- Health monitoring
- Auto-scaling
- Session isolation
- Resource limits

**Configuration**:
```yaml
browser_pool:
  min_instances: 2
  max_instances: 10
  health_check_interval: 30s
```

### 7. Domain Profiling

**What**: Per-domain optimization and caching.

**Capabilities**:
- Warm-start caching
- Structure drift detection
- Optimal strategy selection
- Historical performance tracking

### 8. LLM Abstraction

**What**: Unified interface for multiple AI providers.

**Supported Providers** (8 total):
- OpenAI (GPT-4, GPT-3.5)
- Anthropic (Claude)
- Google Vertex AI (Gemini)
- Azure OpenAI
- AWS Bedrock
- Ollama (local models)
- LocalAI (self-hosted)
- Custom providers (extensible)

**Features**:
- Circuit breakers
- Failover chains
- Cost tracking
- Hot-reload config

---

## 🔄 Request Flow

```
1. Client Request
   ↓
2. Engine Selection (intelligent routing)
   ↓
3. Static Extraction (WASM-accelerated)
   ↓ (if insufficient)
4. Browser Rendering (headless Chrome)
   ↓
5. Content Extraction
   ↓
6. Response Streaming (NDJSON/SSE/WS)
```

---

## 🎯 Key Terminology

| Term | Definition |
|------|------------|
| **Spider** | Deep crawling engine with frontier management |
| **Extractor** | Rule for extracting specific content |
| **Strategy** | Approach for content extraction (static/browser/hybrid) |
| **WASM Component** | WebAssembly module for custom extraction |
| **Session** | Isolated browser context with state |
| **Frontier** | Queue of discovered URLs to crawl |
| **Drift Detection** | Monitoring for website structure changes |
| **Circuit Breaker** | Failure protection for external services |

---

## 📊 Performance Characteristics

| Operation | Latency | Throughput |
|-----------|---------|------------|
| **Static Extraction** | ~50ms | 100 req/s |
| **Browser Rendering** | ~2s | 10 req/s |
| **WASM Processing** | ~10ms | 500 ops/s |
| **Streaming (NDJSON)** | Real-time | Unlimited |

---

## 🔐 Security & Stealth

**Stealth Features**:
- User-agent rotation
- Fingerprint randomization
- Proxy support
- Header spoofing
- Cookie management

**Safety Features**:
- Rate limiting
- Resource quotas
- Timeout enforcement
- Circuit breakers
- Input validation

---

## 🏗️ Architecture Patterns

### Modular Crates (27 total)

**Core**: `riptide-api`, `riptide-cli`, `riptide-facade`
**Extraction**: `riptide-extraction`, `riptide-spider`, `riptide-pdf`
**Infrastructure**: `riptide-workers`, `riptide-pool`, `riptide-cache`
**Intelligence**: `riptide-intelligence`, `riptide-monitoring`

### Event-Driven

All components communicate via event pub/sub:
```rust
events::publish(CrawlCompleted { url, results });
```

### Resource Management

Automatic resource pooling:
- Browser instances
- WASM instances
- HTTP connections
- Redis connections

---

## 💡 Design Philosophy

1. **Fast by Default** - Static extraction first
2. **Reliable Fallbacks** - Browser when needed
3. **Observable** - Prometheus metrics everywhere
4. **Resilient** - Circuit breakers and retries
5. **Scalable** - Horizontal scaling support
6. **Developer-Friendly** - Clear APIs and SDKs

---

**Next**: [Quick Start Guide](./quickstart.md) | [FAQ](./faq.md)
