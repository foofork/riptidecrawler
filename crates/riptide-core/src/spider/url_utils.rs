use anyhow::{Context, Result};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tracing::{debug, warn};
use url::Url;

/// Configuration for URL deduplication and normalization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlUtilsConfig {
    /// Enable bloom filter for memory-efficient duplicate detection
    pub enable_bloom_filter: bool,
    /// Bloom filter expected capacity
    pub bloom_filter_capacity: usize,
    /// Bloom filter false positive rate
    pub bloom_filter_fpr: f64,
    /// Enable exact duplicate tracking (higher memory usage)
    pub enable_exact_tracking: bool,
    /// Maximum URLs to track exactly before switching to bloom-only
    pub max_exact_urls: usize,
    /// Enable URL normalization
    pub enable_normalization: bool,
    /// Remove fragments from URLs (#section)
    pub remove_fragments: bool,
    /// Sort query parameters for consistency
    pub sort_query_params: bool,
    /// Remove default ports (80 for HTTP, 443 for HTTPS)
    pub remove_default_ports: bool,
    /// Convert hostname to lowercase
    pub lowercase_hostname: bool,
    /// Remove trailing slashes from paths
    pub remove_trailing_slash: bool,
    /// Remove www. prefix from hostnames
    pub remove_www_prefix: bool,
    /// Patterns to exclude from crawling
    pub exclude_patterns: Vec<String>,
    /// File extensions to exclude
    pub exclude_extensions: Vec<String>,
}

impl Default for UrlUtilsConfig {
    fn default() -> Self {
        Self {
            enable_bloom_filter: true,
            bloom_filter_capacity: 1_000_000,
            bloom_filter_fpr: 0.01,
            enable_exact_tracking: true,
            max_exact_urls: 100_000,
            enable_normalization: true,
            remove_fragments: true,
            sort_query_params: true,
            remove_default_ports: true,
            lowercase_hostname: true,
            remove_trailing_slash: true,
            remove_www_prefix: false,
            exclude_patterns: vec![
                r"/admin".to_string(),
                r"/\..*".to_string(), // Hidden files
                r".*\.(css|js|ico|png|jpg|jpeg|gif|svg|woff|woff2|ttf|eot)$".to_string(),
            ],
            exclude_extensions: vec![
                "css".to_string(),
                "js".to_string(),
                "ico".to_string(),
                "png".to_string(),
                "jpg".to_string(),
                "jpeg".to_string(),
                "gif".to_string(),
                "svg".to_string(),
                "woff".to_string(),
                "woff2".to_string(),
                "ttf".to_string(),
                "eot".to_string(),
                "zip".to_string(),
                "tar".to_string(),
                "gz".to_string(),
            ],
        }
    }
}

/// Simple bloom filter implementation for duplicate detection
#[derive(Debug)]
struct BloomFilter {
    bits: Vec<bool>,
    hash_functions: usize,
    capacity: usize,
    insertions: AtomicU64,
}

impl BloomFilter {
    fn new(capacity: usize, fpr: f64) -> Self {
        // Calculate optimal bit array size and hash function count
        let bits_per_element = -(fpr.ln() / (2.0_f64.ln()).powi(2));
        let bit_array_size = (capacity as f64 * bits_per_element).ceil() as usize;
        let hash_functions = (bits_per_element * 2.0_f64.ln()).ceil() as usize;

        Self {
            bits: vec![false; bit_array_size],
            hash_functions,
            capacity,
            insertions: AtomicU64::new(0),
        }
    }

    fn add(&mut self, item: &str) {
        for i in 0..self.hash_functions {
            let hash = self.hash_function(item, i);
            let index = hash % self.bits.len();
            self.bits[index] = true;
        }
        self.insertions.fetch_add(1, Ordering::Relaxed);
    }

    fn contains(&self, item: &str) -> bool {
        for i in 0..self.hash_functions {
            let hash = self.hash_function(item, i);
            let index = hash % self.bits.len();
            if !self.bits[index] {
                return false;
            }
        }
        true
    }

    fn hash_function(&self, item: &str, seed: usize) -> usize {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        seed.hash(&mut hasher);
        item.hash(&mut hasher);
        hasher.finish() as usize
    }

    fn estimated_fpr(&self) -> f64 {
        let insertions = self.insertions.load(Ordering::Relaxed) as f64;
        if insertions == 0.0 {
            return 0.0;
        }

        let bit_ratio = insertions / self.bits.len() as f64;
        (1.0 - (-(self.hash_functions as f64) * bit_ratio).exp()).powi(self.hash_functions as i32)
    }
}

