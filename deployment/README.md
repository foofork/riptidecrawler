# RipTide Deployment Guide

This directory contains production-ready deployment configurations for RipTide.

## Quick Start

```bash
# Production deployment (from project root)
docker-compose up -d

# Development environment
cd examples/docker-compose
docker-compose -f docker-compose.dev.yml up -d

# Monitoring stack
cd deployment/monitoring
docker-compose -f docker-compose.monitoring.yml up -d

# Stop services
docker-compose down
```

## Deployment Configurations

### 1. Production Deployment (`/docker-compose.yml`)

**Location:** Project root
**Use Case:** Production environment with full monitoring and persistence

**Features:**
- Native-only extraction (fast, no WASM overhead)
- Redis persistence enabled
- Health checks configured
- Resource limits enforced
- Production-optimized logging

**Services:**
- `riptide-api` (port 8080) - Main API service
- `redis` (port 6379) - Cache and session storage

**Environment Variables:**
```bash
REDIS_URL=redis://redis:6379/0
SERPER_API_KEY=your_api_key_here
RUST_LOG=info,riptide_api=debug
```

**Usage:**
```bash
# From project root
docker-compose up -d
```

### 2. Development Deployment (`examples/docker-compose/docker-compose.dev.yml`)

**Location:** `examples/docker-compose/`
**Use Case:** Local development and testing

**Features:**
- Hot reload support (mount source code)
- Debug logging enabled
- Development-friendly resource limits
- Easy access to logs

**Additional Features:**
- Source code mounted for live changes
- Debug symbols included
- Permissive CORS settings

**Usage:**
```bash
cd examples/docker-compose
docker-compose -f docker-compose.dev.yml up -d
```

### 3. Monitoring Stack (`deployment/monitoring/docker-compose.monitoring.yml`)

**Location:** `deployment/monitoring/`
**Use Case:** Production monitoring with Prometheus, Grafana, and alerting

**Services:**
- Prometheus - Metrics collection
- Grafana - Dashboards and visualization
- Alertmanager - Alert handling
- Node Exporter - Host metrics

**Usage:**
```bash
cd deployment/monitoring
docker-compose -f docker-compose.monitoring.yml up -d
```

## Architecture

```
┌─────────────────────────────────────────────────┐
│                  RipTide API                    │
│         (Native Parser, No WASM)                │
│              Port: 8080                         │
└─────────────────┬───────────────────────────────┘
                  │
                  │ Redis Connection
                  ▼
┌─────────────────────────────────────────────────┐
│                   Redis                         │
│        (Cache & Session Storage)                │
│              Port: 6379                         │
└─────────────────────────────────────────────────┘
```

## Configuration Details

### Resource Limits

**Production:**
- API: 2 CPUs, 1GB RAM (max)
- Redis: 1 CPU, 1GB RAM (max)

**Development:**
- API: 1 CPU, 512MB RAM (max)
- Redis: 0.5 CPU, 256MB RAM (max)

### Health Checks

All deployments include health checks:
- **API**: `/healthz` endpoint (30s interval, 10s timeout)
- **Redis**: `redis-cli ping` (10s interval, 5s timeout)

### Data Persistence

**Volumes:**
- `riptide-data` - Application data
- `riptide-cache` - Extraction cache
- `riptide-logs` - Application logs
- `redis-data` - Redis persistence (AOF enabled)

## Environment Configuration

### Required Variables

```bash
# Redis connection
REDIS_URL=redis://redis:6379/0

# API Configuration
RIPTIDE_API_HOST=0.0.0.0
RIPTIDE_API_PORT=8080
```

### Optional Variables

```bash
# Search configuration
SERPER_API_KEY=your_serper_key

# Logging
RUST_LOG=info,riptide_api=debug
RUST_BACKTRACE=1

# Redis settings
REDIS_PORT=6379
```

## Monitoring

### Check Service Health

```bash
# API health check
curl http://localhost:8080/healthz | jq

# Redis health check
docker-compose exec redis redis-cli ping

# View service logs
docker-compose logs -f riptide-api
docker-compose logs -f redis
```

### Metrics

The `/healthz` endpoint provides:
- Redis connection status
- Headless service status (if enabled)
- API uptime
- Version information

## Troubleshooting

### API Won't Start

1. **Check Redis connection:**
   ```bash
   docker-compose exec redis redis-cli ping
   ```

2. **View API logs:**
   ```bash
   docker-compose logs riptide-api
   ```

3. **Verify environment variables:**
   ```bash
   docker-compose config
   ```

### Redis Connection Issues

1. **Check Redis is running:**
   ```bash
   docker ps | grep redis
   ```

2. **Test connection:**
   ```bash
   docker-compose exec redis redis-cli -h redis ping
   ```

3. **Check network:**
   ```bash
   docker network ls
   docker network inspect deployment_riptide-network
   ```

### Performance Issues

1. **Check resource usage:**
   ```bash
   docker stats
   ```

2. **Review Redis memory:**
   ```bash
   docker-compose exec redis redis-cli INFO memory
   ```

3. **Enable debug logging:**
   ```bash
   RUST_LOG=debug docker-compose up
   ```

## Advanced Deployments

### Custom Configuration

Create a `.env` file in the deployment directory:

```bash
# deployment/.env
REDIS_URL=redis://redis:6379/0
SERPER_API_KEY=your_key_here
RUST_LOG=info
```

### Multiple Environments

```bash
# Staging
docker-compose -f docker-compose.prod.yml --env-file .env.staging up -d

# Production
docker-compose -f docker-compose.prod.yml --env-file .env.production up -d
```

### Scaling

```bash
# Scale API to 3 instances
docker-compose -f docker-compose.prod.yml up -d --scale riptide-api=3

# Use load balancer (nginx, traefik, etc.)
```

## Security Considerations

1. **Secrets Management:**
   - Never commit `.env` files
   - Use Docker secrets or external secret managers
   - Rotate API keys regularly

2. **Network Security:**
   - Use internal networks for service communication
   - Expose only necessary ports
   - Consider using reverse proxy (nginx, traefik)

3. **Container Security:**
   - Images run as non-root user
   - Read-only filesystems where possible
   - Resource limits prevent DoS

## Backup and Recovery

### Backup Data

```bash
# Backup Redis data
docker-compose exec redis redis-cli BGSAVE
docker cp deployment_redis_1:/data ./backup/redis-$(date +%Y%m%d)

# Backup volumes
docker run --rm -v deployment_riptide-data:/data -v $(pwd):/backup \
  alpine tar czf /backup/riptide-data-$(date +%Y%m%d).tar.gz /data
```

### Restore Data

```bash
# Restore Redis data
docker cp ./backup/redis-20241031/dump.rdb deployment_redis_1:/data/

# Restore volumes
docker run --rm -v deployment_riptide-data:/data -v $(pwd):/backup \
  alpine tar xzf /backup/riptide-data-20241031.tar.gz -C /
```

## Updating Deployments

```bash
# Pull latest images
docker-compose pull

# Restart with new images
docker-compose up -d

# View update logs
docker-compose logs -f
```

## Additional Resources

- **Main Documentation**: `/docs/README.md`
- **Docker Examples**: `/examples/docker/README.md`
- **API Documentation**: `/docs/02-api-reference/`
- **Configuration Guide**: `/crates/riptide-config/README.md`

## Support

For issues and questions:
- **GitHub Issues**: https://github.com/yourusername/eventmesh/issues
- **Documentation**: `/docs/`
- **Health Check**: `curl http://localhost:8080/healthz`
