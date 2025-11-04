//! Riptide Utils - Shared utilities for Riptide EventMesh
//!
//! This crate provides common utilities used across the Riptide EventMesh platform:
//!
//! - **Redis**: Connection pool management with health checks
//! - **HTTP**: HTTP client factory with connection pooling
//! - **Retry**: Retry policies with exponential backoff
//! - **Rate Limiting**: Token bucket rate limiting
//! - **Circuit Breaker**: Fault tolerance with circuit breaker pattern
//! - **Time**: Time utilities and timestamp conversions
//! - **Error**: Common error types and result aliases

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod circuit_breaker;
pub mod error;
pub mod http;
pub mod rate_limit;
pub mod redis;
pub mod retry;
pub mod time;

// Re-export commonly used types
pub use circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState};
pub use error::{Error, Result};
pub use http::{HttpClientFactory, HttpConfig};
pub use rate_limit::{RateLimiterBuilder, SimpleRateLimiter};
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
        let _ = SimpleRateLimiter::new(10);
    }
}
