//! Cache Warming Strategies for Instance Pool Optimization
//!
//! This module implements intelligent cache warming strategies to reduce cold start
//! latency and improve performance. It includes:
//!
//! - Pre-warming of browser instances on startup
//! - Intelligent pre-fetching based on URL patterns and usage
//! - Cache hit rate monitoring via the event system
//! - Background warming tasks with configurable intervals
//! - Integration with existing instance pool health monitoring

use anyhow::{anyhow, Result};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::{interval, sleep};
use tracing::{debug, info, warn, error};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::component::ExtractorConfig;
use crate::instance_pool::{AdvancedInstancePool, PooledInstance};
use crate::events::{Event, EventSeverity, BaseEvent, EventBus};
use crate::types::ExtractionMode;

/// Configuration for cache warming strategies
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CacheWarmingConfig {
    /// Enable cache warming functionality
    pub enabled: bool,
    /// Target warm pool size (number of instances to keep ready)
    pub warm_pool_size: usize,
    /// Minimum warm pool size to maintain
    pub min_warm_instances: usize,
    /// Maximum warm pool size limit
    pub max_warm_instances: usize,
    /// Interval for background warming tasks (seconds)
    pub warming_interval_secs: u64,
    /// Cache hit target ratio (0.0 - 1.0)
    pub cache_hit_target: f64,
    /// Enable intelligent pre-fetching
    pub enable_prefetching: bool,
    /// Pre-fetch common URL patterns
    pub prefetch_patterns: Vec<String>,
    /// Pre-fetch common resources on startup
    pub prefetch_resources: Vec<PreFetchResource>,
    /// Maximum age for warm instances (seconds)
    pub max_warm_age_secs: u64,
    /// Enable adaptive warming based on load
    pub adaptive_warming: bool,
    /// Load threshold for triggering additional warming
    pub load_threshold: f64,
}

impl Default for CacheWarmingConfig {
    fn default() -> Self {
        Self {
            enabled: std::env::var("RIPTIDE_CACHE_WARMING_ENABLED")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(true),
            warm_pool_size: std::env::var("RIPTIDE_WARM_POOL_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(4),
            min_warm_instances: std::env::var("RIPTIDE_MIN_WARM_INSTANCES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2),
            max_warm_instances: std::env::var("RIPTIDE_MAX_WARM_INSTANCES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8),
            warming_interval_secs: std::env::var("RIPTIDE_WARMING_INTERVAL_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
            cache_hit_target: std::env::var("RIPTIDE_CACHE_HIT_TARGET")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.85),
            enable_prefetching: std::env::var("RIPTIDE_ENABLE_PREFETCHING")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(true),
            prefetch_patterns: vec![
                "https://example.com/*".to_string(),
                "https://news.ycombinator.com/*".to_string(),
                "https://github.com/*".to_string(),
            ],
            prefetch_resources: vec![
                PreFetchResource {
                    url: "https://httpbin.org/html".to_string(),
                    mode: ExtractionMode::Article,
                    priority: PreFetchPriority::High,
                },
            ],
            max_warm_age_secs: 1800, // 30 minutes
            adaptive_warming: true,
            load_threshold: 0.8,
        }
    }
}

/// Resource to pre-fetch during warming
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreFetchResource {
    pub url: String,
    pub mode: ExtractionMode,
    pub priority: PreFetchPriority,
}

/// Priority levels for pre-fetching
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum PreFetchPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Cache warming statistics
#[derive(Clone, Debug, Default)]
pub struct CacheWarmingStats {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub warm_instances_created: u64,
    pub warm_instances_used: u64,
    pub prefetch_attempts: u64,
    pub prefetch_successes: u64,
    pub prefetch_failures: u64,
    pub avg_warm_time_ms: f64,
    pub last_warming_at: Option<Instant>,
    pub warming_cycles: u64,
}

impl CacheWarmingStats {
    /// Calculate cache hit ratio
    pub fn cache_hit_ratio(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }

    /// Calculate prefetch success ratio
    pub fn prefetch_success_ratio(&self) -> f64 {
        if self.prefetch_attempts == 0 {
            0.0
        } else {
            self.prefetch_successes as f64 / self.prefetch_attempts as f64
        }
    }
}

/// Cache warming events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheWarmingEvent {
    base: BaseEvent,
    pub operation: CacheWarmingOperation,
    pub pool_id: String,
    pub target_warm_size: Option<usize>,
    pub current_warm_size: Option<usize>,
    pub cache_hit_ratio: Option<f64>,
    pub warming_duration_ms: Option<u64>,
}

impl CacheWarmingEvent {
    pub fn new(
        operation: CacheWarmingOperation,
        pool_id: String,
        source: &str,
    ) -> Self {
        let severity = match operation {
            CacheWarmingOperation::WarmingFailed => EventSeverity::Error,
            CacheWarmingOperation::CacheHitRateLow => EventSeverity::Warn,
            _ => EventSeverity::Info,
        };

        let event_type = format!("cache_warming.{}", operation.as_str());
        let base = BaseEvent::new(&event_type, source, severity);

        Self {
            base,
            operation,
            pool_id,
            target_warm_size: None,
            current_warm_size: None,
            cache_hit_ratio: None,
            warming_duration_ms: None,
        }
    }

    pub fn with_warm_sizes(mut self, target: usize, current: usize) -> Self {
        self.target_warm_size = Some(target);
        self.current_warm_size = Some(current);
        self
    }

    pub fn with_cache_hit_ratio(mut self, ratio: f64) -> Self {
        self.cache_hit_ratio = Some(ratio);
        self
    }

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.warming_duration_ms = Some(duration.as_millis() as u64);
        self
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.base.add_metadata(key, value);
    }
}

impl Event for CacheWarmingEvent {
    fn event_type(&self) -> &'static str {
        "cache_warming.operation"
    }

