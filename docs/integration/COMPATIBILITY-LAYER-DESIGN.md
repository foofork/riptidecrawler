# Compatibility Layer Design - riptide-browser-abstraction

**Date:** 2025-10-17
**Status:** Design Complete, Ready for Implementation
**ADR:** See ADR-006-spider-chrome-compatibility.md

## Overview

This document provides the detailed design for **riptide-browser-abstraction**, a compatibility layer that abstracts both chromiumoxide and spider_chrome behind common traits, enabling runtime engine selection and hybrid fallback.

## Crate Structure

```
crates/riptide-browser-abstraction/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs                     # Public API, re-exports
│   ├── traits.rs                  # BrowserEngine, PageHandle traits
│   ├── params.rs                  # Unified parameter types
│   ├── chromiumoxide_impl.rs      # Chromiumoxide wrapper
│   ├── spider_chrome_impl.rs      # Spider-chrome wrapper
│   ├── error.rs                   # Error types
│   ├── factory.rs                 # Engine factory/builder
│   └── tests/
│       ├── chromiumoxide_tests.rs
│       ├── spider_chrome_tests.rs
│       └── integration_tests.rs
└── examples/
    ├── basic_usage.rs
    └── hybrid_fallback.rs
```

## Core Traits

### BrowserEngine Trait

```rust
/// Common browser engine interface
#[async_trait]
pub trait BrowserEngine: Send + Sync + Debug {
    /// Create a new page/tab
    async fn new_page(&self) -> Result<Box<dyn PageHandle>>;

    /// Create a new page and navigate to URL
    async fn new_page_with_url(&self, url: &str) -> Result<Box<dyn PageHandle>>;

    /// Close the browser instance
    async fn close(&self) -> Result<()>;

    /// Get engine type identifier
    fn engine_type(&self) -> EngineType;

    /// Get browser version info
    async fn version(&self) -> Result<BrowserVersion>;

    /// Create new incognito context
    async fn new_context(&self) -> Result<Box<dyn BrowserEngine>>;

    /// Check if browser is still alive
    fn is_alive(&self) -> bool;

    /// Get underlying chromiumoxide browser (if applicable)
    fn as_chromiumoxide(&self) -> Option<&chromiumoxide::Browser> {
        None
    }

    /// Get underlying spider_chrome browser (if applicable)
    fn as_spider_chrome(&self) -> Option<&spider_chrome::Browser> {
        None
    }
}
```

### PageHandle Trait

```rust
/// Common page/tab interface
#[async_trait]
pub trait PageHandle: Send + Sync + Debug {
    /// Navigate to URL
    async fn goto(&self, url: &str) -> Result<()>;

    /// Get page HTML content
    async fn content(&self) -> Result<String>;

    /// Take screenshot
    async fn screenshot(&self, params: ScreenshotParams) -> Result<Vec<u8>>;

    /// Generate PDF
    async fn pdf(&self, params: PdfParams) -> Result<Vec<u8>>;

    /// Wait for navigation to complete
    async fn wait_for_navigation(&self) -> Result<()>;

    /// Evaluate JavaScript
    async fn evaluate(&self, script: &str) -> Result<serde_json::Value>;

    /// Get current page URL
    async fn url(&self) -> Result<String>;

    /// Get page title
    async fn title(&self) -> Result<String>;

    /// Close this page
    async fn close(&self) -> Result<()>;

    /// Get engine type
    fn engine_type(&self) -> EngineType;

    /// Get underlying chromiumoxide page (if applicable)
    fn as_chromiumoxide(&self) -> Option<&chromiumoxide::Page> {
        None
    }

    /// Get underlying spider_chrome page (if applicable)
    fn as_spider_chrome(&self) -> Option<&spider_chrome::Page> {
        None
    }
}
```

## Parameter Types

### ScreenshotParams

