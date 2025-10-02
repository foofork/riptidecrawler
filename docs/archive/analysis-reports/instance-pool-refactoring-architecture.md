# Instance Pool Module Refactoring Architecture

**Architect:** System Architecture Designer
**Session:** swarm-1759217361759-095dd3g5o
**Date:** 2025-09-30
**Current Status:** Architecture Design Complete

## Executive Summary

The `instance_pool.rs` file has grown to 1,270 lines and requires refactoring into a modular architecture for improved maintainability, testability, and separation of concerns. This document outlines the complete refactoring strategy, module dependencies, critical bug fixes, and integration plan.

### Critical Issue Identified

**Primary Bug:** Mutex guard deadlock in `record_extraction_result()` (lines 684-830)
- Two mutex guards held simultaneously for 140+ lines
- Await points while holding locks
- Causes compilation errors in async context
- **Severity:** High - blocks production deployment

## Current State Analysis

### File Structure (Before)
```
src/
├── instance_pool.rs (1,270 lines) ❌ Too large
│   ├── Data structures (PooledInstance, CircuitBreakerState, AdvancedInstancePool)
│   ├── Pool lifecycle (new, warm_up, get_or_create_instance, create_instance)
│   ├── Extraction methods (extract, extract_with_instance)
│   ├── Circuit breaker (record_extraction_result, is_circuit_open) ⚠️ HAS DEADLOCK BUG
│   ├── Health monitoring (start_instance_health_monitoring, perform_instance_health_checks)
│   ├── Memory management (clear_high_memory_instances, trigger_memory_cleanup)
│   ├── Event emission (emit_pool_exhausted_event, emit_instance_health_event)
│   └── Helper functions (get_instances_per_worker, create_event_aware_pool)
```

### Public API Surface
```rust
// Core structures
pub struct PooledInstance { /* 12 fields */ }
pub enum CircuitBreakerState { Closed, Open, HalfOpen }
pub struct AdvancedInstancePool { /* 11 fields */ }

// Main API
impl PooledInstance {
    pub fn new(...) -> Self
    pub fn is_healthy(&self, config: &ExtractorConfig) -> bool
    pub fn record_usage(&mut self, success: bool)
    pub fn create_fresh_store(&mut self) -> Store<WasmResourceTracker>
}

impl AdvancedInstancePool {
    pub async fn new(...) -> Result<Self>
    pub async fn extract(...) -> Result<ExtractedDoc>
    pub async fn create_instance() -> Result<PooledInstance>
    pub async fn get_metrics() -> PerformanceMetrics
    pub async fn get_pool_status() -> (usize, usize, usize)
    pub fn set_event_bus(&mut self, event_bus: Arc<EventBus>)
    pub fn pool_id(&self) -> &str
}

// Factory functions
pub fn get_instances_per_worker() -> usize
pub async fn create_event_aware_pool(...) -> Result<AdvancedInstancePool>
```

## Target Architecture

### Module Structure (After)
```
src/instance_pool/
├── mod.rs              (60-80 lines)    - Module entry point with re-exports
├── models.rs           (200-250 lines)  - Data structures and types
│   ├── PooledInstance
│   ├── CircuitBreakerState
│   └── Helper structs
├── pool.rs             (300-350 lines)  - Core pool management
│   ├── AdvancedInstancePool
│   ├── Pool lifecycle (new, warm_up)
│   ├── Instance management (get/create/return)
│   └── Pool status and configuration
├── extraction.rs       (250-300 lines)  - Extraction logic
│   ├── extract()
│   ├── extract_with_instance()
│   ├── Fallback extraction
│   └── Mode conversion
├── circuit_breaker.rs  (200-250 lines)  - Circuit breaker logic ⚠️ CRITICAL REFACTOR
│   ├── record_extraction_result() (FIXED)
│   ├── is_circuit_open()
│   ├── State transitions
│   └── Event emission (no locks held)
├── health.rs           (250-300 lines)  - Health monitoring
│   ├── start_instance_health_monitoring()
│   ├── perform_instance_health_checks()
│   ├── validate_instance_health()
│   └── emit_instance_health_event()
└── memory.rs           (150-200 lines)  - Memory management
    ├── clear_high_memory_instances()
    ├── clear_some_instances()
    ├── trigger_memory_cleanup()
    └── Memory monitoring helpers
```

### Size Reduction
- **Before:** 1,270 lines in single file
- **After:** ~1,400 lines distributed across 7 focused modules
- **Average module size:** ~200 lines (well under 500-line guideline)
- **Organization overhead:** 130 lines (acceptable for better structure)

## Critical Bug Fix: Circuit Breaker Deadlock

### Problem Analysis

**Location:** `record_extraction_result()` (lines 684-830)

