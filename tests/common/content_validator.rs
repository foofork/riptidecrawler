/// Content Validation Framework for CLI Testing
///
/// Validates extraction results against expected patterns, keywords, and quality metrics.
/// Ensures CLI returns correct content, not just successful status codes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub name: String,
    pub rule_type: RuleType,
    pub threshold: Option<f64>,
    pub expected_value: Option<serde_json::Value>,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleType {
    ContentLength { min: usize, max: Option<usize> },
    KeywordPresence { keywords: Vec<String>, min_matches: usize },
    QualityScore { min: f64 },
    MetadataField { field: String, required: bool },
    ExtractionTime { max_ms: u64 },
    TitlePresence,
    ImagePresence,
    LinkPresence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub passed: bool,
    pub rule_name: String,
    pub message: String,
    pub actual_value: Option<serde_json::Value>,
    pub expected_value: Option<serde_json::Value>,
}

pub struct ContentValidator {
    rules: Vec<ValidationRule>,
}

impl ContentValidator {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
        }
    }

    pub fn with_rules(rules: Vec<ValidationRule>) -> Self {
        Self { rules }
    }

    pub fn add_rule(&mut self, rule: ValidationRule) {
        self.rules.push(rule);
    }

    /// Validate extracted content against all configured rules
    pub fn validate(
        &self,
        content: &str,
        metadata: &HashMap<String, serde_json::Value>,
        duration_ms: u64,
    ) -> Vec<ValidationResult> {
        let mut results = Vec::new();

        for rule in &self.rules {
            let result = self.validate_rule(rule, content, metadata, duration_ms);
            results.push(result);
        }

        results
    }

    fn validate_rule(
        &self,
        rule: &ValidationRule,
        content: &str,
        metadata: &HashMap<String, serde_json::Value>,
        duration_ms: u64,
    ) -> ValidationResult {
        match &rule.rule_type {
            RuleType::ContentLength { min, max } => {
                self.validate_content_length(rule, content, *min, *max)
            }
            RuleType::KeywordPresence { keywords, min_matches } => {
                self.validate_keywords(rule, content, keywords, *min_matches)
            }
            RuleType::QualityScore { min } => {
                self.validate_quality_score(rule, metadata, *min)
            }
            RuleType::MetadataField { field, required } => {
                self.validate_metadata_field(rule, metadata, field, *required)
            }
            RuleType::ExtractionTime { max_ms } => {
                self.validate_extraction_time(rule, duration_ms, *max_ms)
            }
            RuleType::TitlePresence => {
                self.validate_title_presence(rule, content, metadata)
            }
            RuleType::ImagePresence => {
                self.validate_image_presence(rule, metadata)
            }
            RuleType::LinkPresence => {
                self.validate_link_presence(rule, content)
            }
        }
    }

    fn validate_content_length(
        &self,
        rule: &ValidationRule,
        content: &str,
        min: usize,
        max: Option<usize>,
    ) -> ValidationResult {
        let length = content.len();
        let passed = if let Some(max_val) = max {
            length >= min && length <= max_val
        } else {
            length >= min
        };

        ValidationResult {
            passed,
            rule_name: rule.name.clone(),
            message: if passed {
                format!("Content length {} is within range", length)
            } else {
                format!(
                    "Content length {} outside range (min: {}, max: {:?})",
                    length, min, max
                )
            },
            actual_value: Some(serde_json::json!(length)),
            expected_value: Some(serde_json::json!({
                "min": min,
                "max": max
            })),
        }
    }

    fn validate_keywords(
        &self,
        rule: &ValidationRule,
        content: &str,
        keywords: &[String],
        min_matches: usize,
    ) -> ValidationResult {
        let content_lower = content.to_lowercase();
        let matches: Vec<String> = keywords
            .iter()
            .filter(|k| content_lower.contains(&k.to_lowercase()))
            .cloned()
            .collect();

        let passed = matches.len() >= min_matches;

        ValidationResult {
            passed,
            rule_name: rule.name.clone(),
            message: if passed {
                format!("Found {}/{} required keywords: {:?}", matches.len(), min_matches, matches)
            } else {
                format!(
                    "Only found {}/{} required keywords. Expected: {:?}, Found: {:?}",
                    matches.len(), min_matches, keywords, matches
                )
            },
            actual_value: Some(serde_json::json!(matches)),
            expected_value: Some(serde_json::json!({
                "keywords": keywords,
                "min_matches": min_matches
            })),
        }
    }

    fn validate_quality_score(
        &self,
        rule: &ValidationRule,
        metadata: &HashMap<String, serde_json::Value>,
        min: f64,
    ) -> ValidationResult {
        let score = metadata
            .get("quality_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let passed = score >= min;

        ValidationResult {
            passed,
            rule_name: rule.name.clone(),
            message: if passed {
                format!("Quality score {:.2} meets minimum {:.2}", score, min)
            } else {
                format!("Quality score {:.2} below minimum {:.2}", score, min)
            },
            actual_value: Some(serde_json::json!(score)),
            expected_value: Some(serde_json::json!(min)),
        }
    }

    fn validate_metadata_field(
        &self,
        rule: &ValidationRule,
        metadata: &HashMap<String, serde_json::Value>,
        field: &str,
        required: bool,
    ) -> ValidationResult {
        let exists = metadata.contains_key(field);
        let passed = !required || exists;

        ValidationResult {
            passed,
            rule_name: rule.name.clone(),
            message: if passed {
                if exists {
                    format!("Metadata field '{}' present", field)
                } else {
                    format!("Optional metadata field '{}' not present", field)
                }
            } else {
                format!("Required metadata field '{}' missing", field)
            },
            actual_value: metadata.get(field).cloned(),
            expected_value: Some(serde_json::json!({
                "field": field,
                "required": required
            })),
        }
    }

    fn validate_extraction_time(
        &self,
        rule: &ValidationRule,
        duration_ms: u64,
        max_ms: u64,
    ) -> ValidationResult {
        let passed = duration_ms <= max_ms;

        ValidationResult {
            passed,
            rule_name: rule.name.clone(),
            message: if passed {
                format!("Extraction time {}ms within limit {}ms", duration_ms, max_ms)
            } else {
                format!("Extraction time {}ms exceeds limit {}ms", duration_ms, max_ms)
            },
            actual_value: Some(serde_json::json!(duration_ms)),
            expected_value: Some(serde_json::json!(max_ms)),
        }
    }

    fn validate_title_presence(
        &self,
        rule: &ValidationRule,
        content: &str,
        metadata: &HashMap<String, serde_json::Value>,
    ) -> ValidationResult {
        let has_title = metadata.contains_key("title")
            || content.contains("<title>")
            || content.contains("# "); // Markdown title

        ValidationResult {
            passed: has_title,
            rule_name: rule.name.clone(),
            message: if has_title {
                "Title found in content or metadata".to_string()
            } else {
                "No title found in content or metadata".to_string()
            },
            actual_value: Some(serde_json::json!(has_title)),
            expected_value: Some(serde_json::json!(true)),
        }
    }

    fn validate_image_presence(
        &self,
        rule: &ValidationRule,
        metadata: &HashMap<String, serde_json::Value>,
    ) -> ValidationResult {
        let has_images = metadata
            .get("has_images")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        ValidationResult {
            passed: has_images,
            rule_name: rule.name.clone(),
            message: if has_images {
                "Images found in content".to_string()
            } else {
                "No images found in content".to_string()
            },
            actual_value: Some(serde_json::json!(has_images)),
            expected_value: Some(serde_json::json!(true)),
        }
    }

    fn validate_link_presence(
        &self,
        rule: &ValidationRule,
        content: &str,
    ) -> ValidationResult {
        let has_links = content.contains("http://") || content.contains("https://");

        ValidationResult {
            passed: has_links,
            rule_name: rule.name.clone(),
            message: if has_links {
                "Links found in content".to_string()
            } else {
                "No links found in content".to_string()
            },
            actual_value: Some(serde_json::json!(has_links)),
            expected_value: Some(serde_json::json!(true)),
        }
    }

    /// Create a validator with common validation rules
    pub fn create_default(expected: &HashMap<String, serde_json::Value>) -> Self {
        let mut rules = Vec::new();

        // Content length validation
        if let Some(min_length) = expected.get("min_content_length").and_then(|v| v.as_u64()) {
            rules.push(ValidationRule {
                name: "content_length".to_string(),
                rule_type: RuleType::ContentLength {
                    min: min_length as usize,
                    max: None,
                },
                threshold: None,
                expected_value: Some(serde_json::json!(min_length)),
                required: true,
            });
        }

        // Title presence
        if expected.get("has_title").and_then(|v| v.as_bool()).unwrap_or(false) {
            rules.push(ValidationRule {
                name: "title_presence".to_string(),
                rule_type: RuleType::TitlePresence,
                threshold: None,
                expected_value: Some(serde_json::json!(true)),
                required: true,
            });
        }

        // Image presence
        if expected.get("has_images").and_then(|v| v.as_bool()).unwrap_or(false) {
            rules.push(ValidationRule {
                name: "image_presence".to_string(),
                rule_type: RuleType::ImagePresence,
                threshold: None,
                expected_value: Some(serde_json::json!(true)),
                required: false,
            });
        }

        // Quality score
        if let Some(min_quality) = expected.get("min_quality_score").and_then(|v| v.as_f64()) {
            rules.push(ValidationRule {
                name: "quality_score".to_string(),
                rule_type: RuleType::QualityScore { min: min_quality },
                threshold: Some(min_quality),
                expected_value: Some(serde_json::json!(min_quality)),
                required: true,
            });
        }

        // Extraction time
        if let Some(max_time) = expected.get("max_extraction_time_ms").and_then(|v| v.as_u64()) {
            rules.push(ValidationRule {
                name: "extraction_time".to_string(),
                rule_type: RuleType::ExtractionTime { max_ms: max_time },
                threshold: Some(max_time as f64),
                expected_value: Some(serde_json::json!(max_time)),
                required: false,
            });
        }

        Self::with_rules(rules)
    }
}

