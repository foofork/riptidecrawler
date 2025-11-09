//! CDP Module - Chrome DevTools Protocol implementations
//!
//! This module contains:
//! - connection_pool: CDP connection pooling and multiplexing
//! - chromiumoxide_impl: Chromiumoxide CDP implementation
//! - spider_impl: Spider-chrome CDP implementation

mod chromiumoxide_impl;
mod connection_pool;
mod spider_impl;

// Re-export CDP connection pool (existing functionality)
pub use connection_pool::*;

// Re-export CDP implementations
pub use chromiumoxide_impl::{ChromiumoxideEngine, ChromiumoxidePage};
pub use spider_impl::{SpiderChromeEngine, SpiderChromePage};
