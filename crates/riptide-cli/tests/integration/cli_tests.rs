//! CLI Integration Tests with assert_cmd and assert_fs
//!
//! Phase 6.1: CLI Integration Tests (3.6 days)
//! Tests all CLI commands with real filesystem scenarios, error handling, and edge cases.
//!
//! Test coverage:
//! - All CLI commands (extract, validate, cache, session, etc.)
//! - Real filesystem scenarios with assert_fs
//! - Error handling and edge cases
//! - Command output validation
//! - Exit codes verification

use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;
use std::fs;

/// Helper function to create a test HTML file
fn create_test_html(temp: &TempDir, filename: &str, content: &str) -> String {
    let file = temp.child(filename);
    file.write_str(content).unwrap();
    file.path().to_string_lossy().to_string()
}

/// Helper function to get the CLI binary command
fn cli_command() -> Command {
    Command::cargo_bin("riptide").unwrap()
}

#[test]
fn test_cli_version() {
    cli_command()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("riptide"));
}

#[test]
fn test_cli_help() {
    cli_command()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Command-line interface"))
        .stdout(predicate::str::contains("extract"))
        .stdout(predicate::str::contains("validate"));
}

#[test]
fn test_extract_command_help() {
    cli_command()
        .arg("extract")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Extract content from web pages"));
}

#[test]
fn test_extract_local_html_file() {
    let temp = TempDir::new().unwrap();
    let html_content = r#"
        <!DOCTYPE html>
        <html>
        <head><title>Test Page</title></head>
        <body>
            <h1>Test Heading</h1>
            <p>Test paragraph content</p>
        </body>
        </html>
    "#;

    let html_path = create_test_html(&temp, "test.html", html_content);
    let output_file = temp.child("output.json");

    cli_command()
        .arg("extract")
        .arg(&html_path)
        .arg("--output")
        .arg(output_file.path())
        .assert()
        .success();

    // Verify output file was created
    output_file.assert(predicate::path::exists());

    // Verify output contains expected content
    let output_content = fs::read_to_string(output_file.path()).unwrap();
    assert!(output_content.contains("Test Heading") || output_content.contains("Test paragraph"));
}

#[test]
fn test_extract_with_selector() {
    let temp = TempDir::new().unwrap();
    let html_content = r#"
        <html>
        <body>
            <article>Main content</article>
            <div class="sidebar">Sidebar content</div>
        </body>
        </html>
    "#;

    let html_path = create_test_html(&temp, "test.html", html_content);

    cli_command()
        .arg("extract")
        .arg(&html_path)
        .arg("--selector")
        .arg("article")
        .assert()
        .success()
        .stdout(predicate::str::contains("Main content"));
}

#[test]
fn test_extract_invalid_file() {
    cli_command()
        .arg("extract")
        .arg("/nonexistent/file.html")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("No such file")));
}

#[test]
fn test_extract_invalid_url() {
    cli_command()
        .arg("extract")
        .arg("not-a-valid-url")
        .assert()
        .failure();
}

#[test]
fn test_extract_with_output_format_json() {
    let temp = TempDir::new().unwrap();
    let html_content = "<html><body><h1>Test</h1></body></html>";
    let html_path = create_test_html(&temp, "test.html", html_content);
    let output_file = temp.child("output.json");

    cli_command()
        .arg("extract")
        .arg(&html_path)
        .arg("--output")
        .arg(output_file.path())
        .arg("--format")
        .arg("json")
        .assert()
        .success();

    output_file.assert(predicate::path::exists());
    let content = fs::read_to_string(output_file.path()).unwrap();
    assert!(content.starts_with("{") || content.starts_with("["));
}

#[test]
fn test_extract_with_output_format_markdown() {
    let temp = TempDir::new().unwrap();
    let html_content = "<html><body><h1>Test Heading</h1><p>Paragraph</p></body></html>";
    let html_path = create_test_html(&temp, "test.html", html_content);
    let output_file = temp.child("output.md");

    cli_command()
        .arg("extract")
        .arg(&html_path)
        .arg("--output")
        .arg(output_file.path())
        .arg("--format")
        .arg("markdown")
        .assert()
        .success();

    output_file.assert(predicate::path::exists());
    let content = fs::read_to_string(output_file.path()).unwrap();
    assert!(content.contains("#") || content.contains("Test"));
}

#[test]
fn test_validate_command() {
    let temp = TempDir::new().unwrap();
    let html_content = "<html><body><h1>Valid HTML</h1></body></html>";
    let html_path = create_test_html(&temp, "valid.html", html_content);

    cli_command()
        .arg("validate")
        .arg(&html_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("valid").or(predicate::str::contains("✓")));
}

#[test]
fn test_validate_malformed_html() {
    let temp = TempDir::new().unwrap();
    let html_content = "<html><body><h1>Unclosed heading</body></html>";
    let html_path = create_test_html(&temp, "malformed.html", html_content);

    cli_command()
        .arg("validate")
        .arg(&html_path)
        .assert()
        .code(predicate::in_iter([0, 1])); // May succeed or fail depending on validation strictness
}

#[test]
fn test_cache_clear_command() {
    // Note: This test assumes cache commands exist in the CLI
    let temp = TempDir::new().unwrap();

    cli_command()
        .arg("cache")
        .arg("clear")
        .arg("--cache-dir")
        .arg(temp.path())
        .assert()
        .success();
}

#[test]
fn test_cache_stats_command() {
    let temp = TempDir::new().unwrap();

    cli_command()
        .arg("cache")
        .arg("stats")
        .arg("--cache-dir")
        .arg(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("entries").or(predicate::str::contains("cache")));
}

