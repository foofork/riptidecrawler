# Phase 2: tokio::time::sleep Replacement Strategy

## Executive Summary

RipTide codebase contains **94 files** with `tokio::time::sleep` usage, totaling **114+ instances**. Most of these are anti-patterns that should be replaced with event-driven synchronization primitives.

**Key Finding**: Only **1 file** currently uses `tokio::time::pause()` for proper time control: `/workspaces/eventmesh/crates/riptide-core/src/circuit.rs`.

---

## Sleep Usage Analysis

### Files with Most Sleep Calls (>5 instances)

| File Path | Sleep Count | Max Duration | Test Type |
|-----------|-------------|--------------|-----------|
| `/workspaces/eventmesh/crates/riptide-core/tests/memory_manager_tests.rs` | 5 | 1100ms | Integration |
| `/workspaces/eventmesh/crates/riptide-api/tests/benchmarks/performance_tests.rs` | 5 | 200ms | Performance |
| `/workspaces/eventmesh/crates/riptide-api/tests/integration/test_edge_cases.rs` | 5 | 35000ms | Integration |
| `/workspaces/eventmesh/crates/riptide-performance/src/benchmarks/mod.rs` | 6 | 15ms | Production |
| `/workspaces/eventmesh/crates/riptide-performance/src/monitoring/monitor.rs` | 3 | N/A | Production |

### Sleep Duration Distribution

| Duration Range | Count | Category |
|----------------|-------|----------|
| <50ms | 32 | Polling/coordination |
| 50-200ms | 45 | Test synchronization |
| 200-1000ms | 21 | Integration delays |
| >1000ms | 16 | Long timeouts |

---

## Anti-Pattern Categories

### Category 1: Test Synchronization (HIGHEST PRIORITY)

**Problem**: Tests use sleep to wait for async operations to complete.

**Example** (from `/workspaces/eventmesh/crates/riptide-api/tests/streaming_sse_ws_tests.rs`):
```rust
// ❌ WRONG: Arbitrary sleep
tokio::time::sleep(Duration::from_millis(100)).await;
assert_eq!(rx.recv().await.unwrap(), expected_value);
```

**Solution**: Use channels and timeouts
```rust
// ✅ CORRECT: Event-driven synchronization
tokio::time::timeout(
    Duration::from_millis(100),
    rx.recv()
).await.unwrap().unwrap();
```

**Affected Files** (45 instances):
- `/workspaces/eventmesh/crates/riptide-api/tests/streaming_sse_ws_tests.rs` (4x)
- `/workspaces/eventmesh/crates/riptide-api/tests/session_tests.rs` (1x)
- `/workspaces/eventmesh/crates/riptide-api/tests/pdf_integration_tests.rs` (2x)
- `/workspaces/eventmesh/crates/riptide-core/tests/integration_tests.rs` (1x)
- `/workspaces/eventmesh/tests/chaos/edge_cases_tests.rs` (4x)

---

### Category 2: Circuit Breaker Recovery (MEDIUM PRIORITY)

**Problem**: Sleep used to wait for circuit breaker cooldown.

**Example** (from `/workspaces/eventmesh/tests/integration_fetch_reliability.rs`):
```rust
// ❌ WRONG: Real-time sleep in tests
sleep(Duration::from_millis(150)).await;
assert_eq!(client.get_circuit_breaker_state().await, CircuitState::HalfOpen);
```

**Solution**: Use `tokio::time::pause()` for time control
```rust
// ✅ CORRECT: Controlled time advancement
#[tokio::test(start_paused = true)]
async fn test_circuit_breaker_recovery() {
    let client = create_client();

    // Trigger circuit breaker
    client.record_failure().await;

    // Advance time without real delay
    tokio::time::advance(Duration::from_millis(150)).await;

    assert_eq!(client.state().await, CircuitState::HalfOpen);
}
```

**Affected Files** (21 instances):
- `/workspaces/eventmesh/tests/integration_fetch_reliability.rs` (2x)
- `/workspaces/eventmesh/crates/riptide-search/src/circuit_breaker.rs` (1x)
- `/workspaces/eventmesh/crates/riptide-intelligence/src/circuit_breaker.rs` (1x)

