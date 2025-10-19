# RipTide Frequently Asked Questions (FAQ)

**Version:** 2.0.0
**Last Updated:** October 17, 2025

---

## General Questions

### What is RipTide?

RipTide is a high-performance web crawler and content extraction API built in Rust with WebAssembly optimization. It delivers enterprise-grade web crawling with WASM-powered extraction, real-time streaming, and intelligent adaptive routing.

### What's new in v2.0?

V2.0 introduces major performance breakthroughs:
- **91% faster** WASM extraction
- **94% faster** headless extraction
- **2.5x throughput** increase
- API-first architecture with intelligent fallback
- Five optimization layers (engine cache, WASM cache, browser pool, AOT compilation, adaptive timeouts)

See [CHANGELOG.md](../CHANGELOG.md) for complete details.

### Is v2.0 backward compatible with v1.0?

Yes! v2.0 is fully backward compatible with v1.0:
- No breaking changes to existing APIs
- All v1.0 code continues to work
- New features are opt-in
- Direct mode preserves v1.0 behavior with optimizations

---

## Operation Modes

### When should I use API-first mode vs direct mode?

**Use API-first mode (default)** when:
- Running in production with centralized infrastructure
- Need load balancing and horizontal scaling
- Want centralized caching and monitoring
- Working in a team environment
- Need high availability (automatic fallback)

**Use direct mode** when:
- Working offline without API server
- Local development and testing
- Single-machine deployments
- Need lowest possible latency for single operations
- Prototyping and experimentation

**Quick answer:** API-first mode is recommended for most users.

### What happens if the API server is unavailable in API-first mode?

The CLI automatically falls back to direct execution with zero data loss:
1. Health check times out (5s)
2. CLI switches to direct mode
3. All optimizations remain active
4. Operation completes successfully

### How do I enforce API-only mode for compliance?

Use the `--api-only` flag or set `RIPTIDE_CLI_MODE=api_only`:

```bash
export RIPTIDE_CLI_MODE=api_only
riptide extract --url https://example.com
```

This prevents fallback to direct execution and fails if API is unavailable.

---

## Performance

### Why is my first extraction slow but subsequent ones fast?

This is expected behavior (cold start vs warm):

**First Request (Cold Start):**
- Time: ~350ms (WASM) or ~8200ms (Headless)
- All caches empty
- WASM module needs compilation
- Browser pool needs warming
- Timeout uses conservative default

**Second Request (Warm):**
- Time: ~30ms (WASM) or ~500ms (Headless)
- Engine selection: CACHE HIT
- WASM module: CACHE HIT
- Browser pool: WARM CHECKOUT
- Timeout: LEARNED OPTIMAL
- **91-94% faster!**

**Solution:** Pre-warm caches for known domains.

### How do I tune the browser pool size?

Recommendations based on workload:

```bash
# Light workload (< 10 req/min)
RIPTIDE_HEADLESS_POOL_SIZE=1

# Medium workload (10-50 req/min)
RIPTIDE_HEADLESS_POOL_SIZE=2

# Heavy workload (> 50 req/min)
RIPTIDE_HEADLESS_POOL_SIZE=3

# Very heavy (> 100 req/min)
# Consider horizontal scaling instead
```

Monitor with: `riptide metrics | grep browser_pool`

### What are the cache directories and can I delete them?

Cache directories:
- `RIPTIDE_CACHE_DIR/engine-selection/` - Engine selection cache
- `RIPTIDE_CACHE_DIR/wasm-aot/` - WASM AOT compilation cache
- `RIPTIDE_CACHE_DIR/adaptive-timeout/` - Learned timeout profiles

**Safe to delete:** Yes, but performance will return to cold start until warmed again.

**Recommendation:** Use `riptide cache clear` for safe cleanup.

### How do I monitor performance improvements?

```bash
# Check optimization metrics
riptide metrics -o json | jq '.optimization_layers'

# Expected output:
# {
#   "engine_cache_hit_rate": 0.80,
#   "wasm_cache_hit_rate": 0.995,
#   "browser_pool_warm_rate": 0.94,
#   "wasm_aot_cache_hit_rate": 0.67,
#   "adaptive_timeout_accuracy": 0.87
# }

# Cache statistics
riptide cache stats

# System health
riptide system-check
```

