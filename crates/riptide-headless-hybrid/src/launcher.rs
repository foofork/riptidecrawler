//! Hybrid headless browser launcher using spider-chrome with EventMesh stealth

use crate::models::PoolConfig;
use crate::stealth_middleware::apply_stealth;
use anyhow::{ Context, Result};
use chromiumoxide::Browser;
use riptide_stealth::{StealthController, StealthPreset};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Configuration for the hybrid headless launcher
#[derive(Clone, Debug)]
pub struct LauncherConfig {
    /// Browser pool configuration
    pub pool_config: PoolConfig,
    /// Default stealth preset
    pub default_stealth_preset: StealthPreset,
    /// Enable stealth by default
    pub enable_stealth: bool,
    /// Page navigation timeout
    pub page_timeout: Duration,
    /// Enable performance monitoring
    pub enable_monitoring: bool,
}

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            pool_config: PoolConfig::default(),
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
    pub stealth_requests: u64,
    pub non_stealth_requests: u64,
}

/// Hybrid headless launcher combining spider-chrome with EventMesh stealth
pub struct HybridHeadlessLauncher {
    config: LauncherConfig,
    stealth_controller: Arc<RwLock<StealthController>>,
    stats: Arc<RwLock<LauncherStats>>,
    browser: Arc<RwLock<Option<Browser>>>,
}

impl HybridHeadlessLauncher {
    /// Create a new hybrid launcher with default configuration
    pub async fn new() -> Result<Self> {
        Self::with_config(LauncherConfig::default()).await
    }

    /// Create a new hybrid launcher with custom configuration
    pub async fn with_config(config: LauncherConfig) -> Result<Self> {
        info!(
            stealth_enabled = config.enable_stealth,
            preset = ?config.default_stealth_preset,
            "Initializing hybrid headless launcher"
        );

        // Initialize stealth controller
        let stealth_controller = Arc::new(RwLock::new(StealthController::from_preset(
            config.default_stealth_preset.clone(),
        )));

        let stats = Arc::new(RwLock::new(LauncherStats::default()));
        let browser = Arc::new(RwLock::new(None));

        info!("Hybrid headless launcher initialized successfully");

        Ok(Self {
            config,
            stealth_controller,
            stats,
            browser,
        })
    }

    /// Launch a browser page with specified URL and stealth configuration
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
            if *preset != StealthPreset::None {
                let mut stealth_controller = self.stealth_controller.write().await;
                *stealth_controller = StealthController::from_preset(preset.clone());
            }
        }

        // Get or create browser instance
        let browser = self.get_or_create_browser().await?;

        // Create new page with spider-chrome
        let page = browser
            .new_page(url)
            .await
            .context("Failed to create new page")?;

        // Apply stealth configurations if enabled
        if self.config.enable_stealth && stealth_preset != Some(StealthPreset::None) {
            let stealth_controller = self.stealth_controller.read().await;
            if let Err(e) = apply_stealth(&page, &*stealth_controller).await {
                warn!(
                    session_id = %session_id,
                    error = %e,
                    "Failed to apply stealth configurations (non-critical)"
                );
            }
        }

        // Wait for page to load
        if let Err(e) = tokio::time::timeout(
            self.config.page_timeout,
            page.wait_for_navigation(),
        )
        .await
        {
            warn!(
                session_id = %session_id,
                error = ?e,
                "Page load timed out or failed"
            );
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
            start_time,
            launcher: self,
        })
    }

    /// Launch a browser page with default stealth settings
    pub async fn launch_page_default<'a>(&'a self, url: &str) -> Result<LaunchSession<'a>> {
        let stealth_preset = if self.config.enable_stealth {
            Some(self.config.default_stealth_preset.clone())
        } else {
            Some(StealthPreset::None)
        };

        self.launch_page(url, stealth_preset).await
    }

    /// Launch a browser page without stealth (for testing/debugging)
    pub async fn launch_page_no_stealth<'a>(&'a self, url: &str) -> Result<LaunchSession<'a>> {
        self.launch_page(url, Some(StealthPreset::None)).await
    }

    /// Get launcher statistics
    pub async fn stats(&self) -> LauncherStats {
        self.stats.read().await.clone()
    }

    /// Get or create browser instance
    async fn get_or_create_browser(&self) -> Result<Browser> {
        let mut browser_guard = self.browser.write().await;

        if browser_guard.is_none() {
            debug!("Creating new browser instance");

            // Build launch options
            let mut launch_options = chromiumoxide::LaunchOptions::default_builder()
                .headless(true)
                .build()
                .context("Failed to build launch options")?;

            // Apply stealth flags if enabled
            if self.config.enable_stealth {
                let stealth_controller = self.stealth_controller.read().await;
                let stealth_flags = stealth_controller.get_cdp_flags();

                // Add stealth flags to chrome args
                for flag in stealth_flags {
                    launch_options.args.push(flag);
                }

                // Add user agent
                let user_agent = stealth_controller.next_user_agent();
                launch_options.args.push(format!("--user-agent={}", user_agent));
            }

            let browser = Browser::launch(launch_options)
                .await
                .context("Failed to launch browser")?;

            *browser_guard = Some(browser.clone());
            Ok(browser)
        } else {
            Ok(browser_guard.as_ref().unwrap().clone())
        }
    }

    /// Shutdown the launcher and clean up resources
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down hybrid headless launcher");

        let mut browser_guard = self.browser.write().await;
        if let Some(browser) = browser_guard.take() {
            browser.close().await.context("Failed to close browser")?;
        }

        info!("Hybrid headless launcher shutdown completed");
        Ok(())
    }
}

