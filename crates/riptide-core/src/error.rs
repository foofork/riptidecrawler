use std::time::SystemTimeError;
use thiserror::Error;

pub mod telemetry;

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
    HttpClientError {
        message: String,
        #[source]
        source: Option<reqwest::Error>,
    },

    /// Serialization/Deserialization errors
    #[error("Serialization error: {message}")]
    SerializationError {
        message: String,
        #[source]
        source: Option<serde_json::Error>,
    },

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

    /// Create an HTTP client error
    pub fn http_client<E>(message: impl Into<String>, source: E) -> Self
    where
        E: Into<reqwest::Error>,
    {
        Self::HttpClientError {
            message: message.into(),
            source: Some(source.into()),
        }
    }

    /// Create an HTTP client error without source
    pub fn http_client_msg(message: impl Into<String>) -> Self {
        Self::HttpClientError {
            message: message.into(),
            source: None,
        }
    }

    /// Create a serialization error
    pub fn serialization<E>(message: impl Into<String>, source: E) -> Self
    where
        E: Into<serde_json::Error>,
    {
        Self::SerializationError {
            message: message.into(),
            source: Some(source.into()),
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
            source: None, // anyhow::Error doesn't implement the required traits for source
        }
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            CoreError::WasmError { .. } => false, // Engine creation errors are not retryable
            CoreError::WasmInstanceError { .. } => true, // Instance creation might be retryable
            CoreError::MemoryError { .. } => true, // Memory pressure might be temporary
            CoreError::CircuitBreakerError { .. } => true, // Circuit breaker might close
            CoreError::TimeError { .. } => false, // Time errors are system issues
            CoreError::HttpClientError { .. } => true, // Network issues might be temporary
            CoreError::SerializationError { .. } => false, // Data format errors are permanent
            CoreError::ConfigError { .. } => false, // Config errors need fixing
            CoreError::ResourceExhaustion { .. } => true, // Resources might become available
            CoreError::RecoveryError { .. } => false, // Recovery already failed
            CoreError::SystemError { .. } => false, // Generic system errors
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

    /// Get telemetry context for error reporting
    pub fn telemetry_context(&self) -> Vec<(&'static str, String)> {
        let mut context = vec![
            ("error_type", format!("{:?}", std::mem::discriminant(self))),
            ("is_retryable", self.is_retryable().to_string()),
            (
                "recovery_suggestion",
                self.recovery_suggestion().to_string(),
            ),
        ];

        match self {
            CoreError::WasmError { message, .. } => {
                context.push(("wasm_error_message", message.clone()));
            }
            CoreError::WasmInstanceError {
                instance_id: Some(id),
                ..
            } => {
                context.push(("instance_id", id.clone()));
            }
            CoreError::WasmInstanceError {
                instance_id: None, ..
            } => {}
            CoreError::MemoryError {
                current_usage_mb,
                max_usage_mb,
                ..
            } => {
                if let Some(current) = current_usage_mb {
                    context.push(("current_memory_mb", current.to_string()));
                }
                if let Some(max) = max_usage_mb {
                    context.push(("max_memory_mb", max.to_string()));
                }
            }
            CoreError::CircuitBreakerError { state: Some(s), .. } => {
                context.push(("circuit_state", s.clone()));
            }
            CoreError::CircuitBreakerError { state: None, .. } => {}
            CoreError::ResourceExhaustion {
                resource,
                current,
                limit,
                ..
            } => {
                context.push(("resource_type", resource.clone()));
                if let Some(c) = current {
                    context.push(("current_usage", c.to_string()));
                }
                if let Some(l) = limit {
                    context.push(("limit", l.to_string()));
                }
            }
            CoreError::RecoveryError {
                attempts,
                original_error,
                ..
            } => {
                context.push(("recovery_attempts", attempts.to_string()));
                if let Some(orig) = original_error {
                    context.push(("original_error", orig.clone()));
                }
            }
            _ => {}
        }

        context
    }
}

/// Convenience type alias for Results using CoreError
pub type CoreResult<T> = Result<T, CoreError>;

// Note: From<anyhow::Error> implementation removed due to trait conflicts with wasmtime::Error
// Use CoreError::system_error() method instead for anyhow::Error conversion

/// Implement From for wasmtime::Error to CoreError conversion
impl From<wasmtime::Error> for CoreError {
    fn from(error: wasmtime::Error) -> Self {
        Self::WasmError {
            message: error.to_string(),
            source: None, // wasmtime::Error contains anyhow::Error which doesn't implement required traits
        }
    }
}

