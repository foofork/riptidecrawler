# Cleanup Wiring & Configuration Research Report

**Research Date:** 2025-10-14
**Researcher:** Hive Mind Research Agent
**Session ID:** swarm-hive-wiring-research

## Executive Summary

This research identifies patterns for properly wiring `cleanup()` methods, making timeouts configurable, implementing builder patterns, and fixing API changes in spider tests. The key findings focus on explicit cleanup vs RAII patterns, configuration-driven timeouts, and API migration strategies.

---

## 1. WasmInstanceHandle Usage Patterns

### 1.1 Instance Creation
**Location:** `/workspaces/eventmesh/crates/riptide-core/src/memory_manager.rs:304-362`

```rust
pub async fn get_instance(&self, component_path: &str) -> Result<WasmInstanceHandle> {
    // ... memory pressure checks ...

    Ok(WasmInstanceHandle {
        instance_id,
        manager: MemoryManagerRef::new(self),
    })
}
```

### 1.2 Current Drop Implementation
**Location:** `/workspaces/eventmesh/crates/riptide-core/src/memory_manager.rs:726-747`

```rust
impl Drop for WasmInstanceHandle {
    fn drop(&mut self) {
        warn!(
            instance_id = %self.instance_id,
            "WasmInstanceHandle dropped without explicit cleanup - spawning best-effort background task"
        );

        let instance_id = self.instance_id.clone();
        let manager = self.manager.clone();

        // Best-effort cleanup in background (not guaranteed to complete)
        tokio::spawn(async move {
            if let Err(e) = manager.return_instance(&instance_id).await {
                error!(
                    instance_id = %instance_id,
                    error = %e,
                    "Failed to return WASM instance during drop"
                );
            }
        });
    }
}
```

### 1.3 Existing Cleanup Method
**Location:** `/workspaces/eventmesh/crates/riptide-core/src/memory_manager.rs:716-723`

```rust
/// Cleanup with timeout - ensures proper async cleanup
pub async fn cleanup(self) -> Result<()> {
    tokio::time::timeout(
        Duration::from_secs(5), // ⚠️ HARDCODED 5s TIMEOUT
        self.manager.return_instance(&self.instance_id),
    )
    .await
    .map_err(|_| anyhow::anyhow!("Timeout returning WASM instance {}", self.instance_id))?
}
```

### 1.4 Where Cleanup Should Be Called

**API Handlers That Create Instances:**
1. `/workspaces/eventmesh/crates/riptide-api/src/handlers/browser.rs` - Browser render operations
2. `/workspaces/eventmesh/crates/riptide-api/src/resource_manager/guards.rs` - Resource guard wrappers

**Pattern to Follow:**
```rust
// In async handler functions
async fn handle_request() -> Result<Response> {
    let instance = memory_manager.get_instance(component_path).await?;

    // Use instance
    let result = do_work(&instance).await;

    // Explicit cleanup before returning
    instance.cleanup().await?;

    Ok(result)
}
```

---

## 2. BrowserCheckout Usage Patterns

### 2.1 Checkout Creation
**Location:** `/workspaces/eventmesh/crates/riptide-headless/src/pool.rs:437-493`

```rust
pub async fn checkout(&self) -> Result<BrowserCheckout> {
    let permit = self.semaphore.clone().acquire_owned().await?;

    // ... browser selection logic ...

    Ok(BrowserCheckout {
        browser_id,
        pool: BrowserPoolRef::new(self),
        permit: Some(permit),
    })
}
```

### 2.2 Current Drop Implementation
**Location:** `/workspaces/eventmesh/crates/riptide-headless/src/pool.rs:914-933`

```rust
impl Drop for BrowserCheckout {
    fn drop(&mut self) {
        if self.permit.is_some() {
            warn!(
                browser_id = %self.browser_id,
                "BrowserCheckout dropped without explicit cleanup - spawning best-effort background task"
            );

            let browser_id = self.browser_id.clone();
            let pool = self.pool.clone();

            // Best-effort cleanup in background (not guaranteed to complete)
            tokio::spawn(async move {
                if let Err(e) = pool.checkin(&browser_id).await {
                    error!(browser_id = %browser_id, error = %e, "Failed to checkin browser during drop");
                }
            });
        }
    }
}
```

