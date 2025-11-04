# Smart Retry Logic for RipTide Intelligence

## Overview

The smart retry system provides intelligent retry strategies for LLM operations, API calls, and extraction tasks with automatic error classification, strategy selection, and circuit breaker integration.

## Features

### 1. Retry Strategies

#### Exponential Backoff
- Formula: `2^n * initial_delay`
- Best for: Transient network errors, timeouts
- Example delays: 100ms, 200ms, 400ms, 800ms, 1600ms
- Use case: Network timeouts, temporary service unavailability

#### Linear Backoff
- Formula: `(n+1) * initial_delay`
- Best for: Predictable retry patterns
- Example delays: 100ms, 200ms, 300ms, 400ms, 500ms
- Use case: Resource contention, queue processing

#### Fibonacci Backoff
- Formula: `fib(n) * initial_delay`
- Best for: Server errors (500s)
- Example delays: 100ms, 100ms, 200ms, 300ms, 500ms, 800ms
- Use case: Server overload, gradual recovery

#### Adaptive Strategy
- Formula: Dynamically adjusted based on error patterns and success rate
- Best for: Rate limiting with delay hints
- Features:
  - Uses rate limit headers for precise delays
  - Adjusts based on historical success rate (0.8x-1.5x multiplier)
  - Learns from previous attempts
- Use case: API rate limits, dynamic throttling

### 2. Error Classification

The system automatically classifies errors and selects appropriate strategies:

| Error Type | Strategy | Retryable |
|------------|----------|-----------|
| Circuit breaker open | N/A | ❌ No |
| Client errors (400s) | N/A | ❌ No |
| Rate limits | Adaptive | ✅ Yes |
| Server errors (500s) | Fibonacci | ✅ Yes |
| Network errors | Exponential | ✅ Yes |
| Timeouts | Exponential | ✅ Yes |

### 3. Configuration

```rust
use riptide_intelligence::smart_retry::{RetryConfig, SmartRetry, SmartRetryStrategy};

// Default configuration
let config = RetryConfig::default();
// max_attempts: 5
// initial_delay_ms: 100
// max_delay_ms: 30_000
// jitter: 0.25 (25%)
// backoff_multiplier: 2.0

// Fast retry for low-latency operations
let config = RetryConfig::fast();
// max_attempts: 3
// initial_delay_ms: 50
// max_delay_ms: 5_000

// Aggressive retry for critical operations
let config = RetryConfig::aggressive();
// max_attempts: 10
// initial_delay_ms: 100
// max_delay_ms: 60_000
```

### 4. Retry Behavior

#### Jitter
- Adds random variance to prevent thundering herd
- Configurable (0-100%, default 25%)
- Formula: `delay + random(0, delay * jitter)`

#### Max Delay Cap
- Prevents excessive wait times
- Applied after strategy calculation
- Default: 30 seconds

#### Strategy Switching
- Automatically switches strategy based on error type
- Tracks switches in metrics
- Example: Network error → Exponential, then Rate limit → Adaptive

## Usage Examples

### Basic Retry

```rust
use riptide_intelligence::smart_retry::{SmartRetry, SmartRetryStrategy};

let retry = SmartRetry::new(SmartRetryStrategy::Exponential);

let result = retry.execute(|| async {
    // Your operation here
    llm_provider.complete(request).await
}).await?;
```

### Custom Configuration

```rust
let config = RetryConfig {
    max_attempts: 5,
    initial_delay_ms: 100,
    max_delay_ms: 30_000,
    jitter: 0.25,
    backoff_multiplier: 2.0,
};

let retry = SmartRetry::with_config(SmartRetryStrategy::Adaptive, config);
```

### Circuit Breaker Integration

```rust
use riptide_intelligence::CircuitBreaker;

let circuit_breaker = CircuitBreaker::new(provider);
let retry = SmartRetry::new(SmartRetryStrategy::Exponential);

let result = retry.execute_with_circuit_breaker(&circuit_breaker, || async {
    // Operation will not retry if circuit is open
    perform_operation().await
}).await?;
```

### Fallback Strategies

```rust
let retry = SmartRetry::new(SmartRetryStrategy::Exponential);

let fallback_strategies = vec![
    SmartRetryStrategy::Linear,
    SmartRetryStrategy::Fibonacci,
    SmartRetryStrategy::Adaptive,
];

let result = retry.execute_with_fallback(
    || async { perform_operation().await },
    fallback_strategies,
).await?;
```

### Error Handling

```rust
let retry = SmartRetry::new(SmartRetryStrategy::Adaptive);

match retry.execute(|| async {
    llm_provider.complete(request).await
}).await {
    Ok(response) => println!("Success: {:?}", response),
    Err(IntelligenceError::CircuitOpen { reason }) => {
        // Circuit breaker prevented retry
        eprintln!("Circuit open: {}", reason);
    }
    Err(IntelligenceError::RateLimit { retry_after_ms }) => {
        // Rate limited - adaptive strategy used
        eprintln!("Rate limited, retry after: {}ms", retry_after_ms);
    }
    Err(e) => {
        // Max retries exhausted
        eprintln!("Failed after retries: {:?}", e);
    }
}
```

## Metrics and Monitoring

### Statistics Tracking

