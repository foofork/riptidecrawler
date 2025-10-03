use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main persistence configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PersistenceConfig {
    /// Redis connection configuration
    pub redis: RedisConfig,
    /// Cache-specific configuration
    pub cache: CacheConfig,
    /// State management configuration
    pub state: StateConfig,
    /// Tenant management configuration
    pub tenant: TenantConfig,
    /// Distributed coordination configuration
    pub distributed: Option<DistributedConfig>,
    /// Performance configuration
    pub performance: PerformanceConfig,
    /// Security configuration
    pub security: SecurityConfig,
}

/// Redis/DragonflyDB configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    /// Redis connection URL
    pub url: String,
    /// Connection pool size
    pub pool_size: u32,
    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u64,
    /// Command timeout in milliseconds
    pub command_timeout_ms: u64,
    /// Enable Redis Cluster mode
    pub cluster_mode: bool,
    /// Retry attempts for failed operations
    pub retry_attempts: u32,
    /// Retry delay in milliseconds
    pub retry_delay_ms: u64,
    /// Enable pipelining for bulk operations
    pub enable_pipelining: bool,
    /// Maximum pipeline size
    pub max_pipeline_size: usize,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379".to_string(),
            pool_size: 10,
            connection_timeout_ms: 5000,
            command_timeout_ms: 5000,
            cluster_mode: false,
            retry_attempts: 3,
            retry_delay_ms: 100,
            enable_pipelining: true,
            max_pipeline_size: 100,
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Default TTL in seconds
    pub default_ttl_seconds: u64,
    /// Maximum entry size in bytes
    pub max_entry_size_bytes: usize,
    /// Cache key prefix
    pub key_prefix: String,
    /// Cache version for invalidation
    pub version: String,
    /// Enable compression
    pub enable_compression: bool,
    /// Compression threshold in bytes
    pub compression_threshold_bytes: usize,
    /// Compression algorithm
    pub compression_algorithm: CompressionAlgorithm,
    /// Enable cache warming
    pub enable_warming: bool,
    /// Warming batch size
    pub warming_batch_size: usize,
    /// Maximum cache memory usage (bytes)
    pub max_memory_bytes: Option<u64>,
    /// Cache eviction policy
    pub eviction_policy: EvictionPolicy,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            default_ttl_seconds: 24 * 60 * 60,      // 24 hours
            max_entry_size_bytes: 20 * 1024 * 1024, // 20MB
            key_prefix: "riptide".to_string(),
            version: "v1".to_string(),
            enable_compression: true,
            compression_threshold_bytes: 1024, // 1KB
            compression_algorithm: CompressionAlgorithm::Lz4,
            enable_warming: true,
            warming_batch_size: 100,
            max_memory_bytes: None,
            eviction_policy: EvictionPolicy::LRU,
        }
    }
}

/// State management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateConfig {
    /// Session timeout in seconds
    pub session_timeout_seconds: u64,
    /// Enable configuration hot reload
    pub enable_hot_reload: bool,
    /// Configuration file watch paths
    pub config_watch_paths: Vec<String>,
    /// Checkpoint interval in seconds
    pub checkpoint_interval_seconds: u64,
    /// Maximum number of checkpoints to keep
    pub max_checkpoints: u32,
    /// Checkpoint compression
    pub checkpoint_compression: bool,
    /// Enable graceful shutdown with state preservation
    pub enable_graceful_shutdown: bool,
    /// Shutdown timeout in seconds
    pub shutdown_timeout_seconds: u64,
}

impl Default for StateConfig {
    fn default() -> Self {
        Self {
            session_timeout_seconds: 30 * 60, // 30 minutes
            enable_hot_reload: true,
            config_watch_paths: vec!["./config".to_string()],
            checkpoint_interval_seconds: 5 * 60, // 5 minutes
            max_checkpoints: 10,
            checkpoint_compression: true,
            enable_graceful_shutdown: true,
            shutdown_timeout_seconds: 30,
        }
    }
}

/// Tenant configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantConfig {
    /// Enable multi-tenancy
    pub enabled: bool,
    /// Default resource quotas
    pub default_quotas: HashMap<String, u64>,
    /// Tenant isolation level
    pub isolation_level: TenantIsolationLevel,
    /// Enable billing tracking
    pub enable_billing: bool,
    /// Billing aggregation interval in seconds
    pub billing_interval_seconds: u64,
    /// Maximum tenants per instance
    pub max_tenants: u32,
    /// Tenant data encryption
    pub enable_encryption: bool,
}

impl Default for TenantConfig {
    fn default() -> Self {
        let mut default_quotas = HashMap::new();
        default_quotas.insert("memory_bytes".to_string(), 100 * 1024 * 1024); // 100MB
        default_quotas.insert("operations_per_minute".to_string(), 1000);
        default_quotas.insert("storage_bytes".to_string(), 1024 * 1024 * 1024); // 1GB

        Self {
            enabled: true,
            default_quotas,
            isolation_level: TenantIsolationLevel::Strong,
            enable_billing: true,
            billing_interval_seconds: 60, // 1 minute
            max_tenants: 1000,
            enable_encryption: true,
        }
    }
}

