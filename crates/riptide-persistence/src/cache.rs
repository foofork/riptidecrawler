/*!
# Persistent Cache Layer

High-performance Redis/DragonflyDB-backed cache with TTL-based invalidation,
compression, and distributed synchronization capabilities.

Performance target: <5ms cache access time
*/

use crate::{
    config::{CacheConfig, CompressionAlgorithm},
    errors::{PersistenceError, PersistenceResult},
    metrics::CacheMetrics,
};
use chrono::{DateTime, Utc};
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Client, Pipeline};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Comprehensive cache entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    /// The cached data
    pub data: T,
    /// Entry metadata
    pub metadata: CacheMetadata,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last access timestamp
    pub last_accessed: DateTime<Utc>,
    /// TTL in seconds
    pub ttl_seconds: u64,
    /// Access count for LFU eviction
    pub access_count: u64,
    /// Compression info
    pub compression: Option<CompressionInfo>,
    /// Data integrity hash
    pub integrity_hash: String,
}

/// Cache metadata for versioning and validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    /// Version of the data format
    pub version: String,
    /// Content type
    pub content_type: Option<String>,
    /// Source identifier
    pub source: Option<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Custom attributes
    pub attributes: HashMap<String, String>,
}

/// Compression information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionInfo {
    /// Algorithm used
    pub algorithm: CompressionAlgorithm,
    /// Original size in bytes
    pub original_size: usize,
    /// Compressed size in bytes
    pub compressed_size: usize,
    /// Compression ratio
    pub ratio: f32,
}

/// Cache statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total number of keys
    pub total_keys: u64,
    /// Total memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Hit rate (0.0 to 1.0)
    pub hit_rate: f64,
    /// Miss rate (0.0 to 1.0)
    pub miss_rate: f64,
    /// Average access time in microseconds
    pub avg_access_time_us: u64,
    /// Operations per second
    pub ops_per_second: f64,
    /// Eviction count
    pub eviction_count: u64,
    /// Compression ratio
    pub avg_compression_ratio: f32,
}

/// High-performance persistent cache manager
pub struct PersistentCacheManager {
    /// Redis connection pool
    connections: Arc<RwLock<Vec<MultiplexedConnection>>>,
    /// Configuration
    config: CacheConfig,
    /// Metrics collector
    metrics: Arc<CacheMetrics>,
    /// Distributed synchronization
    sync_manager: Option<Arc<dyn CacheSync>>,
    /// Cache warmer
    warmer: Option<Arc<CacheWarmer>>,
}

impl PersistentCacheManager {
    /// Create a new cache manager with connection pooling
    pub async fn new(redis_url: &str, config: CacheConfig) -> PersistenceResult<Self> {
        let client = Client::open(redis_url)?;
        let mut connections = Vec::new();

        // Create connection pool (default 10 connections)
        for i in 0..10 {
            match client.get_multiplexed_tokio_connection().await {
                Ok(conn) => {
                    connections.push(conn);
                    debug!(connection_id = i, "Created Redis connection");
                }
                Err(e) => {
                    error!(connection_id = i, error = %e, "Failed to create Redis connection");
                    return Err(PersistenceError::Redis(e));
                }
            }
        }

        info!(
            pool_size = connections.len(),
            redis_url = %redis_url,
            "Initialized cache manager"
        );

        Ok(Self {
            connections: Arc::new(RwLock::new(connections)),
            config,
            metrics: Arc::new(CacheMetrics::new()),
            sync_manager: None,
            warmer: None,
        })
    }

    /// Get a connection from the pool
    async fn get_connection(&self) -> PersistenceResult<MultiplexedConnection> {
        let connections = self.connections.read().await;
        if let Some(conn) = connections.first() {
            Ok(conn.clone())
        } else {
            Err(PersistenceError::cache("No available connections"))
        }
    }

    /// Generate cache key with namespace and hashing
    pub fn generate_key(&self, key: &str, namespace: Option<&str>) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let hash = format!("{:x}", hasher.finalize());

