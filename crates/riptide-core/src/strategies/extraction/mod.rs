//! Content extraction strategies

pub mod trek;
pub mod css_json;
pub mod regex;
pub mod llm;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::strategies::ExtractedContent;

/// Common extraction trait for all strategies
pub trait ContentExtractor {
    async fn extract(&self, html: &str, url: &str) -> Result<ExtractedContent>;
    fn confidence_score(&self, html: &str) -> f64;
    fn strategy_name(&self) -> &'static str;
}

/// Extraction quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionQuality {
    pub content_length: usize,
    pub title_quality: f64,
    pub content_quality: f64,
    pub structure_score: f64,
    pub metadata_completeness: f64,
}

impl ExtractionQuality {
    pub fn overall_score(&self) -> f64 {
        (self.title_quality + self.content_quality + self.structure_score + self.metadata_completeness) / 4.0
    }
}