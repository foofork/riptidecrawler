# RipTide Browser

**Infrastructure Layer - Browser Automation Adapter**

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)

Unified browser automation core for RipTide providing browser pool management, CDP connection pooling, and high-level launcher API with stealth capabilities.

## Quick Overview

RipTide Browser is the infrastructure adapter that abstracts browser automation concerns from the domain layer. It implements the `BrowserDriver` and `BrowserSession` port traits from `riptide-types`, providing concrete implementations for Chromium-based browser automation.

**What it does:**
- Manages browser instance lifecycle and pooling
- Provides CDP (Chrome DevTools Protocol) connection pooling
- Implements browser abstraction layer (trait-based, no concrete CDP types leak)
- Integrates stealth capabilities for bot detection avoidance
- Offers HTTP API for remote browser automation

**Port Implementation:**
- Implements `BrowserDriver` from `riptide-types`
- Implements `BrowserSession` for page interaction
- Provides `Pool<Browser>` for resource pooling

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                   RipTide Browser Layer                      │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐   │
│  │  Launcher   │  │ Browser Pool │  │  CDP Connection  │   │
│  │    API      │  │  Management  │  │      Pool        │   │
│  └─────────────┘  └──────────────┘  └──────────────────┘   │
│         │                 │                    │            │
│         └─────────────────┴────────────────────┘            │
│                           │                                 │
│                  ┌────────▼────────┐                        │
│                  │  Chromiumoxide  │                        │
│                  │  Spider Chrome  │                        │
│                  └─────────────────┘                        │
└──────────────────────────────────────────────────────────────┘
```

## Port Implementation

This adapter implements the following domain port traits:

### `BrowserDriver` (from riptide-types)

```rust
#[async_trait]
pub trait BrowserDriver: Send + Sync {
    async fn launch(&self) -> Result<Box<dyn BrowserSession>>;
    async fn launch_with_config(&self, config: BrowserConfig) -> Result<Box<dyn BrowserSession>>;
}
```

### `BrowserSession` (from riptide-types)

```rust
#[async_trait]
pub trait BrowserSession: Send + Sync {
    async fn navigate(&self, url: &str) -> Result<()>;
    async fn content(&self) -> Result<String>;
    async fn screenshot(&self) -> Result<Vec<u8>>;
    async fn execute_script(&self, script: &str) -> Result<serde_json::Value>;
}
```

### Why This Adapter Exists

The `riptide-browser` adapter exists to:
1. **Isolate infrastructure concerns** - Keep browser-specific details out of the domain layer
2. **Enable testing** - Domain logic can use mock browser drivers in tests
3. **Support multiple engines** - Abstraction allows switching between chromiumoxide, spider-chrome, etc.
4. **Provide pooling** - Expensive browser instances are reused efficiently
5. **Ensure reliability** - Health checks, automatic recovery, and circuit breakers

## Configuration

### Environment Variables

```bash
# Browser pool settings
BROWSER_POOL_MIN_SIZE=2
BROWSER_POOL_MAX_SIZE=10
BROWSER_POOL_INITIAL_SIZE=5

# Timeout settings
BROWSER_IDLE_TIMEOUT=60
BROWSER_MAX_LIFETIME=600
BROWSER_PAGE_TIMEOUT=30

# Memory management
BROWSER_MEMORY_THRESHOLD_MB=500
BROWSER_ENABLE_RECOVERY=true

# Stealth mode
ENABLE_STEALTH=true
STEALTH_PRESET=medium  # none, low, medium, high

# Health monitoring
BROWSER_HEALTH_CHECK_INTERVAL=10
BROWSER_ENABLE_MONITORING=true
```

### Programmatic Configuration

```rust
use riptide_browser::launcher::{HeadlessLauncher, LauncherConfig};
use riptide_browser::pool::BrowserPoolConfig;
use riptide_stealth::StealthPreset;
use std::time::Duration;

