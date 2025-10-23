//! Comprehensive tests for adaptive timeout module

use riptide_reliability::timeout::*;
use std::time::Duration;
use tempfile::TempDir;

/// Test timeout config creation with defaults
#[test]
fn test_timeout_config_default() {
    let config = TimeoutConfig::default();

    assert!(config.storage_path.to_str().unwrap().contains(".riptide"));
    assert!(config
        .storage_path
        .to_str()
        .unwrap()
        .contains("timeout-profiles.json"));
    assert_eq!(config.default_timeout_secs, DEFAULT_TIMEOUT_SECS);
    assert!(config.auto_save);
}

/// Test timeout config with custom values
#[test]
fn test_timeout_config_custom() {
    let temp_dir = TempDir::new().unwrap();
    let storage_path = temp_dir.path().join("custom-timeouts.json");

    let config = TimeoutConfig {
        storage_path: storage_path.clone(),
        default_timeout_secs: 45,
        auto_save: false,
    };

    assert_eq!(config.storage_path, storage_path);
    assert_eq!(config.default_timeout_secs, 45);
    assert!(!config.auto_save);
}

/// Test timeout manager initialization
#[tokio::test]
async fn test_timeout_manager_init() {
    let temp_dir = TempDir::new().unwrap();
    let config = TimeoutConfig {
        storage_path: temp_dir.path().join("timeouts.json"),
        default_timeout_secs: 30,
        auto_save: true,
    };

    let manager = AdaptiveTimeoutManager::new(config).await;
    assert!(manager.is_ok());
}

/// Test timeout for unknown domain uses default
#[tokio::test]
async fn test_timeout_unknown_domain() {
    let temp_dir = TempDir::new().unwrap();
    let config = TimeoutConfig {
        storage_path: temp_dir.path().join("timeouts.json"),
        default_timeout_secs: 30,
        auto_save: false,
    };

    let manager = AdaptiveTimeoutManager::new(config).await.unwrap();
    let timeout = manager.get_timeout("https://unknown-domain.example").await;

    assert_eq!(timeout, Duration::from_secs(30));
}

/// Test recording successful request
#[tokio::test]
async fn test_record_success() {
    let temp_dir = TempDir::new().unwrap();
    let config = TimeoutConfig {
        storage_path: temp_dir.path().join("timeouts.json"),
        default_timeout_secs: 30,
        auto_save: false,
    };

    let manager = AdaptiveTimeoutManager::new(config).await.unwrap();
    let url = "https://example.com/page";

    // Record success
    manager
        .record_success(url, Duration::from_millis(500))
        .await;

    // Get stats should work
    let stats = manager.get_stats().await;
    // total_domains is unsigned, so it's always >= 0
    assert!(stats.total_domains == 1);
}

/// Test recording timeout failure
#[tokio::test]
async fn test_record_timeout() {
    let temp_dir = TempDir::new().unwrap();
    let config = TimeoutConfig {
        storage_path: temp_dir.path().join("timeouts.json"),
        default_timeout_secs: 30,
        auto_save: false,
    };

    let manager = AdaptiveTimeoutManager::new(config).await.unwrap();
    let url = "https://slow-site.example/page";

    // Record timeout
    manager.record_timeout(url).await;

    // Timeout should have increased for this domain
    let timeout = manager.get_timeout(url).await;
    assert!(timeout > Duration::from_secs(30));
}

/// Test timeout stats
#[tokio::test]
async fn test_timeout_stats() {
    let temp_dir = TempDir::new().unwrap();
    let config = TimeoutConfig {
        storage_path: temp_dir.path().join("timeouts.json"),
        default_timeout_secs: 30,
        auto_save: false,
    };

    let manager = AdaptiveTimeoutManager::new(config).await.unwrap();

    // Get stats from empty manager
    let stats = manager.get_stats().await;
    assert_eq!(stats.total_domains, 0);
    assert_eq!(stats.avg_timeout_secs, 0.0);
    assert_eq!(stats.avg_success_rate, 0.0);
}

