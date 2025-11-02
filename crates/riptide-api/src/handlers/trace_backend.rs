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

/// OTLP-compatible trace backend with support for Jaeger and Tempo query APIs
pub struct OtlpTraceBackend {
    endpoint: String,
    client: reqwest::Client,
    backend_type: OtlpBackendType,
}

/// Type of OTLP-compatible backend
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OtlpBackendType {
    /// Jaeger query API
    Jaeger,
    /// Grafana Tempo query API
    Tempo,
    /// Generic OTLP endpoint (no query support)
    Generic,
}

impl OtlpTraceBackend {
    /// Create a new OTLP trace backend
    pub fn new(endpoint: String, backend_type: OtlpBackendType) -> Self {
        Self {
            endpoint,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            backend_type,
        }
    }

    /// Create from environment variables
    ///
    /// Environment variables:
    /// - `OTLP_TRACE_QUERY_ENDPOINT`: Query endpoint URL (required)
    /// - `OTLP_TRACE_BACKEND_TYPE`: Backend type (jaeger, tempo, generic) - default: jaeger
    pub fn from_env() -> Option<Self> {
        let endpoint = std::env::var("OTLP_TRACE_QUERY_ENDPOINT").ok()?;

        let backend_type = std::env::var("OTLP_TRACE_BACKEND_TYPE")
            .unwrap_or_else(|_| "jaeger".to_string())
            .to_lowercase();

        let backend_type = match backend_type.as_str() {
            "jaeger" => OtlpBackendType::Jaeger,
            "tempo" => OtlpBackendType::Tempo,
            "generic" | _ => OtlpBackendType::Generic,
        };

        Some(Self::new(endpoint, backend_type))
    }

    /// Query traces from Jaeger backend
    async fn query_jaeger_traces(
        &self,
        time_range_secs: u64,
        limit: usize,
        service_filter: Option<String>,
    ) -> Result<Vec<TraceMetadata>, ApiError> {
        use serde_json::Value;

        // Jaeger Query API: GET /api/traces?service={service}&limit={limit}&lookback={lookback}
        let lookback = format!("{}s", time_range_secs);
        let service = service_filter.unwrap_or_else(|| "riptide-api".to_string());

        let url = format!(
            "{}/api/traces?service={}&limit={}&lookback={}",
            self.endpoint, service, limit, lookback
        );

        debug!(url = %url, "Querying Jaeger for traces");

        let response = self.client.get(&url).send().await.map_err(|e| {
            warn!(error = %e, "Failed to query Jaeger traces");
            ApiError::internal(&format!("Failed to query trace backend: {}", e))
        })?;

        if !response.status().is_success() {
            warn!(status = %response.status(), "Jaeger query returned error");
            return Ok(vec![]);
        }

        let data: Value = response
            .json()
            .await
            .map_err(|e| ApiError::internal(&format!("Failed to parse Jaeger response: {}", e)))?;

        // Parse Jaeger response format
        let traces = self.parse_jaeger_traces(data)?;
        Ok(traces)
    }

    /// Parse Jaeger trace list response
    fn parse_jaeger_traces(&self, data: serde_json::Value) -> Result<Vec<TraceMetadata>, ApiError> {
        let traces = data
            .get("data")
            .and_then(|d| d.as_array())
            .ok_or_else(|| ApiError::internal("Invalid Jaeger response format"))?;

        let mut results = Vec::new();

        for trace_data in traces {
            if let Some(metadata) = self.parse_jaeger_trace_metadata(trace_data) {
                results.push(metadata);
            }
        }

        Ok(results)
    }

