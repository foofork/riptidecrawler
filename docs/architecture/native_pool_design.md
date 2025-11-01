# NativeExtractorPool Architecture Design

**Priority**: P1-HIGH
**Status**: Design Phase
**Author**: System Architect
**Date**: 2025-11-01

## Executive Summary

This document specifies the architecture for NativeExtractorPool, a dedicated pooling system for native (CSS/Regex) extraction strategies. Currently, native extraction has NO dedicated pool and is only used as a fallback to WASM. This design **reverses that paradigm**: native becomes PRIMARY with proper pooling support, and WASM becomes the fallback/enhancement strategy.

## Problem Statement

### Current State
- **WASM has pool** (`AdvancedInstancePool` in `/workspaces/eventmesh/crates/riptide-pool/src/pool.rs`)
- **Native has NO pool** - exists only as fallback at line 280 in pool.rs
- **Browser has pool** (`BrowserPool` in `/workspaces/eventmesh/crates/riptide-browser/src/pool/mod.rs`)
- This is backwards: native should be PRIMARY with first-class pooling

### Evidence from Codebase
```rust
// pool.rs:289-324 - WASM uses pool, native is fallback only
// Fallback to native extraction if WASM fails
tracing::warn!(
    url = %url,
    error = %e,
    "WASM extraction failed, falling back to native extraction"
);

// Use native parser as fallback
use riptide_extraction::native_parser::NativeHtmlParser;
let native_parser = NativeHtmlParser::default();
```

### Design Goals
1. **Native First**: Make native extraction PRIMARY with dedicated pool
2. **Performance**: Match or exceed WASM pool performance characteristics
3. **Reliability**: Circuit breaker, health monitoring, resource limits
4. **Integration**: Seamless integration with existing extraction pipeline
5. **Metrics**: Comprehensive performance tracking and observability

---

## Architecture Overview

### System Context

```
┌──────────────────────────────────────────────────────────────────┐
│                     Extraction Pipeline                          │
│                                                                  │
│  ┌──────────────┐      ┌──────────────┐      ┌──────────────┐ │
│  │   Fetch      │─────▶│   Extract    │─────▶│   Response   │ │
│  │   HTML       │      │   Content    │      │   Return     │ │
│  └──────────────┘      └──────────────┘      └──────────────┘ │
│                              │                                  │
│                              ▼                                  │
│                    ┌─────────────────┐                         │
│                    │ NativePool (NEW)│◀─── PRIMARY             │
│                    │ - CSS Strategy  │                         │
│                    │ - Regex Strategy│                         │
│                    └─────────────────┘                         │
│                              │                                  │
│                              │ fallback (if native fails)      │
│                              ▼                                  │
│                    ┌─────────────────┐                         │
│                    │  WASM Pool      │◀─── FALLBACK            │
│                    │ (existing)      │                         │
│                    └─────────────────┘                         │
└──────────────────────────────────────────────────────────────────┘
```

---

## Component Design

### 1. Core Structures

#### 1.1 NativeExtractorPool (Main Pool Manager)

**Location**: `/workspaces/eventmesh/crates/riptide-pool/src/native_pool.rs` (already exists, needs enhancement)

```rust
pub struct NativeExtractorPool {
    /// Pool configuration
    config: NativePoolConfig,

    /// Type of extractor (CSS or Regex)
    extractor_type: NativeExtractorType,

    /// Available instances queue
    available_instances: Arc<Mutex<VecDeque<PooledNativeInstance>>>,

    /// Semaphore for concurrency control
    semaphore: Arc<Semaphore>,

    /// Performance metrics
    metrics: Arc<Mutex<NativePoolMetrics>>,

    /// Circuit breaker state
    circuit_state: Arc<Mutex<CircuitBreakerState>>,

    /// Pool unique identifier
    pool_id: String,

    /// Optional event bus for event emission
    #[cfg(feature = "wasm-pool")]
    event_bus: Option<Arc<EventBus>>,
}
```

