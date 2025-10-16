# Research Report: P1/P2 Fix Implementation Patterns

**Research Agent**: Hive Mind Collective Intelligence System
**Session ID**: swarm-hive-research
**Date**: 2025-10-14
**Status**: Complete

---

## Executive Summary

This research report provides comprehensive implementation patterns, best practices, and code examples for addressing P1 (Priority 1) and P2 (Priority 2) issues in the RipTide EventMesh codebase. The research covers:

1. Safe Arc/Weak patterns to eliminate unsafe code
2. Async cleanup patterns with explicit Drop fallbacks
3. Result-based error handling to replace unwrap/expect
4. WASM instance pool optimization patterns
5. WIT interface validation strategies

---

## 1. Unsafe Code Fix: Arc/Weak Pattern (P1)

### Current Issue (Line 665-667 in memory_manager.rs)

```rust
// ❌ UNSAFE: Creates dangling pointer risk
fn new(manager: &MemoryManager) -> Self {
    Self {
        manager: Arc::downgrade(&Arc::new(unsafe {
            std::ptr::read(manager as *const MemoryManager)
        })),
    }
}
```

**Problem**: Uses `unsafe ptr::read` to create an Arc from a reference, which:
- Creates a second ownership without incrementing ref count
- Risks double-free when original is dropped
- Violates Rust's memory safety guarantees

### Safe Solution: Proper Arc/Weak Pattern

**Pattern Found in Codebase**: `pool.rs` uses safe `Arc::clone()` extensively (200+ occurrences)

```rust
// ✅ SAFE: Use Arc parameter directly
impl MemoryManagerRef {
    fn new(manager: Arc<MemoryManager>) -> Self {
        Self {
            manager: Arc::downgrade(&manager),
        }
    }
}

// ✅ SAFE: Alternative - store Arc and expose methods
pub struct WasmInstanceHandle {
    instance_id: String,
    manager: Arc<MemoryManager>,  // Direct Arc ownership
}

impl WasmInstanceHandle {
    pub fn new(instance_id: String, manager: Arc<MemoryManager>) -> Self {
        Self { instance_id, manager }
    }

    pub async fn return_to_pool(self) -> Result<()> {
        self.manager.return_instance(&self.instance_id).await
    }
}

impl Drop for WasmInstanceHandle {
    fn drop(&mut self) {
        let instance_id = self.instance_id.clone();
        let manager = Arc::clone(&self.manager);  // ✅ Safe clone

        tokio::spawn(async move {
            if let Err(e) = manager.return_instance(&instance_id).await {
                error!("Failed to return instance during drop: {}", e);
            }
        });
    }
}
```

### Best Practices from Codebase

**Example from `/workspaces/eventmesh/crates/riptide-performance/tests/resource_manager_performance_tests.rs`**:

```rust
// Pattern: Clone Arc for concurrent tasks
let mgr = Arc::clone(&manager);
let counter = Arc::clone(&success_count);

tokio::spawn(async move {
    if let Ok(_) = mgr.acquire_resource().await {
        counter.fetch_add(1, Ordering::Relaxed);
    }
});
```

**Key Principles**:
1. Always use `Arc::clone(&arc)` instead of `arc.clone()` for clarity
2. Pass `Arc<T>` to constructors, not `&T`
3. Use `Arc::downgrade()` only when you have an `Arc`, never from `&T`
4. Never use `unsafe ptr::read` unless absolutely necessary and documented

---

## 2. Async Drop Pattern: Explicit Cleanup with Drop Fallback (P1)

### Current Issue

Drop trait cannot be async, but WASM instance cleanup requires async operations.

### Solution Pattern Found in Codebase

**Example from `/workspaces/eventmesh/crates/riptide-headless/src/pool.rs`** (Lines 529, 590, 643, etc.):

