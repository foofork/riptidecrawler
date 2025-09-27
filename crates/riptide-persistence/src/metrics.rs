/*!
# Persistence Metrics

Comprehensive metrics collection for cache performance, tenant usage,
and system health monitoring.
*/

use chrono::{DateTime, Utc};
use prometheus::{Counter, Gauge, Histogram, HistogramOpts, Opts, Registry};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error};

/// Cache performance metrics
#[derive(Debug, Clone)]
pub struct CacheMetrics {
    /// Hit counter
    pub hits: Counter,
    /// Miss counter
    pub misses: Counter,
    /// Set operations counter
    pub sets: Counter,
    /// Delete operations counter
    pub deletes: Counter,
    /// Access time histogram
    pub access_time: Histogram,
    /// Entry size histogram
    pub entry_size: Histogram,
    /// Memory usage gauge
    pub memory_usage: Gauge,
    /// Connection pool gauge
    pub active_connections: Gauge,
    /// Error counter by type
    pub errors: Counter,
    /// Compression ratio gauge
    pub compression_ratio: Gauge,
    /// Internal statistics
    stats: Arc<RwLock<InternalCacheStats>>,
}

/// Internal cache statistics
#[derive(Debug, Clone)]
struct InternalCacheStats {
    total_hits: u64,
    total_misses: u64,
    total_sets: u64,
    total_deletes: u64,
    total_errors: u64,
    access_times: Vec<u64>, // Microseconds
    entry_sizes: Vec<usize>,
    compression_ratios: Vec<f32>,
    last_reset: DateTime<Utc>,
    slow_operations: u64,
}

impl Default for InternalCacheStats {
    fn default() -> Self {
        Self {
            total_hits: 0,
            total_misses: 0,
            total_sets: 0,
            total_deletes: 0,
            total_errors: 0,
            access_times: Vec::new(),
            entry_sizes: Vec::new(),
            compression_ratios: Vec::new(),
            last_reset: Utc::now(),
            slow_operations: 0,
        }
    }
}

impl CacheMetrics {
    /// Create new cache metrics instance
    pub fn new() -> Self {
        let hits = Counter::with_opts(
            Opts::new("riptide_cache_hits_total", "Total cache hits")
        ).expect("Failed to create hits counter");

        let misses = Counter::with_opts(
            Opts::new("riptide_cache_misses_total", "Total cache misses")
        ).expect("Failed to create misses counter");

        let sets = Counter::with_opts(
            Opts::new("riptide_cache_sets_total", "Total cache sets")
        ).expect("Failed to create sets counter");

        let deletes = Counter::with_opts(
            Opts::new("riptide_cache_deletes_total", "Total cache deletes")
        ).expect("Failed to create deletes counter");

        let access_time = Histogram::with_opts(
            HistogramOpts::new("riptide_cache_access_duration_microseconds", "Cache access time")
                .buckets(vec![100.0, 500.0, 1000.0, 2000.0, 5000.0, 10000.0, 20000.0, 50000.0])
        ).expect("Failed to create access time histogram");

        let entry_size = Histogram::with_opts(
            HistogramOpts::new("riptide_cache_entry_size_bytes", "Cache entry size")
                .buckets(vec![1024.0, 10240.0, 102400.0, 1048576.0, 10485760.0, 104857600.0])
        ).expect("Failed to create entry size histogram");

        let memory_usage = Gauge::with_opts(
            Opts::new("riptide_cache_memory_usage_bytes", "Cache memory usage")
        ).expect("Failed to create memory usage gauge");

        let active_connections = Gauge::with_opts(
            Opts::new("riptide_cache_active_connections", "Active Redis connections")
        ).expect("Failed to create connections gauge");

        let errors = Counter::with_opts(
            Opts::new("riptide_cache_errors_total", "Total cache errors")
        ).expect("Failed to create errors counter");

        let compression_ratio = Gauge::with_opts(
            Opts::new("riptide_cache_compression_ratio", "Average compression ratio")
        ).expect("Failed to create compression ratio gauge");

        Self {
            hits,
            misses,
            sets,
            deletes,
            access_time,
            entry_size,
            memory_usage,
            active_connections,
            errors,
            compression_ratio,
            stats: Arc::new(RwLock::new(InternalCacheStats::default())),
        }
    }

