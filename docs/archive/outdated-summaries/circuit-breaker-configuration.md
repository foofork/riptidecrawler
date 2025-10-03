# Circuit Breaker Configuration Guide

## Overview

The RipTide API implements the Circuit Breaker pattern to provide fault tolerance and prevent cascading failures across critical operations. This document describes configuration, behavior, and monitoring.

## Circuit Breaker States

### State Machine
```
CLOSED → (failure threshold exceeded) → OPEN
OPEN → (timeout elapsed) → HALF_OPEN
HALF_OPEN → (success) → CLOSED
HALF_OPEN → (failure) → OPEN
```

### State Descriptions

**CLOSED** (Normal Operation)
- All requests are allowed through
- Success/failure metrics are tracked
- Transitions to OPEN when failure threshold exceeded

**OPEN** (Failure Protection)
- All requests are rejected immediately
- Returns 503 Service Unavailable
- Waits for timeout period before attempting recovery
- Prevents further damage to failing services

**HALF_OPEN** (Recovery Testing)
- Limited number of requests allowed through
- Tests if the underlying service has recovered
- Success → return to CLOSED state
- Failure → return to OPEN state

## Configuration

### Environment Variables

```bash
# Circuit breaker failure threshold (0.0-1.0)
# Default: 0.5 (50% failure rate triggers circuit)
CIRCUIT_BREAKER_FAILURE_THRESHOLD=0.5

# Circuit breaker timeout in milliseconds
# Default: 5000 (5 seconds before attempting recovery)
CIRCUIT_BREAKER_TIMEOUT_MS=5000

# Minimum requests before circuit can open
# Default: 10 (prevents premature tripping on low traffic)
CIRCUIT_BREAKER_MIN_REQUESTS=10
```

### Configuration in Code

```rust
use riptide_api::state::CircuitBreakerConfig;

let config = CircuitBreakerConfig {
    failure_threshold: 0.5,  // 50% failure rate
    timeout_ms: 5000,        // 5 second recovery timeout
    min_requests: 10,        // Minimum 10 requests before opening
};
```

## Protected Operations

The circuit breaker wraps the following critical operations:

### 1. WASM Extraction
- **Operation**: `wasm_extraction`
- **Function**: `extract_with_circuit_breaker()`
- **Purpose**: Fast CSS-based content extraction
- **Failure Scenarios**: Extraction errors, timeout, invalid HTML

### 2. PDF Processing
- **Operation**: `pdf_processing`
- **Function**: `process_pdf_with_circuit_breaker()`
- **Purpose**: PDF document processing and extraction
- **Failure Scenarios**: Corrupt PDFs, memory issues, parsing errors

### 3. Headless Extraction
- **Operation**: `headless_extraction`
- **Function**: `headless_extract_with_circuit_breaker()`
- **Purpose**: JavaScript-heavy page rendering
- **Failure Scenarios**: Headless service down, timeout, rendering errors

## Event Emissions

### Circuit Breaker Events

**circuit_breaker.open** (Severity: Warn)
```json
{
  "event_type": "circuit_breaker.open",
  "source": "circuit_breaker_utils",
  "severity": "warn",
  "metadata": {
    "operation": "wasm_extraction",
    "failure_rate": "65.5",
    "state": "Open"
  }
}
```

**circuit_breaker.state_change** (Severity: Info/Error)
```json
{
  "event_type": "circuit_breaker.state_change",
  "source": "circuit_breaker_utils",
  "severity": "info",
  "metadata": {
    "operation": "pdf_processing",
    "new_state": "Closed",
    "duration_ms": "125"
  }
}
```

## Performance Metrics

### Tracked Metrics

- **Request Duration**: Time taken for each operation (ms)
- **Success Rate**: Percentage of successful operations
- **Failure Rate**: Percentage of failed operations
- **Circuit State**: Current state (Closed/Open/HalfOpen)
- **Trip Count**: Number of times circuit has opened

### Metrics Integration

```rust
// Metrics are automatically tracked via PerformanceMetrics
let metrics = app_state.performance_metrics.lock().await;
let failure_rate = metrics.failure_rate();
let avg_duration = metrics.avg_duration_ms();
```

## Monitoring and Alerts

### Health Check Integration

The circuit breaker state is included in health check responses:

```bash
curl http://localhost:8080/healthz
```

Response:
```json
{
  "healthy": true,
  "redis": "healthy",
  "extractor": "healthy",
  "circuit_breaker": {
    "state": "Closed",
    "failure_rate": 0.0,
    "total_requests": 1234
  }
}
```

### Prometheus Metrics

Available metrics at `/metrics`:

```
# Circuit breaker state (0=Closed, 1=Open, 2=HalfOpen)
circuit_breaker_state{operation="wasm_extraction"} 0

# Circuit breaker failures
circuit_breaker_failures_total{operation="wasm_extraction"} 45

# Circuit breaker successes
circuit_breaker_successes_total{operation="wasm_extraction"} 955

# Circuit breaker trips (state transitions to Open)
circuit_breaker_trips_total{operation="wasm_extraction"} 3
```

### Logging

Circuit breaker events are logged with structured fields:

