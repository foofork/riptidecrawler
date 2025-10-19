# RipTide Headless

A high-performance headless browser service powered by Chromium for JavaScript-heavy websites. Built on top of `chromiumoxide` with advanced pooling, stealth capabilities, and production-ready resource management.

## Overview

`riptide-headless` provides both a library and standalone service for browser automation, featuring:

- **Browser Pooling**: Connection pool management with automatic health checks and recovery
- **Stealth Mode**: Anti-detection techniques for bot-resistant websites
- **Resource Management**: Memory monitoring, lifecycle management, and automatic cleanup
- **Production Ready**: Comprehensive error handling, monitoring, and graceful shutdown
- **Flexible Deployment**: Use as a library or standalone HTTP service

## Browser Automation Capabilities

### Core Features

- **JavaScript Execution**: Full JavaScript runtime support via Chromium
- **Page Navigation**: Navigate URLs with timeout controls and redirect handling
- **Element Interaction**: Wait for elements, click, type, and execute custom scripts
- **Screenshot Capture**: Take full-page or viewport screenshots
- **PDF Generation**: Convert web pages to PDF documents
- **Cookie Management**: Set, get, and manage cookies across sessions
- **Network Interception**: Monitor and modify network requests/responses
- **Stealth Integration**: Built-in anti-detection with configurable presets
- **Viewport Control**: Configure device metrics and screen resolutions
- **Session Management**: Persistent browser sessions with automatic cleanup

### Stealth Capabilities

Built-in stealth mode with multiple presets:

- **None**: Standard browser behavior (for debugging)
- **Low**: Basic anti-detection (navigator overrides)
- **Medium**: Balanced stealth (recommended default)
- **High**: Maximum stealth (full fingerprint randomization)

Stealth features include:
- User-agent rotation
- Navigator property overrides
- WebGL fingerprint randomization
- Canvas fingerprint protection
- Plugin and language spoofing
- Timezone and geolocation masking

## Library Usage

### Basic Example

```rust
use riptide_headless::launcher::HeadlessLauncher;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Create launcher with default configuration
    let launcher = HeadlessLauncher::new().await?;

    // Launch a page with default stealth
    let session = launcher.launch_page_default("https://example.com").await?;

    // Access the page
    let content = session.page().content().await?;
    println!("Page content length: {}", content.len());

    // Take a screenshot
    let screenshot = session.screenshot().await?;
    std::fs::write("screenshot.png", screenshot)?;

    // Session automatically returns browser to pool when dropped
    Ok(())
}
```

### Advanced Configuration

```rust
use riptide_headless::{
    launcher::{HeadlessLauncher, LauncherConfig},
    pool::BrowserPoolConfig,
};
use riptide_stealth::StealthPreset;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let config = LauncherConfig {
        pool_config: BrowserPoolConfig {
            min_pool_size: 2,
            max_pool_size: 10,
            initial_pool_size: 5,
            idle_timeout: Duration::from_secs(60),
            max_lifetime: Duration::from_secs(600),
            memory_threshold_mb: 1000,
            enable_recovery: true,
            ..Default::default()
        },
        default_stealth_preset: StealthPreset::High,
        enable_stealth: true,
        page_timeout: Duration::from_secs(30),
        enable_monitoring: true,
    };

    let launcher = HeadlessLauncher::with_config(config).await?;

    // Use launcher...

    launcher.shutdown().await?;
    Ok(())
}
```

### Session Management

```rust
// Launch with custom stealth preset
let session = launcher.launch_page(
    "https://example.com",
    Some(StealthPreset::Medium)
).await?;

// Navigate to new URL
session.navigate("https://example.com/page2").await?;

// Wait for element to appear
session.wait_for_element(".content", Some(5000)).await?;

// Execute JavaScript
let result = session.execute_script(
    "document.querySelector('.content').textContent"
).await?;

// Get page content
let html = session.content().await?;

// Take screenshot
let screenshot = session.screenshot().await?;

// Session info
println!("Session ID: {}", session.session_id());
println!("Duration: {:?}", session.duration());
```

### Pool Management

```rust
use riptide_headless::pool::BrowserPool;

// Create pool directly for low-level control
let pool = BrowserPool::new(
    BrowserPoolConfig::default(),
    browser_config
).await?;

// Checkout browser
let checkout = pool.checkout().await?;

// Create page
let page = checkout.new_page("https://example.com").await?;

// Use page...

// Return to pool (automatic on drop)
checkout.checkin().await?;

// Get pool statistics
let stats = pool.stats().await;
println!("Available: {}", stats.available);
println!("In use: {}", stats.in_use);
println!("Utilization: {:.1}%", stats.utilization);
```

## Standalone Service (Binary)

Run as an HTTP service for remote browser automation:

### Starting the Service

```bash
# Run with default settings
cargo run --bin riptide-headless

# With custom logging
RUST_LOG=debug cargo run --bin riptide-headless
```

The service listens on `0.0.0.0:9123` by default.

### API Endpoints

#### Health Check

```bash
curl http://localhost:9123/health
```

#### Render Page