**Key Methods**:
```rust
impl NativeExtractorPool {
    // Lifecycle
    pub async fn new(config: NativePoolConfig, extractor_type: NativeExtractorType) -> Result<Self>;
    async fn warm_up(&self) -> Result<()>;

    // Extraction
    pub async fn extract(&self, html: &str, url: &str) -> Result<ExtractedDoc>;
    async fn extract_with_instance(
        &self,
        instance: &mut PooledNativeInstance,
        html: &str,
        url: &str,
    ) -> Result<ExtractedContent>;

    // Instance management
    async fn get_or_create_instance(&self) -> PooledNativeInstance;
    async fn return_instance(&self, instance: PooledNativeInstance);
    fn create_instance(&self) -> PooledNativeInstance;

    // Health and monitoring
    async fn is_circuit_open(&self) -> bool;
    async fn record_extraction_result(&self, success: bool, duration: Duration);
    async fn record_timeout(&self);
    pub async fn get_metrics(&self) -> NativePoolMetrics;
    pub async fn get_pool_status(&self) -> (usize, usize, usize); // (available, active, max)

    // Event integration
    #[cfg(feature = "wasm-pool")]
    pub fn set_event_bus(&mut self, event_bus: Arc<EventBus>);
}
```

#### 1.2 PooledNativeInstance (Individual Instance)

```rust
struct PooledNativeInstance {
    /// Instance unique identifier
    id: String,

    /// Type of extractor
    extractor_type: NativeExtractorType,

    /// CSS extractor (if type is Css)
    css_extractor: Option<CssSelectorStrategy>,

    /// Regex extractor (if type is Regex)
    regex_extractor: Option<RegexPatternStrategy>,

    /// Usage statistics
    use_count: u32,
    failure_count: u32,
    last_used: Instant,
    created_at: Instant,

    /// Resource tracking
    memory_usage: usize,
}

impl PooledNativeInstance {
    fn new(extractor_type: NativeExtractorType) -> Self;
    fn is_healthy(&self, config: &NativePoolConfig) -> bool;
    fn record_success(&mut self);
    fn record_failure(&mut self);
    async fn extract(&self, html: &str, url: &str) -> Result<ExtractedContent>;
}
```

#### 1.3 NativePoolConfig (Configuration)

**Location**: Already exists in `/workspaces/eventmesh/crates/riptide-pool/src/native_pool.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativePoolConfig {
    /// Pool size limits
    pub max_pool_size: usize,          // Default: 8
    pub initial_pool_size: usize,      // Default: 2

    /// Timeout configuration
    pub extraction_timeout: u64,        // Default: 30000ms
    pub health_check_interval: u64,     // Default: 30000ms

    /// Resource limits
    pub memory_limit: Option<usize>,    // Default: Some(256MB)
    pub cpu_limit: Option<f32>,         // Default: Some(80.0%)

    /// Circuit breaker
    pub circuit_breaker_failure_threshold: u32,  // Default: 5
    pub circuit_breaker_timeout: u64,            // Default: 5000ms

    /// Instance lifecycle
    pub max_instance_reuse: u32,        // Default: 1000
    pub max_failure_count: u32,         // Default: 10
}

impl NativePoolConfig {
    pub fn validate(&self) -> Result<()>;
    pub fn from_env() -> Self;  // Load from environment variables
}
```

**Environment Variables**:
```bash
NATIVE_POOL_MAX_SIZE=8
NATIVE_POOL_INITIAL_SIZE=2
NATIVE_POOL_EXTRACTION_TIMEOUT_MS=30000
NATIVE_POOL_MEMORY_LIMIT_BYTES=268435456  # 256MB
NATIVE_POOL_CPU_LIMIT_PCT=80.0
NATIVE_POOL_CIRCUIT_BREAKER_THRESHOLD=5
NATIVE_POOL_CIRCUIT_BREAKER_TIMEOUT_MS=5000
NATIVE_POOL_MAX_INSTANCE_REUSE=1000
NATIVE_POOL_MAX_FAILURE_COUNT=10
```

