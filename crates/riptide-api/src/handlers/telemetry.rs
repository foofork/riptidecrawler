//! Telemetry and Trace Visualization Handlers (TELEM-005)
//!
//! This module provides HTTP endpoints for:
//! - Viewing distributed traces
//! - Visualizing trace tree structures
//! - Querying trace metadata

use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

use crate::errors::ApiError;
use crate::context::ApplicationContext;
use crate::telemetry_config::parse_trace_id;

// parse_span_id used in tests
#[cfg(test)]
use crate::telemetry_config::parse_span_id;

/// Request parameters for trace query
#[derive(Debug, Deserialize)]
pub struct TraceQueryParams {
    /// Trace ID to query (32-char hex string)
    pub trace_id: Option<String>,

    /// Time range in seconds (default: 300 = 5 minutes)
    pub time_range_secs: Option<u64>,

    /// Maximum number of traces to return
    pub limit: Option<usize>,

    /// Filter by service name
    pub service: Option<String>,
}

/// Trace metadata response
#[derive(Debug, Clone, Serialize)]
pub struct TraceMetadata {
    /// Trace ID
    pub trace_id: String,

    /// Root span ID
    pub root_span_id: String,

    /// Trace start time (ISO 8601)
    pub start_time: String,

    /// Total trace duration in milliseconds
    pub duration_ms: u64,

    /// Service name
    pub service_name: String,

    /// Number of spans in the trace
    pub span_count: usize,

    /// Trace status (OK, ERROR, UNSET)
    pub status: String,

    /// Custom attributes
    pub attributes: HashMap<String, String>,
}

/// Individual span in a trace
#[derive(Debug, Serialize)]
pub struct SpanNode {
    /// Span ID
    pub span_id: String,

    /// Span name
    pub name: String,

    /// Parent span ID (if any)
    pub parent_span_id: Option<String>,

    /// Span kind (SERVER, CLIENT, INTERNAL, etc.)
    pub kind: String,

    /// Start time offset from trace start (milliseconds)
    pub start_offset_ms: u64,

    /// Span duration in milliseconds
    pub duration_ms: u64,

    /// Span status
    pub status: String,

    /// Span attributes
    pub attributes: HashMap<String, String>,

    /// Span events
    pub events: Vec<SpanEvent>,

    /// Child spans
    pub children: Vec<SpanNode>,
}

/// Event within a span
#[derive(Debug, Serialize)]
pub struct SpanEvent {
    /// Event name
    pub name: String,

    /// Event timestamp offset from span start (milliseconds)
    pub timestamp_offset_ms: u64,

    /// Event attributes
    pub attributes: HashMap<String, String>,
}

/// Trace tree structure response
#[derive(Debug, Serialize)]
pub struct TraceTreeResponse {
    /// Trace metadata
    pub metadata: TraceMetadata,

    /// Root span with nested children
    pub root_span: SpanNode,

    /// Summary statistics
    pub summary: TraceSummary,
}

/// Trace summary statistics
#[derive(Debug, Serialize)]
pub struct TraceSummary {
    /// Total number of spans
    pub total_spans: usize,

    /// Number of spans with errors
    pub error_count: usize,

    /// Average span duration
    pub avg_span_duration_ms: f64,

    /// Maximum span duration
    pub max_span_duration_ms: u64,

    /// Critical path duration (slowest path through the trace)
    pub critical_path_duration_ms: u64,

    /// Services involved in the trace
    pub services: Vec<String>,
}

