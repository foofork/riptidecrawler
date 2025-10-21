/// Browser Pool Pre-warming Manager for CLI
///
/// This module provides CLI-level browser pool management with:
/// - Pre-warming of 1-3 browser instances on startup
/// - Health check loop (every 30s)
/// - Auto-restart on failure
/// - Checkout/checkin API
/// - Resource monitoring
/// - Graceful cleanup on exit
use anyhow::{anyhow, Result};
use riptide_browser::pool::{BrowserCheckout as HeadlessCheckout, BrowserPool, BrowserPoolConfig};

// spider_chrome types are used directly by pool.rs
// We access them the same way as pool.rs does
use chromiumoxide::{Browser, BrowserConfig, Page};

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use tokio::time::interval;
use tracing::{debug, info, warn};

/// Configuration for CLI-level browser pool manager
#[derive(Clone, Debug)]
pub struct PoolManagerConfig {
    /// Number of browsers to pre-warm on startup
    pub prewarm_count: usize,
    /// Maximum browsers in pool
    pub max_pool_size: usize,
    /// Health check interval
    pub health_check_interval: Duration,
    /// Auto-restart failed browsers
    pub auto_restart: bool,
    /// Enable resource monitoring
    pub enable_monitoring: bool,
}

impl Default for PoolManagerConfig {
    fn default() -> Self {
        Self {
            prewarm_count: 2,
            max_pool_size: 5,
            health_check_interval: Duration::from_secs(30),
            auto_restart: true,
            enable_monitoring: true,
        }
    }
}

/// Resource usage statistics
#[derive(Debug, Clone, Default)]
pub struct ResourceStats {
    pub memory_mb: u64,
    pub cpu_percent: f64,
    pub active_browsers: usize,
    pub idle_browsers: usize,
    pub total_checkouts: u64,
    pub failed_checkouts: u64,
}

/// CLI Browser Pool Manager
pub struct BrowserPoolManager {
    pool: Arc<BrowserPool>,
    config: PoolManagerConfig,
    stats: Arc<RwLock<ResourceStats>>,
    health_checker: Arc<Mutex<HealthChecker>>,
    shutdown_tx: tokio::sync::watch::Sender<bool>,
    _health_task: tokio::task::JoinHandle<()>,
}

impl BrowserPoolManager {
    /// Create new browser pool manager with pre-warming
    pub async fn new(config: PoolManagerConfig) -> Result<Self> {
        info!(
            prewarm_count = config.prewarm_count,
            max_pool_size = config.max_pool_size,
            "Initializing CLI browser pool manager"
        );

        // Configure underlying browser pool
        let pool_config = BrowserPoolConfig {
            min_pool_size: 1,
            max_pool_size: config.max_pool_size,
            initial_pool_size: config.prewarm_count,
            idle_timeout: Duration::from_secs(60),
            max_lifetime: Duration::from_secs(300),
            health_check_interval: config.health_check_interval,
            memory_threshold_mb: 500,
            enable_recovery: config.auto_restart,
            max_retries: 3,
            profile_base_dir: None,
            cleanup_timeout: Duration::from_secs(5),
            // QW-2: Tiered health checks
            enable_tiered_health_checks: true,
            fast_check_interval: Duration::from_secs(2),
            full_check_interval: Duration::from_secs(15),
            error_check_delay: Duration::from_millis(500),
            // QW-3: Memory limits
            enable_memory_limits: true,
            memory_check_interval: Duration::from_secs(5),
            memory_soft_limit_mb: 400,
            memory_hard_limit_mb: 500,
            enable_v8_heap_stats: true,
        };

        // Build browser config
        let browser_config = BrowserConfig::builder()
            .arg("--no-sandbox")
            .arg("--disable-dev-shm-usage")
            .arg("--disable-gpu")
            .build()
            .map_err(|e| anyhow!("Failed to build browser config: {}", e))?;

        // Initialize pool
        let pool = Arc::new(BrowserPool::new(pool_config, browser_config).await?);

        let stats = Arc::new(RwLock::new(ResourceStats::default()));
        let health_checker = Arc::new(Mutex::new(HealthChecker::new()));

        let (shutdown_tx, mut shutdown_rx) = tokio::sync::watch::channel(false);

        // Start health check loop
        let health_task = {
            let pool = Arc::clone(&pool);
            let stats = Arc::clone(&stats);
            let health_checker = Arc::clone(&health_checker);
            let interval_duration = config.health_check_interval;
            let enable_monitoring = config.enable_monitoring;

            tokio::spawn(async move {
                let mut interval = interval(interval_duration);

                loop {
                    tokio::select! {
                        _ = interval.tick() => {
                            debug!("Running health check loop");

                            // Perform health checks
                            let health_result = health_checker.lock().await.check_pool_health(&pool).await;

                            if let Err(e) = health_result {
                                warn!(error = %e, "Health check failed");
                            }

                            // Update resource statistics
                            if enable_monitoring {
                                let headless_stats = pool.stats().await;
                                let mut stats = stats.write().await;
                                stats.active_browsers = headless_stats.in_use;
                                stats.idle_browsers = headless_stats.available;
                                // Memory estimation: ~200MB per browser
                                stats.memory_mb = ((headless_stats.in_use + headless_stats.available) as u64) * 200;
                            }
                        }
                        _ = shutdown_rx.changed() => {
                            if *shutdown_rx.borrow() {
                                info!("Health check loop shutting down");
                                break;
                            }
                        }
                    }
                }
            })
        };

        info!(
            "Browser pool manager initialized with {} pre-warmed browsers",
            config.prewarm_count
        );

        Ok(Self {
            pool,
            config,
            stats,
            health_checker,
            shutdown_tx,
            _health_task: health_task,
        })
    }