#### 1.4 NativePoolMetrics (Performance Tracking)

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NativePoolMetrics {
    /// Extraction counts
    pub total_extractions: u64,
    pub successful_extractions: u64,
    pub failed_extractions: u64,

    /// Timing metrics
    pub avg_processing_time_ms: f64,
    pub avg_semaphore_wait_ms: f64,

    /// Pool state
    pub pool_size: usize,
    pub active_instances: usize,
    pub available_instances: usize,

    /// Circuit breaker
    pub circuit_breaker_trips: u64,
    pub circuit_breaker_state: String,  // "closed", "open", "half-open"

    /// Error tracking
    pub timeout_count: u64,
    pub memory_limit_violations: u64,
    pub instance_creation_failures: u64,
}
```

### 2. Health Monitoring

#### 2.1 Health Check Implementation

```rust
impl PooledNativeInstance {
    /// Multi-level health check
    fn is_healthy(&self, config: &NativePoolConfig) -> bool {
        // Level 1: Usage limits
        if self.use_count >= config.max_instance_reuse {
            return false;
        }

        // Level 2: Failure rate
        if self.failure_count >= config.max_failure_count {
            return false;
        }

        // Level 3: Resource limits
        if let Some(memory_limit) = config.memory_limit {
            if self.memory_usage > memory_limit {
                return false;
            }
        }

        true
    }
}
```

#### 2.2 Circuit Breaker States

```rust
#[derive(Debug, Clone)]
enum CircuitBreakerState {
    Closed {
        failure_count: u32,
        success_count: u32,
        last_failure: Option<Instant>,
    },
    Open {
        opened_at: Instant,
        failure_count: u32,
    },
    HalfOpen {
        test_requests: u32,
        start_time: Instant,
    },
}
```

**State Transitions**:
1. **Closed → Open**: Failure rate ≥ threshold (5 failures in 10 requests = 50%)
2. **Open → HalfOpen**: After timeout period (5000ms)
3. **HalfOpen → Closed**: On successful test request
4. **HalfOpen → Open**: After 3 failed test requests

### 3. Resource Management

#### 3.1 Memory Tracking

**Current Implementation** (already exists):
```rust
// Track memory usage per instance
memory_usage: usize,

// Check against limit
if let Some(memory_limit) = config.memory_limit {
    if self.memory_usage > memory_limit {
        // Instance is unhealthy
    }
}
```

**Enhancement Needed**: Actual memory measurement
- Use `jemalloc` allocation tracking
- Monitor Rust heap allocations
- Track regex compilation memory
- Track CSS selector tree memory

#### 3.2 CPU Tracking (Future Enhancement)

```rust
// Add CPU tracking fields to PooledNativeInstance
struct PooledNativeInstance {
    // ... existing fields ...
    cpu_usage_samples: VecDeque<f32>,  // Rolling window of CPU measurements
    last_cpu_check: Instant,
}

impl PooledNativeInstance {
    fn update_cpu_usage(&mut self) {
        // Use thread CPU time measurement
        // Keep rolling window of last 10 samples
        if self.cpu_usage_samples.len() >= 10 {
            self.cpu_usage_samples.pop_front();
        }
        // Add current sample
        // self.cpu_usage_samples.push_back(current_cpu);
    }

    fn avg_cpu_usage(&self) -> f32 {
        if self.cpu_usage_samples.is_empty() {
            return 0.0;
        }
        self.cpu_usage_samples.iter().sum::<f32>() / self.cpu_usage_samples.len() as f32
    }
}
```

---

## Integration Points

### 1. UnifiedExtractor Integration

**File**: `/workspaces/eventmesh/crates/riptide-extraction/src/unified_extractor.rs`

**Current Strategy** (lines 228-288):
```rust
// ALWAYS try native FIRST (regardless of variant)
let native = NativeExtractor::default();
match native.extract(html, url).await {
    Ok(content) => Ok(content),
    Err(native_err) => {
        // Fallback to WASM if available
    }
}
```

**Enhanced Strategy** (with pool):
```rust
pub enum UnifiedExtractor {
    #[cfg(feature = "wasm-extractor")]
    Wasm(WasmExtractor),

    Native(NativeExtractor),