let config = LauncherConfig {
    pool_config: BrowserPoolConfig {
        min_pool_size: 2,
        max_pool_size: 10,
        initial_pool_size: 5,
        idle_timeout: Duration::from_secs(60),
        max_lifetime: Duration::from_secs(600),
        memory_threshold_mb: 500,
        enable_recovery: true,
        health_check_interval: Duration::from_secs(10),
        max_retries: 3,
    },
    default_stealth_preset: StealthPreset::Medium,
    enable_stealth: true,
    page_timeout: Duration::from_secs(30),
    enable_monitoring: true,
};

let launcher = HeadlessLauncher::with_config(config).await?;
```

## Usage Examples

### Basic Browser Automation

```rust
use riptide_browser::launcher::HeadlessLauncher;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create launcher (initializes browser pool)
    let launcher = HeadlessLauncher::new().await?;

    // Launch a page (checks out browser from pool)
    let session = launcher.launch_page_default("https://example.com").await?;

    // Access page content
    let content = session.content().await?;
    println!("Page content length: {}", content.len());

    // Take screenshot
    let screenshot = session.screenshot().await?;
    std::fs::write("screenshot.png", screenshot)?;

    // Session automatically returns browser to pool when dropped
    Ok(())
}
```

### Custom Stealth Configuration

```rust
use riptide_browser::launcher::HeadlessLauncher;
use riptide_stealth::StealthPreset;

let launcher = HeadlessLauncher::new().await?;

// Launch with high stealth for bot-resistant sites
let session = launcher.launch_page(
    "https://bot-protected-site.com",
    Some(StealthPreset::High)
).await?;

// Navigate and interact
session.navigate("https://bot-protected-site.com/page2").await?;
session.wait_for_element(".content", Some(5000)).await?;

let result = session.execute_script(
    "document.querySelector('.content').textContent"
).await?;
```

### Direct Browser Pool Access

```rust
use riptide_browser::pool::{BrowserPool, BrowserPoolConfig};

// Create browser pool directly for low-level control
let pool = BrowserPool::new(
    BrowserPoolConfig::default(),
    browser_config
).await?;

// Checkout browser from pool
let checkout = pool.checkout().await?;

// Create page
let page = checkout.new_page("https://example.com").await?;

// Use page...
let content = page.content().await?;

// Return to pool (automatic on drop)
checkout.checkin().await?;
```

### CDP Connection Pooling

```rust
use riptide_browser::cdp::{CdpConnectionPool, CdpPoolConfig};

let pool = CdpConnectionPool::new(CdpPoolConfig {
    min_connections: 5,
    max_connections: 20,
    idle_timeout: Duration::from_secs(30),
    health_check_interval: Duration::from_secs(10),
}).await?;

// Get pooled connection
let conn = pool.acquire().await?;

// Use connection for CDP commands
conn.execute_command(command).await?;

// Connection returns to pool on drop
```

### Monitoring Pool Health

```rust
// Get launcher statistics
let stats = launcher.stats().await;
println!("Total requests: {}", stats.total_requests);
println!("Success rate: {:.1}%",
    stats.successful_requests as f64 / stats.total_requests as f64 * 100.0
);
println!("Avg response time: {:.2}ms", stats.avg_response_time_ms);
println!("Pool utilization: {:.1}%", stats.pool_utilization);

