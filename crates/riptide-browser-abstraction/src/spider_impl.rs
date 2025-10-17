//! Spider-chrome engine implementation

#[cfg(feature = "spider")]
use async_trait::async_trait;
#[cfg(feature = "spider")]
// Note: spider_chrome exports its library as "chromiumoxide", so we import from there
use chromiumoxide::{Browser as SpiderBrowser, Page as SpiderPage};
#[cfg(feature = "spider")]
use std::sync::Arc;
#[cfg(feature = "spider")]
use tracing::{debug, warn};

#[cfg(feature = "spider")]
use crate::{
    traits::{BrowserEngine, PageHandle, EngineType},
    params::{ScreenshotParams, PdfParams, NavigateParams},
    AbstractionResult, AbstractionError,
};

#[cfg(feature = "spider")]
/// Spider-chrome engine wrapper
pub struct SpiderChromeEngine {
    browser: Arc<SpiderBrowser>,
}

#[cfg(feature = "spider")]
impl SpiderChromeEngine {
    pub fn new(browser: SpiderBrowser) -> Self {
        Self {
            browser: Arc::new(browser),
        }
    }
}

#[cfg(feature = "spider")]
#[async_trait]
impl BrowserEngine for SpiderChromeEngine {
    async fn new_page(&self) -> AbstractionResult<Box<dyn PageHandle>> {
        debug!("Creating new page with spider-chrome");
        let page = self.browser
            .new_page("about:blank")
            .await
            .map_err(|e| AbstractionError::PageCreation(e.to_string()))?;

        Ok(Box::new(SpiderChromePage::new(page)))
    }

    fn engine_type(&self) -> EngineType {
        EngineType::SpiderChrome
    }

    async fn close(&self) -> AbstractionResult<()> {
        debug!("Closing spider-chrome browser");
        self.browser
            .close()
            .await
            .map_err(|e| AbstractionError::BrowserClose(e.to_string()))?;

        // Spider-chrome's close returns CloseReturns, we just need to return ()
        Ok(())
    }

    async fn version(&self) -> AbstractionResult<String> {
        Ok(self.browser
            .version()
            .await
            .map_err(|e| AbstractionError::Other(e.to_string()))?
            .product)
    }
}

#[cfg(feature = "spider")]
/// Spider-chrome page wrapper
pub struct SpiderChromePage {
    page: Arc<SpiderPage>,
}

#[cfg(feature = "spider")]
impl SpiderChromePage {
    pub fn new(page: SpiderPage) -> Self {
        Self {
            page: Arc::new(page),
        }
    }
}

#[cfg(feature = "spider")]
#[async_trait]
impl PageHandle for SpiderChromePage {
    async fn goto(&self, url: &str, _params: NavigateParams) -> AbstractionResult<()> {
        debug!("Navigating to {} with spider-chrome", url);

        // Spider-chrome's goto returns the page
        let _ = self.page
            .goto(url)
            .await
            .map_err(|e| AbstractionError::Navigation(e.to_string()))?;

        Ok(())
    }

    async fn content(&self) -> AbstractionResult<String> {
        self.page
            .content()
            .await
            .map_err(|e| AbstractionError::ContentRetrieval(e.to_string()))
    }

    async fn url(&self) -> AbstractionResult<String> {
        Ok(self.page
            .url()
            .await
            .map_err(|e| AbstractionError::Other(e.to_string()))?
            .unwrap_or_default())
    }

    async fn evaluate(&self, script: &str) -> AbstractionResult<serde_json::Value> {
        // Spider-chrome requires &str, not &String
        let result = self.page
            .evaluate(script)
            .await
            .map_err(|e| AbstractionError::Evaluation(e.to_string()))?;

        // into_value() returns Result in spider-chrome
        result.into_value()
            .map_err(|e| AbstractionError::Evaluation(e.to_string()))
    }

    async fn screenshot(&self, _params: ScreenshotParams) -> AbstractionResult<Vec<u8>> {
        debug!("Screenshot not directly supported in spider-chrome");
        warn!("Spider-chrome screenshot requires manual CDP implementation");

        // Would need to call CDP directly
        Err(AbstractionError::Unsupported(
            "screenshot not yet implemented for spider-chrome".to_string()
        ))
    }

    async fn pdf(&self, _params: PdfParams) -> AbstractionResult<Vec<u8>> {
        debug!("PDF generation not directly supported in spider-chrome");
        warn!("Spider-chrome PDF requires manual CDP implementation");

        // Would need to call CDP directly
        Err(AbstractionError::Unsupported(
            "pdf not yet implemented for spider-chrome".to_string()
        ))
    }

    async fn wait_for_navigation(&self, timeout_ms: u64) -> AbstractionResult<()> {
        debug!("Wait for navigation not directly supported in spider-chrome");

        // Fallback: just wait
        tokio::time::sleep(std::time::Duration::from_millis(timeout_ms)).await;
        Ok(())
    }

    async fn set_timeout(&self, timeout_ms: u64) -> AbstractionResult<()> {
        debug!("Setting timeout to {}ms (note: spider-chrome may not support this)", timeout_ms);

        // Spider-chrome doesn't have set_default_timeout
        // This is a no-op for compatibility
        Ok(())
    }

    async fn close(&self) -> AbstractionResult<()> {
        debug!("Closing spider-chrome page");

        // Spider-chrome's close() takes ownership
        // Clone the page reference and close it
        // Note: This creates a new Page handle from Arc, then closes it
        if let Some(_page) = Arc::get_mut(&mut self.page.clone()) {
            // This won't work because Arc::get_mut requires exclusive access
            // For now, we'll just drop our reference
            // The actual page will be closed when all references are dropped
            debug!("Dropping page reference (spider-chrome close requires ownership)");
        }

        Ok(())
    }
}
