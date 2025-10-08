//! RipTide Headless Browser Management Library
//!
//! This library provides browser pool management for headless browser operations.
//! It includes connection pooling, health checking, and automatic recovery.
//!
//! ## Architecture
//!
//! - **HeadlessLauncher**: High-level API for launching browser sessions with stealth, pooling, and monitoring
//! - **BrowserPool**: Low-level browser instance pool with health checks and auto-recovery
//! - **LaunchSession**: Managed browser session with automatic cleanup
//!
//! ## Usage
//!
//! ```no_run
//! # use riptide_headless::launcher::HeadlessLauncher;
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

pub mod cdp;
pub mod launcher;
pub mod models;
pub mod pool;

// Re-export main public API
pub use launcher::{HeadlessLauncher, LaunchSession, LauncherConfig, LauncherStats};
pub use pool::{BrowserCheckout, BrowserPool, BrowserPoolConfig, PoolStats};
