# RipTide Configuration Guide

This directory contains comprehensive documentation for configuring RipTide.

## Quick Start

1. **Copy environment template:**
   ```bash
   cp .env.example .env
   ```

2. **Edit configuration:**
   ```bash
   nano .env
   ```

3. **Setup directories:**
   ```bash
   ./scripts/setup-env.sh
   ```

4. **Validate configuration:**
   ```bash
   ./scripts/validate-env.sh
   ```

## Documentation

### Primary Reference
- **[ENVIRONMENT_VARIABLES.md](./ENVIRONMENT_VARIABLES.md)** - Complete environment variable reference
  - All available variables
  - Types, defaults, and ranges
  - Validation rules
  - Usage examples

### Helper Scripts

#### `scripts/setup-env.sh`
Creates and validates output directories.

```bash
# Basic setup
./scripts/setup-env.sh

# Check only (no changes)
./scripts/setup-env.sh --check-only

# Verbose output
./scripts/setup-env.sh --verbose
```

#### `scripts/validate-env.sh`
Validates environment configuration.

```bash
# Basic validation
./scripts/validate-env.sh

# Strict mode (warnings as errors)
./scripts/validate-env.sh --strict

# JSON output
./scripts/validate-env.sh --json

# Auto-fix common issues
./scripts/validate-env.sh --fix
```

## Configuration Sections

### Output Directories
Configure where RipTide stores artifacts:
- Screenshots
- HTML content
- PDFs
- Reports
- Logs
- Cache

**Base directory:** `RIPTIDE_OUTPUT_DIR=./riptide-output`

### CLI Configuration
- `RIPTIDE_API_URL` - API endpoint
- `RIPTIDE_API_KEY` - Authentication key
- `RIPTIDE_CLI_MODE` - Operation mode (api_first, api_only, direct)

### Performance Tuning
- `RIPTIDE_MAX_CONCURRENT_RENDERS=10` - Concurrent renders
- `RIPTIDE_RENDER_TIMEOUT=3` - Render timeout (3s recommended)
- `RIPTIDE_HEADLESS_POOL_SIZE=3` - Browser pool (3 cap recommended)
- `RIPTIDE_MEMORY_LIMIT_MB=2048` - Memory limit

### Rate Limiting
- `RIPTIDE_RATE_LIMIT_RPS=1.5` - Requests per second (1.5 requirement)
- `RIPTIDE_RATE_LIMIT_JITTER=0.1` - Jitter factor (0-1)

### Search Integration
- `SEARCH_BACKEND` - Provider (serper, none, searxng)
- `SERPER_API_KEY` - Serper.dev API key
- `SEARXNG_BASE_URL` - SearXNG instance URL

### PDF Processing
- `RIPTIDE_MAX_CONCURRENT_PDF=2` - Semaphore limit (2 requirement)
- `RIPTIDE_PDF_TIMEOUT=30` - Processing timeout
- `RIPTIDE_PDF_MAX_FILE_SIZE_MB=100` - Max file size

### WASM Runtime
- `RIPTIDE_WASM_INSTANCES_PER_WORKER=1` - Single instance requirement
- `RIPTIDE_WASM_TIMEOUT=10` - Extraction timeout
- `RIPTIDE_WASM_PATH` - Module path

## Environment Naming Conventions

All RipTide variables follow these patterns:

| Pattern | Usage | Example |
|---------|-------|---------|
| `RIPTIDE_*` | All RipTide variables | `RIPTIDE_API_URL` |
| `RIPTIDE_*_DIR` | Output directories | `RIPTIDE_SCREENSHOTS_DIR` |
| `RIPTIDE_*_URL` | Service URLs | `RIPTIDE_API_URL` |
| `RIPTIDE_*_TIMEOUT` | Timeout values | `RIPTIDE_RENDER_TIMEOUT` |
| `RIPTIDE_MAX_*` | Maximum limits | `RIPTIDE_MAX_CONCURRENT_RENDERS` |

## Configuration Profiles

### Production
```bash
# High-performance, secure settings
RIPTIDE_API_URL=https://api.riptide.example.com
RIPTIDE_MAX_CONCURRENT_RENDERS=20
RIPTIDE_MEMORY_LIMIT_MB=8192
RIPTIDE_HEADLESS_POOL_SIZE=5
REQUIRE_AUTH=true
TELEMETRY_ENABLED=true
RUST_LOG=info
```

### Development
```bash
# Local development
RIPTIDE_API_URL=http://localhost:8080
RIPTIDE_CLI_MODE=direct
RIPTIDE_CLI_VERBOSE=true
RUST_LOG=debug
RIPTIDE_DEV_MODE=true
```