```rust
/// Unified screenshot parameters
#[derive(Debug, Clone)]
pub struct ScreenshotParams {
    /// Image format (PNG, JPEG, WEBP)
    pub format: ImageFormat,

    /// Quality (0-100) for JPEG/WEBP
    pub quality: Option<i64>,

    /// Capture full page (scroll viewport)
    pub full_page: bool,

    /// Capture specific region
    pub clip: Option<Viewport>,

    /// Omit background (transparent PNG)
    pub omit_background: bool,

    /// Capture beyond viewport
    pub capture_beyond_viewport: bool,
}

impl Default for ScreenshotParams {
    fn default() -> Self {
        Self {
            format: ImageFormat::Png,
            quality: None,
            full_page: false,
            clip: None,
            omit_background: false,
            capture_beyond_viewport: false,
        }
    }
}

/// Image format enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Webp,
}

/// Viewport/clip region
#[derive(Debug, Clone)]
pub struct Viewport {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub scale: f64,
}
```

### PdfParams

```rust
/// Unified PDF generation parameters
#[derive(Debug, Clone)]
pub struct PdfParams {
    /// Paper width (inches)
    pub paper_width: Option<f64>,

    /// Paper height (inches)
    pub paper_height: Option<f64>,

    /// Landscape orientation
    pub landscape: bool,

    /// Display header/footer
    pub display_header_footer: bool,

    /// Print background graphics
    pub print_background: bool,

    /// Page ranges (e.g., "1-5, 8, 11-13")
    pub page_ranges: Option<String>,

    /// Scale (0.1 to 2.0)
    pub scale: f64,

    /// Paper format preset
    pub format: Option<PaperFormat>,
}

impl Default for PdfParams {
    fn default() -> Self {
        Self {
            paper_width: None,
            paper_height: None,
            landscape: false,
            display_header_footer: false,
            print_background: false,
            page_ranges: None,
            scale: 1.0,
            format: Some(PaperFormat::A4),
        }
    }
}

/// Standard paper formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaperFormat {
    Letter,
    Legal,
    Tabloid,
    Ledger,
    A0, A1, A2, A3, A4, A5, A6,
}
```

### BrowserVersion

```rust
/// Browser version information
#[derive(Debug, Clone)]
pub struct BrowserVersion {
    pub browser: String,
    pub protocol_version: String,
    pub user_agent: String,
    pub v8_version: String,
}
```

## Implementation Wrappers

### ChromiumoxideEngine

```rust
/// Chromiumoxide engine wrapper
#[derive(Debug)]
pub struct ChromiumoxideEngine {
    inner: chromiumoxide::Browser,
    _marker: std::marker::PhantomData<()>,
}

impl ChromiumoxideEngine {
    pub fn new(browser: chromiumoxide::Browser) -> Self {
        Self {
            inner: browser,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn from_config(config: chromiumoxide::BrowserConfig) -> Result<Self> {
        // Launch browser
        let (browser, mut handler) = chromiumoxide::Browser::launch(config)
            .await
            .context("Failed to launch chromiumoxide browser")?;

        // Spawn handler task
        tokio::spawn(async move {
            while let Some(_) = handler.next().await {}
        });

        Ok(Self::new(browser))
    }
}

#[async_trait]
impl BrowserEngine for ChromiumoxideEngine {
    async fn new_page(&self) -> Result<Box<dyn PageHandle>> {
        let page = self.inner.new_page("about:blank")
            .await
            .context("Failed to create new page")?;
        Ok(Box::new(ChromiumoxidePage::new(page)))
    }

    async fn new_page_with_url(&self, url: &str) -> Result<Box<dyn PageHandle>> {
        let page = self.inner.new_page(url)
            .await
            .context("Failed to create new page")?;
        Ok(Box::new(ChromiumoxidePage::new(page)))
    }

    async fn close(&self) -> Result<()> {
        self.inner.close()
            .await
            .context("Failed to close browser")
    }

    fn engine_type(&self) -> EngineType {
        EngineType::Chromiumoxide
    }

    async fn version(&self) -> Result<BrowserVersion> {
        let version = self.inner.version()
            .await
            .context("Failed to get version")?;

        Ok(BrowserVersion {
            browser: version.browser,
            protocol_version: version.protocol_version,
            user_agent: version.user_agent,
            v8_version: version.v8_version,
        })
    }

    fn as_chromiumoxide(&self) -> Option<&chromiumoxide::Browser> {
        Some(&self.inner)
    }
}
```

### ChromiumoxidePage