    // NEW: Pooled native extractors
    NativePooled(Arc<NativeExtractorPool>),
}

impl UnifiedExtractor {
    pub async fn extract(&self, html: &str, url: &str) -> Result<ExtractedContent> {
        match self {
            // PRIMARY: Use pooled native extraction
            Self::NativePooled(pool) => {
                match pool.extract(html, url).await {
                    Ok(doc) => convert_to_extracted_content(doc),
                    Err(native_err) => {
                        // Fallback to WASM if configured
                        self.fallback_to_wasm(html, url, native_err).await
                    }
                }
            }

            // FALLBACK: Non-pooled native
            Self::Native(extractor) => {
                extractor.extract(html, url).await
            }

            // ENHANCEMENT: WASM as secondary strategy
            #[cfg(feature = "wasm-extractor")]
            Self::Wasm(extractor) => {
                extractor.extract(html, url).await
            }
        }
    }
}
```

### 2. AppState Integration

**File**: `/workspaces/eventmesh/crates/riptide-api/src/state.rs`

**Add to AppState**:
```rust
pub struct AppState {
    // ... existing fields ...

    /// Native CSS extractor pool
    pub native_css_pool: Arc<NativeExtractorPool>,

    /// Native Regex extractor pool
    pub native_regex_pool: Arc<NativeExtractorPool>,

    // ... rest of fields ...
}