    fn event_id(&self) -> &str {
        &self.base.event_id
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.base.timestamp
    }

    fn source(&self) -> &str {
        &self.base.source
    }

    fn severity(&self) -> EventSeverity {
        self.base.severity
    }

    fn metadata(&self) -> &HashMap<String, String> {
        &self.base.metadata
    }

    fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(Into::into)
    }
}

/// Cache warming operations
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CacheWarmingOperation {
    WarmingStarted,
    WarmingCompleted,
    WarmingFailed,
    InstancePreWarmed,
    CacheHit,
    CacheMiss,
    CacheHitRateLow,
    PreFetchStarted,
    PreFetchCompleted,
    PreFetchFailed,
    AdaptiveWarmingTriggered,
}

impl CacheWarmingOperation {
    pub fn as_str(&self) -> &'static str {
        match self {
            CacheWarmingOperation::WarmingStarted => "warming_started",
            CacheWarmingOperation::WarmingCompleted => "warming_completed",
            CacheWarmingOperation::WarmingFailed => "warming_failed",
            CacheWarmingOperation::InstancePreWarmed => "instance_pre_warmed",
            CacheWarmingOperation::CacheHit => "cache_hit",
            CacheWarmingOperation::CacheMiss => "cache_miss",
            CacheWarmingOperation::CacheHitRateLow => "cache_hit_rate_low",
            CacheWarmingOperation::PreFetchStarted => "prefetch_started",
            CacheWarmingOperation::PreFetchCompleted => "prefetch_completed",
            CacheWarmingOperation::PreFetchFailed => "prefetch_failed",
            CacheWarmingOperation::AdaptiveWarmingTriggered => "adaptive_warming_triggered",
        }
    }
}

/// Intelligent cache warming manager
pub struct CacheWarmingManager {
    config: CacheWarmingConfig,
    pool: Arc<AdvancedInstancePool>,
    stats: Arc<Mutex<CacheWarmingStats>>,
    event_bus: Option<Arc<EventBus>>,
    warm_instances: Arc<RwLock<VecDeque<WarmInstance>>>,
    usage_patterns: Arc<Mutex<HashMap<String, UrlPattern>>>,
    running: Arc<tokio::sync::RwLock<bool>>,
}

/// Warmed instance with metadata
#[derive(Debug)]
struct WarmInstance {
    instance: PooledInstance,
    warmed_at: Instant,
    pre_fetch_url: Option<String>,
    use_count: u64,
}

