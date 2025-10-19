# RipTide Config

Configuration management and validation for the RipTide web scraping framework.

## Overview

`riptide-config` provides comprehensive configuration management for RipTide, supporting YAML/JSON configuration files, environment variables, and programmatic configuration with validation and type safety.

## Features

- **Multi-Source Configuration**: YAML files, environment variables, and code
- **Type-Safe Settings**: Strong typing with serde deserialization
- **Validation**: Schema validation and constraint checking
- **Hot Reload**: Watch configuration files for changes
- **Environment Overrides**: Environment variables override file settings
- **Default Values**: Sensible defaults with override capability
- **Namespaced Settings**: Logical grouping of related configuration
- **Profile Support**: Development, staging, production profiles

## Architecture

```
┌────────────────────────────────────────────────────────┐
│              Configuration Management                  │
├────────────────────────────────────────────────────────┤
│                                                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │  File Parser │  │   Env Vars   │  │  Validator   │ │
│  │  (YAML/JSON) │  │   Override   │  │   (Schema)   │ │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘ │
│         │                 │                  │         │
│         └─────────────────┼──────────────────┘         │
│                           ▼                            │
│                  ┌─────────────────┐                   │
│                  │  Config Merger  │                   │
│                  └────────┬────────┘                   │
│                           │                            │
│                           ▼                            │
│                  ┌─────────────────┐                   │
│                  │ Validated Config│                   │
│                  │    (Runtime)    │                   │
│                  └─────────────────┘                   │
└────────────────────────────────────────────────────────┘
```

## Usage

### Loading Configuration

```rust
use riptide_config::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Load from file
    let config = Config::from_file("configs/riptide.yml")?;

    // Load with environment overrides
    let config = Config::from_file_with_env("configs/riptide.yml")?;

    // Load from environment only
    let config = Config::from_env()?;

    // Programmatic configuration
    let config = Config::builder()
        .redis_url("redis://localhost:6379")
        .max_concurrency(10)
        .build()?;

    Ok(())
}
```

### Configuration File Format

**YAML Configuration (`configs/riptide.yml`):**

```yaml
# Core settings
resources:
  max_concurrent_renders: 10
  max_concurrent_pdf: 2
  max_concurrent_wasm: 4
  global_timeout_secs: 30

# Performance tuning
performance:
  render_timeout_secs: 3
  pdf_timeout_secs: 10
  wasm_timeout_secs: 5
  worker_threads: 8

# Rate limiting
rate_limiting:
  enabled: true
  requests_per_second_per_host: 1.5
  jitter_factor: 0.1
  burst_capacity_per_host: 3

# Headless browser
headless:
  max_pool_size: 3
  min_pool_size: 1
  idle_timeout_secs: 300
  headless_url: "http://localhost:9123"

# Memory management
memory:
  global_memory_limit_mb: 2048
  pressure_threshold: 0.85
  auto_gc: true

# Cache settings
cache:
  redis_url: "redis://localhost:6379"
  default_ttl_secs: 3600
  max_size_mb: 1024

# Search provider
search:
  backend: "serper"  # serper, searxng, none
  timeout_secs: 30
  enable_url_parsing: true

# Telemetry (optional)
telemetry:
  enabled: true
  otel_endpoint: "http://localhost:4317"
  service_name: "riptide-api"

# Logging
logging:
  level: "info"
  format: "json"
  file: "/var/log/riptide.log"
```

### Environment Variable Overrides

```bash
# Core settings
export RIPTIDE_MAX_CONCURRENT_RENDERS=20
export RIPTIDE_GLOBAL_TIMEOUT_SECS=60

# Performance
export RIPTIDE_RENDER_TIMEOUT=5
export RIPTIDE_WORKER_THREADS=16

# Rate limiting
export RIPTIDE_RATE_LIMIT_RPS=3.0
export RIPTIDE_RATE_LIMIT_ENABLED=true

# Headless
export HEADLESS_URL="http://localhost:9123"
export HEADLESS_POOL_SIZE=5

# Memory
export RIPTIDE_MEMORY_LIMIT_MB=4096

# Cache
export REDIS_URL="redis://localhost:6379/0"
export CACHE_TTL_SECS=7200

# Search
export SEARCH_BACKEND="serper"
export SERPER_API_KEY="your-api-key"

# Telemetry
export OTEL_ENDPOINT="http://jaeger:4317"
export RUST_LOG="info,riptide_core=debug"
```

