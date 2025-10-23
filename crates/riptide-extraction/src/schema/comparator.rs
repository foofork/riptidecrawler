//! Schema comparison functionality

use super::types::ExtractionSchema;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Compares two schemas and identifies differences
pub struct SchemaComparator;

/// Comparison result containing differences and similarities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaComparison {
    pub schema1_name: String,
    pub schema2_name: String,
    pub differences: Vec<String>,
    pub similarities: Vec<String>,
    pub added_fields: Vec<String>,
    pub removed_fields: Vec<String>,
    pub modified_fields: Vec<String>,
    pub common_fields: Vec<String>,
}

impl SchemaComparator {
    /// Create a new schema comparator
    pub fn new() -> Self {
        Self
    }

    /// Compare two schemas
    pub fn compare(
        &self,
        schema1: &ExtractionSchema,
        schema2: &ExtractionSchema,
    ) -> Result<SchemaComparison> {
        let mut differences = Vec::new();
        let mut similarities = Vec::new();

        // Compare metadata
        if schema1.name != schema2.name {
            differences.push(format!("Name: '{}' → '{}'", schema1.name, schema2.name));
        } else {
            similarities.push(format!("Name: {}", schema1.name));
        }

        if schema1.version != schema2.version {
            differences.push(format!(
                "Version: '{}' → '{}'",
                schema1.version, schema2.version
            ));
        } else {
            similarities.push(format!("Version: {}", schema1.version));
        }

        if schema1.goal != schema2.goal {
            differences.push(format!("Goal: '{}' → '{}'", schema1.goal, schema2.goal));
        } else {
            similarities.push(format!("Goal: {}", schema1.goal));
        }

        // Compare fields
        let fields1: HashSet<_> = schema1.fields.keys().collect();
        let fields2: HashSet<_> = schema2.fields.keys().collect();

        let added_fields: Vec<String> = fields2
            .difference(&fields1)
            .map(|f| f.to_string())
            .collect();

        let removed_fields: Vec<String> = fields1
            .difference(&fields2)
            .map(|f| f.to_string())
            .collect();

        let common_fields: Vec<String> = fields1
            .intersection(&fields2)
            .map(|f| f.to_string())
            .collect();

        // Check for modified fields
        let mut modified_fields = Vec::new();
        for field_name in &common_fields {
            let field1 = &schema1.fields[field_name];
            let field2 = &schema2.fields[field_name];

            if field1.field_type != field2.field_type || field1.required != field2.required {
                modified_fields.push(field_name.clone());
                differences.push(format!("Field '{}' was modified", field_name));
            }
        }

        if !added_fields.is_empty() {
            differences.push(format!("Added fields: {}", added_fields.join(", ")));
        }

        if !removed_fields.is_empty() {
            differences.push(format!("Removed fields: {}", removed_fields.join(", ")));
        }

        // Compare selectors
        self.compare_selectors(schema1, schema2, &mut differences, &mut similarities);

        Ok(SchemaComparison {
            schema1_name: schema1.name.clone(),
            schema2_name: schema2.name.clone(),
            differences,
            similarities,
            added_fields,
            removed_fields,
            modified_fields,
            common_fields,
        })
    }

    /// Compare selectors between two schemas
    fn compare_selectors(
        &self,
        schema1: &ExtractionSchema,
        schema2: &ExtractionSchema,
        differences: &mut Vec<String>,
        _similarities: &mut Vec<String>,
    ) {
        for field in schema1.fields.keys() {
            let selectors1 = schema1.selectors.get(field);
            let selectors2 = schema2.selectors.get(field);

            match (selectors1, selectors2) {
                (Some(s1), Some(s2)) => {
                    if s1.len() != s2.len() {
                        differences.push(format!(
                            "Field '{}': selector count changed from {} to {}",
                            field,
                            s1.len(),
                            s2.len()
                        ));
                    }
                }
                (Some(_), None) => {
                    differences.push(format!("Field '{}': selectors removed", field));
                }
                (None, Some(_)) => {
                    differences.push(format!("Field '{}': selectors added", field));
                }
                (None, None) => {}
            }
        }
    }

