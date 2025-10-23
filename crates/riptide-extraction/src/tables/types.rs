//! Data types for table extraction

use serde::{Deserialize, Serialize};

/// Request to extract tables from HTML content
#[derive(Serialize, Clone)]
pub struct TableExtractRequest {
    pub html_content: String,
}

/// Summary information about an extracted table from API
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TableSummary {
    /// Unique table identifier for export operations
    pub id: String,
    /// Number of rows (count, not data)
    pub rows: usize,
    /// Number of columns (count, not data)
    pub columns: usize,
    /// Table headers (if present) - sample data only
    #[serde(default)]
    pub headers: Vec<String>,
    /// Sample data (first few rows) - not full data
    #[serde(default)]
    pub data: Vec<Vec<String>>,
    /// Table metadata
    #[serde(default)]
    pub metadata: TableMetadata,
}

/// Table metadata from API
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct TableMetadata {
    /// Whether table has headers
    #[serde(default)]
    pub has_headers: bool,
    /// Detected data types for columns
    #[serde(default)]
    pub data_types: Vec<String>,
    /// Whether table has complex structure (spans)
    #[serde(default)]
    pub has_complex_structure: bool,
    /// Table caption (if present)
    #[serde(default)]
    pub caption: Option<String>,
    /// CSS classes from original HTML
    #[serde(default)]
    pub css_classes: Vec<String>,
    /// Table ID from HTML (if present)
    #[serde(default)]
    pub html_id: Option<String>,
}

/// Response from table extraction API
#[derive(Deserialize, Serialize, Debug)]
pub struct TableExtractResponse {
    /// Extracted table summaries with IDs
    pub tables: Vec<TableSummary>,
    /// Total extraction time in milliseconds
    #[serde(default)]
    pub extraction_time_ms: u64,
    /// Total number of tables found
    #[serde(default)]
    pub total_tables: usize,
}

/// Full table data structure (used after export)
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TableData {
    pub id: String,
    pub rows: usize,
    pub columns: usize,
    pub headers: Vec<String>,
    pub data: Vec<Vec<String>>,
    #[serde(default)]
    pub caption: Option<String>,
}
