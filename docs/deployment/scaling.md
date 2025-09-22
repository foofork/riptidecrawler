# Scaling and Performance Guide

This guide covers scaling RipTide Crawler from small deployments to high-throughput, enterprise-grade systems handling millions of URLs per day.

## Scaling Overview

RipTide Crawler supports multiple scaling dimensions:

1. **Horizontal Scaling** - Add more instances
2. **Vertical Scaling** - Increase instance resources
3. **Component Scaling** - Scale individual services independently
4. **Geographic Scaling** - Distribute across regions
5. **Caching Scaling** - Optimize cache layers

## Performance Baseline

### Single Instance Performance

**Standard Configuration:**
- 4 CPU cores, 8GB RAM
- Concurrency: 16
- ~500-1000 URLs/hour
- ~10-50 requests/second

**Optimized Configuration:**
- 8 CPU cores, 16GB RAM
- Concurrency: 32
- ~2000-5000 URLs/hour
- ~50-200 requests/second

### Scaling Targets

| Scale Level | URLs/Hour | Requests/Second | Instances | Infrastructure |
|-------------|-----------|-----------------|-----------|----------------|
| Small       | 1K-10K    | 10-50          | 1-3       | Single server  |
| Medium      | 10K-100K  | 50-200         | 3-10      | Load balanced  |
| Large       | 100K-1M   | 200-1000       | 10-50     | Multi-AZ       |
| Enterprise  | 1M-10M    | 1000-5000      | 50-200    | Multi-region   |

## Horizontal Scaling Strategies

### API Service Scaling

#### Load Balancer Configuration

```nginx
# nginx.conf - Advanced load balancing
upstream riptide_api {
    # Load balancing method
    least_conn;  # or ip_hash, hash $uri, random

    # API instances
    server api1.internal:8080 max_fails=3 fail_timeout=30s weight=3;
    server api2.internal:8080 max_fails=3 fail_timeout=30s weight=3;
    server api3.internal:8080 max_fails=3 fail_timeout=30s weight=2;
    server api4.internal:8080 max_fails=3 fail_timeout=30s weight=1 backup;

    # Health checking
    check interval=3000 rise=2 fall=3 timeout=1000;

    # Keep-alive connections
    keepalive 32;
    keepalive_requests 100;
    keepalive_timeout 60s;
}

server {
    listen 80;

    # Connection limiting
    limit_conn_zone $binary_remote_addr zone=conn_limit:10m;
    limit_conn conn_limit 10;

    # Request rate limiting
    limit_req_zone $binary_remote_addr zone=req_limit:10m rate=10r/s;
    limit_req zone=req_limit burst=20 nodelay;

    location / {
        proxy_pass http://riptide_api;

        # Load balancing headers
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Performance optimization
        proxy_buffering on;
        proxy_buffer_size 8k;
        proxy_buffers 32 8k;
        proxy_busy_buffers_size 16k;

        # Timeouts
        proxy_connect_timeout 5s;
        proxy_send_timeout 30s;
        proxy_read_timeout 30s;

        # Keep-alive
        proxy_http_version 1.1;
        proxy_set_header Connection "";
    }

    # Health check endpoint
    location /health {
        proxy_pass http://riptide_api/health;
        access_log off;
    }
}
```

#### Auto-Scaling Configuration

```yaml
# docker-compose.scale.yml
version: '3.8'

services:
  api:
    image: riptide/api:v0.1.0
    deploy:
      replicas: 5
      update_config:
        parallelism: 2
        delay: 10s
        failure_action: rollback
      restart_policy:
        condition: on-failure
        delay: 5s
        max_attempts: 3
        window: 120s
      resources:
        limits:
          memory: 2G
          cpus: '1.0'
        reservations:
          memory: 1G
          cpus: '0.5'

  # Auto-scaler service (custom)
  autoscaler:
    image: riptide/autoscaler:latest
    environment:
      - DOCKER_HOST=unix:///var/run/docker.sock
      - SCALE_TARGET=api
      - MIN_REPLICAS=2
      - MAX_REPLICAS=20
      - CPU_THRESHOLD=70
      - MEMORY_THRESHOLD=80
      - SCALE_UP_COOLDOWN=300
      - SCALE_DOWN_COOLDOWN=600
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro
```