    /// Generate a text report of the comparison
    pub fn generate_text_report(&self, comparison: &SchemaComparison) -> String {
        let mut report = String::new();

        report.push_str(&format!(
            "Comparing: {} vs {}\n\n",
            comparison.schema1_name, comparison.schema2_name
        ));

        if !comparison.differences.is_empty() {
            report.push_str("DIFFERENCES:\n");
            for diff in &comparison.differences {
                report.push_str(&format!("  ✗ {}\n", diff));
            }
            report.push('\n');
        }

        if !comparison.similarities.is_empty() {
            report.push_str("SIMILARITIES:\n");
            for sim in &comparison.similarities {
                report.push_str(&format!("  ✓ {}\n", sim));
            }
            report.push('\n');
        }

        report.push_str("SUMMARY:\n");
        report.push_str(&format!(
            "  Total Differences: {}\n",
            comparison.differences.len()
        ));
        report.push_str(&format!(
            "  Fields Added: {}\n",
            comparison.added_fields.len()
        ));
        report.push_str(&format!(
            "  Fields Removed: {}\n",
            comparison.removed_fields.len()
        ));
        report.push_str(&format!(
            "  Fields Modified: {}\n",
            comparison.modified_fields.len()
        ));
        report.push_str(&format!(
            "  Fields Common: {}\n",
            comparison.common_fields.len()
        ));

        report
    }

    /// Generate a JSON report of the comparison
    pub fn generate_json_report(&self, comparison: &SchemaComparison) -> Result<String> {
        Ok(serde_json::to_string_pretty(comparison)?)
    }

    /// Generate a table-formatted report
    pub fn generate_table_report(&self, comparison: &SchemaComparison) -> Vec<Vec<String>> {
        vec![
            vec!["Category".to_string(), "Value".to_string()],
            vec![
                "Schemas".to_string(),
                format!("{} vs {}", comparison.schema1_name, comparison.schema2_name),
            ],
            vec![
                "Differences".to_string(),
                comparison.differences.len().to_string(),
            ],
            vec![
                "Fields Added".to_string(),
                comparison.added_fields.len().to_string(),
            ],
            vec![
                "Fields Removed".to_string(),
                comparison.removed_fields.len().to_string(),
            ],
            vec![
                "Fields Modified".to_string(),
                comparison.modified_fields.len().to_string(),
            ],
            vec![
                "Fields Common".to_string(),
                comparison.common_fields.len().to_string(),
            ],
        ]
    }
}

impl Default for SchemaComparator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::types::{FieldSchema, SchemaMetadata};

    fn create_schema(name: &str, fields: Vec<&str>) -> ExtractionSchema {
        let mut schema = ExtractionSchema {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            goal: "test".to_string(),
            description: None,
            fields: HashMap::new(),
            selectors: HashMap::new(),
            validation: None,
            metadata: SchemaMetadata::default(),
        };

        for field in fields {
            schema
                .fields
                .insert(field.to_string(), FieldSchema::required("string"));
        }

        schema
    }

    #[test]
    fn test_identical_schemas() {
        let comparator = SchemaComparator::new();
        let schema1 = create_schema("test", vec!["field1", "field2"]);
        let schema2 = create_schema("test", vec!["field1", "field2"]);

        let comparison = comparator.compare(&schema1, &schema2).unwrap();

        assert_eq!(comparison.added_fields.len(), 0);
        assert_eq!(comparison.removed_fields.len(), 0);
        assert_eq!(comparison.common_fields.len(), 2);
    }

    #[test]
    fn test_added_fields() {
        let comparator = SchemaComparator::new();
        let schema1 = create_schema("test", vec!["field1"]);
        let schema2 = create_schema("test", vec!["field1", "field2"]);

        let comparison = comparator.compare(&schema1, &schema2).unwrap();

        assert_eq!(comparison.added_fields.len(), 1);
        assert!(comparison.added_fields.contains(&"field2".to_string()));
    }

    #[test]
    fn test_removed_fields() {
        let comparator = SchemaComparator::new();
        let schema1 = create_schema("test", vec!["field1", "field2"]);
        let schema2 = create_schema("test", vec!["field1"]);

        let comparison = comparator.compare(&schema1, &schema2).unwrap();

        assert_eq!(comparison.removed_fields.len(), 1);
        assert!(comparison.removed_fields.contains(&"field2".to_string()));
    }

    #[test]
    fn test_different_names() {
        let comparator = SchemaComparator::new();
        let schema1 = create_schema("schema1", vec!["field1"]);
        let schema2 = create_schema("schema2", vec!["field1"]);

        let comparison = comparator.compare(&schema1, &schema2).unwrap();

        assert!(comparison.differences.iter().any(|d| d.contains("Name:")));
    }

    #[test]
    fn test_text_report_generation() {
        let comparator = SchemaComparator::new();
        let schema1 = create_schema("test1", vec!["field1"]);
        let schema2 = create_schema("test2", vec!["field1", "field2"]);

        let comparison = comparator.compare(&schema1, &schema2).unwrap();
        let report = comparator.generate_text_report(&comparison);

        assert!(report.contains("DIFFERENCES"));
        assert!(report.contains("SUMMARY"));
    }
}
