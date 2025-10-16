//! Integration tests for metrics collection in CLI commands
//!
//! This test verifies that metrics are properly collected for:
//! - Extract command
//! - Render command
//! - Crawl command
//! - PDF commands

use riptide_cli::metrics::{MetricsManager, MetricsStorageConfig};
use std::time::Duration;
use tokio::time::sleep;

/// Test that metrics are properly initialized
#[tokio::test]
async fn test_metrics_initialization() {
    let manager = MetricsManager::new().expect("Failed to create metrics manager");
    let summary = manager.get_summary().await.expect("Failed to get summary");

    assert_eq!(summary.total_commands, 0);
    assert_eq!(summary.overall_success_rate, 0.0);
}

/// Test extract command metrics tracking
#[tokio::test]
async fn test_extract_command_metrics() {
    let config = MetricsStorageConfig {
        max_command_history: 100,
        retention_days: 1,
        auto_cleanup: false,
        storage_path: "/tmp/riptide_test_metrics.json".to_string(),
        rotation_threshold: 50,
    };

    let manager = MetricsManager::with_config(config).expect("Failed to create metrics manager");

    // Simulate extract command execution
    let tracking_id = manager
        .start_command("extract")
        .await
        .expect("Failed to start command tracking");

    // Simulate some work
    sleep(Duration::from_millis(10)).await;

    // Record progress
    manager
        .record_progress(&tracking_id, 1, 1024, 0, 1)
        .await
        .expect("Failed to record progress");

    // Record engine selection
    manager
        .collector()
        .record_metric("extract.engine.wasm", 1.0)
        .expect("Failed to record engine metric");

    // Complete command
    manager
        .complete_command(&tracking_id)
        .await
        .expect("Failed to complete command");

    // Verify metrics
    let summary = manager.get_summary().await.expect("Failed to get summary");

    assert_eq!(summary.total_commands, 1);
    assert_eq!(summary.total_api_calls, 1);
    assert_eq!(summary.total_bytes_transferred, 1024);
    assert!(summary.overall_success_rate > 0.0);

    // Verify command-specific metrics
    let recent = manager
        .get_recent_commands(10)
        .await
        .expect("Failed to get recent commands");

    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].command_name, "extract");
    assert!(recent[0].success);
    assert_eq!(recent[0].items_processed, 1);
}

/// Test render command metrics tracking
#[tokio::test]
async fn test_render_command_metrics() {
    let manager = MetricsManager::new().expect("Failed to create metrics manager");

    // Simulate render command
    let tracking_id = manager
        .start_command("render")
        .await
        .expect("Failed to start command tracking");

    sleep(Duration::from_millis(50)).await;

    // Record metrics for files saved
    manager
        .record_progress(&tracking_id, 3, 5120, 0, 0)
        .await
        .expect("Failed to record progress");

    manager
        .collector()
        .record_metric("render.duration_ms", 50.0)
        .expect("Failed to record duration");

    manager
        .collector()
        .record_metric("render.wait.load", 1.0)
        .expect("Failed to record wait condition");

    manager
        .complete_command(&tracking_id)
        .await
        .expect("Failed to complete command");

    // Verify
    let summary = manager.get_summary().await.expect("Failed to get summary");

    assert_eq!(summary.total_commands, 1);
    assert_eq!(summary.total_bytes_transferred, 5120);

    let recent = manager
        .get_recent_commands(1)
        .await
        .expect("Failed to get recent commands");

    assert_eq!(recent[0].command_name, "render");
    assert_eq!(recent[0].items_processed, 3);
}

/// Test crawl command metrics tracking
#[tokio::test]
async fn test_crawl_command_metrics() {
    let manager = MetricsManager::new().expect("Failed to create metrics manager");

    let tracking_id = manager
        .start_command("crawl")
        .await
        .expect("Failed to start command tracking");

    sleep(Duration::from_millis(20)).await;

    // Simulate crawling 10 pages
    manager
        .record_progress(&tracking_id, 10, 10240, 0, 1)
        .await
        .expect("Failed to record progress");

    manager
        .collector()
        .record_metric("crawl.pages", 10.0)
        .expect("Failed to record pages");

    manager
        .collector()
        .record_metric("crawl.api.latency_ms", 100.0)
        .expect("Failed to record latency");

    manager
        .complete_command(&tracking_id)
        .await
        .expect("Failed to complete command");

    // Verify
    let recent = manager
        .get_recent_commands(1)
        .await
        .expect("Failed to get recent commands");

    assert_eq!(recent[0].command_name, "crawl");
    assert_eq!(recent[0].items_processed, 10);
    assert_eq!(recent[0].bytes_transferred, 10240);
    assert_eq!(recent[0].api_calls, 1);
}

/// Test PDF command metrics tracking
#[tokio::test]
async fn test_pdf_command_metrics() {
    let manager = MetricsManager::new().expect("Failed to create metrics manager");

    // Test pdf_extract
    let tracking_id = manager
        .start_command("pdf_extract")
        .await
        .expect("Failed to start command tracking");

    sleep(Duration::from_millis(15)).await;

    manager
        .complete_command(&tracking_id)
        .await
        .expect("Failed to complete command");

    // Verify
    let recent = manager
        .get_recent_commands(1)
        .await
        .expect("Failed to get recent commands");

    assert_eq!(recent[0].command_name, "pdf_extract");
    assert!(recent[0].success);
}

