//! Comprehensive test suite for Week 6 Table extraction features
//!
//! This test suite validates all TABLE-001 through TABLE-004 requirements:
//! - TABLE-001: Table parser (thead/tbody/tfoot, colspan/rowspan, nested tables)
//! - TABLE-002: CSV export (RFC 4180 compliant)
//! - TABLE-003: Markdown export with parent_id for nested tables
//! - TABLE-004: Artifacts referenced in NDJSON

use anyhow::Result;
use riptide_html::table_extraction::*;
use serde_json;
use std::collections::HashMap;

/// Test TABLE-001: Basic table structure parsing
#[tokio::test]
async fn test_table_001_basic_structure() -> Result<()> {
    let html = r#"
        <table id="basic-table" class="test-table data-grid">
            <caption>Employee Information</caption>
            <colgroup>
                <col span="2" class="name-cols">
                <col class="age-col">
                <col class="location-col">
            </colgroup>
            <thead>
                <tr>
                    <th>First Name</th>
                    <th>Last Name</th>
                    <th>Age</th>
                    <th>Location</th>
                </tr>
            </thead>
            <tbody>
                <tr class="employee-row">
                    <td>John</td>
                    <td>Doe</td>
                    <td>30</td>
                    <td>New York</td>
                </tr>
                <tr class="employee-row">
                    <td>Jane</td>
                    <td>Smith</td>
                    <td>25</td>
                    <td>Los Angeles</td>
                </tr>
                <tr class="employee-row">
                    <td>Bob</td>
                    <td>Johnson</td>
                    <td>35</td>
                    <td>Chicago</td>
                </tr>
            </tbody>
            <tfoot>
                <tr>
                    <td colspan="2">Total Employees</td>
                    <td>3</td>
                    <td>-</td>
                </tr>
            </tfoot>
        </table>
    "#;

    let tables = extract_tables_advanced(html, None).await?;
    assert_eq!(tables.len(), 1, "Should extract exactly one table");

    let table = &tables[0];

    // Test metadata extraction
    assert_eq!(table.metadata.id, Some("basic-table".to_string()));
    assert!(table.metadata.classes.contains(&"test-table".to_string()));
    assert!(table.metadata.classes.contains(&"data-grid".to_string()));

    // Test caption
    assert_eq!(table.caption, Some("Employee Information".to_string()));

    // Test column groups
    assert_eq!(table.headers.column_groups.len(), 3);
    assert_eq!(table.headers.column_groups[0].span, 2);
    assert!(table.headers.column_groups[0].attributes.contains_key("class"));

    // Test headers
    assert_eq!(table.headers.main.len(), 4);
    assert_eq!(table.headers.main[0].content, "First Name");
    assert_eq!(table.headers.main[1].content, "Last Name");
    assert_eq!(table.headers.main[2].content, "Age");
    assert_eq!(table.headers.main[3].content, "Location");

    // Test body rows
    assert_eq!(table.rows.len(), 3);
    assert_eq!(table.rows[0].cells[0].content, "John");
    assert_eq!(table.rows[1].cells[0].content, "Jane");
    assert_eq!(table.rows[2].cells[0].content, "Bob");

    // Test footer
    assert_eq!(table.footer.len(), 1);
    assert_eq!(table.footer[0].cells[0].content, "Total Employees");
    assert_eq!(table.footer[0].cells[0].colspan, 2);

    // Test structure
    assert_eq!(table.structure.total_columns, 4);
    assert_eq!(table.structure.total_rows, 3);
    assert_eq!(table.structure.header_rows, 1);
    assert_eq!(table.structure.footer_rows, 1);

    Ok(())
}

