# RipTide Reliability

**Infrastructure Layer - Circuit Breakers, Retries & Fault Tolerance**

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)

Reliability patterns and fault tolerance mechanisms for the RipTide web scraping framework, providing circuit breakers, intelligent gates, retry logic, and graceful degradation strategies.

## Quick Overview

RipTide Reliability is the infrastructure adapter that implements resilience patterns to prevent cascading failures and ensure system stability under adverse conditions.

**What it does:**
- Circuit breakers to prevent cascading failures
- Intelligent gates for extraction strategy routing
- Retry logic with exponential backoff
- Timeout handling and graceful degradation
- Reliability pattern implementations

**Port Implementation:**
- Implements `CircuitBreaker` from `riptide-types`
- Implements `RateLimiter` patterns
- Provides `ReliableHttpClient` wrapper for HTTP operations

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│               RipTide Reliability Layer                      │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐   │
│  │   Circuit   │  │   Intelligent│  │    Retry Logic   │   │
│  │   Breakers  │  │     Gates    │  │   & Timeouts     │   │
│  └─────────────┘  └──────────────┘  └──────────────────┘   │
│         │                 │                    │            │
│         └─────────────────┴────────────────────┘            │
│                           │                                 │
│                  ┌────────▼────────┐                        │
│                  │  Event Bus &    │                        │
│                  │    Monitoring   │                        │
│                  └─────────────────┘                        │
└──────────────────────────────────────────────────────────────┘
```

## Port Implementation

This adapter implements fault tolerance and resilience patterns as infrastructure adapters.

### `CircuitBreaker` Trait

```rust
#[async_trait]
pub trait CircuitBreaker: Send + Sync {
    async fn call<F, T, E>(&self, f: F) -> Result<T, E>
    where
        F: FnOnce() -> Result<T, E> + Send,
        E: From<CircuitBreakerError>;

    fn state(&self) -> CircuitState;
    fn record_success(&self);
    fn record_failure(&self);
}
```

### Why This Adapter Exists

The `riptide-reliability` adapter exists to:
1. **Prevent cascading failures** - Circuit breakers stop making requests to failing services
2. **Optimize resource usage** - Gates intelligently route requests to appropriate extraction methods
3. **Improve user experience** - Retry logic handles transient failures automatically
4. **Maintain system stability** - Timeout handling prevents resource exhaustion
5. **Enable observability** - Integration with monitoring and event systems

## Configuration

### Environment Variables

```bash
# Circuit breaker settings
CB_FAILURE_THRESHOLD=5
CB_SUCCESS_THRESHOLD=2
CB_OPEN_TIMEOUT_MS=30000
CB_HALF_OPEN_MAX_REQUESTS=3

# Gate thresholds
GATE_HI_THRESHOLD=0.7
GATE_LO_THRESHOLD=0.3

# Retry settings
RETRY_MAX_ATTEMPTS=3
RETRY_INITIAL_BACKOFF_MS=100
RETRY_MAX_BACKOFF_MS=10000
RETRY_BACKOFF_MULTIPLIER=2.0

# Timeout settings
DEFAULT_TIMEOUT_MS=30000
NAVIGATION_TIMEOUT_MS=30000
SCRIPT_TIMEOUT_MS=5000
```

### Programmatic Configuration

```rust
use riptide_reliability::circuit_breaker::{CircuitBreaker, Config, RealClock};
use std::sync::Arc;
use std::time::Duration;

let config = Config {
    failure_threshold: 5,
    success_threshold: 2,
    open_cooldown_ms: 30_000,
    half_open_max_in_flight: 3,
};

let circuit_breaker = CircuitBreaker::new(
    config,
    Arc::new(RealClock),
);
```

## Usage Examples

### Atomic Circuit Breaker (Lock-Free)

The atomic circuit breaker uses lock-free atomic operations for high-performance scenarios:

```rust
use riptide_reliability::circuit_breaker::{CircuitBreaker, Config, RealClock};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create circuit breaker
    let cb = CircuitBreaker::new(
        Config {
            failure_threshold: 5,      // Open after 5 failures
            success_threshold: 2,       // Close after 2 successes in half-open
            open_cooldown_ms: 30_000,   // Wait 30s before trying half-open
            half_open_max_in_flight: 3, // Max 3 concurrent requests in half-open
        },
        Arc::new(RealClock),
    );

    // Try to execute operation
    match cb.try_acquire() {
        Ok(permit) => {
            // Circuit is closed or half-open, proceed
            match perform_operation().await {
                Ok(result) => {
                    cb.on_success();
                    println!("Operation succeeded: {:?}", result);
                }
                Err(e) => {
                    cb.on_failure();
                    eprintln!("Operation failed: {}", e);
                }
            }
        }
        Err(msg) => {
            // Circuit is open, fail fast
            eprintln!("Circuit open: {}", msg);
        }
    }

    Ok(())
}