/// URL deduplication and normalization manager
pub struct UrlUtils {
    config: UrlUtilsConfig,

    // Exact tracking for high-value URLs
    exact_urls: Option<DashMap<String, ()>>,

    // Bloom filter for memory-efficient deduplication
    bloom_filter: Arc<tokio::sync::Mutex<Option<BloomFilter>>>,

    // Statistics
    total_processed: AtomicU64,
    duplicates_found: AtomicU64,
    normalized_count: AtomicU64,
    excluded_count: AtomicU64,
}

impl UrlUtils {
    pub fn new(config: UrlUtilsConfig) -> Self {
        let exact_urls = if config.enable_exact_tracking {
            Some(DashMap::new())
        } else {
            None
        };

        let bloom_filter = if config.enable_bloom_filter {
            Arc::new(tokio::sync::Mutex::new(Some(BloomFilter::new(
                config.bloom_filter_capacity,
                config.bloom_filter_fpr,
            ))))
        } else {
            Arc::new(tokio::sync::Mutex::new(None))
        };

        Self {
            config,
            exact_urls,
            bloom_filter,
            total_processed: AtomicU64::new(0),
            duplicates_found: AtomicU64::new(0),
            normalized_count: AtomicU64::new(0),
            excluded_count: AtomicU64::new(0),
        }
    }

    /// Normalize a URL according to configuration
    pub fn normalize_url(&self, url: &Url) -> Result<Url> {
        if !self.config.enable_normalization {
            return Ok(url.clone());
        }

        let mut normalized = url.clone();

        // Convert hostname to lowercase
        if self.config.lowercase_hostname {
            if let Some(host) = normalized.host_str() {
                let lowercase_host = host.to_lowercase();
                normalized.set_host(Some(&lowercase_host))
                    .context("Failed to set lowercase hostname")?;
            }
        }

        // Remove www. prefix
        if self.config.remove_www_prefix {
            if let Some(host) = normalized.host_str().map(|s| s.to_string()) {
                if host.starts_with("www.") && host.len() > 4 {
                    let without_www = &host[4..];
                    normalized.set_host(Some(without_www))
                        .context("Failed to remove www prefix")?;
                }
            }
        }

        // Remove default ports
        if self.config.remove_default_ports {
            if let Some(port) = normalized.port() {
                let scheme = normalized.scheme();
                let is_default = (scheme == "http" && port == 80) ||
                                (scheme == "https" && port == 443);
                if is_default {
                    let _ = normalized.set_port(None);
                }
            }
        }

        // Remove fragments
        if self.config.remove_fragments {
            normalized.set_fragment(None);
        }

        // Sort query parameters
        if self.config.sort_query_params {
            let mut params: Vec<(String, String)> = normalized
                .query_pairs()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();

            if !params.is_empty() {
                params.sort_by(|a, b| a.0.cmp(&b.0));
                let query_string = params
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join("&");
                normalized.set_query(Some(&query_string));
            }
        }

        // Remove trailing slash
        if self.config.remove_trailing_slash {
            let path = normalized.path().to_string();
            if path.len() > 1 && path.ends_with('/') {
                normalized.set_path(&path[..path.len()-1]);
            }
        }

        if normalized != *url {
            self.normalized_count.fetch_add(1, Ordering::Relaxed);
            debug!(
                original = %url,
                normalized = %normalized,
                "URL normalized"
            );
        }

        Ok(normalized)
    }

