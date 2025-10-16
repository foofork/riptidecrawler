//! Table extraction API handlers
//!
//! This module implements table extraction endpoints that utilize riptide-extraction's
//! advanced table extraction capabilities to extract structured data from HTML content.

use crate::errors::ApiError;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use riptide_extraction::table_extraction::{
    extract_tables_advanced, AdvancedTableData, TableExtractionConfig,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info};
use uuid::Uuid;

/// Request body for table extraction
#[derive(Deserialize, Debug, Clone)]
pub struct TableExtractionRequest {
    /// HTML content to extract tables from
    pub html_content: String,
    /// Optional extraction configuration
    pub extract_options: Option<TableExtractionOptions>,
}

/// Table extraction configuration options
#[derive(Deserialize, Debug, Clone)]
pub struct TableExtractionOptions {
    /// Include headers in extraction (used in CSV conversion line 334)
    #[serde(default = "default_true")]
    pub include_headers: bool,
    /// Preserve HTML formatting in cells
    #[serde(default)]
    pub preserve_formatting: bool,
    /// Detect data types in columns (wired to detect_column_types() on line 228)
    #[serde(default)]
    pub detect_data_types: bool,
    /// Include nested tables
    #[serde(default = "default_true")]
    pub include_nested: bool,
    /// Maximum nesting depth
    #[serde(default = "default_max_nesting")]
    pub max_nesting_depth: usize,
    /// Minimum table size (rows, columns)
    pub min_size: Option<(usize, usize)>,
    /// Extract only tables with headers
    #[serde(default)]
    pub headers_only: bool,
}

fn default_true() -> bool {
    true
}
fn default_max_nesting() -> usize {
    3
}

/// Response for table extraction
#[derive(Serialize, Debug)]
pub struct TableExtractionResponse {
    /// Extracted tables
    pub tables: Vec<TableSummary>,
    /// Total extraction time in milliseconds
    pub extraction_time_ms: u64,
    /// Number of tables found
    pub total_tables: usize,
}

/// Summary information about an extracted table
#[derive(Serialize, Debug)]
pub struct TableSummary {
    /// Unique table identifier for export operations
    pub id: String,
    /// Number of rows (excluding headers)
    pub rows: usize,
    /// Number of columns
    pub columns: usize,
    /// Table headers (if present)
    pub headers: Vec<String>,
    /// Sample data (first few rows)
    pub data: Vec<Vec<String>>,
    /// Table metadata
    pub metadata: TableMetadata,
}

/// Table metadata for API responses
#[derive(Serialize, Debug)]
pub struct TableMetadata {
    /// Whether table has headers
    pub has_headers: bool,
    /// Detected data types for columns
    pub data_types: Vec<String>,
    /// Whether table has complex structure (spans)
    pub has_complex_structure: bool,
    /// Table caption (if present)
    pub caption: Option<String>,
    /// CSS classes from original HTML
    pub css_classes: Vec<String>,
    /// Table ID from HTML (if present)
    pub html_id: Option<String>,
}

/// Query parameters for table export
#[derive(Deserialize, Debug)]
pub struct ExportQuery {
    /// Export format: "csv" or "markdown"
    pub format: String,
    /// Include headers in export
    #[serde(default = "default_true")]
    pub include_headers: bool,
    /// Include metadata in export (markdown only)
    #[serde(default)]
    pub include_metadata: bool,
}

/// In-memory storage for extracted tables (temporary)
/// In production, this would be replaced with Redis or database storage
static TABLE_STORAGE: std::sync::OnceLock<
    Arc<tokio::sync::Mutex<HashMap<String, AdvancedTableData>>>,
> = std::sync::OnceLock::new();

fn get_table_storage() -> Arc<tokio::sync::Mutex<HashMap<String, AdvancedTableData>>> {
    TABLE_STORAGE
        .get_or_init(|| Arc::new(tokio::sync::Mutex::new(HashMap::new())))
        .clone()
}

