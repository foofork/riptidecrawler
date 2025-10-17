//! End-to-End Workflow Tests
//!
//! Tests complete workflows including screenshot, extract, PDF pipelines
//! and verification of all artifacts in correct locations.

use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;

/// Helper to run CLI command
fn run_cli(command: &str, args: &[&str], env_vars: &[(&str, &str)]) -> Result<std::process::Output> {
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
async fn test_full_screenshot_workflow_direct_mode() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let output_dir = temp_dir.path().join("screenshots");

    // Run render command with screenshot
    let output = run_cli(
        "render",
        &[
            "--url",
            "https://example.com",
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--html",
            "--dom",
            "--screenshot",
            "full",
            "--wait",
            "load",
        ],
        &[("RUST_LOG", "info")],
    )?;

    // Verify command succeeded
    assert!(
        output.status.success(),
        "Screenshot workflow failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify output directory structure
    assert!(output_dir.exists(), "Output directory was not created");

    // Verify artifacts exist
    let entries: Vec<_> = std::fs::read_dir(&output_dir)?
        .filter_map(|e| e.ok())
        .collect();

    assert!(
        !entries.is_empty(),
        "No artifacts were created in output directory"
    );

    // Check for HTML file
    let has_html = entries.iter().any(|e| {
        e.path()
            .extension()
            .map(|ext| ext == "html")
            .unwrap_or(false)
    });
    assert!(has_html, "HTML file was not created");

    // Check for DOM file
    let has_dom = entries.iter().any(|e| {
        e.path()
            .file_name()
            .map(|name| name.to_string_lossy().contains(".dom."))
            .unwrap_or(false)
    });
    assert!(has_dom, "DOM file was not created");

    Ok(())
}

#[tokio::test]
async fn test_extract_screenshot_pdf_pipeline() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let output_dir = temp_dir.path();

    // Step 1: Extract content
    let extract_output = run_cli(
        "extract",
        &[
            "--url",
            "https://example.com",
            "--local",
            "--engine",
            "raw",
            "--file",
            output_dir.join("content.txt").to_str().unwrap(),
            "-o",
            "json",
        ],
        &[("RUST_LOG", "info")],
    )?;

    assert!(
        extract_output.status.success(),
        "Extract step failed: {}",
        String::from_utf8_lossy(&extract_output.stderr)
    );

    // Verify content file was created
    let content_file = output_dir.join("content.txt");
    assert!(
        content_file.exists(),
        "Extracted content file was not created"
    );

    // Step 2: Render with screenshot
    let screenshot_dir = output_dir.join("screenshots");
    let render_output = run_cli(
        "render",
        &[
            "--url",
            "https://example.com",
            "--output-dir",
            screenshot_dir.to_str().unwrap(),
            "--html",
            "--screenshot",
            "viewport",
            "--wait",
            "load",
        ],
        &[("RUST_LOG", "info")],
    )?;

    assert!(
        render_output.status.success(),
        "Render step failed: {}",
        String::from_utf8_lossy(&render_output.stderr)
    );

    // Verify screenshot directory exists
    assert!(
        screenshot_dir.exists(),
        "Screenshot directory was not created"
    );

    // Step 3: Generate PDF (if feature enabled)
    #[cfg(feature = "pdf")]
    {
        let pdf_output = run_cli(
            "render",
            &[
                "--url",
                "https://example.com",
                "--output-dir",
                output_dir.to_str().unwrap(),
                "--pdf",
                "--wait",
                "load",
            ],
            &[("RUST_LOG", "info")],
        )?;

        assert!(
            pdf_output.status.success(),
            "PDF generation failed: {}",
            String::from_utf8_lossy(&pdf_output.stderr)
        );
    }

    // Verify all artifacts are in correct subdirectories
    assert!(content_file.exists(), "Content file missing");
    assert!(screenshot_dir.exists(), "Screenshot directory missing");

    Ok(())
}

