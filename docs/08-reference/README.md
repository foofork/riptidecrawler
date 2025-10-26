# Quick Reference

Fast lookup documentation and cheat sheets for RipTide.

## 🔍 Coming Soon

This section is being organized with quick-reference materials including:

- **API Quick Reference** - Common endpoints and examples
- **Configuration Reference** - All environment variables
- **Error Code Reference** - Complete error code catalog
- **CLI Command Reference** - All CLI commands
- **Metrics Reference** - Available metrics and labels
- **Status Code Reference** - HTTP status codes and meanings

## 📚 Available References (In Other Sections)

Until dedicated quick references are created, see these existing resources:

### API Reference
- **[API Documentation](../02-api-reference/README.md)** - Complete API reference
- **[OpenAPI Specification](../02-api-reference/openapi.yaml)** - Machine-readable schema
- **[Endpoint Catalog](../02-api-reference/ENDPOINT_CATALOG.md)** - All 59 endpoints

### Configuration
- **[Environment Setup](../01-guides/configuration/environment.md)** - Configuration guide
- **[LLM Providers](../01-guides/configuration/llm-providers.md)** - Provider configuration

### Operations
- **[Metrics Quick Reference](../01-guides/reference/metrics-quick-reference.md)** - Metrics overview
- **[Telemetry Quick Reference](../01-guides/reference/telemetry-quick-reference.md)** - Telemetry guide
- **[FAQ](../01-guides/reference/FAQ.md)** - Frequently asked questions

### Development
- **[Coding Standards](../05-development/coding-standards.md)** - Code style reference
- **[Testing Guide](../05-development/testing.md)** - Testing reference

## 🎯 Common Quick Tasks

### Make a Basic Request

```bash
curl -X POST http://localhost:8080/crawl \
  -H "Content-Type: application/json" \
  -d '{"urls":["https://example.com"]}'
```

### Check System Health

```bash
curl http://localhost:8080/healthz
```

### View Metrics

```bash
curl http://localhost:8080/metrics
```

### Stream Results

```bash
curl -X POST http://localhost:8080/crawl/stream \
  -H "Content-Type: application/json" \
  -d '{"urls":["https://example.com"]}'
```

## 🔗 Quick Navigation

### For Users
- **[Getting Started](../00-getting-started/README.md)** - New to RipTide
- **[User Guides](../01-guides/README.md)** - How-to guides
- **[API Reference](../02-api-reference/README.md)** - API documentation

### For Developers
- **[Development Guide](../05-development/README.md)** - Contributing
- **[Architecture](../04-architecture/README.md)** - System design
- **[Deployment](../06-deployment/README.md)** - Production deployment

### For Operators
- **[Operations Guides](../01-guides/operations/)** - Day-to-day operations
- **[Deployment](../06-deployment/README.md)** - Deployment guides
- **[Advanced Topics](../07-advanced/README.md)** - Performance tuning

## 📖 Glossary

### Key Terms

| Term | Definition |
|------|------------|
| **WASM** | WebAssembly - Binary format for fast content extraction |
| **CDP** | Chrome DevTools Protocol - Browser automation protocol |
| **SSE** | Server-Sent Events - One-way streaming protocol |
| **NDJSON** | Newline-Delimited JSON - Streaming JSON format |
| **Extractor** | Component that extracts content from HTML |
| **Spider** | Web crawling engine |
| **Session** | Stateful crawling context |
| **Cache Mode** | How caching is used (read/write/bypass) |
| **Extract Mode** | Type of extraction (article/full/minimal) |

### HTTP Status Codes

| Code | Meaning | Action |
|------|---------|--------|
| **200** | Success | Continue |
| **400** | Bad Request | Check request format |
| **429** | Rate Limited | Reduce request rate |
| **500** | Server Error | Check logs |
| **503** | Service Unavailable | Check dependencies |

### Common Patterns

```javascript
// Basic crawl
POST /crawl
{
  "urls": ["https://example.com"],
  "options": {
    "extract_mode": "article",
    "cache_mode": "read_write"
  }
}

// Streaming crawl
POST /crawl/stream
{
  "urls": ["https://example.com"],
  "options": {
    "concurrency": 5
  }
}

// Deep search
POST /deepsearch
{
  "query": "machine learning",
  "limit": 10,
  "crawl_options": {
    "extract_mode": "article"
  }
}
```

## 🔧 Environment Variables

### Essential Variables

```bash
# Server
PORT=8080
HOST=0.0.0.0

# Redis
REDIS_URL=redis://localhost:6379

# Logging
RUST_LOG=info

# Performance
MAX_CONCURRENCY=10
CACHE_TTL=3600

# Features
ENABLE_METRICS=true
ENABLE_STEALTH=true
```

See [Configuration Reference](../01-guides/configuration/environment.md) for complete list.

## 🆘 Troubleshooting Quick Fixes

| Issue | Quick Fix |
|-------|-----------|
| **Connection refused** | Check service is running: `docker ps` |
| **Slow responses** | Check Redis: `redis-cli ping` |
| **High memory** | Restart service, check for leaks |
| **Cache misses** | Verify Redis connection |
| **Empty results** | Try dynamic mode: `"mode": "dynamic"` |

## 📞 Support Resources

- **[FAQ](../01-guides/reference/FAQ.md)** - Common questions
- **[Troubleshooting](../01-guides/operations/troubleshooting.md)** - Problem solving
- **[GitHub Issues](https://github.com/your-org/eventmesh/issues)** - Bug reports
- **[Discussions](https://github.com/your-org/eventmesh/discussions)** - Community help

---

**Need detailed docs?** → [Full Documentation](../README.md)
