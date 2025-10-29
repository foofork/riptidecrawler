use serde::{Deserialize, Serialize};

/// Result mode for spider crawl operations
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ResultMode {
    /// Return statistics only (default, backward compatible)
    Stats,
    /// Return discovered URLs list
    Urls,
}

impl Default for ResultMode {
    fn default() -> Self {
        Self::Stats
    }
}

/// Statistics result for spider crawl operations (backward compatible)
#[derive(Serialize, Debug)]
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
#[derive(Serialize, Debug)]
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

impl From<&riptide_facade::facades::spider::CrawlSummary> for SpiderResultStats {
    fn from(summary: &riptide_facade::facades::spider::CrawlSummary) -> Self {
        Self {
            pages_crawled: summary.pages_crawled,
            pages_failed: summary.pages_failed,
            duration_seconds: summary.duration_secs,
            stop_reason: summary.stop_reason.clone(),
            domains: summary.domains.clone(),
        }
    }
}

impl From<&riptide_facade::facades::spider::CrawlSummary> for SpiderResultUrls {
    fn from(summary: &riptide_facade::facades::spider::CrawlSummary) -> Self {
        Self {
            pages_crawled: summary.pages_crawled,
            pages_failed: summary.pages_failed,
            duration_seconds: summary.duration_secs,
            stop_reason: summary.stop_reason.clone(),
            domains: summary.domains.clone(),
            discovered_urls: summary.discovered_urls.clone(),
        }
    }
}
