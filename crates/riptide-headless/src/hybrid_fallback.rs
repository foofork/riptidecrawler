//! Hybrid browser fallback: spider-chrome with chromiumoxide fallback
//!
//! This module implements a 20% traffic split to spider-chrome with automatic
//! fallback to chromiumoxide when spider-chrome fails.

use anyhow::{Context, Result};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

#[cfg(feature = "headless")]
use spider_chromiumoxide_cdp::{Browser as ChromiumBrowser, Page as ChromiumPage};

/// Fallback metrics for monitoring spider-chrome adoption
#[derive(Debug, Clone, Default)]
pub struct FallbackMetrics {
    pub spider_chrome_attempts: u64,
    pub spider_chrome_success: u64,
    pub spider_chrome_failures: u64,
    pub chromiumoxide_fallbacks: u64,
    pub chromiumoxide_success: u64,
    pub chromiumoxide_failures: u64,
}

/// Response from browser automation with metadata
pub struct BrowserResponse {
    pub html: String,
    pub screenshot: Option<Vec<u8>>,
    pub pdf: Option<Vec<u8>>,
    pub engine: BrowserEngine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserEngine {
    SpiderChrome,
    Chromiumoxide,
}

/// Hybrid browser fallback coordinator
pub struct HybridBrowserFallback {
    metrics: Arc<RwLock<FallbackMetrics>>,
    spider_chrome_traffic_pct: u8,
    #[cfg(feature = "headless")]
    spider_chrome_launcher: Option<Arc<riptide_headless_hybrid::HybridHeadlessLauncher>>,
}

impl HybridBrowserFallback {
    /// Create new hybrid fallback with 20% spider-chrome traffic
    pub async fn new() -> Result<Self> {
        Self::with_traffic_percentage(20).await
    }

    /// Create hybrid fallback with custom traffic percentage
    pub async fn with_traffic_percentage(spider_chrome_pct: u8) -> Result<Self> {
        if spider_chrome_pct > 100 {
            return Err(anyhow::anyhow!(
                "Traffic percentage must be 0-100, got {}",
                spider_chrome_pct
            ));
        }

        #[cfg(feature = "headless")]
        let spider_chrome_launcher = if spider_chrome_pct > 0 {
            match riptide_headless_hybrid::HybridHeadlessLauncher::new().await {
                Ok(launcher) => {
                    info!(
                        traffic_pct = spider_chrome_pct,
                        "Spider-chrome launcher initialized"
                    );
                    Some(Arc::new(launcher))
                }
                Err(e) => {
                    warn!(
                        error = %e,
                        "Failed to initialize spider-chrome launcher, disabling"
                    );
                    None
                }
            }
        } else {
            None
        };

        #[cfg(not(feature = "headless"))]
        let spider_chrome_launcher = None;

        Ok(Self {
            metrics: Arc::new(RwLock::new(FallbackMetrics::default())),
            spider_chrome_traffic_pct: spider_chrome_pct,
            #[cfg(feature = "headless")]
            spider_chrome_launcher,
        })
    }

    /// Execute page load with fallback logic
    #[cfg(feature = "headless")]
    pub async fn execute_with_fallback(
        &self,
        url: &str,
        chromium_page: &ChromiumPage,
    ) -> Result<BrowserResponse> {
        // Determine which engine to use
        if self.should_use_spider_chrome(url) {
            debug!(url = %url, "Attempting spider-chrome");

            // Update metrics
            {
                let mut metrics = self.metrics.write().await;
                metrics.spider_chrome_attempts += 1;
            }

            // Try spider-chrome first
            match self.try_spider_chrome(url).await {
                Ok(response) => {
                    // Success with spider-chrome
                    {
                        let mut metrics = self.metrics.write().await;
                        metrics.spider_chrome_success += 1;
                    }

                    info!(
                        url = %url,
                        engine = "spider-chrome",
                        "Page loaded successfully"
                    );
                    return Ok(response);
                }
                Err(e) => {
                    // Spider-chrome failed, fall back to chromiumoxide
                    warn!(
                        url = %url,
                        error = %e,
                        "Spider-chrome failed, falling back to chromiumoxide"
                    );

                    {
                        let mut metrics = self.metrics.write().await;
                        metrics.spider_chrome_failures += 1;
                        metrics.chromiumoxide_fallbacks += 1;
                    }
                }
            }
        }

        // Use chromiumoxide (either as fallback or primary)
        self.try_chromiumoxide(url, chromium_page).await
    }

    /// Execute with chromiumoxide only (for testing/comparison)
    #[cfg(feature = "headless")]
    pub async fn execute_chromiumoxide_only(
        &self,
        url: &str,
        chromium_page: &ChromiumPage,
    ) -> Result<BrowserResponse> {
        self.try_chromiumoxide(url, chromium_page).await
    }