async fn perform_operation() -> anyhow::Result<String> {
    // Your operation here
    Ok("success".to_string())
}
```

### State-Based Circuit Breaker (with Events)

The state-based circuit breaker provides rich state tracking and event bus integration:

```rust
use riptide_reliability::circuit_breaker::{
    CircuitBreakerState, ExtractionResult, record_extraction_result
};
use riptide_pool::config::PerformanceMetrics;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = Arc::new(Mutex::new(CircuitBreakerState::default()));
    let metrics = Arc::new(Mutex::new(PerformanceMetrics::default()));

    // Record extraction result
    record_extraction_result(
        &metrics,
        &state,
        &None, // Optional event bus for notifications
        ExtractionResult {
            pool_id: "extraction-pool-1".to_string(),
            failure_threshold: 50,
            timeout_duration: 5000,
            success: true,
            duration: Duration::from_millis(250),
        },
    ).await;

    // Check circuit state
    let current_state = state.lock().await;
    println!("Circuit state: {:?}", current_state.state);
    println!("Failure count: {}", current_state.failure_count);
    println!("Success count: {}", current_state.success_count);

    Ok(())
}
```

### Intelligent Gates for Strategy Selection

Gates make intelligent routing decisions for extraction strategies:

```rust
use riptide_reliability::gate::{GateFeatures, decide, Decision};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Extract features from HTML
    let features = GateFeatures {
        html_bytes: 15_000,          // Total HTML size
        visible_text_chars: 8_000,    // Visible text content
        p_count: 20,                  // Number of <p> tags
        article_count: 1,             // Number of <article> tags
        has_og: true,                 // Open Graph meta tags present
        script_bytes: 800,            // JavaScript size
        spa_markers: 0,               // SPA framework markers
        h1h2_count: 5,                // H1/H2 heading count
        has_jsonld_article: true,     // JSON-LD article schema
        domain_prior: 0.75,           // Historical success rate for domain
    };

    // Make routing decision
    let hi_threshold = 0.7;
    let lo_threshold = 0.3;

    match decide(&features, hi_threshold, lo_threshold) {
        Decision::Raw => {
            println!("Use fast extraction (CSS/Regex)");
            // Proceed with fast extraction
        }
        Decision::ProbesFirst => {
            println!("Try fast first, fallback to headless if needed");
            // Try fast, use headless on failure
        }
        Decision::Headless => {
            println!("Use headless browser rendering");
            // Use browser-based extraction
        }
    }

    Ok(())
}
```

### Reliable HTTP Client with Circuit Breaker

Wrap HTTP client with circuit breaker and retry logic:

```rust
use riptide_reliability::http_client::ReliableHttpClient;
use riptide_fetch::fetch::FetchClient;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let base_client = FetchClient::new()?;

    let reliable_client = ReliableHttpClient::new(
        base_client,
        Config {
            max_retries: 3,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(10),
            timeout: Duration::from_secs(30),
        },
    );

    // Automatic retry with exponential backoff
    let response = reliable_client
        .get("https://example.com")
        .await?;

    println!("Response status: {}", response.status());

    Ok(())
}
```

### Retry Logic with Exponential Backoff

```rust
use riptide_reliability::retry::{retry_with_backoff, RetryConfig};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = RetryConfig {
        max_attempts: 3,
        initial_backoff: Duration::from_millis(100),
        max_backoff: Duration::from_secs(10),
        multiplier: 2.0,
        jitter: true,
    };

    let result = retry_with_backoff(config, || async {
        // Your fallible operation
        fetch_data().await
    }).await?;

    println!("Success: {:?}", result);

    Ok(())
}

async fn fetch_data() -> anyhow::Result<String> {
    // Operation that might fail
    Ok("data".to_string())
}
```

### Timeout Handling

```rust
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let result = timeout(
        Duration::from_secs(30),
        slow_operation()
    ).await;

    match result {
        Ok(Ok(data)) => {
            println!("Operation succeeded: {:?}", data);
        }
        Ok(Err(e)) => {
            eprintln!("Operation failed: {}", e);
        }
        Err(_) => {
            eprintln!("Operation timed out after 30s");
        }
    }

    Ok(())
}

