use crate::output;
use anyhow::{Context, Result};
use riptide_stealth::StealthPreset;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Wait condition for page loading
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WaitCondition {
    /// Wait for page load event
    Load,
    /// Wait for network to be idle
    NetworkIdle,
    /// Wait for specific CSS selector
    Selector(String),
    /// Wait for specific timeout in milliseconds
    Timeout(u64),
}

impl WaitCondition {
    /// Parse wait condition from string
    pub fn from_str(s: &str) -> Result<Self> {
        if s == "load" {
            Ok(WaitCondition::Load)
        } else if s == "network-idle" {
            Ok(WaitCondition::NetworkIdle)
        } else if let Some(selector) = s.strip_prefix("selector:") {
            Ok(WaitCondition::Selector(selector.to_string()))
        } else if let Some(timeout_str) = s.strip_prefix("timeout:") {
            let timeout = timeout_str
                .parse::<u64>()
                .context("Invalid timeout value, expected number in milliseconds")?;
            Ok(WaitCondition::Timeout(timeout))
        } else {
            anyhow::bail!(
                "Invalid wait condition: {}. Must be one of: load, network-idle, selector:<css>, timeout:<ms>",
                s
            )
        }
    }

    /// Get human-readable description
    pub fn description(&self) -> String {
        match self {
            WaitCondition::Load => "page load event".to_string(),
            WaitCondition::NetworkIdle => "network idle".to_string(),
            WaitCondition::Selector(sel) => format!("selector: {}", sel),
            WaitCondition::Timeout(ms) => format!("timeout: {}ms", ms),
        }
    }
}

/// Screenshot capture mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenshotMode {
    /// No screenshot
    None,
    /// Viewport only
    Viewport,
    /// Full page screenshot
    Full,
}

impl ScreenshotMode {
    /// Parse screenshot mode from string
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "none" => Ok(ScreenshotMode::None),
            "viewport" => Ok(ScreenshotMode::Viewport),
            "full" => Ok(ScreenshotMode::Full),
            _ => anyhow::bail!(
                "Invalid screenshot mode: {}. Must be one of: none, viewport, full",
                s
            ),
        }
    }

    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            ScreenshotMode::None => "none",
            ScreenshotMode::Viewport => "viewport",
            ScreenshotMode::Full => "full",
        }
    }
}

/// Arguments for the render command
#[derive(clap::Args)]
pub struct RenderArgs {
    /// URL to render
    #[arg(long)]
    pub url: String,

    /// Wait condition (load, network-idle, selector:<css>, timeout:<ms>)
    #[arg(long, default_value = "load")]
    pub wait: String,

    /// Screenshot mode (none, viewport, full)
    #[arg(long, default_value = "none")]
    pub screenshot: String,

    /// Save rendered HTML
    #[arg(long)]
    pub html: bool,

    /// Save DOM tree
    #[arg(long)]
    pub dom: bool,

    /// Save PDF
    #[arg(long)]
    pub pdf: bool,

    /// Save HAR (HTTP Archive) file
    #[arg(long)]
    pub har: bool,

    /// Use session for cookies, headers, and authentication
    #[arg(long)]
    pub session: Option<String>,

    /// Cookie jar file to load cookies from
    #[arg(long)]
    pub cookie_jar: Option<String>,

    /// Storage state file (localStorage/sessionStorage)
    #[arg(long)]
    pub storage_state: Option<String>,

    /// Proxy URL (e.g., http://proxy.example.com:8080)
    #[arg(long)]
    pub proxy: Option<String>,

    /// Stealth level (off, low, med, high, auto)
    #[arg(long, default_value = "off")]
    pub stealth: String,

    /// Output directory for saved files
    #[arg(long, short = 'o', default_value = ".")]
    pub output_dir: String,

    /// Output file prefix (default: derived from URL)
    #[arg(long)]
    pub prefix: Option<String>,

    /// Viewport width
    #[arg(long, default_value = "1920")]
    pub width: u32,

    /// Viewport height
    #[arg(long, default_value = "1080")]
    pub height: u32,

    /// Custom user agent
    #[arg(long)]
    pub user_agent: Option<String>,

