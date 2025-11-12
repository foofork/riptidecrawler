# 🛠️ RipTide Utils - Shared Infrastructure Utilities

**Category:** Infrastructure & Utilities
**Purpose:** Common utilities for HTTP clients, retry logic, time handling, and circuit breakers

## Quick Overview

`riptide-utils` provides battle-tested infrastructure utilities used across the RipTide platform. It offers HTTP client factories, retry policies with exponential backoff, time utilities, circuit breakers, and health monitoring - all the plumbing needed for reliable distributed systems.

## Why This Exists

Every service needs:
- HTTP clients with connection pooling
- Retry logic with backoff
- Time utilities for timestamps
- Circuit breakers for fault tolerance
- Health monitoring

Rather than duplicate this logic, `riptide-utils` provides production-ready implementations with comprehensive testing.

## Key Features

- **HTTP Client Factory**: Pre-configured HTTP clients with connection pooling
- **Retry Policy**: Exponential backoff with configurable max attempts
- **Time Utilities**: Unix timestamps, ISO 8601 formatting, duration calculations
- **Circuit Breaker**: Fault tolerance with automatic recovery
- **Health Registry**: Health check management for services
- **Error Handling**: Common error types and result aliases

## Quick Start

```rust
use riptide_utils::{HttpClientFactory, RetryPolicy, time};

// Create HTTP client
let client = HttpClientFactory::create_default()?;

// Use retry policy
let policy = RetryPolicy::default();
let result = policy.execute(|| async {
    client.get("https://example.com").send().await
}).await?;

// Get current timestamp
let now = time::now_unix_secs();
println!("Current timestamp: {}", now);
```

## Core Utilities

### HTTP Client Factory

Create configured HTTP clients with connection pooling:

```rust
use riptide_utils::{HttpClientFactory, HttpConfig};
use std::time::Duration;

// Use default client
let client = HttpClientFactory::create_default()?;

// Or customize
let config = HttpConfig {
    timeout: Duration::from_secs(30),
    connect_timeout: Duration::from_secs(10),
    pool_idle_timeout: Duration::from_secs(90),
    pool_max_idle_per_host: 10,
    user_agent: Some("MyApp/1.0".to_string()),
};

let client = HttpClientFactory::create(config)?;

// Make requests
let response = client
    .get("https://example.com")
    .send()
    .await?;

println!("Status: {}", response.status());
```

### Retry Policy

Automatic retries with exponential backoff:

```rust
use riptide_utils::RetryPolicy;
use std::time::Duration;

// Use default policy (3 attempts, exponential backoff)
let policy = RetryPolicy::default();

// Or customize
let policy = RetryPolicy::new(5, Duration::from_secs(1), 2.0, Some(Duration::from_secs(30)));

// Execute with retries
let result = policy.execute(|| async {
    // Your fallible async operation
    risky_operation().await
}).await?;

// Manual retry loop
for attempt in 0..policy.max_attempts {
    match risky_operation().await {
        Ok(result) => return Ok(result),
        Err(e) if attempt == policy.max_attempts - 1 => return Err(e),
        Err(_) => {
            let backoff = policy.calculate_backoff(attempt);
            tokio::time::sleep(backoff).await;
        }
    }
}
```

### Time Utilities

Unix timestamps and date/time conversions:

```rust
use riptide_utils::time;
use chrono::{DateTime, Utc};

// Get current timestamps
let now_secs = time::now_unix_secs();
let now_millis = time::now_unix_millis();
let now_micros = time::now_unix_micros();

println!("Unix seconds: {}", now_secs);
println!("Unix millis: {}", now_millis);

// Convert timestamp to DateTime
let dt = time::unix_secs_to_datetime(1609459200)?;
println!("DateTime: {}", dt);

// Format as ISO 8601
let formatted = time::format_iso8601(&dt);
println!("ISO 8601: {}", formatted);

// Parse ISO 8601
let parsed = time::parse_iso8601("2021-01-01T00:00:00Z")?;

// Calculate duration
let start = time::now_unix_secs();
// ... do work ...
let elapsed = time::now_unix_secs() - start;
println!("Elapsed: {} seconds", elapsed);

// Check expiration
let expires_at = time::now_unix_secs() + 3600; // 1 hour from now
if time::is_expired(expires_at) {
    println!("Expired!");
}
```

### Circuit Breaker

Fault tolerance with automatic recovery:

