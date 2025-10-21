//! RipTide Browser - Unified Browser Automation Facade
//!
//! This crate provides a unified interface for browser automation in RipTide,
//! combining functionality from both `riptide-engine` and `riptide-headless`.
//!
//! ## Architecture
//!
//! - **riptide-engine**: Core browser pool management, CDP connection pooling, launcher
//! - **riptide-headless**: HTTP API wrapper, dynamic rendering, backward compatibility
//! - **riptide-browser**: Unified facade that re-exports both
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
// Re-exports from riptide-engine
// ========================================

// Core pool management
pub use riptide_engine::{
    pool, BrowserCheckout, BrowserPool, BrowserPoolConfig, PoolEvent, PoolStats,
};

// CDP connection pooling
pub use riptide_engine::{
    cdp_pool, BatchExecutionResult, BatchResult, CdpCommand, CdpConnectionPool, CdpPoolConfig,
    ConnectionHealth, ConnectionStats, PooledConnection,
};

// Launcher API
pub use riptide_engine::{
    launcher, HeadlessLauncher, LaunchSession, LauncherConfig, LauncherStats,
};

// Models and CDP types
pub use riptide_engine::{cdp, models};

// Hybrid fallback (riptide-engine has headless feature enabled)
pub use riptide_engine::hybrid_fallback::{
    BrowserResponse, FallbackMetrics, HybridBrowserFallback,
};

// Browser abstraction
pub use riptide_engine::{AbstractBrowserEngine, EngineType, NavigateParams, PageHandle};

// Factory functions (riptide-engine has headless feature enabled)
pub use riptide_engine::factory;

// ========================================
// Re-exports from riptide-headless
// ========================================

// Dynamic rendering module (unique to riptide-headless)
pub use riptide_headless::dynamic;

// Also re-export individual types for convenience
pub use riptide_headless::dynamic::{
    DynamicConfig, DynamicRenderResult, PageAction, RenderArtifacts, ScrollConfig, ViewportConfig,
    WaitCondition,
};

// Re-export abstraction layer directly for convenience
pub use riptide_browser_abstraction::{BrowserEngine, ChromiumoxideEngine, ChromiumoxidePage};
