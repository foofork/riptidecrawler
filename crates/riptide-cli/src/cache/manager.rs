//! Cache manager with LRU eviction policy
//!
//! This module provides the core cache management functionality including:
//! - LRU (Least Recently Used) eviction
//! - Size and count-based limits
//! - Domain filtering
//! - Thread-safe operations

use super::types::{CacheConfig, CacheEntry, CacheStats};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Cache manager with LRU eviction policy
pub struct CacheManager {
    /// Cache entries indexed by URL
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,

    /// Cache statistics
    stats: Arc<RwLock<CacheStats>>,

    /// Configuration
    config: CacheConfig,
}

impl CacheManager {
    /// Create new cache manager with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(CacheConfig::default())
    }

    /// Create cache manager with custom configuration
    pub fn with_config(config: CacheConfig) -> Result<Self> {
        Ok(Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(CacheStats::new())),
            config,
        })
    }

    /// Get entry from cache
    pub async fn get(&self, url: &str) -> Result<Option<CacheEntry>> {
        let mut entries = self.entries.write().await;
        let mut stats = self.stats.write().await;

        if let Some(entry) = entries.get_mut(url) {
            // Check if expired
            if entry.is_expired() {
                entries.remove(url);
                stats.record_miss();
                return Ok(None);
            }

            // Update access metadata
            entry.touch();
            stats.record_hit();
            Ok(Some(entry.clone()))
        } else {
            stats.record_miss();
            Ok(None)
        }
    }

    /// Insert entry into cache
    pub async fn insert(&self, mut entry: CacheEntry) -> Result<()> {
        let mut entries = self.entries.write().await;
        let mut stats = self.stats.write().await;

        // Check if we need to evict entries
        self.evict_if_needed(&mut entries, &mut stats, entry.size_bytes)
            .await?;

        // Set TTL if configured
        if entry.ttl_seconds.is_none() {
            entry.ttl_seconds = self.config.default_ttl_seconds;
        }

        let domain = entry.domain.clone();
        let size = entry.size_bytes;

        entries.insert(entry.url.clone(), entry);
        stats.record_insertion(&domain, size);

        Ok(())
    }

    /// Remove entry from cache
    pub async fn remove(&self, url: &str) -> Result<bool> {
        let mut entries = self.entries.write().await;
        let mut stats = self.stats.write().await;

        if let Some(entry) = entries.remove(url) {
            stats.record_eviction(&entry.domain, entry.size_bytes);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Clear all cache entries
    pub async fn clear(&self) -> Result<()> {
        let mut entries = self.entries.write().await;
        let mut stats = self.stats.write().await;

        entries.clear();
        *stats = CacheStats::new();

        Ok(())
    }

    /// Clear cache entries for a specific domain
    pub async fn clear_domain(&self, domain: &str) -> Result<usize> {
        let mut entries = self.entries.write().await;
        let mut stats = self.stats.write().await;

        let to_remove: Vec<String> = entries
            .iter()
            .filter(|(_, entry)| entry.domain == domain)
            .map(|(url, _)| url.clone())
            .collect();

        let count = to_remove.len();
        for url in to_remove {
            if let Some(entry) = entries.remove(&url) {
                stats.record_eviction(&entry.domain, entry.size_bytes);
            }
        }

        Ok(count)
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> Result<CacheStats> {
        let stats = self.stats.read().await;
        Ok(stats.clone())
    }

    /// Get all URLs in cache
    pub async fn list_urls(&self) -> Result<Vec<String>> {
        let entries = self.entries.read().await;
        Ok(entries.keys().cloned().collect())
    }

    /// Get URLs for a specific domain
    pub async fn list_domain_urls(&self, domain: &str) -> Result<Vec<String>> {
        let entries = self.entries.read().await;
        Ok(entries
            .iter()
            .filter(|(_, entry)| entry.domain == domain)
            .map(|(url, _)| url.clone())
            .collect())
    }

    /// Evict entries if necessary to make room for new entry
    async fn evict_if_needed(
        &self,
        entries: &mut HashMap<String, CacheEntry>,
        stats: &mut CacheStats,
        new_entry_size: u64,
    ) -> Result<()> {
        // Check size limit
        while stats.total_size_bytes + new_entry_size > self.config.max_size_bytes
            && !entries.is_empty()
        {
            self.evict_lru_entry(entries, stats).await?;
        }

        // Check entry count limit
        while entries.len() >= self.config.max_entries && !entries.is_empty() {
            self.evict_lru_entry(entries, stats).await?;
        }

        Ok(())
    }

    /// Evict the least recently used entry
    async fn evict_lru_entry(
        &self,
        entries: &mut HashMap<String, CacheEntry>,
        stats: &mut CacheStats,
    ) -> Result<()> {
        // Find LRU entry (oldest last_accessed)
        let lru_url = entries
            .iter()
            .min_by_key(|(_, entry)| entry.last_accessed)
            .map(|(url, _)| url.clone());

        if let Some(url) = lru_url {
            if let Some(entry) = entries.remove(&url) {
                stats.record_eviction(&entry.domain, entry.size_bytes);
            }
        }

        Ok(())
    }

    /// Remove expired entries
    pub async fn cleanup_expired(&self) -> Result<usize> {
        let mut entries = self.entries.write().await;
        let mut stats = self.stats.write().await;

        let expired_urls: Vec<String> = entries
            .iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(url, _)| url.clone())
            .collect();

        let count = expired_urls.len();
        for url in expired_urls {
            if let Some(entry) = entries.remove(&url) {
                stats.record_eviction(&entry.domain, entry.size_bytes);
            }
        }

        Ok(count)
    }

    /// Get configuration
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_insert_and_get() {
        let manager = CacheManager::new().unwrap();

        let entry = CacheEntry::new(
            "https://example.com/page".to_string(),
            "content".to_string(),
            "text/html".to_string(),
        );

        manager.insert(entry).await.unwrap();

        let retrieved = manager.get("https://example.com/page").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().content, "content");
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let manager = CacheManager::new().unwrap();
        let result = manager.get("https://nonexistent.com").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let manager = CacheManager::new().unwrap();

        let entry = CacheEntry::new(
            "https://example.com/page".to_string(),
            "content".to_string(),
            "text/html".to_string(),
        );

        manager.insert(entry).await.unwrap();
        manager.clear().await.unwrap();

        let result = manager.get("https://example.com/page").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_domain_clearing() {
        let manager = CacheManager::new().unwrap();

        let entry1 = CacheEntry::new(
            "https://example.com/page1".to_string(),
            "content1".to_string(),
            "text/html".to_string(),
        );

        let entry2 = CacheEntry::new(
            "https://example.com/page2".to_string(),
            "content2".to_string(),
            "text/html".to_string(),
        );

        let entry3 = CacheEntry::new(
            "https://other.com/page".to_string(),
            "content3".to_string(),
            "text/html".to_string(),
        );

        manager.insert(entry1).await.unwrap();
        manager.insert(entry2).await.unwrap();
        manager.insert(entry3).await.unwrap();

        let cleared = manager.clear_domain("example.com").await.unwrap();
        assert_eq!(cleared, 2);

        let result = manager.get("https://other.com/page").await.unwrap();
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_lru_eviction() {
        let mut config = CacheConfig::default();
        config.max_entries = 2;

        let manager = CacheManager::with_config(config).unwrap();

        let entry1 = CacheEntry::new(
            "https://example.com/1".to_string(),
            "content1".to_string(),
            "text/html".to_string(),
        );

        let entry2 = CacheEntry::new(
            "https://example.com/2".to_string(),
            "content2".to_string(),
            "text/html".to_string(),
        );

        let entry3 = CacheEntry::new(
            "https://example.com/3".to_string(),
            "content3".to_string(),
            "text/html".to_string(),
        );

        manager.insert(entry1).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        manager.insert(entry2).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // This should evict entry1 (oldest)
        manager.insert(entry3).await.unwrap();

        let result1 = manager.get("https://example.com/1").await.unwrap();
        assert!(result1.is_none());

        let result3 = manager.get("https://example.com/3").await.unwrap();
        assert!(result3.is_some());
    }
}
