//! Error handling for the Riptide CLI
//!
//! This module defines all error types and exit codes for the CLI,
//! following the error mapping specified in cli-spec/cli.yaml

use thiserror::Error;

/// CLI error types with detailed context
#[derive(Error, Debug)]
pub enum CliError {
    /// API connection error (network issues, timeouts, DNS failures)
    #[error("Failed to connect to API: {0}")]
    ApiConnection(String),

    /// API error with HTTP status code and message
    #[error("API error ({status}): {message}")]
    ApiError { status: u16, message: String },

    /// Configuration error (invalid config, missing values, parse errors)
    #[error("Configuration error: {0}")]
    Config(String),

    /// Network error (connection refused, timeout, DNS)
    #[error("Network error: {0}")]
    Network(String),

    /// Parse error (invalid JSON, malformed response)
    #[error("Parse error: {0}")]
    Parse(String),

    /// Invalid arguments (clap validation errors)
    #[error("Invalid arguments: {0}")]
    InvalidArgs(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// HTTP request error
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON parsing error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// YAML parsing error
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

/// Exit codes for CLI following POSIX conventions and cli-spec/cli.yaml
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    /// Success (0)
    Success = 0,

    /// User error: 4xx status codes, network issues, config errors (1)
    UserError = 1,

    /// Server error: 5xx status codes (2)
    ServerError = 2,

    /// Invalid arguments: clap validation errors (3)
    InvalidArgs = 3,
}

impl ExitCode {
    /// Convert exit code to integer
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

/// Map CliError to appropriate exit code
///
/// Error mapping follows cli-spec/cli.yaml:
/// - 4xx HTTP codes → UserError (1)
/// - 5xx HTTP codes → ServerError (2)
/// - Network errors → UserError (1)
/// - Config errors → UserError (1)
/// - Invalid args → InvalidArgs (3)
pub fn map_error_to_exit_code(error: &CliError) -> ExitCode {
    match error {
        // API errors mapped by HTTP status code
        CliError::ApiError { status, .. } => {
            match status {
                // 4xx client errors → UserError
                400..=499 => ExitCode::UserError,
                // 5xx server errors → ServerError
                500..=599 => ExitCode::ServerError,
                // Unexpected status codes → ServerError
                _ => ExitCode::ServerError,
            }
        }

        // Network and connection errors → UserError
        CliError::ApiConnection(_) => ExitCode::UserError,
        CliError::Network(_) => ExitCode::UserError,
        CliError::Http(_) => ExitCode::UserError,

        // Configuration errors → UserError
        CliError::Config(_) => ExitCode::UserError,
        CliError::Yaml(_) => ExitCode::UserError,

        // Parse errors → UserError
        CliError::Parse(_) => ExitCode::UserError,
        CliError::Json(_) => ExitCode::UserError,

        // Invalid arguments → InvalidArgs
        CliError::InvalidArgs(_) => ExitCode::InvalidArgs,

        // I/O errors → UserError
        CliError::Io(_) => ExitCode::UserError,
    }
}

/// Convert HTTP status code to CliError
pub fn http_status_to_error(status: u16, message: String) -> CliError {
    CliError::ApiError { status, message }
}

/// Convert reqwest::Error to CliError with proper categorization
pub fn reqwest_to_cli_error(error: reqwest::Error) -> CliError {
    if error.is_timeout() {
        CliError::Network("Request timeout".to_string())
    } else if error.is_connect() {
        CliError::ApiConnection(error.to_string())
    } else if let Some(status) = error.status() {
        CliError::ApiError {
            status: status.as_u16(),
            message: error.to_string(),
        }
    } else {
        CliError::Http(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_code_mapping_4xx() {
        let error = CliError::ApiError {
            status: 404,
            message: "Not found".to_string(),
        };
        assert_eq!(map_error_to_exit_code(&error), ExitCode::UserError);
    }

    #[test]
    fn test_exit_code_mapping_5xx() {
        let error = CliError::ApiError {
            status: 500,
            message: "Internal server error".to_string(),
        };
        assert_eq!(map_error_to_exit_code(&error), ExitCode::ServerError);
    }

    #[test]
    fn test_exit_code_mapping_network() {
        let error = CliError::Network("Connection refused".to_string());
        assert_eq!(map_error_to_exit_code(&error), ExitCode::UserError);
    }

    #[test]
    fn test_exit_code_mapping_config() {
        let error = CliError::Config("Invalid config".to_string());
        assert_eq!(map_error_to_exit_code(&error), ExitCode::UserError);
    }

    #[test]
    fn test_exit_code_mapping_invalid_args() {
        let error = CliError::InvalidArgs("Missing required argument".to_string());
        assert_eq!(map_error_to_exit_code(&error), ExitCode::InvalidArgs);
    }

    #[test]
    fn test_http_status_to_error() {
        let error = http_status_to_error(404, "Not found".to_string());
        match error {
            CliError::ApiError { status, message } => {
                assert_eq!(status, 404);
                assert_eq!(message, "Not found");
            }
            _ => panic!("Expected ApiError"),
        }
    }

    #[test]
    fn test_exit_code_as_i32() {
        assert_eq!(ExitCode::Success.as_i32(), 0);
        assert_eq!(ExitCode::UserError.as_i32(), 1);
        assert_eq!(ExitCode::ServerError.as_i32(), 2);
        assert_eq!(ExitCode::InvalidArgs.as_i32(), 3);
    }
}