// Get pool statistics
let pool_stats = pool.stats().await;
println!("Available: {}", pool_stats.available);
println!("In use: {}", pool_stats.in_use);
println!("Utilization: {:.1}%", pool_stats.utilization);
```

## Technical Details

### External Dependencies

- **chromiumoxide**: Chromium automation via Chrome DevTools Protocol (CDP)
- **spider_chrome**: Alternative CDP implementation with different performance characteristics
- **tokio**: Async runtime for non-blocking operations
- **anyhow/thiserror**: Error handling
- **serde/serde_json**: Serialization for CDP messages
- **uuid**: Unique identifiers for browsers and sessions
- **tempfile**: Temporary directory management for browser profiles

### Connection Management

**Browser Instance Lifecycle:**
1. **Creation**: Browsers launched with unique user data directories
2. **Validation**: Health checks ensure browser responsiveness
3. **Pooling**: Idle browsers kept warm for fast checkout
4. **Cleanup**: Stale/unhealthy browsers automatically terminated
5. **Limits**: Min/max pool size enforces resource boundaries

**CDP Connection Pooling:**
- WebSocket connections to browser instances are pooled separately
- Connections validated via CDP ping/pong protocol
- Failed connections automatically recreated
- Priority-based connection allocation

### Resource Lifecycle

```
┌─────────────┐
│   Create    │  Browser launched with stealth config
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Health    │  Periodic health checks (every 10s)
│   Check     │  Validates browser responsiveness
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Idle      │  Browser available in pool
│   Wait      │  Waiting for checkout
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Checkout   │  Browser assigned to session
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Return    │  Session ends, browser returns to pool
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Cleanup    │  If idle too long or max lifetime reached
└─────────────┘
```

### Performance Characteristics

| Operation | Latency | Notes |
|-----------|---------|-------|
| Pool checkout (cached) | <50ms | Browser already launched |
| Pool checkout (cold) | 2-3s | New browser launch required |
| Page navigation | 500ms-5s | Depends on target site |
| Screenshot capture | 200-500ms | Full page rendering |
| Script execution | <100ms | Simple scripts |
| CDP command | <50ms | Direct protocol communication |

## Anti-Corruption Layer

The browser adapter converts between infrastructure types and domain types:

```rust
// Infrastructure type (chromiumoxide::Page)
let page: chromiumoxide::Page = /* ... */;

// Convert to domain abstraction (BrowserSession trait)
let session: Box<dyn BrowserSession> = Box::new(ChromiumoxidePage(page));

// Domain layer only sees the trait, not concrete type
async fn extract_content(session: &dyn BrowserSession) -> Result<String> {
    session.content().await  // Works with any BrowserSession implementation
}
```

## Testing

### Integration Test Setup

```bash
# Ensure Chromium/Chrome is installed
# Ubuntu/Debian:
sudo apt-get install chromium-browser

# macOS:
brew install --cask chromium

# Or set CHROME_PATH:
export CHROME_PATH=/path/to/chrome

# Run tests
cargo test -p riptide-browser
```

### Test Fixtures and Helpers

```rust
// tests/common/mod.rs - Test utilities

use riptide_browser::pool::{BrowserPool, BrowserPoolConfig};

pub async fn create_test_pool() -> BrowserPool {
    BrowserPool::new(
        BrowserPoolConfig {
            min_pool_size: 1,
            max_pool_size: 3,
            initial_pool_size: 1,
            ..Default::default()
        },
        Default::default()
    ).await.unwrap()
}
```

### Mock vs Real Dependencies

```rust
// For unit tests, use mock BrowserSession
use mockall::mock;

mock! {
    pub BrowserSession {}

    #[async_trait]
    impl BrowserSession for BrowserSession {
        async fn navigate(&self, url: &str) -> Result<()>;
        async fn content(&self) -> Result<String>;
        async fn screenshot(&self) -> Result<Vec<u8>>;
    }
}

// For integration tests, use real browser
let launcher = HeadlessLauncher::new().await?;
let session = launcher.launch_page_default("https://example.com").await?;
```

## Error Handling

### Error Types

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BrowserError {
    #[error("Browser launch failed: {0}")]
    LaunchError(String),

    #[error("Navigation timeout after {0}s")]
    NavigationTimeout(u64),

    #[error("Browser pool exhausted")]
    PoolExhausted,

    #[error("Browser health check failed: {0}")]
    HealthCheckFailed(String),

    #[error("CDP protocol error: {0}")]
    CdpError(String),
}
```

### Retry Logic

Browser operations automatically retry on transient failures:

```rust
// Navigation with automatic retry
match session.navigate(url).await {
    Ok(_) => {}  // Success
    Err(BrowserError::NavigationTimeout(_)) => {
        // Automatic retry (up to 3 attempts)
    }
    Err(e) => return Err(e),  // Other errors fail fast
}
```

### Timeout Handling

All browser operations have configurable timeouts:

