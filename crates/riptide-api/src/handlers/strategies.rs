//! Strategies handler - <50 LOC after facade refactoring
use crate::errors::ApiError;
use crate::state::AppState;
use axum::{extract::State, Json};
use riptide_facade::facades::strategies::{StrategiesFacade, StrategyRequest, StrategyResponse};
use serde::Deserialize;
use tracing::instrument;

#[derive(Debug, Deserialize)]
pub struct StrategyRequestDTO {
    pub url: String,
    pub force_strategy: Option<String>,
    pub enable_probe: Option<bool>,
}

#[instrument(skip(_state))]
pub async fn handle_strategy_selection(
    State(_state): State<AppState>,
    Json(req): Json<StrategyRequestDTO>,
) -> Result<Json<StrategyResponse>, ApiError> {
    StrategiesFacade::new()
        .select_strategy(StrategyRequest {
            url: req.url,
            force_strategy: req.force_strategy,
            enable_probe: req.enable_probe,
        })
        .await
        .map(Json)
        .map_err(|e| ApiError::internal(format!("Strategy selection failed: {}", e)))
}
