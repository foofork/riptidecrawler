# LLM Integration Guide for Background AI Processor

## Overview

The Background AI Processor now features full integration with the LLM client pool, providing production-ready AI content enhancement with:

- **Multi-provider support** via `LlmRegistry`
- **Automatic failover** with `FailoverManager`
- **Rate limiting** to respect API quotas
- **Exponential backoff** for transient errors
- **Circuit breaker** integration for fault tolerance
- **Concurrent processing** with configurable worker pools

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│         BackgroundAiProcessor                            │
│                                                          │
│  ┌──────────────┐    ┌──────────────┐                  │
│  │ Task Queue   │───▶│  Workers     │                  │
│  │ (Priority)   │    │  (4 default) │                  │
│  └──────────────┘    └──────┬───────┘                  │
│                              │                           │
│                              ▼                           │
│                      ┌──────────────┐                   │
│                      │ Rate Limiter │                   │
│                      └──────┬───────┘                   │
│                              │                           │
│                              ▼                           │
│                      ┌──────────────┐                   │
│                      │LLM Integration│                  │
│                      └──────┬───────┘                   │
│                              │                           │
│              ┌───────────────┴───────────────┐          │
│              ▼                               ▼          │
│      ┌──────────────┐               ┌──────────────┐   │
│      │FailoverMgr   │               │  LlmRegistry │   │
│      │(HA/Fallback) │               │  (Providers) │   │
│      └──────────────┘               └──────────────┘   │
└─────────────────────────────────────────────────────────┘
```

## Configuration

### Basic Setup

```rust
use riptide_intelligence::{
    AiProcessorConfig, BackgroundAiProcessor, LlmRegistry,
};
use std::sync::Arc;
use std::time::Duration;

// Create configuration
let config = AiProcessorConfig {
    num_workers: 4,
    queue_size: 1000,
    max_concurrent_requests: 10,
    worker_timeout: Duration::from_secs(60),
    stream_results: true,

    // LLM-specific settings
    llm_model: "gpt-3.5-turbo".to_string(),
    max_tokens: 2048,
    temperature: 0.7,

    // Rate limiting
    rate_limit_rps: 10.0,  // 10 requests per second

    // Exponential backoff
    initial_backoff: Duration::from_millis(100),
    max_backoff: Duration::from_secs(30),
    backoff_multiplier: 2.0,
};

// Create LLM registry and register providers
let registry = Arc::new(LlmRegistry::new());
// ... register your providers ...

// Create processor with LLM integration
let mut processor = BackgroundAiProcessor::new(config)
    .with_llm_registry(registry);

// Start processing
processor.start().await?;
```

### Advanced Setup with Failover

```rust
use riptide_intelligence::{
    AiProcessorConfig, BackgroundAiProcessor,
    FailoverConfig, FailoverManager, HealthMonitor,
    LlmRegistry, ProviderPriority,
};

// Setup health monitoring
let health_monitor = Arc::new(HealthMonitor::builder()
    .check_interval(Duration::from_secs(30))
    .build());

// Create failover manager
let failover_config = FailoverConfig {
    strategy: FailoverStrategy::LeastLatency,
    max_retries: 3,
    retry_delay: Duration::from_millis(500),
    failback_delay: Duration::from_secs(60),
    health_check_threshold: 3,
    circuit_breaker_enabled: true,
    load_balancing_enabled: true,
};

let (failover_mgr, _events) = FailoverManager::new(
    failover_config,
    health_monitor.clone(),
);
let failover_mgr = Arc::new(failover_mgr);

// Register providers with priorities
let openai_provider = Arc::new(OpenAIProvider::new(api_key));
failover_mgr.add_provider(
    openai_provider,
    ProviderPriority {
        name: "openai".to_string(),
        priority: 1,
        weight: 1.0,
        max_concurrent_requests: 10,
        enabled: true,
    },
).await?;

// Create processor with full integration
let mut processor = BackgroundAiProcessor::new(config)
    .with_llm_registry(registry)
    .with_llm_failover(failover_mgr);

processor.start().await?;
```

## Usage

### Queuing Tasks

```rust
use riptide_intelligence::{AiTask, TaskPriority};

// Create an AI enhancement task
let task = AiTask::new(
    "https://example.com/article".to_string(),
    "Article content here...".to_string(),
)
.with_priority(TaskPriority::High)
.with_timeout(Duration::from_secs(30));

// Queue the task
processor.queue_task(task).await?;
```

### Retrieving Results

```rust
// Non-blocking result retrieval
if let Some(result) = processor.try_recv_result().await {
    if result.success {
        println!("Enhanced content: {}", result.enhanced_content.unwrap());
    } else {
        eprintln!("Enhancement failed: {}", result.error.unwrap());
    }
}