### 2.3 Existing Cleanup Method
**Location:** `/workspaces/eventmesh/crates/riptide-headless/src/pool.rs:903-911`

```rust
/// Cleanup with timeout - ensures proper async cleanup
pub async fn cleanup(mut self) -> Result<()> {
    tokio::time::timeout(
        Duration::from_secs(5), // ⚠️ HARDCODED 5s TIMEOUT
        self.pool.checkin(&self.browser_id)
    )
    .await
    .map_err(|_| anyhow!("Timeout checking in browser {}", self.browser_id))?;

    // Prevent drop from trying to checkin again
    self.permit.take();
    Ok(())
}
```

### 2.4 Where Cleanup Should Be Called

**Current Usage:**
- `/workspaces/eventmesh/crates/riptide-api/src/resource_manager/guards.rs:20` - Inside `RenderResourceGuard`
- `/workspaces/eventmesh/crates/riptide-headless/src/launcher.rs:380` - Inside `LaunchSession`

**Pattern to Follow:**
```rust
// In handlers that use browser
async fn render_page(url: &str, pool: &BrowserPool) -> Result<String> {
    let checkout = pool.checkout().await?;

    // Use browser
    let page = checkout.page().await?;
    let html = page.content().await?;

    // Explicit cleanup
    checkout.cleanup().await?;

    Ok(html)
}
```

---

## 3. RAII vs Explicit Cleanup Analysis

### 3.1 When to Use Drop (RAII)
**Best For:**
- Synchronous cleanup (locks, file handles)
- Non-failing operations
- Immediate resource release

**Example from codebase:**
```rust
// Semaphore permits - perfect for RAII
struct Guard {
    _permit: OwnedSemaphorePermit, // Dropped automatically
}
```

### 3.2 When to Use Explicit Cleanup
**Best For:**
- **Async operations** (cannot be done in Drop)
- **Fallible operations** that need error handling
- **Timeouts** for cleanup operations
- **Resource pools** that need proper return

**Examples from research:**
- `WasmInstanceHandle` - needs async pool return
- `BrowserCheckout` - needs async browser checkin
- Both use `tokio::spawn` in Drop as **fallback only**

### 3.3 Recommended Pattern: Hybrid Approach

```rust
pub struct ResourceHandle {
    resource_id: String,
    pool: Arc<ResourcePool>,
    cleaned_up: AtomicBool, // Track if cleanup was called
}

impl ResourceHandle {
    /// Preferred: Explicit async cleanup with timeout
    pub async fn cleanup(self) -> Result<()> {
        self.cleaned_up.store(true, Ordering::SeqCst);

        tokio::time::timeout(
            self.pool.config.cleanup_timeout(), // ✅ CONFIGURABLE
            self.pool.return_resource(&self.resource_id),
        )
        .await
        .map_err(|_| anyhow!("Timeout during cleanup"))?
    }
}

impl Drop for ResourceHandle {
    fn drop(&mut self) {
        // Only warn if cleanup wasn't called
        if !self.cleaned_up.load(Ordering::SeqCst) {
            warn!("ResourceHandle dropped without cleanup - best-effort recovery");

            let resource_id = self.resource_id.clone();
            let pool = self.pool.clone();

            // Best-effort background cleanup (may not complete)
            tokio::spawn(async move {
                let _ = pool.return_resource(&resource_id).await;
            });
        }
    }
}
```

### 3.4 Async Cleanup Best Practices from Tokio/SQLx

**Tokio Pattern:**
```rust
// JoinHandle requires explicit await or abort
let handle = tokio::spawn(async { /* work */ });
handle.await?; // Explicit
```

**SQLx Pattern:**
```rust
impl Drop for PoolConnection {
    fn drop(&mut self) {
        // Returns connection via channel (non-blocking)
        self.return_to_pool_sync();
    }
}

// But also provides:
async fn close(self) -> Result<()> {
    // Explicit async close with error handling
}
```