### Kubernetes Horizontal Pod Autoscaler

```yaml
# k8s/hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: riptide-api-hpa
  namespace: riptide-crawler
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: riptide-api
  minReplicas: 3
  maxReplicas: 50
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  - type: Pods
    pods:
      metric:
        name: http_requests_per_second
      target:
        type: AverageValue
        averageValue: "100"
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
      - type: Percent
        value: 100
        periodSeconds: 15
      - type: Pods
        value: 4
        periodSeconds: 15
      selectPolicy: Max
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Percent
        value: 10
        periodSeconds: 60

---
# Custom metrics for HPA
apiVersion: v1
kind: Service
metadata:
  name: riptide-api-metrics
  namespace: riptide-crawler
  labels:
    app: riptide-api
spec:
  ports:
  - name: metrics
    port: 9090
    targetPort: 9090
  selector:
    app: riptide-api
```

## Vertical Scaling

### Resource Optimization

#### Memory Scaling

```yaml
# High-memory configuration for large crawls
api:
  # JVM tuning (if applicable)
  heap_size: "8g"
  gc_algorithm: "G1"
  gc_options: "-XX:+UseG1GC -XX:MaxGCPauseMillis=200"

crawl:
  # Increase concurrency with more memory
  concurrency: 64
  max_response_mb: 50  # Allow larger responses

  # Connection pooling
  connection_pool_size: 200
  keep_alive_pool_size: 100

extraction:
  # Parallel WASM execution
  wasm_pool_size: 32
  parallel_extractions: 16

  # Larger content processing
  max_input_bytes: 104857600  # 100MB
  token_chunk_max: 2400
```

#### CPU Scaling

```yaml
# CPU-optimized configuration
api:
  # Worker processes
  workers: 16  # Match CPU cores
  worker_connections: 1024

  # Thread pool sizing
  extraction_threads: 16
  http_threads: 32

crawl:
  # CPU-bound optimizations
  concurrency: 128  # Higher with more CPU
  parallel_dns_resolution: true
  parallel_ssl_handshake: true

dynamic:
  # Chrome process management
  max_concurrent_sessions: 8
  chrome_processes: 4
  process_pool_size: 16
```

### Container Resource Limits

```yaml
# Docker Compose with precise resource allocation
services:
  api:
    deploy:
      resources:
        limits:
          memory: 8G
          cpus: '4.0'
        reservations:
          memory: 4G
          cpus: '2.0'
    environment:
      # Resource-aware configuration
      - RIPTIDE_WORKERS=4
      - RIPTIDE_CRAWL_CONCURRENCY=32
      - JAVA_OPTS=-Xmx6g -XX:+UseG1GC

  headless:
    deploy:
      resources:
        limits:
          memory: 16G
          cpus: '8.0'
        reservations:
          memory: 8G
          cpus: '4.0'
    shm_size: 4gb
    environment:
      - CHROME_FLAGS=--max_old_space_size=8192
```

## Component-Specific Scaling

### Redis Scaling

#### Redis Cluster Setup

```yaml
# Redis cluster for horizontal scaling
version: '3.8'

services:
  redis-1:
    image: redis:7-alpine
    command: >
      redis-server
      --cluster-enabled yes
      --cluster-config-file nodes.conf
      --cluster-node-timeout 5000
      --appendonly yes
      --maxmemory 2gb
      --maxmemory-policy allkeys-lru
    ports:
      - "7001:6379"
    volumes:
      - redis_1_data:/data

  redis-2:
    image: redis:7-alpine
    command: >
      redis-server
      --cluster-enabled yes
      --cluster-config-file nodes.conf
      --cluster-node-timeout 5000
      --appendonly yes
      --maxmemory 2gb
      --maxmemory-policy allkeys-lru
    ports:
      - "7002:6379"
    volumes:
      - redis_2_data:/data

  redis-3:
    image: redis:7-alpine
    command: >
      redis-server
      --cluster-enabled yes
      --cluster-config-file nodes.conf
      --cluster-node-timeout 5000
      --appendonly yes
      --maxmemory 2gb
      --maxmemory-policy allkeys-lru
    ports:
      - "7003:6379"
    volumes:
      - redis_3_data:/data

  # Cluster initialization
  redis-cluster-init:
    image: redis:7-alpine
    depends_on:
      - redis-1
      - redis-2
      - redis-3
    command: >
      sh -c "
        sleep 10
        redis-cli --cluster create
        redis-1:6379 redis-2:6379 redis-3:6379
        --cluster-replicas 0 --cluster-yes
      "

volumes:
  redis_1_data:
  redis_2_data:
  redis_3_data:
```

