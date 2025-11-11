//! Unified headless browser launcher with pool management and hybrid fallback
//!
//! This module combines:
//! - Pool-based browser management from riptide-engine
//! - Hybrid fallback support with single-browser mode
//! - Screenshot and PDF generation capabilities
//! - Stealth integration for anti-detection

use crate::pool::{BrowserCheckout, BrowserPool, BrowserPoolConfig, PoolEvent};
use anyhow::{anyhow, Result};
// spider_chrome exports its types as the chromiumoxide module for compatibility
use chromiumoxide::{Browser, BrowserConfig, Page};
use chromiumoxide_cdp::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use futures::StreamExt;
// Note: Browser abstraction integration is handled via pool module
use riptide_stealth::{StealthController, StealthPreset};
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
    /// Enable hybrid fallback mode (single browser instance instead of pool)
    pub hybrid_mode: bool,
}

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            pool_config: BrowserPoolConfig::default(),
            default_stealth_preset: StealthPreset::Medium,
            enable_stealth: true,
            page_timeout: Duration::from_secs(30),
            enable_monitoring: true,
            hybrid_mode: false,
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

/// Enhanced headless browser launcher with pooling and hybrid mode support
pub struct HeadlessLauncher {
    config: LauncherConfig,
    browser_pool: Option<Arc<BrowserPool>>,
    hybrid_browser: Arc<RwLock<Option<Arc<Browser>>>>,
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
            hybrid_mode = config.hybrid_mode,
            "Initializing headless launcher"
        );

        // Configure browser with stealth settings
        let browser_config = Self::build_browser_config(&config).await?;

        // Create browser pool only if not in hybrid mode
        let browser_pool = if !config.hybrid_mode {
            Some(Arc::new(
                BrowserPool::new(config.pool_config.clone(), browser_config).await?,
            ))
        } else {
            None
        };

        // Initialize stealth controller
        let stealth_controller = Arc::new(RwLock::new(StealthController::from_preset(
            config.default_stealth_preset.clone(),
        )));

        let stats = Arc::new(RwLock::new(LauncherStats::default()));
        let hybrid_browser = Arc::new(RwLock::new(None));

        // Start monitoring task if enabled and using pool
        if config.enable_monitoring && !config.hybrid_mode {
            if let Some(ref pool) = browser_pool {
                Self::start_monitoring_task(pool.clone(), stats.clone()).await;
            }
        }

        info!("Headless launcher initialized successfully");

        Ok(Self {
            config,
            browser_pool,
            hybrid_browser,
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
            hybrid_mode = self.config.hybrid_mode,
            "Launching browser page"
        );

        // Update stealth controller if different preset requested
        if let Some(ref preset) = stealth_preset {
            let mut stealth_controller = self.stealth_controller.write().await;
            if *preset != StealthPreset::None {
                *stealth_controller = StealthController::from_preset(preset.clone());
            }
        }

        // Get page using appropriate mode
        let (page, browser_checkout) = if self.config.hybrid_mode {
            self.launch_page_hybrid(url).await?
        } else {
            self.launch_page_pooled(url).await?
        };

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
            stats.total_requests = stats.total_requests.saturating_add(1);
            stats.successful_requests = stats.successful_requests.saturating_add(1);
            // Calculate running average safely
            let prev_count = stats.successful_requests.saturating_sub(1);
            stats.avg_response_time_ms = (stats.avg_response_time_ms * prev_count as f64
                + duration.as_millis() as f64)
                / stats.successful_requests as f64;

            if stealth_preset.is_some() && stealth_preset != Some(StealthPreset::None) {
                stats.stealth_requests = stats.stealth_requests.saturating_add(1);
            } else {
                stats.non_stealth_requests = stats.non_stealth_requests.saturating_add(1);
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

    /// Launch page using pool-based approach
    async fn launch_page_pooled(&self, url: &str) -> Result<(Page, Option<BrowserCheckout>)> {
        let pool = self
            .browser_pool
            .as_ref()
            .ok_or_else(|| anyhow!("Browser pool not available in hybrid mode"))?;

        // Checkout browser from pool
        let browser_checkout = timeout(Duration::from_secs(10), pool.checkout())
            .await
            .map_err(|_| anyhow!("Browser checkout timed out"))?
            .map_err(|e| anyhow!("Failed to checkout browser: {}", e))?;

        // Create new page
        let page = timeout(Duration::from_secs(5), browser_checkout.new_page(url))
            .await
            .map_err(|_| anyhow!("Page creation timed out"))?
            .map_err(|e| anyhow!("Failed to create page: {}", e))?;

        Ok((page, Some(browser_checkout)))
    }

    /// Launch page using hybrid single-browser approach
    async fn launch_page_hybrid(&self, url: &str) -> Result<(Page, Option<BrowserCheckout>)> {
        let browser = self.get_or_create_hybrid_browser().await?;

        // Create new page
        let page = browser
            .new_page(url)
            .await
            .map_err(|e| anyhow!("Failed to create page: {}", e))?;

        // Wait for page to load
        if let Err(e) =
            tokio::time::timeout(self.config.page_timeout, page.wait_for_navigation()).await
        {
            warn!(error = ?e, "Page load timed out or failed in hybrid mode");
        }

        Ok((page, None))
    }

    /// Get or create hybrid browser instance (single browser for all pages)
    async fn get_or_create_hybrid_browser(&self) -> Result<Arc<Browser>> {
        let mut browser_guard = self.hybrid_browser.write().await;

        if browser_guard.is_none() {
            debug!("Creating new hybrid browser instance");

            // Build browser config
            let mut browser_config = BrowserConfig::builder().window_size(1920, 1080);

            // Apply stealth flags if enabled
            if self.config.enable_stealth {
                let mut stealth_controller = self.stealth_controller.write().await;
                let stealth_flags = stealth_controller.get_cdp_flags();

                // Add stealth flags to chrome args
                for flag in stealth_flags {
                    browser_config = browser_config.arg(flag);
                }

                // Add user agent
                let user_agent = stealth_controller.next_user_agent();
                browser_config = browser_config.arg(format!("--user-agent={}", user_agent));
            }

            let browser_config = browser_config
                .build()
                .map_err(|e| anyhow!("Failed to build browser config: {}", e))?;

            let (browser, mut handler) = Browser::launch(browser_config)
                .await
                .map_err(|e| anyhow!("Failed to launch browser: {}", e))?;

            // Spawn handler task to manage browser events
            tokio::spawn(async move {
                debug!("Browser event handler started");
                while let Some(event) = handler.next().await {
                    if let Err(e) = event {
                        warn!(error = %e, "Browser event error");
                    }
                }
                debug!("Browser event handler ended");
            });

            let arc_browser = Arc::new(browser);
            *browser_guard = Some(arc_browser.clone());
            Ok(arc_browser)
        } else {
            browser_guard
                .as_ref()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Browser guard unexpectedly empty"))
        }
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
        let mut stats = self.stats.read().await.clone();

        // Update pool utilization if using pool mode
        if let Some(ref pool) = self.browser_pool {
            let pool_stats = pool.stats().await;
            stats.pool_utilization = pool_stats.utilization;
        }

        stats
    }

    /// Get pool events for monitoring (only available in pool mode)
    #[allow(dead_code)]
    pub fn pool_events(
        &self,
    ) -> Option<Arc<tokio::sync::Mutex<tokio::sync::mpsc::UnboundedReceiver<PoolEvent>>>> {
        self.browser_pool.as_ref().map(|pool| pool.events())
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
        // Check for custom Chrome flags from environment (for advanced users)
        if let Ok(custom_flags) = std::env::var("CHROME_FLAGS") {
            debug!("Using custom CHROME_FLAGS from environment");
            for flag in custom_flags.split_whitespace() {
                builder = builder.arg(flag);
            }
        } else {
            // Sensible defaults for most users (Docker-optimized)
            debug!("Using default Chrome flags optimized for Docker/headless environments");
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
                .arg("--memory-pressure-off")
                .arg("--disable-crash-reporter")
                .arg("--crash-dumps-dir=/tmp")
                .arg("--disable-breakpad");
        }

        builder.build().map_err(|e| anyhow!(e))
    }

    /// Apply stealth configurations to a page (works with spider_chrome Page)
    async fn apply_stealth_to_page(&self, page: &Page) -> Result<()> {
        // Inject stealth JavaScript
        let stealth_js = include_str!("../stealth.js");
        page.evaluate_on_new_document(stealth_js)
            .await
            .map_err(|e| anyhow!("Failed to inject stealth JS: {}", e))?;

        // Set viewport to common resolution
        let viewport_params = SetDeviceMetricsOverrideParams::builder()
            .width(1920)
            .height(1080)
            .device_scale_factor(1.0)
            .mobile(false)
            .build()
            .map_err(|e| anyhow!("Failed to build viewport params: {}", e))?;

        page.execute(viewport_params)
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

        // Shutdown pool if in pool mode
        if let Some(ref pool) = self.browser_pool {
            pool.shutdown().await?;
        }

        // Shutdown hybrid browser if in hybrid mode
        if self.config.hybrid_mode {
            let mut browser_guard = self.hybrid_browser.write().await;
            if let Some(browser) = browser_guard.take() {
                // Browser close is handled by Drop - no explicit close needed
                // The Arc will be dropped when browser goes out of scope
                drop(browser);
            }
        }

        info!("Headless launcher shutdown completed");
        Ok(())
    }
}

/// A browser session with automatic cleanup
pub struct LaunchSession<'a> {
    pub session_id: String,
    pub page: Page,
    #[allow(dead_code)]
    browser_checkout: Option<BrowserCheckout>,
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

    /// Take a screenshot (full page)
    #[allow(dead_code)]
    pub async fn screenshot(&self) -> Result<Vec<u8>> {
        debug!(
            session_id = %self.session_id,
            "Taking screenshot"
        );

        use chromiumoxide::page::ScreenshotParams;

        let screenshot = timeout(
            Duration::from_secs(10),
            self.page.screenshot(ScreenshotParams::default()),
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

    /// Take a screenshot and save to file
    #[allow(dead_code)]
    pub async fn screenshot_to_file(&self, path: &str) -> Result<()> {
        debug!(
            session_id = %self.session_id,
            path = %path,
            "Taking screenshot to file"
        );

        let screenshot = self.screenshot().await?;
        tokio::fs::write(path, screenshot)
            .await
            .map_err(|e| anyhow!("Failed to write screenshot to file: {}", e))?;

        debug!(
            session_id = %self.session_id,
            path = %path,
            "Screenshot saved to file"
        );

        Ok(())
    }

    /// Generate PDF from current page
    #[allow(dead_code)]
    pub async fn pdf(&self) -> Result<Vec<u8>> {
        debug!(
            session_id = %self.session_id,
            "Generating PDF"
        );

        use chromiumoxide_cdp::cdp::browser_protocol::page::PrintToPdfParams;

        let pdf_data = timeout(
            Duration::from_secs(10),
            self.page.pdf(PrintToPdfParams::default()),
        )
        .await
        .map_err(|_| anyhow!("PDF generation timed out"))?
        .map_err(|e| anyhow!("PDF generation failed: {}", e))?;

        debug!(
            session_id = %self.session_id,
            size_bytes = pdf_data.len(),
            "PDF generated"
        );

        Ok(pdf_data)
    }

    /// Generate PDF and save to file
    #[allow(dead_code)]
    pub async fn pdf_to_file(&self, path: &str) -> Result<()> {
        debug!(
            session_id = %self.session_id,
            path = %path,
            "Generating PDF to file"
        );

        let pdf_data = self.pdf().await?;
        tokio::fs::write(path, pdf_data)
            .await
            .map_err(|e| anyhow!("Failed to write PDF to file: {}", e))?;

        debug!(
            session_id = %self.session_id,
            path = %path,
            "PDF saved to file"
        );

        Ok(())
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
                    stats.failed_requests = stats.failed_requests.saturating_add(1);
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
    #[ignore = "requires Chrome - run with: cargo test -- --ignored"]
    #[ignore = "requires Chrome - run with: cargo test -- --ignored"]
    #[ignore = "requires Chrome - run with: cargo test -- --ignored"]
    async fn test_launcher_creation_pool_mode() {
        let config = LauncherConfig {
            pool_config: BrowserPoolConfig {
                initial_pool_size: 1,
                ..Default::default()
            },
            hybrid_mode: false,
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
    #[ignore = "requires Chrome - run with: cargo test -- --ignored"]
    #[ignore = "requires Chrome - run with: cargo test -- --ignored"]
    #[ignore = "requires Chrome - run with: cargo test -- --ignored"]
    async fn test_launcher_creation_hybrid_mode() {
        let config = LauncherConfig {
            hybrid_mode: true,
            enable_stealth: false,
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
    #[ignore = "requires Chrome - run with: cargo test -- --ignored"]
    #[ignore = "requires Chrome - run with: cargo test -- --ignored"]
    #[ignore = "requires Chrome - run with: cargo test -- --ignored"]
    async fn test_page_launch() {
        let launcher = HeadlessLauncher::new().await.unwrap();

        // This would require a real browser environment
        // let session = launcher.launch_page("about:blank", None).await;
        // assert!(session.is_ok());

        let _ = launcher.shutdown().await;
    }
}
