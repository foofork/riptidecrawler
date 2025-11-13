//! Riptide CLI - High-performance web crawler and content extraction
//!
//! This is the main entry point for the RipTide CLI tool, providing a user-friendly
//! interface to the RipTide extraction and crawling API.

use anyhow::Result;
use clap::Parser;

mod client;
mod commands;
mod config;
mod error;
mod execution_mode;
mod output;

#[cfg(feature = "riptide-pdf")]
mod pdf_impl;

use error::ExitCode;

#[derive(Parser)]
#[command(name = "riptide")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "High-performance web crawler and content extraction CLI", long_about = None)]
struct Cli {
    /// RipTide API server URL
    #[arg(
        long,
        env = "RIPTIDE_BASE_URL",
        default_value = "http://localhost:8080"
    )]
    url: String,

    /// API key for authentication (Bearer token)
    #[arg(long, env = "RIPTIDE_API_KEY")]
    api_key: Option<String>,

    /// Output format (json, text, table)
    #[arg(long, short = 'o', default_value = "text")]
    output: String,

    /// Quiet mode - suppress progress output to stderr
    #[arg(long, short = 'q')]
    quiet: bool,

    /// Verbose mode - show detailed debug information
    #[arg(long, short = 'v')]
    verbose: bool,

    /// Use direct execution only (offline mode, no API calls)
    #[arg(long, global = true)]
    direct: bool,

    /// Use API only (no fallback to direct execution)
    #[arg(long, global = true)]
    api_only: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Extract content with advanced options (PRIMARY command)
    ///
    /// Extracts structured content from a single URL using intelligent parsing.
    Extract(commands::extract::ExtractArgs),

    /// Deep crawl with frontier management
    ///
    /// Performs recursive crawling starting from a seed URL with configurable depth.
    Spider(commands::spider::SpiderArgs),

    /// Search web with content extraction
    ///
    /// Searches the web for a query and extracts content from result pages.
    Search(commands::search::SearchArgs),

    /// Render JavaScript-heavy pages
    ///
    /// Uses headless browser rendering for JavaScript-dependent content.
    Render(commands::render::RenderArgs),

    /// System health diagnostics
    ///
    /// Checks API connectivity, pool health, and system resource usage.
    Doctor(commands::doctor::DoctorArgs),

    /// Session management for authenticated crawling
    ///
    /// Manage browser sessions for sites requiring authentication.
    Session(commands::session::SessionArgs),

    /// Manage extraction strategies
    ///
    /// List, inspect, and use different extraction strategies (native/wasm/headless).
    Strategies(commands::strategies::StrategyArgs),

    /// PDF extraction and processing
    ///
    /// Extract content from PDF files with optional OCR support.
    #[cfg(feature = "pdf")]
    Pdf(commands::pdf::PdfArgs),
}

#[tokio::main]
async fn main() {
    std::process::exit(match run().await {
        Ok(()) => ExitCode::Success.as_i32(),
        Err(e) => {
            eprintln!("Error: {}", e);
            // Map anyhow errors to exit codes
            ExitCode::UserError.as_i32()
        }
    });
}

async fn run() -> Result<()> {
    // Initialize logging based on verbose flag
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    let cli = Cli::parse();

    // Determine execution mode from flags
    let execution_mode = execution_mode::ExecutionMode::from_flags(cli.direct, cli.api_only);
    log::debug!("Execution mode: {}", execution_mode.description());

    // Create API client with base URL and optional API key
    let client = client::ApiClient::new(cli.url, cli.api_key)?;

    // Dispatch to appropriate command handler
    match cli.command {
        Commands::Extract(args) => commands::extract::execute(client, args, cli.output).await,
        Commands::Spider(args) => commands::spider::execute(client, args, cli.output).await,
        Commands::Search(args) => {
            commands::search::execute(client, args, cli.output, cli.quiet).await
        }
        Commands::Render(args) => commands::render::execute(client, args, cli.output).await,
        Commands::Doctor(args) => commands::doctor::execute(client, args, cli.output).await,
        Commands::Session(args) => commands::session::execute(client, args, cli.output).await,
        Commands::Strategies(args) => {
            commands::strategies::execute(client, args, cli.output).await
        }
        #[cfg(feature = "pdf")]
        Commands::Pdf(args) => commands::pdf::execute(&args).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli_structure() {
        // Ensures CLI structure is valid and help text works
        Cli::command().debug_assert();
    }

    #[test]
    fn test_default_values() {
        // Ensure clean environment for this test
        std::env::remove_var("RIPTIDE_BASE_URL");
        std::env::remove_var("RIPTIDE_API_KEY");

        let cli = Cli::parse_from(["riptide", "extract", "https://example.com"]);
        assert_eq!(cli.url, "http://localhost:8080");
        assert_eq!(cli.output, "text");
        assert!(!cli.quiet);
        assert!(!cli.verbose);
        assert!(!cli.direct);
        assert!(!cli.api_only);
    }

    #[test]
    fn test_env_var_override() {
        std::env::set_var("RIPTIDE_BASE_URL", "https://api.example.com");
        std::env::set_var("RIPTIDE_API_KEY", "test-key-123");

        let cli = Cli::parse_from(["riptide", "extract", "https://example.com"]);
        assert_eq!(cli.url, "https://api.example.com");
        assert_eq!(cli.api_key, Some("test-key-123".to_string()));

        std::env::remove_var("RIPTIDE_BASE_URL");
        std::env::remove_var("RIPTIDE_API_KEY");
    }

    #[test]
    fn test_all_commands_present() {
        let extract = Cli::parse_from(["riptide", "extract", "https://example.com"]);
        assert!(matches!(extract.command, Commands::Extract(_)));

        let spider = Cli::parse_from(["riptide", "spider", "https://example.com"]);
        assert!(matches!(spider.command, Commands::Spider(_)));

        let search = Cli::parse_from(["riptide", "search", "test"]);
        assert!(matches!(search.command, Commands::Search(_)));

        let render = Cli::parse_from(["riptide", "render", "https://example.com"]);
        assert!(matches!(render.command, Commands::Render(_)));

        let doctor = Cli::parse_from(["riptide", "doctor"]);
        assert!(matches!(doctor.command, Commands::Doctor(_)));

        let session = Cli::parse_from(["riptide", "session", "list"]);
        assert!(matches!(session.command, Commands::Session(_)));
    }
}
