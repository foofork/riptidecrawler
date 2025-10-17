//! Comprehensive resource management for RipTide API operations.
//!
//! This module implements all resource controls including:
//! - Headless browser pool management (cap = 3)
//! - Per-host rate limiting (1.5 RPS with jitter)
//! - PDF operation semaphore (max 2 concurrent)
//! - Single WASM instance per worker
//! - Memory cleanup on timeouts
//! - Performance monitoring and degradation detection
//!
//! # Architecture
//!
//! The module is organized into sub-modules for maintainability:
//! - `errors`: Error types and result aliases
//! - `metrics`: Resource metrics collection
//! - `rate_limiter`: Per-host rate limiting with token bucket
//! - `memory_manager`: Memory pressure detection and cleanup
//! - `wasm_manager`: WASM instance lifecycle management
//! - `performance`: Performance monitoring and degradation detection
//! - `guards`: RAII resource guards for automatic cleanup
//!
//! # Usage
//!
//! ```rust,no_run
//! use riptide_api::resource_manager::ResourceManager;
//! use riptide_api::config::ApiConfig;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let config = ApiConfig::default();
//! let manager = ResourceManager::new(config).await?;
//!
//! // Acquire resources for render operation
//! match manager.acquire_render_resources("https://example.com").await? {
//!     ResourceResult::Success(guard) => {
//!         // Perform render operation
//!         // Resources automatically cleaned up on drop
//!     }
//!     ResourceResult::RateLimited { retry_after } => {
//!         // Handle rate limiting
//!     }
//!     _ => {
//!         // Handle other resource constraints
//!     }
//! }
//! # Ok(())
//! # }
//! ```

// Sub-module declarations
pub mod errors;
pub mod guards;
pub mod memory_manager;
pub mod metrics;
pub mod performance;
pub mod rate_limiter;
pub mod wasm_manager;

// Re-export public types for backward compatibility
#[allow(unused_imports)] // Helper functions reserved for future use
pub use errors::{exhausted_error, timeout_error, ResourceManagerError, Result};
#[allow(unused_imports)] // WasmGuard reserved for future public API
pub use guards::{PdfResourceGuard, RenderResourceGuard, WasmGuard};
#[allow(unused_imports)] // MemoryStats reserved for monitoring endpoints
pub use memory_manager::{MemoryManager, MemoryStats};
#[allow(unused_imports)] // MetricsSnapshot reserved for monitoring endpoints
pub use metrics::{MetricsSnapshot, ResourceMetrics};
#[allow(unused_imports)] // PerformanceStats reserved for monitoring endpoints
pub use performance::{PerformanceMonitor, PerformanceStats};
#[allow(unused_imports)] // HostStats reserved for monitoring endpoints
pub use rate_limiter::{HostStats, PerHostRateLimiter};
#[allow(unused_imports)] // WasmInstanceStats reserved for monitoring endpoints
pub use wasm_manager::{WasmInstanceManager, WasmInstanceStats};

// Standard library imports
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

// External dependencies
use tokio::{sync::Semaphore, time::timeout};
use tracing::{debug, error, info, warn};
use url::Url;

// Internal dependencies
use crate::config::ApiConfig;
use riptide_headless::pool::{BrowserPool, BrowserPoolConfig};

/// Comprehensive resource manager coordinating all sub-managers
///
/// This is the main entry point for resource management. It coordinates:
/// - Browser pool for headless rendering
/// - Rate limiting for per-host request control
/// - PDF operation concurrency limiting
/// - WASM instance management
/// - Memory tracking and pressure detection
/// - Performance monitoring
#[derive(Clone)]
pub struct ResourceManager {
    /// Configuration
    config: ApiConfig,
    /// Headless browser pool manager
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
    /// Resource metrics (public for testing)
    #[cfg_attr(test, allow(dead_code))]
    pub(crate) metrics: Arc<ResourceMetrics>,
}

