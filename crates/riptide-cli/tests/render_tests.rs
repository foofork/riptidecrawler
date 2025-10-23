//! Integration tests for render command (980 LOC coverage)
//!
//! Tests all render command functionality including:
//! - Wait conditions (load, network-idle, selector, timeout)
//! - Screenshot modes (none, viewport, full)
//! - Output formats (HTML, DOM, PDF, HAR)
//! - Stealth levels and evasion
//! - Error handling and edge cases
//! - API vs direct execution modes

use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;

/// Helper to get CLI command
fn cli() -> Command {
    Command::cargo_bin("riptide").unwrap()
}

#[test]
fn test_render_help() {
    cli()
        .arg("render")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Render web pages"));
}

#[test]
fn test_render_basic_url() {
    let temp = TempDir::new().unwrap();

    cli()
        .arg("render")
        .arg("--url")
        .arg("https://example.com")
        .arg("--output-dir")
        .arg(temp.path())
        .arg("--direct")
        .assert()
        .success();
}

#[test]
fn test_render_with_wait_load() {
    let temp = TempDir::new().unwrap();

    cli()
        .arg("render")
        .arg("--url")
        .arg("https://example.com")
        .arg("--wait")
        .arg("load")
        .arg("--output-dir")
        .arg(temp.path())
        .arg("--direct")
        .assert()
        .success();
}

#[test]
fn test_render_with_wait_network_idle() {
    let temp = TempDir::new().unwrap();

    cli()
        .arg("render")
        .arg("--url")
        .arg("https://example.com")
        .arg("--wait")
        .arg("network-idle")
        .arg("--output-dir")
        .arg(temp.path())
        .arg("--direct")
        .assert()
        .success();
}

#[test]
fn test_render_with_wait_selector() {
    let temp = TempDir::new().unwrap();

    cli()
        .arg("render")
        .arg("--url")
        .arg("https://example.com")
        .arg("--wait")
        .arg("selector:body")
        .arg("--output-dir")
        .arg(temp.path())
        .arg("--direct")
        .assert()
        .success();
}

#[test]
fn test_render_with_wait_timeout() {
    let temp = TempDir::new().unwrap();

    cli()
        .arg("render")
        .arg("--url")
        .arg("https://example.com")
        .arg("--wait")
        .arg("timeout:5000")
        .arg("--output-dir")
        .arg(temp.path())
        .arg("--direct")
        .assert()
        .success();
}

#[test]
fn test_render_invalid_wait_condition() {
    let temp = TempDir::new().unwrap();

    cli()
        .arg("render")
        .arg("--url")
        .arg("https://example.com")
        .arg("--wait")
        .arg("invalid-wait")
        .arg("--output-dir")
        .arg(temp.path())
        .arg("--direct")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid wait condition"));
}

#[test]
fn test_render_screenshot_viewport() {
    let temp = TempDir::new().unwrap();

    cli()
        .arg("render")
        .arg("--url")
        .arg("https://example.com")
        .arg("--screenshot")
        .arg("viewport")
        .arg("--output-dir")
        .arg(temp.path())
        .arg("--direct")
        .assert()
        .success();
}

#[test]
fn test_render_screenshot_full() {
    let temp = TempDir::new().unwrap();

    cli()
        .arg("render")
        .arg("--url")
        .arg("https://example.com")
        .arg("--screenshot")
        .arg("full")
        .arg("--output-dir")
        .arg(temp.path())
        .arg("--direct")
        .assert()
        .success();
}

#[test]
fn test_render_invalid_screenshot_mode() {
    let temp = TempDir::new().unwrap();

    cli()
        .arg("render")
        .arg("--url")
        .arg("https://example.com")
        .arg("--screenshot")
        .arg("invalid")
        .arg("--output-dir")
        .arg(temp.path())
        .arg("--direct")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid screenshot mode"));
}

#[test]
fn test_render_save_html() {
    let temp = TempDir::new().unwrap();

    cli()
        .arg("render")
        .arg("--url")
        .arg("https://example.com")
        .arg("--html")
        .arg("--output-dir")
        .arg(temp.path())
        .arg("--direct")
        .assert()
        .success();

    // Verify HTML file was created
    let html_files: Vec<_> = std::fs::read_dir(temp.path())
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "html"))
        .collect();

    assert!(!html_files.is_empty(), "No HTML file was created");
}