/// Test TABLE-001: Complex table with colspan and rowspan
#[tokio::test]
async fn test_table_001_complex_spans() -> Result<()> {
    let html = r#"
        <table>
            <thead>
                <tr>
                    <th rowspan="2">Employee</th>
                    <th colspan="3">Contact Information</th>
                    <th rowspan="2">Department</th>
                </tr>
                <tr>
                    <th>Email</th>
                    <th>Phone</th>
                    <th>Address</th>
                </tr>
            </thead>
            <tbody>
                <tr>
                    <td rowspan="2">John Doe</td>
                    <td>john@example.com</td>
                    <td>555-0123</td>
                    <td>123 Main St</td>
                    <td rowspan="2">Engineering</td>
                </tr>
                <tr>
                    <td colspan="3">Alternative: jane@example.com, 555-0124, 456 Oak Ave</td>
                </tr>
            </tbody>
        </table>
    "#;

    let tables = extract_tables_advanced(html, None).await?;
    assert_eq!(tables.len(), 1);

    let table = &tables[0];

    // Test complex structure detection
    assert!(table.structure.has_complex_structure);
    assert_eq!(table.structure.max_colspan, 3);
    assert_eq!(table.structure.max_rowspan, 2);

    // Test header structure with spans
    assert_eq!(table.headers.main.len(), 5); // All cells from first header row
    assert_eq!(table.headers.main[0].rowspan, 2);
    assert_eq!(table.headers.main[1].colspan, 3);
    assert_eq!(table.headers.main[4].rowspan, 2);

    // Test spans_over tracking
    let contact_header = &table.headers.main[1]; // "Contact Information"
    assert_eq!(contact_header.spans_over.len(), 2); // Spans over 2 additional cells

    Ok(())
}

/// Test TABLE-001: Nested tables
#[tokio::test]
async fn test_table_001_nested_tables() -> Result<()> {
    let html = r#"
        <table id="outer-table">
            <caption>Outer Table</caption>
            <tr>
                <th>Section</th>
                <th>Details</th>
            </tr>
            <tr>
                <td>Products</td>
                <td>
                    <table id="products-table">
                        <caption>Product List</caption>
                        <tr><th>Name</th><th>Price</th></tr>
                        <tr><td>Laptop</td><td>$999</td></tr>
                        <tr><td>Mouse</td><td>$25</td></tr>
                    </table>
                </td>
            </tr>
            <tr>
                <td>Services</td>
                <td>
                    <table id="services-table">
                        <tr><th>Service</th><th>Cost</th></tr>
                        <tr><td>Support</td><td>$50/month</td></tr>
                    </table>
                </td>
            </tr>
        </table>
    "#;

    let config = TableExtractionConfig {
        include_nested: true,
        ..Default::default()
    };

    let tables = extract_tables_advanced(html, Some(config)).await?;

    // Should find all 3 tables: outer, products, services
    assert!(tables.len() >= 3, "Should find at least 3 tables (outer + 2 nested)");

    // Find the outer table
    let outer_table = tables.iter()
        .find(|t| !t.nested_tables.is_empty())
        .expect("Should find outer table with nested tables");

    assert_eq!(outer_table.caption, Some("Outer Table".to_string()));
    assert_eq!(outer_table.nested_tables.len(), 2);
    assert!(outer_table.parent_id.is_none());

    // Find nested tables
    let products_table = tables.iter()
        .find(|t| t.caption == Some("Product List".to_string()))
        .expect("Should find products table");

    assert!(products_table.parent_id.is_some());
    assert_eq!(products_table.rows.len(), 2);
    assert_eq!(products_table.rows[0].cells[0].content, "Laptop");

    Ok(())
}

/// Test TABLE-002: RFC 4180 compliant CSV export
#[tokio::test]
async fn test_table_002_csv_export_basic() -> Result<()> {
    let table = create_test_table_with_headers();
    let csv = table.to_csv(true)?;

    let lines: Vec<&str> = csv.lines().collect();
    assert_eq!(lines.len(), 4); // Header + 3 data rows

    // Test header row
    assert_eq!(lines[0], "Name,Age,City,Notes");

    // Test data rows
    assert_eq!(lines[1], "John Doe,30,New York,");
    assert_eq!(lines[2], "Jane Smith,25,Los Angeles,Likes hiking");
    assert_eq!(lines[3], "Bob Johnson,35,Chicago,");

    Ok(())
}

/// Test TABLE-002: RFC 4180 compliance with special characters
#[tokio::test]
async fn test_table_002_csv_rfc4180_compliance() -> Result<()> {
    let table = create_test_table_with_special_chars();
    let csv = table.to_csv(true)?;

    let lines: Vec<&str> = csv.lines().collect();

    // Test quoted fields with commas
    assert!(lines[1].contains("\"Johnson, Jr.\""));

    // Test escaped quotes
    assert!(lines[2].contains("\"Said \"\"Hello\"\" to everyone\""));

    // Test multiline content
    assert!(lines[3].contains("\"Line 1\nLine 2\""));

    Ok(())
}