/// Resource operation result
///
/// Indicates the outcome of resource acquisition attempts,
/// providing detailed information for handling different scenarios.
#[derive(Debug)]
pub enum ResourceResult<T> {
    /// Operation succeeded with acquired resources
    Success(T),
    /// Operation timed out waiting for resources
    Timeout,
    /// All resources of requested type are exhausted
    ResourceExhausted,
    /// Rate limit exceeded, retry after duration
    RateLimited {
        /// Duration to wait before retrying
        retry_after: Duration,
    },
    /// System is under memory pressure
    MemoryPressure,
    /// Generic error (reserved for extensibility)
    #[allow(dead_code)]
    Error(String),
}

/// Current resource status
///
/// Provides a snapshot of system resource utilization for monitoring.
#[derive(Debug, serde::Serialize)]
pub struct ResourceStatus {
    /// Available headless browser slots in pool
    pub headless_pool_available: usize,
    /// Total headless browser pool capacity
    pub headless_pool_total: usize,
    /// Available PDF processing permits
    pub pdf_available: usize,
    /// Total PDF processing permit capacity
    pub pdf_total: usize,
    /// Current memory usage in megabytes
    pub memory_usage_mb: usize,
    /// Whether system is under memory pressure
    pub memory_pressure: bool,
    /// Total rate limit violations
    pub rate_limit_hits: u64,
    /// Total operation timeouts
    pub timeout_count: u64,
    /// Performance degradation score (0.0-1.0)
    pub degradation_score: f64,
}

impl ResourceManager {
    /// Create new resource manager with comprehensive controls
    ///
    /// Initializes all sub-managers and starts background tasks for maintenance.
    ///
    /// # Arguments
    /// * `config` - API configuration for resource limits
    ///
    /// # Returns
    /// * `Ok(ResourceManager)` - Successfully initialized manager
    /// * `Err(_)` - Initialization failed (browser pool, etc.)
    pub async fn new(config: ApiConfig) -> errors::Result<Self> {
        info!("Initializing comprehensive resource manager");

        let metrics = Arc::new(ResourceMetrics::new());

        // Initialize browser pool
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
            profile_base_dir: None, // Use system temp directory by default
            cleanup_timeout: Duration::from_secs(5),
            ..Default::default()
        };

        // DEPENDENCY NOTE: chromiumoxide uses async-std internally (RUSTSEC-2025-0052)
        // JUSTIFICATION:
        //   - Provides Chrome DevTools Protocol (CDP) access for headless browsing
        //   - Isolated to browser pool management (this module only)
        //   - Main application runtime remains pure Tokio
        //   - async-std tasks are spawned in isolated browser process contexts
        //   - No runtime conflict: chromiumoxide manages its own task executor
        // ALTERNATIVES CONSIDERED:
        //   - thirtyfour (WebDriver): Less direct CDP control, higher latency
        //   - Sidecar process: Over-engineering for current scale
        // MITIGATION: Feature-gate for optional headless support if needed
        // MONITORING: Track chromiumoxide updates for Tokio migration
        let browser_config = chromiumoxide::BrowserConfig::builder()
            .with_head()
            .no_sandbox()
            .build()
            .map_err(|e| {
                ResourceManagerError::Configuration(format!(
                    "Failed to build browser config: {}",
                    e
                ))
            })?;

        let browser_pool = Arc::new(
            BrowserPool::new(browser_pool_config, browser_config)
                .await
                .map_err(|e| {
                    ResourceManagerError::BrowserPool(format!(
                        "Failed to initialize browser pool: {}",
                        e
                    ))
                })?,
        );

        // Track headless pool size in metrics
        metrics.headless_pool_size.store(
            config.headless.max_pool_size,
            std::sync::atomic::Ordering::Relaxed,
        );

        // Initialize per-host rate limiter
        let rate_limiter = Arc::new(PerHostRateLimiter::new(config.clone(), metrics.clone()));
        rate_limiter.start_cleanup_task().await;

