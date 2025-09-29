//! CSS selector to JSON extraction strategy
//!
//! This module provides functionality to extract content from HTML using CSS selectors
//! and map the results to structured JSON data. It supports flexible selector configuration
//! and provides default selectors for common content types.
//!
//! Week 5 Features:
//! - CSS-001: Enhanced CSS selector engine with class/id/attr selectors, child/descendant combinators, :nth-child pseudo-selectors
//! - CSS-002: :has-text() post-filter for text content matching
//! - CSS-003: 12 content transformers (trim, normalize_ws, number, currency, etc.)
//! - CSS-004: CSS-wins merge policy with conflict resolution

use anyhow::{Result, Context, bail};
use scraper::{Html, Selector};
use std::collections::HashMap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use url::Url;
use crate::ExtractedContent;

/// Enhanced CSS selector configuration with transformers and post-filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CssSelectorConfig {
    /// CSS selector string with enhanced support
    pub selector: String,
    /// Optional transformers to apply to extracted content
    pub transformers: Vec<String>,
    /// Optional :has-text() post-filter
    pub has_text_filter: Option<HasTextFilter>,
    /// Fallback selectors if primary fails
    pub fallbacks: Vec<String>,
    /// Whether this field is required
    pub required: bool,
    /// Field-specific merge policy
    pub merge_policy: Option<MergePolicy>,
}

/// :has-text() pseudo-selector filter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HasTextFilter {
    /// Text pattern to match
    pub pattern: String,
    /// Case-insensitive matching
    pub case_insensitive: bool,
    /// Partial text matching (default: true)
    pub partial_match: bool,
    /// Use regex matching instead of literal string
    pub regex_mode: bool,
    /// Compiled regex pattern (internal use)
    #[serde(skip)]
    pub regex: Option<Regex>,
}

/// Merge policy for handling conflicts between extraction methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MergePolicy {
    /// CSS selector results take precedence
    CssWins,
    /// Other extraction method takes precedence
    OtherWins,
    /// Merge both results
    Merge,
    /// Use first non-empty result
    FirstValid,
}

/// Conflict audit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictAudit {
    pub field: String,
    pub css_value: Option<String>,
    pub other_value: Option<String>,
    pub resolution: String,
    pub policy_used: MergePolicy,
}

/// CSS-based content extractor with enhanced features
pub struct CssJsonExtractor {
    selectors: HashMap<String, CssSelectorConfig>,
    global_merge_policy: MergePolicy,
    transformers: TransformerRegistry,
}

/// Content transformer registry
pub struct TransformerRegistry {
    transformers: HashMap<String, Box<dyn ContentTransformer + Send + Sync>>,
}

impl Default for TransformerRegistry {
    fn default() -> Self {
        let mut registry = Self {
            transformers: HashMap::new(),
        };

        // Register all 12 transformers from CSS-003
        registry.register("trim", Box::new(TrimTransformer));
        registry.register("normalize_ws", Box::new(NormalizeWhitespaceTransformer));
        registry.register("number", Box::new(NumberTransformer));
        registry.register("currency", Box::new(CurrencyTransformer));
        registry.register("date_iso", Box::new(DateIsoTransformer));
        registry.register("url_abs", Box::new(UrlAbsoluteTransformer));
        registry.register("lowercase", Box::new(LowercaseTransformer));
        registry.register("uppercase", Box::new(UppercaseTransformer));
        registry.register("split", Box::new(SplitTransformer));
        registry.register("join", Box::new(JoinTransformer));
        registry.register("regex_extract", Box::new(RegexExtractTransformer));
        registry.register("regex_replace", Box::new(RegexReplaceTransformer));
        registry.register("json_parse", Box::new(JsonParseTransformer));
        registry.register("html_decode", Box::new(HtmlDecodeTransformer));

        registry
    }
}

impl TransformerRegistry {
    pub fn register(&mut self, name: &str, transformer: Box<dyn ContentTransformer + Send + Sync>) {
        self.transformers.insert(name.to_string(), transformer);
    }

    pub fn transform(&self, content: &str, transformer_name: &str, base_url: Option<&str>) -> Result<String> {
        if let Some(transformer) = self.transformers.get(transformer_name) {
            transformer.transform(content, base_url)
        } else {
            bail!("Unknown transformer: {}", transformer_name)
        }
    }
}

/// Content transformer trait for CSS-003
pub trait ContentTransformer {
    fn transform(&self, content: &str, base_url: Option<&str>) -> Result<String>;
}

