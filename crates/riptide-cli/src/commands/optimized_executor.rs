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
    performance_monitor::PerformanceMonitor,
    render::RenderArgs,
    ExtractArgs, // Import from mod.rs
};
use riptide_reliability::engine_selection::Engine;

// Import WASM types from riptide-cache (only when wasm-extractor feature is enabled)
#[cfg(feature = "wasm-extractor")]
use riptide_cache::wasm::{WasmAotCache, WasmCache};

/// Unified executor that orchestrates all optimization modules
///
/// NOTE: This needs updating to use riptide-browser::pool::BrowserPool when re-enabled
#[allow(dead_code)]
pub struct OptimizedExecutor {
    // TODO(phase9): Replace with Arc<riptide_browser::pool::BrowserPool>
    browser_pool: Arc<()>, // Placeholder - BrowserPoolManager removed

    #[cfg(feature = "wasm-extractor")]
    wasm_aot: Arc<WasmAotCache>,
    #[cfg(not(feature = "wasm-extractor"))]
    wasm_aot: Arc<()>, // Placeholder for non-wasm builds

    timeout_mgr: Arc<AdaptiveTimeoutManager>,
    engine_cache: Arc<EngineSelectionCache>,

    #[cfg(feature = "wasm-extractor")]
    wasm_cache: Arc<WasmCache>,
    #[cfg(not(feature = "wasm-extractor"))]
    wasm_cache: Arc<()>, // Placeholder for non-wasm builds

    perf_monitor: Arc<PerformanceMonitor>,
}

impl OptimizedExecutor {
    /// Create a new optimized executor with all modules initialized
    pub async fn new() -> Result<Self> {
        tracing::info!("Initializing optimized executor with all optimization modules");

        // Initialize all global managers using their respective global accessors
        // Note: browser_pool is currently a placeholder - will be replaced with BrowserPool in Phase 9
        let browser_pool = Arc::new(()); // Placeholder until BrowserPool integration
        let timeout_mgr = super::adaptive_timeout::get_global_timeout_manager().await?;
        let engine_cache = EngineSelectionCache::get_global();
        let perf_monitor = PerformanceMonitor::get_global();

        tracing::info!("✓ All optimization modules initialized");

        Ok(Self {
            browser_pool,

            #[cfg(feature = "wasm-extractor")]
            wasm_aot: riptide_cache::wasm::get_global_aot_cache(),
            #[cfg(not(feature = "wasm-extractor"))]
            wasm_aot: Arc::new(()), // Placeholder for non-wasm builds

            timeout_mgr,
            engine_cache,

            #[cfg(feature = "wasm-extractor")]
            wasm_cache: WasmCache::get_global(),
            #[cfg(not(feature = "wasm-extractor"))]
            wasm_cache: Arc::new(()), // Placeholder for non-wasm builds

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

        tracing::info!(
            "Starting optimized extraction for {} (domain: {})",
            url,
            domain
        );

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

            let selected =
                riptide_reliability::engine_selection::decide_engine(&html_for_decision, url);
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
        if result.success {
            self.timeout_mgr.record_success(&domain, duration).await;
        } else {
            self.timeout_mgr.record_timeout(&domain).await;
        }

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
        #[cfg(feature = "wasm-extractor")]
        use riptide_extraction::wasm_extraction::WasmExtractor;

        tracing::info!("Executing WASM extraction with AOT cache");

        let wasm_path = Self::resolve_wasm_path(args);

        // Check WASM cache first for compiled module
        #[cfg(feature = "wasm-extractor")]
        let extractor = if let Some(cached_module) = self.wasm_cache.get(&wasm_path).await {
            tracing::info!("✓ Using cached WASM module");
            cached_module
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

        #[cfg(not(feature = "wasm-extractor"))]
        return Err(anyhow::anyhow!("WASM extractor feature not enabled"));

        #[cfg(feature = "wasm-extractor")]
        {
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
    }

    /// Execute headless extraction with browser pool optimization
    async fn execute_headless_optimized(
        &self,
        _args: &ExtractArgs,
        _url: &str,
    ) -> Result<ExtractResponse> {
        // TODO(phase9): Implement when BrowserPool is integrated
        // For now, return an error indicating this feature needs browser pool
        Err(anyhow::anyhow!(
            "Headless extraction requires browser pool integration (Phase 9). \
             Use --engine wasm or --engine raw as alternatives."
        ))
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
    pub async fn execute_render(&self, _args: RenderArgs) -> Result<RenderResponse> {
        // TODO(phase9): Implement when BrowserPool is integrated
        Err(anyhow::anyhow!(
            "Render functionality requires browser pool integration (Phase 9). \
             Use the 'render' command directly as an alternative."
        ))
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

        // Browser pool shutdown - TODO(phase9): Implement when BrowserPool is integrated
        // self.browser_pool.shutdown().await?;
        tracing::info!("✓ Browser pool shutdown (placeholder)");

        // Save WASM AOT cache
        #[cfg(feature = "wasm-extractor")]
        {
            if let Err(e) = self.wasm_aot.save_cache().await {
                tracing::warn!("Failed to save WASM AOT cache: {}", e);
            }
            tracing::info!("✓ WASM AOT cache saved");
        }

        // Save timeout profiles
        if let Err(e) = self.timeout_mgr.save_profiles().await {
            tracing::warn!("Failed to save timeout profiles: {}", e);
        }
        tracing::info!("✓ Timeout profiles saved");

        // Engine cache doesn't have a save method - it's in-memory only
        tracing::info!("✓ Engine cache (in-memory)");

        // Performance monitor doesn't have a save method - it's in-memory only
        tracing::info!("✓ Performance metrics (in-memory)");

        tracing::info!("Optimized executor shutdown complete");

        Ok(())
    }

    /// Get performance statistics
    pub async fn get_stats(&self) -> OptimizationStats {
        OptimizationStats {
            browser_pool: serde_json::json!({
                "status": "not_implemented",
                "note": "Browser pool integration pending (Phase 9)"
            }),

            #[cfg(feature = "wasm-extractor")]
            wasm_aot_cache: self
                .wasm_aot
                .stats()
                .await
                .map(|s| serde_json::to_value(s).unwrap_or(serde_json::json!({})))
                .unwrap_or(serde_json::json!({})),
            #[cfg(not(feature = "wasm-extractor"))]
            wasm_aot_cache: serde_json::json!({"status": "disabled"}),

            engine_cache: {
                let stats = self.engine_cache.stats().await;
                serde_json::json!({
                    "entries": stats.entries,
                    "total_hits": stats.total_hits,
                    "avg_success_rate": stats.avg_success_rate,
                    "max_capacity": stats.max_capacity,
                })
            },

            performance: {
                let stats = self.perf_monitor.get_stats().await;
                serde_json::json!({
                    "total_operations": stats.total_operations,
                    "successful_operations": stats.successful_operations,
                    "failed_operations": stats.failed_operations,
                    "success_rate": stats.success_rate,
                    "avg_duration_ms": stats.avg_duration_ms,
                    "avg_content_size_bytes": stats.avg_content_size_bytes,
                    "engine_usage": stats.engine_usage,
                })
            },
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
