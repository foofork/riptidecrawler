use crate::client::RipTideClient;
use crate::commands::ExtractArgs;
use crate::output;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

// Local extraction support (only with wasm-extractor feature)
#[cfg(feature = "wasm-extractor")]
use riptide_extraction::wasm_extraction::WasmExtractor;

// Headless browser support
use riptide_browser::launcher::{HeadlessLauncher, LauncherConfig};
use riptide_browser::pool::BrowserPoolConfig;

// Engine selection from consolidated module
use riptide_reliability::engine_selection::Engine;

#[derive(Serialize)]
struct ExtractRequest {
    url: String,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    strategy: Option<String>,
    include_confidence: bool,
}

#[derive(Deserialize, Serialize)]
pub struct ExtractResponse {
    pub content: String,
    #[serde(default)]
    pub confidence: Option<f64>,
    #[serde(default)]
    pub method_used: Option<String>,
    #[serde(default)]
    pub extraction_time_ms: Option<u64>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

pub async fn execute(client: RipTideClient, args: ExtractArgs, output_format: &str) -> Result<()> {
    use crate::metrics::MetricsManager;
    use std::time::Instant;

    // Start metrics tracking
    let metrics_manager = MetricsManager::global();
    let tracking_id = metrics_manager.start_command("extract").await?;
    let overall_start = Instant::now();

    // Determine input source with priority: stdin > file > url
    let (html_content, source_url) = if args.stdin {
        output::print_info("Reading HTML from stdin...");
        use std::io::Read;
        let mut buffer = String::new();
        std::io::stdin().read_to_string(&mut buffer)?;
        (Some(buffer), "stdin".to_string())
    } else if let Some(ref input_file) = args.input_file {
        output::print_info(&format!("Reading HTML from file: {}", input_file));
        let content = fs::read_to_string(input_file)?;
        (Some(content), input_file.clone())
    } else if let Some(ref url) = args.url {
        output::print_info(&format!("Extracting content from: {}", url));
        (None, url.clone())
    } else {
        anyhow::bail!("At least one input source is required: --url, --input-file, or --stdin");
    };

    // Parse engine selection
    let engine = args.engine.parse::<Engine>()?;
    output::print_info(&format!("Engine mode: {}", engine.name()));

    // Record engine selection in metadata
    let _ = metrics_manager
        .collector()
        .record_metric(&format!("extract.engine.{}", engine.name()), 1.0);

    // If we have HTML content directly (from file or stdin), use local extraction
    if let Some(html) = html_content {
        let result = execute_direct_extraction(html, source_url, args, output_format, engine).await;

        // Complete metrics tracking
        match &result {
            Ok(_) => {
                let duration = overall_start.elapsed();
                metrics_manager
                    .collector()
                    .record_metric("extract.duration_ms", duration.as_millis() as f64)?;
                metrics_manager.complete_command(&tracking_id).await?;
            }
            Err(e) => {
                metrics_manager
                    .fail_command(&tracking_id, e.to_string())
                    .await?;
            }
        }

        return result;
    }

    // URL-based extraction follows
    // Use local extraction if --local flag is set
    if args.local {
        let result = execute_local_extraction(args, output_format, engine).await;

        // Complete metrics tracking
        match &result {
            Ok(_) => {
                let duration = overall_start.elapsed();
                metrics_manager
                    .collector()
                    .record_metric("extract.duration_ms", duration.as_millis() as f64)?;
                metrics_manager.complete_command(&tracking_id).await?;
            }
            Err(e) => {
                metrics_manager
                    .fail_command(&tracking_id, e.to_string())
                    .await?;
            }
        }

        return result;
    }

    // API server extraction
    let url = if let Some(url) = args.url.as_ref() {
        url
    } else {
        return Err(anyhow::anyhow!("URL required for extraction"));
    };
    let request = ExtractRequest {
        url: url.clone(),
        method: args.method.clone(),
        selector: args.selector,
        pattern: args.pattern,
        strategy: args.strategy,
        include_confidence: args.show_confidence,
    };

    let api_start = Instant::now();
    let response = client.post("/api/v1/extract", &request).await?;
    let api_latency = api_start.elapsed();

    // Record API call metrics
    metrics_manager
        .record_progress(&tracking_id, 0, 0, 0, 1)
        .await?;
    metrics_manager
        .collector()
        .record_metric("extract.api.latency_ms", api_latency.as_millis() as f64)?;

    let extract_result: ExtractResponse = response.json().await?;

    // Record response size
    let response_size = extract_result.content.len() as u64;
    metrics_manager
        .record_progress(&tracking_id, 1, response_size, 0, 1)
        .await?;

    match output_format {
        "json" => {
            output::print_json(&extract_result);
        }
        "text" => {
            output::print_success("Content extracted successfully");
            println!();

            if args.show_confidence {
                if let Some(confidence) = extract_result.confidence {
                    output::print_key_value("Confidence", &output::format_confidence(confidence));
                }
            }

            if let Some(method) = extract_result.method_used {
                output::print_key_value("Method", &method);
            }

            if let Some(time) = extract_result.extraction_time_ms {
                output::print_key_value("Extraction Time", &format!("{}ms", time));
            }

            if args.metadata {
                if let Some(metadata) = extract_result.metadata {
                    output::print_section("Metadata");
                    println!("{}", serde_json::to_string_pretty(&metadata)?);
                }
            }

            output::print_section("Extracted Content");
            println!("{}", extract_result.content);

            // Save to file if specified
            if let Some(file_path) = args.file {
                fs::write(&file_path, &extract_result.content)?;
                output::print_success(&format!("Content saved to: {}", file_path));
            }
        }
        "table" => {
            let mut table = output::create_table(vec!["Field", "Value"]);
            table.add_row(vec!["Source", url]);

            if let Some(confidence) = extract_result.confidence {
                table.add_row(vec!["Confidence", &output::format_confidence(confidence)]);
            }

            if let Some(method) = extract_result.method_used {
                table.add_row(vec!["Method", &method]);
            }

            if let Some(time) = extract_result.extraction_time_ms {
                table.add_row(vec!["Time", &format!("{}ms", time)]);
            }

            table.add_row(vec![
                "Content Length",
                &format!("{} chars", extract_result.content.len()),
            ]);

            println!("{table}");
        }
        _ => {
            output::print_warning(&format!("Unknown output format: {}", output_format));
            output::print_json(&extract_result);
        }
    }

    // Complete metrics tracking successfully
    let total_duration = overall_start.elapsed();
    metrics_manager
        .collector()
        .record_metric("extract.duration_ms", total_duration.as_millis() as f64)?;
    metrics_manager.complete_command(&tracking_id).await?;

    Ok(())
}

/// Resolve WASM module path with priority order:
/// 1. CLI flag --wasm-path
/// 2. Environment variable RIPTIDE_WASM_PATH
/// 3. Default fallback path
///
/// Note: Currently unused but kept for future WASM path resolution functionality
#[allow(dead_code)]
fn resolve_wasm_path(args: &ExtractArgs) -> String {
    // Priority 1: CLI flag (already checked via clap's env attribute)
    if let Some(ref path) = args.wasm_path {
        output::print_info(&format!("Using WASM path from CLI flag: {}", path));
        return path.clone();
    }

    // Priority 2: Environment variable (fallback if CLI not provided)
    if let Ok(env_path) = std::env::var("RIPTIDE_WASM_PATH") {
        output::print_info(&format!("Using WASM path from environment: {}", env_path));
        return env_path;
    }

    // Priority 3: Default production path
    let default_path = "/opt/riptide/wasm/riptide_extractor_wasm.wasm";

    // Priority 4: Development fallback if production path doesn't exist
    if !std::path::Path::new(default_path).exists() {
        let manifest_dir = std::env!("CARGO_MANIFEST_DIR");
        let dev_path = format!(
            "{}/../../target/wasm32-wasip2/release/riptide-extractor-wasm.component.wasm",
            manifest_dir
        );
        output::print_info(&format!("Using development WASM path: {}", dev_path));
        dev_path
    } else {
        output::print_info(&format!("Using default WASM path: {}", default_path));
        default_path.to_string()
    }
}

/// Execute direct extraction from provided HTML content
async fn execute_direct_extraction(
    html: String,
    source: String,
    args: ExtractArgs,
    output_format: &str,
    mut engine: Engine,
) -> Result<()> {
    // Check if WASM should be skipped
    if args.no_wasm && (engine == Engine::Wasm || engine == Engine::Auto) {
        output::print_warning("WASM extraction disabled via --no-wasm flag");
        engine = Engine::Raw;
    }

    output::print_info("Processing provided HTML content...");

    // Auto-detect engine if set to auto
    if engine == Engine::Auto {
        engine = riptide_reliability::engine_selection::decide_engine(&html, &source);
        output::print_info(&format!("Auto-detected engine: {}", engine.name()));
    }

    // Handle different engines
    match engine {
        Engine::Raw => {
            output::print_info("Using Raw engine - returning HTML content without extraction");
            let extract_result = ExtractResponse {
                content: html.clone(),
                confidence: Some(1.0),
                method_used: Some("raw".to_string()),
                extraction_time_ms: Some(0),
                metadata: Some(serde_json::json!({
                    "engine": "raw",
                    "source": source,
                    "content_length": html.len(),
                })),
            };

            return output_extraction_result(extract_result, &args, output_format, &source);
        }
        Engine::Headless => {
            output::print_warning(
                "Headless engine not supported for direct HTML input - falling back to WASM",
            );
        }
        Engine::Wasm | Engine::Auto => {
            // Continue with WASM extraction below
        }
    }

    // Perform WASM extraction
    #[cfg(feature = "wasm-extractor")]
    {
        output::print_info("Performing local WASM extraction...");
        let extraction_start = Instant::now();

        // Resolve WASM path
        let wasm_path = resolve_wasm_path(&args);

        // Verify WASM file exists
        if !std::path::Path::new(&wasm_path).exists() {
            output::print_warning(&format!("WASM file not found at: {}", wasm_path));
            anyhow::bail!(
                "WASM module not found at '{}'. Please:\n  \
                 1. Build the WASM module: cargo build --release --target wasm32-wasip2\n  \
                 2. Specify path with: --wasm-path <path>\n  \
                 3. Set environment: RIPTIDE_WASM_PATH=<path>",
                wasm_path
            );
        }

        // Create extractor
        output::print_info(&format!("Loading WASM module from: {}", wasm_path));
        let timeout_duration = std::time::Duration::from_millis(args.init_timeout_ms);
        let extractor_result =
            tokio::time::timeout(timeout_duration, WasmExtractor::new(&wasm_path)).await;

        let extractor = match extractor_result {
            Ok(Ok(ext)) => {
                output::print_info("✓ WASM module loaded successfully");
                ext
            }
            Ok(Err(e)) => {
                anyhow::bail!("Failed to initialize WASM module: {}", e);
            }
            Err(_) => {
                anyhow::bail!(
                    "WASM module initialization timed out after {}ms",
                    args.init_timeout_ms
                );
            }
        };

        let mode = if args.metadata {
            "metadata"
        } else if args.method == "full" {
            "full"
        } else {
            "article"
        };

        let result = extractor.extract(html.as_bytes(), &source, mode)?;
        let extraction_time = extraction_start.elapsed();

        // Calculate metrics
        let word_count = result.text.split_whitespace().count();
        let confidence = result.quality_score.unwrap_or(0) as f64 / 100.0;

        // Create response
        let extract_result = ExtractResponse {
            content: result.text.clone(),
            confidence: Some(confidence),
            method_used: Some(format!("local-{}", engine.name())),
            extraction_time_ms: Some(extraction_time.as_millis() as u64),
            metadata: Some(serde_json::json!({
                "engine": engine.name(),
                "source": source,
                "title": result.title,
                "byline": result.byline,
                "published": result.published_iso,
                "site_name": result.site_name,
                "description": result.description,
                "word_count": word_count,
                "reading_time": result.reading_time,
                "quality_score": result.quality_score,
                "links_count": result.links.len(),
                "media_count": result.media.len(),
                "language": result.language,
                "categories": result.categories,
            })),
        };

        output_extraction_result(extract_result, &args, output_format, &source)
    }

    #[cfg(not(feature = "wasm-extractor"))]
    {
        anyhow::bail!("WASM extraction not available. Rebuild with --features wasm-extractor")
    }
}

/// Execute local WASM extraction without API server
async fn execute_local_extraction(
    args: ExtractArgs,
    output_format: &str,
    mut engine: Engine,
) -> Result<()> {
    use riptide_stealth::{StealthController, StealthPreset};
    use std::time::Instant;

    // Validate URL is present
    let url = args
        .url
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("URL is required for local extraction"))?;

