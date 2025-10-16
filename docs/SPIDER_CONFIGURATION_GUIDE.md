# Spider Engine Configuration Guide

## Overview

The Spider engine provides deep crawling capabilities for RipTide, enabling systematic exploration of websites with frontier-based URL management, multiple crawling strategies, and adaptive stopping mechanisms.

## Quick Start

### 1. Enable Spider Engine

Set the following environment variable:

```bash
export SPIDER_ENABLE=true
```

Or in your `.env` file:

```bash
SPIDER_ENABLE=true
```

### 2. Basic Configuration

Minimal configuration for Spider:

```bash
# Enable Spider
SPIDER_ENABLE=true

# Base URL (used as fallback for relative URLs)
SPIDER_BASE_URL=https://example.com

# Crawl limits
SPIDER_MAX_DEPTH=3
SPIDER_MAX_PAGES=100
SPIDER_CONCURRENCY=4
```

### 3. Start the API Server

```bash
cargo run --bin riptide-api
```

You should see log output indicating Spider is enabled:

```
INFO riptide_api::state: Initializing Spider engine (SPIDER_ENABLE=true)
INFO riptide_api::state: Spider configuration initialized successfully from environment variables max_depth=Some(3) max_pages=Some(100) concurrency=4
```

## Configuration Options

### Core Settings

| Variable | Default | Description |
|----------|---------|-------------|
| `SPIDER_ENABLE` | `false` | **Must be `true`** to enable Spider engine |
| `SPIDER_BASE_URL` | `https://example.com` | Base URL for resolving relative links |
| `SPIDER_MAX_DEPTH` | `10` | Maximum crawl depth from seed URLs |
| `SPIDER_MAX_PAGES` | `1000` | Maximum number of pages to crawl |
| `SPIDER_CONCURRENCY` | `4` | Number of concurrent requests |

### Performance Settings

| Variable | Default | Description |
|----------|---------|-------------|
| `SPIDER_TIMEOUT_SECONDS` | `30` | Request timeout in seconds |
| `SPIDER_DELAY_MS` | `500` | Delay between requests (milliseconds) |
| `SPIDER_RESPECT_ROBOTS` | `true` | Respect robots.txt directives |
| `SPIDER_USER_AGENT` | `RipTide Spider/1.0` | User agent string for requests |

## Usage Examples

### Example 1: Basic Spider Crawl

```bash
# Configuration
export SPIDER_ENABLE=true
export SPIDER_MAX_DEPTH=2
export SPIDER_MAX_PAGES=50

# Start API server
cargo run --bin riptide-api

# Make request to Spider endpoint
curl -X POST http://localhost:8080/api/v1/spider/crawl \
  -H "Content-Type: application/json" \
  -d '{
    "seed_urls": ["https://example.com"],
    "max_depth": 2,
    "max_pages": 50,
    "strategy": "breadth_first"
  }'
```

### Example 2: Using Spider via Regular Crawl Endpoint

```bash
# Use the regular crawl endpoint with use_spider option
curl -X POST http://localhost:8080/api/v1/crawl \
  -H "Content-Type: application/json" \
  -d '{
    "urls": ["https://example.com"],
    "options": {
      "use_spider": true,
      "concurrency": 4
    }
  }'
```

### Example 3: CLI Crawl with Spider

```bash
# Configure Spider
export SPIDER_ENABLE=true
export SPIDER_MAX_DEPTH=3
export SPIDER_MAX_PAGES=100

# Use riptide-cli to crawl
riptide crawl https://example.com \
  --depth 3 \
  --max-pages 100 \
  --output-dir ./crawl_results
```

## Response Format

### Spider Crawl Response

```json
{
  "result": {
    "pages_crawled": 42,
    "pages_failed": 3,
    "duration_seconds": 12.5,
    "stop_reason": "max_pages_reached",
    "domains": ["example.com", "www.example.com"]
  },
  "state": {
    "active": false,
    "start_time": "2024-01-01T00:00:00Z",
    "pages_crawled": 42,
    "pages_failed": 3,
    "current_depth": 3
  },
  "performance": {
    "avg_request_duration_ms": 234,
    "requests_per_second": 3.36,
    "frontier_size": 0,
    "duplicate_urls_filtered": 128
  }
}
```

### Regular Crawl Response (with Spider)

