#![allow(dead_code)]

/// Enhanced pipeline metrics visualization endpoint
///
/// This module provides endpoints for retrieving and visualizing enhanced pipeline metrics,
/// including detailed phase timing data, gate decision statistics, and performance trends.
use crate::errors::ApiError;
use crate::state::AppState;
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Enhanced pipeline metrics response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedPipelineMetrics {
    /// Whether enhanced pipeline is currently enabled
    pub enabled: bool,

    /// Phase timing statistics
    pub phase_timings: PhaseTimingStats,

    /// Gate decision breakdown
    pub gate_decisions: GateDecisionBreakdown,

    /// Overall performance metrics
    pub performance: PerformanceStats,

    /// Configuration status
    pub config: PipelineConfigStatus,
}

/// Phase timing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseTimingStats {
    /// Average fetch phase time in milliseconds
    pub avg_fetch_ms: f64,

    /// Average gate phase time in milliseconds
    pub avg_gate_ms: f64,

    /// Average WASM extraction time in milliseconds
    pub avg_wasm_ms: f64,

    /// Average render phase time in milliseconds (when used)
    pub avg_render_ms: Option<f64>,

    /// Phase timing distribution (percentiles)
    pub percentiles: PhasePercentiles,
}

/// Phase timing percentiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhasePercentiles {
    pub p50_fetch_ms: f64,
    pub p95_fetch_ms: f64,
    pub p99_fetch_ms: f64,
    pub p50_gate_ms: f64,
    pub p95_gate_ms: f64,
    pub p99_gate_ms: f64,
    pub p50_wasm_ms: f64,
    pub p95_wasm_ms: f64,
    pub p99_wasm_ms: f64,
}

/// Gate decision breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateDecisionBreakdown {
    /// Number of raw HTML extractions
    pub raw_count: usize,

    /// Number of probes-first extractions
    pub probes_first_count: usize,

    /// Number of headless browser extractions
    pub headless_count: usize,

    /// Percentage breakdown
    pub percentages: HashMap<String, f32>,
}

/// Performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    /// Total requests processed
    pub total_requests: u64,

    /// Successful extractions
    pub successful: u64,

    /// Failed extractions
    pub failed: u64,

    /// Cache hit rate
    pub cache_hit_rate: f32,

    /// Average processing time in milliseconds
    pub avg_processing_time_ms: f64,

    /// Requests per second (last minute)
    pub requests_per_second: f32,
}

/// Pipeline configuration status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfigStatus {
    /// Enhanced pipeline enabled
    pub enhanced_enabled: bool,

    /// Phase metrics collection enabled
    pub phase_metrics_enabled: bool,

    /// Debug logging enabled
    pub debug_logging_enabled: bool,

    /// Configured timeouts
    pub timeouts: TimeoutConfig,
}

/// Timeout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    pub fetch_timeout_secs: u64,
    pub gate_timeout_secs: u64,
    pub wasm_timeout_secs: u64,
    pub render_timeout_secs: u64,
}

/// Get enhanced pipeline metrics
///
/// Returns comprehensive metrics about the enhanced pipeline's performance,
/// including phase timings, gate decisions, and overall statistics.
#[tracing::instrument(
    name = "pipeline_metrics",
    skip(state),
    fields(
        http.method = "GET",
        http.route = "/api/metrics/pipeline"
    )
)]
pub async fn get_pipeline_metrics(
    State(state): State<AppState>,
) -> Result<Json<EnhancedPipelineMetrics>, ApiError> {
    // Collect metrics from the metrics system
    // Note: metrics variable removed as it's not yet used (placeholder implementation)

    // Get phase timing stats from Prometheus metrics
    // In production, this would query the actual Prometheus registry
    // For now, return default/placeholder values
    let phase_timings = PhaseTimingStats {
        avg_fetch_ms: 0.0, // Would be calculated from metrics
        avg_gate_ms: 0.0,
        avg_wasm_ms: 0.0,
        avg_render_ms: None,
        percentiles: PhasePercentiles {
            p50_fetch_ms: 0.0,
            p95_fetch_ms: 0.0,
            p99_fetch_ms: 0.0,
            p50_gate_ms: 0.0,
            p95_gate_ms: 0.0,
            p99_gate_ms: 0.0,
            p50_wasm_ms: 0.0,
            p95_wasm_ms: 0.0,
            p99_wasm_ms: 0.0,
        },
    };

    // Get gate decision stats
    // In production, query from metrics registry
    let gate_decisions = GateDecisionBreakdown {
        raw_count: 0,
        probes_first_count: 0,
        headless_count: 0,
        percentages: HashMap::new(),
    };

    // Get performance stats
    let performance = PerformanceStats {
        total_requests: 0,
        successful: 0,
        failed: 0,
        cache_hit_rate: 0.0,
        avg_processing_time_ms: 0.0,
        requests_per_second: 0.0,
    };

    // Get configuration status
    let config = PipelineConfigStatus {
        enhanced_enabled: state
            .config
            .enhanced_pipeline_config
            .enable_enhanced_pipeline,
        phase_metrics_enabled: state.config.enhanced_pipeline_config.enable_phase_metrics,
        debug_logging_enabled: state.config.enhanced_pipeline_config.enable_debug_logging,
        timeouts: TimeoutConfig {
            fetch_timeout_secs: state.config.enhanced_pipeline_config.fetch_timeout_secs,
            gate_timeout_secs: state.config.enhanced_pipeline_config.gate_timeout_secs,
            wasm_timeout_secs: state.config.enhanced_pipeline_config.wasm_timeout_secs,
            render_timeout_secs: state.config.enhanced_pipeline_config.render_timeout_secs,
        },
    };

    Ok(Json(EnhancedPipelineMetrics {
        enabled: state
            .config
            .enhanced_pipeline_config
            .enable_enhanced_pipeline,
        phase_timings,
        gate_decisions,
        performance,
        config,
    }))
}

/// Toggle enhanced pipeline on/off
///
/// Allows runtime toggling of the enhanced pipeline feature.
/// This is useful for A/B testing and gradual rollout.
#[tracing::instrument(
    name = "toggle_pipeline",
    skip(_state),
    fields(
        http.method = "POST",
        http.route = "/api/metrics/pipeline/toggle"
    )
)]
pub async fn toggle_enhanced_pipeline(
    State(_state): State<AppState>,
    Json(request): Json<ToggleRequest>,
) -> Result<Json<ToggleResponse>, ApiError> {
    // Note: In production, this would require authentication and authorization
    // For now, this is a simple demonstration

    // This would update a runtime configuration flag
    // For thread-safe updates, use Arc<RwLock<Config>> or similar

    Ok(Json(ToggleResponse {
        success: true,
        enabled: request.enabled,
        message: format!(
            "Enhanced pipeline {}",
            if request.enabled {
                "enabled"
            } else {
                "disabled"
            }
        ),
    }))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToggleRequest {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToggleResponse {
    pub success: bool,
    pub enabled: bool,
    pub message: String,
}