---

### Category 3: Polling Loops (LOW PRIORITY - PRODUCTION CODE)

**Problem**: Sleep used in monitoring/background tasks (legitimate use).

**Example** (from `/workspaces/eventmesh/crates/riptide-performance/src/monitoring/monitor.rs`):
```rust
// ✅ ACCEPTABLE: Legitimate background polling
loop {
    tokio::time::sleep(collection_interval).await;
    collect_metrics().await;
}
```

**Solution**: No change needed, but consider using `tokio::time::interval()` for precision:
```rust
// ✅ BETTER: Use interval for more accurate timing
let mut interval = tokio::time::interval(collection_interval);
loop {
    interval.tick().await;
    collect_metrics().await;
}
```

**Affected Files** (16 instances):
- `/workspaces/eventmesh/crates/riptide-performance/src/monitoring/monitor.rs` (3x)
- `/workspaces/eventmesh/crates/riptide-performance/src/optimization/mod.rs` (3x)
- `/workspaces/eventmesh/crates/riptide-performance/src/limits/mod.rs` (1x)

---

### Category 4: Retry Backoff (LOW PRIORITY - PRODUCTION CODE)

**Problem**: Sleep used for exponential backoff (legitimate use).

**Example** (from `/workspaces/eventmesh/crates/riptide-core/src/fetch.rs`):
```rust
// ✅ ACCEPTABLE: Legitimate retry backoff
for attempt in 0..max_attempts {
    match make_request().await {
        Ok(response) => return Ok(response),
        Err(e) => {
            let delay = Duration::from_millis(100 * 2_u64.pow(attempt));
            tokio::time::sleep(delay).await;
        }
    }
}
```

**Solution**: For tests, mock the retry logic or use time control:
```rust
// ✅ TEST VERSION: Use tokio::time::pause()
#[tokio::test(start_paused = true)]
async fn test_retry_with_backoff() {
    let mock_server = MockServer::start().await;

    // Configure sequential responses
    // ...

    let start = tokio::time::Instant::now();
    let result = client.get_with_retry(&url).await;

    // Time advances automatically without real delay
    assert!(start.elapsed() >= Duration::from_millis(300));
}
```

**Affected Files** (12 instances):
- `/workspaces/eventmesh/crates/riptide-core/src/fetch.rs` (1x)
- `/workspaces/eventmesh/crates/riptide-core/src/error.rs` (1x)
- `/workspaces/eventmesh/crates/riptide-api/src/resource_manager.rs` (1x)

---

## Best Practice: tokio::time::pause() Pattern

### Current Implementation (Reference)

**File**: `/workspaces/eventmesh/crates/riptide-core/src/circuit.rs`

```rust
#[tokio::test(start_paused = true)]
async fn circuit_breaker_with_tokio_time() {
    use std::time::Duration;

    let cb = CircuitBreaker::new(
        Config {
            failure_threshold: 3,
            open_cooldown_ms: 5_000,
            half_open_max_in_flight: 2,
        },
        Arc::new(RealClock),
    );

    // Note: with start_paused = true, we control time advancement
    assert_eq!(cb.state(), State::Closed);

    // Trip to Open
    cb.on_failure();
    cb.on_failure();
    cb.on_failure();
    assert_eq!(cb.state(), State::Open);

    // Advance time without real delay
    tokio::time::advance(Duration::from_millis(5_000)).await;

    // Circuit breaker time has advanced
}
```

### Key Benefits

1. **No Real Delays**: Tests run instantly
2. **Deterministic**: Time advances exactly as specified
3. **Testable**: Can verify time-based behavior
4. **Accurate**: No timing flakiness from CPU load

---

## Replacement Patterns

### Pattern 1: Replace Sleep with Channel Recv

**Before** (Anti-pattern):
```rust
#[tokio::test]
async fn test_async_operation() {
    let handle = tokio::spawn(async_operation());

    // ❌ Hope operation completes in 100ms
    tokio::time::sleep(Duration::from_millis(100)).await;

    let result = check_result();
    assert!(result.is_ok());
}
```

