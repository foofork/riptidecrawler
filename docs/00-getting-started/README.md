# Getting Started with RipTide

Welcome to RipTide - a high-performance web crawler and content extraction platform built in Rust with WebAssembly optimization.

## 🚀 Quick Start (5 Minutes)

### Prerequisites
- Docker and Docker Compose
- 4GB RAM minimum
- Redis (included in Docker setup)

### Quick Start with Docker (Fastest)

The fastest way to get RipTide running is with Docker:

**[→ Docker Quick Start Guide](./docker-quickstart.md)** - Get running in 30 seconds

```bash
docker-compose up -d
curl http://localhost:3000/health
```

For detailed Docker setup instructions, see the guide above or continue with the manual steps below.

### Launch with Docker

```bash
# Clone the repository
git clone https://github.com/your-org/eventmesh.git
cd eventmesh

# Start services
docker-compose up -d

# Verify health
curl http://localhost:8080/healthz
```

Expected output:
```json
{"status": "healthy"}
```

### Your First Crawl

```bash
# Simple crawl
curl -X POST http://localhost:8080/crawl \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://example.com",
    "extractors": [{
      "type": "css",
      "selector": "h1, p",
      "field": "content"
    }]
  }'
```

**Success!** You've just extracted content from a webpage.

---

## 📚 What's Next?

### Learn the Basics
1. **[Core Concepts](./concepts.md)** - Understanding extractors, strategies, and pipelines
2. **[FAQ](./faq.md)** - Common questions and answers
3. **[Quick Start Guide](./quickstart.md)** - Detailed walkthrough

### Common Tasks
- **[Crawl a Website](../01-guides/usage/crawling.md)** - Deep crawling guide
- **[Extract Tables](../01-guides/usage/table-extraction.md)** - Structured data extraction
- **[Stream Results](../01-guides/usage/streaming.md)** - Real-time streaming
- **[Extract PDFs](../01-guides/usage/pdf-extraction.md)** - PDF processing

### Configuration
- **[Environment Setup](../01-guides/configuration/environment.md)** - Environment variables
- **[LLM Providers](../01-guides/configuration/llm-providers.md)** - AI provider configuration
- **[Browser Pool](../01-guides/configuration/browser-pool.md)** - Headless browser setup

---

## 🎯 Key Features

- **🚀 Fast** - WASM-powered extraction with intelligent routing
- **🔄 Real-time** - NDJSON, SSE, and WebSocket streaming
- **🕷️ Deep Crawling** - BFS/DFS with frontier management
- **📄 PDF Support** - Text and table extraction
- **🧠 AI-Powered** - Multi-provider LLM integration
- **🎭 Stealth Mode** - Anti-detection and fingerprint randomization
- **📊 Monitoring** - Prometheus metrics and OpenTelemetry

---

## 📖 Documentation Structure

```
docs/
├── 00-getting-started/    ← You are here
├── 01-guides/             Task-oriented guides
├── 02-api-reference/      API documentation
├── 03-sdk/                SDKs and examples
├── 04-architecture/       System design
├── 05-development/        Contributing
├── 06-deployment/         Production deployment
├── 07-advanced/           Advanced topics
└── 08-reference/          Quick reference
```

---

## 🆘 Need Help?

- **[Troubleshooting Guide](../01-guides/operations/troubleshooting.md)**
- **[FAQ](./faq.md)**
- **[GitHub Issues](https://github.com/your-org/eventmesh/issues)**
- **[API Documentation](../02-api-reference/README.md)**

---

## 🎓 Learning Path

**Beginner** (30 minutes):
1. Quick Start (above)
2. Core Concepts
3. First API call

**Intermediate** (2 hours):
1. Deep crawling
2. Streaming protocols
3. Configuration options

**Advanced** (1 day):
1. Custom extractors
2. LLM integration
3. Production deployment

---

**Ready to dive deeper?** → [Core Concepts](./concepts.md)
