//! Chromiumoxide engine implementation (using spider_chrome)
//!
//! Note: spider_chrome package exports crate name as "chromiumoxide"
//! This module contains CONCRETE CDP implementations.

use async_trait::async_trait;
use chromiumoxide::{Browser, Page};
use std::sync::Arc;
use tracing::{debug, warn};

use crate::abstraction::{
    AbstractionError, AbstractionResult, BrowserEngine, EngineType, NavigateParams, PageHandle,
    PdfParams, ScreenshotParams, WaitUntil,
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

        match params.wait_until {
            WaitUntil::Load => {}
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

        self.page
            .screenshot(chromiumoxide::page::ScreenshotParams::default())
            .await
            .map_err(|e| AbstractionError::Screenshot(e.to_string()))
    }

    async fn pdf(&self, _params: PdfParams) -> AbstractionResult<Vec<u8>> {
        debug!("Generating PDF with chromiumoxide");

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

        warn!("set_timeout not supported with chromiumoxide through trait abstraction");
        Ok(())
    }

    async fn close(&self) -> AbstractionResult<()> {
        debug!("Closing chromiumoxide page");

        warn!("explicit close not supported with chromiumoxide through trait abstraction");
        Ok(())
    }
}