    /// Parse individual Jaeger trace metadata
    fn parse_jaeger_trace_metadata(&self, trace_data: &serde_json::Value) -> Option<TraceMetadata> {
        let trace_id = trace_data.get("traceID")?.as_str()?;
        let spans = trace_data.get("spans")?.as_array()?;

        if spans.is_empty() {
            return None;
        }

        // Find root span
        let root_span = spans.iter().find(|s| {
            s.get("references")
                .and_then(|r| r.as_array())
                .map(|refs| refs.is_empty())
                .unwrap_or(true)
        })?;

        let root_span_id = root_span.get("spanID")?.as_str()?;
        let service_name = root_span
            .get("process")
            .and_then(|p| p.get("serviceName"))
            .and_then(|s| s.as_str())
            .unwrap_or("unknown");

        let start_time = root_span.get("startTime")?.as_u64()?;
        let duration = root_span.get("duration")?.as_u64()?;

        // Convert Jaeger timestamp (microseconds) to ISO 8601
        let start_time_dt = chrono::DateTime::from_timestamp(
            (start_time / 1_000_000) as i64,
            ((start_time % 1_000_000) * 1000) as u32,
        )?;

        // Extract status from tags
        let status = root_span
            .get("tags")
            .and_then(|tags| tags.as_array())
            .and_then(|tags| {
                tags.iter()
                    .find(|tag| tag.get("key").and_then(|k| k.as_str()) == Some("otel.status_code"))
            })
            .and_then(|tag| tag.get("value"))
            .and_then(|v| v.as_str())
            .unwrap_or("OK");

        // Extract attributes from tags
        let mut attributes = HashMap::new();
        if let Some(tags) = root_span.get("tags").and_then(|t| t.as_array()) {
            for tag in tags {
                if let (Some(key), Some(value)) = (
                    tag.get("key").and_then(|k| k.as_str()),
                    tag.get("value").and_then(|v| v.as_str()),
                ) {
                    attributes.insert(key.to_string(), value.to_string());
                }
            }
        }

        Some(TraceMetadata {
            trace_id: trace_id.to_string(),
            root_span_id: root_span_id.to_string(),
            start_time: start_time_dt.to_rfc3339(),
            duration_ms: duration / 1000, // Convert microseconds to milliseconds
            service_name: service_name.to_string(),
            span_count: spans.len(),
            status: status.to_string(),
            attributes,
        })
    }

    /// Get single trace from Jaeger
    async fn get_jaeger_trace(
        &self,
        trace_id: &TraceId,
    ) -> Result<Option<CompleteTrace>, ApiError> {
        use serde_json::Value;

        let trace_id_hex = format!("{:032x}", u128::from_be_bytes(trace_id.to_bytes()));
        let url = format!("{}/api/traces/{}", self.endpoint, trace_id_hex);

        debug!(url = %url, trace_id = %trace_id_hex, "Querying Jaeger for trace");

        let response = self.client.get(&url).send().await.map_err(|e| {
            warn!(error = %e, "Failed to get Jaeger trace");
            ApiError::internal(&format!("Failed to get trace: {}", e))
        })?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !response.status().is_success() {
            warn!(status = %response.status(), "Jaeger get trace returned error");
            return Ok(None);
        }

        let data: Value = response
            .json()
            .await
            .map_err(|e| ApiError::internal(&format!("Failed to parse Jaeger trace: {}", e)))?;

        // Parse Jaeger trace response
        self.parse_jaeger_complete_trace(data)
    }

    /// Parse complete Jaeger trace data
    fn parse_jaeger_complete_trace(
        &self,
        data: serde_json::Value,
    ) -> Result<Option<CompleteTrace>, ApiError> {
        let traces = data
            .get("data")
            .and_then(|d| d.as_array())
            .ok_or_else(|| ApiError::internal("Invalid Jaeger trace response"))?;

        if traces.is_empty() {
            return Ok(None);
        }

        let trace_data = &traces[0];
        let metadata = self
            .parse_jaeger_trace_metadata(trace_data)
            .ok_or_else(|| ApiError::internal("Failed to parse trace metadata"))?;

        let spans_data = trace_data
            .get("spans")
            .and_then(|s| s.as_array())
            .ok_or_else(|| ApiError::internal("No spans in trace"))?;

        let mut spans = Vec::new();
        for span_data in spans_data {
            if let Some(span) = self.parse_jaeger_span(span_data) {
                spans.push(span);
            }
        }

        Ok(Some(CompleteTrace { metadata, spans }))
    }

