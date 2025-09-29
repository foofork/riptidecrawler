//! Strategy trait implementations for riptide-html
//!
//! This module provides implementations of the riptide-core ExtractionStrategy trait
//! for the HTML processing components in this crate.
//!
//! This module is only available when the "strategy-traits" feature is enabled.

#[cfg(feature = "strategy-traits")]
pub use strategy_impls::*;

#[cfg(feature = "strategy-traits")]
mod strategy_impls {
    use async_trait::async_trait;
    use std::collections::HashMap;
    use anyhow::Result;

    use crate::{RegexPattern, HtmlProcessor};
    use crate::processor::DefaultHtmlProcessor;

    // Re-export traits from riptide-core
    pub use riptide_core::strategies::traits::{
        ExtractionStrategy, ExtractionResult, ExtractionQuality,
        StrategyCapabilities, PerformanceTier, ResourceRequirements, ResourceTier
    };
    pub use riptide_core::strategies::{PerformanceMetrics, ProcessedContent};
    pub use crate::ExtractedContent;

/// CSS-based extraction strategy implementation for riptide-html
#[derive(Debug, Clone)]
pub struct HtmlCssExtractionStrategy {
    selectors: HashMap<String, String>,
    processor: DefaultHtmlProcessor,
}

impl HtmlCssExtractionStrategy {
    pub fn new(selectors: HashMap<String, String>) -> Self {
        Self {
            selectors,
            processor: DefaultHtmlProcessor::default(),
        }
    }

    pub fn with_default_selectors() -> Self {
        let selectors = crate::css_extraction::default_selectors();
        Self::new(selectors)
    }
}

#[async_trait]
impl ExtractionStrategy for HtmlCssExtractionStrategy {
    async fn extract(&self, html: &str, url: &str) -> Result<ExtractionResult> {
        let start = std::time::Instant::now();

        // Use the processor to extract content
        let content = self.processor
            .extract_with_css(html, url, &self.selectors)
            .await
            .map_err(|e| anyhow::anyhow!("CSS extraction failed: {}", e))?;

        let duration = start.elapsed();

        // Calculate quality metrics
        let quality = ExtractionQuality {
            content_length: content.content.len(),
            title_quality: if content.title.is_empty() { 0.0 } else { 0.9 },
            content_quality: calculate_content_quality(&content.content),
            structure_score: 0.9, // CSS provides excellent structure
            metadata_completeness: if content.summary.is_some() { 0.8 } else { 0.6 },
        };

        // Collect metadata
        let mut metadata = HashMap::new();
        metadata.insert("extraction_time_ms".to_string(), duration.as_millis().to_string());
        metadata.insert("selectors_count".to_string(), self.selectors.len().to_string());
        metadata.insert("processor".to_string(), "riptide-html-css".to_string());

        Ok(ExtractionResult {
            content,
            quality,
            performance: None, // We don't have PerformanceMetrics here
            metadata,
        })
    }

    fn name(&self) -> &str {
        "html_css"
    }

    fn capabilities(&self) -> StrategyCapabilities {
        StrategyCapabilities {
            strategy_type: "css_html_extraction".to_string(),
            supported_content_types: vec![
                "text/html".to_string(),
                "application/xhtml+xml".to_string(),
            ],
            performance_tier: PerformanceTier::Balanced,
            resource_requirements: ResourceRequirements {
                memory_tier: ResourceTier::Medium,
                cpu_tier: ResourceTier::Medium,
                requires_network: false,
                external_dependencies: vec!["scraper".to_string()],
            },
            features: vec![
                "css_selectors".to_string(),
                "structured_extraction".to_string(),
                "dom_aware".to_string(),
            ],
        }
    }

    fn confidence_score(&self, html: &str) -> f64 {
        self.processor.confidence_score(html)
    }
}

/// Regex-based extraction strategy implementation for riptide-html
#[derive(Debug, Clone)]
pub struct HtmlRegexExtractionStrategy {
    patterns: Vec<RegexPattern>,
    processor: DefaultHtmlProcessor,
}

impl HtmlRegexExtractionStrategy {
    pub fn new(patterns: Vec<RegexPattern>) -> Self {
        Self {
            patterns,
            processor: DefaultHtmlProcessor::default(),
        }
    }

    pub fn with_default_patterns() -> Self {
        let patterns = crate::regex_extraction::default_patterns();
        Self::new(patterns)
    }
}

#[async_trait]
impl ExtractionStrategy for HtmlRegexExtractionStrategy {
    async fn extract(&self, html: &str, url: &str) -> Result<ExtractionResult> {
        let start = std::time::Instant::now();

        // Use the processor to extract content
        let content = self.processor
            .extract_with_regex(html, url, &self.patterns)
            .await
            .map_err(|e| anyhow::anyhow!("Regex extraction failed: {}", e))?;

        let duration = start.elapsed();

        // Calculate quality metrics
        let quality = ExtractionQuality {
            content_length: content.content.len(),
            title_quality: if content.title.is_empty() { 0.0 } else { 0.7 },
            content_quality: calculate_content_quality(&content.content),
            structure_score: 0.6, // Regex has less structural understanding
            metadata_completeness: if content.summary.is_some() { 0.7 } else { 0.5 },
        };

        // Collect metadata
        let mut metadata = HashMap::new();
        metadata.insert("extraction_time_ms".to_string(), duration.as_millis().to_string());
        metadata.insert("patterns_count".to_string(), self.patterns.len().to_string());
        metadata.insert("processor".to_string(), "riptide-html-regex".to_string());

        Ok(ExtractionResult {
            content,
            quality,
            performance: None,
            metadata,
        })
    }

    fn name(&self) -> &str {
        "html_regex"
    }