/// List recent traces endpoint
///
/// Returns a list of recent traces with basic metadata.
/// Useful for discovering traces to investigate.
#[tracing::instrument(
    name = "list_traces",
    skip(state),
    fields(
        http.method = "GET",
        http.route = "/telemetry/traces",
        time_range_secs = query.time_range_secs.unwrap_or(300),
        limit = query.limit.unwrap_or(10)
    )
)]
pub async fn list_traces(
    State(state): State<ApplicationContext>,
    Query(query): Query<TraceQueryParams>,
) -> Result<Json<Vec<TraceMetadata>>, ApiError> {
    info!("Listing traces with query: {:?}", query);

    let time_range_secs = query.time_range_secs.unwrap_or(300);
    let limit = query.limit.unwrap_or(10);

    // Query from trace backend if available
    let traces = if let Some(ref backend) = state.trace_backend {
        backend
            .list_traces(time_range_secs, limit, query.service.clone())
            .await?
    } else {
        // Return empty list if no backend configured
        warn!("No trace backend configured - returning empty trace list");
        vec![]
    };

    debug!(
        trace_count = traces.len(),
        backend_available = state.trace_backend.is_some(),
        "Returning traces from backend"
    );

    Ok(Json(traces))
}

/// Get trace tree endpoint (TELEM-005)
///
/// Returns the complete trace tree structure with timing information.
/// Shows parent-child relationships and critical path analysis.
#[tracing::instrument(
    name = "get_trace_tree",
    skip(state),
    fields(
        http.method = "GET",
        http.route = "/telemetry/traces/:trace_id",
        trace_id = %query.trace_id.as_ref().unwrap_or(&"unknown".to_string())
    )
)]
pub async fn get_trace_tree(
    State(state): State<ApplicationContext>,
    Query(query): Query<TraceQueryParams>,
) -> Result<Json<TraceTreeResponse>, ApiError> {
    let trace_id_str = query
        .trace_id
        .ok_or_else(|| ApiError::invalid_request("trace_id parameter is required"))?;

    info!(trace_id = %trace_id_str, "Fetching trace tree");

    // Validate trace ID format and parse
    let trace_id = parse_trace_id(&trace_id_str).ok_or_else(|| {
        ApiError::invalid_request("Invalid trace ID format (expected 32-char hex)")
    })?;

    // Query trace from backend
    let trace_backend = state
        .trace_backend
        .as_ref()
        .ok_or_else(|| ApiError::internal("Trace backend not configured"))?;

    let trace = trace_backend
        .get_trace(&trace_id)
        .await?
        .ok_or_else(|| ApiError::not_found(format!("Trace {} not found", trace_id_str)))?;

    // Build trace tree from flat span list
    use crate::handlers::trace_backend::build_trace_tree;
    let (root_span, summary) = build_trace_tree(&trace)?;

    let response = TraceTreeResponse {
        metadata: trace.metadata.clone(),
        root_span,
        summary,
    };

    info!(
        trace_id = %response.metadata.trace_id,
        span_count = response.summary.total_spans,
        duration_ms = response.metadata.duration_ms,
        backend_type = trace_backend.backend_type(),
        "Trace tree retrieved successfully from backend"
    );

    Ok(Json(response))
}

