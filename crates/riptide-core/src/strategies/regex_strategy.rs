//! Regex Pattern-based extraction strategy
//!
//! This module provides a regex-based strategy for extracting structured data
//! patterns like emails, phone numbers, URLs, and dates from text content.

use anyhow::Result;
use async_trait::async_trait;
use regex::Regex;
use std::collections::HashMap;

use crate::strategies::{traits::*, ExtractedContent};

/// Regex pattern configuration
#[derive(Debug, Clone)]
pub struct PatternConfig {
    pub name: String,
    pub regex: Regex,
    pub description: String,
}

impl PatternConfig {
    pub fn new(name: &str, pattern: &str, description: &str) -> Result<Self> {
        Ok(Self {
            name: name.to_string(),
            regex: Regex::new(pattern)?,
            description: description.to_string(),
        })
    }
}

/// Regex Pattern strategy for structured data extraction
#[derive(Debug, Clone)]
pub struct RegexPatternStrategy {
    patterns: HashMap<String, PatternConfig>,
}

impl RegexPatternStrategy {
    /// Create a new regex strategy with default patterns
    pub fn new() -> Self {
        let mut patterns = HashMap::new();

        // Email pattern
        if let Ok(config) = PatternConfig::new(
            "email",
            r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b",
            "Email addresses",
        ) {
            patterns.insert("email".to_string(), config);
        }

        // Phone pattern (US format)
        if let Ok(config) = PatternConfig::new(
            "phone",
            r"\b(?:\+?1[-.]?)?\(?([0-9]{3})\)?[-.]?([0-9]{3})[-.]?([0-9]{4})\b",
            "Phone numbers",
        ) {
            patterns.insert("phone".to_string(), config);
        }

        // URL pattern
        if let Ok(config) = PatternConfig::new("url", r"https?://[^\s<>]+", "URLs") {
            patterns.insert("url".to_string(), config);
        }

        // Date pattern (ISO format and common formats)
        if let Ok(config) = PatternConfig::new(
            "date",
            r"\b\d{4}-\d{2}-\d{2}\b|\b\d{1,2}/\d{1,2}/\d{4}\b|\b(?:Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)[a-z]*\s+\d{1,2},?\s+\d{4}\b",
            "Dates",
        ) {
            patterns.insert("date".to_string(), config);
        }

        // IP Address pattern
        if let Ok(config) = PatternConfig::new(
            "ip",
            r"\b(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\b",
            "IP addresses",
        ) {
            patterns.insert("ip".to_string(), config);
        }

        // Price pattern
        if let Ok(config) = PatternConfig::new(
            "price",
            r"\$\s*\d+(?:,\d{3})*(?:\.\d{2})?|\d+(?:,\d{3})*(?:\.\d{2})?\s*(?:USD|EUR|GBP)",
            "Prices",
        ) {
            patterns.insert("price".to_string(), config);
        }

        // Social Security Number pattern (masked for security)
        if let Ok(config) = PatternConfig::new(
            "ssn",
            r"\b\d{3}-\d{2}-\d{4}\b",
            "SSN (for detection, not extraction)",
        ) {
            patterns.insert("ssn".to_string(), config);
        }

        // Credit card pattern (basic, for detection)
        if let Ok(config) = PatternConfig::new(
            "credit_card",
            r"\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b",
            "Credit cards (for detection)",
        ) {
            patterns.insert("credit_card".to_string(), config);
        }

        Self { patterns }
    }

    /// Create strategy with custom patterns
    pub fn with_patterns(patterns: HashMap<String, PatternConfig>) -> Self {
        Self { patterns }
    }

    /// Add a custom pattern
    pub fn add_pattern(&mut self, name: String, config: PatternConfig) {
        self.patterns.insert(name, config);
    }

    /// Extract all matches for a specific pattern
    #[allow(dead_code)]
    fn extract_pattern(&self, text: &str, pattern_name: &str) -> Vec<String> {
        let config = match self.patterns.get(pattern_name) {
            Some(c) => c,
            None => return Vec::new(),
        };

        config
            .regex
            .find_iter(text)
            .map(|m| m.as_str().to_string())
            .collect()
    }

