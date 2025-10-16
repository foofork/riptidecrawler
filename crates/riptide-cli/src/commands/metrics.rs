use crate::client::RipTideClient;
use crate::metrics::{ExportFormat, MetricsManager};
use crate::output;
use anyhow::{Context, Result};
use comfy_table::{presets::UTF8_FULL, Cell, Color, ContentArrangement, Table};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::time::Duration;

/// Response structure from the API metrics endpoint
#[derive(Deserialize, Serialize, Clone)]
struct MetricsResponse {
    #[serde(default)]
    requests_total: Option<u64>,
    #[serde(default)]
    requests_per_second: Option<f64>,
    #[serde(default)]
    average_latency_ms: Option<f64>,
    #[serde(default)]
    cache_hit_rate: Option<f64>,
    #[serde(default)]
    worker_queue_size: Option<u64>,
}

/// Display current metrics summary
pub async fn execute(client: RipTideClient, output_format: &str) -> Result<()> {
    output::print_info("Fetching system metrics...");

    // Get CLI metrics from local manager
    let metrics_manager = MetricsManager::global();
    let summary = metrics_manager
        .get_summary()
        .await
        .context("Failed to get CLI metrics summary")?;

    // Try to get server metrics
    let server_metrics = match client.get("/monitoring/metrics/current").await {
        Ok(response) => response.json::<MetricsResponse>().await.ok(),
        Err(_) => None,
    };

    match output_format {
        "json" => {
            let combined = serde_json::json!({
                "cli": summary,
                "server": server_metrics,
            });
            output::print_json(&combined);
        }
        "table" => {
            print_metrics_table(&summary, server_metrics.as_ref());
        }
        _ => {
            print_metrics_text(&summary, server_metrics.as_ref());
        }
    }

    Ok(())
}

/// Live metrics monitoring with real-time updates
pub async fn tail(
    _client: RipTideClient,
    interval_str: &str,
    limit: usize,
    output_format: &str,
) -> Result<()> {
    let interval = parse_interval(interval_str)?;
    let metrics_manager = MetricsManager::global();

    output::print_info(&format!(
        "Monitoring metrics every {}ms (Press Ctrl+C to stop)...",
        interval.as_millis()
    ));
    println!();

    // Set up Ctrl+C handler
    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, std::sync::atomic::Ordering::SeqCst);
    })
    .context("Error setting Ctrl+C handler")?;

    let mut iteration = 0;
    while running.load(std::sync::atomic::Ordering::SeqCst) {
        if iteration > 0 {
            // Clear screen and move cursor to top
            print!("\x1B[2J\x1B[1;1H");
        }

        // Get current metrics
        let summary = metrics_manager.get_summary().await?;
        let recent_commands = metrics_manager.get_recent_commands(limit).await?;

        // Display based on format
        match output_format {
            "json" => {
                let data = serde_json::json!({
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "summary": summary,
                    "recent_commands": recent_commands,
                });
                println!("{}", serde_json::to_string_pretty(&data)?);
            }
            _ => {
                print_tail_display(&summary, &recent_commands, interval_str);
            }
        }

        iteration += 1;
        tokio::time::sleep(interval).await;
    }

    output::print_success("\nMetrics monitoring stopped.");
    Ok(())
}

/// Export metrics in specified format
pub async fn export(
    client: RipTideClient,
    format: &str,
    output_path: Option<String>,
    metric_filter: Option<String>,
) -> Result<()> {
    output::print_info("Exporting metrics...");

    let metrics_manager = MetricsManager::global();

    // Determine export format
    let export_format = match format.to_lowercase().as_str() {
        "prom" | "prometheus" => ExportFormat::Prometheus,
        "csv" => ExportFormat::Csv,
        "json" => ExportFormat::Json,
        _ => {
            anyhow::bail!(
                "Unsupported export format: {}. Use prom, csv, or json",
                format
            );
        }
    };

    // Export from local metrics manager
    let mut export_data = metrics_manager.export(export_format).await?;

    // Apply filter if specified
    if let Some(filter) = metric_filter {
        export_data = filter_metrics(&export_data, &filter);
    }

    // Try to also include server metrics if available
    if let Ok(response) = client.get("/monitoring/metrics/current").await {
        if let Ok(server_metrics) = response.json::<MetricsResponse>().await {
            let server_export = format_server_metrics(&server_metrics, format);
            if !server_export.is_empty() {
                export_data.push_str("\n\n# Server Metrics\n");
                export_data.push_str(&server_export);
            }
        }
    }

    // Write to file or stdout
    if let Some(path) = output_path {
        let mut file =
            File::create(&path).context(format!("Failed to create output file: {}", path))?;
        file.write_all(export_data.as_bytes())
            .context("Failed to write metrics to file")?;
        output::print_success(&format!("Metrics exported to: {}", path));
    } else {
        println!("{}", export_data);
    }

    Ok(())
}