/// Extract tables from HTML content
///
/// This endpoint processes HTML content and extracts structured table data using
/// riptide-html's advanced table extraction capabilities.
///
/// ## Request
/// - `html_content`: The HTML content to process
/// - `extract_options`: Optional configuration for extraction behavior
///
/// ## Response
/// - Returns extracted table summaries with unique IDs for export
/// - Tables are temporarily stored for export operations
/// - Includes processing time and metadata
pub async fn extract_tables(
    State(state): State<AppState>,
    Json(request): Json<TableExtractionRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let start_time = Instant::now();

    info!(
        html_length = request.html_content.len(),
        "Received table extraction request"
    );

    // Validate input
    if request.html_content.trim().is_empty() {
        return Err(ApiError::validation(
            "HTML content cannot be empty".to_string(),
        ));
    }

    if request.html_content.len() > 10_000_000 {
        return Err(ApiError::validation(
            "HTML content too large (max 10MB)".to_string(),
        ));
    }

    // Configure extraction options
    let options = request.extract_options.unwrap_or_default();
    let config = TableExtractionConfig {
        include_nested: options.include_nested,
        preserve_html: options.preserve_formatting,
        max_nesting_depth: options.max_nesting_depth,
        min_size: options.min_size,
        headers_only: options.headers_only,
        custom_selector: None,
    };

    debug!(
        include_nested = config.include_nested,
        preserve_html = config.preserve_html,
        max_nesting_depth = config.max_nesting_depth,
        "Using table extraction configuration"
    );

    // Extract tables using riptide-extraction
    let tables = extract_tables_advanced(&request.html_content, Some(config))
        .await
        .map_err(|e| {
            state.metrics.record_error(crate::metrics::ErrorType::Wasm);
            ApiError::internal(format!("Table extraction failed: {}", e))
        })?;

    info!(tables_found = tables.len(), "Table extraction completed");

    // Store tables temporarily for export and create summaries
    let storage = get_table_storage();
    let mut storage_guard = storage.lock().await;
    let mut table_summaries = Vec::new();

    for table in tables {
        // Generate unique ID for this extraction session
        let export_id = Uuid::new_v4().to_string();

        // Get sample data and headers based on options
        let (headers, sample_data) = if options.include_headers {
            let headers: Vec<String> = table
                .headers
                .main
                .iter()
                .map(|cell| cell.content.clone())
                .collect();

            let sample_data: Vec<Vec<String>> = table
                .rows
                .iter()
                .take(3)
                .map(|row| row.cells.iter().map(|cell| cell.content.clone()).collect())
                .collect();

            (headers, sample_data)
        } else {
            (vec![], vec![])
        };

        // Detect data types if enabled
        let data_types = if options.detect_data_types {
            detect_column_types(&table)
        } else {
            vec![]
        };

        let summary = TableSummary {
            id: export_id.clone(),
            rows: table.structure.total_rows,
            columns: table.structure.total_columns,
            headers: headers.clone(),
            data: sample_data,
            metadata: TableMetadata {
                has_headers: !headers.is_empty(),
                data_types,
                has_complex_structure: table.structure.has_complex_structure,
                caption: table.caption.clone(),
                css_classes: table.metadata.classes.clone(),
                html_id: table.metadata.id.clone(),
            },
        };

        // Store the full table data for export
        storage_guard.insert(export_id, table);
        table_summaries.push(summary);
    }

    drop(storage_guard);

    let processing_time_ms = start_time.elapsed().as_millis() as u64;
    let total_tables = table_summaries.len();

    let response = TableExtractionResponse {
        tables: table_summaries,
        extraction_time_ms: processing_time_ms,
        total_tables,
    };

    info!(
        total_tables = response.total_tables,
        processing_time_ms = processing_time_ms,
        "Table extraction request completed"
    );

    // Record metrics
    state.metrics.record_http_request(
        "POST",
        "/api/v1/tables/extract",
        200,
        start_time.elapsed().as_secs_f64(),
    );

    Ok((StatusCode::OK, Json(response)))
}

/// Export extracted table in specified format
///
/// This endpoint exports a previously extracted table in CSV or Markdown format.
/// Tables must be extracted first using the `/extract` endpoint.
///
/// ## Path Parameters
/// - `id`: The table ID returned from the extraction endpoint
///
/// ## Query Parameters
/// - `format`: Export format ("csv" or "markdown")
/// - `include_headers`: Whether to include headers (default: true)
/// - `include_metadata`: Include metadata in export (markdown only, default: false)
pub async fn export_table(
    State(state): State<AppState>,
    Path(table_id): Path<String>,
    Query(params): Query<ExportQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let start_time = Instant::now();

    info!(
        table_id = %table_id,
        format = %params.format,
        "Received table export request"
    );

    // Validate format
    if !["csv", "markdown"].contains(&params.format.as_str()) {
        return Err(ApiError::validation(
            "Format must be 'csv' or 'markdown'".to_string(),
        ));
    }

    // Retrieve table from storage
    let storage = get_table_storage();
    let storage_guard = storage.lock().await;
    let table = storage_guard
        .get(&table_id)
        .ok_or_else(|| {
            ApiError::not_found(format!("Table with ID '{}' not found or expired", table_id))
        })?
        .clone();
    drop(storage_guard);

    debug!(
        table_id = %table_id,
        format = %params.format,
        rows = table.structure.total_rows,
        columns = table.structure.total_columns,
        "Exporting table"
    );

    // Export based on format
    let (content, content_type) = match params.format.as_str() {
        "csv" => {
            let csv_content = table.to_csv(params.include_headers).map_err(|e| {
                state.metrics.record_error(crate::metrics::ErrorType::Wasm);
                ApiError::internal(format!("CSV export failed: {}", e))
            })?;
            (csv_content, "text/csv")
        }
        "markdown" => {
            let md_content = table.to_markdown(params.include_metadata).map_err(|e| {
                state.metrics.record_error(crate::metrics::ErrorType::Wasm);
                ApiError::internal(format!("Markdown export failed: {}", e))
            })?;
            (md_content, "text/markdown")
        }
        _ => unreachable!("Format validation should prevent this"),
    };

    let processing_time_ms = start_time.elapsed().as_millis();

    info!(
        table_id = %table_id,
        format = %params.format,
        content_length = content.len(),
        processing_time_ms = processing_time_ms,
        "Table export completed"
    );

    // Record metrics
    state.metrics.record_http_request(
        "GET",
        &format!("/api/v1/tables/{}/export", table_id),
        200,
        start_time.elapsed().as_secs_f64(),
    );

    let extension = if params.format == "csv" { "csv" } else { "md" };
    let disposition = format!("attachment; filename=\"table_{}.{}\"", table_id, extension);

    use axum::response::IntoResponse;
    Ok((
        StatusCode::OK,
        [
            (axum::http::header::CONTENT_TYPE, content_type.to_string()),
            (axum::http::header::CONTENT_DISPOSITION, disposition),
        ],
        content,
    )
        .into_response())
}

