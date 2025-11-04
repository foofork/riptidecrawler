//! Table extraction routes configuration

#[cfg(feature = "extraction")]
use crate::handlers::tables;
use crate::state::AppState;
use axum::{
    routing::{get, post},
    Router,
};

/// Create table routes
#[cfg(feature = "extraction")]
pub fn table_routes() -> Router<AppState> {
    Router::new()
        .route("/extract", post(tables::extract_tables))
        .route("/:id/export", get(tables::export_table))
}

/// Create stub table routes when feature is disabled
#[cfg(not(feature = "extraction"))]
pub fn table_routes() -> Router<AppState> {
    Router::new()
}