```json
{
  "total_urls": 1,
  "successful": 42,
  "failed": 3,
  "from_cache": 0,
  "results": [
    {
      "url": "https://example.com/page1",
      "status": 200,
      "from_cache": false,
      "gate_decision": "spider_crawl",
      "quality_score": 0.8,
      "processing_time_ms": 234,
      "document": null,
      "error": null,
      "cache_key": "spider_0"
    }
  ],
  "statistics": {
    "total_processing_time_ms": 12500,
    "avg_processing_time_ms": 297.6,
    "gate_decisions": {
      "raw": 0,
      "probes_first": 0,
      "headless": 0,
      "cached": 0
    },
    "cache_hit_rate": 0.0
  }
}
```

## Crawling Strategies

Spider supports multiple crawling strategies:

### 1. Breadth-First (Default)

Explores all pages at depth N before moving to depth N+1.

```json
{
  "strategy": "breadth_first"
}
```

**Best for:** Site structure analysis, finding content at specific depths

### 2. Depth-First

Explores each path completely before backtracking.

```json
{
  "strategy": "depth_first"
}
```

**Best for:** Deep content discovery, following specific paths

### 3. Best-First

Prioritizes URLs based on content scoring.

```json
{
  "strategy": "best_first"
}
```

**Best for:** Targeted content extraction, relevance-based crawling

## Advanced Configuration

### Resource Optimization

For low-resource environments:

```bash
SPIDER_ENABLE=true
SPIDER_CONCURRENCY=2
SPIDER_MAX_PAGES=50
SPIDER_DELAY_MS=1000
SPIDER_TIMEOUT_SECONDS=20
```

### High-Performance Crawling

For aggressive crawling:

```bash
SPIDER_ENABLE=true
SPIDER_CONCURRENCY=16
SPIDER_MAX_PAGES=10000
SPIDER_DELAY_MS=50
SPIDER_TIMEOUT_SECONDS=30
```

### Respectful Crawling

For polite, robots.txt-compliant crawling:

```bash
SPIDER_ENABLE=true
SPIDER_RESPECT_ROBOTS=true
SPIDER_DELAY_MS=2000
SPIDER_CONCURRENCY=2
SPIDER_USER_AGENT="RipTide Spider/1.0 (+https://example.com/bot)"
```

## Integration with Redis

Spider uses Redis for:
- URL frontier queue management
- Visited URL tracking
- Session persistence
- Crawl state management

Ensure Redis is configured:

```bash
REDIS_URL=redis://localhost:6379/0
```

## Monitoring and Debugging

### Enable Debug Logging

```bash
RUST_LOG=riptide_api=debug,riptide_core::spider=debug
```

### Check Spider Status

```bash
curl -X POST http://localhost:8080/api/v1/spider/status \
  -H "Content-Type: application/json" \
  -d '{"include_metrics": true}'
```

### Monitor Logs

Look for these log messages:

```
INFO riptide_api::state: Initializing Spider engine (SPIDER_ENABLE=true)
DEBUG riptide_api::state: Spider base URL: https://example.com
DEBUG riptide_api::state: Spider max depth: 3
DEBUG riptide_api::state: Spider max pages: 100
INFO riptide_api::state: Spider configuration initialized successfully
```

## Troubleshooting

### Problem: "Spider engine is not enabled"

**Solution:** Set `SPIDER_ENABLE=true` before starting the API server.

```bash
export SPIDER_ENABLE=true
cargo run --bin riptide-api
```

### Problem: Invalid Spider configuration

**Causes:**
- `SPIDER_CONCURRENCY=0` (must be > 0)
- `SPIDER_MAX_DEPTH=0` (must be > 0)
- `SPIDER_TIMEOUT_SECONDS=0` (must be > 0)
- Invalid `SPIDER_BASE_URL`

**Solution:** Check logs for validation errors:

```
ERROR riptide_api::state: Invalid Spider configuration: Concurrency must be greater than 0
```

### Problem: CLI response schema mismatch

**Solution:** The CLI now supports both response formats automatically. Update to the latest version if you see deserialization errors.

### Problem: Spider not crawling beyond seed URL

**Possible causes:**
- `SPIDER_MAX_DEPTH=1` (only crawls seed URLs)
- `SPIDER_MAX_PAGES` reached immediately
- robots.txt blocking crawl
- No valid links found on seed page

**Debugging:**
```bash
RUST_LOG=riptide_core::spider=debug cargo run --bin riptide-api
```

## Docker Configuration

### docker-compose.yml