impl CssJsonExtractor {
    /// Create a new CSS extractor with enhanced selector configurations
    pub fn new(selectors: HashMap<String, CssSelectorConfig>) -> Self {
        Self {
            selectors,
            global_merge_policy: MergePolicy::CssWins,
            transformers: TransformerRegistry::default(),
        }
    }

    /// Create extractor from simple selector strings (backward compatibility)
    pub fn from_simple_selectors(selectors: HashMap<String, String>) -> Self {
        let enhanced_selectors = selectors.into_iter().map(|(field, selector)| {
            (field, CssSelectorConfig {
                selector,
                transformers: vec![],
                has_text_filter: None,
                fallbacks: vec![],
                required: false,
                merge_policy: None,
            })
        }).collect();

        Self::new(enhanced_selectors)
    }

    /// Set global merge policy
    pub fn with_merge_policy(mut self, policy: MergePolicy) -> Self {
        self.global_merge_policy = policy;
        self
    }

    /// Extract content using enhanced CSS selectors with transformers and post-filters
    pub async fn extract(&self, html: &str, url: &str) -> Result<ExtractedContent> {
        let document = Html::parse_document(html);
        let mut extracted_data = HashMap::new();
        let _conflicts: Vec<ConflictAudit> = Vec::new();

        // Extract data using enhanced CSS selectors
        for (field, config) in &self.selectors {
            let values = self.extract_field(&document, field, config, url)
                .with_context(|| format!("Failed to extract field: {}", field))?;

            if !values.is_empty() {
                extracted_data.insert(field.clone(), values);
            } else if config.required {
                eprintln!("WARN: Required field '{}' not found", field);
            }
        }

        // Build content from extracted data
        let title = extracted_data
            .get("title")
            .and_then(|v| v.first())
            .cloned()
            .unwrap_or_else(|| "Untitled".to_string());

        let content = extracted_data
            .get("content")
            .map(|v| v.join("\n\n"))
            .or_else(|| extracted_data.get("body").map(|v| v.join("\n\n")))
            .unwrap_or_else(|| {
                // Fallback: combine all non-title fields
                extracted_data
                    .iter()
                    .filter(|(k, _)| *k != "title")
                    .flat_map(|(_, v)| v.iter())
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("\n\n")
            });

        let summary = extracted_data
            .get("description")
            .and_then(|v| v.first())
            .cloned();

        Ok(ExtractedContent {
            title,
            content,
            summary,
            url: url.to_string(),
            strategy_used: "css_json_enhanced".to_string(),
            extraction_confidence: self.confidence_score(html),
        })
    }

    /// Extract a single field using enhanced CSS selectors
    fn extract_field(
        &self,
        document: &Html,
        _field: &str,
        config: &CssSelectorConfig,
        base_url: &str
    ) -> Result<Vec<String>> {
        let mut selectors_to_try = vec![config.selector.clone()];
        selectors_to_try.extend(config.fallbacks.clone());

        for selector_str in selectors_to_try {
            if let Some(values) = self.try_extract_with_selector(document, &selector_str, config, base_url)? {
                return Ok(values);
            }
        }

        Ok(vec![])
    }

    /// Try extracting with a specific selector
    fn try_extract_with_selector(
        &self,
        document: &Html,
        selector_str: &str,
        config: &CssSelectorConfig,
        base_url: &str,
    ) -> Result<Option<Vec<String>>> {
        // Parse enhanced CSS selector (CSS-001)
        let selector = self.parse_enhanced_selector(selector_str)?;

        let mut values: Vec<String> = document
            .select(&selector)
            .filter_map(|element| {
                // Extract text content
                let text = if let Some(content) = element.value().attr("content") {
                    content.to_string()
                } else {
                    element.text().collect::<Vec<_>>().join(" ").trim().to_string()
                };

                if text.is_empty() {
                    return None;
                }

                // Apply :has-text() post-filter (CSS-002)
                if let Some(filter) = &config.has_text_filter {
                    if !self.apply_has_text_filter(&text, filter) {
                        return None;
                    }
                }

                Some(text)
            })
            .collect();

        if values.is_empty() {
            return Ok(None);
        }

        // Apply transformers (CSS-003)
        for transformer_name in &config.transformers {
            values = values.into_iter()
                .filter_map(|text| {
                    self.transformers.transform(&text, transformer_name, Some(base_url))
                        .map_err(|e| eprintln!("WARN: Transformer '{}' failed: {}", transformer_name, e))
                        .ok()
                })
                .collect();
        }

        Ok(Some(values))
    }

