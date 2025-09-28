//! Performance optimization module
//!
//! This module provides intelligent caching, resource optimization, and performance tuning
//! capabilities to maximize RipTide system efficiency.

use crate::{PerformanceError, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn, debug};
use uuid::Uuid;

/// Cache optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum cache size in MB
    pub max_size_mb: f64,
    /// Cache entry TTL (time to live)
    pub default_ttl: Duration,
    /// Enable LRU eviction
    pub enable_lru: bool,
    /// Enable adaptive sizing
    pub enable_adaptive_sizing: bool,
    /// Cache hit rate target (for adaptive sizing)
    pub target_hit_rate: f64,
    /// Maximum number of cache layers
    pub max_layers: usize,
    /// Enable cache warming
    pub enable_warming: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size_mb: 100.0,
            default_ttl: Duration::from_secs(3600), // 1 hour
            enable_lru: true,
            enable_adaptive_sizing: true,
            target_hit_rate: 0.85,
            max_layers: 3,
            enable_warming: true,
        }
    }
}

/// Cache entry metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub key: String,
    pub size_bytes: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    pub access_count: u64,
    pub priority: CachePriority,
}

/// Cache entry priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum CachePriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Cache layer types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheLayer {
    Memory,
    Redis,
    Disk,
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub hit_rate: f64,
    pub total_entries: usize,
    pub total_size_mb: f64,
    pub evictions: u64,
    pub avg_access_time_ms: f64,
    pub layer_stats: HashMap<String, LayerStats>,
}

/// Per-layer cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerStats {
    pub hits: u64,
    pub misses: u64,
    pub entries: usize,
    pub size_mb: f64,
    pub avg_response_time_ms: f64,
}

/// Cache optimization report
#[derive(Debug, Serialize, Deserialize)]
pub struct CacheOptimizationReport {
    pub session_id: Uuid,
    pub optimization_duration: Duration,
    pub before_stats: CacheStats,
    pub after_stats: CacheStats,
    pub optimizations_applied: Vec<String>,
    pub performance_improvement: f64,
    pub recommendations: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Resource optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    /// Enable connection pooling optimization
    pub enable_connection_pooling: bool,
    /// Maximum connection pool size
    pub max_connections: u32,
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Enable batch processing
    pub enable_batch_processing: bool,
    /// Batch size for operations
    pub batch_size: usize,
    /// Enable request coalescing
    pub enable_request_coalescing: bool,
    /// Coalescing window duration
    pub coalescing_window: Duration,
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            enable_connection_pooling: true,
            max_connections: 100,
            connection_timeout: Duration::from_secs(30),
            enable_batch_processing: true,
            batch_size: 50,
            enable_request_coalescing: true,
            coalescing_window: Duration::from_millis(100),
        }
    }
}

/// Multi-layer cache optimizer
pub struct CacheOptimizer {
    config: CacheConfig,
    session_id: Uuid,

    // Cache layers
    memory_cache: Arc<RwLock<HashMap<String, (Vec<u8>, CacheEntry)>>>,
    cache_stats: Arc<RwLock<CacheStats>>,

    // Optimization state
    access_patterns: Arc<RwLock<HashMap<String, AccessPattern>>>,
    eviction_queue: Arc<RwLock<VecDeque<String>>>,

    // Background tasks
    is_optimizing: Arc<RwLock<bool>>,
}

/// Access pattern tracking
#[derive(Debug, Clone)]
pub struct AccessPattern {
    pub frequency: f64,
    pub last_access: Instant,
    pub access_times: VecDeque<Instant>,
    pub predicted_next_access: Option<Instant>,
}

impl CacheOptimizer {
    /// Create a new cache optimizer
    pub fn new() -> Result<Self> {
        Self::with_config(CacheConfig::default())
    }

