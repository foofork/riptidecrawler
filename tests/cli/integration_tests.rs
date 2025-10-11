use assert_cmd::Command;
use predicates::prelude::*;
use std::env;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

#[tokio::test]
async fn test_cli_help_displays() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("RipTide - High-performance web crawler"));
}

#[tokio::test]
async fn test_cli_version_displays() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("riptide"));
}

#[tokio::test]
async fn test_extract_command_basic() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "content": "Test content",
            "method_used": "trek",
            "extraction_time_ms": 100
        })))
        .mount(&mock_server)
        .await;

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("extract")
        .arg("--url")
        .arg("https://example.com")
        .assert()
        .success();
}

#[tokio::test]
async fn test_extract_with_confidence_scoring() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "content": "Test content",
            "confidence": 0.95,
            "method_used": "trek",
            "extraction_time_ms": 100
        })))
        .mount(&mock_server)
        .await;

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("extract")
        .arg("--url")
        .arg("https://example.com")
        .arg("--show-confidence")
        .assert()
        .success()
        .stdout(predicate::str::contains("Confidence"));
}

#[tokio::test]
async fn test_extract_with_strategy_chain() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "content": "Test content",
            "method_used": "chain:trek,css",
            "extraction_time_ms": 150
        })))
        .mount(&mock_server)
        .await;

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("extract")
        .arg("--url")
        .arg("https://example.com")
        .arg("--strategy")
        .arg("chain:trek,css")
        .assert()
        .success();
}

#[tokio::test]
async fn test_cache_status_command() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/admin/cache/stats"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "total_keys": 1000,
            "memory_used": 50000000,
            "hit_rate": 0.85
        })))
        .mount(&mock_server)
        .await;

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("cache")
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Total Keys"));
}

#[tokio::test]
async fn test_wasm_info_command() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/monitoring/wasm-instances"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "version": "1.0.0",
            "instances": 4,
            "memory_usage": 100000000,
            "features": ["simd", "threads"]
        })))
        .mount(&mock_server)
        .await;

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("wasm")
        .arg("info")
        .assert()
        .success()
        .stdout(predicate::str::contains("WASM"));
}

#[tokio::test]
async fn test_health_command() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/health/detailed"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "healthy",
            "healthy": true,
            "redis": "connected",
            "extractor": "ready",
            "http_client": "ready",
            "worker_service": "ready",
            "uptime_seconds": 3600
        })))
        .mount(&mock_server)
        .await;

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("health")
        .assert()
        .success()
        .stdout(predicate::str::contains("System is healthy"));
}

#[tokio::test]
async fn test_validate_command_success() {
    let mock_server = MockServer::start().await;

    // Mock health check
    Mock::given(method("GET"))
        .and(path("/healthz"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/health/detailed"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "redis": "connected"
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/monitoring/wasm-instances"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/workers/stats/workers"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("validate")
        .assert()
        .success()
        .stdout(predicate::str::contains("All validation checks passed"));
}

#[tokio::test]
async fn test_output_formats() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/health/detailed"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "healthy",
            "healthy": true,
            "redis": "connected",
            "extractor": "ready",
            "http_client": "ready",
            "worker_service": "ready"
        })))
        .mount(&mock_server)
        .await;

    // Test JSON output
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("--output")
        .arg("json")
        .arg("health")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\""));

    // Test table output
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("--output")
        .arg("table")
        .arg("health")
        .assert()
        .success();
}

#[tokio::test]
async fn test_api_key_authentication() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/healthz"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("--api-key")
        .arg("test-api-key")
        .arg("health")
        .assert()
        .success();
}

#[tokio::test]
async fn test_error_handling() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/extract"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal server error"))
        .mount(&mock_server)
        .await;

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(mock_server.uri())
        .arg("extract")
        .arg("--url")
        .arg("https://example.com")
        .assert()
        .failure();
}