    // Check if WASM should be skipped
    if args.no_wasm && (engine == Engine::Wasm || engine == Engine::Auto) {
        output::print_warning("WASM extraction disabled via --no-wasm flag");
        engine = Engine::Raw;
    }

    // Fetch HTML content with optional stealth
    output::print_info("Fetching HTML content...");

    // Configure stealth if requested
    let mut stealth_controller = if let Some(level) = &args.stealth_level {
        let preset = match level.as_str() {
            "none" => StealthPreset::None,
            "low" => StealthPreset::Low,
            "medium" => StealthPreset::Medium,
            "high" => StealthPreset::High,
            _ => StealthPreset::Medium,
        };
        Some(StealthController::from_preset(preset))
    } else {
        None
    };

    // Build HTTP client with stealth options
    let mut client_builder = reqwest::Client::builder().timeout(std::time::Duration::from_secs(30));

    // Apply user agent
    if let Some(custom_ua) = &args.user_agent {
        client_builder = client_builder.user_agent(custom_ua);
    } else if let Some(ref mut controller) = stealth_controller {
        let ua = controller.next_user_agent();
        client_builder = client_builder.user_agent(ua);

        // Add stealth headers
        if args.fingerprint_evasion {
            let headers = controller.generate_headers();
            let mut header_map = reqwest::header::HeaderMap::new();
            for (key, value) in headers {
                if let (Ok(header_name), Ok(header_value)) = (
                    reqwest::header::HeaderName::from_bytes(key.as_bytes()),
                    reqwest::header::HeaderValue::from_str(&value),
                ) {
                    header_map.insert(header_name, header_value);
                }
            }
            client_builder = client_builder.default_headers(header_map);
        }
    } else {
        client_builder = client_builder.user_agent("RipTide/1.0");
    }

