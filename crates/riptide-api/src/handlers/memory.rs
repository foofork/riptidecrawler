//! Memory handler - <50 LOC after facade refactoring
use crate::errors::ApiError;
use crate::state::AppState;
use axum::{extract::State, Json};
use riptide_facade::facades::memory::{MemoryFacade, MemoryUsageResponse};
use tracing::instrument;

#[instrument(skip(_state))]
pub async fn handle_memory_usage(
    State(_state): State<AppState>,
) -> Result<Json<MemoryUsageResponse>, ApiError> {
    MemoryFacade::new()
        .get_memory_usage()
        .await
        .map(Json)
        .map_err(|e| ApiError::internal(format!("Memory usage failed: {}", e)))
}
