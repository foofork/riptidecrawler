pub mod cache;
pub mod crawl;
pub mod domain;
pub mod extract;
pub mod health;
pub mod job;
pub mod metrics;
pub mod pdf;
pub mod render;
pub mod schema;
pub mod search;
pub mod session;
pub mod stealth;
pub mod system_check;
pub mod tables;
pub mod validate;
pub mod wasm;

use clap::Subcommand;
use domain::DomainCommands;
use job::JobCommands;
use pdf::PdfCommands;
use schema::SchemaCommands;
use session::SessionCommands;

#[derive(Subcommand)]
pub enum Commands {
    /// Extract content from a URL with optional confidence scoring
    Extract(ExtractArgs),

    /// Render a page with headless browser capabilities
    Render(render::RenderArgs),

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

    /// Stealth configuration and testing
    Stealth {
        #[command(subcommand)]
        command: StealthCommands,
    },

    /// Domain profile management
    Domain {
        #[command(subcommand)]
        command: DomainCommands,
    },

    /// Check system health
    Health,

    /// View metrics
    Metrics {
        #[command(subcommand)]
        command: Option<MetricsCommands>,
    },

    /// Validate configuration
    Validate,

    /// Perform comprehensive system check
    SystemCheck,

    /// Extract tables from HTML content
    Tables(TablesArgs),

    /// Schema management commands
    Schema {
        #[command(subcommand)]
        command: SchemaCommands,
    },

    /// PDF processing commands
    Pdf {
        #[command(subcommand)]
        command: PdfCommands,
    },

    /// Job management commands
    Job {
        #[command(subcommand)]
        command: JobCommands,
    },

    /// Session management commands
    Session {
        #[command(subcommand)]
        command: SessionCommands,
    },
}

#[derive(clap::Args)]
pub struct ExtractArgs {
    /// URL to extract content from
    #[arg(long)]
    pub url: Option<String>,

    /// Read HTML from a file
    #[arg(long)]
    pub input_file: Option<String>,

    /// Read HTML from stdin
    #[arg(long)]
    pub stdin: bool,

    /// Use local WASM extraction (no API server required)
    #[arg(long)]
    pub local: bool,

    /// Show confidence scores for extracted content
    #[arg(long)]
    pub show_confidence: bool,

    /// Strategy composition mode (chain, parallel, fallback)
    /// Examples:
    ///   --strategy chain:wasm,css,regex
    ///   --strategy parallel:all
    ///   --strategy fallback:wasm,css
    #[arg(long)]
    pub strategy: Option<String>,

    /// Specific extraction method (wasm, css, llm, regex, auto)
    #[arg(long, default_value = "auto")]
    pub method: String,

    /// Extraction engine (auto, raw, wasm, headless)
    /// - auto: Automatically select based on content
    /// - raw: Pure HTTP fetch (no JavaScript execution)
    /// - wasm: WASM-based extraction (fast, local)
    /// - headless: Browser-based extraction (for JavaScript-heavy sites)
    #[arg(long, default_value = "auto")]
    pub engine: String,

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

    // Stealth Options
    /// Stealth level for anti-detection (none, low, medium, high)
    #[arg(long, value_parser = ["none", "low", "medium", "high"])]
    pub stealth_level: Option<String>,

    /// Custom user agent string
    #[arg(long)]
    pub user_agent: Option<String>,

    /// Enable request timing randomization
    #[arg(long)]
    pub randomize_timing: bool,

    /// Enable behavior simulation (mouse movements, scrolling)
    #[arg(long)]
    pub simulate_behavior: bool,

    /// Enable JavaScript-based fingerprint countermeasures
    #[arg(long)]
    pub fingerprint_evasion: bool,

    /// Proxy URL (e.g., http://proxy.example.com:8080)
    #[arg(long)]
    pub proxy: Option<String>,

    // WASM Configuration Options
    /// Path to WASM module (overrides config and environment)
    #[arg(long, env = "RIPTIDE_WASM_PATH")]
    pub wasm_path: Option<String>,

    /// Skip WASM module loading entirely (fallback to API-only mode)
    #[arg(long)]
    pub no_wasm: bool,

    /// WASM initialization timeout in milliseconds
    #[arg(long, default_value = "5000")]
    pub init_timeout_ms: u64,
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

#[derive(Subcommand)]
pub enum StealthCommands {
    /// Configure stealth settings
    Configure {
        /// Stealth preset level (none, low, medium, high)
        #[arg(long, value_parser = ["none", "low", "medium", "high"])]
        preset: String,

        /// Path to custom user agent list file
        #[arg(long)]
        ua_file: Option<String>,

        /// Enable fingerprint countermeasures
        #[arg(long)]
        fingerprint_evasion: bool,

        /// Save configuration to file
        #[arg(long, short = 'o')]
        output: Option<String>,
    },

    /// Test stealth configuration against a URL
    Test {
        /// URL to test stealth against
        #[arg(long)]
        url: String,

        /// Stealth preset to test (none, low, medium, high)
        #[arg(long, default_value = "medium")]
        preset: String,

        /// Show detailed test results
        #[arg(long)]
        verbose: bool,
    },

    /// Show current stealth configuration
    Info,

    /// Generate stealth JavaScript injection code
    Generate {
        /// Stealth level for generated code
        #[arg(long, default_value = "medium")]
        level: String,

        /// Output file path
        #[arg(long, short = 'o')]
        output: Option<String>,
    },
}

#[derive(clap::Args)]
pub struct TablesArgs {
    /// URL to extract tables from
    #[arg(long)]
    pub url: Option<String>,

    /// Local HTML file to extract tables from
    #[arg(long)]
    pub file: Option<String>,

    /// Output format (markdown, csv, json)
    #[arg(long, default_value = "markdown")]
    pub format: String,

    /// Read HTML from stdin
    #[arg(long)]
    pub stdin: bool,

    /// Output file path (optional)
    #[arg(long, short = 'o')]
    pub output: Option<String>,
}

#[derive(Subcommand)]
pub enum MetricsCommands {
    /// View current metrics
    Show,

    /// Export metrics to file
    Export {
        /// Export format (prom, json, csv)
        #[arg(long, default_value = "json")]
        format: String,

        /// Output file path
        #[arg(long, short = 'o')]
        output: Option<String>,

        /// Filter specific metric
        #[arg(long)]
        metric: Option<String>,
    },
}