    /// Parse enhanced CSS selector with support for advanced features
    fn parse_enhanced_selector(&self, selector_str: &str) -> Result<Selector> {
        // Handle custom :has-text() pseudo-selector by removing it for scraper parsing
        let cleaned_selector = if selector_str.contains(":has-text(") {
            // Remove :has-text() for now, we'll handle it as post-filter
            let re = Regex::new(r":has-text\([^)]+\)").unwrap();
            re.replace_all(selector_str, "").trim().to_string()
        } else {
            selector_str.to_string()
        };

        // Handle advanced pseudo-selectors that scraper might not support
        let enhanced_selector = self.enhance_pseudo_selectors(&cleaned_selector);

        Selector::parse(&enhanced_selector)
            .map_err(|e| anyhow::anyhow!("Invalid CSS selector '{}': {:?}", selector_str, e))
    }

    /// Enhance pseudo-selectors for better compatibility
    fn enhance_pseudo_selectors(&self, selector: &str) -> String {
        // Handle :nth-of-type() by converting to :nth-child() approximation
        let re_nth_type = Regex::new(r":nth-of-type\((\w+)\)").unwrap();
        let enhanced = re_nth_type.replace_all(selector, ":nth-child($1)");

        // Handle :first-of-type and :last-of-type
        let enhanced = enhanced.replace(":first-of-type", ":first-child");
        let enhanced = enhanced.replace(":last-of-type", ":last-child");

        enhanced.to_string()
    }

    /// Apply :has-text() post-filter (CSS-002) with regex support
    fn apply_has_text_filter(&self, text: &str, filter: &HasTextFilter) -> bool {
        if filter.regex_mode {
            // Use regex matching
            if let Some(regex) = &filter.regex {
                regex.is_match(text)
            } else {
                // Try to compile regex on-demand
                let pattern = if filter.case_insensitive {
                    format!("(?i){}", filter.pattern)
                } else {
                    filter.pattern.clone()
                };

                if let Ok(regex) = Regex::new(&pattern) {
                    regex.is_match(text)
                } else {
                    false // Invalid regex
                }
            }
        } else {
            // Use literal string matching
            let search_text = if filter.case_insensitive {
                text.to_lowercase()
            } else {
                text.to_string()
            };

            let pattern = if filter.case_insensitive {
                filter.pattern.to_lowercase()
            } else {
                filter.pattern.clone()
            };

            if filter.partial_match {
                search_text.contains(&pattern)
            } else {
                search_text == pattern
            }
        }
    }

    /// Merge extraction results with conflict resolution (CSS-004)
    pub fn merge_with_other(
        &self,
        css_results: &HashMap<String, Vec<String>>,
        other_results: &HashMap<String, Vec<String>>,
    ) -> (HashMap<String, Vec<String>>, Vec<ConflictAudit>) {
        let mut merged = HashMap::new();
        let mut conflicts = Vec::new();

        let all_fields: std::collections::HashSet<_> = css_results.keys()
            .chain(other_results.keys())
            .collect();

        for field in all_fields {
            let css_value = css_results.get(field);
            let other_value = other_results.get(field);

            let policy = self.selectors.get(field)
                .and_then(|config| config.merge_policy.as_ref())
                .unwrap_or(&self.global_merge_policy);

            let (final_value, _resolution) = match (css_value, other_value) {
                (Some(css), Some(other)) if css != other => {
                    // Conflict detected
                    let audit = ConflictAudit {
                        field: field.clone(),
                        css_value: css.first().cloned(),
                        other_value: other.first().cloned(),
                        resolution: "".to_string(),
                        policy_used: policy.clone(),
                    };

                    let (result, desc) = match policy {
                        MergePolicy::CssWins => (css.clone(), "CSS wins"),
                        MergePolicy::OtherWins => (other.clone(), "Other wins"),
                        MergePolicy::Merge => {
                            let mut combined = css.clone();
                            combined.extend(other.clone());
                            (combined, "Merged both")
                        },
                        MergePolicy::FirstValid => (css.clone(), "First valid (CSS)"),
                    };

                    conflicts.push(ConflictAudit {
                        resolution: desc.to_string(),
                        ..audit
                    });

                    (result, desc)
                },
                (Some(css), None) => (css.clone(), "CSS only"),
                (None, Some(other)) => (other.clone(), "Other only"),
                (Some(css), Some(_other)) => (css.clone(), "No conflict"),
                (None, None) => continue,
            };

            merged.insert(field.clone(), final_value);
        }

        (merged, conflicts)
    }