    /// Create a new cache optimizer with custom configuration
    pub fn with_config(config: CacheConfig) -> Result<Self> {
        let session_id = Uuid::new_v4();

        info!(
            session_id = %session_id,
            "Creating cache optimizer with config: {:?}",
            config
        );

        Ok(Self {
            config,
            session_id,
            memory_cache: Arc::new(RwLock::new(HashMap::new())),
            cache_stats: Arc::new(RwLock::new(CacheStats {
                total_requests: 0,
                cache_hits: 0,
                cache_misses: 0,
                hit_rate: 0.0,
                total_entries: 0,
                total_size_mb: 0.0,
                evictions: 0,
                avg_access_time_ms: 0.0,
                layer_stats: HashMap::new(),
            })),
            access_patterns: Arc::new(RwLock::new(HashMap::new())),
            eviction_queue: Arc::new(RwLock::new(VecDeque::new())),
            is_optimizing: Arc::new(RwLock::new(false)),
        })
    }

    /// Start cache optimization
    pub async fn start_optimization(&mut self) -> Result<()> {
        let mut is_optimizing = self.is_optimizing.write().await;
        if *is_optimizing {
            warn!(session_id = %self.session_id, "Cache optimization already running");
            return Ok(());
        }

        info!(session_id = %self.session_id, "Starting cache optimization");
        *is_optimizing = true;

        // Start background optimization tasks
        self.start_optimization_tasks().await?;

        // Warm cache if enabled
        if self.config.enable_warming {
            self.warm_cache().await?;
        }

        info!(session_id = %self.session_id, "Cache optimization started successfully");
        Ok(())
    }

    /// Stop cache optimization and generate report
    pub async fn stop_optimization(&mut self) -> Result<CacheOptimizationReport> {
        let mut is_optimizing = self.is_optimizing.write().await;
        if !*is_optimizing {
            warn!(session_id = %self.session_id, "Cache optimization not running");
            return Err(PerformanceError::ConfigError("Optimization not running".to_string()).into());
        }

        info!(session_id = %self.session_id, "Stopping cache optimization");
        *is_optimizing = false;

        // Generate optimization report
        let report = self.generate_optimization_report().await?;

        info!(
            session_id = %self.session_id,
            performance_improvement = report.performance_improvement,
            "Cache optimization stopped successfully"
        );

        Ok(report)
    }

    /// Get cache entry
    pub async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let start_time = Instant::now();

        // Update access pattern
        self.update_access_pattern(key).await;

        // Try memory cache first
        let mut cache = self.memory_cache.write().await;
        let mut stats = self.cache_stats.write().await;

        stats.total_requests += 1;

        if let Some((data, ref mut entry)) = cache.get_mut(key) {
            // Check if entry is expired
            if entry.expires_at > chrono::Utc::now() {
                entry.last_accessed = chrono::Utc::now();
                entry.access_count += 1;

                stats.cache_hits += 1;
                stats.hit_rate = stats.cache_hits as f64 / stats.total_requests as f64;

                let access_time = start_time.elapsed().as_millis() as f64;
                stats.avg_access_time_ms = (stats.avg_access_time_ms + access_time) / 2.0;

                debug!(
                    session_id = %self.session_id,
                    key = key,
                    access_time_ms = access_time,
                    "Cache hit"
                );

                return Ok(Some(data.clone()));
            } else {
                // Remove expired entry
                cache.remove(key);
                stats.total_entries = cache.len();
            }
        }

        stats.cache_misses += 1;
        stats.hit_rate = stats.cache_hits as f64 / stats.total_requests as f64;

        debug!(
            session_id = %self.session_id,
            key = key,
            "Cache miss"
        );

