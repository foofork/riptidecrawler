//! Session management command - API-based authenticated crawling sessions
//!
//! Provides session management for authenticated crawling via the RipTide API.
//! Sessions maintain browser state and cookies across multiple requests.

use crate::client::ApiClient;
use crate::commands::session_api;
use anyhow::Result;
use clap::Args;

/// Session management arguments
#[derive(Args, Clone, Debug)]
pub struct SessionArgs {
    /// Session subcommand
    #[command(subcommand)]
    pub command: session_api::SessionApiCommands,
}

/// Execute session command by delegating to session_api module
///
/// This function acts as a thin wrapper that forwards the command to the
/// session_api implementation, which handles all API interactions.
///
/// # Arguments
/// * `client` - Configured API client for making requests
/// * `args` - Parsed session command arguments with subcommand
/// * `output_format` - Output format (json, text, table)
///
/// # Example
/// ```no_run
/// # use riptide_cli::{client::ApiClient, commands::session};
/// # async fn example() -> anyhow::Result<()> {
/// let client = ApiClient::new("http://localhost:8080".to_string(), None)?;
/// // Execute would be called by main.rs with parsed args
/// # Ok(())
/// # }
/// ```
pub async fn execute(client: ApiClient, args: SessionArgs, output_format: String) -> Result<()> {
    session_api::execute(client, args.command, &output_format).await
}