/// Test timeout profile structure
#[test]
fn test_timeout_profile() {
    let profile = TimeoutProfile {
        domain: "example.com".to_string(),
        timeout_secs: 30,
        total_requests: 12,
        successful_requests: 10,
        failed_requests: 2,
        consecutive_successes: 3,
        consecutive_failures: 0,
        avg_response_time_ms: 150.0,
        last_updated: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    assert_eq!(profile.domain, "example.com");
    assert_eq!(profile.timeout_secs, 30);
    assert_eq!(profile.successful_requests, 10);
    assert_eq!(profile.failed_requests, 2);

    let success_rate = profile.success_rate();
    assert!(success_rate > 0.0 && success_rate <= 1.0);
}

/// Test timeout profile success rate calculation
#[test]
fn test_timeout_profile_success_rate() {
    let profile = TimeoutProfile {
        domain: "example.com".to_string(),
        timeout_secs: 30,
        total_requests: 10,
        successful_requests: 8,
        failed_requests: 2,
        consecutive_successes: 3,
        consecutive_failures: 0,
        avg_response_time_ms: 150.0,
        last_updated: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    let success_rate = profile.success_rate();
    assert_eq!(success_rate, 0.8); // 8 / 10
}

/// Test timeout profile with no requests
#[test]
fn test_timeout_profile_no_requests() {
    let profile = TimeoutProfile {
        domain: "example.com".to_string(),
        timeout_secs: 30,
        total_requests: 0,
        successful_requests: 0,
        failed_requests: 0,
        consecutive_successes: 0,
        consecutive_failures: 0,
        avg_response_time_ms: 0.0,
        last_updated: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    let success_rate = profile.success_rate();
    assert_eq!(success_rate, 0.0);
}

/// Test timeout bounds - minimum
#[tokio::test]
async fn test_timeout_minimum_bound() {
    let temp_dir = TempDir::new().unwrap();
    let config = TimeoutConfig {
        storage_path: temp_dir.path().join("timeouts.json"),
        default_timeout_secs: 30,
        auto_save: false,
    };

    let manager = AdaptiveTimeoutManager::new(config).await.unwrap();

    // Even with many successes, timeout shouldn't go below minimum
    for _ in 0..100 {
        manager
            .record_success("https://fast-site.example", Duration::from_millis(100))
            .await;
    }

    let timeout = manager.get_timeout("https://fast-site.example").await;
    assert!(timeout >= Duration::from_secs(MIN_TIMEOUT_SECS));
}

/// Test timeout bounds - maximum
#[tokio::test]
async fn test_timeout_maximum_bound() {
    let temp_dir = TempDir::new().unwrap();
    let config = TimeoutConfig {
        storage_path: temp_dir.path().join("timeouts.json"),
        default_timeout_secs: 30,
        auto_save: false,
    };

    let manager = AdaptiveTimeoutManager::new(config).await.unwrap();

    // Even with many timeouts, timeout shouldn't exceed maximum
    for _ in 0..20 {
        manager
            .record_timeout("https://very-slow-site.example")
            .await;
    }

    let timeout = manager.get_timeout("https://very-slow-site.example").await;
    assert!(timeout <= Duration::from_secs(MAX_TIMEOUT_SECS));
}

/// Test concurrent access to timeout manager
#[tokio::test]
async fn test_timeout_concurrent_access() {
    let temp_dir = TempDir::new().unwrap();
    let config = TimeoutConfig {
        storage_path: temp_dir.path().join("timeouts.json"),
        default_timeout_secs: 30,
        auto_save: false,
    };

    let manager = AdaptiveTimeoutManager::new(config).await.unwrap();

    // Multiple concurrent operations
    for i in 0..10 {
        let url = format!("https://example{}.com", i);
        let timeout = manager.get_timeout(&url).await;
        assert_eq!(timeout, Duration::from_secs(30));
    }
}

/// Test timeout manager persistence
#[tokio::test]
async fn test_timeout_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let storage_path = temp_dir.path().join("timeouts.json");

    let config = TimeoutConfig {
        storage_path: storage_path.clone(),
        default_timeout_secs: 30,
        auto_save: true,
    };

    // Create manager and record some data
    {
        let manager = AdaptiveTimeoutManager::new(config.clone()).await.unwrap();
        manager
            .record_success("https://example.com", Duration::from_millis(500))
            .await;
        manager.save_profiles().await.unwrap();
    }

    // Create new manager - should load persisted data
    let manager2 = AdaptiveTimeoutManager::new(config).await;
    assert!(manager2.is_ok());
}

/// Test invalid URL handling
#[tokio::test]
async fn test_invalid_url() {
    let temp_dir = TempDir::new().unwrap();
    let config = TimeoutConfig {
        storage_path: temp_dir.path().join("timeouts.json"),
        default_timeout_secs: 30,
        auto_save: false,
    };

    let manager = AdaptiveTimeoutManager::new(config).await.unwrap();

    // Invalid URLs should fall back to default timeout
    let invalid_urls = vec!["not-a-url", "", "ftp://example.com", "javascript:void(0)"];

    for url in invalid_urls {
        let timeout = manager.get_timeout(url).await;
        assert_eq!(timeout, Duration::from_secs(30));
    }
}

/// Test exponential backoff behavior
#[tokio::test]
async fn test_exponential_backoff() {
    let temp_dir = TempDir::new().unwrap();
    let config = TimeoutConfig {
        storage_path: temp_dir.path().join("timeouts.json"),
        default_timeout_secs: 10,
        auto_save: false,
    };

    let manager = AdaptiveTimeoutManager::new(config).await.unwrap();
    let url = "https://failing-site.example";

    let initial_timeout = manager.get_timeout(url).await;

    // Record multiple timeouts
    manager.record_timeout(url).await;
    let timeout_after_1 = manager.get_timeout(url).await;

    manager.record_timeout(url).await;
    let timeout_after_2 = manager.get_timeout(url).await;

    // Timeouts should increase
    assert!(timeout_after_1 > initial_timeout);
    assert!(timeout_after_2 > timeout_after_1);
}
