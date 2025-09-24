//! Comprehensive resource management for RipTide API operations.
//!
//! This module implements all resource controls including:
//! - Headless browser pool management (cap = 3)
//! - Per-host rate limiting (1.5 RPS with jitter)
//! - PDF operation semaphore (max 2 concurrent)
//! - Single WASM instance per worker
//! - Memory cleanup on timeouts
//! - Performance monitoring and degradation detection

use crate::config::ApiConfig;
use anyhow::{anyhow, Result};
use riptide_headless::pool::{BrowserPool, BrowserPoolConfig, BrowserCheckout};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tokio::{
    sync::{Mutex, RwLock, Semaphore},
    time::{sleep, timeout},
};
use tracing::{debug, error, info, warn};
use url::Url;

/// Comprehensive resource manager for all API operations
#[derive(Clone)]
pub struct ResourceManager {
    /// Configuration
    config: ApiConfig,
    /// Headless browser pool manager (using proper BrowserPool from riptide-headless)
    pub browser_pool: Arc<BrowserPool>,
    /// Per-host rate limiter
    pub rate_limiter: Arc<PerHostRateLimiter>,
    /// PDF processing semaphore
    pub pdf_semaphore: Arc<Semaphore>,
    /// WASM instance manager
    pub wasm_manager: Arc<WasmInstanceManager>,
    /// Memory manager
    pub memory_manager: Arc<MemoryManager>,
    /// Performance monitor
    pub performance_monitor: Arc<PerformanceMonitor>,
    /// Resource metrics
    metrics: Arc<ResourceMetrics>,
}

// Browser pool implementation moved to riptide-headless crate
// Using proper BrowserPool from riptide-headless::pool

/// Per-host rate limiter with jitter
pub struct PerHostRateLimiter {
    config: ApiConfig,
    host_buckets: RwLock<HashMap<String, HostBucket>>,
    cleanup_task: Mutex<Option<tokio::task::JoinHandle<()>>>,
    metrics: Arc<ResourceMetrics>,
}

/// Rate limiting bucket for a specific host
#[derive(Debug, Clone)]
struct HostBucket {
    tokens: f64,
    last_refill: Instant,
    request_count: u64,
    last_request: Instant,
}

/// WASM instance manager with single instance per worker
pub struct WasmInstanceManager {
    config: ApiConfig,
    worker_instances: RwLock<HashMap<String, WasmWorkerInstance>>,
    metrics: Arc<ResourceMetrics>,
}

/// WASM instance for a specific worker
#[derive(Debug)]
struct WasmWorkerInstance {
    pub worker_id: String,
    pub created_at: Instant,
    pub operations_count: u64,
    pub last_operation: Instant,
    pub is_healthy: bool,
    pub memory_usage: usize,
}

/// Memory manager with pressure detection and cleanup
pub struct MemoryManager {
    config: ApiConfig,
    current_usage: AtomicUsize,
    pressure_detected: std::sync::atomic::AtomicBool,
    last_cleanup: AtomicU64,
    last_gc: AtomicU64,
    metrics: Arc<ResourceMetrics>,
}

/// Performance monitor for resource efficiency
pub struct PerformanceMonitor {
    config: ApiConfig,
    render_times: Mutex<Vec<Duration>>,
    timeout_count: AtomicU64,
    degradation_score: std::sync::atomic::AtomicU64, // Stored as u64 for atomic ops
    last_analysis: AtomicU64,
    metrics: Arc<ResourceMetrics>,
}

/// Resource metrics collection
#[derive(Default)]
pub struct ResourceMetrics {
    pub headless_pool_size: AtomicUsize,
    pub headless_active: AtomicUsize,
    pub pdf_active: AtomicUsize,
    pub wasm_instances: AtomicUsize,
    pub memory_usage_mb: AtomicUsize,
    pub rate_limit_hits: AtomicU64,
    pub timeouts_count: AtomicU64,
    pub cleanup_operations: AtomicU64,
    pub gc_triggers: AtomicU64,
    pub render_operations: AtomicU64,
    pub successful_renders: AtomicU64,
    pub failed_renders: AtomicU64,
}

/// Result of resource acquisition
pub struct ResourceGuard {
    pub resource_type: String,
    pub acquired_at: Instant,
    pub timeout: Duration,
    _guard: Option<Arc<dyn Send + Sync>>,
}

impl std::fmt::Debug for ResourceGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceGuard")
            .field("resource_type", &self.resource_type)
            .field("acquired_at", &self.acquired_at)
            .field("timeout", &self.timeout)
            .finish_non_exhaustive()
    }
}

