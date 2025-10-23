//! Unified executor that orchestrates all Phase 3-4 optimizations
//!
//! **NOTE**: This module is currently disabled (see mod.rs). When re-enabled, it will need
//! to be updated to use `riptide-browser::pool::BrowserPool` directly instead of the
//! removed `browser_pool_manager`.
//!
//! This module provides a high-level API that integrates:
//! - Browser pool management (use riptide-browser::pool::BrowserPool)
//! - WASM AOT cache for pre-compiled modules
//! - Adaptive timeout manager for intelligent timeout selection
//! - Engine cache for decision caching
//! - WASM cache for compiled module caching
//! - Performance monitoring for real-time metrics

use anyhow::Result;
use std::sync::Arc;

use super::{
    adaptive_timeout::AdaptiveTimeoutManager,
    // Note: browser_pool_manager removed - use riptide-browser::pool::BrowserPool directly
    engine_cache::EngineSelectionCache,
    extract::ExtractArgs,
    performance_monitor::PerformanceMonitor,
    render::RenderArgs,
    wasm_aot_cache::WasmAotCache,
    wasm_cache::WasmCache,
};
use riptide_reliability::engine_selection::Engine;

/// Unified executor that orchestrates all optimization modules
///
/// NOTE: This needs updating to use riptide-browser::pool::BrowserPool when re-enabled
#[allow(dead_code)]
pub struct OptimizedExecutor {
    // TODO(phase9): Replace with Arc<riptide_browser::pool::BrowserPool>
    browser_pool: Arc<()>, // Placeholder - BrowserPoolManager removed
    wasm_aot: Arc<WasmAotCache>,
    timeout_mgr: Arc<AdaptiveTimeoutManager>,
    engine_cache: Arc<EngineSelectionCache>,
    wasm_cache: Arc<WasmCache>,
    perf_monitor: Arc<PerformanceMonitor>,
}

impl OptimizedExecutor {
    /// Create a new optimized executor with all modules initialized
    pub async fn new() -> Result<Self> {
        tracing::info!("Initializing optimized executor with all optimization modules");

        // Initialize all global managers
        let browser_pool = BrowserPoolManager::initialize_global().await?;
        let wasm_aot = WasmAotCache::initialize_global().await?;
        let timeout_mgr = AdaptiveTimeoutManager::initialize_global().await?;
        let engine_cache = EngineSelectionCache::get_global();
        let wasm_cache = WasmCache::get_global();
        let perf_monitor = PerformanceMonitor::get_global();

        tracing::info!("✓ All optimization modules initialized");

        Ok(Self {
            browser_pool,
            wasm_aot,
            timeout_mgr,
            engine_cache,
            wasm_cache,
            perf_monitor,
        })
    }

