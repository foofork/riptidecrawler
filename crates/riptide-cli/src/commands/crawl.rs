// TODO: Remove this allow once wired into main CLI - these functions will be called from main.rs
#![allow(dead_code)]

/// Crawl command - Basic web crawling with configurable depth and options
///
/// This command crawls multiple URLs with support for depth control, external links,
/// and both streaming and batch modes.
use crate::client::ApiClient;
use crate::output::{self, format_size, truncate_text, OutputFormat};
use anyhow::{Context, Result};
use clap::Args;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::fs;

/// Arguments for the crawl command
#[derive(Args, Clone, Debug)]
pub struct CrawlArgs {
    /// URLs to crawl
    #[arg(required = true)]
    pub urls: Vec<String>,

    /// Maximum depth to crawl
    #[arg(short, long, default_value = "1")]
    pub depth: u32,

    /// Enable streaming output
    #[arg(short, long)]
    pub stream: bool,

    /// Maximum pages to crawl
    #[arg(short = 'p', long)]
    pub max_pages: Option<u32>,

    /// Follow external links
    #[arg(long)]
    pub external: bool,

    /// Save results to file
    #[arg(short = 'f', long)]
    pub output_file: Option<String>,
}

/// Request payload sent to the API
#[derive(Serialize, Debug)]
struct CrawlRequest {
    urls: Vec<String>,
    options: CrawlOptions,
}

/// Crawl configuration options
#[derive(Serialize, Debug)]
struct CrawlOptions {
    max_depth: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_pages: Option<u32>,
    follow_external: bool,
}

/// Response from the API
#[derive(Deserialize, Serialize, Debug)]
pub struct CrawlResponse {
    pub total_urls: usize,
    pub successful: usize,
    pub failed: usize,
    pub from_cache: usize,
    pub results: Vec<CrawlResult>,
    pub statistics: CrawlStatistics,
}

/// Individual crawl result for a single URL
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CrawlResult {
    pub url: String,
    pub status: u16,
    pub from_cache: bool,
    pub gate_decision: String,
    pub quality_score: f32,
    pub processing_time_ms: u64,
    pub document: Option<ExtractedDoc>,
    pub error: Option<ErrorInfo>,
    pub cache_key: String,
}

/// Extracted document structure
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ExtractedDoc {
    pub title: Option<String>,
    pub content: String,
    pub links: Vec<String>,
    pub metadata: serde_json::Value,
}

/// Error information for failed operations
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ErrorInfo {
    pub error_type: String,
    pub message: String,
    pub retryable: bool,
}

/// Statistics for crawl operations
#[derive(Deserialize, Serialize, Debug)]
pub struct CrawlStatistics {
    pub total_processing_time_ms: u64,
    pub avg_processing_time_ms: f64,
    pub gate_decisions: GateDecisionBreakdown,
    pub cache_hit_rate: f64,
}

/// Breakdown of gate decisions
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct GateDecisionBreakdown {
    pub raw: usize,
    pub probes_first: usize,
    pub headless: usize,
    pub cached: usize,
}

/// Streaming update from the API
#[derive(Deserialize, Serialize, Debug)]
struct StreamingUpdate {
    #[serde(rename = "type")]
    update_type: String,
    url: Option<String>,
    status: Option<String>,
    result: Option<CrawlResult>,
    error: Option<String>,
}

/// Execute the crawl command
pub async fn execute(client: ApiClient, args: CrawlArgs, output_format: String) -> Result<()> {
    // Validate arguments
    validate_args(&args)?;

    // Build request payload
    let request = CrawlRequest {
        urls: args.urls.clone(),
        options: CrawlOptions {
            max_depth: args.depth,
            max_pages: args.max_pages,
            follow_external: args.external,
        },
    };

    // Print progress info
    output::print_info(&format!(
        "Crawling {} URL(s) with max depth {} {}...",
        request.urls.len(),
        request.options.max_depth,
        if args.stream { "(streaming)" } else { "" }
    ));

    if args.stream {
        // Stream mode: handle Server-Sent Events or NDJSON stream
        execute_streaming(&client, request, &args, output_format).await
    } else {
        // Batch mode: POST to /crawl and await response
        execute_batch(&client, request, &args, output_format).await
    }
}

