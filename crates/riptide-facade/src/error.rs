//! Error types for the Riptide facade.

use thiserror::Error;

/// Result type for Riptide operations.
pub type RiptideResult<T> = Result<T, RiptideError>;

/// Comprehensive error type for Riptide facade operations.
#[derive(Debug, Error)]
pub enum RiptideError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Network/fetch error
    #[error("Fetch error: {0}")]
    Fetch(String),

    /// Extraction error
    #[error("Extraction error: {0}")]
    Extraction(String),

    /// Invalid URL
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),

    /// Timeout error
    #[error("Operation timed out")]
    Timeout,

    /// Generic error
    #[error("Riptide error: {0}")]
    Other(#[from] anyhow::Error),
}

impl RiptideError {
    /// Create a new configuration error.
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Create a new extraction error.
    pub fn extraction(msg: impl Into<String>) -> Self {
        Self::Extraction(msg.into())
    }

    /// Create a new spider error.
    pub fn spider(msg: impl Into<String>) -> Self {
        Self::Other(anyhow::anyhow!("Spider error: {}", msg.into()))
    }

    /// Create a new search error.
    pub fn search(msg: impl Into<String>) -> Self {
        Self::Other(anyhow::anyhow!("Search error: {}", msg.into()))
    }
}