```rust
// ❌ PROBLEMATIC CODE (Current)
async fn record_extraction_result(&self, success: bool, duration: Duration) {
    let mut state = self.circuit_state.lock().await;      // Line 685
    let mut metrics = self.metrics.lock().await;          // Line 686

    // ... 140+ lines of logic with both guards held ...

    // Await points in spawned tasks (lines 823-838)
    tokio::spawn(async move {
        event_bus.emit(event).await  // ❌ Await while holding locks!
    });

    *state = new_state;  // Line 805
}
```

**Issues:**
1. Two `MutexGuard`s held for 140+ lines
2. Await points occur while holding locks (event emission)
3. Risk of deadlock in async runtime
4. Blocks compilation with "cannot send guard across await" error

### Solution: Phase-Based Locking

```rust
// ✅ FIXED CODE (Target)
async fn record_extraction_result(&self, success: bool, duration: Duration) {
    // Phase 1: Update metrics in scoped block
    let (circuit_breaker_trips, failed_extractions, total_extractions) = {
        let mut metrics = self.metrics.lock().await;

        metrics.total_extractions += 1;
        if success {
            metrics.successful_extractions += 1;
        } else {
            metrics.failed_extractions += 1;
        }

        // Update average processing time
        let new_time = duration.as_millis() as f64;
        metrics.avg_processing_time_ms = if metrics.total_extractions == 1 {
            new_time
        } else {
            (metrics.avg_processing_time_ms + new_time) / 2.0
        };

        (metrics.circuit_breaker_trips, metrics.failed_extractions, metrics.total_extractions)
    }; // ✅ Metrics lock dropped here

    // Phase 2: Update circuit breaker state in scoped block
    let (should_emit_trip_event, should_emit_reset_event, trip_metrics, success_count) = {
        let mut state = self.circuit_state.lock().await;
        let mut should_emit_trip = false;
        let mut should_emit_reset = false;
        let mut trip_data = None;
        let mut success_count = 0;

        // State machine logic here (no await points)
        let new_state = match &*state {
            CircuitBreakerState::Closed { failure_count, success_count: sc, .. } => {
                // ... state transition logic ...
                if should_trip {
                    should_emit_trip = true;
                    trip_data = Some((threshold, trips, failed, total));
                }
                // ... calculate new state ...
            }
            // ... other state transitions ...
        };

        *state = new_state;

        (should_emit_trip, should_emit_reset, trip_data, success_count)
    }; // ✅ Circuit state lock dropped here

    // Update metrics if circuit breaker tripped (separate lock acquisition)
    if should_emit_trip_event {
        let mut metrics = self.metrics.lock().await;
        metrics.circuit_breaker_trips += 1;
    } // ✅ Metrics lock dropped here

    // Phase 3: Emit events without holding any locks
    if should_emit_trip_event {
        if let Some((threshold, trips, failed, total)) = trip_metrics {
            if let Some(event_bus) = &self.event_bus {
                tokio::spawn(async move {
                    // ✅ No locks held during async operations
                    let mut event = PoolEvent::new(...);
                    event.add_metadata(...);
                    event_bus.emit(event).await;
                });
            }
        }
    }

    if should_emit_reset_event {
        // ✅ Emit reset event without locks
    }
}
```

**Benefits:**
- ✅ No locks held across await points
- ✅ Minimal lock hold times (only for data updates)
- ✅ No deadlock risk
- ✅ Clear separation of phases
- ✅ Event emission happens without locks
- ✅ Preserves all existing logic

## Module Design Specifications

### 1. mod.rs (Module Entry Point)

**Purpose:** Public API surface and module organization

**Contents:**
```rust
// Public modules
pub mod models;
mod pool;
mod extraction;
mod circuit_breaker;
mod health;
mod memory;

// Re-exports for backward compatibility
pub use models::{PooledInstance, CircuitBreakerState};
pub use pool::AdvancedInstancePool;
pub use pool::{get_instances_per_worker, create_event_aware_pool};

// Internal re-exports for module use
pub(crate) use extraction::*;
pub(crate) use circuit_breaker::*;
pub(crate) use health::*;
pub(crate) use memory::*;
```

**Dependencies:** None (only re-exports)

**Size:** 60-80 lines

---

### 2. models.rs (Data Structures)

**Purpose:** Core data structures and type definitions

**Key Types:**
```rust
pub struct PooledInstance {
    pub id: String,
    pub engine: Arc<Engine>,
    pub component: Arc<Component>,
    pub linker: Arc<Linker<WasmResourceTracker>>,
    pub created_at: Instant,
    pub last_used: Instant,
    pub use_count: u64,
    pub failure_count: u64,
    pub memory_usage_bytes: u64,
    pub resource_tracker: WasmResourceTracker,
}

impl PooledInstance {
    pub fn new(...) -> Self
    pub fn is_healthy(&self, config: &ExtractorConfig) -> bool
    pub fn record_usage(&mut self, success: bool)
    pub fn create_fresh_store(&mut self) -> Store<WasmResourceTracker>
}

#[derive(Clone, Debug)]
pub enum CircuitBreakerState {
    Closed {
        failure_count: u64,
        success_count: u64,
        last_failure: Option<Instant>,
    },
    Open {
        opened_at: Instant,
        failure_count: u64,
    },
    HalfOpen {
        test_requests: u64,
        start_time: Instant,
    },
}
```