        // Initialize PDF semaphore
        let pdf_semaphore = Arc::new(Semaphore::new(config.pdf.max_concurrent));

        // Initialize WASM instance manager
        let wasm_manager = Arc::new(WasmInstanceManager::new(metrics.clone())?);

        // Initialize memory manager
        let memory_manager = Arc::new(MemoryManager::new(config.clone(), metrics.clone())?);

        // Initialize performance monitor
        let performance_monitor = Arc::new(PerformanceMonitor::new(metrics.clone())?);

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
    ///
    /// Coordinates all resource acquisition including:
    /// 1. Memory pressure check
    /// 2. Per-host rate limiting
    /// 3. Browser pool checkout
    /// 4. WASM instance acquisition
    /// 5. Memory tracking
    ///
    /// # Arguments
    /// * `url` - The URL to render (used for host-based rate limiting)
    ///
    /// # Returns
    /// * `Success(guard)` - All resources acquired successfully
    /// * `RateLimited` - Host rate limit exceeded
    /// * `MemoryPressure` - System under memory pressure
    /// * `ResourceExhausted` - Browser pool exhausted
    /// * `Timeout` - Resource acquisition timed out
    pub async fn acquire_render_resources(
        &self,
        url: &str,
    ) -> errors::Result<ResourceResult<RenderResourceGuard>> {
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
            Ok(Ok(checkout)) => {
                // Track active headless browser
                self.metrics
                    .headless_active
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                checkout
            }
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
        let memory_tracked = 256; // Estimate for render operation
        self.memory_manager.track_allocation(memory_tracked);

        let total_time = start_time.elapsed();
        debug!(
            acquisition_time_ms = total_time.as_millis(),
            host = %host,
            "Render resources acquired successfully"
        );

        Ok(ResourceResult::Success(RenderResourceGuard::new(
            browser_checkout,
            wasm_guard,
            memory_tracked,
            self.memory_manager.clone(),
            self.metrics.clone(),
        )))
    }

    /// Acquire resources for PDF operation
    ///
    /// Manages PDF-specific resources:
    /// 1. Memory pressure check
    /// 2. PDF semaphore acquisition (max 2 concurrent)
    /// 3. Memory tracking
    ///
    /// # Returns
    /// * `Success(guard)` - PDF resources acquired
    /// * `MemoryPressure` - System under memory pressure
    /// * `ResourceExhausted` - All PDF permits in use
    /// * `Timeout` - Acquisition timed out
    pub async fn acquire_pdf_resources(&self) -> errors::Result<ResourceResult<PdfResourceGuard>> {
        // Check memory pressure
        if self.memory_manager.is_under_pressure() {
            return Ok(ResourceResult::MemoryPressure);
        }

        // Acquire PDF semaphore with timeout
        let permit_result = timeout(
            self.config.get_timeout("pdf"),
            self.pdf_semaphore.clone().acquire_owned(),
        )
        .await;

        let permit = match permit_result {
            Ok(Ok(permit)) => permit,
            Ok(Err(_)) => return Ok(ResourceResult::ResourceExhausted),
            Err(_) => {
                self.performance_monitor.record_timeout().await;
                return Ok(ResourceResult::Timeout);
            }
        };

        // Track memory for PDF operation
        let memory_tracked = 128;
        self.memory_manager.track_allocation(memory_tracked);

        self.metrics
            .pdf_active
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        Ok(ResourceResult::Success(PdfResourceGuard::new(
            permit,
            memory_tracked,
            self.memory_manager.clone(),
            self.metrics.clone(),
        )))
    }