    /// Extract matches for all patterns
    fn extract_all_patterns(&self, text: &str) -> HashMap<String, Vec<String>> {
        let mut results = HashMap::new();

        for (name, config) in &self.patterns {
            let matches: Vec<String> = config
                .regex
                .find_iter(text)
                .map(|m| m.as_str().to_string())
                .collect();

            if !matches.is_empty() {
                results.insert(name.clone(), matches);
            }
        }

        results
    }

    /// Format structured data as readable text
    fn format_structured_data(&self, data: &HashMap<String, Vec<String>>) -> String {
        let mut output = Vec::new();

        for (pattern_name, matches) in data {
            if !matches.is_empty() {
                let config = self.patterns.get(pattern_name);
                let description = config
                    .map(|c| c.description.as_str())
                    .unwrap_or(pattern_name);

                output.push(format!("{}:", description));
                for (i, m) in matches.iter().enumerate() {
                    // Mask sensitive data
                    let display = if pattern_name == "ssn" || pattern_name == "credit_card" {
                        "[REDACTED]".to_string()
                    } else {
                        m.clone()
                    };
                    output.push(format!("  {}. {}", i + 1, display));
                }
                output.push(String::new()); // Empty line
            }
        }

        output.join("\n")
    }

    /// Calculate confidence based on number of matches
    fn calculate_confidence(&self, data: &HashMap<String, Vec<String>>) -> f64 {
        if data.is_empty() {
            return 0.3; // Low confidence if no patterns found
        }

        let total_matches: usize = data.values().map(|v| v.len()).sum();

        // More matches = higher confidence
        let match_score = (total_matches as f64 * 0.1).min(0.5);

        // More pattern types = higher confidence
        let pattern_score = (data.len() as f64 * 0.1).min(0.4);

        (0.5 + match_score + pattern_score).min(1.0)
    }
}

impl Default for RegexPatternStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ExtractionStrategy for RegexPatternStrategy {
    async fn extract(&self, html: &str, url: &str) -> Result<ExtractionResult> {
        let start = std::time::Instant::now();

        // Strip HTML tags for text extraction
        let text = strip_html_tags(html);

        // Extract structured data
        let structured_data = self.extract_all_patterns(&text);

        // Generate title
        let title = if structured_data.is_empty() {
            "No Structured Data Found".to_string()
        } else {
            format!("Structured Data ({})", structured_data.len())
        };

        // Format content
        let content = self.format_structured_data(&structured_data);

        // Generate summary
        let summary = if structured_data.is_empty() {
            Some("No recognizable patterns found in the content.".to_string())
        } else {
            let patterns: Vec<&str> = structured_data.keys().map(|s| s.as_str()).collect();
            Some(format!("Found patterns: {}", patterns.join(", ")))
        };

        let confidence = self.calculate_confidence(&structured_data);

        let extracted = ExtractedContent {
            title,
            content: if content.is_empty() {
                "No structured data extracted.".to_string()
            } else {
                content
            },
            summary,
            url: url.to_string(),
            strategy_used: "regex".to_string(),
            extraction_confidence: confidence,
        };

        let duration = start.elapsed();

        // Calculate quality metrics
        let quality = ExtractionQuality {
            content_length: extracted.content.len(),
            title_quality: 0.7,
            content_quality: if structured_data.is_empty() { 0.3 } else { 0.8 },
            structure_score: 0.9, // Regex extracts highly structured data
            metadata_completeness: confidence,
        };

        // Build metadata
        let mut metadata = HashMap::new();
        metadata.insert(
            "extraction_time_ms".to_string(),
            duration.as_millis().to_string(),
        );
        metadata.insert("pattern_count".to_string(), self.patterns.len().to_string());
        metadata.insert(
            "matches_found".to_string(),
            structured_data
                .values()
                .map(|v| v.len())
                .sum::<usize>()
                .to_string(),
        );
        metadata.insert(
            "pattern_types_found".to_string(),
            structured_data.len().to_string(),
        );

        // Add pattern-specific metadata
        for (pattern_name, matches) in &structured_data {
            metadata.insert(format!("{}_count", pattern_name), matches.len().to_string());
        }

        Ok(ExtractionResult {
            content: extracted.clone(),
            quality,
            performance: Some(crate::strategies::PerformanceMetrics::new()),
            metadata,
        })
    }

