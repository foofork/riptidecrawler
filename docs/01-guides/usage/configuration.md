# Configuration Guide

This guide covers all configuration options for RipTide, from basic settings to advanced performance tuning.

## Configuration Overview

RipTide uses a hierarchical configuration system:

1. **Default values** - Built-in sensible defaults
2. **Configuration file** - YAML file (optional, most settings via environment variables)
3. **Environment variables** - Primary configuration method
4. **Command line flags** - Highest priority

## Environment Variable Configuration (Recommended)

RipTide primarily uses environment variables for configuration:

```bash
# Required
SERPER_API_KEY=your_serper_api_key
REDIS_URL=redis://localhost:6379/0

# Optional service URLs
HEADLESS_URL=http://localhost:9123

# Optional logging
RUST_LOG=info
```

## Configuration File Structure (Advanced)

For complex setups, you can use a YAML configuration file:

```yaml
# Search provider settings
search:
  provider: serper
  api_key_env: SERPER_API_KEY
  country: us
  locale: en
  per_query_limit: 25

# HTTP crawling behavior
crawl:
  concurrency: 16
  max_redirects: 5
  timeout_ms: 20000
  user_agent_mode: rotate
  robots_policy: obey
  cache: read_through
  max_response_mb: 20

# Content extraction settings
extraction:
  wasm_module_path: "./wasm/riptide-extractor-wasm/target/wasm32-wasip2/release/riptide_extractor_wasm.wasm"
  version_tag: "wasm:0.1"
  mode: "article"
  produce_markdown: true
  produce_json: true
  token_chunk_max: 1200
  token_overlap: 120

# Dynamic content handling
dynamic:
  enable_headless_fallback: true
  wait_for: null
  scroll:
    enabled: true
    steps: 8
    step_px: 2000
    delay_ms: 300
  screenshot: false

# Stealth and fingerprint settings
stealth:
  enabled: true
  random_ua: true
  viewport: [1280, 800]
  timezone: "Europe/Amsterdam"
  locale: "en-US"

# Proxy configuration
proxies:
  enabled: false
  http_proxy_env: HTTP_PROXY
  https_proxy_env: HTTPS_PROXY

# Cache and storage
redis:
  url: "redis://localhost:6379/0"

artifacts:
  base_dir: "/data/artifacts"

# Logging configuration
logging:
  level: "info"
  format: "json"
```

## Search Configuration

### Provider Settings
```yaml
search:
  # Search provider (currently supports 'serper')
  provider: serper

  # Environment variable containing API key
  api_key_env: SERPER_API_KEY

  # Default country for search results
  country: us

  # Language/locale for results
  locale: en

  # Maximum results per search query
  per_query_limit: 25

  # Custom search engine ID (for Google Custom Search)
  custom_search_engine_id: null

  # Enable safe search filtering
  safe_search: moderate
```

### Supported Providers