    /// Register metrics with Prometheus registry
    pub fn register(&self, registry: &Registry) -> Result<(), prometheus::Error> {
        registry.register(Box::new(self.hits.clone()))?;
        registry.register(Box::new(self.misses.clone()))?;
        registry.register(Box::new(self.sets.clone()))?;
        registry.register(Box::new(self.deletes.clone()))?;
        registry.register(Box::new(self.access_time.clone()))?;
        registry.register(Box::new(self.entry_size.clone()))?;
        registry.register(Box::new(self.memory_usage.clone()))?;
        registry.register(Box::new(self.active_connections.clone()))?;
        registry.register(Box::new(self.errors.clone()))?;
        registry.register(Box::new(self.compression_ratio.clone()))?;
        Ok(())
    }

    /// Record cache hit
    pub async fn record_hit(&self, access_time: Duration) {
        self.hits.inc();
        let access_us = access_time.as_micros() as f64;
        self.access_time.observe(access_us);

        let mut stats = self.stats.write().await;
        stats.total_hits += 1;
        stats.access_times.push(access_time.as_micros() as u64);

        // Keep only recent access times (last 1000)
        if stats.access_times.len() > 1000 {
            stats.access_times.drain(0..100);
        }

        debug!(access_time_us = access_time.as_micros(), "Cache hit recorded");
    }

    /// Record cache miss
    pub async fn record_miss(&self) {
        self.misses.inc();
        let mut stats = self.stats.write().await;
        stats.total_misses += 1;
        debug!("Cache miss recorded");
    }

    /// Record cache set operation
    pub async fn record_set(&self, operation_time: Duration, entry_size: usize) {
        self.sets.inc();
        self.entry_size.observe(entry_size as f64);

        let mut stats = self.stats.write().await;
        stats.total_sets += 1;
        stats.entry_sizes.push(entry_size);

        if stats.entry_sizes.len() > 1000 {
            stats.entry_sizes.drain(0..100);
        }

        debug!(
            operation_time_us = operation_time.as_micros(),
            entry_size = entry_size,
            "Cache set recorded"
        );
    }

    /// Record cache delete operation
    pub async fn record_delete(&self) {
        self.deletes.inc();
        let mut stats = self.stats.write().await;
        stats.total_deletes += 1;
        debug!("Cache delete recorded");
    }

    /// Record batch get operation
    pub async fn record_batch_get(&self, requested: usize, found: usize) {
        for _ in 0..found {
            self.hits.inc();
        }
        for _ in 0..(requested - found) {
            self.misses.inc();
        }

        let mut stats = self.stats.write().await;
        stats.total_hits += found as u64;
        stats.total_misses += (requested - found) as u64;

        debug!(requested = requested, found = found, "Batch get recorded");
    }

    /// Record batch set operation
    pub async fn record_batch_set(&self, count: usize) {
        for _ in 0..count {
            self.sets.inc();
        }

        let mut stats = self.stats.write().await;
        stats.total_sets += count as u64;

        debug!(count = count, "Batch set recorded");
    }

    /// Record slow operation
    pub async fn record_slow_operation(&self, duration: Duration) {
        let mut stats = self.stats.write().await;
        stats.slow_operations += 1;

        error!(
            duration_ms = duration.as_millis(),
            "Slow cache operation detected"
        );
    }

    /// Record error
    pub async fn record_error(&self, error_type: &str) {
        self.errors.inc();
        let mut stats = self.stats.write().await;
        stats.total_errors += 1;

        error!(error_type = error_type, "Cache error recorded");
    }

    /// Record compression ratio
    pub async fn record_compression(&self, ratio: f32) {
        self.compression_ratio.set(ratio as f64);

        let mut stats = self.stats.write().await;
        stats.compression_ratios.push(ratio);

        if stats.compression_ratios.len() > 1000 {
            stats.compression_ratios.drain(0..100);
        }
    }

    /// Update memory usage
    pub async fn update_memory_usage(&self, bytes: u64) {
        self.memory_usage.set(bytes as f64);
    }

    /// Update active connections count
    pub async fn update_connections(&self, count: u32) {
        self.active_connections.set(count as f64);
    }

