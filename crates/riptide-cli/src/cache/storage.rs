//! Cache persistence and storage management
//!
//! This module handles saving and loading cache data to/from disk
#![allow(dead_code)]

use super::types::{CacheEntry, CacheStats};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::fs;

/// Cache storage for persistence
pub struct CacheStorage {
    /// Cache directory path
    cache_dir: PathBuf,

    /// Entries file path
    entries_path: PathBuf,

    /// Statistics file path
    stats_path: PathBuf,
}

impl CacheStorage {
    /// Create new cache storage
    pub fn new(cache_dir: impl AsRef<Path>) -> Result<Self> {
        let cache_dir = cache_dir.as_ref().to_path_buf();
        let entries_path = cache_dir.join("entries.json");
        let stats_path = cache_dir.join("stats.json");

        Ok(Self {
            cache_dir,
            entries_path,
            stats_path,
        })
    }

    /// Initialize storage directory
    pub async fn initialize(&self) -> Result<()> {
        fs::create_dir_all(&self.cache_dir)
            .await
            .context("Failed to create cache directory")?;
        Ok(())
    }

    /// Load cache entries from disk
    pub async fn load_entries(&self) -> Result<Vec<CacheEntry>> {
        if !self.entries_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.entries_path)
            .await
            .context("Failed to read cache entries file")?;

        let entries: Vec<CacheEntry> =
            serde_json::from_str(&content).context("Failed to parse cache entries")?;

        Ok(entries)
    }

    /// Save cache entries to disk
    pub async fn save_entries(&self, entries: &[CacheEntry]) -> Result<()> {
        self.initialize().await?;

        let content =
            serde_json::to_string_pretty(entries).context("Failed to serialize cache entries")?;

        fs::write(&self.entries_path, content)
            .await
            .context("Failed to write cache entries file")?;

        Ok(())
    }

    /// Load cache statistics from disk
    pub async fn load_stats(&self) -> Result<Option<CacheStats>> {
        if !self.stats_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&self.stats_path)
            .await
            .context("Failed to read cache stats file")?;

        let stats: CacheStats =
            serde_json::from_str(&content).context("Failed to parse cache statistics")?;

        Ok(Some(stats))
    }

    /// Save cache statistics to disk
    pub async fn save_stats(&self, stats: &CacheStats) -> Result<()> {
        self.initialize().await?;

        let content =
            serde_json::to_string_pretty(stats).context("Failed to serialize cache statistics")?;

        fs::write(&self.stats_path, content)
            .await
            .context("Failed to write cache stats file")?;

        Ok(())
    }

    /// Clear all cached data
    pub async fn clear_all(&self) -> Result<()> {
        if self.entries_path.exists() {
            fs::remove_file(&self.entries_path)
                .await
                .context("Failed to remove cache entries file")?;
        }

        if self.stats_path.exists() {
            fs::remove_file(&self.stats_path)
                .await
                .context("Failed to remove cache stats file")?;
        }

        Ok(())
    }

    /// Get cache directory path
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Get total cache size on disk
    pub async fn get_disk_usage(&self) -> Result<u64> {
        let mut total_size = 0u64;

        if self.entries_path.exists() {
            let metadata = fs::metadata(&self.entries_path).await?;
            total_size += metadata.len();
        }

        if self.stats_path.exists() {
            let metadata = fs::metadata(&self.stats_path).await?;
            total_size += metadata.len();
        }

        Ok(total_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::types::CacheEntry;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_storage_initialization() {
        let temp_dir = TempDir::new().unwrap();
        let storage = CacheStorage::new(temp_dir.path()).unwrap();

        storage.initialize().await.unwrap();
        assert!(temp_dir.path().exists());
    }

    #[tokio::test]
    async fn test_save_and_load_entries() {
        let temp_dir = TempDir::new().unwrap();
        let storage = CacheStorage::new(temp_dir.path()).unwrap();

        let entries = vec![
            CacheEntry::new(
                "https://example.com/1".to_string(),
                "content1".to_string(),
                "text/html".to_string(),
            ),
            CacheEntry::new(
                "https://example.com/2".to_string(),
                "content2".to_string(),
                "text/html".to_string(),
            ),
        ];

        storage.save_entries(&entries).await.unwrap();
        let loaded = storage.load_entries().await.unwrap();

        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].url, "https://example.com/1");
        assert_eq!(loaded[1].url, "https://example.com/2");
    }

    #[tokio::test]
    async fn test_save_and_load_stats() {
        let temp_dir = TempDir::new().unwrap();
        let storage = CacheStorage::new(temp_dir.path()).unwrap();

        let mut stats = CacheStats::new();
        stats.record_hit();
        stats.record_miss();
        stats.record_insertion("example.com", 1024);

        storage.save_stats(&stats).await.unwrap();
        let loaded = storage.load_stats().await.unwrap();

        assert!(loaded.is_some());
        let loaded_stats = loaded.unwrap();
        assert_eq!(loaded_stats.hits, 1);
        assert_eq!(loaded_stats.misses, 1);
    }

    #[tokio::test]
    async fn test_clear_all() {
        let temp_dir = TempDir::new().unwrap();
        let storage = CacheStorage::new(temp_dir.path()).unwrap();

        let entries = vec![CacheEntry::new(
            "https://example.com".to_string(),
            "content".to_string(),
            "text/html".to_string(),
        )];

        storage.save_entries(&entries).await.unwrap();
        storage.save_stats(&CacheStats::new()).await.unwrap();

        storage.clear_all().await.unwrap();

        let loaded_entries = storage.load_entries().await.unwrap();
        let loaded_stats = storage.load_stats().await.unwrap();

        assert!(loaded_entries.is_empty());
        assert!(loaded_stats.is_none());
    }
}