/// URL pattern tracking for intelligent pre-fetching
#[derive(Debug, Clone)]
struct UrlPattern {
    pattern: String,
    access_count: u64,
    last_accessed: Instant,
    avg_processing_time_ms: f64,
    cache_hit_ratio: f64,
}

impl CacheWarmingManager {
    /// Create new cache warming manager
    pub fn new(
        config: CacheWarmingConfig,
        pool: Arc<AdvancedInstancePool>,
        event_bus: Option<Arc<EventBus>>,
    ) -> Self {
        Self {
            config,
            pool,
            stats: Arc::new(Mutex::new(CacheWarmingStats::default())),
            event_bus,
            warm_instances: Arc::new(RwLock::new(VecDeque::new())),
            usage_patterns: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(tokio::sync::RwLock::new(false)),
        }
    }

    /// Start cache warming background tasks
    pub async fn start(&self) -> Result<()> {
        if !self.config.enabled {
            info!("Cache warming disabled by configuration");
            return Ok(());
        }

        {
            let mut running = self.running.write().await;
            if *running {
                return Err(anyhow!("Cache warming already running"));
            }
            *running = true;
        }

        info!(
            warm_pool_size = self.config.warm_pool_size,
            warming_interval = self.config.warming_interval_secs,
            "Starting cache warming manager"
        );

        // Initial warming
        self.perform_initial_warming().await?;

        // Start background warming task
        let manager = self.clone();
        tokio::spawn(async move {
            manager.background_warming_task().await;
        });

        // Start prefetching task if enabled
        if self.config.enable_prefetching {
            let manager = self.clone();
            tokio::spawn(async move {
                manager.background_prefetch_task().await;
            });
        }

        Ok(())
    }

