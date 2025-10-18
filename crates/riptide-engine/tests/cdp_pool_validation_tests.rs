//! CDP Pool Configuration Validation Tests
//!
//! Comprehensive test suite for CdpPoolConfig::validate() method
//! covering all edge cases and validation rules.
//!
//! **Test Coverage:**
//! - Valid configurations (default and custom)
//! - Invalid max_connections_per_browser (0, >1000)
//! - Invalid connection_idle_timeout (<1s)
//! - Invalid max_connection_lifetime (< idle_timeout)
//! - Invalid health_check_interval (<1s)
//! - Invalid batch_timeout (<1ms, >10s)
//! - Invalid max_batch_size (0, >100)
//! - Boundary cases
//! - Configuration combinations

use riptide_engine::cdp_pool::CdpPoolConfig;
use std::time::Duration;

/// Test 1: Default configuration should be valid
#[test]
fn test_default_config_valid() {
    let config = CdpPoolConfig::default();
    assert!(
        config.validate().is_ok(),
        "Default configuration should be valid"
    );
}

/// Test 2: Custom valid configuration
#[test]
fn test_custom_valid_config() {
    let config = CdpPoolConfig {
        max_connections_per_browser: 20,
        connection_idle_timeout: Duration::from_secs(60),
        max_connection_lifetime: Duration::from_secs(600),
        enable_health_checks: true,
        health_check_interval: Duration::from_secs(30),
        enable_batching: true,
        batch_timeout: Duration::from_millis(100),
        max_batch_size: 50,
    };

    assert!(config.validate().is_ok(), "Custom valid config should pass");
}

/// Test 3: ERROR - max_connections_per_browser = 0
#[test]
fn test_invalid_max_connections_zero() {
    let config = CdpPoolConfig {
        max_connections_per_browser: 0,
        ..Default::default()
    };

    let result = config.validate();
    assert!(result.is_err(), "Zero max_connections should be invalid");
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("max_connections_per_browser must be > 0"));
}

/// Test 4: ERROR - max_connections_per_browser > 1000
#[test]
fn test_invalid_max_connections_too_large() {
    let config = CdpPoolConfig {
        max_connections_per_browser: 1001,
        ..Default::default()
    };

    let result = config.validate();
    assert!(result.is_err(), "Max connections > 1000 should be invalid");
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("max_connections_per_browser must be <= 1000"));
}

/// Test 5: BOUNDARY - max_connections_per_browser = 1 (minimum valid)
#[test]
fn test_boundary_max_connections_one() {
    let config = CdpPoolConfig {
        max_connections_per_browser: 1,
        ..Default::default()
    };

    assert!(
        config.validate().is_ok(),
        "Max connections = 1 should be valid"
    );
}

/// Test 6: BOUNDARY - max_connections_per_browser = 1000 (maximum valid)
#[test]
fn test_boundary_max_connections_thousand() {
    let config = CdpPoolConfig {
        max_connections_per_browser: 1000,
        ..Default::default()
    };

    assert!(
        config.validate().is_ok(),
        "Max connections = 1000 should be valid"
    );
}

/// Test 7: ERROR - connection_idle_timeout < 1 second
#[test]
fn test_invalid_idle_timeout_too_short() {
    let config = CdpPoolConfig {
        connection_idle_timeout: Duration::from_millis(999),
        max_connection_lifetime: Duration::from_secs(300),
        ..Default::default()
    };

    let result = config.validate();
    assert!(result.is_err(), "Idle timeout < 1 second should be invalid");
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("connection_idle_timeout must be >= 1 second"));
}

/// Test 8: BOUNDARY - connection_idle_timeout = 1 second (minimum valid)
#[test]
fn test_boundary_idle_timeout_one_second() {
    let config = CdpPoolConfig {
        connection_idle_timeout: Duration::from_secs(1),
        max_connection_lifetime: Duration::from_secs(2),
        ..Default::default()
    };

    assert!(
        config.validate().is_ok(),
        "Idle timeout = 1 second should be valid"
    );
}

/// Test 9: ERROR - max_connection_lifetime <= connection_idle_timeout
#[test]
fn test_invalid_lifetime_less_than_idle() {
    let config = CdpPoolConfig {
        connection_idle_timeout: Duration::from_secs(60),
        max_connection_lifetime: Duration::from_secs(60), // Equal, not greater
        ..Default::default()
    };

    let result = config.validate();
    assert!(
        result.is_err(),
        "Max lifetime <= idle timeout should be invalid"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("max_connection_lifetime") && err_msg.contains("must be >"));
}

/// Test 10: ERROR - max_connection_lifetime < connection_idle_timeout
#[test]
fn test_invalid_lifetime_less_than_idle_strict() {
    let config = CdpPoolConfig {
        connection_idle_timeout: Duration::from_secs(60),
        max_connection_lifetime: Duration::from_secs(30), // Less than idle
        ..Default::default()
    };

    let result = config.validate();
    assert!(
        result.is_err(),
        "Max lifetime < idle timeout should be invalid"
    );
}

