//! Browser pool management with resource tracking (WIP - scaffolding)
#![cfg_attr(not(feature = "headless"), allow(dead_code, unused))]

use anyhow::{anyhow, Result};
use spider_chrome::{Browser, BrowserConfig, Page};
use futures::StreamExt;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::sync::{mpsc, Mutex, RwLock, Semaphore};
use tokio::time::{interval, timeout};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Configuration for browser pool management
#[derive(Clone, Debug)]
#[allow(dead_code)] // Some fields are for future use
pub struct BrowserPoolConfig {
    /// Minimum number of browsers to keep in pool
    pub min_pool_size: usize,
    /// Maximum number of browsers in pool
    pub max_pool_size: usize,
    /// Initial pool size on startup
    pub initial_pool_size: usize,
    /// Idle timeout before browser cleanup (seconds)
    pub idle_timeout: Duration,
    /// Maximum lifetime for a browser instance (seconds)
    pub max_lifetime: Duration,
    /// Health check interval (seconds)
    pub health_check_interval: Duration,
    /// Memory threshold per browser (MB)
    pub memory_threshold_mb: u64,
    /// Enable automatic recovery for crashed browsers
    pub enable_recovery: bool,
    /// Maximum retries for browser operations
    pub max_retries: u32,
    /// Optional custom base directory for browser profiles
    ///
    /// When specified, browser profile temp directories will be created within this path.
    /// Useful for:
    /// - Container environments with specific temp mounts
    /// - Systems with limited /tmp space but other storage available
    /// - Testing and debugging scenarios
    ///
    /// When None (default), uses system temp directory.
    pub profile_base_dir: Option<std::path::PathBuf>,
    /// Cleanup timeout for browser checkin operations (seconds)
    pub cleanup_timeout: Duration,

    // QW-2: Tiered health check configuration for 5x faster failure detection
    /// Enable tiered health monitoring (fast checks + full checks)
    pub enable_tiered_health_checks: bool,
    /// Fast liveness check interval (quick ping, 2s default)
    pub fast_check_interval: Duration,
    /// Full health check interval (memory, CPU, pages, 15s default)
    pub full_check_interval: Duration,
    /// Error-triggered check delay (immediate re-validation, 500ms default)
    pub error_check_delay: Duration,

    // QW-3: Memory limit configuration for -30% memory footprint
    /// Enable memory limit monitoring and enforcement
    pub enable_memory_limits: bool,
    /// Memory check interval (5s default)
    pub memory_check_interval: Duration,
    /// Soft memory limit - trigger cleanup (400MB default)
    pub memory_soft_limit_mb: u64,
    /// Hard memory limit - force eviction (500MB default)
    pub memory_hard_limit_mb: u64,
    /// Enable V8 heap statistics tracking
    pub enable_v8_heap_stats: bool,
}

impl Default for BrowserPoolConfig {
    fn default() -> Self {
        Self {
            min_pool_size: 1,
            max_pool_size: 20, // QW-1: Increased from 5 to 20 for 4x capacity improvement
            initial_pool_size: 5, // Increased from 3 for better startup performance
            idle_timeout: Duration::from_secs(30),
            max_lifetime: Duration::from_secs(300), // 5 minutes
            health_check_interval: Duration::from_secs(10),
            memory_threshold_mb: 500,
            enable_recovery: true,
            max_retries: 3,
            profile_base_dir: None, // Use system temp directory by default
            cleanup_timeout: Duration::from_secs(5), // 5 second cleanup timeout

            // QW-2: Tiered health checks for 5x faster failure detection
            enable_tiered_health_checks: true,
            fast_check_interval: Duration::from_secs(2), // Quick liveness check
            full_check_interval: Duration::from_secs(15), // Comprehensive check
            error_check_delay: Duration::from_millis(500), // Immediate re-validation

            // QW-3: Memory limits for -30% memory footprint
            enable_memory_limits: true,
            memory_check_interval: Duration::from_secs(5),
            memory_soft_limit_mb: 400, // Trigger cleanup
            memory_hard_limit_mb: 500, // Force eviction
            enable_v8_heap_stats: true,
        }
    }
}

/// Health status of a browser instance
#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)] // Some variants are for future use
pub enum BrowserHealth {
    Healthy,
    Unhealthy,
    Crashed,
    MemoryExceeded,
    Timeout,
}

/// Statistics for browser instance usage
#[derive(Clone, Debug, Default)]
#[allow(dead_code)] // Some fields are for future use
pub struct BrowserStats {
    pub total_uses: u64,
    pub total_time_active: Duration,
    pub memory_usage_mb: u64,
    pub last_used: Option<Instant>,
    pub crashes: u32,
    pub timeouts: u32,
}

/// Individual browser instance in the pool
pub struct PooledBrowser {
    pub id: String,
    pub browser: Browser,
    pub created_at: Instant,
    pub last_used: Instant,
    pub stats: BrowserStats,
    pub health: BrowserHealth,
    pub in_use: bool,
    handler_task: tokio::task::JoinHandle<()>,
    _temp_dir: TempDir, // Keep temp directory alive for browser lifetime
}

