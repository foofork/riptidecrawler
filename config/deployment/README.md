# RipTide Deployment Configurations

This directory contains three deployment configurations for RipTide, each targeting different use cases and infrastructure requirements.

## Quick Selection Guide

| Use Case | Configuration | Command |
|----------|--------------|---------|
| **Local Development** | `minimal.toml` | `cargo run --config config/deployment/minimal.toml` |
| **CI/CD Testing** | `minimal.toml` | `docker run riptide --config /config/minimal.toml` |
| **Production Single-Instance** | `enhanced.toml` | `docker-compose -f docker-compose.simple.yml up` |
| **Enterprise Multi-Instance** | `distributed.toml` | `docker-compose up --scale riptide-api=3` |

---

## Configuration Files

### 🚀 minimal.toml
**Perfect for**: Local development, CI/CD, simple extraction

**Features**:
- ✅ Zero dependencies (no Redis required)
- ✅ In-memory cache with 1-hour TTL
- ✅ Fast startup (<2 seconds)
- ✅ Synchronous job execution
- ⚠️ Cache clears on restart
- ⚠️ No persistent sessions

**Requirements**:
- Memory: ~440MB
- CPU: 0.5-2.0 cores
- Disk: None
- Network: None

**Quick Start**:
```bash
# Run locally
cargo run --config config/deployment/minimal.toml

# Or with Docker
docker run -p 8080:8080 \
  -e CONFIG_PATH=/config/minimal.toml \
  riptide:latest
```

---

### ⚡ enhanced.toml
**Perfect for**: Production single-instance, persistent cache

**Features**:
- ✅ Persistent cache (survives restarts)
- ✅ Session management (24-hour TTL)
- ✅ Better performance for repeated requests
- ✅ Redis-backed state management
- ⚠️ Requires Redis server
- ⚠️ Single-instance only (no horizontal scaling)

**Requirements**:
- Memory: ~700MB (API + Redis)
- CPU: 1.0-2.0 cores
- Disk: ~1GB (Redis persistence)
- Network: Redis connection

**Quick Start**:
```bash
# 1. Start Redis
docker run -d -p 6379:6379 redis:7-alpine

# 2. Start RipTide
cargo run --config config/deployment/enhanced.toml

# Or with Docker Compose
docker-compose -f docker-compose.simple.yml up
```

---

### 🏢 distributed.toml
**Perfect for**: Enterprise production, multi-instance scaling

**Features**:
- ✅ Distributed job queue
- ✅ Multi-instance horizontal scaling
- ✅ Background worker pool
- ✅ Job retry and error recovery
- ✅ Load balancing across instances
- ✅ Distributed coordination
- ⚠️ Requires Redis server
- ⚠️ Requires worker service
- ⚠️ More complex deployment

**Requirements**:
- Memory: ~1.2GB per API instance + ~800MB per worker
- CPU: 2.0-4.0 cores (API + workers)
- Disk: ~5GB (Redis persistence)
- Network: Redis + multi-instance coordination

**Quick Start**:
```bash
# Using Docker Compose (recommended)
docker-compose up --scale riptide-api=3 --scale riptide-worker=2

# Or manually
cargo run --bin riptide-api --config config/deployment/distributed.toml &
cargo run --bin riptide-worker --config config/deployment/distributed.toml &
```

---

## Feature Comparison

| Feature | Minimal | Enhanced | Distributed |
|---------|---------|----------|-------------|
| **Dependencies** | None | Redis | Redis + Workers |
| **Cache Persistence** | ❌ | ✅ | ✅ |
| **Session Persistence** | ❌ | ✅ (24h) | ✅ (24h) |
| **Async Jobs** | ❌ | ❌ | ✅ |
| **Horizontal Scaling** | ❌ | ❌ | ✅ |
| **Job Queue** | ❌ | ❌ | ✅ |
| **Memory Usage** | ~440MB | ~700MB | ~1.2GB/instance |
| **Setup Complexity** | Simple | Medium | Complex |
| **Startup Time** | <2s | <5s | <10s |

---

## Environment Variables

All configurations support environment variable overrides:

### Cache Configuration
```bash
CACHE_BACKEND=memory|redis
REDIS_URL=redis://localhost:6379/0
CACHE_MEMORY_TTL=3600
CACHE_MAX_ENTRIES=10000
```

### Worker Configuration
```bash
WORKERS_ENABLED=false|true
WORKERS_REDIS_URL=redis://localhost:6379/1
WORKER_COUNT=4
JOB_TIMEOUT=300
MAX_RETRIES=3
```

### Example
```bash
# Override configuration with environment variables
CACHE_BACKEND=redis \
REDIS_URL=redis://prod-redis:6379/0 \
cargo run --config config/deployment/enhanced.toml
```

---

## Migration Paths

