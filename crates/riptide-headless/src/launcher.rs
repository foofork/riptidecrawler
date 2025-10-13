//! Headless browser launcher with pool management (WIP - scaffolding)
#![cfg_attr(not(feature = "headless"), allow(dead_code, unused))]

use crate::pool::{BrowserCheckout, BrowserPool, BrowserPoolConfig, PoolEvent};
use anyhow::{anyhow, Result};
use chromiumoxide::{
    cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams, BrowserConfig, Page,
};
use riptide_core::stealth::{StealthController, StealthPreset};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::timeout;
use tracing::{debug, info, warn};

/// Configuration for the headless browser launcher
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct LauncherConfig {
    /// Browser pool configuration
    pub pool_config: BrowserPoolConfig,
    /// Default stealth preset to use
    pub default_stealth_preset: StealthPreset,
    /// Enable stealth mode by default
    pub enable_stealth: bool,
    /// Page timeout for operations
    pub page_timeout: Duration,
    /// Enable performance monitoring
    pub enable_monitoring: bool,
}

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            pool_config: BrowserPoolConfig::default(),
            default_stealth_preset: StealthPreset::Medium,
            enable_stealth: true,
            page_timeout: Duration::from_secs(30),
            enable_monitoring: true,
        }
    }
}

/// Statistics for launcher operations
#[derive(Clone, Debug, Default)]
pub struct LauncherStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_response_time_ms: f64,
    pub pool_utilization: f64,
    pub stealth_requests: u64,
    pub non_stealth_requests: u64,
}

/// Enhanced headless browser launcher with pooling and optimization
pub struct HeadlessLauncher {
    config: LauncherConfig,
    browser_pool: Arc<BrowserPool>,
    stealth_controller: Arc<RwLock<StealthController>>,
    stats: Arc<RwLock<LauncherStats>>,
}

impl HeadlessLauncher {
    /// Create a new headless launcher with default configuration
    pub async fn new() -> Result<Self> {
        Self::with_config(LauncherConfig::default()).await
    }

    /// Create a new headless launcher with custom configuration
    pub async fn with_config(config: LauncherConfig) -> Result<Self> {
        info!(
            pool_min = config.pool_config.min_pool_size,
            pool_max = config.pool_config.max_pool_size,
            stealth_enabled = config.enable_stealth,
            "Initializing headless launcher"
        );

        // Configure browser with stealth settings
        let browser_config = Self::build_browser_config(&config).await?;

        // Create browser pool
        let browser_pool =
            Arc::new(BrowserPool::new(config.pool_config.clone(), browser_config).await?);

        // Initialize stealth controller
        let stealth_controller = Arc::new(RwLock::new(StealthController::from_preset(
            config.default_stealth_preset.clone(),
        )));

        let stats = Arc::new(RwLock::new(LauncherStats::default()));

        // Start monitoring task if enabled
        if config.enable_monitoring {
            Self::start_monitoring_task(browser_pool.clone(), stats.clone()).await;
        }

        info!("Headless launcher initialized successfully");

        Ok(Self {
            config,
            browser_pool,
            stealth_controller,
            stats,
        })
    }

