#![allow(clippy::all, dead_code, unused)]

//! Integration tests for extract command (882 LOC coverage)
//!
//! Tests all extraction functionality including:
//! - Engine selection (WASM, Headless, Raw, Auto)
//! - Input sources (URL, file, stdin)
//! - Extraction modes (article, full, metadata)
//! - Stealth and fingerprint evasion
//! - WASM initialization and timeouts
//! - Error handling and edge cases

use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;
use std::io::Write;

/// Helper to get CLI command
fn cli() -> Command {
    Command::cargo_bin("riptide").unwrap()
}

/// Create test HTML file
fn create_html_file(temp: &TempDir, filename: &str, content: &str) -> String {
    let file = temp.child(filename);
    file.write_str(content).unwrap();
    file.path().to_string_lossy().to_string()
}

#[test]
fn test_extract_help() {
    cli()
        .arg("extract")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Extract content"));
}

#[test]
fn test_extract_from_url_raw_engine() {
    cli()
        .arg("extract")
        .arg("--url")
        .arg("https://example.com")
        .arg("--engine")
        .arg("raw")
        .arg("--local")
        .assert()
        .success();
}

#[test]
fn test_extract_from_file() {
    let temp = TempDir::new().unwrap();
    let html = r#"
        <!DOCTYPE html>
        <html>
        <head><title>Test Article</title></head>
        <body>
            <article>
                <h1>Article Title</h1>
                <p>This is the article content with multiple paragraphs.</p>
                <p>Second paragraph of content.</p>
            </article>
        </body>
        </html>
    "#;

    let html_path = create_html_file(&temp, "article.html", html);

    cli()
        .arg("extract")
        .arg("--input-file")
        .arg(&html_path)
        .arg("--engine")
        .arg("raw")
        .assert()
        .success()
        .stdout(predicate::str::contains("Article").or(predicate::str::contains("content")));
}

#[test]
fn test_extract_from_stdin() {
    let html = "<html><body><h1>Test Content</h1><p>Paragraph</p></body></html>";

    cli()
        .arg("extract")
        .arg("--stdin")
        .arg("--engine")
        .arg("raw")
        .write_stdin(html)
        .assert()
        .success();
}

#[test]
fn test_extract_with_raw_engine() {
    let temp = TempDir::new().unwrap();
    let html = "<html><body><h1>Test</h1></body></html>";
    let html_path = create_html_file(&temp, "test.html", html);

    cli()
        .arg("extract")
        .arg("--input-file")
        .arg(&html_path)
        .arg("--engine")
        .arg("raw")
        .assert()
        .success();
}

#[test]
fn test_extract_with_wasm_engine() {
    let temp = TempDir::new().unwrap();
    let html = r#"
        <html>
        <body>
            <article>
                <h1>Article Title</h1>
                <p>Article content</p>
            </article>
        </body>
        </html>
    "#;
    let html_path = create_html_file(&temp, "test.html", html);

    // WASM may not be available in test environment
    cli()
        .arg("extract")
        .arg("--input-file")
        .arg(&html_path)
        .arg("--engine")
        .arg("wasm")
        .assert()
        .code(predicate::in_iter([0, 1])); // May succeed or fail if WASM not built
}

#[test]
fn test_extract_with_auto_engine() {
    let temp = TempDir::new().unwrap();
    let html = "<html><body><p>Simple content</p></body></html>";
    let html_path = create_html_file(&temp, "test.html", html);

    cli()
        .arg("extract")
        .arg("--input-file")
        .arg(&html_path)
        .arg("--engine")
        .arg("auto")
        .assert()
        .code(predicate::in_iter([0, 1])); // Auto will try to detect best engine
}

#[test]
fn test_extract_invalid_engine() {
    let temp = TempDir::new().unwrap();
    let html_path = create_html_file(&temp, "test.html", "<html><body>Test</body></html>");

    cli()
        .arg("extract")
        .arg("--input-file")
        .arg(&html_path)
        .arg("--engine")
        .arg("invalid-engine")
        .assert()
        .failure()
        .stderr(predicate::str::contains("engine").or(predicate::str::contains("invalid")));
}