```yaml
services:
  riptide-api:
    image: riptide-api:latest
    environment:
      - SPIDER_ENABLE=true
      - SPIDER_MAX_DEPTH=3
      - SPIDER_MAX_PAGES=100
      - SPIDER_CONCURRENCY=4
      - SPIDER_TIMEOUT_SECONDS=30
      - SPIDER_DELAY_MS=500
      - SPIDER_RESPECT_ROBOTS=true
      - REDIS_URL=redis://redis:6379/0
    depends_on:
      - redis
    ports:
      - "8080:8080"

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
```

## Performance Considerations

### Memory Usage

Spider memory usage depends on:
- Frontier size (pending URLs)
- URL deduplication tracking
- Session state

**Formula:**
```
Memory ≈ 1MB base + (max_pages × 256 bytes) + (bloom_filter_capacity × 8 bits)
```

**Example:** 1000 pages ≈ 1MB + 256KB + bloom filter

### Throughput

Expected throughput depends on:
- `SPIDER_CONCURRENCY`: Higher = more throughput
- `SPIDER_DELAY_MS`: Lower = more throughput
- Network latency
- Target site performance

**Example:** With concurrency=4 and delay=500ms:
- Theoretical max: 8 requests/second
- Practical: 4-6 requests/second (accounting for processing)

### Optimization Tips

1. **Increase concurrency** for high-latency sites
2. **Decrease delay** for fast, permissive sites
3. **Lower max_pages** for focused crawls
4. **Use best_first strategy** for targeted content
5. **Enable URL normalization** to reduce duplicates

## Security Best Practices

1. **Set reasonable limits:**
   ```bash
   SPIDER_MAX_PAGES=1000  # Prevent runaway crawls
   SPIDER_TIMEOUT_SECONDS=30  # Prevent hung requests
   ```

2. **Respect robots.txt:**
   ```bash
   SPIDER_RESPECT_ROBOTS=true
   ```

3. **Use polite delays:**
   ```bash
   SPIDER_DELAY_MS=1000  # 1 second between requests
   ```

4. **Identify your bot:**
   ```bash
   SPIDER_USER_AGENT="MyBot/1.0 (+https://mysite.com/bot)"
   ```

5. **Monitor resource usage:**
   - Track memory consumption
   - Monitor network bandwidth
   - Set alerts on error rates

## API Reference

### POST /api/v1/spider/crawl

Dedicated Spider endpoint for deep crawling.

**Request:**
```json
{
  "seed_urls": ["https://example.com"],
  "max_depth": 3,
  "max_pages": 100,
  "strategy": "breadth_first",
  "timeout_seconds": 30,
  "delay_ms": 500,
  "concurrency": 4,
  "respect_robots": true,
  "follow_redirects": true
}
```

**Response:** See "Response Format" section above.

### POST /api/v1/crawl

Regular crawl endpoint with Spider support.

**Request:**
```json
{
  "urls": ["https://example.com"],
  "options": {
    "use_spider": true,
    "concurrency": 4
  }
}
```

### POST /api/v1/spider/status

Get Spider status and metrics.

**Request:**
```json
{
  "include_metrics": true
}
```

### POST /api/v1/spider/control

Control Spider operations.

**Request:**
```json
{
  "action": "stop"  // "stop" or "reset"
}
```

## Environment Variable Reference

Complete list of Spider environment variables:

```bash
# Core Settings
SPIDER_ENABLE=true                          # Enable Spider engine
SPIDER_BASE_URL=https://example.com         # Base URL for crawling

# Crawl Limits
SPIDER_MAX_DEPTH=3                          # Maximum crawl depth
SPIDER_MAX_PAGES=100                        # Maximum pages to crawl
SPIDER_CONCURRENCY=4                        # Concurrent requests

# Performance
SPIDER_TIMEOUT_SECONDS=30                   # Request timeout
SPIDER_DELAY_MS=500                         # Delay between requests

# Behavior
SPIDER_RESPECT_ROBOTS=true                  # Respect robots.txt
SPIDER_USER_AGENT=RipTide Spider/1.0        # User agent string

# Dependencies
REDIS_URL=redis://localhost:6379/0          # Redis connection
```

## Support

For issues or questions:
1. Check logs for error messages
2. Verify configuration with `spider/status` endpoint
3. Review this guide's troubleshooting section
4. Open an issue on GitHub with:
   - Configuration (sanitized)
   - Log output
   - Expected vs actual behavior

## Related Documentation

- [Spider Architecture](/workspaces/eventmesh/crates/riptide-core/src/spider/architecture.md)
- [API Documentation](/workspaces/eventmesh/docs/api/README.md)
- [Spider Engine README](/workspaces/eventmesh/crates/riptide-core/src/spider/README.md)
