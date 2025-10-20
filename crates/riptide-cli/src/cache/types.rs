//! Cache data structures and type definitions
//!
//! This module provides the core data types for the RipTide cache system with:
//! - URL-based caching with metadata
//! - LRU eviction policy support
//! - Domain-based filtering
//! - Cache statistics tracking

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Cache entry containing fetched content and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// URL of the cached resource
    pub url: String,

    /// Domain extracted from URL
    pub domain: String,

    /// Cached content (HTML, JSON, etc.)
    pub content: String,

    /// Content type (e.g., "text/html", "application/json")
    pub content_type: String,

    /// Size in bytes
    pub size_bytes: u64,

    /// Timestamp when cached
    #[serde(with = "chrono::serde::ts_seconds")]
    pub cached_at: DateTime<Utc>,

    /// Last access timestamp (for LRU)
    #[serde(with = "chrono::serde::ts_seconds")]
    pub last_accessed: DateTime<Utc>,

    /// Number of times accessed
    pub access_count: u64,

    /// HTTP status code from original request
    pub status_code: u16,

    /// Response headers
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,

    /// ETag for cache validation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>,

    /// Optional TTL in seconds (None = no expiration)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl_seconds: Option<u64>,
}

impl CacheEntry {
    /// Create new cache entry
    pub fn new(url: String, content: String, content_type: String) -> Self {
        let domain = Self::extract_domain(&url);
        let size_bytes = content.len() as u64;
        let now = Utc::now();

        Self {
            url,
            domain,
            content,
            content_type,
            size_bytes,
            cached_at: now,
            last_accessed: now,
            access_count: 0,
            status_code: 200,
            headers: HashMap::new(),
            etag: None,
            ttl_seconds: None,
        }
    }

    /// Extract domain from URL
    fn extract_domain(url: &str) -> String {
        url::Url::parse(url)
            .ok()
            .and_then(|u| u.host_str().map(|h| h.to_string()))
            .unwrap_or_else(|| "unknown".to_string())
    }

    /// Update last access time and increment counter
    pub fn touch(&mut self) {
        self.last_accessed = Utc::now();
        self.access_count += 1;
    }

    /// Check if entry has expired based on TTL
    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl_seconds {
            let age_seconds = (Utc::now() - self.cached_at).num_seconds();
            age_seconds > ttl as i64
        } else {
            false
        }
    }

    /// Get age in seconds
    pub fn age_seconds(&self) -> i64 {
        (Utc::now() - self.cached_at).num_seconds()
    }

    /// Get time since last access in seconds
    pub fn idle_seconds(&self) -> i64 {
        (Utc::now() - self.last_accessed).num_seconds()
    }
}

/// Cache statistics for monitoring and reporting
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheStats {
    /// Total number of cache entries
    pub total_entries: usize,

    /// Total cache size in bytes
    pub total_size_bytes: u64,

    /// Number of cache hits
    pub hits: u64,

    /// Number of cache misses
    pub misses: u64,

    /// Number of evictions
    pub evictions: u64,

    /// Number of insertions
    pub insertions: u64,

    /// Number of entries by domain
    #[serde(default)]
    pub entries_by_domain: HashMap<String, usize>,

    /// Size by domain in bytes
    #[serde(default)]
    pub size_by_domain: HashMap<String, u64>,

    /// Last update timestamp
    #[serde(with = "chrono::serde::ts_seconds")]
    pub last_updated: DateTime<Utc>,
}

impl CacheStats {
    /// Create new empty statistics
    pub fn new() -> Self {
        Self {
            total_entries: 0,
            total_size_bytes: 0,
            hits: 0,
            misses: 0,
            evictions: 0,
            insertions: 0,
            entries_by_domain: HashMap::new(),
            size_by_domain: HashMap::new(),
            last_updated: Utc::now(),
        }
    }

    /// Calculate cache hit rate (0.0 - 1.0)
    pub fn hit_rate(&self) -> f64 {
        let total_requests = self.hits + self.misses;
        if total_requests == 0 {
            return 0.0;
        }
        self.hits as f64 / total_requests as f64
    }

    /// Calculate cache miss rate (0.0 - 1.0)
    pub fn miss_rate(&self) -> f64 {
        1.0 - self.hit_rate()
    }