impl AppState {
    pub async fn new_with_telemetry_and_api_config(...) -> Result<Self> {
        // ... existing initialization ...

        // Initialize native CSS pool
        let css_pool_config = NativePoolConfig::from_env_with_prefix("NATIVE_CSS_POOL");
        let native_css_pool = Arc::new(
            NativeExtractorPool::new(css_pool_config, NativeExtractorType::Css).await?
        );
        tracing::info!("Native CSS extractor pool initialized");

        // Initialize native Regex pool
        let regex_pool_config = NativePoolConfig::from_env_with_prefix("NATIVE_REGEX_POOL");
        let native_regex_pool = Arc::new(
            NativeExtractorPool::new(regex_pool_config, NativeExtractorType::Regex).await?
        );
        tracing::info!("Native Regex extractor pool initialized");

        // Wire event bus
        if let Some(event_bus) = event_bus {
            native_css_pool.set_event_bus(Arc::clone(&event_bus));
            native_regex_pool.set_event_bus(Arc::clone(&event_bus));
        }

        Ok(Self {
            // ... existing fields ...
            native_css_pool,
            native_regex_pool,
            // ... rest of fields ...
        })
    }
}
```

### 3. Extraction Pipeline Integration

**File**: `/workspaces/eventmesh/crates/riptide-api/src/pipeline_enhanced.rs`

**Enhanced Pipeline Flow**:
```rust
async fn extract_content(
    state: &AppState,
    html: &str,
    url: &str,
) -> Result<ExtractedDoc> {
    let start = Instant::now();

    // PHASE 1: Try native CSS pool (PRIMARY)
    match state.native_css_pool.extract(html, url).await {
        Ok(doc) => {
            let duration = start.elapsed();
            state.metrics.record_extraction_success("native_css", duration);
            return Ok(doc);
        }
        Err(css_err) => {
            tracing::debug!(
                url = %url,
                error = %css_err,
                "Native CSS extraction failed, trying Regex"
            );

            // PHASE 2: Try native Regex pool (FALLBACK 1)
            match state.native_regex_pool.extract(html, url).await {
                Ok(doc) => {
                    let duration = start.elapsed();
                    state.metrics.record_extraction_success("native_regex", duration);
                    return Ok(doc);
                }
                Err(regex_err) => {
                    tracing::debug!(
                        url = %url,
                        css_error = %css_err,
                        regex_error = %regex_err,
                        "Both native extractors failed, trying WASM"
                    );

                    // PHASE 3: Try WASM pool (FALLBACK 2)
                    #[cfg(feature = "wasm-pool")]
                    {
                        match state.extractor.extract(html, url).await {
                            Ok(content) => {
                                let duration = start.elapsed();
                                state.metrics.record_extraction_success("wasm_fallback", duration);
                                // Convert ExtractedContent to ExtractedDoc
                                return Ok(convert_to_doc(content));
                            }
                            Err(wasm_err) => {
                                tracing::error!(
                                    url = %url,
                                    css_error = %css_err,
                                    regex_error = %regex_err,
                                    wasm_error = %wasm_err,
                                    "All extraction strategies failed"
                                );
                                Err(anyhow!(
                                    "All strategies failed: CSS({}), Regex({}), WASM({})",
                                    css_err, regex_err, wasm_err
                                ))
                            }
                        }
                    }

                    #[cfg(not(feature = "wasm-pool"))]
                    {
                        Err(anyhow!(
                            "All native strategies failed: CSS({}), Regex({})",
                            css_err, regex_err
                        ))
                    }
                }
            }
        }
    }
}
```

---

## Migration Strategy

### Phase 1: Pool Enhancement (Current)
**Status**: Already implemented in `native_pool.rs`
- [x] Basic pool structure
- [x] Configuration management
- [x] Instance lifecycle
- [x] Health checks
- [x] Circuit breaker
- [x] Metrics collection

### Phase 2: Resource Tracking (To Be Implemented)
**Estimated Effort**: 2-3 days
- [ ] Add actual memory measurement using jemalloc stats
- [ ] Implement CPU tracking per instance
- [ ] Add resource limit enforcement
- [ ] Implement memory pressure detection
- [ ] Add memory/CPU metrics to NativePoolMetrics

### Phase 3: Pipeline Integration (To Be Implemented)
**Estimated Effort**: 3-4 days
- [ ] Update UnifiedExtractor to support pooled native
- [ ] Wire NativeExtractorPool into AppState
- [ ] Update extraction pipeline to use native pools first
- [ ] Add strategy selection logic (CSS → Regex → WASM)
- [ ] Implement graceful degradation on pool exhaustion

### Phase 4: Testing & Validation (To Be Implemented)
**Estimated Effort**: 2-3 days
- [ ] Add unit tests for pool operations
- [ ] Add integration tests for extraction pipeline
- [ ] Add benchmarks comparing native vs WASM performance
- [ ] Load testing with concurrent requests
- [ ] Circuit breaker testing under failure conditions

### Phase 5: Observability (To Be Implemented)
**Estimated Effort**: 1-2 days
- [ ] Add Prometheus metrics for native pools
- [ ] Add tracing spans for extraction operations
- [ ] Add dashboard templates for monitoring
- [ ] Document pool health indicators

---

## Performance Characteristics

### Expected Improvements

#### Latency
- **Native CSS**: ~5-10ms (vs WASM ~20-30ms)
- **Native Regex**: ~3-8ms (vs WASM ~20-30ms)
- **Pool warmup**: ~50ms for 2 instances
- **Instance creation**: ~1-2ms (vs WASM ~100-200ms)

#### Throughput
- **Native pool**: 500-1000 req/s per instance
- **WASM pool**: 200-400 req/s per instance
- **Expected 2-3x improvement** with native-first strategy

#### Memory
- **Per instance**: 256MB limit (configurable)
- **Total pool**: 8 instances × 256MB = 2GB max
- **More efficient than WASM** due to no sandbox overhead

### Resource Limits

**Default Configuration**:
```
Max pool size:        8 instances
Initial pool size:    2 instances
Memory per instance:  256MB
CPU limit per inst:   80%
Extraction timeout:   30 seconds
Health check:         30 seconds
Circuit breaker:      5 failures in 10 requests
```

**Tuning Guidelines**:
- **High volume**: Increase `max_pool_size` to 16-32
- **Low memory**: Decrease `memory_limit` to 128MB
- **Fast fail**: Decrease `extraction_timeout` to 10s
- **Strict quality**: Decrease `circuit_breaker_failure_threshold` to 3

---

## Files to Modify

### 1. New File (Already Exists)
- ✅ `/workspaces/eventmesh/crates/riptide-pool/src/native_pool.rs` - Main implementation

### 2. Modify Existing Files

#### `/workspaces/eventmesh/crates/riptide-pool/src/lib.rs`
**Changes**:
- Add `pub use native_pool::*;` export
- Update module documentation

#### `/workspaces/eventmesh/crates/riptide-pool/src/pool.rs`
**Changes**:
- Remove native fallback logic (lines 289-324)
- Add reference to NativeExtractorPool for fallback
- Update comments to reflect native-first strategy

#### `/workspaces/eventmesh/crates/riptide-api/src/state.rs`
**Changes**:
- Add `native_css_pool: Arc<NativeExtractorPool>` field
- Add `native_regex_pool: Arc<NativeExtractorPool>` field
- Initialize pools in `new_with_telemetry_and_api_config`
- Wire event bus to native pools

#### `/workspaces/eventmesh/crates/riptide-extraction/src/unified_extractor.rs`
**Changes**:
- Add `NativePooled` variant to enum
- Update `extract` method to prioritize pooled native
- Add fallback logic: Native → WASM
- Update documentation to reflect native-first strategy

#### `/workspaces/eventmesh/crates/riptide-api/src/pipeline_enhanced.rs`
**Changes**:
- Update extraction phase to use native pools
- Add cascading fallback: CSS → Regex → WASM
- Add metrics for each strategy
- Update error handling for multi-strategy failures

### 3. Configuration Files

#### Add to `.env.example`:
```bash
# Native CSS Pool Configuration
NATIVE_CSS_POOL_MAX_SIZE=8
NATIVE_CSS_POOL_INITIAL_SIZE=2
NATIVE_CSS_POOL_EXTRACTION_TIMEOUT_MS=30000
NATIVE_CSS_POOL_MEMORY_LIMIT_BYTES=268435456

