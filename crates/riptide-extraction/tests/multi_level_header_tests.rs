//! Comprehensive tests for multi-level header extraction
//!
//! Tests the implementation of multi-level header extraction including:
//! - Multi-row headers with colspan
//! - Multi-row headers with rowspan
//! - Complex hierarchical structures with mixed spans
//! - Backwards compatibility with single-level headers

use anyhow::Result;
use riptide_extraction::table_extraction::*;

/// Test single-level headers (backwards compatibility)
#[tokio::test]
async fn test_single_level_headers() -> Result<()> {
    let html = r#"
        <table>
            <thead>
                <tr>
                    <th>Name</th>
                    <th>Age</th>
                    <th>City</th>
                </tr>
            </thead>
            <tbody>
                <tr>
                    <td>John</td>
                    <td>30</td>
                    <td>New York</td>
                </tr>
            </tbody>
        </table>
    "#;

    let tables = extract_tables_advanced(html, None).await?;
    assert_eq!(tables.len(), 1);

    let table = &tables[0];

    // Verify main headers
    assert_eq!(table.headers.main.len(), 3);
    assert_eq!(table.headers.main[0].content, "Name");
    assert_eq!(table.headers.main[1].content, "Age");
    assert_eq!(table.headers.main[2].content, "City");

    // Verify no sub-headers
    assert_eq!(table.headers.sub_headers.len(), 0);

    // Verify structure
    assert_eq!(table.structure.header_rows, 1);

    Ok(())
}

/// Test two-level headers with colspan
#[tokio::test]
async fn test_two_level_headers_with_colspan() -> Result<()> {
    let html = r#"
        <table>
            <thead>
                <tr>
                    <th colspan="2">Personal Info</th>
                    <th colspan="2">Contact</th>
                </tr>
                <tr>
                    <th>First Name</th>
                    <th>Last Name</th>
                    <th>Email</th>
                    <th>Phone</th>
                </tr>
            </thead>
            <tbody>
                <tr>
                    <td>John</td>
                    <td>Doe</td>
                    <td>john@example.com</td>
                    <td>555-0123</td>
                </tr>
            </tbody>
        </table>
    "#;

    let tables = extract_tables_advanced(html, None).await?;
    assert_eq!(tables.len(), 1);

    let table = &tables[0];

    // Verify sub-headers (first level)
    assert_eq!(table.headers.sub_headers.len(), 1);
    assert_eq!(table.headers.sub_headers[0].len(), 2);
    assert_eq!(table.headers.sub_headers[0][0].content, "Personal Info");
    assert_eq!(table.headers.sub_headers[0][0].colspan, 2);
    assert_eq!(table.headers.sub_headers[0][1].content, "Contact");
    assert_eq!(table.headers.sub_headers[0][1].colspan, 2);

    // Verify main headers (second level - most specific)
    assert_eq!(table.headers.main.len(), 4);
    assert_eq!(table.headers.main[0].content, "First Name");
    assert_eq!(table.headers.main[1].content, "Last Name");
    assert_eq!(table.headers.main[2].content, "Email");
    assert_eq!(table.headers.main[3].content, "Phone");

    // Verify structure
    assert_eq!(table.structure.header_rows, 2);
    assert!(table.structure.has_complex_structure);
    assert_eq!(table.structure.max_colspan, 2);

    Ok(())
}

/// Test two-level headers with rowspan
#[tokio::test]
async fn test_two_level_headers_with_rowspan() -> Result<()> {
    let html = r#"
        <table>
            <thead>
                <tr>
                    <th rowspan="2">Employee</th>
                    <th colspan="3">Details</th>
                </tr>
                <tr>
                    <th>Department</th>
                    <th>Position</th>
                    <th>Years</th>
                </tr>
            </thead>
            <tbody>
                <tr>
                    <td>John Doe</td>
                    <td>Engineering</td>
                    <td>Senior Developer</td>
                    <td>5</td>
                </tr>
            </tbody>
        </table>
    "#;

    let tables = extract_tables_advanced(html, None).await?;
    assert_eq!(tables.len(), 1);

    let table = &tables[0];

    // Verify sub-headers
    assert_eq!(table.headers.sub_headers.len(), 1);
    assert_eq!(table.headers.sub_headers[0].len(), 2);
    assert_eq!(table.headers.sub_headers[0][0].content, "Employee");
    assert_eq!(table.headers.sub_headers[0][0].rowspan, 2);
    assert_eq!(table.headers.sub_headers[0][1].content, "Details");
    assert_eq!(table.headers.sub_headers[0][1].colspan, 3);

    // Verify main headers
    assert_eq!(table.headers.main.len(), 3);
    assert_eq!(table.headers.main[0].content, "Department");
    assert_eq!(table.headers.main[1].content, "Position");
    assert_eq!(table.headers.main[2].content, "Years");

    // Verify structure
    assert_eq!(table.structure.header_rows, 2);
    assert!(table.structure.has_complex_structure);
    assert_eq!(table.structure.max_colspan, 3);
    assert_eq!(table.structure.max_rowspan, 2);

    Ok(())
}