    /// Launch a browser page with the specified URL and stealth configuration
    pub async fn launch_page<'a>(
        &'a self,
        url: &str,
        stealth_preset: Option<StealthPreset>,
    ) -> Result<LaunchSession<'a>> {
        let start_time = Instant::now();
        let session_id = uuid::Uuid::new_v4().to_string();

        debug!(
            session_id = %session_id,
            url = %url,
            stealth_preset = ?stealth_preset,
            "Launching browser page"
        );

        // Update stealth controller if different preset requested
        if let Some(ref preset) = stealth_preset {
            let mut stealth_controller = self.stealth_controller.write().await;
            if *preset != StealthPreset::None {
                *stealth_controller = StealthController::from_preset(preset.clone());
            }
        }

        // Checkout browser from pool
        let browser_checkout = timeout(Duration::from_secs(10), self.browser_pool.checkout())
            .await
            .map_err(|_| anyhow!("Browser checkout timed out"))?
            .map_err(|e| anyhow!("Failed to checkout browser: {}", e))?;

        // Create new page
        let page = timeout(Duration::from_secs(5), browser_checkout.new_page(url))
            .await
            .map_err(|_| anyhow!("Page creation timed out"))?
            .map_err(|e| anyhow!("Failed to create page: {}", e))?;

        // Apply stealth configurations if enabled
        if self.config.enable_stealth && stealth_preset != Some(StealthPreset::None) {
            if let Err(e) = self.apply_stealth_to_page(&page).await {
                warn!(
                    session_id = %session_id,
                    error = %e,
                    "Failed to apply stealth configurations (non-critical)"
                );
            }
        }

        let duration = start_time.elapsed();

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.total_requests += 1;
            stats.successful_requests += 1;
            stats.avg_response_time_ms = (stats.avg_response_time_ms
                * (stats.successful_requests - 1) as f64
                + duration.as_millis() as f64)
                / stats.successful_requests as f64;

            if stealth_preset.is_some() && stealth_preset != Some(StealthPreset::None) {
                stats.stealth_requests += 1;
            } else {
                stats.non_stealth_requests += 1;
            }
        }

        info!(
            session_id = %session_id,
            url = %url,
            duration_ms = duration.as_millis(),
            "Browser page launched successfully"
        );

        Ok(LaunchSession {
            session_id,
            page,
            browser_checkout,
            start_time,
            launcher: self,
        })
    }

    /// Launch a browser page with default stealth settings
    #[allow(dead_code)]
    pub async fn launch_page_default<'a>(&'a self, url: &str) -> Result<LaunchSession<'a>> {
        let stealth_preset = if self.config.enable_stealth {
            Some(self.config.default_stealth_preset.clone())
        } else {
            Some(StealthPreset::None)
        };

        self.launch_page(url, stealth_preset).await
    }

    /// Launch a browser page without stealth (for testing/debugging)
    #[allow(dead_code)]
    pub async fn launch_page_no_stealth<'a>(&'a self, url: &str) -> Result<LaunchSession<'a>> {
        self.launch_page(url, Some(StealthPreset::None)).await
    }

    /// Get launcher statistics
    pub async fn stats(&self) -> LauncherStats {
        let pool_stats = self.browser_pool.stats().await;
        let mut stats = self.stats.read().await.clone();
        stats.pool_utilization = pool_stats.utilization;
        stats
    }

    /// Get pool events for monitoring
    #[allow(dead_code)]
    pub fn pool_events(
        &self,
    ) -> Arc<tokio::sync::Mutex<tokio::sync::mpsc::UnboundedReceiver<PoolEvent>>> {
        self.browser_pool.events()
    }

    /// Build browser configuration with stealth settings
    /// Note: Unique profile directories are added per-browser in pool.rs to prevent SingletonLock errors
    async fn build_browser_config(config: &LauncherConfig) -> Result<BrowserConfig> {
        let mut builder = BrowserConfig::builder();

        if config.enable_stealth && config.default_stealth_preset != StealthPreset::None {
            let mut stealth_controller =
                StealthController::from_preset(config.default_stealth_preset.clone());

            // Apply stealth flags
            let stealth_flags = stealth_controller.get_cdp_flags();
            for flag in stealth_flags {
                builder = builder.arg(&flag);
            }

            // Set stealth user agent
            let user_agent = stealth_controller.next_user_agent();
            builder = builder.arg(format!("--user-agent={}", user_agent));

            debug!(
                preset = ?config.default_stealth_preset,
                user_agent = %user_agent,
                "Applied stealth configuration to browser"
            );
        } else {
            // Non-stealth configuration for debugging
            builder = builder.with_head();
            debug!("Browser configured without stealth mode");
        }

        // Performance and security optimizations
        builder = builder
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
            .arg("--memory-pressure-off");

        builder.build().map_err(|e| anyhow!(e))
    }

    /// Apply stealth configurations to a page
    async fn apply_stealth_to_page(&self, page: &Page) -> Result<()> {
        // Inject stealth JavaScript
        let stealth_js = include_str!("stealth.js");
        page.evaluate_on_new_document(stealth_js)
            .await
            .map_err(|e| anyhow!("Failed to inject stealth JS: {}", e))?;

        // Set viewport to common resolution
        // Note: stealth_controller is configured at browser launch time via get_cdp_flags()
        page.execute(
            SetDeviceMetricsOverrideParams::builder()
                .width(1920)
                .height(1080)
                .device_scale_factor(1.0)
                .mobile(false)
                .build()
                .unwrap(),
        )
        .await
        .map_err(|e| anyhow!("Failed to set viewport: {}", e))?;

        // Override navigator properties
        let override_script = r#"
            Object.defineProperty(navigator, 'webdriver', {
                get: () => undefined,
            });
            Object.defineProperty(navigator, 'plugins', {
                get: () => [{
                    name: 'Chrome PDF Plugin',
                    description: 'Portable Document Format',
                    filename: 'internal-pdf-viewer'
                }],
            });
            Object.defineProperty(navigator, 'languages', {
                get: () => ['en-US', 'en'],
            });
            "#
        .to_string();

        page.evaluate(&*override_script)
            .await
            .map_err(|e| anyhow!("Failed to apply navigator overrides: {}", e))?;

        debug!("Stealth configurations applied to page");
        Ok(())
    }

    /// Start monitoring task for pool and performance metrics
    async fn start_monitoring_task(
        browser_pool: Arc<BrowserPool>,
        _stats: Arc<RwLock<LauncherStats>>,
    ) {
        let pool_events = browser_pool.events();

        tokio::spawn(async move {
            let mut events = pool_events.lock().await;

            while let Some(event) = events.recv().await {
                match event {
                    PoolEvent::BrowserCreated { id } => {
                        debug!(browser_id = %id, "Browser created in pool");
                    }
                    PoolEvent::BrowserRemoved { id, reason } => {
                        debug!(browser_id = %id, reason = %reason, "Browser removed from pool");
                    }
                    PoolEvent::MemoryAlert {
                        browser_id,
                        memory_mb,
                    } => {
                        warn!(
                            browser_id = %browser_id,
                            memory_mb = memory_mb,
                            "Browser memory alert"
                        );
                    }
                    PoolEvent::HealthCheckCompleted { healthy, unhealthy } => {
                        if unhealthy > 0 {
                            warn!(
                                healthy = healthy,
                                unhealthy = unhealthy,
                                "Browser health check completed with issues"
                            );
                        }
                    }
                    _ => {
                        debug!(event = ?event, "Pool event received");
                    }
                }
            }
        });
    }

    /// Shutdown the launcher and clean up resources
    #[allow(dead_code)]
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down headless launcher");
        self.browser_pool.shutdown().await?;
        info!("Headless launcher shutdown completed");
        Ok(())
    }
}

