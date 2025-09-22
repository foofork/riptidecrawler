# Installation Guide

This guide covers all the ways to install and run RipTide Crawler, from Docker containers to building from source.

## Quick Start with Docker (Recommended)

The fastest way to get RipTide running is with Docker:

```bash
# Clone the repository
git clone <repository-url>
cd riptide-crawler

# Start all services
docker-compose up -d

# Verify installation
curl http://localhost:8080/health
```

That's it! RipTide is now running on `http://localhost:8080`.

## Installation Methods

### 1. Docker (Production Ready)

#### Prerequisites
- Docker 20.10+
- Docker Compose 2.0+
- 4GB RAM minimum, 8GB recommended
- 10GB disk space

#### Simple Setup
```bash
# Download docker-compose file
curl -O https://raw.githubusercontent.com/your-repo/riptide/main/docker-compose.yml

# Set environment variables
cat > .env << EOF
SERPER_API_KEY=your_serper_api_key_here
REDIS_URL=redis://redis:6379
RUST_LOG=info
EOF

# Start services
docker-compose up -d

# Check status
docker-compose ps
```

#### Custom Configuration
```bash
# Download full repository for custom configs
git clone <repository-url>
cd riptide-crawler

# Edit configuration
cp configs/riptide.yml.example configs/riptide.yml
# Edit configs/riptide.yml with your settings

# Start with custom config
docker-compose -f docker-compose.yml up -d
```

### 2. Prebuilt Binaries

#### Download Latest Release
```bash
# Linux x86_64
curl -L -o riptide-linux.tar.gz \
  https://github.com/your-repo/riptide/releases/latest/download/riptide-linux-x86_64.tar.gz

# macOS (Intel)
curl -L -o riptide-macos.tar.gz \
  https://github.com/your-repo/riptide/releases/latest/download/riptide-macos-x86_64.tar.gz

# macOS (Apple Silicon)
curl -L -o riptide-macos.tar.gz \
  https://github.com/your-repo/riptide/releases/latest/download/riptide-macos-aarch64.tar.gz

# Windows
curl -L -o riptide-windows.zip \
  https://github.com/your-repo/riptide/releases/latest/download/riptide-windows-x86_64.zip
```

#### Install Binaries
```bash
# Extract archive
tar -xzf riptide-linux.tar.gz

# Make executable and install
chmod +x riptide-*
sudo mv riptide-* /usr/local/bin/

# Verify installation
riptide-api --version
```

#### Setup Dependencies
```bash
# Install Redis (Ubuntu/Debian)
sudo apt update
sudo apt install redis-server

# Install Redis (macOS)
brew install redis

# Install Redis (CentOS/RHEL)
sudo yum install redis

# Start Redis
sudo systemctl start redis
# or on macOS: brew services start redis
```

### 3. Build from Source

#### Prerequisites
- Rust 1.75+ (latest stable recommended)
- Git
- pkg-config (Linux)
- OpenSSL development headers (Linux)

#### Install Rust
```bash
# Install rustup (Rust installer)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Add WebAssembly targets
rustup target add wasm32-wasi
rustup target add wasm32-wasip2

# Install build tools
cargo install just cargo-deny
```

#### System Dependencies

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install build-essential pkg-config libssl-dev
```

**CentOS/RHEL:**
```bash
sudo yum groupinstall "Development Tools"
sudo yum install pkgconfig openssl-devel
```

**macOS:**
```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install Homebrew (if not already installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

**Windows:**
```bash
# Install Visual Studio Build Tools or Visual Studio Community
# https://visualstudio.microsoft.com/downloads/

# Install Git for Windows
# https://git-scm.com/download/win
```

#### Build RipTide
```bash
# Clone repository
git clone <repository-url>
cd riptide-crawler

# Build all components
just build
# or manually:
# cargo build --workspace --release

# Build WebAssembly extractor
cd wasm/riptide-extractor-wasm
cargo build --release --target wasm32-wasip2
cd ../..

# Install binaries
sudo cp target/release/riptide-* /usr/local/bin/
```

## Configuration

### Environment Variables
Create a `.env` file with your configuration:

```bash
# Required: Search API key
SERPER_API_KEY=your_serper_api_key_here

# Redis connection
REDIS_URL=redis://localhost:6379

# Logging level
RUST_LOG=info

# Optional: Custom configuration file
RIPTIDE_CONFIG=/path/to/your/riptide.yml

# Optional: API server binding
RIPTIDE_API_HOST=0.0.0.0
RIPTIDE_API_PORT=8080

# Optional: Headless service binding
RIPTIDE_HEADLESS_HOST=0.0.0.0
RIPTIDE_HEADLESS_PORT=9123
```

### Configuration File
RipTide uses YAML configuration files. Create `riptide.yml`:

```yaml
# Search configuration
search:
  provider: serper
  api_key_env: SERPER_API_KEY
  country: us
  locale: en
  per_query_limit: 25

# Crawling behavior
crawl:
  concurrency: 16
  timeout_ms: 20000
  max_response_mb: 20
  respect_robots_txt: true

# Content extraction
extraction:
  wasm_module_path: "./target/wasm32-wasip2/release/riptide-extractor-wasm.wasm"
  produce_markdown: true
  token_chunk_max: 1200

# Dynamic content handling
dynamic:
  enable_headless_fallback: true
  scroll_steps: 8

# Caching
redis:
  url: "redis://localhost:6379/0"

# Output
artifacts:
  base_dir: "./data/artifacts"
```

## Running RipTide

### Docker (Recommended)
```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

### Manual Setup
```bash
# Terminal 1: Start Redis
redis-server

# Terminal 2: Start API server
riptide-api --config riptide.yml

# Terminal 3: Start headless service
riptide-headless --port 9123

# Terminal 4: Test the installation
curl -X POST http://localhost:8080/crawl \
  -H "Content-Type: application/json" \
  -d '{"urls": ["https://example.com"]}'
```

### SystemD Service (Linux)
Create service files for automatic startup:

```bash
# Create API service
sudo tee /etc/systemd/system/riptide-api.service > /dev/null << EOF
[Unit]
Description=RipTide Crawler API
After=network.target redis.service

[Service]
Type=simple
User=riptide
Group=riptide
WorkingDirectory=/opt/riptide
Environment=RUST_LOG=info
EnvironmentFile=/opt/riptide/.env
ExecStart=/usr/local/bin/riptide-api --config /opt/riptide/riptide.yml
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

# Create headless service
sudo tee /etc/systemd/system/riptide-headless.service > /dev/null << EOF
[Unit]
Description=RipTide Headless Browser Service
After=network.target

[Service]
Type=simple
User=riptide
Group=riptide
WorkingDirectory=/opt/riptide
Environment=RUST_LOG=info
ExecStart=/usr/local/bin/riptide-headless --port 9123
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

# Enable and start services
sudo systemctl daemon-reload
sudo systemctl enable riptide-api riptide-headless
sudo systemctl start riptide-api riptide-headless
```

## Verification

### Health Checks
```bash
# Check API health
curl http://localhost:8080/health
# Expected: {"status":"healthy","version":"0.1.0"}

# Check headless service
curl http://localhost:9123/health
# Expected: {"status":"healthy","chrome_version":"..."}

# Check Redis connection
redis-cli ping
# Expected: PONG
```

### Basic Functionality Test
```bash
# Test simple crawl
curl -X POST http://localhost:8080/crawl \
  -H "Content-Type: application/json" \
  -d '{
    "urls": ["https://httpbin.org/html"],
    "options": {"cache_mode": "bypass"}
  }' | jq '.'