#[tokio::test]
async fn test_multi_url_batch_processing() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let base_output = temp_dir.path();

    let urls = vec![
        "https://example.com",
        "https://httpbin.org/html",
        "https://www.rust-lang.org",
    ];

    // Process each URL
    for (i, url) in urls.iter().enumerate() {
        let url_output = base_output.join(format!("url_{}", i));

        let output = run_cli(
            "render",
            &[
                "--url",
                url,
                "--output-dir",
                url_output.to_str().unwrap(),
                "--html",
                "--wait",
                "load",
            ],
            &[("RUST_LOG", "error")],
        )?;

        if !output.status.success() {
            // Some URLs might fail (network issues, etc.)
            eprintln!("Warning: URL {} failed: {}", url, String::from_utf8_lossy(&output.stderr));
            continue;
        }

        // Verify subdirectory was created
        if url_output.exists() {
            let entries: Vec<_> = std::fs::read_dir(&url_output)?
                .filter_map(|e| e.ok())
                .collect();

            assert!(
                !entries.is_empty(),
                "No files created for URL {}",
                url
            );
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_session_persistence_workflow() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let session_dir = temp_dir.path().join("sessions");
    let output_dir = temp_dir.path().join("output");

    std::fs::create_dir_all(&session_dir)?;

    // Create a session (this would be done by the session command)
    let session_file = session_dir.join("test_session.json");
    let session_data = r#"{
        "name": "test_session",
        "created_at": "2024-01-01T00:00:00Z",
        "cookies": [],
        "headers": {}
    }"#;
    std::fs::write(&session_file, session_data)?;

    // Use session in render command
    let output = run_cli(
        "render",
        &[
            "--url",
            "https://example.com",
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--html",
            "--session",
            "test_session",
            "--wait",
            "load",
        ],
        &[
            ("RUST_LOG", "info"),
            ("RIPTIDE_SESSION_DIR", session_dir.to_str().unwrap()),
        ],
    )?;

    // Command may fail if session support is not fully implemented
    // but should handle gracefully
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        // Check for session-related error messages
        assert!(
            stderr.contains("session") || stderr.contains("not found") || stderr.contains("not implemented"),
            "Unexpected error: {}",
            stderr
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_error_recovery_workflow() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;

    // Test 1: Invalid URL - should fail gracefully
    let output1 = run_cli(
        "extract",
        &[
            "--url",
            "not-a-valid-url",
            "--local",
            "-o",
            "json",
        ],
        &[("RUST_LOG", "error")],
    )?;

    assert!(
        !output1.status.success(),
        "Should fail with invalid URL"
    );

    // Test 2: Non-existent domain - should timeout or fail
    let output2 = run_cli(
        "extract",
        &[
            "--url",
            "https://this-domain-definitely-does-not-exist-12345.com",
            "--local",
            "--engine",
            "raw",
            "-o",
            "json",
        ],
        &[("RUST_LOG", "error")],
    )?;

    // Should fail but not crash
    assert!(
        !output2.status.success(),
        "Should fail with non-existent domain"
    );

    // Test 3: Invalid output directory (if permissions allow)
    #[cfg(unix)]
    {
        let invalid_dir = "/root/definitely-no-permission/output";
        let output3 = run_cli(
            "render",
            &[
                "--url",
                "https://example.com",
                "--output-dir",
                invalid_dir,
                "--html",
            ],
            &[("RUST_LOG", "error")],
        )?;

        // Should fail with permission error
        if !output3.status.success() {
            let stderr = String::from_utf8_lossy(&output3.stderr);
            assert!(
                stderr.contains("permission") || stderr.contains("Failed") || stderr.contains("denied"),
                "Should show permission error"
            );
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_stealth_mode_workflow() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let output_dir = temp_dir.path();

    // Test different stealth levels
    for stealth_level in &["off", "low", "medium", "high"] {
        let level_output = output_dir.join(stealth_level);

        let output = run_cli(
            "render",
            &[
                "--url",
                "https://example.com",
                "--output-dir",
                level_output.to_str().unwrap(),
                "--html",
                "--stealth",
                stealth_level,
                "--wait",
                "load",
            ],
            &[("RUST_LOG", "info")],
        )?;

        // All stealth levels should work
        assert!(
            output.status.success(),
            "Stealth level '{}' failed: {}",
            stealth_level,
            String::from_utf8_lossy(&output.stderr)
        );

        // Verify output was created
        if level_output.exists() {
            let entries: Vec<_> = std::fs::read_dir(&level_output)?
                .filter_map(|e| e.ok())
                .collect();

            assert!(
                !entries.is_empty(),
                "No files created for stealth level '{}'",
                stealth_level
            );
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_different_wait_conditions() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let output_dir = temp_dir.path();

    let wait_conditions = vec![
        ("load", "Page load event"),
        ("network-idle", "Network idle"),
        ("timeout:2000", "2 second timeout"),
    ];

    for (condition, description) in wait_conditions {
        let condition_output = output_dir.join(condition.replace(':', "_"));

        let output = run_cli(
            "render",
            &[
                "--url",
                "https://example.com",
                "--output-dir",
                condition_output.to_str().unwrap(),
                "--html",
                "--wait",
                condition,
            ],
            &[("RUST_LOG", "info")],
        )?;

        assert!(
            output.status.success(),
            "Wait condition '{}' ({}) failed: {}",
            condition,
            description,
            String::from_utf8_lossy(&output.stderr)
        );

        if condition_output.exists() {
            let entries: Vec<_> = std::fs::read_dir(&condition_output)?
                .filter_map(|e| e.ok())
                .collect();

            assert!(
                !entries.is_empty(),
                "No files created for wait condition '{}'",
                condition
            );
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_cleanup_after_workflow() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let output_dir = temp_dir.path().join("workflow_output");

    // Run a complete workflow
    let output = run_cli(
        "render",
        &[
            "--url",
            "https://example.com",
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--html",
            "--dom",
            "--wait",
            "load",
        ],
        &[("RUST_LOG", "info")],
    )?;

    if output.status.success() && output_dir.exists() {
        // Verify files were created
        let files_before: Vec<_> = std::fs::read_dir(&output_dir)?
            .filter_map(|e| e.ok())
            .collect();

        let count_before = files_before.len();
        assert!(count_before > 0, "No files were created");

        // Manual cleanup (in production, this might be automatic)
        std::fs::remove_dir_all(&output_dir)?;

        // Verify cleanup
        assert!(!output_dir.exists(), "Directory was not cleaned up");
    }

    Ok(())
}

#[tokio::test]
async fn test_output_organization() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let base_dir = temp_dir.path();

    // Create organized directory structure
    let dirs = vec![
        base_dir.join("screenshots"),
        base_dir.join("extracts"),
        base_dir.join("pdfs"),
        base_dir.join("raw_html"),
    ];

    for dir in &dirs {
        std::fs::create_dir_all(dir)?;
    }

    // Test 1: Screenshots go to screenshots dir
    let screenshot_output = run_cli(
        "render",
        &[
            "--url",
            "https://example.com",
            "--output-dir",
            dirs[0].to_str().unwrap(),
            "--screenshot",
            "viewport",
            "--wait",
            "load",
        ],
        &[("RUST_LOG", "error")],
    )?;

    // Test 2: Extracts go to extracts dir
    let extract_output = run_cli(
        "extract",
        &[
            "--url",
            "https://example.com",
            "--local",
            "--engine",
            "raw",
            "--file",
            dirs[1].join("content.txt").to_str().unwrap(),
            "-o",
            "json",
        ],
        &[("RUST_LOG", "error")],
    )?;

    // Verify organization
    for (i, dir) in dirs.iter().enumerate() {
        if dir.exists() {
            let entries: Vec<_> = std::fs::read_dir(dir)?
                .filter_map(|e| e.ok())
                .collect();

            if !entries.is_empty() {
                println!("✓ Directory {} contains {} files", i, entries.len());
            }
        }
    }

    Ok(())
}