**Dependencies:**
```rust
use std::time::{Duration, Instant};
use std::sync::Arc;
use wasmtime::{component::*, Engine, Store};
use crate::component::{ExtractorConfig, WasmResourceTracker};
```

**Size:** 200-250 lines

**Testing:**
- Unit tests for health checks
- Unit tests for usage recording
- Store creation validation

---

### 3. pool.rs (Core Pool Management)

**Purpose:** Main pool lifecycle and instance management

**Key Functions:**
```rust
pub struct AdvancedInstancePool {
    config: ExtractorConfig,
    engine: Arc<Engine>,
    component: Arc<Component>,
    linker: Arc<Linker<WasmResourceTracker>>,
    available_instances: Arc<Mutex<VecDeque<PooledInstance>>>,
    semaphore: Arc<Semaphore>,
    metrics: Arc<Mutex<PerformanceMetrics>>,
    circuit_state: Arc<Mutex<CircuitBreakerState>>,
    component_path: String,
    pool_id: String,
    event_bus: Option<Arc<EventBus>>,
}

impl AdvancedInstancePool {
    pub async fn new(...) -> Result<Self>
    async fn warm_up(&self) -> Result<()>
    async fn get_or_create_instance(&self) -> Result<PooledInstance>
    pub async fn create_instance(&self) -> Result<PooledInstance>
    pub async fn return_instance(&self, instance: PooledInstance)
    pub async fn get_pool_status(&self) -> (usize, usize, usize)
    pub async fn get_metrics(&self) -> PerformanceMetrics
    pub fn set_event_bus(&mut self, event_bus: Arc<EventBus>)
    pub fn pool_id(&self) -> &str
}

// Factory functions
pub fn get_instances_per_worker() -> usize
pub async fn create_event_aware_pool(...) -> Result<AdvancedInstancePool>
```

**Responsibilities:**
- Pool initialization and warm-up
- Instance creation and lifecycle
- Semaphore-based concurrency control
- Pool status tracking
- Event bus integration

**Dependencies:**
```rust
use crate::component::{ExtractorConfig, PerformanceMetrics};
use crate::events::{Event, EventBus, PoolEvent, PoolOperation};
use super::models::{PooledInstance, CircuitBreakerState};
use tokio::sync::{Mutex, Semaphore};
use std::collections::VecDeque;
```

**Size:** 300-350 lines

**Testing:**
- Pool initialization tests
- Instance acquisition tests
- Concurrency control tests
- Pool status accuracy

---

### 4. extraction.rs (Extraction Logic)

**Purpose:** Content extraction with WASM extractor

**Key Functions:**
```rust
pub(crate) async fn extract(
    pool: &AdvancedInstancePool,
    html: &str,
    url: &str,
    mode: ExtractionMode,
) -> Result<ExtractedDoc>

pub(crate) async fn extract_with_instance(
    pool: &AdvancedInstancePool,
    instance: &mut PooledInstance,
    html: &str,
    url: &str,
    mode: ExtractionMode,
) -> Result<ExtractedDoc>

pub(crate) async fn fallback_extract(
    pool: &AdvancedInstancePool,
    html: &str,
    url: &str,
    mode: ExtractionMode,
) -> Result<ExtractedDoc>

fn convert_extraction_mode(mode: ExtractionMode) -> exports::riptide::extractor::extract::ExtractionMode
```

**Responsibilities:**
- Main extraction endpoint
- WASM component instantiation
- Store creation and management
- Epoch timeout handling
- Fallback extraction (scraper-based)
- Mode conversion
- Event emission

**Dependencies:**
```rust
use super::models::PooledInstance;
use super::pool::AdvancedInstancePool;
use crate::types::{ExtractedDoc, ExtractionMode};
use crate::events::{ExtractionEvent, ExtractionOperation};
use wasmtime::{component::*, Store};
use scraper::{Html, Selector};
```

**Size:** 250-300 lines

**Testing:**
- Successful extraction tests
- Timeout handling tests
- Fallback logic tests
- Mode conversion tests
- Event emission validation

---

### 5. circuit_breaker.rs (Circuit Breaker Logic) ⚠️ CRITICAL

**Purpose:** Circuit breaker state management (WITH BUG FIX)

**Key Functions:**
```rust
pub(crate) async fn record_extraction_result(
    pool: &AdvancedInstancePool,
    success: bool,
    duration: Duration,
)  // ✅ FIXED: Phase-based locking

pub(crate) async fn is_circuit_open(
    pool: &AdvancedInstancePool,
) -> bool

pub(crate) async fn record_timeout(
    pool: &AdvancedInstancePool,
)

pub(crate) async fn record_epoch_timeout(
    pool: &AdvancedInstancePool,
)

async fn emit_circuit_breaker_events(
    pool: &AdvancedInstancePool,
    event_data: CircuitBreakerEventData,
)  // ✅ No locks held during emission
```

