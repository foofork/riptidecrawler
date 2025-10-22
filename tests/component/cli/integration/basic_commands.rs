// Basic CLI command integration tests
// Tests: extract, render, health, validate, system-check

use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("RipTide"))
        .stdout(predicate::str::contains("web crawler"));
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("riptide"));
}

#[test]
fn test_extract_help() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("extract")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Extract content"))
        .stdout(predicate::str::contains("--url"))
        .stdout(predicate::str::contains("--method"));
}

#[test]
fn test_extract_requires_input() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("extract")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("error")));
}

#[test]
fn test_extract_from_file() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("test.html");
    input_file
        .write_str("<html><body><h1>Test</h1><p>Content here</p></body></html>")
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
        .success();
}

#[test]
fn test_extract_with_output_json() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("input.html");
    input_file
        .write_str("<html><body><article><h1>Title</h1><p>Paragraph</p></article></body></html>")
        .unwrap();

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("-o")
        .arg("json")
        .arg("extract")
        .arg("--input-file")
        .arg(input_file.path())
        .arg("--local")
        .arg("--method")
        .arg("css")
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("{").or(predicate::str::contains("[")));
}

#[test]
fn test_extract_with_selector() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("input.html");
    input_file
        .write_str("<html><body><div class='content'>Target</div></body></html>")
        .unwrap();

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("extract")
        .arg("--input-file")
        .arg(input_file.path())
        .arg("--local")
        .arg("--method")
        .arg("css")
        .arg("--selector")
        .arg(".content")
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success();
}

#[test]
fn test_extract_with_confidence() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("input.html");
    input_file
        .write_str("<html><body><article><p>Test content</p></article></body></html>")
        .unwrap();

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("extract")
        .arg("--input-file")
        .arg(input_file.path())
        .arg("--local")
        .arg("--method")
        .arg("css")
        .arg("--show-confidence")
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success();
}

#[test]
fn test_extract_with_metadata() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("input.html");
    input_file
        .write_str("<html><head><title>Page Title</title></head><body><p>Content</p></body></html>")
        .unwrap();

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("extract")
        .arg("--input-file")
        .arg(input_file.path())
        .arg("--local")
        .arg("--metadata")
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success();
}

#[test]
fn test_health_command() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    // Health command checks API by default, so will fail without server
    // But should not panic or crash
    cmd.arg("health")
        .timeout(std::time::Duration::from_secs(3))
        .assert()
        .code(predicate::in_iter([0, 1])); // Either success or graceful failure
}

#[test]
fn test_validate_help() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("validate")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Validate"));
}

#[test]
fn test_system_check_help() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("system-check")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("system check"));
}

#[test]
fn test_tables_help() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("tables")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("table"))
        .stdout(predicate::str::contains("--format"));
}

#[test]
fn test_tables_from_file() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("table.html");
    input_file
        .write_str(
            r#"<html><body>
            <table>
                <tr><th>Name</th><th>Age</th></tr>
                <tr><td>Alice</td><td>30</td></tr>
                <tr><td>Bob</td><td>25</td></tr>
            </table>
        </body></html>"#,
        )
        .unwrap();

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("tables")
        .arg("--file")
        .arg(input_file.path())
        .arg("--format")
        .arg("markdown")
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success();
}

#[test]
fn test_output_format_text() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("input.html");
    input_file
        .write_str("<html><body><p>Test</p></body></html>")
        .unwrap();

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("-o")
        .arg("text")
        .arg("extract")
        .arg("--input-file")
        .arg(input_file.path())
        .arg("--local")
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success();
}

#[test]
fn test_verbose_flag() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("input.html");
    input_file
        .write_str("<html><body><p>Test</p></body></html>")
        .unwrap();

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("-v")
        .arg("extract")
        .arg("--input-file")
        .arg(input_file.path())
        .arg("--local")
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success();
}

#[test]
fn test_direct_mode_flag() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("input.html");
    input_file
        .write_str("<html><body><p>Test</p></body></html>")
        .unwrap();

    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("--direct")
        .arg("extract")
        .arg("--input-file")
        .arg(input_file.path())
        .arg("--local")
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success();
}