    /// Calculate confidence score based on selector matches and quality
    pub fn confidence_score(&self, html: &str) -> f64 {
        let document = Html::parse_document(html);
        let mut found_selectors = 0;
        let mut quality_score = 0.0;
        let total_selectors = self.selectors.len();

        for (field, config) in &self.selectors {
            if let Ok(selector) = self.parse_enhanced_selector(&config.selector) {
                if let Some(element) = document.select(&selector).next() {
                    found_selectors += 1;

                    // Calculate quality based on content length and field importance
                    let content = element.text().collect::<String>();
                    let field_weight = match field.as_str() {
                        "title" => 0.3,
                        "content" | "body" => 0.4,
                        "description" | "summary" => 0.2,
                        _ => 0.1,
                    };

                    let content_quality = (content.len().min(500) as f64 / 500.0).min(1.0);
                    quality_score += field_weight * content_quality;
                }
            }
        }

        if total_selectors == 0 {
            0.5
        } else {
            let match_ratio = found_selectors as f64 / total_selectors as f64;
            (match_ratio * 0.6 + quality_score * 0.4).min(0.95)
        }
    }

    /// Get strategy name
    pub fn strategy_name(&self) -> &'static str {
        "css_json"
    }
}

// CSS-003: 12 Content Transformers Implementation

/// Transformer 1: Trim whitespace
struct TrimTransformer;

impl ContentTransformer for TrimTransformer {
    fn transform(&self, content: &str, _base_url: Option<&str>) -> Result<String> {
        Ok(content.trim().to_string())
    }
}

/// Transformer 2: Normalize internal whitespace
struct NormalizeWhitespaceTransformer;

impl ContentTransformer for NormalizeWhitespaceTransformer {
    fn transform(&self, content: &str, _base_url: Option<&str>) -> Result<String> {
        let re = Regex::new(r"\s+").unwrap();
        Ok(re.replace_all(content.trim(), " ").to_string())
    }
}

/// Transformer 3: Extract numeric values
struct NumberTransformer;

impl ContentTransformer for NumberTransformer {
    fn transform(&self, content: &str, _base_url: Option<&str>) -> Result<String> {
        let re = Regex::new(r"[-+]?\d*\.?\d+([eE][-+]?\d+)?").unwrap();
        if let Some(mat) = re.find(content) {
            Ok(mat.as_str().to_string())
        } else {
            bail!("No numeric value found in: {}", content)
        }
    }
}

/// Transformer 4: Parse currency values
struct CurrencyTransformer;

impl ContentTransformer for CurrencyTransformer {
    fn transform(&self, content: &str, _base_url: Option<&str>) -> Result<String> {
        let re = Regex::new(r"[$€£¥]?\s*(\d+(?:[.,]\d{3})*(?:[.,]\d{2})?)").unwrap();
        if let Some(captures) = re.captures(content) {
            if let Some(amount) = captures.get(1) {
                // Normalize decimal separators
                let normalized = amount.as_str().replace(',', ".");
                return Ok(normalized);
            }
        }
        bail!("No currency value found in: {}", content)
    }
}

/// Transformer 5: Convert to ISO date format
struct DateIsoTransformer;

impl ContentTransformer for DateIsoTransformer {
    fn transform(&self, content: &str, _base_url: Option<&str>) -> Result<String> {
        // Simple date patterns - could be enhanced with chrono
        let patterns = [
            r"(\d{4})-(\d{2})-(\d{2})", // Already ISO
            r"(\d{1,2})/(\d{1,2})/(\d{4})", // MM/DD/YYYY
            r"(\d{1,2})\.(\d{1,2})\.(\d{4})", // DD.MM.YYYY
        ];

        for pattern in &patterns {
            let re = Regex::new(pattern).unwrap();
            if let Some(captures) = re.captures(content) {
                match *pattern {
                    r"(\d{4})-(\d{2})-(\d{2})" => return Ok(content.to_string()),
                    r"(\d{1,2})/(\d{1,2})/(\d{4})" => {
                        let month = captures.get(1).unwrap().as_str();
                        let day = captures.get(2).unwrap().as_str();
                        let year = captures.get(3).unwrap().as_str();
                        return Ok(format!("{}-{:02}-{:02}", year, month.parse::<u32>().unwrap(), day.parse::<u32>().unwrap()));
                    },
                    r"(\d{1,2})\.(\d{1,2})\.(\d{4})" => {
                        let day = captures.get(1).unwrap().as_str();
                        let month = captures.get(2).unwrap().as_str();
                        let year = captures.get(3).unwrap().as_str();
                        return Ok(format!("{}-{:02}-{:02}", year, month.parse::<u32>().unwrap(), day.parse::<u32>().unwrap()));
                    },
                    _ => {}
                }
            }
        }
        bail!("No recognizable date found in: {}", content)
    }
}

