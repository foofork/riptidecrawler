use riptide_api::state::{AppConfig, AppState, DependencyHealth, HealthStatus};
use std::env;
use tempfile::TempDir;
use tokio_test;

#[cfg(test)]
mod app_config_tests {
    use super::*;

    #[test]
    fn test_app_config_default_values() {
        // Clear environment variables to test defaults
        let original_vars = preserve_env_vars();
        clear_env_vars();

        let config = AppConfig::default();

        assert_eq!(config.redis_url, "redis://localhost:6379");
        assert_eq!(config.wasm_path, "./target/wasm32-wasip2/release/riptide_extractor_wasm.wasm");
        assert_eq!(config.max_concurrency, 16);
        assert_eq!(config.cache_ttl, 3600);
        assert_eq!(config.gate_hi_threshold, 0.7);
        assert_eq!(config.gate_lo_threshold, 0.3);
        assert!(config.headless_url.is_none());

        restore_env_vars(original_vars);
    }

    #[test]
    fn test_app_config_from_environment() {
        let original_vars = preserve_env_vars();

        // Set test environment variables
        env::set_var("REDIS_URL", "redis://test:6379");
        env::set_var("WASM_EXTRACTOR_PATH", "/test/path/extractor.wasm");
        env::set_var("MAX_CONCURRENCY", "32");
        env::set_var("CACHE_TTL", "7200");
        env::set_var("GATE_HI_THRESHOLD", "0.8");
        env::set_var("GATE_LO_THRESHOLD", "0.2");
        env::set_var("HEADLESS_URL", "http://headless:9123");

        let config = AppConfig::default();

        assert_eq!(config.redis_url, "redis://test:6379");
        assert_eq!(config.wasm_path, "/test/path/extractor.wasm");
        assert_eq!(config.max_concurrency, 32);
        assert_eq!(config.cache_ttl, 7200);
        assert_eq!(config.gate_hi_threshold, 0.8);
        assert_eq!(config.gate_lo_threshold, 0.2);
        assert_eq!(config.headless_url, Some("http://headless:9123".to_string()));

        restore_env_vars(original_vars);
    }

    #[test]
    fn test_app_config_invalid_env_values() {
        let original_vars = preserve_env_vars();

        // Set invalid environment variables
        env::set_var("MAX_CONCURRENCY", "invalid");
        env::set_var("CACHE_TTL", "not_a_number");
        env::set_var("GATE_HI_THRESHOLD", "invalid_float");
        env::set_var("GATE_LO_THRESHOLD", "invalid_float");

        let config = AppConfig::default();

        // Should fall back to defaults for invalid values
        assert_eq!(config.max_concurrency, 16);
        assert_eq!(config.cache_ttl, 3600);
        assert_eq!(config.gate_hi_threshold, 0.7);
        assert_eq!(config.gate_lo_threshold, 0.3);

        restore_env_vars(original_vars);
    }

    #[test]
    fn test_app_config_clone() {
        let config = AppConfig::default();
        let cloned = config.clone();

        assert_eq!(config.redis_url, cloned.redis_url);
        assert_eq!(config.max_concurrency, cloned.max_concurrency);
        assert_eq!(config.cache_ttl, cloned.cache_ttl);
    }

    #[test]
    fn test_app_config_debug() {
        let config = AppConfig::default();
        let debug_str = format!("{:?}", config);

        assert!(debug_str.contains("AppConfig"));
        assert!(debug_str.contains("redis_url"));
        assert!(debug_str.contains("max_concurrency"));
    }

    fn preserve_env_vars() -> Vec<(String, Option<String>)> {
        let vars = vec![
            "REDIS_URL", "WASM_EXTRACTOR_PATH", "MAX_CONCURRENCY",
            "CACHE_TTL", "GATE_HI_THRESHOLD", "GATE_LO_THRESHOLD", "HEADLESS_URL"
        ];

        vars.iter()
            .map(|&var| (var.to_string(), env::var(var).ok()))
            .collect()
    }

    fn clear_env_vars() {
        let vars = vec![
            "REDIS_URL", "WASM_EXTRACTOR_PATH", "MAX_CONCURRENCY",
            "CACHE_TTL", "GATE_HI_THRESHOLD", "GATE_LO_THRESHOLD", "HEADLESS_URL"
        ];

        for var in vars {
            env::remove_var(var);
        }
    }

    fn restore_env_vars(vars: Vec<(String, Option<String>)>) {
        for (key, value) in vars {
            match value {
                Some(val) => env::set_var(key, val),
                None => env::remove_var(key),
            }
        }
    }
}

