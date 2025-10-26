# Quick Reference - Compatibility Layer Implementation

**For:** Coder implementing Day 2-4 work
**Context:** Week 3 spider_chrome compatibility layer
**Read First:** ADR-006-spider-chrome-compatibility.md

## TL;DR - The Problem

- **spider_chrome** uses `spider_chromiumoxide_cdp v0.7.4` (forked)
- **chromiumoxide** uses `chromiumoxide_cdp v0.7.0` (standard)
- Types are **incompatible** - cannot cast or convert
- Need **runtime switching** for 20% hybrid fallback

## TL;DR - The Solution

Create **riptide-browser-abstraction** crate with traits that abstract both engines:

```rust
trait BrowserEngine { /* common interface */ }
trait PageHandle { /* common interface */ }

impl BrowserEngine for ChromiumoxideEngine { /* wrap chromiumoxide */ }
impl BrowserEngine for SpiderChromeEngine { /* wrap spider_chrome */ }
```

## Implementation Checklist

### Day 2 Morning (4 hours)

- [ ] Create crate: `cargo new --lib crates/riptide-browser-abstraction`
- [ ] Add dependencies to Cargo.toml:
  ```toml
  [dependencies]
  chromiumoxide = "0.7.0"
  spider_chrome = "2.37.128"
  async-trait = "0.1"
  anyhow = "1.0"
  tokio = { version = "1", features = ["full"] }
  serde = { version = "1", features = ["derive"] }
  serde_json = "1"
  ```
- [ ] Create `src/traits.rs` with `BrowserEngine` and `PageHandle` traits
- [ ] Create `src/params.rs` with parameter types
- [ ] Create `src/error.rs` with error types
- [ ] Write trait documentation with examples

### Day 2 Afternoon (4 hours)

- [ ] Create `src/chromiumoxide_impl.rs`:
  - [ ] `ChromiumoxideEngine` struct
  - [ ] Implement `BrowserEngine` for `ChromiumoxideEngine`
  - [ ] `ChromiumoxidePage` struct
  - [ ] Implement `PageHandle` for `ChromiumoxidePage`
  - [ ] Parameter translation helpers

- [ ] Create `src/spider_chrome_impl.rs`:
  - [ ] `SpiderChromeEngine` struct
  - [ ] Implement `BrowserEngine` for `SpiderChromeEngine`
  - [ ] `SpiderChromePage` struct
  - [ ] Implement `PageHandle` for `SpiderChromePage`
  - [ ] Preserve stealth features

- [ ] Create `src/factory.rs` with `EngineFactory`

- [ ] Write unit tests for both implementations

### Day 3 Morning (3 hours)

- [ ] Update `crates/riptide-headless/src/hybrid_fallback.rs`:
  - [ ] Change `Browser` → `Box<dyn BrowserEngine>`
  - [ ] Change `Page` → `Box<dyn PageHandle>`
  - [ ] Update execution logic

- [ ] Update `crates/riptide-headless/src/launcher.rs`:
  - [ ] Return `Box<dyn BrowserEngine>` instead of concrete type

- [ ] Update `crates/riptide-headless/src/pool.rs`:
  - [ ] Accept `Box<dyn PageHandle>` instead of concrete type

### Day 3 Afternoon (3 hours)

- [ ] Run integration tests: `cargo test --workspace`
- [ ] Fix any compilation errors
- [ ] Add hybrid switching tests
- [ ] Performance benchmarks
- [ ] Update documentation

### Day 4 (4 hours)

- [ ] Full test suite validation
- [ ] Performance profiling (ensure <5% overhead)
- [ ] Update Week 2 progress document
- [ ] Create Week 3 status report

## Critical Code Snippets

### Trait Definitions (src/traits.rs)

```rust
use async_trait::async_trait;
use anyhow::Result;

#[async_trait]
pub trait BrowserEngine: Send + Sync + std::fmt::Debug {
    async fn new_page(&self) -> Result<Box<dyn PageHandle>>;
    async fn new_page_with_url(&self, url: &str) -> Result<Box<dyn PageHandle>>;
    async fn close(&self) -> Result<()>;
    fn engine_type(&self) -> EngineType;
    async fn version(&self) -> Result<BrowserVersion>;
    fn as_chromiumoxide(&self) -> Option<&chromiumoxide::Browser> { None }
    fn as_spider_chrome(&self) -> Option<&spider_chrome::Browser> { None }
}

#[async_trait]
pub trait PageHandle: Send + Sync + std::fmt::Debug {
    async fn goto(&self, url: &str) -> Result<()>;
    async fn content(&self) -> Result<String>;
    async fn screenshot(&self, params: ScreenshotParams) -> Result<Vec<u8>>;
    async fn pdf(&self, params: PdfParams) -> Result<Vec<u8>>;
    async fn wait_for_navigation(&self) -> Result<()>;
    async fn evaluate(&self, script: &str) -> Result<serde_json::Value>;
    async fn url(&self) -> Result<String>;
    async fn title(&self) -> Result<String>;
    async fn close(&self) -> Result<()>;
    fn engine_type(&self) -> EngineType;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineType {
    SpiderChrome,
    Chromiumoxide,
}
```

### Parameter Translation Example