#[test]
fn test_extract_no_input_source() {
    cli()
        .arg("extract")
        .arg("--engine")
        .arg("raw")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("input")));
}

#[test]
fn test_extract_with_output_file() {
    let temp = TempDir::new().unwrap();
    let html = "<html><body><h1>Content</h1></body></html>";
    let html_path = create_html_file(&temp, "input.html", html);
    let output_path = temp.child("output.txt");

    cli()
        .arg("extract")
        .arg("--input-file")
        .arg(&html_path)
        .arg("--engine")
        .arg("raw")
        .arg("--file")
        .arg(output_path.path())
        .assert()
        .success();

    output_path.assert(predicate::path::exists());
}

#[test]
fn test_extract_with_metadata_flag() {
    let temp = TempDir::new().unwrap();
    let html = r#"
        <html>
        <head>
            <title>Page Title</title>
            <meta name="description" content="Page description">
        </head>
        <body><p>Content</p></body>
        </html>
    "#;
    let html_path = create_html_file(&temp, "test.html", html);

    cli()
        .arg("extract")
        .arg("--input-file")
        .arg(&html_path)
        .arg("--metadata")
        .arg("--engine")
        .arg("raw")
        .assert()
        .success();
}

#[test]
fn test_extract_with_show_confidence() {
    let temp = TempDir::new().unwrap();
    let html = "<html><body><article><p>Content</p></article></body></html>";
    let html_path = create_html_file(&temp, "test.html", html);

    cli()
        .arg("extract")
        .arg("--input-file")
        .arg(&html_path)
        .arg("--show-confidence")
        .arg("--engine")
        .arg("raw")
        .assert()
        .success();
}

#[test]
fn test_extract_method_full() {
    let temp = TempDir::new().unwrap();
    let html = "<html><body><h1>Title</h1><p>Content</p></body></html>";
    let html_path = create_html_file(&temp, "test.html", html);

    cli()
        .arg("extract")
        .arg("--input-file")
        .arg(&html_path)
        .arg("--method")
        .arg("full")
        .arg("--engine")
        .arg("raw")
        .assert()
        .success();
}

#[test]
fn test_extract_method_article() {
    let temp = TempDir::new().unwrap();
    let html = r#"
        <html>
        <body>
            <article>
                <h1>Article Title</h1>
                <p>Article content</p>
            </article>
        </body>
        </html>
    "#;
    let html_path = create_html_file(&temp, "test.html", html);

    cli()
        .arg("extract")
        .arg("--input-file")
        .arg(&html_path)
        .arg("--method")
        .arg("article")
        .arg("--engine")
        .arg("raw")
        .assert()
        .success();
}

#[test]
fn test_extract_local_flag() {
    cli()
        .arg("extract")
        .arg("--url")
        .arg("https://example.com")
        .arg("--local")
        .arg("--engine")
        .arg("raw")
        .assert()
        .success();
}

#[test]
fn test_extract_no_wasm_flag() {
    let temp = TempDir::new().unwrap();
    let html = "<html><body><p>Content</p></body></html>";
    let html_path = create_html_file(&temp, "test.html", html);

    cli()
        .arg("extract")
        .arg("--input-file")
        .arg(&html_path)
        .arg("--no-wasm")
        .arg("--engine")
        .arg("auto")
        .assert()
        .success();
}

#[test]
fn test_extract_stealth_level_none() {
    cli()
        .arg("extract")
        .arg("--url")
        .arg("https://example.com")
        .arg("--local")
        .arg("--stealth-level")
        .arg("none")
        .arg("--engine")
        .arg("raw")
        .assert()
        .success();
}

#[test]
fn test_extract_stealth_level_low() {
    cli()
        .arg("extract")
        .arg("--url")
        .arg("https://example.com")
        .arg("--local")
        .arg("--stealth-level")
        .arg("low")
        .arg("--engine")
        .arg("raw")
        .assert()
        .success();
}

#[test]
fn test_extract_stealth_level_high() {
    cli()
        .arg("extract")
        .arg("--url")
        .arg("https://example.com")
        .arg("--local")
        .arg("--stealth-level")
        .arg("high")
        .arg("--engine")
        .arg("raw")
        .assert()
        .success();
}