    /// Enable JavaScript execution
    #[arg(long, default_value = "true")]
    pub javascript: bool,

    /// Additional timeout in seconds after wait condition
    #[arg(long, default_value = "0")]
    pub extra_timeout: u64,
}

/// Rendering result metadata
#[derive(Serialize, Deserialize)]
pub struct RenderResult {
    pub url: String,
    pub wait_condition: String,
    pub screenshot_mode: String,
    pub stealth_level: String,
    pub viewport: Viewport,
    pub files_saved: Vec<SavedFile>,
    pub render_time_ms: u64,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<RenderMetadata>,
}

#[derive(Serialize, Deserialize)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
}

#[derive(Serialize, Deserialize)]
pub struct SavedFile {
    pub file_type: String,
    pub path: String,
    pub size_bytes: u64,
}

#[derive(Serialize, Deserialize)]
pub struct RenderMetadata {
    pub title: Option<String>,
    pub final_url: Option<String>,
    pub resources_loaded: Option<u32>,
    pub cookies_set: Option<u32>,
    pub storage_items: Option<u32>,
}

/// Execute the render command
pub async fn execute(args: RenderArgs, output_format: &str) -> Result<()> {
    use crate::metrics::MetricsManager;
    use std::time::Instant;

    // Start metrics tracking
    let metrics_manager = MetricsManager::global();
    let tracking_id = metrics_manager.start_command("render").await?;
    let overall_start = Instant::now();

    output::print_info(&format!("Rendering page: {}", args.url));

    // Parse wait condition
    let wait_condition = WaitCondition::from_str(&args.wait)?;
    output::print_info(&format!("Wait condition: {}", wait_condition.description()));

    // Parse screenshot mode
    let screenshot_mode = ScreenshotMode::from_str(&args.screenshot)?;
    if screenshot_mode != ScreenshotMode::None {
        output::print_info(&format!("Screenshot mode: {}", screenshot_mode.name()));
    }

    // Parse stealth level
    let stealth_preset = parse_stealth_level(&args.stealth)?;
    if stealth_preset != StealthPreset::None {
        output::print_info(&format!("Stealth level: {}", args.stealth));
    }

    // Create output directory if needed
    if args.html || args.dom || args.pdf || args.har || screenshot_mode != ScreenshotMode::None {
        fs::create_dir_all(&args.output_dir).context("Failed to create output directory")?;
    }

    // Generate file prefix from URL if not provided
    let file_prefix = args
        .prefix
        .clone()
        .unwrap_or_else(|| generate_file_prefix(&args.url));

    let start_time = Instant::now();

    // Execute rendering based on availability
    let result = if is_headless_available() {
        output::print_info("Using headless browser for rendering");
        execute_headless_render(
            &args,
            &wait_condition,
            screenshot_mode,
            stealth_preset,
            &file_prefix,
        )
        .await
    } else {
        output::print_warning("Headless browser not available, using fallback HTTP rendering");
        execute_fallback_render(&args, &file_prefix).await
    }?;

    let render_time = start_time.elapsed();

    // Create result summary
    let render_result = RenderResult {
        url: args.url.clone(),
        wait_condition: wait_condition.description(),
        screenshot_mode: screenshot_mode.name().to_string(),
        stealth_level: args.stealth.clone(),
        viewport: Viewport {
            width: args.width,
            height: args.height,
        },
        files_saved: result.files_saved,
        render_time_ms: render_time.as_millis() as u64,
        success: result.success,
        error: result.error,
        metadata: result.metadata,
    };

    // Output results
    output_render_result(&render_result, output_format)?;

    // Complete metrics tracking
    if render_result.success {
        let bytes = render_result
            .files_saved
            .iter()
            .map(|f| f.size_bytes)
            .sum::<u64>();
        metrics_manager
            .record_progress(
                &tracking_id,
                render_result.files_saved.len() as u64,
                bytes,
                0,
                0,
            )
            .await?;
        metrics_manager
            .collector()
            .record_metric("render.duration_ms", render_result.render_time_ms as f64)?;
        metrics_manager
            .collector()
            .record_metric(&format!("render.wait.{}", args.wait), 1.0)?;
        metrics_manager.complete_command(&tracking_id).await?;
    } else {
        let error_msg = render_result.error.as_deref().unwrap_or("Unknown error");
        metrics_manager
            .fail_command(&tracking_id, error_msg)
            .await?;
    }

    Ok(())
}

