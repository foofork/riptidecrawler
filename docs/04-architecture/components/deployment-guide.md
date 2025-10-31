# RipTide - Deployment Guide

## Overview

This guide covers deploying RipTide in various environments, from local development to production Kubernetes clusters.

## Architecture Summary

RipTide consists of 3 main services:
- **riptide-api**: RipTide API service component (Port 8080)
- **riptide-headless**: RipTide browser automation component (Port 9123)
- **redis**: Cache and queue backend (Port 6379)

## Local Development Deployment

### Prerequisites

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install required targets
rustup target add wasm32-wasip2

# Install Docker and Docker Compose
# (Platform-specific installation)
```

### Environment Setup

```bash
# Clone repository
git clone <repository-url>
cd riptide-crawler

# Set required environment variables
export SERPER_API_KEY="your-serper-api-key"
export RUST_LOG="info"

# Build WASM component
cd wasm/riptide-extractor-wasm
cargo build --target wasm32-wasip2 --release
cd ../..

# Build main services
cargo build --release
```

### Docker Compose Deployment

```bash
# Start all services
docker-compose -f infra/docker/docker-compose.yml up -d

# Check service health
docker-compose ps
curl http://localhost:8080/healthz
curl http://localhost:9123/healthz

# View logs
docker-compose logs -f api
docker-compose logs -f headless
```

### Local Binary Deployment

```bash
# Start Redis
docker run -d --name redis -p 6379:6379 redis:7-alpine

# Start headless service
./target/release/riptide-headless &

# Start API service
./target/release/riptide-api --config config/application/riptide.yml --bind 0.0.0.0:8080
```

## Production Deployment

### Docker Production Setup

**Production Docker Compose**:
```yaml
# docker-compose.prod.yml
version: '3.8'

services:
  redis:
    image: redis:7-alpine
    restart: always
    volumes:
      - redis-data:/data
    deploy:
      resources:
        limits:
          memory: 512M
          cpus: '0.5'

  api:
    build:
      context: .
      dockerfile: infra/docker/Dockerfile.api
    restart: always
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=warn
      - REDIS_URL=redis://redis:6379/0
      - HEADLESS_URL=http://headless:9123
      - SERPER_API_KEY=${SERPER_API_KEY}
    volumes:
      - artifacts-data:/data/artifacts
      - ./configs:/app/configs:ro
    depends_on:
      - redis
      - headless
    deploy:
      resources:
        limits:
          memory: 1G
          cpus: '1'

  headless:
    build:
      context: .
      dockerfile: infra/docker/Dockerfile.headless
    restart: always
    environment:
      - RUST_LOG=warn
    deploy:
      resources:
        limits:
          memory: 2G
          cpus: '2'

  nginx:
    image: nginx:alpine
    restart: always
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./infra/nginx/nginx.conf:/etc/nginx/nginx.conf:ro
      - ./infra/ssl:/etc/ssl:ro
    depends_on:
      - api

volumes:
  redis-data:
  artifacts-data:
```

**Nginx Configuration**:
```nginx
# infra/nginx/nginx.conf
upstream riptide_api {
    server api:8080;
}

server {
    listen 80;
    server_name your-domain.com;

    location / {
        proxy_pass http://riptide_api;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Timeouts for long-running crawl requests
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 300s;
    }
}
```

### Kubernetes Deployment

**Namespace and ConfigMap**:
```yaml
# k8s/namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: riptide
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: riptide-config
  namespace: riptide
data:
  riptide.yml: |
    crawl:
      concurrency: 32
      timeout_ms: 15000
      cache: read_through
    redis:
      url: "redis://redis-service:6379/0"
    extraction:
      wasm_module_path: "/app/extractor.wasm"
    logging:
      level: "warn"
```

**Secrets**:
```yaml
# k8s/secrets.yaml
apiVersion: v1
kind: Secret
metadata:
  name: riptide-secrets
  namespace: riptide
type: Opaque
data:
  serper-api-key: <base64-encoded-api-key>
```

**Redis Deployment**:
```yaml
# k8s/redis.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: redis
  namespace: riptide
spec:
  replicas: 1
  selector:
    matchLabels:
      app: redis
  template:
    metadata:
      labels:
        app: redis
    spec:
      containers:
      - name: redis
        image: redis:7-alpine
        ports:
        - containerPort: 6379
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        volumeMounts:
        - name: redis-data
          mountPath: /data
      volumes:
      - name: redis-data
        persistentVolumeClaim:
          claimName: redis-pvc
