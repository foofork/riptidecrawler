# .env Variable Analysis - System vs Request Parameters

**Date**: 2025-10-28
**Issue**: Configuration parameter conflicts between .env and API requests

---

## Problem Statement

You're absolutely correct! The `.env.example` file contains **request-level parameters** that should come from API requests, not system-level environment variables. This creates confusion and potential conflicts.

### Example Conflict:

```bash
# In .env.example (line 526-538):
SPIDER_MAX_DEPTH=3
SPIDER_MAX_PAGES=100
SPIDER_CONCURRENCY=4
SPIDER_TIMEOUT_SECONDS=30
SPIDER_DELAY_MS=500

# User's API request:
POST /api/crawl
{
  "urls": ["https://example.com"],
  "use_spider": true,
  "max_depth": 10,      # ← Does this override SPIDER_MAX_DEPTH=3?
  "max_pages": 500,     # ← Which takes precedence?
  "concurrency": 8      # ← Confusing!
}
```

**Question**: Which value wins? The .env default or the API request parameter?

---

## Solution: Separate System-Level from Request-Level

### ✅ **System-Level Variables** (Keep in .env)
These configure the **system/infrastructure**, not individual requests:

```bash
# Core System Configuration
SPIDER_ENABLE=true              # ✅ System: Enable/disable spider functionality
REDIS_URL=redis://redis:6379/0  # ✅ System: Infrastructure connection
HEADLESS_URL=http://...         # ✅ System: Service URL
RUST_LOG=info                   # ✅ System: Logging level
WORKER_POOL_SIZE=4              # ✅ System: Resource allocation

# Optional System Defaults (with API override capability)
SPIDER_BASE_URL=https://example.com  # ✅ System: Default base URL if not in request
```

### ❌ **Request-Level Variables** (Remove from .env)
These configure **individual requests** and should come from API parameters:

```bash
# REMOVE these from .env.example (or comment with warning):
SPIDER_MAX_DEPTH=3              # ❌ Request-level: per-crawl setting
SPIDER_MAX_PAGES=100            # ❌ Request-level: per-crawl limit
SPIDER_CONCURRENCY=4            # ❌ Request-level: per-crawl concurrency
SPIDER_TIMEOUT_SECONDS=30       # ❌ Request-level: per-request timeout
SPIDER_DELAY_MS=500             # ❌ Request-level: per-crawl delay
SPIDER_RESPECT_ROBOTS=true      # ❌ Request-level: per-crawl behavior
```

---

## Recommended Precedence Rules

When both .env and API request provide values:

1. **API Request Parameter** (highest priority)
2. **System Default from Code** (fallback if API param not provided)
3. **.env Variable** (lowest priority, only for system-level config)

### Example:
```rust
// In code:
let max_depth = api_request.max_depth  // Check API request first
    .or(config.spider_config.max_depth)  // Then code defaults
    .unwrap_or(10);  // Finally hardcoded fallback

// DON'T do:
let max_depth = env::var("SPIDER_MAX_DEPTH")  // ❌ .env shouldn't override API request
```

---

## Variables to Review and Fix

### Spider Configuration (lines 515-544):
- [ ] ✅ Keep: `SPIDER_ENABLE=true` (system-level)
- [ ] ❌ Remove: `SPIDER_MAX_DEPTH=3` (request-level)
- [ ] ❌ Remove: `SPIDER_MAX_PAGES=100` (request-level)
- [ ] ❌ Remove: `SPIDER_CONCURRENCY=4` (request-level)
- [ ] ❌ Remove: `SPIDER_TIMEOUT_SECONDS=30` (request-level)
- [ ] ❌ Remove: `SPIDER_DELAY_MS=500` (request-level)
- [ ] ❌ Remove: `SPIDER_RESPECT_ROBOTS=true` (request-level)
- [ ] ⚠️ Review: `SPIDER_BASE_URL=https://example.com` (depends on use case)