/// Test command failure tracking
#[tokio::test]
async fn test_command_failure_metrics() {
    let manager = MetricsManager::new().expect("Failed to create metrics manager");

    let tracking_id = manager
        .start_command("extract")
        .await
        .expect("Failed to start command tracking");

    sleep(Duration::from_millis(5)).await;

    // Simulate failure
    manager
        .fail_command(&tracking_id, "Network timeout")
        .await
        .expect("Failed to record failure");

    // Verify
    let summary = manager.get_summary().await.expect("Failed to get summary");

    assert_eq!(summary.total_commands, 1);
    assert_eq!(summary.overall_success_rate, 0.0);

    let recent = manager
        .get_recent_commands(1)
        .await
        .expect("Failed to get recent commands");

    assert!(!recent[0].success);
    assert_eq!(recent[0].error, Some("Network timeout".to_string()));
}

/// Test metrics export functionality
#[tokio::test]
async fn test_metrics_export() {
    use riptide_cli::metrics::ExportFormat;

    let manager = MetricsManager::new().expect("Failed to create metrics manager");

    // Create some sample data
    let tracking_id = manager.start_command("extract").await.unwrap();
    manager.complete_command(&tracking_id).await.unwrap();

    // Test JSON export
    let json_export = manager
        .export(ExportFormat::Json)
        .await
        .expect("Failed to export as JSON");
    assert!(json_export.contains("extract"));

    // Test CSV export
    let csv_export = manager
        .export(ExportFormat::Csv)
        .await
        .expect("Failed to export as CSV");
    assert!(csv_export.contains("timestamp,command,duration_ms"));

    // Test Prometheus export
    let prom_export = manager
        .export(ExportFormat::Prometheus)
        .await
        .expect("Failed to export as Prometheus");
    assert!(prom_export.contains("riptide_cli_commands_total"));
}

/// Test multiple concurrent commands
#[tokio::test]
async fn test_concurrent_commands() {
    let manager = MetricsManager::new().expect("Failed to create metrics manager");

    // Start multiple commands concurrently
    let id1 = manager.start_command("extract").await.unwrap();
    let id2 = manager.start_command("render").await.unwrap();
    let id3 = manager.start_command("crawl").await.unwrap();

    sleep(Duration::from_millis(10)).await;

    // Complete them
    manager.complete_command(&id1).await.unwrap();
    manager.complete_command(&id2).await.unwrap();
    manager.complete_command(&id3).await.unwrap();

    // Verify
    let summary = manager.get_summary().await.unwrap();
    assert_eq!(summary.total_commands, 3);

    let recent = manager.get_recent_commands(10).await.unwrap();
    assert_eq!(recent.len(), 3);
}

/// Test counter operations
#[tokio::test]
async fn test_counter_operations() {
    let manager = MetricsManager::new().expect("Failed to create metrics manager");

    // Increment counters
    manager
        .increment_counter("test_counter")
        .expect("Failed to increment");
    manager
        .increment_counter("test_counter")
        .expect("Failed to increment");

    let value = manager
        .get_counter("test_counter")
        .expect("Failed to get counter");

    assert_eq!(value, 2);
}

/// Test metric time series recording
#[tokio::test]
async fn test_metric_time_series() {
    let manager = MetricsManager::new().expect("Failed to create metrics manager");

    // Record several data points
    manager
        .record_metric("latency_ms", 42.5)
        .expect("Failed to record metric");
    manager
        .record_metric("latency_ms", 38.2)
        .expect("Failed to record metric");
    manager
        .record_metric("latency_ms", 45.1)
        .expect("Failed to record metric");

    // Verify series was recorded
    let series = manager
        .collector()
        .get_metric_series("latency_ms")
        .expect("Failed to get metric series");

    assert_eq!(series.len(), 3);
    assert_eq!(series[0].value, 42.5);
    assert_eq!(series[1].value, 38.2);
    assert_eq!(series[2].value, 45.1);
}

/// Test aggregates calculation
#[tokio::test]
async fn test_metrics_aggregates() {
    let manager = MetricsManager::new().expect("Failed to create metrics manager");

    // Create multiple extract commands with different results
    for i in 0..5 {
        let tracking_id = manager.start_command("extract").await.unwrap();
        sleep(Duration::from_millis(10 + i * 5)).await;

        if i < 4 {
            manager.complete_command(&tracking_id).await.unwrap();
        } else {
            manager
                .fail_command(&tracking_id, "Test error")
                .await
                .unwrap();
        }
    }

    // Get aggregates
    let aggregates = manager
        .get_aggregates()
        .await
        .expect("Failed to get aggregates");

    let extract_agg = aggregates
        .get("extract")
        .expect("Extract aggregates not found");

    assert_eq!(extract_agg.total_executions, 5);
    assert_eq!(extract_agg.successful_executions, 4);
    assert_eq!(extract_agg.failed_executions, 1);
    assert_eq!(extract_agg.success_rate(), 80.0);
}