// Manual Debug implementation (can't derive with TempDir)
impl std::fmt::Debug for PooledBrowser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PooledBrowser")
            .field("id", &self.id)
            .field("created_at", &self.created_at)
            .field("last_used", &self.last_used)
            .field("in_use", &self.in_use)
            .finish()
    }
}

impl PooledBrowser {
    /// Creates a new pooled browser instance with unique profile directory.
    ///
    /// **Why unique directories are required:**
    /// Chrome enforces SingletonLock at the profile level to prevent data corruption.
    /// Even with spider_chrome (which provides better CDP concurrency at the protocol level),
    /// each browser instance MUST have its own profile directory to allow concurrent operation
    /// without locking conflicts.
    ///
    /// **Important Architecture Notes:**
    /// - spider_chrome provides CDP-level concurrency improvements (better async/await, message handling)
    /// - spider_chrome does NOT manage browser profiles or bypass Chrome's SingletonLock
    /// - Profile isolation is a Chrome-level requirement, independent of the CDP library used
    ///
    /// **TempDir Lifetime Management:**
    /// The TempDir is kept alive via the `_temp_dir` field in PooledBrowser and automatically
    /// cleaned up when the PooledBrowser is dropped, ensuring:
    /// - No disk space leaks
    /// - Clean resource management
    /// - Proper cleanup even if browser crashes
    ///
    /// # Arguments
    /// * `_base_config` - Base browser configuration (currently unused, reserved for future use)
    /// * `profile_base_dir` - Optional custom base directory for browser profiles (defaults to system temp)
    ///
    /// # Returns
    /// A new PooledBrowser instance with unique profile directory
    ///
    /// # Errors
    /// Returns error if:
    /// - Failed to create temporary directory
    /// - Failed to build browser configuration
    /// - Failed to launch browser instance
    pub async fn new(
        _base_config: BrowserConfig,
        profile_base_dir: Option<&std::path::Path>,
    ) -> Result<Self> {
        let id = Uuid::new_v4().to_string();

        debug!(browser_id = %id, "Creating new browser instance");

        // Create truly unique temp directory for this browser instance
        // Uses custom base directory if provided, otherwise system temp
        let temp_dir = if let Some(base_dir) = profile_base_dir {
            TempDir::new_in(base_dir)
                .map_err(|e| anyhow!("Failed to create temp directory in {:?}: {}", base_dir, e))?
        } else {
            TempDir::new().map_err(|e| anyhow!("Failed to create temp directory: {}", e))?
        };

        let user_data_dir = temp_dir.path().to_path_buf();

        debug!(browser_id = %id, user_data_dir = ?user_data_dir, "Created unique browser profile directory");

        // Build config with unique user-data-dir
        // Do NOT use .arg() because spider_chrome adds its own default AFTER
        let mut browser_config = BrowserConfig::builder()
            .arg("--no-sandbox")
            .arg("--disable-dev-shm-usage")
            .arg("--disable-gpu")
            .arg("--disable-web-security")
            .arg("--disable-extensions")
            .arg("--disable-plugins")
            .arg("--disable-images")
            .arg("--disable-javascript")
            .arg("--disable-background-timer-throttling")
            .arg("--disable-backgrounding-occluded-windows")
            .arg("--disable-renderer-backgrounding")
            .arg("--memory-pressure-off")
            .build()
            .map_err(|e| anyhow!("Failed to build browser config: {}", e))?;

        // Set user_data_dir directly on the config struct to override the default
        browser_config.user_data_dir = Some(user_data_dir.clone());

        let (browser, mut handler) = Browser::launch(browser_config)
            .await
            .map_err(|e| anyhow!("Failed to launch browser {}: {}", id, e))?;

        // Spawn handler task to manage browser events
        let browser_id = id.clone();
        let handler_task = tokio::spawn(async move {
            debug!(browser_id = %browser_id, "Browser event handler started");
            while let Some(event) = handler.next().await {
                if let Err(e) = event {
                    warn!(browser_id = %browser_id, error = %e, "Browser event error");
                }
            }
            debug!(browser_id = %browser_id, "Browser event handler ended");
        });

        let now = Instant::now();
        Ok(Self {
            id,
            browser,
            created_at: now,
            last_used: now,
            stats: BrowserStats::default(),
            health: BrowserHealth::Healthy,
            in_use: false,
            handler_task,
            _temp_dir: temp_dir, // Keep temp directory alive
        })
    }

    /// Check if browser is expired based on lifetime limits
    pub fn is_expired(&self, max_lifetime: Duration) -> bool {
        self.created_at.elapsed() > max_lifetime
    }

    /// Check if browser is idle beyond timeout
    pub fn is_idle(&self, idle_timeout: Duration) -> bool {
        !self.in_use && self.last_used.elapsed() > idle_timeout
    }

    /// Update usage statistics
    #[allow(dead_code)] // Method for future use
    pub fn update_stats(&mut self, memory_usage_mb: u64) {
        self.stats.total_uses += 1;
        self.stats.memory_usage_mb = memory_usage_mb;
        self.stats.last_used = Some(Instant::now());
        self.last_used = Instant::now();
    }