    /// Get current statistics summary
    pub async fn get_current_stats(&self) -> CacheStatsSummary {
        let stats = self.stats.read().await;

        let total_operations = stats.total_hits + stats.total_misses;
        let hit_rate = if total_operations > 0 {
            stats.total_hits as f64 / total_operations as f64
        } else {
            0.0
        };

        let miss_rate = 1.0 - hit_rate;

        let avg_access_time_us = if !stats.access_times.is_empty() {
            stats.access_times.iter().sum::<u64>() / stats.access_times.len() as u64
        } else {
            0
        };

        let ops_per_second = if total_operations > 0 {
            let duration = Utc::now().signed_duration_since(stats.last_reset);
            if duration.num_seconds() > 0 {
                total_operations as f64 / duration.num_seconds() as f64
            } else {
                0.0
            }
        } else {
            0.0
        };

        let avg_compression_ratio = if !stats.compression_ratios.is_empty() {
            stats.compression_ratios.iter().sum::<f32>() / stats.compression_ratios.len() as f32
        } else {
            1.0
        };

        CacheStatsSummary {
            hit_rate,
            miss_rate,
            avg_access_time_us,
            ops_per_second,
            total_operations,
            eviction_count: 0, // TODO: Implement eviction tracking
            avg_compression_ratio,
            slow_operations: stats.slow_operations,
            error_count: stats.total_errors,
        }
    }

    /// Reset statistics
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = InternalCacheStats::default();
        debug!("Cache statistics reset");
    }
}

/// Cache statistics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStatsSummary {
    pub hit_rate: f64,
    pub miss_rate: f64,
    pub avg_access_time_us: u64,
    pub ops_per_second: f64,
    pub total_operations: u64,
    pub eviction_count: u64,
    pub avg_compression_ratio: f32,
    pub slow_operations: u64,
    pub error_count: u64,
}

/// Tenant usage metrics
#[derive(Debug, Clone)]
pub struct TenantMetrics {
    /// Operations per tenant
    pub operations: HashMap<String, Counter>,
    /// Memory usage per tenant
    pub memory_usage: HashMap<String, Gauge>,
    /// Data transfer per tenant
    pub data_transfer: HashMap<String, Counter>,
    /// Error count per tenant
    pub errors: HashMap<String, Counter>,
    /// Active sessions per tenant
    pub active_sessions: HashMap<String, Gauge>,
    /// Registry for metric registration
    registry: Arc<Registry>,
}

impl TenantMetrics {
    /// Create new tenant metrics
    pub fn new(registry: Arc<Registry>) -> Self {
        Self {
            operations: HashMap::new(),
            memory_usage: HashMap::new(),
            data_transfer: HashMap::new(),
            errors: HashMap::new(),
            active_sessions: HashMap::new(),
            registry,
        }
    }

    /// Register metrics for a new tenant
    pub fn register_tenant(&mut self, tenant_id: &str) -> Result<(), prometheus::Error> {
        let tenant_label = tenant_id.to_string();

        // Operations counter
        let ops_counter = Counter::with_opts(
            Opts::new("riptide_tenant_operations_total", "Total operations per tenant")
                .const_label("tenant_id", &tenant_label)
        )?;
        self.registry.register(Box::new(ops_counter.clone()))?;
        self.operations.insert(tenant_id.to_string(), ops_counter);

        // Memory usage gauge
        let memory_gauge = Gauge::with_opts(
            Opts::new("riptide_tenant_memory_usage_bytes", "Memory usage per tenant")
                .const_label("tenant_id", &tenant_label)
        )?;
        self.registry.register(Box::new(memory_gauge.clone()))?;
        self.memory_usage.insert(tenant_id.to_string(), memory_gauge);

        // Data transfer counter
        let transfer_counter = Counter::with_opts(
            Opts::new("riptide_tenant_data_transfer_bytes", "Data transfer per tenant")
                .const_label("tenant_id", &tenant_label)
        )?;
        self.registry.register(Box::new(transfer_counter.clone()))?;
        self.data_transfer.insert(tenant_id.to_string(), transfer_counter);

        // Error counter
        let error_counter = Counter::with_opts(
            Opts::new("riptide_tenant_errors_total", "Errors per tenant")
                .const_label("tenant_id", &tenant_label)
        )?;
        self.registry.register(Box::new(error_counter.clone()))?;
        self.errors.insert(tenant_id.to_string(), error_counter);

        // Active sessions gauge
        let sessions_gauge = Gauge::with_opts(
            Opts::new("riptide_tenant_active_sessions", "Active sessions per tenant")
                .const_label("tenant_id", &tenant_label)
        )?;
        self.registry.register(Box::new(sessions_gauge.clone()))?;
        self.active_sessions.insert(tenant_id.to_string(), sessions_gauge);

        debug!(tenant_id = tenant_id, "Tenant metrics registered");
        Ok(())
    }

    /// Record operation for tenant
    pub fn record_operation(&self, tenant_id: &str) {
        if let Some(counter) = self.operations.get(tenant_id) {
            counter.inc();
        }
    }

