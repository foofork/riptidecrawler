//! Riptide CLI library
//!
//! This library provides the core functionality for the RipTide CLI tool,
//! exposing modules for API interaction, command execution, output formatting,
//! configuration management, and error handling.
//!
//! # Architecture
//!
//! The CLI is organized into several key modules:
//!
//! - **client**: HTTP client for interacting with the RipTide API
//! - **commands**: Command handlers for each CLI subcommand (extract, spider, search, etc.)
//! - **output**: Output formatters for JSON, text, and table formats
//! - **error**: Error types and exit code mapping
//! - **config**: Configuration file management for persistent settings
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use riptide_cli::client::ApiClient;
//! use riptide_cli::commands::extract;
//!
//! async fn extract_example() -> anyhow::Result<()> {
//!     let client = ApiClient::new(
//!         "http://localhost:8080".to_string(),
//!         Some("api-key".to_string())
//!     )?;
//!
//!     let args = extract::ExtractArgs {
//!         url: "https://example.com".to_string(),
//!         ..Default::default()
//!     };
//!
//!     extract::execute(client, args, "json".to_string()).await?;
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod commands;
pub mod config;
pub mod error;
pub mod output;

// Legacy module exports (for backward compatibility during refactoring)
pub mod api_client {
    //! Legacy API client module - redirects to new client module
    pub use crate::client::*;
}

// Re-export commonly used types for convenience
pub use client::ApiClient;
pub use commands::config::Config;
pub use error::{CliError, ExitCode};

/// CLI version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default API endpoint
pub const DEFAULT_API_URL: &str = "http://localhost:8080";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_url() {
        assert_eq!(DEFAULT_API_URL, "http://localhost:8080");
    }
}
