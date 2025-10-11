use crate::client::RipTideClient;
use crate::commands::WasmCommands;
use crate::output;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct WasmInfo {
    version: String,
    instances: u32,
    memory_usage: u64,
    #[serde(default)]
    features: Vec<String>,
}

#[derive(Deserialize, Serialize)]
struct BenchmarkResult {
    iterations: u32,
    total_time_ms: u64,
    avg_time_ms: f64,
    min_time_ms: u64,
    max_time_ms: u64,
    throughput: f64,
}

pub async fn execute(
    client: RipTideClient,
    command: WasmCommands,
    output_format: &str,
) -> Result<()> {
    match command {
        WasmCommands::Info => show_info(client, output_format).await,
        WasmCommands::Benchmark { iterations } => {
            run_benchmark(client, iterations, output_format).await
        }
        WasmCommands::Health => show_health(client, output_format).await,
    }
}

async fn show_info(client: RipTideClient, output_format: &str) -> Result<()> {
    output::print_info("Fetching WASM runtime information...");

    let response = client.get("/monitoring/wasm-instances").await?;
    let info: WasmInfo = response.json().await?;

    match output_format {
        "json" => output::print_json(&info),
        "table" => {
            let mut table = output::create_table(vec!["Property", "Value"]);
            table.add_row(vec!["Version", &info.version]);
            table.add_row(vec!["Active Instances", &info.instances.to_string()]);
            table.add_row(vec![
                "Memory Usage",
                &output::format_bytes(info.memory_usage),
            ]);
            table.add_row(vec!["Features", &info.features.join(", ")]);
            println!("{table}");
        }
        _ => {
            output::print_key_value("WASM Version", &info.version);
            output::print_key_value("Active Instances", &info.instances.to_string());
            output::print_key_value("Memory Usage", &output::format_bytes(info.memory_usage));
            if !info.features.is_empty() {
                output::print_key_value("Features", &info.features.join(", "));
            }
        }
    }

    Ok(())
}

async fn run_benchmark(client: RipTideClient, iterations: u32, output_format: &str) -> Result<()> {
    output::print_info(&format!(
        "Running WASM benchmark ({} iterations)...",
        iterations
    ));

    let request = serde_json::json!({
        "iterations": iterations,
        "type": "wasm"
    });

    let response = client.post("/api/profiling/benchmark", &request).await?;
    let result: BenchmarkResult = response.json().await?;

    match output_format {
        "json" => output::print_json(&result),
        "table" => {
            let mut table = output::create_table(vec!["Metric", "Value"]);
            table.add_row(vec!["Iterations", &result.iterations.to_string()]);
            table.add_row(vec!["Total Time", &format!("{}ms", result.total_time_ms)]);
            table.add_row(vec![
                "Average Time",
                &format!("{:.2}ms", result.avg_time_ms),
            ]);
            table.add_row(vec!["Min Time", &format!("{}ms", result.min_time_ms)]);
            table.add_row(vec!["Max Time", &format!("{}ms", result.max_time_ms)]);
            table.add_row(vec![
                "Throughput",
                &format!("{:.2} ops/sec", result.throughput),
            ]);
            println!("{table}");
        }
        _ => {
            output::print_success("Benchmark completed");
            output::print_key_value("Iterations", &result.iterations.to_string());
            output::print_key_value("Average Time", &format!("{:.2}ms", result.avg_time_ms));
            output::print_key_value("Throughput", &format!("{:.2} ops/sec", result.throughput));
        }
    }

    Ok(())
}

async fn show_health(client: RipTideClient, output_format: &str) -> Result<()> {
    show_info(client, output_format).await
}
