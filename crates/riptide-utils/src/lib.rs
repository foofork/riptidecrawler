//! Riptide Utils - Shared utilities for Riptide EventMesh
//!
//! This crate provides common utilities used across the Riptide EventMesh platform:
//!
//! - **Redis**: Connection pool management with health checks
//! - **HTTP**: HTTP client factory with connection pooling
//! - **Retry**: Retry policies with exponential backoff
//! - **Time**: Time utilities and timestamp conversions
//! - **Error**: Common error types and result aliases
//!
//! **Note**: Rate limiting has been moved to specialized crates:
//! - `riptide-stealth` for anti-detection rate limiting
//! - `riptide-api` for HTTP middleware rate limiting
//! - Use the `governor` crate directly for generic rate limiting
//!
//! **Note**: Circuit breaker functionality has been moved to `riptide-reliability` crate.

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod error;
pub mod http;
pub mod redis;
pub mod retry;
pub mod time;

// Re-export commonly used types
pub use error::{Error, Result};
pub use http::{HttpClientFactory, HttpConfig};
pub use redis::{RedisConfig, RedisPool};
pub use retry::RetryPolicy;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Test that all modules are accessible
        let _ = RedisConfig::default();
        let _ = HttpConfig::default();
        let _ = RetryPolicy::default();
    }
}
