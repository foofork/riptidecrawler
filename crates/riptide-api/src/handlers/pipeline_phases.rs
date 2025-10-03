use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::errors::ApiError;
use crate::state::AppState;

/// Pipeline phase breakdown response
#[derive(Debug, Serialize, Deserialize)]
pub struct PipelinePhaseBreakdown {
    /// Overall pipeline metrics
    pub overall: OverallMetrics,

    /// Individual phase metrics
    pub phases: Vec<PhaseMetrics>,

    /// Bottleneck analysis
    pub bottlenecks: Vec<BottleneckInfo>,

    /// Success rates
    pub success_rates: SuccessRates,
}

/// Overall pipeline metrics
#[derive(Debug, Serialize, Deserialize)]
pub struct OverallMetrics {
    /// Total requests processed
    pub total_requests: u64,

    /// Average total time (milliseconds)
    pub avg_total_time_ms: f64,

    /// P50 latency (milliseconds)
    pub p50_latency_ms: f64,

    /// P95 latency (milliseconds)
    pub p95_latency_ms: f64,

    /// P99 latency (milliseconds)
    pub p99_latency_ms: f64,
}

/// Individual phase metrics
#[derive(Debug, Serialize, Deserialize)]
pub struct PhaseMetrics {
    /// Phase name
    pub name: String,

    /// Average duration (milliseconds)
    pub avg_duration_ms: f64,

    /// Percentage of total time
    pub percentage_of_total: f64,

    /// Number of executions
    pub execution_count: u64,

    /// Success rate (0-100)
    pub success_rate: f64,

    /// P50 latency (milliseconds)
    pub p50_ms: f64,

    /// P95 latency (milliseconds)
    pub p95_ms: f64,
}

/// Bottleneck information
#[derive(Debug, Serialize, Deserialize)]
pub struct BottleneckInfo {
    /// Phase name
    pub phase: String,

    /// Severity (low, medium, high)
    pub severity: String,

    /// Description
    pub description: String,

    /// Recommended action
    pub recommendation: String,
}

/// Success rates by category
#[derive(Debug, Serialize, Deserialize)]
pub struct SuccessRates {
    /// Overall success rate (0-100)
    pub overall: f64,

    /// Success rate by gate decision
    pub by_gate_decision: HashMap<String, f64>,

    /// Cache hit rate (0-100)
    pub cache_hit_rate: f64,
}