```rust
// ✅ PATTERN: Explicit async cleanup with Drop as safety net
pub struct BrowserInstance {
    id: String,
    handle: Arc<Mutex<Option<BrowserHandle>>>,
    cleanup_called: Arc<AtomicBool>,
}

impl BrowserInstance {
    /// Explicit async cleanup - PREFERRED METHOD
    pub async fn cleanup(&self) {
        if self.cleanup_called.swap(true, Ordering::SeqCst) {
            return; // Already cleaned up
        }

        if let Some(handle) = self.handle.lock().await.take() {
            if let Err(e) = handle.shutdown().await {
                warn!("Browser cleanup failed: {}", e);
            }
        }

        info!("Browser instance {} cleaned up", self.id);
    }
}

impl Drop for BrowserInstance {
    /// Fallback cleanup - only if explicit cleanup wasn't called
    fn drop(&mut self) {
        if !self.cleanup_called.load(Ordering::SeqCst) {
            warn!("Browser {} dropped without explicit cleanup - spawning cleanup task", self.id);

            let handle = Arc::clone(&self.handle);
            let id = self.id.clone();

            tokio::spawn(async move {
                if let Some(browser) = handle.lock().await.take() {
                    if let Err(e) = browser.shutdown().await {
                        error!("Emergency cleanup failed for {}: {}", id, e);
                    }
                }
            });
        }
    }
}

// Usage pattern:
async fn use_browser() -> Result<()> {
    let browser = BrowserInstance::new().await?;

    // Work with browser...

    // ✅ Explicit cleanup before drop
    browser.cleanup().await;

    Ok(())
}
```

### Best Practices

1. **Primary**: Provide explicit `async fn cleanup(&self)` method
2. **Secondary**: Implement Drop as safety net with warning logs
3. **Track State**: Use `AtomicBool` to prevent double cleanup
4. **Logging**: Warn when Drop is triggered without explicit cleanup
5. **Documentation**: Document that explicit cleanup is preferred

---

## 3. Error Handling: Replace unwrap/expect with Result (P1)

### Current Issues

Found 20 files with `unwrap()`/`expect()` patterns that should use proper error handling.

### Solution Patterns

**Pattern 1: Context-based error enrichment** (from docs and codebase):

```rust
// ❌ BAD: Panics on error
let config = std::fs::read_to_string("config.toml").unwrap();

// ✅ GOOD: Propagates with context
use anyhow::{Context, Result};

let config = std::fs::read_to_string("config.toml")
    .context("Failed to read config file")?;
```

**Pattern 2: Map error types** (from docs):

```rust
// ✅ Map to custom error type
let tenant = db.get_tenant(&id)
    .await
    .map_err(|e| ApiError::internal(format!("Tenant lookup failed: {}", e)))?;
```

**Pattern 3: Error propagation with `?` operator**:

```rust
// ✅ Clean error propagation
async fn create_instance(&self, component_path: &str) -> Result<TrackedWasmInstance> {
    let component = Component::from_file(&self.engine, component_path)
        .map_err(|e| {
            let error = CoreError::from(e);
            report_error!(&error, "create_wasm_instance", "component_path" => component_path);
            error
        })?;

    Ok(TrackedWasmInstance::new(id, store, component))
}
```

### Codebase Pattern: Custom Error Types

**Example from existing code**:

```rust
// Structured error with context
match result {
    Ok(data) => Ok(data),
    Err(e) => Err(anyhow!(
        "Memory pressure too high: {:.2}% (threshold: {:.2}%)",
        memory_pressure,
        self.config.memory_pressure_threshold
    )),
}
```

### Best Practices

1. **Use `anyhow::Result<T>` for applications**: Simplifies error handling
2. **Use `thiserror` for libraries**: Define custom error enums
3. **Add context**: Use `.context()` or `.map_err()` to add meaningful information
4. **Never panic in libraries**: Always return `Result<T, E>`
5. **Log before returning errors**: Use `report_error!` macro for telemetry

---

## 4. WASM Instance Pool Optimization (P2)

### Current State Analysis

**Existing Pool** (`/workspaces/eventmesh/crates/riptide-core/src/instance_pool/pool.rs`):
- ✅ Semaphore-based concurrency control
- ✅ Circuit breaker pattern
- ✅ Health checks and instance recycling
- ✅ Event-driven metrics
- ⚠️ No pre-warmed pool reuse optimization

**Target**: 40-60% latency reduction through instance reuse

### Optimization Pattern

