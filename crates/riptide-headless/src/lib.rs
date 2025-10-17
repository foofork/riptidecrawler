//! RipTide Headless Browser Management Library
//!
//! This library provides browser pool management for headless browser operations.
//! It includes connection pooling, health checking, and automatic recovery.
//!
//! ## Architecture
//!
//! **NOTE**: Core browser engine components have been extracted to `riptide-engine` crate.
//! This crate now serves as a compatibility layer and HTTP API wrapper.
//!
//! - **HeadlessLauncher**: High-level API for launching browser sessions (from riptide-engine)
//! - **BrowserPool**: Low-level browser instance pool (from riptide-engine)
//! - **LaunchSession**: Managed browser session with automatic cleanup (from riptide-engine)
//! - **CDP API**: HTTP endpoints for browser automation
//!
//! ## Usage
//!
//! ```no_run
//! # use riptide_headless::HeadlessLauncher;
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

// Re-export core engine components from riptide-engine
pub use riptide_engine::{
    // Browser pool
    BrowserPool, BrowserPoolConfig, BrowserCheckout, PoolStats, PoolEvent,
    // CDP connection pool
    CdpConnectionPool, CdpPoolConfig, ConnectionStats, ConnectionHealth,
    // Launcher
    HeadlessLauncher, LaunchSession, LauncherConfig, LauncherStats,
    // Models
    models,
};

#[cfg(feature = "headless")]
pub use riptide_engine::hybrid_fallback::{
    BrowserEngine, BrowserResponse, FallbackMetrics, HybridBrowserFallback,
};

// Local CDP HTTP API module (depends on riptide-engine components)
// Temporarily disabled due to chromiumoxide version conflict (Phase 2 resolution)
// pub mod cdp;

// Backward compatibility re-exports
pub mod pool {
    //! Browser pool module - MOVED to riptide-engine
    //!
    //! This module re-exports types from `riptide-engine` for backward compatibility.
    pub use riptide_engine::{BrowserPool, BrowserPoolConfig, BrowserCheckout, PoolStats, PoolEvent};
}

pub mod cdp_pool {
    //! CDP connection pool module - MOVED to riptide-engine
    //!
    //! This module re-exports types from `riptide-engine` for backward compatibility.
    pub use riptide_engine::{CdpConnectionPool, CdpPoolConfig, ConnectionStats, ConnectionHealth, PooledConnection, CdpCommand};
}

pub mod launcher {
    //! Browser launcher module - MOVED to riptide-engine
    //!
    //! This module re-exports types from `riptide-engine` for backward compatibility.
    pub use riptide_engine::{HeadlessLauncher, LaunchSession, LauncherConfig, LauncherStats};
}

// Re-export models from riptide-engine
// Already exported above on line 45, commenting out duplicate
// pub use riptide_engine::models;

#[cfg(feature = "headless")]
pub mod hybrid_fallback {
    //! Hybrid engine fallback module - MOVED to riptide-engine
    //!
    //! This module re-exports types from `riptide-engine` for backward compatibility.
    pub use riptide_engine::hybrid_fallback::{BrowserEngine, BrowserResponse, FallbackMetrics, HybridBrowserFallback};
}
