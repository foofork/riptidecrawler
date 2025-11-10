//! HTTP API request/response types
//!
//! These types define the HTTP API contracts used by both riptide-api (handlers)
//! and riptide-facade (orchestration layer). Living in riptide-types ensures
//! one-way dependency: api → facade → types (no cycles).

use serde::{Deserialize, Serialize};

// ============================================================================
// Extract Endpoint Types
// ============================================================================

/// Extract endpoint request payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractRequest {
    /// URL to extract content from
    pub url: String,
    /// Extraction mode (standard, article, product, etc.)
    #[serde(default = "default_mode")]
    pub mode: String,
    /// Extraction options
    #[serde(default)]
    pub options: ExtractOptions,
}

/// Extraction options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractOptions {
    /// Strategy to use (auto, css, wasm, llm, multi)
    #[serde(default = "default_strategy")]
    pub strategy: String,
    /// Minimum quality threshold (0.0-1.0)
    #[serde(default = "default_quality_threshold")]
    pub quality_threshold: f64,
    /// Timeout in milliseconds
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

impl Default for ExtractOptions {
    fn default() -> Self {
        Self {
            strategy: default_strategy(),
            quality_threshold: default_quality_threshold(),
            timeout_ms: default_timeout(),
        }
    }
}

/// Extract response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractResponse {
    pub url: String,
    pub title: Option<String>,
    pub content: String,
    pub metadata: ContentMetadata,
    pub strategy_used: String,
    pub quality_score: f64,
    pub extraction_time_ms: u64,
    /// Parser metadata for observability (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parser_metadata: Option<ParserMetadataHttp>,
}

/// Content metadata
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ContentMetadata {
    pub author: Option<String>,
    pub publish_date: Option<String>,
    pub word_count: usize,
    pub language: Option<String>,
}

/// Parser metadata for observability (HTTP-specific to avoid confusion with riptide_types::ParserMetadata)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParserMetadataHttp {
    pub parser_used: String,
    pub confidence_score: f64,
    pub fallback_occurred: bool,
    pub parse_time_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extraction_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_error: Option<String>,
}

// ============================================================================
// Search Endpoint Types
// ============================================================================

/// Search query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Search query string
    pub q: String,
    /// Number of results
    #[serde(default = "default_search_limit")]
    pub limit: u32,
    /// Country code
    #[serde(default = "default_country")]
    pub country: String,
    /// Language code
    #[serde(default = "default_language")]
    pub language: String,
    /// Force specific provider
    pub provider: Option<String>,
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub position: u32,
}

/// Search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub total_results: usize,
    pub provider_used: String,
    pub search_time_ms: u64,
}

// ============================================================================
// Spider Endpoint Types
// ============================================================================

/// Result mode for spider crawl operations
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum ResultMode {
    /// Return statistics only (default, backward compatible)
    #[default]
    Stats,
    /// Return discovered URLs list
    Urls,
    /// Return full page objects with content
    Pages,
    /// Stream results as NDJSON (not yet implemented)
    Stream,
    /// Store results for async retrieval (not yet implemented)
    Store,
}

/// Statistics result for spider crawl operations (backward compatible)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpiderResultStats {
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

/// URLs result for spider crawl operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpiderResultUrls {
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
    #[serde(default)]
    pub discovered_urls: Vec<String>,
}

/// Pages result for spider crawl operations with full page details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpiderResultPages {
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
    /// All crawled pages with full details
    #[serde(default)]
    pub pages: Vec<CrawledPage>,
}

/// A single crawled page with all available information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawledPage {
    /// The URL of the page
    pub url: String,
    /// Crawl depth from seed URLs
    pub depth: u32,
    /// HTTP status code received
    pub status_code: u16,
    /// Page title (extracted from HTML)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Raw HTML/text content (only included if requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Normalized markdown content (only included if requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markdown: Option<String>,
    /// Links found on this page
    pub links: Vec<String>,
    /// Whether content/markdown was truncated due to size limits
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncated: Option<bool>,
    /// Final URL after redirects
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_url: Option<String>,
    /// MIME type of the response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime: Option<String>,
    /// Fetch time in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fetch_time_ms: Option<u64>,
    /// Whether robots.txt was obeyed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub robots_obeyed: Option<bool>,
    /// Fetch error if any
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fetch_error: Option<String>,
    /// Parse error if any
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_error: Option<String>,
}

impl CrawledPage {
    /// Create a new CrawledPage with basic information
    pub fn new(url: String, depth: u32, status_code: u16) -> Self {
        Self {
            url,
            depth,
            status_code,
            title: None,
            content: None,
            markdown: None,
            links: Vec::new(),
            truncated: None,
            final_url: None,
            mime: None,
            fetch_time_ms: None,
            robots_obeyed: None,
            fetch_error: None,
            parse_error: None,
        }
    }

    /// Truncate content and markdown fields to specified size limit
    pub fn truncate_content(&mut self, max_content_bytes: usize) {
        if let Some(content) = &mut self.content {
            if content.len() > max_content_bytes {
                content.truncate(max_content_bytes);
                self.truncated = Some(true);
            }
        }

        if let Some(markdown) = &mut self.markdown {
            if markdown.len() > max_content_bytes {
                markdown.truncate(max_content_bytes);
                self.truncated = Some(true);
            }
        }
    }
}

// ============================================================================
// Default Functions (Private)
// ============================================================================

fn default_mode() -> String {
    "standard".to_string()
}

fn default_strategy() -> String {
    "multi".to_string()
}

fn default_quality_threshold() -> f64 {
    0.7
}

fn default_timeout() -> u64 {
    30000
}

fn default_search_limit() -> u32 {
    10
}

fn default_country() -> String {
    "us".to_string()
}

fn default_language() -> String {
    "en".to_string()
}
