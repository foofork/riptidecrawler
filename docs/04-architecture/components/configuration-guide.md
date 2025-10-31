# RipTide - Configuration Guide

## Overview

RipTide configuration is managed through environment variables with sensible defaults defined in code. This guide covers all configuration options and their effects on system behavior.

## Configuration Files

### Primary Configuration: `config/application/riptide.yml`

The main configuration file controls all aspects of the crawler's behavior:

```yaml
# Search API Configuration
search:
  provider: serper                    # Search provider (currently only "serper" supported)
  api_key_env: SERPER_API_KEY        # Environment variable containing API key
  country: us                        # Search result country bias
  locale: en                         # Search result language
  per_query_limit: 25                # Maximum results per search query

# HTTP Crawling Configuration
crawl:
  concurrency: 16                    # Number of concurrent crawl workers
  max_redirects: 5                   # Maximum HTTP redirects to follow
  timeout_ms: 20000                  # Request timeout in milliseconds
  user_agent_mode: rotate            # User agent strategy: "fixed" | "rotate"
  robots_policy: obey                # Robots.txt compliance: "obey" | "ignore"
  cache: read_through                # Cache strategy: "enabled" | "bypass" | "read_through"
  max_response_mb: 20                # Maximum response size in megabytes
  accept_compression: [gzip, br]     # Accepted compression methods
  allowed_content_types:             # Allowed MIME types for processing
    - "text/html"
    - "application/xhtml+xml"
    - "application/pdf"

# Content Extraction Configuration
extraction:
  wasm_module_path: "/opt/riptide/extractor/extractor.wasm"  # Path to WASM extractor
  version_tag: "wasm:0.1"            # Extractor version identifier
  mode: "article"                    # Extraction mode: "article" | "full" | "minimal"
  produce_markdown: true             # Generate markdown output
  produce_json: true                 # Generate JSON output
  token_chunk_max: 1200              # Maximum tokens per content chunk
  token_overlap: 120                 # Token overlap between chunks
  tokenizer: "tiktoken"              # Tokenization method

# Dynamic Content Configuration
dynamic:
  enable_headless_fallback: true     # Use headless browser for JavaScript content
  wait_for: null                     # CSS selector to wait for (optional)
  scroll:
    enabled: true                    # Enable automatic scrolling
    steps: 8                         # Number of scroll steps
    step_px: 2000                    # Pixels per scroll step
    delay_ms: 300                    # Delay between scroll steps
  screenshot: false                  # Capture screenshots
  pdf_snapshot: false                # Generate PDF snapshots
  mhtml_capture: false               # Capture MHTML archives

# Stealth Mode Configuration
stealth:
  enabled: true                      # Enable stealth browsing features
  random_ua: true                    # Randomize user agents
  viewport: [1280, 800]              # Browser viewport dimensions
  timezone: "Europe/Amsterdam"       # Browser timezone
  locale: "en-US"                    # Browser locale
  geolocation: null                  # Geolocation coordinates (optional)

# Proxy Configuration
proxies:
  enabled: false                     # Enable proxy support
  http_proxy_env: HTTP_PROXY         # HTTP proxy environment variable
  https_proxy_env: HTTPS_PROXY       # HTTPS proxy environment variable

# Redis Configuration
redis:
  url: "redis://redis:6379/0"       # Redis connection string

# Artifacts Configuration
artifacts:
  base_dir: "/data/artifacts"        # Base directory for storing extracted content

# Logging Configuration
logging:
  level: "info"                      # Log level: "trace" | "debug" | "info" | "warn" | "error"
```

### Additional Configuration Files

**Security Policies**: `config/application/policies.yml`
```yaml
# Content security and filtering policies
security:
  max_file_size_mb: 50
  blocked_domains: []
  allowed_schemes: ["http", "https"]
  content_filters: []
```

**Content Fingerprints**: `config/application/fingerprints.yml`
```yaml
# Content deduplication fingerprints
fingerprinting:
  enabled: true
  algorithm: "sha256"
  threshold: 0.85
```

**Cargo Deny**: `deny.toml`
```toml
# Dependency security and licensing policies
[licenses]
allow = ["Apache-2.0", "MIT", "BSD-3-Clause"]
deny = ["GPL-3.0"]
```

## Environment Variables

### Required Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `SERPER_API_KEY` | Serper.dev API key for web search | `sk_abc123...` |

### Optional Environment Variables

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `RUST_LOG` | Logging level and filters | `info` | `debug,riptide_core=trace` |
| `REDIS_URL` | Override Redis connection | From config | `redis://localhost:6379/1` |
| `HEADLESS_URL` | Override headless service URL | `http://headless:9123` | `http://localhost:9123` |
| `HTTP_PROXY` | HTTP proxy server | None | `http://proxy:8080` |
| `HTTPS_PROXY` | HTTPS proxy server | None | `https://proxy:8080` |

