//! RipTide Browser - Unified Browser Automation Core
//!
//! **Sprint 4.6: Consolidated Browser Crate** (3 crates → 1)
//!
//! This crate provides the complete browser automation infrastructure:
//! - `abstraction/`: Pure trait definitions (NO concrete CDP types)
//! - `cdp/`: CDP implementations (chromiumoxide, spider-chrome) + connection pooling
//! - `pool/`: Browser instance pooling and lifecycle management
//! - `launcher/`: High-level API for browser launching with stealth
//! - `http/`: HTTP API for headless browser operations
//! - `models/`: Shared types and request/response models
//! - `hybrid/`: Fallback and hybrid engine management
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
// Core Modules - Sprint 4.6 Consolidation
// ========================================

pub mod abstraction; // Trait-only abstractions (NO concrete types)
pub mod cdp; // CDP implementations + connection pool
pub mod http; // HTTP API (from riptide-headless)
pub mod hybrid; // Hybrid engine management
pub mod launcher; // Headless launcher
pub mod models; // Shared types
pub mod pool; // Browser pool management

// ========================================
// Public API - Sprint 4.6 Re-exports
// ========================================

// Abstraction layer (traits only)
pub use abstraction::{
    AbstractionError, AbstractionResult, BrowserEngine, EngineType, NavigateParams, PageHandle,
    PdfParams, ScreenshotFormat, ScreenshotParams, WaitUntil,
};

// CDP implementations
pub use cdp::{ChromiumoxideEngine, ChromiumoxidePage, SpiderChromeEngine, SpiderChromePage};

// CDP connection pooling
pub use cdp::{
    BatchExecutionResult, BatchResult, CdpCommand, CdpConnectionPool, CdpPoolConfig, CdpPoolStats,
    ConnectionHealth, ConnectionPriority, ConnectionStats, PerformanceMetrics, PooledConnection,
};

// Browser pool management
pub use pool::{
    BrowserCheckout, BrowserHealth, BrowserPool, BrowserPoolConfig, BrowserPoolRef, BrowserStats,
    PoolEvent, PoolStats, PooledBrowser,
};

// Launcher API
pub use launcher::{HeadlessLauncher, LaunchSession, LauncherConfig, LauncherStats};

// Hybrid fallback
pub use hybrid::{BrowserResponse, EngineKind, FallbackMetrics, HybridBrowserFallback};

// HTTP API (models re-exported, implementation remains in riptide-headless for now)

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