    /// Record a cache hit
    pub fn record_hit(&mut self) {
        self.hits += 1;
        self.last_updated = Utc::now();
    }

    /// Record a cache miss
    pub fn record_miss(&mut self) {
        self.misses += 1;
        self.last_updated = Utc::now();
    }

    /// Record a cache eviction
    pub fn record_eviction(&mut self, domain: &str, size: u64) {
        self.evictions += 1;
        self.total_entries = self.total_entries.saturating_sub(1);
        self.total_size_bytes = self.total_size_bytes.saturating_sub(size);

        if let Some(count) = self.entries_by_domain.get_mut(domain) {
            *count = count.saturating_sub(1);
        }
        if let Some(size_total) = self.size_by_domain.get_mut(domain) {
            *size_total = size_total.saturating_sub(size);
        }

        self.last_updated = Utc::now();
    }

    /// Record a cache insertion
    pub fn record_insertion(&mut self, domain: &str, size: u64) {
        self.insertions += 1;
        self.total_entries += 1;
        self.total_size_bytes += size;

        *self
            .entries_by_domain
            .entry(domain.to_string())
            .or_insert(0) += 1;
        *self.size_by_domain.entry(domain.to_string()).or_insert(0) += size;

        self.last_updated = Utc::now();
    }
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum cache size in bytes (default: 100MB)
    pub max_size_bytes: u64,

    /// Maximum number of entries (default: 1000)
    pub max_entries: usize,

    /// Default TTL in seconds (None = no expiration)
    pub default_ttl_seconds: Option<u64>,

    /// Cache directory path
    pub cache_dir: String,

    /// Enable persistent storage
    pub persistent: bool,

    /// Auto-save interval in seconds
    pub auto_save_interval_seconds: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size_bytes: 100 * 1024 * 1024, // 100MB
            max_entries: 1000,
            default_ttl_seconds: None,
            cache_dir: dirs::cache_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("riptide")
                .join("cache")
                .to_string_lossy()
                .to_string(),
            persistent: true,
            auto_save_interval_seconds: 300, // 5 minutes
        }
    }
}

/// Cache warm-up options
#[derive(Debug, Clone)]
pub struct WarmOptions {
    /// URLs to warm
    pub urls: Vec<String>,

    /// Maximum concurrent requests
    pub concurrency: usize,

    /// Timeout per request in seconds
    pub timeout_seconds: u64,

    /// Retry failed requests
    pub retry_failures: bool,

    /// Maximum retries per URL
    pub max_retries: u32,
}

impl Default for WarmOptions {
    fn default() -> Self {
        Self {
            urls: Vec::new(),
            concurrency: 10,
            timeout_seconds: 30,
            retry_failures: true,
            max_retries: 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_entry_creation() {
        let entry = CacheEntry::new(
            "https://example.com/page".to_string(),
            "<html>content</html>".to_string(),
            "text/html".to_string(),
        );

        assert_eq!(entry.url, "https://example.com/page");
        assert_eq!(entry.domain, "example.com");
        assert_eq!(entry.size_bytes, 20); // "<html>content</html>" is 20 bytes
        assert_eq!(entry.access_count, 0);
    }

    #[test]
    fn test_cache_entry_touch() {
        let mut entry = CacheEntry::new(
            "https://example.com/page".to_string(),
            "content".to_string(),
            "text/html".to_string(),
        );

        assert_eq!(entry.access_count, 0);
        entry.touch();
        assert_eq!(entry.access_count, 1);
        entry.touch();
        assert_eq!(entry.access_count, 2);
    }

    #[test]
    fn test_cache_stats_hit_rate() {
        let mut stats = CacheStats::new();
        assert_eq!(stats.hit_rate(), 0.0);

        stats.record_hit();
        stats.record_hit();
        stats.record_miss();

        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert!((stats.hit_rate() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_cache_stats_insertion() {
        let mut stats = CacheStats::new();
        stats.record_insertion("example.com", 1024);
        stats.record_insertion("example.com", 512);
        stats.record_insertion("test.com", 256);

        assert_eq!(stats.total_entries, 3);
        assert_eq!(stats.total_size_bytes, 1792);
        assert_eq!(stats.entries_by_domain.get("example.com"), Some(&2));
        assert_eq!(stats.size_by_domain.get("example.com"), Some(&1536));
    }
}