```rust
/// Enhanced pool with pre-warming and smart reuse
pub struct OptimizedWasmPool {
    engine: Arc<Engine>,
    component: Arc<Component>,
    linker: Arc<Linker<WasmResourceTracker>>,

    // Stratified pooling: hot, warm, cold
    hot_instances: Arc<Mutex<VecDeque<PooledInstance>>>,   // < 5s idle
    warm_instances: Arc<Mutex<VecDeque<PooledInstance>>>,  // 5s-30s idle
    available_instances: Arc<Mutex<VecDeque<PooledInstance>>>, // > 30s idle

    semaphore: Arc<Semaphore>,
    config: ExtractorConfig,
    metrics: Arc<Mutex<PoolMetrics>>,
}

impl OptimizedWasmPool {
    /// Fast-path: Try hot pool first for best latency
    pub async fn get_or_create_instance(&self) -> Result<PooledInstance> {
        // Fast path: hot instances (microsecond checkout)
        if let Some(instance) = self.try_hot_pool().await {
            self.metrics.lock().await.hot_hits += 1;
            return Ok(instance);
        }

        // Warm path: pre-warmed instances (millisecond checkout)
        if let Some(instance) = self.try_warm_pool().await {
            self.metrics.lock().await.warm_hits += 1;
            return Ok(instance);
        }

        // Cold path: check available pool
        if let Some(instance) = self.try_available_pool().await {
            if instance.is_healthy(&self.config) {
                self.metrics.lock().await.available_hits += 1;
                return Ok(instance);
            }
        }

        // Last resort: create new instance
        self.metrics.lock().await.cold_creates += 1;
        self.create_instance().await
    }

    /// Return instance to appropriate pool tier
    pub async fn return_instance(&self, mut instance: PooledInstance) {
        let idle_time = instance.last_used.elapsed();

        if idle_time < Duration::from_secs(5) {
            // Keep in hot pool for fast reuse
            let mut hot = self.hot_instances.lock().await;
            if hot.len() < self.config.hot_pool_size {
                instance.tier = PoolTier::Hot;
                hot.push_back(instance);
                return;
            }
        }

        if idle_time < Duration::from_secs(30) {
            // Demote to warm pool
            let mut warm = self.warm_instances.lock().await;
            instance.tier = PoolTier::Warm;
            warm.push_back(instance);
            return;
        }

        // Cold pool
        let mut available = self.available_instances.lock().await;
        instance.tier = PoolTier::Available;
        available.push_back(instance);
    }

    /// Background task: promote warm instances to hot
    async fn pool_optimization_task(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(1));

        loop {
            interval.tick().await;

            // Promote warm -> hot if hot pool is empty
            let hot_empty = {
                let hot = self.hot_instances.lock().await;
                hot.is_empty()
            };

            if hot_empty {
                let mut warm = self.warm_instances.lock().await;
                if let Some(mut instance) = warm.pop_front() {
                    instance.tier = PoolTier::Hot;
                    let mut hot = self.hot_instances.lock().await;
                    hot.push_back(instance);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PoolTier {
    Hot,        // < 5s idle, instant reuse
    Warm,       // 5-30s idle, fast reuse
    Available,  // > 30s idle, normal reuse
}
```

### Performance Metrics

```rust
pub struct PoolMetrics {
    pub hot_hits: u64,        // < 1ms checkout
    pub warm_hits: u64,       // 1-10ms checkout
    pub available_hits: u64,  // 10-50ms checkout
    pub cold_creates: u64,    // 50-200ms creation

    pub avg_checkout_time_us: f64,
    pub cache_efficiency: f64, // (hot_hits + warm_hits) / total
}
```

### Expected Improvements

- **Hot pool hit**: ~50-100μs checkout (vs 50-200ms cold)
- **Warm pool hit**: ~1-5ms checkout
- **Target**: 70%+ hot+warm hit rate = 40-60% overall latency reduction

---

## 5. WIT Interface Validation (P2)

### Current State

**Existing validation** (`/workspaces/eventmesh/crates/riptide-extraction/src/wasm_extraction.rs`):
- ✅ Type conversions between host and WIT types
- ✅ Error mapping from WIT to host errors
- ⚠️ No schema validation at runtime

### Validation Strategy

