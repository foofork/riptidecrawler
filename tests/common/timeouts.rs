//! Test timeout constants for consistent test execution across the codebase.
//!
//! This module provides standardized timeout durations for different test operation types.
//! Timeouts can be scaled using the `TEST_TIMEOUT_MULTIPLIER` environment variable,
//! which is useful for slower CI environments or debugging.
//!
//! # Usage
//!
//! ```rust
//! use std::time::Duration;
//! use tests::common::timeouts::{FAST_OP, MEDIUM_OP, SLOW_OP};
//!
//! #[tokio::test]
//! async fn test_fast_operation() {
//!     let result = tokio::time::timeout(FAST_OP, fast_api_call()).await;
//!     assert!(result.is_ok(), "Fast operation timed out");
//! }
//!
//! #[tokio::test]
//! async fn test_database_query() {
//!     let result = tokio::time::timeout(MEDIUM_OP, db_query()).await;
//!     assert!(result.is_ok(), "Medium operation timed out");
//! }
//!
//! #[tokio::test]
//! async fn test_full_workflow() {
//!     let result = tokio::time::timeout(SLOW_OP, complex_workflow()).await;
//!     assert!(result.is_ok(), "Slow operation timed out");
//! }
//! ```
//!
//! # Environment Variable Scaling
//!
//! Set `TEST_TIMEOUT_MULTIPLIER` to scale all timeouts:
//! ```bash
//! # Double all timeouts for slower CI or debugging
//! export TEST_TIMEOUT_MULTIPLIER=2.0
//! cargo test
//!
//! # Use default timeouts
//! unset TEST_TIMEOUT_MULTIPLIER
//! cargo test
//! ```

use std::time::Duration;

/// Get the timeout multiplier from environment variable.
/// Defaults to 1.0 if not set or invalid.
fn timeout_multiplier() -> f64 {
    std::env::var("TEST_TIMEOUT_MULTIPLIER")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|&m| m > 0.0 && m <= 10.0)
        .unwrap_or(1.0)
}

/// Apply timeout multiplier to a base duration.
fn scaled_duration(base_secs: u64) -> Duration {
    let multiplier = timeout_multiplier();
    Duration::from_secs_f64(base_secs as f64 * multiplier)
}

/// Fast operation timeout: 2 seconds (default)
///
/// Use for:
/// - Simple unit tests
/// - In-memory operations
/// - Fast API health checks
/// - Quick validation logic
pub const FAST_OP_BASE: u64 = 2;

/// Medium operation timeout: 10 seconds (default)
///
/// Use for:
/// - API requests with processing
/// - Database queries
/// - File I/O operations
/// - WASM module initialization
/// - Basic integration tests
pub const MEDIUM_OP_BASE: u64 = 10;

/// Slow operation timeout: 30 seconds (default)
///
/// Use for:
/// - Complex workflows
/// - Multi-step integration tests
/// - Large data processing
/// - Network operations with retries
/// - End-to-end test scenarios
pub const SLOW_OP_BASE: u64 = 30;

/// Very slow operation timeout: 60 seconds (default)
///
/// Use for:
/// - Full system integration tests
/// - Heavy data processing workflows
/// - Tests with multiple external dependencies
/// - Performance benchmarking tests
pub const VERY_SLOW_OP_BASE: u64 = 60;

/// Fast operation timeout with scaling applied
pub fn fast_op() -> Duration {
    scaled_duration(FAST_OP_BASE)
}

/// Medium operation timeout with scaling applied
pub fn medium_op() -> Duration {
    scaled_duration(MEDIUM_OP_BASE)
}

/// Slow operation timeout with scaling applied
pub fn slow_op() -> Duration {
    scaled_duration(SLOW_OP_BASE)
}

/// Very slow operation timeout with scaling applied
pub fn very_slow_op() -> Duration {
    scaled_duration(VERY_SLOW_OP_BASE)
}

// Constant aliases for simpler imports
pub const FAST_OP: Duration = Duration::from_secs(FAST_OP_BASE);
pub const MEDIUM_OP: Duration = Duration::from_secs(MEDIUM_OP_BASE);
pub const SLOW_OP: Duration = Duration::from_secs(SLOW_OP_BASE);
pub const VERY_SLOW_OP: Duration = Duration::from_secs(VERY_SLOW_OP_BASE);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_timeouts() {
        // Without multiplier, should return base durations
        std::env::remove_var("TEST_TIMEOUT_MULTIPLIER");
        assert_eq!(FAST_OP, Duration::from_secs(2));
        assert_eq!(MEDIUM_OP, Duration::from_secs(10));
        assert_eq!(SLOW_OP, Duration::from_secs(30));
        assert_eq!(VERY_SLOW_OP, Duration::from_secs(60));
    }

    #[test]
    fn test_timeout_multiplier_parsing() {
        // Test valid multiplier
        std::env::set_var("TEST_TIMEOUT_MULTIPLIER", "2.0");
        assert_eq!(timeout_multiplier(), 2.0);

        // Test invalid multiplier defaults to 1.0
        std::env::set_var("TEST_TIMEOUT_MULTIPLIER", "invalid");
        assert_eq!(timeout_multiplier(), 1.0);

        // Test zero multiplier defaults to 1.0
        std::env::set_var("TEST_TIMEOUT_MULTIPLIER", "0.0");
        assert_eq!(timeout_multiplier(), 1.0);

        // Test negative multiplier defaults to 1.0
        std::env::set_var("TEST_TIMEOUT_MULTIPLIER", "-1.0");
        assert_eq!(timeout_multiplier(), 1.0);

        // Test too large multiplier defaults to 1.0
        std::env::set_var("TEST_TIMEOUT_MULTIPLIER", "100.0");
        assert_eq!(timeout_multiplier(), 1.0);

        // Cleanup
        std::env::remove_var("TEST_TIMEOUT_MULTIPLIER");
    }

    #[test]
    fn test_scaled_durations() {
        // Test 2x scaling
        std::env::set_var("TEST_TIMEOUT_MULTIPLIER", "2.0");
        assert_eq!(fast_op(), Duration::from_secs(4));
        assert_eq!(medium_op(), Duration::from_secs(20));
        assert_eq!(slow_op(), Duration::from_secs(60));
        assert_eq!(very_slow_op(), Duration::from_secs(120));

        // Test 0.5x scaling
        std::env::set_var("TEST_TIMEOUT_MULTIPLIER", "0.5");
        assert_eq!(fast_op(), Duration::from_secs(1));
        assert_eq!(medium_op(), Duration::from_secs(5));
        assert_eq!(slow_op(), Duration::from_secs(15));
        assert_eq!(very_slow_op(), Duration::from_secs(30));

        // Cleanup
        std::env::remove_var("TEST_TIMEOUT_MULTIPLIER");
    }

    #[test]
    fn test_timeout_ordering() {
        // Ensure timeouts are properly ordered
        assert!(FAST_OP < MEDIUM_OP);
        assert!(MEDIUM_OP < SLOW_OP);
        assert!(SLOW_OP < VERY_SLOW_OP);
    }
}