/// Test 11: BOUNDARY - max_connection_lifetime = idle_timeout + 1ms
#[test]
fn test_boundary_lifetime_just_greater_than_idle() {
    let config = CdpPoolConfig {
        connection_idle_timeout: Duration::from_secs(60),
        max_connection_lifetime: Duration::from_millis(60_001), // Just slightly more
        ..Default::default()
    };

    assert!(
        config.validate().is_ok(),
        "Max lifetime slightly > idle timeout should be valid"
    );
}

/// Test 12: ERROR - health_check_interval < 1 second (when enabled)
#[test]
fn test_invalid_health_check_interval_too_short() {
    let config = CdpPoolConfig {
        enable_health_checks: true,
        health_check_interval: Duration::from_millis(500),
        ..Default::default()
    };

    let result = config.validate();
    assert!(
        result.is_err(),
        "Health check interval < 1 second should be invalid"
    );
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("health_check_interval must be >= 1 second"));
}

/// Test 13: Health check interval < 1 second OK when health checks disabled
#[test]
fn test_health_check_interval_short_when_disabled() {
    let config = CdpPoolConfig {
        enable_health_checks: false,
        health_check_interval: Duration::from_millis(100), // Short but health checks disabled
        ..Default::default()
    };

    assert!(
        config.validate().is_ok(),
        "Short health check interval OK when disabled"
    );
}

/// Test 14: BOUNDARY - health_check_interval = 1 second (minimum valid when enabled)
#[test]
fn test_boundary_health_check_interval_one_second() {
    let config = CdpPoolConfig {
        enable_health_checks: true,
        health_check_interval: Duration::from_secs(1),
        ..Default::default()
    };

    assert!(
        config.validate().is_ok(),
        "Health check interval = 1 second should be valid"
    );
}

/// Test 15: ERROR - batch_timeout < 1ms (when batching enabled)
#[test]
fn test_invalid_batch_timeout_too_short() {
    let config = CdpPoolConfig {
        enable_batching: true,
        batch_timeout: Duration::from_micros(999), // Less than 1ms
        ..Default::default()
    };

    let result = config.validate();
    assert!(result.is_err(), "Batch timeout < 1ms should be invalid");
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("batch_timeout must be >= 1ms"));
}

/// Test 16: ERROR - batch_timeout > 10 seconds (when batching enabled)
#[test]
fn test_invalid_batch_timeout_too_long() {
    let config = CdpPoolConfig {
        enable_batching: true,
        batch_timeout: Duration::from_secs(11), // More than 10 seconds
        ..Default::default()
    };

    let result = config.validate();
    assert!(result.is_err(), "Batch timeout > 10s should be invalid");
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("batch_timeout must be <= 10 seconds"));
}

/// Test 17: BOUNDARY - batch_timeout = 1ms (minimum valid when enabled)
#[test]
fn test_boundary_batch_timeout_one_ms() {
    let config = CdpPoolConfig {
        enable_batching: true,
        batch_timeout: Duration::from_millis(1),
        ..Default::default()
    };

    assert!(
        config.validate().is_ok(),
        "Batch timeout = 1ms should be valid"
    );
}

/// Test 18: BOUNDARY - batch_timeout = 10 seconds (maximum valid when enabled)
#[test]
fn test_boundary_batch_timeout_ten_seconds() {
    let config = CdpPoolConfig {
        enable_batching: true,
        batch_timeout: Duration::from_secs(10),
        ..Default::default()
    };

    assert!(
        config.validate().is_ok(),
        "Batch timeout = 10s should be valid"
    );
}

/// Test 19: Batch timeout outside range OK when batching disabled
#[test]
fn test_batch_timeout_invalid_when_batching_disabled() {
    let config = CdpPoolConfig {
        enable_batching: false,
        batch_timeout: Duration::from_secs(20), // Too long, but batching disabled
        ..Default::default()
    };

    assert!(
        config.validate().is_ok(),
        "Invalid batch timeout OK when batching disabled"
    );
}

/// Test 20: ERROR - max_batch_size = 0 (when batching enabled)
#[test]
fn test_invalid_max_batch_size_zero() {
    let config = CdpPoolConfig {
        enable_batching: true,
        max_batch_size: 0,
        ..Default::default()
    };

    let result = config.validate();
    assert!(result.is_err(), "Max batch size = 0 should be invalid");
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("max_batch_size must be > 0"));
}

/// Test 21: ERROR - max_batch_size > 100 (when batching enabled)
#[test]
fn test_invalid_max_batch_size_too_large() {
    let config = CdpPoolConfig {
        enable_batching: true,
        max_batch_size: 101,
        ..Default::default()
    };

    let result = config.validate();
    assert!(result.is_err(), "Max batch size > 100 should be invalid");
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("max_batch_size must be <= 100"));
}

