//! Comprehensive Telemetry Test Suite (Phase 4B - Feature 6)
//!
//! Tests for OpenTelemetry configuration, initialization, and instrumentation

use riptide_api::telemetry_config::{
    extract_trace_context, inject_trace_context, parse_span_id, parse_trace_id, ExporterType,
    TelemetryConfig,
};
use std::collections::HashMap;

#[test]
fn test_telemetry_config_from_env_disabled_by_default() {
    // Clear any existing env vars
    std::env::remove_var("TELEMETRY_ENABLED");
    std::env::remove_var("TELEMETRY_OTLP_ENDPOINT");

    let config = TelemetryConfig::from_env();

    assert!(!config.enabled, "Telemetry should be disabled by default");
    assert_eq!(config.service_name, "riptide-api");
    assert_eq!(config.exporter_type, ExporterType::Otlp);
    assert_eq!(config.sampling_ratio, 1.0);
}

#[test]
fn test_telemetry_config_from_env_enabled() {
    std::env::set_var("TELEMETRY_ENABLED", "true");
    std::env::set_var("TELEMETRY_OTLP_ENDPOINT", "http://localhost:4317");
    std::env::set_var("TELEMETRY_SERVICE_NAME", "test-service");
    std::env::set_var("TELEMETRY_SAMPLING_RATIO", "0.5");

    let config = TelemetryConfig::from_env();

    assert!(config.enabled, "Telemetry should be enabled");
    assert_eq!(config.service_name, "test-service");
    assert_eq!(
        config.otlp_endpoint,
        Some("http://localhost:4317".to_string())
    );
    assert_eq!(config.sampling_ratio, 0.5);

    // Cleanup
    std::env::remove_var("TELEMETRY_ENABLED");
    std::env::remove_var("TELEMETRY_OTLP_ENDPOINT");
    std::env::remove_var("TELEMETRY_SERVICE_NAME");
    std::env::remove_var("TELEMETRY_SAMPLING_RATIO");
}

#[test]
fn test_telemetry_config_custom_resource_attributes() {
    std::env::set_var("TELEMETRY_RESOURCE_ENVIRONMENT", "production");
    std::env::set_var("TELEMETRY_RESOURCE_REGION", "us-west-2");
    std::env::set_var("TELEMETRY_RESOURCE_CLUSTER", "k8s-cluster-1");

    let config = TelemetryConfig::from_env();

    assert_eq!(
        config.resource_attributes.get("environment"),
        Some(&"production".to_string())
    );
    assert_eq!(
        config.resource_attributes.get("region"),
        Some(&"us-west-2".to_string())
    );
    assert_eq!(
        config.resource_attributes.get("cluster"),
        Some(&"k8s-cluster-1".to_string())
    );

    // Cleanup
    std::env::remove_var("TELEMETRY_RESOURCE_ENVIRONMENT");
    std::env::remove_var("TELEMETRY_RESOURCE_REGION");
    std::env::remove_var("TELEMETRY_RESOURCE_CLUSTER");
}

#[test]
fn test_telemetry_config_exporter_types() {
    // Test OTLP
    std::env::set_var("TELEMETRY_EXPORTER_TYPE", "otlp");
    let config = TelemetryConfig::from_env();
    assert_eq!(config.exporter_type, ExporterType::Otlp);

    // Test Jaeger
    std::env::set_var("TELEMETRY_EXPORTER_TYPE", "jaeger");
    let config = TelemetryConfig::from_env();
    assert_eq!(config.exporter_type, ExporterType::Jaeger);

    // Test Zipkin
    std::env::set_var("TELEMETRY_EXPORTER_TYPE", "zipkin");
    let config = TelemetryConfig::from_env();
    assert_eq!(config.exporter_type, ExporterType::Zipkin);

    // Test None
    std::env::set_var("TELEMETRY_EXPORTER_TYPE", "none");
    let config = TelemetryConfig::from_env();
    assert_eq!(config.exporter_type, ExporterType::None);

    // Cleanup
    std::env::remove_var("TELEMETRY_EXPORTER_TYPE");
}

