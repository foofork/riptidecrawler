# Riptide Stealth

A comprehensive stealth module for anti-detection measures in browser automation. This crate provides advanced techniques to evade bot detection systems through browser fingerprint randomization, user-agent rotation, TLS fingerprinting, and various evasion strategies.

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/riptide-stealth.svg)](https://crates.io/crates/riptide-stealth)

## Overview

Modern web scraping and browser automation faces increasingly sophisticated bot detection systems. `riptide-stealth` provides a comprehensive suite of anti-detection measures that make automated browsers appear more human-like and difficult to fingerprint. The crate is designed to integrate seamlessly with the RipTide browser automation framework, particularly with `riptide-headless`.

### Key Capabilities

- **Multi-layer Fingerprint Protection**: Randomize WebGL, Canvas, Audio, Hardware, and Font fingerprints
- **Intelligent User-Agent Management**: Four rotation strategies (Random, Sequential, Sticky, Domain-based)
- **Browser API Overrides**: Comprehensive JavaScript injection to mask automation signals
- **Request Randomization**: Dynamic header, timing, viewport, and locale variations
- **Adaptive Rate Limiting**: Domain-specific request throttling with burst support
- **Human-like Behavior Simulation**: Natural mouse movements, scrolling, and reading pauses
- **Enhanced Privacy Controls**: WebRTC leak prevention, timezone spoofing, screen resolution masking

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
riptide-stealth = "0.1"

# Enable full stealth features
riptide-stealth = { version = "0.1", features = ["full-stealth"] }
```

## Stealth Techniques

### 1. Browser Fingerprint Randomization

The crate randomizes various browser fingerprinting vectors:

- **WebGL Fingerprinting**: Randomizes GPU vendor/renderer, adds noise to rendering output
- **Canvas Fingerprinting**: Injects pixel-level noise to prevent canvas-based tracking
- **Audio Context Fingerprinting**: Adds noise to audio fingerprinting attempts
- **Hardware Fingerprinting**: Spoofs CPU cores, device memory, battery information
- **Font Fingerprinting**: Limits and controls reported font availability

### 2. User-Agent Rotation

Four sophisticated rotation strategies:

- **Random**: Randomly select from user-agent pool on each request
- **Sequential**: Rotate through agents in order for predictable behavior
- **Sticky**: Maintain the same user-agent for session consistency
- **Domain-based**: Use consistent agents per domain to avoid detection

### 3. Request Randomization

- **Header Variations**: Randomize Accept, Accept-Language, Accept-Encoding headers
- **Timing Jitter**: Add natural delays between requests with configurable variance
- **Viewport Randomization**: Vary screen resolution across common desktop sizes
- **Locale/Timezone Spoofing**: Match locale with appropriate timezone settings

### 4. JavaScript Evasion

Comprehensive browser API overrides to hide automation signals:

- Override `navigator.webdriver` property
- Disable automation-controlled flags
- Mock browser plugins (PDF viewer, Native Client)
- Spoof permissions API
- Override `window.chrome` object
- WebRTC leak prevention
- Media device spoofing

### 5. TLS Fingerprinting

Integration points for TLS fingerprint randomization to avoid network-level detection (requires browser configuration).

## Preset Levels

The crate provides four preset levels for easy configuration:

### None (No Stealth)
- No anti-detection measures applied
- Fastest performance
- Use for trusted environments or testing

```rust
let controller = StealthController::from_preset(StealthPreset::None);
```

### Low (Minimal Stealth)
- Basic automation flag hiding
- Sequential user-agent rotation
- 10% timing jitter
- Minimal fingerprint randomization
- Best for: Low-security sites, internal tools

```rust
let controller = StealthController::from_preset(StealthPreset::Low);
```

### Medium (Balanced) - Default
- Moderate fingerprint randomization
- 20% timing jitter
- Random user-agent rotation
- Enhanced header consistency
- Best for: General web scraping, most commercial sites

```rust
let controller = StealthController::from_preset(StealthPreset::Medium);
```

### High (Maximum Stealth)
- Maximum fingerprint randomization
- 40% timing jitter
- Full WebRTC leak prevention
- Hardware spoofing enabled
- All countermeasures active
- Best for: High-security sites, anti-bot systems

```rust
let controller = StealthController::from_preset(StealthPreset::High);
```

## StealthController API

### Core Methods

```rust
use riptide_stealth::{StealthController, StealthPreset};

// Create controller with preset
let mut controller = StealthController::from_preset(StealthPreset::Medium);

// Get next user-agent (rotates based on strategy)
let user_agent = controller.next_user_agent();

// Generate randomized HTTP headers
let headers = controller.generate_headers();

// Calculate delay before next request (with jitter)
let delay = controller.calculate_delay();
tokio::time::sleep(delay).await;

// Get stealth JavaScript injection code
let stealth_js = controller.get_stealth_js();

// Generate random viewport size
let (width, height) = controller.random_viewport();

// Get locale and timezone pair
let (locale, timezone) = controller.random_locale();

// Check if request is rate-limited
match controller.check_rate_limit_for_domain("example.com").await {
    Ok(delay) => {
        tokio::time::sleep(delay).await;
        // Make request
    }
    Err(retry_after) => {
        println!("Rate limited, retry after {:?}", retry_after);
    }
}

// Record request result for adaptive throttling
controller.record_request_result("example.com", true, Some(200));
```

### Advanced Features

```rust
// Access rate limiter for fine-grained control
let rate_limiter = controller.rate_limiter();
let stats = rate_limiter.get_stats("example.com");
println!("Success rate: {:.2}%", stats.success_rate * 100.0);

// Simulate human-like mouse movement
let behavior = controller.behavior_simulator();
let mouse_path = behavior.generate_mouse_path(
    (0, 0),     // start
    (500, 300), // end
    0.5         // curvature
);

// Generate natural scroll actions
let scroll = behavior.generate_scroll_action(
    1000, // target_y
    500,  // duration_ms
    0.3   // variance
);

// Get enhanced screen resolution with DPI
let resolution = controller.get_enhanced_resolution();
println!("{}x{} @ {}dpi", resolution.width, resolution.height, resolution.pixel_ratio);

// Validate header consistency with current user-agent
match controller.validate_headers(&headers) {
    Ok(_) => println!("Headers are consistent"),
    Err(issues) => println!("Header issues: {:?}", issues),
}
```

## Usage Examples

### Basic Web Scraping

```rust
use riptide_stealth::{StealthController, StealthPreset};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create controller with high stealth
    let mut controller = StealthController::from_preset(StealthPreset::High);

    // Prepare request
    let user_agent = controller.next_user_agent();
    let headers = controller.generate_headers();
    let (width, height) = controller.random_viewport();

    println!("User-Agent: {}", user_agent);
    println!("Viewport: {}x{}", width, height);

    // Add natural delay before request
    let delay = controller.calculate_delay();
    tokio::time::sleep(delay).await;

    // Make request with stealth headers
    // ... your HTTP client code here ...

    // Record success for adaptive rate limiting
    controller.record_request_result("example.com", true, Some(200));

    Ok(())
}
```

### Browser Automation Integration

```rust
use riptide_stealth::{StealthController, StealthPreset};

