# RipTide Docker Deployment Modes

RipTide offers three docker-compose configurations optimized for different use cases, from zero-dependency local development to full-featured distributed production deployments.

## 📋 Quick Reference

| Feature | Minimal | Simple | Distributed (Full) |
|---------|---------|--------|-------------------|
| **File** | `docker-compose.minimal.yml` | `docker-compose.simple.yml` | `docker-compose.yml` |
| **Redis** | ❌ In-memory | ✅ Persistent | ✅ Persistent |
| **Workers** | ❌ Sync only | ❌ Sync only | ✅ Background jobs |
| **Browser** | ❌ WASM only | ❌ WASM only | ✅ Chrome pool |
| **Memory** | ~440MB | ~600MB | ~1.2GB |
| **Containers** | 1 (API only) | 2 (API + Redis) | 3+ (API + Redis + Chrome) |
| **Cache Persistence** | ❌ Cleared on restart | ✅ Survives restart | ✅ Survives restart |
| **JavaScript Rendering** | ❌ Static only | ❌ Static only | ✅ Full Chrome |
| **Best For** | Dev, CI/CD, Testing | Small production | Production, High-volume |

---

## 🎯 Mode Selection Guide

### Use **Minimal** when:
- ✅ Local development and testing
- ✅ CI/CD integration tests
- ✅ Learning and experimentation
- ✅ Simple extraction tasks (1-100 URLs)
- ✅ Zero external dependencies required
- ✅ Minimal resource footprint needed

### Use **Simple** when:
- ✅ Development with persistent cache
- ✅ Small-scale production (< 1000 req/day)
- ✅ API integration development
- ✅ Cache persistence required
- ✅ Single API instance sufficient

### Use **Distributed (Full)** when:
- ✅ Production deployments
- ✅ High-volume workloads (> 1000 req/day)
- ✅ JavaScript rendering required
- ✅ Background job processing needed
- ✅ Multiple API instances for HA
- ✅ Browser pool for dynamic content

---

## 1️⃣ Minimal Mode

**Zero-dependency deployment** - Single container, no external services.

### Features
```yaml
✅ Single RipTide API container
✅ In-memory cache (3600s TTL)
✅ WASM extraction (fast static content)
✅ 440MB memory footprint
❌ No Redis (cache clears on restart)
❌ No background workers
❌ No headless browser
```

### Quick Start
```bash
# Start minimal deployment
docker-compose -f docker-compose.minimal.yml up -d

# View logs
docker-compose -f docker-compose.minimal.yml logs -f

# Stop services
docker-compose -f docker-compose.minimal.yml down
```

### Configuration
```bash
# Environment variables (optional)
export RIPTIDE_API_PORT=8080
export RUST_LOG=info
export CACHE_MAX_ENTRIES=10000
export CACHE_MEMORY_TTL=3600

# Start with custom config
docker-compose -f docker-compose.minimal.yml up -d
```

### Resource Requirements
- **Memory**: ~440MB
- **CPU**: 0.5-2.0 cores
- **Disk**: Minimal (logs only)

### Use Cases
- **CI/CD Integration Tests**
  ```bash
  # In your CI pipeline
  docker-compose -f docker-compose.minimal.yml up -d
  npm run test:integration
  docker-compose -f docker-compose.minimal.yml down
  ```

- **Local Development**
  ```bash
  # Quick testing without external dependencies
  docker-compose -f docker-compose.minimal.yml up -d
  curl http://localhost:8080/extract?url=example.com
  ```

- **Learning & Experimentation**
  ```bash
  # Try out RipTide features
  docker-compose -f docker-compose.minimal.yml up -d
  # Experiment with API endpoints
  ```

### Limitations
⚠️ **Cache clears on restart** - No persistence
⚠️ **Synchronous execution** - Limited throughput
⚠️ **No JavaScript rendering** - Static content only
⚠️ **Single process** - No parallel processing