# Native Regex Pool Configuration
NATIVE_REGEX_POOL_MAX_SIZE=8
NATIVE_REGEX_POOL_INITIAL_SIZE=2
NATIVE_REGEX_POOL_EXTRACTION_TIMEOUT_MS=30000
NATIVE_REGEX_POOL_MEMORY_LIMIT_BYTES=268435456
```

---

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pool_creation() {
        let config = NativePoolConfig::default();
        let pool = NativeExtractorPool::new(config, NativeExtractorType::Css)
            .await
            .unwrap();

        let (available, active, max) = pool.get_pool_status().await;
        assert_eq!(available, 2); // initial_pool_size
        assert_eq!(active, 0);
        assert_eq!(max, 8);
    }

    #[tokio::test]
    async fn test_extraction_success() {
        let config = NativePoolConfig::default();
        let pool = NativeExtractorPool::new(config, NativeExtractorType::Css)
            .await
            .unwrap();

        let html = r#"<html><body><h1>Test</h1><p>Content</p></body></html>"#;
        let result = pool.extract(html, "https://example.com").await;

        assert!(result.is_ok());
        let doc = result.unwrap();
        assert!(doc.title.is_some());
    }

    #[tokio::test]
    async fn test_circuit_breaker() {
        let mut config = NativePoolConfig::default();
        config.circuit_breaker_failure_threshold = 2; // Trip after 2 failures

        let pool = NativeExtractorPool::new(config, NativeExtractorType::Css)
            .await
            .unwrap();

        // Trigger failures
        for _ in 0..3 {
            let _ = pool.extract("invalid html", "https://example.com").await;
        }

        let metrics = pool.get_metrics().await;
        assert!(metrics.circuit_breaker_trips > 0);
    }

    #[tokio::test]
    async fn test_instance_health_check() {
        let config = NativePoolConfig {
            max_instance_reuse: 5,
            ..Default::default()
        };

        let mut instance = PooledNativeInstance::new(NativeExtractorType::Css);

        // Use instance multiple times
        for _ in 0..6 {
            instance.record_success();
        }

        // Should be unhealthy after exceeding reuse limit
        assert!(!instance.is_healthy(&config));
    }
}
```

