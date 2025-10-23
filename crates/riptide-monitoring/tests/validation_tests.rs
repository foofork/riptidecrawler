//! Tests for validation module

use anyhow::Result;
use riptide_monitoring::validation::*;
use serde_json::json;

/// Mock HTTP client for testing
struct MockHttpClient {
    should_fail: bool,
    redis_status: String,
}

#[async_trait::async_trait]
impl HttpClient for MockHttpClient {
    async fn get_json(&self, path: &str) -> Result<serde_json::Value> {
        if self.should_fail {
            anyhow::bail!("Mock connection failure");
        }

        if path.contains("health/detailed") {
            Ok(json!({
                "redis": self.redis_status,
            }))
        } else {
            Ok(json!({}))
        }
    }

    async fn get_health(&self, _path: &str) -> Result<()> {
        if self.should_fail {
            anyhow::bail!("Mock health check failure");
        }
        Ok(())
    }
}

#[tokio::test]
async fn test_check_api_connectivity_success() {
    let client = MockHttpClient {
        should_fail: false,
        redis_status: "connected".to_string(),
    };

    let result = check_api_connectivity(&client).await;

    assert!(matches!(result.status, CheckStatus::Pass));
    assert_eq!(result.name, "API Connectivity");
}

#[tokio::test]
async fn test_check_api_connectivity_failure() {
    let client = MockHttpClient {
        should_fail: true,
        redis_status: "connected".to_string(),
    };

    let result = check_api_connectivity(&client).await;

    assert!(matches!(result.status, CheckStatus::Fail));
    assert!(result.remediation.is_some());
}

#[tokio::test]
async fn test_check_redis_connected() {
    let client = MockHttpClient {
        should_fail: false,
        redis_status: "connected".to_string(),
    };

    let result = check_redis(&client).await;

    assert!(matches!(result.status, CheckStatus::Pass));
    assert_eq!(result.name, "Redis");
}

#[tokio::test]
async fn test_check_redis_not_configured() {
    let client = MockHttpClient {
        should_fail: false,
        redis_status: "not_configured".to_string(),
    };

    let result = check_redis(&client).await;

    assert!(matches!(result.status, CheckStatus::Warning));
}

#[tokio::test]
async fn test_check_redis_disconnected() {
    let client = MockHttpClient {
        should_fail: false,
        redis_status: "disconnected".to_string(),
    };

    let result = check_redis(&client).await;

    assert!(matches!(result.status, CheckStatus::Fail));
    assert!(result.remediation.is_some());
}

#[tokio::test]
async fn test_check_filesystem_permissions() {
    let result = check_filesystem_permissions().await;

    // Should pass if we can create cache directory
    assert!(matches!(
        result.status,
        CheckStatus::Pass | CheckStatus::Fail
    ));
}

#[tokio::test]
async fn test_check_configuration() {
    let result = check_configuration().await;

    // Should return pass or warning depending on env vars
    assert!(matches!(
        result.status,
        CheckStatus::Pass | CheckStatus::Warning
    ));
}

#[tokio::test]
async fn test_check_system_resources() {
    let result = check_system_resources().await;

    // Should always complete
    assert!(matches!(
        result.status,
        CheckStatus::Pass | CheckStatus::Warning
    ));

    // Should have details
    assert!(result.details.is_some());
}

#[tokio::test]
async fn test_validation_report_creation() {
    let checks = vec![
        CheckResult::pass("Test 1", "Passed"),
        CheckResult::fail("Test 2", "Failed", "Fix it"),
        CheckResult::warning("Test 3", "Warning"),
        CheckResult::skipped("Test 4", "Skipped"),
    ];

    let report = ValidationReport::new(checks);

    assert_eq!(report.summary.total_checks, 4);
    assert_eq!(report.summary.passed, 1);
    assert_eq!(report.summary.failed, 1);
    assert_eq!(report.summary.warnings, 1);
    assert_eq!(report.summary.skipped, 1);
    assert!(matches!(report.summary.overall_status, CheckStatus::Fail));
}

#[tokio::test]
async fn test_validation_report_exit_code() {
    let pass_report = ValidationReport::new(vec![CheckResult::pass("Test", "OK")]);
    assert_eq!(pass_report.exit_code(), 0);

    let fail_report = ValidationReport::new(vec![CheckResult::fail("Test", "Failed", "Fix")]);
    assert_eq!(fail_report.exit_code(), 1);

    let warn_report = ValidationReport::new(vec![CheckResult::warning("Test", "Warning")]);
    assert_eq!(warn_report.exit_code(), 0);
}

#[tokio::test]
async fn test_check_result_with_details() {
    let result = CheckResult::pass("Test", "OK").with_details(json!({"key": "value"}));

    assert!(result.details.is_some());
    assert_eq!(result.details.unwrap()["key"], "value");
}

#[tokio::test]
async fn test_comprehensive_validation() {
    let client = MockHttpClient {
        should_fail: false,
        redis_status: "connected".to_string(),
    };

    let report = run_comprehensive_validation(&client, None).await;

    // Should have multiple checks
    assert!(report.checks.len() >= 5);

    // Should have timestamp
    assert!(!report.timestamp.is_empty());
}

#[tokio::test]
async fn test_production_checks() {
    let client = MockHttpClient {
        should_fail: false,
        redis_status: "connected".to_string(),
    };

    let report = run_production_checks(&client).await;

    // Production checks should include all critical checks
    assert!(report.checks.len() >= 5);
}

#[tokio::test]
async fn test_performance_baseline() {
    let client = MockHttpClient {
        should_fail: false,
        redis_status: "connected".to_string(),
    };

    let result = run_performance_baseline(&client).await;

    assert!(result.is_ok());

    let data = result.unwrap();
    assert!(data["api_latency_ms"].is_number());
    assert!(data["timestamp"].is_string());
}
