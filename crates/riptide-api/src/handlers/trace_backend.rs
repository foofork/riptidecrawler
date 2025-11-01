//! Trace Backend Integration (TELEM-005)
//!
//! This module provides trace storage and retrieval capabilities for distributed tracing.
//! It supports OTLP-compatible backends and provides in-memory storage for development.

use crate::errors::ApiError;
use crate::handlers::telemetry::{SpanEvent, SpanNode, TraceMetadata, TraceSummary};
use chrono::{DateTime, Utc};
use opentelemetry::trace::{SpanId, TraceId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Trait for trace backend storage and retrieval
pub trait TraceBackend: Send + Sync {
    /// List recent traces matching the given criteria
    fn list_traces(
        &self,
        time_range_secs: u64,
        limit: usize,
        service_filter: Option<String>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<TraceMetadata>, ApiError>> + Send + '_>,
    >;

    /// Get complete trace data including all spans
    fn get_trace(
        &self,
        trace_id: &TraceId,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Option<CompleteTrace>, ApiError>> + Send + '_>,
    >;

    /// Check if the backend is healthy and connected
    fn health_check(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send + '_>>;

    /// Get backend type identifier
    fn backend_type(&self) -> &str;
}

/// Complete trace data with all spans and metadata
#[derive(Debug, Clone)]
pub struct CompleteTrace {
    pub metadata: TraceMetadata,
    pub spans: Vec<TraceSpan>,
}

/// Individual span data from the backend
#[derive(Debug, Clone)]
pub struct TraceSpan {
    pub span_id: SpanId,
    pub trace_id: TraceId,
    pub parent_span_id: Option<SpanId>,
    pub name: String,
    pub kind: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub status: String,
    pub attributes: HashMap<String, String>,
    pub events: Vec<SpanEventData>,
}

/// Event data within a span
#[derive(Debug, Clone)]
pub struct SpanEventData {
    pub name: String,
    pub timestamp: DateTime<Utc>,
    pub attributes: HashMap<String, String>,
}

impl TraceSpan {
    /// Get span duration in milliseconds
    pub fn duration_ms(&self) -> u64 {
        (self.end_time - self.start_time).num_milliseconds() as u64
    }

    /// Convert to SpanNode for API response
    pub fn to_span_node(&self, trace_start: DateTime<Utc>) -> SpanNode {
        let start_offset_ms = (self.start_time - trace_start).num_milliseconds() as u64;
        let duration_ms = self.duration_ms();

        SpanNode {
            span_id: format!("{:016x}", u64::from_be_bytes(self.span_id.to_bytes())),
            name: self.name.clone(),
            parent_span_id: self
                .parent_span_id
                .map(|id| format!("{:016x}", u64::from_be_bytes(id.to_bytes()))),
            kind: self.kind.clone(),
            start_offset_ms,
            duration_ms,
            status: self.status.clone(),
            attributes: self.attributes.clone(),
            events: self
                .events
                .iter()
                .map(|e| SpanEvent {
                    name: e.name.clone(),
                    timestamp_offset_ms: (e.timestamp - self.start_time).num_milliseconds() as u64,
                    attributes: e.attributes.clone(),
                })
                .collect(),
            children: vec![],
        }
    }
}

/// In-memory trace backend for development and testing
pub struct InMemoryTraceBackend {
    traces: Arc<RwLock<HashMap<TraceId, CompleteTrace>>>,
}

impl InMemoryTraceBackend {
    /// Create a new in-memory trace backend
    pub fn new() -> Self {
        Self {
            traces: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Store a trace in memory (for testing/development)
    #[allow(dead_code)]
    pub async fn store_trace(&self, trace: CompleteTrace) {
        let trace_id = parse_trace_id_from_string(&trace.metadata.trace_id)
            .expect("Invalid trace ID in metadata");
        let mut traces = self.traces.write().await;
        traces.insert(trace_id, trace);
        debug!(trace_id = %trace_id, "Stored trace in memory backend");
    }

    /// Generate mock trace data for demonstration
    pub async fn populate_mock_data(&self) {
        use chrono::Duration;

        let trace_id = TraceId::from_bytes([
            0x0a, 0xf7, 0x65, 0x19, 0x16, 0xcd, 0x43, 0xdd, 0x84, 0x48, 0xeb, 0x21, 0x1c, 0x80,
            0x31, 0x9c,
        ]);

        let root_span_id = SpanId::from_bytes([0xb7, 0xad, 0x6b, 0x71, 0x69, 0x20, 0x33, 0x31]);
        let start_time = Utc::now() - Duration::minutes(5);

        let spans = vec![
            TraceSpan {
                span_id: root_span_id,
                trace_id,
                parent_span_id: None,
                name: "crawl_handler".to_string(),
                kind: "SERVER".to_string(),
                start_time,
                end_time: start_time + Duration::milliseconds(1234),
                status: "OK".to_string(),
                attributes: {
                    let mut attrs = HashMap::new();
                    attrs.insert("http.method".to_string(), "POST".to_string());
                    attrs.insert("http.route".to_string(), "/crawl".to_string());
                    attrs.insert("url_count".to_string(), "5".to_string());
                    attrs
                },
                events: vec![
                    SpanEventData {
                        name: "validation_complete".to_string(),
                        timestamp: start_time + Duration::milliseconds(10),
                        attributes: HashMap::new(),
                    },
                    SpanEventData {
                        name: "pipeline_started".to_string(),
                        timestamp: start_time + Duration::milliseconds(15),
                        attributes: HashMap::new(),
                    },
                ],
            },
            TraceSpan {
                span_id: SpanId::from_bytes([0x00, 0xf0, 0x67, 0xaa, 0x0b, 0xa9, 0x02, 0xb7]),
                trace_id,
                parent_span_id: Some(root_span_id),
                name: "pipeline.fetch".to_string(),
                kind: "INTERNAL".to_string(),
                start_time: start_time + Duration::milliseconds(20),
                end_time: start_time + Duration::milliseconds(470),
                status: "OK".to_string(),
                attributes: {
                    let mut attrs = HashMap::new();
                    attrs.insert("url".to_string(), "https://example.com".to_string());
                    attrs.insert("cache_hit".to_string(), "false".to_string());
                    attrs
                },
                events: vec![],
            },
            TraceSpan {
                span_id: SpanId::from_bytes([0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]),
                trace_id,
                parent_span_id: Some(root_span_id),
                name: "pipeline.gate".to_string(),
                kind: "INTERNAL".to_string(),
                start_time: start_time + Duration::milliseconds(480),
                end_time: start_time + Duration::milliseconds(505),
                status: "OK".to_string(),
                attributes: {
                    let mut attrs = HashMap::new();
                    attrs.insert("decision".to_string(), "raw".to_string());
                    attrs.insert("quality_score".to_string(), "0.85".to_string());
                    attrs
                },
                events: vec![],
            },
            TraceSpan {
                span_id: SpanId::from_bytes([0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10]),
                trace_id,
                parent_span_id: Some(root_span_id),
                name: "pipeline.extract".to_string(),
                kind: "INTERNAL".to_string(),
                start_time: start_time + Duration::milliseconds(510),
                end_time: start_time + Duration::milliseconds(1210),
                status: "OK".to_string(),
                attributes: {
                    let mut attrs = HashMap::new();
                    attrs.insert("extractor".to_string(), "wasm".to_string());
                    attrs.insert("word_count".to_string(), "1234".to_string());
                    attrs
                },
                events: vec![],
            },
        ];

        let metadata = TraceMetadata {
            trace_id: format!("{:032x}", u128::from_be_bytes(trace_id.to_bytes())),
            root_span_id: format!("{:016x}", u64::from_be_bytes(root_span_id.to_bytes())),
            start_time: start_time.to_rfc3339(),
            duration_ms: 1234,
            service_name: "riptide-api".to_string(),
            span_count: spans.len(),
            status: "OK".to_string(),
            attributes: {
                let mut attrs = HashMap::new();
                attrs.insert("http.method".to_string(), "POST".to_string());
                attrs.insert("http.route".to_string(), "/crawl".to_string());
                attrs
            },
        };

        let trace = CompleteTrace { metadata, spans };
        self.store_trace(trace).await;
    }
}

impl Default for InMemoryTraceBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl TraceBackend for InMemoryTraceBackend {
    fn list_traces(
        &self,
        _time_range_secs: u64,
        limit: usize,
        service_filter: Option<String>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<TraceMetadata>, ApiError>> + Send + '_>,
    > {
        Box::pin(async move {
            let traces = self.traces.read().await;
            let mut results: Vec<TraceMetadata> =
                traces.values().map(|t| t.metadata.clone()).collect();

            // Filter by service if specified
            if let Some(ref service_name) = service_filter {
                results.retain(|t| t.service_name.contains(service_name));
            }

            // Sort by start time (most recent first)
            results.sort_by(|a, b| b.start_time.cmp(&a.start_time));

            // Apply limit
            results.truncate(limit);

            debug!(
                trace_count = results.len(),
                service_filter = ?service_filter,
                "Listed traces from in-memory backend"
            );

            Ok(results)
        })
    }

    fn get_trace(
        &self,
        trace_id: &TraceId,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Option<CompleteTrace>, ApiError>> + Send + '_>,
    > {
        let trace_id = *trace_id;
        Box::pin(async move {
            let traces = self.traces.read().await;
            Ok(traces.get(&trace_id).cloned())
        })
    }

    fn health_check(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send + '_>> {
        Box::pin(async move {
            true // In-memory backend is always healthy
        })
    }

    fn backend_type(&self) -> &str {
        "in-memory"
    }
}

/// OTLP-compatible trace backend (placeholder for future implementation)
pub struct OtlpTraceBackend {
    endpoint: String,
    client: reqwest::Client,
}

impl OtlpTraceBackend {
    /// Create a new OTLP trace backend
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            client: reqwest::Client::new(),
        }
    }

    /// Create from environment variables
    pub fn from_env() -> Option<Self> {
        let endpoint = std::env::var("OTLP_TRACE_QUERY_ENDPOINT").ok()?;
        Some(Self::new(endpoint))
    }
}

impl TraceBackend for OtlpTraceBackend {
    fn list_traces(
        &self,
        _time_range_secs: u64,
        _limit: usize,
        _service_filter: Option<String>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<TraceMetadata>, ApiError>> + Send + '_>,
    > {
        let endpoint = self.endpoint.clone();
        Box::pin(async move {
            // NOTE: OTLP doesn't have a standardized query API
            // This would need to be implemented based on the specific backend:
            // - Jaeger: Use Jaeger Query API
            // - Tempo: Use Tempo Query API
            // - Other: Use vendor-specific API

            warn!(
                endpoint = %endpoint,
                "OTLP trace query not yet implemented - falling back to empty result"
            );

            Ok(vec![])
        })
    }

    fn get_trace(
        &self,
        _trace_id: &TraceId,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Option<CompleteTrace>, ApiError>> + Send + '_>,
    > {
        let endpoint = self.endpoint.clone();
        Box::pin(async move {
            warn!(
                endpoint = %endpoint,
                "OTLP trace retrieval not yet implemented"
            );

            Ok(None)
        })
    }

    fn health_check(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send + '_>> {
        let client = self.client.clone();
        let endpoint = self.endpoint.clone();
        Box::pin(async move {
            // Try to ping the OTLP endpoint
            client
                .get(&endpoint)
                .timeout(std::time::Duration::from_secs(5))
                .send()
                .await
                .is_ok()
        })
    }

    fn backend_type(&self) -> &str {
        "otlp"
    }
}

/// Build a trace tree from flat span list
pub fn build_trace_tree(trace: &CompleteTrace) -> Result<(SpanNode, TraceSummary), ApiError> {
    let spans = &trace.spans;

    if spans.is_empty() {
        return Err(ApiError::not_found("No spans found in trace"));
    }

    // Find root span (no parent)
    let root_span = spans
        .iter()
        .find(|s| s.parent_span_id.is_none())
        .ok_or_else(|| ApiError::internal("No root span found in trace"))?;

    let trace_start = root_span.start_time;

    // Build span lookup map
    let span_map: HashMap<SpanId, &TraceSpan> = spans.iter().map(|s| (s.span_id, s)).collect();

    // Recursive function to build tree
    fn build_node(
        span: &TraceSpan,
        span_map: &HashMap<SpanId, &TraceSpan>,
        trace_start: DateTime<Utc>,
    ) -> SpanNode {
        let mut node = span.to_span_node(trace_start);

        // Find and add children
        let children: Vec<SpanNode> = span_map
            .values()
            .filter(|s| s.parent_span_id == Some(span.span_id))
            .map(|child| build_node(child, span_map, trace_start))
            .collect();

        node.children = children;
        node
    }

    let root_node = build_node(root_span, &span_map, trace_start);

    // Calculate summary statistics
    let total_spans = spans.len();
    let error_count = spans.iter().filter(|s| s.status == "ERROR").count();

    let durations: Vec<u64> = spans.iter().map(|s| s.duration_ms()).collect();
    let avg_span_duration_ms = if !durations.is_empty() {
        durations.iter().sum::<u64>() as f64 / durations.len() as f64
    } else {
        0.0
    };

    let max_span_duration_ms = durations.iter().copied().max().unwrap_or(0);
    let critical_path_duration_ms = root_span.duration_ms();

    let services: Vec<String> = spans
        .iter()
        .filter_map(|s| s.attributes.get("service.name").cloned())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let summary = TraceSummary {
        total_spans,
        error_count,
        avg_span_duration_ms,
        max_span_duration_ms,
        critical_path_duration_ms,
        services,
    };

    Ok((root_node, summary))
}

/// Parse trace ID from hex string
fn parse_trace_id_from_string(trace_id_str: &str) -> Option<TraceId> {
    if trace_id_str.len() == 32 {
        let bytes = hex::decode(trace_id_str).ok()?;
        if bytes.len() == 16 {
            let mut arr = [0u8; 16];
            arr.copy_from_slice(&bytes);
            Some(TraceId::from_bytes(arr))
        } else {
            None
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_backend() {
        let backend = InMemoryTraceBackend::new();
        backend.populate_mock_data().await;

        let traces = backend.list_traces(300, 10, None).await.unwrap();
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].service_name, "riptide-api");
    }

    #[tokio::test]
    async fn test_trace_tree_building() {
        let backend = InMemoryTraceBackend::new();
        backend.populate_mock_data().await;

        let traces = backend.list_traces(300, 1, None).await.unwrap();
        let trace_id = parse_trace_id_from_string(&traces[0].trace_id).unwrap();

        let trace = backend.get_trace(&trace_id).await.unwrap().unwrap();
        let (root, summary) = build_trace_tree(&trace).unwrap();

        assert_eq!(root.name, "crawl_handler");
        assert_eq!(root.children.len(), 3);
        assert_eq!(summary.total_spans, 4);
        assert_eq!(summary.error_count, 0);
    }
}
