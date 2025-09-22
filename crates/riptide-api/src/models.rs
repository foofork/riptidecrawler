use crate::state::DependencyHealth;
use riptide_core::types::{CrawlOptions, ExtractedDoc};
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
#[derive(Serialize, Debug, Clone)]
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
#[derive(Serialize, Debug, Clone)]
pub struct ErrorInfo {
    /// Error type identifier
    pub error_type: String,

    /// Human-readable error message
    pub message: String,

    /// Whether this error is retryable
    pub retryable: bool,
}

/// Response for batch crawl operations
#[derive(Serialize, Debug)]
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
#[derive(Serialize, Debug, Clone)]
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
#[derive(Serialize, Debug, Clone, Default)]
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
    #[allow(dead_code)]
    pub country: Option<String>,

    /// Locale for search results (e.g., "en", "es")
    #[allow(dead_code)]
    pub locale: Option<String>,

    /// Whether to include the full crawled content in results
    pub include_content: Option<bool>,

    /// Optional crawl configuration for discovered URLs
    pub crawl_options: Option<CrawlOptions>,
}

/// Response for deep search operations
#[derive(Serialize, Debug)]
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
#[derive(Serialize, Debug)]
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
#[derive(Serialize, Debug)]
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
#[derive(Serialize, Debug)]
pub struct DependencyStatus {
    /// Redis cache health
    pub redis: ServiceHealth,

    /// WASM extractor health
    pub extractor: ServiceHealth,

    /// HTTP client health
    pub http_client: ServiceHealth,

    /// Headless service health (if configured)
    pub headless_service: Option<ServiceHealth>,
}

/// Health status for an individual service
#[derive(Serialize, Debug)]
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
#[derive(Serialize, Debug)]
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
