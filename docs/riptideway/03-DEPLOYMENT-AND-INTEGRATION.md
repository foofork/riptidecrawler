# RipTide Deployment & Integration Guide

**Comprehensive guide for deploying and integrating RipTide**

---

## Table of Contents
- [Deployment Options](#deployment-options)
- [Configuration](#configuration)
- [Integration Patterns](#integration-patterns)
- [Client SDKs](#client-sdks)
- [Performance Tuning](#performance-tuning)
- [Monitoring Setup](#monitoring-setup)
- [Security & Authentication](#security--authentication)

---

## Deployment Options

### 1. Docker (Recommended for Quick Start)

**Pre-built Image**:
```bash
# Pull official image
docker pull riptide/api:latest

# Run with minimal configuration
docker run -d \
  -p 8080:8080 \
  --name riptide \
  -e REDIS_URL=redis://host.docker.internal:6379 \
  riptide/api:latest

# Access API
curl http://localhost:8080/healthz
```

**Docker Compose (Full Stack)**:
```yaml
version: '3.8'

services:
  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis-data:/data
    command: redis-server --maxmemory 2gb --maxmemory-policy allkeys-lru

  riptide-api:
    image: riptide/api:latest
    ports:
      - "8080:8080"
    environment:
      - REDIS_URL=redis://redis:6379
      - WASM_PATH=/app/riptide.wasm
      - MAX_CONCURRENCY=10
      - CACHE_TTL=3600
      - ENHANCED_PIPELINE_ENABLE=true
      # Optional: Headless service
      - HEADLESS_URL=http://headless:3000
      # Optional: Telemetry
      - OTEL_ENDPOINT=http://jaeger:4317
    depends_on:
      - redis
    volumes:
      - wasm-cache:/app/wasm-cache

  # Optional: Headless browser service
  headless:
    image: chromedp/headless-shell:latest
    ports:
      - "3000:9222"
    shm_size: 2gb

  # Optional: Monitoring stack
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
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana-data:/var/lib/grafana

volumes:
  redis-data:
  wasm-cache:
  prometheus-data:
  grafana-data:
```

**Start Services**:
```bash
docker-compose up -d

# Verify health
curl http://localhost:8080/healthz
curl http://localhost:8080/api/health/detailed

# View logs
docker-compose logs -f riptide-api
```

---

### 2. Kubernetes (Production Deployment)

**Deployment Manifest**:
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: riptide-api
  namespace: riptide
spec:
  replicas: 3
  selector:
    matchLabels:
      app: riptide-api
  template:
    metadata:
      labels:
        app: riptide-api
    spec:
      containers:
      - name: riptide-api
        image: riptide/api:latest
        ports:
        - containerPort: 8080
          name: http
        env:
        - name: REDIS_URL
          value: "redis://riptide-redis:6379"
        - name: WASM_PATH
          value: "/app/riptide.wasm"
        - name: MAX_CONCURRENCY
          value: "20"
        - name: RUST_LOG
          value: "info,riptide_api=debug"
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2000m"
        livenessProbe:
          httpGet:
            path: /healthz
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /healthz
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
        volumeMounts:
        - name: wasm-cache
          mountPath: /app/wasm-cache
      volumes:
      - name: wasm-cache
        emptyDir: {}
```

**Service Manifest**:
```yaml
apiVersion: v1
kind: Service
metadata:
  name: riptide-api
  namespace: riptide
spec:
  selector:
    app: riptide-api
  ports:
  - name: http
    port: 80
    targetPort: 8080
  type: ClusterIP
```

**Ingress (with TLS)**:
```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: riptide-api
  namespace: riptide
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/rate-limit: "100"
    nginx.ingress.kubernetes.io/proxy-body-size: "50m"
spec:
  ingressClassName: nginx
  tls:
  - hosts:
    - api.riptide.example.com
    secretName: riptide-tls
  rules:
  - host: api.riptide.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: riptide-api
            port:
              number: 80
```

**HorizontalPodAutoscaler**:
```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: riptide-api
  namespace: riptide
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: riptide-api
  minReplicas: 3
  maxReplicas: 20
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

---

### 3. API Gateway Integration (Kong)

**Docker Compose with Kong**:
```yaml
version: '3.8'

services:
  kong-database:
    image: postgres:13
    environment:
      POSTGRES_DB: kong
      POSTGRES_USER: kong
      POSTGRES_PASSWORD: kong

  kong-migrations:
    image: kong:3.4
    command: kong migrations bootstrap
    environment:
      KONG_DATABASE: postgres
      KONG_PG_HOST: kong-database
      KONG_PG_PASSWORD: kong
    depends_on:
      - kong-database

  kong:
    image: kong:3.4
    ports:
      - "8000:8000"  # Proxy
      - "8001:8001"  # Admin API
      - "8443:8443"  # Proxy SSL
      - "8444:8444"  # Admin SSL
    environment:
      KONG_DATABASE: postgres
      KONG_PG_HOST: kong-database
      KONG_PG_PASSWORD: kong
      KONG_PROXY_ACCESS_LOG: /dev/stdout
      KONG_ADMIN_ACCESS_LOG: /dev/stdout
      KONG_PROXY_ERROR_LOG: /dev/stderr
      KONG_ADMIN_ERROR_LOG: /dev/stderr
      KONG_ADMIN_LISTEN: 0.0.0.0:8001
    depends_on:
      - kong-database
      - kong-migrations

  riptide-api:
    image: riptide/api:latest
    environment:
      - REDIS_URL=redis://redis:6379
    depends_on:
      - redis
```

**Kong Configuration**:
```bash
# Create service
curl -X POST http://localhost:8001/services \
  --data "name=riptide-api" \
  --data "url=http://riptide-api:8080"

# Create route
curl -X POST http://localhost:8001/services/riptide-api/routes \
  --data "paths[]=/api" \
  --data "strip_path=false"

# Add rate limiting
curl -X POST http://localhost:8001/services/riptide-api/plugins \
  --data "name=rate-limiting" \
  --data "config.minute=100" \
  --data "config.policy=local"

# Add API key authentication
curl -X POST http://localhost:8001/services/riptide-api/plugins \
  --data "name=key-auth"

# Create consumer
curl -X POST http://localhost:8001/consumers \
  --data "username=client-app"

# Create API key
curl -X POST http://localhost:8001/consumers/client-app/key-auth \
  --data "key=YOUR_API_KEY"

# Add response caching
curl -X POST http://localhost:8001/services/riptide-api/plugins \
  --data "name=proxy-cache" \
  --data "config.response_code=200" \
  --data "config.request_method=GET" \
  --data "config.content_type=application/json" \
  --data "config.cache_ttl=300"

# Add request size limiting
curl -X POST http://localhost:8001/services/riptide-api/plugins \
  --data "name=request-size-limiting" \
  --data "config.allowed_payload_size=50"

# Add CORS
curl -X POST http://localhost:8001/services/riptide-api/plugins \
  --data "name=cors" \
  --data "config.origins=*" \
  --data "config.methods=GET,POST,PUT,DELETE" \
  --data "config.credentials=true"
```

**Test API Gateway**:
```bash
# Request with API key
curl http://localhost:8000/api/healthz \
  -H "apikey: YOUR_API_KEY"

# Crawl request
curl -X POST http://localhost:8000/api/crawl \
  -H "apikey: YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "urls": ["https://example.com"],
    "options": {"concurrency": 1}
  }'
```

---

## Configuration

### Environment Variables

**Core Settings**:
```bash
# Redis Configuration
REDIS_URL=redis://localhost:6379          # Redis connection string
REDIS_PASSWORD=your_password              # Redis password (if required)
REDIS_DB=0                                # Redis database number

# WASM Configuration
WASM_PATH=/path/to/riptide.wasm          # WASM module path
WASM_CACHE_DIR=/tmp/wasm-cache           # AOT cache directory
WASM_MAX_MEMORY_MB=128                    # Max WASM memory (MB)
WASM_TIMEOUT_SEC=30                       # Extraction timeout (seconds)

# Pipeline Configuration
MAX_CONCURRENCY=10                        # Max concurrent requests
CACHE_TTL=3600                           # Cache TTL (seconds)
GATE_HI_THRESHOLD=0.8                    # Gate high threshold
GATE_LO_THRESHOLD=0.3                    # Gate low threshold

# Enhanced Pipeline
ENHANCED_PIPELINE_ENABLE=true            # Enable dual-path pipeline
ENHANCED_PIPELINE_METRICS=true           # Collect phase metrics

# Optional: Headless Browser
HEADLESS_URL=http://localhost:3000       # Headless service URL
HEADLESS_TIMEOUT=30                      # Browser timeout (seconds)

# Optional: Telemetry
OTEL_ENDPOINT=http://localhost:4317      # OpenTelemetry collector
OTEL_SERVICE_NAME=riptide-api            # Service name
RUST_LOG=info,riptide_api=debug          # Log level

# Server Configuration
BIND_ADDRESS=0.0.0.0:8080                # Server bind address
```

**LLM Provider Configuration**:
```bash
# OpenAI
OPENAI_API_KEY=sk-...
OPENAI_MODEL=gpt-4

# Anthropic
ANTHROPIC_API_KEY=sk-ant-...
ANTHROPIC_MODEL=claude-2

# Azure OpenAI
AZURE_OPENAI_ENDPOINT=https://your-resource.openai.azure.com
AZURE_OPENAI_API_KEY=...
AZURE_OPENAI_DEPLOYMENT=your-deployment

# Local Models
OLLAMA_URL=http://localhost:11434
OLLAMA_MODEL=llama2
```

**Search Provider Configuration**:
```bash
# Serper (Google Search)
SERPER_API_KEY=your_api_key
SEARCH_PROVIDER=serper

# Or use URL parsing only
SEARCH_PROVIDER=none
```

---

## Integration Patterns

### 1. Synchronous API Calls

**Simple Crawl**:
```python
import requests

def crawl_url(url):
    response = requests.post(
        'http://localhost:8080/crawl',
        json={
            'urls': [url],
            'options': {
                'concurrency': 1,
                'cache_mode': 'read_write'
            }
        }
    )
    return response.json()

# Example
result = crawl_url('https://example.com')
print(result['results'][0]['document']['text'])
```

**Batch Crawl with Error Handling**:
```python
def batch_crawl(urls, max_retries=3):
    results = []
    for url in urls:
        for attempt in range(max_retries):
            try:
                result = crawl_url(url)
                results.append(result)
                break
            except requests.RequestException as e:
                if attempt == max_retries - 1:
                    results.append({'error': str(e), 'url': url})
                time.sleep(2 ** attempt)  # Exponential backoff
    return results
```

---

### 2. Asynchronous Processing (Workers)

**Submit Job**:
```python
def submit_crawl_job(urls, priority='normal'):
    response = requests.post(
        'http://localhost:8080/workers/jobs',
        json={
            'job_type': 'batch_crawl',
            'urls': urls,
            'priority': priority,
            'retry_config': {
                'max_retries': 3,
                'base_delay_ms': 1000
            }
        }
    )
    return response.json()['job_id']

# Submit job
job_id = submit_crawl_job(['https://example.com', 'https://example.org'])
```

**Poll for Completion**:
```python
import time

def wait_for_job(job_id, timeout=300):
    start_time = time.time()
    while time.time() - start_time < timeout:
        response = requests.get(f'http://localhost:8080/workers/jobs/{job_id}')
        status = response.json()

        if status['status'] == 'completed':
            # Get results
            result_response = requests.get(
                f'http://localhost:8080/workers/jobs/{job_id}/result'
            )
            return result_response.json()
        elif status['status'] == 'failed':
            raise Exception(f"Job failed: {status.get('error')}")

        time.sleep(5)  # Poll every 5 seconds

    raise TimeoutError(f"Job {job_id} did not complete within {timeout}s")

# Wait for results
results = wait_for_job(job_id)
```

---

### 3. Real-Time Streaming

**NDJSON Streaming**:
```python
import requests
import json

def stream_crawl(urls):
    response = requests.post(
        'http://localhost:8080/crawl/stream',
        json={'urls': urls, 'options': {'concurrency': 5}},
        stream=True
    )

    for line in response.iter_lines():
        if line:
            data = json.loads(line)
            print(f"Status: {data['status']}, URL: {data.get('url')}")

            if data['status'] == 'success':
                yield data['document']

# Process stream
for document in stream_crawl(['https://example.com']):
    print(document['title'])
```

**Server-Sent Events (SSE)**:
```javascript
const eventSource = new EventSource('/crawl/sse');

eventSource.addEventListener('progress', (event) => {
    const data = JSON.parse(event.data);
    console.log('Progress:', data);
});

eventSource.addEventListener('result', (event) => {
    const data = JSON.parse(event.data);
    console.log('Result:', data.document);
});

eventSource.addEventListener('complete', (event) => {
    console.log('Crawl complete');
    eventSource.close();
});
```

**WebSocket**:
```javascript
const ws = new WebSocket('ws://localhost:8080/crawl/ws');

ws.onopen = () => {
    ws.send(JSON.stringify({
        action: 'crawl',
        urls: ['https://example.com']
    }));
};

ws.onmessage = (event) => {
    const data = JSON.parse(event.data);

    switch(data.type) {
        case 'progress':
            console.log('Progress:', data);
            break;
        case 'result':
            console.log('Result:', data.document);
            break;
        case 'complete':
            ws.close();
            break;
    }
};
```

---

### 4. Scheduled Jobs

**Create Scheduled Crawl**:
```python
def create_scheduled_crawl(urls, cron_expression):
    response = requests.post(
        'http://localhost:8080/workers/schedule',
        json={
            'job_type': 'batch_crawl',
            'cron_expression': cron_expression,  # "0 * * * *" = hourly
            'urls': urls,
            'priority': 'normal'
        }
    )
    return response.json()['schedule_id']

# Schedule hourly crawl
schedule_id = create_scheduled_crawl(
    ['https://news-site.com/latest'],
    '0 * * * *'  # Every hour
)

# List scheduled jobs
response = requests.get('http://localhost:8080/workers/schedule')
schedules = response.json()

# Delete schedule
requests.delete(f'http://localhost:8080/workers/schedule/{schedule_id}')
```

---

## Client SDKs

### Python SDK Example

```python
# riptide_client.py
import requests
from typing import List, Dict, Optional
from dataclasses import dataclass

@dataclass
class CrawlOptions:
    concurrency: int = 1
    cache_mode: str = 'read_write'
    use_spider: bool = False
    chunking_config: Optional[Dict] = None

class RiptideClient:
    def __init__(self, base_url: str = 'http://localhost:8080', api_key: Optional[str] = None):
        self.base_url = base_url
        self.session = requests.Session()
        if api_key:
            self.session.headers['apikey'] = api_key

    def crawl(self, urls: List[str], options: Optional[CrawlOptions] = None) -> Dict:
        """Synchronous batch crawl"""
        opts = options or CrawlOptions()
        response = self.session.post(
            f'{self.base_url}/crawl',
            json={
                'urls': urls,
                'options': opts.__dict__
            }
        )
        response.raise_for_status()
        return response.json()

    def stream_crawl(self, urls: List[str], options: Optional[CrawlOptions] = None):
        """NDJSON streaming crawl"""
        opts = options or CrawlOptions()
        response = self.session.post(
            f'{self.base_url}/crawl/stream',
            json={'urls': urls, 'options': opts.__dict__},
            stream=True
        )
        response.raise_for_status()

        for line in response.iter_lines():
            if line:
                yield json.loads(line)

    def submit_job(self, urls: List[str], priority: str = 'normal') -> str:
        """Submit async job"""
        response = self.session.post(
            f'{self.base_url}/workers/jobs',
            json={
                'job_type': 'batch_crawl',
                'urls': urls,
                'priority': priority
            }
        )
        response.raise_for_status()
        return response.json()['job_id']

    def get_job_status(self, job_id: str) -> Dict:
        """Get job status"""
        response = self.session.get(f'{self.base_url}/workers/jobs/{job_id}')
        response.raise_for_status()
        return response.json()

    def get_job_result(self, job_id: str) -> Dict:
        """Get job result"""
        response = self.session.get(f'{self.base_url}/workers/jobs/{job_id}/result')
        response.raise_for_status()
        return response.json()

    def process_pdf(self, pdf_data: bytes, filename: Optional[str] = None) -> Dict:
        """Process PDF"""
        import base64
        response = self.session.post(
            f'{self.base_url}/pdf/process',
            json={
                'pdf_data': base64.b64encode(pdf_data).decode('utf-8'),
                'filename': filename
            }
        )
        response.raise_for_status()
        return response.json()

    def extract_tables(self, html: str) -> List[Dict]:
        """Extract tables from HTML"""
        response = self.session.post(
            f'{self.base_url}/api/v1/tables/extract',
            json={'html_content': html}
        )
        response.raise_for_status()
        return response.json()['tables']

# Usage
client = RiptideClient()

# Synchronous crawl
result = client.crawl(['https://example.com'])
print(result['results'][0]['document']['title'])

# Streaming crawl
for item in client.stream_crawl(['https://example.com']):
    if item['status'] == 'success':
        print(item['document']['title'])

# Async job
job_id = client.submit_job(['https://example.com'])
while True:
    status = client.get_job_status(job_id)
    if status['status'] == 'completed':
        result = client.get_job_result(job_id)
        break
    time.sleep(5)
```

---

## Performance Tuning

### Redis Optimization

```bash
# Memory optimization
maxmemory 4gb
maxmemory-policy allkeys-lru

# Persistence (balance durability vs performance)
save 900 1      # Save if 1 key changed in 15 minutes
save 300 10     # Save if 10 keys changed in 5 minutes
save 60 10000   # Save if 10000 keys changed in 1 minute

# Network
tcp-backlog 511
timeout 300

# Performance
hash-max-ziplist-entries 512
hash-max-ziplist-value 64
```

### RipTide Configuration

**High-Throughput Setup**:
```bash
# Increase concurrency
MAX_CONCURRENCY=50

# Optimize caching
CACHE_TTL=7200  # 2 hours

# Connection pooling
HTTP_MAX_CONNECTIONS=500
REDIS_MAX_CONNECTIONS=100

# WASM optimization
WASM_AOT_CACHE_ENABLE=true
WASM_MAX_MEMORY_MB=256
```

**Low-Latency Setup**:
```bash
# Prefer fast path
GATE_HI_THRESHOLD=0.9
GATE_LO_THRESHOLD=0.5

# Shorter cache TTL
CACHE_TTL=300  # 5 minutes

# Aggressive WASM caching
WASM_AOT_CACHE_ENABLE=true
WASM_PRECOMPILE=true
```

---

## Monitoring Setup

### Prometheus Configuration

```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'riptide-api'
    static_configs:
      - targets: ['riptide-api:8080']
    metrics_path: '/metrics'
```

### Grafana Dashboards

**Key Metrics to Monitor**:
```
Request Metrics:
  - http_requests_total (rate, errors)
  - http_request_duration_seconds (p50, p95, p99)
  - Success rate: rate(http_requests_total{status="200"}[5m])

Cache Metrics:
  - cache_hit_rate
  - cache_size_bytes
  - cache_evictions_total

Pipeline Metrics:
  - fetch_phase_duration_seconds
  - wasm_phase_duration_seconds
  - Quality score distribution

Error Metrics:
  - errors_total (by type)
  - Error rate: rate(errors_total[5m])

Resource Metrics:
  - System CPU usage
  - System memory usage
  - Redis connection pool
```

---

## Security & Authentication

### API Key Authentication (via Kong)

Already covered in Kong integration section above.

### Rate Limiting

**Per-User Rate Limiting**:
```python
# Track usage per API key
# Kong automatically handles this
# Configure in Kong:
curl -X POST http://localhost:8001/plugins \
  --data "name=rate-limiting" \
  --data "config.minute=100" \
  --data "config.hour=5000" \
  --data "config.policy=redis" \
  --data "config.redis_host=redis" \
  --data "config.redis_port=6379"
```

### Input Validation

**Client-Side Validation**:
```python
from urllib.parse import urlparse

def validate_url(url: str) -> bool:
    try:
        result = urlparse(url)
        return all([result.scheme, result.netloc])
    except:
        return False

# Usage
if validate_url(url):
    client.crawl([url])
```

---

## Conclusion

RipTide provides flexible deployment options:

**Quick Start**: Docker (single command)
**Production**: Kubernetes (auto-scaling, HA)
**Enterprise**: API Gateway (Kong, rate limiting, auth)

**Integration Patterns**:
- ✅ Synchronous API calls
- ✅ Asynchronous workers
- ✅ Real-time streaming
- ✅ Scheduled jobs

**Monitoring**: Prometheus + Grafana
**Security**: API keys, rate limiting, input validation

Deploy with confidence using the provided examples and configurations.
