# Chrome Crashpad Handler Solution - RESOLVED ✅

**Date**: 2025-10-28
**Issue**: `chrome_crashpad_handler: --database is required`
**Status**: ✅ **RESOLVED**

---

## Problem Summary

### The Error
```
chrome_crashpad_handler: --database is required
Try 'chrome_crashpad_handler --help' for more information.
[pid:pid:timestamp:ERROR:third_party/crashpad/crashpad/util/linux/socket.cc:120]
  recvmsg: Connection reset by peer (104)
```

### Impact
- Chrome browser pool could not initialize (stayed at 0 instances)
- Continuous crash reporter errors every 15 seconds
- Browser-based extraction unavailable
- Only WASM extraction worked

### When It Started
- Chrome/Chromium version 128.0.6613.113 (September 2024)
- Known issue affecting Docker containers with Chrome 128+

---

## Root Cause

### What We Tried (That Didn't Work)
Added 14+ Chrome launch flags in `launcher/mod.rs:383-398`:
```rust
.arg("--disable-crash-reporter")
.arg("--crash-dumps-dir=/tmp")
.arg("--disable-breakpad")
.arg("--no-sandbox")
.arg("--disable-dev-shm-usage")
.arg("--disable-gpu")
// + 8 more flags
```

**Result**: ❌ Still failed - flags were not respected

### The Real Problem
1. **Chrome 128+ changed crashpad behavior**: Requires writable database location
2. **Flags don't propagate**: `--disable-crash-reporter` doesn't affect crashpad_handler subprocess
3. **Missing environment variables**: Chrome needs `XDG_CONFIG_HOME` and `XDG_CACHE_HOME` to be writable

---

## The Solution ✅

### Add Environment Variables
**File**: `infra/docker/Dockerfile.headless:165-172`

```dockerfile
ENV RUST_LOG=info \
    CHROME_BIN=/usr/bin/chromium \
    CHROME_PATH=/usr/bin/chromium \
    DISPLAY=:99 \
    MALLOC_ARENA_MAX=2 \
    CHROME_DEVEL_SANDBOX=/usr/lib/chromium/chrome-sandbox \
    XDG_CONFIG_HOME=/tmp/.chromium \
    XDG_CACHE_HOME=/tmp/.chromium
```

**Key Change**: Added `XDG_CONFIG_HOME` and `XDG_CACHE_HOME`

### Why This Works
- Chrome crashpad handler checks these environment variables for config/cache directories
- `/tmp/.chromium` is writable by the `riptide` user
- Chrome can now create its crashpad database without errors
- Flags alone are insufficient - environment variables are required

---

## Test Results

### Before Fix
```
Browser pool: 0 instances
Crash errors: Continuous (every 15 seconds)
Browser-based extraction: ❌ Not available
Status: Degraded
```

### After Fix
```
Browser pool: 5 instances initialized ✅
Crash errors: 0 ✅
Browser-based extraction: ✅ Available
Status: Healthy
```

### Logs After Fix
```json
{
  "timestamp":"2025-10-28T09:21:26.819464Z",
  "level":"INFO",
  "message":"Browser pool initialized successfully",
  "initial_browsers":5
}
{
  "timestamp":"2025-10-28T09:21:26.819809Z",
  "level":"INFO",
  "message":"Headless launcher initialized successfully"
}
```

**No crash errors!** ✅

---

## Implementation Details

### Docker Configuration
**Location**: `infra/docker/Dockerfile.headless`

**Environment Variables Added**:
- `XDG_CONFIG_HOME=/tmp/.chromium` - Chrome configuration directory
- `XDG_CACHE_HOME=/tmp/.chromium` - Chrome cache directory

**Why /tmp/.chromium**:
- `/tmp` is writable by non-root users
- Doesn't require additional volume mounts
- Automatically cleaned on container restart
- Meets Chrome's crashpad database requirements

### Directory Structure
```
/tmp/.chromium/
├── (Chrome configuration files)
└── (Crashpad database files)
```

Created automatically by Chrome when container starts.

---

## Related Issues