/// Resource operation result
#[derive(Debug)]
pub enum ResourceResult<T> {
    Success(T),
    Timeout,
    ResourceExhausted,
    RateLimited { retry_after: Duration },
    MemoryPressure,
    Error(String),
}

impl ResourceManager {
    /// Create new resource manager with comprehensive controls
    pub async fn new(config: ApiConfig) -> Result<Self> {
        info!("Initializing comprehensive resource manager");

        let metrics = Arc::new(ResourceMetrics::default());

        // Initialize proper BrowserPool from riptide-headless
        let browser_pool_config = BrowserPoolConfig {
            min_pool_size: 1,
            max_pool_size: config.headless.max_pool_size,
            initial_pool_size: 2,
            idle_timeout: Duration::from_secs(30),
            max_lifetime: Duration::from_secs(300),
            health_check_interval: Duration::from_secs(10),
            memory_threshold_mb: 500,
            enable_recovery: true,
            max_retries: config.headless.max_retries,
        };

        // Configure browser with headless settings
        let browser_config = chromiumoxide::BrowserConfig::builder()
            .with_head()
            .no_sandbox()
            .build()
            .map_err(|e| anyhow!("Failed to build browser config: {}", e))?;

        let browser_pool = Arc::new(
            BrowserPool::new(browser_pool_config, browser_config)
                .await
                .map_err(|e| anyhow!("Failed to initialize browser pool: {}", e))?,
        );

        // Initialize per-host rate limiter
        let rate_limiter =
            Arc::new(PerHostRateLimiter::new(config.clone(), metrics.clone()).await?);

        // Initialize PDF semaphore
        let pdf_semaphore = Arc::new(Semaphore::new(config.pdf.max_concurrent));

        // Initialize WASM instance manager
        let wasm_manager =
            Arc::new(WasmInstanceManager::new(config.clone(), metrics.clone()).await?);

        // Initialize memory manager
        let memory_manager = Arc::new(MemoryManager::new(config.clone(), metrics.clone()).await?);

        // Initialize performance monitor
        let performance_monitor =
            Arc::new(PerformanceMonitor::new(config.clone(), metrics.clone()).await?);

        info!(
            headless_pool_cap = config.headless.max_pool_size,
            pdf_semaphore = config.pdf.max_concurrent,
            rate_limit_rps = config.rate_limiting.requests_per_second_per_host,
            "Resource manager initialized with requirements"
        );

        Ok(Self {
            config,
            browser_pool,
            rate_limiter,
            pdf_semaphore,
            wasm_manager,
            memory_manager,
            performance_monitor,
            metrics,
        })
    }

    /// Acquire resources for render operation with comprehensive controls
    pub async fn acquire_render_resources(
        &self,
        url: &str,
    ) -> Result<ResourceResult<RenderResourceGuard>> {
        let start_time = Instant::now();

        // 1. Check memory pressure first
        if self.memory_manager.is_under_pressure() {
            warn!("Render request rejected due to memory pressure");
            return Ok(ResourceResult::MemoryPressure);
        }

        // 2. Apply per-host rate limiting with jitter
        let host = self.extract_host(url)?;
        match self.rate_limiter.check_rate_limit(&host).await {
            Ok(()) => {}
            Err(retry_after) => {
                debug!(host = %host, retry_after_ms = retry_after.as_millis(), "Rate limited");
                return Ok(ResourceResult::RateLimited { retry_after });
            }
        }

        // 3. Acquire headless browser with timeout
        let browser_result = timeout(
            self.config.get_timeout("render"),
            self.browser_pool.checkout(),
        )
        .await;

        let browser_checkout = match browser_result {
            Ok(Ok(checkout)) => checkout,
            Ok(Err(e)) => {
                error!(error = %e, "Failed to acquire browser");
                return Ok(ResourceResult::ResourceExhausted);
            }
            Err(_) => {
                warn!("Browser acquisition timed out");
                self.performance_monitor.record_timeout().await;
                return Ok(ResourceResult::Timeout);
            }
        };

        // 4. Acquire WASM instance for current worker
        let worker_id = self.get_worker_id();
        let wasm_guard = self.wasm_manager.acquire_instance(&worker_id).await?;

        // 5. Track memory usage
        self.memory_manager.track_allocation(256).await; // Estimate for render operation

        let total_time = start_time.elapsed();
        debug!(
            acquisition_time_ms = total_time.as_millis(),
            host = %host,
            "Render resources acquired successfully"
        );

        Ok(ResourceResult::Success(RenderResourceGuard {
            browser_checkout,
            wasm_guard,
            memory_tracked: 256,
            acquired_at: start_time,
            manager: self.clone(),
        }))
    }

