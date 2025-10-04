# Dead Code Analysis Report - RipTide Workspace

**Generated:** 2025-10-04  
**Analyst:** Claude Code  
**Compiler Errors:** 17 dead code warnings

---

## Executive Summary

This document catalogs all dead code, unused variables, imports, and incomplete implementations found during workspace analysis. Each item is categorized and includes recommended actions.

### Summary Statistics

- **Total Issues:** 21 dead code items
- **Files Affected:** 8 files in `crates/riptide-api/src/`
- **Categories:**
  - INCOMPLETE: 15 items (need implementation)
  - REFACTOR: 3 items (use existing code)
  - REMOVE: 3 items (true dead code)
- **Risk Levels:**
  - Low: 11 items
  - Medium: 10 items
  - High: 0 items

---

## Category Definitions

1. **INCOMPLETE** - Partial implementations that need completion to fulfill intended purpose
2. **REFACTOR** - Code that duplicates or should use existing implementations
3. **REMOVE** - True dead code with no purpose (placeholders, unused after refactoring)

---

## Dead Code Findings by File

### 1. health.rs

**Location:** `crates/riptide-api/src/health.rs:309`

#### Finding: `verify_http_client_config` function

**Code:**
```rust
fn verify_http_client_config(_state: &AppState) -> bool {
    // Verify timeout settings, connection pool, etc.
    // This is a placeholder - add actual configuration checks as needed
    true
}
```

**Category:** REMOVE  
**Status:** Never called, always returns true  
**Reason:** Placeholder function with no actual validation logic  

**Action:** Delete lines 308-313  

**Risk Level:** Low - No functionality loss  
**Tests Needed:** None (placeholder had no tests)  
**Dependencies:** None  

---

### 2. pipeline.rs

**Location:** `crates/riptide-api/src/pipeline.rs`

#### Finding A: `fetch_content` method (Line 524-548)

**Code:**
```rust
async fn fetch_content(&self, url: &str) -> ApiResult<(Response, String)> {
    let fetch_timeout = Duration::from_secs(15);
    
    let response = timeout(fetch_timeout, fetch::get(&self.state.http_client, url))
        .await
        .map_err(|_| ApiError::timeout("content_fetch", format!("Timeout fetching {}", url)))?
        .map_err(|e| ApiError::fetch(url, e.to_string()))?;
    
    let content = timeout(fetch_timeout, response.text())
        .await
        .map_err(|_| {
            ApiError::timeout(
                "content_read",
                format!("Timeout reading content from {}", url),
            )
        })?
        .map_err(|e| ApiError::fetch(url, format!("Failed to read response body: {}", e)))?;
    
    // Recreate response for status code (since we consumed it for text)
    let response = fetch::get(&self.state.http_client, url)
        .await
        .map_err(|e| ApiError::fetch(url, e.to_string()))?;
    
    Ok((response, content))
}
```

**Category:** REFACTOR  
**Status:** Never called  
**Reason:** Duplicates `fetch_content_with_type` (line 432) which is actually used  

**Current Usage Analysis:**
- `fetch_content_with_type` is called in `execute_single` (line 186)
- Returns `(Response, Vec<u8>, Option<String>)` with content-type detection
- This method returns `(Response, String)` - less capable version

**Action:** Remove this method entirely (lines 524-548)  

**Risk Level:** Low - Identical functionality exists in better form  
**Tests Needed:** Verify `fetch_content_with_type` has coverage  
**Dependencies:** None - method is unused  

---

#### Finding B: `extract_with_headless` method (Line 721-735)

**Code:**
```rust
async fn extract_with_headless(&self, url: &str) -> ApiResult<ExtractedDoc> {
    // If headless service is not configured, fall back to fast extraction
    match &self.state.config.headless_url {
        Some(headless_url) => self.render_and_extract(url, headless_url).await,
        None => {
            warn!(url = %url, "Headless extraction requested but no headless service configured, using fast extraction");
            // Try fast extraction as fallback
            self.state
                .extractor
                .extract(&[], url, "article") // Empty HTML, will need to fetch first
                .map(convert_html_doc)
                .map_err(|e| ApiError::extraction(format!("Fallback extraction failed: {}", e)))
        }
    }
}
```

**Category:** INCOMPLETE  
**Status:** Implemented but never integrated  
**Reason:** Decision::Headless case goes through ReliableExtractor instead  

**Current Flow:**
```rust
// In extract_content (line 629)
Decision::Headless => {
    // Uses ReliableExtractor, not this method
    self.state.reliable_extractor.extract_with_reliability(...)
}
```

**Integration Point:** Line 629-700 in `extract_content` method  

**Action:** Integrate as direct headless path
```rust
async fn extract_content(&self, html: &str, url: &str, decision: Decision) -> ApiResult<ExtractedDoc> {
    use crate::reliability_integration::WasmExtractorAdapter;
    use riptide_core::reliability::ExtractionMode;

    match decision {
        Decision::Raw | Decision::ProbesFirst => {
            // Existing ReliableExtractor path
            let extractor_adapter = WasmExtractorAdapter::new(self.state.extractor.clone());
            let extraction_mode = match decision {
                Decision::Raw => ExtractionMode::Fast,
                Decision::ProbesFirst => ExtractionMode::ProbesFirst,
                _ => unreachable!(),
            };
            
            self.state.reliable_extractor
                .extract_with_reliability(url, extraction_mode, &extractor_adapter, None)
                .await
                .or_else(|e| {
                    warn!("ReliableExtractor failed: {}, trying WASM fallback", e);
                    self.fallback_to_wasm_extraction(html, url)
                })
                .await
        }
        Decision::Headless => {
            // NEW: Use direct headless rendering when available
            if self.state.config.headless_url.is_some() {
                info!("Using direct headless rendering for {}", url);
                self.extract_with_headless(url).await
                    .or_else(|e| {
                        warn!("Headless extraction failed: {}, trying ReliableExtractor", e);
                        // Fallback to ReliableExtractor
                        let extractor_adapter = WasmExtractorAdapter::new(self.state.extractor.clone());
                        self.state.reliable_extractor
                            .extract_with_reliability(
                                url,
                                ExtractionMode::Headless,
                                &extractor_adapter,
                                self.state.config.headless_url.as_deref()
                            )
                    })
                    .await
            } else {
                warn!("Headless requested but not configured, using ReliableExtractor");
                let extractor_adapter = WasmExtractorAdapter::new(self.state.extractor.clone());
                self.state.reliable_extractor
                    .extract_with_reliability(
                        url,
                        ExtractionMode::Headless,
                        &extractor_adapter,
                        None
                    )
                    .await
            }
        }
    }
}
```