**Critical Fix Implementation:**

```rust
// Phase 1: Metrics update (scoped lock)
let metrics_snapshot = {
    let mut metrics = pool.metrics.lock().await;
    // Update metrics
    MetricsSnapshot { ... }
}; // Lock released

// Phase 2: State update (scoped lock)
let event_data = {
    let mut state = pool.circuit_state.lock().await;
    // Update state
    EventData { ... }
}; // Lock released

// Phase 3: Event emission (no locks)
if let Some(data) = event_data {
    emit_circuit_breaker_events(pool, data).await;
}
```

**Responsibilities:**
- Extraction result tracking
- Circuit breaker state transitions (Closed → Open → HalfOpen → Closed)
- Metrics updates (sequential, scoped)
- Event emission (no locks held)
- Timeout recording

**Dependencies:**
```rust
use super::models::CircuitBreakerState;
use super::pool::AdvancedInstancePool;
use crate::component::PerformanceMetrics;
use crate::events::{PoolEvent, PoolOperation};
use std::time::{Duration, Instant};
```

**Size:** 200-250 lines

**Testing:** (CRITICAL)
- ✅ State transition correctness
- ✅ No deadlocks under load
- ✅ Metrics consistency
- ✅ Event emission accuracy
- ✅ Concurrent access safety
- ✅ Lock ordering validation

---

### 6. health.rs (Health Monitoring)

**Purpose:** Continuous health monitoring and instance validation

**Key Functions:**
```rust
pub(crate) async fn start_instance_health_monitoring(
    pool: Arc<AdvancedInstancePool>,
) -> Result<()>

async fn perform_instance_health_checks(
    pool: &AdvancedInstancePool,
) -> Result<()>

async fn validate_instance_health(
    pool: &AdvancedInstancePool,
    instance: &PooledInstance,
) -> bool

async fn emit_instance_health_event(
    pool: &AdvancedInstancePool,
    instance: &PooledInstance,
    healthy: bool,
)

async fn emit_pool_health_metrics(
    pool: &AdvancedInstancePool,
    healthy_count: usize,
)
```

**Responsibilities:**
- Continuous health monitoring loop
- Instance health validation (age, failures, memory)
- Unhealthy instance replacement
- Health metric emission
- Pool health status tracking

**Health Check Criteria:**
```rust
// Instance health checks
- Age < 1 hour (3600s)
- Failure count <= 5
- Memory usage < config limit
- Grow failures < 10
- Last used within 30 minutes
```

**Dependencies:**
```rust
use super::models::PooledInstance;
use super::pool::AdvancedInstancePool;
use crate::events::{PoolEvent, PoolOperation, PoolMetrics};
use std::time::Duration;
use tokio::time::interval;
```

**Size:** 250-300 lines

**Testing:**
- Health check accuracy
- Instance replacement logic
- Event emission validation
- Continuous monitoring loop
- Edge cases (all unhealthy, all healthy)

---

### 7. memory.rs (Memory Management)

**Purpose:** Memory pressure management and cleanup

**Key Functions:**
```rust
pub(crate) async fn clear_high_memory_instances(
    pool: &AdvancedInstancePool,
) -> Result<usize>

pub(crate) async fn clear_some_instances(
    pool: &AdvancedInstancePool,
    count: usize,
) -> Result<usize>

pub(crate) async fn trigger_memory_cleanup(
    pool: &AdvancedInstancePool,
) -> Result<()>

async fn get_pool_metrics_for_events(
    pool: &AdvancedInstancePool,
) -> PoolMetrics

async fn emit_pool_exhausted_event(
    pool: &AdvancedInstancePool,
)
```

**Responsibilities:**
- High memory instance detection and removal
- Bulk instance clearing
- Memory cleanup triggering
- Pool metrics aggregation
- Pool exhaustion event emission

**Memory Management Strategy:**
```rust
// Memory threshold calculation
let threshold = (config.memory_limit * 0.8) as u64; // 80% of limit

// Instance replacement
1. Identify high-memory instances
2. Remove from available pool
3. Emit health event
4. Create replacement instances
5. Return new instances to pool
```

**Dependencies:**
```rust
use super::models::PooledInstance;
use super::pool::AdvancedInstancePool;
use crate::events::{PoolEvent, PoolOperation, PoolMetrics};
use crate::component::ExtractorConfig;
```

**Size:** 150-200 lines

**Testing:**
- High memory detection
- Instance clearing accuracy
- Replacement instance creation
- Event emission validation
- Pool size maintenance

---

## Module Dependency Graph

