/// Render command - Execute JavaScript and render dynamic web pages
///
/// This command renders URLs with JavaScript execution, waits for dynamic content,
/// and optionally captures screenshots of the rendered page.
use crate::client::ApiClient;
use crate::output::{self, format_size, sanitize_filename, OutputFormat};
use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use clap::Args;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Args, Clone, Debug)]
pub struct RenderArgs {
    /// URLs to render
    #[arg(required = true)]
    pub urls: Vec<String>,

    /// Wait time in milliseconds after page load
    #[arg(long, short = 'w', default_value = "2000")]
    pub wait: u64,

    /// Capture screenshot of rendered page
    #[arg(long)]
    pub screenshot: bool,

    /// Viewport size (WIDTHxHEIGHT)
    #[arg(long, default_value = "1920x1080")]
    pub viewport: String,

    /// Render timeout in seconds
    #[arg(long, short = 't', default_value = "30")]
    pub timeout: u64,

    /// Save results to file
    #[arg(long, short = 'f')]
    pub output_file: Option<String>,
}

/// Request payload sent to the /render API endpoint
#[derive(Serialize, Debug)]
struct RenderRequest {
    urls: Vec<String>,
    wait_time_ms: u64,
    screenshot: bool,
    viewport: ViewportSize,
    timeout_sec: u64,
}

/// Viewport size configuration
#[derive(Serialize, Debug)]
struct ViewportSize {
    width: u32,
    height: u32,
}

/// Response from the /render API endpoint
#[derive(Deserialize, Serialize, Debug)]
pub struct RenderResponse {
    pub results: Vec<RenderResult>,
    pub summary: RenderSummary,
}

/// Single render result
#[derive(Deserialize, Serialize, Debug)]
pub struct RenderResult {
    pub url: String,
    pub status: String,
    pub rendered_html: Option<String>,
    pub screenshot_path: Option<String>,
    pub screenshot_data: Option<String>, // Base64 encoded screenshot
    pub render_time_ms: Option<u64>,
    pub html_size: Option<usize>,
    pub error: Option<String>,
}

/// Summary of render job
#[derive(Deserialize, Serialize, Debug)]
pub struct RenderSummary {
    pub total: usize,
    pub successful: usize,
    pub failed: usize,
    pub avg_render_time_ms: Option<u64>,
    pub total_html_size: usize,
}

/// Execute the render command
pub async fn execute(client: ApiClient, args: RenderArgs, output_format: String) -> Result<()> {
    // Validate arguments
    validate_args(&args)?;

    // Parse viewport size
    let viewport = parse_viewport(&args.viewport)?;

    // Build request payload
    let request = RenderRequest {
        urls: args.urls.clone(),
        wait_time_ms: args.wait,
        screenshot: args.screenshot,
        viewport,
        timeout_sec: args.timeout,
    };

    // Print progress info
    output::print_info(&format!(
        "Rendering {} URL(s) with {}ms wait time{}...",
        request.urls.len(),
        request.wait_time_ms,
        if args.screenshot {
            " and screenshot capture"
        } else {
            ""
        }
    ));

    // Send request to API
    let response = client
        .post::<RenderRequest, RenderResponse>("/render", &request)
        .await
        .context("Failed to render pages from API")?;

    // Save screenshots if any were captured
    if args.screenshot {
        save_screenshots(&response)?;
    }

    // Save to file if specified
    if let Some(output_file) = &args.output_file {
        save_to_file(output_file, &response)?;
        output::print_success(&format!("Results saved to {}", output_file));
    }

    // Format and print output
    let format = OutputFormat::parse(&output_format)?;
    print_results(&response, format)?;

    // Exit with error if any renders failed
    if response.summary.failed > 0 {
        anyhow::bail!(
            "Rendering completed with {} failed URL(s)",
            response.summary.failed
        );
    }

    Ok(())
}

