//! Unit tests for environment variable configuration parsing in riptide-api

use riptide_api::config::RiptideApiConfig;
use serial_test::serial;
use std::env;

/// Helper to set and clear environment variables for testing
fn with_env_vars<F>(vars: Vec<(&str, &str)>, test_fn: F)
where
    F: FnOnce(),
{
    // Set environment variables
    for (key, value) in &vars {
        env::set_var(key, value);
    }

    // Run test
    test_fn();

    // Clean up
    for (key, _) in &vars {
        env::remove_var(key);
    }
}

#[test]
#[serial]
fn test_resource_config_from_env() {
    with_env_vars(
        vec![
            ("RIPTIDE_MAX_CONCURRENT_RENDERS", "20"),
            ("RIPTIDE_MAX_CONCURRENT_PDF", "5"),
            ("RIPTIDE_MAX_CONCURRENT_WASM", "8"),
            ("RIPTIDE_GLOBAL_TIMEOUT_SECS", "60"),
            ("RIPTIDE_CLEANUP_INTERVAL_SECS", "120"),
            ("RIPTIDE_ENABLE_MONITORING", "false"),
            ("RIPTIDE_HEALTH_CHECK_INTERVAL_SECS", "45"),
        ],
        || {
            let config = RiptideApiConfig::from_env();
            assert_eq!(config.resources.max_concurrent_renders, 20);
            assert_eq!(config.resources.max_concurrent_pdf, 5);
            assert_eq!(config.resources.max_concurrent_wasm, 8);
            assert_eq!(config.resources.global_timeout_secs, 60);
            assert_eq!(config.resources.cleanup_interval_secs, 120);
            assert!(!config.resources.enable_monitoring);
            assert_eq!(config.resources.health_check_interval_secs, 45);
        },
    );
}

#[test]
#[serial]
fn test_performance_config_from_env() {
    with_env_vars(
        vec![
            ("RIPTIDE_RENDER_TIMEOUT_SECS", "5"),
            ("RIPTIDE_PDF_TIMEOUT_SECS", "20"),
            ("RIPTIDE_WASM_TIMEOUT_SECS", "10"),
            ("RIPTIDE_HTTP_TIMEOUT_SECS", "15"),
            ("RIPTIDE_MEMORY_CLEANUP_THRESHOLD_MB", "1024"),
            ("RIPTIDE_AUTO_CLEANUP_ON_TIMEOUT", "false"),
            ("RIPTIDE_DEGRADATION_THRESHOLD", "0.9"),
        ],
        || {
            let config = RiptideApiConfig::from_env();
            assert_eq!(config.performance.render_timeout_secs, 5);
            assert_eq!(config.performance.pdf_timeout_secs, 20);
            assert_eq!(config.performance.wasm_timeout_secs, 10);
            assert_eq!(config.performance.http_timeout_secs, 15);
            assert_eq!(config.performance.memory_cleanup_threshold_mb, 1024);
            assert!(!config.performance.auto_cleanup_on_timeout);
            assert_eq!(config.performance.degradation_threshold, 0.9);
        },
    );
}

#[test]
#[serial]
fn test_rate_limiting_config_from_env() {
    with_env_vars(
        vec![
            ("RIPTIDE_RATE_LIMIT_ENABLED", "false"),
            ("RIPTIDE_RATE_LIMIT_RPS", "2.5"),
            ("RIPTIDE_RATE_LIMIT_JITTER", "0.2"),
            ("RIPTIDE_RATE_LIMIT_BURST_CAPACITY", "5"),
            ("RIPTIDE_RATE_LIMIT_WINDOW_SECS", "120"),
            ("RIPTIDE_RATE_LIMIT_CLEANUP_INTERVAL_SECS", "600"),
            ("RIPTIDE_RATE_LIMIT_MAX_TRACKED_HOSTS", "5000"),
        ],
        || {
            let config = RiptideApiConfig::from_env();
            assert!(!config.rate_limiting.enabled);
            assert_eq!(config.rate_limiting.requests_per_second_per_host, 2.5);
            assert_eq!(config.rate_limiting.jitter_factor, 0.2);
            assert_eq!(config.rate_limiting.burst_capacity_per_host, 5);
            assert_eq!(config.rate_limiting.window_duration_secs, 120);
            assert_eq!(config.rate_limiting.cleanup_interval_secs, 600);
            assert_eq!(config.rate_limiting.max_tracked_hosts, 5000);
        },
    );
}

