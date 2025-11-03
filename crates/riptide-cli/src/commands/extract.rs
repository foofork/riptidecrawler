/// Extract command - the PRIMARY use case for RipTide CLI
///
/// This command extracts content from URLs using various strategies.
/// It's the ONLY command that provides full strategy control (auto/css/wasm/llm/multi).
use crate::client::ApiClient;
use crate::output::{self, OutputFormat};
use anyhow::{Context, Result};
use clap::Args;
use serde::{Deserialize, Serialize};
use std::fs;

/// Arguments for the extract command
#[derive(Args, Clone, Debug)]
pub struct ExtractArgs {
    /// URLs to extract content from (supports multiple URLs)
    #[arg(required = true)]
    pub urls: Vec<String>,

    /// Extraction strategy (auto/css/wasm/llm/multi)
    /// - auto: Automatically select best strategy
    /// - css: CSS selector-based extraction
    /// - wasm: WASM-powered extraction engine
    /// - llm: LLM-powered intelligent extraction
    /// - multi: Try multiple strategies and merge results
    #[arg(long, short = 's', default_value = "multi")]
    pub strategy: String,

    /// CSS selector for content extraction (required for css strategy)
    #[arg(long)]
    pub selector: Option<String>,

    /// Regex pattern for content extraction
    #[arg(long)]
    pub pattern: Option<String>,

    /// Minimum quality threshold (0.0-1.0)
    #[arg(long, default_value = "0.7")]
    pub quality_threshold: f64,

    /// Extraction timeout in milliseconds
    #[arg(long, short = 't', default_value = "30000")]
    pub timeout: u64,

    /// Number of concurrent extraction requests
    #[arg(long, short = 'c', default_value = "5")]
    pub concurrency: u32,

    /// Cache mode (auto/read_write/read_only/write_only/disabled)
    #[arg(long, default_value = "auto")]
    pub cache: String,

    /// Save results to file
    #[arg(long, short = 'f')]
    pub output_file: Option<String>,
}

/// Request payload sent to the API
#[derive(Serialize, Debug)]
struct ExtractRequest {
    urls: Vec<String>,
    strategy: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pattern: Option<String>,
    quality_threshold: f64,
    timeout_ms: u64,
    concurrency: u32,
    cache_mode: String,
}

/// Response from the API
#[derive(Deserialize, Serialize, Debug)]
pub struct ExtractResponse {
    pub results: Vec<ExtractResult>,
    pub summary: ExtractSummary,
}

/// Single extraction result
#[derive(Deserialize, Serialize, Debug)]
pub struct ExtractResult {
    pub url: String,
    pub status: String,
    pub content: Option<String>,
    pub strategy_used: Option<String>,
    pub quality_score: Option<f64>,
    pub content_size: Option<usize>,
    pub error: Option<String>,
}

/// Summary of extraction job
#[derive(Deserialize, Serialize, Debug)]
pub struct ExtractSummary {
    pub total: usize,
    pub successful: usize,
    pub failed: usize,
    pub avg_quality: Option<f64>,
    pub total_size: usize,
}

/// Execute the extract command
pub async fn execute(client: ApiClient, args: ExtractArgs, output_format: String) -> Result<()> {
    // Validate arguments
    validate_args(&args)?;

    // Build request payload
    let request = ExtractRequest {
        urls: args.urls.clone(),
        strategy: args.strategy.clone(),
        selector: args.selector.clone(),
        pattern: args.pattern.clone(),
        quality_threshold: args.quality_threshold,
        timeout_ms: args.timeout,
        concurrency: args.concurrency,
        cache_mode: args.cache.clone(),
    };

    // Print progress info
    output::print_info(&format!(
        "Extracting {} URL(s) with {} strategy...",
        request.urls.len(),
        request.strategy
    ));

    // Send request to API
    let response = client
        .post::<ExtractRequest, ExtractResponse>("/extract", &request)
        .await
        .context("Failed to extract content from API")?;

    // Save to file if specified
    if let Some(output_file) = &args.output_file {
        save_to_file(output_file, &response)?;
        output::print_success(&format!("Results saved to {}", output_file));
    }

    // Format and print output
    let format = OutputFormat::parse(&output_format)?;
    print_results(&response, format)?;

    // Exit with error if any extractions failed
    if response.summary.failed > 0 {
        anyhow::bail!(
            "Extraction completed with {} failed URL(s)",
            response.summary.failed
        );
    }

    Ok(())
}

/// Validate command arguments
fn validate_args(args: &ExtractArgs) -> Result<()> {
    // Validate strategy
    let valid_strategies = ["auto", "css", "wasm", "llm", "multi"];
    if !valid_strategies.contains(&args.strategy.as_str()) {
        anyhow::bail!(
            "Invalid strategy '{}'. Must be one of: {}",
            args.strategy,
            valid_strategies.join(", ")
        );
    }

    // CSS strategy requires selector
    if args.strategy == "css" && args.selector.is_none() {
        anyhow::bail!("CSS strategy requires --selector argument");
    }

    // Validate quality threshold
    if args.quality_threshold < 0.0 || args.quality_threshold > 1.0 {
        anyhow::bail!("Quality threshold must be between 0.0 and 1.0");
    }

    // Validate concurrency
    if args.concurrency < 1 || args.concurrency > 100 {
        anyhow::bail!("Concurrency must be between 1 and 100");
    }

    // Validate cache mode
    let valid_cache_modes = ["auto", "read_write", "read_only", "write_only", "disabled"];
    if !valid_cache_modes.contains(&args.cache.as_str()) {
        anyhow::bail!(
            "Invalid cache mode '{}'. Must be one of: {}",
            args.cache,
            valid_cache_modes.join(", ")
        );
    }

    Ok(())
}