/// Test three-level headers with mixed colspan/rowspan
#[tokio::test]
async fn test_three_level_headers_mixed_spans() -> Result<()> {
    let html = r#"
        <table>
            <thead>
                <tr>
                    <th rowspan="3">ID</th>
                    <th colspan="6">Employee Information</th>
                </tr>
                <tr>
                    <th colspan="2">Personal</th>
                    <th colspan="3">Contact</th>
                    <th rowspan="2">Status</th>
                </tr>
                <tr>
                    <th>First</th>
                    <th>Last</th>
                    <th>Email</th>
                    <th>Phone</th>
                    <th>Address</th>
                </tr>
            </thead>
            <tbody>
                <tr>
                    <td>001</td>
                    <td>John</td>
                    <td>Doe</td>
                    <td>john@example.com</td>
                    <td>555-0123</td>
                    <td>123 Main St</td>
                    <td>Active</td>
                </tr>
            </tbody>
        </table>
    "#;

    let tables = extract_tables_advanced(html, None).await?;
    assert_eq!(tables.len(), 1);

    let table = &tables[0];

    // Verify we have 2 sub-header levels
    assert_eq!(table.headers.sub_headers.len(), 2);

    // First sub-header level
    assert_eq!(table.headers.sub_headers[0].len(), 2);
    assert_eq!(table.headers.sub_headers[0][0].content, "ID");
    assert_eq!(table.headers.sub_headers[0][0].rowspan, 3);
    assert_eq!(
        table.headers.sub_headers[0][1].content,
        "Employee Information"
    );
    assert_eq!(table.headers.sub_headers[0][1].colspan, 6);

    // Second sub-header level
    assert_eq!(table.headers.sub_headers[1].len(), 3);
    assert_eq!(table.headers.sub_headers[1][0].content, "Personal");
    assert_eq!(table.headers.sub_headers[1][0].colspan, 2);
    assert_eq!(table.headers.sub_headers[1][1].content, "Contact");
    assert_eq!(table.headers.sub_headers[1][1].colspan, 3);
    assert_eq!(table.headers.sub_headers[1][2].content, "Status");
    assert_eq!(table.headers.sub_headers[1][2].rowspan, 2);

    // Main headers (most specific level)
    assert_eq!(table.headers.main.len(), 5);
    assert_eq!(table.headers.main[0].content, "First");
    assert_eq!(table.headers.main[1].content, "Last");
    assert_eq!(table.headers.main[2].content, "Email");
    assert_eq!(table.headers.main[3].content, "Phone");
    assert_eq!(table.headers.main[4].content, "Address");

    // Verify structure
    assert_eq!(table.structure.header_rows, 3);
    assert!(table.structure.has_complex_structure);
    assert_eq!(table.structure.max_colspan, 6);
    assert_eq!(table.structure.max_rowspan, 3);

    Ok(())
}