    // Apply proxy if specified
    if let Some(proxy_url) = &args.proxy {
        let proxy = reqwest::Proxy::all(proxy_url)?;
        client_builder = client_builder.proxy(proxy);
    }

    let client = client_builder.build()?;

    // Apply timing randomization if requested
    if args.randomize_timing {
        if let Some(ref mut controller) = stealth_controller {
            let delay = controller.calculate_delay();
            output::print_info(&format!(
                "Applying timing delay: {:.2}s",
                delay.as_secs_f64()
            ));
            tokio::time::sleep(delay).await;
        }
    }

    let start = Instant::now();
    let response = client.get(url).send().await?;
    let html = response.text().await?;
    let fetch_time = start.elapsed();

    // Auto-detect engine if set to auto
    if engine == Engine::Auto {
        engine = riptide_reliability::engine_selection::decide_engine(&html, url);
        output::print_info(&format!("Auto-detected engine: {}", engine.name()));
    }

    // Handle different engines
    match engine {
        Engine::Raw => {
            output::print_info("Using Raw engine - basic HTTP fetch without extraction");
            // Return raw HTML content
            let extract_result = ExtractResponse {
                content: html.clone(),
                confidence: Some(1.0),
                method_used: Some("raw".to_string()),
                extraction_time_ms: Some(fetch_time.as_millis() as u64),
                metadata: Some(serde_json::json!({
                    "engine": "raw",
                    "content_length": html.len(),
                    "fetch_time_ms": fetch_time.as_millis(),
                })),
            };

            return output_extraction_result(extract_result, &args, output_format, url);
        }
        Engine::Headless => {
            output::print_info(
                "Using Headless engine - launching browser for JavaScript-heavy content",
            );
            return execute_headless_extraction(&args, url, output_format).await;
        }
        Engine::Wasm | Engine::Auto => {
            // Continue with WASM extraction below
        }
    }

