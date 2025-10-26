//! Integration tests for table extraction module

#[cfg(test)]
mod integration_tests {
    use crate::tables::*;

    #[test]
    fn test_table_data_serialization() {
        let table = TableData {
            id: "test-1".to_string(),
            rows: 2,
            columns: 3,
            headers: vec!["A".to_string(), "B".to_string(), "C".to_string()],
            data: vec![
                vec!["1".to_string(), "2".to_string(), "3".to_string()],
                vec!["4".to_string(), "5".to_string(), "6".to_string()],
            ],
            caption: Some("Test".to_string()),
        };

        // Test JSON serialization
        let json = serde_json::to_string(&table).unwrap();
        assert!(json.contains("\"id\":\"test-1\""));

        // Test deserialization
        let deserialized: TableData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "test-1");
        assert_eq!(deserialized.rows, 2);
    }

    #[test]
    fn test_format_conversions() {
        let tables = vec![TableData {
            id: "test".to_string(),
            rows: 1,
            columns: 2,
            headers: vec!["Name".to_string(), "Value".to_string()],
            data: vec![vec!["Test".to_string(), "123".to_string()]],
            caption: None,
        }];

        // Test all format conversions
        let markdown = TableConverter::to_markdown(&tables);
        assert!(markdown.contains("| Name | Value |"));

        let csv = TableConverter::to_csv(&tables).unwrap();
        assert!(csv.contains("Name,Value"));

        let json = TableConverter::to_json(&tables).unwrap();
        assert!(json.contains("\"id\": \"test\""));
    }

    #[test]
    fn test_table_metadata() {
        let metadata = TableMetadata {
            has_headers: true,
            data_types: vec!["string".to_string(), "number".to_string()],
            has_complex_structure: false,
            caption: Some("Test Table".to_string()),
            css_classes: vec!["table".to_string(), "data".to_string()],
            html_id: Some("table-1".to_string()),
        };

        // Test serialization
        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("\"has_headers\":true"));

        // Test default values
        let default_metadata = TableMetadata::default();
        assert!(!default_metadata.has_headers);
        assert!(default_metadata.data_types.is_empty());
    }

    #[test]
    fn test_extraction_request_creation() {
        let html = "<table><tr><td>Test</td></tr></table>".to_string();
        let request = TableExtractor::create_request(html.clone());

        assert_eq!(request.html_content, html);
    }

    #[test]
    fn test_parse_round_trip() {
        // Create test data
        let original_data = vec![
            vec!["Name".to_string(), "Age".to_string()],
            vec!["Alice".to_string(), "30".to_string()],
        ];

        // Convert to CSV and parse back
        let table = TableData {
            id: "test".to_string(),
            rows: 2,
            columns: 2,
            headers: vec![],
            data: original_data.clone(),
            caption: None,
        };

        let csv = TableConverter::to_csv(&[table]).unwrap();
        let parsed = parse_csv_to_data(&csv).unwrap();

        // Should match (ignoring comment lines)
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0], original_data[0]);
        assert_eq!(parsed[1], original_data[1]);
    }

    /// Helper function to parse CSV back to data structure for round-trip testing
    fn parse_csv_to_data(csv: &str) -> Result<Vec<Vec<String>>, String> {
        let mut result = Vec::new();

        for line in csv.lines() {
            // Skip empty lines and comment lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let row: Vec<String> = line
                .split(',')
                .map(|cell| cell.trim().to_string())
                .collect();

            result.push(row);
        }

        Ok(result)
    }
}