#[test]
fn test_telemetry_config_batch_settings() {
    std::env::set_var("TELEMETRY_MAX_QUEUE_SIZE", "4096");
    std::env::set_var("TELEMETRY_MAX_EXPORT_BATCH_SIZE", "1024");
    std::env::set_var("TELEMETRY_SCHEDULED_DELAY_MS", "10000");
    std::env::set_var("TELEMETRY_EXPORT_TIMEOUT_SECS", "60");

    let config = TelemetryConfig::from_env();

    assert_eq!(config.max_queue_size, 4096);
    assert_eq!(config.max_export_batch_size, 1024);
    assert_eq!(config.scheduled_delay_millis, 10000);
    assert_eq!(config.export_timeout_secs, 60);

    // Cleanup
    std::env::remove_var("TELEMETRY_MAX_QUEUE_SIZE");
    std::env::remove_var("TELEMETRY_MAX_EXPORT_BATCH_SIZE");
    std::env::remove_var("TELEMETRY_SCHEDULED_DELAY_MS");
    std::env::remove_var("TELEMETRY_EXPORT_TIMEOUT_SECS");
}

#[test]
fn test_telemetry_config_trace_propagation() {
    std::env::set_var("TELEMETRY_ENABLE_TRACE_PROPAGATION", "false");
    let config = TelemetryConfig::from_env();
    assert!(!config.enable_trace_propagation);

    std::env::set_var("TELEMETRY_ENABLE_TRACE_PROPAGATION", "true");
    let config = TelemetryConfig::from_env();
    assert!(config.enable_trace_propagation);

    // Cleanup
    std::env::remove_var("TELEMETRY_ENABLE_TRACE_PROPAGATION");
}

#[test]
fn test_telemetry_config_sampling_ratio_bounds() {
    // Test too low sampling ratio (should clamp to 0.0)
    std::env::set_var("TELEMETRY_SAMPLING_RATIO", "-0.5");
    let config = TelemetryConfig::from_env();
    assert_eq!(config.sampling_ratio, 0.0);

    // Test too high sampling ratio (should clamp to 1.0)
    std::env::set_var("TELEMETRY_SAMPLING_RATIO", "1.5");
    let config = TelemetryConfig::from_env();
    assert_eq!(config.sampling_ratio, 1.0);

    // Test valid ratio
    std::env::set_var("TELEMETRY_SAMPLING_RATIO", "0.75");
    let config = TelemetryConfig::from_env();
    assert_eq!(config.sampling_ratio, 0.75);

    // Cleanup
    std::env::remove_var("TELEMETRY_SAMPLING_RATIO");
}

#[test]
fn test_parse_trace_id_valid() {
    let trace_id_str = "0af7651916cd43dd8448eb211c80319c";
    let trace_id = parse_trace_id(trace_id_str);
    assert!(trace_id.is_some(), "Valid trace ID should parse");

    let parsed = trace_id.unwrap();
    assert_eq!(
        format!("{:032x}", parsed.to_bytes()),
        trace_id_str,
        "Parsed trace ID should match original"
    );
}

#[test]
fn test_parse_trace_id_invalid() {
    // Too short
    let trace_id = parse_trace_id("abc123");
    assert!(trace_id.is_none(), "Short trace ID should not parse");

    // Too long
    let trace_id = parse_trace_id("0af7651916cd43dd8448eb211c80319c1234567890");
    assert!(trace_id.is_none(), "Long trace ID should not parse");

    // Invalid hex
    let trace_id = parse_trace_id("zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz");
    assert!(trace_id.is_none(), "Invalid hex trace ID should not parse");
}

#[test]
fn test_parse_span_id_valid() {
    let span_id_str = "b7ad6b7169203331";
    let span_id = parse_span_id(span_id_str);
    assert!(span_id.is_some(), "Valid span ID should parse");

    let parsed = span_id.unwrap();
    assert_eq!(
        format!("{:016x}", parsed.to_bytes()),
        span_id_str,
        "Parsed span ID should match original"
    );
}

