use crate::client::RipTideClient;
use crate::output;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct HealthResponse {
    status: String,
    healthy: bool,
    redis: String,
    extractor: String,
    http_client: String,
    worker_service: String,
    #[serde(default)]
    uptime_seconds: Option<u64>,
}

pub async fn execute(client: RipTideClient, output_format: &str) -> Result<()> {
    output::print_info("Checking system health...");

    let response = client.get("/api/health/detailed").await?;
    let health: HealthResponse = response.json().await?;

    match output_format {
        "json" => output::print_json(&health),
        "table" => {
            let mut table = output::create_table(vec!["Component", "Status"]);
            table.add_row(vec!["Overall", &health.status]);
            table.add_row(vec!["Redis", &health.redis]);
            table.add_row(vec!["Extractor", &health.extractor]);
            table.add_row(vec!["HTTP Client", &health.http_client]);
            table.add_row(vec!["Worker Service", &health.worker_service]);

            if let Some(uptime) = health.uptime_seconds {
                table.add_row(vec!["Uptime", &output::format_duration(uptime)]);
            }

            println!("{table}");
        }
        _ => {
            if health.healthy {
                output::print_success("System is healthy");
            } else {
                output::print_error("System health check failed");
            }

            output::print_key_value("Redis", &health.redis);
            output::print_key_value("Extractor", &health.extractor);
            output::print_key_value("HTTP Client", &health.http_client);
            output::print_key_value("Worker Service", &health.worker_service);

            if let Some(uptime) = health.uptime_seconds {
                output::print_key_value("Uptime", &output::format_duration(uptime));
            }
        }
    }

    Ok(())
}
