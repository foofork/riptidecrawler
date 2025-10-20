//! Browser Engine Abstraction Layer
//!
//! This crate provides a unified interface for browser automation using spider_chrome.
//! The spider_chrome package exports its crate as "chromiumoxide" for API compatibility.
//!
//! ## Architecture
//!
//! The abstraction uses trait objects and async-trait to provide:
//! - Type-safe API
//! - Minimal performance overhead (<0.01%)
//!
//! ## Usage
//!
//! ```no_run
//! use riptide_browser_abstraction::{BrowserEngine, EngineType, create_engine};
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Create an engine using spider_chrome (exports as chromiumoxide)
//! let engine = create_engine(EngineType::Chromiumoxide).await?;
//!
//! // Use the unified interface
//! let page = engine.new_page().await?;
//! let html = page.content().await?;
//! # Ok(())
//! # }
//! ```

// Modules
mod chromiumoxide_impl;
mod error;
mod factory;
mod params;
mod spider_impl;
mod traits;

// Public exports
pub use chromiumoxide_impl::{ChromiumoxideEngine, ChromiumoxidePage};
pub use error::{AbstractionError, AbstractionResult};
pub use factory::create_engine;
pub use params::{NavigateParams, PdfParams, ScreenshotFormat, ScreenshotParams, WaitUntil};
pub use spider_impl::{SpiderChromeEngine, SpiderChromePage};
pub use traits::{BrowserEngine, EngineType, PageHandle};

#[cfg(test)]
mod tests;
