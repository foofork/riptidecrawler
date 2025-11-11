//! Memory handler - <50 LOC after facade refactoring
use crate::errors::ApiError;
use crate::context::ApplicationContext;
use axum::{extract::State, Json};
use riptide_facade::facades::memory::{MemoryFacade, MemoryUsageResponse};
use tracing::instrument;

/// Future API endpoint for memory usage monitoring
#[allow(dead_code)]
#[instrument(skip(_state))]
pub async fn handle_memory_usage(
    State(_state): State<ApplicationContext>,
) -> Result<Json<MemoryUsageResponse>, ApiError> {
    MemoryFacade::new()
        .get_memory_usage()
        .await
        .map(Json)
        .map_err(|e| ApiError::internal(format!("Memory usage failed: {}", e)))
}

/// Memory profile handler stub - returns basic memory statistics
pub async fn memory_profile_handler(
    State(_state): State<ApplicationContext>,
) -> Result<Json<MemoryUsageResponse>, ApiError> {
    Ok(Json(MemoryUsageResponse {
        total_bytes: 0,
        used_bytes: 0,
        available_bytes: 0,
        usage_percentage: 0.0,
        pressure_level: "normal".to_string(),
        recommendations: vec![],
    }))
}