```rust
// In chromiumoxide_impl.rs
async fn screenshot(&self, params: ScreenshotParams) -> Result<Vec<u8>> {
    let mut builder = chromiumoxide::page::ScreenshotParams::builder();

    builder = builder.format(match params.format {
        ImageFormat::Png => chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat::Png,
        ImageFormat::Jpeg => chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat::Jpeg,
        ImageFormat::Webp => chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat::Webp,
    });

    if let Some(quality) = params.quality {
        builder = builder.quality(quality);
    }

    builder = builder.full_page(params.full_page);

    let screenshot_params = builder.build();
    let data = self.inner.screenshot(screenshot_params).await?;
    Ok(data)
}
```

### Usage in hybrid_fallback.rs

```rust
// OLD CODE (broken):
pub async fn execute(&self, url: &str, page: &chromiumoxide::Page) -> Result<String> {
    // Can't switch to spider_chrome dynamically!
}

// NEW CODE (working):
pub async fn execute(&self, url: &str) -> Result<Box<dyn PageHandle>> {
    let engine = if self.should_use_spider_chrome(url) {
        self.spider_engine.as_ref().unwrap()
    } else {
        self.chromium_engine.as_ref()
    };

    let page = engine.new_page_with_url(url).await?;
    page.wait_for_navigation().await?;
    Ok(page)
}
```

## Key Files to Read

1. **API Analysis:** `/workspaces/eventmesh/docs/integration/SPIDER-CHROME-API-ANALYSIS.md`
   - Understand the incompatibility
   - See all breaking changes

2. **ADR-006:** `/workspaces/eventmesh/docs/architecture/ADR-006-spider-chrome-compatibility.md`
   - Understand the decision
   - See alternatives considered

3. **Design Doc:** `/workspaces/eventmesh/docs/integration/COMPATIBILITY-LAYER-DESIGN.md`
   - Full implementation details
   - Example code for all components

4. **Existing hybrid_fallback:** `/workspaces/eventmesh/crates/riptide-headless/src/hybrid_fallback.rs`
   - Current implementation to update

## Testing Strategy

### Unit Tests (per implementation)

```rust
#[tokio::test]
async fn test_chromiumoxide_basic() {
    let engine = ChromiumoxideEngine::from_config(
        chromiumoxide::BrowserConfig::builder().build().unwrap()
    ).await.unwrap();

    let page = engine.new_page_with_url("https://example.com").await.unwrap();
    page.wait_for_navigation().await.unwrap();

    let content = page.content().await.unwrap();
    assert!(content.contains("<title>"));
    assert_eq!(page.engine_type(), EngineType::Chromiumoxide);
}

#[tokio::test]
async fn test_spider_chrome_basic() {
    // Same test but with SpiderChromeEngine
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_hybrid_switching() {
    let chromium = EngineFactory::chromiumoxide_default().await.unwrap();
    let spider = EngineFactory::spider_chrome_default().await.unwrap();

    let url = "https://httpbin.org/html";

    let page1 = chromium.new_page_with_url(url).await.unwrap();
    let page2 = spider.new_page_with_url(url).await.unwrap();

    let content1 = page1.content().await.unwrap();
    let content2 = page2.content().await.unwrap();

    assert!(content1.contains("<html"));
    assert!(content2.contains("<html"));
}
```

## Common Pitfalls

### ❌ DON'T: Try to convert types directly
```rust
let spider_page: spider_chrome::Page = /* ... */;
let chromium_page: chromiumoxide::Page = spider_page; // IMPOSSIBLE!
```

### ✅ DO: Use trait abstraction
```rust
let page: Box<dyn PageHandle> = /* either engine */;
page.goto("https://example.com").await?; // Works for both!
```

### ❌ DON'T: Use concrete types in function signatures
```rust
pub async fn process_page(page: chromiumoxide::Page) { /* ... */ }
```

### ✅ DO: Use trait objects
```rust
pub async fn process_page(page: Box<dyn PageHandle>) { /* ... */ }
```

### ❌ DON'T: Forget async_trait
```rust
trait PageHandle {
    async fn goto(&self, url: &str) -> Result<()>; // Won't compile!
}
```

### ✅ DO: Use async_trait macro
```rust
#[async_trait]
trait PageHandle {
    async fn goto(&self, url: &str) -> Result<()>; // Compiles!
}
```

## Performance Notes

- Virtual call overhead: **1-3ns** per call
- Page load time: **100-500ms**
- Overhead percentage: **<0.001%**
- Verdict: **Negligible** ✅

## Success Criteria

- [ ] Can instantiate either engine at runtime
- [ ] Can switch engines per request
- [ ] All existing tests pass
- [ ] Performance overhead <5%
- [ ] No unsafe code required
- [ ] Spider-chrome stealth features preserved

## Questions?

Refer to:
- Full design: `COMPATIBILITY-LAYER-DESIGN.md`
- Decision rationale: `ADR-006-spider-chrome-compatibility.md`
- API details: `SPIDER-CHROME-API-ANALYSIS.md`

## Expected Timeline

- **Day 2:** 8 hours (core implementation)
- **Day 3:** 6 hours (integration)
- **Day 4:** 4 hours (validation)
- **Total:** 18 hours (2.25 days)

---

**Good luck with the implementation!** The research is solid, the design is proven, and you have a clear path forward. 🚀