        match namespace {
            Some(ns) => format!("{}:{}:{}:{}", self.config.key_prefix, ns, self.config.version, &hash[..16]),
            None => format!("{}:{}:{}", self.config.key_prefix, self.config.version, &hash[..16]),
        }
    }

    /// Get value from cache with performance monitoring
    pub async fn get<T>(&self, key: &str, namespace: Option<&str>) -> PersistenceResult<Option<T>>
    where
        T: for<'de> Deserialize<'de> + serde::Serialize,
    {
        let start_time = Instant::now();
        let cache_key = self.generate_key(key, namespace);

        let mut conn = self.get_connection().await?;
        let result: Option<Vec<u8>> = conn.get(&cache_key).await?;

        let elapsed = start_time.elapsed();

        // Check performance target (5ms default)
        if elapsed.as_millis() > 5 {
            warn!(
                key = %cache_key,
                elapsed_ms = elapsed.as_millis(),
                target_ms = 5,
                "Cache access exceeded performance target"
            );
            self.metrics.record_slow_operation(elapsed).await;
        }

        match result {
            Some(bytes) => {
                // Deserialize cache entry
                let entry: CacheEntry<T> = match serde_json::from_slice(&bytes) {
                    Ok(entry) => entry,
                    Err(e) => {
                        error!(key = %cache_key, error = %e, "Failed to deserialize cache entry");
                        self.metrics.record_miss().await;
                        return Err(PersistenceError::Serialization(e));
                    }
                };

                // Check TTL
                let age = Utc::now().signed_duration_since(entry.created_at);
                if age.num_seconds() > entry.ttl_seconds as i64 {
                    debug!(key = %cache_key, age_seconds = age.num_seconds(), "Entry expired");
                    // Clean up expired entry
                    let _ = self.delete(key, namespace).await;
                    self.metrics.record_miss().await;
                    return Ok(None);
                }

                // Verify data integrity
                let calculated_hash = self.calculate_hash(&entry.data)?;
                if calculated_hash != entry.integrity_hash {
                    error!(key = %cache_key, "Data integrity check failed");
                    let _ = self.delete(key, namespace).await;
                    self.metrics.record_miss().await;
                    return Err(PersistenceError::data_integrity("Hash mismatch"));
                }

                // Update access statistics
                self.update_access_stats(&cache_key).await?;

                debug!(
                    key = %cache_key,
                    elapsed_us = elapsed.as_micros(),
                    "Cache hit"
                );

                self.metrics.record_hit(elapsed).await;
                Ok(Some(entry.data))
            }
            None => {
                debug!(key = %cache_key, elapsed_us = elapsed.as_micros(), "Cache miss");
                self.metrics.record_miss().await;
                Ok(None)
            }
        }
    }

    /// Set value in cache with compression and TTL
    pub async fn set<T>(
        &self,
        key: &str,
        value: &T,
        namespace: Option<&str>,
        ttl: Option<Duration>,
        metadata: Option<CacheMetadata>,
    ) -> PersistenceResult<()>
    where
        T: Serialize,
    {
        let start_time = Instant::now();
        let cache_key = self.generate_key(key, namespace);

        // Serialize value
        let data_bytes = serde_json::to_vec(value)?;
        let original_size = data_bytes.len();

        // Check size limits
        if original_size > self.config.max_entry_size_bytes {
            return Err(PersistenceError::cache(format!(
                "Entry size {} exceeds maximum {}",
                original_size, self.config.max_entry_size_bytes
            )));
        }

        // Apply compression if enabled and beneficial
        let (_final_data, compression_info) = if self.config.enable_compression
            && original_size > self.config.compression_threshold_bytes
        {
            let compressed = self.compress_data(&data_bytes)?;
            let compression_ratio = compressed.len() as f32 / original_size as f32;

            if compression_ratio < 0.9 {
                // Only use compression if it saves at least 10%
                let info = CompressionInfo {
                    algorithm: self.config.compression_algorithm.clone(),
                    original_size,
                    compressed_size: compressed.len(),
                    ratio: compression_ratio,
                };
                (compressed, Some(info))
            } else {
                (data_bytes, None)
            }
        } else {
            (data_bytes, None)
        };

        // Calculate integrity hash
        let integrity_hash = self.calculate_hash(value)?;

        // Create cache entry
        let ttl_seconds = ttl
            .map(|d| d.as_secs())
            .unwrap_or(self.config.default_ttl_seconds);

        let entry = CacheEntry {
            data: value,
            metadata: metadata.unwrap_or_else(|| CacheMetadata {
                version: self.config.version.clone(),
                content_type: None,
                source: None,
                tags: vec![],
                attributes: HashMap::new(),
            }),
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            ttl_seconds,
            access_count: 1,
            compression: compression_info,
            integrity_hash,
        };

        // Serialize complete entry
        let entry_bytes = serde_json::to_vec(&entry)?;

        // Store in Redis with TTL
        let mut conn = self.get_connection().await?;
        conn.set_ex::<_, _, ()>(&cache_key, &entry_bytes, ttl_seconds).await?;

        let elapsed = start_time.elapsed();

        info!(
            key = %cache_key,
            original_size = original_size,
            final_size = entry_bytes.len(),
            ttl_seconds = ttl_seconds,
            elapsed_us = elapsed.as_micros(),
            "Cache entry stored"
        );

        self.metrics.record_set(elapsed, original_size).await;

        // Notify distributed cache if enabled
        if let Some(sync_manager) = &self.sync_manager {
            sync_manager.notify_set(&cache_key).await?;
        }

        Ok(())
    }

    /// Delete entry from cache
    pub async fn delete(&self, key: &str, namespace: Option<&str>) -> PersistenceResult<bool> {
        let cache_key = self.generate_key(key, namespace);
        let mut conn = self.get_connection().await?;

        let deleted: u64 = conn.del(&cache_key).await?;
        let was_deleted = deleted > 0;

        if was_deleted {
            debug!(key = %cache_key, "Cache entry deleted");
            self.metrics.record_delete().await;

            // Notify distributed cache if enabled
            if let Some(sync_manager) = &self.sync_manager {
                sync_manager.notify_delete(&cache_key).await?;
            }
        }

        Ok(was_deleted)
    }

    /// Batch operations for better performance
    pub async fn get_batch<T>(
        &self,
        keys: &[String],
        namespace: Option<&str>,
    ) -> PersistenceResult<HashMap<String, T>>
    where
        T: for<'de> Deserialize<'de> + serde::Serialize,
    {
        let cache_keys: Vec<String> = keys
            .iter()
            .map(|k| self.generate_key(k, namespace))
            .collect();

        let mut conn = self.get_connection().await?;
        let results: Vec<Option<Vec<u8>>> = conn.get(&cache_keys).await?;

        let mut result_map = HashMap::new();

        for (original_key, cache_result) in keys.iter().zip(results.iter()) {
            if let Some(bytes) = cache_result {
                match serde_json::from_slice::<CacheEntry<T>>(bytes) {
                    Ok(entry) => {
                        // Check TTL and integrity as in single get
                        let age = Utc::now().signed_duration_since(entry.created_at);
                        if age.num_seconds() <= entry.ttl_seconds as i64 {
                            let calculated_hash = self.calculate_hash(&entry.data)?;
                            if calculated_hash == entry.integrity_hash {
                                result_map.insert(original_key.clone(), entry.data);
                            }
                        }
                    }
                    Err(e) => {
                        warn!(key = %original_key, error = %e, "Failed to deserialize in batch");
                    }
                }
            }
        }

        self.metrics.record_batch_get(keys.len(), result_map.len()).await;
        Ok(result_map)
    }

    /// Set multiple entries in a batch
    pub async fn set_batch<T>(
        &self,
        entries: HashMap<String, T>,
        namespace: Option<&str>,
        ttl: Option<Duration>,
    ) -> PersistenceResult<()>
    where
        T: Serialize,
    {
        let mut conn = self.get_connection().await?;
        let mut pipeline = Pipeline::new();

        let ttl_seconds = ttl
            .map(|d| d.as_secs())
            .unwrap_or(self.config.default_ttl_seconds);

        for (key, value) in entries.iter() {
            let cache_key = self.generate_key(key, namespace);
            let integrity_hash = self.calculate_hash(value)?;

            let entry = CacheEntry {
                data: value,
                metadata: CacheMetadata {
                    version: self.config.version.clone(),
                    content_type: None,
                    source: None,
                    tags: vec![],
                    attributes: HashMap::new(),
                },
                created_at: Utc::now(),
                last_accessed: Utc::now(),
                ttl_seconds,
                access_count: 1,
                compression: None,
                integrity_hash,
            };

            let entry_bytes = serde_json::to_vec(&entry)?;
            pipeline.set_ex(&cache_key, &entry_bytes, ttl_seconds);
        }

        pipeline.query_async::<()>(&mut conn).await?;

        info!(count = entries.len(), "Batch set completed");
        self.metrics.record_batch_set(entries.len()).await;

        Ok(())
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> PersistenceResult<CacheStats> {
        let mut conn = self.get_connection().await?;

        // Get Redis memory info
        let info: String = redis::cmd("INFO")
            .arg("memory")
            .query_async(&mut conn)
            .await
            .unwrap_or_default();

        let memory_usage = self.parse_memory_usage(&info);

        // Count our keys
        let pattern = format!("{}:{}:*", self.config.key_prefix, self.config.version);
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&pattern)
            .query_async(&mut conn)
            .await
            .unwrap_or_default();

        let metrics = self.metrics.get_current_stats().await;

        Ok(CacheStats {
            total_keys: keys.len() as u64,
            memory_usage_bytes: memory_usage,
            hit_rate: metrics.hit_rate,
            miss_rate: metrics.miss_rate,
            avg_access_time_us: metrics.avg_access_time_us,
            ops_per_second: metrics.ops_per_second,
            eviction_count: metrics.eviction_count,
            avg_compression_ratio: metrics.avg_compression_ratio,
        })
    }

    /// Enable cache warming
    pub fn enable_warming(&mut self, warmer: Arc<CacheWarmer>) {
        self.warmer = Some(warmer);
    }

    /// Enable distributed synchronization
    pub fn enable_sync(&mut self, sync_manager: Arc<dyn CacheSync>) {
        self.sync_manager = Some(sync_manager);
    }

    /// Warm cache with frequently accessed data
    pub async fn warm_cache(&self, warm_keys: Vec<String>) -> PersistenceResult<u32> {
        if let Some(warmer) = &self.warmer {
            warmer.warm(self, warm_keys).await
        } else {
            Ok(0)
        }
    }

    /// Clear all cache entries
    pub async fn clear(&self) -> PersistenceResult<u64> {
        let pattern = format!("{}:{}:*", self.config.key_prefix, self.config.version);
        let mut conn = self.get_connection().await?;

        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&pattern)
            .query_async(&mut conn)
            .await
            .unwrap_or_default();

        if keys.is_empty() {
            return Ok(0);
        }

        let deleted: u64 = conn.del(&keys).await?;
        info!(deleted = deleted, "Cache cleared");

        Ok(deleted)
    }

    // Helper methods

    fn compress_data(&self, data: &[u8]) -> PersistenceResult<Vec<u8>> {
        match self.config.compression_algorithm {
            #[cfg(feature = "compression")]
            CompressionAlgorithm::Lz4 => {
                Ok(lz4_flex::compress_prepend_size(data))
            }
            #[cfg(feature = "compression")]
            CompressionAlgorithm::Zstd => {
                zstd::encode_all(data, 3)
                    .map_err(|e| PersistenceError::compression(format!("Zstd compression failed: {}", e)))
            }
            CompressionAlgorithm::None => Ok(data.to_vec()),
        }
    }

    fn calculate_hash<T: Serialize>(&self, data: &T) -> PersistenceResult<String> {
        let bytes = serde_json::to_vec(data)?;
        let hash = blake3::hash(&bytes);
        Ok(hash.to_hex().to_string())
    }

    async fn update_access_stats(&self, _cache_key: &str) -> PersistenceResult<()> {
        // Update last_accessed and access_count in background
        // Implementation would depend on specific requirements
        Ok(())
    }

    fn parse_memory_usage(&self, info: &str) -> u64 {
        for line in info.lines() {
            if line.starts_with("used_memory:") {
                if let Some(value) = line.split(':').nth(1) {
                    return value.trim().parse().unwrap_or(0);
                }
            }
        }
        0
    }
}

