//! Chromiumoxide engine implementation (using spider_chromiumoxide_cdp)

use async_trait::async_trait;
use chromiumoxide_cdp::{Browser, Page};
use std::sync::Arc;
use tracing::{debug, warn};

use crate::{
    params::{NavigateParams, PdfParams, ScreenshotParams, WaitUntil},
    traits::{BrowserEngine, EngineType, PageHandle},
    AbstractionError, AbstractionResult,
};

/// Chromiumoxide engine wrapper
pub struct ChromiumoxideEngine {
    browser: Arc<Browser>,
}

impl ChromiumoxideEngine {
    pub fn new(browser: Browser) -> Self {
        Self {
            browser: Arc::new(browser),
        }
    }
}

#[async_trait]
impl BrowserEngine for ChromiumoxideEngine {
    async fn new_page(&self) -> AbstractionResult<Box<dyn PageHandle>> {
        debug!("Creating new page with chromiumoxide");
        let page = self
            .browser
            .new_page("about:blank")
            .await
            .map_err(|e| AbstractionError::PageCreation(e.to_string()))?;

        Ok(Box::new(ChromiumoxidePage::new(page)))
    }

    fn engine_type(&self) -> EngineType {
        EngineType::Chromiumoxide
    }

    async fn close(&self) -> AbstractionResult<()> {
        debug!("Closing chromiumoxide browser (no-op through Arc - browser will close on drop)");
        // chromiumoxide Browser.close() requires &mut self
        // Since we're using Arc<Browser> for thread safety, we can't call it
        // The browser will be cleaned up when all Arc references are dropped
        warn!("explicit browser close not supported through Arc - browser will close on drop");
        Ok(())
    }

    async fn version(&self) -> AbstractionResult<String> {
        Ok(self
            .browser
            .version()
            .await
            .map_err(|e| AbstractionError::Other(e.to_string()))?
            .product)
    }
}

/// Chromiumoxide page wrapper
pub struct ChromiumoxidePage {
    page: Page,
}

impl ChromiumoxidePage {
    pub fn new(page: Page) -> Self {
        Self { page }
    }
}

#[async_trait]
impl PageHandle for ChromiumoxidePage {
    async fn goto(&self, url: &str, params: NavigateParams) -> AbstractionResult<()> {
        debug!("Navigating to {} with chromiumoxide", url);

        self.page
            .goto(url)
            .await
            .map_err(|e| AbstractionError::Navigation(e.to_string()))?;

        // Apply wait conditions
        match params.wait_until {
            WaitUntil::Load => {
                // Default behavior in chromiumoxide
            }
            WaitUntil::DOMContentLoaded => {
                warn!("DOMContentLoaded not explicitly supported in chromiumoxide");
            }
            WaitUntil::NetworkIdle => {
                warn!("NetworkIdle not explicitly supported in chromiumoxide");
            }
        }

        Ok(())
    }

    async fn content(&self) -> AbstractionResult<String> {
        self.page
            .content()
            .await
            .map_err(|e| AbstractionError::ContentRetrieval(e.to_string()))
    }

    async fn url(&self) -> AbstractionResult<String> {
        Ok(self
            .page
            .url()
            .await
            .map_err(|e| AbstractionError::Other(e.to_string()))?
            .unwrap_or_default())
    }

    async fn evaluate(&self, script: &str) -> AbstractionResult<serde_json::Value> {
        let result = self
            .page
            .evaluate(script)
            .await
            .map_err(|e| AbstractionError::Evaluation(e.to_string()))?;

        result
            .into_value()
            .map_err(|e| AbstractionError::Evaluation(e.to_string()))
    }

    async fn screenshot(&self, _params: ScreenshotParams) -> AbstractionResult<Vec<u8>> {
        debug!("Taking screenshot with chromiumoxide");

        // chromiumoxide_cdp 0.7.4 has limited screenshot parameter support
        // The builder methods are private, so we use defaults
        self.page
            .screenshot(chromiumoxide_cdp::page::ScreenshotParams::default())
            .await
            .map_err(|e| AbstractionError::Screenshot(e.to_string()))
    }

    async fn pdf(&self, _params: PdfParams) -> AbstractionResult<Vec<u8>> {
        debug!("Generating PDF with chromiumoxide");

        // Note: chromiumoxide 0.7.0 has limited public PDF support
        // The PrintToPdfParams type is private, so we call pdf() with default params
        self.page
            .pdf(Default::default())
            .await
            .map_err(|e| AbstractionError::PdfGeneration(e.to_string()))
    }

    async fn wait_for_navigation(&self, timeout_ms: u64) -> AbstractionResult<()> {
        debug!(
            "Waiting for navigation with chromiumoxide ({}ms)",
            timeout_ms
        );

        self.page
            .wait_for_navigation()
            .await
            .map_err(|e| AbstractionError::Navigation(e.to_string()))?;

        Ok(())
    }

    async fn set_timeout(&self, _timeout_ms: u64) -> AbstractionResult<()> {
        debug!(
            "Setting timeout to {}ms (not supported on borrowed Page)",
            _timeout_ms
        );

        // chromiumoxide Page.set_default_timeout requires mutable reference
        // Since PageHandle is &self, we can't call it
        // This is a known limitation of the abstraction
        warn!("set_timeout not supported with chromiumoxide through trait abstraction");
        Ok(())
    }

    async fn close(&self) -> AbstractionResult<()> {
        debug!("Closing chromiumoxide page");

        // chromiumoxide Page.close() takes ownership (self)
        // Since PageHandle is &self, we can't call it
        // Thisis a known limitation - pages will be cleaned up when dropped
        warn!("explicit close not supported with chromiumoxide through trait abstraction");
        Ok(())
    }
}