    // Perform local extraction
    #[cfg(not(feature = "wasm-extractor"))]
    {
        anyhow::bail!("Local WASM extraction not available. Rebuild with --features wasm-extractor or use API mode");
    }

    #[cfg(feature = "wasm-extractor")]
    {
        output::print_info("Performing local WASM extraction...");
        let extraction_start = Instant::now();

        // Resolve WASM path with priority order
        let wasm_path = resolve_wasm_path(&args);

        // Verify WASM file exists before attempting to load
        if !std::path::Path::new(&wasm_path).exists() {
            output::print_warning(&format!("WASM file not found at: {}", wasm_path));
            anyhow::bail!(
                "WASM module not found at '{}'. Please:\n  \
             1. Build the WASM module: cargo build --release --target wasm32-wasip2\n  \
             2. Specify path with: --wasm-path <path>\n  \
             3. Set environment: RIPTIDE_WASM_PATH=<path>\n  \
             4. Or use API server mode without --local flag",
                wasm_path
            );
        }

        // Create extractor with timeout handling
        output::print_info(&format!("Loading WASM module from: {}", wasm_path));
        output::print_info(&format!(
            "Initialization timeout: {}ms",
            args.init_timeout_ms
        ));

        let timeout_duration = std::time::Duration::from_millis(args.init_timeout_ms);
        let extractor_result =
            tokio::time::timeout(timeout_duration, WasmExtractor::new(&wasm_path)).await;

        let extractor = match extractor_result {
            Ok(Ok(ext)) => {
                output::print_info("✓ WASM module loaded successfully");
                ext
            }
            Ok(Err(e)) => {
                output::print_warning(&format!("WASM initialization failed: {}", e));
                anyhow::bail!(
                    "Failed to initialize WASM module: {}\n  \
                 Tip: Verify the WASM file is valid and compatible with the current runtime",
                    e
                );
            }
            Err(_) => {
                output::print_warning(&format!(
                    "WASM initialization timed out after {}ms",
                    args.init_timeout_ms
                ));
                anyhow::bail!(
                    "WASM module initialization timed out after {}ms.\n  \
                 Possible causes:\n  \
                 - WASM file is too large or complex\n  \
                 - System resource constraints\n  \
                 - File I/O issues\n  \
                 Try: Increase timeout with --init-timeout-ms <milliseconds>",
                    args.init_timeout_ms
                );
            }
        };

        let mode = if args.metadata {
            "metadata"
        } else if args.method == "full" {
            "full"
        } else {
            "article"
        };

        let result = extractor.extract(html.as_bytes(), url, mode)?;
        let extraction_time = extraction_start.elapsed();

        // Calculate word count and confidence
        let word_count = result.text.split_whitespace().count();
        let confidence = result.quality_score.unwrap_or(0) as f64 / 100.0;

        // Create response structure
        let extract_result = ExtractResponse {
            content: result.text.clone(),
            confidence: Some(confidence),
            method_used: Some(format!("local-{}", engine.name())),
            extraction_time_ms: Some(extraction_time.as_millis() as u64),
            metadata: Some(serde_json::json!({
                "engine": engine.name(),
                "title": result.title,
                "byline": result.byline,
                "published": result.published_iso,
                "site_name": result.site_name,
                "description": result.description,
                "word_count": word_count,
                "reading_time": result.reading_time,
                "quality_score": result.quality_score,
                "links_count": result.links.len(),
                "media_count": result.media.len(),
                "language": result.language,
                "categories": result.categories,
                "fetch_time_ms": fetch_time.as_millis(),
            })),
        };

        output_extraction_result(extract_result, &args, output_format, url)
    }
}