### Docker Environment Variables

When running in Docker, these variables are set automatically:

```yaml
# docker-compose.yml environment variables
environment:
  - RUST_LOG=info
  - REDIS_URL=redis://redis:6379/0
  - HEADLESS_URL=http://headless:9123
  - SERPER_API_KEY=${SERPER_API_KEY}  # Passed from host
```

## Configuration Sections

### Search Configuration

Controls web search functionality for deep search operations:

```yaml
search:
  provider: serper              # Currently only Serper.dev is supported
  api_key_env: SERPER_API_KEY   # Environment variable containing API key
  country: us                   # ISO country code for search bias
  locale: en                    # Language code for search results
  per_query_limit: 25           # Maximum URLs to extract per search
```

**Supported Values**:
- `country`: ISO 3166-1 alpha-2 codes (us, uk, de, fr, etc.)
- `locale`: Language codes (en, es, fr, de, etc.)
- `per_query_limit`: 1-100 (API limits apply)

### Crawl Configuration

Controls HTTP crawling behavior and performance:

```yaml
crawl:
  concurrency: 16               # Concurrent request limit
  max_redirects: 5              # HTTP redirect following
  timeout_ms: 20000             # Request timeout
  user_agent_mode: rotate       # User agent strategy
  robots_policy: obey           # Robots.txt compliance
  cache: read_through           # Caching strategy
  max_response_mb: 20           # Response size limit
```

**Performance Tuning**:
- `concurrency`: Start with 8-16, increase based on target server capacity
- `timeout_ms`: 10-30 seconds depending on content complexity
- `max_response_mb`: Adjust based on expected content size

**Cache Strategies**:
- `enabled`: Always use cache when available
- `bypass`: Never use cache, always fetch fresh
- `read_through`: Use cache for reads, update on misses

**User Agent Modes**:
- `fixed`: Use single user agent string
- `rotate`: Rotate through multiple user agents

### Extraction Configuration

Controls content extraction and processing:

```yaml
extraction:
  wasm_module_path: "/opt/riptide/extractor/extractor.wasm"
  version_tag: "wasm:0.1"       # Extractor version
  mode: "article"               # Extraction strategy
  produce_markdown: true        # Output formats
  produce_json: true
  token_chunk_max: 1200         # Content chunking
  token_overlap: 120
  tokenizer: "tiktoken"         # Tokenization method
```

**Extraction Modes**:
- `article`: Extract main article content (default)
- `full`: Extract all text content
- `minimal`: Extract only title and basic metadata

**Tokenization**:
- Used for AI processing and content chunking
- `token_chunk_max`: Optimal size 800-1500 tokens
- `token_overlap`: 10-20% of chunk size for context preservation

### Dynamic Content Configuration

Controls headless browser behavior for JavaScript-heavy sites:

```yaml
dynamic:
  enable_headless_fallback: true  # Enable browser rendering
  wait_for: null                  # CSS selector to wait for
  scroll:
    enabled: true                 # Auto-scroll for lazy content
    steps: 8                      # Number of scroll actions
    step_px: 2000                 # Pixels per scroll
    delay_ms: 300                 # Delay between scrolls
  screenshot: false               # Capture page screenshots
  pdf_snapshot: false             # Generate PDF versions
  mhtml_capture: false            # Save complete web archives
```

**Wait Strategies**:
- `wait_for: null` - Wait for page load only
- `wait_for: ".content"` - Wait for specific element
- `wait_for: "networkidle"` - Wait for network idle

**Performance Impact**:
- Headless rendering is 5-10x slower than static crawling
- Only enable for sites that require JavaScript
- Consider increasing timeout for complex pages

### Stealth Configuration

Controls anti-detection measures:

```yaml
stealth:
  enabled: true                   # Enable stealth features
  random_ua: true                 # Randomize user agents
  viewport: [1280, 800]           # Browser window size
  timezone: "Europe/Amsterdam"    # Simulated timezone
  locale: "en-US"                 # Browser language
  geolocation: null               # GPS coordinates (optional)
```

**Stealth Features**:
- Randomized user agents from realistic browser pool
- Simulated human-like mouse movements
- Realistic viewport sizes and device characteristics
- Timezone and locale spoofing

### Redis Configuration

Controls caching and queue backend:

```yaml
redis:
  url: "redis://redis:6379/0"    # Connection string
```

