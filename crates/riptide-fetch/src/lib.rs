//! # Riptide Fetch
//!
//! HTTP/network layer for the RipTide web scraping framework.
//! This crate provides low-level HTTP fetching capabilities with:
//!
//! - **Async HTTP client**: Built on reqwest with connection pooling
//! - **Retry logic**: Exponential backoff with configurable retries
//! - **Response caching**: Intelligent HTTP caching
//! - **Rate limiting**: Request throttling and delay management
//! - **Error handling**: Comprehensive HTTP error types
//! - **Metrics**: Request/response monitoring
//!
//! ## Architecture
//!
//! The fetch module is extracted from riptide-core to separate concerns:
//! - **riptide-fetch**: HTTP/network layer (this crate)
//! - **riptide-spider**: Crawling logic (uses riptide-fetch)
//! - **riptide-extraction**: Content parsing (uses riptide-fetch for direct fetches)
//! - **riptide-core**: Orchestration

// Core modules
pub mod circuit;
pub mod fetch;
pub mod robots;
pub mod telemetry;

// Re-export main types
pub use circuit::{CircuitBreaker, State as CircuitState};
pub use fetch::*;
pub use robots::{RobotsConfig, RobotsManager};
