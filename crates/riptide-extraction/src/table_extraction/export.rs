//! Table export functionality with trait-based design
//!
//! This module provides extensible export capabilities for table data:
//! - CSV export (RFC 4180 compliant)
//! - Markdown export with metadata
//! - NDJSON artifacts for data interchange

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Write;

use super::models::{AdvancedTableData, TableExtractionError};

/// Trait for table export functionality
pub trait TableExporter {
    /// Export table to specific format
    fn export(&self, table: &AdvancedTableData) -> Result<String, TableExtractionError>;
}

/// CSV exporter with RFC 4180 compliance
pub struct CsvExporter {
    /// Include header row in output
    pub include_headers: bool,
}

impl CsvExporter {
    pub fn new(include_headers: bool) -> Self {
        Self { include_headers }
    }

    /// Format a single CSV row according to RFC 4180
    fn format_csv_row(&self, cells: &[&String]) -> Result<String, TableExtractionError> {
        let mut row = String::new();

        for (i, cell) in cells.iter().enumerate() {
            if i > 0 {
                row.push(',');
            }

            // RFC 4180 compliance: escape quotes and wrap in quotes if necessary
            if cell.contains(',')
                || cell.contains('"')
                || cell.contains('\n')
                || cell.contains('\r')
            {
                row.push('"');
                // Escape quotes by doubling them
                for ch in cell.chars() {
                    if ch == '"' {
                        row.push('"');
                        row.push('"');
                    } else {
                        row.push(ch);
                    }
                }
                row.push('"');
            } else {
                row.push_str(cell);
            }
        }

        Ok(row)
    }
}

impl TableExporter for CsvExporter {
    fn export(&self, table: &AdvancedTableData) -> Result<String, TableExtractionError> {
        let mut csv_output = String::new();

        // Add headers if present and requested
        if self.include_headers && !table.headers.main.is_empty() {
            let header_row = self.format_csv_row(
                &table
                    .headers
                    .main
                    .iter()
                    .map(|cell| &cell.content)
                    .collect::<Vec<_>>(),
            )?;
            csv_output.push_str(&header_row);
            csv_output.push('\n');
        }

        // Add body rows
        for row in &table.rows {
            let cell_contents: Vec<&String> = row.cells.iter().map(|cell| &cell.content).collect();
            let csv_row = self.format_csv_row(&cell_contents)?;
            csv_output.push_str(&csv_row);
            csv_output.push('\n');
        }

        // Add footer rows if present
        for row in &table.footer {
            let cell_contents: Vec<&String> = row.cells.iter().map(|cell| &cell.content).collect();
            let csv_row = self.format_csv_row(&cell_contents)?;
            csv_output.push_str(&csv_row);
            csv_output.push('\n');
        }

        Ok(csv_output.trim_end().to_string())
    }
}

/// Markdown exporter with metadata support
pub struct MarkdownExporter {
    /// Include metadata comments in output
    pub include_metadata: bool,
}

impl MarkdownExporter {
    pub fn new(include_metadata: bool) -> Self {
        Self { include_metadata }
    }

    /// Escape markdown special characters
    fn escape_markdown(&self, text: &str) -> String {
        text.replace('|', "\\|")
            .replace('\n', " ")
            .replace('\r', "")
    }
}