    /// Execute optimized extraction with all optimizations applied
    pub async fn execute_extract(
        &self,
        mut args: ExtractArgs,
        html: Option<String>,
        url: &str,
    ) -> Result<ExtractResponse> {
        let start = std::time::Instant::now();

        // Validate URL format
        if url.is_empty() {
            return Err(anyhow::anyhow!("URL cannot be empty"));
        }

        let domain = Self::extract_domain(url);

        if domain == "unknown" {
            tracing::warn!("Could not parse domain from URL: {}", url);
        }

        tracing::info!("Starting optimized extraction for {} (domain: {})", url, domain);

        // 1. Check engine cache for previous decisions
        let engine = if let Some(cached_engine) = self.engine_cache.get(&domain).await {
            tracing::info!("✓ Using cached engine decision: {:?}", cached_engine);
            cached_engine
        } else {
            // Fetch HTML if not provided and make gate decision
            let html_for_decision = if let Some(ref h) = html {
                h.clone()
            } else {
                match self.fetch_html(url, &args).await {
                    Ok(html_content) => html_content,
                    Err(e) => {
                        tracing::error!("Failed to fetch HTML for {}: {}", url, e);
                        return Err(anyhow::anyhow!(
                            "Failed to fetch content from URL: {}. Please check the URL is accessible and try again.",
                            e
                        ));
                    }
                }
            };

            let selected = riptide_reliability::engine_selection::decide_engine(&html_for_decision, url);
            if let Err(e) = self.engine_cache.store(&domain, selected, 0.85).await {
                tracing::warn!("Failed to cache engine decision: {}", e);
                // Non-fatal, continue with selected engine
            }
            tracing::info!("✓ Selected and cached engine: {:?}", selected);
            selected
        };

        // Override args engine with optimized selection
        args.engine = engine.name().to_string();

        // 2. Apply adaptive timeout
        let timeout = self.timeout_mgr.get_timeout(&domain).await;
        args.init_timeout_ms = timeout.as_millis() as u64;
        tracing::info!("✓ Applied adaptive timeout: {}ms", args.init_timeout_ms);

        // 3. Route to appropriate engine with optimizations
        let result = match engine {
            Engine::Wasm => {
                tracing::debug!("Executing WASM extraction for {}", url);
                match self.execute_wasm_optimized(&args, html, url).await {
                    Ok(r) => r,
                    Err(e) => {
                        tracing::error!("WASM extraction failed for {}: {}", url, e);
                        return Err(anyhow::anyhow!(
                            "WASM extraction failed: {}. Try using --engine raw as fallback.",
                            e
                        ));
                    }
                }
            }
            Engine::Headless => {
                tracing::debug!("Executing headless browser extraction for {}", url);
                match self.execute_headless_optimized(&args, url).await {
                    Ok(r) => r,
                    Err(e) => {
                        tracing::error!("Headless extraction failed for {}: {}", url, e);
                        return Err(anyhow::anyhow!(
                            "Headless browser extraction failed: {}. Check if browser is available.",
                            e
                        ));
                    }
                }
            }
            Engine::Raw => {
                tracing::debug!("Executing raw HTTP fetch for {}", url);
                match self.execute_raw(&args, html, url).await {
                    Ok(r) => r,
                    Err(e) => {
                        tracing::error!("Raw extraction failed for {}: {}", url, e);
                        return Err(anyhow::anyhow!(
                            "HTTP fetch failed: {}. Check network connectivity and URL.",
                            e
                        ));
                    }
                }
            }
            Engine::Auto => {
                // Should not reach here as gate_decision returns concrete engine
                tracing::warn!("Auto engine reached execution - falling back to WASM");
                self.execute_wasm_optimized(&args, html, url).await?
            }
        };

        let duration = start.elapsed();

        // 4. Update adaptive timeout profile based on actual performance
        self.timeout_mgr
            .record_operation(&domain, duration, result.success)
            .await?;

        // 5. Record performance metrics
        self.perf_monitor
            .record_extraction(url, engine.name(), duration.as_millis() as u64)
            .await?;

        tracing::info!(
            "Extraction completed in {:?} (engine: {:?})",
            duration,
            engine
        );

        Ok(result)
    }