/// Test TABLE-003: Markdown export
#[tokio::test]
async fn test_table_003_markdown_export() -> Result<()> {
    let table = create_test_table_with_headers();
    let markdown = table.to_markdown(true)?;

    // Test table structure
    assert!(markdown.contains("| Name | Age | City | Notes |"));
    assert!(markdown.contains("| --- | --- | --- | --- |"));
    assert!(markdown.contains("| John Doe | 30 | New York |  |"));

    // Test metadata comments
    assert!(markdown.contains("<!-- Table ID:"));
    assert!(markdown.contains("<!-- Columns: 4, Rows: 3 -->"));

    Ok(())
}

/// Test TABLE-003: Markdown export with spans
#[tokio::test]
async fn test_table_003_markdown_with_spans() -> Result<()> {
    let table = create_test_table_with_spans();
    let markdown = table.to_markdown(true)?;

    // Test span notation
    assert!(markdown.contains("(span: 2x1)"));
    assert!(markdown.contains("<!-- Note: Table has complex structure (spans) -->"));

    Ok(())
}

/// Test TABLE-003: Nested table references in markdown
#[tokio::test]
async fn test_table_003_nested_references() -> Result<()> {
    let mut table = create_test_table_with_headers();
    table.nested_tables = vec!["table_2".to_string(), "table_3".to_string()];
    table.parent_id = Some("parent_table_1".to_string());

    let markdown = table.to_markdown(true)?;

    // Test parent reference
    assert!(markdown.contains("<!-- Parent Table: parent_table_1 -->"));

    // Test nested table references
    assert!(markdown.contains("**Nested Tables:** table_2, table_3"));

    Ok(())
}

/// Test TABLE-004: NDJSON artifacts export
#[tokio::test]
async fn test_table_004_ndjson_artifacts() -> Result<()> {
    let table = create_test_table_with_headers();
    let artifacts = table.to_ndjson_artifacts(None)?;

    assert_eq!(artifacts.len(), 3); // CSV, Markdown, Metadata

    // Test CSV artifact
    let csv_artifact: TableArtifact = serde_json::from_str(&artifacts[0])?;
    assert_eq!(csv_artifact.artifact_type, "csv");
    assert_eq!(csv_artifact.table_id, table.id);
    assert!(csv_artifact.content.contains("Name,Age,City"));
    assert_eq!(csv_artifact.metadata.get("format"), Some(&"RFC4180".to_string()));

    // Test Markdown artifact
    let md_artifact: TableArtifact = serde_json::from_str(&artifacts[1])?;
    assert_eq!(md_artifact.artifact_type, "markdown");
    assert!(md_artifact.content.contains("| Name | Age |"));

    // Test Metadata artifact
    let meta_artifact: TableArtifact = serde_json::from_str(&artifacts[2])?;
    assert_eq!(meta_artifact.artifact_type, "metadata");

    Ok(())
}

/// Test TABLE-004: NDJSON artifacts with file paths
#[tokio::test]
async fn test_table_004_ndjson_with_paths() -> Result<()> {
    let table = create_test_table_with_headers();
    let artifacts = table.to_ndjson_artifacts(Some("/tmp/tables"))?;

    let csv_artifact: TableArtifact = serde_json::from_str(&artifacts[0])?;
    assert_eq!(csv_artifact.content, format!("/tmp/tables/{}.csv", table.id));

    let md_artifact: TableArtifact = serde_json::from_str(&artifacts[1])?;
    assert_eq!(md_artifact.content, format!("/tmp/tables/{}.md", table.id));

    Ok(())
}

/// Test edge case: Empty table
#[tokio::test]
async fn test_edge_case_empty_table() -> Result<()> {
    let html = r#"<table></table>"#;

    let tables = extract_tables_advanced(html, None).await?;
    assert_eq!(tables.len(), 1);

    let table = &tables[0];
    assert_eq!(table.structure.total_columns, 0);
    assert_eq!(table.structure.total_rows, 0);
    assert!(!table.structure.has_complex_structure);

    Ok(())
}

/// Test edge case: Table with only headers
#[tokio::test]
async fn test_edge_case_headers_only() -> Result<()> {
    let html = r#"
        <table>
            <thead>
                <tr><th>Name</th><th>Age</th></tr>
            </thead>
        </table>
    "#;

    let tables = extract_tables_advanced(html, None).await?;
    assert_eq!(tables.len(), 1);

    let table = &tables[0];
    assert_eq!(table.headers.main.len(), 2);
    assert_eq!(table.rows.len(), 0);
    assert_eq!(table.structure.total_rows, 0);

    Ok(())
}

