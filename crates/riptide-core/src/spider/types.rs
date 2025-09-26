use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime};
use url::Url;

/// Priority levels for crawl requests
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[derive(Default)]
pub enum Priority {
    Low = 1,
    #[default]
    Medium = 2,
    High = 3,
    Critical = 4,
}


/// A request to crawl a specific URL with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlRequest {
    /// The URL to crawl
    pub url: Url,
    /// Priority level for this request
    pub priority: Priority,
    /// Depth from seed URL
    pub depth: u32,
    /// Parent URL that led to this request
    pub parent: Option<Url>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
    /// When this request was created
    pub created_at: SystemTime,
    /// Number of retry attempts
    pub retry_count: u32,
    /// Maximum retries allowed
    pub max_retries: u32,
    /// Custom scoring for best-first strategy
    pub score: Option<f64>,
}

impl CrawlRequest {
    pub fn new(url: Url) -> Self {
        Self {
            url,
            priority: Priority::Medium,
            depth: 0,
            parent: None,
            metadata: HashMap::new(),
            created_at: SystemTime::now(),
            retry_count: 0,
            max_retries: 3,
            score: None,
        }
    }

    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_depth(mut self, depth: u32) -> Self {
        self.depth = depth;
        self
    }

    pub fn with_parent(mut self, parent: Url) -> Self {
        self.parent = Some(parent);
        self
    }

    pub fn with_score(mut self, score: f64) -> Self {
        self.score = Some(score);
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Get the host from the URL
    pub fn host(&self) -> Option<&str> {
        self.url.host_str()
    }

    /// Check if this request can be retried
    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }

    /// Increment retry count
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }
}

/// Result of a crawl operation
#[derive(Debug, Clone)]
pub struct CrawlResult {
    /// The request that was processed
    pub request: CrawlRequest,
    /// Whether the crawl was successful
    pub success: bool,
    /// HTTP status code if applicable
    pub status_code: Option<u16>,
    /// Content type of the response
    pub content_type: Option<String>,
    /// Size of the content in bytes
    pub content_size: usize,
    /// Text content for analysis
    pub text_content: Option<String>,
    /// Extracted URLs from the page
    pub extracted_urls: Vec<Url>,
    /// Processing time
    pub processing_time: Duration,
    /// Error message if unsuccessful
    pub error: Option<String>,
    /// Custom metadata from processing
    pub metadata: HashMap<String, String>,
}

impl CrawlResult {
    pub fn success(request: CrawlRequest) -> Self {
        Self {
            request,
            success: true,
            status_code: Some(200),
            content_type: None,
            content_size: 0,
            text_content: None,
            extracted_urls: Vec::new(),
            processing_time: Duration::from_millis(0),
            error: None,
            metadata: HashMap::new(),
        }
    }

    pub fn failure(request: CrawlRequest, error: String) -> Self {
        Self {
            request,
            success: false,
            status_code: None,
            content_type: None,
            content_size: 0,
            text_content: None,
            extracted_urls: Vec::new(),
            processing_time: Duration::from_millis(0),
            error: Some(error),
            metadata: HashMap::new(),
        }
    }

    /// Get unique text characters count for adaptive stop analysis
    pub fn unique_text_chars(&self) -> usize {
        self.text_content
            .as_ref()
            .map(|text| {
                let mut chars: Vec<char> = text.chars().collect();
                chars.sort_unstable();
                chars.dedup();
                chars.len()
            })
            .unwrap_or(0)
    }

    /// Calculate content score for best-first strategy
    pub fn calculate_content_score(&self) -> f64 {
        if !self.success {
            return 0.0;
        }

        let mut score = 0.0;

        // Base score from content size (normalized)
        score += (self.content_size as f64).ln().max(0.0_f64) * 0.1;

        // Score from unique text characters
        score += (self.unique_text_chars() as f64).ln().max(0.0_f64) * 0.2;

        // Score from extracted URLs (link richness)
        score += (self.extracted_urls.len() as f64).ln().max(0.0_f64) * 0.15;

        // Bonus for HTML content
        if let Some(content_type) = &self.content_type {
            if content_type.contains("text/html") {
                score *= 1.5;
            }
        }

        score.max(0.0_f64)
    }
}

