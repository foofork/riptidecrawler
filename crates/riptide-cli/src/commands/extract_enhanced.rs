#![allow(dead_code)]

/// Enhanced Extract Command with Caching and Performance Monitoring
///
/// This module wraps the original extract command with:
/// - Domain-based engine selection caching
/// - WASM module caching
/// - Performance monitoring and metrics
/// - Enhanced error diagnostics
///
/// **Note**: This is infrastructure code for Phase 5+ optimization features.
/// Currently unused but designed for future API integration.
use crate::client::RipTideClient;
use crate::commands::engine_cache::EngineSelectionCache;
use crate::commands::performance_monitor::{ExtractionMetrics, PerformanceMonitor, StageTimer};
#[cfg(feature = "wasm-extractor")]
use crate::commands::wasm_cache::WasmModuleCache;
use crate::commands::ExtractArgs;
use crate::output;
use anyhow::Result;
use std::time::Duration;

/// Enhanced extract executor with caching and monitoring
/// Infrastructure: Designed for Phase 5+ optimization features
#[allow(dead_code)]
pub struct EnhancedExtractExecutor {
    engine_cache: EngineSelectionCache,
    #[cfg(feature = "wasm-extractor")]
    wasm_cache: WasmModuleCache,
    perf_monitor: PerformanceMonitor,
}

impl EnhancedExtractExecutor {
    /// Create a new enhanced extract executor
    pub fn new() -> Self {
        Self {
            engine_cache: EngineSelectionCache::new(Duration::from_secs(3600), 1000),
            #[cfg(feature = "wasm-extractor")]
            wasm_cache: WasmModuleCache::new(Duration::from_secs(10)),
            perf_monitor: PerformanceMonitor::new(1000),
        }
    }

    /// Execute extraction with enhancements
    pub async fn execute(
        &self,
        client: RipTideClient,
        args: ExtractArgs,
        output_format: &str,
    ) -> Result<()> {
        let mut timer = StageTimer::new();
        let operation_id = uuid::Uuid::new_v4().to_string();

        // Start overall timing
        timer.start_stage("total");

        // Get URL for caching (if available)
        let url = args.url.clone();
        let domain = url
            .as_ref()
            .map(|u| EngineSelectionCache::extract_domain(u));

        // Check cache for engine selection
        let cached_engine = if let Some(domain) = &domain {
            self.engine_cache.get(domain).await
        } else {
            None
        };

        if let Some(engine) = cached_engine {
            if let Some(domain_str) = domain.as_ref() {
                output::print_info(&format!(
                    "Using cached engine selection: {} for domain: {}",
                    engine.name(),
                    domain_str
                ));
            }
        }

        // Execute the actual extraction
        let result = self
            .execute_with_monitoring(client, args, output_format, &mut timer, &operation_id)
            .await;

        timer.end_stage();

        // Record metrics
        let total_duration = timer.get_stage("total").unwrap_or(Duration::from_secs(0));

        let metrics = ExtractionMetrics {
            operation_id: operation_id.clone(),
            url: url.clone(),
            engine_used: "enhanced".to_string(), // Will be updated by actual execution
            total_duration_ms: total_duration.as_millis() as u64,
            fetch_duration_ms: timer.get_stage("fetch").map(|d| d.as_millis() as u64),
            extraction_duration_ms: timer.get_stage("extraction").map(|d| d.as_millis() as u64),
            wasm_init_duration_ms: timer.get_stage("wasm_init").map(|d| d.as_millis() as u64),
            browser_launch_duration_ms: timer
                .get_stage("browser_launch")
                .map(|d| d.as_millis() as u64),
            content_size_bytes: 0, // Will be filled by actual result
            confidence_score: None,
            success: result.is_ok(),
            error_message: result.as_ref().err().map(|e| e.to_string()),
            timestamp: chrono::Utc::now(),
        };

        self.perf_monitor.record(metrics).await?;

        // Update cache feedback
        if let Some(domain) = &domain {
            self.engine_cache
                .update_feedback(domain, result.is_ok())
                .await?;
        }

        result
    }

    /// Execute with detailed monitoring
    async fn execute_with_monitoring(
        &self,
        client: RipTideClient,
        args: ExtractArgs,
        output_format: &str,
        timer: &mut StageTimer,
        _operation_id: &str,
    ) -> Result<()> {
        // Import the original execute function
        use crate::commands::extract;

        // Start extraction stage
        timer.start_stage("extraction");
        let result = extract::execute(client, args, output_format).await;
        timer.end_stage();

        result
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> String {
        let engine_stats = self.engine_cache.stats().await;

        #[cfg(feature = "wasm-extractor")]
        let wasm_status = {
            let wasm_stats = self.wasm_cache.stats().await;
            if let Some(wasm) = wasm_stats {
                format!(
                    "loaded ({}s old, {} uses)",
                    wasm.age_seconds, wasm.use_count
                )
            } else {
                "not loaded".to_string()
            }
        };

        #[cfg(not(feature = "wasm-extractor"))]
        let wasm_status = "WASM cache disabled (feature not enabled)".to_string();

        let perf_stats = self.perf_monitor.get_stats().await;

        format!(
            "Engine Cache: {} entries, {} hits, {:.2}% success rate\n\
             WASM Cache: {}\n\
             Performance: {} operations, {:.2}% success rate, avg {:.0}ms",
            engine_stats.entries,
            engine_stats.total_hits,
            engine_stats.avg_success_rate * 100.0,
            wasm_status,
            perf_stats.total_operations,
            perf_stats.success_rate * 100.0,
            perf_stats.avg_duration_ms
        )
    }

    /// Get performance statistics
    pub async fn get_performance_stats(&self) -> Result<String> {
        let stats = self.perf_monitor.get_stats().await;
        serde_json::to_string_pretty(&stats)
            .map_err(|e| anyhow::anyhow!("Serialization error: {}", e))
    }
}

impl Default for EnhancedExtractExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_enhanced_executor_creation() {
        let executor = EnhancedExtractExecutor::new();
        assert!(executor.engine_cache.stats().await.entries == 0);
    }
}