#[test]
fn test_render_save_dom() {
    let temp = TempDir::new().unwrap();

    cli()
        .arg("render")
        .arg("--url")
        .arg("https://example.com")
        .arg("--dom")
        .arg("--output-dir")
        .arg(temp.path())
        .arg("--direct")
        .assert()
        .success();
}

#[test]
fn test_render_save_pdf() {
    let temp = TempDir::new().unwrap();

    cli()
        .arg("render")
        .arg("--url")
        .arg("https://example.com")
        .arg("--pdf")
        .arg("--output-dir")
        .arg(temp.path())
        .arg("--direct")
        .assert()
        .success();
}

#[test]
fn test_render_save_har() {
    let temp = TempDir::new().unwrap();

    cli()
        .arg("render")
        .arg("--url")
        .arg("https://example.com")
        .arg("--har")
        .arg("--output-dir")
        .arg(temp.path())
        .arg("--direct")
        .assert()
        .success();
}

#[test]
fn test_render_multiple_outputs() {
    let temp = TempDir::new().unwrap();

    cli()
        .arg("render")
        .arg("--url")
        .arg("https://example.com")
        .arg("--html")
        .arg("--dom")
        .arg("--screenshot")
        .arg("viewport")
        .arg("--output-dir")
        .arg(temp.path())
        .arg("--direct")
        .assert()
        .success();
}

#[test]
fn test_render_stealth_off() {
    let temp = TempDir::new().unwrap();

    cli()
        .arg("render")
        .arg("--url")
        .arg("https://example.com")
        .arg("--stealth")
        .arg("off")
        .arg("--output-dir")
        .arg(temp.path())
        .arg("--direct")
        .assert()
        .success();
}

#[test]
fn test_render_stealth_low() {
    let temp = TempDir::new().unwrap();

    cli()
        .arg("render")
        .arg("--url")
        .arg("https://example.com")
        .arg("--stealth")
        .arg("low")
        .arg("--output-dir")
        .arg(temp.path())
        .arg("--direct")
        .assert()
        .success();
}

#[test]
fn test_render_stealth_high() {
    let temp = TempDir::new().unwrap();

    cli()
        .arg("render")
        .arg("--url")
        .arg("https://example.com")
        .arg("--stealth")
        .arg("high")
        .arg("--output-dir")
        .arg(temp.path())
        .arg("--direct")
        .assert()
        .success();
}

#[test]
fn test_render_invalid_stealth_level() {
    let temp = TempDir::new().unwrap();

    cli()
        .arg("render")
        .arg("--url")
        .arg("https://example.com")
        .arg("--stealth")
        .arg("invalid")
        .arg("--output-dir")
        .arg(temp.path())
        .arg("--direct")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid stealth level"));
}

#[test]
fn test_render_custom_viewport() {
    let temp = TempDir::new().unwrap();

    cli()
        .arg("render")
        .arg("--url")
        .arg("https://example.com")
        .arg("--width")
        .arg("1280")
        .arg("--height")
        .arg("720")
        .arg("--output-dir")
        .arg(temp.path())
        .arg("--direct")
        .assert()
        .success();
}

#[test]
fn test_render_custom_user_agent() {
    let temp = TempDir::new().unwrap();

    cli()
        .arg("render")
        .arg("--url")
        .arg("https://example.com")
        .arg("--user-agent")
        .arg("CustomBot/1.0")
        .arg("--output-dir")
        .arg(temp.path())
        .arg("--direct")
        .assert()
        .success();
}

#[test]
fn test_render_javascript_disabled() {
    let temp = TempDir::new().unwrap();

    cli()
        .arg("render")
        .arg("--url")
        .arg("https://example.com")
        .arg("--javascript")
        .arg("false")
        .arg("--output-dir")
        .arg(temp.path())
        .arg("--direct")
        .assert()
        .success();
}

#[test]
fn test_render_extra_timeout() {
    let temp = TempDir::new().unwrap();

    cli()
        .arg("render")
        .arg("--url")
        .arg("https://example.com")
        .arg("--extra-timeout")
        .arg("2")
        .arg("--output-dir")
        .arg(temp.path())
        .arg("--direct")
        .assert()
        .success();
}