---
apiVersion: v1
kind: Service
metadata:
  name: redis-service
  namespace: riptide
spec:
  selector:
    app: redis
  ports:
  - port: 6379
    targetPort: 6379
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: redis-pvc
  namespace: riptide
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 10Gi
```

**Headless Service Deployment**:
```yaml
# k8s/headless.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: headless
  namespace: riptide
spec:
  replicas: 2
  selector:
    matchLabels:
      app: headless
  template:
    metadata:
      labels:
        app: headless
    spec:
      containers:
      - name: headless
        image: riptide/headless:latest
        ports:
        - containerPort: 9123
        env:
        - name: RUST_LOG
          value: "warn"
        resources:
          requests:
            memory: "1Gi"
            cpu: "1"
          limits:
            memory: "2Gi"
            cpu: "2"
        livenessProbe:
          httpGet:
            path: /healthz
            port: 9123
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /healthz
            port: 9123
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: headless-service
  namespace: riptide
spec:
  selector:
    app: headless
  ports:
  - port: 9123
    targetPort: 9123
```

**API Service Deployment**:
```yaml
# k8s/api.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api
  namespace: riptide
spec:
  replicas: 3
  selector:
    matchLabels:
      app: api
  template:
    metadata:
      labels:
        app: api
    spec:
      containers:
      - name: api
        image: riptide/api:latest
        ports:
        - containerPort: 8080
        env:
        - name: RUST_LOG
          value: "warn"
        - name: REDIS_URL
          value: "redis://redis-service:6379/0"
        - name: HEADLESS_URL
          value: "http://headless-service:9123"
        - name: SERPER_API_KEY
          valueFrom:
            secretKeyRef:
              name: riptide-secrets
              key: serper-api-key
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1"
        volumeMounts:
        - name: config
          mountPath: /app/configs
          readOnly: true
        - name: artifacts
          mountPath: /data/artifacts
        livenessProbe:
          httpGet:
            path: /healthz
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /healthz
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
      volumes:
      - name: config
        configMap:
          name: riptide-config
      - name: artifacts
        persistentVolumeClaim:
          claimName: artifacts-pvc
---
apiVersion: v1
kind: Service
metadata:
  name: api-service
  namespace: riptide
spec:
  selector:
    app: api
  ports:
  - port: 8080
    targetPort: 8080
  type: LoadBalancer
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: artifacts-pvc
  namespace: riptide
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 100Gi
```

**Horizontal Pod Autoscaler**:
```yaml
# k8s/hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: api-hpa
  namespace: riptide
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: api
  minReplicas: 3
  maxReplicas: 10
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
```

### Cloud Provider Specific Deployments

**AWS EKS**:
```yaml
# k8s/aws/storage-class.yaml
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: gp3-encrypted
provisioner: ebs.csi.aws.com
parameters:
  type: gp3
  encrypted: "true"
```

**Google GKE**:
```yaml
# k8s/gcp/storage-class.yaml
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: ssd-encrypted
provisioner: kubernetes.io/gce-pd
parameters:
  type: pd-ssd
  zones: us-central1-a,us-central1-b
```

**Azure AKS**:
```yaml
# k8s/azure/storage-class.yaml
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: premium-encrypted
provisioner: disk.csi.azure.com
parameters:
  skuName: Premium_LRS
  encryption: EncryptionAtRestWithCustomerKey
```

## Monitoring and Observability

### Prometheus Monitoring

```yaml
# k8s/monitoring/prometheus-config.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: prometheus-config
  namespace: riptide
data:
  prometheus.yml: |
    global:
      scrape_interval: 15s

    scrape_configs:
    - job_name: 'riptide-api'
      static_configs:
      - targets: ['api-service:8080']
      metrics_path: /metrics

    - job_name: 'riptide-headless'
      static_configs:
      - targets: ['headless-service:9123']
      metrics_path: /metrics

    - job_name: 'redis'
      static_configs:
      - targets: ['redis-service:6379']
```

### Grafana Dashboard

Key metrics to monitor:
- Request latency (p50, p95, p99)
- Request rate (requests/second)
- Error rate (4xx, 5xx responses)
- Cache hit ratio
- Memory usage
- CPU utilization
- Headless browser pool utilization

### Logging with ELK Stack

```yaml
# k8s/logging/fluentd-config.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: fluentd-config
  namespace: riptide
