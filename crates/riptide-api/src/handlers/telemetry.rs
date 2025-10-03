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
use tracing::{debug, info};

use crate::errors::ApiError;
use crate::state::AppState;
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
#[derive(Debug, Serialize)]
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
    State(_state): State<AppState>, // TODO: Wire up to actual trace backend
    Query(query): Query<TraceQueryParams>,
) -> Result<Json<Vec<TraceMetadata>>, ApiError> {
    info!("Listing traces with query: {:?}", query);

    // Note: In a real implementation, this would query a trace backend like Jaeger/Zipkin
    // For demonstration purposes, we'll return mock data

    let traces = vec![TraceMetadata {
        trace_id: "0af7651916cd43dd8448eb211c80319c".to_string(),
        root_span_id: "b7ad6b7169203331".to_string(),
        start_time: "2025-10-03T10:00:00Z".to_string(),
        duration_ms: 1234,
        service_name: "riptide-api".to_string(),
        span_count: 15,
        status: "OK".to_string(),
        attributes: {
            let mut attrs = HashMap::new();
            attrs.insert("http.method".to_string(), "POST".to_string());
            attrs.insert("http.route".to_string(), "/crawl".to_string());
            attrs
        },
    }];

    debug!(trace_count = traces.len(), "Returning traces");
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
    State(_state): State<AppState>, // TODO: Wire up to actual trace backend
    Query(query): Query<TraceQueryParams>,
) -> Result<Json<TraceTreeResponse>, ApiError> {
    let trace_id_str = query
        .trace_id
        .ok_or_else(|| ApiError::invalid_request("trace_id parameter is required"))?;

    info!(trace_id = %trace_id_str, "Fetching trace tree");

    // Validate trace ID format
    let _trace_id = parse_trace_id(&trace_id_str).ok_or_else(|| {
        ApiError::invalid_request("Invalid trace ID format (expected 32-char hex)")
    })?;

    // Note: In production, this would fetch from a trace backend
    // For demonstration, we'll return a mock trace tree

    let root_span = SpanNode {
        span_id: "b7ad6b7169203331".to_string(),
        name: "crawl_handler".to_string(),
        parent_span_id: None,
        kind: "SERVER".to_string(),
        start_offset_ms: 0,
        duration_ms: 1234,
        status: "OK".to_string(),
        attributes: {
            let mut attrs = HashMap::new();
            attrs.insert("http.method".to_string(), "POST".to_string());
            attrs.insert("http.route".to_string(), "/crawl".to_string());
            attrs.insert("url_count".to_string(), "5".to_string());
            attrs
        },
        events: vec![
            SpanEvent {
                name: "validation_complete".to_string(),
                timestamp_offset_ms: 10,
                attributes: HashMap::new(),
            },
            SpanEvent {
                name: "pipeline_started".to_string(),
                timestamp_offset_ms: 15,
                attributes: HashMap::new(),
            },
        ],
        children: vec![
            SpanNode {
                span_id: "00f067aa0ba902b7".to_string(),
                name: "pipeline.fetch".to_string(),
                parent_span_id: Some("b7ad6b7169203331".to_string()),
                kind: "INTERNAL".to_string(),
                start_offset_ms: 20,
                duration_ms: 450,
                status: "OK".to_string(),
                attributes: {
                    let mut attrs = HashMap::new();
                    attrs.insert("url".to_string(), "https://example.com".to_string());
                    attrs.insert("cache_hit".to_string(), "false".to_string());
                    attrs
                },
                events: vec![],
                children: vec![],
            },
            SpanNode {
                span_id: "0123456789abcdef".to_string(),
                name: "pipeline.gate".to_string(),
                parent_span_id: Some("b7ad6b7169203331".to_string()),
                kind: "INTERNAL".to_string(),
                start_offset_ms: 480,
                duration_ms: 25,
                status: "OK".to_string(),
                attributes: {
                    let mut attrs = HashMap::new();
                    attrs.insert("decision".to_string(), "raw".to_string());
                    attrs.insert("quality_score".to_string(), "0.85".to_string());
                    attrs
                },
                events: vec![],
                children: vec![],
            },
            SpanNode {
                span_id: "fedcba9876543210".to_string(),
                name: "pipeline.extract".to_string(),
                parent_span_id: Some("b7ad6b7169203331".to_string()),
                kind: "INTERNAL".to_string(),
                start_offset_ms: 510,
                duration_ms: 700,
                status: "OK".to_string(),
                attributes: {
                    let mut attrs = HashMap::new();
                    attrs.insert("extractor".to_string(), "wasm".to_string());
                    attrs.insert("word_count".to_string(), "1234".to_string());
                    attrs
                },
                events: vec![],
                children: vec![],
            },
        ],
    };

    let metadata = TraceMetadata {
        trace_id: trace_id_str,
        root_span_id: "b7ad6b7169203331".to_string(),
        start_time: "2025-10-03T10:00:00Z".to_string(),
        duration_ms: 1234,
        service_name: "riptide-api".to_string(),
        span_count: 4,
        status: "OK".to_string(),
        attributes: {
            let mut attrs = HashMap::new();
            attrs.insert("http.method".to_string(), "POST".to_string());
            attrs.insert("http.route".to_string(), "/crawl".to_string());
            attrs
        },
    };

    let summary = TraceSummary {
        total_spans: 4,
        error_count: 0,
        avg_span_duration_ms: 391.25,
        max_span_duration_ms: 700,
        critical_path_duration_ms: 1234,
        services: vec!["riptide-api".to_string()],
    };

    let response = TraceTreeResponse {
        metadata,
        root_span,
        summary,
    };

    info!(
        trace_id = %response.metadata.trace_id,
        span_count = response.summary.total_spans,
        duration_ms = response.metadata.duration_ms,
        "Trace tree retrieved successfully"
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
    State(_state): State<AppState>, // TODO: Use state for runtime telemetry info
) -> Result<Json<serde_json::Value>, ApiError> {
    let config = crate::telemetry_config::TelemetryConfig::from_env();

    let status = serde_json::json!({
        "enabled": config.enabled,
        "service_name": config.service_name,
        "service_version": config.service_version,
        "exporter_type": config.exporter_type,
        "otlp_endpoint": config.otlp_endpoint,
        "sampling_ratio": config.sampling_ratio,
        "trace_propagation_enabled": config.enable_trace_propagation,
        "features": {
            "distributed_tracing": true,
            "custom_attributes": true,
            "trace_visualization": true,
            "trace_export": config.enabled,
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