#### Redis Configuration Optimization

```conf
# redis.conf - Production optimizations

# Memory optimization
maxmemory 4gb
maxmemory-policy allkeys-lru
maxmemory-samples 10

# Network optimization
tcp-keepalive 60
tcp-backlog 511
timeout 300

# Persistence optimization
save 900 1
save 300 10
save 60 10000
stop-writes-on-bgsave-error no

# Performance tuning
hz 10
dynamic-hz yes

# Client optimization
client-output-buffer-limit normal 0 0 0
client-output-buffer-limit replica 256mb 64mb 60
client-output-buffer-limit pubsub 32mb 8mb 60

# Cluster optimization (if using cluster)
cluster-enabled yes
cluster-node-timeout 15000
cluster-replica-validity-factor 0
cluster-migration-barrier 1
```

### Headless Service Scaling

#### Chrome Instance Management

```yaml
# Headless service with multiple Chrome instances
services:
  headless-1:
    image: riptide/headless:v0.1.0
    environment:
      - CHROME_INSTANCES=4
      - INSTANCE_MEMORY_LIMIT=2gb
      - INSTANCE_CPU_LIMIT=1.0
    deploy:
      resources:
        limits:
          memory: 8G
          cpus: '4.0'
    shm_size: 4gb

  headless-2:
    image: riptide/headless:v0.1.0
    environment:
      - CHROME_INSTANCES=4
      - INSTANCE_MEMORY_LIMIT=2gb
      - INSTANCE_CPU_LIMIT=1.0
    deploy:
      resources:
        limits:
          memory: 8G
          cpus: '4.0'
    shm_size: 4gb

  # Chrome instance pool manager
  chrome-pool-manager:
    image: riptide/chrome-pool:latest
    environment:
      - POOL_SIZE=16
      - INSTANCE_LIFETIME=3600  # 1 hour
      - HEALTH_CHECK_INTERVAL=30
      - AUTO_RESTART=true
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
```

#### Chrome Performance Optimization

```yaml
# Optimized Chrome configuration
headless:
  environment:
    - CHROME_FLAGS=--no-sandbox
                   --disable-dev-shm-usage
                   --disable-gpu
                   --disable-extensions
                   --disable-plugins
                   --disable-images
                   --disable-javascript
                   --virtual-time-budget=5000
                   --aggressive-cache-discard
                   --memory-pressure-off
                   --max_old_space_size=2048
                   --disable-background-timer-throttling
                   --disable-backgrounding-occluded-windows
                   --disable-renderer-backgrounding
```

## Geographic Scaling

### Multi-Region Architecture

```yaml
# Global deployment configuration
global_infrastructure:
  regions:
    us-west:
      primary: true
      api_instances: 10
      headless_instances: 5
      redis_cluster: 3
      capacity: "40%"

    us-east:
      primary: false
      api_instances: 8
      headless_instances: 4
      redis_cluster: 3
      capacity: "30%"

    eu-west:
      primary: false
      api_instances: 6
      headless_instances: 3
      redis_cluster: 3
      capacity: "20%"

    ap-southeast:
      primary: false
      api_instances: 4
      headless_instances: 2
      redis_cluster: 3
      capacity: "10%"

  traffic_routing:
    strategy: "geolocation"
    failover: "health_based"
    sticky_sessions: false
```

### CDN Integration

