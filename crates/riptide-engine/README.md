# RipTide Engine

Core browser automation engine for RipTide, providing headless Chrome integration with stealth capabilities and resource management.

## Overview

`riptide-engine` is the browser automation powerhouse of RipTide, implementing high-performance headless Chrome operations with CDP (Chrome DevTools Protocol), stealth mode, and intelligent resource pooling for scalable web rendering.

## Features

- **Chrome DevTools Protocol**: Native CDP integration via spider_chrome
- **Browser Pool Management**: Efficient instance pooling with health monitoring
- **Stealth Mode**: Anti-detection capabilities from riptide-stealth
- **Resource Optimization**: Memory-aware browser lifecycle management
- **Screenshot Capture**: Full-page and element screenshots
- **Network Interception**: Request/response monitoring and modification
- **JavaScript Execution**: Execute custom scripts in browser context
- **Cookie Management**: Session persistence and cookie handling
- **PDF Generation**: Browser-based PDF rendering

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│                  RipTide Engine                          │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │   Browser   │  │   Stealth   │  │   Resource  │     │
│  │    Pool     │  │   Manager   │  │   Monitor   │     │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘     │
│         │                │                 │            │
│         └────────────────┼─────────────────┘            │
│                          ▼                              │
│                  ┌───────────────┐                      │
│                  │ spider_chrome │                      │
│                  │   (CDP API)   │                      │
│                  └───────┬───────┘                      │
└────────────────────────────┼─────────────────────────────┘
                            │
                            ▼
                    ┌──────────────┐
                    │   Chromium   │
                    │   (Headless) │
                    └──────────────┘
```

## Usage

### Basic Browser Operations

```rust
use riptide_engine::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize browser engine
    let engine = BrowserEngine::new().await?;

    // Get browser from pool
    let browser = engine.get_browser().await?;

    // Navigate and render
    let page = browser.new_page().await?;
    page.goto("https://example.com").await?;

    // Extract content
    let html = page.content().await?;
    let title = page.title().await?;

    println!("Title: {}", title);

    // Return browser to pool
    engine.return_browser(browser).await;

    Ok(())
}
```

### Browser Pool Configuration

```rust
use riptide_engine::*;

let config = BrowserPoolConfig {
    max_pool_size: 5,
    min_pool_size: 1,
    idle_timeout_secs: 300,
    launch_args: vec![
        "--no-sandbox",
        "--disable-setuid-sandbox",
        "--disable-dev-shm-usage",
    ],
};

let engine = BrowserEngine::with_config(config).await?;
```

### Stealth Mode

```rust
use riptide_engine::*;

// Enable stealth features
let page = browser.new_page_with_stealth().await?;

// Stealth automatically applies:
// - User-agent randomization
// - WebDriver detection evasion
// - Canvas fingerprint randomization
// - WebGL vendor/renderer spoofing
// - Plugin and media device fingerprinting

page.goto("https://bot-detection-site.com").await?;
```

### Screenshot Capture

```rust
use riptide_engine::*;

// Full page screenshot
let screenshot = page.screenshot_full_page().await?;
std::fs::write("page.png", &screenshot)?;

// Element screenshot
let element_screenshot = page
    .screenshot_element("article.content")
    .await?;

// Custom viewport screenshot
let options = ScreenshotOptions {
    viewport: Some(Viewport { width: 1920, height: 1080 }),
    full_page: false,
    omit_background: true,
};
let screenshot = page.screenshot_with_options(options).await?;
```

### JavaScript Execution

```rust
use riptide_engine::*;

// Execute script
let result = page.evaluate("document.title").await?;
println!("Title: {}", result.as_str().unwrap());

// Execute with arguments
let result = page.evaluate_with_args(
    "selector => document.querySelector(selector).textContent",
    vec!["h1".into()]
).await?;

// Add script to execute on every page load
page.add_script_to_evaluate_on_new_document(
    "Object.defineProperty(navigator, 'webdriver', { get: () => false })"
).await?;
```

### Network Interception

```rust
use riptide_engine::*;

// Intercept requests
page.enable_request_interception().await?;

page.on_request(|request| async move {
    println!("Request: {} {}", request.method(), request.url());

    // Block specific resources
    if request.url().ends_with(".mp4") {
        request.abort().await?;
    } else {
        request.continue_request().await?;
    }

    Ok(())
}).await;

// Intercept responses
page.on_response(|response| async move {
    println!("Response: {} - {}", response.status(), response.url());
    Ok(())
}).await;
```

### Cookie Management

```rust
use riptide_engine::*;