```rust
use wasmtime::component::*;

/// WIT interface validator with schema checking
pub struct WitInterfaceValidator {
    engine: Engine,
    expected_schema: ComponentSchema,
}

#[derive(Debug, Clone)]
pub struct ComponentSchema {
    pub world: String,
    pub exports: Vec<FunctionSignature>,
    pub imports: Vec<FunctionSignature>,
    pub types: Vec<TypeDefinition>,
}

#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: String,
    pub params: Vec<(String, TypeDefinition)>,
    pub results: Vec<TypeDefinition>,
}

impl WitInterfaceValidator {
    pub fn new(engine: Engine) -> Self {
        Self {
            engine,
            expected_schema: Self::build_expected_schema(),
        }
    }

    /// Build expected schema from WIT definition
    fn build_expected_schema() -> ComponentSchema {
        ComponentSchema {
            world: "extractor".to_string(),
            exports: vec![
                FunctionSignature {
                    name: "extract".to_string(),
                    params: vec![
                        ("html".to_string(), TypeDefinition::String),
                        ("url".to_string(), TypeDefinition::String),
                        ("mode".to_string(), TypeDefinition::Variant("ExtractionMode".to_string())),
                    ],
                    results: vec![
                        TypeDefinition::Result(
                            Box::new(TypeDefinition::Record("ExtractedContent".to_string())),
                            Box::new(TypeDefinition::Variant("ExtractionError".to_string())),
                        ),
                    ],
                },
            ],
            imports: vec![],
            types: vec![
                TypeDefinition::Record("ExtractedContent".to_string()),
                TypeDefinition::Variant("ExtractionMode".to_string()),
                TypeDefinition::Variant("ExtractionError".to_string()),
            ],
        }
    }

    /// Validate component against expected schema
    pub async fn validate_component(&self, component_bytes: &[u8]) -> Result<ValidationReport> {
        let component = Component::new(&self.engine, component_bytes)?;

        let mut report = ValidationReport::new();

        // Validate exports
        let exports = self.introspect_exports(&component)?;
        for expected_export in &self.expected_schema.exports {
            match exports.iter().find(|e| e.name == expected_export.name) {
                Some(actual) => {
                    if !self.signatures_match(expected_export, actual) {
                        report.add_error(ValidationError::SignatureMismatch {
                            function: expected_export.name.clone(),
                            expected: expected_export.clone(),
                            actual: actual.clone(),
                        });
                    } else {
                        report.add_success(expected_export.name.clone());
                    }
                }
                None => {
                    report.add_error(ValidationError::MissingExport(expected_export.name.clone()));
                }
            }
        }

        // Validate types
        for expected_type in &self.expected_schema.types {
            if !self.type_exists(&component, expected_type)? {
                report.add_error(ValidationError::MissingType(format!("{:?}", expected_type)));
            }
        }

        Ok(report)
    }

    /// Runtime validation during instantiation
    pub async fn validate_instance(&self, store: &mut Store<WasmResourceTracker>, instance: &Instance) -> Result<()> {
        // Verify all expected exports are accessible
        for export in &self.expected_schema.exports {
            let _func = instance
                .get_func(store, &export.name)
                .ok_or_else(|| anyhow!("Missing export function: {}", export.name))?;

            // Could add parameter/result type validation here
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum TypeDefinition {
    String,
    U32,
    U64,
    Bool,
    List(Box<TypeDefinition>),
    Option(Box<TypeDefinition>),
    Result(Box<TypeDefinition>, Box<TypeDefinition>),
    Record(String),
    Variant(String),
}

#[derive(Debug)]
pub struct ValidationReport {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
    pub successes: Vec<String>,
}

#[derive(Debug)]
pub enum ValidationError {
    MissingExport(String),
    MissingImport(String),
    MissingType(String),
    SignatureMismatch {
        function: String,
        expected: FunctionSignature,
        actual: FunctionSignature,
    },
    VersionMismatch {
        expected: String,
        actual: String,
    },
}
```

### Integration Pattern

```rust
/// Validated component loader
pub async fn load_and_validate_component(
    engine: &Engine,
    component_path: &str,
) -> Result<Component> {
    let component_bytes = std::fs::read(component_path)
        .context("Failed to read component file")?;

    // Schema validation
    let validator = WitInterfaceValidator::new(engine.clone());
    let report = validator.validate_component(&component_bytes).await?;

    if !report.errors.is_empty() {
        error!("Component validation failed: {:?}", report.errors);
        return Err(anyhow!("WIT schema validation failed: {} errors", report.errors.len()));
    }

    info!("Component validation passed: {} functions validated", report.successes.len());

    // Load component
    Component::new(engine, component_bytes)
        .context("Failed to instantiate validated component")
}
```

