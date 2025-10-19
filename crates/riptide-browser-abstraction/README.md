# RipTide Browser Abstraction

Browser abstraction layer for the RipTide web scraping framework, providing a unified interface for browser automation and rendering.

## Overview

`riptide-browser-abstraction` provides a clean, async-first abstraction layer over browser automation engines. It enables consistent browser interaction patterns across different rendering backends while maintaining high performance and reliability.

## Features

- **Unified Browser Interface**: Single API for different browser automation backends
- **Chrome DevTools Protocol**: Built on spider_chrome for high-performance CDP operations
- **Async-First Design**: Full async/await support with Tokio runtime integration
- **Type Safety**: Strong typing with riptide-types integration
- **Error Handling**: Comprehensive error types with thiserror
- **Extensible Architecture**: Trait-based design for custom browser implementations

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│        Browser Abstraction Layer (This Crate)          │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │   Browser    │  │    Page      │  │   Session    │  │
│  │    Trait     │  │   Trait      │  │   Manager    │  │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  │
│         │                 │                  │          │
│         └─────────────────┼──────────────────┘          │
│                           ▼                             │
│                  ┌─────────────────┐                    │
│                  │  spider_chrome  │                    │
│                  │  (CDP Backend)  │                    │
│                  └─────────────────┘                    │
└─────────────────────────────────────────────────────────┘
```

## Usage

### Basic Browser Operations

```rust
use riptide_browser_abstraction::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Create browser instance
    let browser = BrowserBuilder::new()
        .headless(true)
        .build()
        .await?;

    // Navigate to URL
    let page = browser.new_page().await?;
    page.goto("https://example.com").await?;

    // Extract content
    let html = page.content().await?;
    let title = page.title().await?;

    println!("Title: {}", title);

    Ok(())
}
```

### Advanced Features

```rust
use riptide_browser_abstraction::*;

// Custom browser configuration
let browser = BrowserBuilder::new()
    .headless(true)
    .user_agent("Custom UA")
    .viewport(1920, 1080)
    .timeout(Duration::from_secs(30))
    .build()
    .await?;

// JavaScript execution
let result = page.evaluate("document.title").await?;

// Screenshot capture
let screenshot = page.screenshot(ScreenshotOptions::default()).await?;

// Network interception
page.on_request(|req| {
    println!("Request: {}", req.url());
});
```

## Core Traits

### Browser

Main browser control interface:

```rust
#[async_trait]
pub trait Browser: Send + Sync {
    async fn new_page(&self) -> Result<Box<dyn Page>>;
    async fn close(&self) -> Result<()>;
    fn is_closed(&self) -> bool;
}
```

### Page

Individual page/tab operations:

```rust
#[async_trait]
pub trait Page: Send + Sync {
    async fn goto(&self, url: &str) -> Result<()>;
    async fn content(&self) -> Result<String>;
    async fn title(&self) -> Result<String>;
    async fn evaluate(&self, script: &str) -> Result<serde_json::Value>;
    async fn screenshot(&self, options: ScreenshotOptions) -> Result<Vec<u8>>;
}
```

## Integration with RipTide

This crate is used by:

- **riptide-engine**: Core browser pool and rendering
- **riptide-headless**: Headless browser service
- **riptide-headless-hybrid**: Hybrid launcher with stealth capabilities

## Dependencies

- **spider_chrome**: High-performance Chrome DevTools Protocol implementation
- **async-trait**: Async trait support
- **tokio**: Async runtime
- **riptide-types**: Shared type definitions

## Testing

```bash
# Run unit tests
cargo test -p riptide-browser-abstraction

# Run with output
cargo test -p riptide-browser-abstraction -- --nocapture

# Test specific module
cargo test -p riptide-browser-abstraction browser_trait
```

## License

Apache-2.0

## Related Crates

- **riptide-engine**: Browser engine implementation
- **riptide-stealth**: Anti-detection capabilities
- **riptide-headless**: Headless browser service
