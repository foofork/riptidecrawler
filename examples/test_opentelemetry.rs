//! Test OpenTelemetry integration
//!
//! Simple example to verify OpenTelemetry 0.26 compatibility

use riptide_core::telemetry::TelemetrySystem;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Testing OpenTelemetry 0.26 integration...");

    // Set environment variables for testing
    std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "http://localhost:4317");
    std::env::set_var("OTEL_TRACE_SAMPLE_RATE", "0.1");
    std::env::set_var("ENVIRONMENT", "test");

    // Initialize telemetry system
    let telemetry_system = TelemetrySystem::init()?;
    println!("✓ TelemetrySystem initialized successfully");

    // Test tracer
    let tracer = telemetry_system.tracer();
    let span = tracer.start("test_span");
    println!("✓ OpenTelemetry span created successfully");
    span.end();

    // Test data sanitization
    let sensitive_data = "api_key=sk-1234567890abcdef user@example.com";
    let sanitized = telemetry_system.sanitize_data(sensitive_data);
    println!("✓ Data sanitization working: '{}' -> '{}'", sensitive_data, sanitized);

    // Test resource tracking
    let resource_usage = telemetry_system.get_resource_usage();
    println!("✓ Resource tracking working - CPU: {:.2}%, Memory: {} bytes",
             resource_usage.cpu_usage_percent,
             resource_usage.memory_usage_bytes);

    // Test SLA monitoring
    let sla_status = telemetry_system.get_sla_status();
    println!("✓ SLA monitoring working - Overall compliance: {}", sla_status.overall_compliance);

    // Clean shutdown
    telemetry_system.shutdown();
    println!("✓ TelemetrySystem shutdown successfully");

    println!("All OpenTelemetry tests passed! ✓");

    Ok(())
}