//! Streaming-specific error types
//!
//! This module provides comprehensive error handling for streaming operations,
//! including transport failures, processing errors, and lifecycle errors.

use crate::error::RiptideError;
use std::fmt;

/// Streaming operation errors
///
/// Represents all possible errors that can occur during streaming operations.
#[derive(Debug, Clone)]
pub enum StreamingError {
    /// Connection to client failed
    ConnectionFailed {
        /// Reason for connection failure
        reason: String,
        /// Protocol being used
        protocol: String,
    },

    /// Stream processing failed
    ProcessingFailed {
        /// Error message
        message: String,
        /// URL being processed (if applicable)
        url: Option<String>,
        /// Underlying error context
        context: Option<String>,
    },

    /// Buffer overflow occurred
    BufferOverflow {
        /// Current buffer size
        size: usize,
        /// Maximum allowed size
        max_size: usize,
        /// Protocol being used
        protocol: String,
    },

    /// Invalid stream state transition
    InvalidState {
        /// Current state
        current: String,
        /// Attempted transition
        attempted: String,
        /// Reason transition is invalid
        reason: String,
    },

    /// Operation timed out
    Timeout {
        /// Operation that timed out
        operation: String,
        /// Timeout duration (seconds)
        timeout_secs: u64,
    },

    /// Operation cancelled
    Cancelled {
        /// Reason for cancellation
        reason: String,
    },

    /// Serialization error
    SerializationError {
        /// Error message
        message: String,
    },

    /// Protocol-specific error
    ProtocolError {
        /// Protocol name
        protocol: String,
        /// Error message
        message: String,
    },

    /// Configuration error
    ConfigError {
        /// Configuration field
        field: String,
        /// Error message
        message: String,
    },
}

impl StreamingError {
    /// Create a connection failed error
    pub fn connection(reason: impl Into<String>) -> Self {
        Self::ConnectionFailed {
            reason: reason.into(),
            protocol: "unknown".to_string(),
        }
    }

    /// Create a connection failed error with protocol
    pub fn connection_protocol(reason: impl Into<String>, protocol: impl Into<String>) -> Self {
        Self::ConnectionFailed {
            reason: reason.into(),
            protocol: protocol.into(),
        }
    }

    /// Create a processing failed error
    pub fn processing(message: impl Into<String>) -> Self {
        Self::ProcessingFailed {
            message: message.into(),
            url: None,
            context: None,
        }
    }

    /// Create a processing failed error with URL
    pub fn processing_url(message: impl Into<String>, url: impl Into<String>) -> Self {
        Self::ProcessingFailed {
            message: message.into(),
            url: Some(url.into()),
            context: None,
        }
    }

    /// Create a buffer overflow error
    pub fn buffer_overflow(size: usize, max_size: usize, protocol: impl Into<String>) -> Self {
        Self::BufferOverflow {
            size,
            max_size,
            protocol: protocol.into(),
        }
    }

    /// Create an invalid state error
    pub fn invalid_state(
        current: impl Into<String>,
        attempted: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::InvalidState {
            current: current.into(),
            attempted: attempted.into(),
            reason: reason.into(),
        }
    }

    /// Create a timeout error
    pub fn timeout(operation: impl Into<String>, timeout_secs: u64) -> Self {
        Self::Timeout {
            operation: operation.into(),
            timeout_secs,
        }
    }

    /// Create a cancelled error
    pub fn cancelled(reason: impl Into<String>) -> Self {
        Self::Cancelled {
            reason: reason.into(),
        }
    }

    /// Create a serialization error
    pub fn serialization(message: impl Into<String>) -> Self {
        Self::SerializationError {
            message: message.into(),
        }
    }

    /// Create a protocol error
    pub fn protocol(protocol: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ProtocolError {
            protocol: protocol.into(),
            message: message.into(),
        }
    }