impl TableExporter for MarkdownExporter {
    fn export(&self, table: &AdvancedTableData) -> Result<String, TableExtractionError> {
        let mut md_output = String::new();

        // Add caption if present
        if let Some(caption) = &table.caption {
            writeln!(md_output, "*{}*\n", caption)
                .map_err(|e| TableExtractionError::ExportError(e.to_string()))?;
        }

        // Add metadata as comment if requested
        if self.include_metadata {
            writeln!(md_output, "<!-- Table ID: {} -->", table.id)
                .map_err(|e| TableExtractionError::ExportError(e.to_string()))?;
            if let Some(parent_id) = &table.parent_id {
                writeln!(md_output, "<!-- Parent Table: {} -->", parent_id)
                    .map_err(|e| TableExtractionError::ExportError(e.to_string()))?;
            }
            writeln!(
                md_output,
                "<!-- Columns: {}, Rows: {} -->",
                table.structure.total_columns, table.structure.total_rows
            )
            .map_err(|e| TableExtractionError::ExportError(e.to_string()))?;
            if table.structure.has_complex_structure {
                writeln!(
                    md_output,
                    "<!-- Note: Table has complex structure (spans) -->"
                )
                .map_err(|e| TableExtractionError::ExportError(e.to_string()))?;
            }
            writeln!(md_output).map_err(|e| TableExtractionError::ExportError(e.to_string()))?;
        }

        // Handle headers
        if !table.headers.main.is_empty() {
            // Header row
            write!(md_output, "|").map_err(|e| TableExtractionError::ExportError(e.to_string()))?;
            for cell in &table.headers.main {
                write!(md_output, " {} |", self.escape_markdown(&cell.content))
                    .map_err(|e| TableExtractionError::ExportError(e.to_string()))?;
            }
            writeln!(md_output).map_err(|e| TableExtractionError::ExportError(e.to_string()))?;

            // Separator row
            write!(md_output, "|").map_err(|e| TableExtractionError::ExportError(e.to_string()))?;
            for _ in &table.headers.main {
                write!(md_output, " --- |")
                    .map_err(|e| TableExtractionError::ExportError(e.to_string()))?;
            }
            writeln!(md_output).map_err(|e| TableExtractionError::ExportError(e.to_string()))?;
        }

        // Body rows
        for row in &table.rows {
            write!(md_output, "|").map_err(|e| TableExtractionError::ExportError(e.to_string()))?;
            for cell in &row.cells {
                let content = if cell.colspan > 1 || cell.rowspan > 1 {
                    format!(
                        "{} (span: {}x{})",
                        self.escape_markdown(&cell.content),
                        cell.colspan,
                        cell.rowspan
                    )
                } else {
                    self.escape_markdown(&cell.content)
                };
                write!(md_output, " {} |", content)
                    .map_err(|e| TableExtractionError::ExportError(e.to_string()))?;
            }
            writeln!(md_output).map_err(|e| TableExtractionError::ExportError(e.to_string()))?;
        }

        // Footer rows if present
        if !table.footer.is_empty() {
            writeln!(md_output).map_err(|e| TableExtractionError::ExportError(e.to_string()))?;
            writeln!(md_output, "**Footer:**")
                .map_err(|e| TableExtractionError::ExportError(e.to_string()))?;
            for row in &table.footer {
                write!(md_output, "|")
                    .map_err(|e| TableExtractionError::ExportError(e.to_string()))?;
                for cell in &row.cells {
                    write!(md_output, " {} |", self.escape_markdown(&cell.content))
                        .map_err(|e| TableExtractionError::ExportError(e.to_string()))?;
                }
                writeln!(md_output)
                    .map_err(|e| TableExtractionError::ExportError(e.to_string()))?;
            }
        }

        // Add nested tables reference if any
        if !table.nested_tables.is_empty() {
            writeln!(md_output).map_err(|e| TableExtractionError::ExportError(e.to_string()))?;
            writeln!(
                md_output,
                "**Nested Tables:** {}",
                table.nested_tables.join(", ")
            )
            .map_err(|e| TableExtractionError::ExportError(e.to_string()))?;
        }

        Ok(md_output)
    }
}

/// NDJSON artifact structure for data interchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableArtifact {
    /// Table reference
    pub table_id: String,
    /// Artifact type (csv, markdown, metadata)
    pub artifact_type: String,
    /// File path or content
    pub content: String,
    /// Artifact metadata
    pub metadata: HashMap<String, String>,
    /// Creation timestamp
    pub created_at: String,
}

/// NDJSON exporter for artifacts
pub struct NdjsonExporter {
    /// Optional base path for file references
    pub base_path: Option<String>,
}

impl NdjsonExporter {
    pub fn new(base_path: Option<String>) -> Self {
        Self { base_path }
    }

