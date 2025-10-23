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
    /// Load configuration from environment variables with comprehensive support
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Redis configuration (10 fields)
        if let Ok(val) = std::env::var("REDIS_URL") {
            config.redis.url = val;
        }
        if let Ok(val) = std::env::var("REDIS_POOL_SIZE") {
            if let Ok(val) = val.parse() {
                config.redis.pool_size = val;
            }
        }
        if let Ok(val) = std::env::var("REDIS_CONNECTION_TIMEOUT_MS") {
            if let Ok(val) = val.parse() {
                config.redis.connection_timeout_ms = val;
            }
        }
        if let Ok(val) = std::env::var("REDIS_COMMAND_TIMEOUT_MS") {
            if let Ok(val) = val.parse() {
                config.redis.command_timeout_ms = val;
            }
        }
        if let Ok(val) = std::env::var("REDIS_CLUSTER_MODE") {
            config.redis.cluster_mode = val.to_lowercase() == "true";
        }
        if let Ok(val) = std::env::var("REDIS_RETRY_ATTEMPTS") {
            if let Ok(val) = val.parse() {
                config.redis.retry_attempts = val;
            }
        }
        if let Ok(val) = std::env::var("REDIS_RETRY_DELAY_MS") {
            if let Ok(val) = val.parse() {
                config.redis.retry_delay_ms = val;
            }
        }
        if let Ok(val) = std::env::var("REDIS_ENABLE_PIPELINING") {
            config.redis.enable_pipelining = val.to_lowercase() == "true";
        }
        if let Ok(val) = std::env::var("REDIS_MAX_PIPELINE_SIZE") {
            if let Ok(val) = val.parse() {
                config.redis.max_pipeline_size = val;
            }
        }

        // Cache configuration (11 fields)
        if let Ok(val) = std::env::var("CACHE_DEFAULT_TTL_SECONDS") {
            if let Ok(val) = val.parse() {
                config.cache.default_ttl_seconds = val;
            }
        }
        if let Ok(val) = std::env::var("CACHE_MAX_ENTRY_SIZE_BYTES") {
            if let Ok(val) = val.parse() {
                config.cache.max_entry_size_bytes = val;
            }
        }
        if let Ok(val) = std::env::var("CACHE_KEY_PREFIX") {
            config.cache.key_prefix = val;
        }
        if let Ok(val) = std::env::var("CACHE_VERSION") {
            config.cache.version = val;
        }
        if let Ok(val) = std::env::var("CACHE_ENABLE_COMPRESSION") {
            config.cache.enable_compression = val.to_lowercase() == "true";
        }
        if let Ok(val) = std::env::var("CACHE_COMPRESSION_THRESHOLD_BYTES") {
            if let Ok(val) = val.parse() {
                config.cache.compression_threshold_bytes = val;
            }
        }
        if let Ok(val) = std::env::var("CACHE_COMPRESSION_ALGORITHM") {
            config.cache.compression_algorithm = match val.to_lowercase().as_str() {
                "lz4" => CompressionAlgorithm::Lz4,
                "zstd" => CompressionAlgorithm::Zstd,
                "none" => CompressionAlgorithm::None,
                _ => CompressionAlgorithm::Lz4,
            };
        }
        if let Ok(val) = std::env::var("CACHE_ENABLE_WARMING") {
            config.cache.enable_warming = val.to_lowercase() == "true";
        }
        if let Ok(val) = std::env::var("CACHE_WARMING_BATCH_SIZE") {
            if let Ok(val) = val.parse() {
                config.cache.warming_batch_size = val;
            }
        }
        if let Ok(val) = std::env::var("CACHE_MAX_MEMORY_BYTES") {
            if let Ok(val) = val.parse() {
                config.cache.max_memory_bytes = Some(val);
            }
        }
        if let Ok(val) = std::env::var("CACHE_EVICTION_POLICY") {
            config.cache.eviction_policy = match val.to_uppercase().as_str() {
                "LRU" => EvictionPolicy::LRU,
                "LFU" => EvictionPolicy::LFU,
                "TTL" => EvictionPolicy::TTL,
                "RANDOM" => EvictionPolicy::Random,
                _ => EvictionPolicy::LRU,
            };
        }

        // State management configuration (8 fields)
        if let Ok(val) = std::env::var("STATE_SESSION_TIMEOUT_SECONDS") {
            if let Ok(val) = val.parse() {
                config.state.session_timeout_seconds = val;
            }
        }
        if let Ok(val) = std::env::var("STATE_ENABLE_HOT_RELOAD") {
            config.state.enable_hot_reload = val.to_lowercase() == "true";
        }
        if let Ok(val) = std::env::var("STATE_CONFIG_WATCH_PATHS") {
            config.state.config_watch_paths =
                val.split(',').map(|s| s.trim().to_string()).collect();
        }
        if let Ok(val) = std::env::var("STATE_CHECKPOINT_INTERVAL_SECONDS") {
            if let Ok(val) = val.parse() {
                config.state.checkpoint_interval_seconds = val;
            }
        }
        if let Ok(val) = std::env::var("STATE_MAX_CHECKPOINTS") {
            if let Ok(val) = val.parse() {
                config.state.max_checkpoints = val;
            }
        }
        if let Ok(val) = std::env::var("STATE_CHECKPOINT_COMPRESSION") {
            config.state.checkpoint_compression = val.to_lowercase() == "true";
        }
        if let Ok(val) = std::env::var("STATE_ENABLE_GRACEFUL_SHUTDOWN") {
            config.state.enable_graceful_shutdown = val.to_lowercase() == "true";
        }
        if let Ok(val) = std::env::var("STATE_SHUTDOWN_TIMEOUT_SECONDS") {
            if let Ok(val) = val.parse() {
                config.state.shutdown_timeout_seconds = val;
            }
        }

        // Tenant configuration (7 fields)
        if let Ok(val) = std::env::var("TENANT_ENABLED") {
            config.tenant.enabled = val.to_lowercase() == "true";
        }
        if let Ok(val) = std::env::var("TENANT_ISOLATION_LEVEL") {
            config.tenant.isolation_level = match val.to_lowercase().as_str() {
                "none" => TenantIsolationLevel::None,
                "logical" => TenantIsolationLevel::Logical,
                "strong" => TenantIsolationLevel::Strong,
                _ => TenantIsolationLevel::Strong,
            };
        }
        if let Ok(val) = std::env::var("TENANT_ENABLE_BILLING") {
            config.tenant.enable_billing = val.to_lowercase() == "true";
        }
        if let Ok(val) = std::env::var("TENANT_BILLING_INTERVAL_SECONDS") {
            if let Ok(val) = val.parse() {
                config.tenant.billing_interval_seconds = val;
            }
        }
        if let Ok(val) = std::env::var("TENANT_MAX_TENANTS") {
            if let Ok(val) = val.parse() {
                config.tenant.max_tenants = val;
            }
        }
        if let Ok(val) = std::env::var("TENANT_ENABLE_ENCRYPTION") {
            config.tenant.enable_encryption = val.to_lowercase() == "true";
        }

        // Performance configuration (7 fields)
        if let Ok(val) = std::env::var("PERF_TARGET_CACHE_ACCESS_MS") {
            if let Ok(val) = val.parse() {
                config.performance.target_cache_access_ms = val;
            }
        }
        if let Ok(val) = std::env::var("PERF_ENABLE_MONITORING") {
            config.performance.enable_monitoring = val.to_lowercase() == "true";
        }
        if let Ok(val) = std::env::var("PERF_METRICS_INTERVAL_SECONDS") {
            if let Ok(val) = val.parse() {
                config.performance.metrics_interval_seconds = val;
            }
        }
        if let Ok(val) = std::env::var("PERF_ENABLE_SLOW_LOG") {
            config.performance.enable_slow_log = val.to_lowercase() == "true";
        }
        if let Ok(val) = std::env::var("PERF_SLOW_THRESHOLD_MS") {
            if let Ok(val) = val.parse() {
                config.performance.slow_threshold_ms = val;
            }
        }
        if let Ok(val) = std::env::var("PERF_ENABLE_CONNECTION_POOLING") {
            config.performance.enable_connection_pooling = val.to_lowercase() == "true";
        }
        // Pool config (4 fields)
        if let Ok(val) = std::env::var("POOL_MIN_SIZE") {
            if let Ok(val) = val.parse() {
                config.performance.pool_config.min_size = val;
            }
        }
        if let Ok(val) = std::env::var("POOL_MAX_SIZE") {
            if let Ok(val) = val.parse() {
                config.performance.pool_config.max_size = val;
            }
        }
        if let Ok(val) = std::env::var("POOL_IDLE_TIMEOUT_SECONDS") {
            if let Ok(val) = val.parse() {
                config.performance.pool_config.idle_timeout_seconds = val;
            }
        }
        if let Ok(val) = std::env::var("POOL_MAX_LIFETIME_SECONDS") {
            if let Ok(val) = val.parse() {
                config.performance.pool_config.max_lifetime_seconds = val;
            }
        }

        // Security configuration (7 fields)
        if let Ok(val) = std::env::var("SECURITY_ENABLE_ENCRYPTION_AT_REST") {
            config.security.enable_encryption_at_rest = val.to_lowercase() == "true";
        }
        if let Ok(val) = std::env::var("SECURITY_ENCRYPTION_KEY") {
            config.security.encryption_key = Some(val);
        }
        if let Ok(val) = std::env::var("SECURITY_ENABLE_AUDIT_LOGGING") {
            config.security.enable_audit_logging = val.to_lowercase() == "true";
        }
        if let Ok(val) = std::env::var("SECURITY_AUDIT_RETENTION_DAYS") {
            if let Ok(val) = val.parse() {
                config.security.audit_retention_days = val;
            }
        }
        if let Ok(val) = std::env::var("SECURITY_ENABLE_RATE_LIMITING") {
            config.security.enable_rate_limiting = val.to_lowercase() == "true";
        }
        if let Ok(val) = std::env::var("SECURITY_ENABLE_RBAC") {
            config.security.enable_rbac = val.to_lowercase() == "true";
        }

        // Distributed configuration (8 fields) - optional
        if let Ok(val) = std::env::var("DISTRIBUTED_ENABLED") {
            let enabled = val.to_lowercase() == "true";
            if enabled {
                let mut dist_config = DistributedConfig {
                    enabled: true,
                    ..DistributedConfig::default()
                };

                if let Ok(val) = std::env::var("DISTRIBUTED_COORDINATOR_TYPE") {
                    dist_config.coordinator_type = match val.to_lowercase().as_str() {
                        "redis" => CoordinatorType::Redis,
                        "consul" => CoordinatorType::Consul,
                        "etcd" => CoordinatorType::Etcd,
                        "memory" => CoordinatorType::Memory,
                        _ => CoordinatorType::Redis,
                    };
                }
                if let Ok(val) = std::env::var("DISTRIBUTED_NODE_ID") {
                    dist_config.node_id = val;
                }
                if let Ok(val) = std::env::var("DISTRIBUTED_CLUSTER_NODES") {
                    dist_config.cluster_nodes =
                        val.split(',').map(|s| s.trim().to_string()).collect();
                }
                if let Ok(val) = std::env::var("DISTRIBUTED_CONSENSUS_TIMEOUT_MS") {
                    if let Ok(val) = val.parse() {
                        dist_config.consensus_timeout_ms = val;
                    }
                }
                if let Ok(val) = std::env::var("DISTRIBUTED_LEADER_ELECTION_TIMEOUT_MS") {
                    if let Ok(val) = val.parse() {
                        dist_config.leader_election_timeout_ms = val;
                    }
                }
                if let Ok(val) = std::env::var("DISTRIBUTED_HEARTBEAT_INTERVAL_MS") {
                    if let Ok(val) = val.parse() {
                        dist_config.heartbeat_interval_ms = val;
                    }
                }

                config.distributed = Some(dist_config);
            }
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