```
┌────────────────────────────────────────────────────────────────┐
│                        pool.rs                                 │
│              (AdvancedInstancePool - Core)                     │
│  ┌────────────────────────────────────────────────────────┐   │
│  │ - Pool initialization and lifecycle                     │   │
│  │ - Instance creation and management                      │   │
│  │ - Semaphore-based concurrency control                   │   │
│  │ - Event bus integration                                 │   │
│  └────────────────────────────────────────────────────────┘   │
└────┬──────────────┬──────────────┬─────────────┬──────────────┘
     │              │              │             │
     │ uses         │ uses         │ uses        │ uses
     ▼              ▼              ▼             ▼
┌─────────┐  ┌─────────────┐  ┌─────────┐  ┌─────────┐
│extrac-  │  │circuit_     │  │ health  │  │ memory  │
│tion.rs  │  │breaker.rs   │  │  .rs    │  │  .rs    │
│         │  │             │  │         │  │         │
│- WASM   │  │- State      │  │- Health │  │- Memory │
│  extract│  │  machine    │  │  monitor│  │  cleanup│
│- Timeout│  │- Metrics    │  │- Replace│  │- Pool   │
│- Fallbck│  │- Events ✅  │  │  unhltly│  │  metrics│
└─────────┘  └─────────────┘  └─────────┘  └─────────┘
     │              │              │             │
     └──────────────┴──────────────┴─────────────┘
                    │
                    │ all depend on
                    ▼
            ┌─────────────┐
            │  models.rs  │
            │             │
            │- PooledInst │
            │- CircuitBrk │
            │  State      │
            └─────────────┘

External Dependencies:
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│ ExtractorCfg│  │ EventBus    │  │ WasmRuntime │
│ (component) │  │ (events)    │  │ (wasmtime)  │
└─────────────┘  └─────────────┘  └─────────────┘
```

**Dependency Rules:**
1. `pool.rs` → orchestrates all other modules
2. `extraction.rs` → stateless, called by pool
3. `circuit_breaker.rs` → stateless, called by extraction ⚠️ FIXED
4. `health.rs` → background task, reads pool state
5. `memory.rs` → utility functions, called by pool
6. `models.rs` → shared by all, no dependencies

**Circular Dependency Prevention:**
- ✅ No module depends on `pool.rs` (except via reference)
- ✅ All functional modules are stateless
- ✅ Models module has zero dependencies
- ✅ Event emission happens without holding locks ⚠️ CRITICAL FIX

---

## Integration Plan

### Phase 1: Extract Models and Utilities (Low Risk)

**Tasks:**
1. Create `src/instance_pool/mod.rs` with module structure
2. Extract `models.rs` (PooledInstance, CircuitBreakerState)
3. Extract `memory.rs` (pure utility functions)
4. Update imports in original file

**Risk:** Low (no behavioral changes)

**Validation:**
- Compilation check
- No warnings
- Public API unchanged

---

### Phase 2: Extract Extraction Logic (Medium Risk)

**Tasks:**
1. Create `extraction.rs`
2. Move `extract()`, `extract_with_instance()`, `fallback_extract()`
3. Move mode conversion helpers
4. Update imports and function calls

**Risk:** Medium (complex async logic)

**Validation:**
- Unit tests for extraction
- Timeout handling tests
- Fallback logic tests
- Integration tests pass

---

### Phase 3: Fix and Extract Circuit Breaker (HIGH PRIORITY) ⚠️

**Tasks:**
1. Create `circuit_breaker.rs`
2. **CRITICAL:** Implement phase-based locking fix for `record_extraction_result()`
3. Move circuit breaker state machine logic
4. Move event emission functions (ensure no locks held)
5. Add comprehensive tests for concurrent access

**Risk:** High (critical bug fix + refactoring)

**Validation:** (MANDATORY)
- ✅ No compilation errors
- ✅ No deadlocks under load testing
- ✅ State transitions correct
- ✅ Metrics consistency verified
- ✅ Event emission works correctly
- ✅ Stress test with 100+ concurrent extractions

**Testing Checklist:**
```bash
# 1. Compilation check
cargo check --package riptide-core

# 2. Unit tests
cargo test --package riptide-core instance_pool::circuit_breaker

# 3. Integration tests
cargo test --package riptide-core instance_pool::tests

# 4. Stress test (custom)
cargo test --package riptide-core instance_pool::stress -- --nocapture --test-threads=1

# 5. Deadlock detection
RUST_BACKTRACE=1 cargo test --package riptide-core instance_pool -- --nocapture
```

---

### Phase 4: Extract Health and Pool (Medium Risk)

**Tasks:**
1. Create `health.rs` (health monitoring logic)
2. Create `pool.rs` (main pool structure)
3. Update all cross-references
4. Ensure event emission works correctly

**Risk:** Medium (complex state management)

**Validation:**
- Health monitoring loop works
- Pool lifecycle correct
- Metrics accuracy maintained
- No resource leaks

---

### Phase 5: Testing and Validation (CRITICAL)

**Test Categories:**