```rust
/// Chromiumoxide page wrapper
#[derive(Debug)]
pub struct ChromiumoxidePage {
    inner: chromiumoxide::Page,
}

impl ChromiumoxidePage {
    pub fn new(page: chromiumoxide::Page) -> Self {
        Self { inner: page }
    }
}

#[async_trait]
impl PageHandle for ChromiumoxidePage {
    async fn goto(&self, url: &str) -> Result<()> {
        self.inner.goto(url)
            .await
            .context("Failed to navigate")?;
        Ok(())
    }

    async fn content(&self) -> Result<String> {
        let content = self.inner.content()
            .await
            .context("Failed to get content")?;
        Ok(content.unwrap_or_default())
    }

    async fn screenshot(&self, params: ScreenshotParams) -> Result<Vec<u8>> {
        // Translate params
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
        builder = builder.omit_background(params.omit_background);
        builder = builder.capture_beyond_viewport(params.capture_beyond_viewport);

        if let Some(clip) = params.clip {
            builder = builder.clip(chromiumoxide::cdp::browser_protocol::page::Viewport {
                x: clip.x,
                y: clip.y,
                width: clip.width,
                height: clip.height,
                scale: clip.scale,
            });
        }

        let screenshot_params = builder.build();
        let data = self.inner.screenshot(screenshot_params)
            .await
            .context("Screenshot failed")?;

        Ok(data)
    }

    async fn pdf(&self, params: PdfParams) -> Result<Vec<u8>> {
        // Translate params to chromiumoxide format
        let mut pdf_params = chromiumoxide::page::PdfParams::default();

        if let Some(width) = params.paper_width {
            pdf_params = pdf_params.paper_width(width);
        }
        if let Some(height) = params.paper_height {
            pdf_params = pdf_params.paper_height(height);
        }

        pdf_params = pdf_params.landscape(params.landscape);
        pdf_params = pdf_params.print_background(params.print_background);
        pdf_params = pdf_params.scale(params.scale);

        let data = self.inner.pdf(pdf_params)
            .await
            .context("PDF generation failed")?;

        Ok(data)
    }

    async fn wait_for_navigation(&self) -> Result<()> {
        self.inner.wait_for_navigation()
            .await
            .context("Navigation timeout")?;
        Ok(())
    }

    async fn evaluate(&self, script: &str) -> Result<serde_json::Value> {
        let result = self.inner.evaluate(script)
            .await
            .context("Script evaluation failed")?;

        Ok(result.value().clone())
    }

    async fn url(&self) -> Result<String> {
        let url = self.inner.url()
            .await
            .context("Failed to get URL")?;
        Ok(url.unwrap_or_default())
    }

    async fn title(&self) -> Result<String> {
        let title = self.inner.get_title()
            .await
            .context("Failed to get title")?;
        Ok(title.unwrap_or_default())
    }

    async fn close(&self) -> Result<()> {
        self.inner.close()
            .await
            .context("Failed to close page")
    }

    fn engine_type(&self) -> EngineType {
        EngineType::Chromiumoxide
    }

    fn as_chromiumoxide(&self) -> Option<&chromiumoxide::Page> {
        Some(&self.inner)
    }
}
```

### SpiderChromeEngine

```rust
/// Spider-chrome engine wrapper
#[derive(Debug)]
pub struct SpiderChromeEngine {
    inner: spider_chrome::Browser,
}

impl SpiderChromeEngine {
    pub fn new(browser: spider_chrome::Browser) -> Self {
        Self { inner: browser }
    }

    pub async fn from_config(config: spider_chrome::BrowserConfig) -> Result<Self> {
        let (browser, mut handler) = spider_chrome::Browser::launch(config)
            .await
            .context("Failed to launch spider_chrome browser")?;

        tokio::spawn(async move {
            while let Some(_) = handler.next().await {}
        });

        Ok(Self::new(browser))
    }
}

#[async_trait]
impl BrowserEngine for SpiderChromeEngine {
    async fn new_page(&self) -> Result<Box<dyn PageHandle>> {
        let page = self.inner.new_page("about:blank")
            .await
            .context("Failed to create new page")?;
        Ok(Box::new(SpiderChromePage::new(page)))
    }

    async fn new_page_with_url(&self, url: &str) -> Result<Box<dyn PageHandle>> {
        let page = self.inner.new_page(url)
            .await
            .context("Failed to create new page")?;
        Ok(Box::new(SpiderChromePage::new(page)))
    }

    async fn close(&self) -> Result<()> {
        self.inner.close()
            .await
            .context("Failed to close browser")
    }

    fn engine_type(&self) -> EngineType {
        EngineType::SpiderChrome
    }

    async fn version(&self) -> Result<BrowserVersion> {
        let version = self.inner.version()
            .await
            .context("Failed to get version")?;

        Ok(BrowserVersion {
            browser: version.browser,
            protocol_version: version.protocol_version,
            user_agent: version.user_agent,
            v8_version: version.v8_version,
        })
    }

    fn as_spider_chrome(&self) -> Option<&spider_chrome::Browser> {
        Some(&self.inner)
    }
}
```

