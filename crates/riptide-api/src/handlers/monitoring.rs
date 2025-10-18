//! Monitoring endpoints for performance tracking, health scoring, and alerting
//!
//! This module provides HTTP endpoints for accessing the integrated monitoring system,
//! including health scores, performance reports, and alert management.

use crate::errors::ApiError;
use crate::state::AppState;
use axum::{
    extract::State,
    response::{IntoResponse, Json},
};
use serde::Serialize;

// Allow unused imports - these may be used in future endpoint implementations
#[allow(unused_imports)]
use serde::Deserialize;

/// Health score response
#[derive(Debug, Serialize)]
pub struct HealthScoreResponse {
    /// Overall health score (0-100)
    pub health_score: f32,

    /// Health status classification
    pub status: String,

    /// Timestamp of the calculation
    pub timestamp: String,
}

/// GET /monitoring/health-score - Get current health score
///
/// Returns the current system health score (0-100) based on performance metrics.
/// This endpoint provides a single numeric value representing overall system health.
pub async fn get_health_score(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let health_score = state.monitoring_system.calculate_health_score().await?;

    let status = crate::health::classify_health_score(health_score);

    Ok(Json(HealthScoreResponse {
        health_score,
        status: status.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    }))
}

/// GET /monitoring/performance-report - Get comprehensive performance report
///
/// Returns a detailed performance report including metrics, health score, summary,
/// and actionable recommendations for system optimization.
pub async fn get_performance_report(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let report = state
        .monitoring_system
        .generate_performance_report()
        .await?;

    Ok(Json(report))
}

/// Alert rule response
#[derive(Debug, Serialize)]
pub struct AlertRulesResponse {
    /// List of configured alert rules
    pub rules: Vec<AlertRuleSummary>,

    /// Total number of rules
    pub total: usize,

    /// Number of enabled rules
    pub enabled: usize,
}

/// Alert rule summary
#[derive(Debug, Serialize)]
pub struct AlertRuleSummary {
    pub name: String,
    pub metric_name: String,
    pub threshold: f64,
    pub condition: String,
    pub severity: String,
    pub enabled: bool,
}

/// GET /monitoring/alerts/rules - Get configured alert rules
///
/// Returns the list of all configured alert rules, including their thresholds,
/// conditions, and enabled status.
pub async fn get_alert_rules(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let manager = state.monitoring_system.alert_manager.lock().await;
    let rules = manager.get_rules();

    let enabled = rules.iter().filter(|r| r.enabled).count();

    let rule_summaries: Vec<AlertRuleSummary> = rules
        .iter()
        .map(|r| AlertRuleSummary {
            name: r.name.clone(),
            metric_name: r.metric_name.clone(),
            threshold: r.threshold,
            condition: format!("{:?}", r.condition),
            severity: format!("{:?}", r.severity),
            enabled: r.enabled,
        })
        .collect();

    Ok(Json(AlertRulesResponse {
        total: rule_summaries.len(),
        enabled,
        rules: rule_summaries,
    }))
}

/// Active alerts response
#[derive(Debug, Serialize)]
pub struct ActiveAlertsResponse {
    /// List of currently active alert names
    pub active_alerts: Vec<String>,

    /// Total number of active alerts
    pub count: usize,
}

/// GET /monitoring/alerts/active - Get currently active alerts
///
/// Returns the list of alerts that are currently triggered and within their
/// cooldown period.
pub async fn get_active_alerts(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let manager = state.monitoring_system.alert_manager.lock().await;
    let active_alerts = manager.get_active_alerts();

    Ok(Json(ActiveAlertsResponse {
        count: active_alerts.len(),
        active_alerts,
    }))
}

/// Current metrics response
#[derive(Debug, Serialize)]
pub struct CurrentMetricsResponse {
    /// Current performance metrics
    pub metrics: riptide_monitoring::PerformanceMetrics,
}

/// GET /monitoring/metrics/current - Get current performance metrics
///
/// Returns the current snapshot of all performance metrics including timing,
/// throughput, resource usage, and error rates.
pub async fn get_current_metrics(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let metrics = state
        .monitoring_system
        .metrics_collector
        .get_current_metrics()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to get metrics: {}", e)))?;

    Ok(Json(CurrentMetricsResponse { metrics }))
}

/// GET /api/resources/status - Get current resource utilization status
///
/// Returns the current status of all managed resources including:
/// - Headless browser pool availability
/// - PDF processing semaphore permits
/// - Memory usage and pressure status
/// - Rate limiting statistics
/// - Timeout counts and degradation scores
pub async fn get_resource_status(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let status = state.resource_manager.get_resource_status().await;

    Ok(Json(status))
}

// ==================== Performance Profiling Endpoints ====================

/// Memory metrics response
#[derive(Debug, Serialize)]
pub struct MemoryMetricsResponse {
    pub rss_mb: f64,
    pub heap_mb: f64,
    pub virtual_mb: f64,
    pub timestamp: String,
}

/// Leak summary response
#[derive(Debug, Serialize)]
pub struct LeakSummaryResponse {
    pub potential_leak_count: usize,
    pub growth_rate_mb_per_hour: f64,
    pub highest_risk_component: Option<String>,
}

