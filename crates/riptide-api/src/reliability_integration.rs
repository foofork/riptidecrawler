//! Reliability integration module for RipTide API
//!
//! This module provides the integration between riptide-core's ReliableExtractor
//! and riptide-html's WasmExtractor, implementing the trait adapter pattern.

use anyhow::Result;
use riptide_core::{reliability::WasmExtractor as WasmExtractorTrait, types::ExtractedDoc};
use riptide_html::wasm_extraction::WasmExtractor as ConcreteWasmExtractor;
use std::sync::Arc;

/// Adapter to make riptide_html::WasmExtractor compatible with the reliability trait
pub struct WasmExtractorAdapter {
    extractor: Arc<ConcreteWasmExtractor>,
}

impl WasmExtractorAdapter {
    pub fn new(extractor: Arc<ConcreteWasmExtractor>) -> Self {
        Self { extractor }
    }
}

impl WasmExtractorTrait for WasmExtractorAdapter {
    fn extract(&self, html: &[u8], url: &str, mode: &str) -> Result<ExtractedDoc> {
        // Call the concrete extractor
        let doc = self.extractor.extract(html, url, mode)?;

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