/// Implement From for std::error::Error to CoreError conversion
impl From<Box<dyn std::error::Error + Send + Sync>> for CoreError {
    fn from(error: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::SystemError {
            message: error.to_string(),
            source: None, // We can't convert back to anyhow::Error
        }
    }
}

/// Error recovery strategies
#[derive(Debug, Clone)]
pub struct RecoveryStrategy {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub backoff_multiplier: f64,
    pub max_delay_ms: u64,
}

impl Default for RecoveryStrategy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 100,
            backoff_multiplier: 2.0,
            max_delay_ms: 5000,
        }
    }
}

impl RecoveryStrategy {
    /// Calculate delay for retry attempt
    pub fn calculate_delay(&self, attempt: u32) -> std::time::Duration {
        let delay_ms =
            (self.base_delay_ms as f64 * self.backoff_multiplier.powi(attempt as i32)) as u64;
        let delay_ms = delay_ms.min(self.max_delay_ms);
        std::time::Duration::from_millis(delay_ms)
    }
}

/// Helper trait for converting common error types
pub trait IntoCore<T> {
    fn into_core(self) -> CoreResult<T>;
}

impl<T> IntoCore<T> for Result<T, wasmtime::Error> {
    fn into_core(self) -> CoreResult<T> {
        self.map_err(CoreError::from)
    }
}

impl<T> IntoCore<T> for Result<T, reqwest::Error> {
    fn into_core(self) -> CoreResult<T> {
        self.map_err(|e| CoreError::http_client("HTTP client operation failed", e))
    }
}

impl<T> IntoCore<T> for Result<T, serde_json::Error> {
    fn into_core(self) -> CoreResult<T> {
        self.map_err(|e| CoreError::serialization("JSON serialization failed", e))
    }
}

impl<T> IntoCore<T> for Result<T, SystemTimeError> {
    fn into_core(self) -> CoreResult<T> {
        self.map_err(|e| CoreError::time_error("System time error", Some(e)))
    }
}

/// Macro for creating error context
#[macro_export]
macro_rules! core_error {
    ($variant:ident, $msg:expr) => {
        $crate::error::CoreError::$variant { message: $msg.to_string() }
    };
    ($variant:ident, $msg:expr, $($field:ident: $value:expr),+) => {
        $crate::error::CoreError::$variant {
            message: $msg.to_string(),
            $($field: $value),+
        }
    };
}

/// Macro for error recovery with telemetry
#[macro_export]
macro_rules! with_recovery {
    ($operation:expr, $strategy:expr) => {{
        let mut last_error = None;

        for attempt in 0..$strategy.max_attempts {
            match $operation {
                Ok(result) => {
                    if attempt > 0 {
                        tracing::info!(
                            attempt = attempt + 1,
                            "Operation succeeded after retry"
                        );
                    }
                    return Ok(result);
                }
                Err(err) => {
                    if !err.is_retryable() || attempt == $strategy.max_attempts - 1 {
                        last_error = Some(err);
                        break;
                    }

                    let delay = $strategy.calculate_delay(attempt);
                    tracing::warn!(
                        attempt = attempt + 1,
                        delay_ms = delay.as_millis(),
                        error = %err,
                        "Operation failed, retrying"
                    );

                    tokio::time::sleep(delay).await;
                    last_error = Some(err);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            $crate::error::CoreError::recovery("Recovery failed with no error recorded", $strategy.max_attempts, None)
        }))
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = CoreError::wasm_engine_msg("Test WASM error");
        assert!(matches!(error, CoreError::WasmError { .. }));
        assert!(!error.is_retryable());
    }

    #[test]
    fn test_error_retryability() {
        let memory_error = CoreError::memory("Memory pressure", Some(1024), Some(2048));
        assert!(memory_error.is_retryable());

        let config_error = CoreError::ConfigError {
            message: "Invalid config".to_string(),
            field: Some("max_memory".to_string()),
        };
        assert!(!config_error.is_retryable());
    }

    #[test]
    fn test_recovery_strategy() {
        let strategy = RecoveryStrategy::default();
        let delay1 = strategy.calculate_delay(0);
        let delay2 = strategy.calculate_delay(1);

        assert_eq!(delay1.as_millis(), 100);
        assert_eq!(delay2.as_millis(), 200);
    }

    #[test]
    fn test_telemetry_context() {
        let error = CoreError::memory("Test memory error", Some(1024), Some(2048));
        let context = error.telemetry_context();

        assert!(context.iter().any(|(k, _)| k == &"current_memory_mb"));
        assert!(context.iter().any(|(k, _)| k == &"max_memory_mb"));
        assert!(context.iter().any(|(k, _)| k == &"is_retryable"));
    }
}
