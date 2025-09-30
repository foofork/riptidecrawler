//! Advanced table extraction with metadata and export capabilities
//!
//! This module implements comprehensive table extraction capabilities including:
//! - TABLE-001: Complete HTML table parser with thead/tbody/tfoot sections
//! - TABLE-002: RFC 4180 compliant CSV export
//! - TABLE-003: Markdown table export with merged cell handling
//! - TABLE-004: NDJSON artifacts linking
//!
//! # Features
//! - Full support for colspan/rowspan attributes
//! - Nested table detection with parent_id tracking
//! - Proper cell merging logic
//! - Multiple export formats (CSV, Markdown, NDJSON)
//! - Production-ready error handling
//!
//! # Usage
//! ```rust
//! use riptide_html::table_extraction::{extract_tables_advanced, TableExtractionConfig};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let html = r#"<table><tr><th>Name</th><th>Age</th></tr><tr><td>John</td><td>30</td></tr></table>"#;
//! let tables = extract_tables_advanced(html, None).await?;
//!
//! for table in tables {
//!     println!("Table ID: {}", table.id);
//!     let csv = table.to_csv(true)?;
//!     let markdown = table.to_markdown(true)?;
//! }
//! # Ok(())
//! # }
//! ```

pub mod models;
pub mod extractor;
pub mod export;

// Re-export main public API
pub use models::{
    TableExtractionConfig, TableDetectionStrategy, ExtractionQuality,
    AdvancedTableData, TableCell, CellSpan, TableMetadata, TableStats,
    TableSection, TableExtractionError, TableHeaders, TableRow, TableStructure,
    CellType, RowType, CellPosition, ColumnGroup
};
pub use extractor::TableExtractor;
pub use export::{TableExporter, CsvExporter, MarkdownExporter, NdjsonExporter, TableArtifact};

use anyhow::Result;

/// Convenience function for easy table extraction
pub async fn extract_tables_advanced(
    html: &str,
    config: Option<TableExtractionConfig>,
) -> Result<Vec<AdvancedTableData>> {
    let config = config.unwrap_or_default();
    let extractor = TableExtractor::new(html, config);
    extractor.extract_all_tables()
}

/// Extract tables and export to all formats
pub async fn extract_and_export_tables(
    html: &str,
    base_path: Option<&str>,
    config: Option<TableExtractionConfig>,
) -> Result<(Vec<AdvancedTableData>, Vec<String>)> {
    let tables = extract_tables_advanced(html, config).await?;
    let mut all_artifacts = Vec::new();

    for table in &tables {
        let artifacts = table.to_ndjson_artifacts(base_path)?;
        all_artifacts.extend(artifacts);
    }

    Ok((tables, all_artifacts))
}