    /// Create NDJSON artifacts for a table
    pub fn create_artifacts(
        &self,
        table: &AdvancedTableData,
    ) -> Result<Vec<String>, TableExtractionError> {
        let mut artifacts = Vec::new();

        // CSV artifact
        let csv_exporter = CsvExporter::new(true);
        let csv_content = csv_exporter.export(table)?;
        let csv_artifact = TableArtifact {
            table_id: table.id.clone(),
            artifact_type: "csv".to_string(),
            content: if let Some(path) = &self.base_path {
                format!("{}/{}.csv", path, table.id)
            } else {
                csv_content
            },
            metadata: [
                ("format".to_string(), "RFC4180".to_string()),
                (
                    "headers".to_string(),
                    (!table.headers.main.is_empty()).to_string(),
                ),
                ("rows".to_string(), table.structure.total_rows.to_string()),
                (
                    "columns".to_string(),
                    table.structure.total_columns.to_string(),
                ),
            ]
            .into_iter()
            .collect(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        artifacts.push(
            serde_json::to_string(&csv_artifact)
                .map_err(|e| TableExtractionError::ExportError(e.to_string()))?,
        );

        // Markdown artifact
        let md_exporter = MarkdownExporter::new(true);
        let md_content = md_exporter.export(table)?;
        let md_artifact = TableArtifact {
            table_id: table.id.clone(),
            artifact_type: "markdown".to_string(),
            content: if let Some(path) = &self.base_path {
                format!("{}/{}.md", path, table.id)
            } else {
                md_content
            },
            metadata: [
                ("format".to_string(), "markdown".to_string()),
                ("has_metadata".to_string(), "true".to_string()),
                (
                    "complex_structure".to_string(),
                    table.structure.has_complex_structure.to_string(),
                ),
            ]
            .into_iter()
            .collect(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        artifacts.push(
            serde_json::to_string(&md_artifact)
                .map_err(|e| TableExtractionError::ExportError(e.to_string()))?,
        );

        // JSON metadata artifact
        let json_artifact = TableArtifact {
            table_id: table.id.clone(),
            artifact_type: "metadata".to_string(),
            content: if let Some(path) = &self.base_path {
                format!("{}/{}_metadata.json", path, table.id)
            } else {
                serde_json::to_string_pretty(table)
                    .map_err(|e| TableExtractionError::ExportError(e.to_string()))?
            },
            metadata: [
                ("format".to_string(), "json".to_string()),
                ("complete_structure".to_string(), "true".to_string()),
            ]
            .into_iter()
            .collect(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        artifacts.push(
            serde_json::to_string(&json_artifact)
                .map_err(|e| TableExtractionError::ExportError(e.to_string()))?,
        );

        Ok(artifacts)
    }
}

/// Convenience implementations for backward compatibility
impl AdvancedTableData {
    /// Export table to CSV format
    pub fn to_csv(&self, include_headers: bool) -> Result<String, TableExtractionError> {
        let exporter = CsvExporter::new(include_headers);
        exporter.export(self)
    }

    /// Export table to Markdown format
    pub fn to_markdown(&self, include_metadata: bool) -> Result<String, TableExtractionError> {
        let exporter = MarkdownExporter::new(include_metadata);
        exporter.export(self)
    }

    /// Create NDJSON artifacts for the table
    pub fn to_ndjson_artifacts(
        &self,
        base_path: Option<&str>,
    ) -> Result<Vec<String>, TableExtractionError> {
        let exporter = NdjsonExporter::new(base_path.map(|s| s.to_string()));
        exporter.create_artifacts(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table_extraction::models::{
        CellType, RowType, TableCell, TableHeaders, TableMetadata, TableRow, TableStructure,
    };
    use std::collections::HashMap;

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

    #[test]
    fn test_csv_export() {
        let table = create_test_table();
        let csv = table.to_csv(true).unwrap();

        assert!(csv.contains("Name,Age,City"));
        assert!(csv.contains("John,30,New York"));
        assert!(csv.contains("Jane,25,Los Angeles"));
    }

    #[test]
    fn test_markdown_export() {
        let table = create_test_table();
        let md = table.to_markdown(true).unwrap();

        assert!(md.contains("| Name | Age | City |"));
        assert!(md.contains("| --- | --- | --- |"));
        assert!(md.contains("| John | 30 | New York |"));
        assert!(md.contains("<!-- Table ID:"));
    }

    #[test]
    fn test_ndjson_artifacts() {
        let table = create_test_table();
        let artifacts = table.to_ndjson_artifacts(None).unwrap();

        assert_eq!(artifacts.len(), 3); // CSV, Markdown, Metadata

        for artifact in artifacts {
            let artifact_obj: TableArtifact = serde_json::from_str(&artifact).unwrap();
            assert_eq!(artifact_obj.table_id, table.id);
            assert!(["csv", "markdown", "metadata"].contains(&artifact_obj.artifact_type.as_str()));
        }
    }

    #[test]
    fn test_csv_exporter_trait() {
        let table = create_test_table();
        let exporter = CsvExporter::new(true);
        let csv = exporter.export(&table).unwrap();

        assert!(csv.contains("Name,Age,City"));
        assert!(csv.contains("John,30,New York"));
    }

    #[test]
    fn test_markdown_exporter_trait() {
        let table = create_test_table();
        let exporter = MarkdownExporter::new(false);
        let md = exporter.export(&table).unwrap();

        assert!(md.contains("| Name | Age | City |"));
        assert!(!md.contains("<!-- Table ID:")); // metadata disabled
    }

    #[test]
    fn test_ndjson_exporter_with_base_path() {
        let table = create_test_table();
        let exporter = NdjsonExporter::new(Some("/tmp/exports".to_string()));
        let artifacts = exporter.create_artifacts(&table).unwrap();

        assert_eq!(artifacts.len(), 3);

        let csv_artifact: TableArtifact = serde_json::from_str(&artifacts[0]).unwrap();
        assert!(csv_artifact.content.starts_with("/tmp/exports/"));
        assert!(csv_artifact.content.ends_with(".csv"));
    }
}
