//! Monitoring endpoints for performance tracking, health scoring, and alerting
//!
//! This module provides HTTP endpoints for accessing the integrated monitoring system,
//! including health scores, performance reports, and alert management.

use crate::state::{AppState, PerformanceReport};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};

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
) -> Result<Json<HealthScoreResponse>, (StatusCode, String)> {
    let health_score = state
        .monitoring_system
        .calculate_health_score()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

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
) -> Result<Json<PerformanceReport>, (StatusCode, String)> {
    let report = state
        .monitoring_system
        .generate_performance_report()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

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
) -> Result<Json<AlertRulesResponse>, (StatusCode, String)> {
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
) -> Result<Json<ActiveAlertsResponse>, (StatusCode, String)> {
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
) -> Result<Json<CurrentMetricsResponse>, (StatusCode, String)> {
    let metrics = state
        .monitoring_system
        .metrics_collector
        .get_current_metrics()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(CurrentMetricsResponse { metrics }))
}
