pub mod cache;
pub mod crawl;
pub mod extract;
pub mod health;
pub mod metrics;
pub mod search;
pub mod system_check;
pub mod validate;
pub mod wasm;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Extract content from a URL with optional confidence scoring
    Extract(ExtractArgs),

    /// Crawl a website
    Crawl(CrawlArgs),

    /// Search for content
    Search(SearchArgs),

    /// Cache management commands
    Cache {
        #[command(subcommand)]
        command: CacheCommands,
    },

    /// WASM management commands
    Wasm {
        #[command(subcommand)]
        command: WasmCommands,
    },

    /// Check system health
    Health,

    /// View metrics
    Metrics,

    /// Validate configuration
    Validate,

    /// Perform comprehensive system check
    SystemCheck,
}

#[derive(clap::Args)]
pub struct ExtractArgs {
    /// URL to extract content from
    #[arg(long)]
    pub url: String,

    /// Show confidence scores for extracted content
    #[arg(long)]
    pub show_confidence: bool,

    /// Strategy composition mode (chain, parallel, fallback)
    /// Examples:
    ///   --strategy chain:trek,css,regex
    ///   --strategy parallel:all
    ///   --strategy fallback:trek,css
    #[arg(long)]
    pub strategy: Option<String>,

    /// Specific extraction method (trek, css, llm, regex, auto)
    #[arg(long, default_value = "auto")]
    pub method: String,

    /// CSS selector for extraction
    #[arg(long)]
    pub selector: Option<String>,

    /// Regex pattern for extraction
    #[arg(long)]
    pub pattern: Option<String>,

    /// Output file path (optional)
    #[arg(long, short = 'f')]
    pub file: Option<String>,

    /// Include metadata in output
    #[arg(long)]
    pub metadata: bool,
}

#[derive(clap::Args)]
pub struct CrawlArgs {
    /// URL to crawl
    #[arg(long)]
    pub url: String,

    /// Maximum depth to crawl
    #[arg(long, default_value = "3")]
    pub depth: u32,

    /// Maximum pages to crawl
    #[arg(long, default_value = "100")]
    pub max_pages: u32,

    /// Follow external links
    #[arg(long)]
    pub follow_external: bool,

    /// Output directory for crawled content
    #[arg(long, short = 'd')]
    pub output_dir: Option<String>,

    /// Enable streaming mode
    #[arg(long)]
    pub stream: bool,
}

#[derive(clap::Args)]
pub struct SearchArgs {
    /// Search query
    #[arg(long)]
    pub query: String,

    /// Number of results to return
    #[arg(long, default_value = "10")]
    pub limit: u32,

    /// Search in specific domain
    #[arg(long)]
    pub domain: Option<String>,
}

#[derive(Subcommand)]
pub enum CacheCommands {
    /// Show cache status
    Status,

    /// Clear cache
    Clear {
        /// Clear cache for specific method only
        #[arg(long)]
        method: Option<String>,
    },

    /// Validate cache integrity
    Validate,

    /// Show cache statistics
    Stats,
}

#[derive(Subcommand)]
pub enum WasmCommands {
    /// Show WASM runtime information
    Info,

    /// Run WASM performance benchmarks
    Benchmark {
        /// Number of iterations
        #[arg(long, default_value = "100")]
        iterations: u32,
    },

    /// Show WASM instance health
    Health,
}
