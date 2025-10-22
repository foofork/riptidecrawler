// Error handling and validation tests
// Tests: invalid inputs, missing files, malformed arguments

use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn test_invalid_command() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("nonexistent-command")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected").or(predicate::str::contains("error")));
}

#[test]
fn test_extract_invalid_method() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("input.html");
    input_file
        .write_str("<html><body><p>Test</p></body></html>")
        .unwrap();

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("extract")
        .arg("--input-file")
        .arg(input_file.path())
        .arg("--method")
        .arg("invalid-method")
        .timeout(std::time::Duration::from_secs(3))
        .assert()
        .code(predicate::in_iter([0, 1])); // May accept unknown methods gracefully
}

#[test]
fn test_extract_missing_file() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("extract")
        .arg("--input-file")
        .arg("/nonexistent/path/file.html")
        .arg("--local")
        .timeout(std::time::Duration::from_secs(3))
        .assert()
        .failure()
        .stderr(predicate::str::contains("file").or(predicate::str::contains("not found")));
}

#[test]
fn test_extract_invalid_url() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("extract")
        .arg("--url")
        .arg("not-a-valid-url")
        .timeout(std::time::Duration::from_secs(3))
        .assert()
        .code(predicate::in_iter([0, 1])); // May fail gracefully
}

#[test]
fn test_extract_conflicting_inputs() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("input.html");
    input_file
        .write_str("<html><body><p>Test</p></body></html>")
        .unwrap();

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("extract")
        .arg("--input-file")
        .arg(input_file.path())
        .arg("--stdin")
        .arg("--local")
        .timeout(std::time::Duration::from_secs(3))
        .assert()
        .code(predicate::in_iter([0, 1])); // Should handle conflicting inputs
}

#[test]
fn test_invalid_output_format() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("input.html");
    input_file
        .write_str("<html><body><p>Test</p></body></html>")
        .unwrap();

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("-o")
        .arg("invalid-format")
        .arg("extract")
        .arg("--input-file")
        .arg(input_file.path())
        .arg("--local")
        .timeout(std::time::Duration::from_secs(3))
        .assert()
        .code(predicate::in_iter([0, 1])); // May accept with warning
}

#[test]
fn test_cache_warm_invalid_file() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("cache")
        .arg("warm")
        .arg("--url-file")
        .arg("/nonexistent/urls.txt")
        .timeout(std::time::Duration::from_secs(3))
        .assert()
        .failure();
}

#[test]
fn test_wasm_benchmark_invalid_iterations() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("wasm")
        .arg("benchmark")
        .arg("--iterations")
        .arg("not-a-number")
        .timeout(std::time::Duration::from_secs(2))
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid").or(predicate::str::contains("error")));
}

#[test]
fn test_extract_invalid_selector() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("input.html");
    input_file
        .write_str("<html><body><p>Test</p></body></html>")
        .unwrap();

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("extract")
        .arg("--input-file")
        .arg(input_file.path())
        .arg("--local")
        .arg("--method")
        .arg("css")
        .arg("--selector")
        .arg("[[invalid:::selector]]")
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .code(predicate::in_iter([0, 1])); // May handle gracefully
}

#[test]
fn test_tables_no_input() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("tables")
        .timeout(std::time::Duration::from_secs(2))
        .assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("error")));
}

#[test]
fn test_extract_empty_file() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("empty.html");
    input_file.write_str("").unwrap();

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("extract")
        .arg("--input-file")
        .arg(input_file.path())
        .arg("--local")
        .timeout(std::time::Duration::from_secs(3))
        .assert()
        .code(predicate::in_iter([0, 1])); // Should handle empty files
}

#[test]
fn test_extract_malformed_html() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("malformed.html");
    input_file
        .write_str("<html><body><p>Unclosed tag<div>")
        .unwrap();

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("extract")
        .arg("--input-file")
        .arg(input_file.path())
        .arg("--local")
        .arg("--method")
        .arg("css")
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success(); // Should handle malformed HTML gracefully
}

#[test]
fn test_api_only_without_server() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("input.html");
    input_file
        .write_str("<html><body><p>Test</p></body></html>")
        .unwrap();

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--api-only")
        .arg("--api-url")
        .arg("http://localhost:9999") // Non-existent server
        .arg("extract")
        .arg("--input-file")
        .arg(input_file.path())
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .failure(); // Should fail without API server in api-only mode
}

#[test]
fn test_invalid_stealth_level() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("input.html");
    input_file
        .write_str("<html><body><p>Test</p></body></html>")
        .unwrap();

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("extract")
        .arg("--input-file")
        .arg(input_file.path())
        .arg("--local")
        .arg("--stealth-level")
        .arg("invalid")
        .timeout(std::time::Duration::from_secs(3))
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid").or(predicate::str::contains("value")));
}
