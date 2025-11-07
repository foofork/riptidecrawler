#![allow(dead_code)]
use crate::state::DependencyHealth;
#[cfg(feature = "spider")]
use riptide_spider::{CrawlState, PerformanceMetrics};
pub use riptide_types::config::CrawlOptions;
pub use riptide_types::ExtractedDoc;
use serde::{Deserialize, Serialize};

/// Request body for crawling multiple URLs
#[derive(Deserialize, Debug, Clone)]
pub struct CrawlBody {
    /// List of URLs to crawl
    pub urls: Vec<String>,

    /// Optional crawl configuration options
    pub options: Option<CrawlOptions>,
}

/// Individual crawl result for a single URL
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CrawlResult {
    /// The original URL that was crawled
    pub url: String,

    /// HTTP status code from the fetch operation
    pub status: u16,

    /// Whether this result was served from cache
    pub from_cache: bool,

    /// Gate decision made for this URL (raw, probes_first, headless, cached)
    pub gate_decision: String,

    /// Content quality score from gate analysis (0.0 to 1.0)
    pub quality_score: f32,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,

    /// Extracted document content (if successful)
    pub document: Option<ExtractedDoc>,

    /// Error information (if extraction failed)
    pub error: Option<ErrorInfo>,

    /// Cache key used for this URL
    pub cache_key: String,
}

/// Error information for failed operations
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorInfo {
    /// Error type identifier
    pub error_type: String,

    /// Human-readable error message
    pub message: String,

    /// Whether this error is retryable
    pub retryable: bool,
}

/// Response for batch crawl operations
#[derive(Serialize, Deserialize, Debug)]
pub struct CrawlResponse {
    /// Total number of URLs in the request
    pub total_urls: usize,

    /// Number of successful extractions
    pub successful: usize,

    /// Number of failed extractions
    pub failed: usize,

    /// Number of results served from cache
    pub from_cache: usize,

    /// Individual results for each URL
    pub results: Vec<CrawlResult>,

    /// Overall statistics for this batch
    pub statistics: CrawlStatistics,
}

/// Statistics for crawl operations
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CrawlStatistics {
    /// Total processing time for the entire batch
    pub total_processing_time_ms: u64,

    /// Average processing time per URL
    pub avg_processing_time_ms: f64,

    /// Gate decision breakdown
    pub gate_decisions: GateDecisionBreakdown,

    /// Cache performance metrics
    pub cache_hit_rate: f64,
}

/// Breakdown of gate decisions made during crawling
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GateDecisionBreakdown {
    /// Number of URLs that used raw/fast extraction
    pub raw: usize,

    /// Number of URLs that used probing strategy
    pub probes_first: usize,

    /// Number of URLs that required headless rendering
    pub headless: usize,

    /// Number of URLs served from cache
    pub cached: usize,
}

/// Request body for deep search operations
#[derive(Deserialize, Debug, Clone)]
pub struct DeepSearchBody {
    /// Search query string
    pub query: String,

    /// Maximum number of results to return (default: 10, max: 50)
    pub limit: Option<u32>,

    /// Country code for search localization (e.g., "US", "GB")
    pub country: Option<String>,

    /// Locale for search results (e.g., "en", "es")
    pub locale: Option<String>,

    /// Whether to include the full crawled content in results
    pub include_content: Option<bool>,

    /// Optional crawl configuration for discovered URLs
    pub crawl_options: Option<CrawlOptions>,
}

/// Response for deep search operations
#[derive(Serialize, Deserialize, Debug)]
pub struct DeepSearchResponse {
    /// Original search query
    pub query: String,

    /// Number of URLs found and queued for crawling
    pub urls_found: usize,

    /// Number of URLs successfully crawled
    pub urls_crawled: usize,

    /// Search results with extracted content
    pub results: Vec<SearchResult>,

    /// Processing status
    pub status: String,

    /// Total processing time
    pub processing_time_ms: u64,
}

/// Individual search result with extracted content
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchResult {
    /// Original URL from search results
    pub url: String,

    /// Search engine ranking/position
    pub rank: u32,

    /// Title from search results
    pub search_title: Option<String>,

    /// Snippet from search results
    pub search_snippet: Option<String>,

    /// Extracted document content (if crawled successfully)
    pub content: Option<ExtractedDoc>,

    /// Crawl result metadata
    pub crawl_result: Option<CrawlResult>,
}

/// Health check response with detailed dependency status
#[derive(Serialize, Deserialize, Debug)]
pub struct HealthResponse {
    /// Overall health status ("healthy" or "unhealthy")
    pub status: String,

    /// Application version
    pub version: String,

    /// Timestamp of the health check
    pub timestamp: String,

    /// Uptime in seconds
    pub uptime: u64,

    /// Individual dependency health status
    pub dependencies: DependencyStatus,

    /// System metrics
    pub metrics: Option<SystemMetrics>,
}

/// Health status of individual dependencies
#[derive(Serialize, Deserialize, Debug)]
pub struct DependencyStatus {
    /// Redis cache health
    pub redis: ServiceHealth,

    /// WASM extractor health
    pub extractor: ServiceHealth,

    /// HTTP client health
    pub http_client: ServiceHealth,

    /// Headless service health (if configured)
    pub headless_service: Option<ServiceHealth>,

    /// Spider engine health (if configured)
    pub spider_engine: Option<ServiceHealth>,

    /// Worker service health (background job processing)
    pub worker_service: Option<ServiceHealth>,
}

