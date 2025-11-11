# Docker Quick Start Guide

Complete guide for running RipTide Crawler using Docker and Docker Compose.

## Prerequisites

- **Docker**: Version 20.10 or later
- **Docker Compose**: Version 1.29 or later
- **Disk Space**: At least 5GB free for images
- **Memory**: 2GB minimum (4GB recommended for browser mode)

**Install Docker:**
- Ubuntu/Debian: `sudo apt install docker.io docker-compose`
- macOS: Install [Docker Desktop](https://www.docker.com/products/docker-desktop)
- Windows: Install [Docker Desktop](https://www.docker.com/products/docker-desktop)

## Quick Start (30 seconds)

```bash
# Build and start all services
make quick-start

# Or manually:
make docker-build-all
make docker-up

# Check status
docker-compose ps

# View logs
docker-compose logs -f riptide-api
```

**Your API is now running at:** `http://localhost:8080`

## Available Services

### Full Stack (Recommended)
```bash
# Start API + Chrome + Redis + Swagger UI
make docker-up

# Services:
# - API: http://localhost:8080
# - Swagger UI: http://localhost:8081
# - Redis: localhost:6379 (internal)
# - Chrome: Connected via CDP (internal)

# Memory usage: ~1.2GB
# Features: ✅ JavaScript ✅ Browser rendering ✅ Caching
```

### Lite Mode (Low Resource)
```bash
# Start API + Redis only (no browser)
make docker-up-lite

# Services:
# - API: http://localhost:8080
# - Redis: localhost:6379 (internal)

# Memory usage: ~440MB
# Features: ✅ HTML extraction ❌ No JavaScript rendering
```

## Build Commands

### Build Individual Services
```bash
# API server
make docker-build-api

# Headless browser service
make docker-build-headless

# Playground/testing
make docker-build-playground

# All services
make docker-build-all
```

### Build Options
```bash
# Fast rebuild (uses cache)
docker-compose build

# Clean rebuild (no cache)
docker-compose build --no-cache

# Pull latest base images
docker-compose build --pull
```

## Service Management

### Start/Stop Services
```bash
# Start in background
docker-compose up -d

# Start with logs visible
docker-compose up

# Stop all services
docker-compose down

# Stop and remove volumes (CAUTION: deletes data)
docker-compose down -v
```

### View Logs
```bash
# All services
docker-compose logs -f

# Specific service
docker-compose logs -f riptide-api
docker-compose logs -f redis
docker-compose logs -f riptide-headless

# Last 100 lines
docker-compose logs --tail=100 riptide-api
```

### Health Checks
```bash
# Check service status
docker-compose ps

# Test API health endpoint
curl http://localhost:8080/health

# Expected response:
# {"status":"healthy","version":"0.1.0"}

# Check Redis connectivity
docker-compose exec redis redis-cli ping
# Expected: PONG
```

## Configuration

### Environment Variables

Create a `.env` file in the project root:

```env
# API Configuration
RUST_LOG=info
RUST_BACKTRACE=1
RIPTIDE_API_KEY=your-api-key-here

# Redis Configuration
REDIS_URL=redis://redis:6379
REDIS_MAX_CONNECTIONS=10

# Browser Configuration
CHROME_WS_ENDPOINT=ws://riptide-headless:9222
BROWSER_POOL_SIZE=5

# AI/LLM Configuration (optional)
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
SERPER_API_KEY=...

# Feature Flags
ENABLE_WASM=false
ENABLE_PDF_EXTRACTION=true
```

### Volume Mounts

Default volumes for data persistence:

```yaml
volumes:
  - ./config:/opt/riptide/config     # Configuration files
  - riptide-data:/opt/riptide/data   # Crawl data
  - riptide-logs:/opt/riptide/logs   # Application logs
  - riptide-cache:/opt/riptide/cache # Cache data
```

**Access data:**
```bash
# View logs
docker-compose exec riptide-api cat /opt/riptide/logs/app.log

# Copy data out
docker cp riptide-api:/opt/riptide/data ./local-data

# Edit config
vim config/application/riptide.yml
docker-compose restart riptide-api
```

## Testing the Deployment

### 1. Basic Health Check
```bash
# API should respond
curl -s http://localhost:8080/health | jq .

# Expected output:
# {
#   "status": "healthy",
#   "version": "0.1.0",
#   "uptime_seconds": 123
# }
```

### 2. Simple Crawl Request
```bash
# Crawl example.com
curl -X POST http://localhost:8080/api/v1/crawl \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://example.com",
    "mode": "standard"
  }' | jq .
```

### 3. Browser Rendering Test
```bash
# Crawl JavaScript-heavy site
curl -X POST http://localhost:8080/api/v1/crawl \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://quotes.toscrape.com",
    "mode": "enhanced",
    "javascript_enabled": true
  }' | jq .
```

### 4. Check Swagger UI
```bash
# Open in browser
open http://localhost:8081

# Or with curl
curl -s http://localhost:8081 | grep title
```

## Performance Tuning

### Increase Browser Pool Size
```yaml
# docker-compose.yml
services:
  riptide-api:
    environment:
      - BROWSER_POOL_SIZE=10  # Default: 5
```

### Increase Redis Memory
```yaml
# docker-compose.yml
services:
  redis:
    command: redis-server --maxmemory 1gb --maxmemory-policy allkeys-lru
```

### Limit Container Resources
```yaml
# docker-compose.yml
services:
  riptide-api:
    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 2G
        reservations:
          memory: 512M
```

## Troubleshooting

### Problem: Build Fails with "No Space Left"
```bash
# Check disk space
df -h

# Clean Docker build cache
docker builder prune -a

# Clean unused images and containers
docker system prune -a

# If needed, clean Rust build cache
cargo clean
```

### Problem: API Container Exits Immediately
```bash
# Check logs for errors
docker-compose logs riptide-api

# Common issues:
# 1. Missing config file
# 2. Invalid environment variables
# 3. Port already in use

# Verify config exists
ls -la config/application/riptide.yml

# Check port availability
lsof -i :8080
```

### Problem: Chrome Connection Fails
```bash
# Verify headless service is running
docker-compose ps riptide-headless

# Check Chrome process
docker-compose exec riptide-headless ps aux | grep chrome

# Test WebSocket connection
curl -i -N \
  -H "Connection: Upgrade" \
  -H "Upgrade: websocket" \
  http://localhost:9222

# Restart headless service
docker-compose restart riptide-headless
```

### Problem: Redis Connection Errors
```bash
# Verify Redis is running
docker-compose ps redis

# Test Redis connectivity
docker-compose exec redis redis-cli ping

# Check Redis logs
docker-compose logs redis

# Restart Redis
docker-compose restart redis
```

### Problem: High Memory Usage
```bash
# Check container memory
docker stats

# Reduce browser pool size
# Edit docker-compose.yml:
BROWSER_POOL_SIZE=2

# Use lite mode instead
docker-compose down
docker-compose -f docker-compose.lite.yml up -d
```

## Development Workflow

### Hot Reload Development
```bash
# Mount source code for development
docker-compose -f docker-compose.dev.yml up

# Or manually:
docker run -v $(pwd):/app -p 8080:8080 riptide-api:latest \
  cargo watch -x run
```

### Run Tests in Container
```bash
# All tests
docker-compose exec riptide-api cargo test

# Specific package
docker-compose exec riptide-api cargo test -p riptide-api

# Integration tests
docker-compose exec riptide-api cargo test -- --ignored
```

### Access Container Shell
```bash
# API container
docker-compose exec riptide-api bash

# Redis CLI
docker-compose exec redis redis-cli

# Chrome browser (debugging)
docker-compose exec riptide-headless bash
```

## Production Deployment

### Security Checklist
- [ ] Change default API keys in `.env`
- [ ] Use secrets management (Docker Swarm secrets, Kubernetes secrets)
- [ ] Enable TLS/SSL for API endpoints
- [ ] Configure firewall rules (allow only 8080)
- [ ] Set resource limits in docker-compose.yml
- [ ] Enable log rotation
- [ ] Use read-only containers where possible

### Production Configuration
```yaml
# docker-compose.prod.yml
version: "3.8"

services:
  riptide-api:
    image: riptide-api:latest
    restart: unless-stopped
    read_only: true
    security_opt:
      - no-new-privileges:true
    environment:
      - RUST_LOG=warn
      - RUST_BACKTRACE=0
    deploy:
      resources:
        limits:
          cpus: '4.0'
          memory: 4G
      restart_policy:
        condition: on-failure
        delay: 5s
        max_attempts: 3
```

### Deploy with Docker Swarm
```bash
# Initialize swarm
docker swarm init

# Deploy stack
docker stack deploy -c docker-compose.prod.yml riptide

# Check services
docker service ls

# Scale API service
docker service scale riptide_riptide-api=3

# View logs
docker service logs riptide_riptide-api
```

## Monitoring

### Prometheus Metrics (Coming Soon)
```bash
# Metrics endpoint
curl http://localhost:8080/metrics

# Grafana dashboard
# Import dashboard ID: 12345
```

### Health Monitoring Script
```bash
#!/bin/bash
# monitor-health.sh

while true; do
  STATUS=$(curl -s http://localhost:8080/health | jq -r .status)
  if [ "$STATUS" != "healthy" ]; then
    echo "⚠️  API unhealthy! Status: $STATUS"
    docker-compose logs --tail=50 riptide-api
  else
    echo "✅ API healthy"
  fi
  sleep 30
done
```

## Cleanup

### Stop and Remove Everything
```bash
# Stop containers
docker-compose down

# Remove all volumes (CAUTION: deletes data)
docker-compose down -v

# Remove images
docker-compose down --rmi all

# Complete cleanup
docker system prune -a --volumes
```

## Next Steps

1. ✅ Services running - Start crawling websites
2. 📚 Read [API Documentation](./API_DOCUMENTATION.md)
3. 🧪 Run [Integration Tests](./INTEGRATION_TESTING.md)
4. 🚀 Check [Performance Tuning](./PERFORMANCE_GUIDE.md)
5. 🔒 Review [Security Best Practices](./SECURITY.md)

## Support

- **Issues**: https://github.com/yourusername/riptidecrawler/issues
- **Documentation**: `/docs` directory
- **Docker Logs**: `/opt/riptide/logs` (in container)

---

**Happy Crawling!** 🌊
