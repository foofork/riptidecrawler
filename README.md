# RipTide

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-103%20files-green.svg)](tests/)
[![API Docs](https://img.shields.io/badge/API%20docs-59%20endpoints-success.svg)](docs/api/ENDPOINT_CATALOG.md)
[![Coverage](https://img.shields.io/badge/coverage-85%25-brightgreen.svg)](docs/development/testing.md)

**High-performance web crawler and content extraction API built in Rust with WebAssembly optimization.**

RipTide delivers enterprise-grade web crawling with WASM-powered extraction, real-time streaming, and intelligent adaptive routing. Built for scale, security, and performance.

---

## Key Features

### Performance & Scalability
- **WASM-Powered Extraction**: WebAssembly Component Model with adaptive routing for optimal content processing
- **Concurrent Processing**: Configurable worker pools with intelligent load balancing
- **HTTP/2 Optimization**: Prior knowledge support and connection multiplexing
- **Smart Caching**: Redis-backed read-through cache with TTL management
- **Dual-Path Pipeline**: Fast static extraction with intelligent headless browser fallback

### Content Extraction
- **Multi-Strategy Extraction**: Auto-selection, CSS, TREK, LLM, and Regex patterns
- **PDF Processing**: Full pipeline with table extraction and streaming progress
- **Table Extraction**: Structured data extraction with CSV/Markdown export
- **Intelligent Chunking**: Sliding window, topic-based, and sentence-preserving modes
- **Markdown Generation**: Clean, readable markdown output from complex HTML

### Real-Time Streaming
- **NDJSON Streaming**: Newline-delimited JSON for efficient data pipelines
- **Server-Sent Events (SSE)**: Browser-compatible real-time updates
- **WebSocket Support**: Bidirectional communication for interactive crawling
- **Progress Tracking**: Real-time status updates for long-running operations

### Enterprise Features
- **Session Management**: Isolated browsing contexts with cookie persistence
- **Async Job Queue**: Background processing with retry logic and scheduling
- **LLM Abstraction**: Multi-provider support (OpenAI, Anthropic, Google Vertex)
- **Stealth Mode**: Anti-detection with fingerprint randomization
- **Deep Crawling**: Spider engine with frontier management and adaptive strategies
- **Comprehensive Monitoring**: Prometheus metrics, health scores, and bottleneck analysis

---

## Quick Start

### Prerequisites

- **Rust**: 1.75+ with `wasm32-wasip2` target
- **Docker**: 20.10+ with Docker Compose
- **Redis**: 7.0+ (included in Docker setup)
- **Serper API Key**: For search functionality ([Get one here](https://serper.dev))

### Docker Deployment (Recommended)

```bash
# 1. Clone repository
git clone <repository-url>
cd eventmesh

# 2. Configure environment
cp .env.example .env
# Edit .env and add your SERPER_API_KEY

# 3. Start services with Docker Compose
docker-compose up -d

# 4. Verify deployment
curl http://localhost:8080/healthz
```

### Building from Source

```bash
# 1. Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-wasip2

# 2. Clone and setup
git clone <repository-url>
cd eventmesh
export SERPER_API_KEY="your-api-key"

# 3. Build WASM component
cd wasm/riptide-extractor-wasm
cargo build --target wasm32-wasip2 --release
cd ../..

# 4. Build project
cargo build --release

# 5. Start Redis
docker run -d --name redis -p 6379:6379 redis:7-alpine

# 6. Start API service
./target/release/riptide-api --config configs/riptide.yml
```

### First API Request

```bash
# Health check
curl http://localhost:8080/healthz

# Crawl a single URL
curl -X POST http://localhost:8080/crawl \
  -H "Content-Type: application/json" \
  -d '{
    "urls": ["https://example.com"],
    "options": {
      "cache_mode": "auto",
      "concurrency": 5
    }
  }'

# Deep search with content extraction
curl -X POST http://localhost:8080/deepsearch \
  -H "Content-Type: application/json" \
  -d '{
    "query": "rust web scraping",
    "limit": 10,
    "include_content": true
  }'
```

---

## Architecture

### System Overview

RipTide uses a dual-path pipeline architecture for optimal performance:

```
┌─────────────────────────────────────────────────────────────────┐
│                         Client Request                          │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Fetch Phase (HTTP)                         │
│  • Concurrent HTTP/2 requests      • Rate limiting              │
│  • Compression (Gzip/Brotli)       • Robots.txt compliance      │
│  • Redis caching                   • Stealth headers            │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Gate Phase (Decision)                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │ Content-Type │  │  JS Analysis │  │ Quality Score│          │
│  │   Detection  │  │   Heuristics │  │  Calculation │          │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘          │
│         └──────────────────┴─────────────────┘                  │
│                            │                                     │
│         ┌──────────────────┼──────────────────┐                 │
│         ▼                  ▼                  ▼                 │
│    ┌────────┐       ┌──────────┐       ┌──────────┐            │
│    │  Raw   │       │  WASM    │       │ Headless │            │
│    │ (Fast) │       │ (Optimal)│       │ (Render) │            │
│    └────┬───┘       └────┬─────┘       └────┬─────┘            │
└─────────┼────────────────┼──────────────────┼──────────────────┘
          │                │                  │
          ▼                ▼                  ▼
┌─────────────────────────────────────────────────────────────────┐
│                   Extract Phase (Processing)                    │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │              WASM Extraction Engine                      │   │
│  │  • Adaptive strategy selection  • Multi-format output   │   │
│  │  • Content cleaning             • Metadata extraction   │   │
│  │  • Intelligent chunking         • Link extraction       │   │
│  └──────────────────────────────────────────────────────────┘   │
│                              │                                   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │           Chromiumoxide (Headless Browser)               │   │
│  │  • JavaScript rendering         • Screenshot capture    │   │
│  │  • Dynamic content              • Network interception  │   │
│  │  • Stealth mode                 • Cookie management     │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Response (JSON/Stream)                      │
│  • Structured JSON          • Real-time streaming (NDJSON/SSE) │
│  • Quality metrics          • Pipeline performance data        │
│  • Cache metadata           • Error handling                   │
└─────────────────────────────────────────────────────────────────┘
```

### Technology Stack

| Component | Technology | Purpose |
|-----------|-----------|---------|
| **Language** | Rust 2021 Edition | Memory safety, concurrency, performance |
| **Web Framework** | Axum 0.7 | High-performance async HTTP |
| **Async Runtime** | Tokio | Multi-threaded async execution |
| **WASM Runtime** | Wasmtime 34 | Component Model extraction engine |
| **Browser** | Chromiumoxide | Headless Chrome automation |
| **Cache** | Redis 7 | Distributed caching and session store |
| **HTTP Client** | Reqwest 0.12 | HTTP/2, compression, TLS |
| **Search** | Serper API | Web search integration |

---

## Workspace Structure

RipTide is organized as a Cargo workspace with 13 specialized crates:

| Crate | Description |
|-------|-------------|
| **riptide-api** | REST API server with 59 endpoints across 13 categories |
| **riptide-core** | Core crawling engine, orchestration, and shared utilities |
| **riptide-html** | HTML parsing, content extraction, and markdown generation |
| **riptide-search** | Web search integration with circuit breaker and provider abstraction |
| **riptide-headless** | Headless browser service with Chromiumoxide and stealth mode |
| **riptide-workers** | Background job queue with Redis-backed persistence |
| **riptide-intelligence** | Multi-provider LLM abstraction (OpenAI, Anthropic, Google) |
| **riptide-persistence** | Session management and data persistence layer |
| **riptide-streaming** | Real-time streaming protocols (NDJSON, SSE, WebSocket) |
| **riptide-stealth** | Anti-detection capabilities and fingerprint randomization |
| **riptide-pdf** | PDF processing pipeline with table extraction |
| **riptide-performance** | Monitoring, metrics collection, and bottleneck analysis |
| **riptide-extractor-wasm** | WebAssembly content extraction component |

---

## API Overview

**59 endpoints** across **13 categories** with 100% documentation coverage.

| Category | Endpoints | Key Features |
|----------|-----------|--------------|
| **Health & Metrics** | 2 | System health, Prometheus metrics, dependency status |
| **Core Crawling** | 5 | Batch crawling, rendering, adaptive gate, caching |
| **Search** | 2 | Deep search, web integration, content extraction |
| **Streaming** | 4 | NDJSON, SSE, WebSocket real-time data |
| **Spider** | 3 | Deep crawling, frontier management, adaptive strategies |
| **Strategies** | 2 | Multi-strategy extraction, intelligent chunking |
| **PDF Processing** | 3 | PDF extraction, table parsing, progress streaming |
| **Stealth** | 4 | Bot evasion, fingerprint randomization, effectiveness testing |
| **Table Extraction** | 2 | HTML table parsing, CSV/Markdown export |
| **LLM Providers** | 4 | Multi-provider support, runtime switching, config management |
| **Sessions** | 12 | Cookie persistence, TTL management, isolated contexts |
| **Workers & Jobs** | 9 | Async processing, scheduling, retry logic, queue management |
| **Monitoring** | 6 | Health scores, alerts, performance reports, bottleneck analysis |

**Full API documentation**: [Endpoint Catalog](docs/api/ENDPOINT_CATALOG.md) | [OpenAPI Spec](docs/api/openapi.yaml)

---

## Testing & Quality Assurance

### Test Coverage

- **103 test files** across the workspace
- **85%+ code coverage** with unit, integration, and golden tests
- **Continuous integration** with GitHub Actions
- **Security audits** with `cargo audit` and `cargo deny`

### Running Tests

```bash
# All tests
cargo test

# Specific crate
cargo test -p riptide-core

# Integration tests (requires Redis)
docker run -d --name redis -p 6379:6379 redis:7-alpine
cargo test --test '*'

# With coverage
cargo install cargo-tarpaulin
cargo tarpaulin --out Html --output-dir coverage

# WASM component tests
cd wasm/riptide-extractor-wasm
cargo test --target wasm32-wasip2
```

### Code Quality

```bash
# Format code
cargo fmt

# Linting
cargo clippy -- -D warnings

# Security audit
cargo audit

# License and dependency checking
cargo deny check
```

---

## Production Deployment

### Docker Compose (Recommended)

```yaml
version: '3.8'

services:
  riptide-api:
    image: riptide/api:latest
    ports:
      - "8080:8080"
    environment:
      - SERPER_API_KEY=${SERPER_API_KEY}
      - REDIS_URL=redis://redis:6379
      - RUST_LOG=info
    depends_on:
      - redis
      - riptide-headless

  riptide-headless:
    image: riptide/headless:latest
    ports:
      - "9123:9123"
    environment:
      - RUST_LOG=info

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis-data:/data

volumes:
  redis-data:
```

### Kong API Gateway Integration

RipTide includes Kong Gateway configuration for production deployments:

```bash
# Start with Kong Gateway
docker-compose -f docker-compose.gateway.yml up -d

# Features:
# - Rate limiting (100 req/min)
# - Authentication (API key)
# - Request/response logging
# - CORS handling
# - Load balancing
```

**Configuration**: [docker-compose.gateway.yml](docker-compose.gateway.yml)

### Environment Variables

```bash
# Required
SERPER_API_KEY=your-serper-api-key

# Optional - Redis
REDIS_URL=redis://localhost:6379/0
REDIS_POOL_SIZE=10
REDIS_TIMEOUT_MS=5000

# Optional - Performance
CRAWL_CONCURRENCY=16
WORKER_THREADS=8
REQUEST_TIMEOUT_MS=30000

# Optional - Logging
RUST_LOG=info,riptide_core=debug
LOG_FORMAT=json

# Optional - Headless
HEADLESS_URL=http://localhost:9123
HEADLESS_POOL_SIZE=5

# Optional - LLM (if using extraction)
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
GOOGLE_VERTEX_CREDENTIALS=/path/to/credentials.json
```

**Full configuration guide**: [docs/architecture/configuration-guide.md](docs/architecture/configuration-guide.md)

---

## Performance & Monitoring

### Built-in Metrics

RipTide provides comprehensive performance monitoring:

```bash
# Prometheus metrics
curl http://localhost:8080/metrics

# Health score (0-100)
curl http://localhost:8080/monitoring/health-score

# Performance report with recommendations
curl http://localhost:8080/monitoring/performance-report

# Pipeline phase metrics and bottleneck analysis
curl http://localhost:8080/pipeline/phases
```

### Key Performance Indicators

| Metric | Target | Monitoring Endpoint |
|--------|--------|---------------------|
| **Request Latency (p95)** | < 500ms | `/pipeline/phases` |
| **Cache Hit Rate** | > 70% | `/monitoring/metrics/current` |
| **Error Rate** | < 1% | `/monitoring/performance-report` |
| **Throughput** | > 100 req/s | `/healthz` |
| **Memory Usage** | < 2GB | `/healthz` |

### Bottleneck Analysis

The `/pipeline/phases` endpoint provides detailed phase-by-phase metrics:

```json
{
  "phases": [
    {"name": "fetch", "avg_duration_ms": 150, "percentage_of_total": 41.7},
    {"name": "gate", "avg_duration_ms": 10, "percentage_of_total": 2.8},
    {"name": "wasm", "avg_duration_ms": 200, "percentage_of_total": 55.6}
  ],
  "bottlenecks": [
    {
      "phase": "wasm",
      "severity": "medium",
      "recommendation": "Consider scaling WASM instances"
    }
  ]
}
```

**Monitoring guide**: [docs/performance-monitoring.md](docs/performance-monitoring.md)

---

## Memory Profiling

RipTide includes advanced memory profiling with jemalloc:

```bash
# Enable profiling
cargo build --release --features profiling-full

# Generate memory report
curl http://localhost:8080/profiling/memory/report

# Heap dump
curl http://localhost:8080/profiling/memory/heap > heap.dump

# Live profiling with flamegraphs
cargo build --release --features jemalloc
./target/release/riptide-api --profiling-enabled
```

**Features**:
- Real-time memory tracking
- Heap dump generation
- Flamegraph visualization
- Leak detection
- Allocation statistics

**Profiling guide**: [docs/profiling-integration-complete.md](docs/profiling-integration-complete.md)

---

## Documentation

### User Documentation
- [Installation Guide](docs/user/installation.md) - Setup and deployment
- [API Usage](docs/user/api-usage.md) - Request/response examples
- [API Tooling Quickstart](docs/API_TOOLING_QUICKSTART.md) - CLI, SDK, playground
- [Troubleshooting](docs/user/troubleshooting.md) - Common issues and solutions

### Developer Documentation
- [Getting Started](docs/development/getting-started.md) - Development setup
- [Coding Standards](docs/development/coding-standards.md) - Style guide and best practices
- [Contributing](docs/development/contributing.md) - Contribution guidelines
- [Testing](docs/development/testing.md) - Test suite documentation

### Architecture Documentation
- [System Overview](docs/architecture/system-overview.md) - Complete architecture
- [WASM Guide](docs/architecture/WASM_GUIDE.md) - WebAssembly integration
- [PDF Pipeline](docs/architecture/PDF_PIPELINE_GUIDE.md) - PDF processing architecture
- [Configuration Guide](docs/architecture/configuration-guide.md) - Configuration reference
- [Deployment Guide](docs/architecture/deployment-guide.md) - Production deployment

### API Reference
- [Endpoint Catalog](docs/api/ENDPOINT_CATALOG.md) - All 59 endpoints
- [OpenAPI Spec](docs/api/openapi.yaml) - Machine-readable API spec
- [Examples](docs/api/examples.md) - Usage examples
- [Streaming Guide](docs/api/streaming.md) - Real-time protocols
- [Session Management](docs/api/session-management.md) - Session handling

### Tools & SDKs
- [CLI Tool](cli/README.md) - Command-line interface (npm)
- [Python SDK](python-sdk/README.md) - Official Python client (PyPI)
- [Web Playground](playground/README.md) - Interactive API explorer (React)

---

## Roadmap

**Current Status**: v0.1.0 - **82% Production Ready**

### Completed Features ✓

- [x] Core crawling engine with adaptive routing
- [x] WASM-powered content extraction
- [x] Dual-path pipeline (fast static + headless fallback)
- [x] Real-time streaming (NDJSON, SSE, WebSocket)
- [x] PDF processing with table extraction
- [x] Multi-provider LLM abstraction
- [x] Session management and persistence
- [x] Background job queue with retry logic
- [x] Comprehensive monitoring and metrics
- [x] Docker deployment with Kong Gateway
- [x] 59 API endpoints with 100% documentation
- [x] 85%+ test coverage

### In Progress 🚧

- [ ] Distributed crawling coordination
- [ ] Enhanced analytics dashboard
- [ ] GraphQL API endpoint
- [ ] Advanced rate limiting strategies

### Planned 📋

- [ ] Kubernetes Helm charts
- [ ] TypeScript SDK
- [ ] Webhook notifications
- [ ] Custom extraction plugin system
- [ ] Multi-region deployment support

**Full roadmap**: [docs/ROADMAP.md](docs/ROADMAP.md)

---

## Contributing

We welcome contributions! Please see our [Contributing Guide](docs/development/contributing.md) for details.

### Development Workflow

1. **Fork** the repository on GitHub
2. **Create** a feature branch: `git checkout -b feature/amazing-feature`
3. **Make** your changes and add tests
4. **Test** locally: `cargo test && cargo clippy`
5. **Commit** with descriptive messages following [Conventional Commits](https://www.conventionalcommits.org/)
6. **Push** to your fork and create a Pull Request

### Code Standards

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Maintain test coverage above 80%
- Run `cargo fmt` and `cargo clippy` before committing
- Update documentation for API changes
- Add tests for new features
- Keep PRs focused and atomic

---

## Security

### Security Features

- **Stealth Mode**: Anti-detection with fingerprint randomization
- **Rate Limiting**: Configurable request throttling
- **Input Validation**: Comprehensive request validation
- **Secure Headers**: CORS, CSP, HSTS support
- **Redis Security**: TLS support and authentication
- **Dependency Auditing**: Automated security checks with `cargo audit`

### Reporting Security Issues

Please report security vulnerabilities to **security@riptide.dev** (or create a private security advisory on GitHub).

**Do not** open public issues for security vulnerabilities.

---

## License

This project is licensed under the **Apache License 2.0** - see the [LICENSE](LICENSE) file for details.

### Third-Party Licenses

RipTide uses the following major open-source components:

- **Rust Standard Library** - MIT/Apache-2.0
- **Tokio** - MIT
- **Axum** - MIT
- **Wasmtime** - Apache-2.0
- **Chromiumoxide** - MIT
- **Redis** - BSD-3-Clause

---

## Acknowledgments

Built with excellent open-source projects:

- [Tokio](https://tokio.rs/) - Async runtime
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [Wasmtime](https://wasmtime.dev/) - WASM runtime
- [Chromiumoxide](https://github.com/mattsse/chromiumoxide) - Browser automation
- [Redis](https://redis.io/) - Caching and persistence
- [Rust](https://www.rust-lang.org/) - Programming language

---

## Support & Community

- **Documentation**: [docs/](docs/) directory
- **Issues**: [GitHub Issues](https://github.com/your-org/riptide/issues) for bug reports
- **Discussions**: [GitHub Discussions](https://github.com/your-org/riptide/discussions) for questions
- **Changelog**: [CHANGELOG.md](CHANGELOG.md) for version history

---

**Made with ⚡ by the RipTide Team**
