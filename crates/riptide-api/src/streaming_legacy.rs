//! Legacy streaming module - now redirects to the new modular streaming implementation.
//!
//! This file has been refactored into a comprehensive modular structure located in
//! the `streaming/` directory. The new structure provides:
//!
//! - Better separation of concerns
//! - Improved maintainability
//! - Enhanced error handling
//! - Dynamic buffer management
//! - Protocol-specific optimizations
//!
//! # Migration Guide
//!
//! Old imports:
//! ```rust,ignore
//! use crate::streaming::crawl_stream;
//! ```
//!
//! New imports:
//! ```rust,ignore
//! use crate::streaming::ndjson::crawl_stream;
//! use crate::streaming::sse::crawl_sse;
//! use crate::streaming::websocket::crawl_websocket;
//! ```

// Import the new streaming module
mod streaming;

// Re-export the new modular implementation for backward compatibility
pub use streaming::*;

use crate::models::*;
use crate::state::AppState;
use axum::extract::{State, WebSocketUpgrade, Json};
use axum::response::IntoResponse;

/// NDJSON streaming response for crawl operations.
///
/// **Deprecated**: This function has been moved to `streaming::ndjson::crawl_stream`.
/// Use the new modular implementation for better performance and maintainability.
pub async fn crawl_stream(
    State(app): State<AppState>,
    Json(body): Json<CrawlBody>,
) -> impl IntoResponse {
    // Redirect to the new implementation
    streaming::ndjson::crawl_stream(State(app), Json(body)).await
}

/// NDJSON streaming response for deep search operations.
///
/// **Deprecated**: This function has been moved to `streaming::ndjson::deepsearch_stream`.
/// Use the new modular implementation for better performance and maintainability.
pub async fn deepsearch_stream(
    State(app): State<AppState>,
    Json(body): Json<DeepSearchBody>,
) -> impl IntoResponse {
    // Redirect to the new implementation
    streaming::ndjson::deepsearch_stream(State(app), Json(body)).await
}

/// SSE (Server-Sent Events) endpoint for real-time crawl progress.
///
/// **Deprecated**: This function has been moved to `streaming::sse::crawl_sse`.
/// Use the new modular implementation for better performance and maintainability.
pub async fn crawl_sse(
    State(app): State<AppState>,
    Json(body): Json<CrawlBody>,
) -> impl IntoResponse {
    // Redirect to the new implementation
    streaming::sse::crawl_sse(State(app), Json(body)).await
}

/// WebSocket endpoint for bidirectional real-time communication.
///
/// **Deprecated**: This function has been moved to `streaming::websocket::crawl_websocket`.
/// Use the new modular implementation for better performance and maintainability.
pub async fn crawl_websocket(
    ws: WebSocketUpgrade,
    State(app): State<AppState>,
) -> impl IntoResponse {
    // Redirect to the new implementation
    streaming::websocket::crawl_websocket(ws, State(app)).await
}