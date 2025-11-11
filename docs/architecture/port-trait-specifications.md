# Port Trait Architecture Specifications

**Phase**: 1-2 Architecture Design
**Date**: 2025-11-11
**Status**: Complete
**Architect**: System Architecture Designer

---

## Executive Summary

This document specifies all port traits required for the hexagonal architecture migration. It defines the abstract interfaces that enable dependency inversion, facilitate testing, and allow swapping implementations without modifying application logic.

## Design Principles

1. **Dependency Inversion**: Application depends on abstractions (traits), not concrete types
2. **Interface Segregation**: Each port has a single, well-defined responsibility
3. **Testability**: All ports have in-memory implementations for fast testing
4. **Async-First**: All I/O operations are async for maximum concurrency
5. **Thread-Safe**: All ports are `Send + Sync` for multi-threaded usage

---

## Port Trait Catalog

### 1. CacheStorage Port ✅ (Already Exists)

**Location**: `crates/riptide-types/src/ports/cache.rs`

**Purpose**: Backend-agnostic cache abstraction for storing and retrieving binary data

**Key Methods**:
```rust
async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
async fn set(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> Result<()>;
async fn delete(&self, key: &str) -> Result<()>;
async fn exists(&self, key: &str) -> Result<bool>;
async fn mset(&self, items: Vec<(&str, &[u8])>, ttl: Option<Duration>) -> Result<()>;
async fn mget(&self, keys: &[&str]) -> Result<Vec<Option<Vec<u8>>>>;
async fn health_check(&self) -> Result<bool>;
async fn stats(&self) -> Result<CacheStats>;
```

**Implementations**:
- `InMemoryCache` - HashMap-based (testing)
- `RedisCache` - Redis backend (production)
- Future: `MemcachedCache`, `DynamoCache`

**Status**: ✅ Complete

---

### 2. CircuitBreaker Port ⚡ (New - To Be Implemented)

**Location**: `crates/riptide-types/src/ports/reliability.rs` (new file)

**Purpose**: Prevent cascading failures by breaking circuits when error rates exceed thresholds

**State Machine**:
```
     [Closed] ────failure──→ [Open] ────timeout──→ [HalfOpen]
         ↑                                              │
         └────────────────success───────────────────────┘
```

**Trait Definition**:
```rust
/// Circuit breaker port for fault tolerance
#[async_trait]
pub trait CircuitBreaker: Send + Sync {
    /// Attempt to acquire permission to execute an operation
    /// Returns Err if circuit is open
    async fn try_acquire(&self) -> Result<CircuitPermit>;

    /// Record successful operation
    async fn record_success(&self);

    /// Record failed operation
    async fn record_failure(&self);

    /// Get current circuit state
    async fn state(&self) -> CircuitState;

    /// Get circuit breaker metrics
    async fn metrics(&self) -> CircuitMetrics;

    /// Manually reset circuit to closed state
    async fn reset(&self);
}

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, operations allowed
    Closed,
    /// Circuit is open, operations blocked
    Open,
    /// Circuit is half-open, testing if service recovered
    HalfOpen,
}

/// Circuit breaker permit (RAII guard)
pub struct CircuitPermit {
    breaker: Arc<dyn CircuitBreaker>,
}

impl Drop for CircuitPermit {
    fn drop(&mut self) {
        // Auto-record success if not explicitly recorded
    }
}

/// Circuit breaker metrics
#[derive(Debug, Clone)]
pub struct CircuitMetrics {
    pub total_calls: u64,
    pub successful_calls: u64,
    pub failed_calls: u64,
    pub rejected_calls: u64,
    pub current_state: CircuitState,
    pub last_state_change: Option<std::time::Instant>,
    pub failure_rate: f64,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening circuit
    pub failure_threshold: u32,
    /// Duration to wait before transitioning to half-open (ms)
    pub open_cooldown_ms: u64,
    /// Max concurrent requests in half-open state
    pub half_open_max_requests: u32,
    /// Window size for failure rate calculation
    pub failure_rate_window: Duration,
}
```

**Implementations**:
- `AtomicCircuitBreaker` - Lock-free atomic implementation (riptide-utils)
- `StateBasedCircuitBreaker` - Event bus integrated (riptide-reliability)
- `InMemoryCircuitBreaker` - Testing stub (always closed)

**Usage Example**:
```rust
let breaker = AtomicCircuitBreaker::new(config);

match breaker.try_acquire().await {
    Ok(permit) => {
        match risky_operation().await {
            Ok(result) => {
                breaker.record_success().await;
                Ok(result)
            }
            Err(e) => {
                breaker.record_failure().await;
                Err(e)
            }
        }
    }
    Err(_) => {
        // Circuit open, fail fast
        Err(Error::CircuitOpen)
    }
}
```

**Status**: 🔨 To Be Implemented

---

### 3. HealthCheck Port ✅ (Already Exists)

**Location**: `crates/riptide-types/src/ports/health.rs`

**Purpose**: Monitor component health and overall system availability

**Key Methods**:
```rust
async fn check(&self) -> Result<HealthStatus>;
fn name(&self) -> &str;
```

**HealthStatus**:
- `Healthy` - Component fully operational
- `Degraded { reason }` - Operational but impaired
- `Unhealthy { error }` - Not operational