#[cfg(test)]
mod tests {
    use super::*;
    use models::{CellType, RowType};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_basic_table_extraction() {
        let html = r#"
            <table id="test-table" class="data-table">
                <caption>Test Table</caption>
                <thead>
                    <tr><th>Name</th><th>Age</th><th>City</th></tr>
                </thead>
                <tbody>
                    <tr><td>John</td><td>30</td><td>New York</td></tr>
                    <tr><td>Jane</td><td>25</td><td>Los Angeles</td></tr>
                </tbody>
                <tfoot>
                    <tr><td>Total</td><td>2</td><td>-</td></tr>
                </tfoot>
            </table>
        "#;

        let tables = extract_tables_advanced(html, None).await.unwrap();
        assert_eq!(tables.len(), 1);

        let table = &tables[0];
        assert_eq!(table.headers.main.len(), 3);
        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.footer.len(), 1);
        assert_eq!(table.caption, Some("Test Table".to_string()));
        assert_eq!(table.metadata.id, Some("test-table".to_string()));
        assert!(table.metadata.classes.contains(&"data-table".to_string()));
    }

    #[tokio::test]
    async fn test_complex_table_with_spans() {
        let html = r#"
            <table>
                <tr>
                    <th colspan="2">Name</th>
                    <th rowspan="2">Age</th>
                </tr>
                <tr>
                    <th>First</th>
                    <th>Last</th>
                </tr>
                <tr>
                    <td>John</td>
                    <td>Doe</td>
                    <td>30</td>
                </tr>
            </table>
        "#;

        let tables = extract_tables_advanced(html, None).await.unwrap();
        assert_eq!(tables.len(), 1);

        let table = &tables[0];
        assert!(table.structure.has_complex_structure);
        assert_eq!(table.structure.max_colspan, 2);
        assert_eq!(table.structure.max_rowspan, 2);
    }

    #[tokio::test]
    async fn test_nested_tables() {
        let html = r#"
            <table id="outer">
                <tr>
                    <td>Cell 1</td>
                    <td>
                        <table id="inner">
                            <tr><td>Nested 1</td><td>Nested 2</td></tr>
                        </table>
                    </td>
                </tr>
            </table>
        "#;

        let config = TableExtractionConfig {
            include_nested: true,
            ..Default::default()
        };

        let tables = extract_tables_advanced(html, Some(config)).await.unwrap();

        // We should find 2 tables total: outer and inner
        assert_eq!(tables.len(), 2);

        // Find the outer table (should have nested_tables)
        let outer_table = tables.iter().find(|t| !t.nested_tables.is_empty())
            .expect("Should find outer table with nested tables");

        // Find the inner table (should be standalone)
        let inner_table = tables.iter().find(|t| t.nested_tables.is_empty() && t.id != outer_table.id)
            .expect("Should find inner table");

        // Verify nested table detection worked
        assert!(!outer_table.nested_tables.is_empty());
        assert_eq!(outer_table.nested_tables.len(), 1);

        // Verify inner table structure
        assert_eq!(inner_table.rows.len(), 1);
        assert_eq!(inner_table.rows[0].cells.len(), 2);
        assert_eq!(inner_table.rows[0].cells[0].content, "Nested 1");
        assert_eq!(inner_table.rows[0].cells[1].content, "Nested 2");
    }

    #[tokio::test]
    async fn test_extract_and_export() {
        let html = r#"
            <table>
                <tr><th>Product</th><th>Price</th></tr>
                <tr><td>Apple</td><td>$1.50</td></tr>
                <tr><td>Banana</td><td>$0.75</td></tr>
            </table>
        "#;

        let (tables, artifacts) = extract_and_export_tables(html, None, None).await.unwrap();

        assert_eq!(tables.len(), 1);
        assert_eq!(artifacts.len(), 3); // CSV, Markdown, Metadata

        // Verify artifacts are valid JSON
        for artifact in artifacts {
            let _: export::TableArtifact = serde_json::from_str(&artifact).unwrap();
        }
    }

    fn create_test_table() -> AdvancedTableData {
        AdvancedTableData {
            id: "test_1".to_string(),
            headers: TableHeaders {
                main: vec![
                    create_test_cell("Name", CellType::Header, 0, 0),
                    create_test_cell("Age", CellType::Header, 0, 1),
                    create_test_cell("City", CellType::Header, 0, 2),
                ],
                sub_headers: Vec::new(),
                column_groups: Vec::new(),
            },
            rows: vec![
                TableRow {
                    cells: vec![
                        create_test_cell("John", CellType::Data, 1, 0),
                        create_test_cell("30", CellType::Data, 1, 1),
                        create_test_cell("New York", CellType::Data, 1, 2),
                    ],
                    attributes: HashMap::new(),
                    row_type: RowType::Body,
                    index: 0,
                },
                TableRow {
                    cells: vec![
                        create_test_cell("Jane", CellType::Data, 2, 0),
                        create_test_cell("25", CellType::Data, 2, 1),
                        create_test_cell("Los Angeles", CellType::Data, 2, 2),
                    ],
                    attributes: HashMap::new(),
                    row_type: RowType::Body,
                    index: 1,
                },
            ],
            footer: Vec::new(),
            caption: Some("Test Table".to_string()),
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
                total_columns: 3,
                total_rows: 2,
                header_rows: 1,
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
}