/// Get telemetry status and configuration
#[tracing::instrument(
    name = "get_telemetry_status",
    skip(state),
    fields(
        http.method = "GET",
        http.route = "/telemetry/status"
    )
)]
pub async fn get_telemetry_status(
    State(state): State<ApplicationContext>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let config = crate::telemetry_config::TelemetryConfig::from_env();

    // Get trace backend status
    let (backend_type, backend_healthy) = if let Some(ref backend) = state.trace_backend {
        (
            backend.backend_type().to_string(),
            backend.health_check().await,
        )
    } else {
        ("none".to_string(), false)
    };

    // Extract runtime info from AppState - use ResourceFacade (Sprint 4.4)
    let _facade_status = state
        .resource_facade
        .get_status()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to get resource status: {}", e)))?;
    let resource_status = state.resource_manager.get_resource_status().await;
    let streaming_metrics = state.streaming.metrics().await;
    let circuit_breaker_state = {
        let cb = state.circuit_breaker.lock().await;
        if cb.is_open() {
            "open"
        } else if cb.is_half_open() {
            "half_open"
        } else {
            "closed"
        }
    };

    // Get worker service health (feature-gated)
    #[cfg(feature = "workers")]
    let (
        worker_overall_healthy,
        worker_queue_healthy,
        worker_pool_healthy,
        worker_scheduler_healthy,
    ) = {
        let worker_health = state.worker_service.health_check().await;
        (
            worker_health.overall_healthy,
            worker_health.queue_healthy,
            worker_health.worker_pool_healthy,
            worker_health.scheduler_healthy,
        )
    };
    #[cfg(not(feature = "workers"))]
    let (
        worker_overall_healthy,
        worker_queue_healthy,
        worker_pool_healthy,
        worker_scheduler_healthy,
    ) = {
        // Stub values when workers feature is disabled
        (true, true, true, true)
    };

    // Get spider state if available
    #[cfg(feature = "spider")]
    let spider_active = if let Some(ref spider) = state.spider {
        spider.get_crawl_state().await.active
    } else {
        false
    };
    #[cfg(not(feature = "spider"))]
    let spider_active = false;

    // Check spider enabled status (must be outside macro due to cfg! limitations)
    let spider_enabled = {
        #[cfg(feature = "spider")]
        {
            state.spider.is_some()
        }
        #[cfg(not(feature = "spider"))]
        {
            false
        }
    };

    let status = serde_json::json!({
        "enabled": config.enabled,
        "service_name": config.service_name,
        "service_version": config.service_version,
        "exporter_type": config.exporter_type,
        "otlp_endpoint": config.otlp_endpoint,
        "sampling_ratio": config.sampling_ratio,
        "trace_propagation_enabled": config.enable_trace_propagation,
        "trace_backend": {
            "type": backend_type,
            "healthy": backend_healthy,
            "configured": state.trace_backend.is_some(),
        },
        "features": {
            "distributed_tracing": true,
            "custom_attributes": true,
            "trace_visualization": true,
            "trace_export": config.enabled,
            "trace_storage": state.trace_backend.is_some(),
        },
        "runtime": {
            "resource_manager": {
                "memory_pressure": resource_status.memory_pressure,
                "degradation_score": resource_status.degradation_score,
                "headless_pool_available": resource_status.headless_pool_available,
                "headless_pool_total": resource_status.headless_pool_total,
                "pdf_available": resource_status.pdf_available,
                "pdf_total": resource_status.pdf_total,
                "memory_usage_mb": resource_status.memory_usage_mb,
                "rate_limit_hits": resource_status.rate_limit_hits,
            },
            "streaming": {
                "active_connections": streaming_metrics.active_connections,
                "total_connections": streaming_metrics.total_connections,
                "total_messages_sent": streaming_metrics.total_messages_sent,
                "total_messages_dropped": streaming_metrics.total_messages_dropped,
                "error_rate": streaming_metrics.error_rate,
                "memory_usage_bytes": streaming_metrics.memory_usage_bytes,
            },
            "circuit_breaker": {
                "state": circuit_breaker_state,
            },
            "worker_service": {
                "healthy": worker_overall_healthy,
                "queue_healthy": worker_queue_healthy,
                "pool_healthy": worker_pool_healthy,
                "scheduler_healthy": worker_scheduler_healthy,
            },
            "spider": {
                "enabled": spider_enabled,
                "active": spider_active,
            },
            "telemetry_system": {
                "enabled": state.telemetry.is_some(),
            },
        }
    });

    Ok(Json(status))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_id_validation() {
        let valid_trace_id = "0af7651916cd43dd8448eb211c80319c";
        assert!(parse_trace_id(valid_trace_id).is_some());

        let invalid_trace_id = "invalid";
        assert!(parse_trace_id(invalid_trace_id).is_none());
    }

    #[test]
    fn test_span_id_validation() {
        let valid_span_id = "b7ad6b7169203331";
        assert!(parse_span_id(valid_span_id).is_some());

        let invalid_span_id = "invalid";
        assert!(parse_span_id(invalid_span_id).is_none());
    }
}