### Testing
```bash
# Testing environment
RIPTIDE_API_URL=http://localhost:8080
TEST_TIMEOUT_MULTIPLIER=2.0
RUST_LOG=trace
SKIP_REDIS_TESTS=false
```

## Validation Rules

### Required When
- `RIPTIDE_API_KEY` - Required if `REQUIRE_AUTH=true`
- `SERPER_API_KEY` - Required if `SEARCH_BACKEND=serper`
- `SEARXNG_BASE_URL` - Required if `SEARCH_BACKEND=searxng`
- `SPIDER_BASE_URL` - Required if `SPIDER_ENABLE=true`

### Valid Ranges
- Timeouts: 1-300 seconds
- Memory limits: 512-16384 MB
- Concurrent operations: 1-100
- Rate limits: 0.1-100.0 RPS
- Pool sizes: 1-10

### Type Constraints
- URLs: Must be valid HTTP/HTTPS
- Paths: Must be valid filesystem paths
- Booleans: `true`, `false`, `1`, `0`, `yes`, `no`
- Enums: Must match specified options

## Troubleshooting

### Common Issues

**Validation Errors**
```bash
# Check specific issues
./scripts/validate-env.sh --verbose

# Auto-fix common problems
./scripts/validate-env.sh --fix
```

**Directory Permission Issues**
```bash
# Run setup with verbose output
./scripts/setup-env.sh --verbose

# Check specific directory
ls -la ./riptide-output
```

**Configuration Conflicts**
```bash
# Strict validation
./scripts/validate-env.sh --strict

# JSON output for parsing
./scripts/validate-env.sh --json | jq
```

### Debug Mode
```bash
# Enable debug logging
RUST_LOG=debug

# CLI verbose mode
RIPTIDE_CLI_VERBOSE=true

# Development mode
RIPTIDE_DEV_MODE=true
```

## Security Best Practices

1. **Never commit secrets:**
   - `.env` is in `.gitignore`
   - Use `.env.example` for templates
   - Rotate API keys regularly

2. **Strong authentication:**
   - Use long API keys (16+ chars)
   - Enable `REQUIRE_AUTH=true` in production
   - Configure TLS/HTTPS

3. **Resource limits:**
   - Set appropriate memory limits
   - Configure rate limiting
   - Monitor resource usage

4. **Network security:**
   - Use HTTPS for production
   - Configure proxy settings if needed
   - Whitelist allowed domains

## Performance Optimization

### Memory Management
```bash
# Increase global limit
RIPTIDE_MEMORY_LIMIT_MB=4096

# Per-request limit
RIPTIDE_MEMORY_MAX_PER_REQUEST_MB=512

# Enable auto GC
RIPTIDE_MEMORY_AUTO_GC=true
```

### Concurrency Tuning
```bash
# Increase concurrent renders
RIPTIDE_MAX_CONCURRENT_RENDERS=20

# Browser pool size
RIPTIDE_HEADLESS_POOL_SIZE=5

# Worker pool
WORKER_POOL_SIZE=8
```

### Timeout Optimization
```bash
# Fast timeouts for performance
RIPTIDE_RENDER_TIMEOUT=3
RIPTIDE_HTTP_TIMEOUT=10
RIPTIDE_GLOBAL_TIMEOUT=30
```

## Monitoring & Observability

### Telemetry
```bash
# Enable telemetry
TELEMETRY_ENABLED=true

# Configure exporter
TELEMETRY_EXPORTER_TYPE=otlp
TELEMETRY_OTLP_ENDPOINT=http://localhost:4317

# Sampling
TELEMETRY_SAMPLING_RATIO=1.0
```

### Logging
```bash
# Log levels
RUST_LOG=info  # production
RUST_LOG=debug  # development
RUST_LOG=trace  # deep debugging

# Log directory
RIPTIDE_LOGS_DIR=./riptide-output/logs
```

## Advanced Configuration

### Multi-Tenancy
```bash
ENABLE_MULTI_TENANCY=true
```

### Cache Warming
```bash
RIPTIDE_CACHE_WARMING_ENABLED=true
CACHE_TTL=86400
```

### Worker Scheduling
```bash
WORKER_ENABLE_SCHEDULER=true
WORKER_MAX_BATCH_SIZE=100
```

### Circuit Breaker
```bash
CIRCUIT_BREAKER_FAILURE_THRESHOLD=50
CIRCUIT_BREAKER_TIMEOUT_MS=5000
CIRCUIT_BREAKER_MIN_REQUESTS=5
```

## See Also

- [Environment Variables Reference](./ENVIRONMENT_VARIABLES.md) - Complete variable documentation
- [API Documentation](../API.md) - API reference
- [CLI Guide](../CLI.md) - CLI usage
- [Setup Guide](../guides/SETUP.md) - Initial setup
