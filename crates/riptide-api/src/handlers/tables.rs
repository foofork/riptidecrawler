//! Ultra-thin table extraction API handlers
//!
//! Handlers delegate all business logic to riptide-facade::TableFacade.
//! Responsible only for HTTP mapping, metrics, and error conversion.

use crate::{context::ApplicationContext, dto::tables::*, errors::ApiError};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use riptide_facade::facades::{TableFacade, TableFormat};
use std::time::Instant;

static FACADE: std::sync::OnceLock<TableFacade> = std::sync::OnceLock::new();
fn facade() -> &'static TableFacade {
    FACADE.get_or_init(TableFacade::new)
}

/// Extract tables from HTML (7 LOC)
pub async fn extract_tables(
    State(state): State<ApplicationContext>,
    Json(req): Json<ApiTableRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let start = Instant::now();
    let tables = facade()
        .extract_tables_full(req.to_facade_request())
        .await
        .map_err(|e| {
            state.transport_metrics.record_wasm_error();
            ApiError::from(e)
        })?;
    state.record_http_request(
        "POST",
        "/api/v1/tables/extract",
        200,
        start.elapsed().as_secs_f64(),
    );
    Ok((
        StatusCode::OK,
        Json(TableResponse {
            total_tables: tables.len(),
            extraction_time_ms: start.elapsed().as_millis() as u64,
            tables,
        }),
    ))
}

/// Get table by ID (4 LOC) - Future API endpoint
#[allow(dead_code)]
pub async fn get_table(
    State(_state): State<ApplicationContext>,
    Path((request_id, _table_id)): Path<(String, usize)>,
) -> Result<impl IntoResponse, ApiError> {
    let table = facade()
        .get_table(&request_id)
        .await
        .ok_or_else(|| ApiError::not_found("Table not found"))?;
    Ok(Json(table))
}

/// Export table to format (5 LOC)
pub async fn export_table(
    State(state): State<ApplicationContext>,
    Path((request_id, _table_id)): Path<(String, usize)>,
    Query(query): Query<ExportQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let format = match query.format.as_str() {
        "csv" => TableFormat::Csv,
        "markdown" => TableFormat::Markdown,
        _ => return Err(ApiError::validation("Invalid format")),
    };
    let exported = facade()
        .export_table(&request_id, format, query.include_headers, false)
        .await
        .map_err(|e| {
            state.transport_metrics.record_wasm_error();
            ApiError::from(e)
        })?;
    Ok(Json(exported))
}

/// Get extraction statistics (3 LOC) - Future API
#[allow(dead_code)]
pub async fn get_table_stats(
    State(state): State<ApplicationContext>,
    Path(_request_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let stats = facade().get_extraction_stats().await.map_err(|e| {
        state.transport_metrics.record_wasm_error();
        ApiError::from(e)
    })?;
    Ok(Json(stats))
}
