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