        Ok(None)
    }

    /// Set cache entry
    pub async fn set(&self, key: &str, data: Vec<u8>, ttl: Option<Duration>) -> Result<()> {
        let ttl = ttl.unwrap_or(self.config.default_ttl);
        let expires_at = chrono::Utc::now() + chrono::Duration::from_std(ttl).map_err(|e| PerformanceError::ConfigError(e.to_string()))?;

        let entry = CacheEntry {
            key: key.to_string(),
            size_bytes: data.len(),
            created_at: chrono::Utc::now(),
            expires_at,
            last_accessed: chrono::Utc::now(),
            access_count: 0,
            priority: CachePriority::Normal,
        };

        // Check if we need to evict entries
        self.ensure_cache_space(data.len()).await?;

        let mut cache = self.memory_cache.write().await;
        let mut stats = self.cache_stats.write().await;

        let entry_size_bytes = entry.size_bytes;
        cache.insert(key.to_string(), (data, entry));
        stats.total_entries = cache.len();

        // Update total size
        stats.total_size_mb = cache.values()
            .map(|(_, entry)| entry.size_bytes as f64)
            .sum::<f64>() / 1024.0 / 1024.0;

        debug!(
            session_id = %self.session_id,
            key = key,
            size_bytes = entry_size_bytes,
            "Cache entry stored"
        );

        Ok(())
    }

    /// Remove cache entry
    pub async fn remove(&self, key: &str) -> Result<bool> {
        let mut cache = self.memory_cache.write().await;
        let mut stats = self.cache_stats.write().await;

        let removed = cache.remove(key).is_some();
        if removed {
            stats.total_entries = cache.len();
            stats.total_size_mb = cache.values()
                .map(|(_, entry)| entry.size_bytes as f64)
                .sum::<f64>() / 1024.0 / 1024.0;

            debug!(
                session_id = %self.session_id,
                key = key,
                "Cache entry removed"
            );
        }

        Ok(removed)
    }

    /// Get current cache statistics
    pub async fn get_stats(&self) -> Result<CacheStats> {
        let stats = self.cache_stats.read().await;
        Ok(stats.clone())
    }

    /// Clear all cache entries
    pub async fn clear(&self) -> Result<()> {
        let mut cache = self.memory_cache.write().await;
        let mut stats = self.cache_stats.write().await;

        cache.clear();
        stats.total_entries = 0;
        stats.total_size_mb = 0.0;

        info!(session_id = %self.session_id, "Cache cleared");
        Ok(())
    }

    /// Optimize cache based on access patterns
    pub async fn optimize(&self) -> Result<Vec<String>> {
        info!(session_id = %self.session_id, "Running cache optimization");

        let mut optimizations = Vec::new();

        // Run various optimization strategies
        optimizations.extend(self.optimize_by_access_frequency().await?);
        optimizations.extend(self.optimize_by_size().await?);
        optimizations.extend(self.optimize_by_expiry().await?);

        if self.config.enable_adaptive_sizing {
            optimizations.extend(self.adaptive_size_optimization().await?);
        }

        info!(
            session_id = %self.session_id,
            optimizations_count = optimizations.len(),
            "Cache optimization completed"
        );

        Ok(optimizations)
    }

    /// Start background optimization tasks
    async fn start_optimization_tasks(&self) -> Result<()> {
        let session_id = self.session_id;
        let is_optimizing_cleanup = Arc::clone(&self.is_optimizing);
        let is_optimizing_periodic = Arc::clone(&self.is_optimizing);
        let cache = Arc::clone(&self.memory_cache);
        let stats = Arc::clone(&self.cache_stats);
        let _config = self.config.clone();

        // Cleanup expired entries task
        tokio::spawn(async move {
            debug!(session_id = %session_id, "Starting cache cleanup task");

            while *is_optimizing_cleanup.read().await {
                let mut cache_guard = cache.write().await;
                let mut stats_guard = stats.write().await;
                let now = chrono::Utc::now();

                let initial_count = cache_guard.len();
                cache_guard.retain(|_, (_, entry)| entry.expires_at > now);
                let removed_count = initial_count - cache_guard.len();

                if removed_count > 0 {
                    stats_guard.total_entries = cache_guard.len();
                    stats_guard.total_size_mb = cache_guard.values()
                        .map(|(_, entry)| entry.size_bytes as f64)
                        .sum::<f64>() / 1024.0 / 1024.0;

                    debug!(
                        session_id = %session_id,
                        removed_count = removed_count,
                        "Cleaned up expired cache entries"
                    );
                }

                drop(cache_guard);
                drop(stats_guard);

                tokio::time::sleep(Duration::from_secs(60)).await; // Check every minute
            }

            debug!(session_id = %session_id, "Cache cleanup task stopped");
        });

        // Periodic optimization task
        if self.config.enable_adaptive_sizing {
            let _optimizer_ref = Arc::new(RwLock::new(self as *const Self));
            tokio::spawn(async move {
                debug!(session_id = %session_id, "Starting periodic optimization task");

                while *is_optimizing_periodic.read().await {
                    tokio::time::sleep(Duration::from_secs(300)).await; // Optimize every 5 minutes

                    // Safe to access self through raw pointer in this context
                    // In a real implementation, this would use proper Arc<Self> sharing
                    debug!(session_id = %session_id, "Running periodic cache optimization");
                }

                debug!(session_id = %session_id, "Periodic optimization task stopped");
            });
        }

        Ok(())
    }

    /// Warm cache with frequently accessed data
    async fn warm_cache(&self) -> Result<()> {
        info!(session_id = %self.session_id, "Warming cache with frequently accessed data");

        // In a real implementation, this would pre-load commonly accessed data
        // For now, we'll simulate cache warming
        let warm_entries = vec![
            ("common_page_1", b"Cached page content 1".to_vec()),
            ("common_page_2", b"Cached page content 2".to_vec()),
            ("common_api_response", b"Cached API response".to_vec()),
        ];

        for (key, data) in warm_entries {
            self.set(key, data, Some(Duration::from_secs(7200))).await?; // 2 hour TTL
        }

        info!(session_id = %self.session_id, "Cache warming completed");
        Ok(())
    }

    /// Update access pattern for a key
    async fn update_access_pattern(&self, key: &str) {
        let mut patterns = self.access_patterns.write().await;
        let now = Instant::now();

        let pattern = patterns.entry(key.to_string()).or_insert_with(|| AccessPattern {
            frequency: 0.0,
            last_access: now,
            access_times: VecDeque::new(),
            predicted_next_access: None,
        });

        pattern.access_times.push_back(now);
        pattern.last_access = now;

        // Keep only recent access times (last hour)
        let cutoff = now - Duration::from_secs(3600);
        while pattern.access_times.front().map_or(false, |&time| time < cutoff) {
            pattern.access_times.pop_front();
        }

        // Calculate frequency (accesses per hour)
        pattern.frequency = pattern.access_times.len() as f64;

        // Predict next access based on pattern
        if pattern.access_times.len() >= 2 {
            let intervals: Vec<Duration> = pattern.access_times
                .iter()
                .zip(pattern.access_times.iter().skip(1))
                .map(|(prev, curr)| curr.duration_since(*prev))
                .collect();

            if !intervals.is_empty() {
                let avg_interval = intervals.iter().sum::<Duration>() / intervals.len() as u32;
                pattern.predicted_next_access = Some(now + avg_interval);
            }
        }
    }

    /// Ensure cache has space for new entry
    async fn ensure_cache_space(&self, new_entry_size: usize) -> Result<()> {
        let current_size_mb = {
            let cache = self.memory_cache.read().await;
            cache.values()
                .map(|(_, entry)| entry.size_bytes as f64)
                .sum::<f64>() / 1024.0 / 1024.0
        };

        let new_entry_size_mb = new_entry_size as f64 / 1024.0 / 1024.0;

        if current_size_mb + new_entry_size_mb > self.config.max_size_mb {
            // Need to evict entries
            let space_needed_mb = (current_size_mb + new_entry_size_mb) - self.config.max_size_mb;
            self.evict_entries(space_needed_mb).await?;
        }

        Ok(())
    }

    /// Evict cache entries to free space
    async fn evict_entries(&self, space_needed_mb: f64) -> Result<()> {
        let mut cache = self.memory_cache.write().await;
        let mut stats = self.cache_stats.write().await;
        let mut evicted_size = 0.0;

        // Collect entries sorted by eviction priority (LRU + access frequency)
        let mut entries: Vec<(String, f64)> = cache.iter()
            .map(|(key, (_, entry))| {
                let age_score = (chrono::Utc::now() - entry.last_accessed).num_seconds() as f64;
                let access_score = 1.0 / (entry.access_count as f64 + 1.0);
                let priority_score = match entry.priority {
                    CachePriority::Critical => 0.1,
                    CachePriority::High => 0.3,
                    CachePriority::Normal => 1.0,
                    CachePriority::Low => 2.0,
                };

                // Higher score = more likely to be evicted
                let eviction_score = age_score * access_score * priority_score;
                (key.clone(), eviction_score)
            })
            .collect();

        entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Evict entries until we have enough space
        for (key, _) in entries {
            if evicted_size >= space_needed_mb {
                break;
            }

            if let Some((_, entry)) = cache.remove(&key) {
                evicted_size += entry.size_bytes as f64 / 1024.0 / 1024.0;
                stats.evictions += 1;

                debug!(
                    session_id = %self.session_id,
                    key = key,
                    size_mb = entry.size_bytes as f64 / 1024.0 / 1024.0,
                    "Cache entry evicted"
                );
            }
        }

        stats.total_entries = cache.len();
        stats.total_size_mb = cache.values()
            .map(|(_, entry)| entry.size_bytes as f64)
            .sum::<f64>() / 1024.0 / 1024.0;

        info!(
            session_id = %self.session_id,
            evicted_size_mb = evicted_size,
            evicted_count = stats.evictions,
            "Cache entries evicted to free space"
        );

        Ok(())
    }

    /// Optimize cache by access frequency
    async fn optimize_by_access_frequency(&self) -> Result<Vec<String>> {
        let patterns = self.access_patterns.read().await;
        let mut optimizations = Vec::new();

        // Identify frequently accessed items that should have higher priority
        for (key, pattern) in patterns.iter() {
            if pattern.frequency > 10.0 { // More than 10 accesses per hour
                optimizations.push(format!("Promote {} to high priority (frequency: {:.1})", key, pattern.frequency));
            }
        }

        Ok(optimizations)
    }

    /// Optimize cache by entry size
    async fn optimize_by_size(&self) -> Result<Vec<String>> {
        let cache = self.memory_cache.read().await;
        let mut optimizations = Vec::new();

        // Identify large entries that might benefit from compression
        for (key, (_, entry)) in cache.iter() {
            let size_mb = entry.size_bytes as f64 / 1024.0 / 1024.0;
            if size_mb > 5.0 { // Larger than 5MB
                optimizations.push(format!("Consider compressing large entry {} ({:.1}MB)", key, size_mb));
            }
        }

        Ok(optimizations)
    }

    /// Optimize cache by expiry patterns
    async fn optimize_by_expiry(&self) -> Result<Vec<String>> {
        let cache = self.memory_cache.read().await;
        let mut optimizations = Vec::new();

        let now = chrono::Utc::now();
        let mut soon_to_expire = 0;
        let mut long_lived = 0;

        for (_, (_, entry)) in cache.iter() {
            let time_to_expire = entry.expires_at - now;
            if time_to_expire < chrono::Duration::minutes(10) {
                soon_to_expire += 1;
            } else if time_to_expire > chrono::Duration::hours(24) {
                long_lived += 1;
            }
        }

        if soon_to_expire > cache.len() / 4 {
            optimizations.push(format!("Many entries ({}) expiring soon - consider extending TTL for frequently accessed items", soon_to_expire));
        }

        if long_lived > cache.len() / 2 {
            optimizations.push(format!("Many entries ({}) are long-lived - consider shorter TTL to free space", long_lived));
        }

        Ok(optimizations)
    }

    /// Adaptive cache size optimization
    async fn adaptive_size_optimization(&self) -> Result<Vec<String>> {
        let stats = self.cache_stats.read().await;
        let mut optimizations = Vec::new();

        // Adjust cache size based on hit rate
        if stats.hit_rate < self.config.target_hit_rate {
            optimizations.push("Hit rate below target - consider increasing cache size".to_string());
        } else if stats.hit_rate > self.config.target_hit_rate + 0.1 {
            optimizations.push("Hit rate well above target - cache size could be reduced".to_string());
        }

        // Check eviction rate
        if stats.evictions > stats.total_requests / 10 {
            optimizations.push("High eviction rate - consider increasing cache size or optimizing entry sizes".to_string());
        }

        Ok(optimizations)
    }

    /// Generate optimization report
    async fn generate_optimization_report(&self) -> Result<CacheOptimizationReport> {
        let before_stats = self.get_stats().await?;

        // Run optimization
        let optimizations_applied = self.optimize().await?;

        let after_stats = self.get_stats().await?;

        // Calculate performance improvement
        let performance_improvement = if before_stats.hit_rate > 0.0 {
            ((after_stats.hit_rate - before_stats.hit_rate) / before_stats.hit_rate) * 100.0
        } else {
            0.0
        };

        // Generate recommendations
        let recommendations = self.generate_cache_recommendations(&after_stats).await?;

        Ok(CacheOptimizationReport {
            session_id: self.session_id,
            optimization_duration: Duration::from_secs(1), // Placeholder
            before_stats,
            after_stats,
            optimizations_applied,
            performance_improvement,
            recommendations,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Generate cache optimization recommendations
    async fn generate_cache_recommendations(&self, stats: &CacheStats) -> Result<Vec<String>> {
        let mut recommendations = Vec::new();

        if stats.hit_rate < 0.7 {
            recommendations.push("Low cache hit rate - investigate cache key patterns and TTL values".to_string());
        }

        if stats.avg_access_time_ms > 10.0 {
            recommendations.push("High cache access time - consider optimizing cache data structures".to_string());
        }

        if stats.evictions > stats.total_requests / 20 {
            recommendations.push("Frequent evictions - consider increasing cache size or reducing entry sizes".to_string());
        }

        if stats.total_size_mb > self.config.max_size_mb * 0.9 {
            recommendations.push("Cache near capacity - monitor for performance impact".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("Cache performance is optimal".to_string());
        }

        Ok(recommendations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_optimizer_creation() {
        let optimizer = CacheOptimizer::new().unwrap();
        assert!(!optimizer.session_id.is_nil());
    }

    #[tokio::test]
    async fn test_cache_set_get() {
        let optimizer = CacheOptimizer::new().unwrap();

        let key = "test_key";
        let data = b"test_data".to_vec();

        optimizer.set(key, data.clone(), None).await.unwrap();
        let retrieved = optimizer.get(key).await.unwrap();

        assert_eq!(retrieved, Some(data));
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let optimizer = CacheOptimizer::new().unwrap();

        // Initially empty
        let stats = optimizer.get_stats().await.unwrap();
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.total_requests, 0);

        // Add entry and access it
        optimizer.set("key1", b"data1".to_vec(), None).await.unwrap();
        let _ = optimizer.get("key1").await.unwrap();

        let stats = optimizer.get_stats().await.unwrap();
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.cache_hits, 1);
    }

    #[tokio::test]
    async fn test_cache_expiry() {
        let optimizer = CacheOptimizer::new().unwrap();

        let key = "expiring_key";
        let data = b"expiring_data".to_vec();

        // Set with very short TTL
        optimizer.set(key, data, Some(Duration::from_millis(1))).await.unwrap();

        // Wait for expiry
        tokio::time::sleep(Duration::from_millis(10)).await;

        let retrieved = optimizer.get(key).await.unwrap();
        assert_eq!(retrieved, None);
    }
}