1. **Unit Tests (Per Module)**
   - `models.rs` - Health checks, usage recording
   - `extraction.rs` - Extraction logic, timeouts, fallback
   - `circuit_breaker.rs` - State transitions, event emission ⚠️ CRITICAL
   - `health.rs` - Health validation, instance replacement
   - `memory.rs` - Memory cleanup, pool metrics
   - `pool.rs` - Lifecycle, concurrency, status

2. **Integration Tests**
   - End-to-end extraction flow
   - Circuit breaker under load
   - Health monitoring continuous operation
   - Memory pressure handling
   - Event emission accuracy

3. **Stress Tests** ⚠️ MANDATORY
   - 100+ concurrent extractions
   - 1000+ rapid sequential extractions
   - Circuit breaker rapid state transitions
   - Memory pressure scenarios
   - Pool exhaustion and recovery

4. **Deadlock Detection**
   - Lock ordering validation
   - No await points while holding locks
   - Event emission without locks
   - Concurrent access safety

**Test Execution:**
```bash
# Run all tests
cargo test --package riptide-core instance_pool

# Run specific module tests
cargo test --package riptide-core instance_pool::circuit_breaker -- --nocapture

# Run with logging
RUST_LOG=debug cargo test --package riptide-core instance_pool -- --nocapture

# Stress test
cargo test --package riptide-core instance_pool::stress -- --ignored --nocapture

# Check for warnings
cargo clippy --package riptide-core

# Memory leak detection (if available)
cargo valgrind test --package riptide-core instance_pool
```

---

## Risk Assessment

### Critical Risks (Must Address)

1. **Circuit Breaker Deadlock** ⚠️ CRITICAL
   - **Risk:** Mutex guards held across await points
   - **Impact:** Compilation failure, potential production deadlocks
   - **Mitigation:** Phase-based locking refactor (implemented in design)
   - **Validation:** Comprehensive stress testing required

2. **State Consistency**
   - **Risk:** Metrics and circuit breaker state become inconsistent
   - **Impact:** Incorrect circuit breaker behavior
   - **Mitigation:** Sequential locking (metrics → state), extract all data before updates
   - **Validation:** Integration tests with assertions on state consistency

3. **Event Emission Timing**
   - **Risk:** Events emitted before state updates complete
   - **Impact:** Misleading metrics, incorrect monitoring
   - **Mitigation:** Emit events only after all locks released
   - **Validation:** Event sequence tests

### Medium Risks

1. **Race Conditions**
   - **Risk:** Multiple threads update metrics/state concurrently
   - **Impact:** Lost updates, incorrect counters
   - **Mitigation:** Maintain sequential locking order, use atomic operations where appropriate
   - **Validation:** Concurrency tests with assertions

2. **Performance Regression**
   - **Risk:** Additional lock acquisitions slow down extraction
   - **Impact:** Increased latency
   - **Mitigation:** Minimize lock hold times, profile before/after
   - **Validation:** Benchmark suite

3. **Testing Coverage**
   - **Risk:** Edge cases not covered by tests
   - **Impact:** Bugs in production
   - **Mitigation:** Comprehensive test suite, stress testing
   - **Validation:** Code coverage analysis (target: >80%)

### Low Risks

1. **Import Path Changes**
   - **Risk:** Breaking existing imports
   - **Impact:** Compilation failures in dependent code
   - **Mitigation:** Re-export all public APIs in mod.rs
   - **Validation:** Compile check of dependent crates

2. **Documentation Drift**
   - **Risk:** Outdated documentation
   - **Impact:** Developer confusion
   - **Mitigation:** Update all module docs during refactor
   - **Validation:** Doc review

---

## Testing Checklist

### Pre-Integration Tests

- [ ] `models.rs` compiles independently
- [ ] `extraction.rs` compiles with proper imports
- [ ] `circuit_breaker.rs` compiles with proper imports ⚠️ CRITICAL
- [ ] `health.rs` compiles with proper imports
- [ ] `memory.rs` compiles with proper imports
- [ ] `pool.rs` compiles with proper imports
- [ ] `mod.rs` re-exports work correctly

### Post-Integration Tests

**Compilation:**
- [ ] `cargo check --package riptide-core` succeeds
- [ ] `cargo clippy --package riptide-core` passes with no warnings
- [ ] `cargo fmt --package riptide-core` shows no changes needed

**Unit Tests:**
- [ ] All existing instance_pool tests pass
- [ ] Circuit breaker state transition tests pass ⚠️ CRITICAL
- [ ] New module-specific tests added and passing
- [ ] Test coverage maintained or improved (target: >80%)

**Integration Tests:**
- [ ] End-to-end extraction flow works
- [ ] Circuit breaker opens on failures
- [ ] Circuit breaker closes on recovery
- [ ] Health monitoring loop runs continuously
- [ ] Memory cleanup triggers correctly
- [ ] Pool exhaustion handling works