### Accessing Configuration

```rust
use riptide_config::*;

let config = Config::load()?;

// Resource limits
let max_renders = config.resources.max_concurrent_renders;
let timeout = config.resources.global_timeout_secs;

// Performance settings
let render_timeout = config.performance.render_timeout_secs;
let workers = config.performance.worker_threads;

// Rate limiting
if config.rate_limiting.enabled {
    let rps = config.rate_limiting.requests_per_second_per_host;
    println!("Rate limit: {} req/s", rps);
}

// Headless configuration
let pool_size = config.headless.max_pool_size;
let url = &config.headless.headless_url;

// Cache settings
let redis_url = &config.cache.redis_url;
let ttl = config.cache.default_ttl_secs;
```

### Configuration Profiles

```rust
use riptide_config::*;

// Load development profile
let config = Config::for_profile("development")?;

// Load production profile
let config = Config::for_profile("production")?;

// Custom profile
let config = Config::for_profile("staging")?;
```

**Profile Files:**
- `configs/riptide.yml` - Base configuration
- `configs/riptide-dev.yml` - Development overrides
- `configs/riptide-prod.yml` - Production settings
- `configs/riptide-staging.yml` - Staging configuration

### Validation

```rust
use riptide_config::*;

let config = Config::load()?;

// Validate configuration
config.validate()?;

// Custom validation
if config.resources.max_concurrent_renders > 100 {
    return Err(ConfigError::InvalidValue(
        "max_concurrent_renders must be <= 100".to_string()
    ));
}

// Validate constraints
config.validate_constraints()?;
```

### Hot Reload

```rust
use riptide_config::*;

// Watch configuration file
let (config_rx, _watcher) = Config::watch("configs/riptide.yml")?;

tokio::spawn(async move {
    while let Ok(new_config) = config_rx.recv().await {
        println!("Configuration reloaded");
        // Apply new configuration
    }
});
```

## Configuration Structure

### Core Types

```rust
pub struct Config {
    pub resources: ResourceConfig,
    pub performance: PerformanceConfig,
    pub rate_limiting: RateLimitConfig,
    pub headless: HeadlessConfig,
    pub memory: MemoryConfig,
    pub cache: CacheConfig,
    pub search: SearchConfig,
    pub telemetry: TelemetryConfig,
    pub logging: LoggingConfig,
}

pub struct ResourceConfig {
    pub max_concurrent_renders: usize,
    pub max_concurrent_pdf: usize,
    pub max_concurrent_wasm: usize,
    pub global_timeout_secs: u64,
}

pub struct PerformanceConfig {
    pub render_timeout_secs: u64,
    pub pdf_timeout_secs: u64,
    pub wasm_timeout_secs: u64,
    pub worker_threads: usize,
}

// ... additional config types
```

## Default Values

```rust
impl Default for Config {
    fn default() -> Self {
        Self {
            resources: ResourceConfig {
                max_concurrent_renders: 10,
                max_concurrent_pdf: 2,
                max_concurrent_wasm: 4,
                global_timeout_secs: 30,
            },
            performance: PerformanceConfig {
                render_timeout_secs: 3,
                pdf_timeout_secs: 10,
                wasm_timeout_secs: 5,
                worker_threads: 8,
            },
            // ... additional defaults
        }
    }
}
```

## Integration with RipTide

This crate is used by:

- **riptide-api**: API server configuration
- **riptide-core**: Core framework settings
- **riptide-engine**: Browser engine configuration
- **riptide-workers**: Worker queue settings

## Testing

```bash
# Run tests
cargo test -p riptide-config

# Test with sample configs
cargo test -p riptide-config -- --nocapture

# Validate test configurations
cargo test -p riptide-config validation_tests
```

## License

Apache-2.0

## Related Crates

- **riptide-types**: Shared type definitions
- **riptide-api**: API server using configuration
- **riptide-core**: Core framework integration
