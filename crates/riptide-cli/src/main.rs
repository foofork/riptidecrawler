//! Riptide CLI - Phase 1 Minimal Implementation
//!
//! This is a minimal stub that compiles but doesn't provide full functionality yet.
//! Full implementation will be restored in Phase 2 based on the spec.

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(name = "riptide")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "RipTide - High-performance web crawler and content extraction CLI", long_about = None)]
struct Cli {
    /// RipTide API server URL
    #[arg(long, env = "RIPTIDE_API_URL", default_value = "http://localhost:8080")]
    api_url: String,

    /// API key for authentication (Bearer token)
    #[arg(long, env = "RIPTIDE_API_KEY")]
    api_key: Option<String>,

    /// Output format (json, text, table)
    #[arg(long, short = 'o', default_value = "text")]
    output: String,

    /// Verbose output
    #[arg(long, short = 'v')]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Extract content from a URL (Phase 1: Not yet implemented)
    Extract {
        /// URL to extract from
        #[arg(long)]
        url: String,
    },

    /// Check API health (Phase 1: Not yet implemented)
    Health,

    /// Show version info
    Version,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    let cli = Cli::parse();

    println!("RipTide CLI - Phase 1 (Minimal Implementation)");
    println!("Full functionality will be available in Phase 2");
    println!();

    match cli.command {
        Commands::Extract { url } => {
            println!("Extract command for URL: {}", url);
            println!("⚠️  Not yet implemented in Phase 1");
            println!("This will be implemented in Phase 2 based on the spec");
        }
        Commands::Health => {
            println!("Health check for API: {}", cli.api_url);
            println!("⚠️  Not yet implemented in Phase 1");
        }
        Commands::Version => {
            println!("Version: {}", env!("CARGO_PKG_VERSION"));
            println!("Build: Phase 1 - Minimal stub");
        }
    }

    Ok(())
}
