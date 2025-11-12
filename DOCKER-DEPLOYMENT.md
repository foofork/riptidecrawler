# RipTide Docker Deployment Guide

**Three deployment modes for every use case:**
- 🟢 **Minimal** - Zero dependencies (440MB)
- 🟡 **Simple** - API + Redis (600MB)
- 🔴 **Distributed** - Full production (1.2GB)

---

## 🚀 Quick Start

### 1. Choose Your Mode

```bash
# Minimal - Zero dependencies (recommended for testing)
docker-compose -f docker-compose.minimal.yml up -d

# Simple - With Redis (recommended for development)
docker-compose -f docker-compose.simple.yml up -d

# Distributed - Full production (recommended for production)
docker-compose up -d
```

### 2. Verify Deployment

```bash
# Check health
curl http://localhost:8080/health

# Test extraction
curl "http://localhost:8080/extract?url=https://example.com"

# View logs
docker-compose logs -f
```

### 3. Stop Services

```bash
# Minimal
docker-compose -f docker-compose.minimal.yml down

# Simple
docker-compose -f docker-compose.simple.yml down

# Distributed
docker-compose down
```

---

## 📊 Comparison Matrix

| Feature | Minimal | Simple | Distributed |
|---------|:-------:|:------:|:-----------:|
| **File** | `docker-compose.minimal.yml` | `docker-compose.simple.yml` | `docker-compose.yml` |
| **Redis** | ❌ In-memory | ✅ Persistent | ✅ Persistent |
| **Workers** | ❌ | ❌ | ✅ |
| **Chrome** | ❌ | ❌ | ✅ Pool |
| **Memory** | 440MB | 600MB | 1.2GB |
| **Containers** | 1 | 2 | 3+ |
| **Startup** | 5s | 15s | 40s |
| **Throughput** | 30/min | 50/min | 200/min |
| **Cache Persists** | ❌ | ✅ | ✅ |
| **JavaScript** | ❌ WASM | ❌ WASM | ✅ Chrome |
| **Scaling** | Single | Manual | Horizontal |

---

## 📖 Full Documentation

### Getting Started
- **[Quick Start Guide](docs/deployment/quick-start-docker.md)** - Get running in 5 minutes
- **[Deployment Modes Summary](docs/deployment/DEPLOYMENT-MODES-SUMMARY.md)** - Quick reference

### Detailed Guides
- **[Docker Modes Documentation](docs/deployment/docker-modes.md)** - Complete guide (600+ lines)
  - Minimal mode setup and use cases
  - Simple mode with persistent cache
  - Distributed mode for production
  - Switching between modes
  - Performance comparison
  - Security considerations

- **[Testing Guide](docs/deployment/TESTING.md)** - Test all modes
  - Automated test suite
  - Manual testing procedures
  - CI/CD integration
  - Performance benchmarks

- **[Deployment Index](docs/deployment/README.md)** - All deployment documentation

---

## 🎯 Decision Tree

```
What's your use case?
│
├─ Local Development / CI/CD
│  └─ Use: docker-compose.minimal.yml
│     • Zero dependencies
│     • Fast startup (5s)
│     • 440MB RAM
│
├─ Development with Persistent Cache
│  └─ Use: docker-compose.simple.yml
│     • Redis caching
│     • Cache survives restarts
│     • 600MB RAM
│
└─ Production / High Volume
   └─ Use: docker-compose.yml
      • Full Chrome rendering
      • Background workers
      • Horizontal scaling
      • 1.2GB RAM
```

---

## 🧪 Testing

### Automated Tests

```bash
# Test all modes
./scripts/test-docker-modes.sh all

# Test specific mode
./scripts/test-docker-modes.sh minimal
./scripts/test-docker-modes.sh simple
./scripts/test-docker-modes.sh distributed
```

### Manual Testing

```bash
# Start service
docker-compose -f docker-compose.minimal.yml up -d

# Health check
curl http://localhost:8080/health

# Extract content
curl "http://localhost:8080/extract?url=https://example.com"

# Check memory
docker stats --no-stream

# View logs
docker-compose -f docker-compose.minimal.yml logs -f

# Stop
docker-compose -f docker-compose.minimal.yml down
```

---

## 🔧 Configuration

