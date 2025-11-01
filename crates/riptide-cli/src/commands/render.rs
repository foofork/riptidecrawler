use crate::output;
use anyhow::{Context, Result};
use riptide_stealth::StealthPreset;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::str::FromStr;

// Headless browser support
use riptide_browser::launcher::{HeadlessLauncher, LauncherConfig};
use riptide_browser::pool::BrowserPoolConfig;

// chromiumoxide types for screenshot and PDF generation (re-exported by spider_chrome)
use chromiumoxide::cdp::browser_protocol::page::PrintToPdfParams;
use chromiumoxide::page::ScreenshotParams;

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

impl std::str::FromStr for WaitCondition {
    type Err = anyhow::Error;

    /// Parse wait condition from string
    fn from_str(s: &str) -> Result<Self> {
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
}

impl WaitCondition {
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

impl std::str::FromStr for ScreenshotMode {
    type Err = anyhow::Error;

    /// Parse screenshot mode from string
    fn from_str(s: &str) -> Result<Self> {
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
}

impl ScreenshotMode {
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

    /// Output directory for saved files (defaults to RIPTIDE_OUTPUT_DIR or platform-specific data directory)
    #[arg(long, short = 'o')]
    pub output_dir: Option<String>,

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

    /// Force direct execution mode (no API)
    #[arg(long)]
    pub direct: bool,

    /// API-only mode (fail if API unavailable, no fallback)
    #[arg(long)]
    pub api_only: bool,
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
    use crate::config;
    use crate::execution_mode::{get_execution_mode, ExecutionMode};
    use crate::metrics::MetricsManager;
    use std::time::Instant;

    // Start metrics tracking
    let metrics_manager = MetricsManager::global();
    let tracking_id = metrics_manager.start_command("render").await?;
    let _overall_start = Instant::now();

    // Determine execution mode
    let execution_mode = get_execution_mode(args.direct, args.api_only);
    output::print_info(&format!("Execution mode: {}", execution_mode.description()));

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

    // Determine output directory: CLI arg > env var > default
    let output_dir = args
        .output_dir
        .clone()
        .unwrap_or_else(|| config::get_output_directory().to_string_lossy().to_string());

    // Create output directory if needed
    if args.html || args.dom || args.pdf || args.har || screenshot_mode != ScreenshotMode::None {
        fs::create_dir_all(&output_dir).context("Failed to create output directory")?;
    }

    // Generate file prefix from URL if not provided
    let file_prefix = args
        .prefix
        .clone()
        .unwrap_or_else(|| generate_file_prefix(&args.url));

    let start_time = Instant::now();