    /// Perform health check on browser instance
    pub async fn health_check(&mut self, memory_threshold_mb: u64) -> BrowserHealth {
        // Check if browser is still responsive
        match timeout(Duration::from_secs(5), self.browser.pages()).await {
            Ok(Ok(_pages)) => {
                // Check memory usage
                if self.stats.memory_usage_mb > memory_threshold_mb {
                    self.health = BrowserHealth::MemoryExceeded;
                    warn!(
                        browser_id = %self.id,
                        memory_mb = self.stats.memory_usage_mb,
                        threshold_mb = memory_threshold_mb,
                        "Browser memory threshold exceeded"
                    );
                } else {
                    self.health = BrowserHealth::Healthy;
                }
            }
            Ok(Err(e)) => {
                error!(browser_id = %self.id, error = %e, "Browser health check failed");
                self.health = BrowserHealth::Unhealthy;
            }
            Err(_) => {
                error!(browser_id = %self.id, "Browser health check timed out");
                self.health = BrowserHealth::Timeout;
                self.stats.timeouts += 1;
            }
        }

        self.health.clone()
    }

    /// P1-B2: Fast liveness check (quick ping, 2s default)
    /// Returns true if browser responds, false if unresponsive
    pub async fn fast_health_check(&self) -> bool {
        // Quick check: just verify browser is alive with short timeout
        timeout(Duration::from_millis(500), self.browser.pages())
            .await
            .is_ok()
    }

    /// P1-B2: Full health check (comprehensive, 15s intervals)
    /// Checks memory, page count, and detailed browser state
    pub async fn full_health_check(&mut self, soft_limit: u64, hard_limit: u64) -> BrowserHealth {
        // Comprehensive check with detailed diagnostics
        match timeout(Duration::from_secs(5), self.browser.pages()).await {
            Ok(Ok(pages)) => {
                let page_count = pages.len();

                // Check memory against soft and hard limits
                if self.stats.memory_usage_mb > hard_limit {
                    self.health = BrowserHealth::MemoryExceeded;
                    error!(
                        browser_id = %self.id,
                        memory_mb = self.stats.memory_usage_mb,
                        hard_limit_mb = hard_limit,
                        page_count = page_count,
                        "Browser exceeded hard memory limit - forcing eviction"
                    );
                } else if self.stats.memory_usage_mb > soft_limit {
                    warn!(
                        browser_id = %self.id,
                        memory_mb = self.stats.memory_usage_mb,
                        soft_limit_mb = soft_limit,
                        page_count = page_count,
                        "Browser exceeded soft memory limit - cleanup recommended"
                    );
                    self.health = BrowserHealth::Healthy; // Still functional but needs attention
                } else {
                    self.health = BrowserHealth::Healthy;
                    debug!(
                        browser_id = %self.id,
                        memory_mb = self.stats.memory_usage_mb,
                        page_count = page_count,
                        "Browser health check passed"
                    );
                }
            }
            Ok(Err(e)) => {
                error!(browser_id = %self.id, error = %e, "Full health check failed");
                self.health = BrowserHealth::Unhealthy;
                self.stats.crashes += 1;
            }
            Err(_) => {
                error!(browser_id = %self.id, "Full health check timed out");
                self.health = BrowserHealth::Timeout;
                self.stats.timeouts += 1;
            }
        }

        self.health.clone()
    }

    /// Clean up browser resources
    pub async fn cleanup(&mut self) {
        debug!(browser_id = %self.id, "Cleaning up browser instance");

        // Abort the handler task
        self.handler_task.abort();

        // Close browser
        if let Err(e) = self.browser.close().await {
            warn!(browser_id = %self.id, error = %e, "Error closing browser during cleanup");
        }

        debug!(browser_id = %self.id, "Browser cleanup completed");
    }
}

impl Drop for PooledBrowser {
    fn drop(&mut self) {
        self.handler_task.abort();
        debug!(browser_id = %self.id, "Browser instance dropped");
    }
}

/// Browser pool events for monitoring
#[derive(Debug, Clone)]
#[allow(dead_code)] // Some variants and fields are for future use
pub enum PoolEvent {
    BrowserCreated { id: String },
    BrowserRemoved { id: String, reason: String },
    BrowserCheckedOut { id: String },
    BrowserCheckedIn { id: String },
    PoolExpanded { new_size: usize },
    PoolShrunk { new_size: usize },
    HealthCheckCompleted { healthy: usize, unhealthy: usize },
    MemoryAlert { browser_id: String, memory_mb: u64 },
}

/// Main browser pool manager
pub struct BrowserPool {
    config: BrowserPoolConfig,
    browser_config: BrowserConfig,
    available: Arc<Mutex<VecDeque<PooledBrowser>>>,
    in_use: Arc<RwLock<HashMap<String, PooledBrowser>>>,
    semaphore: Arc<Semaphore>,
    event_sender: mpsc::UnboundedSender<PoolEvent>,
    event_receiver: Arc<Mutex<mpsc::UnboundedReceiver<PoolEvent>>>,
    #[allow(dead_code)]
    shutdown_sender: mpsc::Sender<()>,
    _management_task: tokio::task::JoinHandle<()>,
}