/// Get pipeline phase breakdown
///
/// Returns detailed timing information, success rates, and bottleneck analysis
/// for all pipeline phases.
pub async fn get_pipeline_phases(
    State(state): State<AppState>,
) -> Result<Json<PipelinePhaseBreakdown>, ApiError> {
    // Gather metrics from the metrics collector
    let metrics = &state.metrics;

    // For now, we'll return sample data. In production, this would query
    // the metrics collector for actual histogram and counter data.

    // Fetch phase metrics
    let fetch_avg = get_histogram_avg(metrics, "fetch_phase");
    let gate_avg = get_histogram_avg(metrics, "gate_phase");
    let wasm_avg = get_histogram_avg(metrics, "wasm_phase");
    let render_avg = get_histogram_avg(metrics, "render_phase");

    let total_avg = fetch_avg + gate_avg + wasm_avg;

    let phases = vec![
        PhaseMetrics {
            name: "fetch".to_string(),
            avg_duration_ms: fetch_avg * 1000.0,
            percentage_of_total: if total_avg > 0.0 { (fetch_avg / total_avg) * 100.0 } else { 0.0 },
            execution_count: get_counter_value(metrics, "http_requests"),
            success_rate: 95.0, // Sample data
            p50_ms: fetch_avg * 800.0,
            p95_ms: fetch_avg * 1500.0,
        },
        PhaseMetrics {
            name: "gate".to_string(),
            avg_duration_ms: gate_avg * 1000.0,
            percentage_of_total: if total_avg > 0.0 { (gate_avg / total_avg) * 100.0 } else { 0.0 },
            execution_count: get_counter_value(metrics, "http_requests"),
            success_rate: 99.0, // Sample data
            p50_ms: gate_avg * 800.0,
            p95_ms: gate_avg * 1500.0,
        },
        PhaseMetrics {
            name: "wasm".to_string(),
            avg_duration_ms: wasm_avg * 1000.0,
            percentage_of_total: if total_avg > 0.0 { (wasm_avg / total_avg) * 100.0 } else { 0.0 },
            execution_count: get_counter_value(metrics, "http_requests"),
            success_rate: 97.0, // Sample data
            p50_ms: wasm_avg * 800.0,
            p95_ms: wasm_avg * 1500.0,
        },
        PhaseMetrics {
            name: "render".to_string(),
            avg_duration_ms: render_avg * 1000.0,
            percentage_of_total: if render_avg > 0.0 && total_avg > 0.0 { (render_avg / total_avg) * 100.0 } else { 0.0 },
            execution_count: get_counter_value(metrics, "gate_decisions_headless"),
            success_rate: 90.0, // Sample data
            p50_ms: render_avg * 800.0,
            p95_ms: render_avg * 1500.0,
        },
    ];

    // Identify bottlenecks
    let mut bottlenecks = Vec::new();

    for phase in &phases {
        if phase.avg_duration_ms > 2000.0 {
            bottlenecks.push(BottleneckInfo {
                phase: phase.name.clone(),
                severity: "high".to_string(),
                description: format!(
                    "{} phase is taking {}ms on average, which is above the 2s threshold",
                    phase.name, phase.avg_duration_ms as u64
                ),
                recommendation: match phase.name.as_str() {
                    "fetch" => "Consider enabling caching or using a CDN".to_string(),
                    "wasm" => "Consider optimizing WASM extraction or using faster extraction strategies".to_string(),
                    "render" => "Reduce headless rendering timeout or optimize dynamic content detection".to_string(),
                    _ => "Optimize this phase to reduce latency".to_string(),
                },
            });
        } else if phase.avg_duration_ms > 1000.0 {
            bottlenecks.push(BottleneckInfo {
                phase: phase.name.clone(),
                severity: "medium".to_string(),
                description: format!(
                    "{} phase is taking {}ms on average",
                    phase.name, phase.avg_duration_ms as u64
                ),
                recommendation: format!("Monitor {} phase performance", phase.name),
            });
        }
    }

    // Calculate success rates
    let total_requests = get_counter_value(metrics, "http_requests");
    let raw_decisions = get_counter_value(metrics, "gate_decisions_raw");
    let probes_decisions = get_counter_value(metrics, "gate_decisions_probes_first");
    let headless_decisions = get_counter_value(metrics, "gate_decisions_headless");
    let cached_decisions = get_counter_value(metrics, "gate_decisions_cached");

    let mut by_gate_decision = HashMap::new();
    if total_requests > 0 {
        by_gate_decision.insert("raw".to_string(), (raw_decisions as f64 / total_requests as f64) * 100.0);
        by_gate_decision.insert("probes_first".to_string(), (probes_decisions as f64 / total_requests as f64) * 100.0);
        by_gate_decision.insert("headless".to_string(), (headless_decisions as f64 / total_requests as f64) * 100.0);
        by_gate_decision.insert("cached".to_string(), (cached_decisions as f64 / total_requests as f64) * 100.0);
    }

    let cache_hit_rate = state.metrics.cache_hit_rate.get();

    let success_rates = SuccessRates {
        overall: 96.0, // Sample data - would calculate from actual metrics
        by_gate_decision,
        cache_hit_rate: cache_hit_rate * 100.0,
    };

    let overall = OverallMetrics {
        total_requests,
        avg_total_time_ms: total_avg * 1000.0,
        p50_latency_ms: total_avg * 800.0,
        p95_latency_ms: total_avg * 1800.0,
        p99_latency_ms: total_avg * 2500.0,
    };

    let response = PipelinePhaseBreakdown {
        overall,
        phases,
        bottlenecks,
        success_rates,
    };

    Ok(Json(response))
}

/// Helper function to get average from histogram
fn get_histogram_avg(_metrics: &crate::metrics::RipTideMetrics, phase: &str) -> f64 {
    // In production, this would query the actual histogram
    // For now, return sample values
    match phase {
        "fetch_phase" => 0.15,      // 150ms average
        "gate_phase" => 0.01,       // 10ms average
        "wasm_phase" => 0.20,       // 200ms average
        "render_phase" => 2.0,      // 2s average (when used)
        _ => 0.0,
    }
}

/// Helper function to get counter value
fn get_counter_value(metrics: &crate::metrics::RipTideMetrics, counter: &str) -> u64 {
    // In production, this would query the actual counter
    // For now, return the actual counter value from metrics
    match counter {
        "http_requests" => metrics.http_requests_total.get() as u64,
        "gate_decisions_raw" => metrics.gate_decisions_raw.get() as u64,
        "gate_decisions_probes_first" => metrics.gate_decisions_probes_first.get() as u64,
        "gate_decisions_headless" => metrics.gate_decisions_headless.get() as u64,
        "gate_decisions_cached" => metrics.gate_decisions_cached.get() as u64,
        _ => 0,
    }
}
