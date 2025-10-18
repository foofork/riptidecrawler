//! Component module for WASM extraction support
//!
//! This module provides minimal types needed for WASM component integration.
//! The actual extraction logic has been moved to riptide-extraction.
//!
//! **NOTICE**: Pool configuration types have been moved to riptide-pool crate.
//! These re-exports are for backward compatibility.

use crate::reliability::WasmExtractor;
use crate::types::ExtractedDoc;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

// Re-export pool config types from riptide-pool for backward compatibility
pub use riptide_pool::{ExtractorConfig, PerformanceMetrics, WasmResourceTracker};

/// Main extractor component (placeholder)
#[derive(Clone)]
pub struct CmExtractor {
    #[allow(dead_code)]
    config: Arc<ExtractorConfig>,
    #[allow(dead_code)]
    metrics: Arc<Mutex<PerformanceMetrics>>,
}

impl CmExtractor {
    pub fn new(config: ExtractorConfig) -> Self {
        Self {
            config: Arc::new(config),
            metrics: Arc::new(Mutex::new(PerformanceMetrics::default())),
        }
    }

    pub async fn extract(&self, _content: &str) -> Result<String> {
        // Placeholder - actual extraction logic in riptide-extraction
        Ok(String::new())
    }
}

impl WasmExtractor for CmExtractor {
    fn extract(&self, html: &[u8], url: &str, _mode: &str) -> Result<ExtractedDoc> {
        // Mock implementation - actual extraction logic in riptide-extraction
        let html_str = String::from_utf8_lossy(html);
        Ok(ExtractedDoc {
            url: url.to_string(),
            title: Some("Mock Title".to_string()),
            text: html_str.chars().take(1000).collect(),
            quality_score: Some(85),
            links: vec![],
            byline: None,
            published_iso: None,
            markdown: Some("# Mock Content".to_string()),
            media: vec![],
            language: Some("en".to_string()),
            reading_time: Some(2),
            word_count: Some(200),
            categories: vec![],
            site_name: None,
            description: Some("Mock description".to_string()),
        })
    }
}
