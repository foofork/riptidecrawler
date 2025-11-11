//! Pipeline phases handler - <50 LOC after facade refactoring
use crate::errors::ApiError;
use crate::context::ApplicationContext;
use axum::{extract::State, Json};
use riptide_facade::facades::pipeline_phases::{
    PhaseConfig, PhaseExecutionRequest, PhaseExecutionResponse, PipelinePhasesFacade,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::instrument;

/// DTO for phase execution requests - Future API
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct PhaseExecutionRequestDTO {
    pub phase_name: String,
    pub input_data: Value,
    pub config: Option<PhaseConfigDTO>,
}

/// DTO for phase configuration - Future API
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct PhaseConfigDTO {
    pub timeout_secs: Option<u64>,
    pub retry_count: Option<u32>,
    pub enable_metrics: Option<bool>,
}

/// Future API endpoint for pipeline phase execution
#[allow(dead_code)]
#[instrument(skip(_state))]
pub async fn handle_phase_execution(
    State(_state): State<ApplicationContext>,
    Json(req): Json<PhaseExecutionRequestDTO>,
) -> Result<Json<PhaseExecutionResponse>, ApiError> {
    let config = req.config.map(|c| PhaseConfig {
        timeout_secs: c.timeout_secs,
        retry_count: c.retry_count,
        enable_metrics: c.enable_metrics,
    });
    PipelinePhasesFacade::new()
        .execute_phase(PhaseExecutionRequest {
            phase_name: req.phase_name,
            input_data: req.input_data,
            config,
        })
        .await
        .map(Json)
        .map_err(|e| ApiError::internal(format!("Phase execution failed: {}", e)))
}

// Backward compatibility for get_pipeline_phases
#[derive(Debug, Serialize)]
pub struct PipelinePhase {
    pub name: String,
    pub description: String,
    pub order: usize,
}

pub async fn get_pipeline_phases(
    State(_state): State<ApplicationContext>,
) -> Result<Json<Vec<PipelinePhase>>, ApiError> {
    Ok(Json(vec![
        PipelinePhase {
            name: "fetch".into(),
            description: "Fetch content from URL".into(),
            order: 1,
        },
        PipelinePhase {
            name: "gate".into(),
            description: "Content quality gating".into(),
            order: 2,
        },
        PipelinePhase {
            name: "extract".into(),
            description: "Extract structured data".into(),
            order: 3,
        },
        PipelinePhase {
            name: "transform".into(),
            description: "Transform and clean data".into(),
            order: 4,
        },
        PipelinePhase {
            name: "validate".into(),
            description: "Validate output".into(),
            order: 5,
        },
    ]))
}