### Integration Tests
```rust
#[cfg(test)]
mod integration_tests {
    #[tokio::test]
    async fn test_native_first_fallback_to_wasm() {
        // Setup: Create extraction pipeline with native pools
        // Test: Submit request that fails native CSS but succeeds with Regex
        // Verify: Metrics show CSS failure, Regex success
    }

    #[tokio::test]
    async fn test_concurrent_extractions() {
        // Setup: Create pool with max_size=4
        // Test: Submit 10 concurrent requests
        // Verify: All succeed, semaphore properly limits concurrency
    }

    #[tokio::test]
    async fn test_pool_exhaustion_graceful_degradation() {
        // Setup: Create pool with max_size=2
        // Test: Submit 5 concurrent requests
        // Verify: First 2 succeed immediately, others wait or fallback
    }
}
```

### Load Tests (Performance Benchmarks)
```bash
# Benchmark native CSS vs WASM
cargo bench --bench extraction_comparison

# Load test with different pool sizes
artillery run loadtest-native-pool.yml

# Memory profiling
heaptrack ./target/release/riptide-api
```

---

## Monitoring & Observability

### Metrics (Prometheus)

```rust
// Pool-level metrics
native_pool_extractions_total{strategy="css", status="success|failure"}
native_pool_extraction_duration_seconds{strategy="css", quantile="0.5|0.95|0.99"}
native_pool_instances_total{strategy="css", state="available|active"}
native_pool_circuit_breaker_state{strategy="css"} // 0=closed, 1=open, 2=half-open
native_pool_semaphore_wait_seconds{strategy="css"}

// Instance-level metrics
native_pool_instance_use_count{instance_id="..."}
native_pool_instance_failure_count{instance_id="..."}
native_pool_instance_memory_bytes{instance_id="..."}
```

### Logging (Structured with tracing)

```rust
// Pool lifecycle
tracing::info!(
    pool_id = %pool.pool_id,
    extractor_type = ?extractor_type,
    max_pool_size = config.max_pool_size,
    "Native extractor pool initialized"
);

// Extraction events
tracing::debug!(
    url = %url,
    strategy = "native_css",
    duration_ms = ?duration.as_millis(),
    "Content extracted successfully"
);

// Circuit breaker events
tracing::warn!(
    pool_id = %pool.pool_id,
    failure_rate = %failure_rate,
    "Circuit breaker opened"
);

// Health check events
tracing::debug!(
    instance_id = %instance.id,
    use_count = instance.use_count,
    memory_usage = instance.memory_usage,
    "Instance health check: unhealthy"
);
```

### Dashboards (Grafana)

**Native Pool Overview**:
- Extraction rate (req/s) by strategy
- Success rate (%) by strategy
- P50/P95/P99 latency by strategy
- Pool utilization (available vs active instances)
- Circuit breaker state timeline

**Resource Usage**:
- Memory usage per instance
- CPU usage per instance (future)
- Instance creation/destruction rate
- Timeout rate

**Comparison View**:
- Native CSS vs Native Regex vs WASM performance
- Strategy fallback frequency
- Error distribution by strategy

---

## Security Considerations

### 1. Resource Exhaustion
- **Mitigation**: Hard limits on pool size, memory, CPU
- **Monitoring**: Alert on pool exhaustion, high memory usage
- **Recovery**: Circuit breaker prevents cascading failures

### 2. Regex DoS (ReDoS)
- **Mitigation**: Timeout on regex extraction (30s default)
- **Monitoring**: Track timeout frequency by pattern
- **Recovery**: Discard instances with repeated timeouts

### 3. Memory Leaks
- **Mitigation**: Max reuse count (1000 operations per instance)
- **Monitoring**: Track memory growth per instance
- **Recovery**: Automatic instance recreation on health check failure

### 4. Denial of Service
- **Mitigation**: Semaphore limits concurrent extractions
- **Monitoring**: Track semaphore wait times
- **Recovery**: Graceful degradation with error responses

---

## Future Enhancements

### Short-term (Next Sprint)
1. **Actual CPU tracking** using thread CPU time
2. **Memory profiling** using jemalloc stats
3. **Auto-scaling** based on request rate
4. **Pattern caching** for regex compilation