/// Test size filtering configuration
#[tokio::test]
async fn test_size_filtering() -> Result<()> {
    let html = r#"
        <table>
            <tr><td>Small</td></tr>
        </table>
        <table>
            <tr><td>Col1</td><td>Col2</td><td>Col3</td></tr>
            <tr><td>Row1</td><td>Row2</td><td>Row3</td></tr>
            <tr><td>Row1</td><td>Row2</td><td>Row3</td></tr>
        </table>
    "#;

    let config = TableExtractionConfig {
        min_size: Some((2, 2)), // At least 2 rows and 2 columns
        ..Default::default()
    };

    let tables = extract_tables_advanced(html, Some(config)).await?;
    assert_eq!(tables.len(), 1); // Only the larger table should be extracted

    let table = &tables[0];
    assert_eq!(table.structure.total_columns, 3);
    assert_eq!(table.structure.total_rows, 2);

    Ok(())
}

/// Test warehouse-ready CSV output (database loading)
#[tokio::test]
async fn test_warehouse_ready_csv() -> Result<()> {
    let table = create_warehouse_test_table();
    let csv = table.to_csv(true)?;

    let lines: Vec<&str> = csv.lines().collect();

    // Test consistent column count
    for line in &lines {
        let cols: Vec<&str> = line.split(',').collect();
        assert_eq!(cols.len(), 5, "All rows should have same column count");
    }

    // Test proper NULL handling (empty strings)
    assert!(lines[1].contains(",,"));

    // Test no line breaks within cells
    for line in &lines {
        assert!(!line.contains('\n'), "CSV lines should not contain unescaped newlines");
    }

    Ok(())
}

/// Performance test for large tables
#[tokio::test]
async fn test_performance_large_table() -> Result<()> {
    let html = create_large_table_html(100, 10); // 100 rows, 10 columns

    let start = std::time::Instant::now();
    let tables = extract_tables_advanced(&html, None).await?;
    let duration = start.elapsed();

    // Should complete within reasonable time (adjust as needed)
    assert!(duration.as_millis() < 5000, "Large table extraction should complete within 5 seconds");

    assert_eq!(tables.len(), 1);
    let table = &tables[0];
    assert_eq!(table.structure.total_rows, 100);
    assert_eq!(table.structure.total_columns, 10);

    Ok(())
}

/// Test integration with multiple formats
#[tokio::test]
async fn test_multi_format_integration() -> Result<()> {
    let html = r#"
        <table id="integration-test">
            <caption>Integration Test Table</caption>
            <tr><th>ID</th><th>Name</th><th>Status</th></tr>
            <tr><td>1</td><td>Alice</td><td>Active</td></tr>
            <tr><td>2</td><td>Bob</td><td>Inactive</td></tr>
        </table>
    "#;

    let (tables, artifacts) = extract_and_export_tables(html, Some("/tmp/output"), None).await?;

    assert_eq!(tables.len(), 1);
    assert_eq!(artifacts.len(), 3); // CSV, Markdown, Metadata artifacts

    // Test that all artifacts are valid JSON
    for artifact in artifacts {
        let artifact_obj: TableArtifact = serde_json::from_str(&artifact)?;
        assert!(!artifact_obj.table_id.is_empty());
        assert!(!artifact_obj.artifact_type.is_empty());
    }

    Ok(())
}

// Helper functions for test data creation

fn create_test_table_with_headers() -> AdvancedTableData {
    let mut table = create_basic_table_structure();

    table.headers.main = vec![
        create_test_cell("Name", CellType::Header, 0, 0),
        create_test_cell("Age", CellType::Header, 0, 1),
        create_test_cell("City", CellType::Header, 0, 2),
        create_test_cell("Notes", CellType::Header, 0, 3),
    ];

    table.rows = vec![
        create_table_row(vec!["John Doe", "30", "New York", ""], 0),
        create_table_row(vec!["Jane Smith", "25", "Los Angeles", "Likes hiking"], 1),
        create_table_row(vec!["Bob Johnson", "35", "Chicago", ""], 2),
    ];

    table.structure.total_columns = 4;
    table.structure.total_rows = 3;
    table.structure.header_rows = 1;

    table
}

fn create_test_table_with_special_chars() -> AdvancedTableData {
    let mut table = create_basic_table_structure();

    table.headers.main = vec![
        create_test_cell("Name", CellType::Header, 0, 0),
        create_test_cell("Description", CellType::Header, 0, 1),
    ];

    table.rows = vec![
        create_table_row(vec!["Johnson, Jr.", "Regular entry"], 0),
        create_table_row(vec!["Quote Test", "Said \"Hello\" to everyone"], 1),
        create_table_row(vec!["Multiline", "Line 1\nLine 2"], 2),
    ];

    table.structure.total_columns = 2;
    table.structure.total_rows = 3;

    table
}

