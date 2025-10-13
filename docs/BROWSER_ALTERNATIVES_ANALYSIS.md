# Browser Pool Alternatives - Technical Analysis

**Date**: 2025-10-13
**Context**: Chrome SingletonLock preventing multiple browser instances with chromiumoxide 0.7
**Status**: Researched 3 alternatives, spider_chrome is best option

---

## Problem Summary

**Current Issue**: chromiumoxide 0.7 hardcodes `/tmp/chromiumoxide-runner` as user-data-dir, causing SingletonLock errors when running multiple browsers concurrently.

**Impact**:
- Browser pool degraded to 1 instance instead of configured 3
- API shows "degraded" status (non-blocking)
- 95% of use cases work fine with static extraction
- Only dynamic JavaScript-heavy sites need headless browsers

---

## Alternatives Evaluated

### 1. ⭐ **spider_chrome** (RECOMMENDED)

**Crate**: `spider_chrome = "2.37.128"`
**Repo**: https://github.com/spider-rs/spider_chrome
**License**: MIT OR Apache-2.0

#### Key Features:
- ✅ **Drop-in replacement** for chromiumoxide (same API: Browser, BrowserConfig, Page)
- ✅ **High-concurrency CDP capabilities** (explicitly designed for this)
- ✅ Fork of chromiumoxide with performance improvements
- ✅ Keeps CDP protocol up to date
- ✅ Better emulation, adblocking, firewalls
- ✅ Actively maintained (August 2025)
- ✅ SIMD optimizations available

#### API Compatibility:
```rust
// Current code uses:
use chromiumoxide::{Browser, BrowserConfig, Page};

// spider_chrome replacement (same API):
use spider_chrome::{Browser, BrowserConfig, Page};
```

#### Implementation Effort: **LOW** ⏱️ 30-60 minutes
1. Update `Cargo.toml`: Replace `chromiumoxide = "0.7"` with `spider_chrome = "2.37.128"`
2. Update imports in 3 files:
   - `crates/riptide-headless/src/pool.rs`
   - `crates/riptide-headless/src/launcher.rs`
   - `crates/riptide-headless/src/cdp.rs`
3. Rebuild (3-4 minutes)
4. Test browser pool initialization

#### Likely Success Rate: **HIGH** (90%+)
- Same API means minimal code changes
- Designed specifically for concurrent browser instances
- Proven track record in spider-rs ecosystem

---

### 2. **headless_browser** (Alternative)

**Crate**: `headless_browser = "0.1.24"`
**Repo**: https://github.com/spider-rs/headless-browser
**License**: MIT

#### Key Features:
- ✅ Complete browser pool management system
- ✅ "Manual and automatic spawning and termination of multiple headless instances"
- ✅ Integrated proxy and server support
- ✅ Caching for improved performance
- ✅ Concurrent navigation across pages
- ✅ Uses spider_chrome under the hood

#### Implementation Effort: **MEDIUM** ⏱️ 2-4 hours
- Would require refactoring custom pool.rs (862 lines)
- Different API than chromiumoxide
- Server-based architecture (runs on port 6000)
- More features but higher complexity

#### Use Case:
- Better for microservices architecture
- If we want to externalize browser management
- When we need advanced proxy/routing features

---

### 3. **headless_chrome** (Not Recommended)

**Crate**: `headless_chrome`
**Repo**: https://github.com/rust-headless-chrome/rust-headless-chrome

#### Issues:
- ❌ Synchronous API (not async-friendly)
- ❌ No explicit multi-instance support in docs
- ❌ Less actively maintained than spider_chrome
- ❌ Would require significant refactoring

#### Implementation Effort: **HIGH** ⏱️ 4-8 hours
- Complete rewrite of pool.rs and launcher.rs
- Convert from async to sync patterns
- Unknown if it solves SingletonLock issue

---

### 4. **fantoccini** (Not Recommended)

**Crate**: `fantoccini`
**Protocol**: WebDriver