/// Host-specific crawling state and metrics
#[derive(Debug, Clone)]
pub struct HostState {
    /// Host name
    pub host: String,
    /// Current number of in-flight requests
    pub in_flight: u32,
    /// Maximum concurrent requests allowed
    pub max_concurrent: u32,
    /// Total pages crawled from this host
    pub pages_crawled: u64,
    /// Total errors from this host
    pub error_count: u64,
    /// Last successful request time
    pub last_success: Option<Instant>,
    /// Last error time
    pub last_error: Option<Instant>,
    /// Current rate limiting delay
    pub current_delay: Duration,
    /// Whether this host is currently blocked
    pub blocked: bool,
    /// Reason for blocking if applicable
    pub block_reason: Option<String>,
}

impl HostState {
    pub fn new(host: String) -> Self {
        Self {
            host,
            in_flight: 0,
            max_concurrent: 2,
            pages_crawled: 0,
            error_count: 0,
            last_success: None,
            last_error: None,
            current_delay: Duration::from_millis(1000),
            blocked: false,
            block_reason: None,
        }
    }

    /// Check if this host can accept another request
    pub fn can_accept_request(&self) -> bool {
        !self.blocked && self.in_flight < self.max_concurrent
    }

    /// Record a successful request
    pub fn record_success(&mut self) {
        self.pages_crawled += 1;
        self.last_success = Some(Instant::now());
        if self.in_flight > 0 {
            self.in_flight -= 1;
        }
    }

    /// Record a failed request
    pub fn record_error(&mut self, error: String) {
        self.error_count += 1;
        self.last_error = Some(Instant::now());
        if self.in_flight > 0 {
            self.in_flight -= 1;
        }

        // Consider blocking if too many errors
        if self.error_count > 10 && self.pages_crawled < 5 {
            self.blocked = true;
            self.block_reason = Some(format!("Too many errors: {}", error));
        }
    }

    /// Start a request (increment in-flight counter)
    pub fn start_request(&mut self) -> Result<()> {
        if !self.can_accept_request() {
            return Err(anyhow::anyhow!("Host {} cannot accept more requests", self.host));
        }
        self.in_flight += 1;
        Ok(())
    }

    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        let total = self.pages_crawled + self.error_count;
        if total == 0 {
            1.0
        } else {
            self.pages_crawled as f64 / total as f64
        }
    }
}

/// Frontier metrics and statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FrontierMetrics {
    /// Total requests in frontier
    pub total_requests: usize,
    /// Requests by priority
    pub requests_by_priority: HashMap<Priority, usize>,
    /// Requests by host
    pub requests_by_host: HashMap<String, usize>,
    /// Average depth of requests
    pub average_depth: f64,
    /// Memory usage in bytes
    pub memory_usage: usize,
    /// Whether disk spillover is active
    pub disk_spillover_active: bool,
    /// Rate of requests being added
    pub request_add_rate: f64,
    /// Rate of requests being processed
    pub request_process_rate: f64,
}

impl FrontierMetrics {
    pub fn update_rates(&mut self, add_count: u64, process_count: u64, time_window: Duration) {
        let window_secs = time_window.as_secs_f64();
        if window_secs > 0.0 {
            self.request_add_rate = add_count as f64 / window_secs;
            self.request_process_rate = process_count as f64 / window_secs;
        }
    }
}

/// Configuration for URL scoring in best-first strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringConfig {
    /// Weight for URL depth (negative to prefer shallow)
    pub depth_weight: f64,
    /// Weight for URL path length
    pub path_length_weight: f64,
    /// Weight for number of parameters
    pub parameter_weight: f64,
    /// Weight for content type hints
    pub content_type_weight: f64,
    /// Keywords to prioritize in content
    pub content_keywords: Vec<String>,
    /// Custom domain scoring
    pub domain_scores: HashMap<String, f64>,
    /// File extension scoring
    pub extension_scores: HashMap<String, f64>,
}

impl Default for ScoringConfig {
    fn default() -> Self {
        let mut extension_scores = HashMap::new();
        extension_scores.insert("html".to_string(), 1.0);
        extension_scores.insert("php".to_string(), 0.8);
        extension_scores.insert("asp".to_string(), 0.8);
        extension_scores.insert("jsp".to_string(), 0.8);
        extension_scores.insert("pdf".to_string(), 0.5);
        extension_scores.insert("doc".to_string(), 0.5);
        extension_scores.insert("xml".to_string(), 0.3);
        extension_scores.insert("json".to_string(), 0.2);

        Self {
            depth_weight: -0.1,
            content_keywords: Vec::new(),
            path_length_weight: -0.05,
            parameter_weight: -0.02,
            content_type_weight: 0.3,
            domain_scores: HashMap::new(),
            extension_scores,
        }
    }
}

