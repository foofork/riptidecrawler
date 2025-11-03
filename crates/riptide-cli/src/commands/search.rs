/// Search command - Deep search with streaming support
///
/// This command performs deep search queries using the /deepsearch endpoint.
/// It supports both batch and streaming modes with real-time result output.
use crate::client::ApiClient;
use crate::output::{self, OutputFormat};
use anyhow::{Context, Result};
use clap::Args;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Args, Clone, Debug)]
pub struct SearchArgs {
    /// Search query
    #[arg(required = true)]
    pub query: String,

    /// Maximum results to return
    #[arg(long, short = 'l', default_value = "10")]
    pub limit: u32,

    /// Stream results as NDJSON
    #[arg(long)]
    pub stream: bool,

    /// Extract full content from results
    #[arg(long)]
    pub include_content: bool,

    /// Search timeout in seconds
    #[arg(long, short = 't', default_value = "30")]
    pub timeout: u64,

    /// Save results to file
    #[arg(long, short = 'f')]
    pub output_file: Option<String>,
}

/// Request payload sent to the API
#[derive(Serialize, Debug)]
struct SearchRequest {
    query: String,
    limit: u32,
    include_content: bool,
    timeout_secs: u64,
}

/// Response from the batch API
#[derive(Deserialize, Serialize, Debug)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub summary: SearchSummary,
}

/// Single search result
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SearchResult {
    pub url: String,
    pub title: Option<String>,
    pub snippet: Option<String>,
    pub content: Option<String>,
    pub relevance_score: f64,
    pub timestamp: Option<String>,
    pub error: Option<String>,
}

/// Summary of search operation
#[derive(Deserialize, Serialize, Debug)]
pub struct SearchSummary {
    pub total: usize,
    pub successful: usize,
    pub failed: usize,
    pub avg_relevance: f64,
    pub search_time_ms: u64,
}

/// Streaming search result (single item)
#[derive(Deserialize, Serialize, Debug)]
struct StreamResult {
    #[serde(flatten)]
    result: SearchResult,
}

/// Execute the search command
pub async fn execute(
    client: ApiClient,
    args: SearchArgs,
    output_format: String,
    quiet: bool,
) -> Result<()> {
    // Validate arguments
    validate_args(&args)?;

    // Choose execution path based on stream flag
    if args.stream {
        execute_streaming(client, args, output_format, quiet).await
    } else {
        execute_batch(client, args, output_format, quiet).await
    }
}

/// Execute search in batch mode
async fn execute_batch(
    client: ApiClient,
    args: SearchArgs,
    output_format: String,
    quiet: bool,
) -> Result<()> {
    // Build request payload
    let request = SearchRequest {
        query: args.query.clone(),
        limit: args.limit,
        include_content: args.include_content,
        timeout_secs: args.timeout,
    };

    // Print progress info
    if !quiet {
        output::print_info(&format!(
            "Searching for '{}' (limit: {})...",
            args.query, args.limit
        ));
    }

    // Send request to API
    let response = client
        .post::<SearchRequest, SearchResponse>("/deepsearch", &request)
        .await
        .context("Failed to search via API")?;

    // Save to file if specified
    if let Some(output_file) = &args.output_file {
        save_to_file(output_file, &response)?;
        if !quiet {
            output::print_success(&format!("Results saved to {}", output_file));
        }
    }

    // Format and print output
    let format = OutputFormat::parse(&output_format)?;
    print_batch_results(&response, format)?;

    // Exit with error if any searches failed
    if response.summary.failed > 0 {
        anyhow::bail!(
            "Search completed with {} failed result(s)",
            response.summary.failed
        );
    }

    Ok(())
}

/// Execute search in streaming mode
async fn execute_streaming(
    client: ApiClient,
    args: SearchArgs,
    output_format: String,
    quiet: bool,
) -> Result<()> {
    // Build request payload
    let request = serde_json::json!({
        "query": args.query,
        "limit": args.limit,
        "include_content": args.include_content,
        "timeout_secs": args.timeout,
    });

    // Print progress info
    if !quiet {
        output::print_info(&format!(
            "Streaming search for '{}' (limit: {})...",
            args.query, args.limit
        ));
    }

    // Send streaming request to API
    let response = client
        .post_stream("/deepsearch/stream", request)
        .await
        .context("Failed to initiate streaming search")?;

    // Check response status
    let status = response.status();
    if !status.is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| String::from("Unknown error"));
        anyhow::bail!("API returned error {}: {}", status, error_text);
    }

    // Process streaming response
    let format = OutputFormat::parse(&output_format)?;
    let results = process_stream(response, format, args.output_file.as_deref(), quiet).await?;

    // Print summary in non-stream format
    if !quiet && format != OutputFormat::Stream {
        print_stream_summary(&results);
    }

    Ok(())
}

