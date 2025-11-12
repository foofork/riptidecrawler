# RipTide Docker Quick Start

Choose your deployment mode and get started in under 5 minutes.

## 🚀 TL;DR

```bash
# Zero-dependency (recommended for testing)
docker-compose -f docker-compose.minimal.yml up -d

# With Redis (recommended for development)
docker-compose -f docker-compose.simple.yml up -d

# Full production (recommended for production)
docker-compose up -d
```

---

## 📦 Prerequisites

- Docker 20.10+ and Docker Compose 2.0+
- 2GB RAM minimum (4GB recommended)
- 2GB free disk space

### Verify Installation
```bash
docker --version
docker-compose --version
```

---

## Mode 1: Minimal (Zero Dependencies)

**Best for**: CI/CD, testing, local development

### Start
```bash
docker-compose -f docker-compose.minimal.yml up -d
```

### Verify
```bash
# Check status
docker-compose -f docker-compose.minimal.yml ps

# Health check
curl http://localhost:8080/health

# Test extraction
curl "http://localhost:8080/extract?url=https://example.com"
```

### Stop
```bash
docker-compose -f docker-compose.minimal.yml down
```

**Memory**: ~440MB | **Containers**: 1 | **Startup**: ~5s

---

## Mode 2: Simple (With Redis)

**Best for**: Development, small production

### Start
```bash
docker-compose -f docker-compose.simple.yml up -d
```

### Verify
```bash
# Check status
docker-compose -f docker-compose.simple.yml ps

# Health check
curl http://localhost:8080/health

# Test extraction (will be cached)
curl "http://localhost:8080/extract?url=https://example.com"

# Verify cache hit (instant response)
curl "http://localhost:8080/extract?url=https://example.com"

# Check Redis
docker-compose -f docker-compose.simple.yml exec redis redis-cli DBSIZE
```

### Stop
```bash
docker-compose -f docker-compose.simple.yml down
```

**Memory**: ~600MB | **Containers**: 2 | **Startup**: ~15s

---

## Mode 3: Distributed (Full Production)

**Best for**: Production, high-volume, JavaScript rendering

### Setup
```bash
# Copy environment template
cp .env.example .env

# Edit configuration
nano .env
```

### Start
```bash
docker-compose up -d
```

### Verify
```bash
# Check status
docker-compose ps

# Health check
curl http://localhost:8080/health

# Test static extraction
curl "http://localhost:8080/extract?url=https://example.com"

# Test JavaScript rendering
curl -X POST http://localhost:8080/extract \
  -H "Content-Type: application/json" \
  -d '{"url": "https://react-app.com", "render": true}'

# Check browser pool
docker-compose logs riptide-headless | grep "Browser"
```

### Stop
```bash
docker-compose down
```

**Memory**: ~1.2GB | **Containers**: 3 | **Startup**: ~40s

---

## 🔧 Common Operations

### View Logs
```bash
# Minimal
docker-compose -f docker-compose.minimal.yml logs -f

# Simple
docker-compose -f docker-compose.simple.yml logs -f

# Distributed
docker-compose logs -f
```

### Restart Services
```bash
# Minimal
docker-compose -f docker-compose.minimal.yml restart

# Simple
docker-compose -f docker-compose.simple.yml restart

# Distributed
docker-compose restart
```

### Scale API Instances (Simple/Distributed)
```bash
# Simple mode
docker-compose -f docker-compose.simple.yml up -d --scale riptide-api=3

# Distributed mode
docker-compose up -d --scale riptide-api=3
```

### Update Images
```bash
# Pull latest images
docker-compose pull

# Rebuild from source
docker-compose build --no-cache
```

---

## 🐛 Troubleshooting

### Health Check Fails
```bash
# Check logs
docker-compose logs riptide-api

# Verify port availability
netstat -tuln | grep 8080

# Restart service
docker-compose restart riptide-api
```

### Redis Connection Issues (Simple/Distributed)
```bash
# Check Redis status
docker-compose exec redis redis-cli ping

# Check network connectivity
docker-compose exec riptide-api ping redis
```

### Out of Memory
```bash
# Check resource usage
docker stats

# Increase memory limits (docker-compose.override.yml)
services:
  riptide-api:
    deploy:
      resources:
        limits:
          memory: 4G
```

### Port Already in Use
```bash
# Find process using port
sudo lsof -i :8080

# Or change port in .env
export RIPTIDE_API_PORT=8081
```

---

## 🎯 Next Steps

- [API Documentation](../api/README.md)
- [Configuration Guide](../config/README.md)
- [Docker Deployment Modes](./docker-modes.md)
- [Production Best Practices](./production.md)

---

**Need Help?**
- GitHub Issues: https://github.com/ruvnet/riptide/issues
- Documentation: https://docs.riptide.dev