async fn slow_operation() -> anyhow::Result<String> {
    // Potentially slow operation
    tokio::time::sleep(Duration::from_secs(5)).await;
    Ok("result".to_string())
}
```

## Technical Details

### External Dependencies

- **riptide-fetch**: HTTP client for reliable HTTP operations
- **riptide-events**: Event bus integration for circuit breaker state changes
- **riptide-monitoring**: Metrics collection for reliability patterns
- **tokio**: Async runtime for timeout handling
- **chrono**: Timestamp tracking for circuit breaker state

### Circuit Breaker State Machine

```
┌─────────────┐
│   CLOSED    │  Normal operation, all requests pass
└──────┬──────┘
       │ Failure threshold exceeded
       ▼
┌─────────────┐
│    OPEN     │  Fail fast, no requests allowed
└──────┬──────┘
       │ Cooldown period elapsed
       ▼
┌─────────────┐
│ HALF-OPEN   │  Limited requests to test recovery
└──────┬──────┘
       │
       ├─ Success threshold met ──────┐
       │                              │
       └─ Any failure ──┐             │
                        │             │
                        ▼             ▼
                  ┌─────────┐   ┌─────────┐
                  │  OPEN   │   │ CLOSED  │
                  └─────────┘   └─────────┘
```

### Gate Score Calculation

Gates use a scoring algorithm to route extraction requests:

```
Score = (
    visible_text_weight * visible_text_chars +
    structure_weight * (p_count + article_count + h1h2_count) +
    metadata_weight * (has_og + has_jsonld_article) +
    domain_prior_weight * domain_prior -
    spa_penalty * (script_bytes + spa_markers)
) / normalization_factor

if score > hi_threshold: Use Raw (fast extraction)
else if score > lo_threshold: Try Probes First (fast with fallback)
else: Use Headless (browser rendering)
```

### Retry Backoff Strategy

Exponential backoff with jitter prevents thundering herd:

```
backoff = min(
    initial_backoff * (multiplier ^ attempt),
    max_backoff
)

if jitter:
    backoff = backoff * random(0.5, 1.5)

wait(backoff)
```

### Performance Characteristics

| Pattern | Overhead | Benefit |
|---------|----------|---------|
| Circuit Breaker | <1ms per request | Prevents cascading failures |
| Gate Decision | <100μs | Optimizes extraction strategy |
| Retry (success on 1st) | 0ms | None |
| Retry (3 attempts) | ~500ms | Handles transient failures |
| Timeout | <1ms | Prevents resource exhaustion |

## Anti-Corruption Layer

The reliability adapter works with domain operations without leaking infrastructure concerns:

```rust
// Domain operation (generic)
async fn extract_content(url: &str) -> Result<String> {
    // Domain logic
}

// Wrapped with reliability patterns
async fn reliable_extract(url: &str, cb: &CircuitBreaker) -> Result<String> {
    match cb.try_acquire() {
        Ok(_permit) => {
            match extract_content(url).await {
                Ok(content) => {
                    cb.on_success();
                    Ok(content)
                }
                Err(e) => {
                    cb.on_failure();
                    Err(e)
                }
            }
        }
        Err(_) => {
            Err(anyhow::anyhow!("Circuit breaker open"))
        }
    }
}
```

## Testing

### Unit Tests

```bash
# Run all tests
cargo test -p riptide-reliability

# Run specific test module
cargo test -p riptide-reliability circuit_breaker::tests
cargo test -p riptide-reliability gate::tests

# Run with output
cargo test -p riptide-reliability -- --nocapture
```

### Integration Tests

```rust
// tests/integration_test.rs
use riptide_reliability::circuit_breaker::{CircuitBreaker, Config, RealClock};
use std::sync::Arc;

#[tokio::test]
async fn test_circuit_breaker_opens_on_failures() {
    let cb = CircuitBreaker::new(
        Config {
            failure_threshold: 3,
            success_threshold: 2,
            open_cooldown_ms: 1000,
            half_open_max_in_flight: 1,
        },
        Arc::new(RealClock),
    );

    // Record failures
    for _ in 0..3 {
        let _ = cb.try_acquire();
        cb.on_failure();
    }

    // Circuit should now be open
    assert!(cb.try_acquire().is_err());
}
```

### Mock Dependencies

For testing with mock event bus:

```rust
use mockall::mock;