    /// Checkout a browser from the pool
    pub async fn checkout(&self) -> Result<BrowserInstance> {
        debug!("Checking out browser from pool");

        let start = std::time::Instant::now();
        let checkout = self.pool.checkout().await?;
        let checkout_time = start.elapsed();

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_checkouts += 1;
        }

        debug!(
            checkout_time_ms = checkout_time.as_millis(),
            "Browser checked out successfully"
        );

        Ok(BrowserInstance {
            inner: checkout,
            checked_out_at: std::time::Instant::now(),
        })
    }

    /// Check in a browser back to the pool
    pub async fn checkin(&self, instance: BrowserInstance) {
        let duration = instance.checked_out_at.elapsed();
        debug!(
            usage_duration_ms = duration.as_millis(),
            "Checking in browser"
        );

        // Use the cleanup method to properly check in
        if let Err(e) = instance.inner.cleanup().await {
            warn!(error = %e, "Error checking in browser");
        }
    }

    /// Get current pool statistics
    pub async fn pool_stats(&self) -> Result<PoolStats> {
        let headless_pool_stats = self.pool.stats().await;
        let resource_stats = self.stats.read().await;

        Ok(PoolStats {
            available: headless_pool_stats.available,
            in_use: headless_pool_stats.in_use,
            total_capacity: headless_pool_stats.total_capacity,
            utilization: headless_pool_stats.utilization,
            memory_mb: resource_stats.memory_mb,
            cpu_percent: resource_stats.cpu_percent,
            total_checkouts: resource_stats.total_checkouts,
            failed_checkouts: resource_stats.failed_checkouts,
        })
    }

    /// Get resource statistics
    pub async fn resource_stats(&self) -> ResourceStats {
        self.stats.read().await.clone()
    }

    /// Force a health check now
    pub async fn health_check_now(&self) -> Result<HealthStatus> {
        self.health_checker
            .lock()
            .await
            .check_pool_health(&self.pool)
            .await
    }

    /// Gracefully shutdown the pool manager
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down browser pool manager");

        // Signal health check loop to stop
        let _ = self.shutdown_tx.send(true);

        // Wait a moment for health check to stop
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Shutdown the underlying pool
        self.pool.shutdown().await?;

        info!("Browser pool manager shutdown complete");
        Ok(())
    }
}

/// Wrapper around HeadlessCheckout for CLI usage
pub struct BrowserInstance {
    inner: HeadlessCheckout,
    checked_out_at: std::time::Instant,
}

impl BrowserInstance {
    /// Get the browser ID
    pub fn id(&self) -> String {
        self.inner.browser_id().to_string()
    }

