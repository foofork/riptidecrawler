use super::CoreError;
// telemetry_info! is now replaced with tracing::info!
use tracing::{error, warn};

/// Error telemetry integration for comprehensive error tracking and analysis
pub struct ErrorTelemetry;

impl ErrorTelemetry {
    /// Report error with full telemetry context
    pub fn report_error(error: &CoreError, operation: &str, additional_context: &[(&str, String)]) {
        let mut context = error.telemetry_context();
        context.push(("operation", operation.to_string()));
        context.extend_from_slice(additional_context);

        // Log error with structured context
        if error.is_retryable() {
            warn!(
                error = %error,
                operation = operation,
                "Retryable error occurred"
            );
        } else {
            error!(
                error = %error,
                operation = operation,
                "Non-retryable error occurred"
            );
        }

        // Send telemetry info with all context
        let context_vec: Vec<_> = context.iter().map(|(k, v)| (*k, v.as_str())).collect();

        // Use tracing for structured logging instead of telemetry_info!
        for (key, value) in context_vec {
            tracing::info!(key = key, value = value, "error_occurred");
        }
    }

    /// Report error recovery attempt
    pub fn report_recovery_attempt(
        error: &CoreError,
        operation: &str,
        attempt: u32,
        max_attempts: u32,
        delay_ms: u64,
    ) {
        warn!(
            error = %error,
            operation = operation,
            attempt = attempt,
            max_attempts = max_attempts,
            delay_ms = delay_ms,
            "Attempting error recovery"
        );

        // Use tracing for structured logging
        tracing::info!(
            operation = operation,
            error_type = format!("{:?}", std::mem::discriminant(error)),
            attempt = attempt,
            max_attempts = max_attempts,
            delay_ms = delay_ms,
            "error_recovery_attempt"
        );
    }

    /// Report successful recovery
    pub fn report_recovery_success(
        original_error: &CoreError,
        operation: &str,
        attempts_used: u32,
        total_duration_ms: u64,
    ) {
        warn!(
            operation = operation,
            attempts_used = attempts_used,
            total_duration_ms = total_duration_ms,
            "Error recovery successful"
        );

        tracing::info!(
            operation = operation,
            original_error_type = format!("{:?}", std::mem::discriminant(original_error)),
            attempts_used = attempts_used,
            total_duration_ms = total_duration_ms,
            recovery_suggestion = original_error.recovery_suggestion(),
            "error_recovery_success"
        );
    }

    /// Report recovery failure
    pub fn report_recovery_failure(
        original_error: &CoreError,
        operation: &str,
        attempts_used: u32,
        final_error: &CoreError,
    ) {
        error!(
            operation = operation,
            attempts_used = attempts_used,
            original_error = %original_error,
            final_error = %final_error,
            "Error recovery failed after all attempts"
        );

        tracing::info!(
            operation = operation,
            original_error_type = format!("{:?}", std::mem::discriminant(original_error)),
            final_error_type = format!("{:?}", std::mem::discriminant(final_error)),
            attempts_used = attempts_used,
            recovery_suggestion = original_error.recovery_suggestion(),
            "error_recovery_failure"
        );
    }

    /// Report critical error that requires immediate attention
    pub fn report_critical_error(
        error: &CoreError,
        operation: &str,
        system_state: &[(&str, String)],
    ) {
        let mut context = error.telemetry_context();
        context.push(("operation", operation.to_string()));
        context.push(("severity", "critical".to_string()));
        context.extend_from_slice(system_state);

        error!(
            error = %error,
            operation = operation,
            "CRITICAL ERROR: Immediate attention required"
        );

        // Fixed critical error telemetry format
        for (key, value) in context {
            tracing::error!(key = key, value = value, "critical_error");
        }
    }

    /// Report panic prevention (when a potential panic was caught and handled)
    pub fn report_panic_prevention(
        operation: &str,
        potential_panic_reason: &str,
        recovery_action: &str,
    ) {
        warn!(
            operation = operation,
            potential_panic_reason = potential_panic_reason,
            recovery_action = recovery_action,
            "Potential panic prevented through error handling"
        );

        tracing::info!(
            operation = operation,
            potential_panic_reason = potential_panic_reason,
            recovery_action = recovery_action,
            prevention_successful = "true",
            "panic_prevention"
        );
    }

    /// Report error patterns for analysis
    pub fn report_error_pattern(
        error_type: &str,
        frequency: u32,
        time_window_mins: u32,
        affected_operations: &[&str],
    ) {
        warn!(
            error_type = error_type,
            frequency = frequency,
            time_window_mins = time_window_mins,
            "Error pattern detected"
        );

        tracing::info!(
            error_type = error_type,
            frequency = frequency,
            time_window_mins = time_window_mins,
            affected_operations = affected_operations.join(","),
            "error_pattern_detected"
        );
    }
}

/// Convenience macro for error telemetry reporting
#[macro_export]
macro_rules! report_error {
    ($error:expr, $operation:expr) => {
        $crate::error::telemetry::ErrorTelemetry::report_error($error, $operation, &[]);
    };
    ($error:expr, $operation:expr, $($key:expr => $value:expr),+) => {
        $crate::error::telemetry::ErrorTelemetry::report_error(
            $error,
            $operation,
            &[$(($key, $value.to_string())),+]
        );
    };
}

/// Convenience macro for critical error reporting
#[macro_export]
macro_rules! report_critical_error {
    ($error:expr, $operation:expr) => {
        $crate::error::telemetry::ErrorTelemetry::report_critical_error($error, $operation, &[]);
    };
    ($error:expr, $operation:expr, $($key:expr => $value:expr),+) => {
        $crate::error::telemetry::ErrorTelemetry::report_critical_error(
            $error,
            $operation,
            &[$(($key, $value.to_string())),+]
        );
    };
}

/// Convenience macro for panic prevention reporting
#[macro_export]
macro_rules! report_panic_prevention {
    ($operation:expr, $reason:expr, $action:expr) => {
        $crate::error::telemetry::ErrorTelemetry::report_panic_prevention(
            $operation, $reason, $action,
        );
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_telemetry_reporting() {
        let error = CoreError::memory("Test memory error", Some(1024), Some(2048));

        // Test basic error reporting
        ErrorTelemetry::report_error(&error, "test_operation", &[]);

        // Test error reporting with additional context
        ErrorTelemetry::report_error(
            &error,
            "test_operation",
            &[("test_key", "test_value".to_string())],
        );
    }

    #[test]
    fn test_recovery_reporting() {
        let error = CoreError::memory("Test memory error", Some(1024), Some(2048));

        ErrorTelemetry::report_recovery_attempt(&error, "test_operation", 1, 3, 100);
        ErrorTelemetry::report_recovery_success(&error, "test_operation", 2, 500);

        let final_error = CoreError::wasm_engine_msg("Final error");
        ErrorTelemetry::report_recovery_failure(&error, "test_operation", 3, &final_error);
    }

    #[test]
    fn test_critical_error_reporting() {
        let error = CoreError::wasm_engine_msg("Critical WASM error");

        ErrorTelemetry::report_critical_error(
            &error,
            "wasm_initialization",
            &[("system_memory_mb", "8192".to_string())],
        );
    }

    #[test]
    fn test_panic_prevention_reporting() {
        ErrorTelemetry::report_panic_prevention(
            "vector_remove",
            "index_out_of_bounds",
            "bounds_check_added",
        );
    }
}