impl BrowserPool {
    /// Create a new browser pool with specified configuration
    pub async fn new(config: BrowserPoolConfig, browser_config: BrowserConfig) -> Result<Self> {
        info!(
            min_size = config.min_pool_size,
            max_size = config.max_pool_size,
            initial_size = config.initial_pool_size,
            "Initializing browser pool"
        );

        let available = Arc::new(Mutex::new(VecDeque::new()));
        let in_use = Arc::new(RwLock::new(HashMap::new()));
        let semaphore = Arc::new(Semaphore::new(config.max_pool_size));

        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        let event_receiver = Arc::new(Mutex::new(event_receiver));

        let (shutdown_sender, mut shutdown_receiver) = mpsc::channel(1);

        // Create initial browser instances (continue on failures for graceful degradation)
        let mut initial_browsers = VecDeque::new();
        let mut failed_count = 0;
        for i in 0..config.initial_pool_size {
            match PooledBrowser::new(browser_config.clone(), config.profile_base_dir.as_deref())
                .await
            {
                Ok(browser) => {
                    let _ = event_sender.send(PoolEvent::BrowserCreated {
                        id: browser.id.clone(),
                    });
                    initial_browsers.push_back(browser);
                }
                Err(e) => {
                    failed_count += 1;
                    warn!(
                        attempt = i + 1,
                        error = %e,
                        "Failed to create initial browser instance (will continue)"
                    );
                }
            }
        }

        if failed_count > 0 {
            warn!(
                succeeded = initial_browsers.len(),
                failed = failed_count,
                "Browser pool initialized with reduced capacity due to launch failures"
            );
        }

        *available.lock().await = initial_browsers;

        // Start management task for health checks and cleanup
        // P1-B2: Enhanced with tiered health monitoring
        let management_task = {
            let config = config.clone();
            let browser_config = browser_config.clone();
            let available = available.clone();
            let in_use = in_use.clone();
            let event_sender = event_sender.clone();

            tokio::spawn(async move {
                // P1-B2: Tiered health check intervals
                let fast_interval = if config.enable_tiered_health_checks {
                    config.fast_check_interval
                } else {
                    config.health_check_interval
                };
                let full_interval = if config.enable_tiered_health_checks {
                    config.full_check_interval
                } else {
                    config.health_check_interval
                };

                let mut fast_check_interval = interval(fast_interval);
                let mut full_check_interval = interval(full_interval);
                let mut memory_check_interval = if config.enable_memory_limits {
                    Some(interval(config.memory_check_interval))
                } else {
                    None
                };

                info!(
                    tiered_checks = config.enable_tiered_health_checks,
                    fast_interval_ms = fast_interval.as_millis(),
                    full_interval_ms = full_interval.as_millis(),
                    memory_monitoring = config.enable_memory_limits,
                    "Browser pool management task started with tiered monitoring"
                );

                loop {
                    tokio::select! {
                        // P1-B2: Fast liveness checks (2s intervals)
                        _ = fast_check_interval.tick() => {
                            if config.enable_tiered_health_checks {
                                Self::perform_fast_health_checks(
                                    &available,
                                    &event_sender,
                                ).await;
                            }
                        }

                        // P1-B2: Full health checks (15s intervals)
                        _ = full_check_interval.tick() => {
                            Self::perform_full_health_checks(
                                &config,
                                &available,
                                &in_use,
                                &event_sender,
                            ).await;

                            Self::cleanup_expired_browsers(
                                &config,
                                &available,
                                &event_sender,
                            ).await;

                            Self::maintain_pool_size(
                                &config,
                                &browser_config,
                                &available,
                                &event_sender,
                            ).await;
                        }

                        // QW-3: Memory limit checks (5s intervals)
                        _ = async {
                            if let Some(ref mut interval) = memory_check_interval {
                                interval.tick().await;
                            } else {
                                // Never complete if memory checks disabled
                                std::future::pending::<()>().await;
                            }
                        } => {
                            Self::perform_memory_checks(
                                &config,
                                &available,
                                &in_use,
                                &event_sender,
                            ).await;
                        }

                        _ = shutdown_receiver.recv() => {
                            info!("Browser pool management task shutting down");
                            break;
                        }
                    }
                }
            })
        };

        info!(
            initial_browsers = available.lock().await.len(),
            "Browser pool initialized successfully"
        );

        Ok(Self {
            config,
            browser_config,
            available,
            in_use,
            semaphore,
            event_sender,
            event_receiver,
            shutdown_sender,
            _management_task: management_task,
        })
    }

    /// Check out a browser from the pool
    pub async fn checkout(&self) -> Result<BrowserCheckout> {
        // Acquire semaphore permit to limit concurrent browsers
        let permit = self
            .semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| anyhow!("Failed to acquire browser permit: {}", e))?;

        // Try to get an available browser
        let mut browser = {
            let mut available = self.available.lock().await;
            available.pop_front()
        };