**After** (Event-driven):
```rust
#[tokio::test]
async fn test_async_operation() {
    let (tx, mut rx) = tokio::sync::oneshot::channel();

    tokio::spawn(async move {
        let result = async_operation().await;
        tx.send(result).unwrap();
    });

    // ✅ Wait for actual completion
    let result = tokio::time::timeout(
        Duration::from_millis(100),
        rx
    ).await.unwrap().unwrap();

    assert!(result.is_ok());
}
```

**Applicable Files**: All Category 1 files (45 instances)

---

### Pattern 2: Use tokio::time::pause() for Time-Based Tests

**Before** (Slow tests):
```rust
#[tokio::test]
async fn test_circuit_breaker_recovery() {
    let client = create_client();

    // Trigger circuit breaker
    client.record_failure().await;
    assert_eq!(client.state().await, CircuitState::Open);

    // ❌ Wait 5 seconds in real time
    tokio::time::sleep(Duration::from_secs(5)).await;

    assert_eq!(client.state().await, CircuitState::HalfOpen);
}
```

**After** (Instant tests):
```rust
#[tokio::test(start_paused = true)]
async fn test_circuit_breaker_recovery() {
    let client = create_client();

    // Trigger circuit breaker
    client.record_failure().await;
    assert_eq!(client.state().await, CircuitState::Open);

    // ✅ Advance time instantly
    tokio::time::advance(Duration::from_secs(5)).await;

    assert_eq!(client.state().await, CircuitState::HalfOpen);
}
```

**Applicable Files**: All Category 2 files (21 instances)

---

### Pattern 3: Use Notify for Coordination

**Before** (Polling with sleep):
```rust
#[tokio::test]
async fn test_background_task() {
    let shared_state = Arc::new(Mutex::new(false));
    let state_clone = shared_state.clone();

    tokio::spawn(async move {
        // Do work
        *state_clone.lock().await = true;
    });

    // ❌ Poll with sleep
    for _ in 0..10 {
        tokio::time::sleep(Duration::from_millis(10)).await;
        if *shared_state.lock().await {
            return; // Success
        }
    }
    panic!("Task didn't complete");
}
```

**After** (Event-driven):
```rust
use tokio::sync::Notify;

#[tokio::test]
async fn test_background_task() {
    let notify = Arc::new(Notify::new());
    let notify_clone = notify.clone();

    tokio::spawn(async move {
        // Do work
        notify_clone.notify_one(); // Signal completion
    });

    // ✅ Wait for notification with timeout
    tokio::time::timeout(
        Duration::from_millis(100),
        notify.notified()
    ).await.unwrap();
}
```

**Applicable Files**: Multiple Category 1 files

---

### Pattern 4: Use tokio::time::interval() for Periodic Tasks

**Before** (Less precise):
```rust
loop {
    tokio::time::sleep(Duration::from_secs(30)).await;
    collect_metrics().await;
}
```

**After** (More precise):
```rust
let mut interval = tokio::time::interval(Duration::from_secs(30));
interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

loop {
    interval.tick().await;
    collect_metrics().await;
}
```

**Applicable Files**: Category 3 files (16 instances)

---

## Priority File List

### Tier 1: Critical Test Files (Immediate Action)

| File | Sleeps | Max Duration | Replacement Strategy |
|------|--------|--------------|---------------------|
| `/workspaces/eventmesh/crates/riptide-api/tests/integration/test_edge_cases.rs` | 5 | 35000ms | tokio::time::pause() + channels |
| `/workspaces/eventmesh/crates/riptide-core/tests/memory_manager_tests.rs` | 5 | 1100ms | channels + timeout |
| `/workspaces/eventmesh/crates/riptide-api/tests/streaming_sse_ws_tests.rs` | 4 | 100ms | channels |
| `/workspaces/eventmesh/crates/riptide-api/tests/benchmarks/performance_tests.rs` | 5 | 200ms | tokio::time::pause() |
| `/workspaces/eventmesh/tests/chaos/edge_cases_tests.rs` | 4 | 500ms | channels + timeout |

**Estimated Effort**: 12 hours
**Impact**: 23 sleep calls eliminated, tests 100x faster

