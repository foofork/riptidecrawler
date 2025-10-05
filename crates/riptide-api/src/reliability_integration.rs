//! Reliability integration module for RipTide API
//!
//! This module provides the integration between riptide-core's ReliableExtractor
//! and riptide-html's WasmExtractor, implementing the trait adapter pattern.

use anyhow::Result;
use riptide_core::{reliability::WasmExtractor as WasmExtractorTrait, types::ExtractedDoc};
use riptide_html::wasm_extraction::WasmExtractor as ConcreteWasmExtractor;
use std::sync::Arc;
use std::time::Instant;

/// Adapter to make riptide_html::WasmExtractor compatible with the reliability trait
pub struct WasmExtractorAdapter {
    extractor: Arc<ConcreteWasmExtractor>,
    metrics: Option<Arc<crate::metrics::RipTideMetrics>>,
}

impl WasmExtractorAdapter {
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

        // Convert riptide_html::ExtractedDoc to riptide_core::ExtractedDoc
        Ok(ExtractedDoc {
            url: doc.url,
            title: doc.title,
            byline: doc.byline,
            published_iso: doc.published_iso,
            markdown: Some(doc.markdown),
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
        })
    }
}
