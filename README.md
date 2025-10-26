# RipTide

[![Version](https://img.shields.io/badge/version-0.9.0-blue.svg)](CHANGELOG.md)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-1.5k%2B-green.svg)](docs/development/testing.md)
[![Build](https://img.shields.io/badge/build-passing-brightgreen.svg)]()

> **High-performance web crawling and content extraction platform built in Rust with WebAssembly optimization.**

Extract structured data from any website with intelligent routing, multi-provider LLM support, and real-time streaming. Built for speed, reliability, and scale.

---

## ✨ Why RipTide?

- 🚀 **Blazing Fast** - WASM-accelerated extraction with intelligent static/browser routing
- 🧠 **AI-Powered** - 8 LLM providers (OpenAI, Anthropic, Google, Azure, AWS, Ollama) with automatic failover
- 🎯 **Smart Crawling** - Deep spider engine with relevance-based frontier management
- 🔄 **Real-Time Streams** - NDJSON, SSE, and WebSocket protocols with backpressure control
- 🛡️ **Production Ready** - Redis caching, session management, health monitoring, and graceful degradation
- 🧩 **Modular Architecture** - 27-crate workspace with clean separation of concerns

---

## 🎯 Key Features

### Extraction & Processing
- **WASM-Powered Extraction** - WebAssembly Component Model (Wasmtime 37) with AOT caching
- **Dual-Path Pipeline** - Static extraction (90% of requests) with headless browser fallback
- **Multi-Strategy Extraction** - CSS selectors, LLM-based, Regex, with auto-selection
- **PDF Processing** - Full text extraction with table parsing and streaming support
- **Intelligent Routing** - HTML complexity analysis for optimal extraction strategy

### Crawling & Search
- **Deep Spider Engine** - Configurable depth, breadth-first/depth-first strategies
- **Query-Aware Crawling** - Relevance-based frontier management for focused crawls
- **Search Integration** - Serper API with circuit breaker and result parsing
- **Domain Profiling** - Warm-start caching with per-domain configuration

### Reliability & Scale
- **Real-Time Streaming** - NDJSON, Server-Sent Events, WebSocket protocols
- **Smart Caching** - Redis-backed with TTL management and compression
- **Session Management** - Isolated browser contexts with cookie/localStorage persistence
- **Async Job Queue** - Background processing with retry logic and priority scheduling
- **Health Monitoring** - Prometheus metrics, bottleneck analysis, auto-recovery

### AI & Intelligence
- **8 LLM Providers** - OpenAI, Anthropic, Google Vertex, Azure OpenAI, AWS Bedrock, Ollama, LocalAI
- **Automatic Failover** - Circuit breaker, retry policies, provider priority
- **Cost Tracking** - Token usage monitoring and budget controls
- **Stealth Mode** - Anti-detection with fingerprint randomization and human-like behavior

---

## 🚀 Quick Start

### Prerequisites

- **Rust 1.75+** with `wasm32-wasip2` target
- **Docker 20.10+** with Docker Compose
- **Redis 7.0+** (included in Docker setup)
- **Serper API Key** ([free tier available](https://serper.dev))

### Option 1: Docker Deployment (Recommended)

Get up and running in 5 minutes:

```bash
# Clone and configure
git clone <repository-url>
cd eventmesh
cp .env.example .env
# Edit .env and add your SERPER_API_KEY

# Start all services
make docker-build-all
make docker-up

# Verify health
curl http://localhost:8080/healthz
# {"status":"healthy","version":"0.9.0"}

# Access interactive playground
open http://localhost:3000
```

### Option 2: Build from Source

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-wasip2

# Build WASM component and API
make build-wasm
make build-release

# Start Redis
docker run -d --name redis -p 6379:6379 redis:7-alpine

# Run RipTide
./target/release/riptide-api --config configs/riptide.yml
```

---

## 💻 Usage Examples

### REST API

```bash
# Health check
curl http://localhost:8080/healthz

# Extract content from a URL
curl -X POST http://localhost:8080/crawl \
  -H "Content-Type: application/json" \
  -d '{
    "urls": ["https://example.com"],
    "options": {
      "concurrency": 5,
      "cache_mode": "default"
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

# Spider crawl with depth limit
curl -X POST http://localhost:8080/spider/crawl \
  -H "Content-Type: application/json" \
  -d '{
    "base_url": "https://docs.rs",
    "max_depth": 3,
    "max_pages": 50
  }'

# Stream results in real-time (NDJSON)
curl -X POST http://localhost:8080/crawl/stream \
  -H "Content-Type: application/json" \
  -d '{"urls": ["https://example.com"]}'
```

### Python SDK

```bash
# Install SDK
pip install riptide-sdk

# Or from source
cd sdk/python && pip install -e .
```

```python
from riptide_sdk import RipTideClient
import asyncio

async def main():
    async with RipTideClient(base_url="http://localhost:8080") as client:
        # Batch crawl
        result = await client.crawl.batch([
            "https://example.com",
            "https://example.org"
        ])
        print(f"✓ Crawled {result.successful}/{result.total_urls} URLs")

        # Deep search
        results = await client.search.deep(
            query="rust web framework",
            limit=10,
            include_content=True
        )
        for item in results:
            print(f"📄 {item.title}: {item.url}")

        # Spider crawl with streaming
        async for page in client.spider.crawl_stream(
            base_url="https://docs.rs",
            max_depth=2
        ):
            print(f"✓ Crawled: {page.url} ({page.status})")

asyncio.run(main())
```

### CLI Usage

```bash
# Install CLI
cargo install --path crates/riptide-cli

# Crawl and save results
riptide crawl https://example.com -o results.json

# Deep search
riptide search "rust async programming" --limit 20

# Spider crawl
riptide spider https://docs.rs --depth 3 --pages 100
```

---

## 📚 Documentation

### 🚀 Getting Started
- **[Quick Start Guide](docs/00-getting-started/README.md)** - Get up and running in 5 minutes
- **[Core Concepts](docs/00-getting-started/concepts.md)** - Architecture and terminology
- **[FAQ](docs/00-getting-started/faq.md)** - Common questions answered

### 🛠️ Developer Tools
- **[Python SDK](sdk/python/README.md)** - Async Python SDK with full type hints
- **[CLI Reference](crates/riptide-cli/README.md)** - Command-line interface
- **[Interactive Playground](playground/README.md)** - React-based API testing interface
- **[Makefile Commands](Makefile)** - 40+ development shortcuts

### 📖 API Reference
- **[Endpoint Catalog](docs/02-api-reference/ENDPOINT_CATALOG.md)** - 120+ routes, 59 primary endpoints
- **[OpenAPI Specification](docs/02-api-reference/openapi.yaml)** - Complete API schema
- **[Streaming Protocols](docs/02-api-reference/streaming.md)** - NDJSON, SSE, WebSocket
- **[LLM Provider Setup](docs/01-guides/setup/LLM_PROVIDER_SETUP.md)** - Configure AI providers

### 🏗️ Architecture
- **[System Overview](docs/04-architecture/ARCHITECTURE.md)** - High-level architecture
- **[27-Crate Workspace](Cargo.toml)** - Modular design
- **[WASM Integration](docs/04-architecture/components/wasm-architecture.md)** - WebAssembly details
- **[Testing Strategy](docs/development/testing.md)** - 1,500+ tests

### 🚀 Deployment
- **[Production Deployment](docs/01-guides/operations/PRODUCTION_DEPLOYMENT_CHECKLIST.md)** - Production checklist
- **[Docker Setup](docker-compose.yml)** - Container orchestration
- **[Configuration Guide](docs/04-architecture/components/ENVIRONMENT-CONFIGURATION-ANALYSIS.md)** - 400+ env vars
- **[Monitoring Setup](docs/performance-monitoring.md)** - Prometheus and telemetry

**📘 [Browse All Documentation →](docs/)**

---

## 🏗️ Tech Stack

### Core Technologies
- **Language**: Rust 2021 Edition (1.75+)
- **Runtime**: Tokio async runtime with multi-threading
- **Web Framework**: Axum 0.7 with Tower middleware
- **WASM**: Wasmtime 37 with Component Model

### Data & Caching
- **Cache**: Redis 7.0+ with pipelining and compression
- **Serialization**: Serde JSON with streaming support
- **HTTP Client**: Reqwest with connection pooling

### Browser & Extraction
- **Headless Browser**: Spider Chrome (Chromium-based CDP)
- **HTML Parsing**: Scraper, lol_html for fast parsing
- **PDF Processing**: pdfium-render with table extraction

### AI & Intelligence
- **LLM Integration**: 8 providers (OpenAI, Anthropic, Google Vertex, Azure, AWS Bedrock, Ollama, LocalAI)
- **Resilience**: Circuit breakers, retry policies, failover

### Monitoring & Observability
- **Metrics**: Prometheus with custom exporters
- **Tracing**: OpenTelemetry with OTLP export
- **Logging**: Tracing-subscriber with JSON output

### Development
- **Testing**: 1,500+ tests with coverage tracking
- **Build**: Cargo workspace with 27 crates
- **CI/CD**: GitHub Actions, Docker multi-stage builds

---

## 🚢 Deployment Options

### Docker Compose (Recommended for Development)

```bash
# One-command deployment
make docker-build-all && make docker-up

# View logs
make docker-logs

# Stop all services
make docker-down
```

**Services Included:**
- ✅ RipTide API (port 8080)
- ✅ Redis cache (port 6379)
- ✅ Interactive Playground (port 3000)
- ✅ Headless browser pool

### Production Docker

```yaml
services:
  riptide-api:
    image: riptide/api:0.9.0
    ports: ["8080:8080"]
    environment:
      - SERPER_API_KEY=${SERPER_API_KEY}
      - REDIS_URL=redis://redis:6379
      - RUST_LOG=info
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/healthz"]
      interval: 30s
      timeout: 10s
      retries: 3
    depends_on:
      redis:
        condition: service_healthy

  redis:
    image: redis:7-alpine
    ports: ["6379:6379"]
    volumes:
      - redis-data:/data
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5

volumes:
  redis-data:
```

### Kubernetes (Production)

```yaml
# See docs/01-guides/operations/PRODUCTION_DEPLOYMENT_CHECKLIST.md
# for complete Kubernetes manifests with:
# - Horizontal Pod Autoscaling
# - Resource limits and requests
# - Liveness and readiness probes
# - ConfigMaps and Secrets
# - Persistent volume claims
```

### Build Profiles

```bash
# Production (optimized for performance)
cargo build --release --profile release

# CI (optimized for build time)
cargo build --profile ci

# WASM (optimized for size)
make build-wasm --profile wasm
```

---

## 🤝 Contributing

We welcome contributions! Here's how to get started:

```bash
# Fork and clone the repository
git clone https://github.com/your-username/eventmesh.git
cd eventmesh

# Create a feature branch
git checkout -b feature/amazing-feature

# Make your changes and run tests
cargo test --workspace
cargo clippy --all-targets --all-features

# Format code
cargo fmt --all

# Commit and push
git commit -m "Add amazing feature"
git push origin feature/amazing-feature
```

### Development Guidelines
- Write tests for new features
- Follow Rust API guidelines
- Update documentation
- Keep PRs focused and small
- Add examples when appropriate

See **[CONTRIBUTING.md](CONTRIBUTING.md)** for detailed guidelines.

---

## 📊 Project Stats

- **Lines of Code**: ~50,000+ lines of Rust
- **Crates**: 27 modular workspace crates
- **Tests**: 1,500+ unit and integration tests
- **API Routes**: 120+ endpoints
- **LLM Providers**: 8 supported providers
- **Documentation**: 100+ markdown files

---

## 🗺️ Roadmap

### Current Version (0.9.0)
- ✅ WASM-powered extraction with Component Model
- ✅ 8 LLM providers with automatic failover
- ✅ Real-time streaming (NDJSON, SSE, WebSocket)
- ✅ Deep spider crawling with relevance scoring
- ✅ Session management and artifact storage

### Upcoming (1.0.0)
- 🔄 GraphQL API support
- 🔄 Distributed crawling with job queue
- 🔄 Machine learning-based extraction hints
- 🔄 Enhanced anti-detection capabilities
- 🔄 Performance benchmarks and optimizations

### Future
- 🎯 Kubernetes operator for auto-scaling
- 🎯 Browser extension for manual extraction
- 🎯 Visual extraction rule builder
- 🎯 Data pipeline integrations (Kafka, S3, etc.)

See **[ROADMAP.md](ROADMAP.md)** for detailed plans.

---

## 📄 License

**Apache License 2.0** - see [LICENSE](LICENSE)

### Key Dependencies
- **Tokio** (MIT) - Async runtime
- **Axum** (MIT) - Web framework
- **Wasmtime** (Apache-2.0) - WebAssembly runtime
- **Spider Chrome** (MIT) - Browser automation
- **Redis** (BSD-3-Clause) - Caching

---

## 🙏 Acknowledgments

Built with these amazing open-source projects:
- [Rust](https://www.rust-lang.org/) - Systems programming language
- [Tokio](https://tokio.rs/) - Async runtime
- [Wasmtime](https://wasmtime.dev/) - WASM runtime
- [Scraper](https://github.com/causal-agent/scraper) - HTML parsing
- [Axum](https://github.com/tokio-rs/axum) - Web framework

---

## 📞 Support & Community

- **Documentation**: [docs/](docs/)
- **Issues**: [GitHub Issues](https://github.com/your-org/eventmesh/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-org/eventmesh/discussions)
- **Security**: See [SECURITY.md](SECURITY.md) for reporting vulnerabilities

---

<div align="center">

**Built with ⚡ by the RipTide Team**

*High-performance web crawling and content extraction, powered by Rust and WebAssembly*

[![Made with Rust](https://img.shields.io/badge/Made%20with-Rust-orange.svg)](https://www.rust-lang.org/)
[![Powered by WASM](https://img.shields.io/badge/Powered%20by-WebAssembly-654FF0.svg)](https://webassembly.org/)

</div>
