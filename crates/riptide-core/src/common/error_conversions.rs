//! Common error conversion utilities and traits.
//!
//! This module provides standardized error conversion patterns to reduce
//! duplicate From implementations and error handling code across the codebase.

use crate::error::CoreError;
use anyhow::Result;
use std::time::SystemTimeError;

/// Error conversion trait for consistent error handling
pub trait ErrorConverter<T> {
    type Output;
    type Error;

    fn convert_error(self) -> Result<T, Self::Error>;
}

/// Trait for converting errors into CoreError
pub trait IntoCore<T> {
    fn into_core(self) -> Result<T, CoreError>;
}

/// Trait for adding error context
pub trait WithErrorContext<T> {
    fn with_context<F>(self, f: F) -> Result<T, CoreError>
    where
        F: FnOnce() -> String;

    fn with_context_static(self, context: &'static str) -> Result<T, CoreError>;
}

/// Core error converter with common patterns
pub struct CoreErrorConverter;

impl CoreErrorConverter {
    /// Convert wasmtime errors to CoreError
    pub fn from_wasmtime(error: wasmtime::Error) -> CoreError {
        // Convert to string first to avoid trait bound conflicts
        CoreError::wasm_engine_msg(format!("WASM operation failed: {}", error))
    }

    /// Convert HTTP client errors to CoreError
    pub fn from_reqwest(error: reqwest::Error) -> CoreError {
        if error.is_timeout() {
            CoreError::time_error("HTTP request timed out", None)
        } else if error.is_connect() {
            CoreError::http_client("HTTP connection failed", error)
        } else {
            CoreError::http_client("HTTP client error", error)
        }
    }

    /// Convert JSON errors to CoreError
    pub fn from_serde_json(error: serde_json::Error) -> CoreError {
        CoreError::serialization("JSON serialization failed", error)
    }

    /// Convert system time errors to CoreError
    pub fn from_system_time(error: SystemTimeError) -> CoreError {
        CoreError::time_error("System time error", Some(error))
    }

    /// Convert anyhow errors to CoreError
    pub fn from_anyhow(error: anyhow::Error) -> CoreError {
        CoreError::from_anyhow(error)
    }

    /// Convert generic errors with context
    pub fn from_error_with_context<E>(error: E, context: &str) -> CoreError
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        CoreError::SystemError {
            message: format!("{}: {}", context, error),
            source: Some(Box::new(error)),
        }
    }

    /// Create memory error with usage details
    pub fn memory_error(message: &str, current_mb: Option<u64>, max_mb: Option<u64>) -> CoreError {
        CoreError::memory(message, current_mb, max_mb)
    }

    /// Create circuit breaker error with state
    pub fn circuit_breaker_error(message: &str, state: Option<String>) -> CoreError {
        CoreError::circuit_breaker(message, state)
    }

    /// Create resource exhaustion error
    pub fn resource_exhaustion_error(
        resource: &str,
        message: &str,
        current: Option<u64>,
        limit: Option<u64>,
    ) -> CoreError {
        CoreError::resource_exhaustion(resource, message, current, limit)
    }
}

/// API error converter (requires riptide-api dependency)
/// Note: This feature is for integration testing and is not used in normal builds
#[cfg(feature = "api-integration")]
pub struct ApiErrorConverter;

#[cfg(feature = "api-integration")]
impl ApiErrorConverter {
    /// Convert CoreError to a descriptive string format
    /// In real integration, this would convert to actual ApiError types
    pub fn from_core_error_to_string(error: CoreError) -> String {
        match error {
            CoreError::WasmError { message, .. } => {
                format!("WASM extraction failed: {}", message)
            }
            CoreError::HttpClientError { message, .. } => {
                format!("HTTP client error: {}", message)
            }
            CoreError::SerializationError { message, .. } => {
                format!("Serialization error: {}", message)
            }
            CoreError::ConfigError { message, .. } => {
                format!("Configuration error: {}", message)
            }
            CoreError::ResourceExhaustion {
                resource, message, ..
            } => {
                format!("Resource exhausted: {} - {}", resource, message)
            }
            CoreError::TimeError { message, .. } => {
                format!("Timeout error: {}", message)
            }
            CoreError::CircuitBreakerError { message, .. } => {
                format!("Circuit breaker: {}", message)
            }
            _ => error.to_string(),
        }
    }

    pub fn validation_error_string(message: &str) -> String {
        format!("Validation error: {}", message)
    }

    pub fn fetch_error_string(url: &str, message: &str) -> String {
        format!("Fetch error from {}: {}", url, message)
    }

    pub fn timeout_error_string(operation: &str, message: &str) -> String {
        format!("Timeout in {}: {}", operation, message)
    }
}

// Standard implementations for common error types
impl<T> IntoCore<T> for Result<T, wasmtime::Error> {
    fn into_core(self) -> Result<T, CoreError> {
        self.map_err(CoreErrorConverter::from_wasmtime)
    }
}

impl<T> IntoCore<T> for Result<T, reqwest::Error> {
    fn into_core(self) -> Result<T, CoreError> {
        self.map_err(CoreErrorConverter::from_reqwest)
    }
}

impl<T> IntoCore<T> for Result<T, serde_json::Error> {
    fn into_core(self) -> Result<T, CoreError> {
        self.map_err(CoreErrorConverter::from_serde_json)
    }
}

impl<T> IntoCore<T> for Result<T, SystemTimeError> {
    fn into_core(self) -> Result<T, CoreError> {
        self.map_err(CoreErrorConverter::from_system_time)
    }
}