/// Check if headless browser rendering is available
fn is_headless_available() -> bool {
    // TODO: Implement actual headless browser detection
    // For now, return false to use fallback
    false
}

/// Result of rendering operation
struct RenderOutput {
    files_saved: Vec<SavedFile>,
    success: bool,
    error: Option<String>,
    metadata: Option<RenderMetadata>,
}

/// Execute rendering with headless browser
async fn execute_headless_render(
    args: &RenderArgs,
    _wait_condition: &WaitCondition,
    _screenshot_mode: ScreenshotMode,
    _stealth_preset: StealthPreset,
    file_prefix: &str,
) -> Result<RenderOutput> {
    // TODO: Implement headless browser rendering
    // This would use chromiumoxide or similar for browser automation

    output::print_warning("Headless browser rendering not yet implemented");
    output::print_info("Falling back to HTTP-based rendering");

    execute_fallback_render(args, file_prefix).await
}

/// Execute fallback HTTP-based rendering
async fn execute_fallback_render(args: &RenderArgs, file_prefix: &str) -> Result<RenderOutput> {
    use std::time::Duration;

    let mut files_saved = Vec::new();

    // Configure HTTP client
    let mut client_builder = reqwest::Client::builder().timeout(Duration::from_secs(30));

    // Apply user agent
    if let Some(ref ua) = args.user_agent {
        client_builder = client_builder.user_agent(ua);
    } else {
        client_builder = client_builder.user_agent("RipTide-Renderer/1.0");
    }

    // Apply proxy
    if let Some(ref proxy_url) = args.proxy {
        let proxy = reqwest::Proxy::all(proxy_url)?;
        client_builder = client_builder.proxy(proxy);
    }

    let client = client_builder.build()?;

    // Fetch HTML content
    output::print_info("Fetching page content...");
    let response = client.get(&args.url).send().await?;

    let status = response.status();
    let final_url = response.url().to_string();
    let html = response.text().await?;

    output::print_info(&format!("Received {} bytes (HTTP {})", html.len(), status));

    // Save HTML if requested
    if args.html {
        let html_path = Path::new(&args.output_dir).join(format!("{}.html", file_prefix));

        fs::write(&html_path, &html).context("Failed to write HTML file")?;

        let size = html.len() as u64;
        files_saved.push(SavedFile {
            file_type: "html".to_string(),
            path: html_path.to_string_lossy().to_string(),
            size_bytes: size,
        });

        output::print_success(&format!("Saved HTML to: {}", html_path.display()));
    }

    // Save DOM tree if requested
    if args.dom {
        let dom_json = extract_dom_tree(&html)?;
        let dom_path = Path::new(&args.output_dir).join(format!("{}.dom.json", file_prefix));

        fs::write(&dom_path, &dom_json).context("Failed to write DOM file")?;

        let size = dom_json.len() as u64;
        files_saved.push(SavedFile {
            file_type: "dom".to_string(),
            path: dom_path.to_string_lossy().to_string(),
            size_bytes: size,
        });

        output::print_success(&format!("Saved DOM to: {}", dom_path.display()));
    }

    // Extract metadata
    let metadata = Some(RenderMetadata {
        title: extract_title(&html),
        final_url: Some(final_url),
        resources_loaded: None, // Not available in fallback mode
        cookies_set: None,
        storage_items: None,
    });

    Ok(RenderOutput {
        files_saved,
        success: true,
        error: None,
        metadata,
    })
}

/// Extract page title from HTML
fn extract_title(html: &str) -> Option<String> {
    use scraper::{Html, Selector};

    let document = Html::parse_document(html);
    let title_selector = Selector::parse("title").ok()?;

    document
        .select(&title_selector)
        .next()
        .map(|element| element.inner_html().trim().to_string())
}

/// Extract DOM tree as JSON
fn extract_dom_tree(html: &str) -> Result<String> {
    use serde_json::json;

    // Create simplified DOM representation
    // Full DOM tree extraction requires headless browser support
    let dom = json!({
        "type": "document",
        "html_length": html.len(),
        "note": "Full DOM tree extraction requires headless browser support"
    });

    serde_json::to_string_pretty(&dom).context("Failed to serialize DOM tree")
}

