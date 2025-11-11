//! Table extraction routes configuration

use crate::context::ApplicationContext;
#[cfg(feature = "extraction")]
use crate::handlers::tables;
use axum::{
    routing::{get, post},
    Router,
};

/// Create table routes
#[cfg(feature = "extraction")]
pub fn table_routes() -> Router<ApplicationContext> {
    Router::new()
        .route("/extract", post(tables::extract_tables))
        .route("/:id/export", get(tables::export_table))
}

/// Create stub table routes when feature is disabled
/// Returns HTTP 501 "Not Implemented" for all table endpoints
#[cfg(not(feature = "extraction"))]
pub fn table_routes() -> Router<ApplicationContext> {
    use crate::handlers::stubs::*;

    Router::new()
        .route("/extract", post(extraction_tables_stub))
        .route("/:id/export", get(table_export_stub))
}