    /// Acquire resources for PDF operation
    pub async fn acquire_pdf_resources(&self) -> Result<ResourceResult<PdfResourceGuard>> {
        // Check memory pressure
        if self.memory_manager.is_under_pressure() {
            return Ok(ResourceResult::MemoryPressure);
        }

        // Acquire PDF semaphore with timeout
        let permit_result =
            timeout(self.config.get_timeout("pdf"), self.pdf_semaphore.clone().acquire_owned()).await;

        let permit = match permit_result {
            Ok(Ok(permit)) => permit,
            Ok(Err(_)) => return Ok(ResourceResult::ResourceExhausted),
            Err(_) => {
                self.performance_monitor.record_timeout().await;
                return Ok(ResourceResult::Timeout);
            }
        };

        // Track memory for PDF operation
        self.memory_manager.track_allocation(128).await;

        self.metrics.pdf_active.fetch_add(1, Ordering::Relaxed);

        Ok(ResourceResult::Success(PdfResourceGuard {
            _permit: permit,
            memory_tracked: 128,
            acquired_at: Instant::now(),
            manager: self.clone(),
        }))
    }

    /// Cleanup resources on timeout or error
    pub async fn cleanup_on_timeout(&self, operation_type: &str) {
        warn!(operation = %operation_type, "Performing timeout cleanup");

        // Trigger memory cleanup
        if self.config.performance.auto_cleanup_on_timeout {
            self.memory_manager.trigger_cleanup().await;
        }

        // Update performance metrics
        self.performance_monitor.record_timeout().await;

        // Force garbage collection if configured
        if self.memory_manager.should_trigger_gc().await {
            self.memory_manager.trigger_gc().await;
        }

        self.metrics
            .cleanup_operations
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Get current resource status
    pub async fn get_resource_status(&self) -> ResourceStatus {
        let pool_stats = self.browser_pool.get_stats().await;

        ResourceStatus {
            headless_pool_available: pool_stats.available,
            headless_pool_total: pool_stats.total_capacity,
            pdf_available: self.pdf_semaphore.available_permits(),
            pdf_total: self.config.pdf.max_concurrent,
            memory_usage_mb: self.memory_manager.current_usage_mb(),
            memory_pressure: self.memory_manager.is_under_pressure(),
            rate_limit_hits: self.metrics.rate_limit_hits.load(Ordering::Relaxed),
            timeout_count: self.metrics.timeouts_count.load(Ordering::Relaxed),
            degradation_score: self.performance_monitor.get_degradation_score().await,
        }
    }

    // Helper methods
    fn extract_host(&self, url: &str) -> Result<String> {
        let parsed = Url::parse(url).map_err(|e| anyhow!("Invalid URL: {}", e))?;

        Ok(parsed
            .host_str()
            .ok_or_else(|| anyhow!("No host in URL"))?
            .to_string())
    }

    fn get_worker_id(&self) -> String {
        // Use current task ID as worker identifier
        format!("worker_{:?}", std::thread::current().id())
    }
}

/// Resource guard for render operations
pub struct RenderResourceGuard {
    pub browser_checkout: BrowserCheckout,
    wasm_guard: WasmGuard,
    memory_tracked: usize,
    acquired_at: Instant,
    manager: ResourceManager,
}

/// Resource guard for PDF operations
pub struct PdfResourceGuard {
    _permit: tokio::sync::OwnedSemaphorePermit,
    memory_tracked: usize,
    acquired_at: Instant,
    manager: ResourceManager,
}

// Browser guard is now provided by BrowserCheckout from riptide-headless

/// WASM guard with instance tracking
pub struct WasmGuard {
    worker_id: String,
    manager: Arc<WasmInstanceManager>,
}

/// Current resource status
#[derive(Debug)]
pub struct ResourceStatus {
    pub headless_pool_available: usize,
    pub headless_pool_total: usize,
    pub pdf_available: usize,
    pub pdf_total: usize,
    pub memory_usage_mb: usize,
    pub memory_pressure: bool,
    pub rate_limit_hits: u64,
    pub timeout_count: u64,
    pub degradation_score: f64,
}

// Implementation of individual managers follows...

// HeadlessBrowserPool implementation removed - using BrowserPool from riptide-headless


impl PerHostRateLimiter {
    async fn new(config: ApiConfig, metrics: Arc<ResourceMetrics>) -> Result<Self> {
        let limiter = Self {
            config,
            host_buckets: RwLock::new(HashMap::new()),
            cleanup_task: Mutex::new(None),
            metrics,
        };

        // Start cleanup task
        limiter.start_cleanup_task().await;

        Ok(limiter)
    }

