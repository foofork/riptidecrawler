//! Unified error handling for the Riptide facade.
//!
//! Provides a single error type that maps errors from all underlying crates
//! while preserving context and maintaining ergonomics.

use std::fmt;
use thiserror::Error;

/// Unified error type for Riptide facade operations.
///
/// This error type provides a consistent interface for all operations
/// while preserving underlying error details for debugging.
#[derive(Debug, Error)]
pub enum RiptideError {
    // ============================================================================
    // Network and Fetch Errors
    // ============================================================================
    /// HTTP fetch error
    #[error("Fetch error: {0}")]
    FetchError(String),

    /// HTTP error with status code
    #[error("HTTP error: {status} - {message}")]
    HttpError { status: u16, message: String },

    /// Network timeout
    #[error("Network timeout after {timeout_ms}ms")]
    TimeoutError { timeout_ms: u64 },

    /// Connection error
    #[error("Connection error: {0}")]
    ConnectionError(String),

    // ============================================================================
    // Extraction and Parsing Errors
    // ============================================================================
    /// Content extraction error
    #[error("Extraction error: {0}")]
    ExtractionError(String),

    /// HTML/XML parsing error
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Invalid selector or pattern
    #[error("Invalid selector: {0}")]
    InvalidSelectorError(String),

    /// Content quality too low
    #[error("Low quality content: score {score}, threshold {threshold}")]
    LowQualityError { score: f64, threshold: f64 },

    // ============================================================================
    // Browser Automation Errors
    // ============================================================================
    /// Browser operation error
    #[error("Browser error: {0}")]
    BrowserError(String),

    /// Chrome DevTools Protocol error
    #[error("CDP error: {0}")]
    CdpError(String),

    /// Browser pool exhausted
    #[error("Browser pool exhausted: no instances available")]
    PoolExhaustedError,

    /// Browser launch error
    #[error("Browser launch failed: {0}")]
    LaunchError(String),

    // ============================================================================
    // Spider/Crawler Errors
    // ============================================================================
    /// Spider crawl error
    #[error("Spider error: {0}")]
    SpiderError(String),

    /// Budget exceeded
    #[error("Budget exceeded: {resource}")]
    BudgetExceededError { resource: String },

    /// Robots.txt disallowed
    #[error("URL disallowed by robots.txt: {url}")]
    RobotsDisallowedError { url: String },

    // ============================================================================
    // Intelligence/LLM Errors
    // ============================================================================
    /// LLM provider error
    #[error("LLM error from {provider}: {message}")]
    LlmError { provider: String, message: String },

    /// All LLM providers failed
    #[error("All LLM providers failed in fallback chain")]
    AllProvidersFailed,

    /// LLM rate limit
    #[error("LLM rate limit exceeded")]
    LlmRateLimitError,

    // ============================================================================
    // Security Errors
    // ============================================================================
    /// Authentication error
    #[error("Authentication error: {0}")]
    AuthError(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded")]
    RateLimitError,

    /// Invalid API key
    #[error("Invalid API key")]
    InvalidApiKeyError,

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDeniedError(String),

    // ============================================================================
    // Configuration Errors
    // ============================================================================
    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Validation error
    #[error("Validation error in {field}: {message}")]
    ValidationError { field: String, message: String },

    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingFieldError(String),

    // ============================================================================
    // Cache Errors
    // ============================================================================
    /// Cache operation error
    #[error("Cache error: {0}")]
    CacheError(String),

    /// Cache key not found
    #[error("Cache key not found: {0}")]
    CacheKeyNotFoundError(String),

    // ============================================================================
    // Resource Errors
    // ============================================================================
    /// Resource not found
    #[error("Resource not found: {0}")]
    NotFoundError(String),

    /// Resource already exists
    #[error("Resource already exists: {0}")]
    AlreadyExistsError(String),

    /// Resource unavailable
    #[error("Resource unavailable: {0}")]
    UnavailableError(String),

    // ============================================================================
    // Internal Errors
    // ============================================================================
    /// Internal error with context
    #[error("Internal error: {context}")]
    Internal {
        context: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Generic error (catch-all)
    #[error("{0}")]
    Other(String),
}

impl RiptideError {
    /// Add context to an error.
    ///
    /// Wraps the error with additional context information.
    ///
    /// # Example
    ///
    /// ```
    /// # use riptide_facade::error::RiptideError;
    /// let err = RiptideError::FetchError("timeout".to_string());
    /// let err_with_context = err.with_context("Failed to fetch page");
    /// ```
    pub fn with_context<S: Into<String>>(self, context: S) -> Self {
        RiptideError::Internal {
            context: context.into(),
            source: Some(Box::new(self)),
        }
    }

    /// Check if error is retryable.
    ///
    /// Returns true if the operation can be safely retried.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            RiptideError::TimeoutError { .. }
                | RiptideError::ConnectionError(_)
                | RiptideError::PoolExhaustedError
                | RiptideError::LlmRateLimitError
                | RiptideError::RateLimitError
        )
    }

    /// Check if error is a client error (4xx).
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            RiptideError::HttpError { status, .. } if (400..500).contains(status)
        )
    }

    /// Check if error is a server error (5xx).
    pub fn is_server_error(&self) -> bool {
        matches!(
            self,
            RiptideError::HttpError { status, .. } if (500..600).contains(status)
        )
    }
}

// Conversion from common error types
impl From<std::io::Error> for RiptideError {
    fn from(err: std::io::Error) -> Self {
        RiptideError::Internal {
            context: "I/O error".to_string(),
            source: Some(Box::new(err)),
        }
    }
}

impl From<serde_json::Error> for RiptideError {
    fn from(err: serde_json::Error) -> Self {
        RiptideError::ParseError(err.to_string())
    }
}

impl From<url::ParseError> for RiptideError {
    fn from(err: url::ParseError) -> Self {
        RiptideError::ValidationError {
            field: "url".to_string(),
            message: err.to_string(),
        }
    }
}

// Conversions from riptide crates
#[cfg(feature = "scraper")]
impl From<anyhow::Error> for RiptideError {
    fn from(err: anyhow::Error) -> Self {
        RiptideError::Internal {
            context: "Anyhow error".to_string(),
            source: Some(Box::new(err)),
        }
    }
}

/// Result type alias for Riptide facade operations.
pub type Result<T> = std::result::Result<T, RiptideError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = RiptideError::FetchError("test error".to_string());
        assert_eq!(err.to_string(), "Fetch error: test error");
    }

    #[test]
    fn test_error_with_context() {
        let err = RiptideError::FetchError("timeout".to_string());
        let err = err.with_context("Failed to fetch page");
        assert!(err.to_string().contains("Failed to fetch page"));
    }

    #[test]
    fn test_retryable_errors() {
        assert!(RiptideError::TimeoutError { timeout_ms: 1000 }.is_retryable());
        assert!(RiptideError::ConnectionError("test".to_string()).is_retryable());
        assert!(!RiptideError::ConfigError("test".to_string()).is_retryable());
    }

    #[test]
    fn test_client_server_errors() {
        let client_err = RiptideError::HttpError {
            status: 404,
            message: "Not Found".to_string(),
        };
        assert!(client_err.is_client_error());
        assert!(!client_err.is_server_error());

        let server_err = RiptideError::HttpError {
            status: 500,
            message: "Internal Server Error".to_string(),
        };
        assert!(server_err.is_server_error());
        assert!(!server_err.is_client_error());
    }
}