```rust
use riptide_utils::{CircuitBreaker, CircuitState, Config as CircuitConfig};
use std::time::Duration;

// Create circuit breaker
let config = CircuitConfig {
    failure_threshold: 5,
    success_threshold: 2,
    timeout: Duration::from_secs(60),
};

let circuit_breaker = CircuitBreaker::new(config);

// Use circuit breaker
let result = match circuit_breaker.call(|| async {
    fallible_operation().await
}).await {
    Ok(value) => value,
    Err(e) => {
        println!("Circuit breaker error: {}", e);
        return Err(e);
    }
};

// Check circuit state
match circuit_breaker.state() {
    CircuitState::Closed => println!("Circuit closed (normal)"),
    CircuitState::Open => println!("Circuit open (failing)"),
    CircuitState::HalfOpen => println!("Circuit half-open (testing)"),
}

// Manual state management
circuit_breaker.reset(); // Force close the circuit
```

### Guarded Call Helper

Simplified circuit breaker usage:

```rust
use riptide_utils::guarded_call;

// Single call with default config
let result = guarded_call(|| async {
    risky_operation().await
}).await?;
```

### Health Registry

Manage health checks for distributed services:

```rust
use riptide_utils::{InMemoryHealthRegistry, SimpleHealthCheck};

// Create registry
let registry = InMemoryHealthRegistry::new();

// Register health checks
registry.register(
    "database",
    Box::new(SimpleHealthCheck::new(|| {
        // Check database connection
        database.ping().is_ok()
    }))
).await;

registry.register(
    "cache",
    Box::new(SimpleHealthCheck::new(|| {
        // Check cache connection
        cache.ping().is_ok()
    }))
).await;

// Check overall health
let health = registry.check_health().await;
if health.is_healthy {
    println!("All services healthy");
} else {
    println!("Unhealthy services: {:?}", health.failed_checks);
}
```

## Usage Examples

### Combining Utilities

```rust
use riptide_utils::{HttpClientFactory, RetryPolicy, CircuitBreaker};

// Create components
let client = HttpClientFactory::create_default()?;
let retry = RetryPolicy::default();
let circuit = CircuitBreaker::default();

// Combine for robust HTTP calls
let result = circuit.call(|| async {
    retry.execute(|| async {
        client
            .get("https://api.example.com/data")
            .send()
            .await
    }).await
}).await?;
```

### Time-based Cache Key

```rust
use riptide_utils::time;

fn cache_key_for_hour(base: &str) -> String {
    let hour = time::now_unix_secs() / 3600;
    format!("{}:{}", base, hour)
}

let key = cache_key_for_hour("api_response");
println!("Cache key: {}", key);
```

### HTTP Client with Custom Headers

```rust
use riptide_utils::HttpClientFactory;
use reqwest::header;

let client = HttpClientFactory::create_default()?;

let response = client
    .get("https://api.example.com")
    .header(header::AUTHORIZATION, "Bearer token123")
    .header(header::ACCEPT, "application/json")
    .send()
    .await?;

if response.status().is_success() {
    let body = response.text().await?;
    println!("Response: {}", body);
}
```

## API Reference

### HTTP Module

- `HttpClientFactory` - Factory for creating HTTP clients
- `HttpConfig` - HTTP client configuration
  - `timeout: Duration` - Overall request timeout
  - `connect_timeout: Duration` - Connection timeout
  - `pool_idle_timeout: Duration` - Connection pool idle timeout
  - `pool_max_idle_per_host: usize` - Max idle connections per host
  - `user_agent: Option<String>` - Custom User-Agent header

### Retry Module

- `RetryPolicy` - Exponential backoff retry policy
  - `new(max_attempts, initial_delay, multiplier, max_delay)` - Create custom policy
  - `default()` - Default policy (3 attempts, 1s initial, 2x multiplier)
  - `execute<F, T, E>(f: F)` - Execute function with retries
  - `calculate_backoff(attempt)` - Calculate backoff duration

### Time Module

- `now_unix_secs()` - Current Unix timestamp (seconds)
- `now_unix_millis()` - Current Unix timestamp (milliseconds)
- `now_unix_micros()` - Current Unix timestamp (microseconds)
- `unix_secs_to_datetime(secs)` - Convert Unix seconds to DateTime
- `format_iso8601(dt)` - Format DateTime as ISO 8601
- `parse_iso8601(s)` - Parse ISO 8601 string
- `is_expired(timestamp)` - Check if timestamp has passed

### Circuit Breaker Module

