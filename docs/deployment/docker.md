# Docker Deployment Guide

This guide covers deploying RipTide Crawler using Docker containers, from simple single-machine setups to complex multi-node deployments.

## Quick Start

```bash
# Clone repository
git clone <repository-url>
cd riptide-crawler

# Start with default configuration
docker-compose up -d

# Verify deployment
curl http://localhost:8080/health
```

## Container Architecture

RipTide Crawler consists of three main services:

1. **API Service** (`riptide-api`) - REST API server
2. **Headless Service** (`riptide-headless`) - Chrome DevTools Protocol service
3. **Redis** - Caching and session storage

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   API Service   │    │ Headless Service│    │     Redis       │
│   Port 8080     │    │   Port 9123     │    │   Port 6379     │
│                 │◄──►│                 │    │                 │
│ - REST API      │    │ - Chrome CDP    │    │ - Cache         │
│ - WASM Extract  │    │ - Screenshots   │    │ - Session Store │
│ - Load Balance  │    │ - Dynamic Pages │    │ - Rate Limits   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Docker Images

### Official Images

```bash
# Pull official images
docker pull riptide/api:latest
docker pull riptide/headless:latest

# Or specific versions
docker pull riptide/api:v0.1.0
docker pull riptide/headless:v0.1.0
```

### Building Custom Images

```bash
# Build API service
docker build -f infra/docker/Dockerfile.api -t riptide-api:custom .

# Build headless service
docker build -f infra/docker/Dockerfile.headless -t riptide-headless:custom .

# Build with specific WASM extractor
docker build -f infra/docker/Dockerfile.api \
  --build-arg WASM_MODULE=./target/wasm32-wasip2/release/custom-extractor.wasm \
  -t riptide-api:custom .
```

## Basic Docker Compose Setup

### Simple Configuration

```yaml
# docker-compose.yml
version: '3.8'

services:
  api:
    image: riptide/api:latest
    ports:
      - "8080:8080"
    environment:
      - REDIS_URL=redis://redis:6379
      - RUST_LOG=info
      - RIPTIDE_HEADLESS_SERVICE_URL=http://headless:9123
    depends_on:
      - redis
      - headless
    volumes:
      - ./data:/data
      - ./configs:/etc/riptide
    restart: unless-stopped

  headless:
    image: riptide/headless:latest
    ports:
      - "9123:9123"
    environment:
      - RUST_LOG=info
    shm_size: 2gb
    restart: unless-stopped

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data
    restart: unless-stopped

volumes:
  redis_data:
```

### Environment Configuration

```bash
# .env file
SERPER_API_KEY=your_serper_api_key_here
REDIS_URL=redis://redis:6379
RUST_LOG=info

# API Configuration
RIPTIDE_API_HOST=0.0.0.0
RIPTIDE_API_PORT=8080
RIPTIDE_CRAWL_CONCURRENCY=16

# Headless Configuration
RIPTIDE_HEADLESS_HOST=0.0.0.0
RIPTIDE_HEADLESS_PORT=9123
RIPTIDE_HEADLESS_MAX_SESSIONS=2

# Chrome flags for stability
CHROME_FLAGS=--no-sandbox --disable-dev-shm-usage --disable-gpu --remote-debugging-port=9222
```

### Start Services

```bash
# Start in background
docker-compose up -d

# View logs
docker-compose logs -f

# Check status
docker-compose ps

# Stop services
docker-compose down
```

## Production Configuration

### Production Docker Compose

```yaml
# docker-compose.prod.yml
version: '3.8'

services:
  api:
    image: riptide/api:v0.1.0
    ports:
      - "8080:8080"
    environment:
      - REDIS_URL=redis://redis:6379
      - RUST_LOG=warn
      - RIPTIDE_CONFIG_FILE=/etc/riptide/production.yml
    depends_on:
      - redis
      - headless
    volumes:
      - ./configs:/etc/riptide:ro
      - ./data:/data
      - ./logs:/var/log/riptide
    restart: always
    deploy:
      resources:
        limits:
          memory: 2G
          cpus: '2.0'
        reservations:
          memory: 1G
          cpus: '1.0'
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s

  headless:
    image: riptide/headless:v0.1.0
    environment:
      - RUST_LOG=warn
      - CHROME_FLAGS=--no-sandbox --disable-dev-shm-usage --disable-gpu
    shm_size: 2gb
    restart: always
    deploy:
      resources:
        limits:
          memory: 4G
          cpus: '2.0'
        reservations:
          memory: 2G
          cpus: '1.0'
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:9123/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  redis:
    image: redis:7-alpine
    command: redis-server --appendonly yes --maxmemory 1gb --maxmemory-policy allkeys-lru
    volumes:
      - redis_data:/data
    restart: always
    deploy:
      resources:
        limits:
          memory: 1.5G
          cpus: '1.0'
        reservations:
          memory: 512M
          cpus: '0.5'

  # Reverse proxy (optional)
  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./ssl:/etc/ssl:ro
    depends_on:
      - api
    restart: always

volumes:
  redis_data:
    driver: local
```