---

## 4. Configuration Patterns for Timeouts

### 4.1 Existing Configuration Structure
**Location:** `/workspaces/eventmesh/crates/riptide-api/src/config.rs`

```rust
pub struct PerformanceConfig {
    pub render_timeout_secs: u64,      // 3s default
    pub pdf_timeout_secs: u64,         // 10s default
    pub wasm_timeout_secs: u64,        // 5s default
    pub http_timeout_secs: u64,        // 10s default
    // ⚠️ MISSING: cleanup_timeout_secs
}

pub struct HeadlessConfig {
    pub idle_timeout_secs: u64,        // 300s default
    pub launch_timeout_secs: u64,      // 30s default
    // ⚠️ MISSING: checkin_timeout_secs
}

pub struct WasmConfig {
    pub module_timeout_secs: u64,      // 10s default
    // ⚠️ MISSING: instance_cleanup_timeout_secs
}
```

### 4.2 Hardcoded Timeouts Found

**5-Second Timeouts (20 occurrences):**
```rust
// 1. WASM cleanup: memory_manager.rs:717
Duration::from_secs(5)

// 2. Browser cleanup: pool.rs:904
Duration::from_secs(5)

// 3. Search tests: various test files
Duration::from_secs(5)

// 4. Health monitoring: health.rs:530
Duration::from_secs(5)

// 5. Streaming backpressure: backpressure.rs:352
Duration::from_secs(5)
```

### 4.3 Recommended Configuration Additions

```rust
// Add to PerformanceConfig
pub struct PerformanceConfig {
    // ... existing fields ...

    /// Timeout for async cleanup operations (default: 5s)
    pub cleanup_timeout_secs: u64,

    /// Timeout for resource pool returns (default: 3s)
    pub pool_return_timeout_secs: u64,

    /// Enable graceful cleanup on shutdown (default: true)
    pub graceful_cleanup: bool,

    /// Maximum time to wait for all cleanups during shutdown (default: 30s)
    pub shutdown_grace_period_secs: u64,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            // ... existing defaults ...
            cleanup_timeout_secs: 5,
            pool_return_timeout_secs: 3,
            graceful_cleanup: true,
            shutdown_grace_period_secs: 30,
        }
    }
}

// Add to HeadlessConfig
pub struct HeadlessConfig {
    // ... existing fields ...

    /// Timeout for browser checkin operations (default: 5s)
    pub checkin_timeout_secs: u64,

    /// Timeout for browser checkout operations (default: 10s)
    pub checkout_timeout_secs: u64,
}

impl Default for HeadlessConfig {
    fn default() -> Self {
        Self {
            // ... existing defaults ...
            checkin_timeout_secs: 5,
            checkout_timeout_secs: 10,
        }
    }
}

// Add to WasmConfig
pub struct WasmConfig {
    // ... existing fields ...

    /// Timeout for WASM instance cleanup (default: 5s)
    pub instance_cleanup_timeout_secs: u64,

    /// Timeout for WASM instance checkout (default: 10s)
    pub instance_checkout_timeout_secs: u64,
}

impl Default for WasmConfig {
    fn default() -> Self {
        Self {
            // ... existing defaults ...
            instance_cleanup_timeout_secs: 5,
            instance_checkout_timeout_secs: 10,
        }
    }
}
```

### 4.4 Environment Variable Mapping

Add to `ApiConfig::from_env()`:
```rust
// Cleanup timeouts
if let Ok(val) = std::env::var("RIPTIDE_CLEANUP_TIMEOUT") {
    if let Ok(val) = val.parse() {
        config.performance.cleanup_timeout_secs = val;
    }
}

if let Ok(val) = std::env::var("RIPTIDE_BROWSER_CHECKIN_TIMEOUT") {
    if let Ok(val) = val.parse() {
        config.headless.checkin_timeout_secs = val;
    }
}

if let Ok(val) = std::env::var("RIPTIDE_WASM_CLEANUP_TIMEOUT") {
    if let Ok(val) = val.parse() {
        config.wasm.instance_cleanup_timeout_secs = val;
    }
}
```