async fn setup_stealth_browser() -> Result<(), Box<dyn std::error::Error>> {
    let mut controller = StealthController::from_preset(StealthPreset::Medium);

    // Get CDP launch flags
    let cdp_flags = controller.get_cdp_flags();
    println!("CDP Flags: {:?}", cdp_flags);

    // Launch browser with flags
    // let browser = Browser::launch(cdp_flags).await?;

    // Get JavaScript to inject on page load
    let stealth_js = controller.get_stealth_js();

    // Inject into page
    // page.evaluate(&stealth_js).await?;

    // Get consistent headers
    let headers = controller.generate_headers();

    // Set headers in browser
    // page.set_extra_headers(headers).await?;

    Ok(())
}
```

### Domain-Specific Rate Limiting

```rust
use riptide_stealth::{StealthController, StealthConfig, DomainTiming};
use std::collections::HashMap;

async fn scrape_multiple_domains() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = StealthConfig::from_preset(StealthPreset::Medium);

    // Configure per-domain timing
    let mut per_domain = HashMap::new();
    per_domain.insert("slow-site.com".to_string(), DomainTiming {
        min_delay_ms: 2000,
        max_delay_ms: 5000,
        rpm_limit: Some(30),
        burst_size: 3,
    });
    per_domain.insert("fast-site.com".to_string(), DomainTiming {
        min_delay_ms: 500,
        max_delay_ms: 1000,
        rpm_limit: Some(120),
        burst_size: 10,
    });

    config.timing.per_domain = per_domain;
    let mut controller = StealthController::new(config);

    // Check rate limit before request
    let domain = "slow-site.com";
    match controller.check_rate_limit_for_domain(domain).await {
        Ok(delay) => {
            tokio::time::sleep(delay).await;
            println!("Making request to {}", domain);
            // Make request...
            controller.record_request_result(domain, true, Some(200));
        }
        Err(retry_after) => {
            println!("Rate limited on {}, waiting {:?}", domain, retry_after);
            tokio::time::sleep(retry_after).await;
        }
    }

    Ok(())
}
```

### Human-like Behavior Simulation

```rust
use riptide_stealth::{StealthController, StealthPreset};

