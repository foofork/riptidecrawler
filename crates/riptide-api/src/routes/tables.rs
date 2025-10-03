//! Table extraction routes configuration

use crate::handlers::tables;
use crate::state::AppState;
use axum::{
    routing::{get, post},
    Router,
};

/// Create table routes
pub fn table_routes() -> Router<AppState> {
    Router::new()
        .route("/extract", post(tables::extract_tables))
        .route("/:id/export", get(tables::export_table))
}
