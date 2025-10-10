//! Error types for resource management operations.
//!
//! Provides type-safe error handling with detailed context for each failure mode.

use std::time::Duration;
use thiserror::Error;

/// Comprehensive error type for resource management operations
#[derive(Debug, Error)]
#[allow(dead_code)] // Some variants reserved for future use
pub enum ResourceManagerError {
    /// Browser pool operation failed
    #[error("Browser pool error: {0}")]
    BrowserPool(String),

    /// Rate limit exceeded for host
    #[error("Rate limit exceeded, retry after {retry_after:?}")]
    RateLimit {
        /// Duration to wait before retrying
        retry_after: Duration,
    },

    /// System is under memory pressure
    #[error("Memory pressure detected, operation rejected")]
    MemoryPressure,

    /// WASM instance operation failed
    #[error("WASM instance error: {0}")]
    Wasm(String),

    /// Operation timed out
    #[error("Operation '{operation}' timed out after {duration:?}")]
    Timeout {
        /// The operation that timed out
        operation: String,
        /// How long we waited before timing out
        duration: Duration,
    },

    /// Resource exhausted (pool full, semaphore unavailable, etc.)
    #[error("Resource exhausted: {resource_type}")]
    ResourceExhausted {
        /// Type of resource that was exhausted
        resource_type: String,
    },

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// URL parsing error
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// Wrapped error from other sources
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

/// Result type alias for resource manager operations
pub type Result<T> = std::result::Result<T, ResourceManagerError>;

impl From<url::ParseError> for ResourceManagerError {
    fn from(err: url::ParseError) -> Self {
        ResourceManagerError::InvalidUrl(err.to_string())
    }
}

/// Helper function to create a timeout error
#[allow(dead_code)] // Reserved for future error handling paths
pub fn timeout_error(operation: impl Into<String>, duration: Duration) -> ResourceManagerError {
    ResourceManagerError::Timeout {
        operation: operation.into(),
        duration,
    }
}

/// Helper function to create a resource exhausted error
#[allow(dead_code)] // Reserved for future error handling paths
pub fn exhausted_error(resource_type: impl Into<String>) -> ResourceManagerError {
    ResourceManagerError::ResourceExhausted {
        resource_type: resource_type.into(),
    }
}