**Risk Level:** Medium - Affects headless rendering path  
**Tests Needed:**
- Test direct headless extraction path
- Test fallback to ReliableExtractor on headless failure
- Integration test with mock headless service
- Integration test without headless service configured

**Dependencies:** Finding C (`render_and_extract`)  

---

#### Finding C: `render_and_extract` method (Line 738-778)

**Code:**
```rust
async fn render_and_extract(&self, url: &str, headless_url: &str) -> ApiResult<ExtractedDoc> {
    let http_client = self.state.http_client.clone();
    let extractor = self.state.extractor.clone();
    let headless_url_str = headless_url.to_string();
    let url_str = url.to_string();
    let wait_for = self.options.dynamic_wait_for.clone();
    let scroll_steps = self.options.scroll_steps;

    // Construct headless service request
    let render_request = serde_json::json!({
        "url": url_str,
        "wait_for": wait_for,
        "scroll_steps": scroll_steps
    });

    let response = http_client
        .post(format!("{}/render", headless_url_str))
        .json(&render_request)
        .send()
        .await
        .map_err(|e| {
            ApiError::dependency("headless", format!("Headless request failed: {}", e))
        })?;

    if !response.status().is_success() {
        return Err(ApiError::dependency(
            "headless",
            format!("Render request failed: {}", response.status()),
        ));
    }

    let rendered_html = response.text().await.map_err(|e| {
        ApiError::dependency("headless", format!("Failed to read rendered HTML: {}", e))
    })?;

    // Extract from rendered HTML
    extractor
        .extract(rendered_html.as_bytes(), &url_str, "article")
        .map(convert_html_doc)
        .map_err(|e| ApiError::extraction(format!("Headless extraction failed: {}", e)))
}
```

**Category:** INCOMPLETE  
**Status:** Called by `extract_with_headless` but both unused  
**Reason:** Same as Finding B  

**Action:** Keep method, integrate via Finding B  

**Risk Level:** Medium  
**Tests Needed:** Same as Finding B  
**Dependencies:** Finding B (`extract_with_headless`)  

---

### 3. pipeline_dual.rs

**Location:** `crates/riptide-api/src/pipeline_dual.rs:89-97`

#### Finding: Unused fields in `DualPathOrchestrator`

**Code:**
```rust
pub struct DualPathOrchestrator {
    state: AppState,              // Line 90 - Never read
    options: CrawlOptions,         // Line 91 - Never read
    config: DualPathConfig,        // Used
    metrics: Arc<RipTideMetrics>,  // Line 93 - Never read
    ai_processor: Arc<RwLock<BackgroundAiProcessor>>, // Used
    event_bus: Arc<EventBus>,      // Used
    pending_results: Arc<RwLock<HashMap<String, FastPathResult>>>, // Used
}
```

**Category:** INCOMPLETE  
**Status:** Fields stored but functionality not implemented  
**Reason:** Dual-path orchestrator creates its own resources instead of using state  

**Current Problem Areas:**

1. **Line 281-293: `fetch_content` creates new FetchEngine**
```rust
async fn fetch_content(&self, url: &str) -> ApiResult<String> {
    use riptide_core::fetch::FetchEngine;
    
    let fetch_engine = FetchEngine::new()  // Should use self.state.http_client!
        .map_err(|e| ApiError::internal(format!("Failed to create fetch engine: {}", e)))?;
    
    let content = fetch_engine
        .fetch_text(url)
        .await
        .map_err(|e| ApiError::fetch(url, format!("Fetch failed: {}", e)))?;
    
    Ok(content)
}
```

2. **Line 296-326: `extract_with_css` uses hardcoded extraction**
```rust
async fn extract_with_css(&self, url: &str, content: &str) -> ApiResult<ExtractedDoc> {
    // Hardcoded text extraction instead of using self.state.extractor!
    let text = content
        .chars()
        .filter(|c| !c.is_control())
        .take(10000)
        .collect::<String>();
    // ...
}
```

3. **No metrics recording anywhere**

**Action Plan:**

**Step 1:** Use `state.http_client` for fetching
```rust
async fn fetch_content(&self, url: &str) -> ApiResult<String> {
    use riptide_core::fetch;
    
    let response = fetch::get(&self.state.http_client, url)
        .await
        .map_err(|e| ApiError::fetch(url, format!("Fetch failed: {}", e)))?;
    
    let content = response.text()
        .await
        .map_err(|e| ApiError::fetch(url, format!("Read failed: {}", e)))?;
    
    Ok(content)
}
```

**Step 2:** Use `state.extractor` for CSS extraction
```rust
async fn extract_with_css(&self, url: &str, content: &str) -> ApiResult<ExtractedDoc> {
    use crate::pipeline::convert_html_doc;
    
    // Use actual WASM extractor from state
    let extractor = &self.state.extractor;
    let html_doc = extractor
        .extract(content.as_bytes(), url, "article")
        .map_err(|e| ApiError::extraction(format!("CSS extraction failed: {}", e)))?;
    
    Ok(convert_html_doc(html_doc))
}
```

**Step 3:** Record metrics throughout execution
```rust
async fn execute(&self, url: &str) -> ApiResult<DualPathResult> {
    let task_id = Uuid::new_v4().to_string();
    let overall_start = Instant::now();
    
    // Record start
    self.metrics.dual_path_requests.inc();
    
    // Fast path
    let fast_path_result = self.execute_fast_path(&task_id, url).await?;
    self.metrics.fast_path_duration.observe(fast_path_result.processing_time_ms as f64 / 1000.0);
    
    // Enhancement path
    let should_enhance = self.config.enable_ai_enhancement
        && fast_path_result.quality_score < self.config.enhancement_quality_threshold;
    
    if should_enhance {
        self.queue_ai_enhancement(&task_id, url, &fast_path_result).await?;
        self.metrics.ai_enhancements_queued.inc();
    }
    
    // Build result
    let result = DualPathResult { /* ... */ };
    
    // Record completion
    self.metrics.dual_path_duration.observe(result.total_time_ms as f64 / 1000.0);
    self.metrics.dual_path_quality_score.observe(result.quality_score as f64);
    
    Ok(result)
}
```