/// Execute crawl in batch mode
async fn execute_batch(
    client: &ApiClient,
    request: CrawlRequest,
    args: &CrawlArgs,
    output_format: String,
) -> Result<()> {
    // Send request to API
    let response = client
        .post::<CrawlRequest, CrawlResponse>("/crawl", &request)
        .await
        .context("Failed to execute crawl via API")?;

    // Save to file if specified
    if let Some(output_file) = &args.output_file {
        save_to_file(output_file, &response)?;
        output::print_success(&format!("Results saved to {}", output_file));
    }

    // Format and print output
    let format = OutputFormat::parse(&output_format)?;
    print_results(&response, format)?;

    // Exit with error if any URLs failed
    if response.failed > 0 {
        anyhow::bail!("Crawl completed with {} failed URL(s)", response.failed);
    }

    Ok(())
}

/// Execute crawl in streaming mode
async fn execute_streaming(
    client: &ApiClient,
    request: CrawlRequest,
    args: &CrawlArgs,
    output_format: String,
) -> Result<()> {
    // Send streaming request
    let response = client
        .post_stream("/crawl/stream", serde_json::to_value(&request)?)
        .await
        .context("Failed to start streaming crawl")?;

    // Check for HTTP errors
    let status = response.status();
    if !status.is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| String::from("Unknown error"));
        anyhow::bail!("API returned error {}: {}", status, error_text);
    }

    let format = OutputFormat::parse(&output_format)?;
    let mut results = Vec::new();
    let mut stream = response.bytes_stream();

    // Buffer for incomplete lines
    let mut buffer = String::new();

    // Process streaming response
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.context("Failed to read stream chunk")?;
        let text = String::from_utf8_lossy(&chunk);

        // Add to buffer and process complete lines
        buffer.push_str(&text);

        // Split on newlines and process complete lines
        let mut parts: Vec<String> = buffer.split('\n').map(|s| s.to_string()).collect();

        // Keep the last incomplete line in the buffer
        buffer = parts.pop().unwrap_or_default();

        // Process complete lines
        for line in parts {
            if line.trim().is_empty() {
                continue;
            }

            // Try to parse as NDJSON streaming update
            match serde_json::from_str::<StreamingUpdate>(&line) {
                Ok(update) => {
                    process_streaming_update(&update, &mut results, format)?;
                }
                Err(e) => {
                    eprintln!("Warning: Failed to parse streaming update: {}", e);
                    eprintln!("Line: {}", line);
                }
            }
        }
    }

    // Process any remaining data in the buffer
    if !buffer.trim().is_empty() {
        if let Ok(update) = serde_json::from_str::<StreamingUpdate>(&buffer) {
            process_streaming_update(&update, &mut results, format)?;
        }
    }

    // Save to file if specified
    if let Some(output_file) = &args.output_file {
        let summary_response = build_summary_response(results);
        save_to_file(output_file, &summary_response)?;
        output::print_success(&format!("Results saved to {}", output_file));
    }

    Ok(())
}

/// Process a single streaming update
fn process_streaming_update(
    update: &StreamingUpdate,
    results: &mut Vec<CrawlResult>,
    format: OutputFormat,
) -> Result<()> {
    match update.update_type.as_str() {
        "progress" => {
            // Show progress updates
            if let (Some(url), Some(status)) = (&update.url, &update.status) {
                match format {
                    OutputFormat::Json | OutputFormat::Stream => {
                        let json = serde_json::to_string(update)?;
                        println!("{}", json);
                    }
                    _ => {
                        output::print_info(&format!("{}: {}", truncate_text(url, 60), status));
                    }
                }
            }
        }
        "result" => {
            // Store completed results
            if let Some(result) = &update.result {
                results.push(result.clone());

                match format {
                    OutputFormat::Json | OutputFormat::Stream => {
                        let json = serde_json::to_string(result)?;
                        println!("{}", json);
                    }
                    _ => {
                        let status_icon = if result.error.is_none() { "✓" } else { "✗" };
                        output::print_info(&format!(
                            "{} {} ({}ms, quality: {:.2})",
                            status_icon,
                            truncate_text(&result.url, 60),
                            result.processing_time_ms,
                            result.quality_score
                        ));
                    }
                }
            }
        }
        "error" => {
            if let Some(error) = &update.error {
                eprintln!("Error: {}", error);
            }
        }
        _ => {
            // Unknown update type - log and continue
            eprintln!("Warning: Unknown update type: {}", update.update_type);
        }
    }

    Ok(())
}

