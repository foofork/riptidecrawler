//! Table analysis intelligence service
//!
//! Provides type detection and data analysis capabilities for tabular data.

use regex::Regex;
use riptide_extraction::table_extraction::AdvancedTableData;

/// Analyzes table structure and data types
pub struct TableAnalyzer;

impl TableAnalyzer {
    /// Create a new table analyzer
    pub fn new() -> Self {
        Self
    }

    /// Detect column data types from table data
    pub fn detect_column_types(&self, table: &AdvancedTableData) -> Vec<String> {
        let mut column_types = Vec::new();

        if table.rows.is_empty() {
            return column_types;
        }

        let num_columns = table.structure.total_columns;

        for col_index in 0..num_columns {
            let mut sample_values = Vec::new();

            // Collect sample values from this column
            for row in table.rows.iter().take(10) {
                if let Some(cell) = row.cells.get(col_index) {
                    sample_values.push(&cell.content);
                }
            }

            // Detect type based on sample values
            let detected_type = self.detect_type_from_samples(&sample_values);
            column_types.push(detected_type);
        }

        column_types
    }

    /// Detect data type from sample values using heuristics
    fn detect_type_from_samples(&self, samples: &[&String]) -> String {
        if samples.is_empty() {
            return "unknown".to_string();
        }

        let mut numeric_count = 0;
        let mut date_count = 0;
        let mut boolean_count = 0;

        for &sample in samples {
            let trimmed = sample.trim();

            if trimmed.is_empty() {
                continue;
            }

            // Check for boolean
            if ["true", "false", "yes", "no", "1", "0"].contains(&trimmed.to_lowercase().as_str()) {
                boolean_count += 1;
            }
            // Check for numeric (integer or float)
            else if trimmed.parse::<f64>().is_ok() {
                numeric_count += 1;
            }
            // Check for date-like patterns
            else if self.is_date_like(trimmed) {
                date_count += 1;
            }
        }

        let total_samples = samples.len();
        let threshold = (total_samples as f64 * 0.7) as usize; // 70% threshold

        if numeric_count >= threshold {
            "number"
        } else if date_count >= threshold {
            "date"
        } else if boolean_count >= threshold {
            "boolean"
        } else {
            "string"
        }
        .to_string()
    }

    /// Simple date detection using common patterns
    fn is_date_like(&self, text: &str) -> bool {
        let date_patterns = [
            r"\d{4}-\d{2}-\d{2}",       // YYYY-MM-DD
            r"\d{2}/\d{2}/\d{4}",       // MM/DD/YYYY
            r"\d{2}-\d{2}-\d{4}",       // MM-DD-YYYY
            r"\d{1,2}/\d{1,2}/\d{2,4}", // M/D/YY or MM/DD/YYYY
        ];

        for pattern in &date_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if regex.is_match(text) {
                    return true;
                }
            }
        }

        false
    }
}

impl Default for TableAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_type_from_samples_numeric() {
        let analyzer = TableAnalyzer::new();
        let s1 = "123".to_string();
        let s2 = "45.6".to_string();
        let s3 = "0".to_string();
        let samples = vec![&s1, &s2, &s3];
        assert_eq!(analyzer.detect_type_from_samples(&samples), "number");
    }

    #[test]
    fn test_detect_type_from_samples_string() {
        let analyzer = TableAnalyzer::new();
        let s1 = "hello".to_string();
        let s2 = "world".to_string();
        let s3 = "test".to_string();
        let samples = vec![&s1, &s2, &s3];
        assert_eq!(analyzer.detect_type_from_samples(&samples), "string");
    }

    #[test]
    fn test_detect_type_from_samples_boolean() {
        let analyzer = TableAnalyzer::new();
        let s1 = "true".to_string();
        let s2 = "false".to_string();
        let s3 = "yes".to_string();
        let samples = vec![&s1, &s2, &s3];
        assert_eq!(analyzer.detect_type_from_samples(&samples), "boolean");
    }

    #[test]
    fn test_detect_type_from_samples_empty() {
        let analyzer = TableAnalyzer::new();
        let samples = vec![];
        assert_eq!(analyzer.detect_type_from_samples(&samples), "unknown");
    }

    #[test]
    fn test_is_date_like() {
        let analyzer = TableAnalyzer::new();
        assert!(analyzer.is_date_like("2023-12-25"));
        assert!(analyzer.is_date_like("12/25/2023"));
        assert!(analyzer.is_date_like("25-12-2023"));
        assert!(!analyzer.is_date_like("hello world"));
        assert!(!analyzer.is_date_like("123"));
    }
}