    /// Check if URL should be excluded from crawling
    pub fn should_exclude_url(&self, url: &Url) -> bool {
        self.total_processed.fetch_add(1, Ordering::Relaxed);

        // Check file extension exclusions
        if let Some(extension) = url.path().split('.').last() {
            if self.config.exclude_extensions.contains(&extension.to_lowercase()) {
                self.excluded_count.fetch_add(1, Ordering::Relaxed);
                debug!(url = %url, extension = %extension, "URL excluded by extension");
                return true;
            }
        }

        // Check pattern exclusions
        for pattern in &self.config.exclude_patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                if regex.is_match(url.as_str()) {
                    self.excluded_count.fetch_add(1, Ordering::Relaxed);
                    debug!(url = %url, pattern = %pattern, "URL excluded by pattern");
                    return true;
                }
            }
        }

        false
    }

    /// Check if URL is a duplicate and mark it as seen
    pub async fn is_duplicate_and_mark(&self, url: &Url) -> Result<bool> {
        let normalized = self.normalize_url(url)?;
        let url_string = normalized.to_string();

        // Check exact tracking first (if enabled and under limit)
        if let Some(ref exact_urls) = self.exact_urls {
            if exact_urls.len() < self.config.max_exact_urls {
                let is_duplicate = exact_urls.contains_key(&url_string);
                if !is_duplicate {
                    exact_urls.insert(url_string.clone(), ());
                } else {
                    self.duplicates_found.fetch_add(1, Ordering::Relaxed);
                    debug!(url = %url, "Duplicate found in exact tracking");
                }
                return Ok(is_duplicate);
            }
        }

        // Fall back to bloom filter
        let mut bloom_guard = self.bloom_filter.lock().await;
        if let Some(ref mut bloom) = *bloom_guard {
            let is_duplicate = bloom.contains(&url_string);
            if !is_duplicate {
                bloom.add(&url_string);
            } else {
                self.duplicates_found.fetch_add(1, Ordering::Relaxed);
                debug!(url = %url, "Potential duplicate found in bloom filter");
            }

            // Check if bloom filter FPR is getting too high
            let current_fpr = bloom.estimated_fpr();
            if current_fpr > self.config.bloom_filter_fpr * 2.0 {
                warn!(
                    current_fpr = current_fpr,
                    target_fpr = self.config.bloom_filter_fpr,
                    "Bloom filter false positive rate is high"
                );
            }

            return Ok(is_duplicate);
        }

        // No deduplication enabled
        Ok(false)
    }

    /// Check if URL is valid for crawling (not excluded, not duplicate)
    pub async fn is_valid_for_crawling(&self, url: &Url) -> Result<bool> {
        // Check exclusions first (cheaper)
        if self.should_exclude_url(url) {
            return Ok(false);
        }

        // Check for duplicates
        if self.is_duplicate_and_mark(url).await? {
            return Ok(false);
        }

        Ok(true)
    }

    /// Extract and filter URLs from a list
    pub async fn filter_urls(&self, urls: Vec<Url>) -> Result<Vec<Url>> {
        let total_input = urls.len();
        let mut valid_urls = Vec::new();

        for url in urls {
            if self.is_valid_for_crawling(&url).await? {
                valid_urls.push(url);
            }
        }

        debug!(
            total_input = total_input,
            valid_output = valid_urls.len(),
            "Filtered URL list"
        );

        Ok(valid_urls)
    }

    /// Get statistics about URL processing
    pub async fn get_stats(&self) -> UrlUtilsStats {
        let bloom_stats = if let Some(ref bloom) = *self.bloom_filter.lock().await {
            Some(BloomFilterStats {
                estimated_fpr: bloom.estimated_fpr(),
                insertions: bloom.insertions.load(Ordering::Relaxed),
                capacity: bloom.capacity,
                hash_functions: bloom.hash_functions,
            })
        } else {
            None
        };

        UrlUtilsStats {
            total_processed: self.total_processed.load(Ordering::Relaxed),
            duplicates_found: self.duplicates_found.load(Ordering::Relaxed),
            normalized_count: self.normalized_count.load(Ordering::Relaxed),
            excluded_count: self.excluded_count.load(Ordering::Relaxed),
            exact_urls_count: self.exact_urls.as_ref().map(|m| m.len()).unwrap_or(0),
            bloom_filter_stats: bloom_stats,
        }
    }

    /// Clear all tracking data
    pub async fn clear(&self) {
        if let Some(ref exact_urls) = self.exact_urls {
            exact_urls.clear();
        }

        if self.config.enable_bloom_filter {
            let mut bloom_guard = self.bloom_filter.lock().await;
            *bloom_guard = Some(BloomFilter::new(
                self.config.bloom_filter_capacity,
                self.config.bloom_filter_fpr,
            ));
        }

        // Reset counters
        self.total_processed.store(0, Ordering::Relaxed);
        self.duplicates_found.store(0, Ordering::Relaxed);
        self.normalized_count.store(0, Ordering::Relaxed);
        self.excluded_count.store(0, Ordering::Relaxed);
    }

    /// Get configuration
    pub fn get_config(&self) -> &UrlUtilsConfig {
        &self.config
    }

    /// Update configuration (requires restart for some changes)
    pub fn update_config(&mut self, config: UrlUtilsConfig) {
        self.config = config;
    }
}