/// Print metrics in table format
fn print_metrics_table(
    summary: &crate::metrics::CliMetricsSummary,
    server: Option<&MetricsResponse>,
) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Metric")
                .fg(Color::Cyan)
                .set_alignment(comfy_table::CellAlignment::Left),
            Cell::new("Value").fg(Color::Green),
        ]);

    // CLI Metrics Section
    table.add_row(vec![
        Cell::new("CLI METRICS").fg(Color::Yellow),
        Cell::new(""),
    ]);
    table.add_row(vec!["Total Commands", &summary.total_commands.to_string()]);
    table.add_row(vec![
        "Success Rate",
        &format!("{:.2}%", summary.overall_success_rate),
    ]);
    table.add_row(vec![
        "Avg Duration",
        &format!("{:.2}ms", summary.avg_command_duration_ms),
    ]);
    table.add_row(vec![
        "Total Bytes",
        &format_bytes(summary.total_bytes_transferred),
    ]);
    table.add_row(vec!["API Calls", &summary.total_api_calls.to_string()]);

    // Server Metrics Section (if available)
    if let Some(metrics) = server {
        table.add_row(vec![Cell::new(""), Cell::new("")]);
        table.add_row(vec![
            Cell::new("SERVER METRICS").fg(Color::Yellow),
            Cell::new(""),
        ]);

        if let Some(total) = metrics.requests_total {
            table.add_row(vec!["Total Requests", &total.to_string()]);
        }
        if let Some(rps) = metrics.requests_per_second {
            table.add_row(vec!["Requests/Second", &format!("{:.2}", rps)]);
        }
        if let Some(latency) = metrics.average_latency_ms {
            table.add_row(vec!["Avg Latency", &format!("{:.2}ms", latency)]);
        }
        if let Some(hit_rate) = metrics.cache_hit_rate {
            table.add_row(vec!["Cache Hit Rate", &format!("{:.2}%", hit_rate * 100.0)]);
        }
        if let Some(queue_size) = metrics.worker_queue_size {
            table.add_row(vec!["Worker Queue", &queue_size.to_string()]);
        }
    }

    println!("{table}");
}

/// Print metrics in text format
fn print_metrics_text(
    summary: &crate::metrics::CliMetricsSummary,
    server: Option<&MetricsResponse>,
) {
    output::print_success("CLI Metrics Summary");
    println!();

    output::print_key_value("Total Commands", &summary.total_commands.to_string());
    output::print_key_value(
        "Success Rate",
        &format!("{:.2}%", summary.overall_success_rate),
    );
    output::print_key_value(
        "Average Duration",
        &format!("{:.2}ms", summary.avg_command_duration_ms),
    );
    output::print_key_value(
        "Total Bytes Transferred",
        &format_bytes(summary.total_bytes_transferred),
    );
    output::print_key_value("API Calls", &summary.total_api_calls.to_string());

    if let Some(metrics) = server {
        println!();
        output::print_success("Server Metrics");
        println!();

        if let Some(total) = metrics.requests_total {
            output::print_key_value("Total Requests", &total.to_string());
        }
        if let Some(rps) = metrics.requests_per_second {
            output::print_key_value("Requests/Second", &format!("{:.2}", rps));
        }
        if let Some(latency) = metrics.average_latency_ms {
            output::print_key_value("Average Latency", &format!("{:.2}ms", latency));
        }
        if let Some(hit_rate) = metrics.cache_hit_rate {
            output::print_key_value("Cache Hit Rate", &format!("{:.2}%", hit_rate * 100.0));
        }
        if let Some(queue_size) = metrics.worker_queue_size {
            output::print_key_value("Worker Queue Size", &queue_size.to_string());
        }
    }
}

