//! Unit tests for environment variable configuration parsing in riptide-persistence

use riptide_persistence::config::{
    CompressionAlgorithm, CoordinatorType, EvictionPolicy, PersistenceConfig, TenantIsolationLevel,
};
use std::env;

/// Helper to set and clear environment variables for testing
fn with_env_vars<F>(vars: Vec<(&str, &str)>, test_fn: F)
where
    F: FnOnce(),
{
    for (key, value) in &vars {
        env::set_var(key, value);
    }
    test_fn();
    for (key, _) in &vars {
        env::remove_var(key);
    }
}

#[test]
fn test_redis_config_from_env() {
    with_env_vars(
        vec![
            ("REDIS_URL", "redis://test.example.com:6379"),
            ("REDIS_POOL_SIZE", "20"),
            ("REDIS_CONNECTION_TIMEOUT_MS", "10000"),
            ("REDIS_COMMAND_TIMEOUT_MS", "8000"),
            ("REDIS_CLUSTER_MODE", "true"),
            ("REDIS_RETRY_ATTEMPTS", "5"),
            ("REDIS_RETRY_DELAY_MS", "200"),
            ("REDIS_ENABLE_PIPELINING", "false"),
            ("REDIS_MAX_PIPELINE_SIZE", "200"),
        ],
        || {
            let config = PersistenceConfig::from_env();
            assert_eq!(config.redis.url, "redis://test.example.com:6379");
            assert_eq!(config.redis.pool_size, 20);
            assert_eq!(config.redis.connection_timeout_ms, 10000);
            assert_eq!(config.redis.command_timeout_ms, 8000);
            assert!(config.redis.cluster_mode);
            assert_eq!(config.redis.retry_attempts, 5);
            assert_eq!(config.redis.retry_delay_ms, 200);
            assert!(!config.redis.enable_pipelining);
            assert_eq!(config.redis.max_pipeline_size, 200);
        },
    );
}

#[test]
fn test_cache_config_from_env() {
    with_env_vars(
        vec![
            ("CACHE_DEFAULT_TTL_SECONDS", "3600"),
            ("CACHE_MAX_ENTRY_SIZE_BYTES", "10485760"),
            ("CACHE_KEY_PREFIX", "test_prefix"),
            ("CACHE_VERSION", "v2"),
            ("CACHE_ENABLE_COMPRESSION", "false"),
            ("CACHE_COMPRESSION_THRESHOLD_BYTES", "2048"),
            ("CACHE_COMPRESSION_ALGORITHM", "zstd"),
            ("CACHE_ENABLE_WARMING", "false"),
            ("CACHE_WARMING_BATCH_SIZE", "200"),
            ("CACHE_MAX_MEMORY_BYTES", "1073741824"),
            ("CACHE_EVICTION_POLICY", "LFU"),
        ],
        || {
            let config = PersistenceConfig::from_env();
            assert_eq!(config.cache.default_ttl_seconds, 3600);
            assert_eq!(config.cache.max_entry_size_bytes, 10485760);
            assert_eq!(config.cache.key_prefix, "test_prefix");
            assert_eq!(config.cache.version, "v2");
            assert!(!config.cache.enable_compression);
            assert_eq!(config.cache.compression_threshold_bytes, 2048);
            assert!(matches!(
                config.cache.compression_algorithm,
                CompressionAlgorithm::Zstd
            ));
            assert!(!config.cache.enable_warming);
            assert_eq!(config.cache.warming_batch_size, 200);
            assert_eq!(config.cache.max_memory_bytes, Some(1073741824));
            assert!(matches!(config.cache.eviction_policy, EvictionPolicy::LFU));
        },
    );
}

#[test]
fn test_cache_compression_algorithm_variants() {
    with_env_vars(vec![("CACHE_COMPRESSION_ALGORITHM", "lz4")], || {
        let config = PersistenceConfig::from_env();
        assert!(matches!(
            config.cache.compression_algorithm,
            CompressionAlgorithm::Lz4
        ));
    });

    with_env_vars(vec![("CACHE_COMPRESSION_ALGORITHM", "none")], || {
        let config = PersistenceConfig::from_env();
        assert!(matches!(
            config.cache.compression_algorithm,
            CompressionAlgorithm::None
        ));
    });
}

