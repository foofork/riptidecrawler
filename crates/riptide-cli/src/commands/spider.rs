/// Spider command - Web crawling with configurable strategies
///
/// This command crawls websites starting from a seed URL and following links
/// up to a specified depth with various crawling strategies.
use crate::client::ApiClient;
use crate::output::{self, format_size, truncate_text, OutputFormat};
use anyhow::{Context, Result};
use clap::Args;
use serde::{Deserialize, Serialize};
use std::fs;

/// Arguments for the spider command
#[derive(Args, Clone, Debug)]
pub struct SpiderArgs {
    /// Starting URL for spider crawl
    #[arg(required = true)]
    pub seed: String,

    /// Maximum depth to crawl
    #[arg(long, short = 'd', default_value = "3")]
    pub depth: u32,

    /// Maximum pages to crawl
    #[arg(long, short = 'p', default_value = "100")]
    pub pages: u32,

    /// Crawl strategy (breadth_first/depth_first/best_first)
    #[arg(long, default_value = "breadth_first")]
    pub strategy: String,

    /// Number of concurrent requests
    #[arg(long, short = 'c', default_value = "5")]
    pub concurrency: u32,

    /// Request timeout in seconds
    #[arg(long, short = 't', default_value = "30")]
    pub timeout: u64,

    /// Cache mode (auto/read_write/read_only/write_only/disabled)
    #[arg(long, default_value = "auto")]
    pub cache: String,

    /// Save results to file
    #[arg(long, short = 'f')]
    pub output_file: Option<String>,

    /// robots.txt handling (respect/ignore)
    #[arg(long, default_value = "respect")]
    pub robots: String,
}

/// Request payload sent to the API
#[derive(Serialize, Debug)]
struct SpiderRequest {
    seed_url: String,
    max_depth: u32,
    max_pages: u32,
    strategy: String,
    concurrency: u32,
    timeout_seconds: u64,
    cache_mode: String,
    robots_txt: String,
}

/// Response from the API
#[derive(Deserialize, Serialize, Debug)]
pub struct SpiderResponse {
    pub crawl_id: Option<String>,
    pub pages: Vec<CrawledPage>,
    pub summary: CrawlSummary,
}

/// Single crawled page result
#[derive(Deserialize, Serialize, Debug)]
pub struct CrawledPage {
    pub url: String,
    pub depth: u32,
    pub status: String,
    pub status_code: Option<u16>,
    pub title: Option<String>,
    pub content_size: Option<usize>,
    pub links_found: Option<usize>,
    pub crawl_time_ms: Option<u64>,
    pub error: Option<String>,
}

/// Summary of spider crawl job
#[derive(Deserialize, Serialize, Debug)]
pub struct CrawlSummary {
    pub total_pages: usize,
    pub successful: usize,
    pub failed: usize,
    pub max_depth_reached: u32,
    pub total_links_found: usize,
    pub total_size: usize,
    pub total_time_ms: u64,
}

/// Execute the spider command
pub async fn execute(client: ApiClient, args: SpiderArgs, output_format: String) -> Result<()> {
    // Validate arguments
    validate_args(&args)?;

    // Build request payload
    let request = SpiderRequest {
        seed_url: args.seed.clone(),
        max_depth: args.depth,
        max_pages: args.pages,
        strategy: args.strategy.clone(),
        concurrency: args.concurrency,
        timeout_seconds: args.timeout,
        cache_mode: args.cache.clone(),
        robots_txt: args.robots.clone(),
    };

    // Print progress info
    output::print_info(&format!(
        "Starting spider crawl from {} (max depth: {}, max pages: {}, strategy: {})",
        request.seed_url, request.max_depth, request.max_pages, request.strategy
    ));

    // Send request to API
    let response = client
        .post::<SpiderRequest, SpiderResponse>("/spider/crawl", &request)
        .await
        .context("Failed to execute spider crawl via API")?;

    // Save to file if specified
    if let Some(output_file) = &args.output_file {
        save_to_file(output_file, &response)?;
        output::print_success(&format!("Results saved to {}", output_file));
    }

    // Format and print output
    let format = OutputFormat::parse(&output_format)?;
    print_results(&response, format)?;

    // Exit with error if any pages failed
    if response.summary.failed > 0 {
        anyhow::bail!(
            "Crawl completed with {} failed page(s)",
            response.summary.failed
        );
    }

    Ok(())
}

/// Validate command arguments
fn validate_args(args: &SpiderArgs) -> Result<()> {
    // Basic URL format check - prevents obviously broken requests
    if !args.seed.starts_with("http://") && !args.seed.starts_with("https://") {
        anyhow::bail!("Seed URL must start with http:// or https://");
    }

    // Prevent nonsensical values
    if args.depth == 0 {
        anyhow::bail!("Depth must be at least 1");
    }

    if args.pages == 0 {
        anyhow::bail!("Pages must be at least 1");
    }

    if args.concurrency == 0 {
        anyhow::bail!("Concurrency must be at least 1");
    }

    if args.timeout == 0 {
        anyhow::bail!("Timeout must be at least 1 second");
    }

    Ok(())
}

/// Save results to file in JSON format
fn save_to_file(path: &str, response: &SpiderResponse) -> Result<()> {
    let json =
        serde_json::to_string_pretty(response).context("Failed to serialize results to JSON")?;

    fs::write(path, json).context(format!("Failed to write results to {}", path))?;

    Ok(())
}

/// Print spider results in the specified format
fn print_results(response: &SpiderResponse, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(response)?;
            println!("{}", json);
        }
        OutputFormat::Table => {
            print_table(response)?;
        }
        OutputFormat::Text => {
            print_text(response)?;
        }
        OutputFormat::Stream => {
            // For spider command, stream format is same as JSON
            let json = serde_json::to_string_pretty(response)?;
            println!("{}", json);
        }
    }
    Ok(())
}