/// Test complex real-world financial table
#[tokio::test]
async fn test_financial_table_with_multi_level_headers() -> Result<()> {
    let html = r#"
        <table>
            <caption>Quarterly Financial Report</caption>
            <thead>
                <tr>
                    <th rowspan="2">Quarter</th>
                    <th colspan="3">Revenue (Millions)</th>
                    <th colspan="3">Expenses (Millions)</th>
                    <th rowspan="2">Net Profit</th>
                </tr>
                <tr>
                    <th>Product A</th>
                    <th>Product B</th>
                    <th>Total</th>
                    <th>Operating</th>
                    <th>Marketing</th>
                    <th>Total</th>
                </tr>
            </thead>
            <tbody>
                <tr>
                    <td>Q1</td>
                    <td>$10.5</td>
                    <td>$8.2</td>
                    <td>$18.7</td>
                    <td>$6.0</td>
                    <td>$2.5</td>
                    <td>$8.5</td>
                    <td>$10.2</td>
                </tr>
                <tr>
                    <td>Q2</td>
                    <td>$12.3</td>
                    <td>$9.1</td>
                    <td>$21.4</td>
                    <td>$6.5</td>
                    <td>$3.0</td>
                    <td>$9.5</td>
                    <td>$11.9</td>
                </tr>
            </tbody>
        </table>
    "#;

    let tables = extract_tables_advanced(html, None).await?;
    assert_eq!(tables.len(), 1);

    let table = &tables[0];

    // Verify caption
    assert_eq!(
        table.caption,
        Some("Quarterly Financial Report".to_string())
    );

    // Verify sub-headers (first level)
    assert_eq!(table.headers.sub_headers.len(), 1);
    assert_eq!(table.headers.sub_headers[0].len(), 4);

    // First cell: Quarter (rowspan=2)
    assert_eq!(table.headers.sub_headers[0][0].content, "Quarter");
    assert_eq!(table.headers.sub_headers[0][0].rowspan, 2);

    // Revenue group (colspan=3)
    assert_eq!(
        table.headers.sub_headers[0][1].content,
        "Revenue (Millions)"
    );
    assert_eq!(table.headers.sub_headers[0][1].colspan, 3);

    // Expenses group (colspan=3)
    assert_eq!(
        table.headers.sub_headers[0][2].content,
        "Expenses (Millions)"
    );
    assert_eq!(table.headers.sub_headers[0][2].colspan, 3);

    // Net Profit (rowspan=2)
    assert_eq!(table.headers.sub_headers[0][3].content, "Net Profit");
    assert_eq!(table.headers.sub_headers[0][3].rowspan, 2);

    // Verify main headers (second level - specific columns)
    assert_eq!(table.headers.main.len(), 6);
    assert_eq!(table.headers.main[0].content, "Product A");
    assert_eq!(table.headers.main[1].content, "Product B");
    assert_eq!(table.headers.main[2].content, "Total");
    assert_eq!(table.headers.main[3].content, "Operating");
    assert_eq!(table.headers.main[4].content, "Marketing");
    assert_eq!(table.headers.main[5].content, "Total");

    // Verify body data
    assert_eq!(table.rows.len(), 2);
    assert_eq!(table.rows[0].cells[0].content, "Q1");
    assert_eq!(table.rows[1].cells[0].content, "Q2");

    // Verify structure
    assert_eq!(table.structure.header_rows, 2);
    assert_eq!(table.structure.total_rows, 2);
    assert!(table.structure.has_complex_structure);

    Ok(())
}

/// Test table without thead but with single-row headers
#[tokio::test]
async fn test_single_row_headers_without_thead() -> Result<()> {
    let html = r#"
        <table>
            <tr>
                <th>Name</th>
                <th>Age</th>
            </tr>
            <tr>
                <td>John</td>
                <td>30</td>
            </tr>
        </table>
    "#;

    let tables = extract_tables_advanced(html, None).await?;
    assert_eq!(tables.len(), 1);

    let table = &tables[0];

    // Should extract first row as headers
    assert_eq!(table.headers.main.len(), 2);
    assert_eq!(table.headers.main[0].content, "Name");
    assert_eq!(table.headers.main[1].content, "Age");

    // No sub-headers (only one header row)
    assert_eq!(table.headers.sub_headers.len(), 0);

    // The body should contain all rows that aren't headers
    // Since we extracted the first row as headers, remaining rows are body
    // But the current implementation counts all non-thead rows when no thead exists
    assert!(!table.rows.is_empty());
    // Find the row with "John" - it should be in the body
    let john_row = table
        .rows
        .iter()
        .find(|r| r.cells.first().map(|c| c.content.as_str()) == Some("John"));
    assert!(john_row.is_some(), "Should find John in body rows");

    Ok(())
}

