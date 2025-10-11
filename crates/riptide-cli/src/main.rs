mod client;
mod commands;
mod output;

use anyhow::Result;
use clap::Parser;
use commands::Commands;

#[derive(Parser)]
#[command(name = "riptide")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "RipTide - High-performance web crawler and content extraction CLI", long_about = None)]
struct Cli {
    /// RipTide API server URL
    #[arg(long, env = "RIPTIDE_API_URL", default_value = "http://localhost:8080")]
    api_url: String,

    /// API key for authentication
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

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    let cli = Cli::parse();

    // Create API client
    let client = client::RipTideClient::new(cli.api_url, cli.api_key)?;

    // Execute command
    match cli.command {
        Commands::Extract(args) => commands::extract::execute(client, args, &cli.output).await,
        Commands::Crawl(args) => commands::crawl::execute(client, args, &cli.output).await,
        Commands::Search(args) => commands::search::execute(client, args, &cli.output).await,
        Commands::Cache { command } => commands::cache::execute(client, command, &cli.output).await,
        Commands::Wasm { command } => commands::wasm::execute(client, command, &cli.output).await,
        Commands::Health => commands::health::execute(client, &cli.output).await,
        Commands::Metrics => commands::metrics::execute(client, &cli.output).await,
        Commands::Validate => commands::validate::execute(client, &cli.output).await,
        Commands::SystemCheck => commands::system_check::execute(client, &cli.output).await,
    }
}
