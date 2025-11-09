//! Spider-chrome native engine implementation
//!
//! This module provides a direct implementation using spider_chrome's native API.
//! This module contains CONCRETE CDP implementations.

use async_trait::async_trait;
use chromiumoxide::{Browser as SpiderBrowser, Page as SpiderPage};
use std::sync::Arc;
use tracing::{debug, warn};

use crate::abstraction::{
    AbstractionError, AbstractionResult, BrowserEngine, EngineType, NavigateParams, PageHandle,
    PdfParams, ScreenshotParams,
};

/// Spider-chrome engine wrapper
pub struct SpiderChromeEngine {
    browser: Arc<SpiderBrowser>,
}

impl SpiderChromeEngine {
    pub fn new(browser: SpiderBrowser) -> Self {
        Self {
            browser: Arc::new(browser),
        }
    }
}

#[async_trait]
impl BrowserEngine for SpiderChromeEngine {
    async fn new_page(&self) -> AbstractionResult<Box<dyn PageHandle>> {
        debug!("Creating new page with spider-chrome");
        let page = self
            .browser
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

/// Spider-chrome page wrapper
pub struct SpiderChromePage {
    page: Arc<SpiderPage>,
}

impl SpiderChromePage {
    pub fn new(page: SpiderPage) -> Self {
        Self {
            page: Arc::new(page),
        }
    }
}

#[async_trait]
impl PageHandle for SpiderChromePage {
    async fn goto(&self, url: &str, _params: NavigateParams) -> AbstractionResult<()> {
        debug!("Navigating to {} with spider-chrome", url);

        let _ = self
            .page
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

    async fn screenshot(&self, params: ScreenshotParams) -> AbstractionResult<Vec<u8>> {
        debug!("Taking screenshot with spider-chrome native API");

        use chromiumoxide_cdp::cdp::browser_protocol::page::CaptureScreenshotFormat;

        let mut spider_params = chromiumoxide::page::ScreenshotParams::builder();

        spider_params = match params.format {
            crate::abstraction::ScreenshotFormat::Png => {
                spider_params.format(CaptureScreenshotFormat::Png)
            }
            crate::abstraction::ScreenshotFormat::Jpeg => {
                spider_params.format(CaptureScreenshotFormat::Jpeg)
            }
        };

        if let Some(quality) = params.quality {
            spider_params = spider_params.quality(quality);
        }

        if params.full_page {
            spider_params = spider_params.full_page(true);
        }

        let screenshot_params = spider_params.build();

        self.page
            .screenshot(screenshot_params)
            .await
            .map_err(|e| AbstractionError::Screenshot(e.to_string()))
    }

    async fn pdf(&self, params: PdfParams) -> AbstractionResult<Vec<u8>> {
        debug!("Generating PDF with spider-chrome native API");

        use chromiumoxide_cdp::cdp::browser_protocol::page::PrintToPdfParams;

        let pdf_params = PrintToPdfParams {
            landscape: Some(params.landscape),
            display_header_footer: Some(params.display_header_footer),
            print_background: Some(params.print_background),
            scale: params.scale,
            paper_width: params.paper_width,
            paper_height: params.paper_height,
            margin_top: params.margin_top,
            margin_bottom: params.margin_bottom,
            margin_left: params.margin_left,
            margin_right: params.margin_right,
            page_ranges: params.page_ranges.clone(),
            prefer_css_page_size: params.prefer_css_page_size,
            ..Default::default()
        };

        self.page
            .pdf(pdf_params)
            .await
            .map_err(|e| AbstractionError::PdfGeneration(e.to_string()))
    }

    async fn wait_for_navigation(&self, timeout_ms: u64) -> AbstractionResult<()> {
        debug!(
            "Waiting for navigation with spider-chrome (timeout: {}ms)",
            timeout_ms
        );

        tokio::time::timeout(
            std::time::Duration::from_millis(timeout_ms),
            self.page.wait_for_navigation(),
        )
        .await
        .map_err(|_| {
            AbstractionError::Navigation(format!("Navigation timeout after {}ms", timeout_ms))
        })?
        .map_err(|e| AbstractionError::Navigation(e.to_string()))?;

        Ok(())
    }

    async fn set_timeout(&self, timeout_ms: u64) -> AbstractionResult<()> {
        debug!(
            "Setting timeout to {}ms (note: spider-chrome may not support this)",
            timeout_ms
        );

        Ok(())
    }

    async fn close(&self) -> AbstractionResult<()> {
        debug!("Closing spider-chrome page");

        warn!("Explicit page close not supported through Arc - page will close when all references are dropped");
        Ok(())
    }
}
