# Spider Engine Configuration Guide

## Problem

The Spider engine is disabled by default in the API server, causing all crawl commands with `use_spider` flag to fail with:

```
Spider engine is not enabled. Set SPIDER_ENABLE=true to enable spider crawling.
```

## Root Cause

The Spider engine initialization in `/workspaces/eventmesh/crates/riptide-api/src/state.rs` checks for `SPIDER_ENABLE` environment variable which defaults to `false`.

**Configuration Check Location:**
- **File**: `crates/riptide-api/src/state.rs`
- **Function**: `AppConfig::init_spider_config()` (lines 303-376)
- **Default**: `SPIDER_ENABLE=false` (line 308)

## Solution

### Option 1: Enable via Environment Variable (Recommended)

Set the `SPIDER_ENABLE` environment variable to `true` before starting the API server:

```bash
export SPIDER_ENABLE=true
export SPIDER_BASE_URL=https://example.com  # Optional: Set your base URL

# Start the API server
cargo run --bin riptide-api
```

### Option 2: Enable via .env File

Create or update `.env` file in the project root:

```env
# Spider Engine Configuration
SPIDER_ENABLE=true
SPIDER_BASE_URL=https://example.com
SPIDER_USER_AGENT=RipTide Spider/1.0
SPIDER_TIMEOUT_SECONDS=30
SPIDER_DELAY_MS=500
SPIDER_CONCURRENCY=4
SPIDER_MAX_DEPTH=10
SPIDER_MAX_PAGES=1000
SPIDER_RESPECT_ROBOTS=true
```

### Option 3: Enable via Docker/Compose

If using Docker Compose, add to your service environment:

```yaml
services:
  riptide-api:
    environment:
      - SPIDER_ENABLE=true
      - SPIDER_BASE_URL=https://example.com
```

## Configuration Options

### Required Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `SPIDER_ENABLE` | `false` | **Must be set to `true`** to enable Spider engine |
| `SPIDER_BASE_URL` | `https://example.com` | Base URL for crawling (used as default) |

### Optional Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `SPIDER_USER_AGENT` | `RipTide Spider/1.0` | User agent string for requests |
| `SPIDER_TIMEOUT_SECONDS` | `30` | Request timeout in seconds |
| `SPIDER_DELAY_MS` | `500` | Delay between requests (milliseconds) |
| `SPIDER_CONCURRENCY` | `4` | Maximum concurrent requests |
| `SPIDER_MAX_DEPTH` | `10` | Maximum crawl depth |
| `SPIDER_MAX_PAGES` | `1000` | Maximum pages to crawl |
| `SPIDER_RESPECT_ROBOTS` | `true` | Respect robots.txt rules |

## Verification

After enabling Spider, verify it's working:

### 1. Check Health Endpoint

```bash
curl http://localhost:8080/healthz
```

Look for `spider: "healthy"` in the response.

### 2. Test Spider Crawl

```bash
curl -X POST http://localhost:8080/spider/crawl \
  -H "Content-Type: application/json" \
  -d '{
    "seed_urls": ["https://example.com"],
    "max_depth": 3,
    "max_pages": 100
  }'
```

### 3. Test via Regular Crawl with Spider Flag

```bash
curl -X POST http://localhost:8080/crawl \
  -H "Content-Type: application/json" \
  -d '{
    "urls": ["https://example.com"],
    "options": {
      "use_spider": true
    }
  }'
```

## Code References

### Spider Initialization Code

**File**: `crates/riptide-api/src/state.rs` (lines 567-601)

```rust
// Initialize Spider if enabled
let spider = if let Some(ref spider_config) = config.spider_config {
    tracing::info!("Initializing Spider engine for deep crawling");

    let spider_config = spider_config.clone();
    match Spider::new(spider_config).await {
        Ok(spider_engine) => {
            // Spider initialized successfully
            Some(Arc::new(spider_with_integrations))
        }
        Err(e) => {
            tracing::error!("Failed to initialize Spider engine: {}", e);
            None
        }
    }
} else {
    tracing::debug!("Spider engine disabled");
    None
};
```

### Spider Config Check

**File**: `crates/riptide-api/src/state.rs` (lines 303-376)

```rust
fn init_spider_config() -> Option<SpiderConfig> {
    // Check if spider is enabled
    let spider_enabled = std::env::var("SPIDER_ENABLE")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);

    if !spider_enabled {
        return None;  // ← This is why it's disabled
    }

    // Initialize spider configuration...
}
```

## Architecture Notes

### Spider Integration Flow

1. **API Handler** (`crates/riptide-api/src/handlers/crawl.rs`)
   - Checks if `use_spider` option is set
   - Routes to `handle_spider_crawl()` if enabled

2. **Spider Check** (`crates/riptide-api/src/handlers/spider.rs`)
   - Validates that `state.spider.is_some()`
   - Returns error if Spider is `None`

3. **State Initialization** (`crates/riptide-api/src/state.rs`)
   - Reads `SPIDER_ENABLE` from environment
   - Creates `SpiderConfig` if enabled
   - Initializes `Spider` engine with integrations

### Dependencies

Spider engine requires:
- ✅ **FetchEngine** - HTTP client with rate limiting
- ✅ **MemoryManager** - WASM memory management
- ✅ **Redis** - For session/cache storage (already configured)

All dependencies are automatically wired when Spider is initialized.

## Troubleshooting

### Issue: "Spider engine is not enabled"

**Solution**: Set `SPIDER_ENABLE=true` environment variable

### Issue: Spider initializes but crawl fails

**Check**:
1. Redis is running and accessible
2. `REDIS_URL` environment variable is correct
3. Check logs for Spider initialization errors
4. Verify network connectivity to target URLs

### Issue: Invalid SPIDER_BASE_URL

**Error**:
```
Invalid SPIDER_BASE_URL 'xyz': relative URL without a base
```

**Solution**: Ensure `SPIDER_BASE_URL` is a valid absolute URL:
```bash
export SPIDER_BASE_URL=https://example.com
```

## Performance Considerations

### Memory Usage

Spider uses:
- **Frontier queue** - Stores URLs to crawl
- **URL deduplication** - Bloom filter + exact tracking
- **Session state** - For authenticated crawling

Estimated memory: ~100-500MB depending on crawl scale

### Resource Limits

Configure based on your system:

```bash
# High-performance setup
export SPIDER_CONCURRENCY=16
export SPIDER_MAX_PAGES=10000

# Conservative setup (low memory)
export SPIDER_CONCURRENCY=2
export SPIDER_MAX_PAGES=1000
```

## Next Steps

After enabling Spider:

1. ✅ Set `SPIDER_ENABLE=true`
2. ✅ Configure optional settings
3. ✅ Restart API server
4. ✅ Test with `/spider/crawl` endpoint
5. ✅ Monitor performance and adjust settings

## Summary

| What | Where | Action |
|------|-------|--------|
| **Enable Spider** | Environment | `SPIDER_ENABLE=true` |
| **Configure Base URL** | Environment | `SPIDER_BASE_URL=https://...` |
| **Verify** | API | `curl http://localhost:8080/healthz` |
| **Test** | API | Use `/spider/crawl` endpoint |
| **Restart** | Server | Restart required after config changes |
