//! Parsing logic for converting formatted content back to table data

use super::types::TableSummary;
use anyhow::{Context, Result};

/// Parse formatted content back to table data
pub fn parse_content_to_table_data(
    content: &str,
    format: &str,
    summary: &TableSummary,
) -> Result<Vec<Vec<String>>> {
    match format {
        "csv" => parse_csv_to_data(content),
        "markdown" => parse_markdown_to_data(content),
        "json" => {
            // Try to parse as JSON table data
            if let Ok(data) = serde_json::from_str::<Vec<Vec<String>>>(content) {
                Ok(data)
            } else {
                // Fallback to summary data
                Ok(summary.data.clone())
            }
        }
        _ => Ok(summary.data.clone()),
    }
}

/// Parse CSV content to table data
pub fn parse_csv_to_data(csv: &str) -> Result<Vec<Vec<String>>> {
    let mut data = Vec::new();
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(csv.as_bytes());

    for result in reader.records() {
        let record = result.context("Failed to parse CSV record")?;
        let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();

        // Skip comment lines (table separators)
        if !row.is_empty() && !row[0].starts_with('#') {
            data.push(row);
        }
    }

    Ok(data)
}

/// Parse Markdown table content to table data
pub fn parse_markdown_to_data(markdown: &str) -> Result<Vec<Vec<String>>> {
    let mut data = Vec::new();

    for line in markdown.lines() {
        let trimmed = line.trim();

        // Skip headers, separators, and empty lines
        if trimmed.is_empty()
            || trimmed.starts_with('#')
            || (trimmed.starts_with('|') && trimmed.contains("---"))
        {
            continue;
        }

        // Parse table rows (lines starting and ending with |)
        if trimmed.starts_with('|') && trimmed.ends_with('|') {
            let row: Vec<String> = trimmed
                .trim_start_matches('|')
                .trim_end_matches('|')
                .split('|')
                .map(|s| s.trim().to_string())
                .collect();

            if !row.is_empty() {
                data.push(row);
            }
        }
    }

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_csv() {
        let csv = "Name,Age,City\nAlice,30,NYC\nBob,25,LA";
        let data = parse_csv_to_data(csv).unwrap();

        assert_eq!(data.len(), 3);
        assert_eq!(data[0], vec!["Name", "Age", "City"]);
        assert_eq!(data[1], vec!["Alice", "30", "NYC"]);
    }

    #[test]
    fn test_parse_csv_with_comments() {
        let csv = "# Table 1\nName,Age\nAlice,30";
        let data = parse_csv_to_data(csv).unwrap();

        // Should skip comment line
        assert_eq!(data.len(), 2);
        assert_eq!(data[0], vec!["Name", "Age"]);
    }

    #[test]
    fn test_parse_markdown() {
        let markdown = r#"
| Name | Age | City |
| --- | --- | --- |
| Alice | 30 | NYC |
| Bob | 25 | LA |
"#;
        let data = parse_markdown_to_data(markdown).unwrap();

        assert_eq!(data.len(), 3); // Header + 2 rows
        assert_eq!(data[0], vec!["Name", "Age", "City"]);
        assert_eq!(data[1], vec!["Alice", "30", "NYC"]);
    }

    #[test]
    fn test_parse_markdown_with_heading() {
        let markdown = r#"
## Table 1

| Name | Age |
| --- | --- |
| Alice | 30 |
"#;
        let data = parse_markdown_to_data(markdown).unwrap();

        assert_eq!(data.len(), 2);
        assert_eq!(data[0], vec!["Name", "Age"]);
    }
}