**Step 4:** Use `options` for configuration
```rust
async fn execute_fast_path(&self, task_id: &str, url: &str) -> ApiResult<FastPathResult> {
    // Use options for cache mode
    if self.options.cache_mode != "bypass" {
        // Check cache first
        let cache_key = self.generate_cache_key(url);
        if let Some(cached) = self.check_cache(&cache_key).await? {
            return Ok(FastPathResult { /* cached result */ });
        }
    }
    
    // ... rest of implementation
}
```

**Risk Level:** Medium - Affects dual-path pipeline core functionality  
**Tests Needed:**
- Test HTTP client reuse from state
- Test extractor integration from state
- Test metrics recording at all stages
- Test cache integration via options
- Integration test for complete dual-path flow

**Dependencies:** None  
**Required Metrics:** Add to RipTideMetrics
```rust
// In metrics.rs
pub struct RipTideMetrics {
    // ... existing metrics
    pub dual_path_requests: Counter,
    pub fast_path_duration: Histogram,
    pub dual_path_duration: Histogram,
    pub dual_path_quality_score: Histogram,
    pub ai_enhancements_queued: Counter,
}
```

---

### 4. resource_manager.rs

**Location:** `crates/riptide-api/src/resource_manager.rs`

#### Finding A: `cleanup_task` in `PerHostRateLimiter` (Line 57)

**Code:**
```rust
pub struct PerHostRateLimiter {
    config: ApiConfig,
    host_buckets: RwLock<HashMap<String, HostBucket>>,
    cleanup_task: Mutex<Option<tokio::task::JoinHandle<()>>>,  // Never read
    metrics: Arc<ResourceMetrics>,
}
```

**Category:** INCOMPLETE  
**Status:** Field allocated but `start_cleanup_task` is stub (line 485-488)  
**Reason:** Cleanup task stored but implementation missing  

**Current Stub:**
```rust
async fn start_cleanup_task(&self) {
    // Implementation for periodic cleanup of old host buckets
    // This would run in the background to prevent memory leaks
}
```

**Problem:** Rate limiter accumulates host buckets indefinitely → memory leak

**Action:** Implement cleanup task
```rust
async fn start_cleanup_task(&self) {
    let host_buckets = self.host_buckets.clone();
    let cleanup_interval = Duration::from_secs(300); // 5 minutes
    let idle_timeout = Duration::from_secs(3600); // 1 hour
    
    let task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(cleanup_interval);
        loop {
            interval.tick().await;
            
            let mut buckets = host_buckets.write().await;
            let now = Instant::now();
            let initial_count = buckets.len();
            
            // Remove buckets idle for more than 1 hour
            buckets.retain(|host, bucket| {
                let idle_time = now.duration_since(bucket.last_request);
                if idle_time >= idle_timeout {
                    debug!("Removing idle rate limit bucket for host: {}", host);
                    false
                } else {
                    true
                }
            });
            
            let removed = initial_count - buckets.len();
            if removed > 0 {
                info!(
                    "Rate limiter cleanup: removed {} idle buckets, {} active remaining",
                    removed,
                    buckets.len()
                );
            }
        }
    });
    
    *self.cleanup_task.lock().await = Some(task);
}
```

**Risk Level:** Low - Memory leak prevention  
**Tests Needed:**
- Test bucket cleanup after idle period
- Test retention of active buckets
- Test cleanup interval timing
- Test task lifecycle (start/stop)

**Dependencies:** None  

---

#### Finding B: `config` in `WasmInstanceManager` (Line 72)

**Code:**
```rust
pub struct WasmInstanceManager {
    config: ApiConfig,  // Never read
    worker_instances: RwLock<HashMap<String, WasmWorkerInstance>>,
    metrics: Arc<ResourceMetrics>,
}
```

**Category:** INCOMPLETE  
**Status:** Config stored but not used for limits/timeouts  
**Reason:** No enforcement of instance limits or lifecycle management  

**Current Implementation (Line 500-527):**
```rust
async fn acquire_instance(self: &Arc<Self>, worker_id: &str) -> Result<WasmGuard> {
    let mut instances = self.worker_instances.write().await;
    
    // Ensures single instance per worker (requirement)
    if !instances.contains_key(worker_id) {
        let instance = WasmWorkerInstance { /* ... */ };
        instances.insert(worker_id.to_string(), instance);
        self.metrics.wasm_instances.fetch_add(1, Ordering::Relaxed);
    }
    
    // No config checks!
    // No instance limits!
    // No timeout handling!
    
    Ok(WasmGuard { /* ... */ })
}
```

**Action:** Use config for instance management
```rust
async fn acquire_instance(self: &Arc<Self>, worker_id: &str) -> Result<WasmGuard> {
    let mut instances = self.worker_instances.write().await;
    
    // Check global instance limit from config
    if instances.len() >= self.config.wasm.max_instances {
        return Err(anyhow!(
            "WASM instance limit reached: {}/{}", 
            instances.len(), 
            self.config.wasm.max_instances
        ));
    }
    
    // Check if existing instance needs recycling
    if let Some(instance) = instances.get(worker_id) {
        let age = instance.created_at.elapsed();
        let idle = instance.last_operation.elapsed();
        
        // Recycle if too old or idle too long
        if age > self.config.wasm.max_instance_age {
            warn!("WASM instance {} exceeded max age {:?}, recycling", 
                  worker_id, self.config.wasm.max_instance_age);
            instances.remove(worker_id);
        } else if idle > self.config.wasm.idle_timeout {
            warn!("WASM instance {} idle for {:?}, recycling", 
                  worker_id, idle);
            instances.remove(worker_id);
        }
    }
    
    // Create new instance if needed
    if !instances.contains_key(worker_id) {
        let instance = WasmWorkerInstance {
            worker_id: worker_id.to_string(),
            created_at: Instant::now(),
            operations_count: 0,
            last_operation: Instant::now(),
            is_healthy: true,
            memory_usage: 0,
        };
        instances.insert(worker_id.to_string(), instance);
        self.metrics.wasm_instances.fetch_add(1, Ordering::Relaxed);
        info!("Created new WASM instance for worker {}", worker_id);
    }
    
    // Update instance usage
    if let Some(instance) = instances.get_mut(worker_id) {
        instance.operations_count += 1;
        instance.last_operation = Instant::now();
    }
    
    Ok(WasmGuard {
        worker_id: worker_id.to_string(),
        manager: self.clone(),
    })
}
```