    /// Parse individual Jaeger span
    fn parse_jaeger_span(&self, span_data: &serde_json::Value) -> Option<TraceSpan> {
        let span_id_hex = span_data.get("spanID")?.as_str()?;
        let trace_id_hex = span_data.get("traceID")?.as_str()?;

        let span_id_bytes = hex::decode(span_id_hex).ok()?;
        let trace_id_bytes = hex::decode(trace_id_hex).ok()?;

        if span_id_bytes.len() != 8 || trace_id_bytes.len() != 16 {
            return None;
        }

        let mut span_id_arr = [0u8; 8];
        span_id_arr.copy_from_slice(&span_id_bytes);
        let span_id = SpanId::from_bytes(span_id_arr);

        let mut trace_id_arr = [0u8; 16];
        trace_id_arr.copy_from_slice(&trace_id_bytes);
        let trace_id = TraceId::from_bytes(trace_id_arr);

        // Parse parent span ID from references
        let parent_span_id = span_data
            .get("references")
            .and_then(|refs| refs.as_array())
            .and_then(|refs| refs.first())
            .and_then(|ref_data| ref_data.get("spanID"))
            .and_then(|id| id.as_str())
            .and_then(|id_hex| hex::decode(id_hex).ok())
            .and_then(|bytes| {
                if bytes.len() == 8 {
                    let mut arr = [0u8; 8];
                    arr.copy_from_slice(&bytes);
                    Some(SpanId::from_bytes(arr))
                } else {
                    None
                }
            });

        let name = span_data.get("operationName")?.as_str()?.to_string();

        let start_time_us = span_data.get("startTime")?.as_u64()?;
        let duration_us = span_data.get("duration")?.as_u64()?;

        let start_time = chrono::DateTime::from_timestamp(
            (start_time_us / 1_000_000) as i64,
            ((start_time_us % 1_000_000) * 1000) as u32,
        )?;

        let end_time = chrono::DateTime::from_timestamp(
            ((start_time_us + duration_us) / 1_000_000) as i64,
            (((start_time_us + duration_us) % 1_000_000) * 1000) as u32,
        )?;

        // Extract span kind and status from tags
        let mut kind = "INTERNAL".to_string();
        let mut status = "OK".to_string();
        let mut attributes = HashMap::new();

        if let Some(tags) = span_data.get("tags").and_then(|t| t.as_array()) {
            for tag in tags {
                if let (Some(key), Some(value)) =
                    (tag.get("key").and_then(|k| k.as_str()), tag.get("value"))
                {
                    match key {
                        "span.kind" => {
                            if let Some(k) = value.as_str() {
                                kind = k.to_uppercase();
                            }
                        }
                        "otel.status_code" => {
                            if let Some(s) = value.as_str() {
                                status = s.to_string();
                            }
                        }
                        _ => {
                            if let Some(v) = value.as_str() {
                                attributes.insert(key.to_string(), v.to_string());
                            }
                        }
                    }
                }
            }
        }

        // Parse events (logs in Jaeger)
        let mut events = Vec::new();
        if let Some(logs) = span_data.get("logs").and_then(|l| l.as_array()) {
            for log in logs {
                if let Some(event) = self.parse_jaeger_span_event(log) {
                    events.push(event);
                }
            }
        }

        Some(TraceSpan {
            span_id,
            trace_id,
            parent_span_id,
            name,
            kind,
            start_time,
            end_time,
            status,
            attributes,
            events,
        })
    }

