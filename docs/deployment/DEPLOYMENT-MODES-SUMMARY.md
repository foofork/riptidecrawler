# RipTide Docker Deployment Modes - Quick Reference

## 🎯 Choose Your Mode

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        DEPLOYMENT MODE SELECTOR                         │
└─────────────────────────────────────────────────────────────────────────┘

❓ What are you building?

┌─ Local Development / Testing ────────────────────────────────────────┐
│  → docker-compose.minimal.yml                                         │
│  ✅ Zero dependencies                                                 │
│  ✅ Fast startup (~5s)                                                │
│  ✅ 440MB RAM                                                         │
│  ❌ Cache clears on restart                                           │
└───────────────────────────────────────────────────────────────────────┘

┌─ Development with Persistence ───────────────────────────────────────┐
│  → docker-compose.simple.yml                                          │
│  ✅ Redis caching                                                     │
│  ✅ Cache survives restarts                                           │
│  ✅ 600MB RAM                                                         │
│  ❌ No JavaScript rendering                                           │
└───────────────────────────────────────────────────────────────────────┘

┌─ Production / High Volume ───────────────────────────────────────────┐
│  → docker-compose.yml                                                 │
│  ✅ Full Chrome rendering                                             │
│  ✅ Background workers                                                │
│  ✅ Scalable architecture                                             │
│  ⚠️  1.2GB RAM required                                               │
└───────────────────────────────────────────────────────────────────────┘
```

## 📊 Feature Matrix

| Feature | Minimal | Simple | Distributed |
|---------|:-------:|:------:|:-----------:|
| **Command** | `docker-compose -f docker-compose.minimal.yml up` | `docker-compose -f docker-compose.simple.yml up` | `docker-compose up` |
| **Memory** | 440MB | 600MB | 1.2GB |
| **Startup Time** | 5s | 15s | 40s |
| **Containers** | 1 | 2 | 3+ |
| | | | |
| **Redis Cache** | ❌ In-memory | ✅ Persistent | ✅ Persistent |
| **Cache Survives Restart** | ❌ | ✅ | ✅ |
| **Background Workers** | ❌ | ❌ | ✅ |
| **Chrome Browser** | ❌ | ❌ | ✅ Chrome Pool |
| **JavaScript Rendering** | ❌ WASM only | ❌ WASM only | ✅ Full Chrome |
| | | | |
| **Max Throughput** | 30 req/min | 50 req/min | 200 req/min |
| **Scalability** | Single instance | Manual scale | Horizontal |
| **High Availability** | ❌ | ⚠️ Manual | ✅ Built-in |

## 🚀 Quick Start Commands

### Minimal Mode
```bash
# Start
docker-compose -f docker-compose.minimal.yml up -d

# Test
curl http://localhost:8080/health
curl "http://localhost:8080/extract?url=https://example.com"

# Stop
docker-compose -f docker-compose.minimal.yml down
```

### Simple Mode
```bash
# Start
docker-compose -f docker-compose.simple.yml up -d

# Test cache persistence
curl "http://localhost:8080/extract?url=https://example.com"
docker-compose -f docker-compose.simple.yml restart
curl "http://localhost:8080/extract?url=https://example.com"  # ⚡ Instant

# Stop
docker-compose -f docker-compose.simple.yml down
```

### Distributed Mode
```bash
# Setup
cp .env.example .env
# Edit .env with your API keys

# Start
docker-compose up -d

# Test JavaScript rendering
curl -X POST http://localhost:8080/extract \
  -H "Content-Type: application/json" \
  -d '{"url": "https://react-app.com", "render": true}'

# Scale
docker-compose up -d --scale riptide-api=3

# Stop
docker-compose down
```

## 🎓 When to Use Each Mode

### Use Minimal When:
- ✅ Running CI/CD tests
- ✅ Local development
- ✅ Learning RipTide
- ✅ Simple extraction tasks
- ✅ No external dependencies allowed
- ✅ Minimal resource footprint needed

### Use Simple When:
- ✅ Development needs persistent cache
- ✅ Small production workload (< 1000 req/day)
- ✅ Cache hit rate matters
- ✅ Static content extraction
- ✅ Want Redis but not Chrome

### Use Distributed When:
- ✅ Production deployments
- ✅ High-volume workloads (> 1000 req/day)
- ✅ JavaScript-heavy websites
- ✅ Need background job processing
- ✅ Require horizontal scaling
- ✅ High availability required

## 📈 Upgrade Path

```
Minimal (440MB)
    │
    ├─ Add Redis ──────────► Simple (600MB)
    │                           │
    │                           ├─ Add Chrome & Workers
    │                           │
    └─────────────────────────► Distributed (1.2GB)