See [PERFORMANCE_GUIDE.md](PERFORMANCE_GUIDE.md) for detailed monitoring.

---

## Troubleshooting

### API connection failed - what should I do?

**Quick fixes:**

1. Check API server is running:
   ```bash
   curl http://localhost:8080/healthz
   ```

2. Verify environment variables:
   ```bash
   echo $RIPTIDE_API_URL
   echo $RIPTIDE_API_KEY
   ```

3. Use direct mode temporarily:
   ```bash
   riptide --direct extract --url https://example.com
   ```

### WASM module loading failed

**Solutions:**

1. Verify WASM module exists:
   ```bash
   ls target/wasm32-wasip2/release/
   ```

2. Rebuild WASM component:
   ```bash
   cd wasm/riptide-extractor-wasm
   cargo build --target wasm32-wasip2 --release
   ```

3. Clear WASM cache:
   ```bash
   rm -rf riptide-output/cache/wasm
   ```

### Browser pool exhausted - "No available browsers"

**Solutions:**

1. Increase pool size:
   ```bash
   export RIPTIDE_HEADLESS_POOL_SIZE=3
   ```

2. Check Chrome is installed:
   ```bash
   which chromium || which google-chrome
   ```

3. Verify headless service:
   ```bash
   curl http://localhost:9123/health
   ```

### Redis connection failed

**Solutions:**

1. Start Redis:
   ```bash
   docker run -d --name redis -p 6379:6379 redis:7-alpine
   ```

2. Verify connection:
   ```bash
   redis-cli -u redis://localhost:6379 ping
   ```

3. Disable cache temporarily:
   ```bash
   export CACHE_TTL=0
   ```

### Extraction is slower than expected

**Diagnostic steps:**

1. Check if optimizations are active:
   ```bash
   riptide cache status
   ```

2. Warm up caches (run multiple times):
   ```bash
   for i in {1..5}; do
     riptide extract --url https://example.com --direct
   done
   ```

3. Check metrics:
   ```bash
   riptide metrics -o json > metrics.json
   ```

4. Enable debug logging:
   ```bash
   export RIPTIDE_CLI_VERBOSE=true
   export RUST_LOG=debug
   riptide extract --url https://example.com
   ```

---

## Configuration

### Where should I put configuration files?

**Environment variables (.env file):**
```bash
cp .env.example .env
# Edit .env with your configuration
```

**Configuration priority:**
1. Command-line flags (highest)
2. Environment variables
3. .env file
4. Default values (lowest)

### What environment variables are required?

**Minimum required:**
```bash
# For API mode
RIPTIDE_API_URL=http://localhost:8080
REDIS_URL=redis://localhost:6379/0

# For search functionality
SERPER_API_KEY=your_serper_api_key_here
```

**All other variables are optional** with sensible defaults.

See [ENVIRONMENT_VARIABLES.md](configuration/ENVIRONMENT_VARIABLES.md) for complete reference.

### How do I configure output directories?

```bash
# Base output directory
export RIPTIDE_OUTPUT_DIR=./my-output

# Subdirectories (optional overrides)
export RIPTIDE_EXTRACT_DIR=./my-output/extractions
export RIPTIDE_CRAWL_DIR=./my-output/crawls
export RIPTIDE_LOGS_DIR=./my-output/logs
```

See [OUTPUT_DIRECTORIES.md](configuration/OUTPUT_DIRECTORIES.md) for details.

---

## Features

### What extraction methods are available?

RipTide supports multiple extraction strategies:

1. **TREK (WASM)**: Fast, optimal for most pages (~30ms with cache)
2. **CSS Selectors**: Precise element extraction
3. **LLM-Enhanced**: For complex or non-standard content
4. **Regex Patterns**: Pattern-based extraction
5. **Headless Browser**: For JavaScript-heavy pages (~500ms with pool)