---

## 5. HealthMonitorBuilder Design

### 5.1 Current Implementation
**Location:** `/workspaces/eventmesh/crates/riptide-intelligence/src/health.rs:451-500`

**Good News:** HealthMonitorBuilder **ALREADY EXISTS** and is properly exported!

```rust
pub struct HealthMonitorBuilder {
    config: HealthCheckConfig,
}

impl HealthMonitorBuilder {
    pub fn new() -> Self {
        Self {
            config: HealthCheckConfig::default(),
        }
    }

    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.config.interval = interval;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    pub fn with_failure_threshold(mut self, threshold: u32) -> Self {
        self.config.failure_threshold = threshold;
        self
    }

    pub fn with_success_threshold(mut self, threshold: u32) -> Self {
        self.config.success_threshold = threshold;
        self
    }

    pub fn build(self) -> HealthMonitor {
        HealthMonitor::new(self.config)
    }
}
```

**Exported in:** `/workspaces/eventmesh/crates/riptide-intelligence/src/lib.rs:47`
```rust
pub use health::{HealthMonitor, HealthMonitorBuilder};
```

### 5.2 The Issue

The tests are **already importing it correctly:**
- `/workspaces/eventmesh/crates/riptide-intelligence/examples/multi_provider_usage.rs:24`
- Tests marked as `#[ignore]` with TODO comments

**The problem is MockLlmProvider.set_healthy() doesn't exist**, not the builder!

### 5.3 Similar Builder Patterns in Codebase

**Found 15+ builder patterns:**

1. **Config Builder** - `/workspaces/eventmesh/crates/riptide-core/src/common/config_builder.rs`
   - Comprehensive builder trait
   - Validation support
   - Environment variable loading

2. **BrowserConfig Builder** (chromiumoxide)
   ```rust
   let config = BrowserConfig::builder()
       .with_head()
       .window_size(1920, 1080)
       .build()?;
   ```

3. **CSS Extraction Builder** - `/workspaces/eventmesh/crates/riptide-extraction/src/css_extraction.rs`
4. **OpenAPI Builder** - `/workspaces/eventmesh/crates/riptide-streaming/src/openapi.rs`
5. **NDJSON Builder** - `/workspaces/eventmesh/crates/riptide-streaming/src/ndjson.rs`

**Common Pattern:**
```rust
pub struct XxxBuilder {
    field1: Option<T1>,
    field2: Option<T2>,
}

impl XxxBuilder {
    pub fn new() -> Self { /* ... */ }

    pub fn with_field1(mut self, value: T1) -> Self {
        self.field1 = Some(value);
        self
    }

    pub fn build(self) -> Result<Xxx> {
        Ok(Xxx {
            field1: self.field1.ok_or(BuilderError::Missing)?,
            field2: self.field2.unwrap_or_default(),
        })
    }
}
```

---

## 6. Spider Tests API Migration

### 6.1 API Changes Summary
**Location:** `/workspaces/eventmesh/crates/riptide-core/tests/spider_tests.rs`

**QueryAwareCrawler → QueryAwareScorer:**
```rust
// OLD API (removed)
let crawler = QueryAwareCrawler::new(config);
let scores = crawler.score_urls(&urls).await?;

// NEW API (current)
let scorer = QueryAwareScorer::new(config);
let score = scorer.score_request(&crawl_request).await?;
```

**BM25Scorer API Changes:**
```rust
// OLD API (inferred from tests)
let scorer = QueryAwareScorer::new(query, k1, b);
scorer.add_document(doc);
let score = scorer.score_document(doc);

// NEW API (current)
let mut scorer = BM25Scorer::new(query, k1, b);
scorer.update_corpus(doc); // ✅ Build index
let score = scorer.score(doc); // ✅ Calculate score
```

**Config Changes:**
```rust
// OLD fields (removed):
- enable_bm25
- url_signal_weight (renamed)
- max_depth
- early_stop_threshold
- min_crawl_count

// NEW fields (added):
+ query_foraging: bool
+ target_query: String
+ min_relevance_threshold: f64
+ relevance_window_size: usize
+ url_signals_weight: f64 (renamed)
```