# Test deep search (requires SERPER_API_KEY)
curl -X POST http://localhost:8080/deepsearch \
  -H "Content-Type: application/json" \
  -d '{
    "query": "rust programming",
    "limit": 5
  }' | jq '.'

# Test headless rendering
curl -X POST http://localhost:9123/render \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://httpbin.org/html",
    "wait_for": "body"
  }' | jq '.html | length'
```

## Performance Tuning

### System Requirements

**Minimum:**
- 2 CPU cores
- 4GB RAM
- 10GB disk space
- 100 Mbps network

**Recommended:**
- 4+ CPU cores
- 8GB+ RAM
- 50GB+ SSD storage
- 1 Gbps network

### Memory Configuration
```yaml
# In riptide.yml
crawl:
  concurrency: 16  # Adjust based on available memory
  max_response_mb: 20  # Limit per response

extraction:
  token_chunk_max: 1200  # Smaller chunks for lower memory usage

dynamic:
  headless_concurrency: 2  # Chromium is memory-intensive
```

### Redis Tuning
```bash
# /etc/redis/redis.conf
maxmemory 2gb
maxmemory-policy allkeys-lru
save ""  # Disable persistence for better performance
```

## Security Configuration

### Network Security
```bash
# Firewall rules (iptables example)
sudo iptables -A INPUT -p tcp --dport 8080 -s 10.0.0.0/8 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 9123 -s 127.0.0.1 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 6379 -s 127.0.0.1 -j ACCEPT
```

### User Permissions
```bash
# Create dedicated user
sudo useradd -r -s /bin/false riptide
sudo mkdir -p /opt/riptide/{config,data,logs}
sudo chown -R riptide:riptide /opt/riptide

# Set secure permissions
sudo chmod 750 /opt/riptide
sudo chmod 640 /opt/riptide/config/riptide.yml
```

### API Security
```yaml
# In riptide.yml
api:
  cors_origins: ["https://your-domain.com"]
  rate_limiting:
    enabled: true
    requests_per_minute: 100
  auth:
    enabled: false  # Enable for production
    api_keys: []
```

## Monitoring and Logging

### Log Configuration
```yaml
# In riptide.yml
logging:
  level: info
  format: json
  output: stdout
  file: /opt/riptide/logs/riptide.log
```

### Metrics Collection
```bash
# Install Prometheus Node Exporter
wget https://github.com/prometheus/node_exporter/releases/download/v1.6.1/node_exporter-1.6.1.linux-amd64.tar.gz
tar -xzf node_exporter-1.6.1.linux-amd64.tar.gz
sudo mv node_exporter-1.6.1.linux-amd64/node_exporter /usr/local/bin/
```

## Troubleshooting

### Common Issues

**Service won't start:**
```bash
# Check dependencies
systemctl status redis
curl http://localhost:6379

# Check configuration
riptide-api --config riptide.yml --check-config

# Check logs
journalctl -u riptide-api -f
```

**Out of memory:**
```bash
# Reduce concurrency
# Edit riptide.yml:
crawl:
  concurrency: 8  # Reduce from 16

# Monitor memory usage
top -p $(pgrep riptide)
```

**Slow performance:**
```bash
# Check Redis connection
redis-cli --latency

# Monitor CPU usage
htop

# Check disk I/O
iotop
```

### Getting Help

- **Documentation**: Check the full documentation in `docs/`
- **Configuration**: See [Configuration Guide](configuration.md)
- **API Usage**: See [API Usage Guide](api-usage.md)
- **Troubleshooting**: See [Troubleshooting Guide](troubleshooting.md)
- **Issues**: Open an issue on GitHub

## Next Steps

1. **Configure** RipTide for your use case: [Configuration Guide](configuration.md)
2. **Learn the API**: [API Usage Examples](api-usage.md)
3. **Deploy to production**: [Production Deployment](../deployment/production.md)
4. **Scale up**: [Scaling Guide](../deployment/scaling.md)