### Medium-term (Next Quarter)
1. **ML-based strategy selection** (learn which strategy works best for which sites)
2. **Adaptive pool sizing** based on traffic patterns
3. **Instance affinity** (prefer instances that recently succeeded for similar URLs)
4. **Compression** of extracted content before returning

### Long-term (Roadmap)
1. **Distributed pooling** across multiple nodes
2. **Cross-pool instance borrowing** (CSS pool can borrow from Regex pool)
3. **Predictive pre-warming** based on traffic forecasts
4. **Custom extractor plugins** (user-defined extraction strategies)

---

## Conclusion

This architecture provides **native extraction with first-class pooling support**, reversing the current WASM-primary approach. The design prioritizes:

1. **Performance**: Native extractors are faster and more resource-efficient
2. **Reliability**: Circuit breaker, health monitoring, graceful degradation
3. **Observability**: Comprehensive metrics and logging
4. **Maintainability**: Clear separation of concerns, well-documented

**Implementation Status**: Pool core is implemented, needs resource tracking enhancements and pipeline integration.

**Next Steps**:
1. Implement actual memory/CPU tracking
2. Wire native pools into AppState
3. Update extraction pipeline for native-first strategy
4. Add comprehensive testing
5. Deploy and monitor in staging environment

---

## Appendix A: Comparison with Existing Pools

| Feature | WASM Pool | Browser Pool | Native Pool (NEW) |
|---------|-----------|--------------|-------------------|
| **Instance Type** | WASM component | Browser process | Native Rust struct |
| **Startup Time** | 100-200ms | 2-3s | 1-2ms |
| **Memory per Instance** | 512MB | 500MB | 256MB |
| **Extraction Speed** | 20-30ms | 100-200ms | 5-10ms |
| **Pool Size** | 8 instances | 20 instances | 8 instances |
| **Circuit Breaker** | ✅ Yes | ❌ No | ✅ Yes |
| **Health Monitoring** | ✅ Yes | ✅ Yes | ✅ Yes |
| **Event Integration** | ✅ Yes | ❌ No | ✅ Yes |
| **Resource Limits** | ✅ Yes | ✅ Yes | ✅ Yes |

**Conclusion**: Native pool should be **PRIMARY** due to superior performance characteristics.

---

## Appendix B: Configuration Reference

### Complete Configuration Example

```rust
NativePoolConfig {
    max_pool_size: 8,
    initial_pool_size: 2,
    extraction_timeout: 30000,
    health_check_interval: 30000,
    memory_limit: Some(256 * 1024 * 1024),
    cpu_limit: Some(80.0),
    circuit_breaker_failure_threshold: 5,
    circuit_breaker_timeout: 5000,
    max_instance_reuse: 1000,
    max_failure_count: 10,
}
```

### Environment Variable Matrix

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `NATIVE_POOL_MAX_SIZE` | usize | 8 | Maximum pool instances |
| `NATIVE_POOL_INITIAL_SIZE` | usize | 2 | Initial warm-up instances |
| `NATIVE_POOL_EXTRACTION_TIMEOUT_MS` | u64 | 30000 | Per-extraction timeout |
| `NATIVE_POOL_HEALTH_CHECK_INTERVAL_MS` | u64 | 30000 | Health check frequency |
| `NATIVE_POOL_MEMORY_LIMIT_BYTES` | usize | 268435456 | Per-instance memory cap |
| `NATIVE_POOL_CPU_LIMIT_PCT` | f32 | 80.0 | Per-instance CPU limit |
| `NATIVE_POOL_CIRCUIT_BREAKER_THRESHOLD` | u32 | 5 | Failure count to trip |
| `NATIVE_POOL_CIRCUIT_BREAKER_TIMEOUT_MS` | u64 | 5000 | Open state duration |
| `NATIVE_POOL_MAX_INSTANCE_REUSE` | u32 | 1000 | Max uses per instance |
| `NATIVE_POOL_MAX_FAILURE_COUNT` | u32 | 10 | Max failures per instance |

---

**Document Version**: 1.0
**Last Updated**: 2025-11-01
**Status**: Ready for Implementation (Phase 2-5)
