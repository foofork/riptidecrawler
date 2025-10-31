# Docker Variants and Examples

This directory contains alternative Docker configurations and variants for RipTide.

## Files in this Directory

### 🔬 Dockerfile.api.wasm
**WASM-enabled variant** with configurable build arguments.

- Supports both native and WASM extraction modes
- Use `ENABLE_WASM=true` for WASM support (sandboxed extraction)
- Use `ENABLE_WASM=false` for native-only build
- Larger image size due to WASM runtime and extractor module

**When to use:**
- Testing WASM vs native performance
- Security-critical deployments requiring sandboxed extraction
- Environments where WASM portability is valuable

**Build example:**
```bash
# WASM-enabled build
docker build -f examples/docker/Dockerfile.api.wasm \
  --build-arg ENABLE_WASM=true \
  -t riptide-api:wasm .

# Native-only build (same as production Dockerfile)
docker build -f examples/docker/Dockerfile.api.wasm \
  --build-arg ENABLE_WASM=false \
  -t riptide-api:native .
```

---

### 📊 docker-compose.variants.yml
**Side-by-side comparison** configuration running both native and WASM variants simultaneously.

- Runs `riptide-api-native` on port 8080
- Runs `riptide-api-wasm` on port 8081
- Shared Redis backend
- Ideal for benchmarking and performance testing

**When to use:**
- Comparing native vs WASM extraction performance
- Testing both extractors with the same workload
- Performance benchmarking and optimization work

**Usage:**
```bash
# Start both variants
docker-compose -f examples/docker/docker-compose.variants.yml up -d

# Test native variant (port 8080)
curl -X POST http://localhost:8080/crawl \
  -H "Content-Type: application/json" \
  -d '{"urls": ["https://example.com"]}'

# Test WASM variant (port 8081)
curl -X POST http://localhost:8081/crawl \
  -H "Content-Type: application/json" \
  -d '{"urls": ["https://example.com"]}'

# View logs
docker-compose -f examples/docker/docker-compose.variants.yml logs -f

# Stop services
docker-compose -f examples/docker/docker-compose.variants.yml down
```

---

### 💡 docker-compose.lite.yml
**Lightweight deployment** using native-only builds with minimal services.

- RipTide API with native extraction
- Redis for caching
- Swagger UI for API documentation
- No separate headless browser service (uses WASM extraction fallback)
- Smallest memory footprint (~440MB vs ~1.2GB)

**When to use:**
- Memory-constrained environments
- Static content extraction (no JavaScript required)
- Development and testing
- Cost-sensitive deployments

**Usage:**
```bash
# Start lite deployment
docker-compose -f examples/docker/docker-compose.lite.yml up -d

# View logs
docker-compose -f examples/docker/docker-compose.lite.yml logs -f

# Stop services
docker-compose -f examples/docker/docker-compose.lite.yml down
```

---

## Production Deployment

For **production use**, use the main configuration files in the project root:

- `/infra/docker/Dockerfile.api` - Native-only, optimized for production
- `/docker-compose.yml` - Full-featured deployment with headless browser

These are optimized for:
- **Performance**: Native parser only, no WASM overhead
- **Reliability**: Full Chrome rendering with 5-browser pool
- **Features**: Complete functionality including JavaScript execution

```bash
# Production deployment
docker-compose up -d
```

---

## Quick Reference

| Variant | Image Size | Memory | Use Case |
|---------|-----------|--------|----------|
| **Production** (root) | ~800MB | 1.2GB | Production deployments |
| **Lite** | ~700MB | 440MB | Memory-constrained, static content |
| **WASM** | ~900MB | 2GB | Security testing, WASM benchmarking |
| **Variants** | Both | 3GB | Performance comparison |

---

## Architecture Comparison

### Production (Native-only)
```
┌─────────────────┐
│  RipTide API    │  Native Parser
│  (Native Only)  │  No WASM Runtime
└────────┬────────┘
         │
    ┌────┴─────┐
    │  Redis   │
    └──────────┘
```

### WASM Variant
```
┌─────────────────┐
│  RipTide API    │  Native Parser
│  + WASM Runtime │  + WASM Extractor
└────────┬────────┘
         │
    ┌────┴─────┐
    │  Redis   │
    └──────────┘
```

### Variants (Comparison)
```
┌─────────────┐    ┌─────────────┐
│ Native API  │    │  WASM API   │
│   :8080     │    │   :8081     │
└──────┬──────┘    └──────┬──────┘
       └────────┬──────────┘
                │
           ┌────┴─────┐
           │  Redis   │
           └──────────┘
```

---

## Migration Notes

**From old WASM builds:**
- Default production Dockerfile is now native-only
- WASM builds moved to `examples/docker/Dockerfile.api.wasm`
- Use `ENABLE_WASM=true` build arg for WASM support
- No breaking changes to existing deployments using root `docker-compose.yml`

**Environment variables:**
- `WASM_EXTRACTOR_PATH` - Removed from production `docker-compose.yml`
- Still available in WASM variant when `ENABLE_WASM=true`
- Native parser uses no environment configuration

---

## Support

For issues or questions:
- Production Docker: See main README.md
- WASM builds: Check WASM_OPTIONAL_COMPREHENSIVE_PLAN.md
- Performance: See docs/performance/

---

**Last Updated:** 2025-10-31
**Maintained By:** RipTide Development Team
