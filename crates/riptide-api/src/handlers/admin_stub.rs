// Temporary stub implementations for admin handlers until persistence layer is fully integrated

use axum::{extract::State, Json};
use crate::errors::{ApiError, ApiResult};
use crate::context::ApplicationContext;

/// Helper function to return not-implemented error for persistence endpoints
fn persistence_not_implemented() -> ApiError {
    ApiError::internal("Persistence layer not yet fully integrated - this endpoint is under development")
}

// Re-export all the request/response types that admin.rs uses
pub use super::admin::*;
