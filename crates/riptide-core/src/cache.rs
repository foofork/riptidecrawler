use anyhow::Result;
use chrono::{DateTime, Utc};
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Default cache TTL (24 hours)
const DEFAULT_CACHE_TTL: u64 = 24 * 60 * 60; // 24 hours in seconds

/// Maximum content size (20MB)
const MAX_CONTENT_SIZE: usize = 20 * 1024 * 1024; // 20MB

/// Cache configuration with security and performance settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Default TTL in seconds
    pub default_ttl: u64,
    /// Maximum content size in bytes
    pub max_content_size: usize,
    /// Cache key version for invalidation
    pub cache_version: String,
    /// Enable ETag support
    pub enable_etag: bool,
    /// Enable Last-Modified support
    pub enable_last_modified: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            default_ttl: DEFAULT_CACHE_TTL,
            max_content_size: MAX_CONTENT_SIZE,
            cache_version: "v1".to_string(),
            enable_etag: true,
            enable_last_modified: true,
        }
    }
}

/// Cache entry with HTTP caching metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    /// Cached data
    pub data: T,
    /// ETag for conditional requests
    pub etag: Option<String>,
    /// Last-Modified timestamp
    pub last_modified: Option<DateTime<Utc>>,
    /// Cache creation timestamp
    pub created_at: DateTime<Utc>,
    /// TTL in seconds
    pub ttl: u64,
    /// Content size in bytes
    pub content_size: usize,
    /// Cache key metadata
    pub metadata: CacheMetadata,
}

/// Cache key metadata for version-aware caching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    /// Extractor version
    pub extractor_version: String,
    /// Extraction options hash
    pub options_hash: String,
    /// URL hash for collision detection
    pub url_hash: String,
    /// Content type
    pub content_type: Option<String>,
}

/// Enhanced cache manager with security and HTTP caching support
pub struct CacheManager {
    conn: MultiplexedConnection,
    config: CacheConfig,
}

impl CacheManager {
    pub async fn new(redis_url: &str) -> Result<Self> {
        let client = Client::open(redis_url)?;
        let conn = client.get_multiplexed_tokio_connection().await?;
        Ok(Self {
            conn,
            config: CacheConfig::default(),
        })
    }

    pub async fn new_with_config(redis_url: &str, config: CacheConfig) -> Result<Self> {
        let client = Client::open(redis_url)?;
        let conn = client.get_multiplexed_tokio_connection().await?;
        Ok(Self { conn, config })
    }

    /// Generate version-aware cache key
    pub fn generate_cache_key(
        &self,
        url: &str,
        extractor_version: &str,
        options: &HashMap<String, String>,
    ) -> String {
        let mut hasher = Sha256::new();

        // Hash URL
        hasher.update(url.as_bytes());
        let url_hash = format!("{:x}", hasher.finalize_reset());

        // Hash extraction options
        let mut options_vec: Vec<_> = options.iter().collect();
        options_vec.sort_by_key(|(k, _)| *k);
        for (key, value) in options_vec {
            hasher.update(key.as_bytes());
            hasher.update(value.as_bytes());
        }
        let options_hash = format!("{:x}", hasher.finalize());

        format!(
            "riptide:{}:{}:{}:{}",
            self.config.cache_version,
            extractor_version,
            &url_hash[..16], // First 16 chars for brevity
            &options_hash[..16]
        )
    }

    /// Get cached entry with HTTP caching metadata
    pub async fn get<T: for<'de> Deserialize<'de>>(
        &mut self,
        key: &str,
    ) -> Result<Option<CacheEntry<T>>> {
        let data: Option<Vec<u8>> = self.conn.get(key).await?;
        match data {
            Some(bytes) => {
                let entry: CacheEntry<T> = serde_json::from_slice(&bytes)?;

                // Check if entry has expired
                let age = Utc::now().signed_duration_since(entry.created_at);
                if age.num_seconds() > entry.ttl as i64 {
                    debug!(key = %key, "Cache entry expired, removing");
                    let _ = self.delete(key).await; // Clean up expired entry
                    return Ok(None);
                }

                debug!(key = %key, age_seconds = age.num_seconds(), "Cache hit");
                Ok(Some(entry))
            }
            None => {
                debug!(key = %key, "Cache miss");
                Ok(None)
            }
        }
    }

    /// Get with simple value (backward compatibility)
    pub async fn get_simple<T: for<'de> Deserialize<'de>>(
        &mut self,
        key: &str,
    ) -> Result<Option<T>> {
        match self.get::<T>(key).await? {
            Some(entry) => Ok(Some(entry.data)),
            None => Ok(None),
        }
    }

    /// Set cached entry with HTTP caching metadata and validation
    pub async fn set<T: Serialize>(
        &mut self,
        key: &str,
        value: &T,
        metadata: CacheMetadata,
        etag: Option<String>,
        last_modified: Option<DateTime<Utc>>,
        custom_ttl: Option<u64>,
    ) -> Result<()> {
        // Serialize data to check size
        let data_bytes = serde_json::to_vec(value)?;
        let content_size = data_bytes.len();

        // Validate content size
        if content_size > self.config.max_content_size {
            warn!(
                key = %key,
                size = content_size,
                max_size = self.config.max_content_size,
                "Content exceeds maximum cache size, skipping cache"
            );
            return Err(anyhow::anyhow!(
                "Content size {} exceeds maximum {} bytes",
                content_size,
                self.config.max_content_size
            ));
        }

        let ttl = custom_ttl.unwrap_or(self.config.default_ttl);

        let entry = CacheEntry {
            data: value,
            etag: if self.config.enable_etag { etag } else { None },
            last_modified: if self.config.enable_last_modified {
                last_modified
            } else {
                None
            },
            created_at: Utc::now(),
            ttl,
            content_size,
            metadata,
        };

        let entry_bytes = serde_json::to_vec(&entry)?;
        self.conn.set_ex::<_, _, ()>(key, entry_bytes, ttl).await?;

        info!(
            key = %key,
            size = content_size,
            ttl = ttl,
            "Cached entry stored"
        );
        Ok(())
    }