- `CircuitBreaker` - Circuit breaker implementation
- `Config` - Circuit breaker configuration
- `CircuitState` - Circuit state (Closed, Open, HalfOpen)
- `guarded_call<F, T>(f: F)` - Helper for single guarded calls

### Health Registry Module

- `InMemoryHealthRegistry` - In-memory health check registry
- `SimpleHealthCheck` - Simple boolean health check

### Error Module

- `Error` - Common error type
- `Result<T>` - Common result alias

## Integration with Other Crates

### Used By

- **riptide-fetch**: HTTP client for fetching web pages
- **riptide-reliability**: Circuit breakers and retry logic
- **riptide-api**: HTTP server health checks
- **riptide-workers**: Background job retry logic
- **riptide-cache**: Connection pooling and time utilities

### Example Integration

```rust
// In riptide-fetch
use riptide_utils::{HttpClientFactory, RetryPolicy};

pub struct WebFetcher {
    client: reqwest::Client,
    retry: RetryPolicy,
}

impl WebFetcher {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: HttpClientFactory::create_default()?,
            retry: RetryPolicy::default(),
        })
    }

    pub async fn fetch(&self, url: &str) -> Result<String> {
        self.retry.execute(|| async {
            self.client
                .get(url)
                .send()
                .await?
                .text()
                .await
        }).await
    }
}
```

## Testing

```bash
# Run all tests
cargo test -p riptide-utils

# Test specific module
cargo test -p riptide-utils http
cargo test -p riptide-utils retry
cargo test -p riptide-utils time

# Test with output
cargo test -p riptide-utils -- --nocapture

# Run benchmarks
cargo bench -p riptide-utils
```

### Example Tests

```rust
#[tokio::test]
async fn test_retry_policy() {
    let policy = RetryPolicy::new(3, Duration::from_millis(10), 2.0, None);

    let mut attempts = 0;
    let result = policy.execute(|| async {
        attempts += 1;
        if attempts < 3 {
            Err(anyhow::anyhow!("Not yet"))
        } else {
            Ok(42)
        }
    }).await;

    assert!(result.is_ok());
    assert_eq!(attempts, 3);
}

#[test]
fn test_time_utilities() {
    let now = time::now_unix_secs();
    assert!(now > 1600000000); // After 2020

    let dt = time::unix_secs_to_datetime(1609459200).unwrap();
    let formatted = time::format_iso8601(&dt);
    assert!(formatted.contains("2021-01-01"));
}
```

## Best Practices

1. **HTTP Clients**: Reuse clients to leverage connection pooling
2. **Retry Logic**: Use exponential backoff to avoid overwhelming services
3. **Circuit Breakers**: Fail fast to prevent cascading failures
4. **Time Utilities**: Use Unix timestamps for consistency across time zones
5. **Error Handling**: Use the provided `Result` type for consistency

## Performance Considerations

- **HTTP Client**: Connection pooling significantly reduces latency
- **Retry Policy**: Exponential backoff prevents thundering herd
- **Circuit Breaker**: Opens circuit after threshold to fail fast
- **Time Utilities**: Lightweight, inline functions with minimal overhead

## Troubleshooting

### HTTP Client Timeouts

```rust
// Increase timeouts for slow endpoints
let config = HttpConfig {
    timeout: Duration::from_secs(120),
    connect_timeout: Duration::from_secs(30),
    ..Default::default()
};
```

### Too Many Retries

```rust
// Reduce retry attempts or increase delays
let policy = RetryPolicy::new(2, Duration::from_secs(5), 3.0, None);
```

### Circuit Breaker Too Sensitive

```rust
// Increase failure threshold
let config = CircuitConfig {
    failure_threshold: 10, // More failures before opening
    success_threshold: 3,  // More successes before closing
    timeout: Duration::from_secs(120), // Longer recovery time
};
```

## Dependencies

- `tokio` - Async runtime
- `reqwest` - HTTP client
- `chrono` - Date/time handling
- `anyhow` - Error handling
- `thiserror` - Error derive macros
- `tracing` - Logging

## Migration from riptide-core

**Note**: Redis functionality has been moved to `riptide-cache`. Rate limiting has been moved to specialized crates (`riptide-stealth`, `riptide-api`). For generic rate limiting, use the `governor` crate directly.

## License

Apache-2.0

## Related Crates

- `riptide-reliability` - Advanced circuit breakers and fault tolerance
- `riptide-cache` - Redis connection pooling (formerly in this crate)
- `riptide-fetch` - Uses HTTP utilities
- `riptide-workers` - Uses retry logic
