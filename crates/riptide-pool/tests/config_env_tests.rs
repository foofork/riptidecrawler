//! Unit tests for environment variable configuration parsing in riptide-pool

use riptide_pool::config::ExtractorConfig;
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
fn test_pool_config_from_env_all_fields() {
    with_env_vars(
        vec![
            ("POOL_MAX_INSTANCES", "8"),
            ("POOL_ENABLE_METRICS", "false"),
            ("POOL_TIMEOUT_MS", "10000"),
            ("POOL_MEMORY_LIMIT_PAGES", "512"),
            ("POOL_EXTRACTION_TIMEOUT_MS", "60000"),
            ("POOL_MAX_POOL_SIZE", "16"),
            ("POOL_INITIAL_POOL_SIZE", "4"),
            ("POOL_EPOCH_TIMEOUT_MS", "120000"),
            ("POOL_HEALTH_CHECK_INTERVAL_MS", "60000"),
            ("POOL_MEMORY_LIMIT_BYTES", "1073741824"),
            ("POOL_CIRCUIT_BREAKER_TIMEOUT_MS", "10000"),
            ("POOL_CIRCUIT_BREAKER_FAILURE_THRESHOLD", "10"),
            ("POOL_ENABLE_WIT_VALIDATION", "false"),
        ],
        || {
            let config = ExtractorConfig::from_env();
            assert_eq!(config.max_instances, 8);
            assert!(!config.enable_metrics);
            assert_eq!(config.timeout_ms, 10000);
            assert_eq!(config.memory_limit_pages, Some(512));
            assert_eq!(config.extraction_timeout, Some(60000));
            assert_eq!(config.max_pool_size, 16);
            assert_eq!(config.initial_pool_size, 4);
            assert_eq!(config.epoch_timeout_ms, 120000);
            assert_eq!(config.health_check_interval, 60000);
            assert_eq!(config.memory_limit, Some(1073741824));
            assert_eq!(config.circuit_breaker_timeout, 10000);
            assert_eq!(config.circuit_breaker_failure_threshold, 10);
            assert!(!config.enable_wit_validation);
        },
    );
}

#[test]
fn test_pool_size_settings() {
    with_env_vars(
        vec![
            ("POOL_MAX_POOL_SIZE", "20"),
            ("POOL_INITIAL_POOL_SIZE", "5"),
        ],
        || {
            let config = ExtractorConfig::from_env();
            assert_eq!(config.max_pool_size, 20);
            assert_eq!(config.initial_pool_size, 5);
        },
    );
}

#[test]
fn test_timeout_settings() {
    with_env_vars(
        vec![
            ("POOL_TIMEOUT_MS", "8000"),
            ("POOL_EXTRACTION_TIMEOUT_MS", "45000"),
            ("POOL_EPOCH_TIMEOUT_MS", "90000"),
            ("POOL_CIRCUIT_BREAKER_TIMEOUT_MS", "8000"),
        ],
        || {
            let config = ExtractorConfig::from_env();
            assert_eq!(config.timeout_ms, 8000);
            assert_eq!(config.extraction_timeout, Some(45000));
            assert_eq!(config.epoch_timeout_ms, 90000);
            assert_eq!(config.circuit_breaker_timeout, 8000);
        },
    );
}

#[test]
fn test_memory_settings() {
    with_env_vars(
        vec![
            ("POOL_MEMORY_LIMIT_PAGES", "1024"),
            ("POOL_MEMORY_LIMIT_BYTES", "2147483648"),
        ],
        || {
            let config = ExtractorConfig::from_env();
            assert_eq!(config.memory_limit_pages, Some(1024));
            assert_eq!(config.memory_limit, Some(2147483648));
        },
    );
}

#[test]
fn test_circuit_breaker_settings() {
    with_env_vars(
        vec![
            ("POOL_CIRCUIT_BREAKER_TIMEOUT_MS", "15000"),
            ("POOL_CIRCUIT_BREAKER_FAILURE_THRESHOLD", "15"),
        ],
        || {
            let config = ExtractorConfig::from_env();
            assert_eq!(config.circuit_breaker_timeout, 15000);
            assert_eq!(config.circuit_breaker_failure_threshold, 15);
        },
    );
}

#[test]
fn test_boolean_flags() {
    with_env_vars(
        vec![
            ("POOL_ENABLE_METRICS", "true"),
            ("POOL_ENABLE_WIT_VALIDATION", "true"),
        ],
        || {
            let config = ExtractorConfig::from_env();
            assert!(config.enable_metrics);
            assert!(config.enable_wit_validation);
        },
    );

    with_env_vars(
        vec![
            ("POOL_ENABLE_METRICS", "false"),
            ("POOL_ENABLE_WIT_VALIDATION", "false"),
        ],
        || {
            let config = ExtractorConfig::from_env();
            assert!(!config.enable_metrics);
            assert!(!config.enable_wit_validation);
        },
    );
}

#[test]
fn test_default_config_when_no_env_vars() {
    let env_keys = vec![
        "POOL_MAX_INSTANCES",
        "POOL_ENABLE_METRICS",
        "POOL_TIMEOUT_MS",
    ];
    for key in &env_keys {
        env::remove_var(key);
    }

    let config = ExtractorConfig::from_env();
    let default = ExtractorConfig::default();

    assert_eq!(config.max_instances, default.max_instances);
    assert_eq!(config.enable_metrics, default.enable_metrics);
    assert_eq!(config.timeout_ms, default.timeout_ms);
}

