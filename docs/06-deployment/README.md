# Deployment Guide

Production deployment and operations documentation for RipTide.

## 🚀 Quick Deployment

### Docker Deployment (Recommended)

```bash
# Pull latest image
docker pull riptide/api:latest

# Run with Docker Compose
docker-compose up -d

# Verify deployment
curl http://localhost:8080/healthz
```

### Kubernetes Deployment

```bash
# Apply manifests
kubectl apply -f k8s/

# Check status
kubectl get pods -l app=riptide

# Access service
kubectl port-forward svc/riptide 8080:8080
```

## 📚 Production Guides

### Core Deployment
- **[Production Guide](./production/production.md)** - Complete production setup (⏱️ 30 min)
- **[Docker Guide](./production/docker.md)** - Docker deployment details (⏱️ 20 min)
- **[Scaling Guide](./production/scaling.md)** - Horizontal and vertical scaling (⏱️ 25 min)

### Specialized Topics
- **[Swagger UI Deployment](./production/SWAGGER_UI_DEPLOYMENT_GUIDE.md)** - API documentation hosting (⏱️ 15 min)

## 🏗️ Deployment Architecture

### Single Instance (Development)

```
┌──────────────────┐
│  RipTide API     │
│  (Port 8080)     │
└────────┬─────────┘
         │
    ┌────▼─────┐
    │  Redis   │
    └──────────┘
```

### Production Setup (Recommended)

```
                    ┌──────────────┐
                    │ Load Balancer│
                    └──────┬───────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
   ┌────▼────┐        ┌────▼────┐       ┌────▼────┐
   │RipTide 1│        │RipTide 2│       │RipTide 3│
   └────┬────┘        └────┬────┘       └────┬────┘
        │                  │                  │
        └──────────────────┼──────────────────┘
                           │
              ┌────────────▼─────────────┐
              │   Redis Cluster          │
              │  (High Availability)     │
              └──────────────────────────┘
                           │
              ┌────────────▼─────────────┐
              │   Headless Browsers      │
              │  (Browserless Pool)      │
              └──────────────────────────┘
```

## ⚙️ Configuration

### Environment Variables

```bash
# Core Settings
RUST_LOG=info
HOST=0.0.0.0
PORT=8080

# Redis Configuration
REDIS_URL=redis://redis-cluster:6379
REDIS_POOL_SIZE=10
REDIS_MAX_CONNECTIONS=100

# Cache Settings
CACHE_TTL=3600
CACHE_MAX_SIZE_MB=1024

# Performance
MAX_CONCURRENCY=10
REQUEST_TIMEOUT=30
MAX_PAYLOAD_SIZE=10485760

# Browser Pool
HEADLESS_URL=http://browserless:3000
BROWSER_POOL_SIZE=5
BROWSER_TIMEOUT=30000

# Features
ENABLE_METRICS=true
ENABLE_TRACING=true
ENABLE_STEALTH=true

# API Keys (Production)
SERPER_API_KEY=${SERPER_API_KEY}
```

### Docker Compose Example

```yaml
version: '3.8'

services:
  riptide:
    image: riptide/api:latest
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=info
      - REDIS_URL=redis://redis:6379
      - HEADLESS_URL=http://browserless:3000
      - SERPER_API_KEY=${SERPER_API_KEY}
    depends_on:
      - redis
      - browserless
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/healthz"]
      interval: 30s
      timeout: 10s
      retries: 3
    deploy:
      replicas: 3
      resources:
        limits:
          cpus: '2'
          memory: 4G
        reservations:
          cpus: '1'
          memory: 2G

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis-data:/data
    command: redis-server --appendonly yes
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 3s
      retries: 3

  browserless:
    image: ghcr.io/browserless/chromium:latest
    ports:
      - "3000:3000"
    environment:
      - CONCURRENT=10
      - TOKEN=${BROWSER_TOKEN}
      - MAX_CONCURRENT_SESSIONS=10
    deploy:
      resources:
        limits:
          cpus: '4'
          memory: 8G

  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus-data:/prometheus

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3001:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=${GRAFANA_PASSWORD}
    volumes:
      - grafana-data:/var/lib/grafana

volumes:
  redis-data:
  prometheus-data:
  grafana-data:
```

## 📊 Monitoring & Observability

### Health Checks

```bash
# Basic health check
curl http://localhost:8080/healthz

# Detailed health status
curl http://localhost:8080/healthz | jq .

# Prometheus metrics
curl http://localhost:8080/metrics
```

### Metrics Dashboard

