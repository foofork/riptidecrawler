//! Reliability integration module for RipTide API
//!
//! This module provides the integration between riptide-core's ReliableExtractor
//! and riptide-extraction's WasmExtractor, implementing the trait adapter pattern.

use anyhow::Result;
use riptide_extraction::wasm_extraction::WasmExtractor as ConcreteWasmExtractor;
use riptide_reliability::reliability::{
    ReliabilityMetricsRecorder, WasmExtractor as WasmExtractorTrait,
};
use riptide_types::ExtractedDoc;
use std::sync::Arc;
use std::time::Instant;

/// Adapter to make riptide_extraction::WasmExtractor compatible with the reliability trait
pub struct WasmExtractorAdapter {
    extractor: Arc<ConcreteWasmExtractor>,
    metrics: Option<Arc<crate::metrics::RipTideMetrics>>,
}

impl WasmExtractorAdapter {
    /// Create a new WASM extractor adapter without metrics
    /// Public API for reliability system initialization
    #[allow(dead_code)]
    pub fn new(extractor: Arc<ConcreteWasmExtractor>) -> Self {
        Self {
            extractor,
            metrics: None,
        }
    }

    /// Create adapter with metrics tracking
    pub fn with_metrics(
        extractor: Arc<ConcreteWasmExtractor>,
        metrics: Arc<crate::metrics::RipTideMetrics>,
    ) -> Self {
        Self {
            extractor,
            metrics: Some(metrics),
        }
    }
}

impl WasmExtractorTrait for WasmExtractorAdapter {
    fn extract(&self, html: &[u8], url: &str, mode: &str) -> Result<ExtractedDoc> {
        let start = Instant::now();

        // Call the concrete extractor
        let doc = self.extractor.extract(html, url, mode)?;

        // Record WASM cold start time if metrics available
        if let Some(ref metrics) = self.metrics {
            let cold_start_ms = start.elapsed().as_millis() as f64;
            metrics.update_wasm_cold_start_time(cold_start_ms);

            // Note: Memory metrics would require WASM runtime introspection
            // For now, we estimate based on input/output size
            let estimated_pages = (html.len() + doc.text.len()) / 65536; // 64KB per page
            metrics.update_wasm_memory_metrics(
                estimated_pages,
                0, // No grow failures tracked yet
                estimated_pages,
            );
        }

        // Convert riptide_extraction::ExtractedDoc to riptide_core::ExtractedDoc
        Ok(ExtractedDoc {
            url: doc.url,
            title: doc.title,
            byline: doc.byline,
            published_iso: doc.published_iso,
            markdown: doc.markdown,
            text: doc.text,
            links: doc.links,
            media: doc.media,
            language: doc.language,
            reading_time: doc.reading_time,
            quality_score: doc.quality_score,
            word_count: doc.word_count,
            categories: doc.categories,
            site_name: doc.site_name,
            description: doc.description,
            html: None,
        })
    }
}

/// Implement ReliabilityMetricsRecorder for RipTideMetrics
impl ReliabilityMetricsRecorder for crate::metrics::RipTideMetrics {
    fn record_extraction_fallback(&self, from_mode: &str, to_mode: &str, reason: &str) {
        self.record_extraction_fallback(from_mode, to_mode, reason);
    }
}