**Risk Level:** Medium - Affects resource limits and lifecycle  
**Tests Needed:**
- Test instance limit enforcement
- Test instance recycling on max age
- Test instance recycling on idle timeout
- Test instance creation after recycling

**Dependencies:** Requires `WasmConfig` in `ApiConfig`  
**Configuration Addition:**
```rust
pub struct WasmConfig {
    pub max_instances: usize,
    pub max_instance_age: Duration,
    pub idle_timeout: Duration,
    pub max_memory_mb: usize,
}

impl Default for WasmConfig {
    fn default() -> Self {
        Self {
            max_instances: 10,
            max_instance_age: Duration::from_secs(1800), // 30 min
            idle_timeout: Duration::from_secs(600),       // 10 min
            max_memory_mb: 512,
        }
    }
}
```

---

#### Finding C: Unused fields in `WasmWorkerInstance` (Lines 80-85)

**Code:**
```rust
struct WasmWorkerInstance {
    pub worker_id: String,      // Line 80 - Never read
    pub created_at: Instant,    // Line 81 - Never read
    pub operations_count: u64,  // Used
    pub last_operation: Instant, // Used
    pub is_healthy: bool,       // Line 84 - Never read
    pub memory_usage: usize,    // Line 85 - Never read
}
```

**Category:** INCOMPLETE  
**Status:** Monitoring fields exist but not used for health checks  
**Reason:** No health validation logic implemented  

**Action:** Implement health checking
```rust
impl WasmWorkerInstance {
    /// Check if instance is healthy based on config thresholds
    fn is_healthy(&self, config: &WasmConfig) -> bool {
        let age = self.created_at.elapsed();
        let idle = self.last_operation.elapsed();
        
        self.is_healthy
            && age < config.max_instance_age
            && idle < config.idle_timeout
            && self.memory_usage < config.max_memory_mb * 1024 * 1024
    }
    
    /// Update health status based on operation result
    fn update_health(&mut self, success: bool) {
        if !success {
            self.is_healthy = false;
        }
    }
    
    /// Track memory usage
    fn track_memory(&mut self, bytes: usize) {
        self.memory_usage += bytes;
    }
}

// Use in acquire_instance (in Finding B implementation)
if let Some(instance) = instances.get(worker_id) {
    if !instance.is_healthy(self.config) {
        warn!("Unhealthy WASM instance {} detected, removing", worker_id);
        instances.remove(worker_id);
    }
}
```

**Risk Level:** Medium - Affects instance health monitoring  
**Tests Needed:**
- Test health check with various conditions
- Test unhealthy instance removal
- Test memory tracking
- Test health status updates

**Dependencies:** Finding B (needs `WasmConfig`)  

---

#### Finding D: `config` and `last_analysis` in `PerformanceMonitor` (Lines 100, 104)

**Code:**
```rust
pub struct PerformanceMonitor {
    config: ApiConfig,        // Line 100 - Never read
    render_times: Mutex<Vec<Duration>>,
    timeout_count: AtomicU64,
    degradation_score: std::sync::atomic::AtomicU64,
    last_analysis: AtomicU64, // Line 104 - Never read
    metrics: Arc<ResourceMetrics>,
}
```

**Category:** INCOMPLETE  
**Status:** Performance analysis fields exist but no analysis implemented  
**Reason:** `get_degradation_score` just reads stored value, never computes it  

**Current Implementation (Line 632-635):**
```rust
async fn get_degradation_score(&self) -> f64 {
    // Just reads stored value, never computed!
    f64::from_bits(self.degradation_score.load(Ordering::Relaxed))
}
```

**Action:** Implement performance analysis
```rust
pub async fn analyze_performance(&self) -> PerformanceDegradation {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let last = self.last_analysis.swap(now, Ordering::Relaxed);
    
    // Only analyze if enough time has passed (e.g., 60 seconds)
    if now - last < self.config.performance.analysis_interval_secs {
        return PerformanceDegradation::None;
    }
    
    let render_times = self.render_times.lock().await;
    if render_times.is_empty() {
        return PerformanceDegradation::None;
    }
    
    // Calculate average render time
    let total: Duration = render_times.iter().sum();
    let avg_time = total / render_times.len() as u32;
    
    // Calculate timeout rate
    let timeout_count = self.timeout_count.load(Ordering::Relaxed);
    let timeout_rate = timeout_count as f64 / render_times.len() as f64;
    
    // Determine degradation level based on thresholds
    let degradation = if timeout_rate > self.config.performance.critical_timeout_rate {
        PerformanceDegradation::Critical
    } else if timeout_rate > self.config.performance.high_timeout_rate {
        PerformanceDegradation::High
    } else if avg_time > self.config.performance.max_avg_render_time {
        PerformanceDegradation::Medium
    } else {
        PerformanceDegradation::None
    };
    
    // Store degradation score (0.0 - 1.0)
    let score = match degradation {
        PerformanceDegradation::None => 0.0,
        PerformanceDegradation::Medium => 0.5,
        PerformanceDegradation::High => 0.75,
        PerformanceDegradation::Critical => 1.0,
    };
    self.degradation_score.store(score.to_bits(), Ordering::Relaxed);
    
    info!(
        avg_render_time_ms = avg_time.as_millis(),
        timeout_rate = timeout_rate,
        degradation = ?degradation,
        "Performance analysis completed"
    );
    
    degradation
}

// Add enum
pub enum PerformanceDegradation {
    None,
    Medium,
    High,
    Critical,
}
```

**Call periodically:**
```rust
// In ResourceManager::new or background task
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        let degradation = performance_monitor.analyze_performance().await;
        
        match degradation {
            PerformanceDegradation::High | PerformanceDegradation::Critical => {
                warn!("Performance degradation detected: {:?}", degradation);
                // Could trigger alerts, scaling, etc.
            }
            _ => {}
        }
    }
});
```

**Risk Level:** Low - Monitoring enhancement  
**Tests Needed:**
- Test degradation calculation
- Test threshold detection
- Test analysis interval enforcement