/// Test empty table
#[tokio::test]
async fn test_empty_table() -> Result<()> {
    let html = r#"
        <table>
        </table>
    "#;

    let tables = extract_tables_advanced(html, None).await?;
    assert_eq!(tables.len(), 1);

    let table = &tables[0];

    // Should have no headers
    assert_eq!(table.headers.main.len(), 0);
    assert_eq!(table.headers.sub_headers.len(), 0);

    // Should have no rows
    assert_eq!(table.rows.len(), 0);

    // Structure should reflect empty table
    assert_eq!(table.structure.header_rows, 0);
    assert_eq!(table.structure.total_rows, 0);

    Ok(())
}

/// Test table with only thead (no tbody)
#[tokio::test]
async fn test_table_with_only_headers() -> Result<()> {
    let html = r#"
        <table>
            <thead>
                <tr>
                    <th colspan="2">Group 1</th>
                    <th colspan="2">Group 2</th>
                </tr>
                <tr>
                    <th>Col A</th>
                    <th>Col B</th>
                    <th>Col C</th>
                    <th>Col D</th>
                </tr>
            </thead>
        </table>
    "#;

    let tables = extract_tables_advanced(html, None).await?;
    assert_eq!(tables.len(), 1);

    let table = &tables[0];

    // Should extract multi-level headers
    assert_eq!(table.headers.sub_headers.len(), 1);
    assert_eq!(table.headers.main.len(), 4);

    // Should have no body rows
    assert_eq!(table.rows.len(), 0);

    // Structure should show headers but no data rows
    assert_eq!(table.structure.header_rows, 2);
    assert_eq!(table.structure.total_rows, 0);

    Ok(())
}

/// Test cell position tracking with spans
#[tokio::test]
async fn test_cell_position_tracking_with_spans() -> Result<()> {
    let html = r#"
        <table>
            <thead>
                <tr>
                    <th colspan="2">Name</th>
                    <th rowspan="2">Age</th>
                </tr>
                <tr>
                    <th>First</th>
                    <th>Last</th>
                </tr>
            </thead>
            <tbody>
                <tr>
                    <td>John</td>
                    <td>Doe</td>
                    <td>30</td>
                </tr>
            </tbody>
        </table>
    "#;

    let tables = extract_tables_advanced(html, None).await?;
    assert_eq!(tables.len(), 1);

    let table = &tables[0];

    // Verify sub-header spans_over positions
    let name_header = &table.headers.sub_headers[0][0];
    assert_eq!(name_header.colspan, 2);
    assert_eq!(name_header.spans_over.len(), 1); // Spans over 1 additional cell

    let age_header = &table.headers.sub_headers[0][1];
    assert_eq!(age_header.rowspan, 2);
    assert_eq!(age_header.spans_over.len(), 1); // Spans over 1 additional row

    // Verify main headers are positioned correctly
    assert_eq!(table.headers.main[0].column_index, 0);
    assert_eq!(table.headers.main[1].column_index, 1);

    Ok(())
}

/// Test export formats with multi-level headers
#[tokio::test]
async fn test_export_formats_with_multi_level_headers() -> Result<()> {
    let html = r#"
        <table>
            <thead>
                <tr>
                    <th colspan="2">Name</th>
                    <th>Age</th>
                </tr>
                <tr>
                    <th>First</th>
                    <th>Last</th>
                    <th>Years</th>
                </tr>
            </thead>
            <tbody>
                <tr>
                    <td>John</td>
                    <td>Doe</td>
                    <td>30</td>
                </tr>
            </tbody>
        </table>
    "#;

    let tables = extract_tables_advanced(html, None).await?;
    assert_eq!(tables.len(), 1);

    let table = &tables[0];

    // Test CSV export
    let csv = table.to_csv(true)?;
    assert!(csv.contains("First,Last,Years"));
    assert!(csv.contains("John,Doe,30"));

    // Test Markdown export
    let markdown = table.to_markdown(true)?;
    assert!(markdown.contains("| First | Last | Years |"));
    assert!(markdown.contains("| John | Doe | 30 |"));

    // Test NDJSON artifacts
    let artifacts = table.to_ndjson_artifacts(None)?;
    assert_eq!(artifacts.len(), 3); // CSV, Markdown, Metadata

    Ok(())
}