data:
  fluent.conf: |
    <source>
      @type tail
      path /var/log/containers/api-*.log
      pos_file /var/log/fluentd-containers.log.pos
      format json
      tag riptide.api
    </source>

    <match riptide.api>
      @type elasticsearch
      host elasticsearch-service
      port 9200
      index_name riptide-logs
    </match>
```

## Security Considerations

### Network Policies

```yaml
# k8s/security/network-policy.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: riptide-network-policy
  namespace: riptide
spec:
  podSelector: {}
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: ingress-nginx
    ports:
    - protocol: TCP
      port: 8080
  egress:
  - to: []
    ports:
    - protocol: TCP
      port: 53
    - protocol: UDP
      port: 53
  - to:
    - podSelector:
        matchLabels:
          app: redis
    ports:
    - protocol: TCP
      port: 6379
```

### Pod Security Policies

```yaml
# k8s/security/pod-security-policy.yaml
apiVersion: policy/v1beta1
kind: PodSecurityPolicy
metadata:
  name: riptide-psp
  namespace: riptide
spec:
  privileged: false
  allowPrivilegeEscalation: false
  requiredDropCapabilities:
    - ALL
  volumes:
    - 'configMap'
    - 'emptyDir'
    - 'projected'
    - 'secret'
    - 'downwardAPI'
    - 'persistentVolumeClaim'
  runAsUser:
    rule: 'MustRunAsNonRoot'
  seLinux:
    rule: 'RunAsAny'
  fsGroup:
    rule: 'RunAsAny'
```

## Performance Tuning

### Resource Allocation

**API Service**:
- **Memory**: 512Mi request, 1Gi limit
- **CPU**: 500m request, 1 core limit
- **Replicas**: 3-10 based on load

**Headless Service**:
- **Memory**: 1Gi request, 2Gi limit
- **CPU**: 1 core request, 2 cores limit
- **Replicas**: 2-5 based on dynamic content usage

**Redis**:
- **Memory**: 256Mi request, 512Mi limit
- **CPU**: 250m request, 500m limit
- **Storage**: 10Gi for development, 100Gi+ for production

### Configuration Tuning

```yaml
# Production configuration optimizations
crawl:
  concurrency: 32              # Higher concurrency for production
  timeout_ms: 15000            # Shorter timeout for efficiency
  cache: read_through          # Optimal cache strategy

extraction:
  token_chunk_max: 1500        # Larger chunks for better throughput

dynamic:
  scroll:
    delay_ms: 100              # Faster scrolling
```

## Backup and Disaster Recovery

### Redis Backup

```bash
# Automated Redis backup script
#!/bin/bash
DATE=$(date +%Y%m%d_%H%M%S)
kubectl exec -n riptide redis-pod -- redis-cli BGSAVE
kubectl cp riptide/redis-pod:/data/dump.rdb ./backups/redis_$DATE.rdb
```

### Configuration Backup

```bash
# Backup all configurations
kubectl get configmap -n riptide -o yaml > configmap-backup.yaml
kubectl get secret -n riptide -o yaml > secret-backup.yaml
```

### Artifacts Backup

```bash
# Sync artifacts to cloud storage
kubectl exec -n riptide api-pod -- \
  aws s3 sync /data/artifacts s3://riptide-artifacts-backup/$(date +%Y%m%d)/
```

## Troubleshooting

### Common Issues

**Service Discovery Problems**:
```bash
# Check service endpoints
kubectl get endpoints -n riptide
kubectl describe service api-service -n riptide
```

**Resource Constraints**:
```bash
# Check resource usage
kubectl top pods -n riptide
kubectl describe nodes
```

**Configuration Issues**:
```bash
# Verify configuration
kubectl exec -n riptide api-pod -- cat /app/config/application/riptide.yml
kubectl get configmap riptide-config -o yaml
```

**Network Connectivity**:
```bash
# Test internal connectivity
kubectl exec -n riptide api-pod -- curl redis-service:6379
kubectl exec -n riptide api-pod -- curl headless-service:9123/healthz
```

### Performance Debugging

**Enable Debug Logging**:
```yaml
env:
- name: RUST_LOG
  value: "debug,riptide_core=trace"
```

**Monitor Resource Usage**:
```bash
# Real-time monitoring
kubectl top pods -n riptide --watch
```

**Analyze Slow Requests**:
```bash
# Check application logs
kubectl logs -n riptide -l app=api --since=1h | grep "slow_request"
```

This deployment guide provides practical instructions for running RipTide based on the actual system architecture and tested configurations.