/// Print results as a formatted table
fn print_table(response: &SpiderResponse) -> Result<()> {
    use comfy_table::modifiers::UTF8_ROUND_CORNERS;
    use comfy_table::presets::UTF8_FULL;
    use comfy_table::{Cell, Color, Table};

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            "URL", "Depth", "Status", "Code", "Title", "Links", "Size",
        ]);

    for page in &response.pages {
        let status_cell = if page.status == "success" {
            Cell::new(&page.status).fg(Color::Green)
        } else {
            Cell::new(&page.status).fg(Color::Red)
        };

        let status_code = page
            .status_code
            .map(|c| c.to_string())
            .unwrap_or_else(|| "-".to_string());

        let title = page
            .title
            .as_ref()
            .map(|t| truncate_text(t, 30))
            .unwrap_or_else(|| "-".to_string());

        let links = page
            .links_found
            .map(|l| l.to_string())
            .unwrap_or_else(|| "-".to_string());

        let size = page
            .content_size
            .map(format_size)
            .unwrap_or_else(|| "-".to_string());

        table.add_row(vec![
            Cell::new(truncate_text(&page.url, 50)),
            Cell::new(page.depth.to_string()),
            status_cell,
            Cell::new(&status_code),
            Cell::new(&title),
            Cell::new(&links),
            Cell::new(&size),
        ]);
    }

    println!("{}", table);
    print_summary(&response.summary);

    Ok(())
}

/// Print results as formatted text
fn print_text(response: &SpiderResponse) -> Result<()> {
    println!("✓ Crawled {} pages\n", response.summary.total_pages);

    for page in &response.pages {
        println!("URL: {}", page.url);
        println!("Depth: {}", page.depth);
        println!("Status: {}", page.status);

        if let Some(code) = page.status_code {
            println!("Status Code: {}", code);
        }

        if let Some(title) = &page.title {
            println!("Title: {}", title);
        }

        if let Some(links) = page.links_found {
            println!("Links Found: {}", links);
        }

        if let Some(size) = page.content_size {
            println!("Content Size: {}", format_size(size));
        }

        if let Some(time) = page.crawl_time_ms {
            println!("Crawl Time: {}ms", time);
        }

        if let Some(error) = &page.error {
            println!("Error: {}", error);
        }

        println!();
    }

    print_summary(&response.summary);

    Ok(())
}

/// Print summary statistics
fn print_summary(summary: &CrawlSummary) {
    println!("Summary:");
    println!("  Total Pages: {}", summary.total_pages);
    println!("  Successful: {}", summary.successful);
    println!("  Failed: {}", summary.failed);
    println!("  Max Depth Reached: {}", summary.max_depth_reached);
    println!("  Total Links Found: {}", summary.total_links_found);
    println!("  Total Size: {}", format_size(summary.total_size));
    println!("  Total Time: {}ms", summary.total_time_ms);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_args_valid() {
        let args = SpiderArgs {
            seed: "https://example.com".to_string(),
            depth: 3,
            pages: 100,
            strategy: "breadth_first".to_string(),
            concurrency: 5,
            timeout: 30,
            cache: "auto".to_string(),
            output_file: None,
            robots: "respect".to_string(),
        };
        assert!(validate_args(&args).is_ok());
    }

    #[test]
    fn test_validate_args_invalid_url() {
        let args = SpiderArgs {
            seed: "not-a-url".to_string(),
            depth: 3,
            pages: 100,
            strategy: "breadth_first".to_string(),
            concurrency: 5,
            timeout: 30,
            cache: "auto".to_string(),
            output_file: None,
            robots: "respect".to_string(),
        };
        assert!(validate_args(&args).is_err());
    }

    #[test]
    fn test_validate_args_depth_zero() {
        let args = SpiderArgs {
            seed: "https://example.com".to_string(),
            depth: 0,
            pages: 100,
            strategy: "breadth_first".to_string(),
            concurrency: 5,
            timeout: 30,
            cache: "auto".to_string(),
            output_file: None,
            robots: "respect".to_string(),
        };
        assert!(validate_args(&args).is_err());
    }

    #[test]
    fn test_validate_args_pages_zero() {
        let args = SpiderArgs {
            seed: "https://example.com".to_string(),
            depth: 3,
            pages: 0,
            strategy: "breadth_first".to_string(),
            concurrency: 5,
            timeout: 30,
            cache: "auto".to_string(),
            output_file: None,
            robots: "respect".to_string(),
        };
        assert!(validate_args(&args).is_err());
    }

    #[test]
    fn test_validate_args_concurrency_zero() {
        let args = SpiderArgs {
            seed: "https://example.com".to_string(),
            depth: 3,
            pages: 100,
            strategy: "breadth_first".to_string(),
            concurrency: 0,
            timeout: 30,
            cache: "auto".to_string(),
            output_file: None,
            robots: "respect".to_string(),
        };
        assert!(validate_args(&args).is_err());
    }

    #[test]
    fn test_validate_args_timeout_zero() {
        let args = SpiderArgs {
            seed: "https://example.com".to_string(),
            depth: 3,
            pages: 100,
            strategy: "breadth_first".to_string(),
            concurrency: 5,
            timeout: 0,
            cache: "auto".to_string(),
            output_file: None,
            robots: "respect".to_string(),
        };
        assert!(validate_args(&args).is_err());
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(100), "100 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_truncate_text() {
        assert_eq!(truncate_text("short", 10), "short");
        assert_eq!(truncate_text("this is a long text", 10), "this is...");
        assert_eq!(truncate_text("exact", 5), "exact");
    }
}
