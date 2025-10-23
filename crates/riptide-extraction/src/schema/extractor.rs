//! Schema-based extraction implementation

use super::types::{ExtractionSchema, SelectorRule, TestResult};
use anyhow::Result;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::time::Instant;

/// Extracts data from HTML using a schema
pub struct SchemaExtractor {
    schema: ExtractionSchema,
}

impl SchemaExtractor {
    /// Create a new schema extractor
    pub fn new(schema: ExtractionSchema) -> Self {
        Self { schema }
    }

    /// Extract data from HTML content
    pub fn extract(&self, html: &str, _url: &str) -> Result<HashMap<String, serde_json::Value>> {
        let document = Html::parse_document(html);
        let mut extracted = HashMap::new();

        for (field_name, rules) in &self.schema.selectors {
            if let Some(value) = self.extract_field(&document, rules)? {
                extracted.insert(field_name.clone(), value);
            } else if let Some(field_schema) = self.schema.fields.get(field_name) {
                // Use default value if extraction failed
                if let Some(default) = &field_schema.default {
                    extracted.insert(field_name.clone(), default.clone());
                }
            }
        }

        Ok(extracted)
    }

    /// Extract a single field using its selector rules
    fn extract_field(
        &self,
        document: &Html,
        rules: &[SelectorRule],
    ) -> Result<Option<serde_json::Value>> {
        // Sort rules by priority (highest first)
        let mut sorted_rules = rules.to_vec();
        sorted_rules.sort_by(|a, b| b.priority.cmp(&a.priority));

        for rule in &sorted_rules {
            match self.apply_selector(document, rule) {
                Ok(Some(value)) => return Ok(Some(value)),
                Ok(None) => {
                    // Try fallback if available
                    if let Some(fallback) = &rule.fallback {
                        let fallback_rule = SelectorRule {
                            selector: fallback.clone(),
                            selector_type: rule.selector_type.clone(),
                            priority: rule.priority,
                            confidence: rule.confidence * 0.8, // Reduce confidence for fallback
                            fallback: None,
                        };
                        if let Ok(Some(value)) = self.apply_selector(document, &fallback_rule) {
                            return Ok(Some(value));
                        }
                    }
                }
                Err(_) => continue, // Try next rule
            }
        }

        Ok(None)
    }

    /// Apply a single selector to the document
    fn apply_selector(
        &self,
        document: &Html,
        rule: &SelectorRule,
    ) -> Result<Option<serde_json::Value>> {
        match rule.selector_type.as_str() {
            "css" => self.apply_css_selector(document, &rule.selector),
            "xpath" => {
                // XPath not fully supported yet, return None
                Ok(None)
            }
            "regex" => {
                // Regex extraction would go here
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    /// Apply CSS selector to document
    fn apply_css_selector(
        &self,
        document: &Html,
        selector_str: &str,
    ) -> Result<Option<serde_json::Value>> {
        let selector = Selector::parse(selector_str)
            .map_err(|e| anyhow::anyhow!("Invalid CSS selector '{}': {:?}", selector_str, e))?;

        let elements: Vec<_> = document.select(&selector).collect();

        if elements.is_empty() {
            return Ok(None);
        }

        // Extract text from first matching element
        let text = elements[0]
            .text()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string();

        if text.is_empty() {
            Ok(None)
        } else {
            Ok(Some(serde_json::Value::String(text)))
        }
    }

    /// Test the schema against a URL and return test result
    pub async fn test_extraction(&self, html: &str, url: &str) -> Result<TestResult> {
        let start = Instant::now();
        let extracted = self.extract(html, url)?;
        let extraction_time_ms = start.elapsed().as_millis() as u64;

        let fields_extracted = extracted.len() as u32;
        let mut missing_fields = Vec::new();
        let mut errors = Vec::new();

        // Check required fields
        for (field_name, field_schema) in &self.schema.fields {
            if field_schema.required && !extracted.contains_key(field_name) {
                missing_fields.push(field_name.clone());
            }
        }

        // Validate against validation rules
        let success = if let Some(validation) = &self.schema.validation {
            let mut valid = true;

            if let Some(min_fields) = validation.min_fields {
                if fields_extracted < min_fields {
                    errors.push(format!(
                        "Expected at least {} fields, got {}",
                        min_fields, fields_extracted
                    ));
                    valid = false;
                }
            }

            if let Some(required_fields) = &validation.required_fields {
                for field in required_fields {
                    if !extracted.contains_key(field) {
                        errors.push(format!("Required field '{}' not found", field));
                        valid = false;
                    }
                }
            }

            valid && missing_fields.is_empty()
        } else {
            missing_fields.is_empty()
        };

        // Calculate confidence based on field extraction success
        let total_fields = self.schema.fields.len() as f64;
        let confidence = if total_fields > 0.0 {
            fields_extracted as f64 / total_fields
        } else {
            0.0
        };

        Ok(TestResult {
            url: url.to_string(),
            success,
            confidence,
            fields_extracted,
            missing_fields,
            errors,
            extraction_time_ms,
        })
    }

    /// Get the schema being used
    pub fn schema(&self) -> &ExtractionSchema {
        &self.schema
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::types::{FieldSchema, SchemaMetadata};

    fn create_test_schema() -> ExtractionSchema {
        let mut schema = ExtractionSchema {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            goal: "article".to_string(),
            description: None,
            fields: HashMap::new(),
            selectors: HashMap::new(),
            validation: None,
            metadata: SchemaMetadata::default(),
        };

        schema
            .fields
            .insert("title".to_string(), FieldSchema::required("string"));

        schema
            .selectors
            .insert("title".to_string(), vec![SelectorRule::css("h1", 10, 0.9)]);

        schema
    }

    #[test]
    fn test_extractor_creation() {
        let schema = create_test_schema();
        let extractor = SchemaExtractor::new(schema.clone());
        assert_eq!(extractor.schema().name, "test");
    }

    #[test]
    fn test_css_extraction() {
        let schema = create_test_schema();
        let extractor = SchemaExtractor::new(schema);

        let html = r#"<html><head></head><body><h1>Test Title</h1></body></html>"#;
        let result = extractor.extract(html, "http://example.com").unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result.get("title").unwrap().as_str().unwrap(), "Test Title");
    }

    #[test]
    fn test_missing_field_extraction() {
        let schema = create_test_schema();
        let extractor = SchemaExtractor::new(schema);

        let html = r#"<html><head></head><body><p>No title here</p></body></html>"#;
        let result = extractor.extract(html, "http://example.com").unwrap();

        // Should not contain title as h1 is missing
        assert_eq!(result.len(), 0);
    }

    #[tokio::test]
    async fn test_extraction_with_test_result() {
        let schema = create_test_schema();
        let extractor = SchemaExtractor::new(schema);

        let html = r#"<html><head></head><body><h1>Test Title</h1></body></html>"#;
        let result = extractor
            .test_extraction(html, "http://example.com")
            .await
            .unwrap();

        assert!(result.success);
        assert_eq!(result.fields_extracted, 1);
        assert!(result.missing_fields.is_empty());
        assert!(result.confidence > 0.9);
    }
}
