//! Schema type definitions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete extraction schema with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionSchema {
    pub name: String,
    pub version: String,
    pub goal: String,
    pub description: Option<String>,
    pub fields: HashMap<String, FieldSchema>,
    pub selectors: HashMap<String, Vec<SelectorRule>>,
    pub validation: Option<ValidationRules>,
    pub metadata: SchemaMetadata,
}

/// Field schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSchema {
    pub field_type: String,
    pub required: bool,
    pub description: Option<String>,
    pub default: Option<serde_json::Value>,
    pub transform: Option<String>,
}

/// Selector rule with priority and confidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectorRule {
    pub selector: String,
    pub selector_type: String, // css, xpath, regex
    pub priority: u32,
    pub confidence: f64,
    pub fallback: Option<String>,
}

/// Validation rules for extracted data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRules {
    pub min_fields: Option<u32>,
    pub required_fields: Option<Vec<String>>,
    pub min_confidence: Option<f64>,
    pub custom_rules: Option<Vec<String>>,
}

/// Schema metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaMetadata {
    pub created_at: String,
    pub updated_at: String,
    pub author: Option<String>,
    pub tags: Vec<String>,
    pub is_public: bool,
    pub usage_count: u64,
    pub success_rate: Option<f64>,
}

/// Request for learning a schema from a URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaLearnRequest {
    pub url: String,
    pub goal: String,
    pub confidence_threshold: f64,
    pub fields: Option<Vec<String>>,
    pub verbose: bool,
}

/// Response from schema learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaLearnResponse {
    pub schema: ExtractionSchema,
    pub analysis: SchemaAnalysis,
    pub suggestions: Vec<String>,
}

/// Analysis of a learned schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaAnalysis {
    pub confidence: f64,
    pub fields_detected: u32,
    pub selectors_generated: u32,
    pub patterns_found: Vec<String>,
    pub warnings: Vec<String>,
}

/// Request for testing a schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaTestRequest {
    pub schema: ExtractionSchema,
    pub urls: Vec<String>,
    pub fail_fast: bool,
}

/// Response from schema testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaTestResponse {
    pub total_tests: u32,
    pub passed: u32,
    pub failed: u32,
    pub success_rate: f64,
    pub results: Vec<TestResult>,
    pub summary: TestSummary,
}

/// Result of testing a schema against a URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub url: String,
    pub success: bool,
    pub confidence: f64,
    pub fields_extracted: u32,
    pub missing_fields: Vec<String>,
    pub errors: Vec<String>,
    pub extraction_time_ms: u64,
}

/// Summary of test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSummary {
    pub avg_confidence: f64,
    pub avg_extraction_time_ms: u64,
    pub most_common_errors: Vec<String>,
    pub field_success_rates: HashMap<String, f64>,
}

impl ExtractionSchema {
    /// Create a new schema with default metadata
    pub fn new(name: String, version: String, goal: String) -> Self {
        Self {
            name,
            version,
            goal,
            description: None,
            fields: HashMap::new(),
            selectors: HashMap::new(),
            validation: None,
            metadata: SchemaMetadata::default(),
        }
    }

    /// Add a field to the schema
    pub fn add_field(&mut self, name: String, field: FieldSchema) {
        self.fields.insert(name, field);
    }

    /// Add a selector rule for a field
    pub fn add_selector(&mut self, field: String, rule: SelectorRule) {
        self.selectors.entry(field).or_default().push(rule);
    }

    /// Set validation rules
    pub fn set_validation(&mut self, validation: ValidationRules) {
        self.validation = Some(validation);
    }
}

impl Default for SchemaMetadata {
    fn default() -> Self {
        Self {
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            author: None,
            tags: Vec::new(),
            is_public: false,
            usage_count: 0,
            success_rate: None,
        }
    }
}

impl FieldSchema {
    /// Create a new required field
    pub fn required(field_type: impl Into<String>) -> Self {
        Self {
            field_type: field_type.into(),
            required: true,
            description: None,
            default: None,
            transform: None,
        }
    }

    /// Create a new optional field
    pub fn optional(field_type: impl Into<String>) -> Self {
        Self {
            field_type: field_type.into(),
            required: false,
            description: None,
            default: None,
            transform: None,
        }
    }

    /// Set field description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set default value
    pub fn with_default(mut self, default: serde_json::Value) -> Self {
        self.default = Some(default);
        self
    }
}

impl SelectorRule {
    /// Create a new CSS selector rule
    pub fn css(selector: impl Into<String>, priority: u32, confidence: f64) -> Self {
        Self {
            selector: selector.into(),
            selector_type: "css".to_string(),
            priority,
            confidence,
            fallback: None,
        }
    }

    /// Create a new XPath selector rule
    pub fn xpath(selector: impl Into<String>, priority: u32, confidence: f64) -> Self {
        Self {
            selector: selector.into(),
            selector_type: "xpath".to_string(),
            priority,
            confidence,
            fallback: None,
        }
    }

    /// Set fallback selector
    pub fn with_fallback(mut self, fallback: impl Into<String>) -> Self {
        self.fallback = Some(fallback.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_creation() {
        let schema = ExtractionSchema::new(
            "test".to_string(),
            "1.0.0".to_string(),
            "article".to_string(),
        );
        assert_eq!(schema.name, "test");
        assert_eq!(schema.version, "1.0.0");
        assert_eq!(schema.goal, "article");
        assert!(schema.fields.is_empty());
    }

    #[test]
    fn test_field_schema_builders() {
        let required = FieldSchema::required("string").with_description("Test field");
        assert!(required.required);
        assert_eq!(required.field_type, "string");
        assert_eq!(required.description, Some("Test field".to_string()));

        let optional = FieldSchema::optional("number");
        assert!(!optional.required);
    }

    #[test]
    fn test_selector_rule_builders() {
        let css = SelectorRule::css("h1.title", 10, 0.9).with_fallback("h1");
        assert_eq!(css.selector_type, "css");
        assert_eq!(css.priority, 10);
        assert_eq!(css.confidence, 0.9);
        assert_eq!(css.fallback, Some("h1".to_string()));

        let xpath = SelectorRule::xpath("//h1[@class='title']", 5, 0.8);
        assert_eq!(xpath.selector_type, "xpath");
    }

    #[test]
    fn test_schema_mutations() {
        let mut schema = ExtractionSchema::new(
            "test".to_string(),
            "1.0.0".to_string(),
            "article".to_string(),
        );

        schema.add_field("title".to_string(), FieldSchema::required("string"));
        assert_eq!(schema.fields.len(), 1);

        schema.add_selector("title".to_string(), SelectorRule::css("h1", 10, 0.9));
        assert_eq!(schema.selectors.len(), 1);
        assert_eq!(schema.selectors["title"].len(), 1);
    }
}