### Security Configuration

```yaml
# Security-focused setup
services:
  api:
    image: riptide/api:v0.1.0
    user: "1000:1000"  # Non-root user
    read_only: true
    tmpfs:
      - /tmp
      - /var/run
    volumes:
      - ./configs:/etc/riptide:ro
      - ./data:/data
    cap_drop:
      - ALL
    cap_add:
      - NET_BIND_SERVICE
    security_opt:
      - no-new-privileges:true

  headless:
    image: riptide/headless:v0.1.0
    user: "1000:1000"
    cap_drop:
      - ALL
    cap_add:
      - SYS_ADMIN  # Required for Chrome sandboxing
    security_opt:
      - seccomp:unconfined  # Required for Chrome
```

## High Availability Setup

### Multi-Instance API Deployment

```yaml
# docker-compose.ha.yml
version: '3.8'

services:
  # Load balancer
  traefik:
    image: traefik:v2.10
    command:
      - --api.dashboard=true
      - --providers.docker=true
      - --entrypoints.web.address=:80
      - --entrypoints.websecure.address=:443
    ports:
      - "80:80"
      - "443:443"
      - "8080:8080"  # Dashboard
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro

  # API instances
  api-1:
    image: riptide/api:v0.1.0
    environment:
      - REDIS_URL=redis://redis:6379
      - RIPTIDE_HEADLESS_SERVICE_URL=http://headless-1:9123,http://headless-2:9123
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.api.rule=Host(`riptide.example.com`)"
      - "traefik.http.services.api.loadbalancer.server.port=8080"
    depends_on:
      - redis

  api-2:
    image: riptide/api:v0.1.0
    environment:
      - REDIS_URL=redis://redis:6379
      - RIPTIDE_HEADLESS_SERVICE_URL=http://headless-1:9123,http://headless-2:9123
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.api.rule=Host(`riptide.example.com`)"
      - "traefik.http.services.api.loadbalancer.server.port=8080"
    depends_on:
      - redis

  # Headless instances
  headless-1:
    image: riptide/headless:v0.1.0
    shm_size: 2gb

  headless-2:
    image: riptide/headless:v0.1.0
    shm_size: 2gb

  # Redis cluster
  redis:
    image: redis:7-alpine
    command: redis-server --appendonly yes
    volumes:
      - redis_data:/data

volumes:
  redis_data:
```

### Redis Cluster Setup

```yaml
# Redis cluster for high availability
services:
  redis-1:
    image: redis:7-alpine
    command: redis-server --cluster-enabled yes --cluster-config-file nodes.conf --cluster-node-timeout 5000 --appendonly yes
    ports:
      - "7001:6379"
    volumes:
      - redis_1_data:/data

  redis-2:
    image: redis:7-alpine
    command: redis-server --cluster-enabled yes --cluster-config-file nodes.conf --cluster-node-timeout 5000 --appendonly yes
    ports:
      - "7002:6379"
    volumes:
      - redis_2_data:/data

  redis-3:
    image: redis:7-alpine
    command: redis-server --cluster-enabled yes --cluster-config-file nodes.conf --cluster-node-timeout 5000 --appendonly yes
    ports:
      - "7003:6379"
    volumes:
      - redis_3_data:/data

  # Cluster initialization
  redis-cluster-init:
    image: redis:7-alpine
    command: >
      sh -c "
        redis-cli --cluster create
        redis-1:6379 redis-2:6379 redis-3:6379
        --cluster-replicas 0 --cluster-yes
      "
    depends_on:
      - redis-1
      - redis-2
      - redis-3

volumes:
  redis_1_data:
  redis_2_data:
  redis_3_data:
```

