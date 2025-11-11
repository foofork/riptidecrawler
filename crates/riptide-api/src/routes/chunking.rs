//! Content chunking routes configuration

use crate::context::ApplicationContext;
#[cfg(feature = "extraction")]
use crate::handlers::chunking;
use axum::{routing::post, Router};

/// Create content chunking routes
#[cfg(feature = "extraction")]
pub fn chunking_routes() -> Router<ApplicationContext> {
    Router::new().route("/chunk", post(chunking::handle_chunking))
}

/// Create stub chunking routes when feature is disabled
/// Returns HTTP 501 "Not Implemented" for all chunking endpoints
#[cfg(not(feature = "extraction"))]
pub fn chunking_routes() -> Router<ApplicationContext> {
    use crate::handlers::stubs::*;

    Router::new().route("/chunk", post(extraction_chunk_stub))
}
