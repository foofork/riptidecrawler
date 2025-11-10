//! Test helper utilities for integration tests with testcontainers
//!
//! This module provides helpers for setting up test infrastructure:
//! - PostgreSQL containers for database testing (requires postgres feature)
//! - Redis containers for cache testing
//! - Test data initialization and cleanup

#[cfg(feature = "postgres")]
pub mod postgres_helpers;
pub mod redis_helpers;

// Re-export for convenience in test files
// Note: May appear unused when specific features aren't enabled
#[allow(unused_imports)]
#[cfg(feature = "postgres")]
pub use postgres_helpers::*;

#[allow(unused_imports)]
pub use redis_helpers::*;
