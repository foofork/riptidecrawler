use crate::client::RipTideClient;
use crate::output;
use anyhow::Result;

pub async fn execute(client: RipTideClient, _output_format: &str) -> Result<()> {
    output::print_info("Performing comprehensive system check...");
    println!();

    let mut all_checks_passed = true;

    // 1. Health Check
    output::print_section("Health Status");
    match client.get("/api/health/detailed").await {
        Ok(response) => {
            if let Ok(health) = response.json::<serde_json::Value>().await {
                let healthy = health["healthy"].as_bool().unwrap_or(false);

                if healthy {
                    output::print_success("System health: HEALTHY");
                } else {
                    output::print_error("System health: UNHEALTHY");
                    all_checks_passed = false;
                }

                output::print_key_value("Redis", health["redis"].as_str().unwrap_or("unknown"));
                output::print_key_value(
                    "Extractor",
                    health["extractor"].as_str().unwrap_or("unknown"),
                );
                output::print_key_value(
                    "HTTP Client",
                    health["http_client"].as_str().unwrap_or("unknown"),
                );
                output::print_key_value(
                    "Worker Service",
                    health["worker_service"].as_str().unwrap_or("unknown"),
                );
            }
        }
        Err(e) => {
            output::print_error(&format!("Health check failed: {}", e));
            all_checks_passed = false;
        }
    }

    println!();

    // 2. Performance Status
    output::print_section("Performance Metrics");
    match client.get("/monitoring/performance-report").await {
        Ok(response) => {
            if let Ok(perf) = response.json::<serde_json::Value>().await {
                if let Some(avg_latency) = perf["average_latency_ms"].as_f64() {
                    output::print_key_value("Average Latency", &format!("{:.2}ms", avg_latency));

                    if avg_latency < 100.0 {
                        output::print_success("Latency is within acceptable range");
                    } else if avg_latency < 500.0 {
                        output::print_warning("Latency is elevated");
                    } else {
                        output::print_error("Latency is too high");
                        all_checks_passed = false;
                    }
                }

                if let Some(throughput) = perf["requests_per_second"].as_f64() {
                    output::print_key_value("Throughput", &format!("{:.2} req/s", throughput));
                }
            }
        }
        Err(e) => {
            output::print_warning(&format!("Performance metrics unavailable: {}", e));
        }
    }

    println!();

    // 3. Resource Status
    output::print_section("Resource Usage");
    match client.get("/resources/status").await {
        Ok(response) => {
            if let Ok(resources) = response.json::<serde_json::Value>().await {
                if let Some(memory) = resources["memory_usage_mb"].as_f64() {
                    output::print_key_value("Memory Usage", &format!("{:.2} MB", memory));
                }

                if let Some(connections) = resources["active_connections"].as_u64() {
                    output::print_key_value("Active Connections", &connections.to_string());
                }
            }
        }
        Err(e) => {
            output::print_warning(&format!("Resource metrics unavailable: {}", e));
        }
    }

    println!();

    // 4. Cache Status
    output::print_section("Cache Performance");
    match client.get("/admin/cache/stats").await {
        Ok(response) => {
            if let Ok(cache) = response.json::<serde_json::Value>().await {
                if let Some(hit_rate) = cache["hit_rate"].as_f64() {
                    output::print_key_value("Hit Rate", &format!("{:.2}%", hit_rate * 100.0));

                    if hit_rate > 0.8 {
                        output::print_success("Cache performance is good");
                    } else if hit_rate > 0.5 {
                        output::print_warning("Cache hit rate could be improved");
                    } else {
                        output::print_warning("Cache hit rate is low");
                    }
                }
            }
        }
        Err(e) => {
            output::print_warning(&format!("Cache metrics unavailable: {}", e));
        }
    }

    println!();

    // 5. Worker Status
    output::print_section("Worker Service");
    match client.get("/workers/stats/workers").await {
        Ok(response) => {
            if let Ok(workers) = response.json::<serde_json::Value>().await {
                if let Some(active) = workers["active_workers"].as_u64() {
                    output::print_key_value("Active Workers", &active.to_string());
                }

                if let Some(queue_size) = workers["queue_size"].as_u64() {
                    output::print_key_value("Queue Size", &queue_size.to_string());

                    if queue_size > 1000 {
                        output::print_warning("Worker queue is growing large");
                    }
                }
            }
        }
        Err(e) => {
            output::print_warning(&format!("Worker metrics unavailable: {}", e));
        }
    }

    println!();

    // Final Summary
    output::print_section("System Check Summary");
    if all_checks_passed {
        output::print_success("All critical checks passed - System is production ready!");
    } else {
        output::print_error("Some checks failed - System requires attention");
    }

    if all_checks_passed {
        Ok(())
    } else {
        anyhow::bail!("System check failed")
    }
}