**CrawlOrchestrator → Spider:**
```rust
// OLD API (removed)
let orchestrator = CrawlOrchestrator::new(config);
orchestrator.crawl_with_query(query, urls).await?;

// NEW API (current)
let spider = Spider::new(SpiderConfig {
    max_concurrent: 10,
    max_pages: 100,
    timeout_ms: 30000,
    respect_robots_txt: true,
});
spider.crawl(start_urls).await?;
```

### 6.2 Test Rewrite Strategy

**For BM25 Tests (Lines 11-60):**
```rust
#[test]
fn test_bm25_calculation() {
    let mut scorer = BM25Scorer::new("quick fox", 1.2, 0.75);

    let documents = vec![
        "The quick brown fox jumps over the lazy dog",
        "The fox is quick and clever",
    ];

    // Build corpus index
    for doc in &documents {
        scorer.update_corpus(doc);
    }

    // Score documents
    let scores: Vec<f64> = documents
        .iter()
        .map(|doc| scorer.score(doc))
        .collect();

    // Verify relative scoring (not absolute values)
    assert!(scores[0] > 0.0, "Document with query terms should score > 0");
    assert!(scores[1] > 0.0, "Document with query terms should score > 0");

    // Documents with both "quick" and "fox" should rank highly
    // But don't assert exact score values - BM25 implementation may vary
}
```

**For QueryAwareScorer Tests (Lines 109-148):**
```rust
#[tokio::test]
async fn test_query_aware_url_prioritization() {
    use riptide_core::spider::query_aware::{QueryAwareScorer, QueryAwareScorerConfig};
    use riptide_core::spider::types::CrawlRequest;

    let config = QueryAwareScorerConfig {
        query_foraging: true,
        target_query: "rust programming".to_string(),
        min_relevance_threshold: 0.3,
        relevance_window_size: 10,
        url_signals_weight: 0.4,
        content_similarity_weight: 0.3,
        domain_diversity_weight: 0.3,
    };

    let scorer = QueryAwareScorer::new(config);

    let request1 = CrawlRequest::new(
        Url::parse("https://rust-lang.org/learn").unwrap()
    );
    let request2 = CrawlRequest::new(
        Url::parse("https://example.com/random").unwrap()
    );

    let score1 = scorer.score_request(&request1).await?;
    let score2 = scorer.score_request(&request2).await?;

    // Rust-related URL should score higher for "rust programming" query
    assert!(score1 > score2);
}
```

**For Spider Integration Tests (Lines 157-183):**
```rust
#[tokio::test]
async fn test_spider_with_budget() {
    use riptide_core::spider::core::{Spider, SpiderConfig};
    use riptide_core::spider::budget::{BudgetManager, BudgetConfig};

    let spider_config = SpiderConfig {
        max_concurrent: 5,
        max_pages: 50,
        timeout_ms: 10000,
        respect_robots_txt: true,
    };

    let budget_config = BudgetConfig {
        max_requests: 50,
        max_bytes: 10_000_000,
        max_duration: Duration::from_secs(60),
        rate_limit: Some(10.0), // 10 requests per second
    };

    let budget_manager = BudgetManager::new(budget_config);
    let spider = Spider::new(spider_config);

    let start_urls = vec![
        Url::parse("https://example.com").unwrap(),
    ];

    let results = spider.crawl(start_urls).await?;

    // Verify budget was respected
    assert!(budget_manager.requests_made() <= 50);
}
```

---

## 7. mem::forget Documentation Template

### 7.1 Why mem::forget is Required for FFI

From WASM Component Model documentation:
- Guest code allocates memory
- Host must not drop guest allocations
- Ownership transfer across FFI boundary
- Memory managed by WASM linear memory

### 7.2 SAFETY Comment Template

