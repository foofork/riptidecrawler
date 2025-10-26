# Frequently Asked Questions

## General Questions

### What is RipTide?

RipTide is a high-performance web crawler and content extraction platform built in Rust. It combines WebAssembly optimization, headless browser automation, and AI-powered extraction to handle any website - from simple static pages to complex JavaScript applications.

### What makes RipTide different?

- **Intelligent Routing**: Automatically selects the best extraction strategy
- **Dual-Path Pipeline**: Fast static extraction with browser fallback
- **Real-time Streaming**: NDJSON, SSE, and WebSocket support
- **Multi-Provider AI**: Unified interface for OpenAI, Anthropic, Google, etc.
- **Production-Ready**: Prometheus metrics, circuit breakers, health checks

---

## Getting Started

### How do I install RipTide?

**Docker (recommended)**:
```bash
docker-compose up -d
```

**From source**:
```bash
cargo build --release
./target/release/riptide-api
```

See [Installation Guide](../01-guides/installation/docker.md) for details.

### What are the system requirements?

**Minimum**:
- 4GB RAM
- 2 CPU cores
- 10GB disk space
- Docker 20.10+ or Rust 1.70+

**Recommended**:
- 8GB RAM
- 4 CPU cores
- 50GB disk space (for browser pool and cache)

### Do I need Redis?

Yes, Redis is required for:
- Job queue
- Caching
- Domain profiling
- Session storage

The Docker setup includes Redis automatically.

---

## Features & Capabilities

### Can RipTide handle JavaScript-heavy sites?

Yes! RipTide uses a dual-path pipeline:
1. First tries static extraction (fast)
2. Falls back to headless browser if needed (reliable)

This gives you both speed and reliability.

### Does it support PDF extraction?

Yes, RipTide can extract:
- Text content from PDFs
- Tables from PDFs
- Metadata (author, title, etc.)

Example:
```bash
curl -X POST http://localhost:8080/pdf \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com/document.pdf"}'
```

### What about anti-scraping protection?

RipTide includes stealth features:
- User-agent rotation
- Fingerprint randomization
- Proxy support
- Cookie management
- Custom headers

See [Stealth Configuration](../01-guides/configuration/stealth.md).

### Can I use AI for extraction?

Yes! RipTide supports **8 LLM providers**:
- OpenAI (GPT-4, GPT-3.5)
- Anthropic (Claude)
- Google Vertex AI (Gemini)
- Azure OpenAI
- AWS Bedrock
- Ollama (local models)
- LocalAI (self-hosted)
- Custom providers (extensible)

See [LLM Provider Setup](../01-guides/configuration/llm-providers.md).

---

## Performance & Scaling

### How fast is RipTide?

**Static extraction**: ~50ms per page, 100 req/s
**Browser rendering**: ~2s per page, 10 req/s
**WASM processing**: ~10ms, 500 ops/s

Actual performance depends on target site complexity and hardware.

### Can it scale horizontally?

Yes, RipTide is stateless and can scale horizontally:
```bash
docker-compose up --scale api=5
```

Use a load balancer (nginx, HAProxy) to distribute traffic.

### What's the maximum concurrent crawls?

Limited by:
- Available memory (browser instances are expensive)
- CPU cores
- Redis connection pool
- Browser pool configuration

Typical: 10-50 concurrent browser sessions per instance.

### How do I optimize performance?

1. **Enable caching**: Use Redis cache with appropriate TTLs
2. **Domain profiling**: Let RipTide learn optimal strategies
3. **Resource limits**: Configure browser pool size
4. **Streaming**: Use streaming for large crawls

See [Performance Tuning](../07-advanced/performance-optimization.md).

---

## API & Integration

### Is there a Python SDK?

Yes! Full-featured Python SDK with async support:
```python
from riptide import RipTideClient

client = RipTideClient("http://localhost:8080")
results = await client.crawl("https://example.com")
```

See [Python SDK Guide](../03-sdk/python/README.md).