/// A browser session with automatic cleanup
pub struct LaunchSession<'a> {
    pub session_id: String,
    pub page: Page,
    #[allow(dead_code)]
    browser_checkout: BrowserCheckout,
    start_time: Instant,
    launcher: &'a HeadlessLauncher,
}

impl<'a> LaunchSession<'a> {
    /// Get the session ID
    #[allow(dead_code)]
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Get the page instance
    pub fn page(&self) -> &Page {
        &self.page
    }

    /// Get session duration
    pub fn duration(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Navigate to a new URL
    #[allow(dead_code)]
    pub async fn navigate(&self, url: &str) -> Result<()> {
        debug!(
            session_id = %self.session_id,
            url = %url,
            "Navigating to new URL"
        );

        timeout(self.launcher.config.page_timeout, self.page.goto(url))
            .await
            .map_err(|_| anyhow!("Navigation timed out"))?
            .map_err(|e| anyhow!("Navigation failed: {}", e))?;

        debug!(
            session_id = %self.session_id,
            url = %url,
            "Navigation completed"
        );

        Ok(())
    }

    /// Wait for a specific element
    #[allow(dead_code)]
    pub async fn wait_for_element(&self, selector: &str, timeout_ms: Option<u64>) -> Result<()> {
        let timeout_duration = Duration::from_millis(timeout_ms.unwrap_or(5000));

        debug!(
            session_id = %self.session_id,
            selector = %selector,
            timeout_ms = timeout_duration.as_millis(),
            "Waiting for element"
        );

        timeout(timeout_duration, self.page.find_element(selector))
            .await
            .map_err(|_| anyhow!("Element wait timed out: {}", selector))?
            .map_err(|e| anyhow!("Element not found: {}", e))?;

        debug!(
            session_id = %self.session_id,
            selector = %selector,
            "Element found"
        );

        Ok(())
    }

    /// Execute JavaScript on the page
    #[allow(dead_code)]
    pub async fn execute_script(&self, script: &str) -> Result<serde_json::Value> {
        debug!(
            session_id = %self.session_id,
            script_length = script.len(),
            "Executing JavaScript"
        );

        let result = timeout(Duration::from_secs(10), self.page.evaluate(script))
            .await
            .map_err(|_| anyhow!("Script execution timed out"))?
            .map_err(|e| anyhow!("Script execution failed: {}", e))?;

        let value = result
            .into_value()
            .map_err(|e| anyhow!("Failed to parse script result: {}", e))?;

        debug!(
            session_id = %self.session_id,
            "JavaScript executed successfully"
        );

        Ok(value)
    }

    /// Take a screenshot
    #[allow(dead_code)]
    pub async fn screenshot(&self) -> Result<Vec<u8>> {
        debug!(
            session_id = %self.session_id,
            "Taking screenshot"
        );

        let screenshot = timeout(
            Duration::from_secs(10),
            self.page
                .screenshot(chromiumoxide::page::ScreenshotParams::default()),
        )
        .await
        .map_err(|_| anyhow!("Screenshot timed out"))?
        .map_err(|e| anyhow!("Screenshot failed: {}", e))?;

        debug!(
            session_id = %self.session_id,
            size_bytes = screenshot.len(),
            "Screenshot captured"
        );

        Ok(screenshot)
    }

    /// Get page content (HTML)
    #[allow(dead_code)]
    pub async fn content(&self) -> Result<String> {
        debug!(
            session_id = %self.session_id,
            "Getting page content"
        );

        let html = timeout(Duration::from_secs(5), self.page.content())
            .await
            .map_err(|_| anyhow!("Content extraction timed out"))?
            .map_err(|e| anyhow!("Content extraction failed: {}", e))?;

        debug!(
            session_id = %self.session_id,
            content_length = html.len(),
            "Page content retrieved"
        );

        Ok(html)
    }

    /// Manually close the session (automatic on drop)
    #[allow(dead_code)]
    pub async fn close(self) -> Result<()> {
        debug!(
            session_id = %self.session_id,
            duration_ms = self.duration().as_millis(),
            "Closing launch session"
        );

        // The browser checkout will be automatically returned to pool when dropped
        Ok(())
    }
}

impl<'a> Drop for LaunchSession<'a> {
    fn drop(&mut self) {
        // Update failure statistics if session was short-lived (potential error)
        let duration = self.duration();
        if duration < Duration::from_millis(500) {
            tokio::spawn({
                let launcher_stats = self.launcher.stats.clone();
                async move {
                    let mut stats = launcher_stats.write().await;
                    stats.failed_requests += 1;
                }
            });
        }

        debug!(
            session_id = %self.session_id,
            duration_ms = duration.as_millis(),
            "Launch session dropped"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_launcher_creation() {
        let config = LauncherConfig {
            pool_config: BrowserPoolConfig {
                initial_pool_size: 1,
                ..Default::default()
            },
            ..Default::default()
        };

        let launcher = HeadlessLauncher::with_config(config).await;
        assert!(launcher.is_ok());

        if let Ok(launcher) = launcher {
            let stats = launcher.stats().await;
            assert_eq!(stats.total_requests, 0);

            let _ = launcher.shutdown().await;
        }
    }

    #[tokio::test]
    async fn test_page_launch() {
        let launcher = HeadlessLauncher::new().await.unwrap();

        // This would require a real browser environment
        // let session = launcher.launch_page("about:blank", None).await;
        // assert!(session.is_ok());

        let _ = launcher.shutdown().await;
    }
}