    /// Stop cache warming
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        info!("Cache warming stopped");
    }

    /// Perform initial warming on startup
    async fn perform_initial_warming(&self) -> Result<()> {
        let start_time = Instant::now();

        self.emit_warming_event(CacheWarmingOperation::WarmingStarted).await;

        info!(target_size = self.config.warm_pool_size, "Starting initial cache warming");

        let mut warmed_count = 0;
        for i in 0..self.config.warm_pool_size {
            match self.create_warm_instance().await {
                Ok(warm_instance) => {
                    {
                        let mut warm_instances = self.warm_instances.write().await;
                        warm_instances.push_back(warm_instance);
                    }
                    warmed_count += 1;
                    debug!(instance_index = i, "Created warm instance");
                }
                Err(e) => {
                    error!(error = %e, instance_index = i, "Failed to create warm instance");
                }
            }
        }

        let duration = start_time.elapsed();

        // Update statistics
        {
            let mut stats = self.stats.lock().unwrap();
            stats.warm_instances_created += warmed_count;
            stats.avg_warm_time_ms = duration.as_millis() as f64 / warmed_count as f64;
            stats.last_warming_at = Some(start_time);
            stats.warming_cycles += 1;
        }

        // Emit completion event
        let current_size = {
            let warm_instances = self.warm_instances.read().await;
            warm_instances.len()
        };

        let mut event = CacheWarmingEvent::new(
            CacheWarmingOperation::WarmingCompleted,
            self.pool.pool_id().to_string(),
            "cache_warming",
        )
        .with_warm_sizes(self.config.warm_pool_size, current_size)
        .with_duration(duration);

        event.add_metadata("warmed_instances", &warmed_count.to_string());
        self.emit_event(event).await;

        info!(
            warmed_count = warmed_count,
            duration_ms = duration.as_millis(),
            "Initial cache warming completed"
        );

        Ok(())
    }

    /// Create a warm instance with optional pre-fetching
    async fn create_warm_instance(&self) -> Result<WarmInstance> {
        let instance = self.pool.create_instance().await?;

        let mut warm_instance = WarmInstance {
            instance,
            warmed_at: Instant::now(),
            pre_fetch_url: None,
            use_count: 0,
        };

        // Pre-fetch a resource if configured
        if self.config.enable_prefetching && !self.config.prefetch_resources.is_empty() {
            let resource = &self.config.prefetch_resources[0]; // Use first resource for now

            match self.pool.extract(&"<html><body>Warming cache</body></html>",
                                   &resource.url, resource.mode.clone()).await {
                Ok(_) => {
                    warm_instance.pre_fetch_url = Some(resource.url.clone());
                    debug!(url = %resource.url, "Pre-fetched resource for warm instance");

                    // Update statistics
                    {
                        let mut stats = self.stats.lock().unwrap();
                        stats.prefetch_attempts += 1;
                        stats.prefetch_successes += 1;
                    }
                }
                Err(e) => {
                    warn!(error = %e, url = %resource.url, "Failed to pre-fetch resource");

                    // Update statistics
                    {
                        let mut stats = self.stats.lock().unwrap();
                        stats.prefetch_attempts += 1;
                        stats.prefetch_failures += 1;
                    }
                }
            }
        }

        // Emit instance pre-warmed event
        let event = CacheWarmingEvent::new(
            CacheWarmingOperation::InstancePreWarmed,
            self.pool.pool_id().to_string(),
            "cache_warming",
        );
        self.emit_event(event).await;

        Ok(warm_instance)
    }

    /// Get a warm instance for immediate use
    pub async fn get_warm_instance(&self) -> Option<PooledInstance> {
        let mut warm_instances = self.warm_instances.write().await;

        if let Some(mut warm_instance) = warm_instances.pop_front() {
            warm_instance.use_count += 1;

            // Update statistics
            {
                let mut stats = self.stats.lock().unwrap();
                stats.warm_instances_used += 1;
                stats.cache_hits += 1;
            }

            // Emit cache hit event
            let event = CacheWarmingEvent::new(
                CacheWarmingOperation::CacheHit,
                self.pool.pool_id().to_string(),
                "cache_warming",
            );
            self.emit_event(event).await;

            info!(instance_id = %warm_instance.instance.id, "Used warm instance");
            Some(warm_instance.instance)
        } else {
            // Update statistics
            {
                let mut stats = self.stats.lock().unwrap();
                stats.cache_misses += 1;
            }

            // Emit cache miss event
            let event = CacheWarmingEvent::new(
                CacheWarmingOperation::CacheMiss,
                self.pool.pool_id().to_string(),
                "cache_warming",
            );
            self.emit_event(event).await;

            debug!("No warm instances available");
            None
        }
    }

    /// Background warming task
    async fn background_warming_task(&self) {
        let mut interval_timer = interval(Duration::from_secs(self.config.warming_interval_secs));

        while *self.running.read().await {
            interval_timer.tick().await;

            if let Err(e) = self.maintain_warm_pool().await {
                error!(error = %e, "Failed to maintain warm pool");
            }

            if let Err(e) = self.check_cache_hit_ratio().await {
                error!(error = %e, "Failed to check cache hit ratio");
            }
        }
    }

    /// Maintain optimal warm pool size
    async fn maintain_warm_pool(&self) -> Result<()> {
        let current_size = {
            let warm_instances = self.warm_instances.read().await;
            warm_instances.len()
        };

        let target_size = if self.config.adaptive_warming {
            self.calculate_adaptive_target_size().await
        } else {
            self.config.warm_pool_size
        };

        if current_size < target_size {
            let needed = target_size - current_size;
            debug!(current = current_size, target = target_size, needed = needed,
                   "Creating additional warm instances");

            for _ in 0..needed {
                match self.create_warm_instance().await {
                    Ok(warm_instance) => {
                        let mut warm_instances = self.warm_instances.write().await;
                        warm_instances.push_back(warm_instance);
                    }
                    Err(e) => {
                        error!(error = %e, "Failed to create warm instance during maintenance");
                    }
                }
            }
        }

        // Clean up old warm instances
        self.cleanup_old_warm_instances().await;

        Ok(())
    }

    /// Calculate adaptive target size based on load
    async fn calculate_adaptive_target_size(&self) -> usize {
        let (available, active, total) = self.pool.get_pool_status();
        let load_ratio = if total == 0 { 0.0 } else { active as f64 / total as f64 };

        if load_ratio > self.config.load_threshold {
            // High load, increase warm pool size
            let increased_size = (self.config.warm_pool_size as f64 * 1.5) as usize;

            // Emit adaptive warming event
            let mut event = CacheWarmingEvent::new(
                CacheWarmingOperation::AdaptiveWarmingTriggered,
                self.pool.pool_id().to_string(),
                "cache_warming",
            );
            event.add_metadata("load_ratio", &format!("{:.2}", load_ratio));
            event.add_metadata("new_target_size", &increased_size.to_string());
            self.emit_event(event).await;

            increased_size.min(self.config.max_warm_instances)
        } else {
            self.config.warm_pool_size.max(self.config.min_warm_instances)
        }
    }

    /// Clean up old warm instances
    async fn cleanup_old_warm_instances(&self) {
        let max_age = Duration::from_secs(self.config.max_warm_age_secs);
        let mut cleaned = 0;

        {
            let mut warm_instances = self.warm_instances.write().await;
            let mut i = 0;
            while i < warm_instances.len() {
                if warm_instances[i].warmed_at.elapsed() > max_age {
                    let old_instance = warm_instances.remove(i).unwrap();
                    debug!(instance_id = %old_instance.instance.id, age_secs = old_instance.warmed_at.elapsed().as_secs(),
                           "Cleaned up old warm instance");
                    cleaned += 1;
                } else {
                    i += 1;
                }
            }
        }

        if cleaned > 0 {
            info!(cleaned_count = cleaned, "Cleaned up old warm instances");
        }
    }

    /// Check cache hit ratio and emit warnings if below target
    async fn check_cache_hit_ratio(&self) -> Result<()> {
        let (ratio, hits, misses) = {
            let stats = self.stats.lock().unwrap();
            (stats.cache_hit_ratio(), stats.cache_hits, stats.cache_misses)
        };

        if hits + misses > 10 && ratio < self.config.cache_hit_target {
            warn!(
                cache_hit_ratio = ratio,
                target = self.config.cache_hit_target,
                hits = hits,
                misses = misses,
                "Cache hit ratio below target"
            );

            let mut event = CacheWarmingEvent::new(
                CacheWarmingOperation::CacheHitRateLow,
                self.pool.pool_id().to_string(),
                "cache_warming",
            )
            .with_cache_hit_ratio(ratio);

            event.add_metadata("target_ratio", &self.config.cache_hit_target.to_string());
            event.add_metadata("cache_hits", &hits.to_string());
            event.add_metadata("cache_misses", &misses.to_string());
            self.emit_event(event).await;
        }

        Ok(())
    }

    /// Background prefetch task
    async fn background_prefetch_task(&self) {
        let mut interval_timer = interval(Duration::from_secs(300)); // Every 5 minutes

        while *self.running.read().await {
            interval_timer.tick().await;

            if let Err(e) = self.perform_intelligent_prefetch().await {
                error!(error = %e, "Failed to perform intelligent prefetch");
            }
        }
    }

    /// Perform intelligent pre-fetching based on patterns
    async fn perform_intelligent_prefetch(&self) -> Result<()> {
        // Sort prefetch resources by priority
        let mut resources = self.config.prefetch_resources.clone();
        resources.sort_by(|a, b| b.priority.cmp(&a.priority));

        for resource in resources.iter().take(3) { // Limit to top 3 resources
            self.emit_warming_event(CacheWarmingOperation::PreFetchStarted).await;

            match self.pool.extract(&"<html><body>Pre-fetch</body></html>",
                                   &resource.url, resource.mode.clone()).await {
                Ok(_) => {
                    debug!(url = %resource.url, "Successfully pre-fetched resource");
                    self.emit_warming_event(CacheWarmingOperation::PreFetchCompleted).await;
                }
                Err(e) => {
                    warn!(error = %e, url = %resource.url, "Failed to pre-fetch resource");
                    self.emit_warming_event(CacheWarmingOperation::PreFetchFailed).await;
                }
            }

            // Small delay between prefetches
            sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }

    /// Record URL pattern for intelligent caching
    pub async fn record_url_pattern(&self, url: &str, processing_time_ms: f64, cache_hit: bool) {
        let pattern = self.extract_url_pattern(url);

        let mut patterns = self.usage_patterns.lock().unwrap();
        let entry = patterns.entry(pattern.clone()).or_insert_with(|| UrlPattern {
            pattern: pattern.clone(),
            access_count: 0,
            last_accessed: Instant::now(),
            avg_processing_time_ms: 0.0,
            cache_hit_ratio: 0.0,
        });

        entry.access_count += 1;
        entry.last_accessed = Instant::now();

        // Update running average
        let total_time = entry.avg_processing_time_ms * (entry.access_count - 1) as f64 + processing_time_ms;
        entry.avg_processing_time_ms = total_time / entry.access_count as f64;

        // Update cache hit ratio
        let hits = (entry.cache_hit_ratio * (entry.access_count - 1) as f64) + if cache_hit { 1.0 } else { 0.0 };
        entry.cache_hit_ratio = hits / entry.access_count as f64;
    }

    /// Extract URL pattern for grouping
    fn extract_url_pattern(&self, url: &str) -> String {
        if let Ok(parsed) = url::Url::parse(url) {
            format!("{}://{}/", parsed.scheme(), parsed.host_str().unwrap_or("unknown"))
        } else {
            "unknown".to_string()
        }
    }

    /// Get cache warming statistics
    pub fn get_stats(&self) -> CacheWarmingStats {
        self.stats.lock().unwrap().clone()
    }

    /// Get current warm pool size
    pub async fn get_warm_pool_size(&self) -> usize {
        let warm_instances = self.warm_instances.read().await;
        warm_instances.len()
    }

    /// Emit cache warming event
    async fn emit_warming_event(&self, operation: CacheWarmingOperation) {
        let event = CacheWarmingEvent::new(
            operation,
            self.pool.pool_id().to_string(),
            "cache_warming",
        );
        self.emit_event(event).await;
    }

    /// Emit event through event bus
    async fn emit_event(&self, event: CacheWarmingEvent) {
        if let Some(event_bus) = &self.event_bus {
            if let Err(e) = event_bus.emit(event).await {
                error!(error = %e, "Failed to emit cache warming event");
            }
        }
    }
}

