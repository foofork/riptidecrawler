//! Cache management module
//!
//! This module provides comprehensive caching functionality for RipTide CLI including:
//! - LRU eviction policy
//! - Domain-based filtering
//! - Persistent storage
//! - Cache warming/preloading
//! - Statistics tracking

pub mod manager;
pub mod storage;
pub mod types;

pub use manager::CacheManager;
pub use storage::CacheStorage;
pub use types::{CacheConfig, CacheEntry, CacheStats, WarmOptions};

use anyhow::Result;
use std::sync::Arc;

/// Unified cache system combining manager and storage
pub struct Cache {
    /// Cache manager
    manager: Arc<CacheManager>,

    /// Persistent storage
    storage: Arc<CacheStorage>,

    /// Whether persistence is enabled
    persistent: bool,
}

impl Cache {
    /// Create new cache with default configuration
    pub async fn new() -> Result<Self> {
        let config = CacheConfig::default();
        Self::with_config(config).await
    }

    /// Create cache with custom configuration
    pub async fn with_config(config: CacheConfig) -> Result<Self> {
        let persistent = config.persistent;
        let manager = Arc::new(CacheManager::with_config(config.clone())?);
        let storage = Arc::new(CacheStorage::new(&config.cache_dir)?);

        if persistent {
            storage.initialize().await?;
        }

        let cache = Self {
            manager,
            storage,
            persistent,
        };

        // Load existing cache if persistent
        if persistent {
            cache.load().await?;
        }

        Ok(cache)
    }

    /// Get entry from cache
    pub async fn get(&self, url: &str) -> Result<Option<CacheEntry>> {
        self.manager.get(url).await
    }

    /// Insert entry into cache
    pub async fn insert(&self, entry: CacheEntry) -> Result<()> {
        self.manager.insert(entry).await?;

        if self.persistent {
            self.save().await?;
        }

        Ok(())
    }

    /// Remove entry from cache
    pub async fn remove(&self, url: &str) -> Result<bool> {
        let removed = self.manager.remove(url).await?;

        if removed && self.persistent {
            self.save().await?;
        }

        Ok(removed)
    }

    /// Clear all cache entries
    pub async fn clear(&self) -> Result<()> {
        self.manager.clear().await?;

        if self.persistent {
            self.storage.clear_all().await?;
        }

        Ok(())
    }

    /// Clear cache for a specific domain
    pub async fn clear_domain(&self, domain: &str) -> Result<usize> {
        let count = self.manager.clear_domain(domain).await?;

        if count > 0 && self.persistent {
            self.save().await?;
        }

        Ok(count)
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> Result<CacheStats> {
        self.manager.get_stats().await
    }

    /// List all cached URLs
    pub async fn list_urls(&self) -> Result<Vec<String>> {
        self.manager.list_urls().await
    }

    /// List URLs for a specific domain
    pub async fn list_domain_urls(&self, domain: &str) -> Result<Vec<String>> {
        self.manager.list_domain_urls(domain).await
    }

    /// Clean up expired entries
    pub async fn cleanup_expired(&self) -> Result<usize> {
        let count = self.manager.cleanup_expired().await?;

        if count > 0 && self.persistent {
            self.save().await?;
        }

        Ok(count)
    }

    /// Warm cache by prefetching URLs
    pub async fn warm(&self, options: WarmOptions) -> Result<WarmResult> {
        use futures::stream::{self, StreamExt};

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(options.timeout_seconds))
            .build()?;

        let mut result = WarmResult::default();
        result.total_urls = options.urls.len();

        let results: Vec<_> = stream::iter(options.urls)
            .map(|url| {
                let client = client.clone();
                let max_retries = options.max_retries;
                let retry_failures = options.retry_failures;

                async move {
                    let mut attempts = 0;
                    loop {
                        attempts += 1;
                        match client.get(&url).send().await {
                            Ok(response) => {
                                let status = response.status().as_u16();
                                let content_type = response
                                    .headers()
                                    .get("content-type")
                                    .and_then(|v| v.to_str().ok())
                                    .unwrap_or("text/html")
                                    .to_string();

                                match response.text().await {
                                    Ok(content) => {
                                        let mut entry =
                                            CacheEntry::new(url.clone(), content, content_type);
                                        entry.status_code = status;
                                        return Ok(entry);
                                    }
                                    Err(e) => {
                                        if retry_failures && attempts < max_retries {
                                            continue;
                                        }
                                        return Err(anyhow::anyhow!(
                                            "Failed to read response: {}",
                                            e
                                        ));
                                    }
                                }
                            }
                            Err(e) => {
                                if retry_failures && attempts < max_retries {
                                    continue;
                                }
                                return Err(anyhow::anyhow!("Request failed: {}", e));
                            }
                        }
                    }
                }
            })
            .buffer_unordered(options.concurrency)
            .collect()
            .await;

        for entry_result in results {
            match entry_result {
                Ok(entry) => {
                    self.insert(entry).await?;
                    result.successful += 1;
                }
                Err(_) => {
                    result.failed += 1;
                }
            }
        }

        Ok(result)
    }

    /// Save cache to disk
    async fn save(&self) -> Result<()> {
        let urls = self.manager.list_urls().await?;
        let mut entries = Vec::new();

        for url in urls {
            if let Some(entry) = self.manager.get(&url).await? {
                entries.push(entry);
            }
        }

        self.storage.save_entries(&entries).await?;

        let stats = self.manager.get_stats().await?;
        self.storage.save_stats(&stats).await?;

        Ok(())
    }

    /// Load cache from disk
    async fn load(&self) -> Result<()> {
        let entries = self.storage.load_entries().await?;

        for entry in entries {
            // Skip expired entries
            if !entry.is_expired() {
                self.manager.insert(entry).await?;
            }
        }

        Ok(())
    }

    /// Get cache manager reference
    pub fn manager(&self) -> &Arc<CacheManager> {
        &self.manager
    }

    /// Get cache storage reference
    pub fn storage(&self) -> &Arc<CacheStorage> {
        &self.storage
    }
}

/// Result of cache warming operation
#[derive(Debug, Clone, Default)]
pub struct WarmResult {
    /// Total URLs attempted
    pub total_urls: usize,

    /// Successfully cached URLs
    pub successful: usize,

    /// Failed URLs
    pub failed: usize,
}

impl WarmResult {
    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_urls == 0 {
            return 0.0;
        }
        (self.successful as f64 / self.total_urls as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_creation() {
        let cache = Cache::new().await.unwrap();
        let stats = cache.get_stats().await.unwrap();
        assert_eq!(stats.total_entries, 0);
    }

    #[tokio::test]
    async fn test_cache_insert_and_get() {
        let cache = Cache::new().await.unwrap();

        let entry = CacheEntry::new(
            "https://example.com".to_string(),
            "content".to_string(),
            "text/html".to_string(),
        );

        cache.insert(entry).await.unwrap();

        let retrieved = cache.get("https://example.com").await.unwrap();
        assert!(retrieved.is_some());
    }
}