**Dependencies:** Requires `PerformanceConfig`  
**Configuration Addition:**
```rust
pub struct PerformanceConfig {
    pub analysis_interval_secs: u64,
    pub max_avg_render_time: Duration,
    pub high_timeout_rate: f64,
    pub critical_timeout_rate: f64,
    pub auto_cleanup_on_timeout: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            analysis_interval_secs: 60,
            max_avg_render_time: Duration::from_secs(5),
            high_timeout_rate: 0.1,      // 10%
            critical_timeout_rate: 0.25, // 25%
            auto_cleanup_on_timeout: true,
        }
    }
}
```

---

#### Finding E: `wasm_guard` and `acquired_at` in `RenderResourceGuard` (Lines 384, 386)

**Code:**
```rust
pub struct RenderResourceGuard {
    pub browser_checkout: BrowserCheckout,
    wasm_guard: WasmGuard,  // Never read
    memory_tracked: usize,
    acquired_at: Instant,   // Never read
    manager: ResourceManager,
}
```

**Category:** INCOMPLETE  
**Status:** Timeout tracking fields exist but no validation  
**Reason:** No timeout checking during long-running operations  

**Action:** Add timeout and monitoring methods
```rust
impl RenderResourceGuard {
    /// Check if resource has been held too long
    pub fn check_timeout(&self, max_duration: Duration) -> Result<()> {
        let elapsed = self.acquired_at.elapsed();
        if elapsed > max_duration {
            Err(anyhow!(
                "Resource held for {:?}, exceeds maximum {:?}",
                elapsed,
                max_duration
            ))
        } else {
            Ok(())
        }
    }
    
    /// Get time elapsed since acquisition
    pub fn elapsed(&self) -> Duration {
        self.acquired_at.elapsed()
    }
    
    /// Get WASM worker ID for debugging
    pub fn wasm_worker_id(&self) -> &str {
        &self.wasm_guard.worker_id
    }
    
    /// Get acquisition timestamp
    pub fn acquired_at(&self) -> Instant {
        self.acquired_at
    }
}

// Usage in render operations
pub async fn render_with_timeout(
    &self,
    guard: &RenderResourceGuard,
    url: &str,
    timeout: Duration,
) -> Result<String> {
    // Check timeout before starting
    guard.check_timeout(timeout)?;
    
    // Perform rendering...
    let result = /* render operation */;
    
    // Check timeout after
    guard.check_timeout(timeout)?;
    
    Ok(result)
}
```

**Risk Level:** Low - Monitoring enhancement  
**Tests Needed:**
- Test timeout detection
- Test elapsed time tracking
- Test WASM worker ID retrieval

**Dependencies:** None  

---

#### Finding F: `acquired_at` in `PdfResourceGuard` (Line 394)

**Code:**
```rust
pub struct PdfResourceGuard {
    _permit: tokio::sync::OwnedSemaphorePermit,
    memory_tracked: usize,
    acquired_at: Instant,  // Never read
    manager: ResourceManager,
}
```

**Category:** INCOMPLETE  
**Status:** Same as Finding E  
**Reason:** Same timeout tracking need  

**Action:** Same implementation pattern as Finding E
```rust
impl PdfResourceGuard {
    pub fn check_timeout(&self, max_duration: Duration) -> Result<()> {
        let elapsed = self.acquired_at.elapsed();
        if elapsed > max_duration {
            Err(anyhow!(
                "PDF resource held for {:?}, exceeds maximum {:?}",
                elapsed,
                max_duration
            ))
        } else {
            Ok(())
        }
    }
    
    pub fn elapsed(&self) -> Duration {
        self.acquired_at.elapsed()
    }
}
```

**Risk Level:** Low  
**Tests Needed:** Same as Finding E  
**Dependencies:** None  

---

#### Finding G: `worker_id` and `manager` in `WasmGuard` (Lines 402-403)

**Code:**
```rust
pub struct WasmGuard {
    worker_id: String,                    // Never read
    manager: Arc<WasmInstanceManager>,    // Never read
}
```

**Category:** INCOMPLETE  
**Status:** No Drop implementation, no accessors  
**Reason:** Instance tracking not updated on release  

**Action:** Implement Drop and accessors
```rust
impl WasmGuard {
    /// Get the worker ID for this guard
    pub fn worker_id(&self) -> &str {
        &self.worker_id
    }
}

impl Drop for WasmGuard {
    fn drop(&mut self) {
        let manager = self.manager.clone();
        let worker_id = self.worker_id.clone();
        
        // Update instance stats on drop
        tokio::spawn(async move {
            let mut instances = manager.worker_instances.write().await;
            if let Some(instance) = instances.get_mut(&worker_id) {
                // Update last operation time
                instance.last_operation = Instant::now();
                
                debug!(
                    "WASM guard released for worker {}, operations: {}",
                    worker_id,
                    instance.operations_count
                );
            }
        });
    }
}
```

**Risk Level:** Medium - Affects resource lifecycle  
**Tests Needed:**
- Test guard drop behavior
- Test instance stat updates on release
- Test concurrent guard acquisition/release

**Dependencies:** None  

---

### 5. rpc_client.rs

**Location:** `crates/riptide-api/src/rpc_client.rs`

#### Finding A: `Scroll` variant in `HeadlessPageAction` (Line 182-186)

**Code:**
```rust
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum HeadlessPageAction {
    WaitForCss { /* ... */ },
    WaitForJs { /* ... */ },
    Scroll {              // Never constructed
        steps: u32,
        step_px: u32,
        delay_ms: u64,
    },
    Js { /* ... */ },
    Click { /* ... */ },
    Type { /* ... */ },
}
```

**Category:** INCOMPLETE  
**Status:** Variant defined but not mapped from `PageAction`  
**Reason:** Scroll conversion not implemented in `convert_actions` (line 217)  

**Current State in `convert_actions`:**
```rust
fn convert_actions(actions: &[PageAction]) -> Vec<HeadlessPageAction> {
    actions
        .iter()
        .filter_map(|action| match action {
            PageAction::Click { /* ... */ } => Some(/* ... */),
            PageAction::Type { /* ... */ } => Some(/* ... */),
            PageAction::Evaluate { /* ... */ } => Some(/* ... */),
            PageAction::Wait(_) => Some(/* ... */),
            // Missing: PageAction::Scroll
            _ => {
                warn!("Skipping unsupported action: {:?}", action);
                None
            }
        })
        .collect()
}
```