---

## 6. Implementation Priority Matrix

| Issue | Priority | Impact | Effort | Risk |
|-------|----------|--------|--------|------|
| Unsafe Arc/Weak (memory_manager.rs:666) | P1 | 🔴 High | Low | High |
| Async Drop Pattern | P1 | 🟡 Medium | Medium | Medium |
| Error Handling (unwrap/expect) | P1 | 🟡 Medium | Low | Low |
| WASM Pool Optimization | P2 | 🟢 Low | High | Low |
| WIT Validation | P2 | 🟢 Low | Medium | Low |

---

## 7. Code Examples by File

### `/workspaces/eventmesh/crates/riptide-core/src/memory_manager.rs`

**Lines 662-678**: Replace unsafe Arc creation

```rust
// BEFORE (unsafe):
impl MemoryManagerRef {
    fn new(manager: &MemoryManager) -> Self {
        Self {
            manager: Arc::downgrade(&Arc::new(unsafe {
                std::ptr::read(manager as *const MemoryManager)
            })),
        }
    }
}

// AFTER (safe):
impl MemoryManagerRef {
    fn new(manager: Arc<MemoryManager>) -> Self {
        Self {
            manager: Arc::downgrade(&manager),
        }
    }
}

// Update caller at line 361:
impl MemoryManager {
    pub async fn get_instance(&self, component_path: &str) -> Result<WasmInstanceHandle> {
        // ... instance creation logic ...

        Ok(WasmInstanceHandle {
            instance_id,
            manager: Arc::new(self.clone()),  // Or pass Arc<Self> through
        })
    }
}
```

### `/workspaces/eventmesh/crates/riptide-core/src/memory_manager.rs`

**Lines 698-716**: Add explicit cleanup method

```rust
// ADD: Explicit cleanup with Drop fallback
impl WasmInstanceHandle {
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    /// ✅ Explicit async cleanup (PREFERRED)
    pub async fn cleanup(self) -> Result<()> {
        self.manager.return_instance(&self.instance_id).await
    }
}

impl Drop for WasmInstanceHandle {
    fn drop(&mut self) {
        let instance_id = self.instance_id.clone();
        let manager = self.manager.manager.clone();

        // ⚠️ Fallback cleanup with warning
        warn!("WasmInstanceHandle dropped without explicit cleanup");

        tokio::spawn(async move {
            if let Some(manager) = manager.upgrade() {
                if let Err(e) = manager.return_instance(&instance_id).await {
                    error!(
                        instance_id = %instance_id,
                        error = %e,
                        "Emergency cleanup failed during drop"
                    );
                }
            }
        });
    }
}
```

---

## 8. Testing Strategy

### Unit Tests for Safe Arc Pattern

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_safe_arc_ref_creation() {
        let manager = Arc::new(MemoryManager::new(config, engine).await?);
        let ref1 = MemoryManagerRef::new(Arc::clone(&manager));
        let ref2 = MemoryManagerRef::new(Arc::clone(&manager));

        // Both refs should be valid
        assert!(ref1.manager.upgrade().is_some());
        assert!(ref2.manager.upgrade().is_some());

        // Drop manager
        drop(manager);

        // Refs should now be invalid
        assert!(ref1.manager.upgrade().is_none());
        assert!(ref2.manager.upgrade().is_none());
    }

    #[tokio::test]
    async fn test_no_double_free() {
        let manager = Arc::new(MemoryManager::new(config, engine).await?);
        let handle = WasmInstanceHandle::new("test-id".to_string(), manager);

        // Explicit cleanup
        handle.cleanup().await?;

        // Should not double-free on drop
    } // Drop occurs here - should be no-op
}
```

### Integration Tests for Async Cleanup

```rust
#[tokio::test]
async fn test_explicit_cleanup_pattern() {
    let handle = create_test_handle().await?;

    // Work with handle
    assert_eq!(handle.instance_id(), "test-instance");

    // Explicit cleanup
    handle.cleanup().await?;

    // Verify instance returned to pool
    let stats = manager.stats();
    assert_eq!(stats.idle_instances, 1);
}