/// Test 22: BOUNDARY - max_batch_size = 1 (minimum valid when enabled)
#[test]
fn test_boundary_max_batch_size_one() {
    let config = CdpPoolConfig {
        enable_batching: true,
        max_batch_size: 1,
        ..Default::default()
    };

    assert!(
        config.validate().is_ok(),
        "Max batch size = 1 should be valid"
    );
}

/// Test 23: BOUNDARY - max_batch_size = 100 (maximum valid when enabled)
#[test]
fn test_boundary_max_batch_size_hundred() {
    let config = CdpPoolConfig {
        enable_batching: true,
        max_batch_size: 100,
        ..Default::default()
    };

    assert!(
        config.validate().is_ok(),
        "Max batch size = 100 should be valid"
    );
}

/// Test 24: Max batch size 0 OK when batching disabled
#[test]
fn test_max_batch_size_zero_when_batching_disabled() {
    let config = CdpPoolConfig {
        enable_batching: false,
        max_batch_size: 0, // Invalid, but batching disabled
        ..Default::default()
    };

    assert!(
        config.validate().is_ok(),
        "Zero batch size OK when batching disabled"
    );
}

/// Test 25: Extreme valid configuration
#[test]
fn test_extreme_valid_config() {
    let config = CdpPoolConfig {
        max_connections_per_browser: 1000,               // Maximum
        connection_idle_timeout: Duration::from_secs(1), // Minimum
        max_connection_lifetime: Duration::from_secs(2), // Just valid
        enable_health_checks: true,
        health_check_interval: Duration::from_secs(1), // Minimum
        enable_batching: true,
        batch_timeout: Duration::from_millis(1), // Minimum
        max_batch_size: 100,                     // Maximum
    };

    assert!(
        config.validate().is_ok(),
        "Extreme but valid config should pass"
    );
}

/// Test 26: Conservative valid configuration
#[test]
fn test_conservative_valid_config() {
    let config = CdpPoolConfig {
        max_connections_per_browser: 1, // Minimum
        connection_idle_timeout: Duration::from_secs(10),
        max_connection_lifetime: Duration::from_secs(3600), // 1 hour
        enable_health_checks: true,
        health_check_interval: Duration::from_secs(60), // 1 minute
        enable_batching: true,
        batch_timeout: Duration::from_secs(1),
        max_batch_size: 1, // Minimum
    };

    assert!(
        config.validate().is_ok(),
        "Conservative config should be valid"
    );
}

/// Test 27: All features disabled configuration
#[test]
fn test_all_features_disabled_config() {
    let config = CdpPoolConfig {
        max_connections_per_browser: 5,
        connection_idle_timeout: Duration::from_secs(30),
        max_connection_lifetime: Duration::from_secs(300),
        enable_health_checks: false,
        health_check_interval: Duration::from_millis(100), // Invalid but disabled
        enable_batching: false,
        batch_timeout: Duration::from_micros(1), // Invalid but disabled
        max_batch_size: 0,                       // Invalid but disabled
    };

    assert!(
        config.validate().is_ok(),
        "Config with disabled features should ignore their invalid values"
    );
}

/// Test 28: Multiple validation errors (test first error returned)
#[test]
fn test_multiple_validation_errors() {
    let config = CdpPoolConfig {
        max_connections_per_browser: 0,                      // Error 1
        connection_idle_timeout: Duration::from_millis(100), // Error 2
        max_connection_lifetime: Duration::from_millis(50),  // Error 3
        enable_batching: true,
        batch_timeout: Duration::from_micros(1), // Error 4
        max_batch_size: 0,                       // Error 5
        ..Default::default()
    };

    let result = config.validate();
    assert!(result.is_err(), "Should fail validation");
    // Should return first error encountered (max_connections_per_browser)
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("max_connections_per_browser"));
}

/// Test 29: Production-like valid configuration
#[test]
fn test_production_like_config() {
    let config = CdpPoolConfig {
        max_connections_per_browser: 50,
        connection_idle_timeout: Duration::from_secs(60),
        max_connection_lifetime: Duration::from_secs(1800), // 30 minutes
        enable_health_checks: true,
        health_check_interval: Duration::from_secs(30),
        enable_batching: true,
        batch_timeout: Duration::from_millis(50),
        max_batch_size: 20,
    };

    assert!(
        config.validate().is_ok(),
        "Production-like config should be valid"
    );
}

/// Test 30: Edge case - very large but valid connection lifetime
#[test]
fn test_edge_case_very_large_lifetime() {
    let config = CdpPoolConfig {
        connection_idle_timeout: Duration::from_secs(1),
        max_connection_lifetime: Duration::from_secs(86400), // 24 hours
        ..Default::default()
    };

    assert!(
        config.validate().is_ok(),
        "Very large lifetime should be valid if > idle timeout"
    );
}
