use crate::client::RipTideClient;
use crate::commands::CrawlArgs;
use crate::output;
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize)]
struct CrawlRequest {
    urls: Vec<String>,
    max_depth: u32,
    max_pages: u32,
    follow_external_links: bool,
}

#[derive(Deserialize, Serialize)]
struct CrawlResponse {
    #[serde(default)]
    pages_crawled: Option<u64>,
    #[serde(default)]
    total_urls: Option<usize>,
    #[serde(default)]
    successful: Option<usize>,
    #[serde(default)]
    failed: Option<usize>,
    #[serde(default)]
    total_time_ms: u64,
    #[serde(default)]
    pages: Vec<PageResult>,
    #[serde(default)]
    results: Vec<CrawlResultItem>,
}

#[derive(Deserialize, Serialize)]
struct CrawlResultItem {
    url: String,
    status: u16,
    #[serde(default)]
    from_cache: bool,
    #[serde(default)]
    gate_decision: String,
}

#[derive(Deserialize, Serialize)]
struct PageResult {
    url: String,
    status: u16,
    content_length: usize,
}

pub async fn execute(client: RipTideClient, args: CrawlArgs, output_format: &str) -> Result<()> {
    use crate::metrics::MetricsManager;
    use std::time::Instant;

    // Start metrics tracking
    let metrics_manager = MetricsManager::global();
    let tracking_id = metrics_manager.start_command("crawl").await?;
    let _overall_start = Instant::now();

    output::print_info(&format!("Starting crawl of: {}", args.url));
    output::print_info(&format!(
        "Max depth: {}, Max pages: {}",
        args.depth, args.max_pages
    ));

    let pb = ProgressBar::new(args.max_pages as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("█▓▒░ "),
    );

    let request = CrawlRequest {
        urls: vec![args.url.clone()],
        max_depth: args.depth,
        max_pages: args.max_pages,
        follow_external_links: args.follow_external,
    };

    let api_start = Instant::now();
    let response = client.post("/api/v1/crawl", &request).await?;
    let result: CrawlResponse = response.json().await?;
    let api_latency = api_start.elapsed();

    pb.finish_with_message("Crawl completed");

    // Determine the actual number of pages crawled (supports both API formats)
    let pages_count = result
        .pages_crawled
        .or(result.successful.map(|s| s as u64))
        .unwrap_or(result.pages.len() as u64 + result.results.len() as u64);

    // Record metrics
    let total_bytes: u64 = result.pages.iter().map(|p| p.content_length as u64).sum();
    metrics_manager
        .record_progress(&tracking_id, pages_count, total_bytes, 0, 1)
        .await?;
    metrics_manager
        .collector()
        .record_metric("crawl.api.latency_ms", api_latency.as_millis() as f64)?;
    metrics_manager
        .collector()
        .record_metric("crawl.pages", pages_count as f64)?;
    metrics_manager
        .collector()
        .record_metric("crawl.duration_ms", result.total_time_ms as f64)?;

    match output_format {
        "json" => output::print_json(&result),
        "table" => {
            output::print_success(&format!(
                "Crawled {} pages in {}ms",
                pages_count, result.total_time_ms
            ));

            // Display results from either format
            if !result.pages.is_empty() {
                let mut table = output::create_table(vec!["URL", "Status", "Size"]);
                for page in &result.pages {
                    table.add_row(vec![
                        &page.url,
                        &page.status.to_string(),
                        &output::format_bytes(page.content_length as u64),
                    ]);
                }
                println!("{table}");
            } else if !result.results.is_empty() {
                let mut table =
                    output::create_table(vec!["URL", "Status", "From Cache", "Decision"]);
                for item in &result.results {
                    table.add_row(vec![
                        &item.url,
                        &item.status.to_string(),
                        &(if item.from_cache {
                            "Yes".to_string()
                        } else {
                            "No".to_string()
                        }),
                        &item.gate_decision,
                    ]);
                }
                println!("{table}");
            }
        }
        _ => {
            output::print_success(&format!(
                "Crawled {} pages in {}ms",
                pages_count, result.total_time_ms
            ));

            if let Some(ref output_dir_arg) = args.output_dir {
                let output_dir = output_dir_arg;
                fs::create_dir_all(output_dir)?;
                output::print_info(&format!("Saving results to: {}", output_dir));

                // Save crawl results to files (support both formats)
                if !result.pages.is_empty() {
                    for (idx, page) in result.pages.iter().enumerate() {
                        let filename = format!("{}/page_{}.txt", output_dir, idx + 1);
                        fs::write(&filename, &page.url)?;
                    }
                    output::print_success(&format!("Saved {} pages", result.pages.len()));
                } else if !result.results.is_empty() {
                    for (idx, item) in result.results.iter().enumerate() {
                        let filename = format!("{}/page_{}.txt", output_dir, idx + 1);
                        fs::write(&filename, &item.url)?;
                    }
                    output::print_success(&format!("Saved {} pages", result.results.len()));
                }
            }
        }
    }

    // Complete metrics tracking
    metrics_manager.complete_command(&tracking_id).await?;

    Ok(())
}
