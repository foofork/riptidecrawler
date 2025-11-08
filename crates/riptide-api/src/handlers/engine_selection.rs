//! Engine Selection API handlers - Ultra-thin delegation to EngineFacade
//!
//! Phase 3 Sprint 3.1: Refactored to <35 LOC by delegating all business logic to EngineFacade.
//! Handlers are now pure HTTP mapping layer.

use crate::{dto::engine_selection::*, errors::ApiResult, state::AppState};
use axum::{extract::State, response::Json};
use riptide_facade::facades::{EngineCapability, EngineConfig, EngineStats};

/// POST /engine/analyze - Analyze HTML and recommend engine (3 LOC)
pub async fn analyze_engine(
    State(state): State<AppState>,
    Json(request): Json<AnalyzeRequest>,
) -> ApiResult<Json<EngineConfig>> {
    let engine_facade = riptide_facade::facades::EngineFacade::new(state.cache.clone());
    let config = engine_facade.select_engine(request.to_criteria()).await?;
    Ok(Json(config))
}

/// POST /engine/decide - Decide engine with flags (3 LOC)
pub async fn decide_engine(
    State(state): State<AppState>,
    Json(request): Json<DecideRequest>,
) -> ApiResult<Json<EngineConfig>> {
    let engine_facade = riptide_facade::facades::EngineFacade::new(state.cache.clone());
    let config = engine_facade.select_engine(request.to_criteria()).await?;
    Ok(Json(config))
}

/// GET /engine/capabilities - Get all engine capabilities (2 LOC)
pub async fn get_engine_capabilities(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<EngineCapability>>> {
    let engine_facade = riptide_facade::facades::EngineFacade::new(state.cache.clone());
    Ok(Json(engine_facade.list_capabilities().await?))
}

/// GET /engine/stats - Get engine statistics (2 LOC)
pub async fn get_engine_stats(State(state): State<AppState>) -> ApiResult<Json<EngineStats>> {
    let engine_facade = riptide_facade::facades::EngineFacade::new(state.cache.clone());
    Ok(Json(engine_facade.get_stats().await?))
}

/// POST /engine/probe-first - Toggle probe-first mode (3 LOC)
pub async fn set_probe_first(
    State(state): State<AppState>,
    Json(request): Json<ProbeFirstRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let engine_facade = riptide_facade::facades::EngineFacade::new(state.cache.clone());
    engine_facade.update_probe_first(request.enabled).await?;
    Ok(Json(
        serde_json::json!({ "probe_first_enabled": request.enabled }),
    ))
}