## Monitoring and Logging

### Monitoring Stack

```yaml
# docker-compose.monitoring.yml
version: '3.8'

services:
  # Prometheus for metrics
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus_data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.console.libraries=/etc/prometheus/console_libraries'
      - '--web.console.templates=/etc/prometheus/consoles'

  # Grafana for visualization
  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin123
    volumes:
      - grafana_data:/var/lib/grafana
      - ./grafana/dashboards:/etc/grafana/provisioning/dashboards:ro
      - ./grafana/datasources:/etc/grafana/provisioning/datasources:ro

  # Loki for log aggregation
  loki:
    image: grafana/loki:latest
    ports:
      - "3100:3100"
    volumes:
      - ./loki-config.yml:/etc/loki/local-config.yaml:ro
      - loki_data:/tmp/loki

  # Promtail for log collection
  promtail:
    image: grafana/promtail:latest
    volumes:
      - /var/log:/var/log:ro
      - /var/lib/docker/containers:/var/lib/docker/containers:ro
      - ./promtail-config.yml:/etc/promtail/config.yml:ro

volumes:
  prometheus_data:
  grafana_data:
  loki_data:
```

### Log Configuration

```yaml
# Configure logging for containers
services:
  api:
    logging:
      driver: "json-file"
      options:
        max-size: "100m"
        max-file: "5"
    environment:
      - RUST_LOG=info
      - RIPTIDE_LOGGING_FORMAT=json

  headless:
    logging:
      driver: "syslog"
      options:
        syslog-address: "tcp://127.0.0.1:514"
        tag: "riptide-headless"
```

## Scaling Strategies

### Horizontal Scaling

```bash
# Scale API instances
docker-compose up -d --scale api=3

# Scale headless instances
docker-compose up -d --scale headless=2

# Scale with Docker Swarm
docker stack deploy -c docker-compose.swarm.yml riptide
```

### Docker Swarm Configuration

```yaml
# docker-compose.swarm.yml
version: '3.8'

services:
  api:
    image: riptide/api:v0.1.0
    deploy:
      replicas: 3
      restart_policy:
        condition: on-failure
        delay: 5s
        max_attempts: 3
      update_config:
        parallelism: 1
        delay: 10s
        failure_action: rollback
      resources:
        limits:
          memory: 2G
          cpus: '1.0'
        reservations:
          memory: 1G
          cpus: '0.5'
    networks:
      - riptide-network

  headless:
    image: riptide/headless:v0.1.0
    deploy:
      replicas: 2
      placement:
        constraints:
          - node.labels.headless == true
      resources:
        limits:
          memory: 4G
          cpus: '2.0'
    networks:
      - riptide-network

  redis:
    image: redis:7-alpine
    deploy:
      replicas: 1
      placement:
        constraints:
          - node.role == manager
    networks:
      - riptide-network

networks:
  riptide-network:
    driver: overlay
    attachable: true
```

## Performance Optimization

### Resource Limits

```yaml
services:
  api:
    deploy:
      resources:
        limits:
          memory: 2G
          cpus: '2.0'
        reservations:
          memory: 1G
          cpus: '1.0'
    # Optimize JVM if using Java components
    environment:
      - JAVA_OPTS=-Xmx1g -XX:+UseG1GC
    ulimits:
      nofile:
        soft: 65536
        hard: 65536

  headless:
    deploy:
      resources:
        limits:
          memory: 4G
          cpus: '2.0'
    shm_size: 2gb
    ulimits:
      nofile:
        soft: 65536
        hard: 65536
```

### Volume Optimization

```yaml
services:
  api:
    volumes:
      # Use tmpfs for temporary files
      - type: tmpfs
        target: /tmp
        tmpfs:
          size: 1G

      # Use bind mounts for config (read-only)
      - type: bind
        source: ./configs
        target: /etc/riptide
        read_only: true

      # Use named volumes for data
      - type: volume
        source: riptide_data
        target: /data
        volume:
          nocopy: true

volumes:
  riptide_data:
    driver: local
    driver_opts:
      type: none
      o: bind
      device: /fast/ssd/riptide
```

### Network Optimization