**Strategy composition:**
```bash
# Chain multiple methods
riptide extract --strategy "chain:css,regex" --url https://example.com

# Parallel execution
riptide extract --strategy "parallel:all" --url https://example.com

# Fallback chain
riptide extract --strategy "fallback:css" --url https://example.com
```

### Does RipTide support streaming?

Yes! Three streaming protocols:

1. **NDJSON** (Newline-Delimited JSON):
   ```bash
   riptide crawl --url https://example.com --stream
   ```

2. **Server-Sent Events (SSE)**:
   ```bash
   curl -N http://localhost:8080/crawl/sse?url=https://example.com
   ```

3. **WebSocket** (bidirectional):
   ```bash
   # Connect to ws://localhost:8080/crawl/ws
   ```

### Can I crawl JavaScript-heavy websites?

Yes! RipTide supports multiple approaches:

1. **Headless browser rendering** (automatic for JS-heavy sites)
2. **Stealth mode** for anti-bot protection
3. **Session management** for authentication
4. **Custom headers and cookies**

```bash
# Force headless rendering
riptide extract --url https://js-heavy-site.com --method headless

# With stealth mode
riptide extract --url https://protected-site.com --stealth
```

### Does RipTide support PDF extraction?

Yes! Comprehensive PDF processing:

```bash
# Extract text from PDF
riptide extract --url https://example.com/document.pdf

# Extract tables
riptide extract --url https://example.com/report.pdf --extract-tables

# Streaming for large PDFs
riptide extract --url https://example.com/large.pdf --stream
```

---

## Deployment

### Can I deploy RipTide in Docker?

Yes! Docker deployment is recommended:

```bash
# Using docker-compose (easiest)
docker-compose up -d

# Verify deployment
curl http://localhost:8080/healthz
```

See [DEPLOYMENT.md](DEPLOYMENT.md) for production deployment guide.

### How do I scale RipTide horizontally?

**Load balancing approaches:**

1. **Multiple API servers** behind load balancer:
   ```
   Load Balancer
   ├── API Server 1 (with Redis)
   ├── API Server 2 (with Redis)
   └── API Server 3 (with Redis)
   ```

2. **Shared Redis** for cache consistency
3. **Browser pool per server** (no sharing needed)
4. **Stateless API servers** (easy to scale)

**Recommendations:**
- 1 API server per 100 req/min
- 1 Redis instance (shared across servers)
- 3 browser pool size per server

### What are the system requirements?

**Minimum:**
- CPU: 2 cores
- RAM: 2GB
- Disk: 10GB
- OS: Linux, macOS, Windows

**Recommended (Production):**
- CPU: 4+ cores
- RAM: 4-8GB
- Disk: 50GB+ SSD
- OS: Linux (Ubuntu 20.04+, Debian 11+)

**For Heavy Workloads:**
- CPU: 8+ cores
- RAM: 16-32GB
- Disk: 100GB+ SSD
- Network: 1Gbps+

### How much does RipTide cost to run?

RipTide is **open source and free**! Operational costs:

**Cloud VM (AWS/GCP/Azure):**
- Small instance: $20-50/month
- Medium instance: $100-200/month
- Large instance: $300-500/month

**External Dependencies:**
- Redis: Free (open source) or managed ~$10-30/month
- Serper API: ~$50/month for 10k searches

**Self-hosted:**
- Hardware costs only
- Zero licensing fees

---

## Development

### How do I contribute to RipTide?

See [CONTRIBUTING.md](development/contributing.md) for guidelines.

**Quick start:**
1. Fork repository
2. Create feature branch
3. Make changes with tests
4. Run `cargo test && cargo clippy`
5. Submit pull request

### What's the test coverage?

**v2.0 Coverage:**
- Overall: **91%**
- Core modules: **90%+**
- Total tests: **188 tests**
- Test lines: **11,659 lines**

Run tests:
```bash
cargo test
```

### How can I add a new extraction method?

See [development/EXTRACTION_GUIDE.md](development/EXTRACTION_GUIDE.md) for implementation guide.

