# 🛠️ RipTide Config - Typed Configuration Management

**Category:** Configuration & Settings
**Purpose:** Type-safe configuration with validation, builders, and environment support

## Quick Overview

`riptide-config` provides comprehensive configuration management for the RipTide web scraping framework. It centralizes all configuration logic with strong typing, validation, and flexible loading from files, environment variables, or programmatic builders.

## Why This Exists

Configuration management was previously scattered across multiple crates, leading to:
- Inconsistent configuration patterns
- Duplicate validation logic
- Hard-to-track settings
- Difficult environment overrides

This crate consolidates all configuration into a single, well-tested module with strong type safety and comprehensive validation.

## Key Features

- **Builder Pattern**: Flexible, chainable configuration builders
- **Environment Variables**: Automatic loading with `RIPTIDE_` prefix
- **Validation**: Comprehensive security and format validation
- **Type Safety**: Strong typing with compile-time guarantees
- **Presets**: Pre-configured settings for common scenarios
- **Spider Config**: Specialized crawling configurations
- **API Config**: Authentication, rate limiting, and request settings

## Quick Start

```rust
use riptide_config::{SpiderConfig, SpiderPresets};

// Use a preset
let config = SpiderPresets::development();

// Or build custom
let config = SpiderConfig::default()
    .with_concurrency(8)
    .with_timeout(std::time::Duration::from_secs(30));

println!("Max concurrent requests: {}", config.max_concurrent_requests);
```

## Core Configuration Types

### SpiderConfig

Main configuration for web crawling operations:

```rust
use riptide_config::SpiderConfig;

let config = SpiderConfig {
    max_concurrent_requests: 10,
    request_timeout: Duration::from_secs(30),
    max_depth: Some(3),
    respect_robots_txt: true,
    user_agent: "RipTide/1.0".to_string(),
    ..Default::default()
};
```

### ApiConfig

HTTP API and authentication settings:

```rust
use riptide_config::{ApiConfig, RateLimitConfig};

let api_config = ApiConfig {
    rate_limit: RateLimitConfig {
        requests_per_minute: 60,
        burst_size: 10,
        ..Default::default()
    },
    authentication: Some(AuthenticationConfig {
        api_key_header: "X-API-Key".to_string(),
        ..Default::default()
    }),
    ..Default::default()
};
```

### ValidationConfig

Input validation and security settings:

```rust
use riptide_config::{ValidationConfig, CommonValidator};

let validator = CommonValidator::new(ValidationConfig {
    max_url_length: 2048,
    max_content_size: 10 * 1024 * 1024, // 10MB
    allowed_content_types: vec![
        "text/html".to_string(),
        "application/json".to_string(),
    ],
    ..Default::default()
});

// Validate URL
validator.validate_url("https://example.com")?;

// Validate content size
validator.validate_size(1024)?;
```

## Usage Examples

### Loading from Environment

```rust
use riptide_config::load_from_env;

// Reads from RIPTIDE_* environment variables
let config = load_from_env()?;

// Environment variables:
// RIPTIDE_MAX_CONCURRENCY=20
// RIPTIDE_TIMEOUT_SECS=60
// RIPTIDE_USER_AGENT="MyBot/1.0"
```

### Using Presets

```rust
use riptide_config::SpiderPresets;

// Development preset (permissive, verbose logging)
let dev_config = SpiderPresets::development();

// Production preset (strict, optimized)
let prod_config = SpiderPresets::production();

// Performance preset (high concurrency, aggressive caching)
let perf_config = SpiderPresets::performance();
```

### Custom Configuration with Validation

```rust
use riptide_config::{SpiderConfig, ValidationConfig};

let config = SpiderConfig::default()
    .with_concurrency(15)
    .with_timeout(Duration::from_secs(45))
    .with_max_depth(Some(5));

// Validate configuration
if config.max_concurrent_requests > 100 {
    return Err("Concurrency too high".into());
}

// Use validated config
println!("Config valid, starting crawler...");
```

### Builder Pattern

```rust
use riptide_config::{ConfigBuilder, DefaultConfigBuilder};

let config = DefaultConfigBuilder::new()
    .set_max_concurrency(20)
    .set_timeout_secs(60)
    .set_user_agent("CustomBot/1.0")
    .set_respect_robots_txt(true)
    .build()?;
```

### URL Processing Configuration

```rust
use riptide_config::UrlProcessingConfig;

let url_config = UrlProcessingConfig {
    normalize_urls: true,
    follow_redirects: true,
    max_redirects: 5,
    allowed_schemes: vec!["http".to_string(), "https".to_string()],
    blocked_domains: vec!["malicious.com".to_string()],
    allowed_domains: None, // Allow all except blocked
};
```

## API Reference

### Main Types

- `SpiderConfig` - Main crawling configuration
- `ApiConfig` - HTTP API settings
- `ValidationConfig` - Input validation rules
- `UrlProcessingConfig` - URL handling settings
- `PerformanceConfig` - Performance tuning options