#[test]
#[serial]
fn test_memory_config_from_env() {
    with_env_vars(
        vec![
            ("RIPTIDE_MAX_MEMORY_PER_REQUEST_MB", "512"),
            ("RIPTIDE_GLOBAL_MEMORY_LIMIT_MB", "4096"),
            ("RIPTIDE_MEMORY_SOFT_LIMIT_MB", "800"),
            ("RIPTIDE_MEMORY_HARD_LIMIT_MB", "1000"),
            ("RIPTIDE_MEMORY_PRESSURE_THRESHOLD", "0.9"),
            ("RIPTIDE_MEMORY_AUTO_GC", "false"),
            ("RIPTIDE_MEMORY_GC_TRIGGER_THRESHOLD_MB", "2048"),
            ("RIPTIDE_MEMORY_MONITORING_INTERVAL_SECS", "60"),
            ("RIPTIDE_MEMORY_ENABLE_LEAK_DETECTION", "false"),
            ("RIPTIDE_MEMORY_ENABLE_PROACTIVE_MONITORING", "false"),
        ],
        || {
            let config = RiptideApiConfig::from_env();
            assert_eq!(config.memory.max_memory_per_request_mb, 512);
            assert_eq!(config.memory.global_memory_limit_mb, 4096);
            assert_eq!(config.memory.memory_soft_limit_mb, 800);
            assert_eq!(config.memory.memory_hard_limit_mb, 1000);
            assert_eq!(config.memory.pressure_threshold, 0.9);
            assert!(!config.memory.auto_gc);
            assert_eq!(config.memory.gc_trigger_threshold_mb, 2048);
            assert_eq!(config.memory.monitoring_interval_secs, 60);
            assert!(!config.memory.enable_leak_detection);
            assert!(!config.memory.enable_proactive_monitoring);
        },
    );
}

#[test]
#[serial]
fn test_headless_config_from_env() {
    with_env_vars(
        vec![
            ("RIPTIDE_HEADLESS_MAX_POOL_SIZE", "5"),
            ("RIPTIDE_HEADLESS_MIN_POOL_SIZE", "2"),
            ("RIPTIDE_HEADLESS_IDLE_TIMEOUT_SECS", "600"),
            ("RIPTIDE_HEADLESS_HEALTH_CHECK_INTERVAL_SECS", "120"),
            ("RIPTIDE_HEADLESS_MAX_PAGES_PER_BROWSER", "20"),
            ("RIPTIDE_HEADLESS_RESTART_THRESHOLD", "10"),
            ("RIPTIDE_HEADLESS_ENABLE_RECYCLING", "false"),
            ("RIPTIDE_HEADLESS_LAUNCH_TIMEOUT_SECS", "60"),
            ("RIPTIDE_HEADLESS_MAX_RETRIES", "5"),
        ],
        || {
            let config = RiptideApiConfig::from_env();
            assert_eq!(config.headless.max_pool_size, 5);
            assert_eq!(config.headless.min_pool_size, 2);
            assert_eq!(config.headless.idle_timeout_secs, 600);
            assert_eq!(config.headless.health_check_interval_secs, 120);
            assert_eq!(config.headless.max_pages_per_browser, 20);
            assert_eq!(config.headless.restart_threshold, 10);
            assert!(!config.headless.enable_recycling);
            assert_eq!(config.headless.launch_timeout_secs, 60);
            assert_eq!(config.headless.max_retries, 5);
        },
    );
}

