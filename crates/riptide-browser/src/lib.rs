//! RipTide Browser - Unified Browser Automation Core
//!
//! This crate provides the core browser automation infrastructure for RipTide:
//! - Browser pool management with resource tracking
//! - CDP connection pooling for multiplexing
//! - Headless browser launcher with stealth capabilities
//! - Unified browser abstraction layer
//!
//! ## Architecture
//!
//! **riptide-browser** now contains the REAL implementations (migrated from riptide-engine):
//! - `pool`: Browser instance pooling and lifecycle management
//! - `cdp`: CDP connection pooling with batching and multiplexing
//! - `launcher`: High-level API for browser launching with stealth
//! - `models`: Shared types and abstractions
//!
//! ## Usage
//!
//! ```no_run
//! # use riptide_browser::launcher::HeadlessLauncher;
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

// ========================================
// Core Modules (REAL IMPLEMENTATIONS)
// ========================================

pub mod cdp;
pub mod launcher;
pub mod models;
pub mod pool;

// ========================================
// Public API - Re-export key types
// ========================================

// Pool management
pub use pool::{
    BrowserCheckout, BrowserHealth, BrowserPool, BrowserPoolConfig, BrowserPoolRef, BrowserStats,
    PoolEvent, PoolStats, PooledBrowser,
};

// CDP connection pooling
pub use cdp::{
    BatchExecutionResult, BatchResult, CdpCommand, CdpConnectionPool, CdpPoolConfig, CdpPoolStats,
    ConnectionHealth, ConnectionPriority, ConnectionStats, PerformanceMetrics, PooledConnection,
};

// Launcher API
pub use launcher::{HeadlessLauncher, LaunchSession, LauncherConfig, LauncherStats};

// Models
pub use models::{
    Artifacts, ArtifactsOut, PageAction, RenderErrorResp, RenderReq, RenderResp, Timeouts,
};

// ========================================
// External Dependencies Re-exports
// ========================================

// Re-export spider_chrome types for consumers
pub use chromiumoxide::{Browser, BrowserConfig, Page};
pub use chromiumoxide_cdp::cdp::browser_protocol::target::SessionId;

// Re-export browser abstraction for consumers
pub use riptide_browser_abstraction::{BrowserEngine, ChromiumoxideEngine, ChromiumoxidePage};

// ========================================
// Integration with riptide-headless
// ========================================
// Note: riptide-headless depends on riptide-browser, not the other way around
// Dynamic rendering capabilities are provided by riptide-headless