```
WARN circuit_breaker is OPEN, rejecting request
  operation=wasm_extraction state=Open failure_rate=65.5

INFO Operation succeeded
  operation=pdf_processing duration_ms=125

ERROR Operation failed
  operation=headless_extraction duration_ms=5000 error="timeout" failure_rate=55.0
```

## Tuning Guidelines

### Failure Threshold

**Conservative** (0.3 - 40% failure rate)
- Use for critical services
- Opens circuit early to protect dependencies
- May cause unnecessary circuit trips

**Balanced** (0.5 - 50% failure rate) - **DEFAULT**
- Good for most use cases
- Balances protection and availability

**Permissive** (0.7 - 70% failure rate)
- Use for less critical operations
- Tolerates more failures before opening
- Risk of cascading failures

### Timeout Duration

**Short** (1-3 seconds)
- Fast recovery attempts
- Use when service recovers quickly
- May cause rapid open/close cycles

**Medium** (5 seconds) - **DEFAULT**
- Balanced recovery time
- Good for most scenarios

**Long** (10-30 seconds)
- Use for services with slow recovery
- Prevents rapid retry storms
- May delay recovery detection

### Minimum Requests Threshold

**Low** (5-10) - **DEFAULT**
- Opens circuit faster on failures
- Good for high-traffic services
- May trip prematurely on low traffic

**High** (20-50)
- Requires more data before tripping
- Good for bursty traffic patterns
- May delay circuit opening

## Best Practices

### 1. Set Appropriate Thresholds
```bash
# Production - balanced protection
CIRCUIT_BREAKER_FAILURE_THRESHOLD=0.5
CIRCUIT_BREAKER_TIMEOUT_MS=5000
CIRCUIT_BREAKER_MIN_REQUESTS=10

# Development - more permissive
CIRCUIT_BREAKER_FAILURE_THRESHOLD=0.7
CIRCUIT_BREAKER_TIMEOUT_MS=3000
CIRCUIT_BREAKER_MIN_REQUESTS=5
```

### 2. Monitor Circuit State
- Set up alerts for circuit OPEN events
- Track failure rates and recovery times
- Review circuit trip patterns

### 3. Handle Circuit Open Errors
```rust
match pipeline.execute_single(url).await {
    Ok(result) => { /* success */ },
    Err(ApiError::ServiceUnavailable(_)) => {
        // Circuit breaker is open
        // Return cached data or graceful degradation
    },
    Err(e) => { /* other errors */ }
}
```

### 4. Test Circuit Behavior
```rust
#[tokio::test]
async fn test_circuit_opens_on_failures() {
    // Generate failures to trip circuit
    for _ in 0..10 {
        let _ = pipeline.execute_single("http://invalid").await;
    }

    // Verify circuit is open
    let state = circuit_breaker.lock().await;
    assert_eq!(state.state(), CircuitState::Open);
}
```

## Troubleshooting

### Circuit Stuck Open
**Symptoms**: Circuit remains OPEN even when service recovers

**Causes**:
- Timeout too long
- Underlying service still failing
- Half-open requests failing

**Solutions**:
1. Check service health: `curl http://localhost:8080/healthz`
2. Reduce timeout: `CIRCUIT_BREAKER_TIMEOUT_MS=3000`
3. Review failure logs for root cause

### Frequent Circuit Trips
**Symptoms**: Circuit opens/closes repeatedly

**Causes**:
- Threshold too low
- Service intermittently failing
- Traffic spikes overwhelming service

**Solutions**:
1. Increase threshold: `CIRCUIT_BREAKER_FAILURE_THRESHOLD=0.6`
2. Increase min requests: `CIRCUIT_BREAKER_MIN_REQUESTS=20`
3. Add caching layer to reduce load

### Circuit Never Opens
**Symptoms**: Circuit stays CLOSED even with failures

**Causes**:
- Min requests threshold too high
- Failure rate below threshold
- Requests not being tracked

**Solutions**:
1. Lower min requests: `CIRCUIT_BREAKER_MIN_REQUESTS=5`
2. Lower threshold: `CIRCUIT_BREAKER_FAILURE_THRESHOLD=0.4`
3. Verify metrics tracking enabled

## Related Documentation

- [Performance Roadmap](./performance/implementation-roadmap.md) - Phase 1 Circuit Breaker items
- [Event System](./event-system.md) - Event emission and handling
- [Health Checks](./health-checks.md) - System health monitoring
- [Metrics](./metrics.md) - Prometheus metrics and monitoring

## References

### Circuit Breaker Pattern
- [Martin Fowler - Circuit Breaker](https://martinfowler.com/bliki/CircuitBreaker.html)
- [Microsoft - Circuit Breaker Pattern](https://docs.microsoft.com/en-us/azure/architecture/patterns/circuit-breaker)
- [Resilience4j Circuit Breaker](https://resilience4j.readme.io/docs/circuitbreaker)

### Implementation
- Source: `crates/riptide-api/src/circuit_breaker_utils.rs`
- Tests: `tests/unit/riptide_search_circuit_breaker_tests.rs`
- Integration: `crates/riptide-api/src/pipeline.rs`