#### Issues:
- ❌ Requires chromedriver/geckodriver as separate process
- ❌ WebDriver protocol (different from CDP)
- ❌ Geckodriver doesn't support multiple instances
- ❌ Additional deployment complexity

#### Implementation Effort: **VERY HIGH** ⏱️ 8+ hours
- Complete rewrite of CDP-based code
- Deploy and manage chromedriver separately
- Different protocol and capabilities

---

## Recommendation: spider_chrome

### Why spider_chrome?

1. **Minimal Risk**: Drop-in replacement = minimal code changes
2. **High Confidence**: Designed specifically for concurrent browser automation
3. **Low Effort**: 30-60 minutes vs 2-8+ hours for alternatives
4. **Active Development**: Part of spider-rs ecosystem, well-maintained
5. **Performance**: SIMD optimizations, better CDP handling
6. **Future-Proof**: Keeps CDP protocol up to date

### Implementation Plan

#### Phase 1: Quick Test (15 minutes)
```bash
# Update workspace Cargo.toml
[dependencies]
spider_chrome = "2.37.128"  # Replace chromiumoxide

# Update 3 files
sed -i 's/chromiumoxide/spider_chrome/g' crates/riptide-headless/src/*.rs

# Rebuild
cargo build --release --bin riptide-api
```

#### Phase 2: Validation (15 minutes)
```bash
# Start API
pkill -f riptide-api
cargo run --release --bin riptide-api > /tmp/spider-test.log 2>&1 &
sleep 15

# Check browser pool initialization
tail -50 /tmp/spider-test.log | grep -E "(Browser pool|initial_browsers)"

# Verify health
curl -s http://localhost:8080/healthz | jq '.status, .dependencies.headless_service'

# Check for SingletonLock errors
grep -i singleton /tmp/spider-test.log
```

#### Phase 3: Rollback if Needed (5 minutes)
```bash
git checkout HEAD -- Cargo.toml crates/riptide-headless/src/*.rs
cargo build --release --bin riptide-api
```

---

## Decision Matrix

| Criteria | spider_chrome | headless_browser | headless_chrome | fantoccini |
|----------|---------------|------------------|-----------------|------------|
| **Implementation Time** | ⏱️ 30-60 min | ⏱️ 2-4 hours | ⏱️ 4-8 hours | ⏱️ 8+ hours |
| **Success Likelihood** | ✅ 90% | ⚠️ 70% | ⚠️ 50% | ❌ 40% |
| **Code Changes** | 🟢 Minimal | 🟡 Moderate | 🔴 Major | 🔴 Major |
| **API Compatibility** | ✅ Drop-in | ❌ Different | ❌ Different | ❌ Different |
| **Concurrency Support** | ✅ Built-in | ✅ Built-in | ⚠️ Unknown | ⚠️ Limited |
| **Maintenance** | ✅ Active | ✅ Active | ⚠️ Moderate | ⚠️ Moderate |
| **Risk Level** | 🟢 Low | 🟡 Medium | 🔴 High | 🔴 High |

---

## Next Steps

### Option A: Implement spider_chrome Now (Recommended)
- ⏱️ Total time: 45-60 minutes
- 🎯 High success probability
- ✅ Fixes browser pool issue
- ✅ Phase 2A fully healthy

### Option B: Defer to Phase 2B
- Keep current "degraded" status
- Implement alongside Grafana dashboards
- More time for testing and validation

### Option C: Accept Current State
- Phase 2A objectives achieved
- 95% functionality works fine
- Browser pool enhancement can wait

---

## References

- spider_chrome: https://github.com/spider-rs/spider_chrome
- headless-browser: https://github.com/spider-rs/headless-browser
- chromiumoxide comparison: https://crates.io/crates/spider_chrome
- CDP Protocol: https://chromedevtools.github.io/devtools-protocol/

---

**Created**: 2025-10-13 20:30 UTC
**Author**: Hive Mind Collective
**Decision**: Awaiting user confirmation
