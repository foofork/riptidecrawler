//! Content chunking routes configuration

use crate::handlers::chunking;
use crate::state::AppState;
use axum::{routing::post, Router};

/// Create content chunking routes
pub fn chunking_routes() -> Router<AppState> {
    Router::new().route("/chunk", post(chunking::chunk_content))
}