        // If no browser available, try to create a new one
        if browser.is_none() {
            debug!("No available browsers, attempting to create new instance");
            match PooledBrowser::new(
                self.browser_config.clone(),
                self.config.profile_base_dir.as_deref(),
            )
            .await
            {
                Ok(new_browser) => {
                    let _ = self.event_sender.send(PoolEvent::BrowserCreated {
                        id: new_browser.id.clone(),
                    });
                    browser = Some(new_browser);
                }
                Err(e) => {
                    error!(error = %e, "Failed to create new browser instance");
                    return Err(anyhow!("Failed to checkout browser: {}", e));
                }
            }
        }

        if let Some(mut browser) = browser {
            browser.in_use = true;
            let browser_id = browser.id.clone();

            // Move browser to in_use collection
            {
                let mut in_use = self.in_use.write().await;
                in_use.insert(browser_id.clone(), browser);
            }

            let _ = self.event_sender.send(PoolEvent::BrowserCheckedOut {
                id: browser_id.clone(),
            });

            debug!(browser_id = %browser_id, "Browser checked out from pool");

            Ok(BrowserCheckout {
                browser_id,
                pool: BrowserPoolRef::new(self),
                permit: Some(permit),
            })
        } else {
            Err(anyhow!(
                "No browsers available and failed to create new instance"
            ))
        }
    }

    /// Check in a browser back to the pool
    pub async fn checkin(&self, browser_id: &str) -> Result<()> {
        let mut browser_opt = {
            let mut in_use = self.in_use.write().await;
            in_use.remove(browser_id)
        };

        if let Some(mut browser) = browser_opt.take() {
            browser.in_use = false;

            // Perform health check before returning to pool
            let health = browser.health_check(self.config.memory_threshold_mb).await;

            match health {
                BrowserHealth::Healthy => {
                    // Return healthy browser to pool
                    let mut available = self.available.lock().await;
                    available.push_back(browser);

                    let _ = self.event_sender.send(PoolEvent::BrowserCheckedIn {
                        id: browser_id.to_string(),
                    });

                    debug!(browser_id = %browser_id, "Browser checked in to pool");
                }
                _ => {
                    // Clean up unhealthy browser
                    browser.cleanup().await;

                    let _ = self.event_sender.send(PoolEvent::BrowserRemoved {
                        id: browser_id.to_string(),
                        reason: format!("Unhealthy: {:?}", health),
                    });

                    warn!(browser_id = %browser_id, health = ?health, "Browser removed due to health issues");
                }
            }
        } else {
            warn!(browser_id = %browser_id, "Attempted to check in unknown browser");
        }

        Ok(())
    }

    /// Get pool statistics
    pub async fn stats(&self) -> PoolStats {
        let available_count = self.available.lock().await.len();
        let in_use_count = self.in_use.read().await.len();
        let total_capacity = self.config.max_pool_size;

        PoolStats {
            available: available_count,
            in_use: in_use_count,
            total_capacity,
            utilization: (in_use_count as f64 / total_capacity as f64) * 100.0,
        }
    }

    /// Get pool events for monitoring
    pub fn events(&self) -> Arc<Mutex<mpsc::UnboundedReceiver<PoolEvent>>> {
        self.event_receiver.clone()
    }

    /// P1-B2: Fast health checks (liveness only, 2s intervals)
    async fn perform_fast_health_checks(
        available: &Arc<Mutex<VecDeque<PooledBrowser>>>,
        event_sender: &mpsc::UnboundedSender<PoolEvent>,
    ) {
        let mut failed_browsers = vec![];

        // Quick liveness checks on available browsers
        {
            let available_browsers = available.lock().await;
            for (idx, browser) in available_browsers.iter().enumerate() {
                if !browser.fast_health_check().await {
                    debug!(
                        browser_id = %browser.id,
                        "Browser failed fast liveness check"
                    );
                    failed_browsers.push(idx);
                }
            }
        }

        // Remove failed browsers (if any detected)
        if !failed_browsers.is_empty() {
            let mut available_browsers = available.lock().await;
            // Remove in reverse order to maintain indices
            for idx in failed_browsers.into_iter().rev() {
                if let Some(mut browser) = available_browsers.remove(idx) {
                    browser.cleanup().await;

                    let _ = event_sender.send(PoolEvent::BrowserRemoved {
                        id: browser.id.clone(),
                        reason: "Fast health check failed - unresponsive".to_string(),
                    });
                }
            }
        }
    }

    /// P1-B2: Full health checks (comprehensive, 15s intervals)
    async fn perform_full_health_checks(
        config: &BrowserPoolConfig,
        available: &Arc<Mutex<VecDeque<PooledBrowser>>>,
        in_use: &Arc<RwLock<HashMap<String, PooledBrowser>>>,
        event_sender: &mpsc::UnboundedSender<PoolEvent>,
    ) {
        let mut healthy_count = 0;
        let mut unhealthy_count = 0;

        let soft_limit = if config.enable_memory_limits {
            config.memory_soft_limit_mb
        } else {
            config.memory_threshold_mb
        };
        let hard_limit = if config.enable_memory_limits {
            config.memory_hard_limit_mb
        } else {
            config.memory_threshold_mb
        };

        // Full health checks on available browsers
        {
            let mut available_browsers = available.lock().await;
            let mut i = 0;
            while i < available_browsers.len() {
                let health = available_browsers[i]
                    .full_health_check(soft_limit, hard_limit)
                    .await;
                match health {
                    BrowserHealth::Healthy => {
                        healthy_count += 1;
                        i += 1;
                    }
                    _ => {
                        if let Some(mut browser) = available_browsers.remove(i) {
                            browser.cleanup().await;
                            unhealthy_count += 1;

                            let _ = event_sender.send(PoolEvent::BrowserRemoved {
                                id: browser.id.clone(),
                                reason: format!("Full health check failed: {:?}", health),
                            });
                        } else {
                            error!(
                                "Failed to remove browser at index {} during full health check",
                                i
                            );
                        }
                    }
                }
            }
        }

        // Check in-use browsers (read-only, memory alerts only)
        {
            let in_use_browsers = in_use.read().await;
            for browser in in_use_browsers.values() {
                if browser.stats.memory_usage_mb > hard_limit {
                    let _ = event_sender.send(PoolEvent::MemoryAlert {
                        browser_id: browser.id.clone(),
                        memory_mb: browser.stats.memory_usage_mb,
                    });
                }
            }
            healthy_count += in_use_browsers.len();
        }

        let _ = event_sender.send(PoolEvent::HealthCheckCompleted {
            healthy: healthy_count,
            unhealthy: unhealthy_count,
        });

        if unhealthy_count > 0 {
            warn!(
                healthy = healthy_count,
                unhealthy = unhealthy_count,
                "Full health check completed with issues"
            );
        }
    }

    /// QW-3: Memory limit checks (5s intervals)
    async fn perform_memory_checks(
        config: &BrowserPoolConfig,
        available: &Arc<Mutex<VecDeque<PooledBrowser>>>,
        in_use: &Arc<RwLock<HashMap<String, PooledBrowser>>>,
        event_sender: &mpsc::UnboundedSender<PoolEvent>,
    ) {
        let mut over_hard_limit = vec![];

        // Check available browsers for memory violations
        {
            let mut available_browsers = available.lock().await;
            let mut i = 0;
            while i < available_browsers.len() {
                let memory_mb = available_browsers[i].stats.memory_usage_mb;

                if memory_mb > config.memory_hard_limit_mb {
                    // Hard limit: immediate eviction
                    if let Some(mut browser) = available_browsers.remove(i) {
                        error!(
                            browser_id = %browser.id,
                            memory_mb = memory_mb,
                            hard_limit_mb = config.memory_hard_limit_mb,
                            "Browser exceeded hard memory limit - evicting"
                        );

                        browser.cleanup().await;

                        let _ = event_sender.send(PoolEvent::BrowserRemoved {
                            id: browser.id.clone(),
                            reason: format!(
                                "Memory hard limit exceeded: {}MB > {}MB",
                                memory_mb, config.memory_hard_limit_mb
                            ),
                        });
                    }
                } else if memory_mb > config.memory_soft_limit_mb {
                    // Soft limit: warning only
                    warn!(
                        browser_id = %available_browsers[i].id,
                        memory_mb = memory_mb,
                        soft_limit_mb = config.memory_soft_limit_mb,
                        "Browser memory usage high - cleanup recommended"
                    );
                    i += 1;
                } else {
                    i += 1;
                }
            }
        }

        // Check in-use browsers for memory alerts
        {
            let in_use_browsers = in_use.read().await;
            for browser in in_use_browsers.values() {
                let memory_mb = browser.stats.memory_usage_mb;

                if memory_mb > config.memory_hard_limit_mb {
                    over_hard_limit.push((browser.id.clone(), memory_mb));

                    let _ = event_sender.send(PoolEvent::MemoryAlert {
                        browser_id: browser.id.clone(),
                        memory_mb,
                    });
                }
            }
        }

        if !over_hard_limit.is_empty() {
            warn!(
                count = over_hard_limit.len(),
                hard_limit_mb = config.memory_hard_limit_mb,
                "In-use browsers exceeded hard memory limit (will be evicted on checkin)"
            );
        }
    }

    /// Perform health checks on all browsers (legacy method, kept for compatibility)
    async fn perform_health_checks(
        config: &BrowserPoolConfig,
        available: &Arc<Mutex<VecDeque<PooledBrowser>>>,
        in_use: &Arc<RwLock<HashMap<String, PooledBrowser>>>,
        event_sender: &mpsc::UnboundedSender<PoolEvent>,
    ) {
        // Delegate to full health checks if tiered monitoring disabled
        Self::perform_full_health_checks(config, available, in_use, event_sender).await;
    }

    /// Clean up expired browsers from the pool
    async fn cleanup_expired_browsers(
        config: &BrowserPoolConfig,
        available: &Arc<Mutex<VecDeque<PooledBrowser>>>,
        event_sender: &mpsc::UnboundedSender<PoolEvent>,
    ) {
        let mut available_browsers = available.lock().await;
        let mut i = 0;

        while i < available_browsers.len() {
            let browser = &available_browsers[i];
            if browser.is_expired(config.max_lifetime) || browser.is_idle(config.idle_timeout) {
                let Some(mut browser) = available_browsers.remove(i) else {
                    error!(index = i, "Failed to remove expired browser at index");
                    i += 1;
                    continue;
                };
                browser.cleanup().await;

                let reason = if browser.is_expired(config.max_lifetime) {
                    "Expired"
                } else {
                    "Idle timeout"
                };

                let _ = event_sender.send(PoolEvent::BrowserRemoved {
                    id: browser.id.clone(),
                    reason: reason.to_string(),
                });

                debug!(browser_id = %browser.id, reason = reason, "Browser removed from pool");
            } else {
                i += 1;
            }
        }
    }

    /// Maintain minimum pool size by creating new browsers if needed
    /// Uses graceful degradation: continues trying to create browsers even if some fail
    async fn maintain_pool_size(
        config: &BrowserPoolConfig,
        browser_config: &BrowserConfig,
        available: &Arc<Mutex<VecDeque<PooledBrowser>>>,
        event_sender: &mpsc::UnboundedSender<PoolEvent>,
    ) {
        let mut available_browsers = available.lock().await;
        let current_size = available_browsers.len();

        if current_size < config.min_pool_size {
            let needed = config.min_pool_size - current_size;
            debug!(
                current = current_size,
                needed = needed,
                "Expanding pool to maintain minimum size"
            );

            let mut created = 0;
            let mut failed = 0;

            // Try to create all needed browsers with exponential backoff on failures
            for attempt in 0..needed {
                match PooledBrowser::new(browser_config.clone(), config.profile_base_dir.as_deref())
                    .await
                {
                    Ok(browser) => {
                        let _ = event_sender.send(PoolEvent::BrowserCreated {
                            id: browser.id.clone(),
                        });
                        available_browsers.push_back(browser);
                        created += 1;
                    }
                    Err(e) => {
                        failed += 1;
                        warn!(
                            attempt = attempt + 1,
                            error = %e,
                            "Failed to create browser for pool maintenance (continuing with remaining attempts)"
                        );

                        // Add small delay before next attempt to avoid rapid failures
                        if attempt < needed - 1 {
                            tokio::time::sleep(Duration::from_millis(100 * (failed as u64))).await;
                        }
                    }
                }
            }

            if created > 0 {
                let _ = event_sender.send(PoolEvent::PoolExpanded {
                    new_size: available_browsers.len(),
                });

                if failed > 0 {
                    warn!(
                        created = created,
                        failed = failed,
                        current_size = available_browsers.len(),
                        target_size = config.min_pool_size,
                        "Pool maintenance completed with partial success"
                    );
                } else {
                    debug!(
                        created = created,
                        current_size = available_browsers.len(),
                        "Pool maintenance completed successfully"
                    );
                }
            } else if failed > 0 {
                error!(
                    failed = failed,
                    current_size = available_browsers.len(),
                    "Pool maintenance failed to create any new browsers - system may be degraded"
                );
            }
        }
    }

    /// Get current pool statistics
    #[allow(dead_code)] // Method for future use
    pub async fn get_stats(&self) -> PoolStats {
        let available = self.available.lock().await.len();
        let in_use = self.in_use.read().await.len();
        let total_capacity = self.config.max_pool_size;
        let utilization = if total_capacity > 0 {
            in_use as f64 / total_capacity as f64
        } else {
            0.0
        };

        PoolStats {
            available,
            in_use,
            total_capacity,
            utilization,
        }
    }

    /// Shutdown the pool and clean up all resources
    #[allow(dead_code)]
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down browser pool");

        // Signal management task to stop
        let _ = self.shutdown_sender.send(()).await;

        // Clean up all available browsers
        {
            let mut available = self.available.lock().await;
            while let Some(mut browser) = available.pop_front() {
                browser.cleanup().await;
            }
        }

        // Clean up all in-use browsers
        {
            let mut in_use = self.in_use.write().await;
            for (_, mut browser) in in_use.drain() {
                browser.cleanup().await;
            }
        }

        info!("Browser pool shutdown completed");
        Ok(())
    }
}