    /// Update memory usage for tenant
    pub fn update_memory_usage(&self, tenant_id: &str, bytes: u64) {
        if let Some(gauge) = self.memory_usage.get(tenant_id) {
            gauge.set(bytes as f64);
        }
    }

    /// Record data transfer for tenant
    pub fn record_data_transfer(&self, tenant_id: &str, bytes: u64) {
        if let Some(counter) = self.data_transfer.get(tenant_id) {
            counter.inc_by(bytes as f64);
        }
    }

    /// Record error for tenant
    pub fn record_error(&self, tenant_id: &str) {
        if let Some(counter) = self.errors.get(tenant_id) {
            counter.inc();
        }
    }

    /// Update active sessions for tenant
    pub fn update_active_sessions(&self, tenant_id: &str, count: u32) {
        if let Some(gauge) = self.active_sessions.get(tenant_id) {
            gauge.set(count as f64);
        }
    }
}

/// Overall persistence metrics
#[derive(Debug, Clone)]
pub struct PersistenceMetrics {
    /// Cache metrics
    pub cache: Arc<CacheMetrics>,
    /// Tenant metrics
    pub tenant: Arc<RwLock<TenantMetrics>>,
    /// System metrics
    pub system: SystemMetrics,
    /// Prometheus registry
    pub registry: Arc<Registry>,
}

impl PersistenceMetrics {
    /// Create new persistence metrics
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Arc::new(Registry::new());
        let cache_metrics = Arc::new(CacheMetrics::new());
        cache_metrics.register(&registry)?;

        let tenant_metrics = Arc::new(RwLock::new(TenantMetrics::new(registry.clone())));
        let system_metrics = SystemMetrics::new(&registry)?;

        Ok(Self {
            cache: cache_metrics,
            tenant: tenant_metrics,
            system: system_metrics,
            registry,
        })
    }

    /// Get metrics in Prometheus format
    pub fn gather(&self) -> Vec<prometheus::proto::MetricFamily> {
        self.registry.gather()
    }
}

/// System-level metrics
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    /// Total uptime
    pub uptime: Gauge,
    /// Configuration reloads
    pub config_reloads: Counter,
    /// Checkpoint operations
    pub checkpoints: Counter,
    /// Distributed sync operations
    pub sync_operations: Counter,
}

impl SystemMetrics {
    pub fn new(registry: &Registry) -> Result<Self, prometheus::Error> {
        let uptime = Gauge::with_opts(
            Opts::new("riptide_system_uptime_seconds", "System uptime")
        )?;
        registry.register(Box::new(uptime.clone()))?;

        let config_reloads = Counter::with_opts(
            Opts::new("riptide_config_reloads_total", "Configuration reloads")
        )?;
        registry.register(Box::new(config_reloads.clone()))?;

        let checkpoints = Counter::with_opts(
            Opts::new("riptide_checkpoints_total", "Checkpoint operations")
        )?;
        registry.register(Box::new(checkpoints.clone()))?;

        let sync_operations = Counter::with_opts(
            Opts::new("riptide_sync_operations_total", "Distributed sync operations")
        )?;
        registry.register(Box::new(sync_operations.clone()))?;

        Ok(Self {
            uptime,
            config_reloads,
            checkpoints,
            sync_operations,
        })
    }
}

/// Performance metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Cache performance
    pub cache_performance: CachePerformanceMetrics,
    /// Tenant resource usage
    pub tenant_usage: HashMap<String, TenantUsageMetrics>,
    /// System performance
    pub system_performance: SystemPerformanceMetrics,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Cache-specific performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachePerformanceMetrics {
    /// Hit rate percentage
    pub hit_rate_percent: f64,
    /// Average access time in microseconds
    pub avg_access_time_us: u64,
    /// P95 access time in microseconds
    pub p95_access_time_us: u64,
    /// Operations per second
    pub ops_per_second: f64,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Compression ratio
    pub compression_ratio: f32,
}

/// Tenant usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantUsageMetrics {
    /// Operations count
    pub operations: u64,
    /// Memory usage in bytes
    pub memory_bytes: u64,
    /// Data transfer in bytes
    pub data_transfer_bytes: u64,
    /// Error count
    pub error_count: u64,
    /// Active sessions
    pub active_sessions: u32,
}

/// System performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPerformanceMetrics {
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Memory usage percentage
    pub memory_usage_percent: f64,
    /// Network I/O bytes per second
    pub network_io_bps: u64,
    /// Disk I/O bytes per second
    pub disk_io_bps: u64,
    /// Active connections
    pub active_connections: u32,
}