```rust
let session = launcher.launch_page_with_timeout(
    "https://slow-site.com",
    Duration::from_secs(60)  // Override default 30s timeout
).await?;
```

### Recovery Strategies

When browsers become unhealthy:
1. **Soft recovery**: Attempt to reset browser state (clear cookies, navigate to about:blank)
2. **Hard recovery**: Terminate and relaunch browser
3. **Circuit breaker**: If failure rate exceeds threshold, temporarily stop launching browsers

## Production Considerations

### Resource Limits

**Memory:**
- Each browser instance: 200-500 MB RAM
- Configure `max_pool_size` based on available memory
- Set `memory_threshold_mb` to detect memory leaks

**CPU:**
- Browser automation is CPU-intensive
- Limit concurrent browser operations
- Use connection pooling to reuse existing browsers

**File Descriptors:**
- Each browser: ~50-100 file descriptors
- Increase system limits: `ulimit -n 4096`

### Connection Pooling

**Recommended Pool Sizes:**

| Traffic Level | Min Pool | Max Pool | Initial |
|--------------|----------|----------|---------|
| Low | 1 | 3 | 2 |
| Medium | 2 | 5 | 3 |
| High | 5 | 10 | 5 |
| Very High | 10 | 20 | 10 |

**Tuning Guidelines:**
- Set `min_pool_size` to handle baseline load
- Set `max_pool_size` based on memory constraints
- Monitor `pool_utilization` and adjust accordingly

### Monitoring and Metrics

```rust
// Subscribe to pool events
let events = launcher.pool_events();
let mut receiver = events.lock().await;

tokio::spawn(async move {
    while let Some(event) = receiver.recv().await {
        match event {
            PoolEvent::BrowserCreated { id } => {
                info!("Browser created: {}", id);
            }
            PoolEvent::BrowserFailed { id, error } => {
                error!("Browser failed: {} - {}", id, error);
            }
            PoolEvent::MemoryAlert { browser_id, memory_mb } => {
                warn!("Memory alert: {} using {} MB", browser_id, memory_mb);
            }
            _ => {}
        }
    }
});
```

### Failure Modes

**Browser Launch Failure:**
- Chrome/Chromium not found → Install browser or set CHROME_PATH
- Port conflict → Browser pool will retry on different port
- Out of memory → Reduce `max_pool_size`

**Connection Timeout:**
- Browser unresponsive → Health check will remove browser
- Network issues → Retry logic will attempt reconnection
- Page load timeout → Increase `page_timeout`

**Memory Leak:**
- Browser memory exceeds threshold → Browser automatically recycled
- Enable `max_lifetime` to force periodic recycling

## Dependencies

### External Systems Required

- **Chrome/Chromium 90+**: Browser engine (installed separately)
- None (self-contained, no external services)

### Rust Crate Dependencies

| Dependency | Purpose |
|------------|---------|
| chromiumoxide | Core CDP implementation |
| spider_chrome | Alternative CDP implementation |
| tokio | Async runtime |
| riptide-stealth | Anti-detection capabilities |
| tempfile | Browser profile directories |
| serde/serde_json | CDP message serialization |
| tracing | Structured logging |
| uuid | Unique identifiers |

## Feature Flags

```toml
[dependencies]
riptide-browser = { version = "0.9", features = ["headless"] }
```

**Available Features:**
- `headless` (default): Full headless browser support (WIP, not fully wired)
- `stealth` (default): Stealth capabilities via riptide-stealth

## Performance Tips

1. **Reuse sessions** for multiple pages on same domain
2. **Disable JavaScript** if not needed (faster page loads)
3. **Disable images** to reduce bandwidth
4. **Enable monitoring** to identify bottlenecks
5. **Tune timeouts** based on target site characteristics
6. **Use CDP pooling** for high-frequency operations

## Related Crates

- **riptide-headless**: HTTP API wrapper around riptide-browser
- **riptide-stealth**: Stealth configuration and anti-detection
- **riptide-pool**: Generic resource pooling (browser pools build on this)
- **riptide-types**: Port trait definitions

## License

Apache-2.0
