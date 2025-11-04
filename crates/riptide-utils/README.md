# Riptide Utils

Shared utilities for Riptide EventMesh platform (Phase 0 Week 0-1).

## Overview

This crate provides common utilities used across the Riptide EventMesh platform, following TDD (Test-Driven Development) principles with comprehensive test coverage.

## Modules

### Redis (`redis.rs`)
- **RedisPool**: Connection pool with health checks
- **RedisConfig**: Configuration for Redis connections
- Features:
  - Multiplexed async connections
  - Configurable timeouts
  - PING-based health checks
  - Connection retry support

### HTTP (`http.rs`)
- **HttpClientFactory**: Factory for creating configured HTTP clients
- **HttpConfig**: HTTP client configuration
- Features:
  - Connection pooling
  - Configurable timeouts (request, connect, idle)
  - rustls-tls for secure connections
  - Custom user agent support

### Retry (`retry.rs`)
- **RetryPolicy**: Exponential backoff retry policy
- Features:
  - Configurable max attempts
  - Exponential backoff calculation
  - Backoff capping
  - Async operation retry with generic error handling

### Rate Limiting (`rate_limit.rs`)
- **SimpleRateLimiter**: Token bucket rate limiter
- **RateLimiterBuilder**: Builder pattern for rate limiter configuration
- Features:
  - Requests per second limiting
  - Burst size configuration
  - Async wait support
  - Thread-safe (Arc-based)

### Time (`time.rs`)
- Unix timestamp utilities (seconds, milliseconds, microseconds)
- DateTime conversion functions
- ISO 8601 formatting and parsing
- Duration calculations
- Expiration checking

### Error (`error.rs`)
- Re-exports of `anyhow` and `thiserror`
- Common `Result` type alias
- Convenient error handling utilities

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
riptide-utils = { path = "../riptide-utils" }
```

### Examples

#### Redis Pool
```rust
use riptide_utils::{RedisConfig, RedisPool};

let config = RedisConfig {
    url: "redis://localhost:6379".to_string(),
    timeout_ms: 5000,
    max_retries: 3,
    health_check_interval_secs: 30,
};

let mut pool = RedisPool::new(config).await?;
pool.health_check().await?;
```

#### HTTP Client
```rust
use riptide_utils::{HttpClientFactory, HttpConfig};

let client = HttpClientFactory::create_default()?;
// or with custom config
let config = HttpConfig {
    timeout_ms: 30000,
    ..Default::default()
};
let client = HttpClientFactory::create(config)?;
```

#### Retry Policy
```rust
use riptide_utils::RetryPolicy;

let policy = RetryPolicy::default();
let result = policy.execute(|| async {
    // Your async operation here
    Ok(42)
}).await?;
```

#### Rate Limiting
```rust
use riptide_utils::SimpleRateLimiter;

let limiter = SimpleRateLimiter::new(100); // 100 req/s
if limiter.check() {
    // Proceed with operation
}

// Or wait for permit
limiter.wait().await;
```

#### Time Utilities
```rust
use riptide_utils::time;

let now_secs = time::now_unix_secs();
let now_millis = time::now_unix_millis();

let dt = time::unix_secs_to_datetime(1609459200);
let formatted = time::format_iso8601(&dt.unwrap());
```

## Testing

Run tests:
```bash
cargo test -p riptide-utils
```

Run with clippy:
```bash
cargo clippy -p riptide-utils --all-targets -- -D warnings
```

## Test Coverage

- **40 unit tests** covering all modules
- **100% function coverage** for public APIs
- **Zero clippy warnings**
- **Zero compiler warnings** with `-D warnings`

## Dependencies

- `tokio`: Async runtime
- `redis`: Redis client
- `reqwest`: HTTP client
- `governor`: Rate limiting
- `chrono`: Time utilities
- `anyhow`, `thiserror`: Error handling
- `tracing`: Logging

## License

Apache-2.0
