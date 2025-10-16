mod cache;
mod client;
mod commands;
mod job;
mod metrics;
mod output;
mod session;

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

    /// Global WASM module path (can be overridden per-command)
    #[arg(long, env = "RIPTIDE_WASM_PATH", global = true)]
    wasm_path: Option<String>,

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
        Commands::Render(args) => commands::render::execute(args, &cli.output).await,
        Commands::Crawl(args) => commands::crawl::execute(client, args, &cli.output).await,
        Commands::Search(args) => commands::search::execute(client, args, &cli.output).await,
        Commands::Cache { command } => commands::cache::execute(client, command, &cli.output).await,
        Commands::Wasm { command } => commands::wasm::execute(client, command, &cli.output).await,
        Commands::Stealth { command } => commands::stealth::execute(command).await,
        Commands::Domain { command } => {
            commands::domain::execute(client, command, &cli.output).await
        }
        Commands::Health => commands::health::execute(client, &cli.output).await,
        Commands::Metrics { command } => match command {
            Some(commands::MetricsCommands::Show) | None => {
                commands::metrics::execute(client, &cli.output).await
            }
            Some(commands::MetricsCommands::Tail { interval, limit }) => {
                commands::metrics::tail(client, &interval, limit, &cli.output).await
            }
            Some(commands::MetricsCommands::Export {
                format,
                output,
                metric,
            }) => commands::metrics::export(client, &format, output, metric).await,
        },
        Commands::Validate => commands::validate::execute(client, &cli.output).await,
        Commands::SystemCheck => commands::system_check::execute(client, &cli.output).await,
        Commands::Tables(args) => commands::tables::execute(client, args, &cli.output).await,
        Commands::Schema { command } => {
            commands::schema::execute(client, command, &cli.output).await
        }
        Commands::Pdf { command } => commands::pdf::execute(client, command, &cli.output).await,
        Commands::Job { command } => commands::job::execute(client, command, &cli.output).await,
        Commands::JobLocal { command } => commands::job_local::execute(command, &cli.output).await,
        Commands::Session { command } => commands::session::execute(command, &cli.output).await,
    }
}
