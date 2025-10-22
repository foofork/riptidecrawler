use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod cli_test_harness;
mod comparison_tool;

use cli_test_harness::{TestHarness, TestUrls};
use comparison_tool::ComparisonTool;

#[derive(Parser)]
#[command(name = "webpage-extraction-tests")]
#[command(about = "Comprehensive webpage extraction testing suite", long_about = None)]
struct Cli {
    /// Path to the extraction binary
    #[arg(short, long, default_value = "../../target/release/eventmesh-cli")]
    binary: PathBuf,

    /// Output directory for results
    #[arg(short, long, default_value = "./results")]
    output_dir: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run extraction tests
    Run {
        /// Path to test URLs JSON file
        #[arg(short, long, default_value = "./test-urls.json")]
        urls: PathBuf,

        /// Extraction methods to test
        #[arg(short, long, value_delimiter = ',')]
        methods: Vec<String>,

        /// Timeout per test in seconds
        #[arg(short, long, default_value = "30")]
        timeout: u64,
    },

    /// Compare extraction methods
    Compare {
        /// Session ID to analyze
        session_id: String,
    },

    /// Diff two test sessions
    Diff {
        /// First session ID
        session1: String,

        /// Second session ID
        session2: String,
    },

    /// List available sessions
    List,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    // Create output directory if it doesn't exist
    std::fs::create_dir_all(&cli.output_dir)
        .context("Failed to create output directory")?;

    match cli.command {
        Commands::Run { urls, methods, timeout } => {
            run_tests(cli.binary, cli.output_dir, urls, methods, timeout).await?;
        }
        Commands::Compare { session_id } => {
            compare_methods(cli.output_dir, &session_id)?;
        }
        Commands::Diff { session1, session2 } => {
            diff_sessions(cli.output_dir, &session1, &session2)?;
        }
        Commands::List => {
            list_sessions(cli.output_dir)?;
        }
    }

    Ok(())
}

async fn run_tests(
    binary: PathBuf,
    output_dir: PathBuf,
    urls_path: PathBuf,
    methods: Vec<String>,
    _timeout: u64,
) -> Result<()> {
    println!("🧪 Webpage Extraction Test Suite");
    println!("=" .repeat(80));

    let harness = TestHarness::new(output_dir.clone(), binary);

    // Load test URLs
    println!("\n📋 Loading test URLs from: {}", urls_path.display());
    let test_urls = harness.load_test_urls(&urls_path).await
        .context("Failed to load test URLs")?;

    println!("   Found {} test URLs", test_urls.test_urls.len());

    // Determine methods to test
    let methods = if methods.is_empty() {
        vec![
            "jina".to_string(),
            "playwright".to_string(),
            "selenium".to_string(),
            "puppeteer".to_string(),
            "firecrawl".to_string(),
            "r2r".to_string(),
        ]
    } else {
        methods
    };

    println!("\n⚙️  Testing methods: {}", methods.join(", "));

    // Run test suite
    let session = harness.run_test_suite(&test_urls, &methods).await?;

    // Generate comparison report
    let comparison_tool = ComparisonTool::new(output_dir);
    let report = comparison_tool.compare_methods(&session)?;

    println!("\n");
    comparison_tool.print_report(&report);

    Ok(())
}

fn compare_methods(output_dir: PathBuf, session_id: &str) -> Result<()> {
    println!("🔍 Comparing Methods");
    println!("=" .repeat(80));

    let tool = ComparisonTool::new(output_dir);
    let session = tool.load_session(session_id)
        .context("Failed to load session")?;

    let report = tool.compare_methods(&session)?;
    tool.print_report(&report);

    Ok(())
}

fn diff_sessions(output_dir: PathBuf, session1: &str, session2: &str) -> Result<()> {
    println!("🔄 Comparing Sessions");
    println!("=" .repeat(80));

    let tool = ComparisonTool::new(output_dir);
    tool.diff_sessions(session1, session2)?;

    Ok(())
}

fn list_sessions(output_dir: PathBuf) -> Result<()> {
    println!("📋 Available Test Sessions");
    println!("=" .repeat(80));

    let mut sessions: Vec<_> = std::fs::read_dir(&output_dir)
        .context("Failed to read output directory")?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file() && path.extension()? == "json" {
                let filename = path.file_name()?.to_string_lossy().to_string();
                if filename.starts_with("test-session-") {
                    Some((filename, path))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    sessions.sort_by(|a, b| b.0.cmp(&a.0)); // Sort by name (timestamp) descending

    if sessions.is_empty() {
        println!("\nNo test sessions found in {}", output_dir.display());
        return Ok(());
    }

    println!("\nFound {} sessions:\n", sessions.len());

    for (filename, path) in sessions.iter().take(10) {
        // Try to read session metadata
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(session) = serde_json::from_str::<serde_json::Value>(&content) {
                let total = session["total_tests"].as_u64().unwrap_or(0);
                let success = session["successful_tests"].as_u64().unwrap_or(0);
                let rate = if total > 0 {
                    (success as f64 / total as f64) * 100.0
                } else {
                    0.0
                };

                println!("  📊 {}", filename);
                println!("     Tests: {}/{} ({:.1}%)", success, total, rate);

                if let Some(start_time) = session["start_time"].as_str() {
                    println!("     Time:  {}", start_time);
                }
                println!();
            }
        }
    }

    if sessions.len() > 10 {
        println!("... and {} more", sessions.len() - 10);
    }

    Ok(())
}
