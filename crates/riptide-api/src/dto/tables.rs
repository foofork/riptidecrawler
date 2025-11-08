//! Tables API DTOs - Request/Response types for table extraction
//!
//! Extracted from handlers/tables.rs (Phase 3 Sprint 3.1)
//! Contains all 4 DTOs + helper functions

use riptide_facade::facades::{FacadeTableSummary, TableExtractionRequest};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ApiTableRequest {
    pub html_content: String,
    pub extract_options: Option<TableOptions>,
}

impl ApiTableRequest {
    pub fn to_facade_request(&self) -> TableExtractionRequest {
        let opts = self.extract_options.as_ref().cloned().unwrap_or_default();
        TableExtractionRequest {
            html_content: self.html_content.clone(),
            include_nested: opts.include_nested,
            preserve_html: opts.preserve_formatting,
            max_nesting_depth: opts.max_nesting_depth,
            min_size: opts.min_size,
            headers_only: opts.headers_only,
            include_headers: opts.include_headers,
            detect_data_types: opts.detect_data_types,
        }
    }
}

#[derive(Deserialize, Clone, Default)]
pub struct TableOptions {
    #[serde(default = "default_true")]
    pub include_headers: bool,
    #[serde(default)]
    pub preserve_formatting: bool,
    #[serde(default)]
    pub detect_data_types: bool,
    #[serde(default = "default_true")]
    pub include_nested: bool,
    #[serde(default = "default_max_nesting")]
    pub max_nesting_depth: usize,
    pub min_size: Option<(usize, usize)>,
    #[serde(default)]
    pub headers_only: bool,
}

#[derive(Serialize)]
pub struct TableResponse {
    pub tables: Vec<FacadeTableSummary>,
    pub extraction_time_ms: u64,
    pub total_tables: usize,
}

#[derive(Deserialize)]
pub struct ExportQuery {
    pub format: String,
    #[serde(default = "default_true")]
    pub include_headers: bool,
    #[serde(default)]
    pub include_metadata: bool,
}

fn default_true() -> bool {
    true
}

fn default_max_nesting() -> usize {
    3
}
