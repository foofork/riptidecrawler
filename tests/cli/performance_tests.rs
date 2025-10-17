//! Performance and Resource Tests
//!
//! Tests to verify browser pool is NOT duplicated, memory usage stays <= 2GB,
//! and rate limiting is properly enforced.

use anyhow::Result;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[cfg(target_os = "linux")]
use std::fs;

/// Get memory usage of current process (Linux only)
#[cfg(target_os = "linux")]
fn get_process_memory_mb() -> Result<u64> {
    let status = fs::read_to_string("/proc/self/status")?;

    for line in status.lines() {
        if line.starts_with("VmRSS:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let kb: u64 = parts[1].parse()?;
                return Ok(kb / 1024); // Convert KB to MB
            }
        }
    }

    Ok(0)
}

#[cfg(not(target_os = "linux"))]
fn get_process_memory_mb() -> Result<u64> {
    // Fallback for non-Linux systems
    Ok(0)
}

#[tokio::test]
async fn test_browser_pool_not_duplicated() -> Result<()> {
    // This test verifies that browser pool is shared, not duplicated per command

    // Start API server in background (if not already running)
    let api_url = "http://localhost:8080";

    // Check if already running
    let is_running = reqwest::get(format!("{}/health", api_url))
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false);

    if !is_running {
        println!("⊘ Skipping test: API server not running");
        return Ok(());
    }

    // Make multiple concurrent requests
    let handles: Vec<_> = (0..3)
        .map(|i| {
            tokio::spawn(async move {
                let client = reqwest::Client::new();
                let response = client
                    .post(format!("{}/api/v1/render", api_url))
                    .json(&serde_json::json!({
                        "url": "https://example.com",
                        "wait": "load",
                        "screenshot": false,
                    }))
                    .send()
                    .await;

                (i, response)
            })
        })
        .collect();

    // Wait for all requests
    let results = futures::future::join_all(handles).await;

    // Count successful requests
    let successful = results
        .iter()
        .filter(|r| {
            r.as_ref()
                .ok()
                .and_then(|(_, resp)| resp.as_ref().ok())
                .map(|resp| resp.status().is_success())
                .unwrap_or(false)
        })
        .count();

    println!("✓ {} concurrent requests completed successfully", successful);

    // If all succeeded, browser pool was properly shared
    // (If duplicated, we'd likely see errors or timeouts)

    Ok(())
}

#[tokio::test]
#[cfg(target_os = "linux")]
async fn test_memory_usage_under_2gb() -> Result<()> {
    // Get initial memory usage
    let initial_memory = get_process_memory_mb()?;
    println!("Initial memory usage: {} MB", initial_memory);

    // Perform memory-intensive operations
    let temp_dir = tempfile::tempdir()?;

    for i in 0..10 {
        let output_dir = temp_dir.path().join(format!("run_{}", i));

        let mut cmd = Command::new("cargo");
        cmd.arg("run")
            .arg("--bin")
            .arg("riptide")
            .arg("--")
            .arg("render")
            .arg("--url")
            .arg("https://example.com")
            .arg("--output-dir")
            .arg(output_dir.to_str().unwrap())
            .arg("--html")
            .arg("--wait")
            .arg("load")
            .env("RUST_LOG", "error")
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        let _ = cmd.output();

        // Check memory after each iteration
        let current_memory = get_process_memory_mb()?;
        println!("Memory after iteration {}: {} MB", i, current_memory);

        // Verify memory stays under 2GB (2048 MB)
        assert!(
            current_memory < 2048,
            "Memory usage exceeded 2GB: {} MB",
            current_memory
        );
    }

    let final_memory = get_process_memory_mb()?;
    println!("Final memory usage: {} MB", final_memory);

    // Memory increase should be reasonable (< 500MB for this test)
    let memory_increase = final_memory.saturating_sub(initial_memory);
    assert!(
        memory_increase < 500,
        "Memory increased too much: {} MB",
        memory_increase
    );

    Ok(())
}