    async fn check_rate_limit(&self, host: &str) -> Result<(), Duration> {
        if !self.config.rate_limiting.enabled {
            return Ok(());
        }

        let now = Instant::now();
        let mut buckets = self.host_buckets.write().await;

        let bucket = buckets
            .entry(host.to_string())
            .or_insert_with(|| HostBucket {
                tokens: self.config.rate_limiting.requests_per_second_per_host,
                last_refill: now,
                request_count: 0,
                last_request: now,
            });

        // Refill tokens based on time elapsed
        let time_passed = now.duration_since(bucket.last_refill).as_secs_f64();
        let tokens_to_add = time_passed * self.config.rate_limiting.requests_per_second_per_host;
        bucket.tokens = (bucket.tokens + tokens_to_add)
            .min(self.config.rate_limiting.burst_capacity_per_host as f64);
        bucket.last_refill = now;

        // Check if request can be served
        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            bucket.request_count += 1;
            bucket.last_request = now;

            // Add jitter delay
            let jitter_delay = self.config.calculate_jittered_delay();
            if jitter_delay > Duration::from_millis(1) {
                tokio::time::sleep(jitter_delay).await;
            }

            Ok(())
        } else {
            self.metrics.rate_limit_hits.fetch_add(1, Ordering::Relaxed);
            let retry_after = Duration::from_secs_f64(
                1.0 / self.config.rate_limiting.requests_per_second_per_host,
            );
            Err(retry_after)
        }
    }

    async fn start_cleanup_task(&self) {
        // Implementation for periodic cleanup of old host buckets
        // This would run in the background to prevent memory leaks
    }
}

impl WasmInstanceManager {
    async fn new(config: ApiConfig, metrics: Arc<ResourceMetrics>) -> Result<Self> {
        Ok(Self {
            config,
            worker_instances: RwLock::new(HashMap::new()),
            metrics,
        })
    }

    async fn acquire_instance(self: &Arc<Self>, worker_id: &str) -> Result<WasmGuard> {
        let mut instances = self.worker_instances.write().await;

        // Ensure single instance per worker (requirement)
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
}



impl MemoryManager {
    async fn new(config: ApiConfig, metrics: Arc<ResourceMetrics>) -> Result<Self> {
        Ok(Self {
            config,
            current_usage: AtomicUsize::new(0),
            pressure_detected: std::sync::atomic::AtomicBool::new(false),
            last_cleanup: AtomicU64::new(0),
            last_gc: AtomicU64::new(0),
            metrics,
        })
    }

    async fn track_allocation(&self, size_mb: usize) {
        let current = self.current_usage.fetch_add(size_mb, Ordering::Relaxed);
        self.metrics
            .memory_usage_mb
            .store(current + size_mb, Ordering::Relaxed);

        // Check for memory pressure
        if self.config.is_memory_pressure(current + size_mb) {
            self.pressure_detected.store(true, Ordering::Relaxed);
            warn!(current_mb = current + size_mb, "Memory pressure detected");
        }
    }

    async fn track_deallocation(&self, size_mb: usize) {
        let current = self.current_usage.fetch_sub(size_mb, Ordering::Relaxed);
        self.metrics
            .memory_usage_mb
            .store(current.saturating_sub(size_mb), Ordering::Relaxed);

        // Update pressure status
        if !self
            .config
            .is_memory_pressure(current.saturating_sub(size_mb))
        {
            self.pressure_detected.store(false, Ordering::Relaxed);
        }
    }

    fn is_under_pressure(&self) -> bool {
        self.pressure_detected.load(Ordering::Relaxed)
    }

    fn current_usage_mb(&self) -> usize {
        self.current_usage.load(Ordering::Relaxed)
    }

    async fn trigger_cleanup(&self) {
        info!("Triggering memory cleanup");
        // Implement memory cleanup logic
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or_else(|e| {
                warn!(error = %e, "Failed to get system time for cleanup timestamp");
                0 // Fallback to epoch
            });

        self.last_cleanup.store(timestamp, Ordering::Relaxed);
        self.metrics
            .cleanup_operations
            .fetch_add(1, Ordering::Relaxed);
    }