**Basic steps:**
1. Implement `Extractor` trait
2. Add to engine selection logic
3. Write comprehensive tests
4. Document method and use cases
5. Submit pull request

---

## Support

### Where can I get help?

- **Documentation**: [docs/](.)
- **Issues**: [GitHub Issues](https://github.com/your-org/riptide/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-org/riptide/discussions)
- **User Guide**: [USER_GUIDE.md](USER_GUIDE.md)
- **Troubleshooting**: [Troubleshooting Guide](user/troubleshooting.md)

### How do I report a bug?

1. Check existing issues: [GitHub Issues](https://github.com/your-org/riptide/issues)
2. Create new issue with:
   - RipTide version (`riptide --version`)
   - Operating system and version
   - Steps to reproduce
   - Expected vs actual behavior
   - Relevant logs (`RUST_LOG=debug`)

### How do I request a feature?

1. Check [GitHub Discussions](https://github.com/your-org/riptide/discussions)
2. Create feature request with:
   - Use case description
   - Expected behavior
   - Example usage
   - Why it's valuable

---

## Migration

### How do I upgrade from v1.0 to v2.0?

See [MIGRATION_GUIDE.md](guides/MIGRATION_GUIDE.md) for detailed instructions.

**Quick upgrade:**
```bash
# 1. Update code
git pull origin main
cargo build --release

# 2. Configure API-first mode (optional)
export RIPTIDE_API_URL=http://localhost:8080
export RIPTIDE_API_KEY=your_token

# 3. Use as before (optimizations automatic)
riptide extract --url https://example.com
```

**No breaking changes** - v2.0 is backward compatible!

### Will my v1.0 scripts break?

No! v2.0 is fully backward compatible:
- All v1.0 commands work unchanged
- All v1.0 APIs remain functional
- New features are opt-in
- Direct mode preserves v1.0 behavior

### Do I need to change my configuration?

No required changes! New optional features:

```bash
# Optional: Enable API-first mode for best performance
export RIPTIDE_API_URL=http://localhost:8080
export RIPTIDE_API_KEY=your_token
export RIPTIDE_CLI_MODE=api_first
```

---

## Performance Optimization

### What's the fastest way to extract content?

**For single pages:**
```bash
# API-first mode with warm caches (fastest)
riptide extract --url https://example.com
# Time: ~30ms (after warmup)
```

**For batch operations:**
```bash
# Batch API call (amortize cold start)
curl -X POST http://localhost:8080/crawl \
  -d '{"urls": ["url1", "url2", ...], "options": {"concurrency": 10}}'
```

**For repeated domains:**
```bash
# Pre-warm caches first
riptide extract --url https://example.com --direct
# Subsequent requests: 91% faster
```

### How do I reduce memory usage?

**Configuration:**
```bash
# Memory limits
export RIPTIDE_MEMORY_LIMIT_MB=1024
export RIPTIDE_MEMORY_MAX_PER_REQUEST_MB=128

# Auto garbage collection
export RIPTIDE_MEMORY_AUTO_GC=true
export RIPTIDE_MEMORY_GC_TRIGGER_MB=512

# Browser pool size (lower = less memory)
export RIPTIDE_HEADLESS_POOL_SIZE=1
```

**v2.0 already optimizes memory:**
- 40% reduction vs v1.0 (1.69GB → 1.03GB)
- Smart pooling and caching
- Automatic cleanup

### What's the optimal configuration for high throughput?

**For 100+ req/min:**
```bash
# Browser pool
export RIPTIDE_HEADLESS_POOL_SIZE=3
export RIPTIDE_HEADLESS_MIN_POOL_SIZE=1

# Concurrency
export RIPTIDE_MAX_CONCURRENT_RENDERS=20
export RIPTIDE_MAX_CONCURRENT_WASM=8

# Timeouts
export RIPTIDE_RENDER_TIMEOUT=3
export RIPTIDE_HTTP_TIMEOUT=10

# Memory
export RIPTIDE_MEMORY_LIMIT_MB=4096
export RIPTIDE_MEMORY_AUTO_GC=true
```

---

**RipTide v2.0** - Built with ⚡ by the Hive Mind collective