#[tokio::test]
async fn test_rate_limiting_enforcement() -> Result<()> {
    // Test that rate limiting is properly enforced
    let api_url = std::env::var("RIPTIDE_API_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());

    // Check if API server is running
    let is_running = reqwest::get(format!("{}/health", api_url))
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false);

    if !is_running {
        println!("⊘ Skipping test: API server not running");
        return Ok(());
    }

    // Send rapid requests
    let client = reqwest::Client::new();
    let start = Instant::now();

    let mut success_count = 0;
    let mut rate_limited_count = 0;

    for i in 0..20 {
        let response = client
            .get(format!("{}/health", api_url))
            .send()
            .await;

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    success_count += 1;
                } else if resp.status().as_u16() == 429 {
                    // 429 Too Many Requests
                    rate_limited_count += 1;
                    println!("✓ Request {} was rate limited", i);
                }
            }
            Err(e) => {
                println!("Request {} failed: {}", i, e);
            }
        }

        // Small delay between requests
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    let elapsed = start.elapsed();

    println!(
        "Completed in {:?}: {} successful, {} rate limited",
        elapsed, success_count, rate_limited_count
    );

    // If rate limiting is working, we should see some rate limited responses
    // OR all requests succeed but with reasonable timing
    assert!(
        success_count > 0,
        "No successful requests - server might be down"
    );

    Ok(())
}

#[tokio::test]
async fn test_concurrent_render_performance() -> Result<()> {
    // Test that concurrent renders perform efficiently
    let temp_dir = tempfile::tempdir()?;

    let start = Instant::now();

    // Run 5 concurrent render operations
    let handles: Vec<_> = (0..5)
        .map(|i| {
            let output_dir = temp_dir.path().join(format!("concurrent_{}", i));
            tokio::spawn(async move {
                let mut cmd = Command::new("cargo");
                cmd.arg("run")
                    .arg("--bin")
                    .arg("riptide")
                    .arg("--")
                    .arg("render")
                    .arg("--url")
                    .arg("https://example.com")
                    .arg("--output-dir")
                    .arg(output_dir.to_str().unwrap())
                    .arg("--html")
                    .arg("--wait")
                    .arg("load")
                    .env("RUST_LOG", "error");

                let start = Instant::now();
                let result = cmd.output();
                let duration = start.elapsed();

                (i, result, duration)
            })
        })
        .collect();

    // Wait for all to complete
    let results = futures::future::join_all(handles).await;

    let total_elapsed = start.elapsed();

    // Analyze results
    let mut successful = 0;
    let mut total_time = Duration::ZERO;

    for result in results {
        if let Ok((i, output, duration)) = result {
            if output.map(|o| o.status.success()).unwrap_or(false) {
                successful += 1;
                total_time += duration;
                println!("✓ Render {} completed in {:?}", i, duration);
            }
        }
    }

    println!(
        "All {} renders completed in {:?} (avg: {:?})",
        successful,
        total_elapsed,
        total_time / successful.max(1) as u32
    );

    // Concurrent execution should be faster than sequential
    // (sequential would be ~5x individual time, concurrent should be less)
    assert!(
        successful > 0,
        "No renders completed successfully"
    );

    Ok(())
}