/// Statistics about URL processing
#[derive(Debug, Clone)]
pub struct UrlUtilsStats {
    pub total_processed: u64,
    pub duplicates_found: u64,
    pub normalized_count: u64,
    pub excluded_count: u64,
    pub exact_urls_count: usize,
    pub bloom_filter_stats: Option<BloomFilterStats>,
}

/// Bloom filter statistics
#[derive(Debug, Clone)]
pub struct BloomFilterStats {
    pub estimated_fpr: f64,
    pub insertions: u64,
    pub capacity: usize,
    pub hash_functions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_url_normalization() {
        let config = UrlUtilsConfig::default();
        let url_utils = UrlUtils::new(config);

        let url = Url::from_str("https://WWW.Example.COM:443/path/?b=2&a=1#fragment").expect("Valid URL");
        let normalized = url_utils.normalize_url(&url).expect("Normalization should work");

        assert_eq!(normalized.host_str().unwrap(), "example.com");
        assert_eq!(normalized.port(), None); // Default port removed
        assert_eq!(normalized.fragment(), None); // Fragment removed
        assert_eq!(normalized.query(), Some("a=1&b=2")); // Sorted params
    }

    #[test]
    fn test_url_exclusion() {
        let config = UrlUtilsConfig::default();
        let url_utils = UrlUtils::new(config);

        let css_url = Url::from_str("https://example.com/style.css").expect("Valid URL");
        let html_url = Url::from_str("https://example.com/page.html").expect("Valid URL");
        let admin_url = Url::from_str("https://example.com/admin/panel").expect("Valid URL");

        assert!(url_utils.should_exclude_url(&css_url));
        assert!(!url_utils.should_exclude_url(&html_url));
        assert!(url_utils.should_exclude_url(&admin_url));
    }

    #[tokio::test]
    async fn test_duplicate_detection() {
        let config = UrlUtilsConfig::default();
        let url_utils = UrlUtils::new(config);

        let url1 = Url::from_str("https://example.com/page").expect("Valid URL");
        let url2 = Url::from_str("https://example.com/page").expect("Valid URL");
        let url3 = Url::from_str("https://example.com/other").expect("Valid URL");

        // First occurrence should not be duplicate
        assert!(!url_utils.is_duplicate_and_mark(&url1).await.expect("Should work"));

        // Second occurrence should be duplicate
        assert!(url_utils.is_duplicate_and_mark(&url2).await.expect("Should work"));

        // Different URL should not be duplicate
        assert!(!url_utils.is_duplicate_and_mark(&url3).await.expect("Should work"));
    }

    #[tokio::test]
    async fn test_url_filtering() {
        let config = UrlUtilsConfig::default();
        let url_utils = UrlUtils::new(config);

        let urls = vec![
            Url::from_str("https://example.com/page1.html").expect("Valid URL"),
            Url::from_str("https://example.com/style.css").expect("Valid URL"), // Should be excluded
            Url::from_str("https://example.com/page2.html").expect("Valid URL"),
            Url::from_str("https://example.com/page1.html").expect("Valid URL"), // Duplicate
        ];

        let filtered = url_utils.filter_urls(urls).await.expect("Filtering should work");

        // Should have 2 unique, non-excluded URLs
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|url| url.path().ends_with(".html")));
    }

    #[test]
    fn test_bloom_filter() {
        let mut bloom = BloomFilter::new(1000, 0.01);

        assert!(!bloom.contains("test1"));
        bloom.add("test1");
        assert!(bloom.contains("test1"));

        bloom.add("test2");
        assert!(bloom.contains("test2"));
        assert!(bloom.contains("test1")); // Should still be there

        assert!(!bloom.contains("test3")); // Should not be there
    }

    #[tokio::test]
    async fn test_statistics() {
        let config = UrlUtilsConfig::default();
        let url_utils = UrlUtils::new(config);

        let url1 = Url::from_str("https://example.com/page").expect("Valid URL");
        let url2 = Url::from_str("https://example.com/style.css").expect("Valid URL");

        // Process some URLs
        let _ = url_utils.is_valid_for_crawling(&url1).await;
        let _ = url_utils.is_valid_for_crawling(&url2).await;
        let _ = url_utils.is_valid_for_crawling(&url1).await; // Duplicate

        let stats = url_utils.get_stats().await;
        assert!(stats.total_processed > 0);
        assert!(stats.excluded_count > 0); // CSS file should be excluded
        assert!(stats.duplicates_found > 0); // Duplicate should be detected
    }
}