#[tokio::test]
async fn test_drop_fallback_cleanup() {
    let handle = create_test_handle().await?;

    // Drop without explicit cleanup (should trigger warning)
    drop(handle);

    // Give background task time to execute
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify emergency cleanup occurred
    let stats = manager.stats();
    assert_eq!(stats.idle_instances, 1);
}
```

---

## 9. Benchmarks for Pool Optimization

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_pool_checkout(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let pool = rt.block_on(OptimizedWasmPool::new(config, engine, "component.wasm")).unwrap();

    // Pre-warm pool
    rt.block_on(pool.warm_up()).unwrap();

    c.bench_function("hot_pool_checkout", |b| {
        b.to_async(&rt).iter(|| async {
            let instance = pool.get_or_create_instance().await.unwrap();
            pool.return_instance(instance).await;
        });
    });

    c.bench_function("cold_creation", |b| {
        b.to_async(&rt).iter(|| async {
            let instance = pool.create_instance().await.unwrap();
            black_box(instance);
        });
    });
}

criterion_group!(benches, benchmark_pool_checkout);
criterion_main!(benches);
```

---

## 10. Documentation Updates Required

### Add to `/workspaces/eventmesh/docs/CONTRIBUTING.md`

```markdown
## Memory Safety Guidelines

### Arc/Weak Pattern
- ✅ Always use `Arc::clone(&arc)` for clarity
- ✅ Pass `Arc<T>` to constructors, not `&T`
- ❌ Never use `unsafe ptr::read` to create Arc from reference
- ❌ Never use `Arc::downgrade(&Arc::new(unsafe {...}))`

### Async Cleanup Pattern
- ✅ Provide explicit `async fn cleanup(&self)` method
- ✅ Implement Drop as fallback with warning
- ✅ Document that explicit cleanup is preferred
- ❌ Never rely solely on Drop for async operations

### Error Handling
- ✅ Use `anyhow::Result<T>` for applications
- ✅ Use `.context()` to add error information
- ✅ Use `?` operator for error propagation
- ❌ Never use `unwrap()` or `expect()` in library code
```

---

## 11. References

### Rust Best Practices
- [Rust Arc Documentation](https://doc.rust-lang.org/std/sync/struct.Arc.html)
- [Async Drop Pattern RFC](https://rust-lang.github.io/rfcs/3185-async-drop.html)
- [Error Handling in Rust](https://doc.rust-lang.org/book/ch09-00-error-handling.html)

### Codebase Examples
- Safe Arc patterns: `/workspaces/eventmesh/crates/riptide-performance/**/*.rs` (200+ occurrences)
- Async cleanup: `/workspaces/eventmesh/crates/riptide-headless/src/pool.rs` (lines 529, 590, 643)
- Error context: `/workspaces/eventmesh/docs/**/*.md` (30+ examples)

### Wasmtime Resources
- [Wasmtime Component Model](https://docs.wasmtime.dev/api/wasmtime/component/)
- [WIT Specification](https://component-model.bytecodealliance.org/design/wit.html)
- [Resource Limiting](https://docs.wasmtime.dev/api/wasmtime/struct.ResourceLimiter.html)

---

## 12. Next Steps

1. **Coder Agent**: Implement P1 fixes in order:
   - Fix unsafe Arc/Weak pattern (memory_manager.rs:666)
   - Add explicit cleanup methods
   - Replace unwrap/expect with Result

2. **Tester Agent**: Create tests for:
   - Safe Arc reference counting
   - Async cleanup patterns
   - Error propagation

3. **Reviewer Agent**: Verify:
   - No new unsafe code introduced
   - All async cleanup has Drop fallback
   - Proper error context added

4. **Architect Agent** (P2): Design:
   - WASM pool optimization architecture
   - WIT validation integration points

---

## Research Completion Checklist

- [x] Analyzed unsafe Arc/Weak pattern issue
- [x] Researched safe Arc::clone patterns in codebase
- [x] Found async cleanup examples (pool.rs)
- [x] Documented error handling best practices
- [x] Studied existing WASM pool implementation
- [x] Researched WIT validation approaches
- [x] Created code examples for all patterns
- [x] Documented testing strategies
- [x] Provided benchmarking approach
- [x] Listed documentation updates needed

---

**End of Research Report**