```yaml
# Cloudflare configuration for global distribution
cloudflare:
  zones:
    - zone: "api.riptide.yourdomain.com"
      ssl: "strict"
      cache_level: "aggressive"
      development_mode: false

  page_rules:
    - url: "api.riptide.yourdomain.com/health*"
      settings:
        cache_level: "bypass"
        browser_ttl: 0

    - url: "api.riptide.yourdomain.com/crawl*"
      settings:
        cache_level: "bypass"
        browser_ttl: 0

    - url: "api.riptide.yourdomain.com/static/*"
      settings:
        cache_level: "cache_everything"
        browser_ttl: 86400
        edge_ttl: 86400

  load_balancer:
    pools:
      - name: "us-west"
        origins:
          - address: "us-west.api.internal"
            weight: 1
      - name: "us-east"
        origins:
          - address: "us-east.api.internal"
            weight: 1

    rules:
      - condition: "http.geo.country == 'US' && http.geo.region == 'California'"
        pool: "us-west"
      - condition: "http.geo.country == 'US'"
        pool: "us-east"
      - condition: "http.geo.continent == 'NA'"
        pool: "us-west"
      - pool: "us-west"  # default
```

## High-Throughput Optimization

### Queue-Based Architecture

```yaml
# Message queue integration for high throughput
services:
  # API Gateway (lightweight)
  api-gateway:
    image: riptide/api-gateway:latest
    ports:
      - "8080:8080"
    environment:
      - QUEUE_BACKEND=rabbitmq
      - RABBITMQ_URL=amqp://rabbitmq:5672
    deploy:
      replicas: 5

  # Message Queue
  rabbitmq:
    image: rabbitmq:3-management
    environment:
      - RABBITMQ_DEFAULT_USER=riptide
      - RABBITMQ_DEFAULT_PASS=${RABBITMQ_PASSWORD}
    volumes:
      - rabbitmq_data:/var/lib/rabbitmq
    deploy:
      resources:
        limits:
          memory: 4G
          cpus: '2.0'

  # Worker nodes
  worker:
    image: riptide/worker:latest
    environment:
      - QUEUE_BACKEND=rabbitmq
      - RABBITMQ_URL=amqp://rabbitmq:5672
      - WORKER_CONCURRENCY=16
    deploy:
      replicas: 20
      resources:
        limits:
          memory: 4G
          cpus: '2.0'

  # Result aggregator
  aggregator:
    image: riptide/aggregator:latest
    environment:
      - QUEUE_BACKEND=rabbitmq
      - RABBITMQ_URL=amqp://rabbitmq:5672
      - REDIS_URL=redis://redis-cluster:6379
    deploy:
      replicas: 3

volumes:
  rabbitmq_data:
```

### Stream Processing

```yaml
# Apache Kafka for stream processing
services:
  zookeeper:
    image: confluentinc/cp-zookeeper:latest
    environment:
      ZOOKEEPER_CLIENT_PORT: 2181
      ZOOKEEPER_TICK_TIME: 2000

  kafka:
    image: confluentinc/cp-kafka:latest
    depends_on:
      - zookeeper
    environment:
      KAFKA_BROKER_ID: 1
      KAFKA_ZOOKEEPER_CONNECT: zookeeper:2181
      KAFKA_ADVERTISED_LISTENERS: PLAINTEXT://kafka:9092
      KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR: 1
      KAFKA_LOG_RETENTION_HOURS: 168
      KAFKA_LOG_SEGMENT_BYTES: 1073741824
      KAFKA_NUM_PARTITIONS: 16

  # Kafka producer (URL intake)
  url-producer:
    image: riptide/kafka-producer:latest
    environment:
      - KAFKA_BROKERS=kafka:9092
      - TOPIC_URLS=riptide-urls
      - BATCH_SIZE=1000

  # Kafka consumer (URL processing)
  url-consumer:
    image: riptide/kafka-consumer:latest
    environment:
      - KAFKA_BROKERS=kafka:9092
      - TOPIC_URLS=riptide-urls
      - TOPIC_RESULTS=riptide-results
      - CONSUMER_GROUP=riptide-workers
    deploy:
      replicas: 10
```

## Performance Monitoring and Optimization

### Comprehensive Metrics

