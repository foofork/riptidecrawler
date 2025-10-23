//! Schema validation functionality

use super::extractor::SchemaExtractor;
use super::types::{
    ExtractionSchema, SchemaTestRequest, SchemaTestResponse, TestResult, TestSummary,
};
use anyhow::Result;
use std::collections::HashMap;

/// Validates schemas and extraction results
pub struct SchemaValidator;

impl SchemaValidator {
    /// Create a new schema validator
    pub fn new() -> Self {
        Self
    }

    /// Test a schema against multiple URLs
    pub async fn test_schema(
        &self,
        request: SchemaTestRequest,
        html_fetcher: impl Fn(&str) -> Result<String>,
    ) -> Result<SchemaTestResponse> {
        let extractor = SchemaExtractor::new(request.schema.clone());
        let mut results = Vec::new();
        let mut passed = 0;
        let mut failed = 0;

        for url in &request.urls {
            // Fetch HTML (in real implementation, would use HTTP client)
            let html = html_fetcher(url)?;

            match extractor.test_extraction(&html, url).await {
                Ok(result) => {
                    if result.success {
                        passed += 1;
                    } else {
                        failed += 1;
                        if request.fail_fast {
                            results.push(result);
                            break;
                        }
                    }
                    results.push(result);
                }
                Err(e) => {
                    failed += 1;
                    results.push(TestResult {
                        url: url.clone(),
                        success: false,
                        confidence: 0.0,
                        fields_extracted: 0,
                        missing_fields: Vec::new(),
                        errors: vec![e.to_string()],
                        extraction_time_ms: 0,
                    });

                    if request.fail_fast {
                        break;
                    }
                }
            }
        }

        let total_tests = results.len() as u32;
        let success_rate = if total_tests > 0 {
            passed as f64 / total_tests as f64
        } else {
            0.0
        };

        let summary = self.generate_summary(&results);

        Ok(SchemaTestResponse {
            total_tests,
            passed,
            failed,
            success_rate,
            results,
            summary,
        })
    }

    /// Generate summary statistics from test results
    fn generate_summary(&self, results: &[TestResult]) -> TestSummary {
        if results.is_empty() {
            return TestSummary {
                avg_confidence: 0.0,
                avg_extraction_time_ms: 0,
                most_common_errors: Vec::new(),
                field_success_rates: HashMap::new(),
            };
        }

        // Calculate averages
        let total_confidence: f64 = results.iter().map(|r| r.confidence).sum();
        let avg_confidence = total_confidence / results.len() as f64;

        let total_time: u64 = results.iter().map(|r| r.extraction_time_ms).sum();
        let avg_extraction_time_ms = total_time / results.len() as u64;

        // Collect error frequencies
        let mut error_counts: HashMap<String, usize> = HashMap::new();
        for result in results {
            for error in &result.errors {
                *error_counts.entry(error.clone()).or_insert(0) += 1;
            }
        }

        let mut most_common_errors: Vec<(String, usize)> = error_counts.into_iter().collect();
        most_common_errors.sort_by(|a, b| b.1.cmp(&a.1));
        let most_common_errors: Vec<String> = most_common_errors
            .into_iter()
            .take(5)
            .map(|(error, _)| error)
            .collect();

        // Calculate field success rates
        let mut field_attempts: HashMap<String, usize> = HashMap::new();

        for result in results {
            for field in &result.missing_fields {
                *field_attempts.entry(field.clone()).or_insert(0) += 1;
            }
        }

        let field_success_rates: HashMap<String, f64> = field_attempts
            .iter()
            .map(|(field, attempts)| {
                let successes = results.len() - attempts;
                let rate = successes as f64 / results.len() as f64;
                (field.clone(), rate)
            })
            .collect();

        TestSummary {
            avg_confidence,
            avg_extraction_time_ms,
            most_common_errors,
            field_success_rates,
        }
    }

    /// Validate schema structure and completeness
    pub fn validate_schema_structure(&self, schema: &ExtractionSchema) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // Check for empty fields
        if schema.fields.is_empty() {
            warnings.push("Schema has no fields defined".to_string());
        }

        // Check for fields without selectors
        for field_name in schema.fields.keys() {
            if !schema.selectors.contains_key(field_name) {
                warnings.push(format!("Field '{}' has no selectors defined", field_name));
            }
        }

        // Check for selectors without fields
        for selector_field in schema.selectors.keys() {
            if !schema.fields.contains_key(selector_field) {
                warnings.push(format!(
                    "Selector for field '{}' has no corresponding field definition",
                    selector_field
                ));
            }
        }

        // Check selector confidence
        for (field, rules) in &schema.selectors {
            for rule in rules {
                if rule.confidence < 0.5 {
                    warnings.push(format!(
                        "Field '{}' has selector with low confidence: {}",
                        field, rule.confidence
                    ));
                }
            }
        }

        // Check required fields have good selectors
        for (field_name, field) in &schema.fields {
            if field.required {
                if let Some(rules) = schema.selectors.get(field_name) {
                    let max_confidence = rules.iter().map(|r| r.confidence).fold(0.0, f64::max);
                    if max_confidence < 0.7 {
                        warnings.push(format!(
                            "Required field '{}' has no high-confidence selectors",
                            field_name
                        ));
                    }
                }
            }
        }

        Ok(warnings)
    }
}

impl Default for SchemaValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::types::{FieldSchema, SchemaMetadata, SelectorRule};

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
    fn test_validator_creation() {
        let _validator = SchemaValidator::new();
        // Just ensure it can be created
    }

    #[test]
    fn test_schema_structure_validation() {
        let validator = SchemaValidator::new();
        let schema = create_test_schema();

        let warnings = validator.validate_schema_structure(&schema).unwrap();
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_empty_schema_validation() {
        let validator = SchemaValidator::new();
        let schema = ExtractionSchema::new(
            "test".to_string(),
            "1.0.0".to_string(),
            "article".to_string(),
        );

        let warnings = validator.validate_schema_structure(&schema).unwrap();
        assert!(!warnings.is_empty());
        assert!(warnings[0].contains("no fields"));
    }

    #[test]
    fn test_low_confidence_warning() {
        let validator = SchemaValidator::new();
        let mut schema = create_test_schema();

        // Add low confidence selector
        schema.selectors.get_mut("title").unwrap()[0].confidence = 0.3;

        let warnings = validator.validate_schema_structure(&schema).unwrap();
        assert!(warnings.iter().any(|w| w.contains("low confidence")));
    }
}