async fn simulate_human_interaction() {
    let controller = StealthController::from_preset(StealthPreset::High);
    let behavior = controller.behavior_simulator();

    // Generate natural mouse movement to click target
    let mouse_path = behavior.generate_mouse_path(
        (100, 100),  // current position
        (500, 350),  // button position
        0.7          // higher curvature = more natural
    );

    println!("Mouse path has {} points", mouse_path.len());

    // Simulate mouse movement with delays
    for point in mouse_path {
        // Move mouse to point.x, point.y
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    // Add natural delay before click
    let click_delay = behavior.generate_click_delay();
    tokio::time::sleep(click_delay).await;

    // Generate natural scroll to element
    let scroll = behavior.generate_scroll_action(
        1200,  // target Y position
        800,   // duration in ms
        0.2    // timing variance
    );

    println!("Scrolling with {} steps", scroll.steps.len());

    // Simulate reading pause
    let reading_pause = behavior.generate_reading_pause(5000); // 5 seconds of content
    tokio::time::sleep(reading_pause).await;
}
```

## Configuration

### Custom Configuration

```rust
use riptide_stealth::{
    StealthConfig, StealthPreset, UserAgentConfig, RotationStrategy,
    RequestRandomization, HeaderRandomization, TimingJitter,
    ViewportRandomization, LocaleRandomization, LocaleStrategy,
    FingerprintingConfig, WebGlConfig, CanvasConfig,
};

let mut config = StealthConfig::from_preset(StealthPreset::Medium);

// Customize user-agent rotation
config.user_agent.strategy = RotationStrategy::DomainBased;
config.user_agent.agents = vec![
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string(),
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36".to_string(),
];

// Customize timing jitter
config.request_randomization.timing_jitter.base_delay_ms = 1500;
config.request_randomization.timing_jitter.jitter_percentage = 0.3;

// Customize fingerprinting
config.fingerprinting.webgl.noise_level = 0.15;
config.fingerprinting.canvas.noise_intensity = 0.08;

// Load custom user-agents from file
config.ua_file_path = Some("custom_agents.txt".to_string());

let controller = StealthController::new(config);
```

### Loading User-Agents from File

Create a text file (e.g., `user_agents.txt`):

```text
# Desktop browsers
Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36
Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36
Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36

# Comments are ignored
Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:121.0) Gecko/20100101 Firefox/121.0
```

```rust
use riptide_stealth::{load_user_agents_from_file, StealthConfig};

let agents = load_user_agents_from_file("user_agents.txt")?;
let mut config = StealthConfig::default();
config.user_agent.agents = agents;
```

## Feature Flags

### Default Features
- `stealth`: Basic stealth capabilities (enabled by default)

### Optional Features
- `full-stealth`: Enables all advanced stealth features including:
  - Enhanced WebRTC protection
  - Advanced timezone spoofing
  - Sophisticated header consistency checks
  - Behavior simulation
  - Rate limiting

```toml
[dependencies]
riptide-stealth = { version = "0.1", features = ["full-stealth"] }
```

## Integration with riptide-headless

The stealth module is designed to integrate seamlessly with `riptide-headless`:

```rust
use riptide_stealth::{StealthController, StealthPreset};
use riptide_headless::Browser;

