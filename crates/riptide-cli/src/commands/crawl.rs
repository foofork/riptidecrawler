use crate::client::RipTideClient;
use crate::commands::CrawlArgs;
use crate::output;
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize)]
struct CrawlRequest {
    url: String,
    max_depth: u32,
    max_pages: u32,
    follow_external_links: bool,
}

#[derive(Deserialize, Serialize)]
struct CrawlResponse {
    pages_crawled: u32,
    total_time_ms: u64,
    #[serde(default)]
    pages: Vec<PageResult>,
}

#[derive(Deserialize, Serialize)]
struct PageResult {
    url: String,
    status: u16,
    content_length: usize,
}

pub async fn execute(client: RipTideClient, args: CrawlArgs, output_format: &str) -> Result<()> {
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
        url: args.url.clone(),
        max_depth: args.depth,
        max_pages: args.max_pages,
        follow_external_links: args.follow_external,
    };

    let response = client.post("/api/v1/crawl", &request).await?;
    let result: CrawlResponse = response.json().await?;

    pb.finish_with_message("Crawl completed");

    match output_format {
        "json" => output::print_json(&result),
        "table" => {
            output::print_success(&format!(
                "Crawled {} pages in {}ms",
                result.pages_crawled, result.total_time_ms
            ));

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
            }
        }
        _ => {
            output::print_success(&format!(
                "Crawled {} pages in {}ms",
                result.pages_crawled, result.total_time_ms
            ));

            if let Some(output_dir) = args.output_dir {
                fs::create_dir_all(&output_dir)?;
                output::print_info(&format!("Saving results to: {}", output_dir));
                // Save crawl results to files
                for (idx, page) in result.pages.iter().enumerate() {
                    let filename = format!("{}/page_{}.txt", output_dir, idx + 1);
                    fs::write(&filename, &page.url)?;
                }
                output::print_success(&format!("Saved {} pages", result.pages.len()));
            }
        }
    }

    Ok(())
}