```yaml
# Prometheus configuration for scaling metrics
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "scaling_rules.yml"

scrape_configs:
  - job_name: 'riptide-api'
    static_configs:
      - targets: ['api1:8080', 'api2:8080', 'api3:8080']
    metrics_path: /metrics
    scrape_interval: 5s

  - job_name: 'riptide-headless'
    static_configs:
      - targets: ['headless1:9123', 'headless2:9123']
    metrics_path: /metrics
    scrape_interval: 10s

  - job_name: 'redis'
    static_configs:
      - targets: ['redis-exporter:9121']

  - job_name: 'nginx'
    static_configs:
      - targets: ['nginx-exporter:9113']

  - job_name: 'node'
    static_configs:
      - targets: ['node-exporter:9100']
```

### Scaling Rules

```yaml
# scaling_rules.yml
groups:
  - name: scaling_rules
    rules:
      # Scale up indicators
      - alert: HighCPUUsage
        expr: avg(cpu_usage_percent) by (instance) > 80
        for: 2m
        labels:
          severity: warning
          action: scale_up
        annotations:
          summary: "High CPU usage detected"

      - alert: HighMemoryUsage
        expr: avg(memory_usage_percent) by (instance) > 85
        for: 2m
        labels:
          severity: warning
          action: scale_up

      - alert: HighRequestRate
        expr: rate(http_requests_total[5m]) > 1000
        for: 1m
        labels:
          severity: info
          action: scale_up

      # Scale down indicators
      - alert: LowResourceUsage
        expr: avg(cpu_usage_percent) by (instance) < 20 AND avg(memory_usage_percent) by (instance) < 30
        for: 10m
        labels:
          severity: info
          action: scale_down

      # Performance degradation
      - alert: SlowResponseTime
        expr: histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m])) > 10
        for: 3m
        labels:
          severity: critical
          action: investigate

      - alert: QueueBacklog
        expr: rabbitmq_queue_messages > 10000
        for: 2m
        labels:
          severity: warning
          action: scale_up
```

## Cost-Efficient Scaling

### Spot Instance Management

```hcl
# AWS Auto Scaling with Spot Instances
resource "aws_autoscaling_group" "riptide_workers" {
  name                = "riptide-workers"
  vpc_zone_identifier = var.private_subnet_ids
  target_group_arns   = [aws_lb_target_group.riptide_api.arn]
  health_check_type   = "ELB"

  min_size         = 2
  max_size         = 50
  desired_capacity = 5

  mixed_instances_policy {
    launch_template {
      launch_template_specification {
        launch_template_id = aws_launch_template.riptide_worker.id
        version            = "$Latest"
      }

      override {
        instance_type     = "c5.large"
        weighted_capacity = "1"
      }

      override {
        instance_type     = "c5.xlarge"
        weighted_capacity = "2"
      }

      override {
        instance_type     = "c4.large"
        weighted_capacity = "1"
      }
    }

    instances_distribution {
      on_demand_base_capacity                  = 2
      on_demand_percentage_above_base_capacity = 20
      spot_allocation_strategy                 = "diversified"
      spot_instance_pools                      = 3
      spot_max_price                          = "0.10"
    }
  }

  tag {
    key                 = "Name"
    value               = "riptide-worker"
    propagate_at_launch = true
  }
}

# Auto Scaling Policies
resource "aws_autoscaling_policy" "scale_up" {
  name                   = "riptide-scale-up"
  scaling_adjustment     = 2
  adjustment_type        = "ChangeInCapacity"
  cooldown              = 300
  autoscaling_group_name = aws_autoscaling_group.riptide_workers.name
}

resource "aws_autoscaling_policy" "scale_down" {
  name                   = "riptide-scale-down"
  scaling_adjustment     = -1
  adjustment_type        = "ChangeInCapacity"
  cooldown              = 600
  autoscaling_group_name = aws_autoscaling_group.riptide_workers.name
}
```

### Resource Scheduling