```bash
curl -X POST http://localhost:9123/render \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://example.com",
    "wait_for": ".content",
    "scroll_steps": 3,
    "artifacts": {
      "screenshot": true,
      "mhtml": true
    },
    "stealth_config": {
      "preset": "medium"
    }
  }'
```

### Request Fields

- `url` (required): Target URL to render
- `wait_for` (optional): CSS selector to wait for before capturing
- `scroll_steps` (optional): Number of scroll steps to perform
- `session_id` (optional): Session ID for persistent browser sessions
- `actions` (optional): Array of interactive page actions
- `timeouts` (optional): Custom timeout configurations
- `artifacts` (optional): Capture options (screenshot, mhtml)
- `stealth_config` (optional): Stealth mode configuration

### Response Format

```json
{
  "final_url": "https://example.com",
  "html": "<html>...</html>",
  "session_id": "uuid",
  "artifacts": {
    "screenshot_b64": "base64_image_data",
    "mhtml_b64": "base64_mhtml_data"
  }
}
```

### Page Actions

Execute interactive actions before capturing:

```json
{
  "url": "https://example.com",
  "actions": [
    {
      "type": "wait_for_css",
      "css": ".content",
      "timeout_ms": 5000
    },
    {
      "type": "click",
      "css": ".button"
    },
    {
      "type": "type",
      "css": "#search",
      "text": "query",
      "delay_ms": 100
    },
    {
      "type": "scroll",
      "steps": 5,
      "step_px": 500,
      "delay_ms": 200
    },
    {
      "type": "js",
      "code": "document.querySelector('.modal').remove();"
    }
  ]
}
```

## Configuration

### Environment Variables

```bash
# Logging level
export RUST_LOG=info

# Custom port (modify in main.rs)
# Default: 9123
```

### Browser Pool Configuration

- `min_pool_size`: Minimum browsers to keep warm (default: 1)
- `max_pool_size`: Maximum concurrent browsers (default: 5)
- `initial_pool_size`: Browsers created at startup (default: 3)
- `idle_timeout`: Time before idle browser cleanup (default: 30s)
- `max_lifetime`: Maximum browser lifetime (default: 5 minutes)
- `health_check_interval`: Frequency of health checks (default: 10s)
- `memory_threshold_mb`: Memory limit per browser (default: 500 MB)
- `enable_recovery`: Auto-recovery for crashed browsers (default: true)
- `max_retries`: Maximum retries for operations (default: 3)

### Launcher Configuration

- `default_stealth_preset`: Default stealth level (None, Low, Medium, High)
- `enable_stealth`: Enable stealth mode globally (default: true)
- `page_timeout`: Timeout for page operations (default: 30s)
- `enable_monitoring`: Enable performance monitoring (default: true)

## Stealth Integration

### Using Stealth Presets

```rust
use riptide_stealth::StealthPreset;

// No stealth (for debugging)
let session = launcher.launch_page(
    "https://example.com",
    Some(StealthPreset::None)
).await?;

// Medium stealth (recommended)
let session = launcher.launch_page(
    "https://example.com",
    Some(StealthPreset::Medium)
).await?;

// High stealth (maximum protection)
let session = launcher.launch_page(
    "https://example.com",
    Some(StealthPreset::High)
).await?;
```

### Custom Stealth Configuration

```rust
use riptide_stealth::StealthController;

let mut stealth = StealthController::from_preset(StealthPreset::Medium);

// Get CDP flags for browser configuration
let flags = stealth.get_cdp_flags();

// Get randomized user agent
let user_agent = stealth.next_user_agent();
```

### Stealth JavaScript

The crate includes `stealth.js` which provides:
- WebDriver detection bypass
- Navigator property overrides
- Plugin simulation
- Canvas fingerprint randomization
- WebGL fingerprint protection
- Language and timezone spoofing

## Performance Considerations

### Memory Management

- Each browser instance consumes ~200-500 MB of memory
- Configure `memory_threshold_mb` based on available system memory
- Idle browsers are automatically cleaned up after `idle_timeout`
- Maximum lifetime (`max_lifetime`) prevents memory leaks from long-running browsers

### Pool Sizing

**Recommended configurations:**

- **Low traffic**: `min_pool_size: 1`, `max_pool_size: 3`
- **Medium traffic**: `min_pool_size: 2`, `max_pool_size: 5`
- **High traffic**: `min_pool_size: 5`, `max_pool_size: 10`

### Performance Tips

1. **Reuse sessions** for multiple pages in the same domain
2. **Disable JavaScript** if not needed (`--disable-javascript` flag)
3. **Disable images** to reduce bandwidth (`--disable-images` flag)
4. **Enable monitoring** to track pool utilization
5. **Tune timeouts** based on target website response times
6. **Use connection pooling** to avoid browser startup overhead

### Monitoring