/// Print live tail display
fn print_tail_display(
    summary: &crate::metrics::CliMetricsSummary,
    recent_commands: &[crate::metrics::CommandMetrics],
    interval: &str,
) {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!(
        "  RipTide Metrics Monitor (updating every {})  {}",
        interval,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    // Summary stats
    println!("📊 SUMMARY");
    println!(
        "   Commands: {}  |  Success: {:.1}%  |  Avg: {:.0}ms",
        summary.total_commands, summary.overall_success_rate, summary.avg_command_duration_ms
    );
    println!(
        "   Transferred: {}  |  API Calls: {}",
        format_bytes(summary.total_bytes_transferred),
        summary.total_api_calls
    );
    println!();

    // Recent commands table
    println!("🕒 RECENT COMMANDS");
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Time").fg(Color::Cyan),
            Cell::new("Command").fg(Color::Cyan),
            Cell::new("Duration").fg(Color::Cyan),
            Cell::new("Status").fg(Color::Cyan),
            Cell::new("Items").fg(Color::Cyan),
        ]);

    for cmd in recent_commands.iter().take(10) {
        let time = cmd.started_at.format("%H:%M:%S").to_string();
        let duration = cmd
            .duration_ms
            .map(|d| format!("{}ms", d))
            .unwrap_or_else(|| "-".to_string());
        let status = if cmd.success {
            Cell::new("✓ OK").fg(Color::Green)
        } else {
            Cell::new("✗ FAIL").fg(Color::Red)
        };

        table.add_row(vec![
            Cell::new(time),
            Cell::new(&cmd.command_name),
            Cell::new(duration),
            status,
            Cell::new(cmd.items_processed),
        ]);
    }

    println!("{}", table);
    println!();
    println!("Press Ctrl+C to stop monitoring");
}

/// Format server metrics for export
fn format_server_metrics(metrics: &MetricsResponse, format: &str) -> String {
    match format.to_lowercase().as_str() {
        "prom" | "prometheus" => {
            let mut output = String::new();
            if let Some(total) = metrics.requests_total {
                output.push_str("# HELP riptide_server_requests_total Total server requests\n");
                output.push_str("# TYPE riptide_server_requests_total counter\n");
                output.push_str(&format!("riptide_server_requests_total {}\n", total));
            }
            if let Some(rps) = metrics.requests_per_second {
                output.push_str("# HELP riptide_server_rps Server requests per second\n");
                output.push_str("# TYPE riptide_server_rps gauge\n");
                output.push_str(&format!("riptide_server_rps {:.2}\n", rps));
            }
            if let Some(latency) = metrics.average_latency_ms {
                output.push_str("# HELP riptide_server_latency_ms Server average latency\n");
                output.push_str("# TYPE riptide_server_latency_ms gauge\n");
                output.push_str(&format!("riptide_server_latency_ms {:.2}\n", latency));
            }
            output
        }
        "csv" => {
            let mut output = String::from("metric,value\n");
            if let Some(total) = metrics.requests_total {
                output.push_str(&format!("server_requests_total,{}\n", total));
            }
            if let Some(rps) = metrics.requests_per_second {
                output.push_str(&format!("server_rps,{:.2}\n", rps));
            }
            if let Some(latency) = metrics.average_latency_ms {
                output.push_str(&format!("server_latency_ms,{:.2}\n", latency));
            }
            output
        }
        "json" => serde_json::to_string_pretty(metrics).unwrap_or_default(),
        _ => String::new(),
    }
}

/// Filter metrics output based on pattern
fn filter_metrics(data: &str, filter: &str) -> String {
    data.lines()
        .filter(|line| line.contains(filter))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Parse interval string to Duration
fn parse_interval(interval: &str) -> Result<Duration> {
    let interval = interval.trim().to_lowercase();

    if let Some(ms_str) = interval.strip_suffix("ms") {
        let ms: u64 = ms_str.parse().context("Invalid interval format")?;
        return Ok(Duration::from_millis(ms));
    }

    if let Some(s_str) = interval.strip_suffix('s') {
        let s: u64 = s_str.parse().context("Invalid interval format")?;
        return Ok(Duration::from_secs(s));
    }

    // Default to treating as seconds
    let s: u64 = interval.parse().context("Invalid interval format")?;
    Ok(Duration::from_secs(s))
}

/// Format bytes into human-readable form
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_idx])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_interval() {
        assert_eq!(parse_interval("1s").unwrap(), Duration::from_secs(1));
        assert_eq!(parse_interval("500ms").unwrap(), Duration::from_millis(500));
        assert_eq!(parse_interval("2s").unwrap(), Duration::from_secs(2));
        assert_eq!(
            parse_interval("1000ms").unwrap(),
            Duration::from_millis(1000)
        );
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0.00 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1536), "1.50 KB");
    }

    #[test]
    fn test_filter_metrics() {
        let data = "metric1,100\nmetric2,200\nother,300";
        let filtered = filter_metrics(data, "metric");
        assert!(filtered.contains("metric1"));
        assert!(filtered.contains("metric2"));
        assert!(!filtered.contains("other"));
    }
}