impl Clone for CacheWarmingManager {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            pool: self.pool.clone(),
            stats: self.stats.clone(),
            event_bus: self.event_bus.clone(),
            warm_instances: self.warm_instances.clone(),
            usage_patterns: self.usage_patterns.clone(),
            running: self.running.clone(),
        }
    }
}

/// Extension trait for AdvancedInstancePool to support cache warming
pub trait CacheWarmingPoolExt {
    /// Create cache warming manager for this pool
    fn create_cache_warming_manager(
        &self,
        config: CacheWarmingConfig,
        event_bus: Option<Arc<EventBus>>,
    ) -> CacheWarmingManager;
}

impl CacheWarmingPoolExt for Arc<AdvancedInstancePool> {
    fn create_cache_warming_manager(
        &self,
        config: CacheWarmingConfig,
        event_bus: Option<Arc<EventBus>>,
    ) -> CacheWarmingManager {
        CacheWarmingManager::new(config, self.clone(), event_bus)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_warming_config_default() {
        let config = CacheWarmingConfig::default();
        assert!(config.enabled);
        assert!(config.warm_pool_size > 0);
        assert!(config.cache_hit_target > 0.0 && config.cache_hit_target <= 1.0);
    }

    #[test]
    fn test_cache_warming_stats() {
        let mut stats = CacheWarmingStats::default();
        stats.cache_hits = 85;
        stats.cache_misses = 15;

        assert_eq!(stats.cache_hit_ratio(), 0.85);

        stats.prefetch_attempts = 10;
        stats.prefetch_successes = 8;

        assert_eq!(stats.prefetch_success_ratio(), 0.8);
    }

    #[test]
    fn test_prefetch_priority_ordering() {
        assert!(PreFetchPriority::Critical > PreFetchPriority::High);
        assert!(PreFetchPriority::High > PreFetchPriority::Medium);
        assert!(PreFetchPriority::Medium > PreFetchPriority::Low);
    }
}