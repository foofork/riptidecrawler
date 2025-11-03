//! Error handling for the Riptide CLI
//!
//! The thin CLI uses anyhow::Result for simple error handling.
//! All business logic and complex error handling is in the API server.

/// Exit codes for CLI following POSIX conventions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    /// Success (0)
    Success = 0,

    /// User error: 4xx status codes, network issues, config errors (1)
    UserError = 1,
}

impl ExitCode {
    /// Convert exit code to integer
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}