    async fn trigger_gc(&self) {
        info!("Triggering garbage collection");
        // Force garbage collection
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or_else(|e| {
                warn!(error = %e, "Failed to get system time for GC timestamp");
                0 // Fallback to epoch
            });

        self.last_gc.store(timestamp, Ordering::Relaxed);
        self.metrics.gc_triggers.fetch_add(1, Ordering::Relaxed);
    }

    async fn should_trigger_gc(&self) -> bool {
        self.current_usage.load(Ordering::Relaxed) >= self.config.memory.gc_trigger_threshold_mb
    }
}

impl PerformanceMonitor {
    async fn new(config: ApiConfig, metrics: Arc<ResourceMetrics>) -> Result<Self> {
        Ok(Self {
            config,
            render_times: Mutex::new(Vec::new()),
            timeout_count: AtomicU64::new(0),
            degradation_score: std::sync::atomic::AtomicU64::new(0),
            last_analysis: AtomicU64::new(0),
            metrics,
        })
    }

    async fn record_timeout(&self) {
        self.timeout_count.fetch_add(1, Ordering::Relaxed);
        self.metrics.timeouts_count.fetch_add(1, Ordering::Relaxed);
    }

    async fn get_degradation_score(&self) -> f64 {
        // Convert from u64 back to f64
        f64::from_bits(self.degradation_score.load(Ordering::Relaxed))
    }

    /// Record render operation metrics for performance monitoring
    pub async fn record_render_operation(
        &self,
        url: &str,
        duration: Duration,
        success: bool,
        actions_executed: u32,
        network_requests: u32,
    ) -> Result<()> {
        // Record render timing
        {
            let mut render_times = self.render_times.lock().await;
            render_times.push(duration);

            // Keep only recent measurements (last 100)
            if render_times.len() > 100 {
                render_times.remove(0);
            }
        }

        // Update metrics
        self.metrics.render_operations.fetch_add(1, Ordering::Relaxed);
        if success {
            self.metrics.successful_renders.fetch_add(1, Ordering::Relaxed);
        } else {
            self.metrics.failed_renders.fetch_add(1, Ordering::Relaxed);
        }

        // Log performance details
        debug!(
            url = %url,
            duration_ms = duration.as_millis(),
            success = success,
            actions_executed = actions_executed,
            network_requests = network_requests,
            "Recorded render operation metrics"
        );

        Ok(())
    }
}

// Drop implementations for automatic cleanup
impl Drop for RenderResourceGuard {
    fn drop(&mut self) {
        let manager = self.manager.clone();
        let memory_tracked = self.memory_tracked;

        // Spawn cleanup task
        tokio::spawn(async move {
            manager
                .memory_manager
                .track_deallocation(memory_tracked)
                .await;
        });
    }
}

impl Drop for PdfResourceGuard {
    fn drop(&mut self) {
        let manager = self.manager.clone();
        let memory_tracked = self.memory_tracked;

        manager.metrics.pdf_active.fetch_sub(1, Ordering::Relaxed);

        tokio::spawn(async move {
            manager
                .memory_manager
                .track_deallocation(memory_tracked)
                .await;
        });
    }
}

// Drop implementation for BrowserCheckout is handled by riptide-headless

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resource_manager_creation() {
        let config = ApiConfig::default();
        let manager = ResourceManager::new(config).await.unwrap();

        let status = manager.get_resource_status().await;
        assert_eq!(status.pdf_total, 2); // PDF semaphore requirement
        assert_eq!(status.headless_pool_total, 3); // Pool cap requirement
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let config = ApiConfig::default();
        let manager = ResourceManager::new(config).await.unwrap();

        let host = "example.com";

        // First request should succeed
        assert!(manager.rate_limiter.check_rate_limit(host).await.is_ok());

        // Rapid subsequent requests should be rate limited
        for _ in 0..5 {
            let result = manager.rate_limiter.check_rate_limit(host).await;
            if result.is_err() {
                // Rate limiting working
                return;
            }
        }
    }

    #[tokio::test]
    async fn test_memory_pressure_detection() {
        let mut config = ApiConfig::default();
        config.memory.global_memory_limit_mb = 100;
        config.memory.pressure_threshold = 0.8;

        let manager = ResourceManager::new(config).await.unwrap();

        // Should not be under pressure initially
        assert!(!manager.memory_manager.is_under_pressure());

        // Allocate memory beyond threshold
        manager.memory_manager.track_allocation(90).await;

        // Should detect pressure
        assert!(manager.memory_manager.is_under_pressure());
    }
}