### SpiderChromePage

```rust
/// Spider-chrome page wrapper (similar to ChromiumoxidePage)
#[derive(Debug)]
pub struct SpiderChromePage {
    inner: spider_chrome::Page,
}

// Implementation similar to ChromiumoxidePage
// with spider_chrome types instead
```

## Factory Pattern

```rust
/// Engine factory for creating browsers
pub struct EngineFactory;

impl EngineFactory {
    /// Create chromiumoxide engine with default config
    pub async fn chromiumoxide_default() -> Result<Box<dyn BrowserEngine>> {
        let config = chromiumoxide::BrowserConfig::builder()
            .build()
            .context("Failed to build config")?;

        let engine = ChromiumoxideEngine::from_config(config).await?;
        Ok(Box::new(engine))
    }

    /// Create spider_chrome engine with default config
    pub async fn spider_chrome_default() -> Result<Box<dyn BrowserEngine>> {
        let config = spider_chrome::BrowserConfig::builder()
            .build()
            .context("Failed to build config")?;

        let engine = SpiderChromeEngine::from_config(config).await?;
        Ok(Box::new(engine))
    }

    /// Create engine based on type
    pub async fn create(engine_type: EngineType) -> Result<Box<dyn BrowserEngine>> {
        match engine_type {
            EngineType::Chromiumoxide => Self::chromiumoxide_default().await,
            EngineType::SpiderChrome => Self::spider_chrome_default().await,
        }
    }
}
```

## Test Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_chromiumoxide_basic() {
        let engine = EngineFactory::chromiumoxide_default().await.unwrap();
        let page = engine.new_page_with_url("https://example.com").await.unwrap();

        page.wait_for_navigation().await.unwrap();
        let content = page.content().await.unwrap();

        assert!(content.contains("<title>"));
        assert_eq!(page.engine_type(), EngineType::Chromiumoxide);
    }

    #[tokio::test]
    async fn test_spider_chrome_basic() {
        let engine = EngineFactory::spider_chrome_default().await.unwrap();
        let page = engine.new_page_with_url("https://example.com").await.unwrap();

        page.wait_for_navigation().await.unwrap();
        let content = page.content().await.unwrap();

        assert!(content.contains("<title>"));
        assert_eq!(page.engine_type(), EngineType::SpiderChrome);
    }

    #[tokio::test]
    async fn test_screenshot_params_translation() {
        let params = ScreenshotParams {
            format: ImageFormat::Jpeg,
            quality: Some(80),
            full_page: true,
            ..Default::default()
        };

        // Both engines should accept same params
        let engine1 = EngineFactory::chromiumoxide_default().await.unwrap();
        let page1 = engine1.new_page().await.unwrap();
        let _screenshot1 = page1.screenshot(params.clone()).await.unwrap();

        let engine2 = EngineFactory::spider_chrome_default().await.unwrap();
        let page2 = engine2.new_page().await.unwrap();
        let _screenshot2 = page2.screenshot(params).await.unwrap();
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_hybrid_switching() {
        // Create both engines
        let chromium = EngineFactory::chromiumoxide_default().await.unwrap();
        let spider = EngineFactory::spider_chrome_default().await.unwrap();

        // Both should handle same URL
        let url = "https://httpbin.org/html";

        let page1 = chromium.new_page_with_url(url).await.unwrap();
        page1.wait_for_navigation().await.unwrap();
        let content1 = page1.content().await.unwrap();

        let page2 = spider.new_page_with_url(url).await.unwrap();
        page2.wait_for_navigation().await.unwrap();
        let content2 = page2.content().await.unwrap();

        // Both should get valid HTML
        assert!(content1.contains("<html"));
        assert!(content2.contains("<html"));
    }
}
```

## Usage Examples

### Basic Usage

```rust
use riptide_browser_abstraction::{EngineFactory, EngineType, ScreenshotParams};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create engine
    let engine = EngineFactory::create(EngineType::SpiderChrome).await?;

    // Create page
    let page = engine.new_page_with_url("https://example.com").await?;
    page.wait_for_navigation().await?;

    // Get content
    let html = page.content().await?;
    println!("HTML length: {}", html.len());

    // Screenshot
    let screenshot = page.screenshot(ScreenshotParams::default()).await?;
    std::fs::write("screenshot.png", screenshot)?;

    Ok(())
}
```

### Hybrid Fallback Integration

```rust
// Update hybrid_fallback.rs
use riptide_browser_abstraction::{BrowserEngine, PageHandle, EngineFactory, EngineType};