mock! {
    pub EventBus {}

    #[async_trait]
    impl EventEmitter for EventBus {
        async fn emit(&self, event: Box<dyn Event>);
    }
}

// Use MockEventBus in tests
let mock_bus = MockEventBus::new();
mock_bus.expect_emit()
    .times(1)
    .returning(|_| ());
```

## Error Handling

### Error Types

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReliabilityError {
    #[error("Circuit breaker open")]
    CircuitBreakerOpen,

    #[error("Operation timed out after {0}s")]
    Timeout(u64),

    #[error("Retry exhausted after {0} attempts")]
    RetryExhausted(usize),

    #[error("Gate decision failed: {0}")]
    GateError(String),
}
```

### Retry Strategies

Different retry strategies for different error types:

```rust
use riptide_reliability::retry::RetryStrategy;

// Retry on transient errors only
let strategy = RetryStrategy::TransientOnly;

// Retry on all errors
let strategy = RetryStrategy::All;

// Custom retry predicate
let strategy = RetryStrategy::Custom(Box::new(|error| {
    matches!(error, MyError::NetworkTimeout | MyError::ServerError)
}));
```

### Recovery Patterns

When circuit breaker opens:

1. **Fail Fast**: Return error immediately to prevent resource exhaustion
2. **Fallback**: Use alternative strategy (e.g., cached data)
3. **Degrade**: Reduce functionality but maintain service
4. **Monitor**: Emit events for observability

## Production Considerations

### Resource Limits

- **Circuit Breakers**: Minimal memory overhead (~100 bytes per breaker)
- **Gates**: Stateless, no memory overhead
- **Retry Logic**: Memory proportional to retry attempts (negligible)

### Monitoring and Metrics

```rust
// Monitor circuit breaker state changes
let events = circuit_breaker.state_changes();

tokio::spawn(async move {
    while let Some(event) = events.recv().await {
        match event {
            StateChange::Opened => {
                warn!("Circuit breaker opened");
                // Alert operations team
            }
            StateChange::HalfOpen => {
                info!("Circuit breaker half-open, testing recovery");
            }
            StateChange::Closed => {
                info!("Circuit breaker closed, service recovered");
            }
        }
    }
});
```

### Failure Modes

**Circuit Breaker Never Opens:**
- `failure_threshold` too high → Lower threshold
- Failures not recorded → Ensure `on_failure()` is called
- State not shared → Use Arc wrapper

**Circuit Breaker Stuck Open:**
- `open_cooldown_ms` too long → Reduce cooldown
- Half-open requests failing → Check service health
- `success_threshold` too high → Lower threshold

**Gate Always Routes to Headless:**
- `hi_threshold` too high → Lower to 0.6-0.7
- Feature extraction incorrect → Validate HTML parsing
- Domain prior too low → Check historical data

## Dependencies

### External Systems Required

None (self-contained, no external services)

### Rust Crate Dependencies

| Dependency | Purpose |
|------------|---------|
| riptide-fetch | HTTP client operations |
| riptide-events | Event bus integration |
| riptide-monitoring | Metrics collection |
| riptide-utils | Circuit breaker utilities |
| tokio | Async runtime |
| chrono | Timestamp tracking |
| thiserror | Error definitions |
| tracing | Structured logging |

## Feature Flags

```toml
[dependencies]
riptide-reliability = { version = "0.9", features = ["full"] }
```

**Available Features:**
- `events` (default): Event bus integration
- `monitoring` (default): Metrics collection
- `reliability-patterns` (default): Full pattern implementations
- `full`: All features enabled

## Performance Tips

1. **Use atomic circuit breaker** for high-throughput scenarios (lock-free)
2. **Tune thresholds** based on actual failure rates
3. **Enable monitoring** to track pattern effectiveness
4. **Adjust retry limits** to balance latency vs reliability
5. **Use gates** to optimize resource usage (fast extraction when possible)
6. **Set appropriate timeouts** to prevent resource exhaustion

## Related Crates

- **riptide-fetch**: HTTP client for reliable operations
- **riptide-pool**: Resource pooling (uses circuit breakers)
- **riptide-events**: Event system integration
- **riptide-monitoring**: Metrics and observability
- **riptide-utils**: Shared utility functions

## License

Apache-2.0