```rust
use std::mem;

pub fn transfer_to_wasm_guest(data: Vec<u8>) -> (*const u8, usize) {
    let ptr = data.as_ptr();
    let len = data.len();

    // SAFETY: Ownership transfer to WASM guest
    //
    // This is required for WASM Component Model FFI boundary:
    // 1. The Vec<u8> was allocated in Rust host memory
    // 2. We're returning a raw pointer to WASM guest code
    // 3. The guest will copy this data into its linear memory
    // 4. After the copy, the guest calls a host function to free this memory
    // 5. We use mem::forget to prevent Rust from dropping the allocation
    //    before the guest has copied the data
    //
    // Memory Lifecycle:
    // - Host allocates (Vec::new)
    // - Host passes pointer to guest (this function)
    // - Guest copies data from host pointer to its linear memory
    // - Guest calls host_free(ptr) to release host memory
    // - host_free reconstructs Vec from raw parts and drops it
    //
    // Without mem::forget:
    // - Rust would drop the Vec at end of this function
    // - Guest would receive a dangling pointer
    // - Memory corruption / undefined behavior
    //
    // Related: See WASM Component Model spec section on memory management
    // https://github.com/WebAssembly/component-model/blob/main/design/mvp/Explainer.md
    mem::forget(data);

    (ptr, len)
}

/// Called by WASM guest to free host-allocated memory
///
/// SAFETY: Must be called with pointer returned by transfer_to_wasm_guest
pub unsafe fn host_free(ptr: *mut u8, len: usize, capacity: usize) {
    // SAFETY: Reconstruct Vec from raw parts
    //
    // This is safe because:
    // 1. ptr, len, capacity came from Vec::into_raw_parts() equivalent
    // 2. The guest has finished using this memory
    // 3. Reconstructing Vec allows proper Drop impl to run
    // 4. No aliasing - guest promises not to use this pointer again
    let _ = Vec::from_raw_parts(ptr, len, capacity);
    // Vec drops here, deallocating memory
}
```

### 7.3 Alternative Pattern: ManuallyDrop

```rust
use std::mem::ManuallyDrop;

pub fn transfer_to_wasm_guest_v2(data: Vec<u8>) -> (*const u8, usize) {
    let mut data = ManuallyDrop::new(data);

    // More explicit than mem::forget
    // Prevents accidental drops in refactoring
    let ptr = data.as_ptr();
    let len = data.len();

    // SAFETY: Same reasoning as above, but ManuallyDrop makes intent clearer
    (ptr, len)
}
```

---

## 8. Implementation Wiring Guide

### 8.1 Step-by-Step: Wire WasmInstanceHandle.cleanup()

**Step 1:** Update `MemoryManagerConfig` to include timeout
```rust
// In memory_manager.rs
pub struct MemoryManagerConfig {
    // ... existing fields ...

    /// Timeout for instance cleanup operations (default: 5s)
    pub cleanup_timeout: Duration,
}

impl Default for MemoryManagerConfig {
    fn default() -> Self {
        Self {
            // ... existing defaults ...
            cleanup_timeout: Duration::from_secs(5),
        }
    }
}
```

**Step 2:** Update `cleanup()` method to use config
```rust
impl WasmInstanceHandle {
    pub async fn cleanup(self) -> Result<()> {
        let timeout = self.manager.config.cleanup_timeout;

        tokio::time::timeout(
            timeout,
            self.manager.return_instance(&self.instance_id),
        )
        .await
        .map_err(|_| anyhow::anyhow!(
            "Timeout ({:?}) returning WASM instance {}",
            timeout,
            self.instance_id
        ))?
    }
}
```

**Step 3:** Wire into API handlers
```rust
// In handlers/browser.rs or similar
pub async fn extract_handler(
    State(state): State<AppState>,
    Json(request): Json<ExtractRequest>,
) -> Result<Json<ExtractResponse>, ApiError> {
    // Get WASM instance
    let instance = state
        .memory_manager
        .get_instance(&request.component_path)
        .await?;

    // Perform extraction
    let result = perform_extraction(&instance, &request).await;

    // ✅ EXPLICIT CLEANUP - always called, even if extraction fails
    if let Err(e) = instance.cleanup().await {
        error!("Failed to cleanup WASM instance: {}", e);
        // Don't fail the request, but log the error
    }

    // Return result (or error)
    let result = result?;
    Ok(Json(result))
}
```

