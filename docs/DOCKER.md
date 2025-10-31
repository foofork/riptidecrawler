# Docker Deployment Guide

RipTide supports **two Docker image variants** via build arguments, allowing you to choose between native-only (fast, small) or WASM-enabled (sandboxed) deployments.

---

## Image Variants

### Native Extraction (Default, Recommended)

**Optimized for performance and resource efficiency:**

**Image Characteristics:**
- **Tag**: `riptide-api:native`
- **Binary Size**: ~45 MB
- **Image Size**: ~200 MB
- **Build Time**: ~5 minutes
- **Performance**: 2-5ms per page
- **Memory**: ~500 MB runtime
- **Extractor**: Pure Rust native parser

**Use Case:**
- ✅ 99% of production deployments
- ✅ Trusted HTML sources
- ✅ High-throughput applications
- ✅ Resource-constrained environments
- ✅ Fast CI/CD pipelines

---

### WASM Extraction (Specialized)

**Optimized for security and isolation:**

**Image Characteristics:**
- **Tag**: `riptide-api:wasm`
- **Binary Size**: ~95 MB (+110%)
- **Image Size**: ~350 MB (+75%)
- **Build Time**: ~8 minutes (+60%)
- **Performance**: 10-20ms per page (4x slower)
- **Memory**: ~800 MB runtime (+60%)
- **Extractor**: WASM with sandboxing

**Use Case:**
- ✅ Untrusted HTML sources
- ✅ Security-critical applications
- ✅ Compliance requirements (sandboxing)
- ✅ Plugin architectures
- ❌ Not recommended for general use

---

## Quick Start

### Option 1: Native Image (Default)

**Build:**
```bash
docker build \
  --build-arg ENABLE_WASM=false \
  -t riptide-api:native \
  -f infra/docker/Dockerfile.api \
  .
```

**Run:**
```bash
docker run -d \
  --name riptide-api \
  -p 8080:8080 \
  -e REDIS_URL=redis://redis:6379 \
  riptide-api:native
```

**No WASM_EXTRACTOR_PATH needed!**

---

### Option 2: WASM Image (Specialized)

**Build:**
```bash
docker build \
  --build-arg ENABLE_WASM=true \
  -t riptide-api:wasm \
  -f infra/docker/Dockerfile.api \
  .
```

**Run:**
```bash
docker run -d \
  --name riptide-api-wasm \
  -p 8080:8080 \
  -e REDIS_URL=redis://redis:6379 \
  -e WASM_EXTRACTOR_PATH=/opt/riptide/extractor/extractor.wasm \
  riptide-api:wasm
```

**WASM_EXTRACTOR_PATH automatically configured in image.**

---

## Docker Compose

### Native Deployment (Recommended)

**File: `docker-compose.yml`**

```yaml
version: '3.8'

services:
  # Native extraction (fast, small)
  riptide-api:
    build:
      context: .
      dockerfile: infra/docker/Dockerfile.api
      args:
        ENABLE_WASM: "false"
    image: riptide-api:native
    container_name: riptide-api-native
    restart: unless-stopped

    ports:
      - "8080:8080"

    environment:
      - REDIS_URL=redis://redis:6379
      - RUST_LOG=info,riptide_api=debug
      # No WASM_EXTRACTOR_PATH needed

    depends_on:
      redis:
        condition: service_healthy

    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/healthz"]
      interval: 30s
      timeout: 5s
      retries: 3

    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 1G
        reservations:
          cpus: '0.5'
          memory: 512M

  redis:
    image: redis:7-alpine
    container_name: riptide-redis
    restart: unless-stopped

    ports:
      - "6379:6379"

    volumes:
      - redis-data:/data

    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5

volumes:
  redis-data:
```

**Start:**
```bash
docker-compose up -d
docker-compose logs -f riptide-api
```

---

### WASM Deployment (Specialized)

**File: `docker-compose.wasm.yml`**

```yaml
version: '3.8'

services:
  # WASM extraction (sandboxed, slower)
  riptide-api-wasm:
    build:
      context: .
      dockerfile: infra/docker/Dockerfile.api
      args:
        ENABLE_WASM: "true"
    image: riptide-api:wasm
    container_name: riptide-api-wasm
    restart: unless-stopped

    ports:
      - "8081:8080"  # Different port to avoid conflict

    environment:
      - REDIS_URL=redis://redis:6379
      - RUST_LOG=info,riptide_api=debug
      - WASM_EXTRACTOR_PATH=/opt/riptide/extractor/extractor.wasm

    depends_on:
      redis:
        condition: service_healthy

    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/healthz"]
      interval: 30s
      timeout: 5s
      retries: 3

    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 2G  # More memory for WASM
        reservations:
          cpus: '0.5'
          memory: 768M

  redis:
    image: redis:7-alpine
    # ... same as native deployment
```

