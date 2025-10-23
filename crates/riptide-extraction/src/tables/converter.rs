//! Table format conversion logic

use super::types::TableData;
use anyhow::Result;

/// Table converter for different output formats
pub struct TableConverter;

impl TableConverter {
    /// Convert tables to Markdown format
    pub fn to_markdown(tables: &[TableData]) -> String {
        let mut output = String::new();

        for (idx, table) in tables.iter().enumerate() {
            // Add caption if present
            if let Some(caption) = &table.caption {
                output.push_str(&format!("\n## Table {} - {}\n\n", idx + 1, caption));
            } else {
                output.push_str(&format!("\n## Table {}\n\n", idx + 1));
            }

            // Add headers
            if !table.headers.is_empty() {
                output.push_str("| ");
                output.push_str(&table.headers.join(" | "));
                output.push_str(" |\n");

                // Add separator
                output.push('|');
                for _ in &table.headers {
                    output.push_str(" --- |");
                }
                output.push('\n');
            }

            // Add rows
            for row in &table.data {
                output.push_str("| ");
                output.push_str(&row.join(" | "));
                output.push_str(" |\n");
            }

            output.push('\n');
        }

        output
    }

    /// Convert tables to CSV format
    pub fn to_csv(tables: &[TableData]) -> Result<String> {
        let mut output = String::new();

        for (idx, table) in tables.iter().enumerate() {
            // Add table separator for multiple tables
            if idx > 0 {
                output.push_str("\n\n");
            }

            // Add caption as comment
            if let Some(caption) = &table.caption {
                output.push_str(&format!("# Table {} - {}\n", idx + 1, caption));
            } else {
                output.push_str(&format!("# Table {}\n", idx + 1));
            }

            // Add headers if present
            if !table.headers.is_empty() {
                output.push_str(&Self::escape_csv_row(&table.headers));
                output.push('\n');
            }

            // Add rows
            for row in &table.data {
                output.push_str(&Self::escape_csv_row(row));
                output.push('\n');
            }
        }

        Ok(output)
    }

    /// Convert tables to JSON format
    pub fn to_json(tables: &[TableData]) -> Result<String> {
        serde_json::to_string_pretty(tables)
            .map_err(|e| anyhow::anyhow!("Failed to serialize to JSON: {}", e))
    }

    /// Escape a CSV row with proper quoting
    fn escape_csv_row(row: &[String]) -> String {
        row.iter()
            .map(|cell| {
                if cell.contains(',') || cell.contains('"') || cell.contains('\n') {
                    format!("\"{}\"", cell.replace('"', "\"\""))
                } else {
                    cell.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(",")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_table() -> TableData {
        TableData {
            id: "test-1".to_string(),
            rows: 2,
            columns: 3,
            headers: vec!["Name".to_string(), "Age".to_string(), "City".to_string()],
            data: vec![
                vec!["Alice".to_string(), "30".to_string(), "NYC".to_string()],
                vec!["Bob".to_string(), "25".to_string(), "LA".to_string()],
            ],
            caption: Some("Test Table".to_string()),
        }
    }

    #[test]
    fn test_to_markdown() {
        let table = create_test_table();
        let markdown = TableConverter::to_markdown(&[table]);

        assert!(markdown.contains("## Table 1 - Test Table"));
        assert!(markdown.contains("| Name | Age | City |"));
        assert!(markdown.contains("| --- | --- | --- |"));
        assert!(markdown.contains("| Alice | 30 | NYC |"));
    }

    #[test]
    fn test_to_csv() {
        let table = create_test_table();
        let csv = TableConverter::to_csv(&[table]).unwrap();

        assert!(csv.contains("# Table 1 - Test Table"));
        assert!(csv.contains("Name,Age,City"));
        assert!(csv.contains("Alice,30,NYC"));
    }

    #[test]
    fn test_to_json() {
        let table = create_test_table();
        let json = TableConverter::to_json(&[table]).unwrap();

        assert!(json.contains("\"id\": \"test-1\""));
        assert!(json.contains("\"rows\": 2"));
        assert!(json.contains("\"headers\""));
    }

    #[test]
    fn test_csv_escaping() {
        let row = vec![
            "Normal".to_string(),
            "Has,comma".to_string(),
            "Has\"quote".to_string(),
            "Has\nNewline".to_string(),
        ];

        let escaped = TableConverter::escape_csv_row(&row);
        assert_eq!(
            escaped,
            "Normal,\"Has,comma\",\"Has\"\"quote\",\"Has\nNewline\""
        );
    }
}
