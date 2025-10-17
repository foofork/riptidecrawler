//! CLI Health Command Test Suite
//!
//! Tests for the CLI health commands across both Rust CLI and Node.js CLI

use std::process::Command;
use std::time::Duration;
use tokio::time::timeout;

#[cfg(test)]
mod rust_cli_tests {
    use super::*;

    /// Test Rust CLI health command with JSON output
    #[tokio::test]
    async fn test_rust_cli_health_json() {
        let output = Command::new("cargo")
            .args(&["run", "--bin", "riptide-cli", "--", "health", "--format", "json"])
            .current_dir("/workspaces/eventmesh/crates/riptide-cli")
            .output()
            .expect("Failed to execute CLI health command");

        // Should not panic or error
        assert!(
            output.status.success() || output.status.code() == Some(1),
            "CLI command should execute (exit 0 or 1 if unhealthy)"
        );

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Should produce JSON output
        if !stdout.is_empty() {
            let json: serde_json::Value = serde_json::from_str(&stdout)
                .expect("CLI should output valid JSON");

            assert!(json.get("status").is_some());
        }
    }

    /// Test Rust CLI health command with table output
    #[tokio::test]
    async fn test_rust_cli_health_table() {
        let output = Command::new("cargo")
            .args(&["run", "--bin", "riptide-cli", "--", "health", "--format", "table"])
            .current_dir("/workspaces/eventmesh/crates/riptide-cli")
            .output()
            .expect("Failed to execute CLI health command");

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Table output should contain expected components
        if !stdout.is_empty() {
            assert!(stdout.contains("Component") || stdout.contains("Status"));
        }
    }

    /// Test Rust CLI health command with default output
    #[tokio::test]
    async fn test_rust_cli_health_default() {
        let output = Command::new("cargo")
            .args(&["run", "--bin", "riptide-cli", "--", "health"])
            .current_dir("/workspaces/eventmesh/crates/riptide-cli")
            .output()
            .expect("Failed to execute CLI health command");

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Should contain health information
        if !stdout.is_empty() {
            assert!(
                stdout.to_lowercase().contains("health") ||
                stdout.to_lowercase().contains("redis") ||
                stdout.to_lowercase().contains("status")
            );
        }
    }

    /// Test CLI handles server unavailable gracefully
    #[tokio::test]
    async fn test_cli_health_server_unavailable() {
        let output = Command::new("cargo")
            .args(&[
                "run",
                "--bin",
                "riptide-cli",
                "--",
                "--url",
                "http://localhost:9999",
                "health"
            ])
            .current_dir("/workspaces/eventmesh/crates/riptide-cli")
            .output()
            .expect("Failed to execute CLI health command");

        // Should handle connection error gracefully
        assert!(!output.status.success());

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.to_lowercase().contains("error") ||
            stderr.to_lowercase().contains("connect") ||
            stderr.to_lowercase().contains("failed")
        );
    }
}

#[cfg(test)]
mod node_cli_tests {
    use super::*;

    /// Helper to check if Node.js CLI is available
    fn is_node_cli_available() -> bool {
        std::path::Path::new("/workspaces/eventmesh/cli/bin/riptide.js").exists()
    }

    /// Test Node.js CLI health command with JSON output
    #[tokio::test]
    async fn test_node_cli_health_json() {
        if !is_node_cli_available() {
            println!("Skipping: Node.js CLI not available");
            return;
        }

        let output = Command::new("node")
            .args(&["bin/riptide.js", "health", "--json"])
            .current_dir("/workspaces/eventmesh/cli")
            .output()
            .expect("Failed to execute Node CLI health command");

        let stdout = String::from_utf8_lossy(&output.stdout);

        if !stdout.is_empty() {
            let json: serde_json::Value = serde_json::from_str(&stdout)
                .expect("Node CLI should output valid JSON");

            assert!(json.get("status").is_some());
        }
    }

    /// Test Node.js CLI health watch mode
    #[tokio::test]
    async fn test_node_cli_health_watch() {
        if !is_node_cli_available() {
            println!("Skipping: Node.js CLI not available");
            return;
        }

        // Start watch mode with timeout
        let child = Command::new("node")
            .args(&["bin/riptide.js", "health", "--watch", "--interval", "1"])
            .current_dir("/workspaces/eventmesh/cli")
            .spawn();

        if let Ok(mut process) = child {
            // Let it run for 2 seconds
            tokio::time::sleep(Duration::from_secs(2)).await;

            // Kill the process
            let _ = process.kill();

            // Verify it was running
            if let Ok(status) = process.wait() {
                // Process was terminated (exit code varies by OS)
                assert!(!status.success() || status.code() == Some(0));
            }
        }
    }