/// Distributed coordination configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedConfig {
    /// Enable distributed coordination
    pub enabled: bool,
    /// Coordinator type
    pub coordinator_type: CoordinatorType,
    /// Node ID
    pub node_id: String,
    /// Cluster nodes
    pub cluster_nodes: Vec<String>,
    /// Consensus timeout in milliseconds
    pub consensus_timeout_ms: u64,
    /// Leader election timeout in milliseconds
    pub leader_election_timeout_ms: u64,
    /// Heartbeat interval in milliseconds
    pub heartbeat_interval_ms: u64,
}

impl Default for DistributedConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            coordinator_type: CoordinatorType::Redis,
            node_id: uuid::Uuid::new_v4().to_string(),
            cluster_nodes: vec![],
            consensus_timeout_ms: 5000,
            leader_election_timeout_ms: 10000,
            heartbeat_interval_ms: 1000,
        }
    }
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Target cache access time in milliseconds
    pub target_cache_access_ms: u64,
    /// Enable performance monitoring
    pub enable_monitoring: bool,
    /// Metrics collection interval in seconds
    pub metrics_interval_seconds: u64,
    /// Enable slow operation logging
    pub enable_slow_log: bool,
    /// Slow operation threshold in milliseconds
    pub slow_threshold_ms: u64,
    /// Enable connection pooling
    pub enable_connection_pooling: bool,
    /// Connection pool configuration
    pub pool_config: PoolConfig,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            target_cache_access_ms: 5,
            enable_monitoring: true,
            metrics_interval_seconds: 60,
            enable_slow_log: true,
            slow_threshold_ms: 100,
            enable_connection_pooling: true,
            pool_config: PoolConfig::default(),
        }
    }
}

/// Connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    /// Minimum pool size
    pub min_size: u32,
    /// Maximum pool size
    pub max_size: u32,
    /// Connection idle timeout in seconds
    pub idle_timeout_seconds: u64,
    /// Maximum connection lifetime in seconds
    pub max_lifetime_seconds: u64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_size: 2,
            max_size: 10,
            idle_timeout_seconds: 600,  // 10 minutes
            max_lifetime_seconds: 3600, // 1 hour
        }
    }
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable encryption at rest
    pub enable_encryption_at_rest: bool,
    /// Encryption key
    pub encryption_key: Option<String>,
    /// Enable audit logging
    pub enable_audit_logging: bool,
    /// Audit log retention in days
    pub audit_retention_days: u32,
    /// Enable rate limiting
    pub enable_rate_limiting: bool,
    /// Rate limit configuration
    pub rate_limits: HashMap<String, u64>,
    /// Enable RBAC
    pub enable_rbac: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        let mut rate_limits = HashMap::new();
        rate_limits.insert("operations_per_minute".to_string(), 1000);
        rate_limits.insert("data_per_minute_bytes".to_string(), 10 * 1024 * 1024); // 10MB

        Self {
            enable_encryption_at_rest: false,
            encryption_key: None,
            enable_audit_logging: true,
            audit_retention_days: 30,
            enable_rate_limiting: true,
            rate_limits,
            enable_rbac: false,
        }
    }
}

/// Compression algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    /// LZ4 - fast compression
    Lz4,
    /// Zstd - balanced compression
    Zstd,
    /// No compression
    None,
}

/// Cache eviction policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvictionPolicy {
    /// Least Recently Used
    LRU,
    /// Least Frequently Used
    LFU,
    /// Time To Live only
    TTL,
    /// Random eviction
    Random,
}

/// Tenant isolation levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TenantIsolationLevel {
    /// No isolation (single tenant)
    None,
    /// Logical isolation (namespace-based)
    Logical,
    /// Strong isolation (separate instances)
    Strong,
}

/// Coordinator types for distributed systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoordinatorType {
    /// Redis-based coordination
    Redis,
    /// Consul-based coordination
    Consul,
    /// etcd-based coordination
    Etcd,
    /// In-memory coordination (for testing)
    Memory,
}

impl PersistenceConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(redis_url) = std::env::var("REDIS_URL") {
            config.redis.url = redis_url;
        }

        if let Ok(cache_ttl) = std::env::var("CACHE_DEFAULT_TTL_SECONDS") {
            if let Ok(ttl) = cache_ttl.parse() {
                config.cache.default_ttl_seconds = ttl;
            }
        }

        if let Ok(enable_compression) = std::env::var("ENABLE_COMPRESSION") {
            config.cache.enable_compression = enable_compression.to_lowercase() == "true";
        }

        if let Ok(enable_multi_tenancy) = std::env::var("ENABLE_MULTI_TENANCY") {
            config.tenant.enabled = enable_multi_tenancy.to_lowercase() == "true";
        }

        config
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate Redis URL
        if self.redis.url.is_empty() {
            return Err("Redis URL cannot be empty".to_string());
        }

        // Validate performance targets
        if self.performance.target_cache_access_ms == 0 {
            return Err("Target cache access time must be greater than 0".to_string());
        }

        if self.performance.target_cache_access_ms > 100 {
            return Err(
                "Target cache access time should be less than 100ms for optimal performance"
                    .to_string(),
            );
        }

        // Validate cache settings
        if self.cache.max_entry_size_bytes == 0 {
            return Err("Maximum entry size must be greater than 0".to_string());
        }

        // Validate tenant settings
        if self.tenant.enabled && self.tenant.max_tenants == 0 {
            return Err(
                "Maximum tenants must be greater than 0 when multi-tenancy is enabled".to_string(),
            );
        }

        Ok(())
    }
}