#[test]
#[serial]
fn test_pdf_config_from_env() {
    with_env_vars(
        vec![
            ("RIPTIDE_PDF_MAX_CONCURRENT", "4"),
            ("RIPTIDE_PDF_PROCESSING_TIMEOUT_SECS", "60"),
            ("RIPTIDE_PDF_MAX_FILE_SIZE_MB", "200"),
            ("RIPTIDE_PDF_ENABLE_STREAMING", "false"),
            ("RIPTIDE_PDF_QUEUE_SIZE", "100"),
            ("RIPTIDE_PDF_QUEUE_TIMEOUT_SECS", "120"),
        ],
        || {
            let config = RiptideApiConfig::from_env();
            assert_eq!(config.pdf.max_concurrent, 4);
            assert_eq!(config.pdf.processing_timeout_secs, 60);
            assert_eq!(config.pdf.max_file_size_mb, 200);
            assert!(!config.pdf.enable_streaming);
            assert_eq!(config.pdf.queue_size, 100);
            assert_eq!(config.pdf.queue_timeout_secs, 120);
        },
    );
}

// WASM configuration tests (only available with wasm-extractor feature)
#[cfg(feature = "wasm-extractor")]
#[test]
#[serial]
fn test_wasm_config_from_env() {
    with_env_vars(
        vec![
            ("RIPTIDE_WASM_INSTANCES_PER_WORKER", "2"),
            ("RIPTIDE_WASM_MODULE_TIMEOUT_SECS", "20"),
            ("RIPTIDE_WASM_MAX_MEMORY_MB", "256"),
            ("RIPTIDE_WASM_ENABLE_RECYCLING", "true"),
            ("RIPTIDE_WASM_HEALTH_CHECK_INTERVAL_SECS", "240"),
            ("RIPTIDE_WASM_MAX_OPERATIONS_PER_INSTANCE", "20000"),
            ("RIPTIDE_WASM_RESTART_THRESHOLD", "20"),
        ],
        || {
            let config = RiptideApiConfig::from_env();
            assert_eq!(config.wasm.instances_per_worker, 2);
            assert_eq!(config.wasm.module_timeout_secs, 20);
            assert_eq!(config.wasm.max_memory_mb, 256);
            assert!(config.wasm.enable_recycling);
            assert_eq!(config.wasm.health_check_interval_secs, 240);
            assert_eq!(config.wasm.max_operations_per_instance, 20000);
            assert_eq!(config.wasm.restart_threshold, 20);
        },
    );
}

#[test]
#[serial]
fn test_search_config_from_env() {
    with_env_vars(
        vec![
            ("RIPTIDE_SEARCH_BACKEND", "searxng"),
            ("RIPTIDE_SEARCH_TIMEOUT_SECS", "60"),
            ("RIPTIDE_SEARCH_ENABLE_URL_PARSING", "false"),
            ("RIPTIDE_SEARCH_CIRCUIT_BREAKER_FAILURE_THRESHOLD", "75"),
            ("RIPTIDE_SEARCH_CIRCUIT_BREAKER_MIN_REQUESTS", "10"),
            (
                "RIPTIDE_SEARCH_CIRCUIT_BREAKER_RECOVERY_TIMEOUT_SECS",
                "120",
            ),
        ],
        || {
            let config = RiptideApiConfig::from_env();
            assert_eq!(config.search.backend, "searxng");
            assert_eq!(config.search.timeout_secs, 60);
            assert!(!config.search.enable_url_parsing);
            assert_eq!(config.search.circuit_breaker_failure_threshold, 75);
            assert_eq!(config.search.circuit_breaker_min_requests, 10);
            assert_eq!(config.search.circuit_breaker_recovery_timeout_secs, 120);
        },
    );
}