    /// Execute WASM extraction with AOT cache optimization
    async fn execute_wasm_optimized(
        &self,
        args: &ExtractArgs,
        html: Option<String>,
        url: &str,
    ) -> Result<ExtractResponse> {
        use riptide_extraction::wasm_extraction::WasmExtractor;

        tracing::info!("Executing WASM extraction with AOT cache");

        let wasm_path = Self::resolve_wasm_path(args);

        // Check WASM cache first for compiled module
        let extractor = if let Some(cached_module) = self.wasm_cache.get(&wasm_path).await {
            tracing::info!("✓ Using cached WASM module");
            // Note: WasmCache returns the module, but WasmExtractor needs the path
            // We'll use AOT cache which handles pre-compilation
            match self.wasm_aot.get_or_compile(&wasm_path).await {
                Ok(module) => {
                    tracing::info!("✓ Using AOT-compiled WASM module");
                    // Create extractor from pre-compiled module
                    // Note: Current API requires path, future enhancement could accept Module
                    WasmExtractor::new(&wasm_path).await?
                }
                Err(e) => {
                    tracing::warn!(
                        "AOT cache miss or error: {}, falling back to standard load",
                        e
                    );
                    WasmExtractor::new(&wasm_path).await?
                }
            }
        } else {
            // Try AOT cache
            match self.wasm_aot.get_or_compile(&wasm_path).await {
                Ok(_module) => {
                    tracing::info!("✓ Compiled and cached WASM module in AOT cache");
                    WasmExtractor::new(&wasm_path).await?
                }
                Err(e) => {
                    tracing::warn!("AOT compilation failed: {}, using standard load", e);
                    WasmExtractor::new(&wasm_path).await?
                }
            }
        };

        // Fetch HTML if not provided
        let html_content = if let Some(h) = html {
            h
        } else {
            self.fetch_html(url, args).await?
        };

        // Extract with WASM
        let mode = if args.metadata {
            "metadata"
        } else if args.method == "full" {
            "full"
        } else {
            "article"
        };

        let result = extractor.extract(html_content.as_bytes(), url, mode)?;

        Ok(ExtractResponse {
            content: result.text,
            confidence: result.quality_score.map(|s| s as f64 / 100.0),
            method_used: Some("wasm-optimized".to_string()),
            success: true,
            metadata: Some(serde_json::json!({
                "title": result.title,
                "byline": result.byline,
                "published": result.published_iso,
                "optimizations": ["wasm_cache", "aot_cache", "adaptive_timeout"],
            })),
        })
    }

    /// Execute headless extraction with browser pool optimization
    async fn execute_headless_optimized(
        &self,
        args: &ExtractArgs,
        url: &str,
    ) -> Result<ExtractResponse> {
        use riptide_stealth::StealthPreset;

        tracing::info!("Executing headless extraction with browser pool");

        // Determine stealth preset
        let stealth = if let Some(ref level) = args.stealth_level {
            match level.as_str() {
                "low" => StealthPreset::Low,
                "medium" => StealthPreset::Medium,
                "high" => StealthPreset::High,
                _ => StealthPreset::None,
            }
        } else {
            StealthPreset::None
        };

        // Checkout browser from pool
        let browser = self.browser_pool.checkout().await?;
        tracing::info!("✓ Checked out browser from pool");

        // Launch page with browser
        let session = browser
            .launch_page(url, Some(stealth))
            .await
            .map_err(|e| anyhow::anyhow!("Failed to launch page: {}", e))?;

        // Wait for page load
        let page = session.page();
        let _ = tokio::time::timeout(
            std::time::Duration::from_millis(args.headless_timeout.unwrap_or(30000)),
            page.wait_for_navigation(),
        )
        .await;

        // Extract HTML
        let html = page
            .content()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to extract HTML: {}", e))?;

        // Return browser to pool
        self.browser_pool.checkin(browser).await;
        tracing::info!("✓ Returned browser to pool");

        // Now extract with WASM from rendered HTML
        let wasm_result = self.execute_wasm_optimized(args, Some(html), url).await?;

        Ok(ExtractResponse {
            method_used: Some("headless-optimized".to_string()),
            metadata: Some(serde_json::json!({
                "optimizations": ["browser_pool", "wasm_cache", "adaptive_timeout"],
                "stealth_level": stealth.to_string(),
            })),
            ..wasm_result
        })
    }

    /// Execute raw HTTP fetch without extraction
    async fn execute_raw(
        &self,
        args: &ExtractArgs,
        html: Option<String>,
        url: &str,
    ) -> Result<ExtractResponse> {
        let html_content = if let Some(h) = html {
            h
        } else {
            self.fetch_html(url, args).await?
        };

        Ok(ExtractResponse {
            content: html_content,
            confidence: Some(1.0),
            method_used: Some("raw".to_string()),
            success: true,
            metadata: Some(serde_json::json!({
                "engine": "raw",
                "optimizations": ["adaptive_timeout"],
            })),
        })
    }