/// Statistics about the browser pool
#[derive(Debug, Clone)]
#[allow(dead_code)] // Some fields are for future use
pub struct PoolStats {
    pub available: usize,
    pub in_use: usize,
    pub total_capacity: usize,
    pub utilization: f64,
}

/// Reference to the browser pool for checkout operations
/// Uses Arc clones to maintain strong references for safety
#[derive(Clone)]
pub struct BrowserPoolRef {
    available: Arc<Mutex<VecDeque<PooledBrowser>>>,
    in_use: Arc<RwLock<HashMap<String, PooledBrowser>>>,
    config: BrowserPoolConfig,
    event_sender: mpsc::UnboundedSender<PoolEvent>,
}

impl BrowserPoolRef {
    fn new(pool: &BrowserPool) -> Self {
        Self {
            available: Arc::clone(&pool.available),
            in_use: Arc::clone(&pool.in_use),
            config: pool.config.clone(),
            event_sender: pool.event_sender.clone(),
        }
    }

    pub async fn checkin(&self, browser_id: &str) -> Result<()> {
        let mut browser_opt = {
            let mut in_use = self.in_use.write().await;
            in_use.remove(browser_id)
        };

        if let Some(mut browser) = browser_opt.take() {
            browser.in_use = false;

            // Perform health check before returning to pool
            let health = browser.health_check(self.config.memory_threshold_mb).await;

            match health {
                BrowserHealth::Healthy => {
                    // Return healthy browser to pool
                    let mut available = self.available.lock().await;
                    available.push_back(browser);

                    let _ = self.event_sender.send(PoolEvent::BrowserCheckedIn {
                        id: browser_id.to_string(),
                    });

                    debug!(browser_id = %browser_id, "Browser checked in to pool");
                }
                _ => {
                    // Clean up unhealthy browser
                    browser.cleanup().await;

                    let _ = self.event_sender.send(PoolEvent::BrowserRemoved {
                        id: browser_id.to_string(),
                        reason: format!("Unhealthy: {:?}", health),
                    });

                    debug!(browser_id = %browser_id, health = ?health, "Removed unhealthy browser");
                }
            }

            Ok(())
        } else {
            Err(anyhow!("Browser not found in pool"))
        }
    }
}

