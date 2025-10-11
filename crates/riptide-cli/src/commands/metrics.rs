use crate::client::RipTideClient;
use crate::output;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
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
