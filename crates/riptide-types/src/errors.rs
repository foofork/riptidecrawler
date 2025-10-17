//! Error types for RipTide extraction system

use std::time::SystemTimeError;
use thiserror::Error;

/// Core error types for the RipTide system with proper error handling and recovery strategies
#[derive(Error, Debug)]
pub enum CoreError {
    /// WASM-related errors
    #[error("WASM engine error: {message}")]
    WasmError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// WASM instance management errors
    #[error("WASM instance error: {message}")]
    WasmInstanceError {
        message: String,
        instance_id: Option<String>,
    },

    /// Memory management errors
    #[error("Memory management error: {message}")]
    MemoryError {
        message: String,
        current_usage_mb: Option<u64>,
        max_usage_mb: Option<u64>,
    },

    /// Circuit breaker errors
    #[error("Circuit breaker error: {message}")]
    CircuitBreakerError {
        message: String,
        state: Option<String>,
    },

    /// Time/Clock related errors
    #[error("Time error: {message}")]
    TimeError {
        message: String,
        #[source]
        source: Option<SystemTimeError>,
    },

    /// HTTP client initialization errors
    #[error("HTTP client error: {message}")]
    HttpClientError { message: String },

    /// Serialization/Deserialization errors
    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    ConfigError {
        message: String,
        field: Option<String>,
    },

    /// Resource exhaustion errors
    #[error("Resource exhaustion: {resource} - {message}")]
    ResourceExhaustion {
        resource: String,
        message: String,
        current: Option<u64>,
        limit: Option<u64>,
    },

    /// Recovery errors
    #[error("Recovery failed: {message}")]
    RecoveryError {
        message: String,
        attempts: u32,
        original_error: Option<String>,
    },

    /// Generic system error for fallback cases
    #[error("System error: {message}")]
    SystemError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl CoreError {
    /// Create a WASM engine error with source
    pub fn wasm_engine<E>(message: impl Into<String>, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::WasmError {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a WASM engine error without source
    pub fn wasm_engine_msg(message: impl Into<String>) -> Self {
        Self::WasmError {
            message: message.into(),
            source: None,
        }
    }

    /// Create a WASM instance error
    pub fn wasm_instance(message: impl Into<String>, instance_id: Option<String>) -> Self {
        Self::WasmInstanceError {
            message: message.into(),
            instance_id,
        }
    }

    /// Create a memory error with usage info
    pub fn memory(
        message: impl Into<String>,
        current_mb: Option<u64>,
        max_mb: Option<u64>,
    ) -> Self {
        Self::MemoryError {
            message: message.into(),
            current_usage_mb: current_mb,
            max_usage_mb: max_mb,
        }
    }

    /// Create a circuit breaker error
    pub fn circuit_breaker(message: impl Into<String>, state: Option<String>) -> Self {
        Self::CircuitBreakerError {
            message: message.into(),
            state,
        }
    }

    /// Create a time error with source
    pub fn time_error(message: impl Into<String>, source: Option<SystemTimeError>) -> Self {
        Self::TimeError {
            message: message.into(),
            source,
        }
    }

    /// Create an HTTP client error without source
    pub fn http_client_msg(message: impl Into<String>) -> Self {
        Self::HttpClientError {
            message: message.into(),
        }
    }

    /// Create a serialization error
    pub fn serialization_msg(message: impl Into<String>) -> Self {
        Self::SerializationError {
            message: message.into(),
        }
    }

    /// Create a resource exhaustion error
    pub fn resource_exhaustion(
        resource: impl Into<String>,
        message: impl Into<String>,
        current: Option<u64>,
        limit: Option<u64>,
    ) -> Self {
        Self::ResourceExhaustion {
            resource: resource.into(),
            message: message.into(),
            current,
            limit,
        }
    }

    /// Create a recovery error
    pub fn recovery(
        message: impl Into<String>,
        attempts: u32,
        original_error: Option<String>,
    ) -> Self {
        Self::RecoveryError {
            message: message.into(),
            attempts,
            original_error,
        }
    }

    /// Create a system error from anyhow::Error (to avoid trait conflicts)
    pub fn from_anyhow(error: anyhow::Error) -> Self {
        Self::SystemError {
            message: error.to_string(),
            source: None,
        }
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            CoreError::WasmError { .. } => false,
            CoreError::WasmInstanceError { .. } => true,
            CoreError::MemoryError { .. } => true,
            CoreError::CircuitBreakerError { .. } => true,
            CoreError::TimeError { .. } => false,
            CoreError::HttpClientError { .. } => true,
            CoreError::SerializationError { .. } => false,
            CoreError::ConfigError { .. } => false,
            CoreError::ResourceExhaustion { .. } => true,
            CoreError::RecoveryError { .. } => false,
            CoreError::SystemError { .. } => false,
        }
    }

    /// Get suggested recovery action
    pub fn recovery_suggestion(&self) -> &'static str {
        match self {
            CoreError::WasmError { .. } => "Check WASM engine configuration and component files",
            CoreError::WasmInstanceError { .. } => {
                "Retry instance creation or check resource availability"
            }
            CoreError::MemoryError { .. } => "Trigger garbage collection or increase memory limits",
            CoreError::CircuitBreakerError { .. } => {
                "Wait for circuit breaker cooldown or check service health"
            }
            CoreError::TimeError { .. } => "Check system clock and NTP synchronization",
            CoreError::HttpClientError { .. } => "Retry request or check network connectivity",
            CoreError::SerializationError { .. } => "Validate data format and schema",
            CoreError::ConfigError { .. } => "Check configuration values and environment variables",
            CoreError::ResourceExhaustion { .. } => "Free up resources or increase limits",
            CoreError::RecoveryError { .. } => "Manual intervention required",
            CoreError::SystemError { .. } => "Check system resources and dependencies",
        }
    }
}

/// Convenience type alias for Results using CoreError
pub type CoreResult<T> = Result<T, CoreError>;
