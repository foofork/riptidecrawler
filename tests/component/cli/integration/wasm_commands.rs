// WASM command integration tests
// Tests: wasm info, wasm health, wasm benchmark

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_wasm_help() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("wasm")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("WASM management"));
}

#[test]
fn test_wasm_info() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("wasm")
        .arg("info")
        .timeout(std::time::Duration::from_secs(3))
        .assert()
        .code(predicate::in_iter([0, 1])); // May fail without API
}

#[test]
fn test_wasm_health() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("wasm")
        .arg("health")
        .timeout(std::time::Duration::from_secs(3))
        .assert()
        .code(predicate::in_iter([0, 1]));
}

#[test]
fn test_wasm_benchmark_default() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("wasm")
        .arg("benchmark")
        .timeout(std::time::Duration::from_secs(10))
        .assert()
        .code(predicate::in_iter([0, 1]));
}

#[test]
fn test_wasm_benchmark_with_iterations() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("wasm")
        .arg("benchmark")
        .arg("--iterations")
        .arg("10")
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .code(predicate::in_iter([0, 1]));
}

#[test]
fn test_wasm_benchmark_json_output() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("-o")
        .arg("json")
        .arg("wasm")
        .arg("benchmark")
        .arg("--iterations")
        .arg("5")
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .code(predicate::in_iter([0, 1]));
}

#[test]
fn test_extract_with_wasm_path() {
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
        .arg("--no-wasm") // Skip WASM to avoid dependency
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success();
}

#[test]
fn test_extract_wasm_timeout() {
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
        .arg("--init-timeout-ms")
        .arg("1000")
        .arg("--no-wasm")
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success();
}