**HealthRegistry**:
```rust
async fn register(&mut self, check: Arc<dyn HealthCheck>);
async fn unregister(&mut self, name: &str) -> bool;
async fn check_all(&self) -> HashMap<String, HealthStatus>;
async fn is_healthy(&self) -> bool;
async fn overall_status(&self) -> HealthStatus;
```

**Status**: ✅ Complete

---

### 4. ResourcePool Port ✅ (Already Exists)

**Location**: `crates/riptide-types/src/ports/pool.rs`

**Purpose**: Generic pool interface for managing pooled resources (WASM, Browser, LLM)

**Key Methods**:
```rust
async fn acquire(&self) -> Result<PooledResource<T>, PoolError>;
async fn release(&self, resource: T) -> Result<(), PoolError>;
async fn size(&self) -> usize;
async fn available(&self) -> usize;
async fn health(&self) -> PoolHealth;
async fn stats(&self) -> PoolStats;
```

**PooledResource<T>**: RAII wrapper that auto-releases on drop

**Implementations**:
- `WasmInstancePool` - WASM instance pooling
- `BrowserPool` - Browser session pooling
- `LlmClientPool` - LLM client pooling

**Status**: ✅ Complete

---

### 5. MetricsRegistry Port ✅ (Already Exists)

**Location**: `crates/riptide-types/src/ports/metrics.rs`

**Purpose**: Low-level and business metrics collection

**MetricsCollector Trait**:
```rust
fn record_counter(&self, name: &str, value: u64, tags: &[(&str, &str)]);
fn record_histogram(&self, name: &str, value: f64, tags: &[(&str, &str)]);
fn record_gauge(&self, name: &str, value: f64, tags: &[(&str, &str)]);
```

**BusinessMetrics Trait**:
```rust
fn record_extraction(&self, duration: Duration, success: bool);
fn record_cache_hit(&self, key_type: &str);
fn record_cache_miss(&self, key_type: &str);
fn record_event_published(&self, event_type: &str);
fn record_http_request(&self, method: &str, status: u16, duration: Duration);
fn record_pipeline_stage(&self, stage: &str, duration: Duration, success: bool);
fn record_error(&self, error_type: &str, context: &str);
```

**Implementations**:
- `PrometheusMetrics` - Prometheus exporter
- `DatadogMetrics` - Datadog integration
- `InMemoryMetrics` - Testing stub

**Status**: ✅ Complete

---

### 6. Additional Ports Already Implemented

#### 6.1 Repository Port ✅
**Location**: `crates/riptide-types/src/ports/repository.rs`

Generic repository pattern for domain entities with CRUD operations.

#### 6.2 EventBus Port ✅
**Location**: `crates/riptide-types/src/ports/events.rs`

Publish/subscribe event bus for domain events.

#### 6.3 IdempotencyStore Port ✅
**Location**: `crates/riptide-types/src/ports/idempotency.rs`

Idempotency token management for duplicate request prevention.

#### 6.4 SessionStorage Port ✅
**Location**: `crates/riptide-types/src/ports/session.rs`

Session management for browser and user sessions.

#### 6.5 StreamingTransport Port ✅
**Location**: `crates/riptide-types/src/ports/streaming.rs`

Real-time streaming transport (SSE, WebSocket).

#### 6.6 RateLimiter Port ✅
**Location**: `crates/riptide-types/src/ports/rate_limit.rs`

Rate limiting with per-host and global limits.

#### 6.7 HttpClient Port ✅
**Location**: `crates/riptide-types/src/ports/http.rs`

HTTP client abstraction for testable HTTP operations.

---

## Port Summary Table

| Port Name | Status | Location | Implementations |
|-----------|--------|----------|-----------------|
| CacheStorage | ✅ Complete | ports/cache.rs | InMemory, Redis |
| CircuitBreaker | 🔨 New | ports/reliability.rs | Atomic, StateBased, InMemory |
| HealthCheck | ✅ Complete | ports/health.rs | Component-specific |
| ResourcePool | ✅ Complete | ports/pool.rs | WASM, Browser, LLM |
| MetricsRegistry | ✅ Complete | ports/metrics.rs | Prometheus, Datadog, InMemory |
| Repository | ✅ Complete | ports/repository.rs | Postgres, InMemory |
| EventBus | ✅ Complete | ports/events.rs | Outbox, InMemory |
| IdempotencyStore | ✅ Complete | ports/idempotency.rs | Redis, InMemory |
| SessionStorage | ✅ Complete | ports/session.rs | Postgres, InMemory |
| StreamingTransport | ✅ Complete | ports/streaming.rs | SSE, WebSocket |
| RateLimiter | ✅ Complete | ports/rate_limit.rs | TokenBucket, SlidingWindow |
| HttpClient | ✅ Complete | ports/http.rs | Reqwest, MockClient |

**Summary**: 11/12 ports complete, 1 new port to implement (CircuitBreaker)

---

## Next Steps

1. **Implement CircuitBreaker Port** (Phase 2.1)
   - Create `crates/riptide-types/src/ports/reliability.rs`
   - Define trait and types
   - Add to ports/mod.rs exports

2. **Verify Port Consistency** (Phase 2.2)
   - Ensure all ports follow naming conventions
   - Verify async trait usage
   - Check error handling patterns

3. **Design ApplicationContext** (Phase 2.3)
   - See: `application-context-design.md`

4. **Create Port Adapters** (Phase 3)
   - One adapter per port implementation
   - Follow hexagonal architecture