/// Transformer 6: Convert relative URLs to absolute
struct UrlAbsoluteTransformer;

impl ContentTransformer for UrlAbsoluteTransformer {
    fn transform(&self, content: &str, base_url: Option<&str>) -> Result<String> {
        if let Some(base) = base_url {
            let base_url = Url::parse(base).context("Invalid base URL")?;
            let url = base_url.join(content.trim()).context("Failed to join URL")?;
            Ok(url.to_string())
        } else {
            Ok(content.to_string())
        }
    }
}

/// Transformer 7: Convert to lowercase
struct LowercaseTransformer;

impl ContentTransformer for LowercaseTransformer {
    fn transform(&self, content: &str, _base_url: Option<&str>) -> Result<String> {
        Ok(content.to_lowercase())
    }
}

/// Transformer 8: Convert to uppercase
struct UppercaseTransformer;

impl ContentTransformer for UppercaseTransformer {
    fn transform(&self, content: &str, _base_url: Option<&str>) -> Result<String> {
        Ok(content.to_uppercase())
    }
}

/// Transformer 9: Split text by delimiter
struct SplitTransformer;

impl ContentTransformer for SplitTransformer {
    fn transform(&self, content: &str, _base_url: Option<&str>) -> Result<String> {
        // Default split by comma, could be parameterized
        let parts: Vec<&str> = content.split(',').map(|s| s.trim()).collect();
        serde_json::to_string(&parts).context("Failed to serialize split result")
    }
}

/// Transformer 10: Extract via regex pattern
struct RegexExtractTransformer;

impl ContentTransformer for RegexExtractTransformer {
    fn transform(&self, content: &str, _base_url: Option<&str>) -> Result<String> {
        // Example pattern - could be parameterized
        let re = Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap();
        if let Some(mat) = re.find(content) {
            Ok(mat.as_str().to_string())
        } else {
            bail!("No match found for regex pattern in: {}", content)
        }
    }
}

/// Transformer 11: Parse JSON strings
struct JsonParseTransformer;

impl ContentTransformer for JsonParseTransformer {
    fn transform(&self, content: &str, _base_url: Option<&str>) -> Result<String> {
        let parsed: serde_json::Value = serde_json::from_str(content.trim())
            .context("Failed to parse JSON")?;
        serde_json::to_string_pretty(&parsed)
            .context("Failed to serialize parsed JSON")
    }
}

/// Transformer 10: Join array elements into string
struct JoinTransformer;

impl ContentTransformer for JoinTransformer {
    fn transform(&self, content: &str, _base_url: Option<&str>) -> Result<String> {
        // Try to parse as JSON array, if successful join elements
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(content) {
            if let serde_json::Value::Array(arr) = parsed {
                let strings: Vec<String> = arr.into_iter()
                    .filter_map(|v| match v {
                        serde_json::Value::String(s) => Some(s),
                        serde_json::Value::Number(n) => Some(n.to_string()),
                        serde_json::Value::Bool(b) => Some(b.to_string()),
                        _ => None,
                    })
                    .collect();
                return Ok(strings.join(", "));
            }
        }

        // If not JSON, split by common delimiters and rejoin
        let parts: Vec<&str> = content.split(&[',', ';', '\n', '\t'][..])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        Ok(parts.join(", "))
    }
}

/// Transformer 11: Replace text using regex patterns
struct RegexReplaceTransformer;

impl ContentTransformer for RegexReplaceTransformer {
    fn transform(&self, content: &str, _base_url: Option<&str>) -> Result<String> {
        // Example: Remove all HTML tags - could be parameterized
        let re = Regex::new(r"<[^>]*>").unwrap();
        let cleaned = re.replace_all(content, "");

        // Also clean up extra whitespace
        let re_ws = Regex::new(r"\s+").unwrap();
        let normalized = re_ws.replace_all(cleaned.trim(), " ");

        Ok(normalized.to_string())
    }
}

/// Transformer 12: Decode HTML entities
struct HtmlDecodeTransformer;