impl Drop for HybridHeadlessLauncher {
    fn drop(&mut self) {
        debug!("HybridHeadlessLauncher dropped");
    }
}

/// A browser session with automatic cleanup
pub struct LaunchSession<'a> {
    pub session_id: String,
    pub page: chromiumoxide::Page,
    start_time: Instant,
    launcher: &'a HybridHeadlessLauncher,
}

impl<'a> LaunchSession<'a> {
    /// Get the session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Get session duration
    pub fn duration(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Navigate to a new URL
    pub async fn navigate(&self, url: &str) -> Result<()> {
        debug!(
            session_id = %self.session_id,
            url = %url,
            "Navigating to new URL"
        );

        self.page
            .goto(url)
            .await
            .context("Failed to navigate to URL")?;

        self.page
            .wait_for_navigation()
            .await
            .context("Failed to wait for navigation")?;

        debug!(
            session_id = %self.session_id,
            url = %url,
            "Navigation completed"
        );

        Ok(())
    }

    /// Wait for a specific element
    pub async fn wait_for_element(&self, selector: &str, timeout_ms: Option<u64>) -> Result<()> {
        let timeout_duration = Duration::from_millis(timeout_ms.unwrap_or(5000));

        debug!(
            session_id = %self.session_id,
            selector = %selector,
            timeout_ms = timeout_duration.as_millis(),
            "Waiting for element"
        );

        tokio::time::timeout(timeout_duration, async {
            self.page
                .wait_for_selector(selector)
                .await
                .context("Element not found")
        })
        .await
        .context("Element wait timed out")??;

        debug!(
            session_id = %self.session_id,
            selector = %selector,
            "Element found"
        );

        Ok(())
    }

    /// Execute JavaScript on the page
    pub async fn execute_script(&self, script: &str) -> Result<serde_json::Value> {
        debug!(
            session_id = %self.session_id,
            script_length = script.len(),
            "Executing JavaScript"
        );

        let result = self
            .page
            .evaluate(script)
            .await
            .context("Failed to execute script")?;

        debug!(
            session_id = %self.session_id,
            "JavaScript executed successfully"
        );

        Ok(result)
    }

    /// Take a screenshot (full page)
    pub async fn screenshot(&self) -> Result<Vec<u8>> {
        debug!(
            session_id = %self.session_id,
            "Taking full page screenshot"
        );

        let screenshot = self
            .page
            .screenshot()
            .await
            .context("Failed to take screenshot")?;

        debug!(
            session_id = %self.session_id,
            size_bytes = screenshot.len(),
            "Screenshot captured"
        );

        Ok(screenshot)
    }

    /// Take a screenshot and save to file
    pub async fn screenshot_to_file(&self, path: &str) -> Result<()> {
        debug!(
            session_id = %self.session_id,
            path = %path,
            "Taking screenshot to file"
        );

        let screenshot = self.screenshot().await?;
        tokio::fs::write(path, screenshot)
            .await
            .context("Failed to write screenshot to file")?;

        debug!(
            session_id = %self.session_id,
            path = %path,
            "Screenshot saved to file"
        );

        Ok(())
    }

    /// Generate PDF from current page
    pub async fn pdf(&self) -> Result<Vec<u8>> {
        debug!(
            session_id = %self.session_id,
            "Generating PDF"
        );

        let pdf_data = self
            .page
            .pdf()
            .await
            .context("Failed to generate PDF")?;

        debug!(
            session_id = %self.session_id,
            size_bytes = pdf_data.len(),
            "PDF generated"
        );

        Ok(pdf_data)
    }

    /// Generate PDF and save to file
    pub async fn pdf_to_file(&self, path: &str) -> Result<()> {
        debug!(
            session_id = %self.session_id,
            path = %path,
            "Generating PDF to file"
        );

        let pdf_data = self.pdf().await?;
        tokio::fs::write(path, pdf_data)
            .await
            .context("Failed to write PDF to file")?;

        debug!(
            session_id = %self.session_id,
            path = %path,
            "PDF saved to file"
        );

        Ok(())
    }

    /// Get page content (HTML)
    pub async fn content(&self) -> Result<String> {
        debug!(
            session_id = %self.session_id,
            "Getting page content"
        );

        let html = self
            .page
            .content()
            .await
            .context("Failed to get page content")?;

        debug!(
            session_id = %self.session_id,
            content_length = html.len(),
            "Page content retrieved"
        );

        Ok(html)
    }

    /// Manually close the session (automatic on drop)
    pub async fn close(self) -> Result<()> {
        debug!(
            session_id = %self.session_id,
            duration_ms = self.duration().as_millis(),
            "Closing launch session"
        );

        self.page
            .close()
            .await
            .context("Failed to close page")?;

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
            enable_stealth: false,
            ..Default::default()
        };

        let launcher = HybridHeadlessLauncher::with_config(config).await;
        assert!(launcher.is_ok());

        if let Ok(launcher) = launcher {
            let stats = launcher.stats().await;
            assert_eq!(stats.total_requests, 0);

            let _ = launcher.shutdown().await;
        }
    }

    #[tokio::test]
    async fn test_stats_tracking() {
        let launcher = HybridHeadlessLauncher::new().await.unwrap();
        let initial_stats = launcher.stats().await;
        assert_eq!(initial_stats.total_requests, 0);
        let _ = launcher.shutdown().await;
    }
}