---

## 2️⃣ Simple Mode

**API + Redis** - Persistent caching without background workers.

### Features
```yaml
✅ RipTide API container
✅ Redis for persistent caching
✅ WASM extraction
✅ Cache survives restarts
✅ 600MB memory footprint
❌ No background workers
❌ No headless browser
```

### Quick Start
```bash
# Start simple deployment
docker-compose -f docker-compose.simple.yml up -d

# View logs
docker-compose -f docker-compose.simple.yml logs -f

# Stop services
docker-compose -f docker-compose.simple.yml down
```

### Configuration
```bash
# Environment variables
export RIPTIDE_API_PORT=8080
export REDIS_PORT=6379
export RUST_LOG=info
export RIPTIDE_MAX_CONCURRENT_REQUESTS=100

# Start with custom config
docker-compose -f docker-compose.simple.yml up -d
```

### Resource Requirements
- **Memory**: ~600MB (API: 440MB, Redis: 160MB)
- **CPU**: 1.0-2.5 cores
- **Disk**: ~100MB (Redis persistence)

### Use Cases
- **Development with Persistent Cache**
  ```bash
  # Cache persists across restarts
  docker-compose -f docker-compose.simple.yml up -d
  # Extract content (cached)
  curl http://localhost:8080/extract?url=example.com
  # Restart - cache survives
  docker-compose -f docker-compose.simple.yml restart
  curl http://localhost:8080/extract?url=example.com  # Instant cache hit
  ```

- **Small Production Workloads**
  ```bash
  # Handle moderate traffic with caching
  docker-compose -f docker-compose.simple.yml up -d
  # Suitable for < 1000 requests/day
  ```

- **API Integration Development**
  ```bash
  # Test with real caching infrastructure
  docker-compose -f docker-compose.simple.yml up -d
  # Develop against production-like setup
  ```

### Performance Characteristics
- **Throughput**: ~50-100 req/min
- **Latency**: 200ms-2s per request
- **Cache Hit Rate**: 70-90% for repeated URLs

### Scaling
```bash
# Scale API instances (requires load balancer)
docker-compose -f docker-compose.simple.yml up -d --scale riptide-api=3

# Note: Configure external load balancer for traffic distribution
```

---

## 3️⃣ Distributed Mode (Full)

**Production deployment** - Complete feature set with Chrome rendering.

### Features
```yaml
✅ RipTide API container
✅ Redis for caching
✅ Chrome browser pool (5 browsers)
✅ Background workers
✅ JavaScript rendering
✅ Full production capabilities
✅ 1.2GB memory footprint
```

### Quick Start
```bash
# Start distributed deployment (recommended for production)
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

### Configuration
```bash
# Copy environment template
cp .env.example .env

# Edit .env with your settings
nano .env

# Required: Add API keys
SERPER_API_KEY=your-key-here

# Start services
docker-compose up -d
```

### Resource Requirements
- **Memory**: ~1.2GB (API: 440MB, Redis: 160MB, Chrome: 600MB)
- **CPU**: 2.0-4.0 cores
- **Disk**: ~500MB (Redis + logs + cache)

### Architecture
```
┌─────────────────────────────────────────────┐
│          Load Balancer (Optional)           │
└────────────┬────────────────────────────────┘
             │
    ┌────────┴────────┐
    │                 │
┌───▼────┐      ┌────▼───┐
│ API #1 │      │ API #2 │  (Scalable)
└───┬────┘      └────┬───┘
    │                │
    └────────┬───────┘
             │
    ┌────────▼─────────┐
    │      Redis       │  (Shared cache)
    └──────────────────┘
             │
    ┌────────▼─────────┐
    │  Chrome Pool     │  (5 browsers)
    └──────────────────┘