    fn name(&self) -> &str {
        "regex_pattern"
    }

    fn capabilities(&self) -> StrategyCapabilities {
        StrategyCapabilities {
            strategy_type: "regex_extraction".to_string(),
            supported_content_types: vec![
                "text/html".to_string(),
                "text/plain".to_string(),
                "application/xhtml+xml".to_string(),
            ],
            performance_tier: PerformanceTier::Fast,
            resource_requirements: ResourceRequirements {
                memory_tier: ResourceTier::Low,
                cpu_tier: ResourceTier::Low,
                requires_network: false,
                external_dependencies: vec!["regex".to_string()],
            },
        }
    }

    fn confidence_score(&self, html: &str) -> f64 {
        let text = strip_html_tags(html);
        let data = self.extract_all_patterns(&text);
        self.calculate_confidence(&data)
    }
}

/// Strip HTML tags from content
fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    let mut in_script_or_style = false;

    let html_lower = html.to_lowercase();

    for (i, c) in html.chars().enumerate() {
        match c {
            '<' => {
                in_tag = true;
                // Check if entering script or style tag
                let remaining = &html_lower[i..];
                if remaining.starts_with("<script") || remaining.starts_with("<style") {
                    in_script_or_style = true;
                }
            }
            '>' => {
                in_tag = false;
                // Check if exiting script or style tag
                let preceding = &html_lower[..i.min(html_lower.len())];
                if preceding.ends_with("script") || preceding.ends_with("style") {
                    in_script_or_style = false;
                }
            }
            _ if !in_tag && !in_script_or_style => result.push(c),
            _ => {}
        }
    }

    // Clean up whitespace
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_regex_email_extraction() {
        let strategy = RegexPatternStrategy::new();
        let text = "Contact us at support@example.com or sales@example.org";

        let result = strategy.extract(text, "https://example.com").await.unwrap();

        assert_eq!(result.content.strategy_used, "regex");
        assert!(result.content.content.contains("Email"));
        assert!(
            result
                .metadata
                .get("email_count")
                .unwrap()
                .parse::<usize>()
                .unwrap()
                == 2
        );
    }

    #[tokio::test]
    async fn test_regex_phone_extraction() {
        let strategy = RegexPatternStrategy::new();
        let text = "Call us at 555-123-4567 or (555) 987-6543";

        let result = strategy.extract(text, "https://example.com").await.unwrap();

        assert!(result.content.content.contains("Phone"));
    }

    #[tokio::test]
    async fn test_regex_url_extraction() {
        let strategy = RegexPatternStrategy::new();
        let text = "Visit https://example.com or http://test.org";

        let result = strategy.extract(text, "https://example.com").await.unwrap();

        assert!(result.content.content.contains("URLs"));
    }

    #[tokio::test]
    async fn test_regex_no_matches() {
        let strategy = RegexPatternStrategy::new();
        let text = "Just plain text with nothing special";

        let result = strategy.extract(text, "https://example.com").await.unwrap();

        assert!(result.content.content.contains("No structured data"));
        assert!(result.content.extraction_confidence < 0.5);
    }

    #[tokio::test]
    async fn test_sensitive_data_redaction() {
        let strategy = RegexPatternStrategy::new();
        let text = "SSN: 123-45-6789 and card 1234-5678-9012-3456";

        let result = strategy.extract(text, "https://example.com").await.unwrap();

        // Should detect but redact sensitive data
        assert!(result.content.content.contains("[REDACTED]"));
        assert!(!result.content.content.contains("123-45-6789"));
    }
}
