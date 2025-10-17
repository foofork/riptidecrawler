# RipTide Headless Hybrid

A hybrid headless browser launcher that combines **spider-chrome** for high-performance browser automation with **EventMesh stealth features** for anti-detection.

## Overview

This crate is part of Phase 1 Week 1 (P1-C1) of the spider-chrome migration strategy. It provides a **facade pattern** that:

1. Uses `spider-chrome` internally for browser automation
2. Applies EventMesh stealth features as middleware
3. Maintains API compatibility with the existing `riptide-headless` interface
4. Enables gradual migration without breaking changes

## Architecture

```
┌─────────────────────────────────────┐
│  HybridHeadlessLauncher (Facade)    │
│  - Maintains existing API           │
│  - Coordinates components           │
└──────────┬──────────────────────────┘
           │
           ├──────────────────────────┐
           │                          │
┌──────────▼────────┐      ┌─────────▼──────────┐
│  spider-chrome    │      │ Stealth Middleware │
│  - Browser launch │      │ - Fingerprinting   │
│  - Page control   │      │ - Navigator override│
│  - CDP automation │      │ - Canvas/WebGL     │
└───────────────────┘      └────────────────────┘
```

## Features

- ✅ **spider-chrome Integration**: Uses spider-chrome 2.37.128 for browser automation
- ✅ **Stealth Middleware**: Applies EventMesh anti-detection measures
- ✅ **API Compatibility**: Maintains LaunchSession interface
- ✅ **Fingerprinting Protection**: WebGL, Canvas, Audio, Hardware spoofing
- ✅ **Navigator Overrides**: Removes automation signals
- ✅ **Performance Monitoring**: Built-in statistics and metrics

## Usage

### Basic Usage

```rust
use riptide_headless_hybrid::HybridHeadlessLauncher;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create launcher with default config
    let launcher = HybridHeadlessLauncher::new().await?;

    // Launch a page
    let session = launcher.launch_page_default("https://example.com").await?;

    // Get page content
    let html = session.content().await?;
    println!("Page HTML length: {}", html.len());

    // Session automatically cleaned up when dropped
    Ok(())
}
```

### With Stealth Configuration

```rust
use riptide_headless_hybrid::{HybridHeadlessLauncher, LauncherConfig};
use riptide_stealth::StealthPreset;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create launcher with high stealth
    let config = LauncherConfig {
        default_stealth_preset: StealthPreset::High,
        enable_stealth: true,
        ..Default::default()
    };

    let launcher = HybridHeadlessLauncher::with_config(config).await?;

    // Launch with custom stealth level
    let session = launcher
        .launch_page("https://example.com", Some(StealthPreset::High))
        .await?;

    // Execute JavaScript
    let result = session
        .execute_script("return navigator.webdriver === undefined;")
        .await?;

    println!("Webdriver hidden: {}", result);

    Ok(())
}
```

### Statistics and Monitoring

```rust
use riptide_headless_hybrid::HybridHeadlessLauncher;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let launcher = HybridHeadlessLauncher::new().await?;

    // Launch multiple pages
    for url in &["https://example.com", "https://google.com"] {
        let session = launcher.launch_page_default(url).await?;
        let _ = session.content().await?;
    }

    // Get statistics
    let stats = launcher.stats().await;
    println!("Total requests: {}", stats.total_requests);
    println!("Successful: {}", stats.successful_requests);
    println!("Failed: {}", stats.failed_requests);
    println!("Avg response time: {:.2}ms", stats.avg_response_time_ms);
    println!("Stealth requests: {}", stats.stealth_requests);

    Ok(())
}
```

## Stealth Features

The hybrid launcher applies these anti-detection measures:

### 1. Navigator Overrides
- Sets `navigator.webdriver` to `undefined`
- Mocks realistic plugin list
- Sets proper language arrays
- Overrides permissions API

### 2. Fingerprinting Protection
- **WebGL**: Randomizes vendor and renderer strings
- **Canvas**: Adds subtle noise to prevent fingerprinting
- **Audio**: Adds noise to audio context
- **Hardware**: Spoofs CPU cores and device memory
- **Screen**: Sets realistic resolution and color depth

### 3. CDP Flags
- Disables automation-controlled flag
- Removes detection signals
- Configures stealth Chrome arguments

## LaunchSession API

The `LaunchSession` provides these methods:

```rust
impl LaunchSession<'_> {
    fn session_id(&self) -> &str;
    fn duration(&self) -> Duration;
    async fn navigate(&self, url: &str) -> Result<()>;
    async fn wait_for_element(&self, selector: &str, timeout_ms: Option<u64>) -> Result<()>;
    async fn execute_script(&self, script: &str) -> Result<serde_json::Value>;
    async fn screenshot(&self) -> Result<Vec<u8>>;
    async fn content(&self) -> Result<String>;
    async fn close(self) -> Result<()>;
}
```

This matches the existing `riptide-headless` API for compatibility.

## Configuration

### LauncherConfig

```rust
pub struct LauncherConfig {
    /// Browser pool configuration
    pub pool_config: PoolConfig,

    /// Default stealth preset
    pub default_stealth_preset: StealthPreset,

    /// Enable stealth by default
    pub enable_stealth: bool,

    /// Page navigation timeout
    pub page_timeout: Duration,

    /// Enable performance monitoring
    pub enable_monitoring: bool,
}
```

### Stealth Presets

- **None**: No stealth measures applied
- **Low**: Basic stealth with minimal fingerprint changes
- **Medium**: Balanced detection vs performance (default)
- **High**: Maximum stealth with all countermeasures enabled

## Migration Path

This crate enables gradual migration:

1. **Phase 1 (Current)**: Use `HybridHeadlessLauncher` alongside existing code
2. **Phase 2**: Replace `riptide-headless` usage with hybrid launcher
3. **Phase 3**: Remove old `chromiumoxide` dependency
4. **Phase 4**: Optimize for spider-chrome native features

## Testing

Run tests:

```bash
cargo test -p riptide-headless-hybrid
```

Note: Browser launch tests are commented out by default. To run full integration tests, you need Chrome/Chromium installed.

## Dependencies

- `spider_chrome`: 2.37.128 - High-concurrency CDP automation
- `riptide-stealth`: EventMesh anti-detection features
- `riptide-core`: Core EventMesh types

## Performance

Expected improvements with spider-chrome:

- **10-20x** faster concurrent browser launches
- **Improved** memory management with pooling
- **Better** CDP connection stability
- **Native** async/await patterns

## Compatibility

This crate maintains compatibility with:

- Existing `LaunchSession` interface
- Stealth configuration patterns
- Statistics and monitoring APIs
- Session lifecycle management

## Next Steps (Phase 2)

After P1-C1 completion, Phase 2 will:

1. Port pool management to use spider-chrome
2. Implement advanced session persistence
3. Add concurrent page management
4. Optimize CDP communication patterns

## License

Apache-2.0

## Authors

RipTide Team
