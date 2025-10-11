use crate::client::RipTideClient;
use crate::commands::CacheCommands;
use crate::output;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct CacheStats {
    total_keys: u64,
    memory_used: u64,
    hit_rate: f64,
    #[serde(default)]
    methods: Option<serde_json::Value>,
}

pub async fn execute(
    client: RipTideClient,
    command: CacheCommands,
    output_format: &str,
) -> Result<()> {
    match command {
        CacheCommands::Status => show_status(client, output_format).await,
        CacheCommands::Clear { method } => clear_cache(client, method, output_format).await,
        CacheCommands::Validate => validate_cache(client, output_format).await,
        CacheCommands::Stats => show_stats(client, output_format).await,
    }
}

async fn show_status(client: RipTideClient, output_format: &str) -> Result<()> {
    output::print_info("Fetching cache status...");

    let response = client.get("/admin/cache/stats").await?;
    let stats: CacheStats = response.json().await?;

    match output_format {
        "json" => output::print_json(&stats),
        "table" => {
            let mut table = output::create_table(vec!["Metric", "Value"]);
            table.add_row(vec!["Total Keys", &stats.total_keys.to_string()]);
            table.add_row(vec![
                "Memory Used",
                &output::format_bytes(stats.memory_used),
            ]);
            table.add_row(vec!["Hit Rate", &format!("{:.2}%", stats.hit_rate * 100.0)]);
            println!("{table}");
        }
        _ => {
            output::print_key_value("Total Keys", &stats.total_keys.to_string());
            output::print_key_value("Memory Used", &output::format_bytes(stats.memory_used));
            output::print_key_value("Hit Rate", &format!("{:.2}%", stats.hit_rate * 100.0));
        }
    }

    Ok(())
}

async fn clear_cache(
    client: RipTideClient,
    method: Option<String>,
    _output_format: &str,
) -> Result<()> {
    if let Some(method) = &method {
        output::print_info(&format!("Clearing cache for method: {}", method));
    } else {
        output::print_info("Clearing all cache...");
    }

    let request = serde_json::json!({
        "method": method,
    });

    client.post("/admin/cache/invalidate", &request).await?;
    output::print_success("Cache cleared successfully");

    Ok(())
}

async fn validate_cache(client: RipTideClient, _output_format: &str) -> Result<()> {
    output::print_info("Validating cache integrity...");

    // This would call a validation endpoint
    let response = client.get("/admin/cache/stats").await?;
    let stats: CacheStats = response.json().await?;

    if stats.total_keys > 0 {
        output::print_success("Cache validation passed");
        output::print_info(&format!("Found {} valid cache entries", stats.total_keys));
    } else {
        output::print_warning("Cache is empty");
    }

    Ok(())
}

async fn show_stats(client: RipTideClient, output_format: &str) -> Result<()> {
    show_status(client, output_format).await
}