### Builders

- `ConfigBuilder` - Generic configuration builder trait
- `DefaultConfigBuilder` - Default implementation
- `ValidationPatterns` - Regex patterns for validation

### Validators

- `CommonValidator` - General-purpose validator
- `UrlValidator` - URL-specific validation
- `ContentTypeValidator` - Content type checking
- `SizeValidator` - Size limit validation
- `ParameterValidator` - Query parameter validation

### Presets

- `SpiderPresets::development()` - Development settings
- `SpiderPresets::production()` - Production settings
- `SpiderPresets::performance()` - High-performance settings

## Integration with Other Crates

### Used By

- **riptide-spider**: Spider crawling configuration
- **riptide-api**: API server settings
- **riptide-fetch**: HTTP request configuration
- **riptide-headless**: Browser automation settings

### Example Integration

```rust
use riptide_config::SpiderConfig;
use riptide_spider::Spider;

let config = SpiderConfig::default()
    .with_concurrency(10);

let spider = Spider::with_config(config);
spider.crawl("https://example.com").await?;
```

## Configuration File Support

While this crate focuses on programmatic configuration, you can load from files:

```rust
use riptide_config::{SpiderConfig, load_from_env};
use std::fs;

// Load from JSON file
let json_str = fs::read_to_string("config.json")?;
let config: SpiderConfig = serde_json::from_str(&json_str)?;

// Or combine with environment overrides
let config = load_from_env()?;
```

## Testing

```bash
# Run all tests
cargo test -p riptide-config

# Test specific module
cargo test -p riptide-config validation

# Test with coverage
cargo tarpaulin -p riptide-config

# Run benchmarks
cargo bench -p riptide-config
```

### Example Test

```rust
use riptide_config::{SpiderConfig, ValidationConfig};

#[test]
fn test_config_validation() {
    let config = SpiderConfig::default();
    assert!(config.max_concurrent_requests > 0);
    assert!(config.request_timeout.as_secs() > 0);
}

#[test]
fn test_url_validation() {
    let validator = CommonValidator::new(ValidationConfig::default());

    assert!(validator.validate_url("https://example.com").is_ok());
    assert!(validator.validate_url("not a url").is_err());
    assert!(validator.validate_url("javascript:alert(1)").is_err());
}
```

## Common Patterns

### Layered Configuration

```rust
use riptide_config::{SpiderConfig, load_from_env};

// Start with defaults
let mut config = SpiderConfig::default();

// Apply environment overrides
if let Ok(env_config) = load_from_env() {
    config.max_concurrent_requests = env_config.max_concurrent_requests;
}

// Apply runtime overrides
config.user_agent = "CustomBot/1.0".to_string();
```

### Configuration Validation

```rust
use riptide_config::{SpiderConfig, ValidationConfig};

fn validate_config(config: &SpiderConfig) -> Result<(), String> {
    if config.max_concurrent_requests < 1 {
        return Err("Concurrency must be at least 1".into());
    }

    if config.max_depth.unwrap_or(0) > 10 {
        return Err("Max depth too high".into());
    }

    Ok(())
}
```

## Environment Variables

The following environment variables are supported:

| Variable | Description | Default |
|----------|-------------|---------|
| `RIPTIDE_MAX_CONCURRENCY` | Max concurrent requests | `10` |
| `RIPTIDE_TIMEOUT_SECS` | Request timeout | `30` |
| `RIPTIDE_MAX_DEPTH` | Maximum crawl depth | `None` |
| `RIPTIDE_USER_AGENT` | User-Agent header | `"RipTide/1.0"` |
| `RIPTIDE_RESPECT_ROBOTS` | Respect robots.txt | `true` |

## Best Practices

1. **Use Presets**: Start with a preset and customize from there
2. **Validate Early**: Validate configuration at startup, not during crawling
3. **Environment Overrides**: Use environment variables for deployment-specific settings
4. **Type Safety**: Leverage strong typing to catch errors at compile time
5. **Documentation**: Document custom configurations for your team

## Troubleshooting

### Common Issues

**Issue**: Configuration not loading from environment
```rust
// Solution: Check environment variable names
use std::env;
println!("RIPTIDE_MAX_CONCURRENCY={:?}", env::var("RIPTIDE_MAX_CONCURRENCY"));
```

**Issue**: Validation failing unexpectedly
```rust
// Solution: Check validation rules
let validator = CommonValidator::new(ValidationConfig::default());
match validator.validate_url(url) {
    Ok(_) => println!("Valid"),
    Err(e) => println!("Invalid: {}", e),
}
```

## License

Apache-2.0

## Related Crates

- `riptide-types` - Shared type definitions
- `riptide-spider` - Uses spider configuration
- `riptide-api` - Uses API configuration
- `riptide-fetch` - Uses request configuration