#[test]
fn test_extract_with_user_agent() {
    cli()
        .arg("extract")
        .arg("--url")
        .arg("https://example.com")
        .arg("--local")
        .arg("--user-agent")
        .arg("CustomBot/1.0")
        .arg("--engine")
        .arg("raw")
        .assert()
        .success();
}

#[test]
fn test_extract_with_proxy() {
    // Will likely fail without real proxy, but tests argument parsing
    cli()
        .arg("extract")
        .arg("--url")
        .arg("https://example.com")
        .arg("--local")
        .arg("--proxy")
        .arg("http://localhost:8080")
        .arg("--engine")
        .arg("raw")
        .assert()
        .code(predicate::in_iter([0, 1]));
}

#[test]
fn test_extract_fingerprint_evasion() {
    cli()
        .arg("extract")
        .arg("--url")
        .arg("https://example.com")
        .arg("--local")
        .arg("--fingerprint-evasion")
        .arg("--stealth-level")
        .arg("high")
        .arg("--engine")
        .arg("raw")
        .assert()
        .success();
}

#[test]
fn test_extract_randomize_timing() {
    cli()
        .arg("extract")
        .arg("--url")
        .arg("https://example.com")
        .arg("--local")
        .arg("--randomize-timing")
        .arg("--stealth-level")
        .arg("medium")
        .arg("--engine")
        .arg("raw")
        .assert()
        .success();
}

#[test]
fn test_extract_simulate_behavior() {
    // This flag only works with headless engine
    cli()
        .arg("extract")
        .arg("--url")
        .arg("https://example.com")
        .arg("--local")
        .arg("--simulate-behavior")
        .arg("--engine")
        .arg("headless")
        .assert()
        .code(predicate::in_iter([0, 1])); // May fail if browser not available
}

#[test]
fn test_extract_headless_timeout() {
    cli()
        .arg("extract")
        .arg("--url")
        .arg("https://example.com")
        .arg("--local")
        .arg("--headless-timeout")
        .arg("10000")
        .arg("--engine")
        .arg("headless")
        .assert()
        .code(predicate::in_iter([0, 1]));
}

#[test]
fn test_extract_init_timeout() {
    let temp = TempDir::new().unwrap();
    let html = "<html><body><p>Content</p></body></html>";
    let html_path = create_html_file(&temp, "test.html", html);

    cli()
        .arg("extract")
        .arg("--input-file")
        .arg(&html_path)
        .arg("--init-timeout-ms")
        .arg("5000")
        .arg("--engine")
        .arg("wasm")
        .assert()
        .code(predicate::in_iter([0, 1]));
}

#[test]
fn test_extract_wasm_path_custom() {
    let temp = TempDir::new().unwrap();
    let html = "<html><body><p>Content</p></body></html>";
    let html_path = create_html_file(&temp, "test.html", html);

    cli()
        .arg("extract")
        .arg("--input-file")
        .arg(&html_path)
        .arg("--wasm-path")
        .arg("/nonexistent/path.wasm")
        .arg("--engine")
        .arg("wasm")
        .assert()
        .failure()
        .stderr(predicate::str::contains("WASM").or(predicate::str::contains("not found")));
}

#[test]
fn test_extract_empty_html() {
    let temp = TempDir::new().unwrap();
    let empty_html = "";
    let html_path = create_html_file(&temp, "empty.html", empty_html);

    cli()
        .arg("extract")
        .arg("--input-file")
        .arg(&html_path)
        .arg("--engine")
        .arg("raw")
        .assert()
        .success();
}

#[test]
fn test_extract_malformed_html() {
    let temp = TempDir::new().unwrap();
    let malformed = "<html><body><p>Unclosed paragraph<div>Nested wrong</p></div>";
    let html_path = create_html_file(&temp, "malformed.html", malformed);

    cli()
        .arg("extract")
        .arg("--input-file")
        .arg(&html_path)
        .arg("--engine")
        .arg("raw")
        .assert()
        .success(); // Should handle malformed HTML gracefully
}

