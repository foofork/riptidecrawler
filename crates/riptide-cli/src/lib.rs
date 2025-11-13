//! Riptide CLI library
//!
//! This library provides the core functionality for the RipTide CLI tool,
//! exposing modules for API interaction, command execution, output formatting,
//! and error handling.
//!
//! # Architecture
//!
//! The CLI is organized into several key modules:
//!
//! - **client**: HTTP client for interacting with the RipTide API
//! - **commands**: Command handlers for each CLI subcommand (extract, spider, search, etc.)
//! - **output**: Output formatters for JSON, text, and table formats
//! - **error**: Error types and exit code mapping
//! - **config**: Output directory helpers and environment variable management
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
//!         urls: vec!["https://example.com".to_string()],
//!         strategy: "multi".to_string(),
//!         selector: None,
//!         pattern: None,
//!         quality_threshold: 0.7,
//!         timeout: 30000,
//!         concurrency: 5,
//!         cache: "auto".to_string(),
//!         output_file: None,
//!     };
//!
//!     extract::execute(client, args, "json".to_string()).await?;
//!     Ok(())
//! }
//! ```

#![allow(dead_code)]

pub mod client;
pub mod commands;
pub mod config;
pub mod error;
pub mod output;

// Supporting modules for CLI operations
pub mod api_wrapper;
pub mod execution_mode;
pub mod validation_adapter;

#[cfg(feature = "riptide-pdf")]
pub mod pdf_impl;

// Re-export commonly used types for convenience
pub use client::ApiClient;
pub use error::ExitCode;

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