async fn launch_stealth_browser() -> Result<(), Box<dyn std::error::Error>> {
    let mut controller = StealthController::from_preset(StealthPreset::High);

    // Get CDP flags for browser launch
    let mut launch_args = controller.get_cdp_flags();
    launch_args.push("--headless=new".to_string());

    // Launch browser
    let browser = Browser::launch_with_args(&launch_args).await?;
    let page = browser.new_page().await?;

    // Inject stealth JavaScript
    let stealth_js = controller.get_stealth_js();
    page.evaluate_on_new_document(&stealth_js).await?;

    // Set randomized viewport
    let (width, height) = controller.random_viewport();
    page.set_viewport(width, height).await?;

    // Set consistent headers
    let headers = controller.generate_headers();
    page.set_extra_http_headers(headers).await?;

    // Navigate with rate limiting
    match controller.check_rate_limit_for_domain("example.com").await {
        Ok(delay) => {
            tokio::time::sleep(delay).await;
            page.goto("https://example.com").await?;
            controller.record_request_result("example.com", true, Some(200));
        }
        Err(retry_after) => {
            tokio::time::sleep(retry_after).await;
            page.goto("https://example.com").await?;
        }
    }

    Ok(())
}
```

## Testing

The crate includes comprehensive unit and integration tests:

```bash
# Run all tests
cargo test

# Run specific test module
cargo test --test stealth_tests

# Run with output
cargo test -- --nocapture

# Test specific preset
cargo test test_high_preset
```

### Test Coverage

- Unit tests for all core components
- Integration tests for stealth lifecycle
- JavaScript injection verification
- Rate limiting behavior
- Header consistency validation
- Fingerprint randomization
- Behavior simulation accuracy

## Effectiveness Considerations

### What This Crate Can Do

✅ Hide obvious automation signals (`navigator.webdriver`, automation flags)
✅ Randomize fingerprints to prevent tracking across sessions
✅ Implement natural timing patterns and delays
✅ Simulate human-like interaction patterns
✅ Manage rate limiting to avoid triggering defenses
✅ Maintain header consistency to avoid detection

### What This Crate Cannot Do Alone

❌ Bypass CAPTCHAs (requires CAPTCHA solving services)
❌ Defeat advanced ML-based bot detection (requires behavioral modeling)
❌ Hide from ISP-level monitoring
❌ Circumvent authenticated bot detection systems
❌ Guarantee 100% undetectability (arms race with detection systems)

### Best Practices

1. **Start Conservative**: Begin with `Medium` or `Low` presets
2. **Monitor Success Rates**: Use `record_request_result()` to track effectiveness
3. **Respect Rate Limits**: Configure appropriate delays and RPM limits
4. **Rotate Properly**: Use `DomainBased` strategy for multi-domain scraping
5. **Keep Updated**: User-agent lists and fingerprints should be refreshed regularly
6. **Test Thoroughly**: Verify stealth measures against target sites
7. **Combine Techniques**: Use with proxies, residential IPs, and CAPTCHA solvers
8. **Stay Legal**: Respect robots.txt and terms of service

### Detection Indicators

Even with stealth measures, these can still reveal automation:

- Perfect timing consistency (use jitter)
- Identical request patterns (vary behavior)
- Unrealistic interaction speed (add pauses)
- Missing mouse/keyboard events (simulate behavior)
- Suspicious header combinations (validate consistency)
- IP reputation (use residential proxies)

## Performance Considerations

Stealth measures add overhead:

- **None**: No overhead (~0ms per request)
- **Low**: Minimal overhead (~1-5ms per request)
- **Medium**: Moderate overhead (~5-15ms per request)
- **High**: Higher overhead (~15-50ms per request)

The overhead comes from:
- Random number generation for fingerprints
- JavaScript code generation and injection
- Header consistency validation
- Rate limit calculations
- Behavior simulation

For high-throughput scenarios, consider:
- Using `Low` or `Medium` presets
- Pre-generating user-agents
- Caching JavaScript injection code
- Batching rate limit checks

## Dependencies

Core dependencies (minimal):
- `serde` - Configuration serialization
- `rand` - Randomization for fingerprints
- `tracing` - Logging and diagnostics
- `dashmap` - Concurrent rate limiting

Development dependencies:
- `tokio-test` - Async testing
- `anyhow` - Error handling in tests

## Contributing

Contributions are welcome! Areas for improvement:

- Additional fingerprinting techniques
- More user-agent profiles
- Advanced timing patterns
- ML-based behavior modeling
- Platform-specific optimizations

## License

Licensed under the Apache License, Version 2.0. See LICENSE file for details.

## See Also

- [`riptide-headless`](../riptide-headless) - Headless browser automation
- [`riptide-api`](../riptide-api) - API server for browser automation
- [`riptide-search`](../riptide-search) - Web search capabilities

## Version History

### 0.1.0 (Current)
- Initial release
- Four stealth presets (None, Low, Medium, High)
- User-agent rotation strategies
- Fingerprint randomization
- Rate limiting and behavior simulation
- Integration with riptide-headless