**Start:**
```bash
docker-compose -f docker-compose.wasm.yml up -d
```

---

### Side-by-Side Comparison

Run both variants simultaneously for benchmarking:

**File: `docker-compose.both.yml`**

```yaml
version: '3.8'

services:
  # Native variant on port 8080
  riptide-api-native:
    build:
      context: .
      dockerfile: infra/docker/Dockerfile.api
      args:
        ENABLE_WASM: "false"
    image: riptide-api:native
    container_name: riptide-native
    ports:
      - "8080:8080"
    environment:
      - REDIS_URL=redis://redis:6379
    depends_on:
      - redis

  # WASM variant on port 8081
  riptide-api-wasm:
    build:
      context: .
      dockerfile: infra/docker/Dockerfile.api
      args:
        ENABLE_WASM: "true"
    image: riptide-api:wasm
    container_name: riptide-wasm
    ports:
      - "8081:8080"
    environment:
      - REDIS_URL=redis://redis:6379
      - WASM_EXTRACTOR_PATH=/opt/riptide/extractor/extractor.wasm
    depends_on:
      - redis

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis-data:/data

volumes:
  redis-data:
```

**Benchmark:**
```bash
docker-compose -f docker-compose.both.yml up -d

# Test native (port 8080)
time curl -X POST http://localhost:8080/crawl \
  -H "Content-Type: application/json" \
  -d '{"urls": ["https://example.com"]}'

# Test WASM (port 8081)
time curl -X POST http://localhost:8081/crawl \
  -H "Content-Type: application/json" \
  -d '{"urls": ["https://example.com"]}'
```

---

## Build Scripts

### Automated Build Script

**File: `scripts/docker-build.sh`**

```bash
#!/bin/bash
set -e

MODE="${1:-native}"

case "$MODE" in
  native)
    echo "🏗️  Building native-only image (faster, smaller)..."
    docker build \
      --build-arg ENABLE_WASM=false \
      -t riptide-api:native \
      -f infra/docker/Dockerfile.api \
      .
    echo "✅ Built: riptide-api:native"
    docker images riptide-api:native --format "Size: {{.Size}}"
    ;;

  wasm)
    echo "🏗️  Building WASM-enabled image (slower, larger)..."
    docker build \
      --build-arg ENABLE_WASM=true \
      -t riptide-api:wasm \
      -f infra/docker/Dockerfile.api \
      .
    echo "✅ Built: riptide-api:wasm"
    docker images riptide-api:wasm --format "Size: {{.Size}}"
    ;;

  both)
    echo "🏗️  Building both images..."
    $0 native
    echo ""
    $0 wasm
    echo ""
    echo "📊 Image Size Comparison:"
    docker images riptide-api --format "table {{.Repository}}:{{.Tag}}\t{{.Size}}\t{{.CreatedAt}}"
    ;;

  *)
    echo "Usage: $0 {native|wasm|both}"
    echo ""
    echo "Examples:"
    echo "  $0 native    # Build native-only image (recommended)"
    echo "  $0 wasm      # Build WASM-enabled image"
    echo "  $0 both      # Build both for comparison"
    exit 1
    ;;
esac
```

**Usage:**
```bash
chmod +x scripts/docker-build.sh

# Build native image (recommended)
./scripts/docker-build.sh native

# Build WASM image
./scripts/docker-build.sh wasm

# Build both and compare sizes
./scripts/docker-build.sh both
```

---

## Image Size Comparison

### Actual Sizes (After Optimization)

| Variant | Binary | WASM File | Image | Total Disk |
|---------|--------|-----------|-------|------------|
| Native  | 45 MB  | N/A       | ~200 MB | ~200 MB |
| WASM    | 95 MB  | 2-5 MB    | ~350 MB | ~350 MB |

### Layer Breakdown

**Native Image Layers:**
```
Layer 1: Base (debian:trixie-slim)           ~80 MB
Layer 2: Runtime deps (ca-certs, libssl)     ~30 MB
Layer 3: Binary (riptide-api)                ~45 MB
Layer 4: Configs and scripts                 ~5 MB
Layer 5: Directory structure                 ~1 MB
-----------------------------------------------
Total:                                       ~200 MB
```

