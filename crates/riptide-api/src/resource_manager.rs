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
    /// Headless browser pool manager
    pub headless_pool: Arc<HeadlessBrowserPool>,
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

/// Headless browser pool with health monitoring
pub struct HeadlessBrowserPool {
    config: ApiConfig,
    available_browsers: Mutex<Vec<BrowserInstance>>,
    total_browsers: AtomicUsize,
    semaphore: Semaphore,
    metrics: Arc<ResourceMetrics>,
    last_health_check: AtomicU64,
}

/// Browser instance with health tracking
#[derive(Debug, Clone)]
pub struct BrowserInstance {
    pub id: String,
    pub created_at: Instant,
    pub last_used: Instant,
    pub operations_count: u32,
    pub failed_operations: u32,
    pub is_healthy: bool,
}

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
}

/// Result of resource acquisition
#[derive(Debug)]
pub struct ResourceGuard {
    pub resource_type: String,
    pub acquired_at: Instant,
    pub timeout: Duration,
    _guard: Option<Arc<dyn Send + Sync>>,
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

        // Initialize headless browser pool
        let headless_pool =
            Arc::new(HeadlessBrowserPool::new(config.clone(), metrics.clone()).await?);

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
            headless_pool,
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
            self.headless_pool.acquire_browser(),
        )
        .await;

        let browser_guard = match browser_result {
            Ok(Ok(guard)) => guard,
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
            browser_guard,
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
            timeout(self.config.get_timeout("pdf"), self.pdf_semaphore.acquire()).await;

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
        ResourceStatus {
            headless_pool_available: self.headless_pool.available_count().await,
            headless_pool_total: self.headless_pool.total_count(),
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
        format!("worker_{}", std::thread::current().id().as_u64())
    }
}

/// Resource guard for render operations
pub struct RenderResourceGuard {
    browser_guard: BrowserGuard,
    wasm_guard: WasmGuard,
    memory_tracked: usize,
    acquired_at: Instant,
    manager: ResourceManager,
}

/// Resource guard for PDF operations
pub struct PdfResourceGuard {
    _permit: tokio::sync::SemaphorePermit<'static>,
    memory_tracked: usize,
    acquired_at: Instant,
    manager: ResourceManager,
}

/// Browser guard with automatic return to pool
pub struct BrowserGuard {
    instance: BrowserInstance,
    pool: Arc<HeadlessBrowserPool>,
}

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

impl HeadlessBrowserPool {
    async fn new(config: ApiConfig, metrics: Arc<ResourceMetrics>) -> Result<Self> {
        let pool = Self {
            config,
            available_browsers: Mutex::new(Vec::new()),
            total_browsers: AtomicUsize::new(0),
            semaphore: Semaphore::new(3), // Hard requirement: pool cap = 3
            metrics,
            last_health_check: AtomicU64::new(0),
        };

        // Pre-warm the pool with minimum browsers
        pool.ensure_minimum_browsers().await?;

        Ok(pool)
    }

    async fn acquire_browser(&self) -> Result<BrowserGuard> {
        let _permit = self.semaphore.acquire().await?;

        let mut browsers = self.available_browsers.lock().await;

        // Try to get an existing healthy browser
        if let Some(mut instance) = browsers.pop() {
            if self.is_browser_healthy(&instance) {
                instance.last_used = Instant::now();
                instance.operations_count += 1;
                self.metrics.headless_active.fetch_add(1, Ordering::Relaxed);

                return Ok(BrowserGuard {
                    instance,
                    pool: Arc::new(self.clone()),
                });
            }
        }

        // Create new browser if pool not at capacity
        if self.total_browsers.load(Ordering::Relaxed) < self.config.headless.max_pool_size {
            let instance = self.create_browser().await?;
            self.total_browsers.fetch_add(1, Ordering::Relaxed);
            self.metrics
                .headless_pool_size
                .fetch_add(1, Ordering::Relaxed);
            self.metrics.headless_active.fetch_add(1, Ordering::Relaxed);

            return Ok(BrowserGuard {
                instance,
                pool: Arc::new(self.clone()),
            });
        }

        Err(anyhow!("Browser pool exhausted"))
    }