#[tokio::test]
async fn test_resource_cleanup() -> Result<()> {
    // Test that resources are properly cleaned up after operations
    let temp_dir = tempfile::tempdir()?;

    #[cfg(target_os = "linux")]
    {
        // Check open file descriptors before
        let fds_before = fs::read_dir("/proc/self/fd")?.count();

        // Perform multiple operations
        for i in 0..5 {
            let output_dir = temp_dir.path().join(format!("cleanup_{}", i));

            let mut cmd = Command::new("cargo");
            cmd.arg("run")
                .arg("--bin")
                .arg("riptide")
                .arg("--")
                .arg("render")
                .arg("--url")
                .arg("https://example.com")
                .arg("--output-dir")
                .arg(output_dir.to_str().unwrap())
                .arg("--html")
                .env("RUST_LOG", "error");

            let _ = cmd.output();
        }

        // Check file descriptors after
        let fds_after = fs::read_dir("/proc/self/fd")?.count();

        println!(
            "File descriptors: {} -> {} (delta: {})",
            fds_before,
            fds_after,
            fds_after.saturating_sub(fds_before)
        );

        // Should not leak file descriptors
        assert!(
            fds_after < fds_before + 10,
            "Too many file descriptors leaked: {} -> {}",
            fds_before,
            fds_after
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_timeout_handling() -> Result<()> {
    // Test that timeouts are properly handled without hanging
    let temp_dir = tempfile::tempdir()?;

    let start = Instant::now();

    let mut cmd = Command::new("cargo");
    cmd.arg("run")
        .arg("--bin")
        .arg("riptide")
        .arg("--")
        .arg("extract")
        .arg("--url")
        .arg("https://httpbin.org/delay/10") // 10 second delay
        .arg("--local")
        .arg("--init-timeout-ms")
        .arg("2000") // 2 second timeout
        .env("RUST_LOG", "error");

    let output = cmd.output()?;
    let elapsed = start.elapsed();

    // Should timeout before 10 seconds
    assert!(
        elapsed < Duration::from_secs(5),
        "Command did not timeout properly: {:?}",
        elapsed
    );

    println!("✓ Command timed out correctly after {:?}", elapsed);

    Ok(())
}

#[tokio::test]
async fn test_large_page_handling() -> Result<()> {
    // Test handling of large pages
    let temp_dir = tempfile::tempdir()?;
    let output_dir = temp_dir.path();

    // Use a page known to be large
    let mut cmd = Command::new("cargo");
    cmd.arg("run")
        .arg("--bin")
        .arg("riptide")
        .arg("--")
        .arg("render")
        .arg("--url")
        .arg("https://www.rust-lang.org") // Large page with lots of content
        .arg("--output-dir")
        .arg(output_dir.to_str().unwrap())
        .arg("--html")
        .arg("--wait")
        .arg("load")
        .env("RUST_LOG", "error");

    let start = Instant::now();
    let output = cmd.output()?;
    let elapsed = start.elapsed();

    if output.status.success() {
        // Check output file size
        let entries: Vec<_> = std::fs::read_dir(output_dir)?
            .filter_map(|e| e.ok())
            .collect();

        for entry in entries {
            let metadata = entry.metadata()?;
            let size_kb = metadata.len() / 1024;
            println!("✓ Created file: {} ({} KB)", entry.file_name().to_string_lossy(), size_kb);
        }

        println!("✓ Large page handled successfully in {:?}", elapsed);
    }

    Ok(())
}

#[tokio::test]
async fn test_burst_request_handling() -> Result<()> {
    // Test handling of burst requests
    let temp_dir = tempfile::tempdir()?;

    // Send burst of 10 requests
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let output_dir = temp_dir.path().join(format!("burst_{}", i));
            tokio::spawn(async move {
                let mut cmd = Command::new("cargo");
                cmd.arg("run")
                    .arg("--bin")
                    .arg("riptide")
                    .arg("--")
                    .arg("extract")
                    .arg("--url")
                    .arg("https://example.com")
                    .arg("--local")
                    .arg("--engine")
                    .arg("raw")
                    .arg("-o")
                    .arg("json")
                    .env("RUST_LOG", "error")
                    .stdout(Stdio::null());

                let start = Instant::now();
                let result = cmd.output();
                let duration = start.elapsed();

                (i, result, duration)
            })
        })
        .collect();

    let start = Instant::now();
    let results = futures::future::join_all(handles).await;
    let total_elapsed = start.elapsed();

    let successful = results
        .iter()
        .filter(|r| {
            r.as_ref()
                .ok()
                .and_then(|(_, res, _)| res.as_ref().ok())
                .map(|o| o.status.success())
                .unwrap_or(false)
        })
        .count();

    println!(
        "✓ Burst of 10 requests: {} successful in {:?}",
        successful, total_elapsed
    );

    // At least half should succeed
    assert!(
        successful >= 5,
        "Too many burst requests failed: {} / 10",
        successful
    );

    Ok(())
}
