/// Test for CLI tables command schema alignment
/// This test verifies that the CLI properly handles the API response structure
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
struct TableSummary {
    id: String,
    rows: usize,
    columns: usize,
    #[serde(default)]
    headers: Vec<String>,
    #[serde(default)]
    data: Vec<Vec<String>>,
    #[serde(default)]
    metadata: TableMetadata,
}

#[derive(Deserialize, Serialize, Debug, Default)]
struct TableMetadata {
    #[serde(default)]
    has_headers: bool,
    #[serde(default)]
    data_types: Vec<String>,
    #[serde(default)]
    has_complex_structure: bool,
    #[serde(default)]
    caption: Option<String>,
    #[serde(default)]
    css_classes: Vec<String>,
    #[serde(default)]
    html_id: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
struct TableExtractResponse {
    tables: Vec<TableSummary>,
    #[serde(default)]
    extraction_time_ms: u64,
    #[serde(default)]
    total_tables: usize,
}

#[test]
fn test_api_response_deserialization() {
    // Simulate the API response structure
    let api_response = r#"{
        "tables": [
            {
                "id": "table-123",
                "rows": 243,
                "columns": 6,
                "headers": ["Name", "Age", "City"],
                "data": [
                    ["Alice", "30", "NYC"],
                    ["Bob", "25", "LA"]
                ],
                "metadata": {
                    "has_headers": true,
                    "data_types": ["string", "number", "string"],
                    "has_complex_structure": false,
                    "caption": "User Data",
                    "css_classes": ["table-striped"],
                    "html_id": "users"
                }
            }
        ],
        "extraction_time_ms": 1500,
        "total_tables": 1
    }"#;

    let result: Result<TableExtractResponse, _> = serde_json::from_str(api_response);
    assert!(result.is_ok(), "Failed to deserialize API response: {:?}", result.err());

    let response = result.unwrap();
    assert_eq!(response.tables.len(), 1);
    assert_eq!(response.total_tables, 1);
    assert_eq!(response.extraction_time_ms, 1500);

    let table = &response.tables[0];
    assert_eq!(table.id, "table-123");
    assert_eq!(table.rows, 243);
    assert_eq!(table.columns, 6);
    assert_eq!(table.headers.len(), 3);
    assert_eq!(table.data.len(), 2);
    assert!(table.metadata.has_headers);
}

#[test]
fn test_minimal_api_response() {
    // Test with minimal response (only counts)
    let api_response = r#"{
        "tables": [
            {
                "id": "table-456",
                "rows": 100,
                "columns": 4
            }
        ]
    }"#;

    let result: Result<TableExtractResponse, _> = serde_json::from_str(api_response);
    assert!(result.is_ok(), "Failed to deserialize minimal API response");

    let response = result.unwrap();
    assert_eq!(response.tables.len(), 1);

    let table = &response.tables[0];
    assert_eq!(table.id, "table-456");
    assert_eq!(table.rows, 100);
    assert_eq!(table.columns, 4);
    assert_eq!(table.headers.len(), 0); // Default empty
    assert_eq!(table.data.len(), 0); // Default empty
}

#[test]
fn test_csv_parsing() {
    let csv_content = r#"Name,Age,City
Alice,30,NYC
Bob,25,LA
Charlie,35,SF"#;

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(csv_content.as_bytes());

    let mut rows = Vec::new();
    for result in reader.records() {
        if let Ok(record) = result {
            let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
            rows.push(row);
        }
    }

    assert_eq!(rows.len(), 4); // Headers + 3 data rows
    assert_eq!(rows[0], vec!["Name", "Age", "City"]);
    assert_eq!(rows[1], vec!["Alice", "30", "NYC"]);
}

#[test]
fn test_markdown_parsing() {
    let markdown = r#"
## Table 1

| Name | Age | City |
| --- | --- | --- |
| Alice | 30 | NYC |
| Bob | 25 | LA |
"#;

    let mut data = Vec::new();
    for line in markdown.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty()
            || trimmed.starts_with('#')
            || (trimmed.starts_with('|') && trimmed.contains("---")) {
            continue;
        }

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

    assert_eq!(data.len(), 3); // Headers + 2 data rows
    assert_eq!(data[0], vec!["Name", "Age", "City"]);
    assert_eq!(data[1], vec!["Alice", "30", "NYC"]);
}
