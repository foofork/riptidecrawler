/// Real-world CLI tests against actual RipTide API server
///
/// Prerequisites:
/// 1. Start Redis: `docker run -d -p 6379:6379 redis:alpine`
/// 2. Start API: `cargo run --bin riptide-api`
/// 3. Run tests: `cargo test --test real_api_tests -- --test-threads=1`

use assert_cmd::Command;
use predicates::prelude::*;

const API_URL: &str = "http://localhost:8080";

#[tokio::test]
#[ignore] // Run with: cargo test --test real_api_tests -- --ignored
async fn test_cli_health_check() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(API_URL)
        .arg("health")
        .assert()
        .success()
        .stdout(predicate::str::contains("System"));
}

#[tokio::test]
#[ignore]
async fn test_extract_wikipedia() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(API_URL)
        .arg("extract")
        .arg("--url")
        .arg("https://en.wikipedia.org/wiki/Rust_(programming_language)")
        .arg("--show-confidence")
        .assert()
        .success()
        .stdout(predicate::str::contains("Rust"));
}

#[tokio::test]
#[ignore]
async fn test_extract_github_readme() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(API_URL)
        .arg("extract")
        .arg("--url")
        .arg("https://github.com/rust-lang/rust/blob/master/README.md")
        .arg("--method")
        .arg("trek")
        .assert()
        .success()
        .stdout(predicate::str::contains("README"));
}

#[tokio::test]
#[ignore]
async fn test_extract_with_confidence_scoring() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(API_URL)
        .arg("extract")
        .arg("--url")
        .arg("https://example.com")
        .arg("--show-confidence")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("confidence"));
}

#[tokio::test]
#[ignore]
async fn test_cache_status() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(API_URL)
        .arg("cache")
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Cache"));
}

#[tokio::test]
#[ignore]
async fn test_wasm_info() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(API_URL)
        .arg("wasm")
        .arg("info")
        .assert()
        .success()
        .stdout(predicate::str::contains("WASM"));
}

#[tokio::test]
#[ignore]
async fn test_metrics() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(API_URL)
        .arg("metrics")
        .assert()
        .success();
}

#[tokio::test]
#[ignore]
async fn test_validate() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(API_URL)
        .arg("validate")
        .assert()
        .success()
        .stdout(predicate::str::contains("validation"));
}

#[tokio::test]
#[ignore]
async fn test_system_check() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(API_URL)
        .arg("system-check")
        .assert()
        .success();
}

#[tokio::test]
#[ignore]
async fn test_extract_with_strategy_composition() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(API_URL)
        .arg("extract")
        .arg("--url")
        .arg("https://example.com")
        .arg("--strategy")
        .arg("chain:trek,css")
        .arg("--show-confidence")
        .assert()
        .success();
}

#[tokio::test]
#[ignore]
async fn test_extract_save_to_file() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(API_URL)
        .arg("extract")
        .arg("--url")
        .arg("https://example.com")
        .arg("--file")
        .arg("/tmp/riptide_test_output.txt")
        .assert()
        .success();

    // Verify file was created
    assert!(std::path::Path::new("/tmp/riptide_test_output.txt").exists());
    std::fs::remove_file("/tmp/riptide_test_output.txt").ok();
}

#[tokio::test]
#[ignore]
async fn test_output_formats() {
    // Test JSON output
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(API_URL)
        .arg("--output")
        .arg("json")
        .arg("health")
        .assert()
        .success()
        .stdout(predicate::str::contains("{"));

    // Test table output
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-url")
        .arg(API_URL)
        .arg("--output")
        .arg("table")
        .arg("health")
        .assert()
        .success();
}
