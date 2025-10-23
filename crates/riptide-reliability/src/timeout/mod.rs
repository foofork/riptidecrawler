//! # Adaptive Timeout System
//!
//! This module provides intelligent timeout management that learns optimal
//! timeouts per domain based on historical success/failure patterns.
//!
//! ## Features
//!
//! - **Per-Domain Learning**: Track timeout success/failure per domain
//! - **Adaptive Adjustment**: Learn optimal timeouts (5s-60s range)
//! - **Exponential Backoff**: Automatically increase timeouts on failures
//! - **Success Optimization**: Reduce timeouts after consecutive successes
//! - **Persistent Profiles**: Save/load timeout profiles across sessions
//! - **Global Manager**: Thread-safe singleton for application-wide usage
//!
//! ## Usage
//!
//! ```rust,ignore
//! use riptide_reliability::timeout::{get_global_timeout_manager, TimeoutConfig};
//!
//! // Get the global timeout manager
//! let manager = get_global_timeout_manager().await?;
//!
//! // Get timeout for a URL
//! let timeout = manager.get_timeout("https://example.com/page").await;
//!
//! // Record successful request
//! let start = Instant::now();
//! // ... perform request ...
//! manager.record_success("https://example.com/page", start.elapsed()).await;
//!
//! // Record timeout failure
//! manager.record_timeout("https://slow-site.com/page").await;
//!
//! // Get statistics
//! let stats = manager.get_stats().await;
//! println!("Average timeout: {}s", stats.avg_timeout_secs);
//! println!("Success rate: {:.1}%", stats.avg_success_rate);
//! ```
//!
//! ## Configuration
//!
//! ```rust,ignore
//! use riptide_reliability::timeout::{AdaptiveTimeoutManager, TimeoutConfig};
//! use std::path::PathBuf;
//!
//! let config = TimeoutConfig {
//!     storage_path: PathBuf::from("./timeouts.json"),
//!     default_timeout_secs: 30,
//!     auto_save: true,
//! };
//!
//! let manager = AdaptiveTimeoutManager::new(config).await?;
//! ```

mod manager;
mod profile;

pub use manager::{get_global_timeout_manager, AdaptiveTimeoutManager, TimeoutConfig};
pub use profile::{TimeoutProfile, TimeoutStats};

// Re-export constants for configuration
pub use manager::{
    BACKOFF_MULTIPLIER, DEFAULT_TIMEOUT_SECS, MAX_TIMEOUT_SECS, MIN_TIMEOUT_SECS, SUCCESS_REDUCTION,
};