**Serper (Recommended):**
- Fast and reliable
- Good rate limits
- Affordable pricing
- Sign up at [serper.dev](https://serper.dev)

```yaml
search:
  provider: serper
  api_key_env: SERPER_API_KEY
  country: us
  locale: en
```

**Google Custom Search (Alternative):**
```yaml
search:
  provider: google_custom
  api_key_env: GOOGLE_API_KEY
  custom_search_engine_id: your_cse_id
  country: us
  locale: en
```

## Crawling Configuration

### Basic Crawling Settings
```yaml
crawl:
  # Number of concurrent HTTP requests
  concurrency: 16

  # Maximum redirects to follow
  max_redirects: 5

  # Request timeout in milliseconds
  timeout_ms: 20000

  # Connection timeout in milliseconds
  connect_timeout_ms: 5000

  # Maximum response size in megabytes
  max_response_mb: 20

  # Supported compression formats
  accept_compression: [gzip, br, deflate]

  # Allowed content types
  allowed_content_types:
    - "text/html"
    - "application/xhtml+xml"
    - "application/pdf"
```

### User Agent Configuration
```yaml
crawl:
  # User agent strategy: 'fixed', 'rotate', 'random'
  user_agent_mode: rotate

  # Custom user agent (when mode is 'fixed')
  custom_user_agent: "RipTide/1.0 (+https://your-site.com/bot)"

  # User agent rotation list (when mode is 'rotate')
  user_agents:
    - "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
    - "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15"
    - "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36"
```

### Robots.txt Policy
```yaml
crawl:
  # How to handle robots.txt: 'obey', 'ignore', 'warn'
  robots_policy: obey

  # Cache robots.txt files for this duration (seconds)
  robots_cache_ttl: 3600

  # Custom robots.txt overrides
  robots_overrides:
    "example.com": "allow"  # Always allow crawling
    "blocked-site.com": "deny"  # Always deny crawling
```

### Rate Limiting
```yaml
crawl:
  # Enable per-domain rate limiting
  rate_limiting: true

  # Default delay between requests to same domain (ms)
  default_delay_ms: 1000

  # Per-domain delay overrides
  domain_delays:
    "slow-site.com": 5000
    "fast-site.com": 200

  # Burst allowance (requests before rate limiting kicks in)
  burst_allowance: 3
```

### Cache Configuration
```yaml
crawl:
  # Cache strategy: 'enabled', 'bypass', 'read_through'
  cache: read_through

  # Cache TTL for successful responses (seconds)
  cache_ttl_success: 86400  # 24 hours

  # Cache TTL for failed responses (seconds)
  cache_ttl_error: 3600  # 1 hour

  # Maximum cached response size (bytes)
  max_cache_size: 10485760  # 10MB

  # Cache invalidation patterns
  cache_invalidation:
    - "*/admin/*"  # Never cache admin pages
    - "*/search?*"  # Never cache search results
```

## Content Extraction Configuration

### WASM Extractor Settings
```yaml
extraction:
  # Path to WASM extraction module
  wasm_module_path: "./wasm/riptide-extractor-wasm/target/wasm32-wasip2/release/riptide_extractor_wasm.wasm"

  # WASM module version/tag for cache invalidation
  version_tag: "wasm:0.1"

  # Extraction mode: 'article', 'full', 'metadata'
  mode: "article"

  # Timeout for WASM execution (seconds)
  timeout_seconds: 30

  # Maximum input size for WASM (bytes)
  max_input_bytes: 20971520  # 20MB

  # Enable/disable specific extractors
  extractors:
    text: true
    metadata: true
    links: true
    images: true
    videos: false
```

### Output Format Configuration
```yaml
extraction:
  # Generate markdown output
  produce_markdown: true

  # Generate JSON output
  produce_json: true

  # Generate plain text output
  produce_text: true

  # Include raw HTML in output
  include_raw_html: false

  # Text chunking for large content
  token_chunk_max: 1200
  token_overlap: 120

  # Tokenizer to use: 'tiktoken', 'simple', 'unicode'
  tokenizer: "tiktoken"
```

### Content Filtering
```yaml
extraction:
  # Minimum content length (characters)
  min_content_length: 100

  # Maximum content length (characters)
  max_content_length: 1000000

  # Remove specific elements
  remove_elements:
    - "script"
    - "style"
    - "nav"
    - "footer"
    - ".advertisement"

  # Content quality thresholds
  quality_thresholds:
    min_text_ratio: 0.1  # Text to HTML ratio
    min_paragraph_count: 3
    max_link_density: 0.3
```

## Dynamic Content Configuration

### Headless Browser Settings
```yaml
dynamic:
  # Enable headless browser fallback
  enable_headless_fallback: true

  # Headless service URL
  headless_service_url: "http://localhost:9123"

  # Maximum concurrent headless sessions
  max_concurrent_sessions: 2

  # Session timeout (seconds)
  session_timeout: 60

  # Default wait condition
  wait_for: null  # CSS selector to wait for

  # Page load timeout (seconds)
  page_load_timeout: 30
```

### Scroll and Interaction
```yaml
dynamic:
  scroll:
    # Enable automatic scrolling
    enabled: true

    # Number of scroll steps
    steps: 8

    # Pixels per scroll step
    step_px: 2000

    # Delay between scroll steps (ms)
    delay_ms: 300

    # Wait for content after scrolling
    wait_after_scroll: true

  # Custom JavaScript to execute
  custom_js: |
    // Click accept cookies button if present
    const acceptBtn = document.querySelector('[data-testid="accept-cookies"]');
    if (acceptBtn) acceptBtn.click();

  # Element interactions
  interactions:
    - selector: ".load-more-button"
      action: "click"
      wait_after: 2000
    - selector: "#search-input"
      action: "type"
      text: "search query"
```

### Capture Options
```yaml
dynamic:
  # Take screenshots
  screenshot: false

  # Screenshot format: 'png', 'jpeg'
  screenshot_format: png

  # Screenshot quality (for JPEG)
  screenshot_quality: 80

  # Capture PDF snapshots
  pdf_snapshot: false

  # Capture MHTML files
  mhtml_capture: false

  # Full page capture vs viewport only
  full_page_capture: true
```

## Stealth and Anti-Detection

### Browser Fingerprinting
```yaml
stealth:
  # Enable stealth mode
  enabled: true

  # Randomize user agents
  random_ua: true

  # Browser viewport size
  viewport: [1280, 800]

  # Randomize viewport size
  randomize_viewport: true
  viewport_range:
    width: [1200, 1920]
    height: [800, 1080]

  # Browser timezone
  timezone: "Europe/Amsterdam"

  # Browser locale
  locale: "en-US"

  # Randomize timezone and locale
  randomize_locale: false
```

### Request Headers
```yaml
stealth:
  # Custom headers to add/override
  headers:
    "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
    "Accept-Language": "en-US,en;q=0.5"
    "Accept-Encoding": "gzip, deflate, br"
    "DNT": "1"
    "Connection": "keep-alive"
    "Upgrade-Insecure-Requests": "1"

  # Remove identifying headers
  remove_headers:
    - "X-Forwarded-For"
    - "X-Real-IP"

  # TLS fingerprint randomization
  randomize_tls: true
```

### Proxy Configuration
```yaml
proxies:
  # Enable proxy usage
  enabled: false

  # Proxy rotation strategy: 'round_robin', 'random', 'sticky'
  rotation_strategy: "round_robin"

  # Proxy list
  proxy_list:
    - "http://proxy1.example.com:8080"
    - "http://proxy2.example.com:8080"
    - "socks5://proxy3.example.com:1080"

  # Proxy authentication
  proxy_auth:
    "proxy1.example.com:8080":
      username: "user1"
      password: "pass1"

  # Proxy health checking
  health_check:
    enabled: true
    interval_seconds: 300
    test_url: "http://httpbin.org/ip"

  # Fallback to direct connection
  fallback_direct: true
```

## Storage and Caching

### Redis Configuration
```yaml
redis:
  # Redis connection URL
  url: "redis://localhost:6379/0"

  # Connection pool settings
  pool_size: 10
  pool_timeout_seconds: 5

  # Key prefix for namespacing
  key_prefix: "riptide:"

  # Default TTL for cached items (seconds)
  default_ttl: 86400

  # Compression for large values
  compression: true
  compression_threshold: 1024  # bytes

  # Redis cluster configuration (if using cluster)
  cluster_nodes:
    - "redis-1.example.com:6379"
    - "redis-2.example.com:6379"
    - "redis-3.example.com:6379"
```

### Artifact Storage
```yaml
artifacts:
  # Base directory for storing extracted content
  base_dir: "/data/artifacts"

  # File organization: 'flat', 'date', 'domain', 'hash'
  organization: "date"

  # Maximum file size (bytes)
  max_file_size: 52428800  # 50MB

  # Cleanup old files
  cleanup:
    enabled: true
    max_age_days: 30
    max_total_size_gb: 100

  # Compression for stored files
  compression:
    enabled: true
    format: "gzip"  # or "zstd", "lz4"
    level: 6

  # External storage (S3, GCS, etc.)
  external_storage:
    enabled: false
    provider: "s3"
    bucket: "my-riptide-bucket"
    region: "us-west-2"
    credentials_env: "AWS_CREDENTIALS"
```

## API Configuration

### Server Settings
```yaml
api:
  # Bind address
  host: "0.0.0.0"
  port: 8080

  # Request size limits
  max_request_size: 10485760  # 10MB
  max_json_payload: 1048576   # 1MB

  # Request timeout
  request_timeout_seconds: 300

  # Keep-alive settings
  keep_alive_seconds: 75

  # CORS configuration
  cors:
    enabled: true
    origins:
      - "https://your-frontend.com"
      - "http://localhost:3000"  # Development
    methods: ["GET", "POST", "OPTIONS"]
    headers: ["Content-Type", "Authorization"]
```

### Authentication and Authorization
```yaml
api:
  auth:
    # Enable authentication
    enabled: false

    # Authentication method: 'api_key', 'jwt', 'basic'
    method: "api_key"

    # API keys (hash with bcrypt in production)
    api_keys:
      - key: "your-secret-api-key"
        name: "Production Client"
        permissions: ["crawl", "search"]
        rate_limit: 1000  # requests per hour

      - key: "development-key"
        name: "Development"
        permissions: ["crawl"]
        rate_limit: 100

  # Rate limiting
  rate_limiting:
    enabled: true

    # Global rate limits
    global:
      requests_per_minute: 1000
      burst: 100

    # Per-IP rate limits
    per_ip:
      requests_per_minute: 100
      burst: 10

    # Per-API-key rate limits (see auth.api_keys)
    per_key: true
```

## Logging and Monitoring

### Log Configuration
```yaml
logging:
  # Log level: 'error', 'warn', 'info', 'debug', 'trace'
  level: "info"

  # Log format: 'text', 'json'
  format: "json"

  # Log output: 'stdout', 'stderr', file path
  output: "stdout"

  # File-specific settings (when output is a file path)
  file:
    path: "/var/log/riptide/riptide.log"
    rotation:
      max_size_mb: 100
      max_files: 10
      compress: true

  # Structured logging fields
  fields:
    service: "riptide-crawler"
    version: "0.1.0"
    environment: "production"

  # Filter sensitive information
  filters:
    - "password"
    - "api_key"
    - "authorization"
```

### Metrics and Telemetry
```yaml
monitoring:
  # Enable metrics collection
  metrics:
    enabled: true
    endpoint: "/metrics"  # Prometheus metrics endpoint

  # Performance tracking
  performance:
    track_request_duration: true
    track_cache_hit_rate: true
    track_extraction_success_rate: true
    track_headless_usage: true

  # Health checks
  health:
    endpoint: "/health"
    checks:
      - redis_connection
      - headless_service
      - disk_space
      - memory_usage

  # Distributed tracing (OpenTelemetry)
  tracing:
    enabled: false
    endpoint: "http://jaeger:14268/api/traces"
    sample_rate: 0.1
```

## Environment Variables

Any configuration value can be overridden with environment variables using the format `RIPTIDE_<SECTION>_<KEY>`:

```bash
# Override search provider
export RIPTIDE_SEARCH_PROVIDER=google_custom

# Override concurrency
export RIPTIDE_CRAWL_CONCURRENCY=32

# Override Redis URL
export RIPTIDE_REDIS_URL=redis://redis-cluster:6379

# Override log level
export RIPTIDE_LOGGING_LEVEL=debug

# Complex nested values (JSON format)
export RIPTIDE_STEALTH_VIEWPORT='[1920, 1080]'
export RIPTIDE_CRAWL_DOMAIN_DELAYS='{"slow-site.com": 5000}'
```

## Performance Tuning

### High-Throughput Configuration
```yaml
# Optimized for high throughput
crawl:
  concurrency: 64
  timeout_ms: 10000
  max_response_mb: 5

extraction:
  timeout_seconds: 15
  token_chunk_max: 800

dynamic:
  enable_headless_fallback: false  # Disable for pure speed

redis:
  pool_size: 20

api:
  request_timeout_seconds: 60
```

### Low-Memory Configuration
```yaml
# Optimized for low memory usage
crawl:
  concurrency: 4
  max_response_mb: 2

extraction:
  token_chunk_max: 400
  max_input_bytes: 5242880  # 5MB

dynamic:
  max_concurrent_sessions: 1

redis:
  pool_size: 3
```

### High-Quality Extraction Configuration
```yaml
# Optimized for extraction quality
extraction:
  mode: "full"
  timeout_seconds: 60
  produce_markdown: true
  produce_json: true
  min_content_length: 200

dynamic:
  enable_headless_fallback: true
  scroll:
    enabled: true
    steps: 12
  wait_for: ".main-content, article, .post-content"
```

## Validation and Testing

### Configuration Validation
```bash
# Validate configuration file
riptide-api --config riptide.yml --check-config

# Test with dry run
riptide-api --config riptide.yml --dry-run

# Validate specific sections
riptide-api --config riptide.yml --validate-section extraction
```

### Configuration Examples

The `config/application/` directory contains example configurations:

- `riptide.yml.example` - Basic configuration template
- `high-performance.yml` - High-throughput setup
- `low-resource.yml` - Minimal resource usage
- `quality-focused.yml` - Maximum extraction quality
- `development.yml` - Development environment

## Next Steps

- **API Usage**: Learn how to use the configured crawler in [API Usage Guide](api-usage.md)
- **Deployment**: Deploy your configured setup with [Production Deployment](../deployment/production.md)
- **Troubleshooting**: Debug configuration issues with [Troubleshooting Guide](troubleshooting.md)
- **Scaling**: Scale your deployment with [Scaling Guide](../deployment/scaling.md)