//! Spider-chrome native engine implementation
//!
//! This module provides a direct implementation using spider_chrome's native API,
//! unlike the chromiumoxide_impl which uses the chromiumoxide compatibility layer.
//!
//! ## Architecture
//!
//! - **Browser**: Uses `spider_chrome::Browser` directly
//! - **Page**: Uses `spider_chrome::Page` directly
//! - **CDP**: Uses CDP types from `spider_chrome::handler::blockers` for screenshot/PDF
//!
//! ## Key Differences from chromiumoxide_impl
//!
//! 1. **Direct API Access**: Uses spider_chrome's native types and methods
//! 2. **CDP Integration**: Screenshot and PDF use CDP protocol directly via spider_chrome
//! 3. **Thread Safety**: Uses `Arc<Page>` for safe concurrent access
//! 4. **Navigation**: Implements proper timeout-based navigation waiting
//!
//! ## Implementation Notes
//!
//! ### Screenshot & PDF
//! These methods use CDP (Chrome DevTools Protocol) types from spider_chrome:
//! - `spider_chrome::handler::blockers::CaptureScreenshotFormat`
//! - `spider_chrome::handler::blockers::PrintToPdfParams`
//!
//! ### Close Method
//! The `close()` method has a known limitation due to spider_chrome's API design:
//! - spider_chrome's `Page::close()` takes ownership (`self`, not `&self`)
//! - We use `Arc<Page>` for thread-safety, which prevents calling close()
//! - Pages are automatically cleaned up when all Arc references are dropped
//!
//! ### Navigation
//! Uses spider_chrome's native `wait_for_navigation()` with tokio timeout wrapper
//! for proper timeout handling as specified by the trait.

use async_trait::async_trait;
// spider_chrome package exports its crate as "chromiumoxide"
// We use it directly here to access native spider_chrome types
use chromiumoxide::{Browser as SpiderBrowser, Page as SpiderPage};
use std::sync::Arc;
use tracing::{debug, warn};

use crate::{
    params::{NavigateParams, PdfParams, ScreenshotParams},
    traits::{BrowserEngine, EngineType, PageHandle},
    AbstractionError, AbstractionResult,
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

        // Spider-chrome's close returns CloseReturns, we just need to return ()
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

        // Spider-chrome's goto returns the page
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
        // Spider-chrome requires &str, not &String
        let result = self
            .page
            .evaluate(script)
            .await
            .map_err(|e| AbstractionError::Evaluation(e.to_string()))?;

        // into_value() returns Result in spider-chrome
        result
            .into_value()
            .map_err(|e| AbstractionError::Evaluation(e.to_string()))
    }

    async fn screenshot(&self, params: ScreenshotParams) -> AbstractionResult<Vec<u8>> {
        debug!("Taking screenshot with spider-chrome native API");

        // Use CDP types from chromiumoxide_cdp (spider_chromiumoxide_cdp package)
        // Note: CDP types are in chromiumoxide_cdp crate (spider's fork)
        use chromiumoxide_cdp::cdp::browser_protocol::page::CaptureScreenshotFormat;

        let mut spider_params = chromiumoxide::page::ScreenshotParams::builder();

        // Set format (CDP CaptureScreenshotFormat enum)
        spider_params = match params.format {
            crate::params::ScreenshotFormat::Png => {
                spider_params.format(CaptureScreenshotFormat::Png)
            }
            crate::params::ScreenshotFormat::Jpeg => {
                spider_params.format(CaptureScreenshotFormat::Jpeg)
            }
        };

        // Set quality (JPEG only, 0-100)
        if let Some(quality) = params.quality {
            spider_params = spider_params.quality(quality);
        }

        // Set full page capture (captures beyond viewport)
        if params.full_page {
            spider_params = spider_params.full_page(true);
        }

        let screenshot_params = spider_params.build();

        // Execute screenshot via spider_chrome's Page::screenshot()
        // This internally uses CDP Page.captureScreenshot command
        self.page
            .screenshot(screenshot_params)
            .await
            .map_err(|e| AbstractionError::Screenshot(e.to_string()))
    }

    async fn pdf(&self, params: PdfParams) -> AbstractionResult<Vec<u8>> {
        debug!("Generating PDF with spider-chrome native API");

        // Use CDP PrintToPdfParams from chromiumoxide_cdp (spider_chromiumoxide_cdp package)
        // Note: CDP types are in chromiumoxide_cdp crate (spider's fork)
        use chromiumoxide_cdp::cdp::browser_protocol::page::PrintToPdfParams;

        // Map our abstraction params to CDP PrintToPdfParams
        // All fields are optional in CDP, matching our API design
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

        // Execute PDF generation via spider_chrome's Page::pdf()
        // This internally uses CDP Page.printToPDF command
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

        // Use spider_chrome's native wait_for_navigation with timeout
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

        // Spider-chrome doesn't have set_default_timeout
        // This is a no-op for compatibility
        Ok(())
    }

    async fn close(&self) -> AbstractionResult<()> {
        debug!("Closing spider-chrome page");

        // Spider-chrome's close() takes ownership (self, not &self)
        // Since we're behind an Arc and the trait requires &self, we cannot call close()
        // The page will be automatically closed when all Arc references are dropped
        //
        // This is a known limitation of the abstraction layer when using Arc for thread-safety
        // Alternative: If explicit close is critical, the caller should ensure they hold
        // the last reference and call Arc::try_unwrap() + close() manually

        warn!("Explicit page close not supported through Arc - page will close when all references are dropped");
        Ok(())
    }
}
