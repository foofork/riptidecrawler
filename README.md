# RipTide

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Phase 1](https://img.shields.io/badge/phase%201-97%25%20complete-success.svg)](docs/COMPREHENSIVE-ROADMAP.md)
[![Tests](https://img.shields.io/badge/tests-103%20files-green.svg)](docs/development/testing.md)
[![Coverage](https://img.shields.io/badge/coverage-85%25-brightgreen.svg)](docs/development/testing.md)

<!-- IN DEVELOPMENT: Phase 1 - 97% Complete -->
<!-- CURRENT FOCUS: P1-C1 compilation fixes and spider-chrome integration -->

**High-performance web crawler and content extraction platform built in Rust with WebAssembly optimization.**

---

## 📊 Status

**Phase 1: 90+% Complete** ([Full Roadmap](docs/COMPREHENSIVE-ROADMAP.md))

**Current Focus:** Testing, optimizing, and validating.  Note: There will be dead code, some things may be useful and some may be trash, marking or wiring it up. 

---


## ⚡ Key Features

### Core Capabilities
- **WASM-Powered Extraction** - WebAssembly Component Model with adaptive routing
- **Dual-Path Pipeline** - Fast static extraction with headless browser fallback
- **Real-Time Streaming** - NDJSON, SSE, WebSocket protocols
- **Smart Caching** - Redis-backed with TTL management
- **Multi-Strategy Extraction** - CSS, LLM, Regex, Auto-selection
- **PDF Processing** - Full pipeline with table extraction
- **Deep Crawling** - Spider engine with frontier management

### Enterprise Features
- **Session Management** - Isolated browsing contexts with cookie persistence
- **Async Job Queue** - Background processing with retry logic
- **LLM Abstraction** - Multi-provider support (OpenAI, Anthropic, Google)
- **Stealth Mode** - Anti-detection with fingerprint randomization
- **Monitoring** - Prometheus metrics, health scores, bottleneck analysis
- **API-First Architecture** - 59 endpoints across 13 categories

---

## 🏗️ Architecture

### System Overview

```
Client (CLI/SDK/API)
        ↓
REST API (59 endpoints)
        ↓
Facade Layer (Browser, Extraction, Scraper, Pipeline)
        ↓
Core Services (27 specialized crates)
        ↓
External Integrations (Redis, Chromium, WASM, LLMs)
```


---

## 🚀 Quick Start

### Prerequisites

- Rust 1.75+ with `wasm32-wasip2` target
- Docker 20.10+ with Docker Compose
- Redis 7.0+ (included in Docker setup)
- Serper API Key ([get one here](https://serper.dev))

### Docker Deployment

```bash
git clone <repository-url>
cd eventmesh
cp .env.example .env
# Edit .env and add your SERPER_API_KEY

docker-compose up -d
curl http://localhost:8080/healthz
```

### Building from Source

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-wasip2

# Build WASM component
cd wasm/riptide-extractor-wasm
cargo build --target wasm32-wasip2 --release
cd ../..

# Build and run
cargo build --release
docker run -d --name redis -p 6379:6379 redis:7-alpine
./target/release/riptide-api --config configs/riptide.yml
```

### First API Request

```bash
# Health check
curl http://localhost:8080/healthz

# Crawl a URL
curl -X POST http://localhost:8080/crawl \
  -H "Content-Type: application/json" \
  -d '{"urls": ["https://example.com"], "options": {"concurrency": 5}}'

# Deep search with content extraction
curl -X POST http://localhost:8080/deepsearch \
  -H "Content-Type: application/json" \
  -d '{"query": "rust web scraping", "limit": 10, "include_content": true}'
```


---

## 📚 Documentation (drafts abound)

### Getting Started
- **[Quick Deployment](docs/guides/quickstart/QUICK_DEPLOYMENT_GUIDE.md)** - Get up and running
- **[API Tooling](docs/guides/quickstart/API_TOOLING_QUICKSTART.md)** - CLI, SDK, playground
- **[User Guide](docs/guides/operations/USER_GUIDE.md)** - Daily operations

### Developer
- **[Development Setup](docs/development/getting-started.md)** - Local development
- **[Architecture](docs/architecture/system-overview.md)** - System design
- **[Testing](docs/development/testing.md)** - Test suite and coverage
- **[Contributing](docs/development/contributing.md)** - Contribution guidelines

### API Reference
- **[Endpoint Catalog](docs/api/ENDPOINT_CATALOG.md)** - All 59 endpoints
- **[OpenAPI Spec](docs/api/openapi.yaml)** - Machine-readable spec
- **[Streaming](docs/api/streaming.md)** - Real-time protocols

### Operations
- **[Deployment](docs/guides/quickstart/QUICK_DEPLOYMENT_GUIDE.md)** - Production setup
- **[Configuration](docs/architecture/configuration-guide.md)** - Config reference
- **[Monitoring](docs/performance-monitoring.md)** - Metrics and health
- **[Troubleshooting](docs/user/troubleshooting.md)** - Common issues

**[Browse all docs](docs/)**

---

## 🧪 Testing

```bash
# Run all tests
cargo test

# Specific crate
cargo test -p riptide-core

# With coverage
cargo tarpaulin --out Html --output-dir coverage

# Code quality
cargo fmt && cargo clippy -- -D warnings

# Security audit
cargo audit && cargo deny check
```

**Metrics:** 103 test files, 85%+ coverage, automated safety audits

**[Testing documentation](docs/development/testing.md)**

---

## 🚢 Production Deployment

### Docker Compose

```yaml
services:
  riptide-api:
    image: riptide/api:latest
    ports: ["8080:8080"]
    environment:
      - SERPER_API_KEY=${SERPER_API_KEY}
      - REDIS_URL=redis://redis:6379
    depends_on: [redis]

  redis:
    image: redis:7-alpine
    ports: ["6379:6379"]
```

### Kong API Gateway

```bash
docker-compose -f docker-compose.gateway.yml up -d
# Features: rate limiting, auth, logging, CORS, load balancing
```

**[Deployment guide](docs/architecture/deployment-guide.md)**

---

## 🛣️ Roadmap

**Phase 1 (97%)** - Architecture & Performance
- ✅ 27-crate modular workspace
- ✅ Facade composition layer
- ✅ Browser pool scaling, CDP multiplexing
- ⚙️ Spider-chrome integration (95%)

**Phase 2 (Planned)** - Scale & Distribution
- Distributed crawling coordination
- Enhanced analytics dashboard
- GraphQL API endpoint
- Advanced rate limiting

**[Full roadmap](docs/COMPREHENSIVE-ROADMAP.md)**

---

## 🤝 Contributing

See **[Contributing Guide](docs/development/contributing.md)** for details.

**Quick Guidelines:**
1. Fork and create feature branch
2. Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
3. Maintain 80%+ test coverage
4. Run `cargo fmt && cargo clippy`
5. Use [Conventional Commits](https://www.conventionalcommits.org/)

---

## 📄 License

Apache License 2.0 - see [LICENSE](LICENSE)

**Dependencies:** Tokio (MIT), Axum (MIT), Wasmtime (Apache-2.0), Chromiumoxide (MIT), Redis (BSD-3-Clause)


**Built with ⚡ by the RipTide Team**