### What about other languages?

RipTide is a REST API - use any HTTP client:
- JavaScript/Node.js: `fetch`, `axios`
- Java: `OkHttp`, `Apache HttpClient`
- Go: `net/http`
- PHP: `Guzzle`
- Ruby: `Faraday`, `HTTParty`

See [API Reference](../02-api-reference/README.md) for OpenAPI spec with 120+ routes.

### Can I use it as a library?

RipTide crates are published and can be used as Rust libraries:
```toml
[dependencies]
riptide-extraction = "0.1"
riptide-spider = "0.1"
```

However, the API server is the recommended integration method.

---

## Configuration

### Where are configuration files?

**Docker**: `docker-compose.yml` and `.env`
**Source**: `riptide.yml` (YAML config)
**Environment**: `.env` file

See [Configuration Guide](../01-guides/configuration/environment.md).

### How do I configure LLM providers?

Edit `riptide.yml`:
```yaml
intelligence:
  providers:
    - name: openai
      api_key: ${OPENAI_API_KEY}
      model: gpt-4
      enabled: true
```

Supports hot-reload - no restart required!

### What environment variables are available?

Over 50 environment variables. Key ones:
- `REDIS_URL` - Redis connection
- `BROWSER_POOL_SIZE` - Max browser instances
- `OPENAI_API_KEY` - OpenAI API key
- `LOG_LEVEL` - Logging verbosity

See [Environment Reference](../08-reference/environment-variables.md).

---

## Troubleshooting

### API returns "Service Unavailable"

Check:
1. Is Redis running? `docker-compose ps`
2. Health check: `curl http://localhost:8080/healthz`
3. Logs: `docker-compose logs api`

See [Troubleshooting Guide](../01-guides/operations/troubleshooting.md).

### Browser crawls fail

Common causes:
1. Insufficient memory (browser instances are heavy)
2. Browser pool exhausted (increase `BROWSER_POOL_SIZE`)
3. Timeout too short (increase `BROWSER_TIMEOUT`)

Check browser pool status:
```bash
curl http://localhost:8080/api/v1/browser/pool/status
```

### Extraction returns empty results

Try:
1. Test with browser strategy explicitly
2. Check selector validity
3. Enable debug logging: `LOG_LEVEL=debug`
4. Use playground to test: `/playground`

### Rate limiting errors

RipTide implements rate limiting per domain. Adjust:
```yaml
rate_limit:
  requests_per_second: 10
  burst: 20
```

---

## Development & Contributing

### How do I contribute?

1. Fork the repository
2. Create a feature branch
3. Write tests
4. Submit a pull request

See [Contributing Guide](../05-development/contributing.md).

### How do I run tests?

```bash
# All tests
cargo test

# Integration tests
cargo test --test '*'

# Specific crate
cargo test -p riptide-extraction
```

### Where's the test coverage?

```bash
cargo tarpaulin --out Html
open tarpaulin-report.html
```

Current coverage: ~75%

---

## Deployment

### Is RipTide production-ready?

Yes! RipTide includes:
- Health checks
- Prometheus metrics
- Circuit breakers
- Resource limits
- Error handling
- Graceful shutdown

See [Production Deployment](../06-deployment/production.md).

### What about Kubernetes?

Kubernetes manifests available:
```bash
kubectl apply -f k8s/
```

See [Kubernetes Guide](../06-deployment/kubernetes.md).

### How do I monitor RipTide?

Built-in Prometheus metrics:
```bash
curl http://localhost:8080/metrics
```

Use with Grafana for visualization.
See [Monitoring Guide](../01-guides/operations/monitoring.md).

---

## Licensing & Support

### What's the license?

Check the LICENSE file in the repository.

### Where do I report bugs?

[GitHub Issues](https://github.com/your-org/eventmesh/issues)

### Is there commercial support?

Check the project README for support options.

---

**Still have questions?** Open an issue or check the [full documentation](../README.md).
