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
use riptide_extraction::table_extraction::{extract_tables_advanced, TableExtractionConfig};
use riptide_facade::facades::{FacadeTableExtractionOptions, TableFacade};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tracing::{debug, info};

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

/// Global table facade for table operations
static TABLE_FACADE: std::sync::OnceLock<TableFacade> = std::sync::OnceLock::new();

fn get_table_facade() -> &'static TableFacade {
    TABLE_FACADE.get_or_init(TableFacade::new)
}

/// Extract tables from HTML content
///
/// This endpoint processes HTML content and extracts structured table data using
/// riptide-extraction's advanced table extraction capabilities.
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

    // Use facade to store tables and create summaries
    let facade = get_table_facade();
    let facade_options = FacadeTableExtractionOptions {
        include_headers: options.include_headers,
        detect_data_types: options.detect_data_types,
    };

    let table_summaries = facade.store_and_summarize(tables, &facade_options).await;

    // Convert facade summaries to API summaries
    let table_summaries: Vec<TableSummary> = table_summaries
        .into_iter()
        .map(|s| TableSummary {
            id: s.id,
            rows: s.rows,
            columns: s.columns,
            headers: s.headers,
            data: s.data,
            metadata: TableMetadata {
                has_headers: s.metadata.has_headers,
                data_types: s.metadata.data_types,
                has_complex_structure: s.metadata.has_complex_structure,
                caption: s.metadata.caption,
                css_classes: s.metadata.css_classes,
                html_id: s.metadata.html_id,
            },
        })
        .collect();

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

    // Retrieve table from facade
    let facade = get_table_facade();
    let table = facade.get_table(&table_id).await.ok_or_else(|| {
        ApiError::not_found(format!("Table with ID '{}' not found or expired", table_id))
    })?;

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