/// Detect column data types from table data (simplified implementation)
fn detect_column_types(table: &AdvancedTableData) -> Vec<String> {
    let mut column_types = Vec::new();

    if table.rows.is_empty() {
        return column_types;
    }

    let num_columns = table.structure.total_columns;

    for col_index in 0..num_columns {
        let mut sample_values = Vec::new();

        // Collect sample values from this column
        for row in table.rows.iter().take(10) {
            // Sample first 10 rows
            if let Some(cell) = row.cells.get(col_index) {
                sample_values.push(&cell.content);
            }
        }

        // Detect type based on sample values
        let detected_type = detect_type_from_samples(&sample_values);
        column_types.push(detected_type);
    }

    column_types
}

/// Detect data type from sample values
fn detect_type_from_samples(samples: &[&String]) -> String {
    if samples.is_empty() {
        return "unknown".to_string();
    }

    let mut numeric_count = 0;
    let mut date_count = 0;
    let mut boolean_count = 0;

    for &sample in samples {
        let trimmed = sample.trim();

        if trimmed.is_empty() {
            continue;
        }

        // Check for boolean
        if ["true", "false", "yes", "no", "1", "0"].contains(&trimmed.to_lowercase().as_str()) {
            boolean_count += 1;
        }
        // Check for numeric (integer or float)
        else if trimmed.parse::<f64>().is_ok() {
            numeric_count += 1;
        }
        // Check for date-like patterns (simplified)
        else if is_date_like(trimmed) {
            date_count += 1;
        }
    }

    let total_samples = samples.len();
    let threshold = (total_samples as f64 * 0.7) as usize; // 70% threshold

    if numeric_count >= threshold {
        "number"
    } else if date_count >= threshold {
        "date"
    } else if boolean_count >= threshold {
        "boolean"
    } else {
        "string"
    }
    .to_string()
}

/// Simple date detection (basic patterns)
fn is_date_like(text: &str) -> bool {
    // Very basic date pattern detection
    use regex::Regex;

    let date_patterns = [
        r"\d{4}-\d{2}-\d{2}",       // YYYY-MM-DD
        r"\d{2}/\d{2}/\d{4}",       // MM/DD/YYYY
        r"\d{2}-\d{2}-\d{4}",       // MM-DD-YYYY
        r"\d{1,2}/\d{1,2}/\d{2,4}", // M/D/YY or MM/DD/YYYY
    ];

    for pattern in &date_patterns {
        if let Ok(regex) = Regex::new(pattern) {
            if regex.is_match(text) {
                return true;
            }
        }
    }

    false
}

impl Default for TableExtractionOptions {
    fn default() -> Self {
        Self {
            include_headers: true,
            preserve_formatting: false,
            detect_data_types: false,
            include_nested: true,
            max_nesting_depth: 3,
            min_size: None,
            headers_only: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_type_from_samples() {
        // Test numeric detection
        let s1 = "123".to_string();
        let s2 = "45.6".to_string();
        let s3 = "0".to_string();
        let numeric_samples = vec![&s1, &s2, &s3];
        assert_eq!(detect_type_from_samples(&numeric_samples), "number");

        // Test string detection
        let s4 = "hello".to_string();
        let s5 = "world".to_string();
        let s6 = "test".to_string();
        let string_samples = vec![&s4, &s5, &s6];
        assert_eq!(detect_type_from_samples(&string_samples), "string");

        // Test boolean detection
        let s7 = "true".to_string();
        let s8 = "false".to_string();
        let s9 = "yes".to_string();
        let boolean_samples = vec![&s7, &s8, &s9];
        assert_eq!(detect_type_from_samples(&boolean_samples), "boolean");

        // Test empty samples
        let empty_samples = vec![];
        assert_eq!(detect_type_from_samples(&empty_samples), "unknown");
    }

    #[test]
    fn test_is_date_like() {
        assert!(is_date_like("2023-12-25"));
        assert!(is_date_like("12/25/2023"));
        assert!(is_date_like("25-12-2023"));
        assert!(!is_date_like("hello world"));
        assert!(!is_date_like("123"));
    }
}