```

### From Minimal to Simple
```bash
docker-compose -f docker-compose.minimal.yml down
docker-compose -f docker-compose.simple.yml up -d
```

### From Simple to Distributed
```bash
docker-compose -f docker-compose.simple.yml down
docker-compose up -d
```

### From Minimal to Distributed
```bash
docker-compose -f docker-compose.minimal.yml down
cp .env.example .env  # Configure first
docker-compose up -d
```

## 🔧 Configuration Differences

### Minimal Mode
```bash
# Environment variables
CACHE_BACKEND=memory          # In-memory cache
WORKERS_ENABLED=false         # No background workers
SPIDER_ENABLE=true            # Spider works without Redis
```

### Simple Mode
```bash
# Environment variables
CACHE_BACKEND=redis           # Persistent cache
REDIS_URL=redis://redis:6379  # Redis connection
WORKERS_ENABLED=false         # Still no workers
```

### Distributed Mode
```bash
# Environment variables
CACHE_BACKEND=redis           # Persistent cache
REDIS_URL=redis://redis:6379  # Redis connection
WORKERS_ENABLED=true          # Background job processing
HEADLESS_URL=http://riptide-headless:9123  # Chrome pool
```

## 💾 Data Persistence

### Minimal Mode
```yaml
volumes:
  riptide-minimal-data:     # Logs only
  riptide-minimal-cache:    # Temporary cache
  riptide-minimal-logs:     # Application logs
# ⚠️ Cache cleared on container restart
```

### Simple Mode
```yaml
volumes:
  riptide-simple-data:      # Extraction results
  riptide-simple-cache:     # Application cache
  riptide-simple-logs:      # Application logs
  redis-simple-data:        # Redis persistence
# ✅ Cache survives container restart
```

### Distributed Mode
```yaml
volumes:
  riptide-data:             # Extraction results
  riptide-cache:            # Application cache
  riptide-logs:             # Application logs
  redis-data:               # Redis persistence
# ✅ Full persistence with backup support
```

## 🐛 Troubleshooting by Mode

### Minimal Mode Issues
```bash
# Out of memory
export CACHE_MAX_ENTRIES=5000  # Reduce cache size

# Slow performance
# → Upgrade to Simple mode for persistent cache
```

### Simple Mode Issues
```bash
# Redis connection failed
docker-compose -f docker-compose.simple.yml exec redis redis-cli ping

# Out of memory
# → Check Redis memory: docker stats riptide-simple-redis
```

### Distributed Mode Issues
```bash
# Browser pool exhausted
docker-compose logs riptide-headless

# Scale browser instances
# → Increase browser pool size in config
```

## 📊 Performance Benchmarks

### Minimal Mode
```
Cold request:    500ms - 2s
Cache hit:       1-5ms (in-memory)
Throughput:      ~30 requests/min
Memory stable:   440MB
```

### Simple Mode
```
Cold request:    500ms - 2s
Cache hit:       10-20ms (Redis)
Throughput:      ~50 requests/min
Memory stable:   600MB
```

### Distributed Mode
```
Cold request:    500ms - 2s (static) | 2s - 5s (JS)
Cache hit:       10-20ms (Redis)
Throughput:      ~200 requests/min
Memory stable:   1.2GB
Scalable:        Linear with API instances
```

## 🔗 Additional Resources

- [Full Documentation](./docker-modes.md)
- [Quick Start Guide](./quick-start-docker.md)
- [Configuration Reference](../config/README.md)
- [API Documentation](../api/README.md)

---

**Quick Decision Tree**:
- Need zero dependencies? → **Minimal**
- Need persistent cache? → **Simple**
- Need JavaScript rendering? → **Distributed**
- Need > 100 req/min? → **Distributed**
- Need HA/scaling? → **Distributed**

---

**Last Updated**: 2025-11-12 | **Version**: 2.0.0