### Minimal Mode
- **Config**: `config/deployment/minimal.toml`
- **Cache**: In-memory (3600s TTL)
- **Workers**: Disabled
- **Browser**: WASM only

### Simple Mode
- **Config**: Uses `docker-compose.simple.yml` environment
- **Cache**: Redis (persistent)
- **Workers**: Disabled
- **Browser**: WASM only

### Distributed Mode
- **Config**: `.env` file + `docker-compose.yml`
- **Cache**: Redis (persistent)
- **Workers**: Enabled
- **Browser**: Chrome pool (5 browsers)

---

## 📈 Upgrade Path

```
Minimal → Simple → Distributed

Step 1: Add Redis
  docker-compose -f docker-compose.minimal.yml down
  docker-compose -f docker-compose.simple.yml up -d

Step 2: Add Chrome + Workers
  docker-compose -f docker-compose.simple.yml down
  docker-compose up -d
```

---

## 🔒 Security

### Before Production Deployment

```bash
# Set API key
export RIPTIDE_API_KEY=$(openssl rand -hex 32)

# Enable authentication
export REQUIRE_AUTH=true

# Configure CORS
export CORS_ORIGINS=https://yourdomain.com

# Use HTTPS (reverse proxy)
# See: docs/deployment/ssl-setup.md
```

---

## 🐛 Troubleshooting

### Health Check Fails
```bash
# Check logs
docker-compose logs riptide-api

# Verify port
netstat -tuln | grep 8080

# Restart
docker-compose restart riptide-api
```

### Redis Connection Issues
```bash
# Check Redis
docker-compose exec redis redis-cli ping

# Check network
docker-compose exec riptide-api ping redis
```

### Out of Memory
```bash
# Check usage
docker stats

# Increase limits in docker-compose.yml
services:
  riptide-api:
    deploy:
      resources:
        limits:
          memory: 4G
```

---

## 📚 Additional Resources

- [API Documentation](docs/api/README.md)
- [Configuration Guide](docs/config/README.md)
- [Production Best Practices](docs/deployment/production.md)
- [Load Balancing](docs/deployment/load-balancing.md)
- [Monitoring Setup](docs/deployment/monitoring.md)

---

## 🎉 Quick Examples

### Example 1: CI/CD Testing
```bash
# In your CI pipeline
docker-compose -f docker-compose.minimal.yml up -d
npm run test:integration
docker-compose -f docker-compose.minimal.yml down
```

### Example 2: Development with Cache
```bash
# Start with persistent cache
docker-compose -f docker-compose.simple.yml up -d

# Develop and test
curl "http://localhost:8080/extract?url=https://example.com"

# Restart - cache persists
docker-compose -f docker-compose.simple.yml restart
curl "http://localhost:8080/extract?url=https://example.com"  # ⚡ Instant
```

### Example 3: Production Deployment
```bash
# Configure
cp .env.example .env
nano .env  # Add API keys

# Start
docker-compose up -d

# Scale
docker-compose up -d --scale riptide-api=3

# Monitor
docker-compose logs -f
```

---

## 📊 Performance Metrics

### Minimal Mode
- **Startup**: ~5 seconds
- **Memory**: ~440MB stable
- **Cold Request**: 500ms - 2s
- **Cache Hit**: 1-5ms
- **Throughput**: ~30 requests/minute

### Simple Mode
- **Startup**: ~15 seconds
- **Memory**: ~600MB stable
- **Cold Request**: 500ms - 2s
- **Cache Hit**: 10-20ms (Redis)
- **Throughput**: ~50 requests/minute

### Distributed Mode
- **Startup**: ~40 seconds
- **Memory**: ~1.2GB stable
- **Cold Request**: 500ms - 2s (static), 2s - 5s (JavaScript)
- **Cache Hit**: 10-20ms (Redis)
- **Throughput**: ~200 requests/minute (scalable)

---

## 🆘 Support

- **GitHub Issues**: https://github.com/ruvnet/riptide/issues
- **Documentation**: https://docs.riptide.dev
- **Community**: https://discord.gg/riptide

---

**Created**: 2025-11-12
**Version**: 2.0.0
**Total Documentation**: 2,700+ lines
**Files Created**: 10
