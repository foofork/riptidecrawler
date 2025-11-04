//! Content chunking routes configuration

#[cfg(feature = "extraction")]
use crate::handlers::chunking;
use crate::state::AppState;
use axum::{routing::post, Router};

/// Create content chunking routes
#[cfg(feature = "extraction")]
pub fn chunking_routes() -> Router<AppState> {
    Router::new().route("/chunk", post(chunking::chunk_content))
}

/// Create stub chunking routes when feature is disabled
#[cfg(not(feature = "extraction"))]
pub fn chunking_routes() -> Router<AppState> {
    Router::new()
}