// Get cookies
let cookies = page.get_cookies().await?;

// Set cookies
page.set_cookies(vec![
    Cookie {
        name: "session".to_string(),
        value: "abc123".to_string(),
        domain: Some("example.com".to_string()),
        ..Default::default()
    }
]).await?;

// Delete cookies
page.delete_cookies(vec!["session"]).await?;
```

### PDF Generation

```rust
use riptide_engine::*;

// Generate PDF
let pdf_options = PdfOptions {
    landscape: false,
    display_header_footer: true,
    print_background: true,
    scale: 1.0,
    paper_width: 8.5,
    paper_height: 11.0,
    ..Default::default()
};

let pdf_bytes = page.pdf(pdf_options).await?;
std::fs::write("page.pdf", &pdf_bytes)?;
```

## Resource Management

### Browser Pool Health

```rust
use riptide_engine::*;

let engine = BrowserEngine::new().await?;

// Get pool statistics
let stats = engine.pool_stats().await;
println!("Active browsers: {}", stats.active);
println!("Idle browsers: {}", stats.idle);
println!("Total browsers: {}", stats.total);

// Health check
if engine.is_healthy().await {
    println!("Browser pool is healthy");
}

// Cleanup idle browsers
engine.cleanup_idle().await;
```

### Memory Monitoring

```rust
use riptide_engine::*;

// Monitor memory usage
let memory = engine.memory_stats().await?;
println!("Browser memory: {} MB", memory.browser_mb);
println!("System memory: {} MB", memory.system_mb);

// Force garbage collection
page.force_gc().await?;

// Get detailed metrics
let metrics = page.metrics().await?;
for (key, value) in metrics {
    println!("{}: {}", key, value);
}
```

## Configuration

### Environment Variables

```bash
# Browser pool
export HEADLESS_URL="http://localhost:9123"
export HEADLESS_POOL_SIZE=5
export HEADLESS_MIN_POOL_SIZE=1
export HEADLESS_IDLE_TIMEOUT=300

# Launch options
export CHROME_BIN="/usr/bin/chromium-browser"
export CHROME_ARGS="--no-sandbox,--disable-setuid-sandbox"

# Timeouts
export BROWSER_LAUNCH_TIMEOUT=30
export PAGE_LOAD_TIMEOUT=30
export NAVIGATION_TIMEOUT=30

# Stealth
export ENABLE_STEALTH=true
export RANDOMIZE_USER_AGENT=true
```

### Programmatic Configuration

```rust
use riptide_engine::*;

let config = EngineConfig {
    pool: BrowserPoolConfig {
        max_pool_size: 5,
        min_pool_size: 1,
        idle_timeout_secs: 300,
    },
    launch: LaunchConfig {
        chrome_bin: Some("/usr/bin/chromium".to_string()),
        args: vec![
            "--no-sandbox".to_string(),
            "--disable-dev-shm-usage".to_string(),
        ],
        headless: true,
    },
    stealth: StealthConfig {
        enabled: true,
        randomize_user_agent: true,
        evasions: vec!["webdriver", "chrome", "plugins"],
    },
    timeouts: TimeoutConfig {
        launch: Duration::from_secs(30),
        page_load: Duration::from_secs(30),
        navigation: Duration::from_secs(30),
    },
};

let engine = BrowserEngine::with_config(config).await?;
```

## Integration with RipTide

This crate is used by:

- **riptide-headless**: Headless browser service
- **riptide-headless-hybrid**: Hybrid launcher with stealth
- **riptide-api**: Browser action execution
- **riptide-facade**: Browser facade integration

## Performance Characteristics

- **Pool Initialization**: < 1s for first browser
- **Page Navigation**: 100-500ms for static pages
- **Screenshot**: 50-200ms depending on page size
- **Memory**: ~50-100MB per browser instance
- **Concurrent Browsers**: Up to 10 recommended per core

## Testing

```bash
# Run tests
cargo test -p riptide-engine

# Run with headless Chrome (requires Chrome installed)
cargo test -p riptide-engine --features headless

# Integration tests
cargo test -p riptide-engine --test '*'

# With serial execution (for resource tests)
cargo test -p riptide-engine -- --test-threads=1
```

## License

Apache-2.0

## Related Crates

- **riptide-browser-abstraction**: Browser trait definitions
- **riptide-stealth**: Anti-detection capabilities
- **riptide-headless**: Headless service implementation
- **spider_chrome**: CDP protocol implementation
