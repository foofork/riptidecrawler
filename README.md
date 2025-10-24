# RipTide

[![Version](https://img.shields.io/badge/version-2.0.0-blue.svg)](CHANGELOG.md)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-74%2B%20new%20tests-green.svg)](docs/development/testing.md)
[![Coverage](https://img.shields.io/badge/coverage-85%25+-brightgreen.svg)](docs/development/testing.md)

**A web crawler and content extraction platform built in Rust with WebAssembly optimization.**

High-performance crawling, intelligent extraction, and real-time streaming with comprehensive developer tooling.

---

## 📊 Status

**Current Focus:** optimization and testing.

---


## ⚡ Key Features

### Core Capabilities
- **WASM-Powered Extraction** - WebAssembly Component Model with adaptive routing
- **Dual-Path Pipeline** - Fast static extraction with headless browser fallback
- **Real-Time Streaming** - NDJSON, SSE, WebSocket protocols
- **Smart Caching** - Redis-backed with TTL management and domain profiling
- **Multi-Strategy Extraction** - CSS, LLM, Regex, Auto-selection
- **PDF Processing** - Full pipeline with table extraction
- **Deep Crawling** - Spider engine with frontier management
- **Domain Profiling** - Warm-start caching with per-domain configuration
- **Engine Selection** - Intelligent HTML analysis for optimal extraction strategy
- **Session Management** - Isolated browsing contexts with cookie persistence
- **Async Job Queue** - Background processing with retry logic
- **LLM Abstraction** - Multi-provider support (OpenAI, Anthropic, Google)
- **Stealth Mode** - Anti-detection with fingerprint randomization
- **Monitoring** - Prometheus metrics, health scores, bottleneck analysis

---

## 🚀 Quick Start

### Prerequisites

- Rust 1.75+ with `wasm32-wasip2` target
- Docker 20.10+ with Docker Compose
- Redis 7.0+ (included in Docker setup)
- Serper API Key ([get one here](https://serper.dev))

### Docker Deployment (Recommended)

```bash
git clone <repository-url>
cd eventmesh
cp .env.example .env
# Edit .env and add your SERPER_API_KEY

# Quick start with all services
make docker-build-all
make docker-up

# Verify health
curl http://localhost:8080/healthz

# Access playground
open http://localhost:3000
```

### Building from Source

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-wasip2

# Build WASM component
make build-wasm

# Build and run (using Makefile)
make build-release
docker run -d --name redis -p 6379:6379 redis:7-alpine
./target/release/riptide-api --config configs/riptide.yml
```

### Python SDK Usage

```bash
# Install SDK
pip install riptide-sdk

# Or from source
cd sdk/python
pip install -e .
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
        print(f"Successful: {result.successful}/{result.total_urls}")

asyncio.run(main())
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

## 📚 Documentation

### Getting Started
- **[Quick Deployment](docs/guides/quickstart/QUICK_DEPLOYMENT_GUIDE.md)** - Get up and running in 5 minutes
- **[API Tooling Quickstart](docs/guides/quickstart/API_TOOLING_QUICKSTART.md)** - CLI, SDK, and playground setup
- **[User Guide](docs/guides/operations/USER_GUIDE.md)** - Daily operations and best practices

### Developer Tools
- **[Python SDK](sdk/python/README.md)** - Async Python SDK with full type hints
- **[Interactive Playground](playground/README.md)** - React-based API testing interface
- **[OpenAPI Specification](docs/api/openapi.yaml)** - 110+ endpoints documented
- **[Makefile Commands](Makefile)** - 40+ development shortcuts
- **[Development Setup](docs/development/getting-started.md)** - Local development environment

### Architecture & Design
- **[System Overview](docs/architecture/system-overview.md)** - High-level architecture
- **[27-Crate Workspace](docs/implementation/P1/)** - Modular design documentation
- **[Testing Strategy](docs/development/testing.md)** - 74+ tests, 85%+ coverage
- **[Contributing Guide](docs/development/contributing.md)** - Contribution guidelines

### API Reference
- **[Endpoint Catalog](docs/api/ENDPOINT_CATALOG.md)** - All 110+ endpoints organized by category
- **[Streaming Protocols](docs/api/streaming.md)** - NDJSON, SSE, WebSocket
- **[Domain Profiles API](docs/api/)** - Phase 10.4 profiling endpoints
- **[Engine Selection API](docs/api/)** - Phase 10 intelligent routing

### Operations & Production
- **[Deployment Guide](docs/guides/quickstart/QUICK_DEPLOYMENT_GUIDE.md)** - Docker, Kubernetes, cloud deployment
- **[Configuration Reference](docs/architecture/configuration-guide.md)** - All config options explained
- **[Monitoring & Metrics](docs/performance-monitoring.md)** - Prometheus integration
- **[Troubleshooting](docs/user/troubleshooting.md)** - Common issues and solutions

**[Browse all documentation](docs/)**


**[Full testing documentation](docs/development/testing.md)**

---

## 🚢 Production Deployment

### Docker Compose (Recommended)

```bash
# Build all Docker images
make docker-build-all

# Start all services
make docker-up

# View logs
make docker-logs

# Stop services
make docker-down
```

**docker-compose.yml includes:**
- RipTide API (port 8080)
- Redis cache (port 6379)
- Interactive Playground (port 3000)
- Headless browser service

### Manual Docker Build

```yaml
services:
  riptide-api:
    image: riptide/api:latest
    ports: ["8080:8080"]
    environment:
      - SERPER_API_KEY=${SERPER_API_KEY}
      - REDIS_URL=redis://redis:6379
    depends_on: [redis]

  riptide-playground:
    image: riptide/playground:latest
    ports: ["3000:80"]
    environment:
      - VITE_API_BASE_URL=http://riptide-api:8080

  redis:
    image: redis:7-alpine
    ports: ["6379:6379"]
```

### Kong API Gateway (Optional)

```bash
docker-compose -f docker-compose.gateway.yml up -d
# Features: rate limiting, auth, logging, CORS, load balancing
```

### Production Profiles

```bash
# Build with release optimizations
make build-release

# Build with CI profile
make build-ci

# Build for fast development
make build-fast-dev
```

**[Full deployment guide](docs/guides/quickstart/QUICK_DEPLOYMENT_GUIDE.md)**

---

## 📄 License

Apache License 2.0 - see [LICENSE](LICENSE)

**Dependencies:** Tokio (MIT), Axum (MIT), Wasmtime (Apache-2.0), Chromiumoxide (MIT), Redis (BSD-3-Clause)


**Built with ⚡ by the RipTide Team**
