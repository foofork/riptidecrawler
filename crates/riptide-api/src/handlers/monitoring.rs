//! Monitoring handler - <50 LOC after facade refactoring
use crate::errors::ApiError;
use crate::state::AppState;
use axum::{extract::State, Json};
use riptide_facade::facades::monitoring::{
    HealthScoreResponse, MonitoringFacade, PerformanceReportResponse,
};
use tracing::instrument;

#[instrument(skip(_state))]
pub async fn handle_health_score(
    State(_state): State<AppState>,
) -> Result<Json<HealthScoreResponse>, ApiError> {
    MonitoringFacade::new()
        .get_health_score()
        .await
        .map(Json)
        .map_err(|e| ApiError::internal(format!("Health score failed: {}", e)))
}

#[instrument(skip(_state))]
pub async fn handle_performance_report(
    State(_state): State<AppState>,
) -> Result<Json<PerformanceReportResponse>, ApiError> {
    MonitoringFacade::new()
        .get_performance_report()
        .await
        .map(Json)
        .map_err(|e| ApiError::internal(format!("Performance report failed: {}", e)))
}