impl ContentTransformer for HtmlDecodeTransformer {
    fn transform(&self, content: &str, _base_url: Option<&str>) -> Result<String> {
        // Enhanced HTML entity decoding
        let decoded = content
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&apos;", "'")
            .replace("&nbsp;", " ")
            .replace("&hellip;", "...")
            .replace("&mdash;", "—")
            .replace("&ndash;", "–")
            .replace("&copy;", "©")
            .replace("&reg;", "®")
            .replace("&trade;", "™");

        // Handle numeric character references
        let re_numeric = Regex::new(r"&#(\d+);").unwrap();
        let result = re_numeric.replace_all(&decoded, |caps: &regex::Captures| {
            if let Ok(code) = caps[1].parse::<u32>() {
                if let Some(ch) = char::from_u32(code) {
                    ch.to_string()
                } else {
                    caps[0].to_string() // Keep original if invalid
                }
            } else {
                caps[0].to_string()
            }
        });

        Ok(result.to_string())
    }
}

/// Default CSS selectors for common content types with enhanced configurations
pub fn default_selectors() -> HashMap<String, CssSelectorConfig> {
    let mut selectors = HashMap::new();

    selectors.insert("title".to_string(), CssSelectorConfig {
        selector: "title, h1, [property='og:title']".to_string(),
        transformers: vec!["trim".to_string(), "normalize_ws".to_string()],
        has_text_filter: None,
        fallbacks: vec!["h2".to_string(), ".title".to_string()],
        required: true,
        merge_policy: Some(MergePolicy::CssWins),
    });

    selectors.insert("description".to_string(), CssSelectorConfig {
        selector: "[name='description'], [property='og:description']".to_string(),
        transformers: vec!["trim".to_string(), "normalize_ws".to_string()],
        has_text_filter: None,
        fallbacks: vec![".description".to_string(), ".summary".to_string()],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    selectors.insert("content".to_string(), CssSelectorConfig {
        selector: "article, .content, .post-content, .entry-content, main".to_string(),
        transformers: vec!["trim".to_string()],
        has_text_filter: None,
        fallbacks: vec!["body p".to_string(), ".text".to_string()],
        required: true,
        merge_policy: Some(MergePolicy::CssWins),
    });

    selectors.insert("author".to_string(), CssSelectorConfig {
        selector: "[name='author'], .author, .byline, [rel='author']".to_string(),
        transformers: vec!["trim".to_string(), "normalize_ws".to_string()],
        has_text_filter: None,
        fallbacks: vec![".writer".to_string(), ".created-by".to_string()],
        required: false,
        merge_policy: Some(MergePolicy::FirstValid),
    });

    selectors.insert("date".to_string(), CssSelectorConfig {
        selector: "time, .date, .published, [property='article:published_time']".to_string(),
        transformers: vec!["trim".to_string(), "date_iso".to_string()],
        has_text_filter: None,
        fallbacks: vec![".timestamp".to_string(), ".publish-date".to_string()],
        required: false,
        merge_policy: Some(MergePolicy::CssWins),
    });

    selectors.insert("tags".to_string(), CssSelectorConfig {
        selector: ".tag, .category, .label, [property='article:tag']".to_string(),
        transformers: vec!["trim".to_string(), "lowercase".to_string()],
        has_text_filter: None,
        fallbacks: vec![".keywords".to_string(), ".topics".to_string()],
        required: false,
        merge_policy: Some(MergePolicy::Merge),
    });

    selectors
}

/// Backward compatibility function
pub fn default_selectors_simple() -> HashMap<String, String> {
    let enhanced = default_selectors();
    enhanced.into_iter().map(|(field, config)| {
        (field, config.selector)
    }).collect()
}

/// Common selectors for different content types
pub fn news_article_selectors() -> HashMap<String, String> {
    let mut selectors = HashMap::new();

    selectors.insert("title".to_string(), "h1, .headline, .article-title, [property='og:title']".to_string());
    selectors.insert("subtitle".to_string(), ".subtitle, .sub-headline, .article-subtitle".to_string());
    selectors.insert("author".to_string(), ".author, .byline, [rel='author'], .writer".to_string());
    selectors.insert("date".to_string(), "time, .date, .publish-date, [property='article:published_time']".to_string());
    selectors.insert("content".to_string(), ".article-body, .content, .post-content, article p".to_string());
    selectors.insert("summary".to_string(), ".summary, .excerpt, .lead, .article-summary".to_string());
    selectors.insert("category".to_string(), ".category, .section, .topic, [property='article:section']".to_string());
    selectors.insert("tags".to_string(), ".tag, .keyword, [property='article:tag']".to_string());

    selectors
}

/// Blog post selectors
pub fn blog_post_selectors() -> HashMap<String, String> {
    let mut selectors = HashMap::new();

    selectors.insert("title".to_string(), "h1, .post-title, .entry-title, [property='og:title']".to_string());
    selectors.insert("author".to_string(), ".author, .post-author, .entry-author".to_string());
    selectors.insert("date".to_string(), ".post-date, .entry-date, time".to_string());
    selectors.insert("content".to_string(), ".post-content, .entry-content, .blog-content".to_string());
    selectors.insert("excerpt".to_string(), ".excerpt, .post-excerpt, .summary".to_string());
    selectors.insert("category".to_string(), ".category, .post-category".to_string());
    selectors.insert("tags".to_string(), ".tag, .post-tag, .label".to_string());

    selectors
}

/// E-commerce product selectors
pub fn product_selectors() -> HashMap<String, String> {
    let mut selectors = HashMap::new();

    selectors.insert("title".to_string(), "h1, .product-title, .product-name".to_string());
    selectors.insert("price".to_string(), ".price, .product-price, .cost, .amount".to_string());
    selectors.insert("description".to_string(), ".description, .product-description, .details".to_string());
    selectors.insert("brand".to_string(), ".brand, .manufacturer, .product-brand".to_string());
    selectors.insert("model".to_string(), ".model, .product-model, .sku".to_string());
    selectors.insert("rating".to_string(), ".rating, .stars, .review-score".to_string());
    selectors.insert("availability".to_string(), ".availability, .stock, .in-stock".to_string());
    selectors.insert("images".to_string(), ".product-image img, .gallery img".to_string());

    selectors
}

/// Direct extraction function with custom selectors
pub async fn extract(html: &str, url: &str, selectors: &HashMap<String, String>) -> Result<ExtractedContent> {
    let extractor = CssJsonExtractor::from_simple_selectors(selectors.clone());
    extractor.extract(html, url).await
}

/// Direct extraction function with simple selectors (for backward compatibility)
pub async fn extract_simple(html: &str, url: &str, selectors: &HashMap<String, String>) -> Result<ExtractedContent> {
    extract(html, url, selectors).await
}

/// Direct extraction function with default selectors
pub async fn extract_default(html: &str, url: &str) -> Result<ExtractedContent> {
    extract(html, url, &default_selectors_simple()).await
}

/// Extract using news article selectors
pub async fn extract_news_article(html: &str, url: &str) -> Result<ExtractedContent> {
    extract(html, url, &news_article_selectors()).await
}

/// Extract using blog post selectors
pub async fn extract_blog_post(html: &str, url: &str) -> Result<ExtractedContent> {
    extract(html, url, &blog_post_selectors()).await
}

/// Extract using product selectors
pub async fn extract_product(html: &str, url: &str) -> Result<ExtractedContent> {
    extract(html, url, &product_selectors()).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_extraction() {
        let html = r#"
            <html>
                <head>
                    <title>Test Article</title>
                    <meta name="description" content="This is a test article">
                </head>
                <body>
                    <article>
                        <h1>Main Title</h1>
                        <p>This is the main content of the article.</p>
                        <p>Second paragraph with more content.</p>
                    </article>
                </body>
            </html>
        "#;

        let result = extract_default(html, "https://example.com").await.unwrap();

        assert_eq!(result.title, "Test Article");
        assert!(result.content.contains("This is the main content"));
        assert_eq!(result.summary, Some("This is a test article".to_string()));
        assert_eq!(result.strategy_used, "css_json_enhanced");
        assert!(result.extraction_confidence > 0.0);
    }

    #[tokio::test]
    async fn test_custom_selectors() {
        let html = r#"
            <html>
                <body>
                    <div class="custom-title">Custom Title</div>
                    <div class="custom-content">Custom content here</div>
                </body>
            </html>
        "#;

        let mut selectors = HashMap::new();
        selectors.insert("title".to_string(), ".custom-title".to_string());
        selectors.insert("content".to_string(), ".custom-content".to_string());

        let result = extract_simple(html, "https://example.com", &selectors).await.unwrap();

        assert_eq!(result.title, "Custom Title");
        assert_eq!(result.content, "Custom content here");
    }

    #[tokio::test]
    async fn test_confidence_score() {
        let extractor = CssJsonExtractor::new(default_selectors());

        let html_with_matches = r#"
            <html>
                <head><title>Test</title></head>
                <body><article>Content</article></body>
            </html>
        "#;

        let html_without_matches = r#"
            <html>
                <body><div>Some content</div></body>
            </html>
        "#;

        let score_with = extractor.confidence_score(html_with_matches);
        let score_without = extractor.confidence_score(html_without_matches);

        assert!(score_with > score_without);
    }
}

// Additional helper functions for Week 5 features

/// Create an enhanced CSS selector configuration builder
pub struct CssConfigBuilder {
    config: CssSelectorConfig,
}

impl CssConfigBuilder {
    pub fn new(selector: &str) -> Self {
        Self {
            config: CssSelectorConfig {
                selector: selector.to_string(),
                transformers: vec![],
                has_text_filter: None,
                fallbacks: vec![],
                required: false,
                merge_policy: None,
            },
        }
    }

    pub fn transform(mut self, transformer: &str) -> Self {
        self.config.transformers.push(transformer.to_string());
        self
    }

    pub fn transforms(mut self, transformers: &[&str]) -> Self {
        self.config.transformers.extend(transformers.iter().map(|t| t.to_string()));
        self
    }

    pub fn has_text(mut self, pattern: &str, case_insensitive: bool, partial: bool) -> Self {
        self.config.has_text_filter = Some(HasTextFilter {
            pattern: pattern.to_string(),
            case_insensitive,
            partial_match: partial,
            regex_mode: false,
            regex: None,
        });
        self
    }

    pub fn has_text_regex(mut self, pattern: &str, case_insensitive: bool) -> Self {
        let regex = if case_insensitive {
            Regex::new(&format!("(?i){}", pattern)).ok()
        } else {
            Regex::new(pattern).ok()
        };

        self.config.has_text_filter = Some(HasTextFilter {
            pattern: pattern.to_string(),
            case_insensitive,
            partial_match: true,
            regex_mode: true,
            regex,
        });
        self
    }

    pub fn fallback(mut self, selector: &str) -> Self {
        self.config.fallbacks.push(selector.to_string());
        self
    }

    pub fn fallbacks(mut self, selectors: &[&str]) -> Self {
        self.config.fallbacks.extend(selectors.iter().map(|s| s.to_string()));
        self
    }

    pub fn required(mut self) -> Self {
        self.config.required = true;
        self
    }

    pub fn merge_policy(mut self, policy: MergePolicy) -> Self {
        self.config.merge_policy = Some(policy);
        self
    }

    pub fn build(self) -> CssSelectorConfig {
        self.config
    }
}

/// Convenience function to create CSS config builder
pub fn css(selector: &str) -> CssConfigBuilder {
    CssConfigBuilder::new(selector)
}

/// Example usage functions for Week 5 features
pub mod examples {
    use super::*;

    /// Example: E-commerce product extraction with enhanced features
    pub fn ecommerce_selectors() -> HashMap<String, CssSelectorConfig> {
        let mut selectors = HashMap::new();

        // Product title with normalization
        selectors.insert("title".to_string(),
            css(".product-title, h1")
                .transforms(&["trim", "normalize_ws"])
                .fallbacks(&[".name", ".product-name"])
                .required()
                .merge_policy(MergePolicy::CssWins)
                .build()
        );

        // Price with currency extraction
        selectors.insert("price".to_string(),
            css(".price, .product-price")
                .transform("currency")
                .fallbacks(&[".cost", ".amount"])
                .merge_policy(MergePolicy::CssWins)
                .build()
        );

        // Reviews that mention "excellent"
        selectors.insert("excellent_reviews".to_string(),
            css(".review-text")
                .has_text("excellent", true, true)
                .transforms(&["trim", "normalize_ws"])
                .build()
        );

        selectors
    }

    /// Example: News article extraction with date parsing
    pub fn news_selectors() -> HashMap<String, CssSelectorConfig> {
        let mut selectors = HashMap::new();

        selectors.insert("headline".to_string(),
            css("h1, .headline")
                .transforms(&["trim", "normalize_ws"])
                .fallbacks(&["h2", ".title"])
                .required()
                .build()
        );

        selectors.insert("publish_date".to_string(),
            css("time, .date")
                .transforms(&["trim", "date_iso"])
                .fallbacks(&[".published", ".timestamp"])
                .build()
        );

        selectors.insert("breaking_news".to_string(),
            css(".article-content")
                .has_text("breaking", true, true)
                .transforms(&["trim"])
                .build()
        );

        selectors
    }
}