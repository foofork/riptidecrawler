# Phase 2: Spider-Chrome Migration Architecture

**Status:** Design Complete - Ready for Implementation
**Created:** 2025-10-20
**Track:** Phase 2 - Engine Migration
**Estimated Duration:** 12-16 days

## Executive Summary

This document provides a comprehensive architecture for migrating from chromiumoxide to spider-chrome. The migration follows a phased approach with clear rollback strategies and zero-downtime requirements.

### Key Findings

**Current State:**
- chromiumoxide 0.7.0 (via spider_chrome package)
- spider_chrome 2.37.128 exports as "chromiumoxide" library name
- Type incompatibility: `spider_chromiumoxide_cdp` vs `chromiumoxide_cdp`
- Abstraction layer exists but incomplete

**Target State:**
- Pure spider_chrome API usage
- No chromiumoxide dependency
- 100% spider-chrome for all browser operations
- Maintained performance and stealth features

**Critical Discovery:**
The codebase already uses spider_chrome! The package `spider_chrome` exports its library as "chromiumoxide" for API compatibility. However, there are type-level incompatibilities between `spider_chromiumoxide_cdp` (v0.7.4) and standard `chromiumoxide_cdp` (v0.7.0).

---

## Table of Contents

1. [Current Architecture Analysis](#current-architecture-analysis)
2. [API Mapping: chromiumoxide → spider_chrome](#api-mapping)
3. [Type Conversion Strategy](#type-conversion-strategy)
4. [Migration Phases](#migration-phases)
5. [Architecture Diagrams](#architecture-diagrams)
6. [Risk Mitigation](#risk-mitigation)
7. [Rollback Plan](#rollback-plan)
8. [Testing Strategy](#testing-strategy)
9. [Performance Impact](#performance-impact)
10. [Implementation Checklist](#implementation-checklist)

---

## Current Architecture Analysis

### Package Dependency Tree

```
Current State (ACTUAL):
┌────────────────────────────────────┐
│   riptide-browser-abstraction      │
│   ├─ spider_chrome: 2.37.128       │ ← Exports as "chromiumoxide"
│   │  └─ spider_chromiumoxide_cdp   │ ← v0.7.4 (forked)
│   └─ Use: chromiumoxide::*         │ ← Actually spider_chrome!
└────────────────────────────────────┘
```

### Key Insight

**The migration is NOT chromiumoxide → spider_chrome**
**It is: spider_chrome (as chromiumoxide) → pure spider_chrome API**

### Current Implementation Files

| File | Lines | Engine | Status |
|------|-------|--------|--------|
| `riptide-browser-abstraction/src/chromiumoxide_impl.rs` | 189 | spider_chrome (as chromiumoxide) | ✅ Implemented |
| `riptide-browser-abstraction/src/spider_impl.rs` | 179 | spider_chrome (native API) | ⚠️ Incomplete |
| `riptide-engine/src/launcher.rs` | 400+ | chromiumoxide | Needs update |
| `riptide-engine/src/pool.rs` | 800+ | chromiumoxide | Needs update |
| `riptide-headless/src/launcher.rs` | 400+ | spider_chrome | ✅ Updated |
| `riptide-headless/src/pool.rs` | 800+ | chromiumoxide | Needs update |

### Trait Abstraction Status

**Implemented:**
```rust
// ✅ Exists
pub trait BrowserEngine {
    async fn new_page(&self) -> AbstractionResult<Box<dyn PageHandle>>;
    async fn close(&self) -> AbstractionResult<()>;
    async fn version(&self) -> AbstractionResult<String>;
    fn engine_type(&self) -> EngineType;
}

pub trait PageHandle {
    async fn goto(&self, url: &str, params: NavigateParams) -> AbstractionResult<()>;
    async fn content(&self) -> AbstractionResult<String>;
    async fn evaluate(&self, script: &str) -> AbstractionResult<serde_json::Value>;
    async fn screenshot(&self, params: ScreenshotParams) -> AbstractionResult<Vec<u8>>;
    async fn pdf(&self, params: PdfParams) -> AbstractionResult<Vec<u8>>;
    async fn wait_for_navigation(&self, timeout_ms: u64) -> AbstractionResult<()>;
    async fn close(&self) -> AbstractionResult<()>;
}
```

**Missing/Incomplete:**
- `EngineType::SpiderChrome` variant (only `Chromiumoxide` exists)
- `SpiderChromeEngine` implementation (skeleton exists, incomplete)
- `SpiderChromePage` implementation (skeleton exists, incomplete)
- Screenshot/PDF implementations in spider_impl.rs

---

## API Mapping

### Core API Mapping Table

| Operation | chromiumoxide (current) | spider_chrome (target) | Compatibility |
|-----------|------------------------|------------------------|---------------|
| **Browser Lifecycle** ||||
| Launch | `Browser::launch(config)` | `Browser::launch(config)` | ✅ Identical |
| Connect | `Browser::connect(url)` | `Browser::connect(url)` | ✅ Identical |
| New Page | `browser.new_page(url)` | `browser.new_page(url)` | ✅ Identical |
| Close | `browser.close()` | `browser.close()` | ⚠️ Signature differs |
| Version | `browser.version()` | `browser.version()` | ✅ Identical |
| **Page Navigation** ||||
| Navigate | `page.goto(url)` | `page.goto(url)` | ✅ Identical |
| Reload | `page.reload()` | `page.reload()` | ✅ Identical |
| Back | `page.go_back()` | `page.go_back()` | ✅ Identical |
| Forward | `page.go_forward()` | `page.go_forward()` | ✅ Identical |
| **Content Access** ||||
| HTML | `page.content()` | `page.content()` | ✅ Identical |
| URL | `page.url()` | `page.url()` | ✅ Identical |
| Title | `page.title()` | `page.title()` | ✅ Identical |
| **JavaScript** ||||
| Evaluate | `page.evaluate(script)` | `page.evaluate(script)` | ⚠️ Parameter type differs |
| Add Script | `page.add_script_tag()` | `page.add_script_tag()` | ✅ Identical |
| **Screenshots** ||||
| Screenshot | `page.screenshot(params)` | ❌ CDP required | ❌ Not exposed |
| **PDF** ||||
| PDF | `page.pdf(params)` | ❌ CDP required | ❌ Not exposed |
| **Wait Operations** ||||
| Wait Nav | `page.wait_for_navigation()` | `page.wait_for_navigation()` | ✅ Identical |
| Timeout | `page.set_default_timeout(ms)` | ⚠️ No direct API | ❌ Different |
| **Cleanup** ||||
| Close Page | `page.close()` | `page.close()` | ⚠️ Takes ownership |

### Detailed Method Signatures

#### Browser::close()

**chromiumoxide (via spider_chrome):**
```rust
// From chromiumoxide_impl.rs (line 46-53)
async fn close(&self) -> AbstractionResult<()> {
    // chromiumoxide Browser.close() requires &mut self
    // Since we're using Arc<Browser> for thread safety, we can't call it
    warn!("explicit browser close not supported through Arc");
    Ok(())
}
```

**spider_chrome native:**
```rust
// From spider_impl.rs (line 45-54)
async fn close(&self) -> AbstractionResult<()> {
    self.browser
        .close()
        .await
        .map_err(|e| AbstractionError::BrowserClose(e.to_string()))?;
    Ok(())
}
```

**Issue:** chromiumoxide's `close()` requires `&mut self`, incompatible with `Arc<Browser>`.
**Solution:** spider_chrome's `close()` works with shared references.

#### Page::evaluate()

**chromiumoxide (via spider_chrome):**
```rust
// From chromiumoxide_impl.rs (line 118-128)
async fn evaluate(&self, script: &str) -> AbstractionResult<serde_json::Value> {
    let result = self.page
        .evaluate(script)  // Takes &str directly
        .await
        .map_err(|e| AbstractionError::Evaluation(e.to_string()))?;

    result.into_value()
        .map_err(|e| AbstractionError::Evaluation(e.to_string()))
}
```

**spider_chrome native:**
```rust
// From spider_impl.rs (line 110-122)
async fn evaluate(&self, script: &str) -> AbstractionResult<serde_json::Value> {
    let result = self.page
        .evaluate(script)  // Takes &str, same signature
        .await
        .map_err(|e| AbstractionError::Evaluation(e.to_string()))?;

    result.into_value()
        .map_err(|e| AbstractionError::Evaluation(e.to_string()))
}
```

**Status:** ✅ Identical - No changes needed

#### Page::screenshot()

**chromiumoxide (via spider_chrome):**
```rust
// From chromiumoxide_impl.rs (line 130-139)
async fn screenshot(&self, _params: ScreenshotParams) -> AbstractionResult<Vec<u8>> {
    self.page
        .screenshot(chromiumoxide::page::ScreenshotParams::default())
        .await
        .map_err(|e| AbstractionError::Screenshot(e.to_string()))
}
```

**spider_chrome native:**
```rust
// From spider_impl.rs (line 124-132)
async fn screenshot(&self, _params: ScreenshotParams) -> AbstractionResult<Vec<u8>> {
    debug!("Screenshot not directly supported in spider-chrome");
    warn!("Spider-chrome screenshot requires manual CDP implementation");

    // Would need to call CDP directly
    Err(AbstractionError::Unsupported(
        "screenshot not yet implemented for spider-chrome".to_string(),
    ))
}
```

**Issue:** spider_chrome doesn't expose `screenshot()` method directly.
**Solution:** Implement via CDP `Page.captureScreenshot` command.

#### Page::pdf()

**chromiumoxide (via spider_chrome):**
```rust
// From chromiumoxide_impl.rs (line 141-150)
async fn pdf(&self, _params: PdfParams) -> AbstractionResult<Vec<u8>> {
    self.page
        .pdf(Default::default())
        .await
        .map_err(|e| AbstractionError::PdfGeneration(e.to_string()))
}
```

**spider_chrome native:**
```rust
// From spider_impl.rs (line 134-142)
async fn pdf(&self, _params: PdfParams) -> AbstractionResult<Vec<u8>> {
    debug!("PDF generation not directly supported in spider-chrome");
    warn!("Spider-chrome PDF requires manual CDP implementation");

    Err(AbstractionError::Unsupported(
        "pdf not yet implemented for spider-chrome".to_string(),
    ))
}
```

**Issue:** spider_chrome doesn't expose `pdf()` method directly.
**Solution:** Implement via CDP `Page.printToPDF` command.

---

## Type Conversion Strategy

### CDP Type Hierarchy

```
spider_chromiumoxide_cdp v0.7.4 (forked)
├── cdp::browser_protocol::page::CaptureScreenshotParams
├── cdp::browser_protocol::page::PrintToPdfParams
├── cdp::browser_protocol::emulation::*
└── cdp::js_protocol::runtime::*

vs.

chromiumoxide_cdp v0.7.0 (standard)
├── cdp::browser_protocol::page::CaptureScreenshotParams
├── cdp::browser_protocol::page::PrintToPdfParams
├── cdp::browser_protocol::emulation::*
└── cdp::js_protocol::runtime::*
```

### Conversion Approach

**Option 1: Serialize → Deserialize (Safe, Slow)**
```rust
fn convert_screenshot_params(
    params: &chromiumoxide_cdp::page::ScreenshotParams
) -> Result<spider_chromiumoxide_cdp::page::ScreenshotParams> {
    let json = serde_json::to_value(params)?;
    serde_json::from_value(json)
}
```

**Option 2: Field-by-Field Mapping (Fast, Brittle)**
```rust
fn convert_screenshot_params(
    params: &chromiumoxide_cdp::page::ScreenshotParams
) -> spider_chromiumoxide_cdp::page::ScreenshotParams {
    spider_chromiumoxide_cdp::page::ScreenshotParams {
        format: params.format.clone(),
        quality: params.quality,
        // ... copy all fields
    }
}
```

**Chosen: Option 3 - Abstraction Layer (Recommended)**
```rust
// Our own types in riptide-browser-abstraction
pub struct ScreenshotParams {
    pub format: ScreenshotFormat,
    pub quality: Option<i64>,
    pub full_page: bool,
    pub clip: Option<Viewport>,
}

impl Into<spider_chromiumoxide_cdp::page::CaptureScreenshotParams> for ScreenshotParams {
    // Convert to spider CDP types
}
```

---

## Migration Phases

### Phase 1: Engine Abstraction (Complete) ✅

**Status:** Implemented in `riptide-browser-abstraction` crate

**Deliverables:**
- ✅ `BrowserEngine` trait
- ✅ `PageHandle` trait
- ✅ `ChromiumoxideEngine` (using spider_chrome as chromiumoxide)
- ⚠️ `SpiderChromeEngine` (skeleton exists, needs completion)

### Phase 2: Screenshot/PDF Implementation (4 days)

**Goal:** Implement missing screenshot and PDF functionality in `SpiderChromePage`

#### Day 1-2: CDP Integration

**Tasks:**
1. Add CDP command execution helper to `SpiderChromePage`
2. Implement `Page.captureScreenshot` wrapper
3. Implement `Page.printToPDF` wrapper
4. Add parameter conversion from `ScreenshotParams` → CDP params

**Code Changes:**
```rust
// In spider_impl.rs
impl SpiderChromePage {
    async fn execute_cdp<T: Command>(&self, command: T) -> AbstractionResult<T::Response> {
        self.page
            .execute(command)
            .await
            .map_err(|e| AbstractionError::CDP(e.to_string()))
    }

    async fn screenshot_internal(&self, params: ScreenshotParams) -> AbstractionResult<Vec<u8>> {
        use spider_chrome::cdp::browser_protocol::page::{
            CaptureScreenshotParams, CaptureScreenshotFormat
        };

        let cdp_params = CaptureScreenshotParams {
            format: match params.format {
                ScreenshotFormat::PNG => Some(CaptureScreenshotFormat::Png),
                ScreenshotFormat::JPEG => Some(CaptureScreenshotFormat::Jpeg),
            },
            quality: params.quality,
            clip: params.clip.map(|c| /* convert viewport */),
            from_surface: None,
            capture_beyond_viewport: Some(params.full_page),
        };

        let result = self.execute_cdp(cdp_params).await?;
        Ok(base64::decode(&result.data)?)
    }
}
```

**Tests:**
- Screenshot format conversion (PNG, JPEG)
- PDF generation with custom page size
- Viewport clipping
- Full page screenshot

#### Day 3-4: Headless Migration

**Goal:** Update `riptide-headless` crate to use pure spider_chrome API

**Files to Update:**
1. `crates/riptide-headless/src/pool.rs` (800+ lines)
   - Change `use chromiumoxide::*` → `use spider_chrome::*`
   - Update type annotations
   - Test pool lifecycle

2. `crates/riptide-headless/src/launcher.rs` (✅ Already updated)
   - Already uses spider_chrome natively
   - No changes needed

3. `crates/riptide-headless/src/hybrid_fallback.rs`
   - Update engine selection logic
   - Ensure fallback works with new types

**Validation:**
```bash
cargo test -p riptide-headless --all-features
```

### Phase 3: Engine Migration (4 days)

**Goal:** Update `riptide-engine` crate to use spider_chrome

#### Day 1-2: Pool Migration

**File:** `crates/riptide-engine/src/pool.rs`

**Changes:**
```diff
- use chromiumoxide::{Browser, BrowserConfig, Page};
+ use spider_chrome::{Browser, BrowserConfig, Page};

- use chromiumoxide_cdp::cdp::browser_protocol::emulation::*;
+ use spider_chrome::cdp::browser_protocol::emulation::*;
```

**Risk:** High - Pool is critical infrastructure
**Mitigation:**
- Extensive testing
- Gradual rollout with feature flag
- Keep chromiumoxide version as backup

#### Day 3-4: Launcher Migration

**File:** `crates/riptide-engine/src/launcher.rs`

**Changes:**
```diff
- use chromiumoxide::{BrowserConfig, Page};
+ use spider_chrome::{BrowserConfig, Page};
```

**Tests:**
- Browser launch
- Page creation
- Stealth injection
- Pool integration

### Phase 4: Abstraction Layer Cleanup (2 days)

**Goal:** Remove redundant chromiumoxide-as-spider_chrome implementation

**Tasks:**
1. Mark `ChromiumoxideEngine` as deprecated
2. Update all code to use `SpiderChromeEngine`
3. Remove `chromiumoxide_impl.rs`
4. Update `EngineType` enum:
   ```rust
   pub enum EngineType {
       SpiderChrome, // Only variant
   }
   ```

### Phase 5: Testing & Validation (4 days)

**Comprehensive Test Suite:**

1. **Unit Tests** (Day 1)
   - All PageHandle methods
   - Parameter conversions
   - Error handling

2. **Integration Tests** (Day 2)
   - Browser lifecycle
   - Pool operations
   - Fallback logic
   - Stealth features

3. **Performance Tests** (Day 3)
   - Concurrency benchmarks (200+ sessions)
   - Memory usage (<50MB per session)
   - Response time (<100ms)

4. **Production Validation** (Day 4)
   - Canary deployment (10% traffic)
   - Monitor error rates
   - Validate stealth effectiveness
   - Performance regression checks

---

## Architecture Diagrams

### Before Migration

```
┌─────────────────────────────────────────────────────┐
│           Application Layer                          │
├─────────────────────────────────────────────────────┤
│  riptide-engine        │    riptide-headless         │
│  (chromiumoxide)       │    (spider_chrome)          │
└────────────┬───────────┴─────────┬───────────────────┘
             │                     │
             ▼                     ▼
┌─────────────────────────────────────────────────────┐
│      riptide-browser-abstraction                    │
│  ┌──────────────┐       ┌──────────────┐           │
│  │ Chromium-    │       │ SpiderChrome │           │
│  │ oxide Engine │       │ Engine       │           │
│  │ (complete)   │       │ (incomplete) │           │
│  └──────┬───────┘       └──────┬───────┘           │
└─────────┼──────────────────────┼───────────────────┘
          │                      │
          ▼                      ▼
┌─────────────────────────────────────────────────────┐
│        spider_chrome v2.37.128                      │
│        (exports as "chromiumoxide")                 │
│  ┌─────────────────────────────────────────┐       │
│  │  spider_chromiumoxide_cdp v0.7.4        │       │
│  └─────────────────────────────────────────┘       │
└─────────────────────────────────────────────────────┘
```

### After Migration

```
┌─────────────────────────────────────────────────────┐
│           Application Layer                          │
├─────────────────────────────────────────────────────┤
│  riptide-engine        │    riptide-headless         │
│  (spider_chrome)       │    (spider_chrome)          │
└────────────┬───────────┴─────────┬───────────────────┘
             │                     │
             ▼                     ▼
┌─────────────────────────────────────────────────────┐
│      riptide-browser-abstraction                    │
│  ┌──────────────────────────────────────┐           │
│  │ SpiderChrome Engine (complete)       │           │
│  │  ✅ Screenshot (CDP)                 │           │
│  │  ✅ PDF (CDP)                        │           │
│  │  ✅ All PageHandle methods           │           │
│  └──────────────┬───────────────────────┘           │
└─────────────────┼───────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────┐
│        spider_chrome v2.37.128                      │
│        (native API usage)                           │
│  ┌─────────────────────────────────────────┐       │
│  │  spider_chromiumoxide_cdp v0.7.4        │       │
│  │  ✅ Direct CDP access                   │       │
│  └─────────────────────────────────────────┘       │
└─────────────────────────────────────────────────────┘
```

### Data Flow: Screenshot Operation

```
┌──────────────┐
│ Application  │
│ Code         │
└──────┬───────┘
       │ page.screenshot(params)
       ▼
┌──────────────────────┐
│ PageHandle Trait     │
│ (Abstraction Layer)  │
└──────┬───────────────┘
       │ Convert params
       ▼
┌──────────────────────┐
│ SpiderChromePage     │
│ Implementation       │
└──────┬───────────────┘
       │ execute_cdp()
       ▼
┌──────────────────────────────┐
│ spider_chrome CDP            │
│ Page.captureScreenshot       │
└──────┬───────────────────────┘
       │ WebSocket → Chrome
       ▼
┌──────────────────────┐
│ Chrome Browser       │
│ (Headless)           │
└──────┬───────────────┘
       │ PNG/JPEG bytes
       ▼
┌──────────────────────┐
│ Base64 Decode        │
│ Return Vec<u8>       │
└──────────────────────┘
```

---

## Risk Mitigation

### Risk Matrix

| Risk | Probability | Impact | Mitigation | Rollback Time |
|------|-------------|--------|------------|---------------|
| **Screenshot/PDF CDP implementation breaks** | Medium (40%) | High | Extensive CDP testing, fallback to chromiumoxide | 1 hour |
| **Pool migration causes crashes** | Low (20%) | Critical | Feature flag, gradual rollout, monitoring | 30 minutes |
| **Performance regression** | Low (15%) | High | Benchmarks before/after, profiling | 2 hours |
| **Type conversion errors** | Medium (30%) | Medium | Comprehensive type tests, validation | 1 hour |
| **Stealth features broken** | Low (10%) | Critical | Stealth test suite, fingerprint validation | 2 hours |
| **Memory leaks in new implementation** | Low (15%) | High | Long-running tests, memory profiling | 4 hours |

### Mitigation Strategies

#### 1. Feature Flags

```rust
#[cfg(feature = "use-spider-chrome")]
use spider_chrome as engine;

#[cfg(not(feature = "use-spider-chrome"))]
use chromiumoxide as engine;
```

**Rollout Plan:**
1. Week 1: Internal testing only
2. Week 2: 10% production traffic
3. Week 3: 50% production traffic
4. Week 4: 100% production traffic

#### 2. Canary Deployment

```rust
pub enum EngineSelection {
    SpiderChrome,
    Chromiumoxide, // Fallback
    Auto { spider_percentage: u8 },
}

impl EngineSelection {
    pub fn should_use_spider(&self, request_hash: u64) -> bool {
        match self {
            Self::SpiderChrome => true,
            Self::Chromiumoxide => false,
            Self::Auto { spider_percentage } => {
                (request_hash % 100) < *spider_percentage as u64
            }
        }
    }
}
```

#### 3. Monitoring & Alerts

**Metrics to Track:**
- Screenshot success rate (target: >99.9%)
- PDF generation success rate (target: >99.9%)
- Page load time (target: <3s p95)
- Memory usage per session (target: <50MB)
- Pool health (target: >95% healthy browsers)
- Error rate by engine type

**Alerts:**
- Error rate >1% for 5 minutes
- Memory usage >100MB per session
- Response time >5s p95
- Pool health <90%

#### 4. Automated Testing

```bash
# Pre-migration validation
cargo test --all-features
cargo bench --all-features

# Post-migration validation
./scripts/validate-migration.sh
  - Run all tests
  - Compare benchmarks
  - Check memory usage
  - Validate stealth features
  - Screenshot/PDF quality checks
```

---

## Rollback Plan

### Rollback Triggers

**Automatic Rollback:**
- Error rate >5% for 10 minutes
- Memory leak detected (>200MB per session)
- Critical functionality broken (screenshot/PDF)

**Manual Rollback:**
- Stealth detection increased >20%
- Performance degradation >50%
- Production incident

### Rollback Procedures

#### Level 1: Feature Flag Rollback (30 seconds)

```rust
// In runtime config
pub const USE_SPIDER_CHROME: bool = false; // Set to false
```

```bash
# Update config and restart
kubectl set env deployment/riptide USE_SPIDER_CHROME=false
kubectl rollout restart deployment/riptide
```

#### Level 2: Code Rollback (2 hours)

```bash
# Revert to previous release
git revert HEAD~1  # Revert migration commit
cargo build --release
docker build -t riptide:rollback .
kubectl set image deployment/riptide riptide=riptide:rollback
```

#### Level 3: Full Rollback (4 hours)

```bash
# Restore chromiumoxide implementation
git checkout main~10  # Before migration started
cargo clean
cargo build --release --all-features
# Full redeployment
```

### Rollback Validation

After rollback:
1. ✅ All tests passing
2. ✅ Error rate <0.1%
3. ✅ Performance baseline restored
4. ✅ Memory usage normal
5. ✅ Stealth features working

---

## Testing Strategy

### Test Pyramid

```
           ┌─────────────┐
           │   Manual    │ 5%
           │   E2E       │
           ├─────────────┤
           │ Integration │ 15%
           │   Tests     │
           ├─────────────┤
           │    Unit     │ 80%
           │   Tests     │
           └─────────────┘
```

### Unit Tests (80% coverage)

**Test Coverage by Component:**

1. **SpiderChromePage Implementation** (30 tests)
   ```rust
   #[tokio::test]
   async fn test_screenshot_png_format() { }

   #[tokio::test]
   async fn test_screenshot_jpeg_quality() { }

   #[tokio::test]
   async fn test_pdf_default_params() { }

   #[tokio::test]
   async fn test_pdf_custom_page_size() { }

   #[tokio::test]
   async fn test_evaluate_javascript() { }
   ```

2. **Parameter Conversion** (20 tests)
   ```rust
   #[test]
   fn test_screenshot_params_to_cdp() { }

   #[test]
   fn test_pdf_params_to_cdp() { }

   #[test]
   fn test_viewport_conversion() { }
   ```

3. **Error Handling** (15 tests)
   ```rust
   #[tokio::test]
   async fn test_screenshot_timeout() { }

   #[tokio::test]
   async fn test_invalid_url_error() { }

   #[tokio::test]
   async fn test_cdp_connection_lost() { }
   ```

### Integration Tests (15% coverage)

**Test Scenarios:**

1. **Browser Lifecycle** (10 tests)
   - Launch browser
   - Create multiple pages
   - Screenshot all pages
   - Close pages
   - Close browser
   - Verify no leaks

2. **Pool Operations** (15 tests)
   - Pool initialization
   - Browser checkout/checkin
   - Pool exhaustion handling
   - Health checks
   - Automatic recovery
   - Memory limits

3. **Stealth Features** (8 tests)
   - Navigator.webdriver check
   - Chrome object presence
   - WebGL fingerprint
   - Canvas fingerprint
   - Audio context
   - Screen properties

### Performance Tests (5% coverage)

**Benchmarks:**

1. **Concurrency Test**
   ```rust
   #[tokio::test]
   #[ignore]
   async fn bench_concurrent_screenshots() {
       // Target: 200+ concurrent sessions
       // Time: <30s for 200 screenshots
   }
   ```

2. **Memory Test**
   ```rust
   #[tokio::test]
   #[ignore]
   async fn bench_memory_usage() {
       // Target: <50MB per session
       // Run: 100 iterations, measure peak
   }
   ```

3. **Response Time Test**
   ```rust
   #[tokio::test]
   #[ignore]
   async fn bench_screenshot_latency() {
       // Target: <100ms p50, <300ms p95
   }
   ```

### Test Execution Plan

```bash
# Phase 2: Screenshot/PDF
cargo test -p riptide-browser-abstraction spider_chrome --lib
cargo test -p riptide-browser-abstraction screenshot --lib
cargo test -p riptide-browser-abstraction pdf --lib

# Phase 3: Engine Migration
cargo test -p riptide-engine --lib
cargo test -p riptide-headless --lib

# Phase 4: Integration
cargo test --workspace --all-features

# Phase 5: Performance
cargo test --workspace --all-features --ignored

# Benchmarks
cargo bench --all-features
```

---

## Performance Impact

### Expected Performance Changes

| Metric | Before | After | Change |
|--------|--------|-------|--------|
| **Concurrency** | 50-70 sessions | 200+ sessions | +185% |
| **Response Time (p50)** | 150ms | <100ms | -33% |
| **Response Time (p95)** | 300ms | <300ms | 0% |
| **Memory per Session** | 80-120MB | <50MB | -42% |
| **Browser Launch Time** | 1.5s | 1.2s | -20% |
| **Screenshot Time** | 200ms | <300ms | +50% ⚠️ |
| **PDF Generation** | 500ms | <1000ms | +100% ⚠️ |

**Note:** Screenshot/PDF times may increase due to CDP overhead, but stay within acceptable limits.

### Performance Monitoring

**Before Migration Baseline:**
```bash
# Capture baseline metrics
cargo bench --all-features > baseline.txt

# Memory profiling
valgrind --tool=massif --massif-out-file=baseline.massif ./target/release/riptide
```

**After Migration Validation:**
```bash
# Compare benchmarks
cargo bench --all-features > migrated.txt
diff baseline.txt migrated.txt

# Memory comparison
valgrind --tool=massif --massif-out-file=migrated.massif ./target/release/riptide
ms_print baseline.massif > baseline_mem.txt
ms_print migrated.massif > migrated_mem.txt
diff baseline_mem.txt migrated_mem.txt
```

**Acceptance Criteria:**
- ✅ No regression >10% in any metric
- ✅ Concurrency improved by >100%
- ✅ Memory usage reduced or stable
- ✅ No new memory leaks

---

## Implementation Checklist

### Phase 1: Engine Abstraction ✅ (Complete)

- [x] Create `riptide-browser-abstraction` crate
- [x] Define `BrowserEngine` trait
- [x] Define `PageHandle` trait
- [x] Implement `ChromiumoxideEngine`
- [x] Implement `ChromiumoxidePage`
- [x] Basic unit tests

### Phase 2: Screenshot/PDF Implementation (4 days)

**Day 1-2: CDP Integration**
- [ ] Add CDP command executor to `SpiderChromePage`
- [ ] Implement `Page.captureScreenshot` wrapper
- [ ] Implement `Page.printToPDF` wrapper
- [ ] Add parameter conversion helpers
- [ ] Unit tests for CDP commands

**Day 3-4: Headless Migration**
- [ ] Update `riptide-headless/pool.rs` imports
- [ ] Update `riptide-headless/hybrid_fallback.rs`
- [ ] Run headless integration tests
- [ ] Fix any type errors
- [ ] Performance validation

### Phase 3: Engine Migration (4 days)

**Day 1-2: Pool Migration**
- [ ] Update `riptide-engine/pool.rs` imports
- [ ] Replace chromiumoxide types with spider_chrome
- [ ] Update CDP command usage
- [ ] Run pool tests
- [ ] Health check validation

**Day 3-4: Launcher Migration**
- [ ] Update `riptide-engine/launcher.rs` imports
- [ ] Update stealth injection code
- [ ] Run launcher tests
- [ ] Integration testing
- [ ] End-to-end validation

### Phase 4: Abstraction Layer Cleanup (2 days)

**Day 1: Deprecation**
- [ ] Mark `ChromiumoxideEngine` as deprecated
- [ ] Add deprecation warnings
- [ ] Update documentation
- [ ] Migration guide for users

**Day 2: Removal**
- [ ] Remove `chromiumoxide_impl.rs`
- [ ] Update `EngineType` enum
- [ ] Remove deprecated code paths
- [ ] Final cleanup

### Phase 5: Testing & Validation (4 days)

**Day 1: Unit Tests**
- [ ] All PageHandle methods tested
- [ ] Parameter conversions tested
- [ ] Error handling tested
- [ ] 80% code coverage achieved

**Day 2: Integration Tests**
- [ ] Browser lifecycle tests
- [ ] Pool operation tests
- [ ] Stealth feature tests
- [ ] All integration tests passing

**Day 3: Performance Tests**
- [ ] Concurrency benchmarks
- [ ] Memory usage tests
- [ ] Response time tests
- [ ] No regressions detected

**Day 4: Production Validation**
- [ ] Canary deployment (10%)
- [ ] Monitor error rates
- [ ] Performance validation
- [ ] Stealth effectiveness check
- [ ] Sign-off for full rollout

---

## Timeline Summary

### Critical Path (12-16 days)

```
Day 1-2:   Screenshot/PDF CDP Implementation
Day 3-4:   Headless Migration
Day 5-6:   Pool Migration
Day 7-8:   Launcher Migration
Day 9-10:  Abstraction Cleanup
Day 11-14: Testing & Validation
Day 15-16: Production Rollout (optional buffer)
```

### Parallel Work Streams

**Can be done concurrently:**
- Screenshot/PDF implementation (2 engineers)
- Test writing (1 engineer)
- Documentation updates (1 engineer)

**Must be sequential:**
- Headless → Engine migration
- Migration → Cleanup
- Cleanup → Validation

---

## Success Criteria

### Technical Metrics

- ✅ All tests passing (100% pass rate)
- ✅ Code coverage >80%
- ✅ No performance regression >10%
- ✅ Memory usage <50MB per session
- ✅ Concurrency >200 sessions
- ✅ Screenshot success rate >99.9%
- ✅ PDF generation success rate >99.9%

### Operational Metrics

- ✅ Zero production incidents
- ✅ Error rate <0.1%
- ✅ Rollback plan tested
- ✅ Monitoring in place
- ✅ Documentation complete

### Business Metrics

- ✅ No user-facing disruptions
- ✅ Stealth effectiveness maintained
- ✅ Cost reduction (lower memory usage)
- ✅ Improved capacity (higher concurrency)

---

## References

### Documentation
- [Spider-Chrome API Analysis](/workspaces/eventmesh/docs/integration/SPIDER-CHROME-API-ANALYSIS.md)
- [Spider-Chrome Research](/workspaces/eventmesh/docs/research/spider-chrome-analysis.md)
- [ADR-001: Browser Automation Strategy](/workspaces/eventmesh/docs/architecture/ADR-001-browser-automation.md)
- [ADR-006: Spider-Chrome Compatibility](/workspaces/eventmesh/docs/architecture/ADR-006-spider-chrome-compatibility.md)

### Code
- `crates/riptide-browser-abstraction/` - Abstraction layer
- `crates/riptide-engine/src/pool.rs` - Browser pool
- `crates/riptide-engine/src/launcher.rs` - Browser launcher
- `crates/riptide-headless/src/pool.rs` - Headless pool
- `crates/riptide-headless/src/launcher.rs` - Headless launcher

### External
- [spider-chrome crate](https://crates.io/crates/spider_chrome)
- [spider-chrome GitHub](https://github.com/spider-rs/spider-chrome)
- [Chrome DevTools Protocol](https://chromedevtools.github.io/devtools-protocol/)

---

## Appendix

### A. Type Compatibility Matrix

| Type | chromiumoxide | spider_chrome | Compatible? |
|------|---------------|---------------|-------------|
| Browser | ✅ | ✅ | ✅ Same struct |
| Page | ✅ | ✅ | ✅ Same struct |
| BrowserConfig | ✅ | ✅ | ✅ Same struct |
| CDP Events | ❌ chromiumoxide_cdp | ❌ spider_chromiumoxide_cdp | ❌ Different packages |
| CDP Commands | ❌ chromiumoxide_cdp | ❌ spider_chromiumoxide_cdp | ❌ Different packages |
| ScreenshotParams | ⚠️ chromiumoxide::page | ⚠️ spider_chrome::cdp | ❌ Different types |
| PrintToPdfParams | ⚠️ chromiumoxide::page | ⚠️ spider_chrome::cdp | ❌ Different types |

### B. Migration Command Reference

```bash
# Phase 2: Screenshot/PDF
cd crates/riptide-browser-abstraction
cargo test spider_chrome::screenshot --lib
cargo test spider_chrome::pdf --lib

# Phase 3: Engine Migration
cd ../riptide-engine
rg "use chromiumoxide" src/
sed -i 's/use chromiumoxide/use spider_chrome/g' src/pool.rs
sed -i 's/use chromiumoxide/use spider_chrome/g' src/launcher.rs
cargo check
cargo test --lib

# Phase 4: Cleanup
cd ../riptide-browser-abstraction
git rm src/chromiumoxide_impl.rs
# Update lib.rs to remove chromiumoxide exports
cargo check

# Phase 5: Validation
cd ../..
cargo test --workspace --all-features
cargo bench --all-features
```

### C. Rollback Command Reference

```bash
# Level 1: Feature flag (30 seconds)
kubectl set env deployment/riptide USE_SPIDER_CHROME=false
kubectl rollout status deployment/riptide

# Level 2: Git revert (2 hours)
git revert HEAD~1
cargo build --release
docker build -t riptide:rollback .
kubectl set image deployment/riptide riptide=riptide:rollback

# Level 3: Full rollback (4 hours)
git checkout v1.0.0  # Before migration
cargo clean && cargo build --release --all-features
./deploy.sh production
```

---

**Document Version:** 1.0
**Author:** System Architecture Designer
**Status:** Design Complete - Ready for Implementation
**Next Review:** After Phase 2 Completion