Access Grafana at `http://localhost:3001`:
- Request rate and latency
- Cache hit ratio
- Error rates
- Resource utilization
- Browser pool status

### Logging

```bash
# Container logs
docker logs -f riptide-api

# Kubernetes logs
kubectl logs -f deployment/riptide

# Filter by level
docker logs riptide-api 2>&1 | grep "ERROR"
```

## 🔒 Security

### Production Checklist

- [ ] Use environment variables for secrets
- [ ] Enable HTTPS/TLS
- [ ] Configure rate limiting
- [ ] Set up firewall rules
- [ ] Enable authentication
- [ ] Regular security updates
- [ ] Monitor access logs
- [ ] Implement backup strategy

### TLS Configuration

```nginx
server {
    listen 443 ssl http2;
    server_name api.example.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        proxy_pass http://riptide:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

## 📈 Scaling Strategies

### Horizontal Scaling

```bash
# Docker Compose
docker-compose up -d --scale riptide=5

# Kubernetes
kubectl scale deployment riptide --replicas=5

# Auto-scaling (K8s)
kubectl autoscale deployment riptide --min=3 --max=10 --cpu-percent=70
```

### Vertical Scaling

Adjust resource limits in Docker Compose or Kubernetes:

```yaml
resources:
  limits:
    cpus: '4'
    memory: 8G
  reservations:
    cpus: '2'
    memory: 4G
```

### Performance Tuning

See [Scaling Guide](./production/scaling.md) for:
- Redis clustering
- Browser pool optimization
- Connection pooling
- Cache strategies
- Load balancing

## 🔄 Backup & Recovery

### Redis Backup

```bash
# Manual backup
docker exec redis redis-cli BGSAVE

# Scheduled backups (cron)
0 2 * * * docker exec redis redis-cli BGSAVE
```

### Application State

```bash
# Export configuration
docker exec riptide env > config-backup.env

# Database dump
docker exec redis redis-cli --rdb /data/dump.rdb
```

## 🚨 Troubleshooting

### Common Issues

| Issue | Symptoms | Solution |
|-------|----------|----------|
| **High Memory Usage** | OOM errors | Scale vertically, adjust cache size |
| **Slow Response Times** | Timeouts | Check Redis latency, scale horizontally |
| **Connection Errors** | 502/503 errors | Verify services are healthy |
| **Cache Misses** | Low hit rate | Review cache TTL, warm cache |

### Debug Commands

```bash
# Check service status
docker-compose ps

# View resource usage
docker stats

# Test connectivity
docker exec riptide curl redis:6379

# Check browser pool
curl http://localhost:3000/metrics
```

## 📋 Deployment Checklist

### Pre-Deployment
- [ ] Review [Production Guide](./production/production.md)
- [ ] Configure environment variables
- [ ] Set up monitoring and alerting
- [ ] Configure backups
- [ ] Review security settings
- [ ] Load test application
- [ ] Prepare rollback plan

### Deployment
- [ ] Deploy to staging environment
- [ ] Run smoke tests
- [ ] Monitor metrics during rollout
- [ ] Verify health checks pass
- [ ] Test critical workflows
- [ ] Update documentation

### Post-Deployment
- [ ] Monitor error rates
- [ ] Check performance metrics
- [ ] Verify logging and tracing
- [ ] Review security alerts
- [ ] Document any issues
- [ ] Update runbooks

## 🎓 Learning Path

**Beginner** (1 hour):
1. [Docker Guide](./production/docker.md)
2. Deploy locally with Docker Compose
3. Verify health checks

**Intermediate** (3 hours):
1. [Production Guide](./production/production.md)
2. [Scaling Guide](./production/scaling.md)
3. Set up monitoring
4. Configure backups

**Advanced** (Full day):
1. Kubernetes deployment
2. High availability setup
3. Performance optimization
4. Disaster recovery planning

## 🔗 Related Documentation

- **[Architecture](../04-architecture/README.md)** - System design
- **[Development Guide](../05-development/README.md)** - Developer setup
- **[Advanced Topics](../07-advanced/README.md)** - Performance tuning
- **[Operations Guides](../01-guides/operations/)** - Day-to-day operations

## 🆘 Need Help?

- **[Troubleshooting](../01-guides/operations/troubleshooting.md)** - Common issues
- **[GitHub Issues](https://github.com/your-org/eventmesh/issues)** - Support
- **[Discussions](https://github.com/your-org/eventmesh/discussions)** - Questions

---

**Ready for production?** → [Production Guide](./production/production.md)
