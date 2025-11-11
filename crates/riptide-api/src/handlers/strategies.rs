//! Strategies handler - <50 LOC after facade refactoring
use crate::context::ApplicationContext;
use crate::errors::ApiError;
use axum::{extract::State, Json};
use riptide_facade::facades::strategies::{StrategiesFacade, StrategyRequest, StrategyResponse};
use serde::Deserialize;
use tracing::instrument;

/// DTO for strategy selection requests - Future API
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct StrategyRequestDTO {
    pub url: String,
    pub force_strategy: Option<String>,
    pub enable_probe: Option<bool>,
}

/// Future API endpoint for extraction strategy selection
#[allow(dead_code)]
#[instrument(skip(_state))]
pub async fn handle_strategy_selection(
    State(_state): State<ApplicationContext>,
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

/// Crawl strategies endpoint - returns available crawling strategies
#[instrument]
pub async fn strategies_crawl() -> Result<Json<StrategyResponse>, ApiError> {
    use riptide_facade::facades::strategies::AlternativeStrategy;

    Ok(Json(StrategyResponse {
        recommended_strategy: "auto".to_string(),
        confidence_score: 0.95,
        reasoning:
            "Automatic strategy selection based on URL characteristics and available engines"
                .to_string(),
        alternatives: vec![
            AlternativeStrategy {
                strategy: "native".to_string(),
                score: 1.0,
                pros: vec![
                    "Fast HTTP client".to_string(),
                    "Low resource overhead".to_string(),
                    "Efficient for static content".to_string(),
                ],
                cons: vec!["No JavaScript execution".to_string()],
            },
            AlternativeStrategy {
                strategy: "wasm".to_string(),
                score: 0.8,
                pros: vec![
                    "JavaScript execution support".to_string(),
                    "SPA compatibility".to_string(),
                ],
                cons: vec!["Higher overhead than native".to_string()],
            },
            AlternativeStrategy {
                strategy: "headless".to_string(),
                score: 0.6,
                pros: vec![
                    "Full browser environment".to_string(),
                    "Complex JS handling".to_string(),
                ],
                cons: vec![
                    "Highest resource usage".to_string(),
                    "Slower execution".to_string(),
                ],
            },
        ],
        processing_time_ms: 1,
    }))
}

/// Get strategies info endpoint - returns current strategy configuration
#[instrument]
pub async fn get_strategies_info() -> Result<Json<StrategyResponse>, ApiError> {
    use riptide_facade::facades::strategies::AlternativeStrategy;

    Ok(Json(StrategyResponse {
        recommended_strategy: "info".to_string(),
        confidence_score: 1.0,
        reasoning: "Current engine priority configuration: native (1) → wasm (2) → headless (3). Auto-selection enabled with fallback chain. Engine weight persistence pending Phase 6 implementation.".to_string(),
        alternatives: vec![
            AlternativeStrategy {
                strategy: "native".to_string(),
                score: 1.0,
                pros: vec!["Priority 1 - Default choice".to_string()],
                cons: vec![],
            },
            AlternativeStrategy {
                strategy: "wasm".to_string(),
                score: 0.8,
                pros: vec!["Priority 2 - Fallback option".to_string()],
                cons: vec![],
            },
            AlternativeStrategy {
                strategy: "headless".to_string(),
                score: 0.6,
                pros: vec!["Priority 3 - Last resort".to_string()],
                cons: vec![],
            },
        ],
        processing_time_ms: 0,
    }))
}