```

### Use Cases
- **Production Deployments**
  ```bash
  # Full-featured production setup
  docker-compose up -d

  # Monitor health
  docker-compose ps
  curl http://localhost:8080/health
  ```

- **High-Volume Workloads**
  ```bash
  # Handle > 1000 requests/day
  docker-compose up -d

  # Scale API instances
  docker-compose up -d --scale riptide-api=3
  ```

- **JavaScript-Heavy Sites**
  ```bash
  # Extract dynamic content
  curl -X POST http://localhost:8080/extract \
    -H "Content-Type: application/json" \
    -d '{"url": "https://spa-site.com", "render": true}'
  ```

### Advanced Configuration

#### External Browser Farm
```yaml
# docker-compose.override.yml
services:
  riptide-api:
    environment:
      - HEADLESS_URL=https://your-browser-farm.example.com:9123

# Comment out riptide-headless service in docker-compose.yml
```

#### Custom Worker Configuration
```yaml
# docker-compose.override.yml
services:
  riptide-api:
    environment:
      - WORKERS_ENABLED=true
      - WORKER_COUNT=8
      - JOB_TIMEOUT=300
```

### Scaling Strategies

#### Horizontal Scaling (Multiple API Instances)
```bash
# Scale to 3 API instances
docker-compose up -d --scale riptide-api=3

# Configure Nginx/HAProxy for load balancing
# See: docs/deployment/load-balancing.md
```

#### Vertical Scaling (More Resources)
```yaml
# docker-compose.override.yml
services:
  riptide-api:
    deploy:
      resources:
        limits:
          cpus: '4.0'
          memory: 4G
```

---

## 🔄 Switching Between Modes

### Upgrade Path: Minimal → Simple → Distributed

#### From Minimal to Simple
```bash
# Stop minimal deployment
docker-compose -f docker-compose.minimal.yml down

# Start simple deployment (adds Redis)
docker-compose -f docker-compose.simple.yml up -d
```

#### From Simple to Distributed
```bash
# Stop simple deployment
docker-compose -f docker-compose.simple.yml down

# Start distributed deployment (adds Chrome + workers)
docker-compose up -d
```

### Downgrade Path: Distributed → Simple → Minimal

#### From Distributed to Simple
```bash
# Stop distributed deployment
docker-compose down

# Start simple deployment (removes Chrome + workers)
docker-compose -f docker-compose.simple.yml up -d
```

#### From Simple to Minimal
```bash
# Stop simple deployment
docker-compose -f docker-compose.simple.yml down

# Start minimal deployment (removes Redis)
docker-compose -f docker-compose.minimal.yml up -d
```

### Preserving Data During Transitions

```bash
# Backup Redis data before downgrade
docker-compose exec redis redis-cli SAVE
docker cp riptide-redis:/data/dump.rdb ./backup-dump.rdb

# Restore after upgrade
docker cp ./backup-dump.rdb riptide-redis:/data/dump.rdb
docker-compose restart redis
```

---

## 🧪 Testing Each Mode

### Test Minimal Mode
```bash
# Start
docker-compose -f docker-compose.minimal.yml up -d

# Health check
curl http://localhost:8080/health

# Extract static content
curl "http://localhost:8080/extract?url=https://example.com"

# Verify in-memory cache
curl "http://localhost:8080/extract?url=https://example.com"  # Should be instant

# Restart and verify cache cleared
docker-compose -f docker-compose.minimal.yml restart
curl "http://localhost:8080/extract?url=https://example.com"  # Fresh extraction

# Cleanup
docker-compose -f docker-compose.minimal.yml down
```

### Test Simple Mode
```bash
# Start
docker-compose -f docker-compose.simple.yml up -d

# Health check
curl http://localhost:8080/health

# Extract and cache
curl "http://localhost:8080/extract?url=https://example.com"

# Restart and verify cache persists
docker-compose -f docker-compose.simple.yml restart
curl "http://localhost:8080/extract?url=https://example.com"  # Instant cache hit

# Check Redis
docker-compose -f docker-compose.simple.yml exec redis redis-cli DBSIZE