    fn capabilities(&self) -> StrategyCapabilities {
        StrategyCapabilities {
            strategy_type: "regex_html_extraction".to_string(),
            supported_content_types: vec![
                "text/html".to_string(),
                "text/plain".to_string(),
                "application/xml".to_string(),
            ],
            performance_tier: PerformanceTier::Fast,
            resource_requirements: ResourceRequirements {
                memory_tier: ResourceTier::Low,
                cpu_tier: ResourceTier::Low,
                requires_network: false,
                external_dependencies: vec!["regex".to_string()],
            },
            features: vec![
                "pattern_matching".to_string(),
                "fast_extraction".to_string(),
                "flexible_patterns".to_string(),
            ],
        }
    }

    fn confidence_score(&self, html: &str) -> f64 {
        // Regex is always available but not the best choice
        // Slightly higher for simple/unstructured content
        if html.contains("<html") && html.contains("</html>") {
            0.4 // Structured HTML - CSS would be better
        } else {
            0.6 // Less structured - regex might work well
        }
    }
}

/// Unified HTML processor that implements ExtractionStrategy
#[derive(Debug, Clone)]
pub struct HtmlProcessorStrategy {
    processor: DefaultHtmlProcessor,
    default_selectors: HashMap<String, String>,
    default_patterns: Vec<RegexPattern>,
    prefer_css: bool,
}

impl HtmlProcessorStrategy {
    pub fn new() -> Self {
        Self {
            processor: DefaultHtmlProcessor::default(),
            default_selectors: crate::css_extraction::default_selectors(),
            default_patterns: crate::regex_extraction::default_patterns(),
            prefer_css: true,
        }
    }

    pub fn prefer_regex(mut self) -> Self {
        self.prefer_css = false;
        self
    }

    pub fn with_selectors(mut self, selectors: HashMap<String, String>) -> Self {
        self.default_selectors = selectors;
        self
    }

    pub fn with_patterns(mut self, patterns: Vec<RegexPattern>) -> Self {
        self.default_patterns = patterns;
        self
    }
}

impl Default for HtmlProcessorStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ExtractionStrategy for HtmlProcessorStrategy {
    async fn extract(&self, html: &str, url: &str) -> Result<ExtractionResult> {
        let start = std::time::Instant::now();

        // Try CSS first if preferred and HTML is structured
        let (content, method_used) = if self.prefer_css && html.contains("<") {
            match self.processor.extract_with_css(html, url, &self.default_selectors).await {
                Ok(content) => (content, "css"),
                Err(_) => {
                    // Fallback to regex
                    let content = self.processor
                        .extract_with_regex(html, url, &self.default_patterns)
                        .await
                        .map_err(|e| anyhow::anyhow!("Both CSS and regex extraction failed: {}", e))?;
                    (content, "regex_fallback")
                }
            }
        } else {
            // Use regex directly
            let content = self.processor
                .extract_with_regex(html, url, &self.default_patterns)
                .await
                .map_err(|e| anyhow::anyhow!("Regex extraction failed: {}", e))?;
            (content, "regex")
        };

        let duration = start.elapsed();

        // Calculate quality based on method used
        let quality = ExtractionQuality {
            content_length: content.content.len(),
            title_quality: if content.title.is_empty() { 0.0 } else {
                match method_used {
                    "css" => 0.9,
                    "regex" => 0.7,
                    "regex_fallback" => 0.6,
                    _ => 0.5,
                }
            },
            content_quality: calculate_content_quality(&content.content),
            structure_score: match method_used {
                "css" => 0.9,
                "regex" => 0.6,
                "regex_fallback" => 0.5,
                _ => 0.4,
            },
            metadata_completeness: if content.summary.is_some() { 0.8 } else { 0.6 },
        };

        // Collect metadata
        let mut metadata = HashMap::new();
        metadata.insert("extraction_time_ms".to_string(), duration.as_millis().to_string());
        metadata.insert("method_used".to_string(), method_used.to_string());
        metadata.insert("processor".to_string(), "riptide-html-unified".to_string());

        Ok(ExtractionResult {
            content,
            quality,
            performance: None,
            metadata,
        })
    }

    fn name(&self) -> &str {
        "html_processor"
    }

    fn capabilities(&self) -> StrategyCapabilities {
        StrategyCapabilities {
            strategy_type: "unified_html_extraction".to_string(),
            supported_content_types: vec![
                "text/html".to_string(),
                "application/xhtml+xml".to_string(),
                "text/plain".to_string(),
            ],
            performance_tier: PerformanceTier::Balanced,
            resource_requirements: ResourceRequirements {
                memory_tier: ResourceTier::Medium,
                cpu_tier: ResourceTier::Medium,
                requires_network: false,
                external_dependencies: vec!["scraper".to_string(), "regex".to_string()],
            },
            features: vec![
                "css_selectors".to_string(),
                "regex_patterns".to_string(),
                "adaptive_method".to_string(),
                "fallback_support".to_string(),
            ],
        }
    }

    fn confidence_score(&self, html: &str) -> f64 {
        self.processor.confidence_score(html)
    }
}

/// Calculate content quality score
fn calculate_content_quality(content: &str) -> f64 {
    let mut score: f64 = 0.0;

    // Length bonus
    let length = content.len();
    if length > 100 {
        score += 0.3;
    }
    if length > 500 {
        score += 0.2;
    }
    if length > 1000 {
        score += 0.2;
    }

    // Structure indicators
    let sentences = content.matches('.').count() + content.matches('!').count() + content.matches('?').count();
    if sentences > 3 {
        score += 0.2;
    }

    // Paragraph structure
    let paragraphs = content.matches('\n').count();
    if paragraphs > 1 {
        score += 0.1;
    }

    score.min(1.0)
}

} // end of strategy_impls module