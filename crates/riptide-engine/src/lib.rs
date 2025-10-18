//! Browser engine and pool management for RipTide
//!
//! This crate provides browser automation infrastructure including:
//! - Browser pool lifecycle management with resource tracking
//! - CDP (Chrome DevTools Protocol) connection pooling and batching
//! - Instance health monitoring and auto-recovery
//! - Hybrid engine selection (chromiumoxide, spider-chrome)
//! - Stealth and fingerprint management integration
//!
//! ## Architecture
//!
//! The `riptide-engine` crate consolidates browser automation components
//! from across the workspace:
//!
//! - **pool**: Browser pool management (from riptide-headless)
//! - **cdp_pool**: CDP connection pooling and command batching (from riptide-headless)
//! - **cdp**: CDP types and utilities (from riptide-headless)
//! - **launcher**: High-level browser session launcher (from riptide-headless)
//! - **hybrid_fallback**: Engine selection and fallback logic (from riptide-headless)
//!
//! ## Usage
//!
//! ```no_run
//! # use riptide_engine::launcher::HeadlessLauncher;
//! # async fn example() -> anyhow::Result<()> {
//! // Create launcher with pooling
//! let launcher = HeadlessLauncher::new().await?;
//!
//! // Launch a page with stealth
//! let session = launcher.launch_page_default("https://example.com").await?;
//!
//! // Access the page
//! let page = session.page();
//! let content = page.content().await?;
//!
//! // Session automatically returns browser to pool when dropped
//! # Ok(())
//! # }
//! ```
//!
//! ## Features
//!
//! - **Resource Management**: Automatic browser lifecycle, connection pooling
//! - **Health Monitoring**: Tiered health checks (fast liveness + full diagnostics)
//! - **Memory Optimization**: V8 heap tracking, automatic cleanup
//! - **Stealth Integration**: Fingerprint randomization, anti-detection
//! - **Hybrid Engines**: Automatic fallback between chromiumoxide and spider-chrome
//! - **Performance Tracking**: Built-in metrics and statistics

// Core browser pool management
pub mod pool;

// CDP connection pooling (P1-B4 optimization)
pub mod cdp_pool;

// Models and types for CDP API
pub mod models;

// High-level launcher API
pub mod launcher;

// CDP HTTP API types and utilities (temporarily disabled - depends on headless crate structure)
// TODO: Re-enable after resolving circular dependencies with riptide-headless
// pub mod cdp;

// Hybrid engine fallback
#[cfg(feature = "headless")]
pub mod hybrid_fallback;

// Re-export browser abstraction types
pub use riptide_browser_abstraction::{
    BrowserEngine as AbstractBrowserEngine, EngineType, NavigateParams, PageHandle,
};

// Factory functions for wrapping spider_chrome instances (exports as chromiumoxide)
#[cfg(feature = "headless")]
pub mod factory {
    use chromiumoxide::{Browser, Page};
    use riptide_browser_abstraction::{BrowserEngine, PageHandle};
    use riptide_browser_abstraction::{ChromiumoxideEngine, ChromiumoxidePage};

    /// Wrap a chromiumoxide Browser in the BrowserEngine trait
    pub fn wrap_browser(browser: Browser) -> Box<dyn BrowserEngine> {
        Box::new(ChromiumoxideEngine::new(browser))
    }

    /// Wrap a chromiumoxide Page in the PageHandle trait
    pub fn wrap_page(page: Page) -> Box<dyn PageHandle> {
        Box::new(ChromiumoxidePage::new(page))
    }
}

// Re-export main public API
pub use cdp_pool::{
    BatchExecutionResult, BatchResult, CdpCommand, CdpConnectionPool, CdpPoolConfig,
    ConnectionHealth, ConnectionStats, PooledConnection,
};
pub use launcher::{HeadlessLauncher, LaunchSession, LauncherConfig, LauncherStats};
pub use pool::{BrowserCheckout, BrowserPool, BrowserPoolConfig, PoolEvent, PoolStats};

#[cfg(feature = "headless")]
pub use hybrid_fallback::{BrowserResponse, EngineKind, FallbackMetrics, HybridBrowserFallback};