```yaml
# Optimize Docker networking
networks:
  riptide-internal:
    driver: bridge
    ipam:
      config:
        - subnet: 172.20.0.0/16
    driver_opts:
      com.docker.network.bridge.name: riptide-br
      com.docker.network.driver.mtu: 9000  # Jumbo frames if supported

services:
  api:
    networks:
      riptide-internal:
        ipv4_address: 172.20.0.10

  redis:
    networks:
      riptide-internal:
        ipv4_address: 172.20.0.20
```

## Security Best Practices

### Image Security

```dockerfile
# Use official, minimal base images
FROM rust:1.75-slim-bookworm AS builder

# Create non-root user
RUN addgroup --system --gid 1000 riptide && \
    adduser --system --uid 1000 --gid 1000 riptide

# Build application
COPY . /app
WORKDIR /app
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN addgroup --system --gid 1000 riptide && \
    adduser --system --uid 1000 --gid 1000 riptide

# Copy binary and set ownership
COPY --from=builder --chown=riptide:riptide /app/target/release/riptide-api /usr/local/bin/

# Switch to non-root user
USER 1000:1000

EXPOSE 8080
CMD ["riptide-api"]
```

### Runtime Security

```yaml
services:
  api:
    security_opt:
      - no-new-privileges:true
      - apparmor:docker-riptide
    cap_drop:
      - ALL
    cap_add:
      - NET_BIND_SERVICE
    read_only: true
    tmpfs:
      - /tmp:noexec,nosuid,size=100m

  headless:
    security_opt:
      - seccomp:unconfined  # Required for Chrome
    cap_drop:
      - ALL
    cap_add:
      - SYS_ADMIN  # Required for Chrome sandbox
```

### Secrets Management

```yaml
# Using Docker secrets
version: '3.8'

services:
  api:
    image: riptide/api:v0.1.0
    environment:
      - SERPER_API_KEY_FILE=/run/secrets/serper_api_key
    secrets:
      - serper_api_key

secrets:
  serper_api_key:
    file: ./secrets/serper_api_key.txt
```

## Backup and Recovery

### Data Backup

```bash
#!/bin/bash
# backup.sh

BACKUP_DIR="/backups/$(date +%Y%m%d_%H%M%S)"
mkdir -p "$BACKUP_DIR"

# Backup Redis data
docker-compose exec redis redis-cli BGSAVE
docker cp $(docker-compose ps -q redis):/data/dump.rdb "$BACKUP_DIR/redis_dump.rdb"

# Backup extracted content
tar -czf "$BACKUP_DIR/artifacts.tar.gz" ./data/artifacts/

# Backup configuration
cp -r ./configs "$BACKUP_DIR/"

echo "Backup completed: $BACKUP_DIR"
```

### Disaster Recovery

```bash
#!/bin/bash
# restore.sh

BACKUP_DIR="$1"

if [ -z "$BACKUP_DIR" ]; then
    echo "Usage: $0 <backup_directory>"
    exit 1
fi

# Stop services
docker-compose down

# Restore Redis data
docker-compose up -d redis
sleep 5
docker cp "$BACKUP_DIR/redis_dump.rdb" $(docker-compose ps -q redis):/data/
docker-compose restart redis

# Restore artifacts
tar -xzf "$BACKUP_DIR/artifacts.tar.gz" -C ./data/

# Restore configuration
cp -r "$BACKUP_DIR/configs" ./

# Start all services
docker-compose up -d

echo "Restore completed from: $BACKUP_DIR"
```

## Troubleshooting

### Common Docker Issues

```bash
# Check container logs
docker-compose logs api
docker-compose logs headless

# Inspect container
docker-compose exec api bash

# Check resource usage
docker stats

# Clean up resources
docker system prune -f
docker volume prune -f

# Rebuild containers
docker-compose build --no-cache
docker-compose up -d --force-recreate
```

### Health Checks

```bash
# Container health checks
docker-compose ps

# Service health checks
curl http://localhost:8080/health
curl http://localhost:9123/health

# Redis connectivity
docker-compose exec redis redis-cli ping
```

## Next Steps

- **Production Deployment**: See [Production Deployment Guide](production.md)
- **Scaling**: Learn about [Scaling and Performance](scaling.md)
- **Monitoring**: Set up comprehensive monitoring
- **Security**: Implement additional security measures