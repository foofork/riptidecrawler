//! Engine selection routes configuration
//!
//! Provides routing for engine selection endpoints:
//! - POST /engine/analyze - Analyze HTML and recommend engine
//! - POST /engine/decide - Make engine decision with flags
//! - GET /engine/stats - Get engine usage statistics
//! - PUT /engine/probe-first - Toggle probe-first mode

use crate::handlers::engine_selection;
use crate::context::ApplicationContext;
use axum::{
    routing::{get, post, put},
    Router,
};

/// Create engine selection routes
pub fn engine_routes() -> Router<ApplicationContext> {
    Router::new()
        .route("/analyze", post(engine_selection::analyze_engine))
        .route("/decide", post(engine_selection::decide_engine))
        .route("/stats", get(engine_selection::get_engine_stats))
        .route("/probe-first", put(engine_selection::set_probe_first))
}