    /// Execute optimized render with all optimizations
    pub async fn execute_render(&self, args: RenderArgs) -> Result<RenderResponse> {
        let start = std::time::Instant::now();
        let domain = Self::extract_domain(&args.url);

        tracing::info!("Starting optimized render for {}", args.url);

        // Apply adaptive timeout
        let timeout = self.timeout_mgr.get_timeout(&domain).await;
        tracing::info!("✓ Applied adaptive timeout: {:?}", timeout);

        // Use browser pool for rendering
        let browser = self.browser_pool.checkout().await?;
        tracing::info!("✓ Checked out browser from pool");

        // Parse stealth level
        let stealth = match args.stealth.as_str() {
            "low" => riptide_stealth::StealthPreset::Low,
            "med" | "medium" => riptide_stealth::StealthPreset::Medium,
            "high" => riptide_stealth::StealthPreset::High,
            _ => riptide_stealth::StealthPreset::None,
        };

        // Launch page
        let session = browser
            .launch_page(&args.url, Some(stealth))
            .await
            .map_err(|e| anyhow::anyhow!("Failed to launch page: {}", e))?;

        let page = session.page();

        // Wait based on condition
        self.apply_wait_condition(&args, page).await?;

        // Extract results
        let html = page.content().await.ok();
        let title = page
            .evaluate("document.title")
            .await
            .ok()
            .and_then(|r| r.into_value::<String>().ok());

        // Return browser to pool
        self.browser_pool.checkin(browser).await;
        tracing::info!("✓ Returned browser to pool");

        let duration = start.elapsed();

        // Update timeout profile
        self.timeout_mgr
            .record_operation(&domain, duration, html.is_some())
            .await?;

        // Record metrics
        self.perf_monitor
            .record_render(&args.url, duration.as_millis() as u64)
            .await?;

        Ok(RenderResponse {
            html,
            title,
            success: true,
            render_time_ms: duration.as_millis() as u64,
            metadata: serde_json::json!({
                "optimizations": ["browser_pool", "adaptive_timeout"],
                "stealth_level": args.stealth,
            }),
        })
    }