# Cleanup
docker-compose -f docker-compose.simple.yml down
```

### Test Distributed Mode
```bash
# Start
docker-compose up -d

# Health check all services
curl http://localhost:8080/health
docker-compose ps

# Extract with Chrome rendering
curl -X POST http://localhost:8080/extract \
  -H "Content-Type: application/json" \
  -d '{"url": "https://react-app.com", "render": true}'

# Check browser pool
docker-compose logs riptide-headless | grep "Browser pool"

# Cleanup
docker-compose down
```

---

## 🔍 Monitoring & Debugging

### View Logs
```bash
# Minimal mode
docker-compose -f docker-compose.minimal.yml logs -f riptide-api

# Simple mode
docker-compose -f docker-compose.simple.yml logs -f riptide-api
docker-compose -f docker-compose.simple.yml logs -f redis

# Distributed mode
docker-compose logs -f riptide-api
docker-compose logs -f redis
docker-compose logs -f riptide-headless
```

### Check Resource Usage
```bash
# All modes
docker stats

# Specific container
docker stats riptide-api
```

### Inspect Health
```bash
# API health
curl http://localhost:8080/health

# Redis health (simple/distributed)
docker-compose exec redis redis-cli ping

# Container health
docker-compose ps
```

---

## 📊 Performance Comparison

| Metric | Minimal | Simple | Distributed |
|--------|---------|--------|-------------|
| **Startup Time** | ~5s | ~15s | ~40s |
| **Memory Usage** | 440MB | 600MB | 1.2GB |
| **Cold Request** | 500ms-2s | 500ms-2s | 500ms-2s |
| **Cache Hit** | 1-5ms | 10-20ms | 10-20ms |
| **Throughput** | ~30 req/min | ~50 req/min | ~200 req/min |
| **JavaScript Support** | ❌ | ❌ | ✅ |

---

## 🛡️ Security Considerations

### Minimal Mode
- No network exposure beyond API port
- In-memory data only (no persistence attacks)
- Smallest attack surface

### Simple Mode
- Secure Redis with authentication
- Network isolation between containers
- Persistent data requires backup strategy

### Distributed Mode
- Secure browser isolation (sandboxed Chrome)
- Rate limiting for API protection
- Multiple attack vectors to monitor

### Security Best Practices
```bash
# Set API key authentication
export RIPTIDE_API_KEY=$(openssl rand -hex 32)
export REQUIRE_AUTH=true

# Enable HTTPS (use reverse proxy)
# See: docs/deployment/ssl-setup.md

# Restrict CORS
export CORS_ORIGINS=https://yourdomain.com

# Start with security enabled
docker-compose up -d
```

---

## 🚀 Production Recommendations

### For Small Projects (< 100 req/day)
```bash
# Use minimal mode
docker-compose -f docker-compose.minimal.yml up -d
```

### For Medium Projects (100-1000 req/day)
```bash
# Use simple mode with monitoring
docker-compose -f docker-compose.simple.yml up -d
# Add monitoring: Prometheus + Grafana
```

### For Large Projects (> 1000 req/day)
```bash
# Use distributed mode with scaling
docker-compose up -d --scale riptide-api=3
# Add load balancer, monitoring, auto-scaling
```

---

## 📚 Additional Resources

- [API Documentation](../api/README.md)
- [Configuration Guide](../config/README.md)
- [Load Balancing Setup](./load-balancing.md)
- [SSL/TLS Configuration](./ssl-setup.md)
- [Monitoring & Metrics](./monitoring.md)
- [Troubleshooting Guide](./troubleshooting.md)

---

## 🆘 Support

- **GitHub Issues**: https://github.com/ruvnet/riptide/issues
- **Documentation**: https://docs.riptide.dev
- **Community**: https://discord.gg/riptide

---

**Last Updated**: 2025-11-12
**Version**: 2.0.0