/// Allocation metrics response
#[derive(Debug, Serialize)]
pub struct AllocationMetricsResponse {
    pub top_allocators: Vec<(String, u64)>,
    pub efficiency_score: f64,
    pub recommendations: Vec<String>,
}

/// GET /monitoring/profiling/memory - Get current memory usage metrics
///
/// Returns real-time memory usage including RSS, heap, and virtual memory.
/// Useful for tracking memory consumption and identifying leaks.
pub async fn get_memory_metrics(
    State(_state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    // TODO(P2): Implement memory profiling integration
    // STATUS: Profiler field exists in AppState but not activated
    // PLAN: Wire up memory profiler to collect real-time metrics
    // IMPLEMENTATION:
    //   1. Enable profiler in AppState initialization
    //   2. Integrate with jemalloc or tikv-jemallocator
    //   3. Add profiling data collection in background task
    //   4. Return real RSS/heap/virtual memory stats
    // DEPENDENCIES: Memory profiling crate (jemalloc_ctl or similar)
    // EFFORT: Medium (6-8 hours)
    // PRIORITY: Nice-to-have for production debugging
    // BLOCKER: None
    Ok(Json(MemoryMetricsResponse {
        rss_mb: 0.0,
        heap_mb: 0.0,
        virtual_mb: 0.0,
        timestamp: chrono::Utc::now().to_rfc3339(),
    }))
}

/// GET /monitoring/profiling/leaks - Get memory leak analysis
///
/// Returns analysis of potential memory leaks based on allocation patterns.
/// Includes growth rates, suspicious patterns, and highest-risk components.
pub async fn get_leak_analysis(
    State(_state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    // TODO(P2): Implement leak detection integration
    // STATUS: Profiler field exists but not activated
    // PLAN: Add memory leak detection analysis
    // IMPLEMENTATION:
    //   1. Track allocation patterns over time windows
    //   2. Analyze growth rates and identify suspicious patterns
    //   3. Categorize allocations by component/module
    //   4. Generate leak detection reports with root cause hints
    // DEPENDENCIES: Requires memory profiling (see monitoring.rs:219)
    // EFFORT: High (8-12 hours)
    // PRIORITY: Optional - for advanced debugging
    // BLOCKER: Must implement memory profiling first
    Ok(Json(LeakSummaryResponse {
        potential_leak_count: 0,
        growth_rate_mb_per_hour: 0.0,
        highest_risk_component: None,
    }))
}

/// GET /monitoring/profiling/allocations - Get allocation analysis
///
/// Returns allocation patterns and optimization recommendations.
/// Includes top allocators, size distribution, and efficiency scoring.
pub async fn get_allocation_metrics(
    State(_state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    // TODO(P2): Implement allocation analysis integration
    // STATUS: Profiler field exists but not activated
    // PLAN: Add allocation pattern analysis and optimization recommendations
    // IMPLEMENTATION:
    //   1. Track allocation sizes and frequencies by call site
    //   2. Identify top allocators and hot paths
    //   3. Calculate allocation efficiency scores
    //   4. Generate actionable optimization recommendations
    // DEPENDENCIES: Requires memory profiling (see monitoring.rs:219)
    // EFFORT: High (8-12 hours)
    // PRIORITY: Optional - for performance optimization
    // BLOCKER: Must implement memory profiling first
    Ok(Json(AllocationMetricsResponse {
        top_allocators: vec![],
        efficiency_score: 0.0,
        recommendations: vec![],
    }))
}

// ==================== WASM Instance Health Monitoring ====================

/// WASM instance health information
#[derive(Debug, Serialize)]
pub struct WasmInstanceHealth {
    pub worker_id: String,
    pub is_healthy: bool,
    pub operations_count: u64,
    pub memory_usage_bytes: usize,
    pub uptime_seconds: u64,
}

/// WASM health response
#[derive(Debug, Serialize)]
pub struct WasmHealthResponse {
    pub instances: Vec<WasmInstanceHealth>,
    pub total_instances: usize,
    pub healthy_instances: usize,
    pub needs_cleanup: bool,
    pub timestamp: String,
}

/// GET /monitoring/wasm-instances - Get WASM instance health status
///
/// Returns health information for all WASM worker instances including:
/// - Worker ID and health status
/// - Operation counts and uptime
/// - Cleanup recommendations
pub async fn get_wasm_health(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let health_data = state
        .resource_manager
        .wasm_manager
        .get_instance_health()
        .await;

    let needs_cleanup = state.resource_manager.wasm_manager.needs_cleanup().await;

    let instances: Vec<WasmInstanceHealth> = health_data
        .into_iter()
        .map(
            |(worker_id, is_healthy, operations_count, memory_usage, uptime)| WasmInstanceHealth {
                worker_id,
                is_healthy,
                operations_count,
                memory_usage_bytes: memory_usage,
                uptime_seconds: uptime.as_secs(),
            },
        )
        .collect();

    let healthy_count = instances.iter().filter(|i| i.is_healthy).count();

    Ok(Json(WasmHealthResponse {
        total_instances: instances.len(),
        healthy_instances: healthy_count,
        needs_cleanup,
        instances,
        timestamp: chrono::Utc::now().to_rfc3339(),
    }))
}
