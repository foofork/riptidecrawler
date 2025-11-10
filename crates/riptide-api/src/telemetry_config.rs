//! Telemetry Configuration and Utilities (TELEM-006, TELEM-007)
//!
//! This module provides comprehensive OpenTelemetry configuration including:
//! - OTLP exporter setup (Jaeger, Zipkin, generic OTLP)
//! - Trace context extraction and propagation
//! - Telemetry configuration from environment variables
//! - Span utilities for consistent instrumentation

use opentelemetry::trace::{SpanContext, SpanId, TraceId};

use opentelemetry::trace::TraceContextExt;
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::{self as sdktrace, RandomIdGenerator, Sampler};
use opentelemetry_sdk::Resource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;
use tracing::warn;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Telemetry configuration loaded from environment variables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// Enable telemetry export (default: false)
    pub enabled: bool,

    /// Service name for tracing (default: "riptide-api")
    pub service_name: String,

    /// Service version
    pub service_version: String,

    /// OTLP exporter endpoint (e.g., "http://localhost:4317")
    pub otlp_endpoint: Option<String>,

    /// Exporter type: "otlp", "jaeger", "zipkin", or "none"
    pub exporter_type: ExporterType,

    /// Sampling ratio (0.0 to 1.0, default: 1.0 = 100%)
    pub sampling_ratio: f64,

    /// Export timeout in seconds (default: 30)
    pub export_timeout_secs: u64,

    /// Maximum queue size for spans (default: 2048)
    pub max_queue_size: usize,

    /// Maximum batch size for export (default: 512)
    pub max_export_batch_size: usize,

    /// Scheduled delay for batch export in milliseconds (default: 5000)
    pub scheduled_delay_millis: u64,

    /// Custom resource attributes
    pub resource_attributes: HashMap<String, String>,

    /// Enable trace context propagation in HTTP headers
    pub enable_trace_propagation: bool,
}

/// Type of telemetry exporter to use
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ExporterType {
    /// OpenTelemetry Protocol (OTLP) - recommended
    #[default]
    Otlp,
    /// Jaeger native protocol
    Jaeger,
    /// Zipkin protocol
    Zipkin,
    /// No exporter (telemetry disabled)
    None,
}

impl FromStr for ExporterType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "otlp" => Ok(Self::Otlp),
            "jaeger" => Ok(Self::Jaeger),
            "zipkin" => Ok(Self::Zipkin),
            "none" => Ok(Self::None),
            _ => Err(format!("Unknown exporter type: {}", s)),
        }
    }
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

impl TelemetryConfig {
    /// Load telemetry configuration from environment variables
    pub fn from_env() -> Self {
        let enabled = std::env::var("TELEMETRY_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        let service_name =
            std::env::var("TELEMETRY_SERVICE_NAME").unwrap_or_else(|_| "riptide-api".to_string());

        let service_version = std::env::var("TELEMETRY_SERVICE_VERSION")
            .or_else(|_| std::env::var("CARGO_PKG_VERSION"))
            .unwrap_or_else(|_| "0.1.0".to_string());

        let otlp_endpoint = std::env::var("TELEMETRY_OTLP_ENDPOINT").ok();

        let exporter_type = std::env::var("TELEMETRY_EXPORTER_TYPE")
            .unwrap_or_else(|_| "otlp".to_string())
            .parse()
            .unwrap_or(ExporterType::Otlp);

        let sampling_ratio: f64 = std::env::var("TELEMETRY_SAMPLING_RATIO")
            .unwrap_or_else(|_| "1.0".to_string())
            .parse::<f64>()
            .unwrap_or(1.0)
            .clamp(0.0, 1.0);

        let export_timeout_secs = std::env::var("TELEMETRY_EXPORT_TIMEOUT_SECS")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .unwrap_or(30);

        let max_queue_size = std::env::var("TELEMETRY_MAX_QUEUE_SIZE")
            .unwrap_or_else(|_| "2048".to_string())
            .parse()
            .unwrap_or(2048);

        let max_export_batch_size = std::env::var("TELEMETRY_MAX_EXPORT_BATCH_SIZE")
            .unwrap_or_else(|_| "512".to_string())
            .parse()
            .unwrap_or(512);

        let scheduled_delay_millis = std::env::var("TELEMETRY_SCHEDULED_DELAY_MS")
            .unwrap_or_else(|_| "5000".to_string())
            .parse()
            .unwrap_or(5000);

        let enable_trace_propagation = std::env::var("TELEMETRY_ENABLE_TRACE_PROPAGATION")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        // Parse custom resource attributes from env vars with TELEMETRY_RESOURCE_ prefix
        let mut resource_attributes = HashMap::new();
        for (key, value) in std::env::vars() {
            if let Some(attr_name) = key.strip_prefix("TELEMETRY_RESOURCE_") {
                resource_attributes.insert(attr_name.to_lowercase(), value);
            }
        }

        Self {
            enabled,
            service_name,
            service_version,
            otlp_endpoint,
            exporter_type,
            sampling_ratio,
            export_timeout_secs,
            max_queue_size,
            max_export_batch_size,
            scheduled_delay_millis,
            resource_attributes,
            enable_trace_propagation,
        }
    }

    /// Initialize the OpenTelemetry tracing subscriber
    #[allow(dead_code)] // Public API for application initialization, called from main.rs
    pub fn init_tracing(&self) -> anyhow::Result<()> {
        if !self.enabled || self.exporter_type == ExporterType::None {
            tracing::info!("Telemetry disabled, using default tracing subscriber");
            return Ok(());
        }

        // Set up trace context propagator for distributed tracing
        global::set_text_map_propagator(TraceContextPropagator::new());

        // Create tracer from provider (OpenTelemetry 0.26 API)
        let tracer = self.create_tracer()?;

        // Create OpenTelemetry tracing layer
        let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

        // Initialize tracing subscriber with OpenTelemetry
        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::from_default_env())
            .with(tracing_subscriber::fmt::layer())
            .with(telemetry_layer)
            .try_init()?;

        tracing::info!(
            service = %self.service_name,
            version = %self.service_version,
            exporter = ?self.exporter_type,
            endpoint = ?self.otlp_endpoint,
            sampling = self.sampling_ratio,
            "OpenTelemetry tracing initialized"
        );

        Ok(())
    }