    /// Apply wait condition from render args
    async fn apply_wait_condition(
        &self,
        args: &RenderArgs,
        page: &chromiumoxide::Page,
    ) -> Result<()> {
        use super::render::WaitCondition;

        let condition = WaitCondition::from_str(&args.wait)?;

        match condition {
            WaitCondition::Load => {
                let _ = tokio::time::timeout(
                    std::time::Duration::from_secs(10),
                    page.wait_for_navigation(),
                )
                .await;
            }
            WaitCondition::NetworkIdle => {
                let _ = tokio::time::timeout(
                    std::time::Duration::from_secs(5),
                    page.wait_for_navigation(),
                )
                .await;
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
            WaitCondition::Selector(selector) => {
                let _ = tokio::time::timeout(
                    std::time::Duration::from_secs(10),
                    page.find_element(&selector),
                )
                .await;
            }
            WaitCondition::Timeout(ms) => {
                tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
            }
        }

        Ok(())
    }

    /// Fetch HTML content with optional stealth
    async fn fetch_html(&self, url: &str, args: &ExtractArgs) -> Result<String> {
        use riptide_stealth::{StealthController, StealthPreset};

        let mut client_builder =
            reqwest::Client::builder().timeout(std::time::Duration::from_secs(30));

        // Apply stealth if configured
        if let Some(ref level) = args.stealth_level {
            let preset = match level.as_str() {
                "low" => StealthPreset::Low,
                "medium" => StealthPreset::Medium,
                "high" => StealthPreset::High,
                _ => StealthPreset::None,
            };

            let mut controller = StealthController::from_preset(preset);
            let ua = controller.next_user_agent();
            client_builder = client_builder.user_agent(ua);
        } else {
            client_builder = client_builder.user_agent("RipTide/1.0");
        }

        let client = client_builder.build()?;
        let response = client.get(url).send().await?;
        let html = response.text().await?;

        Ok(html)
    }

    /// Extract domain from URL
    fn extract_domain(url: &str) -> String {
        url::Url::parse(url)
            .ok()
            .and_then(|u| u.host_str().map(|h| h.to_string()))
            .unwrap_or_else(|| "unknown".to_string())
    }

    /// Resolve WASM path with fallbacks
    fn resolve_wasm_path(args: &ExtractArgs) -> String {
        if let Some(ref path) = args.wasm_path {
            return path.clone();
        }

        if let Ok(env_path) = std::env::var("RIPTIDE_WASM_PATH") {
            return env_path;
        }

        let default_path = "/opt/riptide/wasm/riptide_extractor_wasm.wasm";
        if std::path::Path::new(default_path).exists() {
            return default_path.to_string();
        }

        // Development fallback
        let manifest_dir = std::env!("CARGO_MANIFEST_DIR");
        format!(
            "{}/../../target/wasm32-wasip2/release/riptide-extractor-wasm.component.wasm",
            manifest_dir
        )
    }

    /// Graceful shutdown of all optimization modules
    pub async fn shutdown(&self) -> Result<()> {
        tracing::info!("Shutting down optimized executor");

        // Shutdown browser pool
        self.browser_pool.shutdown().await?;
        tracing::info!("✓ Browser pool shutdown");

        // Save WASM AOT cache
        self.wasm_aot.save_cache().await?;
        tracing::info!("✓ WASM AOT cache saved");

        // Save timeout profiles
        self.timeout_mgr.save_profiles().await?;
        tracing::info!("✓ Timeout profiles saved");

        // Save engine cache
        self.engine_cache.save().await?;
        tracing::info!("✓ Engine cache saved");

        // Save performance metrics
        self.perf_monitor.save().await?;
        tracing::info!("✓ Performance metrics saved");

        tracing::info!("Optimized executor shutdown complete");

        Ok(())
    }

    /// Get performance statistics
    pub async fn get_stats(&self) -> OptimizationStats {
        OptimizationStats {
            browser_pool: self.browser_pool.get_stats().await,
            wasm_aot_cache: self.wasm_aot.get_stats().await,
            engine_cache: self.engine_cache.get_stats().await,
            performance: self.perf_monitor.get_stats().await,
        }
    }
}

/// Response from extraction operation
#[derive(Debug, serde::Serialize)]
pub struct ExtractResponse {
    pub content: String,
    pub confidence: Option<f64>,
    pub method_used: Option<String>,
    pub success: bool,
    pub metadata: Option<serde_json::Value>,
}

/// Response from render operation
#[derive(Debug, serde::Serialize)]
pub struct RenderResponse {
    pub html: Option<String>,
    pub title: Option<String>,
    pub success: bool,
    pub render_time_ms: u64,
    pub metadata: serde_json::Value,
}

/// Combined optimization statistics
#[derive(Debug, serde::Serialize)]
pub struct OptimizationStats {
    pub browser_pool: serde_json::Value,
    pub wasm_aot_cache: serde_json::Value,
    pub engine_cache: serde_json::Value,
    pub performance: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_executor_initialization() {
        let executor = OptimizedExecutor::new().await;
        assert!(executor.is_ok(), "Executor should initialize successfully");
    }

    #[test]
    fn test_domain_extraction() {
        let domain = OptimizedExecutor::extract_domain("https://example.com/path");
        assert_eq!(domain, "example.com");

        let domain = OptimizedExecutor::extract_domain("invalid-url");
        assert_eq!(domain, "unknown");
    }
}