### Minimal → Enhanced (Adding Redis)

**Why**: You need persistent cache or session management

**Steps**:
1. Install and start Redis:
   ```bash
   docker run -d -p 6379:6379 redis:7-alpine
   ```

2. Update configuration:
   ```toml
   [cache]
   backend = "redis"
   redis_url = "redis://localhost:6379/0"
   ```

3. Restart RipTide

**Impact**: +260MB RAM, cache persists across restarts, +1-2ms latency

---

### Enhanced → Distributed (Adding Workers)

**Why**: You need background job processing or horizontal scaling

**Steps**:
1. Update configuration:
   ```toml
   [workers]
   enabled = true
   redis_url = "redis://localhost:6379/1"
   worker_count = 8
   ```

2. Start worker service:
   ```bash
   docker-compose up --scale riptide-worker=2
   ```

3. Scale API instances:
   ```bash
   docker-compose up --scale riptide-api=3
   ```

**Impact**: Async job processing, horizontal scalability, +worker resource overhead

---

## Configuration Validation

All configurations are validated on startup. Common errors:

### Redis backend requires URL
```
ERROR: cache.redis_url is required when cache.backend = 'redis'
```
**Fix**: Set `redis_url` in config or via `REDIS_URL` environment variable

### Workers require Redis URL
```
ERROR: workers.redis_url is required when workers.enabled = true
```
**Fix**: Set `workers.redis_url` in config or via `WORKERS_REDIS_URL` environment variable

### Workers require Redis cache
```
ERROR: Workers require Redis cache backend
```
**Fix**: Set `cache.backend = "redis"` when enabling workers

---

## Health Check

Verify your configuration mode:

```bash
curl http://localhost:8080/health/capabilities
```

Response shows current configuration:
```json
{
  "cache_backend": "Redis",
  "async_jobs": true,
  "distributed": true,
  "persistent_cache": true,
  "session_persistence": true
}
```

---

## Troubleshooting

### API fails to start with Redis error
**Problem**: Redis not running or URL incorrect

**Solutions**:
1. Verify Redis is running: `redis-cli ping` (should return `PONG`)
2. Check Redis URL format: `redis://host:port/database`
3. Fall back to minimal mode: `CACHE_BACKEND=memory cargo run`

### Workers not processing jobs
**Problem**: Worker service not started or misconfigured

**Solutions**:
1. Verify worker service is running: `docker ps | grep worker`
2. Check Redis job queue: `redis-cli LLEN riptide:jobs:normal`
3. Review worker logs: `docker logs riptide-worker`

### Cache not persisting
**Problem**: Using memory backend or Redis persistence not configured

**Solutions**:
1. Verify backend: `curl localhost:8080/health/capabilities`
2. Check Redis AOF: `redis-cli CONFIG GET appendonly` (should be `yes`)
3. Switch to enhanced or distributed mode

---

## Performance Tuning

### Minimal Mode
```toml
[cache]
memory_ttl = 7200              # Increase cache TTL
max_memory_entries = 50000     # More entries (requires more RAM)

[extraction]
max_concurrent = 100           # Higher concurrency
```

### Enhanced Mode
```bash
# Redis tuning
redis-server --maxmemory 1gb --maxmemory-policy allkeys-lru
```

### Distributed Mode
```toml
[workers]
worker_count = 16              # More workers (requires more CPU)

[redis]
pool_size = 100                # Larger connection pool
```

---

## Security Best Practices

### 1. Never Hardcode Credentials
```toml
# ❌ DON'T
redis_url = "redis://user:password@host:6379"

# ✅ DO
redis_url = "${REDIS_URL}"
```

### 2. Use Environment Variables
```bash
export REDIS_URL="redis://prod-redis:6379"
export RIPTIDE_API_KEY="your-secret-key"
```

### 3. Use Secrets Management
```bash
# Kubernetes
kubectl create secret generic riptide-redis \
  --from-literal=url="redis://..."

# Docker Swarm
echo "redis://..." | docker secret create redis_url -

# Vault
export REDIS_URL=$(vault kv get -field=url secret/redis)
```

---

## Additional Resources

- **Architecture Design**: `/workspaces/riptidecrawler/docs/architecture/phase1-configuration-design.md`
- **Summary**: `/workspaces/riptidecrawler/docs/architecture/phase1-configuration-summary.md`
- **Implementation Roadmap**: `/workspaces/riptidecrawler/docs/investigations/redis-optional/03-implementation-roadmap.md`

---

## Support

For questions or issues:
1. Check the [FAQ](../../docs/00-getting-started/faq.md)
2. Review [Configuration Design](../../docs/architecture/phase1-configuration-design.md)
3. Open an issue on GitHub

---

**Version**: 1.0
**Last Updated**: 2025-11-12
**Phase**: Phase 1 - Make Redis Optional
