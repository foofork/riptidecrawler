//! BusinessMetrics integration for BrowserFacade
//!
//! This module extends BrowserFacade with business metrics capabilities.

use super::browser::{BrowserFacade, BrowserSession, ScreenshotOptions};
use crate::error::RiptideResult;
use crate::metrics::BusinessMetrics;
use std::sync::Arc;

/// Wrapper for BrowserFacade with integrated metrics
pub struct MetricsBrowserFacade {
    facade: Arc<BrowserFacade>,
    metrics: Arc<BusinessMetrics>,
}

impl MetricsBrowserFacade {
    /// Create a new metrics-enabled browser facade
    pub fn new(facade: BrowserFacade, metrics: Arc<BusinessMetrics>) -> Self {
        Self {
            facade: Arc::new(facade),
            metrics,
        }
    }

    /// Launch a browser session (automatically records metrics)
    pub async fn launch(&self) -> RiptideResult<BrowserSession<'_>> {
        let result = self.facade.launch().await;

        // Record metrics
        if result.is_ok() {
            self.metrics.record_session_created();
        }

        result
    }

    /// Navigate to a URL (automatically records metrics)
    pub async fn navigate(&self, session: &BrowserSession<'_>, url: &str) -> RiptideResult<()> {
        self.metrics.record_browser_action();
        self.facade.navigate(session, url).await
    }

    /// Take a screenshot (automatically records metrics)
    pub async fn screenshot(
        &self,
        session: &BrowserSession<'_>,
        options: ScreenshotOptions,
    ) -> RiptideResult<Vec<u8>> {
        self.metrics.record_browser_action();
        self.metrics.record_screenshot_taken();
        self.facade.screenshot(session, options).await
    }

    /// Execute JavaScript (automatically records metrics)
    pub async fn execute_script(
        &self,
        session: &BrowserSession<'_>,
        script: &str,
    ) -> RiptideResult<serde_json::Value> {
        self.metrics.record_browser_action();
        self.facade.execute_script(session, script).await
    }

    /// Close a browser session (automatically records metrics)
    pub async fn close(&self, session: BrowserSession<'_>) -> RiptideResult<()> {
        let result = self.facade.close(session).await;

        // Record metrics
        if result.is_ok() {
            self.metrics.record_session_closed();
        }

        result
    }

    /// Get reference to underlying facade
    pub fn facade(&self) -> &BrowserFacade {
        &self.facade
    }
}
