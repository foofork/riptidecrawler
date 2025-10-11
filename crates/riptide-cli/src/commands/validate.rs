use crate::client::RipTideClient;
use crate::output;
use anyhow::Result;

pub async fn execute(client: RipTideClient, _output_format: &str) -> Result<()> {
    output::print_info("Validating system configuration...");

    let mut checks_passed = 0;
    let mut checks_failed = 0;

    // Check API connectivity
    output::print_info("Checking API connectivity...");
    match client.get("/healthz").await {
        Ok(_) => {
            output::print_success("API is reachable");
            checks_passed += 1;
        }
        Err(e) => {
            output::print_error(&format!("API connectivity check failed: {}", e));
            checks_failed += 1;
        }
    }

    // Check Redis connection
    output::print_info("Checking Redis connection...");
    match client.get("/api/health/detailed").await {
        Ok(response) => {
            if let Ok(health) = response.json::<serde_json::Value>().await {
                if health["redis"].as_str() == Some("connected") {
                    output::print_success("Redis is connected");
                    checks_passed += 1;
                } else {
                    output::print_error("Redis connection check failed");
                    checks_failed += 1;
                }
            }
        }
        Err(e) => {
            output::print_error(&format!("Redis health check failed: {}", e));
            checks_failed += 1;
        }
    }

    // Check WASM extractor
    output::print_info("Checking WASM extractor...");
    match client.get("/monitoring/wasm-instances").await {
        Ok(_) => {
            output::print_success("WASM extractor is operational");
            checks_passed += 1;
        }
        Err(e) => {
            output::print_error(&format!("WASM extractor check failed: {}", e));
            checks_failed += 1;
        }
    }

    // Check worker service
    output::print_info("Checking worker service...");
    match client.get("/workers/stats/workers").await {
        Ok(_) => {
            output::print_success("Worker service is operational");
            checks_passed += 1;
        }
        Err(e) => {
            output::print_error(&format!("Worker service check failed: {}", e));
            checks_failed += 1;
        }
    }

    println!();
    output::print_section("Validation Summary");
    output::print_key_value("Checks Passed", &checks_passed.to_string());
    output::print_key_value("Checks Failed", &checks_failed.to_string());

    if checks_failed == 0 {
        output::print_success("All validation checks passed!");
        Ok(())
    } else {
        output::print_error(&format!("{} validation checks failed", checks_failed));
        anyhow::bail!("System validation failed")
    }
}