/// Execute headless browser extraction for JavaScript-heavy sites
async fn execute_headless_extraction(
    args: &ExtractArgs,
    url: &str,
    _output_format: &str,
) -> Result<()> {
    use riptide_stealth::StealthPreset;
    use std::time::Instant;

    output::print_info("Initializing headless browser...");
    let extraction_start = Instant::now();

    // Determine stealth preset from CLI args
    let stealth_preset = if let Some(ref level) = args.stealth_level {
        match level.as_str() {
            "none" => StealthPreset::None,
            "low" => StealthPreset::Low,
            "medium" => StealthPreset::Medium,
            "high" => StealthPreset::High,
            _ => StealthPreset::Medium,
        }
    } else {
        StealthPreset::None
    };

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
        enable_stealth: args.stealth_level.is_some() && stealth_preset != StealthPreset::None,
        page_timeout: std::time::Duration::from_millis(args.headless_timeout.unwrap_or(30000)),
        enable_monitoring: false,
        hybrid_mode: false,
    };

    // Initialize launcher
    let launcher = HeadlessLauncher::with_config(launcher_config).await?;

    // Launch browser page with stealth if configured
    output::print_info(&format!("Navigating to: {}", url));
    let session = launcher
        .launch_page(url, Some(stealth_preset))
        .await
        .map_err(|e| anyhow::anyhow!("Failed to launch browser page: {}", e))?;

    let page = session.page();

    // Wait for page to fully load with timeout
    let wait_timeout = std::time::Duration::from_millis(5000);
    tokio::time::timeout(wait_timeout, page.wait_for_navigation())
        .await
        .ok(); // Ignore timeout, proceed with current state

    // Additional wait for dynamic content if behavior simulation is enabled
    if args.simulate_behavior {
        output::print_info("Simulating user behavior...");

        // Scroll down the page to trigger lazy-loaded content
        let scroll_script = r#"
            window.scrollTo(0, document.body.scrollHeight / 2);
            await new Promise(resolve => setTimeout(resolve, 500));
            window.scrollTo(0, document.body.scrollHeight);
        "#;

        if let Err(e) = page.evaluate(scroll_script).await {
            output::print_warning(&format!("Scroll simulation failed: {}", e));
        }

        // Wait for content to settle
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    }

    // Extract HTML content
    output::print_info("Extracting rendered HTML...");
    let _html = tokio::time::timeout(std::time::Duration::from_millis(5000), page.content())
        .await
        .map_err(|_| anyhow::anyhow!("HTML content extraction timed out"))?
        .map_err(|e| anyhow::anyhow!("Failed to extract HTML content: {}", e))?;

    // Get final URL after redirects
    let _final_url = page
        .url()
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| url.to_string());

    // Shutdown launcher to clean up resources
    launcher.shutdown().await?;

    let _extraction_time = extraction_start.elapsed();

    // Now use WASM extractor to parse the rendered HTML
    #[cfg(not(feature = "wasm-extractor"))]
    {
        anyhow::bail!(
            "Headless extraction requires WASM extractor. Rebuild with --features wasm-extractor"
        );
    }

    #[cfg(feature = "wasm-extractor")]
    {
        output::print_info("Parsing rendered content with WASM extractor...");

        let wasm_path = resolve_wasm_path(args);
        if !std::path::Path::new(&wasm_path).exists() {
            anyhow::bail!(
            "WASM module not found at '{}'. Headless extraction requires WASM for content parsing.",
            wasm_path
        );
        }

        let timeout_duration = std::time::Duration::from_millis(args.init_timeout_ms);
        let extractor_result =
            tokio::time::timeout(timeout_duration, WasmExtractor::new(&wasm_path)).await;

        let extractor = match extractor_result {
            Ok(Ok(ext)) => ext,
            Ok(Err(e)) => anyhow::bail!("Failed to initialize WASM module: {}", e),
            Err(_) => anyhow::bail!("WASM module initialization timed out"),
        };

        let mode = if args.metadata {
            "metadata"
        } else if args.method == "full" {
            "full"
        } else {
            "article"
        };

        let result = extractor.extract(html.as_bytes(), &final_url, mode)?;

        // Calculate metrics
        let word_count = result.text.split_whitespace().count();
        let confidence = result.quality_score.unwrap_or(0) as f64 / 100.0;

        // Create response
        let extract_result = ExtractResponse {
            content: result.text.clone(),
            confidence: Some(confidence),
            method_used: Some("headless".to_string()),
            extraction_time_ms: Some(extraction_time.as_millis() as u64),
            metadata: Some(serde_json::json!({
                "engine": "headless",
                "title": result.title,
                "byline": result.byline,
                "published": result.published_iso,
                "site_name": result.site_name,
                "description": result.description,
                "word_count": word_count,
                "reading_time": result.reading_time,
                "quality_score": result.quality_score,
                "links_count": result.links.len(),
                "media_count": result.media.len(),
                "language": result.language,
                "categories": result.categories,
                "final_url": final_url,
                "stealth_enabled": args.stealth_level.is_some(),
                "stealth_level": args.stealth_level.as_ref().unwrap_or(&"none".to_string()),
            })),
        };

        output_extraction_result(extract_result, args, output_format, url)
    }
}

