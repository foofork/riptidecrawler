//! Table extraction data models and types
//!
//! This module contains all data structures used for advanced table extraction,
//! including table metadata, cell information, and configuration options.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Advanced table data structure with full metadata support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedTableData {
    /// Unique table identifier
    pub id: String,
    /// Table headers organized by sections
    pub headers: TableHeaders,
    /// Table body rows with cell metadata
    pub rows: Vec<TableRow>,
    /// Table footer rows (if present)
    pub footer: Vec<TableRow>,
    /// Table caption (if present)
    pub caption: Option<String>,
    /// Table-level metadata and attributes
    pub metadata: TableMetadata,
    /// Parent table ID for nested tables
    pub parent_id: Option<String>,
    /// Nested tables contained within this table
    pub nested_tables: Vec<String>,
    /// Table structure information
    pub structure: TableStructure,
}

/// Table headers organized by sections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableHeaders {
    /// Main header row(s)
    pub main: Vec<TableCell>,
    /// Sub-headers (multi-level headers)
    pub sub_headers: Vec<Vec<TableCell>>,
    /// Column groups information
    pub column_groups: Vec<ColumnGroup>,
}

/// Individual table row
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRow {
    /// Row cells
    pub cells: Vec<TableCell>,
    /// Row-level attributes
    pub attributes: HashMap<String, String>,
    /// Row type (header, body, footer)
    pub row_type: RowType,
    /// Row index within its section
    pub index: usize,
}

/// Individual table cell with full metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCell {
    /// Cell content (text)
    pub content: String,
    /// Raw HTML content
    pub html_content: String,
    /// Column span
    pub colspan: usize,
    /// Row span
    pub rowspan: usize,
    /// Cell type (header, data)
    pub cell_type: CellType,
    /// Cell-level attributes
    pub attributes: HashMap<String, String>,
    /// Column index (accounting for spans)
    pub column_index: usize,
    /// Row index (accounting for spans)
    pub row_index: usize,
    /// Cells that this cell spans over
    pub spans_over: Vec<CellPosition>,
}

/// Column group information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnGroup {
    /// Column span
    pub span: usize,
    /// Group attributes
    pub attributes: HashMap<String, String>,
    /// Group label/title
    pub label: Option<String>,
}

/// Table metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableMetadata {
    /// All table attributes
    pub attributes: HashMap<String, String>,
    /// CSS classes
    pub classes: Vec<String>,
    /// Table ID attribute
    pub id: Option<String>,
    /// Processing timestamp
    pub processed_at: String,
    /// Source URL or document reference
    pub source: Option<String>,
}

/// Table structure information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableStructure {
    /// Total number of columns (accounting for spans)
    pub total_columns: usize,
    /// Total number of rows in body
    pub total_rows: usize,
    /// Number of header rows
    pub header_rows: usize,
    /// Number of footer rows
    pub footer_rows: usize,
    /// Has complex structure (spans, nested tables)
    pub has_complex_structure: bool,
    /// Maximum colspan value
    pub max_colspan: usize,
    /// Maximum rowspan value
    pub max_rowspan: usize,
}

/// Row type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RowType {
    Header,
    Body,
    Footer,
}

/// Cell type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CellType {
    Header,
    Data,
}

/// Cell position for span tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellPosition {
    pub row: usize,
    pub column: usize,
}

/// Table extraction configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableExtractionConfig {
    /// Include nested tables
    pub include_nested: bool,
    /// Preserve HTML formatting in cells
    pub preserve_html: bool,
    /// Maximum nesting depth for tables
    pub max_nesting_depth: usize,
    /// Minimum table size (rows x cols)
    pub min_size: Option<(usize, usize)>,
    /// Extract only tables with headers
    pub headers_only: bool,
    /// Custom table selector
    pub custom_selector: Option<String>,
}

impl Default for TableExtractionConfig {
    fn default() -> Self {
        Self {
            include_nested: true,
            preserve_html: false,
            max_nesting_depth: 3,
            min_size: None,
            headers_only: false,
            custom_selector: None,
        }
    }
}

/// Error types for table extraction
#[derive(Debug, thiserror::Error)]
pub enum TableExtractionError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Invalid table structure: {0}")]
    InvalidStructure(String),

    #[error("Export format error: {0}")]
    ExportError(String),

    #[error("Nesting depth exceeded: {max_depth}")]
    NestingDepthExceeded { max_depth: usize },

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Table artifact for NDJSON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableArtifact {
    /// Table reference
    pub table_id: String,
    /// Artifact type (csv, markdown)
    pub artifact_type: String,
    /// File path or content
    pub content: String,
    /// Artifact metadata
    pub metadata: HashMap<String, String>,
    /// Creation timestamp
    pub created_at: String,
}