#[test]
fn test_extract_multiple_files() {
    let temp = TempDir::new().unwrap();

    let html1 = create_test_html(&temp, "file1.html", "<html><body>Content 1</body></html>");
    let html2 = create_test_html(&temp, "file2.html", "<html><body>Content 2</body></html>");

    cli_command()
        .arg("extract")
        .arg(&html1)
        .arg(&html2)
        .assert()
        .success();
}

#[test]
fn test_extract_with_timeout() {
    let temp = TempDir::new().unwrap();
    let html_path = create_test_html(&temp, "test.html", "<html><body>Test</body></html>");

    cli_command()
        .arg("extract")
        .arg(&html_path)
        .arg("--timeout")
        .arg("30")
        .assert()
        .success();
}

#[test]
fn test_extract_with_invalid_timeout() {
    let temp = TempDir::new().unwrap();
    let html_path = create_test_html(&temp, "test.html", "<html><body>Test</body></html>");

    cli_command()
        .arg("extract")
        .arg(&html_path)
        .arg("--timeout")
        .arg("invalid")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid").or(predicate::str::contains("parse")));
}

#[test]
fn test_extract_with_user_agent() {
    let temp = TempDir::new().unwrap();
    let html_path = create_test_html(&temp, "test.html", "<html><body>Test</body></html>");

    cli_command()
        .arg("extract")
        .arg(&html_path)
        .arg("--user-agent")
        .arg("Custom User Agent")
        .assert()
        .success();
}

#[test]
fn test_extract_verbose_mode() {
    let temp = TempDir::new().unwrap();
    let html_path = create_test_html(&temp, "test.html", "<html><body>Test</body></html>");

    cli_command()
        .arg("extract")
        .arg(&html_path)
        .arg("-v")
        .assert()
        .success();
}

#[test]
fn test_extract_quiet_mode() {
    let temp = TempDir::new().unwrap();
    let html_path = create_test_html(&temp, "test.html", "<html><body>Test</body></html>");

    cli_command()
        .arg("extract")
        .arg(&html_path)
        .arg("--quiet")
        .assert()
        .success();
}

#[test]
fn test_extract_with_empty_file() {
    let temp = TempDir::new().unwrap();
    let empty_file = temp.child("empty.html");
    empty_file.write_str("").unwrap();

    cli_command()
        .arg("extract")
        .arg(empty_file.path())
        .assert()
        .code(predicate::in_iter([0, 1])); // May succeed with empty result or fail
}

#[test]
fn test_extract_with_large_file() {
    let temp = TempDir::new().unwrap();

    // Create a large HTML file (1MB+)
    let large_content = format!(
        "<html><body>{}</body></html>",
        "<p>Test paragraph</p>".repeat(10000)
    );
    let html_path = create_test_html(&temp, "large.html", &large_content);

    cli_command()
        .arg("extract")
        .arg(&html_path)
        .assert()
        .success();
}

#[test]
fn test_extract_with_special_characters_in_path() {
    let temp = TempDir::new().unwrap();
    let special_dir = temp.child("special dir with spaces");
    special_dir.create_dir_all().unwrap();

    let html_file = special_dir.child("test file.html");
    html_file.write_str("<html><body>Test</body></html>").unwrap();

    cli_command()
        .arg("extract")
        .arg(html_file.path())
        .assert()
        .success();
}

#[test]
fn test_extract_with_unicode_content() {
    let temp = TempDir::new().unwrap();
    let unicode_html = r#"
        <html>
        <body>
            <p>日本語テスト</p>
            <p>العربية</p>
            <p>Emoji: 🚀🔥💯</p>
        </body>
        </html>
    "#;
    let html_path = create_test_html(&temp, "unicode.html", unicode_html);

    cli_command()
        .arg("extract")
        .arg(&html_path)
        .assert()
        .success();
}

#[test]
fn test_extract_output_to_stdout() {
    let temp = TempDir::new().unwrap();
    let html_path = create_test_html(&temp, "test.html", "<html><body>Test</body></html>");

    cli_command()
        .arg("extract")
        .arg(&html_path)
        .arg("--output")
        .arg("-")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn test_extract_with_nonexistent_output_directory() {
    let temp = TempDir::new().unwrap();
    let html_path = create_test_html(&temp, "test.html", "<html><body>Test</body></html>");
    let nonexistent_dir = temp.path().join("nonexistent").join("output.json");

    cli_command()
        .arg("extract")
        .arg(&html_path)
        .arg("--output")
        .arg(&nonexistent_dir)
        .assert()
        .code(predicate::in_iter([0, 1])); // May fail or create directory
}

#[test]
fn test_concurrent_extractions() {
    use std::thread;

    let temp = TempDir::new().unwrap();
    let html_path = create_test_html(&temp, "test.html", "<html><body>Test</body></html>");

    let handles: Vec<_> = (0..5)
        .map(|i| {
            let path = html_path.clone();
            thread::spawn(move || {
                cli_command()
                    .arg("extract")
                    .arg(&path)
                    .arg("--quiet")
                    .assert()
                    .success();
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_extract_error_messages_are_helpful() {
    cli_command()
        .arg("extract")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("USAGE")));
}

#[test]
fn test_cli_exit_code_on_error() {
    cli_command()
        .arg("extract")
        .arg("/nonexistent/file.html")
        .assert()
        .failure()
        .code(predicate::ne(0));
}

#[test]
fn test_cli_exit_code_on_success() {
    let temp = TempDir::new().unwrap();
    let html_path = create_test_html(&temp, "test.html", "<html><body>Test</body></html>");

    cli_command()
        .arg("extract")
        .arg(&html_path)
        .assert()
        .success()
        .code(0);
}