```yaml
# Kubernetes resource optimization
apiVersion: v1
kind: ResourceQuota
metadata:
  name: riptide-quota
  namespace: riptide-crawler
spec:
  hard:
    requests.cpu: "50"
    requests.memory: 100Gi
    limits.cpu: "100"
    limits.memory: 200Gi
    pods: "50"

---
apiVersion: v1
kind: LimitRange
metadata:
  name: riptide-limits
  namespace: riptide-crawler
spec:
  limits:
  - default:
      cpu: "2"
      memory: "4Gi"
    defaultRequest:
      cpu: "500m"
      memory: "1Gi"
    type: Container

---
# Priority classes for workload scheduling
apiVersion: scheduling.k8s.io/v1
kind: PriorityClass
metadata:
  name: riptide-high-priority
value: 1000
globalDefault: false
description: "High priority for critical riptide components"

---
apiVersion: scheduling.k8s.io/v1
kind: PriorityClass
metadata:
  name: riptide-low-priority
value: 100
globalDefault: false
description: "Low priority for batch processing"
```

## Troubleshooting Scaling Issues

### Performance Debugging

```bash
#!/bin/bash
# Performance analysis script

echo "=== System Resources ==="
echo "CPU Usage:"
top -bn1 | grep "Cpu(s)" | sed "s/.*, *\([0-9.]*\)%* id.*/\1/" | awk '{print 100 - $1"% used"}'

echo "Memory Usage:"
free -h

echo "Disk I/O:"
iostat -x 1 1

echo "=== Container Resources ==="
docker stats --no-stream --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.NetIO}}\t{{.BlockIO}}"

echo "=== Network Connections ==="
ss -tuln | grep -E ':(8080|9123|6379)'

echo "=== Application Metrics ==="
curl -s http://localhost:8080/metrics | grep -E "(http_requests_total|memory_usage|cpu_usage)"

echo "=== Redis Performance ==="
redis-cli info stats | grep -E "(instantaneous_ops_per_sec|used_memory_human|connected_clients)"

echo "=== Load Balancer Status ==="
curl -s http://localhost/nginx_status

echo "=== Queue Status ==="
curl -s http://localhost:15672/api/queues | jq '.[] | {name: .name, messages: .messages, consumers: .consumers}'
```

### Scaling Bottleneck Identification

```python
#!/usr/bin/env python3
# bottleneck_analyzer.py

import requests
import time
import statistics
import json

def analyze_bottlenecks():
    metrics = {
        'response_times': [],
        'cpu_usage': [],
        'memory_usage': [],
        'queue_sizes': [],
        'cache_hit_rates': []
    }

    # Collect metrics over time
    for i in range(60):  # 1 minute of data
        try:
            # API response time
            start_time = time.time()
            response = requests.get('http://localhost:8080/health')
            response_time = time.time() - start_time
            metrics['response_times'].append(response_time)

            # System metrics from Prometheus
            prom_metrics = requests.get('http://localhost:9090/api/v1/query', params={
                'query': 'avg(rate(cpu_usage_total[1m]))'
            }).json()

            if prom_metrics['data']['result']:
                cpu_usage = float(prom_metrics['data']['result'][0]['value'][1])
                metrics['cpu_usage'].append(cpu_usage)

            time.sleep(1)

        except Exception as e:
            print(f"Error collecting metrics: {e}")
            continue

    # Analyze bottlenecks
    analysis = {
        'avg_response_time': statistics.mean(metrics['response_times']),
        'p95_response_time': statistics.quantiles(metrics['response_times'], n=20)[18],
        'avg_cpu_usage': statistics.mean(metrics['cpu_usage']) if metrics['cpu_usage'] else 0,
        'max_cpu_usage': max(metrics['cpu_usage']) if metrics['cpu_usage'] else 0,
    }

    # Recommendations
    recommendations = []

    if analysis['p95_response_time'] > 5.0:
        recommendations.append("High response times detected - consider scaling API instances")

    if analysis['avg_cpu_usage'] > 80:
        recommendations.append("High CPU usage - consider vertical scaling or adding instances")

    if analysis['avg_response_time'] > analysis['p95_response_time'] * 0.8:
        recommendations.append("Consistent high response times - check for system bottlenecks")

    print(json.dumps({
        'analysis': analysis,
        'recommendations': recommendations
    }, indent=2))

if __name__ == '__main__':
    analyze_bottlenecks()
```

This comprehensive scaling guide provides the foundation for growing RipTide Crawler from a small deployment to an enterprise-scale system capable of handling millions of requests efficiently and cost-effectively.