**WASM Image Layers:**
```
Layer 1: Base (debian:trixie-slim)           ~80 MB
Layer 2: Runtime deps (ca-certs, libssl)     ~30 MB
Layer 3: Binary (riptide-api with WASM)      ~95 MB
Layer 4: WASM module (extractor.wasm)        ~5 MB
Layer 5: Configs and scripts                 ~5 MB
Layer 6: Directory structure                 ~1 MB
-----------------------------------------------
Total:                                       ~350 MB
```

---

## Build Time Comparison

### Local Build

| Variant | First Build | Cached Build | CI Build |
|---------|-------------|--------------|----------|
| Native  | ~5 min      | ~2 min       | ~4 min   |
| WASM    | ~8 min      | ~3 min       | ~6 min   |

### CI/CD Optimization

**Recommended: Build native by default, WASM only on main branch**

```yaml
# .github/workflows/docker-build.yml
jobs:
  docker-native:
    name: Build Native Image
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Build native image
        run: docker build --build-arg ENABLE_WASM=false -t riptide-api:native .

  docker-wasm:
    name: Build WASM Image
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'  # Only on main branch
    steps:
      - uses: actions/checkout@v3

      - name: Build WASM image
        run: docker build --build-arg ENABLE_WASM=true -t riptide-api:wasm .
```

---

## Production Deployment

### Single-Variant Production

**Native (Recommended for most):**

```yaml
# docker-compose.prod.yml
version: '3.8'

services:
  riptide-api:
    image: riptide-api:native
    restart: always

    ports:
      - "8080:8080"

    environment:
      - REDIS_URL=redis://redis:6379
      - RUST_LOG=warn,riptide_api=info
      - RIPTIDE_API_KEY=${RIPTIDE_API_KEY}
      - REQUIRE_AUTH=true

    deploy:
      replicas: 3
      resources:
        limits:
          cpus: '2'
          memory: 1G
        reservations:
          cpus: '0.5'
          memory: 512M
      restart_policy:
        condition: on-failure
        delay: 5s
        max_attempts: 3

  redis:
    image: redis:7-alpine
    restart: always
    command: redis-server --appendonly yes --maxmemory 2gb
    volumes:
      - redis-data:/data
```

---

### Multi-Variant Production

Use native for general traffic, WASM for untrusted sources:

```yaml
version: '3.8'

services:
  # Main API - Native (high throughput)
  riptide-api-main:
    image: riptide-api:native
    restart: always
    ports:
      - "8080:8080"
    environment:
      - REDIS_URL=redis://redis:6379
    deploy:
      replicas: 5  # More replicas for main traffic

  # Security API - WASM (sandboxed)
  riptide-api-secure:
    image: riptide-api:wasm
    restart: always
    ports:
      - "8081:8080"
    environment:
      - REDIS_URL=redis://redis:6379
      - WASM_EXTRACTOR_PATH=/opt/riptide/extractor/extractor.wasm
    deploy:
      replicas: 2  # Fewer replicas for security-critical paths
```

---

## Health Checks

### Native Image Health

```bash
curl http://localhost:8080/healthz | jq
```

**Response:**
```json
{
  "status": "healthy",
  "version": "0.9.0",
  "extractor": {
    "type": "native",
    "wasm_available": false
  },
  "uptime_seconds": 3600
}
```

---

### WASM Image Health

```bash
curl http://localhost:8081/healthz | jq
```

**Response:**
```json
{
  "status": "healthy",
  "version": "0.9.0",
  "extractor": {
    "type": "wasm",
    "wasm_available": true,
    "wasm_file": "/opt/riptide/extractor/extractor.wasm"
  },
  "uptime_seconds": 3600
}
```

---

## Migration from Old Dockerfile

### Before (Always WASM)

**Old Dockerfile pattern:**
```dockerfile
# Always built WASM (even if not needed)
RUN rustup target add wasm32-wasip2
RUN cd wasm/riptide-extractor-wasm && \
    cargo build --profile ci --target wasm32-wasip2

# Always included WASM in image
COPY --from=builder /app/target/wasm32-wasip2/ci/*.wasm /opt/riptide/extractor/

# Always set WASM path
ENV WASM_EXTRACTOR_PATH=/opt/riptide/extractor/extractor.wasm
```