/// Adaptive stop analysis window
#[derive(Debug, Clone)]
pub struct ContentWindow {
    /// Sliding window of unique text character counts
    pub char_counts: Vec<usize>,
    /// Window size
    pub window_size: usize,
    /// Current position in window
    pub position: usize,
    /// Whether window is full
    pub full: bool,
}

impl ContentWindow {
    pub fn new(window_size: usize) -> Self {
        Self {
            char_counts: vec![0; window_size],
            window_size,
            position: 0,
            full: false,
        }
    }

    /// Add a new measurement to the window
    pub fn add_measurement(&mut self, char_count: usize) {
        self.char_counts[self.position] = char_count;
        self.position = (self.position + 1) % self.window_size;
        if self.position == 0 {
            self.full = true;
        }
    }

    /// Calculate the average gain in the window
    pub fn average_gain(&self) -> f64 {
        if !self.full && self.position < 2 {
            return f64::INFINITY; // Not enough data
        }

        let values = if self.full {
            &self.char_counts
        } else {
            &self.char_counts[..self.position]
        };

        if values.len() < 2 {
            return f64::INFINITY;
        }

        let total_gain: f64 = values
            .windows(2)
            .map(|window| (window[1] as f64 - window[0] as f64).max(0.0_f64))
            .sum();

        total_gain / (values.len() - 1) as f64
    }

    /// Get the latest measurement
    pub fn latest(&self) -> Option<usize> {
        if self.position == 0 && !self.full {
            None
        } else {
            let idx = if self.position == 0 {
                self.window_size - 1
            } else {
                self.position - 1
            };
            Some(self.char_counts[idx])
        }
    }

    /// Check if window has enough data for analysis
    pub fn has_sufficient_data(&self) -> bool {
        self.full || self.position >= 3
    }
}

/// Configuration for crawling strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    /// Default strategy to use
    pub default_strategy: String,
    /// Scoring configuration for best-first strategy
    pub scoring: ScoringConfig,
    /// Enable adaptive strategy switching
    pub enable_adaptive: bool,
    /// Adaptive switching criteria
    pub adaptive_criteria: AdaptiveCriteria,
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            default_strategy: "BreadthFirst".to_string(),
            scoring: ScoringConfig::default(),
            enable_adaptive: false,
            adaptive_criteria: AdaptiveCriteria::default(),
        }
    }
}

/// Adaptive criteria for strategy switching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveCriteria {
    /// Switch if frontier size exceeds this threshold
    pub max_frontier_size: usize,
    /// Switch if average depth exceeds this threshold
    pub max_average_depth: f64,
    /// Switch if success rate drops below this threshold
    pub min_success_rate: f64,
    /// Minimum pages before considering switch
    pub min_pages_for_switch: usize,
    /// Cooldown period between switches
    pub switch_cooldown_pages: usize,
}

impl Default for AdaptiveCriteria {
    fn default() -> Self {
        Self {
            max_frontier_size: 10000,
            max_average_depth: 5.0,
            min_success_rate: 0.7,
            min_pages_for_switch: 100,
            switch_cooldown_pages: 50,
        }
    }
}

/// Configuration for sitemap processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitemapConfig {
    /// Enable automatic sitemap discovery
    pub enable_discovery: bool,
    /// Custom sitemap URLs to process
    pub custom_sitemaps: Vec<String>,
    /// Follow sitemap index files
    pub follow_sitemap_index: bool,
    /// Maximum URLs to extract from sitemaps
    pub max_urls: Option<usize>,
    /// Respect lastmod dates in sitemaps
    pub respect_lastmod: bool,
    /// Minimum priority threshold (0.0 to 1.0)
    pub min_priority: f64,
    /// User agent for sitemap requests
    pub user_agent: String,
    /// Timeout for sitemap requests
    pub timeout_seconds: u64,
}

impl Default for SitemapConfig {
    fn default() -> Self {
        Self {
            enable_discovery: true,
            custom_sitemaps: Vec::new(),
            follow_sitemap_index: true,
            max_urls: Some(10000),
            respect_lastmod: false,
            min_priority: 0.0,
            user_agent: "RipTide Spider/1.0".to_string(),
            timeout_seconds: 30,
        }
    }
}