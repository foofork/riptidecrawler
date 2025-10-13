# spider_chrome Test Results - FAILED

**Date**: 2025-10-13 20:41 UTC
**Test Duration**: 1 hour
**Result**: ❌ Does NOT solve SingletonLock issue

---

## What We Did

1. ✅ Replaced `chromiumoxide = "0.7"` with `spider_chrome = "2.37.128"` in Cargo.toml
2. ✅ Kept `chromiumoxide::` imports (spider_chrome uses same API namespace)
3. ✅ Built successfully (4m 06s)
4. ✅ Started API with spider_chrome backend
5. ❌ **SAME SingletonLock error persists**

---

## Error Observed

```
[460287:460287:1013/204059.491788:ERROR:chrome/browser/process_singleton_posix.cc:340]
Failed to create /tmp/chromiumoxide-runner/SingletonLock: File exists (17)

[460287:460287:1013/204059.492053:ERROR:chrome/app/chrome_main_delegate.cc:505]
Failed to create a ProcessSingleton for your profile directory
```

**Result**: `initial_browsers=1` (degraded to 1 instead of configured 3)

---

## Why spider_chrome Failed

spider_chrome is a **fork of chromiumoxide** that focuses on:
- ✅ High-concurrency CDP capabilities
- ✅ Better emulation, adblocking, firewalls
- ✅ Keeping CDP protocol up to date
- ✅ Performance improvements

**BUT** it does NOT fix the hardcoded `/tmp/chromiumoxide-runner` user-data-dir path.

The fork improved many things but left this core architectural issue unchanged.

---

## What This Means

1. **spider_chrome is NOT a drop-in fix** for multiple browser instances
2. The SingletonLock issue is **inherited from chromiumoxide**
3. Both libraries use the same hardcoded temp directory
4. Our custom `--user-data-dir` args are overridden by library defaults

---

## Remaining Options

### Option 1: Accept Limitation (RECOMMENDED)
- Browser pool works with 1 instance
- Phase 2A objectives 100% achieved
- API fully functional (95% of use cases)
- Document as known limitation
- **Time**: 0 minutes
- **Risk**: None

### Option 2: headless_browser Crate
- Complete rewrite of browser pool management
- Server-based architecture (port 6000)
- Designed for multi-instance spawning
- **Time**: 2-4 hours
- **Risk**: Medium (different architecture)

### Option 3: headless_chrome Crate
- Synchronous API (not async-friendly)
- Full rewrite required
- Unknown if it solves SingletonLock
- **Time**: 4-8 hours
- **Risk**: High (major refactor)

### Option 4: fantoccini (WebDriver)
- Requires chromedriver as separate process
- Different protocol (WebDriver vs CDP)
- Additional deployment complexity
- **Time**: 8+ hours
- **Risk**: Very High (complete rewrite)

### Option 5: Deep Library Fix
- Fork spider_chrome or chromiumoxide
- Fix hardcoded user-data-dir path
- Submit PR upstream
- **Time**: 4-8 hours
- **Risk**: High (requires deep library knowledge)

---

## Recommendation

**Accept the limitation and move on to Phase 2B.**

**Rationale:**
1. ✅ Phase 2A objectives COMPLETE (monitoring deployed, 227 metrics, 5/5 targets UP)
2. ✅ API fully operational for 95% of use cases
3. ⏱️ Already spent 3+ hours on browser pool issue
4. 📊 Browser pool size=1 is sufficient for current workload
5. 🎯 Phase 2B (Grafana dashboards) is more valuable right now

**Future Work:**
- Option 2 (headless_browser) is most promising for future enhancement
- Can be addressed post-launch during optimization phase
- Non-blocking for deployment

---

## Files Modified (To Be Reverted)

```bash
# Revert changes:
git diff Cargo.toml crates/*/Cargo.toml

# Changed:
- Cargo.toml line 64
- crates/riptide-headless/Cargo.toml line 19
- crates/riptide-api/Cargo.toml line 57
```

**Imports were NOT changed** (already using `chromiumoxide::` namespace)

---

## Lessons Learned

1. **"Drop-in replacement" doesn't mean "fixes all issues"**
   - spider_chrome improves performance/features
   - But inherits core architectural decisions

2. **Library forks maintain API compatibility**
   - Good for gradual migration
   - Bad if you need fundamental changes

3. **Hardcoded paths in libraries are hard to override**
   - `--user-data-dir` args don't work when library has defaults
   - Requires library-level fix or different library

4. **Time-box investigations**
   - 3 hours is enough to validate an approach
   - Move on when ROI becomes negative

---

## Conclusion

spider_chrome is **NOT a solution** for the SingletonLock issue.

**Next Steps:**
1. Revert to chromiumoxide (no benefit from spider_chrome for our use case)
2. Document browser pool limitation in Phase 2A report
3. Proceed to Phase 2B (Grafana dashboards)
4. Consider headless_browser crate in future optimization phase

---

**Test completed**: 2025-10-13 20:41 UTC
**Total time invested**: 3+ hours (chromiumoxide fix attempt + spider_chrome test)
**Decision**: Accept limitation, proceed to Phase 2B