/// Save results to file in JSON format
fn save_to_file(path: &str, response: &ExtractResponse) -> Result<()> {
    let json =
        serde_json::to_string_pretty(response).context("Failed to serialize results to JSON")?;

    fs::write(path, json).context(format!("Failed to write results to {}", path))?;

    Ok(())
}

/// Print extraction results in the specified format
fn print_results(response: &ExtractResponse, format: OutputFormat) -> Result<()> {
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
            // For extract command, stream format is same as JSON
            let json = serde_json::to_string_pretty(response)?;
            println!("{}", json);
        }
    }
    Ok(())
}

/// Print results as a formatted table
fn print_table(response: &ExtractResponse) -> Result<()> {
    use comfy_table::modifiers::UTF8_ROUND_CORNERS;
    use comfy_table::presets::UTF8_FULL;
    use comfy_table::{Cell, Color, Table};

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec!["URL", "Status", "Strategy", "Quality", "Size"]);

    for result in &response.results {
        let status_cell = if result.status == "success" {
            Cell::new(&result.status).fg(Color::Green)
        } else {
            Cell::new(&result.status).fg(Color::Red)
        };

        let quality = result
            .quality_score
            .map(|q| format!("{:.2}", q))
            .unwrap_or_else(|| "-".to_string());

        let size = result
            .content_size
            .map(format_size)
            .unwrap_or_else(|| "-".to_string());

        let strategy = result.strategy_used.as_deref().unwrap_or("-");

        table.add_row(vec![
            Cell::new(&result.url),
            status_cell,
            Cell::new(strategy),
            Cell::new(&quality),
            Cell::new(&size),
        ]);
    }

    println!("{}", table);
    print_summary(&response.summary);

    Ok(())
}

/// Print results as formatted text
fn print_text(response: &ExtractResponse) -> Result<()> {
    println!("✓ Extracted {} URLs\n", response.summary.total);

    for result in &response.results {
        println!("URL: {}", result.url);
        println!("Status: {}", result.status);

        if let Some(strategy) = &result.strategy_used {
            println!("Strategy: {}", strategy);
        }

        if let Some(quality) = result.quality_score {
            println!("Quality: {:.2}", quality);
        }

        if let Some(size) = result.content_size {
            println!("Size: {}", format_size(size));
        }

        if let Some(error) = &result.error {
            println!("Error: {}", error);
        }

        println!();
    }

    print_summary(&response.summary);

    Ok(())
}

/// Print summary statistics
fn print_summary(summary: &ExtractSummary) {
    println!("Summary:");
    println!("  Successful: {}", summary.successful);
    println!("  Failed: {}", summary.failed);

    if let Some(avg_quality) = summary.avg_quality {
        println!("  Avg Quality: {:.2}", avg_quality);
    }

    println!("  Total Size: {}", format_size(summary.total_size));
}

/// Format byte size in human-readable form
fn format_size(size: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = KB * 1024;
    const GB: usize = MB * 1024;

    if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_args_valid() {
        let args = ExtractArgs {
            urls: vec!["https://example.com".to_string()],
            strategy: "multi".to_string(),
            selector: None,
            pattern: None,
            quality_threshold: 0.7,
            timeout: 30000,
            concurrency: 5,
            cache: "auto".to_string(),
            output_file: None,
        };
        assert!(validate_args(&args).is_ok());
    }

    #[test]
    fn test_validate_args_invalid_strategy() {
        let args = ExtractArgs {
            urls: vec!["https://example.com".to_string()],
            strategy: "invalid".to_string(),
            selector: None,
            pattern: None,
            quality_threshold: 0.7,
            timeout: 30000,
            concurrency: 5,
            cache: "auto".to_string(),
            output_file: None,
        };
        assert!(validate_args(&args).is_err());
    }

    #[test]
    fn test_validate_args_css_requires_selector() {
        let args = ExtractArgs {
            urls: vec!["https://example.com".to_string()],
            strategy: "css".to_string(),
            selector: None,
            pattern: None,
            quality_threshold: 0.7,
            timeout: 30000,
            concurrency: 5,
            cache: "auto".to_string(),
            output_file: None,
        };
        assert!(validate_args(&args).is_err());
    }

    #[test]
    fn test_validate_args_quality_threshold_bounds() {
        let mut args = ExtractArgs {
            urls: vec!["https://example.com".to_string()],
            strategy: "multi".to_string(),
            selector: None,
            pattern: None,
            quality_threshold: -0.1,
            timeout: 30000,
            concurrency: 5,
            cache: "auto".to_string(),
            output_file: None,
        };
        assert!(validate_args(&args).is_err());

        args.quality_threshold = 1.1;
        assert!(validate_args(&args).is_err());

        args.quality_threshold = 0.5;
        assert!(validate_args(&args).is_ok());
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(100), "100 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
    }
}