/// Process NDJSON stream and output results in real-time
async fn process_stream(
    response: reqwest::Response,
    format: OutputFormat,
    output_file: Option<&str>,
    quiet: bool,
) -> Result<Vec<SearchResult>> {
    use tokio::io::{AsyncBufReadExt, BufReader};

    let mut results = Vec::new();
    let mut file_writer: Option<std::fs::File> = None;

    // Open output file if specified
    if let Some(path) = output_file {
        file_writer = Some(
            std::fs::File::create(path)
                .context(format!("Failed to create output file {}", path))?,
        );
    }

    // Convert response to async stream of bytes
    let stream = response.bytes_stream();
    let stream =
        stream.map(|result| result.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)));

    // Convert to AsyncRead
    let async_read = tokio_util::io::StreamReader::new(stream);
    let mut lines = BufReader::new(async_read).lines();

    // Process each line of NDJSON
    while let Some(line) = lines
        .next_line()
        .await
        .context("Failed to read stream line")?
    {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Parse NDJSON line
        let stream_result: StreamResult = serde_json::from_str(trimmed)
            .context(format!("Failed to parse NDJSON line: {}", trimmed))?;

        let result = stream_result.result;

        // Save to file if specified
        if let Some(ref mut writer) = file_writer {
            use std::io::Write;
            writeln!(writer, "{}", trimmed).context("Failed to write to output file")?;
        }

        // Print result in real-time
        if !quiet {
            print_stream_result(&result, format)?;
        }

        results.push(result);
    }

    // Close file writer
    if let Some(_writer) = file_writer {
        if !quiet {
            output::print_success(&format!(
                "Results saved to {}",
                output_file.unwrap_or("file")
            ));
        }
    }

    Ok(results)
}

/// Validate command arguments
fn validate_args(args: &SearchArgs) -> Result<()> {
    // Validate limit
    if args.limit < 1 || args.limit > 1000 {
        anyhow::bail!("Limit must be between 1 and 1000");
    }

    // Validate timeout
    if args.timeout < 1 || args.timeout > 300 {
        anyhow::bail!("Timeout must be between 1 and 300 seconds");
    }

    Ok(())
}

/// Save batch results to file in JSON format
fn save_to_file(path: &str, response: &SearchResponse) -> Result<()> {
    let json =
        serde_json::to_string_pretty(response).context("Failed to serialize results to JSON")?;

    fs::write(path, json).context(format!("Failed to write results to {}", path))?;

    Ok(())
}

/// Print batch search results in the specified format
fn print_batch_results(response: &SearchResponse, format: OutputFormat) -> Result<()> {
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
            // For batch mode, stream format is same as JSON
            let json = serde_json::to_string_pretty(response)?;
            println!("{}", json);
        }
    }
    Ok(())
}

/// Print streaming result in real-time
fn print_stream_result(result: &SearchResult, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Stream | OutputFormat::Json => {
            // NDJSON format - one JSON object per line
            let json = serde_json::to_string(result)?;
            println!("{}", json);
        }
        OutputFormat::Text => {
            // Human-readable format
            println!("\n{}", "─".repeat(80));
            println!("URL: {}", result.url);
            if let Some(title) = &result.title {
                println!("Title: {}", title);
            }
            println!("Relevance: {:.2}", result.relevance_score);
            if let Some(snippet) = &result.snippet {
                println!("Snippet: {}", snippet);
            }
            if let Some(content) = &result.content {
                let preview = if content.len() > 200 {
                    format!("{}...", &content[..200])
                } else {
                    content.clone()
                };
                println!("Content: {}", preview);
            }
            if let Some(error) = &result.error {
                println!("Error: {}", error);
            }
        }
        OutputFormat::Table => {
            // For streaming, use text format instead of building a table
            print_stream_result(result, OutputFormat::Text)?;
        }
    }
    Ok(())
}

/// Print stream summary
fn print_stream_summary(results: &[SearchResult]) {
    let total = results.len();
    let successful = results.iter().filter(|r| r.error.is_none()).count();
    let failed = total - successful;
    let avg_relevance = if !results.is_empty() {
        results.iter().map(|r| r.relevance_score).sum::<f64>() / total as f64
    } else {
        0.0
    };

    println!("\n{}", "═".repeat(80));
    println!("Summary:");
    println!("  Total: {}", total);
    println!("  Successful: {}", successful);
    println!("  Failed: {}", failed);
    println!("  Avg Relevance: {:.2}", avg_relevance);
}