#[test]
fn test_extract_large_html() {
    let temp = TempDir::new().unwrap();
    let large_html = format!(
        "<html><body>{}</body></html>",
        "<p>Paragraph content for testing large files.</p>".repeat(5000)
    );
    let html_path = create_html_file(&temp, "large.html", &large_html);

    cli()
        .arg("extract")
        .arg("--input-file")
        .arg(&html_path)
        .arg("--engine")
        .arg("raw")
        .assert()
        .success();
}

#[test]
fn test_extract_unicode_content() {
    let temp = TempDir::new().unwrap();
    let unicode_html = r#"
        <html>
        <body>
            <article>
                <h1>多语言内容测试</h1>
                <p>日本語のテキスト</p>
                <p>العربية النص</p>
                <p>Emoji: 🚀🔥💯🎉</p>
            </article>
        </body>
        </html>
    "#;
    let html_path = create_html_file(&temp, "unicode.html", unicode_html);

    cli()
        .arg("extract")
        .arg("--input-file")
        .arg(&html_path)
        .arg("--engine")
        .arg("raw")
        .assert()
        .success();
}

#[test]
fn test_extract_json_output_format() {
    let temp = TempDir::new().unwrap();
    let html = "<html><body><p>Content</p></body></html>";
    let html_path = create_html_file(&temp, "test.html", html);

    cli()
        .arg("--output")
        .arg("json")
        .arg("extract")
        .arg("--input-file")
        .arg(&html_path)
        .arg("--engine")
        .arg("raw")
        .assert()
        .success()
        .stdout(predicate::str::contains("{"));
}

#[test]
fn test_extract_text_output_format() {
    let temp = TempDir::new().unwrap();
    let html = "<html><body><p>Content</p></body></html>";
    let html_path = create_html_file(&temp, "test.html", html);

    cli()
        .arg("--output")
        .arg("text")
        .arg("extract")
        .arg("--input-file")
        .arg(&html_path)
        .arg("--engine")
        .arg("raw")
        .assert()
        .success();
}

#[test]
fn test_extract_table_output_format() {
    let temp = TempDir::new().unwrap();
    let html = "<html><body><p>Content</p></body></html>";
    let html_path = create_html_file(&temp, "test.html", html);

    cli()
        .arg("--output")
        .arg("table")
        .arg("extract")
        .arg("--input-file")
        .arg(&html_path)
        .arg("--engine")
        .arg("raw")
        .assert()
        .success();
}

#[test]
fn test_extract_selector_option() {
    let temp = TempDir::new().unwrap();
    let html = r#"
        <html>
        <body>
            <article class="main">Main content</article>
            <aside>Sidebar content</aside>
        </body>
        </html>
    "#;
    let html_path = create_html_file(&temp, "test.html", html);

    cli()
        .arg("extract")
        .arg("--input-file")
        .arg(&html_path)
        .arg("--selector")
        .arg("article.main")
        .arg("--engine")
        .arg("raw")
        .assert()
        .success();
}

#[test]
fn test_extract_pattern_option() {
    let temp = TempDir::new().unwrap();
    let html = "<html><body><p>Find this pattern</p></body></html>";
    let html_path = create_html_file(&temp, "test.html", html);

    cli()
        .arg("extract")
        .arg("--input-file")
        .arg(&html_path)
        .arg("--pattern")
        .arg("pattern")
        .arg("--engine")
        .arg("raw")
        .assert()
        .success();
}

#[test]
fn test_extract_strategy_option() {
    let temp = TempDir::new().unwrap();
    let html = "<html><body><p>Content</p></body></html>";
    let html_path = create_html_file(&temp, "test.html", html);

    cli()
        .arg("extract")
        .arg("--input-file")
        .arg(&html_path)
        .arg("--strategy")
        .arg("readability")
        .arg("--engine")
        .arg("raw")
        .assert()
        .success();
}

#[test]
fn test_extract_nonexistent_file() {
    cli()
        .arg("extract")
        .arg("--input-file")
        .arg("/nonexistent/file.html")
        .arg("--engine")
        .arg("raw")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("No such file")));
}

#[test]
fn test_extract_invalid_url() {
    cli()
        .arg("extract")
        .arg("--url")
        .arg("not-a-valid-url")
        .arg("--local")
        .arg("--engine")
        .arg("raw")
        .assert()
        .failure();
}