fn create_test_table_with_spans() -> AdvancedTableData {
    let mut table = create_basic_table_structure();

    table.headers.main = vec![
        create_test_cell("Name", CellType::Header, 0, 0),
        create_test_cell_with_span("Contact", CellType::Header, 0, 1, 2, 1),
    ];

    table.rows = vec![
        create_table_row(vec!["John", "email@test.com"], 0),
    ];

    table.structure.has_complex_structure = true;
    table.structure.max_colspan = 2;
    table.structure.total_columns = 3;

    table
}

fn create_warehouse_test_table() -> AdvancedTableData {
    let mut table = create_basic_table_structure();

    table.headers.main = vec![
        create_test_cell("id", CellType::Header, 0, 0),
        create_test_cell("name", CellType::Header, 0, 1),
        create_test_cell("age", CellType::Header, 0, 2),
        create_test_cell("department", CellType::Header, 0, 3),
        create_test_cell("salary", CellType::Header, 0, 4),
    ];

    table.rows = vec![
        create_table_row(vec!["1", "John Doe", "30", "", "50000"], 0),
        create_table_row(vec!["2", "Jane Smith", "25", "Engineering", ""], 1),
        create_table_row(vec!["3", "Bob Wilson", "35", "Marketing", "45000"], 2),
    ];

    table.structure.total_columns = 5;
    table.structure.total_rows = 3;

    table
}

fn create_basic_table_structure() -> AdvancedTableData {
    AdvancedTableData {
        id: "test_table_1".to_string(),
        headers: TableHeaders {
            main: Vec::new(),
            sub_headers: Vec::new(),
            column_groups: Vec::new(),
        },
        rows: Vec::new(),
        footer: Vec::new(),
        caption: None,
        metadata: TableMetadata {
            attributes: HashMap::new(),
            classes: Vec::new(),
            id: Some("test".to_string()),
            processed_at: chrono::Utc::now().to_rfc3339(),
            source: None,
        },
        parent_id: None,
        nested_tables: Vec::new(),
        structure: TableStructure {
            total_columns: 0,
            total_rows: 0,
            header_rows: 0,
            footer_rows: 0,
            has_complex_structure: false,
            max_colspan: 1,
            max_rowspan: 1,
        },
    }
}

fn create_test_cell(content: &str, cell_type: CellType, row: usize, col: usize) -> TableCell {
    TableCell {
        content: content.to_string(),
        html_content: content.to_string(),
        colspan: 1,
        rowspan: 1,
        cell_type,
        attributes: HashMap::new(),
        column_index: col,
        row_index: row,
        spans_over: Vec::new(),
    }
}

fn create_test_cell_with_span(
    content: &str,
    cell_type: CellType,
    row: usize,
    col: usize,
    colspan: usize,
    rowspan: usize,
) -> TableCell {
    let mut spans_over = Vec::new();
    for r in 0..rowspan {
        for c in 0..colspan {
            if r > 0 || c > 0 {
                spans_over.push(CellPosition {
                    row: row + r,
                    column: col + c,
                });
            }
        }
    }

    TableCell {
        content: content.to_string(),
        html_content: content.to_string(),
        colspan,
        rowspan,
        cell_type,
        attributes: HashMap::new(),
        column_index: col,
        row_index: row,
        spans_over,
    }
}

fn create_table_row(cell_contents: Vec<&str>, index: usize) -> TableRow {
    let cells = cell_contents
        .iter()
        .enumerate()
        .map(|(col, content)| create_test_cell(content, CellType::Data, index, col))
        .collect();

    TableRow {
        cells,
        attributes: HashMap::new(),
        row_type: RowType::Body,
        index,
    }
}

fn create_large_table_html(rows: usize, cols: usize) -> String {
    let mut html = String::from("<table><thead><tr>");

    // Header
    for i in 0..cols {
        html.push_str(&format!("<th>Col{}</th>", i + 1));
    }
    html.push_str("</tr></thead><tbody>");

    // Body rows
    for row in 0..rows {
        html.push_str("<tr>");
        for col in 0..cols {
            html.push_str(&format!("<td>R{}C{}</td>", row + 1, col + 1));
        }
        html.push_str("</tr>");
    }

    html.push_str("</tbody></table>");
    html
}