    /// Parse Jaeger span event (log)
    fn parse_jaeger_span_event(&self, log_data: &serde_json::Value) -> Option<SpanEventData> {
        let timestamp_us = log_data.get("timestamp")?.as_u64()?;
        let timestamp = chrono::DateTime::from_timestamp(
            (timestamp_us / 1_000_000) as i64,
            ((timestamp_us % 1_000_000) * 1000) as u32,
        )?;

        let mut name = "event".to_string();
        let mut attributes = HashMap::new();

        if let Some(fields) = log_data.get("fields").and_then(|f| f.as_array()) {
            for field in fields {
                if let (Some(key), Some(value)) = (
                    field.get("key").and_then(|k| k.as_str()),
                    field.get("value").and_then(|v| v.as_str()),
                ) {
                    if key == "event" {
                        name = value.to_string();
                    } else {
                        attributes.insert(key.to_string(), value.to_string());
                    }
                }
            }
        }

        Some(SpanEventData {
            name,
            timestamp,
            attributes,
        })
    }
}

impl TraceBackend for OtlpTraceBackend {
    fn list_traces(
        &self,
        time_range_secs: u64,
        limit: usize,
        service_filter: Option<String>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<TraceMetadata>, ApiError>> + Send + '_>,
    > {
        let endpoint = self.endpoint.clone();
        let backend_type = self.backend_type;

        Box::pin(async move {
            match backend_type {
                OtlpBackendType::Jaeger => {
                    debug!(
                        endpoint = %endpoint,
                        time_range_secs,
                        limit,
                        "Querying Jaeger backend for traces"
                    );
                    self.query_jaeger_traces(time_range_secs, limit, service_filter)
                        .await
                }
                OtlpBackendType::Tempo | OtlpBackendType::Generic => {
                    warn!(
                        backend_type = ?backend_type,
                        endpoint = %endpoint,
                        "Tempo/Generic query not yet implemented - returning empty list"
                    );
                    Ok(vec![])
                }
            }
        })
    }

    fn get_trace(
        &self,
        trace_id: &TraceId,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Option<CompleteTrace>, ApiError>> + Send + '_>,
    > {
        let endpoint = self.endpoint.clone();
        let backend_type = self.backend_type;
        let trace_id = *trace_id;

        Box::pin(async move {
            match backend_type {
                OtlpBackendType::Jaeger => {
                    debug!(
                        endpoint = %endpoint,
                        trace_id = %format!("{:032x}", u128::from_be_bytes(trace_id.to_bytes())),
                        "Getting trace from Jaeger backend"
                    );
                    self.get_jaeger_trace(&trace_id).await
                }
                OtlpBackendType::Tempo | OtlpBackendType::Generic => {
                    warn!(
                        backend_type = ?backend_type,
                        endpoint = %endpoint,
                        "Tempo/Generic trace retrieval not yet implemented"
                    );
                    Ok(None)
                }
            }
        })
    }

    fn health_check(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send + '_>> {
        let client = self.client.clone();
        let endpoint = self.endpoint.clone();
        let backend_type = self.backend_type;

        Box::pin(async move {
            // Try backend-specific health check
            let health_url = match backend_type {
                OtlpBackendType::Jaeger => format!("{}/api/services", endpoint),
                OtlpBackendType::Tempo => format!("{}/api/echo", endpoint),
                OtlpBackendType::Generic => endpoint.clone(),
            };

            debug!(url = %health_url, "Checking trace backend health");

            match client
                .get(&health_url)
                .timeout(std::time::Duration::from_secs(5))
                .send()
                .await
            {
                Ok(response) => {
                    let is_healthy = response.status().is_success();
                    debug!(
                        url = %health_url,
                        status = %response.status(),
                        healthy = is_healthy,
                        "Trace backend health check result"
                    );
                    is_healthy
                }
                Err(e) => {
                    warn!(error = %e, url = %health_url, "Trace backend health check failed");
                    false
                }
            }
        })
    }

    fn backend_type(&self) -> &str {
        match self.backend_type {
            OtlpBackendType::Jaeger => "jaeger",
            OtlpBackendType::Tempo => "tempo",
            OtlpBackendType::Generic => "otlp",
        }
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
