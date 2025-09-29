use anyhow::{anyhow, Result};
use chromiumoxide::{Browser, BrowserConfig, Page};
use futures::StreamExt;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};
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
}

impl Default for BrowserPoolConfig {
    fn default() -> Self {
        Self {
            min_pool_size: 1,
            max_pool_size: 5,
            initial_pool_size: 3,
            idle_timeout: Duration::from_secs(30),
            max_lifetime: Duration::from_secs(300), // 5 minutes
            health_check_interval: Duration::from_secs(10),
            memory_threshold_mb: 500,
            enable_recovery: true,
            max_retries: 3,
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
#[derive(Debug)]
pub struct PooledBrowser {
    pub id: String,
    pub browser: Browser,
    pub created_at: Instant,
    pub last_used: Instant,
    pub stats: BrowserStats,
    pub health: BrowserHealth,
    pub in_use: bool,
    handler_task: tokio::task::JoinHandle<()>,
}

impl PooledBrowser {
    pub async fn new(browser_config: BrowserConfig) -> Result<Self> {
        let id = Uuid::new_v4().to_string();

        debug!(browser_id = %id, "Creating new browser instance");

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

        // Create initial browser instances
        let mut initial_browsers = VecDeque::new();
        for _ in 0..config.initial_pool_size {
            match PooledBrowser::new(browser_config.clone()).await {
                Ok(browser) => {
                    let _ = event_sender.send(PoolEvent::BrowserCreated {
                        id: browser.id.clone(),
                    });
                    initial_browsers.push_back(browser);
                }
                Err(e) => {
                    error!(error = %e, "Failed to create initial browser instance");
                }
            }
        }

        *available.lock().await = initial_browsers;

        // Start management task for health checks and cleanup
        let management_task = {
            let config = config.clone();
            let browser_config = browser_config.clone();
            let available = available.clone();
            let in_use = in_use.clone();
            let event_sender = event_sender.clone();

            tokio::spawn(async move {
                let mut health_check_interval = interval(config.health_check_interval);

                loop {
                    tokio::select! {
                        _ = health_check_interval.tick() => {
                            Self::perform_health_checks(
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
            match PooledBrowser::new(self.browser_config.clone()).await {
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

    /// Perform health checks on all browsers
    async fn perform_health_checks(
        config: &BrowserPoolConfig,
        available: &Arc<Mutex<VecDeque<PooledBrowser>>>,
        in_use: &Arc<RwLock<HashMap<String, PooledBrowser>>>,
        event_sender: &mpsc::UnboundedSender<PoolEvent>,
    ) {
        let mut healthy_count = 0;
        let mut unhealthy_count = 0;

        // Check available browsers
        {
            let mut available_browsers = available.lock().await;
            let mut i = 0;
            while i < available_browsers.len() {
                let health = available_browsers[i]
                    .health_check(config.memory_threshold_mb)
                    .await;
                match health {
                    BrowserHealth::Healthy => {
                        healthy_count += 1;
                        i += 1;
                    }
                    _ => {
                        let mut browser = available_browsers.remove(i).unwrap();
                        browser.cleanup().await;
                        unhealthy_count += 1;

                        let _ = event_sender.send(PoolEvent::BrowserRemoved {
                            id: browser.id.clone(),
                            reason: format!("Health check failed: {:?}", health),
                        });
                    }
                }
            }
        }

        // Check in-use browsers (read-only health check)
        {
            let in_use_browsers = in_use.read().await;
            for browser in in_use_browsers.values() {
                if browser.stats.memory_usage_mb > config.memory_threshold_mb {
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
                "Browser health check completed"
            );
        }
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
                let mut browser = available_browsers.remove(i).unwrap();
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

            for _ in 0..needed {
                match PooledBrowser::new(browser_config.clone()).await {
                    Ok(browser) => {
                        let _ = event_sender.send(PoolEvent::BrowserCreated {
                            id: browser.id.clone(),
                        });
                        available_browsers.push_back(browser);
                    }
                    Err(e) => {
                        error!(error = %e, "Failed to create browser for pool maintenance");
                        break;
                    }
                }
            }

            let _ = event_sender.send(PoolEvent::PoolExpanded {
                new_size: available_browsers.len(),
            });
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
pub struct BrowserPoolRef {
    pool: Weak<BrowserPool>,
}

impl BrowserPoolRef {
    fn new(pool: &BrowserPool) -> Self {
        Self {
            pool: Arc::downgrade(&Arc::new(unsafe {
                std::ptr::read(pool as *const BrowserPool)
            })),
        }
    }

    pub async fn checkin(&self, browser_id: &str) -> Result<()> {
        if let Some(pool) = self.pool.upgrade() {
            pool.checkin(browser_id).await
        } else {
            Err(anyhow!("Browser pool has been dropped"))
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
        if let Some(pool) = self.pool.pool.upgrade() {
            let in_use = pool.in_use.read().await;
            if let Some(pooled_browser) = in_use.get(&self.browser_id) {
                let page = pooled_browser.browser.new_page(url).await?;
                Ok(page)
            } else {
                Err(anyhow!("Browser not found in pool"))
            }
        } else {
            Err(anyhow!("Browser pool has been dropped"))
        }
    }

    /// Manually check in the browser (consumes the checkout)
    pub async fn checkin(mut self) -> Result<()> {
        let result = self.pool.checkin(&self.browser_id).await;
        // Prevent drop from trying to checkin again
        self.permit.take();
        result
    }
}

impl Drop for BrowserCheckout {
    fn drop(&mut self) {
        if self.permit.is_some() {
            let browser_id = self.browser_id.clone();
            let pool = self.pool.pool.clone();

            // Spawn a task to handle async checkin during drop
            tokio::spawn(async move {
                if let Some(pool) = pool.upgrade() {
                    if let Err(e) = pool.checkin(&browser_id).await {
                        error!(browser_id = %browser_id, error = %e, "Failed to checkin browser during drop");
                    }
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chromiumoxide::BrowserConfig;

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

        // Checkin browser
        checkout.checkin().await.unwrap();
        let stats = pool.stats().await;
        assert_eq!(stats.available, 1);
        assert_eq!(stats.in_use, 0);

        let _ = pool.shutdown().await;
    }
}