    /// Create a configuration error
    pub fn config(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ConfigError {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            StreamingError::Timeout { .. } | StreamingError::BufferOverflow { .. }
        )
    }

    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            StreamingError::ConnectionFailed { .. } => ErrorSeverity::High,
            StreamingError::ProcessingFailed { .. } => ErrorSeverity::Medium,
            StreamingError::BufferOverflow { .. } => ErrorSeverity::Medium,
            StreamingError::InvalidState { .. } => ErrorSeverity::High,
            StreamingError::Timeout { .. } => ErrorSeverity::Low,
            StreamingError::Cancelled { .. } => ErrorSeverity::Low,
            StreamingError::SerializationError { .. } => ErrorSeverity::Medium,
            StreamingError::ProtocolError { .. } => ErrorSeverity::High,
            StreamingError::ConfigError { .. } => ErrorSeverity::High,
        }
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// Low severity - operation can continue
    Low,
    /// Medium severity - may affect stream quality
    Medium,
    /// High severity - stream should be terminated
    High,
}

impl fmt::Display for StreamingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StreamingError::ConnectionFailed { reason, protocol } => {
                write!(f, "Connection failed ({protocol}): {reason}")
            }
            StreamingError::ProcessingFailed {
                message,
                url,
                context,
            } => {
                write!(f, "Processing failed: {message}")?;
                if let Some(url) = url {
                    write!(f, " (URL: {url})")?;
                }
                if let Some(ctx) = context {
                    write!(f, " - {ctx}")?;
                }
                Ok(())
            }
            StreamingError::BufferOverflow {
                size,
                max_size,
                protocol,
            } => {
                write!(
                    f,
                    "Buffer overflow ({protocol}): {size} bytes exceeds max {max_size} bytes"
                )
            }
            StreamingError::InvalidState {
                current,
                attempted,
                reason,
            } => {
                write!(
                    f,
                    "Invalid state transition from '{current}' to '{attempted}': {reason}"
                )
            }
            StreamingError::Timeout {
                operation,
                timeout_secs,
            } => {
                write!(f, "Operation '{operation}' timed out after {timeout_secs}s")
            }
            StreamingError::Cancelled { reason } => {
                write!(f, "Operation cancelled: {reason}")
            }
            StreamingError::SerializationError { message } => {
                write!(f, "Serialization error: {message}")
            }
            StreamingError::ProtocolError { protocol, message } => {
                write!(f, "Protocol error ({protocol}): {message}")
            }
            StreamingError::ConfigError { field, message } => {
                write!(f, "Configuration error in '{field}': {message}")
            }
        }
    }
}

impl std::error::Error for StreamingError {}

// Conversion from serde_json::Error
impl From<serde_json::Error> for StreamingError {
    fn from(err: serde_json::Error) -> Self {
        StreamingError::serialization(err.to_string())
    }
}

// Conversion to RiptideError
impl From<StreamingError> for RiptideError {
    fn from(err: StreamingError) -> Self {
        RiptideError::custom(err.to_string())
    }
}

// Conversion from RiptideError
impl From<RiptideError> for StreamingError {
    fn from(err: RiptideError) -> Self {
        StreamingError::processing(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_error() {
        let err = StreamingError::connection("Network timeout");
        assert!(matches!(err, StreamingError::ConnectionFailed { .. }));
        assert_eq!(err.severity(), ErrorSeverity::High);
    }

    #[test]
    fn test_processing_error() {
        let err = StreamingError::processing_url("Failed to parse", "https://example.com");
        assert!(matches!(err, StreamingError::ProcessingFailed { .. }));
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_buffer_overflow() {
        let err = StreamingError::buffer_overflow(1024, 512, "websocket");
        assert!(matches!(err, StreamingError::BufferOverflow { .. }));
        assert!(err.is_retryable());
    }

    #[test]
    fn test_timeout_error() {
        let err = StreamingError::timeout("send_message", 30);
        assert!(matches!(err, StreamingError::Timeout { .. }));
        assert_eq!(err.severity(), ErrorSeverity::Low);
        assert!(err.is_retryable());
    }

    #[test]
    fn test_error_display() {
        let err = StreamingError::invalid_state("streaming", "paused", "Already completed");
        let display = format!("{}", err);
        assert!(display.contains("Invalid state transition"));
        assert!(display.contains("streaming"));
        assert!(display.contains("paused"));
    }

    #[test]
    fn test_serde_json_conversion() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let streaming_err: StreamingError = json_err.into();
        assert!(matches!(
            streaming_err,
            StreamingError::SerializationError { .. }
        ));
    }

    #[test]
    fn test_riptide_error_conversion() {
        let streaming_err = StreamingError::processing("Test error");
        let riptide_err: RiptideError = streaming_err.into();
        assert!(matches!(riptide_err, RiptideError::Custom(_)));
    }
}