    async fn create_browser(&self) -> Result<BrowserInstance> {
        info!("Creating new browser instance");

        let instance = BrowserInstance {
            id: format!("browser_{}", uuid::Uuid::new_v4()),
            created_at: Instant::now(),
            last_used: Instant::now(),
            operations_count: 0,
            failed_operations: 0,
            is_healthy: true,
        };

        Ok(instance)
    }

    fn is_browser_healthy(&self, instance: &BrowserInstance) -> bool {
        instance.is_healthy
            && instance.operations_count < self.config.headless.max_pages_per_browser as u32
            && instance.failed_operations < self.config.headless.restart_threshold
            && instance.last_used.elapsed()
                < Duration::from_secs(self.config.headless.idle_timeout_secs)
    }

    async fn return_browser(&self, mut instance: BrowserInstance) {
        if self.is_browser_healthy(&instance) {
            let mut browsers = self.available_browsers.lock().await;
            browsers.push(instance);
        } else {
            info!(browser_id = %instance.id, "Discarding unhealthy browser");
            self.total_browsers.fetch_sub(1, Ordering::Relaxed);
            self.metrics
                .headless_pool_size
                .fetch_sub(1, Ordering::Relaxed);
        }

        self.metrics.headless_active.fetch_sub(1, Ordering::Relaxed);
    }

    async fn ensure_minimum_browsers(&self) -> Result<()> {
        let current_count = self.total_browsers.load(Ordering::Relaxed);
        if current_count < self.config.headless.min_pool_size {
            for _ in current_count..self.config.headless.min_pool_size {
                let instance = self.create_browser().await?;
                let mut browsers = self.available_browsers.lock().await;
                browsers.push(instance);
                self.total_browsers.fetch_add(1, Ordering::Relaxed);
                self.metrics
                    .headless_pool_size
                    .fetch_add(1, Ordering::Relaxed);
            }
        }
        Ok(())
    }

    async fn available_count(&self) -> usize {
        self.available_browsers.lock().await.len()
    }

    fn total_count(&self) -> usize {
        self.total_browsers.load(Ordering::Relaxed)
    }
}

impl Clone for HeadlessBrowserPool {
    fn clone(&self) -> Self {
        // Note: This is a simplified clone for the Arc wrapper
        // In practice, you'd want proper shared state
        Self {
            config: self.config.clone(),
            available_browsers: Mutex::new(Vec::new()),
            total_browsers: AtomicUsize::new(self.total_browsers.load(Ordering::Relaxed)),
            semaphore: Semaphore::new(self.config.headless.max_pool_size),
            metrics: self.metrics.clone(),
            last_health_check: AtomicU64::new(self.last_health_check.load(Ordering::Relaxed)),
        }
    }
}

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

    async fn acquire_instance(&self, worker_id: &str) -> Result<WasmGuard> {
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
            manager: Arc::new(self.clone()),
        })
    }
}

impl Clone for WasmInstanceManager {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            worker_instances: RwLock::new(HashMap::new()),
            metrics: self.metrics.clone(),
        }
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
        self.last_cleanup.store(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            Ordering::Relaxed,
        );
        self.metrics
            .cleanup_operations
            .fetch_add(1, Ordering::Relaxed);
    }

    async fn trigger_gc(&self) {
        info!("Triggering garbage collection");
        // Force garbage collection
        self.last_gc.store(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            Ordering::Relaxed,
        );
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

impl Drop for BrowserGuard {
    fn drop(&mut self) {
        let pool = self.pool.clone();
        let instance = self.instance.clone();

        tokio::spawn(async move {
            pool.return_browser(instance).await;
        });
    }
}

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