```rust
// Get launcher statistics
let stats = launcher.stats().await;
println!("Total requests: {}", stats.total_requests);
println!("Success rate: {:.1}%",
    stats.successful_requests as f64 / stats.total_requests as f64 * 100.0
);
println!("Avg response time: {:.2}ms", stats.avg_response_time_ms);
println!("Pool utilization: {:.1}%", stats.pool_utilization);

// Subscribe to pool events
let events = launcher.pool_events();
let mut receiver = events.lock().await;
while let Some(event) = receiver.recv().await {
    match event {
        PoolEvent::BrowserCreated { id } => {
            println!("Browser created: {}", id);
        }
        PoolEvent::MemoryAlert { browser_id, memory_mb } => {
            println!("Memory alert: {} using {} MB", browser_id, memory_mb);
        }
        _ => {}
    }
}
```

## Testing

Run the test suite:

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_browser_pool_creation
```

### Test Coverage

- Browser pool creation and configuration
- Checkout/checkin lifecycle
- Multiple concurrent checkouts
- Health checks and recovery
- Launcher initialization
- Stealth preset configurations
- Statistics tracking
- Graceful shutdown

### Integration Tests

Located in `tests/headless_tests.rs`:

```bash
# Run integration tests
cargo test --test headless_tests
```

## Feature Flags

### `headless` (WIP)

Full headless browser automation scaffolding. Currently under development.

```toml
[dependencies]
riptide-headless = { version = "0.1", features = ["headless"] }
```

Note: This feature is work-in-progress and not yet fully wired into the system.

## Troubleshooting

### Common Issues

#### Browser Launch Failures

**Problem**: `Failed to launch browser: Cannot find chrome executable`

**Solution**: Install Chromium or Chrome:

```bash
# Ubuntu/Debian
apt-get install chromium-browser

# macOS
brew install --cask chromium

# Or set CHROME_PATH environment variable
export CHROME_PATH=/path/to/chrome
```

#### Connection Timeout

**Problem**: `Browser checkout timed out`

**Solutions**:
- Increase `max_pool_size` to handle more concurrent requests
- Increase checkout timeout in code
- Check system resources (CPU, memory)
- Verify Chromium is installed and accessible

#### Memory Alerts

**Problem**: `Browser memory threshold exceeded`

**Solutions**:
- Increase `memory_threshold_mb` if legitimate high usage
- Reduce `max_lifetime` to recycle browsers more frequently
- Implement page cleanup (close unused tabs)
- Check for memory leaks in target websites

#### Health Check Failures

**Problem**: `Browser health check failed: Timeout`

**Solutions**:
- Check if Chromium process is hung (kill manually)
- Verify system has sufficient resources
- Reduce `max_pool_size` to lower system load
- Enable `enable_recovery` for automatic recovery

### Debugging

Enable debug logging:

```bash
RUST_LOG=riptide_headless=debug cargo run
```

Disable stealth for debugging:

```rust
let config = LauncherConfig {
    enable_stealth: false,
    ..Default::default()
};
```

Use non-headless mode (requires display):

```rust
// Modify build_browser_config to add:
builder = builder.with_head();
```

### Performance Debugging

Check pool statistics regularly:

```rust
let stats = launcher.stats().await;
println!("Pool stats: {:?}", stats);

let pool_stats = pool.stats().await;
println!("Available: {}, In use: {}, Utilization: {:.1}%",
    pool_stats.available,
    pool_stats.in_use,
    pool_stats.utilization
);
```

Monitor browser events:

```rust
let events = launcher.pool_events();
tokio::spawn(async move {
    let mut receiver = events.lock().await;
    while let Some(event) = receiver.recv().await {
        eprintln!("Pool event: {:?}", event);
    }
});
```

## Architecture

The crate is organized into several modules:

- **`launcher`**: High-level API for browser session management
  - `HeadlessLauncher`: Main entry point with pooling and stealth
  - `LaunchSession`: Managed browser session with automatic cleanup
  - `LauncherConfig`: Configuration for launcher behavior

- **`pool`**: Low-level browser pool management
  - `BrowserPool`: Connection pool with health checks
  - `PooledBrowser`: Individual browser instance wrapper
  - `BrowserCheckout`: RAII guard for checked-out browsers

- **`cdp`**: Chrome DevTools Protocol integration
  - HTTP API handlers
  - Page interaction and rendering
  - Artifact capture (screenshots, MHTML)

- **`models`**: Request/response data structures
  - `RenderReq`: Enhanced render request with actions
  - `RenderResp`: Render response with artifacts
  - `PageAction`: Interactive page action types

- **`stealth.js`**: Anti-detection JavaScript injection

## Dependencies

- **chromiumoxide**: Chromium automation via CDP
- **tokio**: Async runtime
- **axum**: HTTP server framework
- **anyhow**: Error handling
- **tracing**: Structured logging
- **riptide-core**: Shared stealth and core functionality

## License

Apache-2.0

## Contributing

Contributions are welcome! Please ensure:

1. All tests pass: `cargo test`
2. Code is formatted: `cargo fmt`
3. No clippy warnings: `cargo clippy`
4. Documentation is updated for public APIs

## See Also

- [chromiumoxide](https://github.com/mattsse/chromiumoxide) - Chromium automation
- [Chrome DevTools Protocol](https://chromedevtools.github.io/devtools-protocol/) - CDP documentation
- [Puppeteer](https://pptr.dev/) - Similar tool for Node.js