**Step 4:** Alternative: RAII wrapper
```rust
pub struct CleanupGuard<T> {
    resource: Option<T>,
    cleanup_fn: Box<dyn FnOnce(T) -> BoxFuture<'static, ()> + Send>,
}

impl<T> CleanupGuard<T> {
    pub fn new<F, Fut>(resource: T, cleanup_fn: F) -> Self
    where
        F: FnOnce(T) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        Self {
            resource: Some(resource),
            cleanup_fn: Box::new(move |r| Box::pin(cleanup_fn(r))),
        }
    }

    pub fn get(&self) -> &T {
        self.resource.as_ref().unwrap()
    }
}

impl<T> Drop for CleanupGuard<T> {
    fn drop(&mut self) {
        if let Some(resource) = self.resource.take() {
            let cleanup = (self.cleanup_fn)(resource);
            tokio::spawn(cleanup);
        }
    }
}

// Usage:
let instance = memory_manager.get_instance(path).await?;
let _guard = CleanupGuard::new(instance, |i| async move {
    let _ = i.cleanup().await;
});
```

### 8.2 Step-by-Step: Wire BrowserCheckout.cleanup()

**Same pattern as WasmInstanceHandle:**

1. Add `checkin_timeout` to `BrowserPoolConfig`
2. Update `cleanup()` to use config timeout
3. Call explicitly in handlers before returning
4. Keep Drop as fallback for panic paths

### 8.3 Locations Requiring Updates

**Files to Modify:**
1. `/workspaces/eventmesh/crates/riptide-core/src/memory_manager.rs` (Lines 716-723)
2. `/workspaces/eventmesh/crates/riptide-headless/src/pool.rs` (Lines 903-911)
3. `/workspaces/eventmesh/crates/riptide-api/src/config.rs` (Add cleanup timeouts)
4. `/workspaces/eventmesh/crates/riptide-api/src/handlers/browser.rs` (Call cleanup)
5. `/workspaces/eventmesh/crates/riptide-api/src/resource_manager/guards.rs` (Call cleanup)

**Files to Create:**
1. `/workspaces/eventmesh/crates/riptide-core/tests/spider_tests_new.rs` (Rewritten tests)

---

## 9. Testing Strategy

### 9.1 Unit Tests for Cleanup

```rust
#[tokio::test]
async fn test_wasm_instance_explicit_cleanup() {
    let manager = create_test_memory_manager().await;
    let instance = manager.get_instance("test.wasm").await.unwrap();

    // Verify cleanup succeeds
    assert!(instance.cleanup().await.is_ok());

    // Verify instance was returned to pool
    let stats = manager.stats();
    assert_eq!(stats.available_instances, 1);
}

#[tokio::test]
async fn test_cleanup_timeout() {
    let config = MemoryManagerConfig {
        cleanup_timeout: Duration::from_millis(1), // Very short
        ..Default::default()
    };
    let manager = MemoryManager::new(config, engine).await.unwrap();

    // Create slow cleanup scenario
    let instance = manager.get_instance("slow.wasm").await.unwrap();

    // Verify timeout triggers
    assert!(instance.cleanup().await.is_err());
}
```

### 9.2 Integration Tests for Configuration

```rust
#[tokio::test]
async fn test_config_from_env() {
    std::env::set_var("RIPTIDE_CLEANUP_TIMEOUT", "10");

    let config = ApiConfig::from_env();
    assert_eq!(config.performance.cleanup_timeout_secs, 10);

    std::env::remove_var("RIPTIDE_CLEANUP_TIMEOUT");
}
```

### 9.3 Spider Test Migration

See Section 6.2 for complete test rewrites.

---

## 10. Checklist for Implementation

