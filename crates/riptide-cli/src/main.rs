mod api_client;
mod cache;
mod client;
mod commands;
mod config;
mod execution_mode;
mod job;
mod metrics;
mod output;
mod session;
mod validation_adapter;

use anyhow::Result;
use clap::Parser;
use commands::Commands;

// Phase 5 optimized executor - re-enabled after fixing global() methods
use commands::optimized_executor::OptimizedExecutor;

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

    /// Force direct/offline mode (bypass API)
    #[arg(long, env = "RIPTIDE_DIRECT")]
    direct: bool,

    /// API-only mode (no fallback to direct execution)
    #[arg(long, env = "RIPTIDE_API_ONLY")]
    api_only: bool,

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

    // Phase 5: Initialize global optimization manager
    // Re-enabled after completing Phase 4 global() method implementations
    let optimized_executor = match OptimizedExecutor::new().await {
        Ok(executor) => {
            tracing::info!("✓ Optimized executor initialized successfully");
            Some(executor)
        }
        Err(e) => {
            tracing::warn!(
                "Failed to initialize optimized executor: {}. Falling back to standard execution.",
                e
            );
            None
        }
    };

    // Determine execution mode based on flags and environment
    let execution_mode = execution_mode::get_execution_mode(cli.direct, cli.api_only);

    // Create API client and check health (unless in direct-only mode)
    let mut client = client::RipTideClient::new(cli.api_url.clone(), cli.api_key.clone())?;

    // Check API availability if mode allows API usage
    if execution_mode.allows_api() {
        match client.check_health().await {
            Ok(true) => {
                tracing::info!("API server is available at {}", cli.api_url);
            }
            Ok(false) => {
                if execution_mode.allows_fallback() {
                    tracing::warn!("API server unavailable, will use direct execution as fallback");
                } else {
                    anyhow::bail!(
                        "API server is unavailable at {} and fallback is disabled. Use --direct to force direct execution.",
                        cli.api_url
                    );
                }
            }
            Err(e) => {
                if execution_mode.allows_fallback() {
                    tracing::warn!(
                        "Failed to check API health: {}. Will use direct execution as fallback",
                        e
                    );
                } else {
                    anyhow::bail!(
                        "Failed to check API health: {}. API-only mode requires a healthy API server.",
                        e
                    );
                }
            }
        }
    } else {
        tracing::info!("Using direct execution mode (offline)");
    }

    // Execute command (with optional optimizations)
    let result = match cli.command {
        Commands::Extract(args) => {
            // Use optimized executor if available and local mode is enabled
            if let Some(ref _executor) = optimized_executor {
                if args.local {
                    tracing::info!("Using optimized extraction pipeline (local mode)");
                    // For now, use standard execution as the optimized path needs more integration
                    // TODO: Wire up _executor.execute_extract() for fully optimized path
                    commands::extract::execute(client, args, &cli.output).await
                } else {
                    commands::extract::execute(client, args, &cli.output).await
                }
            } else {
                commands::extract::execute(client, args, &cli.output).await
            }
        }
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
        Commands::Validate(args) => commands::validate::execute(client, args).await,
        Commands::SystemCheck(args) => commands::system_check::execute(client, args).await,
        Commands::Tables(args) => commands::tables::execute(client, args, &cli.output).await,
        Commands::Schema { command } => {
            commands::schema::execute(client, command, &cli.output).await
        }
        Commands::Pdf { command } => commands::pdf::execute(client, command, &cli.output).await,
        Commands::Job { command } => commands::job::execute(client, command, &cli.output).await,
        Commands::JobLocal { command } => commands::job_local::execute(command, &cli.output).await,
        Commands::Session { command } => commands::session::execute(command, &cli.output).await,
    };

    // Phase 5: Graceful shutdown of optimizations
    // Re-enabled after completing Phase 4 global() method implementations
    if let Some(executor) = optimized_executor {
        if let Err(e) = executor.shutdown().await {
            tracing::warn!("Error during optimized executor shutdown: {}", e);
        }
    }

    result
}