    /// Get usage duration
    pub fn usage_duration(&self) -> Duration {
        self.checked_out_at.elapsed()
    }

    /// Create a new page in the browser
    pub async fn new_page(&self, url: &str) -> Result<Page> {
        self.inner.new_page(url).await
    }

    /// Manually consume and checkin (preferred over drop)
    pub async fn checkin(self) -> Result<()> {
        self.inner.cleanup().await
    }
}

/// Pool statistics for monitoring
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub available: usize,
    pub in_use: usize,
    pub total_capacity: usize,
    pub utilization: f64,
    pub memory_mb: u64,
    pub cpu_percent: f64,
    pub total_checkouts: u64,
    pub failed_checkouts: u64,
}

/// Health check status
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub healthy: bool,
    pub healthy_count: usize,
    pub unhealthy_count: usize,
    pub issues: Vec<String>,
}

/// Health checker for browser pool
struct HealthChecker {
    last_check: Option<std::time::Instant>,
    consecutive_failures: u32,
}

impl HealthChecker {
    fn new() -> Self {
        Self {
            last_check: None,
            consecutive_failures: 0,
        }
    }

    async fn check_pool_health(&mut self, pool: &BrowserPool) -> Result<HealthStatus> {
        self.last_check = Some(std::time::Instant::now());

        let stats = pool.stats().await;
        let mut issues = Vec::new();

        // Check for common issues
        if stats.available == 0 && stats.in_use == 0 {
            issues.push("No browsers available in pool".to_string());
            self.consecutive_failures += 1;
        } else {
            self.consecutive_failures = 0;
        }

        if stats.utilization > 90.0 {
            issues.push(format!("High pool utilization: {:.1}%", stats.utilization));
        }

        let healthy = issues.is_empty();

        if !healthy {
            warn!(
                issues = ?issues,
                consecutive_failures = self.consecutive_failures,
                "Health check detected issues"
            );
        } else {
            debug!("Health check passed");
        }

        Ok(HealthStatus {
            healthy,
            healthy_count: stats.available + stats.in_use,
            unhealthy_count: 0,
            issues,
        })
    }
}

/// Global pool manager instance (lazy initialization)
static GLOBAL_POOL_MANAGER: tokio::sync::OnceCell<Arc<BrowserPoolManager>> =
    tokio::sync::OnceCell::const_new();

/// Get or initialize the global pool manager
pub async fn get_global_pool_manager() -> Result<Arc<BrowserPoolManager>> {
    GLOBAL_POOL_MANAGER
        .get_or_try_init(|| async {
            let config = PoolManagerConfig::default();
            let manager = BrowserPoolManager::new(config).await?;
            Ok(Arc::new(manager))
        })
        .await
        .map(Arc::clone)
}

/// Shutdown the global pool manager if initialized
pub async fn shutdown_global_pool_manager() -> Result<()> {
    if let Some(manager) = GLOBAL_POOL_MANAGER.get() {
        manager.shutdown().await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pool_manager_creation() {
        let config = PoolManagerConfig {
            prewarm_count: 1,
            max_pool_size: 3,
            health_check_interval: Duration::from_secs(60),
            auto_restart: true,
            enable_monitoring: true,
        };

        let manager = BrowserPoolManager::new(config).await;
        assert!(manager.is_ok(), "Pool manager should initialize");

        if let Ok(manager) = manager {
            let stats = manager.pool_stats().await.unwrap();
            assert!(stats.available > 0, "Should have pre-warmed browsers");

            let _ = manager.shutdown().await;
        }
    }

    #[tokio::test]
    async fn test_checkout_checkin() {
        let config = PoolManagerConfig {
            prewarm_count: 1,
            ..Default::default()
        };

        let manager = BrowserPoolManager::new(config).await.unwrap();

        let instance = manager.checkout().await.unwrap();
        assert!(!instance.id().is_empty());

        manager.checkin(instance).await;

        let _ = manager.shutdown().await;
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = PoolManagerConfig {
            prewarm_count: 1,
            ..Default::default()
        };

        let manager = BrowserPoolManager::new(config).await.unwrap();

        let health = manager.health_check_now().await.unwrap();
        assert!(health.healthy, "Pool should be healthy");

        let _ = manager.shutdown().await;
    }
}