/// Health status for an individual service
#[derive(Serialize, Deserialize, Debug)]
pub struct ServiceHealth {
    /// Service status ("healthy", "unhealthy", "unknown")
    pub status: String,

    /// Detailed status message
    pub message: Option<String>,

    /// Response time in milliseconds (if available)
    pub response_time_ms: Option<u64>,

    /// Last check timestamp
    pub last_check: String,
}

/// System performance metrics
#[derive(Serialize, Deserialize, Debug)]
pub struct SystemMetrics {
    /// Current memory usage in bytes
    pub memory_usage_bytes: u64,

    /// Number of active connections
    pub active_connections: u32,

    /// Total requests processed since startup
    pub total_requests: u64,

    /// Current requests per second (approximate)
    pub requests_per_second: f64,

    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,

    /// CPU usage percentage (0-100)
    pub cpu_usage_percent: Option<f32>,

    /// Disk usage in bytes
    pub disk_usage_bytes: Option<u64>,

    /// File descriptor count
    pub file_descriptor_count: Option<u32>,

    /// Thread count
    pub thread_count: Option<u32>,

    /// Load average [1min, 5min, 15min]
    pub load_average: Option<[f32; 3]>,
}

impl From<DependencyHealth> for ServiceHealth {
    fn from(health: DependencyHealth) -> Self {
        match health {
            DependencyHealth::Healthy => ServiceHealth {
                status: "healthy".to_string(),
                message: None,
                response_time_ms: None,
                last_check: chrono::Utc::now().to_rfc3339(),
            },
            DependencyHealth::Unhealthy(msg) => ServiceHealth {
                status: "unhealthy".to_string(),
                message: Some(msg),
                response_time_ms: None,
                last_check: chrono::Utc::now().to_rfc3339(),
            },
            DependencyHealth::Unknown => ServiceHealth {
                status: "unknown".to_string(),
                message: None,
                response_time_ms: None,
                last_check: chrono::Utc::now().to_rfc3339(),
            },
        }
    }
}

/// Request body for spider crawl operations
#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
pub struct SpiderCrawlBody {
    /// Seed URLs to start crawling from
    pub seed_urls: Vec<String>,

    /// Maximum depth to crawl (optional)
    pub max_depth: Option<usize>,

    /// Maximum pages to crawl (optional)
    pub max_pages: Option<usize>,

    /// Crawling strategy: "breadth_first", "depth_first", "best_first"
    pub strategy: Option<String>,

    /// Request timeout in seconds
    pub timeout_seconds: Option<u64>,

    /// Delay between requests in milliseconds
    pub delay_ms: Option<u64>,

    /// Maximum concurrent requests
    pub concurrency: Option<usize>,

    /// Whether to respect robots.txt
    pub respect_robots: Option<bool>,

    /// Whether to follow redirects
    pub follow_redirects: Option<bool>,
}

/// Response for spider crawl operations with statistics
#[derive(Serialize, Debug)]
pub struct SpiderCrawlResponseStats {
    /// Spider crawl result statistics
    pub result: SpiderApiResult,

    /// Current crawl state
    pub state: CrawlState,

    /// Performance metrics
    pub performance: PerformanceMetrics,
}

/// Response for spider crawl operations with URLs
#[derive(Serialize, Debug)]
pub struct SpiderCrawlResponseUrls {
    /// Spider crawl result with URLs
    pub result: SpiderApiResultUrls,

    /// Current crawl state
    pub state: CrawlState,

    /// Performance metrics
    pub performance: PerformanceMetrics,
}

/// API-friendly version of SpiderResult (statistics only)
#[derive(Serialize, Debug)]
pub struct SpiderApiResult {
    /// Total pages crawled
    pub pages_crawled: u64,

    /// Total pages failed
    pub pages_failed: u64,

    /// Crawl duration in seconds
    pub duration_seconds: f64,

    /// Reason for stopping
    pub stop_reason: String,

    /// Domains crawled
    pub domains: Vec<String>,
}

/// API-friendly version of SpiderResult with discovered URLs
#[derive(Serialize, Debug)]
pub struct SpiderApiResultUrls {
    /// Total pages crawled
    pub pages_crawled: u64,

    /// Total pages failed
    pub pages_failed: u64,

    /// Crawl duration in seconds
    pub duration_seconds: f64,

    /// Reason for stopping
    pub stop_reason: String,

    /// Domains crawled
    pub domains: Vec<String>,

    /// All URLs discovered during the crawl
    pub discovered_urls: Vec<String>,
}

/// Request body for spider status
#[derive(Deserialize, Debug, Clone)]
pub struct SpiderStatusRequest {
    /// Whether to include detailed metrics
    pub include_metrics: Option<bool>,
}

/// Response for spider status
#[derive(Serialize, Debug)]
pub struct SpiderStatusResponse {
    /// Current crawl state
    pub state: CrawlState,

    /// Performance metrics (if requested)
    #[cfg(feature = "spider")]
    pub performance: Option<PerformanceMetrics>,

    /// Frontier statistics
    #[cfg(feature = "spider")]
    pub frontier_stats: Option<riptide_spider::types::FrontierMetrics>,

    /// Adaptive stop statistics
    #[cfg(feature = "spider")]
    pub adaptive_stop_stats: Option<riptide_spider::adaptive_stop::AdaptiveStopStats>,
}

/// Request body for spider control operations
#[derive(Deserialize, Debug, Clone)]
pub struct SpiderControlRequest {
    /// Action to perform: "start", "stop", "reset"
    pub action: String,
}