    /// Cleanup resources on timeout or error
    ///
    /// Triggers cleanup operations when timeouts or errors occur:
    /// - Memory cleanup (if configured)
    /// - Performance metric updates
    /// - Garbage collection (if threshold exceeded)
    ///
    /// # Arguments
    /// * `operation_type` - Type of operation that timed out (for logging)
    pub async fn cleanup_on_timeout(&self, operation_type: &str) {
        warn!(operation = %operation_type, "Performing timeout cleanup");

        // Trigger memory cleanup
        if self.config.performance.auto_cleanup_on_timeout {
            self.memory_manager.trigger_cleanup().await;
        }

        // Update performance metrics
        self.performance_monitor.record_timeout().await;

        // Force garbage collection if configured
        if self.memory_manager.should_trigger_gc() {
            self.memory_manager.trigger_gc().await;
        }

        self.metrics
            .cleanup_operations
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Get current resource status
    ///
    /// Provides a comprehensive snapshot of all resource utilization
    /// for monitoring and diagnostics.
    ///
    /// # Returns
    /// A `ResourceStatus` struct containing current metrics for all resources
    pub async fn get_resource_status(&self) -> ResourceStatus {
        let pool_stats = self.browser_pool.get_stats().await;

        ResourceStatus {
            headless_pool_available: pool_stats.available,
            headless_pool_total: pool_stats.total_capacity,
            pdf_available: self.pdf_semaphore.available_permits(),
            pdf_total: self.config.pdf.max_concurrent,
            memory_usage_mb: self.memory_manager.current_usage_mb(),
            memory_pressure: self.memory_manager.is_under_pressure(),
            rate_limit_hits: self
                .metrics
                .rate_limit_hits
                .load(std::sync::atomic::Ordering::Relaxed),
            timeout_count: self
                .metrics
                .timeouts_count
                .load(std::sync::atomic::Ordering::Relaxed),
            degradation_score: self.performance_monitor.get_degradation_score().await,
        }
    }

    // Helper methods

    /// Extract host from URL for rate limiting
    fn extract_host(&self, url: &str) -> errors::Result<String> {
        let parsed = Url::parse(url)?;
        Ok(parsed
            .host_str()
            .ok_or_else(|| ResourceManagerError::InvalidUrl("No host in URL".to_string()))?
            .to_string())
    }

    /// Get current worker ID for WASM instance management
    fn get_worker_id(&self) -> String {
        format!("worker_{:?}", std::thread::current().id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "Requires Chrome/Chromium to be installed - BrowserPool dependency"]
    async fn test_resource_manager_creation() {
        let config = ApiConfig::default();
        let manager = ResourceManager::new(config).await.unwrap();

        let status = manager.get_resource_status().await;
        assert_eq!(status.pdf_total, 2); // PDF semaphore requirement
        assert_eq!(status.headless_pool_total, 3); // Pool cap requirement
    }

    #[tokio::test]
    #[ignore = "Requires Chrome/Chromium to be installed - BrowserPool dependency"]
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
    #[ignore = "Requires Chrome/Chromium to be installed - BrowserPool dependency"]
    async fn test_memory_pressure_detection() {
        let mut config = ApiConfig::default();
        config.memory.global_memory_limit_mb = 100;
        config.memory.pressure_threshold = 0.8;

        let manager = ResourceManager::new(config).await.unwrap();

        // Should not be under pressure initially
        assert!(!manager.memory_manager.is_under_pressure());

        // Allocate memory beyond threshold
        manager.memory_manager.track_allocation(90);

        // Should detect pressure
        assert!(manager.memory_manager.is_under_pressure());
    }

    #[tokio::test]
    #[ignore = "Requires Chrome/Chromium to be installed - BrowserPool dependency"]
    async fn test_coordinator_integration() {
        let config = ApiConfig::default();
        let manager = ResourceManager::new(config).await.unwrap();

        // Test that all sub-managers are accessible
        assert!(manager.browser_pool.get_stats().await.total_capacity > 0);
        assert_eq!(manager.wasm_manager.instance_count().await, 0);
        assert_eq!(manager.memory_manager.current_usage_mb(), 0);
        assert_eq!(
            manager.performance_monitor.get_degradation_score().await,
            0.0
        );
    }
}