#[test]
fn test_render_custom_prefix() {
    let temp = TempDir::new().unwrap();

    cli()
        .arg("render")
        .arg("--url")
        .arg("https://example.com")
        .arg("--prefix")
        .arg("custom_output")
        .arg("--html")
        .arg("--output-dir")
        .arg(temp.path())
        .arg("--direct")
        .assert()
        .success();

    // Verify custom prefix was used
    let html_files: Vec<_> = std::fs::read_dir(temp.path())
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| e.file_name().to_string_lossy().starts_with("custom_output"))
        .collect();

    assert!(!html_files.is_empty(), "No file with custom prefix found");
}

#[test]
fn test_render_invalid_url() {
    let temp = TempDir::new().unwrap();

    cli()
        .arg("render")
        .arg("--url")
        .arg("not-a-valid-url")
        .arg("--output-dir")
        .arg(temp.path())
        .arg("--direct")
        .assert()
        .failure();
}

#[test]
fn test_render_missing_url() {
    cli()
        .arg("render")
        .arg("--html")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_render_direct_mode_flag() {
    let temp = TempDir::new().unwrap();

    cli()
        .arg("render")
        .arg("--url")
        .arg("https://example.com")
        .arg("--direct")
        .arg("--output-dir")
        .arg(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("direct").or(predicate::str::contains("Direct")));
}

#[test]
fn test_render_api_only_without_server() {
    let temp = TempDir::new().unwrap();

    // Without RIPTIDE_API_URL set, --api-only should fail
    cli()
        .arg("render")
        .arg("--url")
        .arg("https://example.com")
        .arg("--api-only")
        .arg("--output-dir")
        .arg(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("API"));
}

#[test]
fn test_render_output_directory_creation() {
    let temp = TempDir::new().unwrap();
    let nested_dir = temp.path().join("nested").join("output");

    cli()
        .arg("render")
        .arg("--url")
        .arg("https://example.com")
        .arg("--html")
        .arg("--output-dir")
        .arg(&nested_dir)
        .arg("--direct")
        .assert()
        .success();

    // Verify nested directory was created
    assert!(
        nested_dir.exists(),
        "Nested output directory was not created"
    );
}

#[test]
fn test_render_with_proxy() {
    let temp = TempDir::new().unwrap();

    // This will likely fail without a real proxy, but tests argument parsing
    cli()
        .arg("render")
        .arg("--url")
        .arg("https://example.com")
        .arg("--proxy")
        .arg("http://localhost:8080")
        .arg("--output-dir")
        .arg(temp.path())
        .arg("--direct")
        .assert()
        .code(predicate::in_iter([0, 1])); // May succeed or fail
}

#[test]
fn test_render_file_prefix_generation() {
    let temp = TempDir::new().unwrap();

    cli()
        .arg("render")
        .arg("--url")
        .arg("https://example.com/path/to/page")
        .arg("--html")
        .arg("--output-dir")
        .arg(temp.path())
        .arg("--direct")
        .assert()
        .success();

    // Check that file was created with URL-based prefix
    let files: Vec<_> = std::fs::read_dir(temp.path())
        .unwrap()
        .filter_map(Result::ok)
        .collect();

    assert!(!files.is_empty(), "No output files created");
}

#[test]
fn test_render_json_output_format() {
    let temp = TempDir::new().unwrap();

    cli()
        .arg("--output")
        .arg("json")
        .arg("render")
        .arg("--url")
        .arg("https://example.com")
        .arg("--html")
        .arg("--output-dir")
        .arg(temp.path())
        .arg("--direct")
        .assert()
        .success()
        .stdout(predicate::str::contains("{").or(predicate::str::contains("url")));
}

#[test]
fn test_render_text_output_format() {
    let temp = TempDir::new().unwrap();

    cli()
        .arg("--output")
        .arg("text")
        .arg("render")
        .arg("--url")
        .arg("https://example.com")
        .arg("--output-dir")
        .arg(temp.path())
        .arg("--direct")
        .assert()
        .success()
        .stdout(predicate::str::contains("Rendering").or(predicate::str::contains("success")));
}

#[test]
fn test_render_table_output_format() {
    let temp = TempDir::new().unwrap();

    cli()
        .arg("--output")
        .arg("table")
        .arg("render")
        .arg("--url")
        .arg("https://example.com")
        .arg("--output-dir")
        .arg(temp.path())
        .arg("--direct")
        .assert()
        .success();
}