/// Generate file prefix from URL
fn generate_file_prefix(url: &str) -> String {
    use url::Url;

    if let Ok(parsed_url) = Url::parse(url) {
        let host = parsed_url.host_str().unwrap_or("page");
        let path = parsed_url.path();

        // Create safe filename from host and path
        let mut prefix = host.replace('.', "_");

        if path != "/" && !path.is_empty() {
            let path_part = path
                .trim_start_matches('/')
                .trim_end_matches('/')
                .replace('/', "_")
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
                .take(50)
                .collect::<String>();

            if !path_part.is_empty() {
                prefix.push('_');
                prefix.push_str(&path_part);
            }
        }

        prefix
    } else {
        "render_output".to_string()
    }
}

/// Parse stealth level string to preset
fn parse_stealth_level(level: &str) -> Result<StealthPreset> {
    match level.to_lowercase().as_str() {
        "off" | "none" => Ok(StealthPreset::None),
        "low" => Ok(StealthPreset::Low),
        "med" | "medium" => Ok(StealthPreset::Medium),
        "high" => Ok(StealthPreset::High),
        "auto" => Ok(StealthPreset::Medium), // Default to medium for auto
        _ => anyhow::bail!(
            "Invalid stealth level: {}. Must be one of: off, low, med, high, auto",
            level
        ),
    }
}

/// Output rendering results in specified format
fn output_render_result(result: &RenderResult, output_format: &str) -> Result<()> {
    match output_format {
        "json" => {
            output::print_json(&result);
        }
        "text" => {
            if result.success {
                output::print_success("Page rendered successfully");
            } else {
                output::print_error("Page rendering failed");
                if let Some(ref error) = result.error {
                    println!("Error: {}", error);
                }
                return Ok(());
            }

            println!();
            output::print_key_value("URL", &result.url);
            output::print_key_value("Wait Condition", &result.wait_condition);
            output::print_key_value("Render Time", &format!("{}ms", result.render_time_ms));
            output::print_key_value(
                "Viewport",
                &format!("{}x{}", result.viewport.width, result.viewport.height),
            );

            if let Some(ref metadata) = result.metadata {
                println!();
                output::print_section("Page Metadata");

                if let Some(ref title) = metadata.title {
                    output::print_key_value("Title", title);
                }
                if let Some(ref final_url) = metadata.final_url {
                    output::print_key_value("Final URL", final_url);
                }
            }

            if !result.files_saved.is_empty() {
                println!();
                output::print_section("Files Saved");

                for file in &result.files_saved {
                    println!(
                        "  {} [{}] - {} bytes",
                        file.path, file.file_type, file.size_bytes
                    );
                }
            }
        }
        "table" => {
            let mut table = output::create_table(vec!["Field", "Value"]);

            table.add_row(vec!["URL", &result.url]);
            table.add_row(vec![
                "Status",
                if result.success { "Success" } else { "Failed" },
            ]);
            table.add_row(vec!["Render Time", &format!("{}ms", result.render_time_ms)]);
            table.add_row(vec![
                "Viewport",
                &format!("{}x{}", result.viewport.width, result.viewport.height),
            ]);
            table.add_row(vec!["Wait Condition", &result.wait_condition]);
            table.add_row(vec!["Screenshot", &result.screenshot_mode]);
            table.add_row(vec!["Stealth Level", &result.stealth_level]);
            table.add_row(vec!["Files Saved", &result.files_saved.len().to_string()]);

            if let Some(ref metadata) = result.metadata {
                if let Some(ref title) = metadata.title {
                    table.add_row(vec!["Page Title", title]);
                }
            }

            println!("{table}");

            if !result.files_saved.is_empty() {
                println!();
                output::print_section("Saved Files");
                for file in &result.files_saved {
                    println!("  • {} ({})", file.path, file.file_type);
                }
            }
        }
        _ => {
            output::print_warning(&format!("Unknown output format: {}", output_format));
            output::print_json(&result);
        }
    }

    Ok(())
}