**Problems:**
- ❌ Slower builds (always compiles WASM)
- ❌ Larger images (always includes WASM)
- ❌ No choice for users
- ❌ Wasted resources for 99% of users

---

### After (Optional WASM)

**New Dockerfile pattern:**
```dockerfile
# ARG allows conditional WASM build
ARG ENABLE_WASM=false

# Only build WASM if requested
RUN if [ "$ENABLE_WASM" = "true" ]; then \
    rustup target add wasm32-wasip2 && \
    cd wasm/riptide-extractor-wasm && \
    cargo build --profile ci --target wasm32-wasip2; \
  fi

# Only copy WASM if built
RUN if [ "$ENABLE_WASM" = "true" ]; then \
    COPY --from=builder /app/target/wasm32-wasip2/ci/*.wasm \
      /opt/riptide/extractor/; \
  fi

# No default WASM path (uses native by default)
# Set only if WASM enabled
RUN if [ "$ENABLE_WASM" = "true" ]; then \
    echo "export WASM_EXTRACTOR_PATH=/opt/riptide/extractor/extractor.wasm" \
      >> /etc/environment; \
  fi
```

**Benefits:**
- ✅ Faster builds (native: 5min vs WASM: 8min)
- ✅ Smaller images (native: 200MB vs WASM: 350MB)
- ✅ User choice (build arg)
- ✅ Optimized for common case (native)

---

## Kubernetes Deployment

### Native Deployment

```yaml
# k8s/deployment-native.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: riptide-api-native
spec:
  replicas: 3
  selector:
    matchLabels:
      app: riptide-api
      variant: native
  template:
    metadata:
      labels:
        app: riptide-api
        variant: native
    spec:
      containers:
      - name: riptide-api
        image: riptide-api:native
        ports:
        - containerPort: 8080
        env:
        - name: REDIS_URL
          value: redis://redis-service:6379
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "2000m"
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
          initialDelaySeconds: 10
          periodSeconds: 5
```

---

### WASM Deployment

```yaml
# k8s/deployment-wasm.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: riptide-api-wasm
spec:
  replicas: 2  # Fewer replicas (slower, more resources)
  selector:
    matchLabels:
      app: riptide-api
      variant: wasm
  template:
    metadata:
      labels:
        app: riptide-api
        variant: wasm
    spec:
      containers:
      - name: riptide-api
        image: riptide-api:wasm
        ports:
        - containerPort: 8080
        env:
        - name: REDIS_URL
          value: redis://redis-service:6379
        - name: WASM_EXTRACTOR_PATH
          value: /opt/riptide/extractor/extractor.wasm
        resources:
          requests:
            memory: "768Mi"  # More memory
            cpu: "500m"
          limits:
            memory: "2Gi"  # More memory
            cpu: "2000m"
```

---

## Troubleshooting

### Build Issues

**Problem:** `ENABLE_WASM` not recognized
```
Solution: Ensure Docker version supports build args (Docker 20.10+)
Command: docker build --build-arg ENABLE_WASM=false ...
```

**Problem:** WASM build fails
```
Check: Is wasm32-wasip2 target installed in builder?
Solution: Builder image should run: rustup target add wasm32-wasip2
```

---

### Runtime Issues

**Problem:** Native image can't find extractor
```
This is expected. Native image doesn't need WASM_EXTRACTOR_PATH.
Solution: Remove WASM_EXTRACTOR_PATH environment variable.
```

**Problem:** WASM image says "native extractor" in logs
```
Check: Is WASM_EXTRACTOR_PATH set correctly?
Solution: export WASM_EXTRACTOR_PATH=/opt/riptide/extractor/extractor.wasm
```

---

### Performance Issues

**Problem:** Slow extraction in Docker
```
Check: Which image variant are you using?
Solution: Use native variant for 4x faster extraction.
Command: docker images | grep riptide-api
```

**Problem:** High memory usage
```
Check: Are you using WASM variant?
Solution: Switch to native variant for lower memory footprint.
```

---

## See Also

- **[Feature Flags](FEATURES.md)** - Native vs WASM comparison
- **[Production Deployment](01-guides/operations/PRODUCTION_DEPLOYMENT_CHECKLIST.md)** - Deployment checklist
- **[Architecture](04-architecture/ARCHITECTURE.md)** - System design
- **[Performance](performance-monitoring.md)** - Monitoring and optimization

---

**Built with 🐳 by the RipTide Team**

*Choose the right Docker image for your deployment: Native for speed, WASM for security.*