/// Validate command arguments
fn validate_args(args: &RenderArgs) -> Result<()> {
    // Validate viewport format - UX check to prevent typos
    if !args.viewport.contains('x') {
        anyhow::bail!(
            "Invalid viewport format '{}'. Expected format: WIDTHxHEIGHT (e.g., 1920x1080)",
            args.viewport
        );
    }

    // Prevent nonsensical timeout
    if args.timeout == 0 {
        anyhow::bail!("Timeout must be at least 1 second");
    }

    // Prevent empty URL list
    if args.urls.is_empty() {
        anyhow::bail!("At least one URL is required");
    }

    Ok(())
}

/// Parse viewport string (e.g., "1920x1080") into ViewportSize
fn parse_viewport(viewport: &str) -> Result<ViewportSize> {
    let parts: Vec<&str> = viewport.split('x').collect();
    if parts.len() != 2 {
        anyhow::bail!(
            "Invalid viewport format '{}'. Expected format: WIDTHxHEIGHT",
            viewport
        );
    }

    let width = parts[0].parse::<u32>().context("Invalid viewport width")?;
    let height = parts[1].parse::<u32>().context("Invalid viewport height")?;

    // NOTE: Keep viewport dimension checks - they provide good UX by catching typos
    // like "192x108" instead of "1920x1080" before sending to server
    if !(320..=7680).contains(&width) {
        anyhow::bail!("Viewport width must be between 320 and 7680 pixels");
    }
    if !(240..=4320).contains(&height) {
        anyhow::bail!("Viewport height must be between 240 and 4320 pixels");
    }

    Ok(ViewportSize { width, height })
}

/// Save screenshots to disk
fn save_screenshots(response: &RenderResponse) -> Result<()> {
    let mut saved_count = 0;

    for result in &response.results {
        if let Some(screenshot_data) = &result.screenshot_data {
            // Decode base64 screenshot data
            let decoded = general_purpose::STANDARD
                .decode(screenshot_data)
                .context("Failed to decode screenshot data")?;

            // Generate filename from URL
            let filename = generate_screenshot_filename(&result.url);

            // Save to file
            fs::write(&filename, decoded)
                .context(format!("Failed to write screenshot to {}", filename))?;

            output::print_success(&format!("Screenshot saved: {}", filename));
            saved_count += 1;
        }
    }

    if saved_count > 0 {
        output::print_info(&format!("Saved {} screenshot(s)", saved_count));
    }

    Ok(())
}

/// Generate a safe filename from a URL
fn generate_screenshot_filename(url: &str) -> String {
    // Extract domain and path for filename
    let clean_url = url.replace("https://", "").replace("http://", "");

    let sanitized = sanitize_filename(&clean_url);

    // Truncate if too long and add timestamp for uniqueness
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("System time should be after UNIX_EPOCH")
        .as_secs();

    let base = if sanitized.len() > 50 {
        &sanitized[..50]
    } else {
        &sanitized
    };

    format!("screenshot_{}_{}.png", base, timestamp)
}

/// Save results to file in JSON format
fn save_to_file(path: &str, response: &RenderResponse) -> Result<()> {
    let json =
        serde_json::to_string_pretty(response).context("Failed to serialize results to JSON")?;

    fs::write(path, json).context(format!("Failed to write results to {}", path))?;

    Ok(())
}

/// Print render results in the specified format
fn print_results(response: &RenderResponse, format: OutputFormat) -> Result<()> {
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
            // For render command, stream format is same as JSON
            let json = serde_json::to_string_pretty(response)?;
            println!("{}", json);
        }
    }
    Ok(())
}

