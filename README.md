# 🌊 RipTide - High-Performance Web Crawler & Content Extraction

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![API Docs](https://img.shields.io/badge/API-100%25%20documented-brightgreen.svg)](docs/api/openapi.yaml)
[![Tests](https://img.shields.io/badge/tests-passing-brightgreen.svg)](tests/)

> **Enterprise-grade web crawling and content extraction with WASM-powered extraction, adaptive routing, and real-time streaming.**

RipTide is a blazingly fast, production-ready API for web content extraction. Built in Rust with WebAssembly optimization, it intelligently routes requests through multiple extraction strategies to deliver the best quality content with minimal latency.

---

## ✨ Key Features

### 🚀 **Performance**
- **WASM-Powered Extraction** - High-performance content extraction using WebAssembly
- **Dual-Path Pipeline** - Fast CSS extraction + async AI enhancement
- **Adaptive Gate System** - Smart routing (raw/probes/headless/cached)
- **Event-Driven Architecture** - Built on core event bus for real-time monitoring
- **Circuit Breaker Pattern** - Automatic failover for external dependencies

### 🎯 **Extraction Capabilities**
- **Multi-Strategy Extraction** - CSS, TREK (WASM), LLM, Regex, Auto-detection
- **Spider Deep Crawling** - Frontier management with intelligent link discovery
- **PDF Processing** - Native PDF content extraction with streaming
- **Table Extraction** - Extract tables from HTML/PDF with export (CSV/Markdown)
- **Stealth Browsing** - Bot evasion with configurable anti-detection measures

### 🔄 **Real-Time Streaming**
- **NDJSON Streaming** - Newline-delimited JSON for live results
- **Server-Sent Events (SSE)** - Browser-friendly streaming
- **WebSocket** - Bidirectional real-time communication
- **Progress Tracking** - Live updates for long-running operations

### 🏢 **Enterprise Features**
- **Session Management** - Persistent sessions with cookie management (12 endpoints)
- **Async Job Queue** - Background processing with scheduling (9 endpoints)
- **LLM Provider Abstraction** - Runtime switching between AI providers (4 endpoints)
- **Monitoring & Alerts** - Health scores, performance reports, active alerts (6 endpoints)
- **API Gateway Ready** - Kong, Tyk, AWS API Gateway integration

---

## 📊 API Overview

**59 Endpoints** across **13 Categories** - **100% Documented**

| Category | Endpoints | Description |
|----------|-----------|-------------|
| **Health** | 2 | System health checks & Prometheus metrics |
| **Crawling** | 5 | Batch crawling with adaptive routing |
| **Streaming** | 4 | NDJSON, SSE, WebSocket protocols |
| **Search** | 2 | Deep search with content extraction |
| **Spider** | 3 | Deep crawling with frontier management |
| **Strategies** | 2 | Multi-strategy extraction (CSS/TREK/LLM/Regex) |
| **PDF** | 3 | PDF processing with streaming |
| **Stealth** | 4 | Stealth configuration & testing |
| **Tables** | 2 | Table extraction & export |
| **LLM** | 4 | LLM provider management |
| **Sessions** | 12 | Session & cookie management |
| **Workers** | 9 | Async job queue & scheduling |
| **Monitoring** | 6 | Metrics, alerts, health scores |

📚 **[Complete API Documentation](docs/api/openapi.yaml)** | 🚀 **[Quick Start Guide](docs/API_TOOLING_QUICKSTART.md)**

---

## 🚀 Quick Start

### ⚡ Try RipTide in 30 Seconds

**Option 1: Pre-built Docker Image (Fastest)**

```bash
# Pull and run RipTide instantly (no build required!)
docker run -d \
  -p 8080:8080 \
  -p 8081:8080 \
  --name riptide \
  -e REDIS_URL=redis://host.docker.internal:6379 \
  riptide/api:latest

# Or use our quick-start script (recommended)
curl -fsSL https://raw.githubusercontent.com/your-org/riptide-api/main/scripts/quick-start.sh | bash

# Access:
# - API: http://localhost:8080
# - Swagger UI: http://localhost:8081
# - Health: http://localhost:8080/healthz
```

**Option 2: Docker Compose (Full Stack)**

```bash
# Automated setup with Redis + Swagger UI
./scripts/quick-start.sh

# Or manually:
docker-compose up -d

# Access API at: http://localhost:8080
# Swagger UI at: http://localhost:8081
```

**Option 3: From Source (Development)**

```bash
# Clone the repository
git clone https://github.com/your-org/riptide-api.git
cd riptide-api

# Build the project
cargo build --release

# Run the API server
cargo run --release --package riptide-api

# API available at: http://localhost:8080
```

### 🧪 Test Your Installation

```bash
# Run automated tests
./scripts/test-riptide.sh

# Or test manually:
# Health check
curl http://localhost:8080/healthz

# Crawl a URL
curl -X POST http://localhost:8080/crawl \
  -H "Content-Type: application/json" \
  -d '{
    "urls": ["https://example.com"],
    "options": {
      "concurrency": 1,
      "cache_mode": "read_write"
    }
  }'
```

### 📖 Self-Hosting Guide

For detailed instructions on production deployment, security, scaling, and advanced configurations, see our [Self-Hosting Guide](SELF_HOSTING.md).

---

## 📖 Interactive API Documentation

### Swagger UI - Try APIs in Your Browser

```bash
# Start Swagger UI
docker-compose -f docker-compose.swagger.yml up -d swagger-ui

# Open in browser: http://localhost:8081
```

**Features:**
- 🧪 **Test any endpoint** directly in browser (no Postman needed!)
- 📝 **See request/response examples** for all 59 endpoints
- 🔍 **Search and filter** endpoints by category
- 📊 **View schemas** and data models
- 💾 **Download** OpenAPI specification

**Alternative:** [ReDoc](http://localhost:8082) - Clean, mobile-friendly documentation

---

## 🏗️ Architecture

**[📐 View Full System Diagram](docs/architecture/system-diagram.md)** - Complete visual architecture with data flows

### Dual-Path Pipeline

```
┌─────────────────────────────────────────────────────┐
│  Request → Gate Decision                             │
├─────────────────────────────────────────────────────┤
│  ┌─────────────┐    ┌──────────────────────────┐   │
│  │ Fast Path   │    │ Enhanced Path            │   │
│  │             │    │                          │   │
│  │ CSS Extract │ OR │ WASM → AI Enhancement    │   │
│  │ (~500ms)    │    │ (~2-3s)                  │   │
│  └─────────────┘    └──────────────────────────┘   │
├─────────────────────────────────────────────────────┤
│  Response with quality score & metrics               │
└─────────────────────────────────────────────────────┘
```

### Core Components

- **Adaptive Gate** - Intelligent routing based on content analysis
- **Event Bus** - Central event system for monitoring & coordination
- **Circuit Breaker** - Automatic failover for external dependencies
- **WASM Extractor** - High-performance WebAssembly content extraction (~45ms avg)
- **Redis Cache** - Distributed caching with TTL (40-60% hit rate)
- **Worker Pool** - Async job processing with scheduling

📐 **[Full Diagram](docs/architecture/system-diagram.md)** | 🏛️ **[System Overview](docs/architecture/system-overview.md)** | 🔧 **[Configuration Guide](docs/architecture/configuration-guide.md)**

---

## 🧪 Testing & Quality Assurance

### Automated Testing Suite

```bash
# Run all tests
cargo test

# Run integration tests only
cargo test --test '*'

# Run with coverage
cargo tarpaulin --out Html
```

### API Contract Testing

```bash
# Install Dredd
npm install -g dredd

# Test API against OpenAPI spec
dredd docs/api/openapi.yaml http://localhost:8080
```

### API Fuzzing

```bash
# Install Schemathesis
pip install schemathesis

# Automatic fuzzing
schemathesis run docs/api/openapi.yaml --base-url http://localhost:8080
```

**Test Coverage:** 85%+ | **103 Test Files** | **46,000+ Lines of Test Code**

📋 **[Testing Guide](docs/TESTING_GUIDE.md)** | 🔬 **[CI/CD Pipeline](.github/workflows/api-contract-tests.yml)**

---

## 🌐 Production Deployment

### With API Gateway (Kong)

```bash
# Start Kong Gateway + RipTide
docker-compose -f docker-compose.gateway.yml up -d

# Configure rate limiting
curl -X POST http://localhost:8001/services/riptide-api/plugins \
  --data "name=rate-limiting" \
  --data "config.minute=100"

# Add API key authentication
curl -X POST http://localhost:8001/services/riptide-api/plugins \
  --data "name=key-auth"
```

**Access Points:**
- 🚪 API Gateway: `http://localhost:8000/api`
- 🔧 Admin API: `http://localhost:8001`
- 📊 Dashboard: `http://localhost:8002`
- 📚 Docs: `http://localhost:8081`

### Environment Configuration

```bash
# Core settings
export REDIS_URL=redis://localhost:6379
export WASM_PATH=/path/to/riptide.wasm
export MAX_CONCURRENCY=10

# Enhanced pipeline
export ENHANCED_PIPELINE_ENABLE=true
export ENHANCED_PIPELINE_METRICS=true

# Optional headless service
export HEADLESS_URL=http://localhost:3000
```

📦 **[Deployment Guide](docs/architecture/deployment-guide.md)** | ⚙️ **[Configuration Reference](docs/user/configuration.md)**

---

## 🔧 Developer Tools

### Generate Client SDKs

```bash
# TypeScript/JavaScript
openapi-generator-cli generate \
  -i docs/api/openapi.yaml \
  -g typescript-axios \
  -o clients/typescript

# Python
openapi-generator-cli generate \
  -i docs/api/openapi.yaml \
  -g python \
  -o clients/python

# Also available: Rust, Go, Java, PHP, Ruby, C#, Swift, Kotlin
```

### Import to API Tools

**Postman:**
```bash
openapi2postmanv2 -s docs/api/openapi.yaml -o riptide.postman.json
# Import: File → Import → riptide.postman.json
```

**Insomnia:**
1. Import/Export → Import Data
2. Select: `docs/api/openapi.yaml`
3. ✅ 59 requests auto-created

🛠️ **[Developer Guide](docs/development/getting-started.md)** | 📚 **[API Examples](docs/api/examples.md)**

---

## 📈 Performance & Monitoring

### Metrics Endpoints

```bash
# Prometheus metrics
curl http://localhost:8080/metrics

# Health score (0-100)
curl http://localhost:8080/monitoring/health-score

# Performance report
curl http://localhost:8080/monitoring/performance-report

# Pipeline phase metrics
curl http://localhost:8080/pipeline/phases
```

### Key Metrics

- **Response Time:** p50 ≤1.5s, p95 ≤5s
- **Success Rate:** ≥99.5%
- **Cache Hit Rate:** ~40-60%
- **Concurrent Requests:** Up to 100/sec

📊 **[Performance Guide](docs/api/performance.md)** | 🔍 **[Monitoring Setup](docs/ENHANCED_PIPELINE_IMPLEMENTATION.md)**

---

## 🧠 Memory Profiling

**Production-Ready Memory Profiling System**

RipTide includes comprehensive memory profiling capabilities for production monitoring and optimization:

### Components

- **Memory Tracker** - Real-time memory monitoring with jemalloc integration
- **Leak Detector** - Automatic memory leak detection and analysis
- **Allocation Analyzer** - Pattern analysis and optimization recommendations

### Features

- ✅ Real-time memory snapshots
- ✅ Leak detection with growth rate analysis
- ✅ Allocation pattern optimization
- ✅ HTTP endpoints for monitoring
- ✅ Prometheus metrics integration
- ✅ < 2% performance overhead

### Quick Start

```rust
use riptide_performance::profiling::MemoryProfiler;
use uuid::Uuid;

let session_id = Uuid::new_v4();
let mut profiler = MemoryProfiler::new(session_id)?;

profiler.start_profiling().await?;
// ... run your workload ...
let report = profiler.stop_profiling().await?;

println!("Peak memory: {:.2}MB", report.peak_memory_mb);
println!("Efficiency score: {:.2}/1.0", report.memory_efficiency_score);
```

### HTTP Endpoints

```bash
# Get current memory snapshot
curl http://localhost:8080/profiling/snapshot

# Check for memory alerts
curl http://localhost:8080/profiling/alerts

# Get profiling report
curl http://localhost:8080/profiling/report
```

📘 **[Activation Guide](docs/memory-profiling-activation-guide.md)** | 💡 **[Usage Examples](docs/memory-profiling-examples.md)**

---

## 📚 Documentation

### For Users
- 📘 **[API Usage Guide](docs/user/api-usage.md)** - How to use the API
- 🔧 **[Configuration](docs/user/configuration.md)** - Environment variables & settings
- 🚀 **[Installation Guide](docs/user/installation.md)** - Setup instructions
- ❓ **[Troubleshooting](docs/user/troubleshooting.md)** - Common issues & solutions

### For Developers
- 🏁 **[Getting Started](docs/development/getting-started.md)** - Developer setup
- 🎨 **[Coding Standards](docs/development/coding-standards.md)** - Style guide
- 🤝 **[Contributing](docs/development/contributing.md)** - Contribution guidelines
- 🧪 **[Testing Guide](docs/development/testing.md)** - Test writing guide

### API Reference
- 📖 **[OpenAPI Spec](docs/api/openapi.yaml)** - Complete API specification
- 📋 **[Endpoint Catalog](docs/api/ENDPOINT_CATALOG.md)** - Detailed endpoint reference
- 🚀 **[Quick Start](docs/API_TOOLING_QUICKSTART.md)** - API tooling guide
- 🎨 **[Swagger UI Guide](docs/SWAGGER_UI_DEPLOYMENT_GUIDE.md)** - Interactive docs setup

### Architecture & Design
- 🏛️ **[System Overview](docs/architecture/system-overview.md)** - High-level architecture
- 📋 **[Roadmap](docs/ROADMAP.md)** - Implementation status (82% complete)
- 🔄 **[Streaming Integration](docs/architecture/streaming-integration-executive-summary.md)** - Real-time features
- 🎯 **[Phase 3 Summary](docs/PHASE_3_FINAL_SUMMARY.md)** - Latest completion report

---

## 🗺️ Project Roadmap

**Current Status:** 82% Complete (61/74 tasks)

### ✅ Phase 1: Core System (100% Complete)
- Event-driven architecture with EventBus
- Circuit breaker for external dependencies
- Adaptive gate system
- WASM-powered extraction

### ✅ Phase 2: Reliability & Monitoring (100% Complete)
- Reliability module with retry logic
- Monitoring system with health scores
- Strategies routes (CSS/JSON/Regex/LLM)
- Worker service with job queue

### ✅ Phase 3: Enhanced Features (100% Complete)
- Enhanced pipeline with phase metrics
- Telemetry & distributed tracing
- Session management
- PDF & table extraction

### ⚠️ Phase 3: Optional Enhancements (25% Complete)
- FetchEngine integration (foundation + docs)
- Cache warming (foundation + docs)

📋 **[Full Roadmap](docs/ROADMAP.md)** | 📈 **[Progress Report](docs/PHASE_3_FINAL_SUMMARY.md)**

---

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](docs/development/contributing.md) for details.

### Development Setup

```bash
# Clone repository
git clone https://github.com/your-org/riptide-api.git
cd riptide-api

# Install dependencies
cargo build

# Run tests
cargo test

# Run linter
cargo clippy -- -D warnings

# Format code
cargo fmt
```

### Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

## 🔒 Security


### Security Features

- 🛡️ **Circuit Breaker** - Protection against cascading failures
- 🔐 **API Key Authentication** - Via Kong Gateway
- ⏱️ **Rate Limiting** - Configurable per endpoint
- 🔍 **Input Validation** - Comprehensive request validation
- 🚫 **CORS Protection** - Configurable CORS policies

📋 **[Security Documentation](docs/api/security.md)**

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

Built with:
- [Rust](https://www.rust-lang.org/) - Systems programming language
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [Tokio](https://tokio.rs/) - Async runtime
- [WebAssembly](https://webassembly.org/) - High-performance extraction
- [Redis](https://redis.io/) - Caching layer