/// Output extraction results in the specified format
fn output_extraction_result(
    extract_result: ExtractResponse,
    args: &ExtractArgs,
    output_format: &str,
    source: &str,
) -> Result<()> {
    match output_format {
        "json" => {
            output::print_json(&extract_result);
        }
        "text" => {
            output::print_success(&format!(
                "Content extracted successfully ({})",
                extract_result
                    .method_used
                    .as_ref()
                    .unwrap_or(&"unknown".to_string())
            ));
            println!();

            if args.show_confidence {
                if let Some(confidence) = extract_result.confidence {
                    output::print_key_value("Confidence", &output::format_confidence(confidence));
                }
            }

            if let Some(method) = extract_result.method_used {
                output::print_key_value("Method", &method);
            }

            if let Some(time) = extract_result.extraction_time_ms {
                output::print_key_value("Extraction Time", &format!("{}ms", time));
            }

            if args.metadata {
                if let Some(metadata) = extract_result.metadata {
                    output::print_section("Metadata");
                    println!("{}", serde_json::to_string_pretty(&metadata)?);
                }
            }

            output::print_section("Extracted Content");
            println!("{}", extract_result.content);

            // Save to file if specified
            if let Some(ref file_path) = args.file {
                fs::write(file_path, &extract_result.content)?;
                output::print_success(&format!("Content saved to: {}", file_path));
            }
        }
        "table" => {
            let mut table = output::create_table(vec!["Field", "Value"]);
            table.add_row(vec!["Source", source]);

            if let Some(ref method) = extract_result.method_used {
                table.add_row(vec!["Mode", method]);
            }

            if let Some(confidence) = extract_result.confidence {
                table.add_row(vec!["Confidence", &output::format_confidence(confidence)]);
            }

            if let Some(method) = extract_result.method_used {
                table.add_row(vec!["Method", &method]);
            }

            if let Some(time) = extract_result.extraction_time_ms {
                table.add_row(vec!["Time", &format!("{}ms", time)]);
            }

            table.add_row(vec![
                "Content Length",
                &format!("{} chars", extract_result.content.len()),
            ]);

            println!("{table}");
        }
        _ => {
            output::print_warning(&format!("Unknown output format: {}", output_format));
            output::print_json(&extract_result);
        }
    }

    Ok(())
}
