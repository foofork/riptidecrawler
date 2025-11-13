# RipTide Quick Start Guide

Get RipTide running in under 5 minutes with pre-built binaries or Docker images. No 15-minute compilation required!

## 🚀 Three Ways to Get Started

Choose the method that works best for you:

1. **[Binary Download](#option-1-binary-download-fastest)** - Fastest, download and run
2. **[Docker Image](#option-2-docker-image-easiest)** - Easiest, pre-built containers
3. **[Source Compilation](#option-3-source-compilation-most-flexible)** - Most flexible, build from source

---

## Option 1: Binary Download (Fastest)

### Automatic Installation (Linux/macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/your-org/riptidecrawler/main/scripts/install.sh | bash
```

Or for WASM-enabled version:

```bash
curl -fsSL https://raw.githubusercontent.com/your-org/riptidecrawler/main/scripts/install.sh | bash -s -- --wasm
```

### Manual Installation

#### 1. Download Binary for Your Platform

**Linux (x86_64):**
```bash
# Native variant (recommended)
curl -L https://github.com/your-org/riptidecrawler/releases/latest/download/riptide-linux-x64-native.tar.gz -o riptide.tar.gz

# WASM variant (advanced features)
curl -L https://github.com/your-org/riptidecrawler/releases/latest/download/riptide-linux-x64-wasm.tar.gz -o riptide.tar.gz
```

**macOS (Apple Silicon):**
```bash
curl -L https://github.com/your-org/riptidecrawler/releases/latest/download/riptide-macos-arm64.tar.gz -o riptide.tar.gz
```

**macOS (Intel):**
```bash
curl -L https://github.com/your-org/riptidecrawler/releases/latest/download/riptide-macos-x64.tar.gz -o riptide.tar.gz
```

**Windows (PowerShell):**
```powershell
Invoke-WebRequest -Uri "https://github.com/your-org/riptidecrawler/releases/latest/download/riptide-windows-x64.zip" -OutFile "riptide.zip"
```

#### 2. Extract Archive

**Linux/macOS:**
```bash
tar xzf riptide.tar.gz
cd riptide-*
```

**Windows:**
```powershell
Expand-Archive riptide.zip -DestinationPath riptide
cd riptide
```

#### 3. Configure Environment

```bash
# Copy example configuration
cp .env.example .env

# Edit configuration (optional for testing)
nano .env
```

For quick testing, set minimal configuration:
```bash
export REQUIRE_AUTH=false
export LOG_LEVEL=info
```

#### 4. Run RipTide

```bash
# Start the API server
./riptide-api

# Or use the CLI
./riptide-cli --help
```

### Verify Installation

```bash
# Check version
./riptide-api --version

# Test health endpoint
curl http://localhost:8080/health
```

---

## Option 2: Docker Image (Easiest)

Pre-built images are available on GitHub Container Registry (GHCR).

### Quick Start with Docker

**Native variant (recommended):**
```bash
docker pull ghcr.io/your-org/riptidecrawler:latest

docker run -d \
  --name riptide-api \
  -p 8080:8080 \
  -e REQUIRE_AUTH=false \
  ghcr.io/your-org/riptidecrawler:latest
```

**WASM variant:**
```bash
docker pull ghcr.io/your-org/riptidecrawler:latest-wasm

docker run -d \
  --name riptide-api \
  -p 8080:8080 \
  -e REQUIRE_AUTH=false \
  ghcr.io/your-org/riptidecrawler:latest-wasm
```

### Docker Compose

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  riptide-api:
    image: ghcr.io/your-org/riptidecrawler:latest
    ports:
      - "8080:8080"
    environment:
      - REQUIRE_AUTH=false
      - RUST_LOG=info
      - MAX_CONCURRENT_REQUESTS=100
    volumes:
      - ./config:/opt/riptide/config
      - ./data:/opt/riptide/data
      - ./logs:/opt/riptide/logs
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  riptide-headless:
    image: ghcr.io/your-org/riptidecrawler-headless:latest
    environment:
      - RUST_LOG=info
    restart: unless-stopped
```

Start services:
```bash
docker-compose up -d
```

### Development with Docker

Mount your local code for development:

```bash
docker run -it --rm \
  -p 8080:8080 \
  -v $(pwd)/config:/opt/riptide/config \
  -v $(pwd)/data:/opt/riptide/data \
  ghcr.io/your-org/riptidecrawler:latest
```

---

## Option 3: Source Compilation (Most Flexible)

### Prerequisites

- Rust 1.75+ (install from https://rustup.rs/)
- pkg-config
- libssl-dev (Linux) or OpenSSL (macOS)

### Quick Build

```bash
# Clone repository
git clone https://github.com/your-org/riptidecrawler.git
cd riptidecrawler

# Fast build (native-parser only, ~8 minutes)
cargo build --release -p riptide-api --no-default-features --features native-parser

# Full build with WASM (~15 minutes)
cargo build --release -p riptide-api --features wasm-extractor
```

### Optimized Build (Production)

```bash
# Use release profile
RUSTFLAGS="-C target-cpu=native" cargo build --release

# Binaries will be in target/release/
./target/release/riptide-api --version
```

### Development Build

```bash
# Fast development builds
cargo build --profile fast-dev -p riptide-api

# Run with hot-reload
cargo watch -x 'run -p riptide-api'
```

---

## Testing Your Installation

### 1. Health Check

```bash
curl http://localhost:8080/health
```

Expected response:
```json
{
  "status": "healthy",
  "version": "0.9.0"
}
```

### 2. Simple Crawl

```bash
curl -X POST http://localhost:8080/api/v1/crawl \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://example.com",
    "max_depth": 1
  }'
```

### 3. CLI Test

```bash
# Using installed binary
riptide crawl https://example.com

# Using local build
./target/release/riptide-cli crawl https://example.com
```

---

## Configuration

### Minimal Configuration (.env)

For quick testing:

```bash
# Disable authentication for testing
REQUIRE_AUTH=false

# Logging
LOG_LEVEL=info
RUST_LOG=riptide=debug

# Performance
MAX_CONCURRENT_REQUESTS=100
REQUEST_TIMEOUT_SECS=30
```

### Production Configuration

```bash
# Security
REQUIRE_AUTH=true
AUTH_TOKEN=your-secure-token-here

# Redis (optional)
REDIS_URL=redis://localhost:6379

# Monitoring
ENABLE_METRICS=true
OTLP_ENDPOINT=http://localhost:4317

# Performance
MAX_CONCURRENT_REQUESTS=1000
CONNECTION_POOL_SIZE=50
```

### Configuration Files

RipTide uses YAML configuration files in `config/application/`:

```yaml
# config/application/riptide.yml
server:
  host: "0.0.0.0"
  port: 8080
  max_connections: 1000

crawler:
  max_depth: 5
  max_pages: 1000
  timeout: 30s
  respect_robots_txt: true

extraction:
  parser: "native"  # or "wasm"
  max_content_size: 10485760  # 10MB
```

---

## Performance Comparison

| Method | Installation Time | First Run | Disk Space |
|--------|------------------|-----------|------------|
| **Binary (Native)** | < 1 minute | Instant | ~50MB |
| **Binary (WASM)** | < 1 minute | Instant | ~80MB |
| **Docker Pull** | 2-5 minutes | Instant | ~200MB |
| **Source (Native)** | 8-12 minutes | Instant | ~2GB |
| **Source (Full)** | 15-20 minutes | Instant | ~3GB |

---

## Next Steps

### 1. Explore the API

Visit http://localhost:8080/swagger-ui for interactive API documentation.

### 2. Run Examples

```bash
# Crawl with depth limit
curl -X POST http://localhost:8080/api/v1/crawl \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://news.ycombinator.com",
    "max_depth": 2,
    "max_pages": 100
  }'

# Extract content
curl -X POST http://localhost:8080/api/v1/extract \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://example.com",
    "format": "markdown"
  }'
```

### 3. Monitor Performance

```bash
# Check metrics endpoint
curl http://localhost:8080/metrics

# View logs
tail -f logs/riptide.log

# Docker logs
docker logs -f riptide-api
```

### 4. Read Documentation

- [Architecture Guide](../04-architecture/OVERVIEW.md)
- [API Reference](../02-api-reference/README.md)
- [Configuration Guide](../01-guides/CONFIGURATION.md)
- [Deployment Guide](../06-deployment/README.md)

---

## Troubleshooting

### Binary Not Starting

**Permission denied:**
```bash
chmod +x riptide-api riptide-cli riptide-headless riptide-workers
```

**Library not found (Linux):**
```bash
# Install OpenSSL
sudo apt-get update && sudo apt-get install -y libssl3
```

**Library not found (macOS):**
```bash
brew install openssl
```

### Docker Issues

**Container exits immediately:**
```bash
# Check logs
docker logs riptide-api

# Run in foreground
docker run --rm -it -p 8080:8080 ghcr.io/your-org/riptidecrawler:latest
```

**Permission issues:**
```bash
# Ensure volumes have correct permissions
chmod -R 755 config data logs
```

### Port Already in Use

```bash
# Change port
docker run -p 8081:8080 ghcr.io/your-org/riptidecrawler:latest

# Or in .env
SERVER_PORT=8081
```

### Out of Memory

```bash
# Limit concurrent requests
export MAX_CONCURRENT_REQUESTS=50

# Adjust Docker memory
docker run -m 2g ghcr.io/your-org/riptidecrawler:latest
```

---

## Updating RipTide

### Binary Installations

```bash
# Re-run install script
curl -fsSL https://raw.githubusercontent.com/your-org/riptidecrawler/main/scripts/install.sh | bash

# Or manually download latest release
curl -L https://github.com/your-org/riptidecrawler/releases/latest/download/riptide-linux-x64-native.tar.gz -o riptide.tar.gz
```

### Docker Installations

```bash
# Pull latest image
docker pull ghcr.io/your-org/riptidecrawler:latest

# Restart containers
docker-compose down && docker-compose up -d
```

### Source Installations

```bash
git pull
cargo build --release
```

---

## Uninstalling

### Binary Installations

```bash
# Automatic uninstall
curl -fsSL https://raw.githubusercontent.com/your-org/riptidecrawler/main/scripts/install.sh | bash -s -- --uninstall

# Manual removal
rm -rf ~/.riptide ~/.local/bin/riptide*
```

### Docker Installations

```bash
# Remove containers
docker-compose down -v

# Remove images
docker rmi ghcr.io/your-org/riptidecrawler:latest
```

---

## Getting Help

- **Documentation**: [Full docs](../README.md)
- **Issues**: [GitHub Issues](https://github.com/your-org/riptidecrawler/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-org/riptidecrawler/discussions)
- **Discord**: [Join our community](https://discord.gg/riptide)

---

## What's Next?

Now that RipTide is running, explore:

- **[API Documentation](../02-api-reference/README.md)** - Learn about all endpoints
- **[Configuration Guide](../01-guides/CONFIGURATION.md)** - Advanced configuration
- **[Performance Tuning](../07-advanced/PERFORMANCE.md)** - Optimize for your use case
- **[Deployment Guide](../06-deployment/README.md)** - Production deployment
- **[Development Guide](../05-development/README.md)** - Contribute to RipTide

Happy crawling! 🌊
