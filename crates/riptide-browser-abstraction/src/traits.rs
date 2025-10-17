//! Core trait definitions for browser abstraction

use crate::{AbstractionResult, NavigateParams, PdfParams, ScreenshotParams};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Browser engine type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineType {
    /// Standard chromiumoxide engine
    Chromiumoxide,
    // Spider-chrome engine with enhanced stealth (not yet implemented due to type conflicts)
    // See ADR-006 for details on the incompatibility
    // #[cfg(feature = "spider")]
    // SpiderChrome,
}

/// Unified browser engine interface
///
/// This trait abstracts over chromiumoxide and spider-chrome engines,
/// providing a common API for page creation and management.
#[async_trait]
pub trait BrowserEngine: Send + Sync {
    /// Create a new page/tab in the browser
    async fn new_page(&self) -> AbstractionResult<Box<dyn PageHandle>>;

    /// Get the engine type
    fn engine_type(&self) -> EngineType;

    /// Close the browser and cleanup resources
    async fn close(&self) -> AbstractionResult<()>;

    /// Get browser version info
    async fn version(&self) -> AbstractionResult<String>;
}

/// Unified page interface
///
/// This trait abstracts over chromiumoxide::Page and spider_chrome's Page,
/// providing a common API for page automation.
#[async_trait]
pub trait PageHandle: Send + Sync {
    /// Navigate to a URL
    async fn goto(&self, url: &str, params: NavigateParams) -> AbstractionResult<()>;

    /// Get the page HTML content
    async fn content(&self) -> AbstractionResult<String>;

    /// Get the current URL
    async fn url(&self) -> AbstractionResult<String>;

    /// Execute JavaScript and get result
    async fn evaluate(&self, script: &str) -> AbstractionResult<serde_json::Value>;

    /// Take a screenshot
    async fn screenshot(&self, params: ScreenshotParams) -> AbstractionResult<Vec<u8>>;

    /// Generate PDF
    async fn pdf(&self, params: PdfParams) -> AbstractionResult<Vec<u8>>;

    /// Wait for navigation to complete
    async fn wait_for_navigation(&self, timeout_ms: u64) -> AbstractionResult<()>;

    /// Set page timeout
    async fn set_timeout(&self, timeout_ms: u64) -> AbstractionResult<()>;

    /// Close the page
    async fn close(&self) -> AbstractionResult<()>;
}