// Note: Removed conflicting anyhow::Error implementation to avoid trait conflicts
// Use CoreErrorConverter::from_anyhow() directly instead

// WithErrorContext implementations
impl<T, E> WithErrorContext<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn with_context<F>(self, f: F) -> Result<T, CoreError>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| CoreErrorConverter::from_error_with_context(e, &f()))
    }

    fn with_context_static(self, context: &'static str) -> Result<T, CoreError> {
        self.map_err(|e| CoreErrorConverter::from_error_with_context(e, context))
    }
}

// Trait for API error conversions (if api feature is enabled)
// Note: Simplified for compilation compatibility
#[cfg(feature = "api-integration")]
pub trait IntoApiString<T> {
    fn into_api_string(self) -> Result<T, String>;
}

#[cfg(feature = "api-integration")]
impl<T> IntoApiString<T> for Result<T, CoreError> {
    fn into_api_string(self) -> Result<T, String> {
        self.map_err(ApiErrorConverter::from_core_error_to_string)
    }
}

#[cfg(feature = "api-integration")]
impl<T> IntoApiString<T> for Result<T, anyhow::Error> {
    fn into_api_string(self) -> Result<T, String> {
        self.map_err(|e| format!("Internal error: {}", e))
    }
}

#[cfg(feature = "api-integration")]
impl<T> IntoApiString<T> for Result<T, reqwest::Error> {
    fn into_api_string(self) -> Result<T, String> {
        self.map_err(|e| {
            if e.is_timeout() {
                format!("HTTP timeout: {}", e)
            } else if e.is_connect() {
                let url = e.url().map(|u| u.to_string()).unwrap_or_default();
                format!("Connection failed to {}: {}", url, e)
            } else {
                let url = e.url().map(|u| u.to_string()).unwrap_or_default();
                format!("HTTP error from {}: {}", url, e)
            }
        })
    }
}

/// Macro for easy error conversion
#[macro_export]
macro_rules! convert_error {
    ($result:expr, $error_type:ident) => {
        $result.map_err(|e| $crate::common::error_conversions::CoreErrorConverter::$error_type(e))
    };

    ($result:expr, $error_type:ident, $context:expr) => {
        $result.map_err(|e| {
            $crate::common::error_conversions::CoreErrorConverter::from_error_with_context(
                e, $context,
            )
        })
    };
}

/// Macro for creating context-aware error conversions
#[macro_export]
macro_rules! with_error_context {
    ($result:expr, $context:expr) => {
        $result.map_err(|e| {
            $crate::common::error_conversions::CoreErrorConverter::from_error_with_context(
                e, $context,
            )
        })
    };
}

/// Helper for creating common error patterns
pub struct ErrorPatterns;

impl ErrorPatterns {
    /// Create a validation error pattern
    pub fn validation_error(field: &str, reason: &str) -> CoreError {
        CoreError::ConfigError {
            message: format!("Validation failed for {}: {}", field, reason),
            field: Some(field.to_string()),
        }
    }

    /// Create a timeout error pattern
    pub fn timeout_error(operation: &str, duration_ms: u64) -> CoreError {
        CoreError::TimeError {
            message: format!(
                "Operation '{}' timed out after {}ms",
                operation, duration_ms
            ),
            source: None,
        }
    }

    /// Create a resource limit error pattern
    pub fn resource_limit_error(resource: &str, current: u64, limit: u64) -> CoreError {
        CoreError::resource_exhaustion(
            resource,
            format!("Limit exceeded: {} > {}", current, limit),
            Some(current),
            Some(limit),
        )
    }

    /// Create a dependency error pattern
    pub fn dependency_error(service: &str, reason: &str) -> CoreError {
        CoreError::SystemError {
            message: format!("Dependency '{}' failed: {}", service, reason),
            source: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_error_conversions() {
        // Test wasmtime error conversion
        let wasmtime_err = wasmtime::Error::msg("test error");
        let core_err = CoreErrorConverter::from_wasmtime(wasmtime_err);
        assert!(matches!(core_err, CoreError::WasmError { .. }));

        // Test memory error creation
        let mem_err = CoreErrorConverter::memory_error("Out of memory", Some(1024), Some(2048));
        assert!(matches!(mem_err, CoreError::MemoryError { .. }));
    }

    #[test]
    fn test_into_core_trait() {
        let result: Result<i32, SystemTimeError> = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::now() + std::time::Duration::from_secs(1))
            .map(|_| 42);

        let core_result = result.into_core();
        assert!(core_result.is_err());
        assert!(matches!(
            core_result.unwrap_err(),
            CoreError::TimeError { .. }
        ));
    }

    #[test]
    fn test_error_patterns() {
        let validation_err = ErrorPatterns::validation_error("url", "invalid format");
        assert!(matches!(validation_err, CoreError::ConfigError { .. }));

        let timeout_err = ErrorPatterns::timeout_error("extraction", 5000);
        assert!(matches!(timeout_err, CoreError::TimeError { .. }));

        let resource_err = ErrorPatterns::resource_limit_error("memory", 2048, 1024);
        assert!(matches!(resource_err, CoreError::ResourceExhaustion { .. }));
    }

    #[test]
    fn test_with_error_context() {
        use std::io;

        let result: Result<String, io::Error> =
            Err(io::Error::new(io::ErrorKind::NotFound, "file not found"));
        let context_result = result.with_context_static("loading configuration");

        assert!(context_result.is_err());
        let err = context_result.unwrap_err();
        assert!(err.to_string().contains("loading configuration"));
    }
}