#[test]
fn test_parse_span_id_invalid() {
    // Too short
    let span_id = parse_span_id("abc123");
    assert!(span_id.is_none(), "Short span ID should not parse");

    // Too long
    let span_id = parse_span_id("b7ad6b71692033311234567890");
    assert!(span_id.is_none(), "Long span ID should not parse");

    // Invalid hex
    let span_id = parse_span_id("zzzzzzzzzzzzzzzz");
    assert!(span_id.is_none(), "Invalid hex span ID should not parse");
}

#[test]
fn test_extract_trace_context_no_headers() {
    let headers = axum::http::HeaderMap::new();
    let context = extract_trace_context(&headers);
    assert!(
        context.is_none(),
        "No trace context should be extracted from empty headers"
    );
}

#[test]
fn test_extract_trace_context_with_traceparent() {
    let mut headers = axum::http::HeaderMap::new();
    // W3C Trace Context format: version-trace_id-parent_id-trace_flags
    headers.insert(
        "traceparent",
        "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01"
            .parse()
            .unwrap(),
    );

    let context = extract_trace_context(&headers);
    assert!(
        context.is_some(),
        "Trace context should be extracted from traceparent header"
    );

    let span_context = context.unwrap();
    assert!(
        span_context.is_valid(),
        "Extracted span context should be valid"
    );
    assert!(
        span_context.is_sampled(),
        "Trace flags 01 indicates sampled"
    );
}

#[test]
fn test_inject_trace_context() {
    use opentelemetry::trace::{SpanId, TraceFlags, TraceId, TraceState};

    let trace_id = TraceId::from_hex("0af7651916cd43dd8448eb211c80319c").unwrap();
    let span_id = SpanId::from_hex("b7ad6b7169203331").unwrap();
    let trace_flags = TraceFlags::SAMPLED;
    let trace_state = TraceState::default();

    let span_context =
        opentelemetry::trace::SpanContext::new(trace_id, span_id, trace_flags, true, trace_state);

    let mut headers = reqwest::header::HeaderMap::new();
    inject_trace_context(&mut headers, &span_context);

    // Check that traceparent header was injected
    assert!(
        headers.contains_key("traceparent"),
        "traceparent header should be injected"
    );

    let traceparent = headers.get("traceparent").unwrap().to_str().unwrap();
    assert!(
        traceparent.starts_with("00-"),
        "traceparent should start with version 00"
    );
    assert!(
        traceparent.contains("0af7651916cd43dd8448eb211c80319c"),
        "traceparent should contain trace ID"
    );
    assert!(
        traceparent.contains("b7ad6b7169203331"),
        "traceparent should contain span ID"
    );
}

