# RipTide - Refactoring Underway

[![Version](https://img.shields.io/badge/version-0.9.0-blue.svg)](CHANGELOG.md)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-1.5k%2B-green.svg)](docs/development/testing.md)
[![Build](https://img.shields.io/badge/build-passing-brightgreen.svg)]()

> **High-performance web crawling and content extraction platform built in Rust with WebAssembly optimization.**

Extract structured data from any website with intelligent routing, multi-provider LLM support, and real-time streaming. Built for speed, reliability, and scale.

---

## ⚠️ NOT READY FOR PRIME TIME

**This project is undergoing major refactoring and is NOT production-ready:**

- 🚧 **API v1.0 is in active development** - Breaking changes expected
- 🔄 **Major refactoring underway** - See [RipTide Roadmap](/docs/architecture/ENHANCED_LAYERING_ROADMAP.md)


---

## ✨ Why RipTide?

- 🚀 **Blazing Fast** - Native Rust extraction (2-5ms) with optional WASM sandboxing
- 🧠 **AI-Powered** - 8 LLM providers (OpenAI, Anthropic, Google, Azure, AWS, Ollama) with automatic failover
- 🎯 **Smart Crawling** - Deep spider engine with relevance-based frontier management
- 🔄 **Real-Time Streams** - NDJSON, SSE, and WebSocket protocols with backpressure control
- 🛡️ **Production Ready** - Redis caching, session management, health monitoring, and graceful degradation
- 🧩 **Modular Architecture** - 27-crate workspace with clean separation of concerns
- ⚡ **Flexible Extraction** - Choose native (fast) or WASM (sandboxed) with three-tier fallback

---

## 🎯 Key Features

### Extraction & Processing
- **Native Parser (Default)** - Pure Rust extraction (2-5ms) with scraper crate for 99% of use cases
- **Optional WASM Sandboxing** - WebAssembly Component Model (Wasmtime 37) for security-critical applications
- **Three-Tier Fallback** - Compile-time → Runtime → Execution fallback for maximum reliability
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

Get up and running in 5 minutes with **COMPLETE functionality out-of-box**:

```bash
# Clone and configure
git clone <repository-url>
cd riptide  # Or whatever you named the directory
cp .env.example .env
# Edit .env and add your SERPER_API_KEY

# Start all services - FULL functionality with one command!
docker-compose up -d

# Verify health
curl http://localhost:8080/healthz
# {"status":"healthy","version":"0.9.0"}

# Access API documentation
open http://localhost:8081
```

**What You Get (FULL Stack by Default):**
- ✅ **RipTide API** (168MB) - REST API + WebSocket
- ✅ **Chrome Browser Service** (783MB) - 5-browser pool for JavaScript rendering
- ✅ **Redis Cache** - Fast session & data caching
- ✅ **Swagger UI** - Interactive API documentation
- ✅ **JavaScript Execution** - Full support for SPA pages
- ✅ **WASM Extraction** - Fallback for static content

**Memory:** ~1.2GB total | **No Configuration Needed!**

**Optional Lightweight Mode** (WASM-only, 60% smaller):
```bash
docker-compose -f docker-compose.lite.yml up -d  # ~440MB, no Chrome
```

**See: [Complete Docker Deployment Guide →](docs/DEPLOYMENT_GUIDE.md)**

### Option 2: Build from Source

**Default (Native Parser - Fast, Recommended):**
```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build API (native parser only, 40% faster)
cargo build --release

# Start Redis
docker run -d --name redis -p 6379:6379 redis:7-alpine

# Run RipTide (no WASM needed!)
./target/release/riptide-api
```

**With WASM (Optional - Security-Critical):**
```bash
# Install Rust with WASM target
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-wasip2

# Build with WASM feature
cargo build --release --features wasm-extractor
cargo build --release --target wasm32-wasip2 -p riptide-extractor-wasm

# Start Redis
docker run -d --name redis -p 6379:6379 redis:7-alpine

# Run RipTide with WASM
export WASM_EXTRACTOR_PATH=target/wasm32-wasip2/release/riptide_extractor_wasm.wasm
./target/release/riptide-api
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
- **[Streaming Protocols](docs/02-api-reference/streaming.md)** - NDJSON, SSE, WebSocket
- **[LLM Provider Setup](docs/01-guides/setup/LLM_PROVIDER_SETUP.md)** - Configure AI providers
- **[Feature Flags](docs/FEATURES.md)** - Native vs WASM extraction modes
- **[Docker Guide](docs/DOCKER.md)** - Container deployment options

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

### Docker Compose (Production & Development)

**Default (Full Stack - Recommended for 95% of users):**
```bash
# Quick start - one command gives you EVERYTHING
docker-compose up -d

# Or using Makefile
make docker-up

# View logs
make docker-logs

# Stop services
make docker-down
```

**Services Included:**
- ✅ RipTide API (port 8080) - REST + WebSocket
- ✅ Chrome Browser Service (port 9123) - 5-browser pool for JavaScript
- ✅ Redis Cache (port 6379) - Session storage
- ✅ Swagger UI (port 8081) - API documentation
- ✅ Full JavaScript execution
- ✅ SPA page support

**Memory:** ~1.2GB | **Configuration:** Zero

---

**Lightweight Mode** (WASM-only, 60% smaller):
```bash
# For memory-constrained environments
docker-compose -f docker-compose.lite.yml up -d

# Or using Makefile
make docker-up-lite

# View logs
make docker-logs-lite

# Stop services
make docker-down-lite
```

**Services Included:**
- ✅ RipTide API (port 8080) - WASM extraction only
- ✅ Redis Cache (port 6379) - Session storage
- ✅ Swagger UI (port 8081) - API documentation
- ❌ No Chrome service (no JavaScript)

**Memory:** ~440MB | **Best For:** Static content extraction

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

### ⭐ RipTide v1.0 Roadmap (THE PLAN)

**Status:** ✅ Validated (95% confidence) by 4-agent swarm
**Timeline:** 18 weeks to production-ready v1.0
**Current Phase:** Phase 0 - Foundations (Week 0-2.5)

See **[RIPTIDE-V1-DEFINITIVE-ROADMAP.md](docs/roadmap/RIPTIDE-V1-DEFINITIVE-ROADMAP.md)** for the complete validated roadmap.

**Key Goals:**
1. ✅ Extract in 1 line: `client.extract(url)`
2. ✅ Spider without extract: `client.spider(url)`
3. ✅ Extract without spider: `client.extract_html(html)`
4. ✅ Compose flexibly: `client.spider(url).and_extract()`
5. ✅ Use from Python: `pip install riptide`

**If yes to all 5 → Ship v1.0 🚀**

### Phases Overview
- **Phase 0 (Week 0-2.5):** Utils consolidation, config system, health endpoints
- **Phase 1 (Week 2.5-9):** Spider decoupling, facades, composable APIs
- **Phase 2 (Week 9-14):** LLM facades, Python SDK, events schema
- **Phase 3 (Week 14-18):** Integration testing, documentation, launch

### Supporting Documentation
- [Validation Synthesis](docs/roadmap/VALIDATION-SYNTHESIS.md) - Why corrections were made
- [UX Vision](docs/roadmap/riptide-v1-ux-design.md) - User experience design
- [Breaking Changes](docs/roadmap/BREAKING-CHANGES-MIGRATION.md) - Migration guide

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