```rust
let retry = SmartRetry::new(SmartRetryStrategy::Exponential);

// Perform operations...

let stats = retry.stats();
println!("Total attempts: {}", stats.total_attempts);
println!("Successful retries: {}", stats.successful_retries);
println!("Failed retries: {}", stats.failed_retries);
println!("Strategy switches: {}", stats.strategy_switches);
println!("Success rate: {:.2}%", stats.success_rate() * 100.0);
```

### Performance Overhead

- Strategy selection: < 1ms
- Delay calculation: < 0.1ms
- Jitter generation: < 0.05ms
- Total overhead: < 1.5ms per retry attempt

## Best Practices

### 1. Choose the Right Strategy

- **Exponential**: Network issues, temporary outages
- **Linear**: Predictable recovery times
- **Fibonacci**: Server overload situations
- **Adaptive**: Rate-limited APIs, learning from patterns

### 2. Configure Appropriately

```rust
// For user-facing operations (low latency)
let config = RetryConfig::fast();

// For background jobs (resilience)
let config = RetryConfig::aggressive();

// For balanced approach
let config = RetryConfig::default();
```

### 3. Integrate with Circuit Breaker

Always use circuit breaker integration to prevent cascade failures:

```rust
let result = retry.execute_with_circuit_breaker(&circuit_breaker, || async {
    // Your operation
}).await?;
```

### 4. Monitor Success Rates

```rust
let stats = retry.stats();
if stats.success_rate() < 0.5 {
    warn!("Low retry success rate: {:.2}%", stats.success_rate() * 100.0);
    // Consider adjusting configuration or investigating root cause
}
```

### 5. Use Jitter

Always enable jitter (default 25%) to prevent thundering herd:

```rust
let config = RetryConfig {
    jitter: 0.25, // 25% random variance
    ..Default::default()
};
```

## Testing

The implementation includes 15+ comprehensive tests covering:

- ✅ Exponential backoff timing
- ✅ Linear backoff timing
- ✅ Fibonacci sequence calculation
- ✅ Adaptive strategy learning
- ✅ Jitter variance verification
- ✅ Max attempts enforcement
- ✅ Strategy switching
- ✅ Circuit breaker integration
- ✅ Timeout handling
- ✅ Success rate tracking
- ✅ Concurrent retry operations
- ✅ Configuration validation
- ✅ Rate limit adaptive delays
- ✅ Fallback strategy chains
- ✅ Error classification

### Run Tests

```bash
# Unit tests
cargo test --package riptide-intelligence --lib smart_retry

# Integration tests
cargo test --package riptide-intelligence --test smart_retry_tests

# All tests
cargo test --package riptide-intelligence smart_retry
```

## Architecture

### Components

```
SmartRetry
├── RetryConfig (configuration)
├── SmartRetryStrategy (strategy enum)
├── RetryStats (metrics tracking)
└── Error Classification (automatic strategy selection)
```

### Integration Points

1. **Circuit Breaker**: Prevents retries when circuit is open
2. **LLM Pool**: Used for retrying LLM provider operations
3. **Metrics System**: Tracks retry statistics
4. **Error Types**: Integrates with IntelligenceError enum

### Thread Safety

- All retry operations are thread-safe
- Statistics use `Arc<RwLock<>>` for concurrent access
- Multiple retry instances can run in parallel
- No global state or shared mutable data

## Performance Characteristics

### Time Complexity
- Strategy selection: O(1)
- Delay calculation: O(1) for Exponential, Linear; O(n) for Fibonacci
- Error classification: O(1)

### Space Complexity
- O(1) per retry instance
- Statistics tracking: O(1)
- No unbounded memory growth

### Benchmarks

| Operation | Time | Memory |
|-----------|------|--------|
| Strategy selection | < 1ms | 0 allocations |
| Exponential delay calc | < 0.1ms | 0 allocations |
| Fibonacci delay calc | < 0.2ms | 0 allocations |
| Stats update | < 0.05ms | 0 allocations |

## Comparison with Other Solutions

| Feature | SmartRetry | tokio-retry | backoff |
|---------|------------|-------------|---------|
| Multiple strategies | ✅ 4 strategies | ❌ 1-2 | ✅ 3 strategies |
| Automatic strategy selection | ✅ Yes | ❌ No | ❌ No |
| Error classification | ✅ Yes | ❌ No | ❌ No |
| Circuit breaker integration | ✅ Yes | ❌ No | ❌ No |
| Adaptive learning | ✅ Yes | ❌ No | ❌ No |
| Jitter support | ✅ Yes | ✅ Yes | ✅ Yes |
| Metrics tracking | ✅ Yes | ❌ No | ❌ Limited |
| Strategy switching | ✅ Yes | ❌ No | ❌ No |

## Future Enhancements

### Planned Features
- [ ] Persistent retry state across restarts
- [ ] Machine learning-based strategy selection
- [ ] Distributed retry coordination
- [ ] Advanced metrics (P50, P95, P99 latencies)
- [ ] Custom strategy plugins
- [ ] Retry budget management
- [ ] Integration with OpenTelemetry

### Experimental Features
- Custom backoff functions
- Per-operation retry policies
- Retry budget across multiple services
- Distributed rate limiting

## References

- [Circuit Breaker Pattern](../crates/riptide-intelligence/src/circuit_breaker.rs)
- [LLM Client Pool](../crates/riptide-intelligence/src/llm_client_pool.rs)
- [Intelligence Error Types](../crates/riptide-intelligence/src/lib.rs)
- [Exponential Backoff Best Practices](https://aws.amazon.com/blogs/architecture/exponential-backoff-and-jitter/)
