use crate::cache::{Cache, CacheConfig, WarmOptions};
use crate::client::RipTideClient;
use crate::commands::CacheCommands;
use crate::output;
use anyhow::{Context, Result};
use std::path::Path;
use tokio::fs;

pub async fn execute(
    _client: RipTideClient,
    command: CacheCommands,
    output_format: &str,
) -> Result<()> {
    match command {
        CacheCommands::Status => show_status(output_format).await,
        CacheCommands::Clear { domain } => clear_cache(domain, output_format).await,
        CacheCommands::Warm { url_file } => warm_cache(url_file, output_format).await,
        CacheCommands::Validate => validate_cache(output_format).await,
        CacheCommands::Stats => show_stats(output_format).await,
    }
}

async fn show_status(output_format: &str) -> Result<()> {
    output::print_info("Fetching cache status...");

    let cache = Cache::new().await?;
    let stats = cache.get_stats().await?;

    match output_format {
        "json" => {
            let json = serde_json::to_string_pretty(&stats)?;
            println!("{}", json);
        }
        "table" => {
            let mut table = output::create_table(vec!["Metric", "Value"]);
            table.add_row(vec!["Total Entries", &stats.total_entries.to_string()]);
            table.add_row(vec![
                "Total Size",
                &output::format_bytes(stats.total_size_bytes),
            ]);
            table.add_row(vec!["Cache Hits", &stats.hits.to_string()]);
            table.add_row(vec!["Cache Misses", &stats.misses.to_string()]);
            table.add_row(vec![
                "Hit Rate",
                &format!("{:.2}%", stats.hit_rate() * 100.0),
            ]);
            table.add_row(vec!["Evictions", &stats.evictions.to_string()]);
            table.add_row(vec!["Insertions", &stats.insertions.to_string()]);

            println!("{table}");

            // Domain breakdown
            if !stats.entries_by_domain.is_empty() {
                println!("\nCache by Domain:");
                let mut domain_table = output::create_table(vec!["Domain", "Entries", "Size"]);

                let mut domains: Vec<_> = stats.entries_by_domain.iter().collect();
                domains.sort_by(|a, b| b.1.cmp(a.1)); // Sort by entry count

                for (domain, count) in domains.iter().take(10) {
                    let size = stats.size_by_domain.get(*domain).unwrap_or(&0);
                    domain_table.add_row(vec![
                        domain.as_str(),
                        &count.to_string(),
                        &output::format_bytes(*size),
                    ]);
                }

                println!("{domain_table}");
            }
        }
        _ => {
            output::print_key_value("Total Entries", &stats.total_entries.to_string());
            output::print_key_value("Total Size", &output::format_bytes(stats.total_size_bytes));
            output::print_key_value("Cache Hits", &stats.hits.to_string());
            output::print_key_value("Cache Misses", &stats.misses.to_string());
            output::print_key_value("Hit Rate", &format!("{:.2}%", stats.hit_rate() * 100.0));
            output::print_key_value("Evictions", &stats.evictions.to_string());
            output::print_key_value("Insertions", &stats.insertions.to_string());

            if !stats.entries_by_domain.is_empty() {
                println!("\nTop Domains:");
                let mut domains: Vec<_> = stats.entries_by_domain.iter().collect();
                domains.sort_by(|a, b| b.1.cmp(a.1));

                for (domain, count) in domains.iter().take(5) {
                    let size = stats.size_by_domain.get(*domain).unwrap_or(&0);
                    println!(
                        "  {} - {} entries ({})",
                        domain,
                        count,
                        output::format_bytes(*size)
                    );
                }
            }
        }
    }

    Ok(())
}