#[test]
fn test_invalid_env_var_values_use_defaults() {
    with_env_vars(
        vec![
            ("POOL_MAX_INSTANCES", "invalid"),
            ("POOL_TIMEOUT_MS", "not_a_number"),
            ("POOL_ENABLE_METRICS", "maybe"),
        ],
        || {
            let config = ExtractorConfig::from_env();
            let default = ExtractorConfig::default();

            // Should fall back to defaults when parsing fails
            assert_eq!(config.max_instances, default.max_instances);
            assert_eq!(config.timeout_ms, default.timeout_ms);
            // Boolean parsing defaults to false for invalid values
            assert!(!config.enable_metrics);
        },
    );
}

#[test]
fn test_config_validation() {
    with_env_vars(
        vec![
            ("POOL_MAX_INSTANCES", "10"),
            ("POOL_MAX_POOL_SIZE", "20"),
            ("POOL_INITIAL_POOL_SIZE", "5"),
            ("POOL_TIMEOUT_MS", "5000"),
            ("POOL_CIRCUIT_BREAKER_FAILURE_THRESHOLD", "5"),
        ],
        || {
            let config = ExtractorConfig::from_env();
            assert!(config.validate().is_ok());
        },
    );
}

#[test]
fn test_config_validation_failures() {
    // Test max_instances = 0
    with_env_vars(vec![("POOL_MAX_INSTANCES", "0")], || {
        let config = ExtractorConfig::from_env();
        assert!(config.validate().is_err());
    });

    // Test max_pool_size = 0
    with_env_vars(vec![("POOL_MAX_POOL_SIZE", "0")], || {
        let config = ExtractorConfig::from_env();
        assert!(config.validate().is_err());
    });

    // Test initial_pool_size > max_pool_size
    with_env_vars(
        vec![
            ("POOL_MAX_POOL_SIZE", "5"),
            ("POOL_INITIAL_POOL_SIZE", "10"),
        ],
        || {
            let config = ExtractorConfig::from_env();
            assert!(config.validate().is_err());
        },
    );

    // Test timeout_ms = 0
    with_env_vars(vec![("POOL_TIMEOUT_MS", "0")], || {
        let config = ExtractorConfig::from_env();
        assert!(config.validate().is_err());
    });

    // Test circuit_breaker_failure_threshold = 0
    with_env_vars(
        vec![("POOL_CIRCUIT_BREAKER_FAILURE_THRESHOLD", "0")],
        || {
            let config = ExtractorConfig::from_env();
            assert!(config.validate().is_err());
        },
    );
}

#[test]
fn test_partial_env_var_override() {
    with_env_vars(
        vec![("POOL_MAX_INSTANCES", "6"), ("POOL_MAX_POOL_SIZE", "12")],
        || {
            let config = ExtractorConfig::from_env();
            let default = ExtractorConfig::default();

            // Overridden values
            assert_eq!(config.max_instances, 6);
            assert_eq!(config.max_pool_size, 12);

            // Default values for non-overridden fields
            assert_eq!(config.enable_metrics, default.enable_metrics);
            assert_eq!(config.timeout_ms, default.timeout_ms);
            assert_eq!(config.initial_pool_size, default.initial_pool_size);
        },
    );
}

#[test]
fn test_health_check_and_monitoring() {
    with_env_vars(
        vec![
            ("POOL_HEALTH_CHECK_INTERVAL_MS", "45000"),
            ("POOL_ENABLE_METRICS", "true"),
        ],
        || {
            let config = ExtractorConfig::from_env();
            assert_eq!(config.health_check_interval, 45000);
            assert!(config.enable_metrics);
        },
    );
}

#[test]
fn test_wit_validation_flag() {
    with_env_vars(vec![("POOL_ENABLE_WIT_VALIDATION", "false")], || {
        let config = ExtractorConfig::from_env();
        assert!(!config.enable_wit_validation);
    });

    with_env_vars(vec![("POOL_ENABLE_WIT_VALIDATION", "true")], || {
        let config = ExtractorConfig::from_env();
        assert!(config.enable_wit_validation);
    });
}

#[test]
fn test_realistic_production_config() {
    with_env_vars(
        vec![
            ("POOL_MAX_INSTANCES", "16"),
            ("POOL_ENABLE_METRICS", "true"),
            ("POOL_TIMEOUT_MS", "30000"),
            ("POOL_MEMORY_LIMIT_PAGES", "512"),
            ("POOL_EXTRACTION_TIMEOUT_MS", "120000"),
            ("POOL_MAX_POOL_SIZE", "32"),
            ("POOL_INITIAL_POOL_SIZE", "8"),
            ("POOL_EPOCH_TIMEOUT_MS", "300000"),
            ("POOL_HEALTH_CHECK_INTERVAL_MS", "60000"),
            ("POOL_MEMORY_LIMIT_BYTES", "2147483648"),
            ("POOL_CIRCUIT_BREAKER_TIMEOUT_MS", "10000"),
            ("POOL_CIRCUIT_BREAKER_FAILURE_THRESHOLD", "10"),
            ("POOL_ENABLE_WIT_VALIDATION", "true"),
        ],
        || {
            let config = ExtractorConfig::from_env();
            assert!(config.validate().is_ok());
            assert_eq!(config.max_instances, 16);
            assert_eq!(config.max_pool_size, 32);
            assert_eq!(config.initial_pool_size, 8);
            assert!(config.enable_metrics);
            assert!(config.enable_wit_validation);
        },
    );
}
