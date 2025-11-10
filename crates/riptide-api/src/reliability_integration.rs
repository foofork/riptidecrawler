//! Reliability integration module for RipTide API
//!
//! This module provides the integration between riptide-core's ReliableExtractor
//! and riptide-extraction's UnifiedExtractor, implementing the trait adapter pattern.

use riptide_reliability::reliability::ReliabilityMetricsRecorder;

#[cfg(feature = "wasm-extractor")]
use {
    anyhow::Result, riptide_types::extractors::WasmExtractor as WasmExtractorTrait,
    riptide_types::ExtractedDoc, std::sync::Arc, std::time::Instant,
};

/// Adapter to make riptide_extraction::UnifiedExtractor compatible with the reliability trait
#[cfg(feature = "wasm-extractor")]
pub struct WasmExtractorAdapter {
    extractor: Arc<riptide_extraction::UnifiedExtractor>,
    #[allow(deprecated)]
    metrics: Option<Arc<crate::metrics::RipTideMetrics>>,
}

#[cfg(feature = "wasm-extractor")]
impl WasmExtractorAdapter {
    /// Create a new WASM extractor adapter without metrics
    /// Public API for reliability system initialization
    #[allow(dead_code)]
    pub fn new(extractor: Arc<riptide_extraction::UnifiedExtractor>) -> Self {
        Self {
            extractor,
            metrics: None,
        }
    }

    #[allow(dead_code)]
    /// Create adapter with custom metrics instance (available for future metrics migration)
    /// Phase D: Deprecated metrics parameter - kept for reliability integration compatibility
    /// Reserved for future metrics migration to BusinessMetrics + TransportMetrics
    #[allow(deprecated)]
    pub fn with_metrics(
        extractor: Arc<riptide_extraction::UnifiedExtractor>,
        metrics: Arc<crate::metrics::RipTideMetrics>,
    ) -> Self {
        Self {
            extractor,
            metrics: Some(metrics),
        }
    }
}

#[cfg(feature = "wasm-extractor")]
impl WasmExtractorTrait for WasmExtractorAdapter {
    fn extract(&self, html: &[u8], url: &str, _mode: &str) -> Result<ExtractedDoc> {
        let start = Instant::now();

        // Convert bytes to string
        let html_str = String::from_utf8_lossy(html);

        // Call the unified extractor asynchronously (UnifiedExtractor::extract is async)
        // We need to block on the async call since the trait requires a sync method
        let extracted_content = tokio::runtime::Handle::current()
            .block_on(self.extractor.extract(&html_str, url))
            .map_err(|e| anyhow::anyhow!("Extraction failed: {}", e))?;

        // Record WASM cold start time if metrics available
        if let Some(ref _metrics) = self.metrics {
            let _cold_start_ms = start.elapsed().as_millis() as f64;
            // TODO: Migrate to BusinessMetrics + TransportMetrics
            // These deprecated methods need to be replaced with the new metrics architecture:
            // - update_wasm_cold_start_time -> Use BusinessMetrics::record_extraction_latency
            // - update_wasm_memory_metrics -> Use TransportMetrics for memory tracking
            // metrics.update_wasm_cold_start_time(cold_start_ms);

            // Note: Memory metrics would require WASM runtime introspection
            // For now, we estimate based on input/output size
            let _estimated_pages = (html.len() + extracted_content.content.len()) / 65536;
            // 64KB per page
            // TODO: Migrate to BusinessMetrics + TransportMetrics
            // metrics.update_wasm_memory_metrics(
            //     estimated_pages,
            //     0, // No grow failures tracked yet
            //     estimated_pages,
            // );
        }

        // Convert ExtractedContent to ExtractedDoc
        // ExtractedContent has: title, content, summary, url, strategy_used, extraction_confidence
        // ExtractedDoc has many more fields
        Ok(ExtractedDoc {
            url: extracted_content.url,
            title: Some(extracted_content.title),
            text: extracted_content.content.clone(),
            byline: None,
            published_iso: None,
            markdown: None,
            links: Vec::new(),
            media: Vec::new(),
            language: None,
            reading_time: None,
            quality_score: Some(crate::utils::safe_conversions::confidence_to_quality_score(
                extracted_content.extraction_confidence,
            )),
            word_count: Some(crate::utils::safe_conversions::word_count_to_u32(
                extracted_content.content.split_whitespace().count(),
            )),
            categories: Vec::new(),
            site_name: None,
            parser_metadata: None, // ExtractedContent doesn't have parser metadata
            description: extracted_content.summary,
            html: None,
        })
    }
}

/// Implement ReliabilityMetricsRecorder for RipTideMetrics
/// Phase D: Deprecated metrics - kept with #[allow(deprecated)] for reliability integration
#[allow(deprecated)]
impl ReliabilityMetricsRecorder for crate::metrics::RipTideMetrics {
    fn record_extraction_fallback(&self, from_mode: &str, to_mode: &str, reason: &str) {
        self.record_extraction_fallback(from_mode, to_mode, reason);
    }
}