#[test]
fn test_state_config_from_env() {
    with_env_vars(
        vec![
            ("STATE_SESSION_TIMEOUT_SECONDS", "3600"),
            ("STATE_ENABLE_HOT_RELOAD", "false"),
            ("STATE_CONFIG_WATCH_PATHS", "/config,/etc/config"),
            ("STATE_CHECKPOINT_INTERVAL_SECONDS", "600"),
            ("STATE_MAX_CHECKPOINTS", "20"),
            ("STATE_CHECKPOINT_COMPRESSION", "false"),
            ("STATE_ENABLE_GRACEFUL_SHUTDOWN", "false"),
            ("STATE_SHUTDOWN_TIMEOUT_SECONDS", "60"),
        ],
        || {
            let config = PersistenceConfig::from_env();
            assert_eq!(config.state.session_timeout_seconds, 3600);
            assert!(!config.state.enable_hot_reload);
            assert_eq!(
                config.state.config_watch_paths,
                vec!["/config".to_string(), "/etc/config".to_string()]
            );
            assert_eq!(config.state.checkpoint_interval_seconds, 600);
            assert_eq!(config.state.max_checkpoints, 20);
            assert!(!config.state.checkpoint_compression);
            assert!(!config.state.enable_graceful_shutdown);
            assert_eq!(config.state.shutdown_timeout_seconds, 60);
        },
    );
}

#[test]
fn test_tenant_config_from_env() {
    with_env_vars(
        vec![
            ("TENANT_ENABLED", "false"),
            ("TENANT_ISOLATION_LEVEL", "logical"),
            ("TENANT_ENABLE_BILLING", "false"),
            ("TENANT_BILLING_INTERVAL_SECONDS", "120"),
            ("TENANT_MAX_TENANTS", "2000"),
            ("TENANT_ENABLE_ENCRYPTION", "false"),
        ],
        || {
            let config = PersistenceConfig::from_env();
            assert!(!config.tenant.enabled);
            assert!(matches!(
                config.tenant.isolation_level,
                TenantIsolationLevel::Logical
            ));
            assert!(!config.tenant.enable_billing);
            assert_eq!(config.tenant.billing_interval_seconds, 120);
            assert_eq!(config.tenant.max_tenants, 2000);
            assert!(!config.tenant.enable_encryption);
        },
    );
}

#[test]
fn test_tenant_isolation_level_variants() {
    with_env_vars(vec![("TENANT_ISOLATION_LEVEL", "none")], || {
        let config = PersistenceConfig::from_env();
        assert!(matches!(
            config.tenant.isolation_level,
            TenantIsolationLevel::None
        ));
    });

    with_env_vars(vec![("TENANT_ISOLATION_LEVEL", "strong")], || {
        let config = PersistenceConfig::from_env();
        assert!(matches!(
            config.tenant.isolation_level,
            TenantIsolationLevel::Strong
        ));
    });
}

#[test]
fn test_performance_config_from_env() {
    with_env_vars(
        vec![
            ("PERF_TARGET_CACHE_ACCESS_MS", "10"),
            ("PERF_ENABLE_MONITORING", "false"),
            ("PERF_METRICS_INTERVAL_SECONDS", "120"),
            ("PERF_ENABLE_SLOW_LOG", "false"),
            ("PERF_SLOW_THRESHOLD_MS", "200"),
            ("PERF_ENABLE_CONNECTION_POOLING", "false"),
            ("POOL_MIN_SIZE", "5"),
            ("POOL_MAX_SIZE", "20"),
            ("POOL_IDLE_TIMEOUT_SECONDS", "1200"),
            ("POOL_MAX_LIFETIME_SECONDS", "7200"),
        ],
        || {
            let config = PersistenceConfig::from_env();
            assert_eq!(config.performance.target_cache_access_ms, 10);
            assert!(!config.performance.enable_monitoring);
            assert_eq!(config.performance.metrics_interval_seconds, 120);
            assert!(!config.performance.enable_slow_log);
            assert_eq!(config.performance.slow_threshold_ms, 200);
            assert!(!config.performance.enable_connection_pooling);
            assert_eq!(config.performance.pool_config.min_size, 5);
            assert_eq!(config.performance.pool_config.max_size, 20);
            assert_eq!(config.performance.pool_config.idle_timeout_seconds, 1200);
            assert_eq!(config.performance.pool_config.max_lifetime_seconds, 7200);
        },
    );
}

#[test]
fn test_security_config_from_env() {
    with_env_vars(
        vec![
            ("SECURITY_ENABLE_ENCRYPTION_AT_REST", "true"),
            ("SECURITY_ENCRYPTION_KEY", "test-key-12345"),
            ("SECURITY_ENABLE_AUDIT_LOGGING", "false"),
            ("SECURITY_AUDIT_RETENTION_DAYS", "60"),
            ("SECURITY_ENABLE_RATE_LIMITING", "false"),
            ("SECURITY_ENABLE_RBAC", "true"),
        ],
        || {
            let config = PersistenceConfig::from_env();
            assert!(config.security.enable_encryption_at_rest);
            assert_eq!(
                config.security.encryption_key,
                Some("test-key-12345".to_string())
            );
            assert!(!config.security.enable_audit_logging);
            assert_eq!(config.security.audit_retention_days, 60);
            assert!(!config.security.enable_rate_limiting);
            assert!(config.security.enable_rbac);
        },
    );
}

