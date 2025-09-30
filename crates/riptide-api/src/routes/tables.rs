//! Table extraction routes configuration

use crate::handlers::tables;
use axum::{routing::{get, post}, Router};
use crate::state::AppState;

/// Create table routes
pub fn table_routes() -> Router<AppState> {
    Router::new()
        .route("/extract", post(tables::extract_tables))
        .route("/:id/export", get(tables::export_table))
}