### GitHub Issues (This is a widespread problem)
- [chrome-php/chrome#649](https://github.com/chrome-php/chrome/issues/649) - `--database` flag required in Chrome 128
- [chrome-php/chrome#661](https://github.com/chrome-php/chrome/issues/661) - Crashpad handler issue
- [hardkoded/puppeteer-sharp#2633](https://github.com/hardkoded/puppeteer-sharp/issues/2633) - Docker-specific issue
- [puppeteer/puppeteer#11023](https://github.com/puppeteer/puppeteer/issues/11023) - Read-only filesystem issue
- [microsoft/playwright#34031](https://github.com/microsoft/playwright/issues/34031) - Playwright affected
- [NixOS/nixpkgs#132702](https://github.com/NixOS/nixpkgs/issues/132702) - XDG_CONFIG_HOME requirement

### Chrome Version Affected
- Chrome/Chromium 128.0.6613.113+
- Released: September 2024
- Affects all Linux distributions (Debian, Ubuntu, etc.)

---

## For External Users

### If You're Using RipTide

**Good News**: This is already fixed in the Docker images!

**What Was Changed**:
1. Added `XDG_CONFIG_HOME` and `XDG_CACHE_HOME` to headless Dockerfile
2. Chrome browser pool now initializes with 5 browsers
3. No configuration needed on your end

### If You're Building Custom Images

**Required Changes**:
```dockerfile
# Add to your Dockerfile:
ENV XDG_CONFIG_HOME=/tmp/.chromium \
    XDG_CACHE_HOME=/tmp/.chromium
```

**Or in docker-compose.yml**:
```yaml
services:
  headless:
    environment:
      - XDG_CONFIG_HOME=/tmp/.chromium
      - XDG_CACHE_HOME=/tmp/.chromium
```

---

## Alternative Solutions (Not Recommended)

### 1. Provide Database Path
```dockerfile
ENV CHROME_FLAGS="--crash-dumps-dir=/tmp/crashes"
```
**Issue**: Still requires writable directory, doesn't fully disable crashpad

### 2. Mount Volume
```yaml
volumes:
  - ./chrome-data:/tmp/.chromium
```
**Issue**: Requires host filesystem, less portable

### 3. Use Older Chrome
```dockerfile
RUN apt-get install chromium=127.*
```
**Issue**: Missing security updates, not recommended

---

## Performance Impact

### Before Fix (WASM Only)
- Memory: ~440MB (Mode 2 standalone)
- Extraction: WASM only
- SPA Support: ❌ No

### After Fix (Full Chrome Support)
- Memory: ~1,208MB (Mode 1 microservices)
- Extraction: WASM + Chrome browser
- SPA Support: ✅ Yes

**Trade-off**: ~768MB more memory for Chrome browser capability

---

## Deployment Modes Updated

### Mode 1: Microservices (Now Fully Working ✅)
```
docker-compose -f docker-compose.production.yml up -d
```
**Features**:
- API + Headless service (with working Chrome!)
- Browser pool: 5 instances
- SPA page support
- JavaScript execution
- Memory: ~1.2GB

**Use When**: Need JavaScript rendering for SPA pages

### Mode 2: Standalone (Still Recommended for WASM)
```
docker-compose -f docker-compose.standalone.yml up -d
```
**Features**:
- API only (no Chrome)
- WASM extraction
- Lower memory (~440MB)
- Faster startup

**Use When**: Static content extraction sufficient

---

## Testing Checklist

### Verify Fix Is Working
```bash
# 1. Check browser pool initialized
docker-compose -f docker-compose.production.yml logs riptide-headless | grep "Browser pool initialized"

# Expected: "Browser pool initialized successfully","initial_browsers":5

# 2. Check for crash errors
docker-compose -f docker-compose.production.yml logs riptide-headless | grep crash | wc -l

# Expected: 0

# 3. Test extraction
curl -X POST http://localhost:8080/api/v1/extract \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com", "extraction_type": "Full"}'

# Expected: Success with extraction data
```

---

## Key Takeaways

1. **Chrome flags alone are insufficient** for Chrome 128+
2. **Environment variables are required**: `XDG_CONFIG_HOME` and `XDG_CACHE_HOME`
3. **This is a Chrome 128+ requirement**, not a RipTide bug
4. **Solution is simple**: Add 2 environment variables to Dockerfile
5. **Works across all platforms**: Linux, Docker, Kubernetes, etc.

---

## Related Documentation
- [Mode 1 Microservices Test](./MODE_1_MICROSERVICES_TEST.md) - Updated with fix
- [Mode 2 Standalone Test](./MODE_2_STANDALONE_TEST.md) - WASM-only alternative
- [Deployment Comparison](./DEPLOYMENT_MODE_COMPARISON.md) - Mode comparison
- [Comprehensive Status Report](./COMPREHENSIVE_STATUS_REPORT.md) - Full project status

---

## Credits

**Solution Found**: Web search of GitHub issues and community reports
**Affected Projects**: Puppeteer, Playwright, chrome-php, PuppeteerSharp, and many others
**Root Cause**: Chrome 128+ crashpad behavior change (September 2024)

---

**Status**: ✅ RESOLVED
**Browser Pool**: ✅ 5 instances initialized
**SPA Support**: ✅ Available
**Recommended**: Mode 1 (Microservices) now fully functional for JavaScript-heavy pages
