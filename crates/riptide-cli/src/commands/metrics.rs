use crate::client::RipTideClient;
use crate::output;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;

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

pub async fn execute(client: RipTideClient, output_format: &str) -> Result<()> {
    output::print_info("Fetching system metrics...");

    let response = client.get("/monitoring/metrics/current").await?;
    let metrics: MetricsResponse = response.json().await?;

    match output_format {
        "json" => output::print_json(&metrics),
        "table" => {
            let mut table = output::create_table(vec!["Metric", "Value"]);

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
                table.add_row(vec!["Worker Queue Size", &queue_size.to_string()]);
            }

            println!("{table}");
        }
        _ => {
            output::print_success("System Metrics");
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

    Ok(())
}

pub async fn export(
    client: RipTideClient,
    format: &str,
    output_path: Option<String>,
    metric_filter: Option<String>,
) -> Result<()> {
    output::print_info("Exporting metrics...");

    let response = client.get("/monitoring/metrics/current").await?;
    let metrics: MetricsResponse = response.json().await?;

    let export_data = match format {
        "prom" | "prometheus" => format_prometheus(&metrics, metric_filter.as_deref()),
        "csv" => format_csv(&metrics, metric_filter.as_deref()),
        "json" => {
            serde_json::to_string_pretty(&metrics).context("Failed to serialize metrics to JSON")?
        }
        _ => {
            anyhow::bail!(
                "Unsupported export format: {}. Use prom, csv, or json",
                format
            );
        }
    };

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

fn format_prometheus(metrics: &MetricsResponse, filter: Option<&str>) -> String {
    let mut output = String::new();

    if should_include("requests_total", filter) {
        if let Some(total) = metrics.requests_total {
            output.push_str("# HELP riptide_requests_total Total number of requests processed\n");
            output.push_str("# TYPE riptide_requests_total counter\n");
            output.push_str(&format!("riptide_requests_total {}\n", total));
        }
    }

    if should_include("requests_per_second", filter) {
        if let Some(rps) = metrics.requests_per_second {
            output.push_str("# HELP riptide_requests_per_second Current requests per second\n");
            output.push_str("# TYPE riptide_requests_per_second gauge\n");
            output.push_str(&format!("riptide_requests_per_second {:.2}\n", rps));
        }
    }

    if should_include("average_latency_ms", filter) {
        if let Some(latency) = metrics.average_latency_ms {
            output.push_str(
                "# HELP riptide_average_latency_ms Average request latency in milliseconds\n",
            );
            output.push_str("# TYPE riptide_average_latency_ms gauge\n");
            output.push_str(&format!("riptide_average_latency_ms {:.2}\n", latency));
        }
    }

    if should_include("cache_hit_rate", filter) {
        if let Some(hit_rate) = metrics.cache_hit_rate {
            output
                .push_str("# HELP riptide_cache_hit_rate Cache hit rate as a ratio (0.0 to 1.0)\n");
            output.push_str("# TYPE riptide_cache_hit_rate gauge\n");
            output.push_str(&format!("riptide_cache_hit_rate {:.4}\n", hit_rate));
        }
    }

    if should_include("worker_queue_size", filter) {
        if let Some(queue_size) = metrics.worker_queue_size {
            output.push_str("# HELP riptide_worker_queue_size Current worker queue size\n");
            output.push_str("# TYPE riptide_worker_queue_size gauge\n");
            output.push_str(&format!("riptide_worker_queue_size {}\n", queue_size));
        }
    }

    output
}

fn format_csv(metrics: &MetricsResponse, filter: Option<&str>) -> String {
    let mut output = String::from("metric,value\n");

    if should_include("requests_total", filter) {
        if let Some(total) = metrics.requests_total {
            output.push_str(&format!("requests_total,{}\n", total));
        }
    }

    if should_include("requests_per_second", filter) {
        if let Some(rps) = metrics.requests_per_second {
            output.push_str(&format!("requests_per_second,{:.2}\n", rps));
        }
    }

    if should_include("average_latency_ms", filter) {
        if let Some(latency) = metrics.average_latency_ms {
            output.push_str(&format!("average_latency_ms,{:.2}\n", latency));
        }
    }

    if should_include("cache_hit_rate", filter) {
        if let Some(hit_rate) = metrics.cache_hit_rate {
            output.push_str(&format!("cache_hit_rate,{:.4}\n", hit_rate));
        }
    }

    if should_include("worker_queue_size", filter) {
        if let Some(queue_size) = metrics.worker_queue_size {
            output.push_str(&format!("worker_queue_size,{}\n", queue_size));
        }
    }

    output
}

fn should_include(metric_name: &str, filter: Option<&str>) -> bool {
    match filter {
        None => true,
        Some(f) => metric_name.contains(f),
    }
}