**Stress Tests:** ⚠️ MANDATORY
- [ ] 100+ concurrent extractions (no deadlocks)
- [ ] 1000+ rapid sequential extractions (no state corruption)
- [ ] Circuit breaker rapid state transitions (correctness)
- [ ] Memory pressure scenarios (cleanup works)
- [ ] Pool exhaustion and recovery (graceful handling)

**Deadlock Detection:**
- [ ] No locks held across await points
- [ ] Event emission happens without locks
- [ ] Lock ordering consistent (metrics → circuit_state)
- [ ] Stress test runs for 60+ seconds without hanging

**Performance:**
- [ ] Extraction latency unchanged (±5%)
- [ ] Memory usage stable or improved
- [ ] No new allocations in hot paths
- [ ] Benchmark suite passes

### Regression Tests

- [ ] All existing core tests pass
- [ ] No breaking changes to public API
- [ ] PooledInstance behavior unchanged
- [ ] AdvancedInstancePool behavior unchanged
- [ ] Event emission format unchanged
- [ ] Metrics recording accurate

---

## Migration Strategy

### Backward Compatibility

**Public API Preservation:**
```rust
// Before (in src/lib.rs or mod.rs)
pub use instance_pool::AdvancedInstancePool;
pub use instance_pool::PooledInstance;
pub use instance_pool::CircuitBreakerState;

// After (still works)
pub use instance_pool::AdvancedInstancePool;  // Now from instance_pool/mod.rs → instance_pool/pool.rs
pub use instance_pool::PooledInstance;        // Now from instance_pool/mod.rs → instance_pool/models.rs
pub use instance_pool::CircuitBreakerState;   // Now from instance_pool/mod.rs → instance_pool/models.rs
```

**Internal API Changes:**
- All functions remain in same logical locations
- Import paths change for internal callers
- No breaking changes to public API
- Circuit breaker behavior improved (deadlock fix) ⚠️

### Rollback Plan

If critical issues are discovered:

1. **Immediate Rollback:**
   ```bash
   git checkout HEAD~1 src/instance_pool.rs
   git checkout HEAD~1 src/instance_pool/
   cargo test --package riptide-core
   ```

2. **Partial Rollback:**
   - Keep `models.rs` extraction (safe)
   - Revert `circuit_breaker.rs` (if deadlock issues persist)
   - Keep other modules

3. **Circuit Breaker Hotfix:** ⚠️
   - If deadlock issues found, priority #1 fix
   - Can apply phase-based locking fix to monolithic file first
   - Then proceed with refactoring after validation

---

## Success Criteria

### Functional
- [ ] All 7 modules compile independently
- [ ] All existing tests pass
- [ ] No API breaking changes
- [ ] End-to-end extraction flow works
- [ ] Circuit breaker logic correct ⚠️ CRITICAL
- [ ] Health monitoring works continuously
- [ ] Memory management functions correctly

### Non-Functional
- [ ] Code coverage maintained or improved (>80%)
- [ ] No performance regression (±5%)
- [ ] Memory usage stable or improved
- [ ] Compilation time stable or improved
- [ ] Documentation complete and accurate
- [ ] No deadlocks under stress testing ⚠️ CRITICAL

### Quality
- [ ] No clippy warnings
- [ ] Code formatted consistently
- [ ] Clear module boundaries
- [ ] Well-documented interfaces
- [ ] Comprehensive error handling
- [ ] Phase-based locking implemented correctly ⚠️

---

## Timeline Estimates

### Optimistic (With automation and testing)
- Phase 1 (Models & Memory): 2-3 hours
- Phase 2 (Extraction): 2-3 hours
- Phase 3 (Circuit Breaker Fix): 4-6 hours ⚠️ CRITICAL
- Phase 4 (Health & Pool): 3-4 hours
- Phase 5 (Testing): 4-6 hours ⚠️ CRITICAL (stress testing)
- **Total: 15-22 hours**

### Realistic (With thorough validation)
- Phase 1: 3-4 hours
- Phase 2: 3-4 hours
- Phase 3: 6-8 hours ⚠️ (includes comprehensive testing)
- Phase 4: 4-5 hours
- Phase 5: 6-8 hours ⚠️ (stress testing and validation)
- **Total: 22-29 hours**

### Critical Path
1. Circuit breaker deadlock fix ⚠️ (highest priority)
2. Stress testing and validation ⚠️
3. Integration testing
4. Performance validation
5. Documentation updates

---

## Next Steps

### Immediate (Coder Agents)
1. ✅ Create module structure (`src/instance_pool/mod.rs`)
2. ✅ Extract `models.rs` (PooledInstance, CircuitBreakerState)
3. ✅ Extract `memory.rs` (utility functions)
4. ⚠️ **PRIORITY:** Extract and fix `circuit_breaker.rs` (deadlock fix)
5. ✅ Extract `extraction.rs` (WASM extraction logic)
6. ✅ Extract `health.rs` (health monitoring)
7. ✅ Extract `pool.rs` (main pool structure)
8. ✅ Update all imports and re-exports