### Pipeline Configuration (lines 547-564):
- [ ] ✅ Keep: `ENHANCED_PIPELINE_ENABLE=true` (system-level)
- [ ] ✅ Keep: `ENHANCED_PIPELINE_METRICS=true` (system-level)
- [ ] ❌ Remove: `ENHANCED_PIPELINE_FETCH_TIMEOUT=10` (request-level)
- [ ] ❌ Remove: `ENHANCED_PIPELINE_RENDER_TIMEOUT=3` (request-level)
- [ ] ❌ Remove: `ENHANCED_PIPELINE_WASM_TIMEOUT=5` (request-level)

### Resource Configuration (lines 234-256):
- [ ] ⚠️ Review: `RIPTIDE_MAX_CONCURRENT_RENDERS=10` (system-level resource limit)
- [ ] ⚠️ Review: `RIPTIDE_GLOBAL_TIMEOUT_SECS=30` (system-level cap, but requests should be able to set lower)

---

## Recommended .env.example Structure

```bash
# ============================================================================
# Spider/Crawler Configuration
# ============================================================================

# SYSTEM-LEVEL: Enable spider/crawler functionality
SPIDER_ENABLE=true

# SYSTEM-LEVEL: Default base URL (can be overridden per request)
# SPIDER_BASE_URL=https://example.com

# ============================================================================
# ⚠️ REQUEST-LEVEL PARAMETERS (Set via API requests, not .env)
# ============================================================================
# The following are per-request parameters and should be set via API calls:
#
# - max_depth: Maximum crawl depth (default: 10)
# - max_pages: Maximum pages to crawl (default: 1000)
# - concurrency: Concurrent requests (default: 4)
# - timeout: Request timeout in seconds (default: 30)
# - delay_ms: Delay between requests (default: 500)
# - respect_robots: Respect robots.txt (default: true)
#
# Example API request:
# POST /api/crawl
# {
#   "urls": ["https://example.com"],
#   "use_spider": true,
#   "max_depth": 5,
#   "max_pages": 100,
#   "concurrency": 2
# }
#
# DO NOT set these in .env as they should vary per request:
# SPIDER_MAX_DEPTH=3              # ❌ Don't set - use API parameter
# SPIDER_MAX_PAGES=100            # ❌ Don't set - use API parameter
# SPIDER_CONCURRENCY=4            # ❌ Don't set - use API parameter
# ============================================================================
```

---

## Impact of Not Fixing

If we leave request-level params in .env:

1. **User Confusion**: "Why doesn't my API request's `max_depth=10` work?"
2. **Debugging Nightmare**: Hard to tell if issue is from .env or API request
3. **Inflexibility**: Every request uses same depth/pages/concurrency
4. **Documentation Burden**: Must document precedence rules everywhere
5. **Production Issues**: Different behavior between environments

---

## Action Items

1. [ ] Remove request-level params from `.env.example`
2. [ ] Add clear comment section explaining the difference
3. [ ] Update API documentation to show parameter sources
4. [ ] Add validation to reject .env vars that shouldn't be set
5. [ ] Test that API request params properly override system defaults
6. [ ] Update deployment guides

---

## Testing Checklist

After fixing .env.example:

```bash
# Test 1: API request params work without .env
unset SPIDER_MAX_DEPTH
curl -X POST http://localhost:8080/api/crawl \
  -d '{"urls": ["https://example.com"], "max_depth": 5}'
# Expected: Uses max_depth=5 from request

# Test 2: System config from .env works
export SPIDER_ENABLE=true
docker-compose up -d
curl http://localhost:8080/healthz | jq .dependencies.spider_engine
# Expected: Shows spider_engine as healthy

# Test 3: Different requests use different params
curl -X POST ... -d '{"max_depth": 5}' &
curl -X POST ... -d '{"max_depth": 10}' &
# Expected: First uses 5, second uses 10 (not both using .env default)
```

---

**Conclusion**: The .env.example should only contain system-level configuration. Request-level parameters should come exclusively from API requests. This provides maximum flexibility and eliminates confusion.
