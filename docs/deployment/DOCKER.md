# RipTide Docker Deployment

Simple Docker deployment with optional Redis.

## Quick Start

### Option 1: Without Redis (In-Memory)
```bash
# Start services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

### Option 2: With Redis (Persistent Cache)
```bash
# Start services with Redis
docker-compose --profile redis up -d

# Or set environment variable
export CACHE_BACKEND=redis
docker-compose --profile redis up -d

# Stop services
docker-compose down
```

## What You Get

### Without Redis (`docker-compose up -d`)
- ✅ RipTide API on port 8080
- ✅ Headless Chrome browser service
- ✅ In-memory caching (no persistence)
- ✅ Perfect for development and testing
- 📊 Memory: ~1GB

### With Redis (`docker-compose --profile redis up -d`)
- ✅ Everything above, plus:
- ✅ Redis cache on port 6379
- ✅ Persistent caching across restarts
- ✅ Session storage
- ✅ Better for production
- 📊 Memory: ~1.5GB

## Configuration

### Environment Variables

Create a `.env` file:

```bash
# API Configuration
RIPTIDE_API_PORT=8080
RUST_LOG=info
REQUIRE_AUTH=false

# Cache Backend (memory or redis)
CACHE_BACKEND=memory

# Redis (only if using --profile redis)
REDIS_URL=redis://redis:6379/0
REDIS_PORT=6379

# Spider Configuration
SPIDER_ENABLE=true

# Search API Keys
SERPER_API_KEY=your-serper-key

# Optional LLM Keys
OPENAI_API_KEY=your-openai-key
ANTHROPIC_API_KEY=your-anthropic-key
```

### Using Redis

To use Redis, you need both:
1. Enable the Redis profile: `--profile redis`
2. Set the cache backend: `CACHE_BACKEND=redis`

```bash
# Create .env file
cat > .env << EOF
CACHE_BACKEND=redis
SERPER_API_KEY=your-key-here
EOF

# Start with Redis
docker-compose --profile redis up -d
```

## Native Build Only

This Docker setup uses **native compilation only** (no WASM). Benefits:
- ✅ Faster performance
- ✅ Smaller image size
- ✅ Simpler deployment
- ✅ Full feature support

## Service Details

### RipTide API
- **Port**: 8080 (configurable)
- **Health**: `http://localhost:8080/health`
- **Docs**: `http://localhost:8080/docs`
- **Image**: Built from `infra/docker/Dockerfile.api`

### Redis (Optional)
- **Port**: 6379 (configurable)
- **Data**: Persistent volume `redis-data`
- **Memory**: 512MB limit with LRU eviction
- **Image**: `redis:7-alpine`

### Headless Browser
- **Port**: 9123 (internal)
- **Engine**: Chromium
- **Pool**: 5 browsers
- **Memory**: 2GB shared memory

## Docker Commands

```bash
# Build from scratch
docker-compose build --no-cache

# View logs
docker-compose logs -f riptide-api
docker-compose logs -f redis

# Check status
docker-compose ps

# Restart a service
docker-compose restart riptide-api

# Stop all services
docker-compose down

# Stop and remove volumes
docker-compose down -v

# Scale API instances (if needed)
docker-compose up -d --scale riptide-api=3
```

## Volumes

Data is persisted in Docker volumes:
- `riptide-data`: Scraped data and outputs
- `riptide-cache`: Local cache files
- `riptide-logs`: Application logs
- `redis-data`: Redis persistence (if using Redis profile)

```bash
# List volumes
docker volume ls | grep riptide

# Inspect a volume
docker volume inspect riptide-data

# Remove all volumes (⚠️ destroys data)
docker-compose down -v
```

## Troubleshooting

### API won't start
```bash
# Check logs
docker-compose logs riptide-api

# Check health
curl http://localhost:8080/health
```

### Redis connection issues
```bash
# Verify Redis is running (only if using --profile redis)
docker-compose --profile redis ps

# Test Redis connection
docker exec -it riptide-redis redis-cli ping
```

### Out of memory
```bash
# Check resource usage
docker stats

# Increase Docker memory limit in Docker Desktop settings
# Recommended: 4GB minimum, 8GB preferred
```

### Port conflicts
```bash
# Change ports in .env
RIPTIDE_API_PORT=8090
REDIS_PORT=6380
```

## Production Deployment

For production, consider:

1. **Use Redis**: Better performance and persistence
   ```bash
   docker-compose --profile redis up -d
   ```

2. **Set resource limits**: Already configured in docker-compose.yml

3. **Enable authentication**:
   ```bash
   REQUIRE_AUTH=true
   RIPTIDE_API_KEY=your-secure-api-key
   ```

4. **Use a reverse proxy**: Nginx or Traefik for SSL/TLS

5. **Monitor logs**:
   ```bash
   docker-compose logs -f --tail=100
   ```

## Architecture

```
┌─────────────────┐
│   riptide-api   │ :8080
│  (Native Rust)  │
└────────┬────────┘
         │
    ┌────┴────┐
    │         │
    ▼         ▼
┌────────┐  ┌──────────────┐
│ redis  │  │   headless   │ :9123
│(opt.)  │  │  (Chromium)  │
└────────┘  └──────────────┘
```

## Next Steps

- [Configuration Guide](../README.md#configuration)
- [API Documentation](../api/README.md)
- [Development Guide](../DEVELOPMENT.md)