/// Print results as a formatted table
fn print_table(response: &SearchResponse) -> Result<()> {
    use comfy_table::modifiers::UTF8_ROUND_CORNERS;
    use comfy_table::presets::UTF8_FULL;
    use comfy_table::{Cell, Color, Table};

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec!["URL", "Title", "Relevance", "Snippet"]);

    for result in &response.results {
        let relevance_cell = {
            let score = result.relevance_score;
            let color = if score >= 0.8 {
                Color::Green
            } else if score >= 0.5 {
                Color::Yellow
            } else {
                Color::Red
            };
            Cell::new(format!("{:.2}", score)).fg(color)
        };

        let title = result
            .title
            .as_deref()
            .unwrap_or("-")
            .chars()
            .take(30)
            .collect::<String>();

        let snippet = result
            .snippet
            .as_deref()
            .unwrap_or("-")
            .chars()
            .take(50)
            .collect::<String>();

        let url = result.url.chars().take(40).collect::<String>();

        table.add_row(vec![
            Cell::new(&url),
            Cell::new(&title),
            relevance_cell,
            Cell::new(&snippet),
        ]);
    }

    println!("{}", table);
    print_summary(&response.summary);

    Ok(())
}

/// Print results as formatted text
fn print_text(response: &SearchResponse) -> Result<()> {
    println!("✓ Found {} results\n", response.summary.total);

    for (i, result) in response.results.iter().enumerate() {
        println!("{}. {}", i + 1, result.url);

        if let Some(title) = &result.title {
            println!("   Title: {}", title);
        }

        println!("   Relevance: {:.2}", result.relevance_score);

        if let Some(snippet) = &result.snippet {
            println!("   Snippet: {}", snippet);
        }

        if let Some(content) = &result.content {
            let preview = if content.len() > 200 {
                format!("{}...", &content[..200])
            } else {
                content.clone()
            };
            println!("   Content: {}", preview);
        }

        if let Some(error) = &result.error {
            println!("   Error: {}", error);
        }

        println!();
    }

    print_summary(&response.summary);

    Ok(())
}

/// Print summary statistics
fn print_summary(summary: &SearchSummary) {
    println!("Summary:");
    println!("  Successful: {}", summary.successful);
    println!("  Failed: {}", summary.failed);
    println!("  Avg Relevance: {:.2}", summary.avg_relevance);
    println!("  Search Time: {}ms", summary.search_time_ms);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_args_valid() {
        let args = SearchArgs {
            query: "test query".to_string(),
            limit: 10,
            stream: false,
            include_content: false,
            timeout: 30,
            output_file: None,
        };
        assert!(validate_args(&args).is_ok());
    }

    #[test]
    fn test_validate_args_invalid_limit() {
        let mut args = SearchArgs {
            query: "test".to_string(),
            limit: 0,
            stream: false,
            include_content: false,
            timeout: 30,
            output_file: None,
        };
        assert!(validate_args(&args).is_err());

        args.limit = 1001;
        assert!(validate_args(&args).is_err());

        args.limit = 100;
        assert!(validate_args(&args).is_ok());
    }

    #[test]
    fn test_validate_args_invalid_timeout() {
        let mut args = SearchArgs {
            query: "test".to_string(),
            limit: 10,
            stream: false,
            include_content: false,
            timeout: 0,
            output_file: None,
        };
        assert!(validate_args(&args).is_err());

        args.timeout = 301;
        assert!(validate_args(&args).is_err());

        args.timeout = 60;
        assert!(validate_args(&args).is_ok());
    }

    #[test]
    fn test_search_request_serialization() {
        let request = SearchRequest {
            query: "rust async".to_string(),
            limit: 20,
            include_content: true,
            timeout_secs: 60,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("rust async"));
        assert!(json.contains("\"limit\":20"));
        assert!(json.contains("\"include_content\":true"));
        assert!(json.contains("\"timeout_secs\":60"));
    }

    #[test]
    fn test_search_result_deserialization() {
        let json = r#"{
            "url": "https://example.com",
            "title": "Test Page",
            "snippet": "A test snippet",
            "content": null,
            "relevance_score": 0.95,
            "timestamp": "2025-01-01T12:00:00Z",
            "error": null
        }"#;

        let result: SearchResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.url, "https://example.com");
        assert_eq!(result.title, Some("Test Page".to_string()));
        assert_eq!(result.relevance_score, 0.95);
        assert!(result.content.is_none());
        assert!(result.error.is_none());
    }

    #[test]
    fn test_stream_result_deserialization() {
        let json = r#"{
            "url": "https://example.com",
            "title": "Test",
            "snippet": "snippet",
            "content": null,
            "relevance_score": 0.8,
            "timestamp": null,
            "error": null
        }"#;

        let stream_result: StreamResult = serde_json::from_str(json).unwrap();
        assert_eq!(stream_result.result.url, "https://example.com");
        assert_eq!(stream_result.result.relevance_score, 0.8);
    }
}