    /// Determine if URL should use spider-chrome (based on hash)
    fn should_use_spider_chrome(&self, url: &str) -> bool {
        // If spider-chrome is disabled or not available, return false
        #[cfg(feature = "headless")]
        if self.spider_chrome_launcher.is_none() || self.spider_chrome_traffic_pct == 0 {
            return false;
        }

        #[cfg(not(feature = "headless"))]
        return false;

        // Hash-based traffic splitting for consistent routing
        #[cfg(feature = "headless")]
        {
            let mut hasher = DefaultHasher::new();
            url.hash(&mut hasher);
            let hash_value = hasher.finish();
            (hash_value % 100) < self.spider_chrome_traffic_pct as u64
        }
    }

    /// Try spider-chrome execution
    #[cfg(feature = "headless")]
    async fn try_spider_chrome(&self, url: &str) -> Result<BrowserResponse> {
        let launcher = self
            .spider_chrome_launcher
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Spider-chrome launcher not available"))?;

        // Launch page with stealth
        let session = launcher
            .launch_page_default(url)
            .await
            .context("Failed to launch spider-chrome page")?;

        // Get HTML content
        let html = session.content().await.context("Failed to get HTML")?;

        // Try to get screenshot (optional, non-critical)
        let screenshot = session.screenshot().await.ok();

        // Try to get PDF (optional, non-critical)
        let pdf = session.pdf().await.ok();

        // Clean up
        let _ = session.close().await;

        Ok(BrowserResponse {
            html,
            screenshot,
            pdf,
            engine: BrowserEngine::SpiderChrome,
        })
    }

    /// Try chromiumoxide execution
    #[cfg(feature = "headless")]
    async fn try_chromiumoxide(
        &self,
        url: &str,
        page: &ChromiumPage,
    ) -> Result<BrowserResponse> {
        // Navigate to URL
        page.goto(url)
            .await
            .context("Failed to navigate with chromiumoxide")?;

        // Wait for navigation
        page.wait_for_navigation()
            .await
            .context("Navigation timeout")?;

        // Get HTML content
        let html = page
            .content()
            .await
            .context("Failed to get page content")?
            .unwrap_or_default();

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.chromiumoxide_success += 1;
        }

        info!(
            url = %url,
            engine = "chromiumoxide",
            "Page loaded successfully"
        );

        Ok(BrowserResponse {
            html,
            screenshot: None,
            pdf: None,
            engine: BrowserEngine::Chromiumoxide,
        })
    }

    /// Get current fallback metrics
    pub async fn metrics(&self) -> FallbackMetrics {
        self.metrics.read().await.clone()
    }

    /// Get spider-chrome success rate (0.0 to 1.0)
    pub async fn spider_chrome_success_rate(&self) -> f64 {
        let metrics = self.metrics.read().await;
        if metrics.spider_chrome_attempts == 0 {
            return 0.0;
        }
        metrics.spider_chrome_success as f64 / metrics.spider_chrome_attempts as f64
    }

    /// Get chromiumoxide fallback rate (0.0 to 1.0)
    pub async fn fallback_rate(&self) -> f64 {
        let metrics = self.metrics.read().await;
        if metrics.spider_chrome_attempts == 0 {
            return 0.0;
        }
        metrics.chromiumoxide_fallbacks as f64 / metrics.spider_chrome_attempts as f64
    }
}

#[cfg(all(test, feature = "headless"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fallback_creation() {
        let fallback = HybridBrowserFallback::new().await;
        assert!(fallback.is_ok());
    }

    #[tokio::test]
    async fn test_traffic_percentage() {
        let fallback = HybridBrowserFallback::with_traffic_percentage(50)
            .await
            .unwrap();

        // Test URL hashing for traffic split
        let test_urls: Vec<&str> = (0..100)
            .map(|i| match i {
                _ => "https://example.com",
            })
            .collect();

        let spider_chrome_count = test_urls
            .iter()
            .filter(|url| fallback.should_use_spider_chrome(url))
            .count();

        // Should be roughly 50% (allow some variance)
        assert!(
            spider_chrome_count >= 30 && spider_chrome_count <= 70,
            "Expected ~50 spider-chrome URLs, got {}",
            spider_chrome_count
        );
    }

    #[tokio::test]
    async fn test_metrics_tracking() {
        let fallback = HybridBrowserFallback::new().await.unwrap();
        let metrics = fallback.metrics().await;

        assert_eq!(metrics.spider_chrome_attempts, 0);
        assert_eq!(metrics.spider_chrome_success, 0);
        assert_eq!(metrics.chromiumoxide_fallbacks, 0);
    }
}