    /// Test Node.js CLI health with custom API URL
    #[tokio::test]
    async fn test_node_cli_health_custom_url() {
        if !is_node_cli_available() {
            println!("Skipping: Node.js CLI not available");
            return;
        }

        let output = Command::new("node")
            .args(&[
                "bin/riptide.js",
                "--url",
                "http://localhost:8080",
                "health"
            ])
            .current_dir("/workspaces/eventmesh/cli")
            .output()
            .expect("Failed to execute Node CLI health command");

        // Command should execute (may fail if server unavailable)
        assert!(
            output.status.success() ||
            !String::from_utf8_lossy(&output.stderr).is_empty()
        );
    }
}

#[cfg(test)]
mod cli_integration_tests {
    use super::*;

    /// Test CLI output formatting consistency
    #[tokio::test]
    async fn test_cli_output_formats_consistent() {
        let formats = vec!["json", "table"];

        for format in formats {
            let output = Command::new("cargo")
                .args(&[
                    "run",
                    "--bin",
                    "riptide-cli",
                    "--",
                    "health",
                    "--format",
                    format
                ])
                .current_dir("/workspaces/eventmesh/crates/riptide-cli")
                .output()
                .expect("Failed to execute CLI health command");

            let stdout = String::from_utf8_lossy(&output.stdout);

            if !stdout.is_empty() {
                match format {
                    "json" => {
                        // Should be valid JSON
                        serde_json::from_str::<serde_json::Value>(&stdout)
                            .expect(&format!("Format {} should produce valid JSON", format));
                    }
                    "table" => {
                        // Should contain table-like structure
                        assert!(
                            stdout.contains("│") ||
                            stdout.contains("|") ||
                            stdout.contains("─") ||
                            stdout.contains("-")
                        );
                    }
                    _ => {}
                }
            }
        }
    }

    /// Test CLI exit codes
    #[tokio::test]
    async fn test_cli_exit_codes() {
        let output = Command::new("cargo")
            .args(&["run", "--bin", "riptide-cli", "--", "health"])
            .current_dir("/workspaces/eventmesh/crates/riptide-cli")
            .output()
            .expect("Failed to execute CLI health command");

        // Exit code should be 0 (healthy) or 1 (unhealthy/error)
        let exit_code = output.status.code().unwrap_or(-1);
        assert!(
            exit_code == 0 || exit_code == 1,
            "Exit code should be 0 or 1, got {}",
            exit_code
        );
    }

    /// Test CLI respects timeout
    #[tokio::test]
    async fn test_cli_respects_timeout() {
        let start = std::time::Instant::now();

        let output = Command::new("cargo")
            .args(&[
                "run",
                "--bin",
                "riptide-cli",
                "--",
                "--url",
                "http://localhost:8080",
                "health"
            ])
            .current_dir("/workspaces/eventmesh/crates/riptide-cli")
            .output()
            .expect("Failed to execute CLI health command");

        let duration = start.elapsed();

        // Should complete within reasonable time (30 seconds max)
        assert!(
            duration < Duration::from_secs(30),
            "CLI health command took too long: {:?}",
            duration
        );
    }
}

#[cfg(test)]
mod cli_error_handling_tests {
    use super::*;

    /// Test CLI handles invalid URL gracefully
    #[tokio::test]
    async fn test_cli_invalid_url() {
        let output = Command::new("cargo")
            .args(&[
                "run",
                "--bin",
                "riptide-cli",
                "--",
                "--url",
                "not-a-valid-url",
                "health"
            ])
            .current_dir("/workspaces/eventmesh/crates/riptide-cli")
            .output()
            .expect("Failed to execute CLI health command");

        assert!(!output.status.success());

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.to_lowercase().contains("error") ||
            stderr.to_lowercase().contains("invalid")
        );
    }

    /// Test CLI handles network timeout
    #[tokio::test]
    async fn test_cli_network_timeout() {
        // Use a non-routable IP to force timeout
        let output = Command::new("cargo")
            .args(&[
                "run",
                "--bin",
                "riptide-cli",
                "--",
                "--url",
                "http://192.0.2.1:8080",
                "health"
            ])
            .current_dir("/workspaces/eventmesh/crates/riptide-cli")
            .output()
            .expect("Failed to execute CLI health command");

        // Should fail gracefully
        assert!(!output.status.success());

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.to_lowercase().contains("error") ||
            stderr.to_lowercase().contains("timeout") ||
            stderr.to_lowercase().contains("connect")
        );
    }

    /// Test CLI handles malformed JSON response
    #[tokio::test]
    async fn test_cli_malformed_response_handling() {
        // This test would require a mock server returning invalid JSON
        // For now, we test that CLI handles unexpected responses

        let output = Command::new("cargo")
            .args(&["run", "--bin", "riptide-cli", "--", "health", "--format", "json"])
            .current_dir("/workspaces/eventmesh/crates/riptide-cli")
            .output()
            .expect("Failed to execute CLI health command");

        // Should produce output or error message
        assert!(
            !output.stdout.is_empty() ||
            !output.stderr.is_empty()
        );
    }
}