**Connection Formats**:
- Local: `redis://localhost:6379/0`
- Docker: `redis://redis:6379/0`
- Authenticated: `redis://user:pass@host:6379/0`
- SSL: `rediss://host:6379/0`

**Database Selection**:
- Database 0: Default cache storage
- Database 1: Job queue (future use)
- Database 2: Session storage (future use)

## Configuration Validation

### Startup Validation

The system validates configuration at startup:

```rust
// Configuration loading and validation
let config = Config::from_file("config/application/riptide.yml")?;
config.validate()?;
```

**Validation Checks**:
- File existence and readability
- Required fields presence
- Value range validation
- Environment variable availability
- Service connectivity (Redis, headless browser)

### Runtime Configuration Updates

Some configuration can be updated at runtime:

**Hot-reloadable**:
- Logging levels (`RUST_LOG`)
- Cache settings
- Concurrency limits

**Requires Restart**:
- Service URLs
- WASM module paths
- Security policies

## Configuration Examples

### Development Configuration

```yaml
# config/application/riptide-dev.yml
crawl:
  concurrency: 4                # Lower concurrency for development
  timeout_ms: 30000             # Longer timeout for debugging
  cache: bypass                 # Always fetch fresh content

logging:
  level: debug                  # Verbose logging

dynamic:
  screenshot: true              # Capture screenshots for debugging
  scroll:
    delay_ms: 1000              # Slower scrolling for observation
```

### Production Configuration

```yaml
# config/application/riptide-prod.yml
crawl:
  concurrency: 32               # Higher concurrency for performance
  timeout_ms: 15000             # Shorter timeout for efficiency
  cache: read_through           # Optimize cache usage

logging:
  level: warn                   # Minimal logging for performance

dynamic:
  scroll:
    delay_ms: 100               # Faster scrolling for efficiency
```

### High-Security Configuration

```yaml
# config/application/riptide-secure.yml
stealth:
  enabled: true
  random_ua: true
  timezone: "UTC"               # Neutral timezone

proxies:
  enabled: true                 # Use proxy rotation
  http_proxy_env: HTTP_PROXY
  https_proxy_env: HTTPS_PROXY

crawl:
  robots_policy: obey           # Strict compliance
  user_agent_mode: rotate       # Anti-detection
```

## Deployment Configurations

### Docker Compose

```yaml
# docker-compose.yml
version: '3.8'
services:
  api:
    environment:
      - RUST_LOG=info
      - REDIS_URL=redis://redis:6379/0
      - HEADLESS_URL=http://headless:9123
      - SERPER_API_KEY=${SERPER_API_KEY}
    volumes:
      - ./configs:/app/configs:ro
      - artifacts-data:/data/artifacts
```

### Kubernetes

```yaml
# k8s-config.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: riptide-config
data:
  riptide.yml: |
    crawl:
      concurrency: 16
      cache: read_through
    redis:
      url: "redis://redis-service:6379/0"
---
apiVersion: v1
kind: Secret
metadata:
  name: riptide-secrets
data:
  serper-api-key: <base64-encoded-key>
```

## Troubleshooting Configuration

### Common Issues

**Configuration Not Found**:
```bash
# Check file path and permissions
ls -la config/application/riptide.yml
# Verify Docker volume mounts
docker inspect riptide-api | grep -A 10 Mounts
```

**Environment Variables Not Set**:
```bash
# Check environment variable availability
env | grep SERPER_API_KEY
docker exec riptide-api env | grep SERPER_API_KEY
```

**Service Connectivity Issues**:
```bash
# Test Redis connectivity
redis-cli -h redis ping
# Test headless service
curl http://headless:9123/healthz
```

### Configuration Debugging

Enable debug logging to trace configuration loading:

```bash
RUST_LOG=debug,riptide_core::config=trace ./riptide-api
```

Validate configuration syntax:

```bash
# YAML syntax check
python -c "import yaml; yaml.safe_load(open('config/application/riptide.yml'))"

# JSON schema validation (if schema available)
jsonschema -i configs/riptide.yml schema/config.schema.json
```

## Best Practices

### Security
1. **Never commit API keys** to version control
2. **Use environment variables** for sensitive data
3. **Restrict file permissions** on configuration files
4. **Validate all inputs** before processing
5. **Use secrets management** in production

### Performance
1. **Tune concurrency** based on target server capacity
2. **Enable caching** for repeated crawls
3. **Set appropriate timeouts** for your use case
4. **Monitor resource usage** and adjust limits
5. **Use headless rendering** only when necessary

### Maintenance
1. **Document configuration changes** in version control
2. **Test configuration** in staging before production
3. **Monitor logs** for configuration warnings
4. **Keep configurations** environment-specific
5. **Backup configurations** with deployment artifacts