pub struct HybridBrowserFallback {
    chromium_engine: Box<dyn BrowserEngine>,
    spider_engine: Option<Box<dyn BrowserEngine>>,
    traffic_pct: u8,
}

impl HybridBrowserFallback {
    pub async fn new() -> Result<Self> {
        let chromium_engine = EngineFactory::chromiumoxide_default().await?;

        let spider_engine = match EngineFactory::spider_chrome_default().await {
            Ok(engine) => Some(engine),
            Err(e) => {
                warn!("Spider-chrome unavailable: {}", e);
                None
            }
        };

        Ok(Self {
            chromium_engine,
            spider_engine,
            traffic_pct: 20,
        })
    }

    pub async fn execute(&self, url: &str) -> Result<Box<dyn PageHandle>> {
        let use_spider = self.should_use_spider(url);

        if use_spider && self.spider_engine.is_some() {
            match self.spider_engine.as_ref().unwrap().new_page_with_url(url).await {
                Ok(page) => return Ok(page),
                Err(e) => warn!("Spider failed, falling back: {}", e),
            }
        }

        // Fallback to chromium
        self.chromium_engine.new_page_with_url(url).await
    }
}
```

## Performance Considerations

### Virtual Call Overhead
- **Cost:** 1-3ns per trait method call
- **Impact:** <0.001% of typical page load (100-500ms)
- **Verdict:** Negligible

### Heap Allocation
- **Box<dyn Trait>:** One allocation per Browser/Page
- **Cost:** ~16 bytes overhead
- **Verdict:** Negligible

### Parameter Translation
- **Cost:** Struct copies and enum mappings
- **Impact:** <10μs per screenshot/PDF call
- **Verdict:** Negligible

**Total Performance Impact:** <0.01% of end-to-end latency

## Migration Path

### Phase 1: Add Abstraction (Non-Breaking)
1. Add `riptide-browser-abstraction` crate
2. Implement wrappers
3. Keep existing code unchanged

### Phase 2: Gradual Migration
1. Update `hybrid_fallback.rs` first
2. Update launchers
3. Update pools last

### Phase 3: Complete Migration (Optional)
1. Replace all `chromiumoxide::` imports with `riptide_browser_abstraction::`
2. Use trait objects everywhere
3. Remove direct dependencies (keep as transitive)

## Next Steps

1. **Day 2 Morning:** Implement core traits and params
2. **Day 2 Afternoon:** Implement both engine wrappers
3. **Day 3 Morning:** Integrate with hybrid_fallback.rs
4. **Day 3 Afternoon:** Tests and validation
5. **Day 4:** Performance testing and documentation

## References

- ADR-006: `/workspaces/eventmesh/docs/architecture/ADR-006-spider-chrome-compatibility.md`
- API Analysis: `/workspaces/eventmesh/docs/integration/SPIDER-CHROME-API-ANALYSIS.md`
- Hybrid Fallback: `/workspaces/eventmesh/crates/riptide-headless/src/hybrid_fallback.rs`
