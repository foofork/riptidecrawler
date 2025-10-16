# Spider Engine - Quick Start Guide

## Enable Spider in 3 Steps

### 1. Set Environment Variable

```bash
export SPIDER_ENABLE=true
```

### 2. Start API Server

```bash
cargo run --bin riptide-api
```

### 3. Test Spider

```bash
curl -X POST http://localhost:8080/api/v1/spider/crawl \
  -H "Content-Type: application/json" \
  -d '{
    "seed_urls": ["https://example.com"],
    "max_depth": 2,
    "max_pages": 50
  }'
```

## Configuration Cheat Sheet

### Essential Variables

```bash
SPIDER_ENABLE=true              # Enable Spider (required)
SPIDER_MAX_DEPTH=3              # Crawl depth (default: 10)
SPIDER_MAX_PAGES=100            # Page limit (default: 1000)
SPIDER_CONCURRENCY=4            # Parallel requests (default: 4)
```

### Performance Tuning

```bash
# Fast crawling
SPIDER_DELAY_MS=50              # Fast delay (default: 500)
SPIDER_CONCURRENCY=16           # High concurrency

# Polite crawling
SPIDER_DELAY_MS=2000            # Slow delay
SPIDER_CONCURRENCY=2            # Low concurrency
SPIDER_RESPECT_ROBOTS=true      # Respect robots.txt
```

## Common Use Cases

### Development/Testing

```bash
SPIDER_ENABLE=true
SPIDER_MAX_DEPTH=2
SPIDER_MAX_PAGES=50
SPIDER_CONCURRENCY=2
SPIDER_DELAY_MS=100
```

### Production Crawling

```bash
SPIDER_ENABLE=true
SPIDER_MAX_DEPTH=5
SPIDER_MAX_PAGES=1000
SPIDER_CONCURRENCY=8
SPIDER_DELAY_MS=500
SPIDER_RESPECT_ROBOTS=true
```

### Aggressive Crawling

```bash
SPIDER_ENABLE=true
SPIDER_MAX_DEPTH=10
SPIDER_MAX_PAGES=10000
SPIDER_CONCURRENCY=16
SPIDER_DELAY_MS=50
SPIDER_TIMEOUT_SECONDS=20
```

## API Endpoints

### Spider Crawl

```bash
POST /api/v1/spider/crawl
{
  "seed_urls": ["https://example.com"],
  "max_depth": 3,
  "max_pages": 100,
  "strategy": "breadth_first"
}
```

### Regular Crawl with Spider

```bash
POST /api/v1/crawl
{
  "urls": ["https://example.com"],
  "options": {
    "use_spider": true
  }
}
```

### Spider Status

```bash
POST /api/v1/spider/status
{
  "include_metrics": true
}
```

## CLI Usage

```bash
# Basic crawl
riptide crawl https://example.com --depth 3 --max-pages 100

# With output directory
riptide crawl https://example.com \
  --depth 3 \
  --max-pages 100 \
  --output-dir ./results

# JSON output
riptide crawl https://example.com \
  --depth 3 \
  --max-pages 100 \
  --output json
```

## Troubleshooting

### "Spider engine is not enabled"

```bash
# Solution: Enable Spider
export SPIDER_ENABLE=true
cargo run --bin riptide-api
```

### Invalid Configuration

```bash
# Check current settings
env | grep SPIDER_

# View debug logs
RUST_LOG=debug cargo run --bin riptide-api
```

### Schema Mismatch

The CLI now supports both response formats automatically. Update to latest version if issues persist.

## Docker Setup

```yaml
# docker-compose.yml
services:
  riptide-api:
    environment:
      - SPIDER_ENABLE=true
      - SPIDER_MAX_DEPTH=3
      - SPIDER_MAX_PAGES=100
      - REDIS_URL=redis://redis:6379/0
```

## Verification

### Check Logs

Look for:
```
INFO riptide_api::state: Initializing Spider engine (SPIDER_ENABLE=true)
INFO riptide_api::state: Spider configuration initialized successfully
```

### Test Status Endpoint

```bash
curl -X POST http://localhost:8080/api/v1/spider/status \
  -H "Content-Type: application/json" \
  -d '{"include_metrics": true}' | jq
```

## Complete Documentation

- **Full Guide:** [SPIDER_CONFIGURATION_GUIDE.md](./SPIDER_CONFIGURATION_GUIDE.md)
- **Implementation:** [SPIDER_IMPLEMENTATION_SUMMARY.md](./SPIDER_IMPLEMENTATION_SUMMARY.md)
- **Architecture:** [../crates/riptide-core/src/spider/architecture.md](../crates/riptide-core/src/spider/architecture.md)

## Support

1. Review logs with `RUST_LOG=debug`
2. Check configuration with `env | grep SPIDER_`
3. Test with minimal settings first
4. See full documentation for detailed troubleshooting