/// Build summary response from streaming results
fn build_summary_response(results: Vec<CrawlResult>) -> CrawlResponse {
    let total = results.len();
    let successful = results.iter().filter(|r| r.error.is_none()).count();
    let failed = total - successful;
    let from_cache = results.iter().filter(|r| r.from_cache).count();

    let mut gate_decisions = GateDecisionBreakdown::default();
    let mut total_time = 0u64;

    for result in &results {
        total_time += result.processing_time_ms;
        match result.gate_decision.as_str() {
            "raw" => gate_decisions.raw += 1,
            "probes_first" => gate_decisions.probes_first += 1,
            "headless" => gate_decisions.headless += 1,
            "cached" => gate_decisions.cached += 1,
            _ => {}
        }
    }

    let avg_time = if total > 0 {
        total_time as f64 / total as f64
    } else {
        0.0
    };

    let cache_hit_rate = if total > 0 {
        from_cache as f64 / total as f64
    } else {
        0.0
    };

    CrawlResponse {
        total_urls: total,
        successful,
        failed,
        from_cache,
        results,
        statistics: CrawlStatistics {
            total_processing_time_ms: total_time,
            avg_processing_time_ms: avg_time,
            gate_decisions,
            cache_hit_rate,
        },
    }
}

/// Validate command arguments
fn validate_args(args: &CrawlArgs) -> Result<()> {
    // Prevent empty URL list
    if args.urls.is_empty() {
        anyhow::bail!("At least one URL is required");
    }

    // Basic URL format check
    for url in &args.urls {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            anyhow::bail!("URL must start with http:// or https://: {}", url);
        }
    }

    // Prevent nonsensical depth
    if args.depth == 0 {
        anyhow::bail!("Depth must be at least 1");
    }

    // Validate max_pages if provided
    if let Some(max_pages) = args.max_pages {
        if max_pages == 0 {
            anyhow::bail!("Max pages must be at least 1");
        }
    }

    Ok(())
}

/// Save results to file in JSON format
fn save_to_file(path: &str, response: &CrawlResponse) -> Result<()> {
    let json =
        serde_json::to_string_pretty(response).context("Failed to serialize results to JSON")?;

    fs::write(path, json).context(format!("Failed to write results to {}", path))?;

    Ok(())
}

/// Print crawl results in the specified format
fn print_results(response: &CrawlResponse, format: OutputFormat) -> Result<()> {
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
            // For batch crawl, stream format is same as JSON
            let json = serde_json::to_string_pretty(response)?;
            println!("{}", json);
        }
    }
    Ok(())
}

/// Print results as a formatted table
fn print_table(response: &CrawlResponse) -> Result<()> {
    use comfy_table::modifiers::UTF8_ROUND_CORNERS;
    use comfy_table::presets::UTF8_FULL;
    use comfy_table::{Cell, Color, Table};

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            "URL",
            "Status",
            "Cache",
            "Gate",
            "Quality",
            "Time (ms)",
        ]);

    for result in &response.results {
        let status_cell = if result.error.is_none() {
            Cell::new(result.status.to_string()).fg(Color::Green)
        } else {
            Cell::new("Error").fg(Color::Red)
        };

        let cache = if result.from_cache { "Yes" } else { "No" };

        table.add_row(vec![
            Cell::new(truncate_text(&result.url, 50)),
            status_cell,
            Cell::new(cache),
            Cell::new(&result.gate_decision),
            Cell::new(format!("{:.2}", result.quality_score)),
            Cell::new(result.processing_time_ms.to_string()),
        ]);
    }

    println!("{}", table);
    print_summary(response);

    Ok(())
}

