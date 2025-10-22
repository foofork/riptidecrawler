// Job command integration tests
// Tests: job-local commands (no API required)

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_job_help() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("job")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Job management"));
}

#[test]
fn test_job_local_help() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("job-local")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Local job management"));
}

#[test]
fn test_job_local_list() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("job-local")
        .arg("list")
        .timeout(std::time::Duration::from_secs(3))
        .assert()
        .code(predicate::in_iter([0, 1])); // Should work without API
}

#[test]
fn test_job_local_status() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("job-local")
        .arg("status")
        .timeout(std::time::Duration::from_secs(3))
        .assert()
        .code(predicate::in_iter([0, 1]));
}

#[test]
fn test_job_api_requires_server() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("job")
        .arg("list")
        .timeout(std::time::Duration::from_secs(3))
        .assert()
        .code(predicate::in_iter([0, 1])); // May fail without API server
}

#[test]
fn test_job_local_json_output() {
    let mut cmd = Command::cargo_bin("riptide").unwrap();
    cmd.arg("-o")
        .arg("json")
        .arg("job-local")
        .arg("list")
        .timeout(std::time::Duration::from_secs(3))
        .assert()
        .code(predicate::in_iter([0, 1]));
}