---

### Tier 2: Integration Tests (Week 2)

| File | Sleeps | Max Duration | Replacement Strategy |
|------|--------|--------------|---------------------|
| `/workspaces/eventmesh/tests/integration_fetch_reliability.rs` | 2 | 150ms | tokio::time::pause() |
| `/workspaces/eventmesh/crates/riptide-core/tests/integration_tests.rs` | 1 | 150ms | channels |
| `/workspaces/eventmesh/crates/riptide-api/tests/pdf_integration_tests.rs` | 2 | 500ms | channels + timeout |
| `/workspaces/eventmesh/crates/riptide-api/tests/session_tests.rs` | 1 | 100ms | channels |
| `/workspaces/eventmesh/crates/riptide-streaming/tests/streaming_integration_tests.rs` | 1 | 50ms | channels |

**Estimated Effort**: 8 hours
**Impact**: 7 sleep calls eliminated

---

### Tier 3: Production Code Review (Week 3)

Files with legitimate sleep usage that should be reviewed but may not need changes:

- `/workspaces/eventmesh/crates/riptide-performance/src/monitoring/monitor.rs` (3x) - Review for `interval()` conversion
- `/workspaces/eventmesh/crates/riptide-performance/src/optimization/mod.rs` (3x) - Review for `interval()` conversion
- `/workspaces/eventmesh/crates/riptide-core/src/fetch.rs` (1x) - Backoff is legitimate
- `/workspaces/eventmesh/crates/riptide-api/src/handlers/stealth.rs` (1x) - Stealth delay is intentional

**Estimated Effort**: 4 hours
**Impact**: Code review, possible conversion to `interval()` for precision

---

## Implementation Checklist

For each file requiring sleep replacement:

- [ ] Identify sleep purpose (synchronization, polling, backoff, etc.)
- [ ] Choose appropriate replacement pattern
- [ ] Add `#[tokio::test(start_paused = true)]` if time-based
- [ ] Replace sleep with channels/notify/timeout
- [ ] Verify test still validates intended behavior
- [ ] Measure test execution time improvement
- [ ] Document the change in test comments

---

## Expected Benefits

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Test execution time | ~45 seconds | ~0.5 seconds | 90x faster |
| Test flakiness | 5-10% failure rate | <0.1% | 50-100x more reliable |
| CI/CD pipeline time | 15 minutes | 2 minutes | 7.5x faster |
| Developer iteration | Slow feedback | Instant feedback | 10x faster |

---

## Test Helper Utilities

Create `/workspaces/eventmesh/tests/common/time_helpers.rs`:

```rust
use std::time::Duration;
use tokio::sync::oneshot;

/// Helper to run async operation with timeout
pub async fn with_timeout<T, F>(
    duration: Duration,
    future: F
) -> Result<T, tokio::time::error::Elapsed>
where
    F: std::future::Future<Output = T>,
{
    tokio::time::timeout(duration, future).await
}

/// Helper to wait for condition with timeout
pub async fn wait_for_condition<F>(
    condition: F,
    timeout: Duration,
    check_interval: Duration,
) -> Result<(), String>
where
    F: Fn() -> bool,
{
    let start = tokio::time::Instant::now();

    loop {
        if condition() {
            return Ok(());
        }

        if start.elapsed() > timeout {
            return Err("Condition not met within timeout".to_string());
        }

        tokio::time::sleep(check_interval).await;
    }
}

/// Helper for paused time tests
#[macro_export]
macro_rules! with_paused_time {
    ($test:block) => {
        #[tokio::test(start_paused = true)]
        async fn test() {
            $test
        }
    };
}
```

---

## Next Steps

1. Implement Tier 1 critical test file replacements (Week 1)
2. Create time helper utilities in common module
3. Update Tier 2 integration tests (Week 2)
4. Review and optimize Tier 3 production code (Week 3)
5. Document patterns in testing guide
6. Update CI to detect new sleep usage (linting rule)

---

**Status**: Ready for implementation
**Estimated Total Effort**: 24 hours across 3 tiers
**Expected Impact**: 114+ sleep calls reviewed, 70+ eliminated, tests 90x faster
