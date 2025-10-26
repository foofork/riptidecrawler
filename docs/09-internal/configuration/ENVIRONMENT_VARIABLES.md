# RipTide Environment Variables Reference

Complete reference for all environment variables used in RipTide API, CLI, and associated services.

## Table of Contents

- [Naming Conventions](#naming-conventions)
- [Output Directory Configuration](#output-directory-configuration)
- [CLI Configuration](#cli-configuration)
- [Core Services](#core-services)
- [Search Configuration](#search-configuration)
- [Performance & Resource Limits](#performance--resource-limits)
- [Rate Limiting](#rate-limiting)
- [Browser Pool Configuration](#browser-pool-configuration)
- [Memory Management](#memory-management)
- [PDF Processing](#pdf-processing)
- [WASM Runtime](#wasm-runtime)
- [LLM/AI Providers](#llmai-providers)
- [Telemetry & Observability](#telemetry--observability)
- [Authentication & Security](#authentication--security)
- [Spider/Crawler](#spidercrawler)
- [Worker Configuration](#worker-configuration)
- [Cache & Persistence](#cache--persistence)
- [Circuit Breaker](#circuit-breaker)
- [Streaming](#streaming)
- [Development & Testing](#development--testing)
- [Validation Rules](#validation-rules)

## Naming Conventions

All RipTide environment variables follow these naming patterns:

| Pattern | Description | Example |
|---------|-------------|---------|
| `RIPTIDE_*` | All RipTide-specific variables | `RIPTIDE_API_URL` |
| `RIPTIDE_*_OUTPUT_DIR` | Output directories | `RIPTIDE_SCREENSHOTS_DIR` |
| `RIPTIDE_*_URL` | Service URLs | `RIPTIDE_API_URL` |
| `RIPTIDE_*_TIMEOUT` | Timeout values (seconds) | `RIPTIDE_RENDER_TIMEOUT` |
| `RIPTIDE_*_LIMIT` | Resource limits | `RIPTIDE_MEMORY_LIMIT_MB` |
| `RIPTIDE_MAX_*` | Maximum values | `RIPTIDE_MAX_CONCURRENT_RENDERS` |

## Output Directory Configuration

### `RIPTIDE_OUTPUT_DIR`
- **Type**: String (path)
- **Default**: `./riptide-output`
- **Description**: Base directory for all RipTide output artifacts
- **Example**: `RIPTIDE_OUTPUT_DIR=/var/riptide/output`

### `RIPTIDE_SCREENSHOTS_DIR`
- **Type**: String (path)
- **Default**: `${RIPTIDE_OUTPUT_DIR}/screenshots`
- **Description**: Directory for screenshot storage
- **Example**: `RIPTIDE_SCREENSHOTS_DIR=/var/riptide/screenshots`

### `RIPTIDE_HTML_DIR`
- **Type**: String (path)
- **Default**: `${RIPTIDE_OUTPUT_DIR}/html`
- **Description**: Directory for HTML content storage
- **Example**: `RIPTIDE_HTML_DIR=/var/riptide/html`

### `RIPTIDE_PDF_DIR`
- **Type**: String (path)
- **Default**: `${RIPTIDE_OUTPUT_DIR}/pdf`
- **Description**: Directory for PDF output storage
- **Example**: `RIPTIDE_PDF_DIR=/var/riptide/pdf`

### `RIPTIDE_REPORTS_DIR`
- **Type**: String (path)
- **Default**: `${RIPTIDE_OUTPUT_DIR}/reports`
- **Description**: Directory for report generation
- **Example**: `RIPTIDE_REPORTS_DIR=/var/riptide/reports`

### `RIPTIDE_ARTIFACTS_DIR`
- **Type**: String (path)
- **Default**: `${RIPTIDE_OUTPUT_DIR}/artifacts`
- **Description**: Directory for misc artifacts
- **Example**: `RIPTIDE_ARTIFACTS_DIR=/var/riptide/artifacts`

### `RIPTIDE_TEMP_DIR`
- **Type**: String (path)
- **Default**: `${RIPTIDE_OUTPUT_DIR}/temp`
- **Description**: Temporary file storage
- **Example**: `RIPTIDE_TEMP_DIR=/tmp/riptide`

### `RIPTIDE_LOGS_DIR`
- **Type**: String (path)
- **Default**: `${RIPTIDE_OUTPUT_DIR}/logs`
- **Description**: Log file directory
- **Example**: `RIPTIDE_LOGS_DIR=/var/log/riptide`

### `RIPTIDE_CACHE_DIR`
- **Type**: String (path)
- **Default**: `${RIPTIDE_OUTPUT_DIR}/cache`
- **Description**: Local cache directory
- **Example**: `RIPTIDE_CACHE_DIR=/var/cache/riptide`

## CLI Configuration

### `RIPTIDE_API_URL`
- **Type**: URL
- **Default**: `http://localhost:8080`
- **Required**: Yes (for CLI)
- **Description**: RipTide API server endpoint URL
- **Example**: `RIPTIDE_API_URL=https://api.riptide.example.com`
- **Validation**: Must be valid HTTP/HTTPS URL

### `RIPTIDE_API_KEY`
- **Type**: String
- **Default**: None
- **Required**: If `REQUIRE_AUTH=true`
- **Description**: API authentication key
- **Example**: `RIPTIDE_API_KEY=sk-abc123xyz`
- **Security**: Never commit to version control

### `RIPTIDE_CLI_MODE`
- **Type**: Enum
- **Default**: `api_first`
- **Options**: `api_first`, `api_only`, `direct`
- **Description**: CLI operation mode
  - `api_first`: Try API first, fallback to direct
  - `api_only`: API only, fail if unavailable
  - `direct`: Direct extraction only (no API)
- **Example**: `RIPTIDE_CLI_MODE=direct`

### `RIPTIDE_CLI_OUTPUT_FORMAT`
- **Type**: Enum
- **Default**: `text`
- **Options**: `json`, `text`, `table`, `markdown`
- **Description**: Default output format for CLI
- **Example**: `RIPTIDE_CLI_OUTPUT_FORMAT=json`

### `RIPTIDE_CLI_VERBOSE`
- **Type**: Boolean
- **Default**: `false`
- **Description**: Enable verbose CLI output
- **Example**: `RIPTIDE_CLI_VERBOSE=true`

### `RIPTIDE_WASM_PATH`
- **Type**: String (path)
- **Default**: `./target/wasm32-wasi/release/riptide-extraction.wasm`
- **Description**: Path to WASM extraction module
- **Example**: `RIPTIDE_WASM_PATH=/opt/riptide/extractor.wasm`
- **Validation**: File must exist and be readable

## Core Services

### `REDIS_URL`
- **Type**: URL
- **Default**: `redis://localhost:6379/0`
- **Description**: Redis connection URL for caching
- **Example**: `REDIS_URL=redis://redis.example.com:6379/0`
- **Format**: `redis://[user:password@]host:port/database`

### `HEADLESS_URL`
- **Type**: URL
- **Default**: `http://localhost:9123`
- **Description**: Headless browser service URL
- **Example**: `HEADLESS_URL=http://chrome-service:9123`

### `RIPTIDE_API_HOST`
- **Type**: IP Address
- **Default**: `0.0.0.0`
- **Description**: API server bind address
- **Example**: `RIPTIDE_API_HOST=127.0.0.1`

### `RIPTIDE_API_PORT`
- **Type**: Integer
- **Default**: `8080`
- **Range**: 1-65535
- **Description**: API server port
- **Example**: `RIPTIDE_API_PORT=3000`

## Search Configuration

### `SEARCH_BACKEND`
- **Type**: Enum
- **Default**: `serper`
- **Options**: `serper`, `none`, `searxng`
- **Description**: Search provider backend
- **Example**: `SEARCH_BACKEND=searxng`

### `SERPER_API_KEY`
- **Type**: String
- **Required**: If `SEARCH_BACKEND=serper`
- **Description**: Serper.dev API key
- **Example**: `SERPER_API_KEY=abc123...`
- **Security**: Never commit to version control

### `SEARXNG_BASE_URL`
- **Type**: URL
- **Required**: If `SEARCH_BACKEND=searxng`
- **Description**: SearXNG instance URL
- **Example**: `SEARXNG_BASE_URL=http://localhost:8888`

### `SEARCH_TIMEOUT`
- **Type**: Integer (seconds)
- **Default**: `30`
- **Range**: 1-300
- **Description**: Search operation timeout
- **Example**: `SEARCH_TIMEOUT=60`

### `SEARCH_ENABLE_URL_PARSING`
- **Type**: Boolean
- **Default**: `true`
- **Description**: Enable URL parsing for None provider
- **Example**: `SEARCH_ENABLE_URL_PARSING=false`

## Performance & Resource Limits

### `RIPTIDE_MAX_CONCURRENT_RENDERS`
- **Type**: Integer
- **Default**: `10`
- **Range**: 1-100
- **Description**: Maximum concurrent render operations
- **Example**: `RIPTIDE_MAX_CONCURRENT_RENDERS=20`
- **Impact**: Higher values increase memory usage

### `RIPTIDE_MAX_CONCURRENT_PDF`
- **Type**: Integer
- **Default**: `2`
- **Range**: 1-10
- **Description**: Maximum concurrent PDF operations (semaphore limit)
- **Example**: `RIPTIDE_MAX_CONCURRENT_PDF=4`
- **Note**: Requirement is 2, adjust carefully

### `RIPTIDE_MAX_CONCURRENT_WASM`
- **Type**: Integer
- **Default**: `4`
- **Range**: 1-20
- **Description**: Maximum concurrent WASM instances
- **Example**: `RIPTIDE_MAX_CONCURRENT_WASM=8`

### `RIPTIDE_RENDER_TIMEOUT`
- **Type**: Integer (seconds)
- **Default**: `3`
- **Range**: 1-30
- **Description**: Hard timeout for render operations
- **Example**: `RIPTIDE_RENDER_TIMEOUT=5`
- **Note**: 3s recommended for optimal performance

### `RIPTIDE_PDF_TIMEOUT`
- **Type**: Integer (seconds)
- **Default**: `30`
- **Range**: 5-120
- **Description**: PDF processing timeout
- **Example**: `RIPTIDE_PDF_TIMEOUT=60`

### `RIPTIDE_WASM_TIMEOUT`
- **Type**: Integer (seconds)
- **Default**: `10`
- **Range**: 1-60
- **Description**: WASM extraction timeout
- **Example**: `RIPTIDE_WASM_TIMEOUT=15`

### `RIPTIDE_HTTP_TIMEOUT`
- **Type**: Integer (seconds)
- **Default**: `10`
- **Range**: 1-120
- **Description**: HTTP request timeout
- **Example**: `RIPTIDE_HTTP_TIMEOUT=30`

### `RIPTIDE_GLOBAL_TIMEOUT`
- **Type**: Integer (seconds)
- **Default**: `30`
- **Range**: 5-300
- **Description**: Global operation timeout (fallback)
- **Example**: `RIPTIDE_GLOBAL_TIMEOUT=60`

## Rate Limiting

### `RIPTIDE_RATE_LIMIT_ENABLED`
- **Type**: Boolean
- **Default**: `true`
- **Description**: Enable rate limiting
- **Example**: `RIPTIDE_RATE_LIMIT_ENABLED=false`

### `RIPTIDE_RATE_LIMIT_RPS`
- **Type**: Float
- **Default**: `1.5`
- **Range**: 0.1-100.0
- **Description**: Requests per second per host
- **Example**: `RIPTIDE_RATE_LIMIT_RPS=2.0`
- **Note**: 1.5 RPS requirement

### `RIPTIDE_RATE_LIMIT_JITTER`
- **Type**: Float
- **Default**: `0.1`
- **Range**: 0.0-1.0
- **Description**: Jitter factor for rate limiting
- **Example**: `RIPTIDE_RATE_LIMIT_JITTER=0.2`

### `RIPTIDE_RATE_LIMIT_BURST_CAPACITY`
- **Type**: Integer
- **Default**: `3`
- **Range**: 1-20
- **Description**: Burst capacity per host
- **Example**: `RIPTIDE_RATE_LIMIT_BURST_CAPACITY=5`

### `RIPTIDE_RATE_LIMIT_WINDOW_SECS`
- **Type**: Integer (seconds)
- **Default**: `60`
- **Range**: 10-3600
- **Description**: Rate limit window duration
- **Example**: `RIPTIDE_RATE_LIMIT_WINDOW_SECS=120`

### `RIPTIDE_RATE_LIMIT_MAX_HOSTS`
- **Type**: Integer
- **Default**: `10000`
- **Range**: 100-1000000
- **Description**: Maximum tracked hosts
- **Example**: `RIPTIDE_RATE_LIMIT_MAX_HOSTS=50000`

## Browser Pool Configuration

### `RIPTIDE_HEADLESS_POOL_SIZE`
- **Type**: Integer
- **Default**: `3`
- **Range**: 1-10
- **Description**: Maximum browser pool size
- **Example**: `RIPTIDE_HEADLESS_POOL_SIZE=5`
- **Note**: Pool cap = 3 requirement

### `RIPTIDE_HEADLESS_MIN_POOL_SIZE`
- **Type**: Integer
- **Default**: `1`
- **Range**: 1-5
- **Description**: Minimum browser pool size
- **Example**: `RIPTIDE_HEADLESS_MIN_POOL_SIZE=2`

### `RIPTIDE_HEADLESS_IDLE_TIMEOUT`
- **Type**: Integer (seconds)
- **Default**: `300`
- **Range**: 30-3600
- **Description**: Browser idle timeout
- **Example**: `RIPTIDE_HEADLESS_IDLE_TIMEOUT=600`

### `RIPTIDE_HEADLESS_HEALTH_CHECK_INTERVAL`
- **Type**: Integer (seconds)
- **Default**: `60`
- **Range**: 10-300
- **Description**: Browser health check interval
- **Example**: `RIPTIDE_HEADLESS_HEALTH_CHECK_INTERVAL=30`

### `RIPTIDE_HEADLESS_MAX_PAGES_PER_BROWSER`
- **Type**: Integer
- **Default**: `10`
- **Range**: 1-50
- **Description**: Maximum pages per browser instance
- **Example**: `RIPTIDE_HEADLESS_MAX_PAGES_PER_BROWSER=20`

### `RIPTIDE_HEADLESS_RESTART_THRESHOLD`
- **Type**: Integer
- **Default**: `5`
- **Range**: 1-20
- **Description**: Browser restart threshold (failed operations)
- **Example**: `RIPTIDE_HEADLESS_RESTART_THRESHOLD=10`

### `RIPTIDE_HEADLESS_ENABLE_RECYCLING`
- **Type**: Boolean
- **Default**: `true`
- **Description**: Enable browser recycling
- **Example**: `RIPTIDE_HEADLESS_ENABLE_RECYCLING=false`

### `RIPTIDE_HEADLESS_LAUNCH_TIMEOUT`
- **Type**: Integer (seconds)
- **Default**: `30`
- **Range**: 10-120
- **Description**: Browser launch timeout
- **Example**: `RIPTIDE_HEADLESS_LAUNCH_TIMEOUT=60`

### `RIPTIDE_HEADLESS_MAX_RETRIES`
- **Type**: Integer
- **Default**: `3`
- **Range**: 1-10
- **Description**: Maximum retries for browser operations
- **Example**: `RIPTIDE_HEADLESS_MAX_RETRIES=5`

## Memory Management

### `RIPTIDE_MEMORY_LIMIT_MB`
- **Type**: Integer (MB)
- **Default**: `2048`
- **Range**: 512-16384
- **Description**: Global memory limit
- **Example**: `RIPTIDE_MEMORY_LIMIT_MB=4096`

### `RIPTIDE_MEMORY_MAX_PER_REQUEST_MB`
- **Type**: Integer (MB)
- **Default**: `256`
- **Range**: 64-2048
- **Description**: Maximum memory per request
- **Example**: `RIPTIDE_MEMORY_MAX_PER_REQUEST_MB=512`

### `RIPTIDE_MEMORY_PRESSURE_THRESHOLD`
- **Type**: Float
- **Default**: `0.85`
- **Range**: 0.5-0.95
- **Description**: Memory pressure detection threshold
- **Example**: `RIPTIDE_MEMORY_PRESSURE_THRESHOLD=0.90`

### `RIPTIDE_MEMORY_AUTO_GC`
- **Type**: Boolean
- **Default**: `true`
- **Description**: Enable automatic garbage collection
- **Example**: `RIPTIDE_MEMORY_AUTO_GC=false`

### `RIPTIDE_MEMORY_GC_TRIGGER_MB`
- **Type**: Integer (MB)
- **Default**: `1024`
- **Range**: 256-8192
- **Description**: GC trigger threshold
- **Example**: `RIPTIDE_MEMORY_GC_TRIGGER_MB=2048`

### `RIPTIDE_MEMORY_MONITORING_INTERVAL`
- **Type**: Integer (seconds)
- **Default**: `30`
- **Range**: 10-300
- **Description**: Memory monitoring interval
- **Example**: `RIPTIDE_MEMORY_MONITORING_INTERVAL=60`

### `RIPTIDE_MEMORY_LEAK_DETECTION`
- **Type**: Boolean
- **Default**: `true`
- **Description**: Enable memory leak detection
- **Example**: `RIPTIDE_MEMORY_LEAK_DETECTION=false`

### `RIPTIDE_MEMORY_CLEANUP_THRESHOLD_MB`
- **Type**: Integer (MB)
- **Default**: `512`
- **Range**: 128-4096
- **Description**: Memory cleanup threshold
- **Example**: `RIPTIDE_MEMORY_CLEANUP_THRESHOLD_MB=1024`

## PDF Processing

### `RIPTIDE_PDF_MAX_CONCURRENT`
- **Type**: Integer
- **Default**: `2`
- **Range**: 1-10
- **Description**: Maximum concurrent PDF operations
- **Example**: `RIPTIDE_PDF_MAX_CONCURRENT=4`
- **Note**: 2 semaphore requirement

### `RIPTIDE_PDF_PROCESSING_TIMEOUT`
- **Type**: Integer (seconds)
- **Default**: `30`
- **Range**: 10-300
- **Description**: PDF processing timeout
- **Example**: `RIPTIDE_PDF_PROCESSING_TIMEOUT=60`

### `RIPTIDE_PDF_MAX_FILE_SIZE_MB`
- **Type**: Integer (MB)
- **Default**: `100`
- **Range**: 1-1000
- **Description**: Maximum PDF file size
- **Example**: `RIPTIDE_PDF_MAX_FILE_SIZE_MB=200`

### `RIPTIDE_PDF_ENABLE_STREAMING`
- **Type**: Boolean
- **Default**: `true`
- **Description**: Enable PDF streaming processing
- **Example**: `RIPTIDE_PDF_ENABLE_STREAMING=false`

### `RIPTIDE_PDF_QUEUE_SIZE`
- **Type**: Integer
- **Default**: `50`
- **Range**: 10-500
- **Description**: PDF queue size
- **Example**: `RIPTIDE_PDF_QUEUE_SIZE=100`

### `RIPTIDE_PDF_QUEUE_TIMEOUT`
- **Type**: Integer (seconds)
- **Default**: `60`
- **Range**: 10-300
- **Description**: PDF priority queue timeout
- **Example**: `RIPTIDE_PDF_QUEUE_TIMEOUT=120`

## WASM Runtime

### `RIPTIDE_WASM_INSTANCES_PER_WORKER`
- **Type**: Integer
- **Default**: `1`
- **Range**: 1-4
- **Description**: WASM instances per worker
- **Example**: `RIPTIDE_WASM_INSTANCES_PER_WORKER=1`
- **Note**: Single instance per worker requirement

### `RIPTIDE_WASM_MODULE_TIMEOUT`
- **Type**: Integer (seconds)
- **Default**: `10`
- **Range**: 1-60
- **Description**: WASM module timeout
- **Example**: `RIPTIDE_WASM_MODULE_TIMEOUT=20`

### `RIPTIDE_WASM_MAX_MEMORY_MB`
- **Type**: Integer (MB)
- **Default**: `128`
- **Range**: 32-512
- **Description**: Maximum WASM memory
- **Example**: `RIPTIDE_WASM_MAX_MEMORY_MB=256`

### `RIPTIDE_WASM_ENABLE_RECYCLING`
- **Type**: Boolean
- **Default**: `false`
- **Description**: Enable WASM instance recycling
- **Example**: `RIPTIDE_WASM_ENABLE_RECYCLING=true`
- **Note**: Not needed with single instance

### `RIPTIDE_WASM_HEALTH_CHECK_INTERVAL`
- **Type**: Integer (seconds)
- **Default**: `120`
- **Range**: 30-600
- **Description**: WASM instance health check interval
- **Example**: `RIPTIDE_WASM_HEALTH_CHECK_INTERVAL=180`

### `RIPTIDE_WASM_MAX_OPERATIONS_PER_INSTANCE`
- **Type**: Integer
- **Default**: `10000`
- **Range**: 100-100000
- **Description**: Maximum operations per instance
- **Example**: `RIPTIDE_WASM_MAX_OPERATIONS_PER_INSTANCE=20000`

### `RIPTIDE_WASM_RESTART_THRESHOLD`
- **Type**: Integer
- **Default**: `10`
- **Range**: 1-50
- **Description**: WASM instance restart threshold
- **Example**: `RIPTIDE_WASM_RESTART_THRESHOLD=20`

## LLM/AI Providers

### OpenAI
- `OPENAI_API_KEY` - OpenAI API key
- `OPENAI_BASE_URL` - OpenAI API base URL (default: https://api.openai.com/v1)

### Anthropic/Claude
- `ANTHROPIC_API_KEY` - Anthropic API key

### Azure OpenAI
- `AZURE_OPENAI_KEY` - Azure OpenAI key
- `AZURE_OPENAI_ENDPOINT` - Azure OpenAI endpoint

### Ollama (Local LLM)
- `OLLAMA_BASE_URL` - Ollama base URL (default: http://localhost:11434)

## Telemetry & Observability

### `TELEMETRY_ENABLED`
- **Type**: Boolean
- **Default**: `true`
- **Description**: Enable telemetry collection
- **Example**: `TELEMETRY_ENABLED=false`

### `TELEMETRY_SERVICE_NAME`
- **Type**: String
- **Default**: `riptide-api`
- **Description**: Service name for telemetry
- **Example**: `TELEMETRY_SERVICE_NAME=riptide-prod`

### `TELEMETRY_EXPORTER_TYPE`
- **Type**: Enum
- **Default**: `stdout`
- **Options**: `otlp`, `stdout`
- **Description**: Telemetry exporter type
- **Example**: `TELEMETRY_EXPORTER_TYPE=otlp`

### `TELEMETRY_SAMPLING_RATIO`
- **Type**: Float
- **Default**: `1.0`
- **Range**: 0.0-1.0
- **Description**: Sampling ratio
- **Example**: `TELEMETRY_SAMPLING_RATIO=0.1`

### `TELEMETRY_EXPORT_TIMEOUT_SECS`
- **Type**: Integer (seconds)
- **Default**: `30`
- **Range**: 1-120
- **Description**: Export timeout
- **Example**: `TELEMETRY_EXPORT_TIMEOUT_SECS=60`

## Authentication & Security

### `REQUIRE_AUTH`
- **Type**: Boolean
- **Default**: `false`
- **Description**: Require authentication
- **Example**: `REQUIRE_AUTH=true`

### `API_KEYS`
- **Type**: String (comma-separated)
- **Description**: Valid API keys
- **Example**: `API_KEYS=key1,key2,key3`
- **Security**: Never commit to version control

### TLS Configuration
- `RIPTIDE_ENABLE_TLS` - Enable HTTPS/TLS
- `RIPTIDE_TLS_CERT_PATH` - TLS certificate path
- `RIPTIDE_TLS_KEY_PATH` - TLS key path

## Spider/Crawler

### `SPIDER_ENABLE`
- **Type**: Boolean
- **Default**: `false`
- **Description**: Enable spider/crawler functionality
- **Example**: `SPIDER_ENABLE=true`

### `SPIDER_BASE_URL`
- **Type**: URL
- **Required**: If `SPIDER_ENABLE=true`
- **Description**: Base URL for spider operations
- **Example**: `SPIDER_BASE_URL=https://example.com`

### `SPIDER_MAX_DEPTH`
- **Type**: Integer
- **Default**: `3`
- **Range**: 1-10
- **Description**: Maximum crawl depth
- **Example**: `SPIDER_MAX_DEPTH=5`

### `SPIDER_MAX_PAGES`
- **Type**: Integer
- **Default**: `100`
- **Range**: 1-10000
- **Description**: Maximum pages to crawl
- **Example**: `SPIDER_MAX_PAGES=1000`

### `SPIDER_CONCURRENCY`
- **Type**: Integer
- **Default**: `4`
- **Range**: 1-20
- **Description**: Concurrent requests
- **Example**: `SPIDER_CONCURRENCY=8`

### `SPIDER_TIMEOUT_SECONDS`
- **Type**: Integer (seconds)
- **Default**: `30`
- **Range**: 5-300
- **Description**: Request timeout
- **Example**: `SPIDER_TIMEOUT_SECONDS=60`

### `SPIDER_DELAY_MS`
- **Type**: Integer (milliseconds)
- **Default**: `500`
- **Range**: 0-5000
- **Description**: Delay between requests
- **Example**: `SPIDER_DELAY_MS=1000`

### `SPIDER_RESPECT_ROBOTS`
- **Type**: Boolean
- **Default**: `true`
- **Description**: Respect robots.txt
- **Example**: `SPIDER_RESPECT_ROBOTS=false`

### `SPIDER_USER_AGENT`
- **Type**: String
- **Default**: `RipTide Spider/1.0`
- **Description**: User agent string
- **Example**: `SPIDER_USER_AGENT=Custom Bot/2.0`

## Worker Configuration

### `WORKER_POOL_SIZE`
- **Type**: Integer
- **Default**: `4`
- **Range**: 1-32
- **Description**: Worker pool size
- **Example**: `WORKER_POOL_SIZE=8`

### `WORKER_MAX_BATCH_SIZE`
- **Type**: Integer
- **Default**: `100`
- **Range**: 10-1000
- **Description**: Maximum batch size for worker operations
- **Example**: `WORKER_MAX_BATCH_SIZE=200`

### `WORKER_MAX_CONCURRENCY`
- **Type**: Integer
- **Default**: `10`
- **Range**: 1-50
- **Description**: Maximum concurrency for workers
- **Example**: `WORKER_MAX_CONCURRENCY=20`

### `WORKER_ENABLE_SCHEDULER`
- **Type**: Boolean
- **Default**: `false`
- **Description**: Enable worker scheduler
- **Example**: `WORKER_ENABLE_SCHEDULER=true`

### `RIPTIDE_WORKER_TIMEOUT`
- **Type**: Integer (seconds)
- **Default**: `60`
- **Range**: 10-600
- **Description**: Worker timeout
- **Example**: `RIPTIDE_WORKER_TIMEOUT=120`

### `RIPTIDE_WORKER_MAX_RETRIES`
- **Type**: Integer
- **Default**: `3`
- **Range**: 0-10
- **Description**: Worker retry attempts
- **Example**: `RIPTIDE_WORKER_MAX_RETRIES=5`

## Cache & Persistence

### `CACHE_TTL`
- **Type**: Integer (seconds)
- **Default**: `86400`
- **Range**: 60-604800
- **Description**: Cache TTL
- **Example**: `CACHE_TTL=3600`

### `ENABLE_COMPRESSION`
- **Type**: Boolean
- **Default**: `true`
- **Description**: Enable cache compression
- **Example**: `ENABLE_COMPRESSION=false`

### `ENABLE_MULTI_TENANCY`
- **Type**: Boolean
- **Default**: `false`
- **Description**: Enable multi-tenancy
- **Example**: `ENABLE_MULTI_TENANCY=true`

### `RIPTIDE_CACHE_INVALIDATION_INTERVAL`
- **Type**: Integer (seconds)
- **Default**: `300`
- **Range**: 60-3600
- **Description**: Cache invalidation interval
- **Example**: `RIPTIDE_CACHE_INVALIDATION_INTERVAL=600`

### `RIPTIDE_CACHE_WARMING_ENABLED`
- **Type**: Boolean
- **Default**: `false`
- **Description**: Enable cache warming
- **Example**: `RIPTIDE_CACHE_WARMING_ENABLED=true`

## Circuit Breaker

### `CIRCUIT_BREAKER_FAILURE_THRESHOLD`
- **Type**: Integer (percentage)
- **Default**: `50`
- **Range**: 0-100
- **Description**: Failure threshold
- **Example**: `CIRCUIT_BREAKER_FAILURE_THRESHOLD=75`

### `CIRCUIT_BREAKER_TIMEOUT_MS`
- **Type**: Integer (milliseconds)
- **Default**: `5000`
- **Range**: 100-30000
- **Description**: Circuit breaker timeout
- **Example**: `CIRCUIT_BREAKER_TIMEOUT_MS=10000`

### `CIRCUIT_BREAKER_MIN_REQUESTS`
- **Type**: Integer
- **Default**: `5`
- **Range**: 1-100
- **Description**: Minimum requests before opening
- **Example**: `CIRCUIT_BREAKER_MIN_REQUESTS=10`

### `CIRCUIT_BREAKER_RECOVERY_TIMEOUT`
- **Type**: Integer (seconds)
- **Default**: `60`
- **Range**: 10-600
- **Description**: Recovery timeout
- **Example**: `CIRCUIT_BREAKER_RECOVERY_TIMEOUT=120`

## Streaming

### `STREAM_BUFFER_SIZE`
- **Type**: Integer (bytes)
- **Default**: `8192`
- **Range**: 1024-65536
- **Description**: Stream buffer size
- **Example**: `STREAM_BUFFER_SIZE=16384`

### `STREAM_BUFFER_MAX_SIZE`
- **Type**: Integer (bytes)
- **Default**: `65536`
- **Range**: 8192-1048576
- **Description**: Maximum stream buffer size
- **Example**: `STREAM_BUFFER_MAX_SIZE=131072`

### `WS_MAX_MESSAGE_SIZE`
- **Type**: Integer (bytes)
- **Default**: `16777216`
- **Range**: 1024-67108864
- **Description**: WebSocket maximum message size
- **Example**: `WS_MAX_MESSAGE_SIZE=33554432`

### `WS_PING_INTERVAL`
- **Type**: Integer (seconds)
- **Default**: `30`
- **Range**: 10-300
- **Description**: WebSocket ping interval
- **Example**: `WS_PING_INTERVAL=60`

### `STREAM_MAX_CONCURRENT`
- **Type**: Integer
- **Default**: `100`
- **Range**: 10-1000
- **Description**: Maximum concurrent streams
- **Example**: `STREAM_MAX_CONCURRENT=200`

### `STREAM_DEFAULT_TIMEOUT`
- **Type**: Integer (seconds)
- **Default**: `300`
- **Range**: 30-3600
- **Description**: Default stream timeout
- **Example**: `STREAM_DEFAULT_TIMEOUT=600`

### `STREAM_RATE_LIMIT_ENABLED`
- **Type**: Boolean
- **Default**: `true`
- **Description**: Enable stream rate limiting
- **Example**: `STREAM_RATE_LIMIT_ENABLED=false`

### `STREAM_RATE_LIMIT_RPS`
- **Type**: Integer
- **Default**: `10`
- **Range**: 1-100
- **Description**: Stream rate limit (RPS)
- **Example**: `STREAM_RATE_LIMIT_RPS=20`

## Development & Testing

### `RUST_LOG`
- **Type**: Enum
- **Default**: `info`
- **Options**: `error`, `warn`, `info`, `debug`, `trace`
- **Description**: Logging level
- **Example**: `RUST_LOG=debug`

### `RIPTIDE_DEV_MODE`
- **Type**: Boolean
- **Default**: `false`
- **Description**: Enable development mode
- **Example**: `RIPTIDE_DEV_MODE=true`

### `HEALTH_CHECK_PORT`
- **Type**: Integer
- **Default**: API port
- **Range**: 1-65535
- **Description**: Health check port
- **Example**: `HEALTH_CHECK_PORT=8081`

### Feature Flags
- `RIPTIDE_FEATURE_PDF` - Enable PDF features
- `RIPTIDE_FEATURE_BENCHMARKS` - Enable benchmarks
- `RIPTIDE_FEATURE_API_INTEGRATION` - Enable API integration

### Test Configuration
- `TEST_REDIS_URL` - Redis URL for tests
- `TEST_WASM_PATH` - WASM path for tests
- `SKIP_PERSISTENCE_TESTS` - Skip persistence tests
- `SKIP_REDIS_TESTS` - Skip Redis tests
- `TEST_TIMEOUT_MULTIPLIER` - Test timeout multiplier

## Validation Rules

### Path Variables
- Must be absolute or relative paths
- Parent directories must exist
- Must have write permissions

### URL Variables
- Must be valid HTTP/HTTPS URLs
- Must be reachable (for service URLs)

### Timeout Variables
- Must be positive integers
- Should be reasonable for operation type
- Consider network latency

### Limit Variables
- Must be positive integers
- Consider system resources
- Balance performance vs resource usage

### Boolean Variables
- Accept: `true`, `false`, `1`, `0`, `yes`, `no`
- Case-insensitive

### Range Constraints
All numeric values must be within specified ranges. Values outside ranges will:
1. Log a warning
2. Use the closest valid value (clamped)
3. Or fail validation (depending on criticality)

## Usage Examples

### Basic Setup
```bash
# Copy example file
cp .env.example .env

# Edit with your values
nano .env

# Validate configuration
./scripts/validate-env.sh
```

### Production Configuration
```bash
# High-performance production settings
RIPTIDE_API_URL=https://api.riptide.example.com
RIPTIDE_MAX_CONCURRENT_RENDERS=20
RIPTIDE_MEMORY_LIMIT_MB=8192
RIPTIDE_HEADLESS_POOL_SIZE=5
REQUIRE_AUTH=true
TELEMETRY_ENABLED=true
```

### Development Configuration
```bash
# Local development settings
RIPTIDE_API_URL=http://localhost:8080
RIPTIDE_CLI_MODE=direct
RIPTIDE_CLI_VERBOSE=true
RUST_LOG=debug
RIPTIDE_DEV_MODE=true
```

### Testing Configuration
```bash
# Testing environment
RIPTIDE_API_URL=http://localhost:8080
SKIP_REDIS_TESTS=false
TEST_TIMEOUT_MULTIPLIER=2.0
RUST_LOG=trace
```

## Troubleshooting

### Common Issues

**Output directories not found**
```bash
# Run setup script to create directories
./scripts/setup-env.sh
```

**Invalid timeout values**
```bash
# Check validation
./scripts/validate-env.sh

# Adjust to valid ranges
RIPTIDE_RENDER_TIMEOUT=3  # Not 0 or negative
```

**Memory pressure**
```bash
# Increase limits or reduce concurrent operations
RIPTIDE_MEMORY_LIMIT_MB=4096
RIPTIDE_MAX_CONCURRENT_RENDERS=5
```

**Rate limiting issues**
```bash
# Adjust rate limits
RIPTIDE_RATE_LIMIT_RPS=2.0
RIPTIDE_RATE_LIMIT_JITTER=0.2
```

## See Also

- [Configuration Guide](../guides/CONFIGURATION.md)
- [Setup Guide](../guides/SETUP.md)
- [API Documentation](../API.md)
- [CLI Reference](../CLI.md)