    /// Set with simple value (backward compatibility)
    pub async fn set_simple<T: Serialize>(
        &mut self,
        key: &str,
        value: &T,
        ttl_secs: u64,
    ) -> Result<()> {
        let metadata = CacheMetadata {
            extractor_version: "legacy".to_string(),
            options_hash: "none".to_string(),
            url_hash: "unknown".to_string(),
            content_type: None,
        };

        self.set(key, value, metadata, None, None, Some(ttl_secs))
            .await
    }

    /// Check if cached content matches conditional request headers
    pub async fn check_conditional<T: for<'de> Deserialize<'de>>(
        &mut self,
        key: &str,
        if_none_match: Option<&str>, // ETag from If-None-Match header
        if_modified_since: Option<DateTime<Utc>>, // DateTime from If-Modified-Since
    ) -> Result<ConditionalResult<T>> {
        match self.get::<T>(key).await? {
            Some(entry) => {
                // Check ETag
                if let (Some(client_etag), Some(cache_etag)) = (if_none_match, &entry.etag) {
                    if client_etag == cache_etag {
                        debug!(key = %key, etag = %cache_etag, "ETag match - not modified");
                        return Ok(ConditionalResult::NotModified(entry));
                    }
                }

                // Check Last-Modified
                if let (Some(if_modified), Some(last_modified)) =
                    (if_modified_since, &entry.last_modified)
                {
                    if *last_modified <= if_modified {
                        debug!(key = %key, last_modified = %last_modified, "Not modified since last request");
                        return Ok(ConditionalResult::NotModified(entry));
                    }
                }

                debug!(key = %key, "Content modified - returning cached data");
                Ok(ConditionalResult::Modified(entry))
            }
            None => Ok(ConditionalResult::Miss),
        }
    }

    /// Get cache statistics
    pub async fn get_stats(&mut self) -> Result<CacheStats> {
        // Get Redis info
        let info: String = redis::cmd("INFO")
            .arg("memory")
            .query_async(&mut self.conn)
            .await
            .unwrap_or_default();

        let memory_usage = Self::parse_redis_memory(&info);

        // Count keys with our prefix
        let pattern = format!("riptide:{}:*", self.config.cache_version);
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&pattern)
            .query_async(&mut self.conn)
            .await
            .unwrap_or_default();

        Ok(CacheStats {
            total_keys: keys.len() as u64,
            memory_usage_bytes: memory_usage,
            cache_version: self.config.cache_version.clone(),
            max_content_size: self.config.max_content_size,
            default_ttl: self.config.default_ttl,
        })
    }

    fn parse_redis_memory(info: &str) -> u64 {
        for line in info.lines() {
            if line.starts_with("used_memory:") {
                if let Some(value) = line.split(':').nth(1) {
                    return value.trim().parse().unwrap_or(0);
                }
            }
        }
        0
    }

    pub async fn delete(&mut self, key: &str) -> Result<()> {
        let deleted: u64 = self.conn.del(key).await?;
        if deleted > 0 {
            debug!(key = %key, "Cache entry deleted");
        }
        Ok(())
    }

    /// Warm the cache with frequently accessed keys
    /// This should be called at startup or after cache clear operations
    pub async fn warm_cache(&mut self, frequent_keys: Vec<String>) -> Result<u32> {
        let mut warmed_count = 0;

        for key in frequent_keys {
            // Check if key exists, if not it will be populated on first access
            let exists: bool = self.conn.exists(&key).await?;
            if exists {
                warmed_count += 1;
                debug!(key = %key, "Cache key already warmed");
            } else {
                debug!(key = %key, "Cache key will be populated on first access");
            }
        }

        info!(warmed_count = warmed_count, "Cache warming completed");
        Ok(warmed_count)
    }

    /// Clear all cache entries for this version
    pub async fn clear_cache(&mut self) -> Result<u64> {
        let pattern = format!("riptide:{}:*", self.config.cache_version);
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&pattern)
            .query_async(&mut self.conn)
            .await
            .unwrap_or_default();

        if keys.is_empty() {
            return Ok(0);
        }

        let deleted: u64 = self.conn.del(&keys).await?;
        info!(deleted = deleted, pattern = %pattern, "Cache cleared");
        Ok(deleted)
    }
}

/// Result of conditional cache check
#[derive(Debug)]
pub enum ConditionalResult<T> {
    /// Content not modified, return 304
    NotModified(CacheEntry<T>),
    /// Content modified, return cached data
    Modified(CacheEntry<T>),
    /// Cache miss
    Miss,
}

/// Cache statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_keys: u64,
    pub memory_usage_bytes: u64,
    pub cache_version: String,
    pub max_content_size: usize,
    pub default_ttl: u64,
}