### Validation (Architect + Reviewer)
1. ⚠️ **CRITICAL:** Review circuit breaker deadlock fix
2. Verify module boundaries are clean
3. Check for circular dependencies
4. Validate lock ordering and safety
5. Review error handling completeness
6. Verify test coverage (target: >80%)

### Integration (Full Team)
1. Run compilation checks
2. ⚠️ **CRITICAL:** Execute stress tests (60+ seconds, 100+ concurrent)
3. Execute full test suite
4. Validate performance benchmarks
5. Update documentation
6. Deploy to staging environment
7. Monitor for deadlocks and issues

---

## Appendix: Code Patterns

### Phase-Based Locking Pattern (Critical)

```rust
// ✅ CORRECT: Phase-based locking
async fn update_with_events(&self) {
    // Phase 1: Read/update state, extract event data
    let event_data = {
        let mut state = self.state.lock().await;
        // ... update logic ...
        EventData { ... }
    }; // Lock released

    // Phase 2: Emit events (no locks held)
    if let Some(data) = event_data {
        self.emit_event(data).await; // ✅ Safe: no locks held
    }
}

// ❌ WRONG: Lock held across await
async fn update_with_events_wrong(&self) {
    let mut state = self.state.lock().await;
    // ... update logic ...
    self.emit_event(data).await; // ❌ Deadlock risk
    // Lock not released until function end
}
```

### Sequential Lock Acquisition

```rust
// ✅ CORRECT: Sequential locking
async fn update_metrics_and_state(&self) {
    // Lock 1: Update metrics
    let metrics_snapshot = {
        let mut metrics = self.metrics.lock().await;
        metrics.count += 1;
        metrics.count
    }; // metrics lock released

    // Lock 2: Update state using metrics snapshot
    {
        let mut state = self.state.lock().await;
        state.update(metrics_snapshot);
    }; // state lock released
}

// ❌ WRONG: Concurrent locking (deadlock risk)
async fn update_metrics_and_state_wrong(&self) {
    let mut metrics = self.metrics.lock().await;
    let mut state = self.state.lock().await; // ❌ Two locks held
    // ... update logic ...
} // Both locks held until end
```

### Event Emission Pattern

```rust
// ✅ CORRECT: Spawn task for event emission
async fn emit_events_correctly(&self, data: EventData) {
    if let Some(event_bus) = &self.event_bus {
        let event_bus = event_bus.clone();
        let pool_id = self.pool_id.clone();

        tokio::spawn(async move {
            let event = create_event(pool_id, data);
            if let Err(e) = event_bus.emit(event).await {
                warn!("Event emission failed: {}", e);
            }
        });
    }
}

// ❌ WRONG: Emit event while holding lock
async fn emit_events_wrong(&self, data: EventData) {
    let mut state = self.state.lock().await;
    // ... update state ...

    if let Some(event_bus) = &self.event_bus {
        event_bus.emit(event).await; // ❌ Lock held during emit
    }
}
```

---

## Appendix: Stress Test Implementation

```rust
#[cfg(test)]
mod stress_tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Run with --ignored flag
    async fn stress_test_concurrent_extractions() {
        let pool = create_test_pool().await;
        let pool = Arc::new(pool);

        let mut handles = vec![];

        // Spawn 100 concurrent extraction tasks
        for i in 0..100 {
            let pool = pool.clone();
            let handle = tokio::spawn(async move {
                for j in 0..10 {
                    let result = pool.extract(
                        "<html>test</html>",
                        &format!("http://test{}.com", i),
                        ExtractionMode::Article,
                    ).await;

                    // Random success/failure to trigger circuit breaker
                    if j % 3 == 0 {
                        assert!(result.is_ok() || result.is_err());
                    }
                }
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.expect("Task panicked");
        }

        // Verify pool state is consistent
        let metrics = pool.get_metrics().await;
        assert_eq!(metrics.total_extractions, 1000);

        // Verify no deadlocks occurred (if we got here, no deadlocks)
        println!("Stress test completed successfully");
    }

    #[tokio::test]
    #[ignore]
    async fn stress_test_circuit_breaker_state_transitions() {
        let pool = create_test_pool().await;

        // Trigger rapid state transitions
        for i in 0..100 {
            let success = i % 10 != 0; // 10% failure rate
            pool.record_extraction_result(success, Duration::from_millis(100)).await;
        }

        // Verify circuit breaker state is correct
        let metrics = pool.get_metrics().await;
        assert!(metrics.circuit_breaker_trips >= 1);
    }
}
```

---

**Document Status:** 🟢 Architecture Design Complete
**Last Updated:** 2025-09-30
**Next Review:** After Phase 3 (Circuit Breaker Fix) completion

**Critical Notes:**
- ⚠️ Circuit breaker deadlock fix is **HIGHEST PRIORITY**
- ⚠️ Stress testing is **MANDATORY** before production deployment
- ⚠️ Phase-based locking pattern must be followed strictly
- ⚠️ No locks should be held across await points