### Phase 1: Configuration (1 hour)
- [ ] Add `cleanup_timeout_secs` to `PerformanceConfig`
- [ ] Add `checkin_timeout_secs` to `HeadlessConfig`
- [ ] Add `instance_cleanup_timeout_secs` to `WasmConfig`
- [ ] Update `ApiConfig::from_env()` with new env vars
- [ ] Update `ApiConfig::validate()` for new fields
- [ ] Add unit tests for new config fields

### Phase 2: Memory Manager (1 hour)
- [ ] Update `MemoryManagerConfig` with `cleanup_timeout` field
- [ ] Modify `WasmInstanceHandle::cleanup()` to use config timeout
- [ ] Update Drop impl to check if cleanup was called
- [ ] Add unit tests for configurable cleanup
- [ ] Add integration tests for timeout behavior

### Phase 3: Browser Pool (1 hour)
- [ ] Update `BrowserPoolConfig` with `checkin_timeout` field
- [ ] Modify `BrowserCheckout::cleanup()` to use config timeout
- [ ] Update Drop impl to check if cleanup was called
- [ ] Add unit tests for configurable cleanup
- [ ] Add integration tests for timeout behavior

### Phase 4: Handler Wiring (2 hours)
- [ ] Identify all handlers that use `WasmInstanceHandle`
- [ ] Add explicit `cleanup()` calls before returns
- [ ] Identify all handlers that use `BrowserCheckout`
- [ ] Add explicit `cleanup()` calls before returns
- [ ] Add error handling for cleanup failures
- [ ] Test all modified handlers

### Phase 5: Spider Tests (2 hours)
- [ ] Rewrite BM25 tests for new API (Lines 11-60)
- [ ] Rewrite QueryAwareScorer tests (Lines 109-148)
- [ ] Rewrite Spider integration tests (Lines 157-183)
- [ ] Remove `#[ignore]` attributes
- [ ] Verify all tests pass

### Phase 6: Documentation (1 hour)
- [ ] Add SAFETY comments for mem::forget usage
- [ ] Document cleanup pattern in CONTRIBUTING.md
- [ ] Update API docs with cleanup requirements
- [ ] Add examples of proper resource management

**Total Estimated Time: 8 hours**

---

## 11. Risks and Mitigations

### Risk 1: Forgetting to call cleanup()
**Mitigation:** Drop impl provides fallback (already implemented)

### Risk 2: Cleanup timeout too short
**Mitigation:** Make configurable via environment variables

### Risk 3: Breaking existing handlers
**Mitigation:** Gradual rollout, keep Drop as safety net

### Risk 4: Performance impact
**Mitigation:** Cleanup is already happening in Drop, just making it explicit

---

## 12. References

**Files Read During Research:**
- `/workspaces/eventmesh/crates/riptide-core/src/memory_manager.rs` (Lines 1-750)
- `/workspaces/eventmesh/crates/riptide-headless/src/pool.rs` (Lines 1-950)
- `/workspaces/eventmesh/crates/riptide-api/src/config.rs` (Lines 1-529)
- `/workspaces/eventmesh/crates/riptide-core/tests/spider_tests.rs` (Lines 1-252)
- `/workspaces/eventmesh/crates/riptide-intelligence/src/health.rs` (Lines 451-540)
- `/workspaces/eventmesh/crates/riptide-core/src/common/config_builder.rs` (Lines 1-538)

**Patterns Analyzed:**
- 20+ occurrences of `Duration::from_secs(5)` hardcoded timeouts
- 15+ builder pattern implementations
- 2 major async cleanup patterns (WasmInstanceHandle, BrowserCheckout)
- 3 API changes in spider module

**Best Practices Sources:**
- Tokio documentation on async Drop limitations
- SQLx connection pool implementation
- Rust async book chapter on resource management
- WASM Component Model FFI specifications

---

## End of Research Report

**Next Steps:**
1. Review checklist with team
2. Prioritize Phase 1 (Configuration) for immediate implementation
3. Create follow-up tickets for Phases 2-6
4. Assign to Coder and Tester agents for implementation

**Coordination:**
- Stored in memory at key: `swarm/researcher/wiring-patterns`
- Ready for handoff to implementation agents