    // Execute rendering based on execution mode
    let result = match execution_mode {
        ExecutionMode::ApiFirst => {
            // Try API first, fallback to direct if unavailable
            if let Some(api_client) = try_create_api_client()? {
                match execute_api_render(
                    &api_client,
                    &args,
                    &wait_condition,
                    screenshot_mode,
                    &file_prefix,
                    &output_dir,
                )
                .await
                {
                    Ok(res) => {
                        output::print_success("Rendered via API");
                        res
                    }
                    Err(e) => {
                        output::print_warning(&format!(
                            "API render failed: {}. Falling back to direct execution...",
                            e
                        ));
                        execute_headless_render(
                            &args,
                            &wait_condition,
                            screenshot_mode,
                            stealth_preset,
                            &file_prefix,
                            &output_dir,
                        )
                        .await?
                    }
                }
            } else {
                output::print_warning("API not available, using direct execution");
                execute_headless_render(
                    &args,
                    &wait_condition,
                    screenshot_mode,
                    stealth_preset,
                    &file_prefix,
                    &output_dir,
                )
                .await?
            }
        }
        ExecutionMode::ApiOnly => {
            // API only, fail if unavailable
            let api_client = try_create_api_client()?.context(
                "API-only mode requires RIPTIDE_API_URL environment variable or --api-url flag",
            )?;

            execute_api_render(
                &api_client,
                &args,
                &wait_condition,
                screenshot_mode,
                &file_prefix,
                &output_dir,
            )
            .await
            .context("API render failed in API-only mode")?
        }
        ExecutionMode::DirectOnly => {
            // Direct execution only
            output::print_info("Using direct headless browser execution");
            execute_headless_render(
                &args,
                &wait_condition,
                screenshot_mode,
                stealth_preset,
                &file_prefix,
                &output_dir,
            )
            .await?
        }
    };

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

/// Try to create API client from environment
fn try_create_api_client() -> Result<Option<crate::api_client::RiptideApiClient>> {
    // Try to get API URL from environment
    let api_url = std::env::var("RIPTIDE_API_URL").ok();
    let api_key = std::env::var("RIPTIDE_API_KEY").ok();

    if let Some(url) = api_url {
        let client = crate::api_client::RiptideApiClient::new(url, api_key)?;
        Ok(Some(client))
    } else {
        Ok(None)
    }
}

/// Execute rendering via API
async fn execute_api_render(
    api_client: &crate::api_client::RiptideApiClient,
    args: &RenderArgs,
    wait_condition: &WaitCondition,
    screenshot_mode: ScreenshotMode,
    file_prefix: &str,
    output_dir: &str,
) -> Result<RenderOutput> {
    use crate::api_client::{RenderRequest, ViewportConfig};

    output::print_info("Checking API availability...");

    // Check if API is available
    if !api_client.is_available().await {
        anyhow::bail!("API server is not available at {}", api_client.base_url());
    }

    output::print_info("Sending render request to API...");

    // Build API request
    let request = RenderRequest {
        url: args.url.clone(),
        wait_condition: wait_condition.description(),
        screenshot_mode: screenshot_mode.name().to_string(),
        viewport: ViewportConfig {
            width: args.width,
            height: args.height,
        },
        stealth_level: args.stealth.clone(),
        javascript_enabled: args.javascript,
        extra_timeout: args.extra_timeout,
        user_agent: args.user_agent.clone(),
        proxy: args.proxy.clone(),
        session_id: args.session.clone(),
    };

    // Execute API request
    let api_response = api_client.render(request).await?;

    if !api_response.success {
        anyhow::bail!(
            "API render failed: {}",
            api_response
                .error
                .unwrap_or_else(|| "Unknown error".to_string())
        );
    }

    // Save files from API response
    let mut files_saved = Vec::new();

    if args.html {
        if let Some(html) = api_response.html {
            let html_path = Path::new(output_dir).join(format!("{}.html", file_prefix));
            fs::write(&html_path, &html).context("Failed to write HTML file")?;

            let size = html.len() as u64;
            files_saved.push(SavedFile {
                file_type: "html".to_string(),
                path: html_path.to_string_lossy().to_string(),
                size_bytes: size,
            });

            output::print_success(&format!("HTML saved to: {}", html_path.display()));
        }
    }

    if args.dom {
        if let Some(dom) = api_response.dom {
            let dom_path = Path::new(output_dir).join(format!("{}.dom.json", file_prefix));
            fs::write(&dom_path, &dom).context("Failed to write DOM file")?;

            let size = dom.len() as u64;
            files_saved.push(SavedFile {
                file_type: "dom".to_string(),
                path: dom_path.to_string_lossy().to_string(),
                size_bytes: size,
            });

            output::print_success(&format!("DOM saved to: {}", dom_path.display()));
        }
    }

    if screenshot_mode != ScreenshotMode::None {
        if let Some(screenshot_bytes) = api_response.screenshot {
            let screenshot_path = Path::new(output_dir).join(format!("{}.png", file_prefix));
            fs::write(&screenshot_path, &screenshot_bytes).context("Failed to write screenshot")?;

            let size = screenshot_bytes.len() as u64;
            files_saved.push(SavedFile {
                file_type: "screenshot".to_string(),
                path: screenshot_path.to_string_lossy().to_string(),
                size_bytes: size,
            });

            output::print_success(&format!(
                "Screenshot saved to: {}",
                screenshot_path.display()
            ));
        }
    }

    if args.pdf {
        if let Some(pdf_bytes) = api_response.pdf {
            let pdf_path = Path::new(output_dir).join(format!("{}.pdf", file_prefix));
            fs::write(&pdf_path, &pdf_bytes).context("Failed to write PDF")?;

            let size = pdf_bytes.len() as u64;
            files_saved.push(SavedFile {
                file_type: "pdf".to_string(),
                path: pdf_path.to_string_lossy().to_string(),
                size_bytes: size,
            });

            output::print_success(&format!("PDF saved to: {}", pdf_path.display()));
        }
    }

    if args.har {
        if let Some(har) = api_response.har {
            let har_path = Path::new(output_dir).join(format!("{}.har", file_prefix));
            fs::write(&har_path, &har).context("Failed to write HAR file")?;

            let size = har.len() as u64;
            files_saved.push(SavedFile {
                file_type: "har".to_string(),
                path: har_path.to_string_lossy().to_string(),
                size_bytes: size,
            });

            output::print_success(&format!("HAR saved to: {}", har_path.display()));
        }
    }

    // Convert API metadata to local format
    let metadata = Some(RenderMetadata {
        title: api_response.metadata.title,
        final_url: Some(api_response.metadata.final_url),
        resources_loaded: Some(api_response.metadata.resources_loaded),
        cookies_set: Some(api_response.metadata.cookies_set),
        storage_items: None,
    });

    Ok(RenderOutput {
        files_saved,
        success: true,
        error: None,
        metadata,
    })
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
    wait_condition: &WaitCondition,
    screenshot_mode: ScreenshotMode,
    stealth_preset: StealthPreset,
    file_prefix: &str,
    output_dir: &str,
) -> Result<RenderOutput> {
    use std::time::Instant;

    let start_time = Instant::now();
    let mut files_saved = Vec::new();

    output::print_info("Initializing headless browser launcher...");

    // Configure headless launcher with stealth and timeout settings
    let launcher_config = LauncherConfig {
        pool_config: BrowserPoolConfig {
            initial_pool_size: 1,
            min_pool_size: 1,
            max_pool_size: 3,
            idle_timeout: std::time::Duration::from_secs(30),
            ..Default::default()
        },
        default_stealth_preset: stealth_preset.clone(),
        enable_stealth: stealth_preset != StealthPreset::None,
        page_timeout: std::time::Duration::from_secs(30),
        enable_monitoring: false,
        hybrid_mode: false,
    };

    // Initialize launcher
    let launcher = HeadlessLauncher::with_config(launcher_config)
        .await
        .context("Failed to initialize headless launcher")?;

    // Launch browser page with stealth if configured
    output::print_info(&format!("Launching browser for: {}", args.url));
    let session = launcher
        .launch_page(&args.url, Some(stealth_preset))
        .await
        .context("Failed to launch browser page")?;

    let page = session.page();

    // Apply wait condition
    output::print_info(&format!("Waiting for: {}", wait_condition.description()));
    match wait_condition {
        WaitCondition::Load => {
            // Wait for page load event
            if let Err(e) = tokio::time::timeout(
                std::time::Duration::from_secs(10),
                page.wait_for_navigation(),
            )
            .await
            {
                output::print_warning(&format!("Page load timeout: {:?}", e));
            }
        }
        WaitCondition::NetworkIdle => {
            // Wait for page load then additional idle time
            let _ = tokio::time::timeout(
                std::time::Duration::from_secs(5),
                page.wait_for_navigation(),
            )
            .await;

            // Additional idle wait
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
        WaitCondition::Selector(selector) => {
            // Wait for specific selector
            if let Err(e) = tokio::time::timeout(
                std::time::Duration::from_secs(10),
                page.find_element(selector),
            )
            .await
            {
                output::print_warning(&format!("Selector wait timeout: {:?}", e));
            }
        }
        WaitCondition::Timeout(ms) => {
            // Simple timeout wait
            tokio::time::sleep(std::time::Duration::from_millis(*ms)).await;
        }
    }

    // Apply extra timeout if specified
    if args.extra_timeout > 0 {
        output::print_info(&format!("Applying extra timeout: {}s", args.extra_timeout));
        tokio::time::sleep(std::time::Duration::from_secs(args.extra_timeout)).await;
    }

    // Capture screenshot if requested
    if screenshot_mode != ScreenshotMode::None {
        output::print_info(&format!("Capturing screenshot: {}", screenshot_mode.name()));

        let mut screenshot_params = ScreenshotParams::builder();

        // Configure screenshot based on mode
        match screenshot_mode {
            ScreenshotMode::Full => {
                screenshot_params = screenshot_params.full_page(true);
            }
            ScreenshotMode::Viewport => {
                screenshot_params = screenshot_params.full_page(false);
            }
            ScreenshotMode::None => unreachable!(),
        }

        match tokio::time::timeout(
            std::time::Duration::from_secs(10),
            page.screenshot(screenshot_params.build()),
        )
        .await
        {
            Ok(Ok(screenshot_data)) => {
                let screenshot_path = Path::new(output_dir).join(format!("{}.png", file_prefix));

                fs::write(&screenshot_path, &screenshot_data)
                    .context("Failed to write screenshot file")?;

                let size = screenshot_data.len() as u64;
                files_saved.push(SavedFile {
                    file_type: "screenshot".to_string(),
                    path: screenshot_path.to_string_lossy().to_string(),
                    size_bytes: size,
                });

                output::print_success(&format!(
                    "Screenshot saved to: {}",
                    screenshot_path.display()
                ));
            }
            Ok(Err(e)) => {
                output::print_warning(&format!("Screenshot capture failed: {}", e));
            }
            Err(_) => {
                output::print_warning("Screenshot capture timed out");
            }
        }
    }

    // Extract HTML content if requested
    if args.html {
        output::print_info("Extracting rendered HTML...");

        match tokio::time::timeout(std::time::Duration::from_secs(5), page.content()).await {
            Ok(Ok(html)) => {
                let html_path = Path::new(output_dir).join(format!("{}.html", file_prefix));

                fs::write(&html_path, &html).context("Failed to write HTML file")?;

                let size = html.len() as u64;
                files_saved.push(SavedFile {
                    file_type: "html".to_string(),
                    path: html_path.to_string_lossy().to_string(),
                    size_bytes: size,
                });

                output::print_success(&format!("HTML saved to: {}", html_path.display()));
            }
            Ok(Err(e)) => {
                output::print_warning(&format!("HTML extraction failed: {}", e));
            }
            Err(_) => {
                output::print_warning("HTML extraction timed out");
            }
        }
    }

    // Extract DOM tree if requested
    if args.dom {
        output::print_info("Extracting DOM tree...");

        // Use JavaScript to extract simplified DOM structure
        let dom_script = r#"
            JSON.stringify({
                type: 'document',
                title: document.title,
                url: window.location.href,
                html_length: document.documentElement.outerHTML.length,
                links_count: document.links.length,
                images_count: document.images.length,
                scripts_count: document.scripts.length,
                stylesheets_count: document.styleSheets.length,
                meta_tags: Array.from(document.querySelectorAll('meta')).map(m => ({
                    name: m.getAttribute('name'),
                    property: m.getAttribute('property'),
                    content: m.getAttribute('content')
                }))
            }, null, 2)
        "#;

        match tokio::time::timeout(std::time::Duration::from_secs(5), page.evaluate(dom_script))
            .await
        {
            Ok(Ok(result)) => {
                if let Ok(dom_json) = result.into_value::<String>() {
                    let dom_path = Path::new(output_dir).join(format!("{}.dom.json", file_prefix));

                    fs::write(&dom_path, &dom_json).context("Failed to write DOM file")?;

                    let size = dom_json.len() as u64;
                    files_saved.push(SavedFile {
                        file_type: "dom".to_string(),
                        path: dom_path.to_string_lossy().to_string(),
                        size_bytes: size,
                    });

                    output::print_success(&format!("DOM saved to: {}", dom_path.display()));
                }
            }
            Ok(Err(e)) => {
                output::print_warning(&format!("DOM extraction failed: {}", e));
            }
            Err(_) => {
                output::print_warning("DOM extraction timed out");
            }
        }
    }

    // Generate PDF if requested
    if args.pdf {
        output::print_info("Generating PDF...");

        let pdf_params = PrintToPdfParams::default();

        match tokio::time::timeout(std::time::Duration::from_secs(10), page.pdf(pdf_params)).await {
            Ok(Ok(pdf_data)) => {
                let pdf_path = Path::new(output_dir).join(format!("{}.pdf", file_prefix));

                fs::write(&pdf_path, &pdf_data).context("Failed to write PDF file")?;

                let size = pdf_data.len() as u64;
                files_saved.push(SavedFile {
                    file_type: "pdf".to_string(),
                    path: pdf_path.to_string_lossy().to_string(),
                    size_bytes: size,
                });

                output::print_success(&format!("PDF saved to: {}", pdf_path.display()));
            }
            Ok(Err(e)) => {
                output::print_warning(&format!("PDF generation failed: {}", e));
            }
            Err(_) => {
                output::print_warning("PDF generation timed out");
            }
        }
    }

    // HAR archive generation (if requested)
    if args.har {
        output::print_info("Generating HAR archive...");
        // Note: HAR generation requires CDP protocol access
        // This is a placeholder for now - full implementation would need CDP command access
        output::print_warning("HAR archive generation requires additional CDP protocol support (not yet fully implemented)");
    }

    // Get page title and final URL
    let title = page
        .evaluate("document.title")
        .await
        .ok()
        .and_then(|r| r.into_value::<String>().ok());

    let final_url = page.url().await.ok().flatten();

    // Shutdown launcher to clean up resources
    launcher
        .shutdown()
        .await
        .context("Failed to shutdown launcher")?;

    let duration = start_time.elapsed();
    output::print_success(&format!(
        "Rendering completed in {:.2}s",
        duration.as_secs_f64()
    ));

    // Extract metadata
    let metadata = Some(RenderMetadata {
        title,
        final_url,
        resources_loaded: None, // Would need CDP for accurate count
        cookies_set: None,      // Would need CDP for cookie tracking
        storage_items: None,    // Would need CDP for storage inspection
    });

    Ok(RenderOutput {
        files_saved,
        success: true,
        error: None,
        metadata,
    })
}

// Legacy HTTP-based fallback functions removed (superseded by spider-chrome)
// These functions were part of the old HTTP-only rendering path:
// - execute_fallback_render() - Basic HTTP fetch without JS support
// - extract_title() - Simple HTML title extraction via scraper crate
// - extract_dom_tree() - Simplified DOM representation
//
// Modern path uses spider-chrome for full browser automation with:
// - JavaScript execution
// - Dynamic content rendering
// - Full DOM access via CDP
// - Network interception and control
//
// If HTTP-only fallback is needed, use spider-chrome in HTTP-first mode
// or implement via the extraction engine with EngineType::Raw

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
