//! Error types for the Riptide framework
//!
//! This module provides a unified error handling system using
//! thiserror for ergonomic error definitions.

use thiserror::Error;

/// Result type alias using RiptideError
pub type Result<T> = std::result::Result<T, RiptideError>;

/// Main error type for Riptide operations
#[derive(Error, Debug)]
pub enum RiptideError {
    /// Browser initialization failed
    #[error("Browser initialization failed: {0}")]
    BrowserInitialization(String),

    /// Browser operation failed
    #[error("Browser operation failed: {0}")]
    BrowserOperation(String),

    /// Navigation error
    #[error("Failed to navigate to URL: {0}")]
    Navigation(String),

    /// Content extraction failed
    #[error("Content extraction failed: {0}")]
    Extraction(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    Configuration(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// Timeout error
    #[error("Operation timed out after {0}ms")]
    Timeout(u64),

    /// Parse error
    #[error("Parse error: {0}")]
    Parse(String),

    /// URL parse error
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Cache error
    #[error("Cache operation failed: {0}")]
    Cache(String),

    /// Cache error (alias for compatibility)
    #[error("Cache operation failed: {0}")]
    CacheError(String),

    /// Storage error
    #[error("Storage operation failed: {0}")]
    Storage(String),

    /// Database error
    #[error("Database operation failed: {0}")]
    DatabaseError(String),

    /// Validation error
    #[error("Validation failed: {0}")]
    ValidationError(String),

    /// Serialization error
    #[error("Serialization failed: {0}")]
    SerializationError(String),

    /// Resource not found
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Resource already exists
    #[error("Resource already exists: {0}")]
    AlreadyExists(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded for tenant: {tenant_id}")]
    RateLimitExceeded {
        /// Tenant identifier
        tenant_id: String,
    },

    /// Generic error with custom message
    #[error("{0}")]
    Custom(String),

    /// Error from anyhow for interoperability
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl RiptideError {
    /// Create a custom error with a message
    pub fn custom<S: Into<String>>(msg: S) -> Self {
        RiptideError::Custom(msg.into())
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            RiptideError::Network(_) | RiptideError::Timeout(_) | RiptideError::BrowserOperation(_)
        )
    }

    /// Check if error is a client error (4xx equivalent)
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            RiptideError::InvalidUrl(_)
                | RiptideError::Configuration(_)
                | RiptideError::NotFound(_)
                | RiptideError::AlreadyExists(_)
                | RiptideError::PermissionDenied(_)
        )
    }

    /// Check if error is a server error (5xx equivalent)
    pub fn is_server_error(&self) -> bool {
        matches!(
            self,
            RiptideError::BrowserInitialization(_)
                | RiptideError::BrowserOperation(_)
                | RiptideError::Extraction(_)
                | RiptideError::Cache(_)
                | RiptideError::Storage(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = RiptideError::custom("test error");
        assert_eq!(err.to_string(), "test error");
    }

    #[test]
    fn test_retryable_errors() {
        assert!(RiptideError::Network("test".to_string()).is_retryable());
        assert!(RiptideError::Timeout(1000).is_retryable());
        assert!(!RiptideError::Configuration("test".to_string()).is_retryable());
    }

    #[test]
    fn test_client_errors() {
        assert!(RiptideError::NotFound("test".to_string()).is_client_error());
        assert!(RiptideError::Configuration("test".to_string()).is_client_error());
        assert!(!RiptideError::BrowserOperation("test".to_string()).is_client_error());
    }

    #[test]
    fn test_server_errors() {
        assert!(RiptideError::BrowserInitialization("test".to_string()).is_server_error());
        assert!(RiptideError::Extraction("test".to_string()).is_server_error());
        assert!(!RiptideError::Configuration("test".to_string()).is_server_error());
    }

    #[test]
    fn test_url_parse_error_conversion() {
        let url_err = url::Url::parse("not a url").unwrap_err();
        let riptide_err: RiptideError = url_err.into();
        assert!(matches!(riptide_err, RiptideError::InvalidUrl(_)));
    }
}