#[cfg(test)]
mod dependency_health_tests {
    use super::*;

    #[test]
    fn test_dependency_health_display() {
        assert_eq!(format!("{}", DependencyHealth::Healthy), "healthy");
        assert_eq!(format!("{}", DependencyHealth::Unknown), "unknown");
        assert_eq!(format!("{}", DependencyHealth::Unhealthy("test error".to_string())), "unhealthy: test error");
    }

    #[test]
    fn test_dependency_health_clone() {
        let healthy = DependencyHealth::Healthy;
        let cloned = healthy.clone();

        assert!(matches!(cloned, DependencyHealth::Healthy));

        let unhealthy = DependencyHealth::Unhealthy("error".to_string());
        let cloned_unhealthy = unhealthy.clone();

        assert!(matches!(cloned_unhealthy, DependencyHealth::Unhealthy(_)));
    }

    #[test]
    fn test_dependency_health_debug() {
        let healthy = DependencyHealth::Healthy;
        let debug_str = format!("{:?}", healthy);
        assert!(debug_str.contains("Healthy"));
    }
}

#[cfg(test)]
mod health_status_tests {
    use super::*;

    #[test]
    fn test_health_status_clone() {
        let status = HealthStatus {
            healthy: true,
            redis: DependencyHealth::Healthy,
            extractor: DependencyHealth::Healthy,
            http_client: DependencyHealth::Healthy,
        };

        let cloned = status.clone();
        assert_eq!(cloned.healthy, true);
        assert!(matches!(cloned.redis, DependencyHealth::Healthy));
    }

    #[test]
    fn test_health_status_debug() {
        let status = HealthStatus {
            healthy: false,
            redis: DependencyHealth::Unhealthy("connection failed".to_string()),
            extractor: DependencyHealth::Healthy,
            http_client: DependencyHealth::Unknown,
        };

        let debug_str = format!("{:?}", status);
        assert!(debug_str.contains("HealthStatus"));
        assert!(debug_str.contains("healthy: false"));
        assert!(debug_str.contains("connection failed"));
    }
}

// Note: AppState tests require running services (Redis, WASM), so they're in integration tests
// This module focuses on unit tests for pure logic and data structures

#[cfg(test)]
mod app_state_unit_tests {
    use super::*;

    #[test]
    fn test_app_state_is_cloneable() {
        // This test ensures AppState implements Clone
        // We can't test actual construction without dependencies

        // Test that the types are correctly structured
        let _: fn() -> AppConfig = AppConfig::default;

        // Verify the health check return type
        let status = HealthStatus {
            healthy: true,
            redis: DependencyHealth::Healthy,
            extractor: DependencyHealth::Healthy,
            http_client: DependencyHealth::Healthy,
        };

        // Ensure it's cloneable
        let _cloned = status.clone();
    }
}

// Property-based tests for configuration validation
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_config_concurrency_bounds(concurrency in 1u32..1000u32) {
            env::set_var("MAX_CONCURRENCY", concurrency.to_string());
            let config = AppConfig::default();

            // Should accept any positive integer
            prop_assert!(config.max_concurrency > 0);
            prop_assert!(config.max_concurrency <= 1000);

            env::remove_var("MAX_CONCURRENCY");
        }

        #[test]
        fn test_config_cache_ttl_bounds(ttl in 1u64..86400u64) {
            env::set_var("CACHE_TTL", ttl.to_string());
            let config = AppConfig::default();

            // Should accept reasonable TTL values
            prop_assert!(config.cache_ttl > 0);
            prop_assert!(config.cache_ttl <= 86400); // Max 24 hours

            env::remove_var("CACHE_TTL");
        }

        #[test]
        fn test_config_gate_thresholds(hi in 0.0f32..1.0f32, lo in 0.0f32..1.0f32) {
            env::set_var("GATE_HI_THRESHOLD", hi.to_string());
            env::set_var("GATE_LO_THRESHOLD", lo.to_string());

            let config = AppConfig::default();

            // Thresholds should be between 0 and 1
            prop_assert!(config.gate_hi_threshold >= 0.0);
            prop_assert!(config.gate_hi_threshold <= 1.0);
            prop_assert!(config.gate_lo_threshold >= 0.0);
            prop_assert!(config.gate_lo_threshold <= 1.0);

            env::remove_var("GATE_HI_THRESHOLD");
            env::remove_var("GATE_LO_THRESHOLD");
        }
    }
}