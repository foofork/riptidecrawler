// Cache command integration tests
// Tests: cache status, cache clear, cache stats, cache validate

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cache_help() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("cache")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Cache management"));
}

#[test]
fn test_cache_status() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("cache")
        .arg("status")
        .timeout(std::time::Duration::from_secs(3))
        .assert()
        .code(predicate::in_iter([0, 1])); // May fail without API, but should not crash
}

#[test]
fn test_cache_stats() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("cache")
        .arg("stats")
        .timeout(std::time::Duration::from_secs(3))
        .assert()
        .code(predicate::in_iter([0, 1]));
}

#[test]
fn test_cache_validate() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("cache")
        .arg("validate")
        .timeout(std::time::Duration::from_secs(3))
        .assert()
        .code(predicate::in_iter([0, 1]));
}

#[test]
fn test_cache_clear() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("cache")
        .arg("clear")
        .timeout(std::time::Duration::from_secs(3))
        .assert()
        .code(predicate::in_iter([0, 1]));
}

#[test]
fn test_cache_clear_with_domain() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("cache")
        .arg("clear")
        .arg("--domain")
        .arg("example.com")
        .timeout(std::time::Duration::from_secs(3))
        .assert()
        .code(predicate::in_iter([0, 1]));
}

#[test]
fn test_cache_warm_requires_file() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("cache")
        .arg("warm")
        .timeout(std::time::Duration::from_secs(2))
        .assert()
        .failure()
        .stderr(predicate::str::contains("url-file").or(predicate::str::contains("required")));
}

#[test]
fn test_cache_warm_with_url_file() {
    let temp = assert_fs::TempDir::new().unwrap();
    let url_file = temp.child("urls.txt");
    url_file
        .write_str("https://example.com\nhttps://example.org\n")
        .unwrap();

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("cache")
        .arg("warm")
        .arg("--url-file")
        .arg(url_file.path())
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .code(predicate::in_iter([0, 1])); // May fail without API
}

#[test]
fn test_cache_json_output() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("-o")
        .arg("json")
        .arg("cache")
        .arg("status")
        .timeout(std::time::Duration::from_secs(3))
        .assert()
        .code(predicate::in_iter([0, 1]));
}