impl Default for ContentValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_length_validation() {
        let mut validator = ContentValidator::new();
        validator.add_rule(ValidationRule {
            name: "length_check".to_string(),
            rule_type: RuleType::ContentLength { min: 100, max: Some(1000) },
            threshold: None,
            expected_value: None,
            required: true,
        });

        let content = "A".repeat(500);
        let metadata = HashMap::new();
        let results = validator.validate(&content, &metadata, 100);

        assert_eq!(results.len(), 1);
        assert!(results[0].passed);
    }

    #[test]
    fn test_keyword_validation() {
        let mut validator = ContentValidator::new();
        validator.add_rule(ValidationRule {
            name: "keyword_check".to_string(),
            rule_type: RuleType::KeywordPresence {
                keywords: vec!["rust".to_string(), "programming".to_string()],
                min_matches: 2,
            },
            threshold: None,
            expected_value: None,
            required: true,
        });

        let content = "Rust is a great programming language";
        let metadata = HashMap::new();
        let results = validator.validate(content, &metadata, 100);

        assert_eq!(results.len(), 1);
        assert!(results[0].passed);
    }

    #[test]
    fn test_quality_score_validation() {
        let mut validator = ContentValidator::new();
        validator.add_rule(ValidationRule {
            name: "quality_check".to_string(),
            rule_type: RuleType::QualityScore { min: 0.8 },
            threshold: Some(0.8),
            expected_value: None,
            required: true,
        });

        let mut metadata = HashMap::new();
        metadata.insert("quality_score".to_string(), serde_json::json!(0.9));
        let results = validator.validate("test content", &metadata, 100);

        assert_eq!(results.len(), 1);
        assert!(results[0].passed);
    }
}