/// Print results as formatted text
fn print_text(response: &CrawlResponse) -> Result<()> {
    println!("✓ Crawled {} URLs\n", response.total_urls);

    for result in &response.results {
        println!("URL: {}", result.url);
        println!("Status: {}", result.status);
        println!("From Cache: {}", result.from_cache);
        println!("Gate Decision: {}", result.gate_decision);
        println!("Quality Score: {:.2}", result.quality_score);
        println!("Processing Time: {}ms", result.processing_time_ms);

        if let Some(doc) = &result.document {
            if let Some(title) = &doc.title {
                println!("Title: {}", title);
            }
            println!("Content Size: {}", format_size(doc.content.len()));
            println!("Links Found: {}", doc.links.len());
        }

        if let Some(error) = &result.error {
            println!("Error Type: {}", error.error_type);
            println!("Error Message: {}", error.message);
            println!("Retryable: {}", error.retryable);
        }

        println!();
    }

    print_summary(response);

    Ok(())
}

/// Print summary statistics
fn print_summary(response: &CrawlResponse) {
    println!("Summary:");
    println!("  Total URLs: {}", response.total_urls);
    println!("  Successful: {}", response.successful);
    println!("  Failed: {}", response.failed);
    println!("  From Cache: {}", response.from_cache);
    println!(
        "  Cache Hit Rate: {:.2}%",
        response.statistics.cache_hit_rate * 100.0
    );
    println!(
        "  Avg Processing Time: {:.2}ms",
        response.statistics.avg_processing_time_ms
    );
    println!(
        "  Total Processing Time: {}ms",
        response.statistics.total_processing_time_ms
    );

    println!("\nGate Decisions:");
    println!("  Raw: {}", response.statistics.gate_decisions.raw);
    println!(
        "  Probes First: {}",
        response.statistics.gate_decisions.probes_first
    );
    println!(
        "  Headless: {}",
        response.statistics.gate_decisions.headless
    );
    println!("  Cached: {}", response.statistics.gate_decisions.cached);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_args_valid() {
        let args = CrawlArgs {
            urls: vec!["https://example.com".to_string()],
            depth: 2,
            stream: false,
            max_pages: Some(10),
            external: false,
            output_file: None,
        };
        assert!(validate_args(&args).is_ok());
    }

    #[test]
    fn test_validate_args_empty_urls() {
        let args = CrawlArgs {
            urls: vec![],
            depth: 2,
            stream: false,
            max_pages: None,
            external: false,
            output_file: None,
        };
        assert!(validate_args(&args).is_err());
    }

    #[test]
    fn test_validate_args_invalid_url() {
        let args = CrawlArgs {
            urls: vec!["not-a-url".to_string()],
            depth: 2,
            stream: false,
            max_pages: None,
            external: false,
            output_file: None,
        };
        assert!(validate_args(&args).is_err());
    }

    #[test]
    fn test_validate_args_depth_zero() {
        let args = CrawlArgs {
            urls: vec!["https://example.com".to_string()],
            depth: 0,
            stream: false,
            max_pages: None,
            external: false,
            output_file: None,
        };
        assert!(validate_args(&args).is_err());
    }

    #[test]
    fn test_validate_args_max_pages_zero() {
        let args = CrawlArgs {
            urls: vec!["https://example.com".to_string()],
            depth: 2,
            stream: false,
            max_pages: Some(0),
            external: false,
            output_file: None,
        };
        assert!(validate_args(&args).is_err());
    }

    #[test]
    fn test_build_summary_response() {
        let results = vec![
            CrawlResult {
                url: "https://example.com".to_string(),
                status: 200,
                from_cache: false,
                gate_decision: "raw".to_string(),
                quality_score: 0.9,
                processing_time_ms: 100,
                document: None,
                error: None,
                cache_key: "key1".to_string(),
            },
            CrawlResult {
                url: "https://example.org".to_string(),
                status: 200,
                from_cache: true,
                gate_decision: "cached".to_string(),
                quality_score: 0.95,
                processing_time_ms: 50,
                document: None,
                error: None,
                cache_key: "key2".to_string(),
            },
        ];

        let response = build_summary_response(results);

        assert_eq!(response.total_urls, 2);
        assert_eq!(response.successful, 2);
        assert_eq!(response.failed, 0);
        assert_eq!(response.from_cache, 1);
        assert_eq!(response.statistics.cache_hit_rate, 0.5);
        assert_eq!(response.statistics.avg_processing_time_ms, 75.0);
        assert_eq!(response.statistics.gate_decisions.raw, 1);
        assert_eq!(response.statistics.gate_decisions.cached, 1);
    }
}