    /// Create a tracer with configured exporter
    /// OpenTelemetry 0.26: install_batch() returns TracerProvider, call .tracer() to get Tracer
    #[allow(dead_code)] // Used by init_tracing
    fn create_tracer(&self) -> anyhow::Result<opentelemetry_sdk::trace::Tracer> {
        use opentelemetry::trace::TracerProvider as _;

        let mut resource_kvs = vec![
            KeyValue::new("service.name", self.service_name.clone()),
            KeyValue::new("service.version", self.service_version.clone()),
        ];

        // Add custom resource attributes
        for (key, value) in &self.resource_attributes {
            resource_kvs.push(KeyValue::new(key.clone(), value.clone()));
        }

        let resource = Resource::new(resource_kvs);

        let batch_config = sdktrace::BatchConfig::default();

        match &self.exporter_type {
            ExporterType::Otlp => {
                let endpoint = self
                    .otlp_endpoint
                    .clone()
                    .unwrap_or_else(|| "http://localhost:4317".to_string());

                let exporter = opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(endpoint)
                    .with_timeout(Duration::from_secs(self.export_timeout_secs));

                // install_batch() returns TracerProvider in 0.26
                let provider = opentelemetry_otlp::new_pipeline()
                    .tracing()
                    .with_exporter(exporter)
                    .with_trace_config(
                        sdktrace::Config::default()
                            .with_sampler(Sampler::TraceIdRatioBased(self.sampling_ratio))
                            .with_id_generator(RandomIdGenerator::default())
                            .with_resource(resource),
                    )
                    .with_batch_config(batch_config)
                    .install_batch(opentelemetry_sdk::runtime::Tokio)?;

                // Get tracer from provider
                Ok(provider.tracer(self.service_name.clone()))
            }
            ExporterType::Jaeger => {
                // Jaeger-specific configuration would go here
                // For now, fall back to OTLP
                warn!("Jaeger exporter not yet fully implemented, falling back to OTLP");
                self.create_otlp_tracer(resource, batch_config)
            }
            ExporterType::Zipkin => {
                // Zipkin-specific configuration would go here
                // For now, fall back to OTLP
                warn!("Zipkin exporter not yet fully implemented, falling back to OTLP");
                self.create_otlp_tracer(resource, batch_config)
            }
            ExporterType::None => {
                anyhow::bail!("Cannot create tracer with ExporterType::None")
            }
        }
    }

    /// Helper to create OTLP tracer
    #[allow(dead_code)] // Used by create_tracer
    fn create_otlp_tracer(
        &self,
        resource: Resource,
        batch_config: sdktrace::BatchConfig,
    ) -> anyhow::Result<opentelemetry_sdk::trace::Tracer> {
        use opentelemetry::trace::TracerProvider as _;

        let endpoint = self
            .otlp_endpoint
            .clone()
            .unwrap_or_else(|| "http://localhost:4317".to_string());

        let exporter = opentelemetry_otlp::new_exporter()
            .tonic()
            .with_endpoint(endpoint)
            .with_timeout(Duration::from_secs(self.export_timeout_secs));

        // install_batch() returns TracerProvider in 0.26
        let provider = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(exporter)
            .with_trace_config(
                sdktrace::Config::default()
                    .with_sampler(Sampler::TraceIdRatioBased(self.sampling_ratio))
                    .with_id_generator(RandomIdGenerator::default())
                    .with_resource(resource),
            )
            .with_batch_config(batch_config)
            .install_batch(opentelemetry_sdk::runtime::Tokio)?;

        // Get tracer from provider
        Ok(provider.tracer(self.service_name.clone()))
    }