**Check if PageAction::Scroll exists:**
Need to verify in `riptide_core::dynamic::PageAction` enum

**Action:** Add scroll action conversion
```rust
fn convert_actions(actions: &[PageAction]) -> Vec<HeadlessPageAction> {
    actions
        .iter()
        .filter_map(|action| match action {
            PageAction::Click { selector, .. } => Some(HeadlessPageAction::Click {
                css: selector.clone(),
            }),
            PageAction::Type { selector, text, wait_after, .. } => {
                Some(HeadlessPageAction::Type {
                    css: selector.clone(),
                    text: text.clone(),
                    delay_ms: wait_after.map(|d| d.as_millis() as u64),
                })
            },
            PageAction::Evaluate { script, .. } => Some(HeadlessPageAction::Js {
                code: script.clone(),
            }),
            // NEW: Add scroll action conversion
            PageAction::Scroll { steps, step_px, delay, .. } => {
                Some(HeadlessPageAction::Scroll {
                    steps: *steps,
                    step_px: *step_px,
                    delay_ms: delay.as_millis() as u64,
                })
            },
            PageAction::Wait(wait_condition) => {
                // ... existing wait conversion
            }
            _ => {
                warn!("Skipping unsupported action: {:?}", action);
                None
            }
        })
        .collect()
}
```

**Risk Level:** Medium - Affects scroll functionality  
**Tests Needed:**
- Test scroll action conversion (unit test)
- Test scroll execution in headless service (integration)
- Test scroll with various step/pixel configurations

**Dependencies:** Check riptide-core PageAction enum definition  

---

#### Finding B: `final_url` and `session_id` in `HeadlessRenderResponse` (Lines 203, 205)

**Code:**
```rust
#[derive(Debug, Deserialize)]
struct HeadlessRenderResponse {
    final_url: String,           // Never read
    html: String,
    session_id: Option<String>,  // Never read
    artifacts: HeadlessArtifactsOut,
}
```

**Category:** INCOMPLETE  
**Status:** Response fields received but not used  
**Reason:** Data not propagated to `DynamicRenderResult`  

**Current Usage (Line 94-110):**
```rust
let headless_response: HeadlessRenderResponse = response.json().await?;

// Creates result but ignores final_url and session_id
let result = DynamicRenderResult {
    success: true,
    html: headless_response.html,
    artifacts: convert_artifacts(headless_response.artifacts),
    error: None,
    render_time_ms,
    actions_executed: extract_action_names(&config.actions),
    wait_conditions_met: vec!["dom_content_loaded".to_string()],
};
```

**Check DynamicRenderResult definition:**
Need to see if it has fields for final_url and session_id

**Action:** Propagate to result
```rust
// Update DynamicRenderResult creation
let result = DynamicRenderResult {
    success: true,
    html: headless_response.html,
    final_url: Some(headless_response.final_url.clone()),  // NEW
    session_id: headless_response.session_id.clone(),      // NEW
    artifacts: convert_artifacts(
        headless_response.artifacts,
        &headless_response.final_url  // Pass to metadata
    ),
    error: None,
    render_time_ms,
    actions_executed: extract_action_names(&config.actions),
    wait_conditions_met: vec!["dom_content_loaded".to_string()],
};

// Update convert_artifacts signature
fn convert_artifacts(
    artifacts: HeadlessArtifactsOut,
    final_url: &str  // NEW parameter
) -> Option<RenderArtifacts> {
    if artifacts.screenshot_b64.is_none() && artifacts.mhtml_b64.is_none() {
        return None;
    }
    
    Some(RenderArtifacts {
        screenshot: artifacts.screenshot_b64,
        mhtml: artifacts.mhtml_b64,
        metadata: riptide_core::dynamic::PageMetadata {
            title: None,
            description: None,
            og_tags: std::collections::HashMap::new(),
            twitter_tags: std::collections::HashMap::new(),
            json_ld: Vec::new(),
            final_url: final_url.to_string(),  // Use actual final URL
            headers: std::collections::HashMap::new(),
            timing: None,
        },
        console_logs: Vec::new(),
        network_activity: Vec::new(),
    })
}
```

**Risk Level:** Low - Data enrichment, no behavior change  
**Tests Needed:**
- Test final URL capture in result
- Test session ID persistence
- Test redirect tracking via final_url

**Dependencies:** Check `DynamicRenderResult` structure in riptide-core  

---

### 6. strategies_pipeline.rs

**Location:** `crates/riptide-api/src/strategies_pipeline.rs:496-537`

#### Finding: `create_github_selectors` and `create_blog_selectors` functions

**Code:**
```rust
/// Create GitHub-specific CSS selectors
fn create_github_selectors() -> std::collections::HashMap<String, String> {
    let mut selectors = std::collections::HashMap::new();
    selectors.insert(
        "title".to_string(),
        "h1.entry-title, .js-issue-title, .repository-content h1".to_string(),
    );
    selectors.insert(
        "content".to_string(),
        ".entry-content, .markdown-body, .comment-body".to_string(),
    );
    selectors.insert(
        "author".to_string(),
        ".author, .commit-author, .discussion-item-header a".to_string(),
    );
    selectors.insert(
        "date".to_string(),
        "time, .commit-date, relative-time".to_string(),
    );
    selectors
}

/// Create blog-specific CSS selectors
fn create_blog_selectors() -> std::collections::HashMap<String, String> {
    let mut selectors = std::collections::HashMap::new();
    selectors.insert(
        "title".to_string(),
        "h1, .entry-title, .post-title, [data-testid='storyTitle']".to_string(),
    );
    selectors.insert(
        "content".to_string(),
        ".entry-content, .post-content, .story-content, article".to_string(),
    );
    selectors.insert(
        "author".to_string(),
        ".author, .byline, .writer, [data-testid='authorName']".to_string(),
    );
    selectors.insert(
        "date".to_string(),
        "time, .date, .published, [data-testid='storyPublishDate']".to_string(),
    );
    selectors
}

// News pattern function removed since regex strategies are no longer used
```

**Category:** REMOVE  
**Status:** Never called  
**Reason:** Regex-based strategy system removed (confirmed by comment on line 539)  
**Context:** File comment indicates strategies were refactored away from regex patterns  

**Action:** Remove lines 495-537 (both functions)  