/// Print results as a formatted table
fn print_table(response: &RenderResponse) -> Result<()> {
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
            "Render Time",
            "HTML Size",
            "Screenshot",
        ]);

    for result in &response.results {
        let status_cell = if result.status == "success" {
            Cell::new(&result.status).fg(Color::Green)
        } else {
            Cell::new(&result.status).fg(Color::Red)
        };

        let render_time = result
            .render_time_ms
            .map(|t| format!("{}ms", t))
            .unwrap_or_else(|| "-".to_string());

        let size = result
            .html_size
            .map(format_size)
            .unwrap_or_else(|| "-".to_string());

        let screenshot = if result.screenshot_data.is_some() {
            "✓"
        } else {
            "-"
        };

        table.add_row(vec![
            Cell::new(&result.url),
            status_cell,
            Cell::new(&render_time),
            Cell::new(&size),
            Cell::new(screenshot),
        ]);
    }

    println!("{}", table);
    print_summary(&response.summary);

    Ok(())
}

/// Print results as formatted text
fn print_text(response: &RenderResponse) -> Result<()> {
    println!("✓ Rendered {} URLs\n", response.summary.total);

    for result in &response.results {
        println!("URL: {}", result.url);
        println!("Status: {}", result.status);

        if let Some(render_time) = result.render_time_ms {
            println!("Render Time: {}ms", render_time);
        }

        if let Some(size) = result.html_size {
            println!("HTML Size: {}", format_size(size));
        }

        if result.screenshot_data.is_some() {
            println!("Screenshot: Available");
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
fn print_summary(summary: &RenderSummary) {
    println!("Summary:");
    println!("  Successful: {}", summary.successful);
    println!("  Failed: {}", summary.failed);

    if let Some(avg_time) = summary.avg_render_time_ms {
        println!("  Avg Render Time: {}ms", avg_time);
    }

    println!(
        "  Total HTML Size: {}",
        format_size(summary.total_html_size)
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_viewport_valid() {
        let viewport = parse_viewport("1920x1080").unwrap();
        assert_eq!(viewport.width, 1920);
        assert_eq!(viewport.height, 1080);

        let viewport = parse_viewport("1280x720").unwrap();
        assert_eq!(viewport.width, 1280);
        assert_eq!(viewport.height, 720);
    }

    #[test]
    fn test_parse_viewport_invalid_format() {
        assert!(parse_viewport("1920").is_err());
        assert!(parse_viewport("1920x1080x24").is_err());
        assert!(parse_viewport("abcxdef").is_err());
    }

    #[test]
    fn test_parse_viewport_bounds() {
        // Too small
        assert!(parse_viewport("100x100").is_err());
        // Too large
        assert!(parse_viewport("10000x10000").is_err());
        // Just right
        assert!(parse_viewport("1920x1080").is_ok());
    }

    #[test]
    fn test_validate_args_valid() {
        let args = RenderArgs {
            urls: vec!["https://example.com".to_string()],
            wait: 2000,
            screenshot: true,
            viewport: "1920x1080".to_string(),
            timeout: 30,
            output_file: None,
        };
        assert!(validate_args(&args).is_ok());
    }

    #[test]
    fn test_validate_args_invalid_viewport() {
        let args = RenderArgs {
            urls: vec!["https://example.com".to_string()],
            wait: 2000,
            screenshot: true,
            viewport: "invalid".to_string(),
            timeout: 30,
            output_file: None,
        };
        assert!(validate_args(&args).is_err());
    }

    #[test]
    fn test_validate_args_timeout_zero() {
        let args = RenderArgs {
            urls: vec!["https://example.com".to_string()],
            wait: 2000,
            screenshot: true,
            viewport: "1920x1080".to_string(),
            timeout: 0,
            output_file: None,
        };
        assert!(validate_args(&args).is_err());
    }

    #[test]
    fn test_validate_args_empty_urls() {
        let args = RenderArgs {
            urls: vec![],
            wait: 2000,
            screenshot: true,
            viewport: "1920x1080".to_string(),
            timeout: 30,
            output_file: None,
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
    fn test_generate_screenshot_filename() {
        let filename = generate_screenshot_filename("https://example.com/path/to/page");
        assert!(filename.starts_with("screenshot_"));
        assert!(filename.ends_with(".png"));
        assert!(filename.contains("example.com"));
    }
}
