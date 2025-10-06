//! Monitoring endpoints for performance tracking, health scoring, and alerting
//!
//! This module provides HTTP endpoints for accessing the integrated monitoring system,
//! including health scores, performance reports, and alert management.

use crate::errors::ApiError;
use crate::state::AppState;
use axum::{extract::State, response::{IntoResponse, Json}};
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
    let health_score = state
        .monitoring_system
        .calculate_health_score()
        .await?;

    let status = if health_score >= 95.0 {
        "excellent"
    } else if health_score >= 85.0 {
        "good"
    } else if health_score >= 70.0 {
        "fair"
    } else if health_score >= 50.0 {
        "poor"
    } else {
        "critical"
    };

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
pub async fn get_alert_rules(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
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
    pub metrics: riptide_core::monitoring::PerformanceMetrics,
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
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let perf_manager = state.performance_metrics.lock().await;

    let snapshot = perf_manager
        .profiler
        .tracker
        .get_current_snapshot()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to get memory snapshot: {}", e)))?;

    Ok(Json(MemoryMetricsResponse {
        rss_mb: snapshot.rss_bytes as f64 / 1024.0 / 1024.0,
        heap_mb: snapshot.heap_bytes as f64 / 1024.0 / 1024.0,
        virtual_mb: snapshot.virtual_bytes as f64 / 1024.0 / 1024.0,
        timestamp: snapshot.timestamp.to_rfc3339(),
    }))
}

/// GET /monitoring/profiling/leaks - Get memory leak analysis
///
/// Returns analysis of potential memory leaks based on allocation patterns.
/// Includes growth rates, suspicious patterns, and highest-risk components.
pub async fn get_leak_analysis(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let perf_manager = state.performance_metrics.lock().await;

    let analysis = perf_manager
        .profiler
        .leak_detector
        .analyze_leaks()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to analyze leaks: {}", e)))?;

    Ok(Json(LeakSummaryResponse {
        potential_leak_count: analysis.potential_leaks.len(),
        growth_rate_mb_per_hour: analysis.growth_rate_mb_per_hour,
        highest_risk_component: analysis
            .potential_leaks
            .first()
            .map(|leak| leak.component.clone()),
    }))
}

/// GET /monitoring/profiling/allocations - Get allocation analysis
///
/// Returns allocation patterns and optimization recommendations.
/// Includes top allocators, size distribution, and efficiency scoring.
pub async fn get_allocation_metrics(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let perf_manager = state.performance_metrics.lock().await;

    let top_allocators = perf_manager
        .profiler
        .allocation_analyzer
        .get_top_allocators()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to get top allocators: {}", e)))?;

    let recommendations = perf_manager
        .profiler
        .allocation_analyzer
        .analyze_patterns()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to analyze patterns: {}", e)))?;

    let efficiency = perf_manager
        .profiler
        .allocation_analyzer
        .calculate_efficiency_score()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to calculate efficiency: {}", e)))?;

    Ok(Json(AllocationMetricsResponse {
        top_allocators,
        efficiency_score: efficiency,
        recommendations,
    }))
}