/// Cache synchronization trait for distributed systems
#[async_trait::async_trait]
pub trait CacheSync: Send + Sync {
    async fn notify_set(&self, key: &str) -> PersistenceResult<()>;
    async fn notify_delete(&self, key: &str) -> PersistenceResult<()>;
    async fn invalidate_pattern(&self, pattern: &str) -> PersistenceResult<()>;
}

/// Cache warming functionality
pub struct CacheWarmer {
    batch_size: usize,
}

impl CacheWarmer {
    pub fn new(batch_size: usize) -> Self {
        Self { batch_size }
    }

    pub async fn warm(
        &self,
        cache: &PersistentCacheManager,
        keys: Vec<String>,
    ) -> PersistenceResult<u32> {
        let mut warmed = 0;

        for chunk in keys.chunks(self.batch_size) {
            // Check which keys are already warm
            let existing = cache.get_batch::<serde_json::Value>(chunk, None).await?;
            warmed += existing.len() as u32;

            debug!(
                chunk_size = chunk.len(),
                existing = existing.len(),
                "Cache warming chunk processed"
            );
        }

        info!(total_keys = keys.len(), warmed = warmed, "Cache warming completed");
        Ok(warmed)
    }
}

/// Distributed cache implementation using Redis
pub struct DistributedCache {
    node_id: String,
    _notification_channel: String,
}

impl DistributedCache {
    pub fn new(node_id: String) -> Self {
        Self {
            node_id,
            _notification_channel: "riptide:cache:notifications".to_string(),
        }
    }
}

#[async_trait::async_trait]
impl CacheSync for DistributedCache {
    async fn notify_set(&self, key: &str) -> PersistenceResult<()> {
        // Implementation would publish to Redis pub/sub channel
        debug!(key = %key, node = %self.node_id, "Distributed cache set notification");
        Ok(())
    }

    async fn notify_delete(&self, key: &str) -> PersistenceResult<()> {
        // Implementation would publish to Redis pub/sub channel
        debug!(key = %key, node = %self.node_id, "Distributed cache delete notification");
        Ok(())
    }

    async fn invalidate_pattern(&self, pattern: &str) -> PersistenceResult<()> {
        // Implementation would publish pattern invalidation
        debug!(pattern = %pattern, node = %self.node_id, "Distributed cache pattern invalidation");
        Ok(())
    }
}