    /// Shutdown telemetry (flush remaining spans)
    #[allow(dead_code)] // Public API for application shutdown, called from main.rs
    pub fn shutdown() {
        global::shutdown_tracer_provider();
    }
}

/// Extract trace context from HTTP headers (TELEM-004)
/// OpenTelemetry 0.26: Use global::get_text_map_propagator() for extract
pub fn extract_trace_context(headers: &axum::http::HeaderMap) -> Option<SpanContext> {
    use opentelemetry::propagation::Extractor;
    use opentelemetry::trace::TraceContextExt;

    struct HeaderExtractor<'a>(&'a axum::http::HeaderMap);

    impl<'a> Extractor for HeaderExtractor<'a> {
        fn get(&self, key: &str) -> Option<&str> {
            self.0.get(key).and_then(|v| v.to_str().ok())
        }

        fn keys(&self) -> Vec<&str> {
            self.0.keys().map(|k| k.as_str()).collect()
        }
    }

    let extractor = HeaderExtractor(headers);

    // Use global propagator (OpenTelemetry 0.26 API)
    let context = global::get_text_map_propagator(|propagator| propagator.extract(&extractor));

    let span_context = context.span().span_context().clone();
    if span_context.is_valid() {
        Some(span_context)
    } else {
        None
    }
}

/// Inject trace context into HTTP headers (TELEM-004)
/// OpenTelemetry 0.26: Use global::get_text_map_propagator() for inject
#[allow(dead_code)] // Public API for outbound HTTP requests, used when making external calls
pub fn inject_trace_context(headers: &mut reqwest::header::HeaderMap, span_context: &SpanContext) {
    use opentelemetry::propagation::Injector;

    struct HeaderInjector<'a>(&'a mut reqwest::header::HeaderMap);

    impl<'a> Injector for HeaderInjector<'a> {
        fn set(&mut self, key: &str, value: String) {
            if let Ok(header_name) = reqwest::header::HeaderName::from_bytes(key.as_bytes()) {
                if let Ok(header_value) = reqwest::header::HeaderValue::from_str(&value) {
                    self.0.insert(header_name, header_value);
                }
            }
        }
    }

    let mut injector = HeaderInjector(headers);

    // Create a context with the span
    let context = opentelemetry::Context::current().with_remote_span_context(span_context.clone());

    // Use global propagator (OpenTelemetry 0.26 API)
    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&context, &mut injector);
    });
}

/// Parse trace ID from string
pub fn parse_trace_id(trace_id_str: &str) -> Option<TraceId> {
    if trace_id_str.len() == 32 {
        // Hex string format
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

/// Parse span ID from string
#[allow(dead_code)] // Used in test modules
pub fn parse_span_id(span_id_str: &str) -> Option<SpanId> {
    if span_id_str.len() == 16 {
        // Hex string format
        let bytes = hex::decode(span_id_str).ok()?;
        if bytes.len() == 8 {
            let mut arr = [0u8; 8];
            arr.copy_from_slice(&bytes);
            Some(SpanId::from_bytes(arr))
        } else {
            None
        }
    } else {
        None
    }
}

/// Utility macros for consistent span creation
#[macro_export]
macro_rules! telemetry_span {
    ($name:expr) => {
        tracing::info_span!($name)
    };
    ($name:expr, $($field:tt)*) => {
        tracing::info_span!($name, $($field)*)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_config_default() {
        let config = TelemetryConfig::default();
        assert_eq!(config.service_name, "riptide-api");
        assert!(!config.enabled); // Default is disabled
        assert_eq!(config.exporter_type, ExporterType::Otlp);
    }

    #[test]
    fn test_exporter_type_parsing() {
        assert_eq!("otlp".parse::<ExporterType>().unwrap(), ExporterType::Otlp);
        assert_eq!(
            "jaeger".parse::<ExporterType>().unwrap(),
            ExporterType::Jaeger
        );
        assert_eq!(
            "zipkin".parse::<ExporterType>().unwrap(),
            ExporterType::Zipkin
        );
        assert_eq!("none".parse::<ExporterType>().unwrap(), ExporterType::None);
        assert!("invalid".parse::<ExporterType>().is_err());
    }

    #[test]
    fn test_trace_id_parsing() {
        let trace_id_str = "0af7651916cd43dd8448eb211c80319c";
        let trace_id = parse_trace_id(trace_id_str);
        assert!(trace_id.is_some());
    }

    #[test]
    fn test_span_id_parsing() {
        let span_id_str = "b7ad6b7169203331";
        let span_id = parse_span_id(span_id_str);
        assert!(span_id.is_some());
    }
}