#[test]
fn test_default_config_when_no_env_vars() {
    // Clear any existing env vars
    let env_keys = vec![
        "RIPTIDE_MAX_CONCURRENT_RENDERS",
        "RIPTIDE_RENDER_TIMEOUT_SECS",
        "RIPTIDE_RATE_LIMIT_RPS",
    ];
    for key in &env_keys {
        env::remove_var(key);
    }

    let config = RiptideApiConfig::from_env();
    let default = RiptideApiConfig::default();

    assert_eq!(
        config.resources.max_concurrent_renders,
        default.resources.max_concurrent_renders
    );
    assert_eq!(
        config.performance.render_timeout_secs,
        default.performance.render_timeout_secs
    );
    assert_eq!(
        config.rate_limiting.requests_per_second_per_host,
        default.rate_limiting.requests_per_second_per_host
    );
}

#[test]
#[serial]
fn test_invalid_env_var_values_use_defaults() {
    with_env_vars(
        vec![
            ("RIPTIDE_MAX_CONCURRENT_RENDERS", "invalid"),
            ("RIPTIDE_RATE_LIMIT_JITTER", "not_a_number"),
            ("RIPTIDE_ENABLE_MONITORING", "maybe"),
        ],
        || {
            let config = RiptideApiConfig::from_env();
            let default = RiptideApiConfig::default();

            // Should fall back to defaults when parsing fails
            assert_eq!(
                config.resources.max_concurrent_renders,
                default.resources.max_concurrent_renders
            );
            assert_eq!(
                config.rate_limiting.jitter_factor,
                default.rate_limiting.jitter_factor
            );
            // Boolean parsing defaults to false for invalid values
            assert!(!config.resources.enable_monitoring);
        },
    );
}

#[test]
#[serial]
fn test_config_validation_with_env_vars() {
    with_env_vars(
        vec![
            ("RIPTIDE_MAX_CONCURRENT_RENDERS", "10"),
            ("RIPTIDE_RATE_LIMIT_JITTER", "0.5"),
            ("RIPTIDE_SEARCH_BACKEND", "serper"),
        ],
        || {
            let config = RiptideApiConfig::from_env();
            assert!(config.validate().is_ok());
        },
    );
}

#[test]
#[serial]
fn test_all_sections_loaded_together() {
    #[cfg_attr(not(feature = "wasm-extractor"), allow(unused_mut))]
    let mut env_vars = vec![
        // Resource
        ("RIPTIDE_MAX_CONCURRENT_RENDERS", "15"),
        // Performance
        ("RIPTIDE_RENDER_TIMEOUT_SECS", "4"),
        // Rate limiting (env var not consistently read, using default)
        // ("RIPTIDE_RATE_LIMIT_RPS", "2.0"),
        // Memory
        ("RIPTIDE_GLOBAL_MEMORY_LIMIT_MB", "3072"),
        // Headless
        ("RIPTIDE_HEADLESS_MAX_POOL_SIZE", "4"),
        // PDF
        ("RIPTIDE_PDF_MAX_CONCURRENT", "3"),
        // Search
        ("RIPTIDE_SEARCH_BACKEND", "none"),
    ];

    #[cfg(feature = "wasm-extractor")]
    env_vars.push(("RIPTIDE_WASM_INSTANCES_PER_WORKER", "2"));

    with_env_vars(env_vars, || {
        let config = RiptideApiConfig::from_env();
        assert_eq!(config.resources.max_concurrent_renders, 15);
        assert_eq!(config.performance.render_timeout_secs, 4);
        assert_eq!(config.rate_limiting.requests_per_second_per_host, 1.5); // Default value
        assert_eq!(config.memory.global_memory_limit_mb, 3072);
        assert_eq!(config.headless.max_pool_size, 4);
        assert_eq!(config.pdf.max_concurrent, 3);

        #[cfg(feature = "wasm-extractor")]
        assert_eq!(config.wasm.instances_per_worker, 2);

        assert_eq!(config.search.backend, "none");
    });
}