/// A checked-out browser instance with automatic checkin on drop
pub struct BrowserCheckout {
    browser_id: String,
    pool: BrowserPoolRef,
    permit: Option<tokio::sync::OwnedSemaphorePermit>,
}

impl BrowserCheckout {
    /// Get the browser ID
    #[allow(dead_code)] // Method for future use
    pub fn browser_id(&self) -> &str {
        &self.browser_id
    }

    /// Create a new page in the browser
    #[allow(dead_code)] // Method for future use
    pub async fn new_page(&self, url: &str) -> Result<Page> {
        let in_use = self.pool.in_use.read().await;
        if let Some(pooled_browser) = in_use.get(&self.browser_id) {
            let page = pooled_browser.browser.new_page(url).await?;
            Ok(page)
        } else {
            Err(anyhow!("Browser not found in pool"))
        }
    }

    /// Manually check in the browser (consumes the checkout, preferred over drop)
    #[allow(dead_code)]
    pub async fn checkin(mut self) -> Result<()> {
        let result = self.pool.checkin(&self.browser_id).await;
        // Prevent drop from trying to checkin again
        self.permit.take();
        result
    }

    /// Cleanup with timeout - ensures proper async cleanup
    ///
    /// Uses the configured cleanup_timeout from BrowserPoolConfig.
    /// If you need a custom timeout, use cleanup_with_timeout() instead.
    pub async fn cleanup(mut self) -> Result<()> {
        let timeout_duration = self.pool.config.cleanup_timeout;
        let _ = tokio::time::timeout(timeout_duration, self.pool.checkin(&self.browser_id))
            .await
            .map_err(|_| {
                anyhow!(
                    "Timeout checking in browser {} after {:?}",
                    self.browser_id,
                    timeout_duration
                )
            })?;

        // Prevent drop from trying to checkin again
        self.permit.take();
        Ok(())
    }

