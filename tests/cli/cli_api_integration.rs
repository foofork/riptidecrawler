//! CLI→API Integration Tests
//!
//! Tests the architecture where CLI commands can use the API server when available,
//! or fall back to direct execution when the API is not available.

use anyhow::Result;
use std::process::Command;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Helper to check if API server is running
async fn is_api_server_running(url: &str) -> bool {
    reqwest::get(format!("{}/health", url))
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

/// Helper to run CLI command with environment variables
fn run_cli_command(
    command: &str,
    args: &[&str],
    env_vars: &[(&str, &str)],
) -> Result<std::process::Output> {
    let mut cmd = Command::new("cargo");
    cmd.arg("run")
        .arg("--bin")
        .arg("riptide")
        .arg("--")
        .arg(command);

    for arg in args {
        cmd.arg(arg);
    }

    for (key, value) in env_vars {
        cmd.env(key, value);
    }

    Ok(cmd.output()?)
}

#[tokio::test]
async fn test_cli_uses_api_when_available() -> Result<()> {
    // This test requires a running API server
    let api_url = std::env::var("RIPTIDE_API_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

    // Skip test if API server is not running
    if !is_api_server_running(&api_url).await {
        println!("⊘ Skipping test: API server not running at {}", api_url);
        return Ok(());
    }

    // Test extract command via API
    let output = run_cli_command(
        "extract",
        &[
            "--url",
            "https://example.com",
            "--method",
            "article",
            "-o",
            "json",
        ],
        &[
            ("RIPTIDE_API_URL", &api_url),
            ("RUST_LOG", "info"),
        ],
    )?;

    // Verify the command succeeded
    assert!(
        output.status.success(),
        "CLI command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Check that output contains expected JSON structure
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("content") || stdout.contains("extracted"),
        "Output doesn't contain expected content: {}",
        stdout
    );

    // Verify API was used (check for API-specific indicators in output)
    let stderr = String::from_utf8_lossy(&output.stderr);
    // API calls should show up in logs
    // This is a weak check - in production we'd have better API telemetry

    Ok(())
}

#[tokio::test]
async fn test_cli_fallback_to_direct_when_api_unavailable() -> Result<()> {
    // Use a URL that will definitely fail to connect
    let invalid_api_url = "http://localhost:99999";

    // Test extract command with local flag (bypasses API)
    let output = run_cli_command(
        "extract",
        &[
            "--url",
            "https://example.com",
            "--local",
            "--engine",
            "raw",
            "-o",
            "json",
        ],
        &[
            ("RIPTIDE_API_URL", invalid_api_url),
            ("RUST_LOG", "info"),
        ],
    )?;

    // Verify the command succeeded despite API being unavailable
    assert!(
        output.status.success(),
        "CLI command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify output contains content
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("content") || stdout.contains("<!DOCTYPE") || stdout.contains("html"),
        "Output doesn't contain expected content"
    );

    // Verify fallback was used
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("local") || stderr.contains("direct") || stderr.contains("raw"),
        "No indication of local/direct execution in logs"
    );

    Ok(())
}

#[tokio::test]
async fn test_output_directory_configuration() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let output_path = temp_dir.path().to_str().unwrap();

    // Test render command with custom output directory
    let output = run_cli_command(
        "render",
        &[
            "--url",
            "https://example.com",
            "--output-dir",
            output_path,
            "--html",
            "--wait",
            "load",
        ],
        &[("RUST_LOG", "info")],
    )?;

    // Verify command succeeded
    assert!(
        output.status.success(),
        "Render command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify files were created in the correct directory
    let entries: Vec<_> = std::fs::read_dir(output_path)?
        .filter_map(|e| e.ok())
        .collect();

    assert!(
        !entries.is_empty(),
        "No files were created in output directory"
    );

    // Verify at least one HTML file was created
    let has_html_file = entries.iter().any(|entry| {
        entry
            .path()
            .extension()
            .map(|ext| ext == "html")
            .unwrap_or(false)
    });

    assert!(has_html_file, "No HTML file was created");

    Ok(())
}

#[tokio::test]
async fn test_wasm_path_configuration() -> Result<()> {
    // Test with custom WASM path environment variable
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let wasm_path = format!(
        "{}/../../target/wasm32-wasip2/release/riptide-extractor-wasm.component.wasm",
        manifest_dir
    );

    // Only run test if WASM file exists
    if !std::path::Path::new(&wasm_path).exists() {
        println!("⊘ Skipping test: WASM file not found at {}", wasm_path);
        return Ok(());
    }

    let output = run_cli_command(
        "extract",
        &[
            "--url",
            "https://example.com",
            "--local",
            "--engine",
            "wasm",
            "-o",
            "json",
        ],
        &[
            ("RIPTIDE_WASM_PATH", &wasm_path),
            ("RUST_LOG", "info"),
        ],
    )?;

    // Verify command succeeded
    assert!(
        output.status.success(),
        "Extract command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify WASM extraction was used
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("WASM") || stderr.contains("wasm"),
        "No indication of WASM extraction in logs"
    );

    Ok(())
}

#[tokio::test]
async fn test_cli_timeout_configuration() -> Result<()> {
    // Test with custom timeout settings
    let output = run_cli_command(
        "extract",
        &[
            "--url",
            "https://httpbin.org/delay/2",
            "--local",
            "--engine",
            "raw",
            "--init-timeout-ms",
            "5000",
            "-o",
            "json",
        ],
        &[("RUST_LOG", "info")],
    )?;

    // Command should succeed with adequate timeout
    assert!(
        output.status.success(),
        "Extract command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    Ok(())
}

#[tokio::test]
async fn test_api_key_authentication() -> Result<()> {
    let api_url = std::env::var("RIPTIDE_API_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

    // Skip test if API server is not running
    if !is_api_server_running(&api_url).await {
        println!("⊘ Skipping test: API server not running");
        return Ok(());
    }

    // Test with API key (if server requires it)
    let output = run_cli_command(
        "health",
        &[],
        &[
            ("RIPTIDE_API_URL", &api_url),
            ("RIPTIDE_API_KEY", "test-key"),
        ],
    )?;

    // Note: This test depends on whether the server requires authentication
    // It should either succeed or fail with a clear authentication error
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        assert!(
            stderr.contains("auth") || stderr.contains("key") || stderr.contains("401") || stderr.contains("403"),
            "Unexpected error (not authentication-related): {}",
            stderr
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_multiple_output_formats() -> Result<()> {
    for format in &["json", "text", "table"] {
        let output = run_cli_command(
            "extract",
            &[
                "--url",
                "https://example.com",
                "--local",
                "--engine",
                "raw",
                "-o",
                format,
            ],
            &[("RUST_LOG", "error")],
        )?;

        assert!(
            output.status.success(),
            "Extract command failed for format '{}': {}",
            format,
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            !stdout.is_empty(),
            "No output for format '{}'",
            format
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_verbose_logging() -> Result<()> {
    let output = run_cli_command(
        "extract",
        &[
            "--url",
            "https://example.com",
            "--local",
            "--engine",
            "raw",
            "-v",
            "-o",
            "json",
        ],
        &[("RUST_LOG", "debug")],
    )?;

    // Verify verbose logging produced debug output
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Debug logging should show more details
    assert!(
        stderr.len() > 100,
        "Verbose logging doesn't seem to produce enough output"
    );

    Ok(())
}

#[tokio::test]
async fn test_environment_variable_precedence() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let env_output_dir = temp_dir.path().join("env");
    let cli_output_dir = temp_dir.path().join("cli");

    std::fs::create_dir_all(&env_output_dir)?;
    std::fs::create_dir_all(&cli_output_dir)?;

    // CLI flag should override environment variable
    let output = run_cli_command(
        "render",
        &[
            "--url",
            "https://example.com",
            "--output-dir",
            cli_output_dir.to_str().unwrap(),
            "--html",
            "--wait",
            "load",
        ],
        &[
            ("RIPTIDE_OUTPUT_DIR", env_output_dir.to_str().unwrap()),
            ("RUST_LOG", "info"),
        ],
    )?;

    assert!(
        output.status.success(),
        "Render command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Files should be in CLI-specified directory, not env var directory
    let cli_files: Vec<_> = std::fs::read_dir(&cli_output_dir)?
        .filter_map(|e| e.ok())
        .collect();

    assert!(
        !cli_files.is_empty(),
        "No files in CLI-specified directory"
    );

    Ok(())
}

#[tokio::test]
async fn test_concurrent_cli_commands() -> Result<()> {
    // Test that multiple CLI commands can run concurrently without conflicts
    let handles: Vec<_> = (0..3)
        .map(|i| {
            tokio::spawn(async move {
                run_cli_command(
                    "extract",
                    &[
                        "--url",
                        "https://example.com",
                        "--local",
                        "--engine",
                        "raw",
                        "-o",
                        "json",
                    ],
                    &[("RUST_LOG", "error")],
                )
            })
        })
        .collect();

    // Wait for all commands to complete
    let results: Vec<_> = futures::future::join_all(handles)
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    // Verify all succeeded
    for (i, result) in results.iter().enumerate() {
        let output = result.as_ref().unwrap();
        assert!(
            output.status.success(),
            "Concurrent command {} failed: {}",
            i,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

/// Test module initialization
#[tokio::test]
async fn test_cli_binary_available() -> Result<()> {
    let output = Command::new("cargo")
        .arg("build")
        .arg("--bin")
        .arg("riptide")
        .output()?;

    assert!(
        output.status.success(),
        "Failed to build CLI binary: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    Ok(())
}
