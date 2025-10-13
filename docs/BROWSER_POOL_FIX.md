# Browser Pool SingletonLock Fix

**Status**: 🔧 Code ready, pending long build
**Priority**: P2 (Enhancement, non-blocking)
**Est. Time**: 10-15 minutes once build completes

---

## Problem

Chrome's SingletonLock prevents multiple browser instances from sharing the same profile directory:

```
ERROR:chrome/browser/process_singleton_posix.cc:340]
Failed to create /tmp/chromiumoxide-runner/SingletonLock: File exists
```

**Current Behavior**:
- First browser launches successfully  ✅
- Subsequent browsers fail with SingletonLock error ❌
- Pool degraded to size=1 instead of configured size=3

**Impact**:
- API reports status="degraded"
- Headless browser pool limited to 1 instance
- No blocking issues for static extraction

---

## Solution Implemented

### Files Modified

1. **`/workspaces/eventmesh/crates/riptide-headless/src/pool.rs`** (lines 92-122)
   ```rust
   pub async fn new(base_config: BrowserConfig) -> Result<Self> {
       let id = Uuid::new_v4().to_string();

       // CRITICAL FIX: Unique profile directory per browser
       let user_data_dir = format!("/tmp/chromiumoxide-runner-{}", id);

       let mut builder = BrowserConfig::builder();
       builder = builder.arg(format!("--user-data-dir={}", user_data_dir));
       // ... copy core settings ...
   }
   ```

2. **`/workspaces/eventmesh/crates/riptide-headless/src/launcher.rs`** (line 225)
   - Added comment documenting that unique profiles are handled in pool.rs

### How It Works

**Before**: All browsers shared `/tmp/chromiumoxide-runner/`
**After**: Each browser gets `/tmp/chromiumoxide-runner-{uuid}`

Example:
```
Browser 1: /tmp/chromiumoxide-runner-a1b2c3d4-...
Browser 2: /tmp/chromiumoxide-runner-e5f6g7h8-...
Browser 3: /tmp/chromiumoxide-runner-i9j0k1l2-...
```

---

## Testing Plan

Once build completes:

1. **Kill existing API**:
   ```bash
   pkill -f "riptide-api"
   ```

2. **Start new binary**:
   ```bash
   cargo run --release --bin riptide-api > /tmp/riptide-browser-test.log 2>&1 &
   sleep 15
   ```

3. **Verify pool initialization**:
   ```bash
   tail -50 /tmp/riptide-browser-test.log | grep -E "(Browser pool|initial_browsers)"
   # Expected: "Browser pool initialized successfully initial_browsers=3"
   ```

4. **Check health status**:
   ```bash
   curl -s http://localhost:8080/healthz | jq '.status, .dependencies.headless_service'
   # Expected: "healthy" (not "degraded")
   # Expected: headless_service: {"status": "healthy"}
   ```

5. **Monitor for SingletonLock errors**:
   ```bash
   tail -f /tmp/riptide-browser-test.log | grep -i singleton
   # Expected: No errors
   ```

---

## Why Phase 2A is Complete Without This

### ✅ Phase 2A Success Criteria Met

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Prometheus deployed | ✅ | 5/5 targets UP |
| Metrics exposed | ✅ | 227 metrics, 88 series |
| Grafana accessible | ✅ | http://localhost:3000 |
| AlertManager running | ✅ | http://localhost:9093 |
| API operational | ✅ | Health: degraded but functional |
| Week 1 tests passing | ✅ | 6/6 golden tests |

### 🎯 Browser Pool is Nice-to-Have

**The API works fine without it**:
- Static HTML extraction: ✅ Full functionality
- Redis caching: ✅ Working
- WASM extractor: ✅ Loaded
- HTTP client: ✅ Operational
- Gate analysis: ✅ Working
- Metrics collection: ✅ All 227 metrics

**Only limitation**:
- Dynamic JavaScript-heavy sites may need headless browser
- With pool size=1, throughput is limited for those specific cases
- Most web scraping works fine with static extraction

---

## When to Apply This Fix

### Option 1: Complete Now (if time permits)
- Wait for cargo build to finish (~5-10 more minutes)
- Test and verify
- Update Phase 2A report with "fully healthy" status

### Option 2: Defer to Phase 2B
- Phase 2A is about monitoring infrastructure ✅
- Browser pool optimization is a performance enhancement
- Can be done alongside Grafana dashboard work
- Non-blocking for deployment

### Option 3: Post-Launch
- System is production-ready as-is
- Enhancement can be applied during normal maintenance
- Zero customer impact

---

## Rollback Plan

If issues occur after applying fix:

```bash
# 1. Revert the changes
git checkout HEAD -- crates/riptide-headless/src/pool.rs
git checkout HEAD -- crates/riptide-headless/src/launcher.rs

# 2. Rebuild
cargo build --release --bin riptide-api

# 3. Restart API
pkill -f "riptide-api"
cargo run --release --bin riptide-api > /tmp/riptide-api.log 2>&1 &
```

---

## Recommendation

**For Phase 2A completion**: ✅ Mark as COMPLETE with browser pool documented as known limitation

**Rationale**:
1. All monitoring objectives achieved
2. API is functional and stable
3. Browser pool is enhancement, not blocker
4. Can be addressed in Phase 2B or post-launch

**Next Steps**:
- Document current state in final report
- Proceed to Phase 2B (Grafana dashboards)
- Circle back to browser pool during dashboard testing

---

**Created**: 2025-10-13 19:40 UTC
**Author**: Hive Mind Collective
**Status**: Ready for deployment after cargo build completes
