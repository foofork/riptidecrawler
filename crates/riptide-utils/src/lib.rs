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
//! **Note**: Circuit breaker functionality is provided in the `circuit_breaker` module.
//! This is infrastructure-level shared code that avoids circular dependencies.

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod circuit_breaker;
pub mod error;
pub mod health_registry;
pub mod http;
pub mod redis;
pub mod retry;
pub mod time;

// Re-export commonly used types
pub use circuit_breaker::{
    guarded_call, CircuitBreaker, Clock, Config as CircuitConfig, RealClock, State as CircuitState,
};
pub use error::{Error, Result};
pub use health_registry::{InMemoryHealthRegistry, SimpleHealthCheck};
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