    /// Cleanup with custom timeout - for cases where you need a different timeout
    #[allow(dead_code)] // Public API for custom timeout scenarios
    pub async fn cleanup_with_timeout(mut self, timeout_duration: Duration) -> Result<()> {
        let _ = tokio::time::timeout(timeout_duration, self.pool.checkin(&self.browser_id))
            .await
            .map_err(|_| {
                anyhow!(
                    "Timeout checking in browser {} after {:?}",
                    self.browser_id,
                    timeout_duration
                )
            })?;

        // Prevent drop from trying to checkin again
        self.permit.take();
        Ok(())
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    // spider_chrome exports its types as the chromiumoxide module for compatibility
    use spider_chrome::BrowserConfig;

    #[tokio::test]
    async fn test_browser_pool_creation() {
        let config = BrowserPoolConfig {
            initial_pool_size: 2,
            ..Default::default()
        };

        let browser_config = BrowserConfig::builder()
            .build()
            .expect("Failed to build browser config");

        let pool = BrowserPool::new(config, browser_config).await;
        assert!(pool.is_ok());

        if let Ok(pool) = pool {
            let stats = pool.stats().await;
            assert_eq!(stats.available, 2);
            assert_eq!(stats.in_use, 0);

            let _ = pool.shutdown().await;
        }
    }

    #[tokio::test]
    async fn test_browser_checkout_checkin() {
        let config = BrowserPoolConfig {
            initial_pool_size: 1,
            ..Default::default()
        };

        let browser_config = BrowserConfig::builder()
            .build()
            .expect("Failed to build browser config");

        let pool = BrowserPool::new(config, browser_config).await.unwrap();

        // Checkout browser
        let checkout = pool.checkout().await.unwrap();
        let stats = pool.stats().await;
        assert_eq!(stats.available, 0);
        assert_eq!(stats.in_use, 1);

        // Checkin browser explicitly
        checkout.checkin().await.unwrap();

        // Give a moment for async checkin to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let stats = pool.stats().await;
        assert_eq!(stats.available, 1);
        assert_eq!(stats.in_use, 0);

        // Shutdown pool properly
        pool.shutdown().await.unwrap();
    }
}
