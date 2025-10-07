// TODO(P2): Streaming infrastructure - will be activated when routes are added
// STATUS: Complete error types prepared for streaming
// PLAN: Activate with streaming routes (see streaming/config.rs:1)
// EFFORT: Part of streaming feature implementation
// PRIORITY: Future feature
// BLOCKER: Same as streaming/config.rs
#![allow(dead_code)]

//! Error types for streaming operations.
//!
//! This module provides comprehensive error handling for all streaming operations
//! including buffer overflow, connection failures, and processing errors.

use crate::errors::ApiError;
use std::fmt;

/// Error types specific to streaming operations.
#[derive(Debug, thiserror::Error)]
pub enum StreamingError {
    /// Buffer overflow or capacity issues
    #[error("Buffer overflow: {message}")]
    BufferOverflow { message: String },

    /// Connection-related errors
    #[error("Connection error: {message}")]
    Connection { message: String },

    /// Serialization/deserialization errors
    #[error("Serialization error: {source}")]
    Serialization {
        #[from]
        source: serde_json::Error,
    },

    /// Channel communication errors
    #[error("Channel error: {message}")]
    Channel { message: String },

    /// Backpressure threshold exceeded
    #[error("Backpressure threshold exceeded for connection {connection_id}")]
    BackpressureExceeded { connection_id: String },

    /// Client disconnected unexpectedly
    #[error("Client disconnected: {reason}")]
    ClientDisconnected { reason: String },

    /// Processing pipeline error
    #[error("Pipeline processing failed: {source}")]
    Pipeline {
        #[from]
        source: anyhow::Error,
    },

    /// Invalid request format
    #[error("Invalid request: {message}")]
    InvalidRequest { message: String },

    /// Timeout during streaming operation
    #[error("Operation timed out after {seconds} seconds")]
    Timeout { seconds: u64 },
}

impl StreamingError {
    /// Create a buffer overflow error
    pub fn buffer_overflow(message: impl Into<String>) -> Self {
        Self::BufferOverflow {
            message: message.into(),
        }
    }

    /// Create a connection error
    pub fn connection(message: impl Into<String>) -> Self {
        Self::Connection {
            message: message.into(),
        }
    }

    /// Create a channel error
    pub fn channel(message: impl Into<String>) -> Self {
        Self::Channel {
            message: message.into(),
        }
    }

    /// Create a backpressure exceeded error
    pub fn backpressure_exceeded(connection_id: impl Into<String>) -> Self {
        Self::BackpressureExceeded {
            connection_id: connection_id.into(),
        }
    }

    /// Create a client disconnected error
    pub fn client_disconnected(reason: impl Into<String>) -> Self {
        Self::ClientDisconnected {
            reason: reason.into(),
        }
    }

    /// Create an invalid request error
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::InvalidRequest {
            message: message.into(),
        }
    }

    /// Create a timeout error
    pub fn timeout(seconds: u64) -> Self {
        Self::Timeout { seconds }
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::BufferOverflow { .. } => true,
            Self::Connection { .. } => true,
            Self::Channel { .. } => true,
            Self::BackpressureExceeded { .. } => false,
            Self::ClientDisconnected { .. } => false,
            Self::Pipeline { .. } => true,
            Self::InvalidRequest { .. } => false,
            Self::Timeout { .. } => true,
            Self::Serialization { .. } => false,
        }
    }

    /// Check if error indicates client issues
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            Self::BackpressureExceeded { .. }
                | Self::ClientDisconnected { .. }
                | Self::InvalidRequest { .. }
        )
    }
}

impl From<StreamingError> for ApiError {
    fn from(err: StreamingError) -> Self {
        match err {
            StreamingError::InvalidRequest { message } => ApiError::validation(message),
            StreamingError::Timeout { seconds } => ApiError::timeout(
                "streaming_operation",
                format!("Operation timed out after {} seconds", seconds),
            ),
            StreamingError::BackpressureExceeded { connection_id } => ApiError::RateLimited {
                message: format!(
                    "Connection {} is too slow, dropping messages",
                    connection_id
                ),
            },
            StreamingError::ClientDisconnected { reason } => ApiError::InternalError {
                message: format!("Client disconnected: {}", reason),
            },
            _ => ApiError::InternalError {
                message: err.to_string(),
            },
        }
    }
}

/// Result type for streaming operations
pub type StreamingResult<T> = Result<T, StreamingError>;

/// Connection state for error context
#[derive(Debug, Clone)]
pub struct ConnectionContext {
    pub session_id: String,
    pub client_type: ClientType,
    pub connected_at: std::time::Instant,
}

/// Client type for error context
#[derive(Debug, Clone)]
pub enum ClientType {
    WebSocket,
    ServerSentEvents,
    NdJson,
}

impl fmt::Display for ClientType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientType::WebSocket => write!(f, "websocket"),
            ClientType::ServerSentEvents => write!(f, "sse"),
            ClientType::NdJson => write!(f, "ndjson"),
        }
    }
}

/// Error recovery strategies
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// Retry the operation
    Retry { attempts: u32, delay_ms: u64 },
    /// Drop the message and continue
    Drop,
    /// Disconnect the client
    Disconnect,
    /// Fail the entire operation
    Fail,
}

impl StreamingError {
    /// Get the recommended recovery strategy for this error
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            Self::BufferOverflow { .. } => RecoveryStrategy::Drop,
            Self::Connection { .. } => RecoveryStrategy::Retry {
                attempts: 3,
                delay_ms: 1000,
            },
            Self::Channel { .. } => RecoveryStrategy::Retry {
                attempts: 2,
                delay_ms: 500,
            },
            Self::BackpressureExceeded { .. } => RecoveryStrategy::Disconnect,
            Self::ClientDisconnected { .. } => RecoveryStrategy::Disconnect,
            Self::Pipeline { .. } => RecoveryStrategy::Retry {
                attempts: 2,
                delay_ms: 500,
            },
            Self::InvalidRequest { .. } => RecoveryStrategy::Fail,
            Self::Timeout { .. } => RecoveryStrategy::Fail,
            Self::Serialization { .. } => RecoveryStrategy::Fail,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = StreamingError::buffer_overflow("Buffer full");
        assert!(matches!(err, StreamingError::BufferOverflow { .. }));
        assert!(err.is_retryable());
    }

    #[test]
    fn test_error_retryable() {
        assert!(StreamingError::connection("test").is_retryable());
        assert!(!StreamingError::invalid_request("test").is_retryable());
        assert!(!StreamingError::client_disconnected("test").is_retryable());
    }

    #[test]
    fn test_client_error() {
        assert!(StreamingError::backpressure_exceeded("test").is_client_error());
        assert!(StreamingError::invalid_request("test").is_client_error());
        assert!(!StreamingError::buffer_overflow("test").is_client_error());
    }

    #[test]
    fn test_recovery_strategy() {
        let err = StreamingError::buffer_overflow("test");
        assert!(matches!(err.recovery_strategy(), RecoveryStrategy::Drop));

        let err = StreamingError::backpressure_exceeded("test");
        assert!(matches!(
            err.recovery_strategy(),
            RecoveryStrategy::Disconnect
        ));
    }

    #[test]
    fn test_api_error_conversion() {
        let streaming_err = StreamingError::invalid_request("Bad format");
        let api_err: ApiError = streaming_err.into();
        assert!(matches!(api_err, ApiError::Validation { .. }));
    }
}