#[test]
fn test_distributed_config_from_env() {
    with_env_vars(
        vec![
            ("DISTRIBUTED_ENABLED", "true"),
            ("DISTRIBUTED_COORDINATOR_TYPE", "consul"),
            ("DISTRIBUTED_NODE_ID", "node-123"),
            (
                "DISTRIBUTED_CLUSTER_NODES",
                "node1:8080,node2:8080,node3:8080",
            ),
            ("DISTRIBUTED_CONSENSUS_TIMEOUT_MS", "10000"),
            ("DISTRIBUTED_LEADER_ELECTION_TIMEOUT_MS", "20000"),
            ("DISTRIBUTED_HEARTBEAT_INTERVAL_MS", "2000"),
        ],
        || {
            let config = PersistenceConfig::from_env();
            assert!(config.distributed.is_some());
            let dist = config.distributed.unwrap();
            assert!(dist.enabled);
            assert!(matches!(dist.coordinator_type, CoordinatorType::Consul));
            assert_eq!(dist.node_id, "node-123");
            assert_eq!(
                dist.cluster_nodes,
                vec![
                    "node1:8080".to_string(),
                    "node2:8080".to_string(),
                    "node3:8080".to_string()
                ]
            );
            assert_eq!(dist.consensus_timeout_ms, 10000);
            assert_eq!(dist.leader_election_timeout_ms, 20000);
            assert_eq!(dist.heartbeat_interval_ms, 2000);
        },
    );
}

#[test]
fn test_distributed_config_disabled() {
    with_env_vars(vec![("DISTRIBUTED_ENABLED", "false")], || {
        let config = PersistenceConfig::from_env();
        // Should still be None when disabled
        assert!(config.distributed.is_none());
    });
}

#[test]
fn test_coordinator_type_variants() {
    let types = vec![
        ("redis", CoordinatorType::Redis),
        ("consul", CoordinatorType::Consul),
        ("etcd", CoordinatorType::Etcd),
        ("memory", CoordinatorType::Memory),
    ];

    for (type_str, expected_type) in types {
        with_env_vars(
            vec![
                ("DISTRIBUTED_ENABLED", "true"),
                ("DISTRIBUTED_COORDINATOR_TYPE", type_str),
            ],
            || {
                let config = PersistenceConfig::from_env();
                let dist = config.distributed.unwrap();
                assert!(matches!(dist.coordinator_type, expected_type));
            },
        );
    }
}

#[test]
fn test_default_config_when_no_env_vars() {
    let env_keys = vec!["REDIS_URL", "CACHE_DEFAULT_TTL_SECONDS", "TENANT_ENABLED"];
    for key in &env_keys {
        env::remove_var(key);
    }

    let config = PersistenceConfig::from_env();
    let default = PersistenceConfig::default();

    assert_eq!(config.redis.url, default.redis.url);
    assert_eq!(
        config.cache.default_ttl_seconds,
        default.cache.default_ttl_seconds
    );
    assert_eq!(config.tenant.enabled, default.tenant.enabled);
}

#[test]
fn test_invalid_env_var_values_use_defaults() {
    with_env_vars(
        vec![
            ("REDIS_POOL_SIZE", "invalid"),
            ("CACHE_ENABLE_COMPRESSION", "maybe"),
            ("PERF_TARGET_CACHE_ACCESS_MS", "not_a_number"),
        ],
        || {
            let config = PersistenceConfig::from_env();
            let default = PersistenceConfig::default();

            assert_eq!(config.redis.pool_size, default.redis.pool_size);
            assert!(!config.cache.enable_compression);
            assert_eq!(
                config.performance.target_cache_access_ms,
                default.performance.target_cache_access_ms
            );
        },
    );
}

#[test]
fn test_config_validation_with_env_vars() {
    with_env_vars(
        vec![
            ("REDIS_URL", "redis://valid:6379"),
            ("PERF_TARGET_CACHE_ACCESS_MS", "10"),
        ],
        || {
            let config = PersistenceConfig::from_env();
            assert!(config.validate().is_ok());
        },
    );
}

#[test]
fn test_all_sections_loaded_together() {
    with_env_vars(
        vec![
            ("REDIS_URL", "redis://multi-test:6379"),
            ("CACHE_DEFAULT_TTL_SECONDS", "7200"),
            ("STATE_SESSION_TIMEOUT_SECONDS", "2400"),
            ("TENANT_ENABLED", "true"),
            ("PERF_TARGET_CACHE_ACCESS_MS", "8"),
            ("SECURITY_ENABLE_AUDIT_LOGGING", "true"),
        ],
        || {
            let config = PersistenceConfig::from_env();
            assert_eq!(config.redis.url, "redis://multi-test:6379");
            assert_eq!(config.cache.default_ttl_seconds, 7200);
            assert_eq!(config.state.session_timeout_seconds, 2400);
            assert!(config.tenant.enabled);
            assert_eq!(config.performance.target_cache_access_ms, 8);
            assert!(config.security.enable_audit_logging);
        },
    );
}