// Retrieve all pending results
let results = processor.recv_all_results().await;
for result in results {
    // Process results...
}
```

### Monitoring

```rust
// Get processor statistics
let stats = processor.stats().await;
println!("Queue size: {}", stats.queue_size);
println!("Active workers: {}", stats.active_workers);
println!("Total workers: {}", stats.total_workers);
println!("Is running: {}", stats.is_running);
```

## Features

### 1. Rate Limiting

The processor includes built-in rate limiting to respect API quotas:

- Configurable requests per second (`rate_limit_rps`)
- Token bucket algorithm with minimum interval enforcement
- Automatic request spacing to prevent rate limit errors

### 2. Exponential Backoff

Handles transient failures with exponential backoff:

- Configurable initial backoff duration
- Configurable maximum backoff duration
- Configurable backoff multiplier
- Respects `retry_after` headers from rate limit responses

### 3. Error Handling

Comprehensive error handling for all failure modes:

- **Rate Limits**: Automatic retry with proper delay
- **Circuit Breaker**: Detects and waits for circuit to close
- **Network Errors**: Retry with exponential backoff
- **Provider Errors**: Retry with backoff or fail
- **Timeouts**: Configurable per-task timeouts

### 4. High Availability

With `FailoverManager` integration:

- Automatic provider failover on errors
- Health-based provider selection
- Multiple failover strategies (priority, round-robin, least-latency, etc.)
- Circuit breaker integration per provider

### 5. Performance Optimization

- Work-stealing queue for load balancing
- Configurable worker pool size
- Semaphore-based concurrency control
- Async/await throughout for efficiency

## Error Recovery Strategies

### Transient Errors (Retried)

1. **Rate Limit Exceeded**
   - Waits for `retry_after_ms` from API
   - Does not count against retry budget

2. **Circuit Breaker Open**
   - Waits with backoff
   - Attempts fallback providers if available

3. **Network Errors**
   - Exponential backoff retry
   - Up to `max_retries` attempts

4. **Timeout**
   - Recorded in metrics
   - Not retried (fails immediately)

### Non-Retryable Errors

1. **Invalid Request**
   - Configuration error
   - Fails immediately

2. **All Providers Failed**
   - No providers available
   - Fails immediately

3. **Provider Configuration Error**
   - Registry not configured
   - Falls back to placeholder (warning logged)

## Best Practices

### 1. Choose Appropriate Rate Limits

```rust
// For OpenAI GPT-3.5
rate_limit_rps: 3.0  // Conservative, allows for burst

// For high-tier accounts
rate_limit_rps: 10.0  // More aggressive
```

### 2. Configure Retry Strategy

```rust
// For critical content
max_retries: 5,
initial_backoff: Duration::from_millis(100),
max_backoff: Duration::from_secs(60),

// For best-effort enhancement
max_retries: 2,
initial_backoff: Duration::from_millis(500),
max_backoff: Duration::from_secs(10),
```

### 3. Monitor Queue Depth

```rust
// Periodically check queue depth
let stats = processor.stats().await;
if stats.queue_size > config.queue_size * 80 / 100 {
    warn!("Queue is 80% full, consider scaling workers");
}
```

### 4. Use Task Priorities

```rust
// High priority for user-facing content
let urgent_task = AiTask::new(url, content)
    .with_priority(TaskPriority::Critical);

// Normal priority for background enhancement
let background_task = AiTask::new(url, content)
    .with_priority(TaskPriority::Normal);
```

### 5. Handle Graceful Shutdown

```rust
// Stop accepting new tasks
processor.stop().await?;

// Wait for in-flight tasks to complete
tokio::time::sleep(Duration::from_secs(5)).await;

// Retrieve any remaining results
let final_results = processor.recv_all_results().await;
```

## Migration from Placeholder

If you're migrating from the placeholder implementation:

### Before (Placeholder)
```rust
let processor = BackgroundAiProcessor::new(config);
// No LLM integration, just simulated delays
```

### After (Full LLM Integration)
```rust
// Setup registry
let registry = Arc::new(setup_llm_registry()?);

// Create processor with LLM
let processor = BackgroundAiProcessor::new(config)
    .with_llm_registry(registry);
```

No changes needed to task queuing or result retrieval APIs!

## Troubleshooting

### High Rate Limit Errors

**Problem**: Frequent rate limit errors in logs

**Solution**: Reduce `rate_limit_rps`:
```rust
rate_limit_rps: 5.0  // Reduce from 10.0
```

### Tasks Timing Out

**Problem**: Tasks frequently timeout

**Solution**: Increase timeout or reduce task size:
```rust
worker_timeout: Duration::from_secs(120),  // Increase from 60
// OR
max_tokens: 1024,  // Reduce from 2048
```

### Circuit Breaker Always Open

**Problem**: Circuit breaker frequently opens

**Solution**: Check provider health and adjust thresholds:
```rust
health_check_threshold: 5,  // Increase from 3
failback_delay: Duration::from_secs(120),  // Increase wait
```

### Queue Filling Up

**Problem**: Queue grows unbounded

**Solution**: Increase workers or reduce incoming rate:
```rust
num_workers: 8,  // Increase from 4
max_concurrent_requests: 20,  // Increase from 10
```

## Performance Metrics

Expected performance characteristics:

- **Throughput**: Up to `rate_limit_rps * num_workers` tasks/sec
- **Latency**: Median 1-3s (depends on LLM provider)
- **Queue capacity**: Configurable via `queue_size`
- **Memory**: ~10MB base + ~5KB per queued task

## References

- [LLM Provider Guide](./providers.md)
- [Failover Configuration](./failover.md)
- [Circuit Breaker Patterns](./circuit-breaker.md)
- [Rate Limiting Strategies](./rate-limiting.md)