**Risk Level:** Low - Confirmed dead code after refactoring  
**Tests Needed:** 
- Ensure no selector-based tests exist that reference these
- Verify strategy tests use new approach

**Dependencies:** None  
**Verification:** Search codebase for references to function names  

---

### 7. streaming/lifecycle.rs

**Location:** `crates/riptide-api/src/streaming/lifecycle.rs:89`

#### Finding: `metrics` field in `StreamLifecycleManager`

**Code:**
```rust
pub struct StreamLifecycleManager {
    /// Event channel sender
    event_tx: mpsc::UnboundedSender<LifecycleEvent>,
    /// Metrics collector
    metrics: Arc<RipTideMetrics>,  // Never read
    /// Active connections tracking
    active_connections: Arc<tokio::sync::RwLock<std::collections::HashMap<String, ConnectionInfo>>>,
}
```

**Category:** INCOMPLETE  
**Status:** Metrics collector stored but not used  
**Reason:** Streaming metrics not tracked  

**Action:** Implement streaming metrics
```rust
impl StreamLifecycleManager {
    /// Record new connection
    pub async fn on_connection_opened(&self, info: ConnectionInfo) {
        // Track connection
        let mut connections = self.active_connections.write().await;
        connections.insert(info.connection_id.clone(), info);
        
        // Update metrics
        self.metrics.streaming_active_connections
            .set(connections.len() as i64);
        self.metrics.streaming_connections_total.inc();
        
        debug!("Connection opened: {}", info.connection_id);
    }
    
    /// Record connection closure
    pub async fn on_connection_closed(&self, connection_id: &str) {
        let mut connections = self.active_connections.write().await;
        if let Some(info) = connections.remove(connection_id) {
            let duration = info.start_time.elapsed();
            
            // Update metrics
            self.metrics.streaming_active_connections
                .set(connections.len() as i64);
            self.metrics.streaming_connection_duration
                .observe(duration.as_secs_f64());
            
            debug!(
                "Connection closed: {}, duration: {:?}, bytes: {}, messages: {}",
                connection_id,
                duration,
                info.bytes_sent,
                info.messages_sent
            );
        }
    }
    
    /// Record message sent
    pub async fn on_message_sent(&self, connection_id: &str, bytes: usize) {
        let mut connections = self.active_connections.write().await;
        if let Some(info) = connections.get_mut(connection_id) {
            info.bytes_sent += bytes;
            info.messages_sent += 1;
        }
        
        // Update metrics
        self.metrics.streaming_bytes_sent.inc_by(bytes as u64);
        self.metrics.streaming_messages_sent.inc();
    }
    
    /// Get streaming statistics
    pub async fn get_stats(&self) -> StreamingStats {
        let connections = self.active_connections.read().await;
        
        StreamingStats {
            active_connections: connections.len(),
            total_bytes_sent: connections.values()
                .map(|c| c.bytes_sent)
                .sum(),
            total_messages_sent: connections.values()
                .map(|c| c.messages_sent)
                .sum(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct StreamingStats {
    pub active_connections: usize,
    pub total_bytes_sent: usize,
    pub total_messages_sent: usize,
}
```

**Risk Level:** Low - Monitoring enhancement  
**Tests Needed:**
- Test connection tracking
- Test metrics updates on events
- Test stats aggregation

**Dependencies:** Requires metrics in RipTideMetrics  
**Required Metrics:**
```rust
// Add to RipTideMetrics
pub struct RipTideMetrics {
    // ... existing
    pub streaming_active_connections: Gauge,
    pub streaming_connections_total: Counter,
    pub streaming_connection_duration: Histogram,
    pub streaming_bytes_sent: Counter,
    pub streaming_messages_sent: Counter,
}
```

---

### 8. validation.rs

**Location:** `crates/riptide-api/src/validation.rs`

#### Finding A: `ALLOWED_SCHEMES` constant (Line 15)

**Code:**
```rust
/// Supported URL schemes for crawling
const ALLOWED_SCHEMES: &[&str] = &["http", "https"];  // Never used
```

**Category:** REFACTOR  
**Status:** Never used  
**Reason:** Scheme validation moved to `CommonValidator` in riptide-core  
**Context:** Line 124 uses `CommonValidator::validate_url` which handles scheme validation  

**Validation Flow:**
```rust
// Line 123-140
fn validate_url(url_str: &str, index: usize) -> ApiResult<()> {
    let validator = CommonValidator::new_default();
    
    // CommonValidator checks schemes internally
    match validator.validate_url(url_str) {
        Ok(_) => {
            validate_url_patterns(url_str, index)?;
            Ok(())
        }
        Err(e) => Err(ApiError::invalid_url(/* ... */))
    }
}
```

**Action:** Remove lines 14-15  

**Risk Level:** Low - Functionality exists in CommonValidator  
**Tests Needed:** Verify CommonValidator tests cover scheme validation  
**Dependencies:** None  

---

#### Finding B: `is_private_or_localhost` function (Line 144-147)

**Code:**
```rust
// Use common validator for private/localhost checking
fn is_private_or_localhost(host: &str) -> bool {
    let validator = CommonValidator::new_default();
    validator.is_private_or_local_address(host)
}
```

**Category:** REFACTOR  
**Status:** Never called  
**Reason:** Thin wrapper around CommonValidator method, unused  
**Context:** Comment says to use CommonValidator, but function itself is never used  

**Search Results:** Function not called anywhere in codebase  

**Action:** Remove lines 143-147  

**Risk Level:** Low - Direct CommonValidator calls already used elsewhere  
**Tests Needed:** None - wrapper has no additional logic  
**Dependencies:** None  

---

## Implementation Plan

### Phase 1: Safe Removals (1-2 hours)
**Risk:** Low  
**Files:** 4 files  
**Lines Removed:** ~70  

1. ✅ Remove `verify_http_client_config` from `health.rs`
2. ✅ Remove `create_github_selectors` and `create_blog_selectors` from `strategies_pipeline.rs`
3. ✅ Remove `ALLOWED_SCHEMES` and `is_private_or_localhost` from `validation.rs`
4. ✅ Remove duplicate `fetch_content` from `pipeline.rs`

**Verification:**
```bash
cargo check
cargo test --lib
```

---

### Phase 2: High-Value Completions (1-2 days)
**Risk:** Medium  
**Files:** 3 files  
**Lines Added:** ~300  