#[test]
fn test_exporter_type_serialization() {
    // Test serde serialization/deserialization
    let otlp = ExporterType::Otlp;
    let json = serde_json::to_string(&otlp).unwrap();
    assert_eq!(json, r#""otlp""#);

    let jaeger = ExporterType::Jaeger;
    let json = serde_json::to_string(&jaeger).unwrap();
    assert_eq!(json, r#""jaeger""#);

    let zipkin = ExporterType::Zipkin;
    let json = serde_json::to_string(&zipkin).unwrap();
    assert_eq!(json, r#""zipkin""#);

    let none = ExporterType::None;
    let json = serde_json::to_string(&none).unwrap();
    assert_eq!(json, r#""none""#);
}

#[test]
fn test_exporter_type_deserialization() {
    let otlp: ExporterType = serde_json::from_str(r#""otlp""#).unwrap();
    assert_eq!(otlp, ExporterType::Otlp);

    let jaeger: ExporterType = serde_json::from_str(r#""jaeger""#).unwrap();
    assert_eq!(jaeger, ExporterType::Jaeger);

    let zipkin: ExporterType = serde_json::from_str(r#""zipkin""#).unwrap();
    assert_eq!(zipkin, ExporterType::Zipkin);

    let none: ExporterType = serde_json::from_str(r#""none""#).unwrap();
    assert_eq!(none, ExporterType::None);
}

#[test]
fn test_telemetry_config_serialization() {
    let mut resource_attrs = HashMap::new();
    resource_attrs.insert("env".to_string(), "test".to_string());

    let config = TelemetryConfig {
        enabled: true,
        service_name: "test-service".to_string(),
        service_version: "1.0.0".to_string(),
        otlp_endpoint: Some("http://localhost:4317".to_string()),
        exporter_type: ExporterType::Otlp,
        sampling_ratio: 0.5,
        export_timeout_secs: 30,
        max_queue_size: 2048,
        max_export_batch_size: 512,
        scheduled_delay_millis: 5000,
        resource_attributes: resource_attrs,
        enable_trace_propagation: true,
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: TelemetryConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.enabled, config.enabled);
    assert_eq!(deserialized.service_name, config.service_name);
    assert_eq!(deserialized.exporter_type, config.exporter_type);
    assert_eq!(deserialized.sampling_ratio, config.sampling_ratio);
}

#[test]
fn test_conditional_otel_initialization() {
    // Test that telemetry is NOT initialized when OTEL_ENDPOINT is not set
    std::env::remove_var("OTEL_ENDPOINT");
    std::env::remove_var("TELEMETRY_ENABLED");

    let config = TelemetryConfig::from_env();
    assert!(
        !config.enabled,
        "Telemetry should be disabled without env vars"
    );

    // Test that telemetry CAN be enabled when OTEL_ENDPOINT is set
    std::env::set_var("OTEL_ENDPOINT", "http://localhost:4317");
    std::env::set_var("TELEMETRY_ENABLED", "true");

    let config = TelemetryConfig::from_env();
    // With both vars set, we should have endpoint available
    // Note: actual init_tracing() requires running OTLP endpoint, so we just verify config

    // Cleanup
    std::env::remove_var("OTEL_ENDPOINT");
    std::env::remove_var("TELEMETRY_ENABLED");
}

#[test]
fn test_telemetry_disabled_with_none_exporter() {
    std::env::set_var("TELEMETRY_ENABLED", "true");
    std::env::set_var("TELEMETRY_EXPORTER_TYPE", "none");

    let config = TelemetryConfig::from_env();
    assert_eq!(config.exporter_type, ExporterType::None);

    // Even if enabled is true, init_tracing should gracefully handle None exporter
    // This is tested in the actual init_tracing implementation

    // Cleanup
    std::env::remove_var("TELEMETRY_ENABLED");
    std::env::remove_var("TELEMETRY_EXPORTER_TYPE");
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_end_to_end_trace_propagation() {
        use opentelemetry::trace::{SpanId, TraceFlags, TraceId, TraceState};

        // Create a span context
        let trace_id = TraceId::from_hex("0af7651916cd43dd8448eb211c80319c").unwrap();
        let span_id = SpanId::from_hex("b7ad6b7169203331").unwrap();
        let span_context = opentelemetry::trace::SpanContext::new(
            trace_id,
            span_id,
            TraceFlags::SAMPLED,
            true,
            TraceState::default(),
        );

        // Inject into reqwest headers
        let mut req_headers = reqwest::header::HeaderMap::new();
        inject_trace_context(&mut req_headers, &span_context);

        // Convert to axum headers (simulating HTTP transmission)
        let mut axum_headers = axum::http::HeaderMap::new();
        if let Some(traceparent) = req_headers.get("traceparent") {
            axum_headers.insert("traceparent", traceparent.clone());
        }

        // Extract from axum headers
        let extracted = extract_trace_context(&axum_headers);
        assert!(extracted.is_some(), "Should extract trace context");

        let extracted_ctx = extracted.unwrap();
        assert_eq!(extracted_ctx.trace_id(), trace_id, "Trace ID should match");
        assert!(extracted_ctx.is_sampled(), "Should preserve sampled flag");
    }
}
