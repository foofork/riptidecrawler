# RipTide - Technical Documentation

## Project Status: v0.1.0 - 85% Production Ready

RipTide is a high-performance web content extraction system built in Rust with advanced WASM-based extraction technology. The system uses WASM-based content processing, resource management, and monitoring capabilities.

**Current Version**: v0.1.0
**Production Readiness**: 85% (see [Roadmap](ROADMAP.md) for details)
**Major Milestones Complete**: WASM enhancements, PDF pipeline integration, CI optimization, code quality improvements, query-aware spider, multi-provider LLM, topic chunking, and session persistence are finished.

## 📚 Documentation Navigation

### Quick Access
- [Getting Started](#quick-start) - Set up and run the system
- [Active Roadmap](ROADMAP.md) - Current development status
- [API Reference](api/README.md) - REST API documentation
- [Architecture](architecture/system-overview.md) - System design and components
- [Development Setup](development/getting-started.md) - Contributor onboarding
- [Crawl4AI Comparison](CRAWL4AI_COMPARISON_REPORT.md) - Feature comparison analysis
- [API Tooling Guide](API_TOOLING_QUICKSTART.md) - CLI, SDK, and playground tools

### By Role
- **Users**: [Installation](user/installation.md) → [API Usage](user/api-usage.md) → [Troubleshooting](user/troubleshooting.md)
- **Developers**: [Development Setup](development/getting-started.md) → [Architecture](architecture/system-overview.md) → [Contributing](development/contributing.md)
- **Operators**: [Production Deployment](deployment/production.md) → [Configuration](architecture/configuration-guide.md) → [Monitoring](archive/implementation-reports/monitoring-implementation-report.md)

## 📝 Documentation Maintenance

Keep documentation current as the project evolves:

- **API Changes**: Update [API Documentation](api/README.md) when adding/modifying endpoints
- **New Components**: Update [System Overview](architecture/system-overview.md) and system diagrams
- **Configuration**: Update [Configuration Guide](architecture/configuration-guide.md) for new settings
- **Deployment**: Update [Deployment Guide](architecture/deployment-guide.md) for infrastructure changes
- **Dependencies**: Update version requirements in Quick Start section below
- **Examples**: Update code examples when APIs change
- **Troubleshooting**: Add new common issues as they're discovered

*Run `cargo doc --open` to verify inline documentation matches actual code.*

## Documentation Structure

### Architecture Documentation

- **[System Overview](architecture/system-overview.md)** - Complete system architecture and data flow
- **[System Diagram](architecture/system-diagram.md)** - Visual architecture diagrams
- **[Configuration Guide](architecture/configuration-guide.md)** - Comprehensive configuration reference and best practices
- **[Deployment Guide](architecture/deployment-guide.md)** - Production deployment instructions for Docker and Kubernetes
- **[WASM Guide](architecture/WASM_GUIDE.md)** - WebAssembly integration guide
- **[PDF Pipeline](architecture/PDF_PIPELINE_GUIDE.md)** - PDF processing architecture

### API Documentation

- **[API Overview](api/README.md)** - Complete API documentation index
- **[Endpoint Catalog](api/ENDPOINT_CATALOG.md)** - All 59 API endpoints
- **[Examples](api/examples.md)** - API usage examples
- **[Streaming](api/streaming.md)** - Real-time streaming protocols
- **[Session Management](api/session-management.md)** - Session handling
- **[Security](api/security.md)** - Security best practices

### User Tools & SDKs

- **[CLI Tool](../cli/README.md)** - Command-line interface for RipTide (npm)
- **[Python SDK](../python-sdk/README.md)** - Official Python client library (PyPI)
- **[Web Playground](../playground/README.md)** - Interactive API playground (React)
- **[API Tooling Quickstart](API_TOOLING_QUICKSTART.md)** - Complete guide to all tools

## Quick Start

### System Requirements

- **Rust**: 1.70+ with `wasm32-wasip2` target
- **Docker**: 20.10+ with Docker Compose
- **Redis**: 7.0+ (included in Docker setup)
- **Serper API Key**: For search functionality

### Local Development

```bash
# 1. Clone and setup
git clone <repository-url>
cd riptide-crawler
export SERPER_API_KEY="your-api-key"

# 2. Build WASM component
cd wasm/riptide-extractor-wasm
cargo build --target wasm32-wasip2 --release
cd ../..

# 3. Start services
docker-compose -f infra/docker/docker-compose.yml up -d

# 4. Test API
curl http://localhost:8080/healthz
```

### Production Deployment

```bash
# Docker Compose (Recommended)
docker-compose -f docker-compose.prod.yml up -d

# Kubernetes
kubectl apply -f k8s/
```

## System Architecture

### Core Components

```
┌─────────────┐    ┌─────────────┐    ┌─────────────────────┐
│ riptide-api │────│ riptide-    │────│ riptide-extractor-  │
│ (REST API)  │    │ headless    │    │ wasm (Component)    │
│ Port: 8080  │    │ (CDP/Chrome)│    │ WASM Content Proc.  │
└─────────────┘    │ Port: 9123  │    └─────────────────────┘
        │            └─────────────┘              │
        │                   │                    │
        └─────────┬─────────┴────────────────────┘
                  │
           ┌─────────────┐    ┌─────────────────────────────┐
           │ riptide-    │    │          Redis              │
           │ core        │    │     (Cache & Queue)         │
           │ (Shared)    │    │      Port: 6379             │
           └─────────────┘    └─────────────────────────────┘
```

### Technology Stack

- **Language**: Rust (Edition 2021)
- **Web Framework**: Axum 0.7
- **Async Runtime**: Tokio
- **Browser Automation**: Chromiumoxide
- **WASM Runtime**: Wasmtime 26 with Component Model
- **Caching**: Redis 7
- **HTTP Client**: Reqwest 0.12

## Key Features

### Performance
- **Concurrent Processing**: 16 default workers (configurable)
- **HTTP/2 Optimization**: Prior knowledge and multiplexing
- **Compression**: Gzip and Brotli support
- **Caching**: Redis-backed read-through cache
- **WASM Processing**: Content extraction

### Web Scraping Capabilities
- **Static Content**: HTTP-based crawling
- **Dynamic Content**: Headless browser rendering for JavaScript-based sites
- **Content Extraction**: Article text, metadata, links, and media
- **Format Support**: HTML, PDF, and other document types
- **Output Formats**: Markdown, JSON, and plain text

### Security & Compliance
- **Robots.txt**: Configurable compliance
- **Rate Limiting**: Configurable crawling delays
- **Stealth Mode**: Anti-detection capabilities
- **Proxy Support**: Optional proxy rotation

## API Endpoints

### Health Check
```bash
GET /healthz
```

### Batch URL Crawling
```bash
POST /crawl
Content-Type: application/json

{
  "urls": ["https://example.com/article1", "https://example.com/article2"],
  "options": {
    "concurrency": 8,
    "cache_mode": "read_through"
  }
}
```

### Search-Driven Crawling
```bash
POST /deepsearch
Content-Type: application/json

{
  "query": "artificial intelligence news",
  "limit": 10
}
```

## Configuration

### Primary Configuration (`configs/riptide.yml`)

```yaml
# HTTP crawling settings
crawl:
  concurrency: 16
  timeout_ms: 20000
  cache: read_through

# Content extraction settings
extraction:
  mode: "article"
  produce_markdown: true
  produce_json: true

# Dynamic content settings
dynamic:
  enable_headless_fallback: true
  scroll:
    enabled: true
    steps: 8

# Infrastructure settings
redis:
  url: "redis://redis:6379/0"
```

### Environment Variables

```bash
# Required
export SERPER_API_KEY="your-serper-api-key"

# Optional
export RUST_LOG="info"
export REDIS_URL="redis://localhost:6379/0"
export HEADLESS_URL="http://localhost:9123"
```

## Development

### Building from Source

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-wasip2

# Build all components
cargo build --release

# Build WASM component
cd wasm/riptide-extractor-wasm
cargo build --target wasm32-wasip2 --release
```

### Running Tests

```bash
# Unit tests
cargo test

# Integration tests (requires running services)
cargo test --test integration

# WASM component tests
cd wasm/riptide-extractor-wasm
cargo test --target wasm32-wasip2
```

### Local Services

```bash
# Start Redis
docker run -d --name redis -p 6379:6379 redis:7-alpine

# Start headless service
cargo run --bin riptide-headless

# Start API service
cargo run --bin riptide-api -- --config configs/riptide.yml
```

## Monitoring and Debugging

### Health Monitoring

```bash
# Check service health
curl http://localhost:8080/healthz
curl http://localhost:9123/healthz

# Check Redis connectivity
redis-cli ping
```

### Performance Monitoring

Key metrics to track:
- Request latency (p50, p95, p99)
- Request rate (requests/second)
- Error rate (4xx, 5xx responses)
- Cache hit ratio
- Memory and CPU usage
- Headless browser utilization

### Debugging

```bash
# Enable debug logging
export RUST_LOG="debug,riptide_core=trace"

# Monitor service logs
docker-compose logs -f api
docker-compose logs -f headless

# Check configuration
curl http://localhost:8080/config  # If config endpoint exists
```

## Troubleshooting

### Common Issues

**Service Not Starting**:
- Check port availability: `netstat -tulpn | grep :8080`
- Verify configuration: `yaml-lint configs/riptide.yml`
- Check environment variables: `env | grep SERPER_API_KEY`

**Connection Errors**:
- Test Redis: `redis-cli -h localhost ping`
- Test headless service: `curl http://localhost:9123/healthz`
- Check network connectivity between services

**Performance Issues**:
- Monitor resource usage: `docker stats`
- Adjust concurrency settings in configuration
- Check cache hit rates in logs
- Consider scaling headless service for dynamic content

### Error Codes

- **400 Bad Request**: Invalid request format or parameters
- **429 Too Many Requests**: Rate limiting (adjust concurrency)
- **500 Internal Server Error**: Service configuration or dependency issues
- **502 Bad Gateway**: Service communication failure
- **503 Service Unavailable**: Temporary overload or maintenance

## Contributing

### Development Workflow

1. **Fork** the repository
2. **Create** a feature branch: `git checkout -b feature/new-feature`
3. **Make** changes and add tests
4. **Test** locally: `cargo test && docker-compose up -d && integration-tests`
5. **Commit** with descriptive messages
6. **Push** and create a pull request

### Code Standards

- Follow Rust standard formatting: `cargo fmt`
- Pass all lints: `cargo clippy`
- Maintain test coverage: `cargo test`
- Update documentation for API changes
- Use semantic versioning for releases

## License

Apache-2.0 License - see [LICENSE](../LICENSE) file for details.

## Support

- **Documentation**: This repository's docs/ directory
- **Issues**: GitHub Issues for bug reports and feature requests
- **Discussions**: GitHub Discussions for questions and community support

## Changelog

### v0.1.0 (Current - Production Ready)
- REST API with /health, /crawl, and /deepsearch endpoints
- WASM Component Model content extraction with wasm-rs integration
- Error handling and performance monitoring
- Instance pooling and circuit breaker patterns
- Test suite with golden tests and benchmarks
- Codebase with zero compilation errors
- Redis caching integration with read-through patterns
- Performance optimization and monitoring system

### Roadmap
- Worker service for background processing
- Enhanced search provider support
- Advanced content filtering and processing
- Real-time crawling with WebSocket support
- Distributed crawling coordination
- Enhanced monitoring and analytics