async fn clear_cache(domain: Option<String>, output_format: &str) -> Result<()> {
    let cache = Cache::new().await?;

    if let Some(domain) = domain {
        output::print_info(&format!("Clearing cache for domain: {}", domain));
        let count = cache.clear_domain(&domain).await?;
        output::print_success(&format!("Cleared {} entries for {}", count, domain));
    } else {
        output::print_info("Clearing all cache...");
        cache.clear().await?;
        output::print_success("Cache cleared successfully");
    }

    let stats = cache.get_stats().await?;

    if output_format == "json" {
        let result = serde_json::json!({
            "cleared": true,
            "remaining_entries": stats.total_entries
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    }

    Ok(())
}

async fn warm_cache(url_file: String, output_format: &str) -> Result<()> {
    output::print_info(&format!("Reading URLs from: {}", url_file));

    // Read URLs from file
    let content = fs::read_to_string(&url_file)
        .await
        .context("Failed to read URL file")?;

    let urls: Vec<String> = content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.to_string())
        .collect();

    if urls.is_empty() {
        output::print_warning("No URLs found in file");
        return Ok(());
    }

    output::print_info(&format!("Found {} URLs to warm", urls.len()));

    let cache = Cache::new().await?;

    let options = WarmOptions {
        urls,
        concurrency: 10,
        timeout_seconds: 30,
        retry_failures: true,
        max_retries: 3,
    };

    // Show progress
    output::print_info("Warming cache...");
    let result = cache.warm(options).await?;

    match output_format {
        "json" => {
            let json = serde_json::json!({
                "total_urls": result.total_urls,
                "successful": result.successful,
                "failed": result.failed,
                "success_rate": result.success_rate()
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        "table" => {
            let mut table = output::create_table(vec!["Metric", "Value"]);
            table.add_row(vec!["Total URLs", &result.total_urls.to_string()]);
            table.add_row(vec!["Successful", &result.successful.to_string()]);
            table.add_row(vec!["Failed", &result.failed.to_string()]);
            table.add_row(vec![
                "Success Rate",
                &format!("{:.2}%", result.success_rate()),
            ]);
            println!("{table}");
        }
        _ => {
            output::print_success(&format!(
                "Cache warming complete: {}/{} succeeded ({:.1}% success rate)",
                result.successful,
                result.total_urls,
                result.success_rate()
            ));
            if result.failed > 0 {
                output::print_warning(&format!("{} URLs failed to cache", result.failed));
            }
        }
    }

    Ok(())
}

async fn validate_cache(output_format: &str) -> Result<()> {
    output::print_info("Validating cache integrity...");

    let cache = Cache::new().await?;

    // Clean up expired entries
    let expired = cache.cleanup_expired().await?;
    if expired > 0 {
        output::print_info(&format!("Removed {} expired entries", expired));
    }

    let stats = cache.get_stats().await?;
    let urls = cache.list_urls().await?;

    // Validate each entry
    let mut valid_count = 0;
    let mut invalid_count = 0;

    for url in &urls {
        match cache.get(url).await {
            Ok(Some(_)) => valid_count += 1,
            _ => invalid_count += 1,
        }
    }

    match output_format {
        "json" => {
            let result = serde_json::json!({
                "valid": true,
                "total_entries": stats.total_entries,
                "valid_entries": valid_count,
                "invalid_entries": invalid_count,
                "expired_removed": expired
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        "table" => {
            let mut table = output::create_table(vec!["Check", "Result"]);
            table.add_row(vec!["Total Entries", &stats.total_entries.to_string()]);
            table.add_row(vec!["Valid Entries", &valid_count.to_string()]);
            table.add_row(vec!["Invalid Entries", &invalid_count.to_string()]);
            table.add_row(vec!["Expired Removed", &expired.to_string()]);
            println!("{table}");
        }
        _ => {
            output::print_success("Cache validation completed");
            output::print_key_value("Total Entries", &stats.total_entries.to_string());
            output::print_key_value("Valid Entries", &valid_count.to_string());
            if invalid_count > 0 {
                output::print_warning(&format!("{} invalid entries found", invalid_count));
            }
            if expired > 0 {
                output::print_info(&format!("Removed {} expired entries", expired));
            }
        }
    }

    Ok(())
}

async fn show_stats(output_format: &str) -> Result<()> {
    show_status(output_format).await
}