1. 🔧 Integrate `extract_with_headless` and `render_and_extract` in `pipeline.rs`
   - Modify `extract_content` method
   - Add headless direct path for Decision::Headless
   - Add fallback chain
   - Write integration tests

2. 🔧 Implement scroll action in `rpc_client.rs`
   - Update `convert_actions` function
   - Add scroll variant conversion
   - Write unit and integration tests

3. 🔧 Use state/options/metrics in `pipeline_dual.rs`
   - Replace `FetchEngine` with `state.http_client`
   - Use `state.extractor` for CSS extraction
   - Add metrics recording throughout
   - Update tests

4. 🔧 Implement rate limiter cleanup in `resource_manager.rs`
   - Complete `start_cleanup_task` implementation
   - Add background cleanup loop
   - Write cleanup tests

**Testing:**
```bash
cargo test --lib resource_manager
cargo test --lib pipeline
cargo test --lib rpc_client
cargo test --lib pipeline_dual
```

---

### Phase 3: Monitoring Enhancements (1 day)
**Risk:** Low  
**Files:** 2 files  
**Lines Added:** ~200  

1. 📊 WASM instance health checking in `resource_manager.rs`
   - Implement `WasmWorkerInstance::is_healthy`
   - Add health checks to `acquire_instance`
   - Add tests

2. 📊 Performance degradation detection in `resource_manager.rs`
   - Implement `PerformanceMonitor::analyze_performance`
   - Add background analysis task
   - Add tests

3. 📊 Resource guard timeout tracking in `resource_manager.rs`
   - Add `check_timeout` to RenderResourceGuard
   - Add `check_timeout` to PdfResourceGuard
   - Add accessor methods
   - Add tests

4. 📊 Streaming metrics in `lifecycle.rs`
   - Implement connection tracking
   - Add metric updates
   - Add stats API
   - Add tests

5. 📊 RPC response data propagation in `rpc_client.rs`
   - Add final_url to result
   - Add session_id to result
   - Update convert_artifacts
   - Add tests

**Testing:**
```bash
cargo test --lib resource_manager::tests
cargo test --lib streaming::lifecycle
cargo test --lib rpc_client::tests
```

---

### Phase 4: Resource Management Refinements (1 day)
**Risk:** Medium  
**Files:** 1 file  
**Lines Added:** ~150  

1. 🔧 Use config for WASM instance limits in `resource_manager.rs`
   - Add WasmConfig to ApiConfig
   - Implement instance limit checking
   - Implement age/idle timeout recycling
   - Add tests

2. 🔧 Implement WasmGuard cleanup in `resource_manager.rs`
   - Add Drop implementation
   - Add worker_id accessor
   - Update instance stats on drop
   - Add tests

**Configuration:**
```rust
// Add to config.rs
pub struct WasmConfig {
    pub max_instances: usize,
    pub max_instance_age: Duration,
    pub idle_timeout: Duration,
    pub max_memory_mb: usize,
}

pub struct PerformanceConfig {
    pub analysis_interval_secs: u64,
    pub max_avg_render_time: Duration,
    pub high_timeout_rate: f64,
    pub critical_timeout_rate: f64,
    pub auto_cleanup_on_timeout: bool,
}
```

**Testing:**
```bash
cargo test --lib resource_manager
cargo test --lib config
```

---

## Testing Strategy

### Unit Tests Required

**resource_manager.rs:**
- Rate limiter bucket cleanup timing
- WASM instance health validation
- Performance degradation calculation
- Timeout detection in guards
- Instance limit enforcement
- WasmGuard drop behavior

**pipeline.rs:**
- Headless extraction path selection
- Fallback chain execution
- Error handling in extraction flow

**pipeline_dual.rs:**
- HTTP client reuse
- Extractor integration
- Metrics recording

**rpc_client.rs:**
- Scroll action conversion
- Final URL capture
- Session ID propagation

**lifecycle.rs:**
- Connection tracking
- Metrics updates
- Stats aggregation

### Integration Tests Required

**Pipeline Integration:**
- Headless service integration
- Fallback behavior without headless
- Complete extraction workflow

**Dual-Path Integration:**
- Fast path execution
- AI enhancement queuing
- Result merging

**Resource Management Integration:**
- Resource guard lifecycle
- Cleanup task execution
- Limit enforcement under load

**Streaming Integration:**
- WebSocket connection lifecycle
- SSE connection lifecycle
- Metrics collection

### Documentation Tests

- Add examples for monitoring APIs
- Document dual-path configuration
- Document resource guard usage
- Document headless integration

---

## Risk Assessment

### Low Risk Items (11 items)
- Safe to implement immediately
- No behavior changes
- Monitoring/observability only
- **Examples:** Metrics tracking, timeout detection, cleanup tasks

### Medium Risk Items (10 items)
- Require careful testing
- Change execution paths
- Affect resource management
- **Examples:** Headless integration, dual-path refactoring, limit enforcement

### High Risk Items (0 items)
- None identified

---

## Dependencies & Prerequisites

### Configuration Additions
1. `WasmConfig` - For instance management
2. `PerformanceConfig` - For degradation detection

### Metrics Additions
```rust
// Add to RipTideMetrics
pub dual_path_requests: Counter,
pub fast_path_duration: Histogram,
pub dual_path_duration: Histogram,
pub dual_path_quality_score: Histogram,
pub ai_enhancements_queued: Counter,
pub streaming_active_connections: Gauge,
pub streaming_connections_total: Counter,
pub streaming_connection_duration: Histogram,
pub streaming_bytes_sent: Counter,
pub streaming_messages_sent: Counter,
```

### External Dependencies
- Verify `PageAction::Scroll` exists in riptide-core
- Verify `DynamicRenderResult` has final_url/session_id fields
- Check headless service API compatibility

---

## Next Steps

1. ✅ Review and approve this analysis
2. 📋 Create GitHub issues for each phase
3. 🗑️ Implement Phase 1 (safe removals)
4. ✅ Write tests for Phase 2
5. 🔧 Implement Phase 2 (high-value)
6. 📊 Implement Phase 3 (monitoring)
7. 🔧 Implement Phase 4 (refinements)
8. ✅ Final verification

---

**Report Complete**  
**Total Issues Documented:** 21  
**Estimated Total Effort:** 4-6 days  
**Recommended Start:** Phase 1 (safe removals)
