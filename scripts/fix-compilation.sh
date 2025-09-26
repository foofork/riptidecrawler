#!/bin/bash
echo "🔧 Fixing compilation errors..."

# Fix all the compilation issues in batch
cat > /tmp/fixes.txt << 'EOF'
# Fix 1: monitoring/reports.rs - dereference mutex guard
/workspaces/riptide/crates/riptide-core/src/monitoring/reports.rs:101:            &*LockManager::acquire_mutex(request_rates, "request_rates")?,

# Fix 2: extract.rs - make constructor async
/workspaces/riptide/crates/riptide-core/src/extract.rs:10-11:
    pub async fn new(wasm_path: &str) -> Result<Self> {
        let cm_extractor = CmExtractor::new(wasm_path).await?;

# Fix 3: monitoring/alerts.rs - remove Deserialize for Instant
/workspaces/riptide/crates/riptide-core/src/monitoring/alerts.rs:38:#[derive(Debug, Clone, Serialize)]

# Fix 4: fetch.rs - fix Result unwrapping in tests
/workspaces/riptide/crates/riptide-core/src/fetch.rs:461-462:
            ReliableHttpClient::new(RetryConfig::default(), CircuitBreakerConfig::default())
                .unwrap()
                .with_robots_manager(RobotsConfig::default());

/workspaces/riptide/crates/riptide-core/src/fetch.rs:517-519:
        let client = client.unwrap();
        assert_eq!(client.calculate_delay(0), Duration::from_millis(100));
        assert_eq!(client.calculate_delay(1), Duration::from_millis(200));
        assert_eq!(client.calculate_delay(2), Duration::from_millis(400));

# Fix 5: telemetry.rs - fix OpenTelemetry API changes
/workspaces/riptide/crates/riptide-core/src/telemetry.rs:111-119:
    // Updated OpenTelemetry API
    use opentelemetry_otlp::WithExportConfig;
    let otlp_exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(endpoint);

    let tracer_provider = opentelemetry_sdk::trace::TracerProvider::builder()
        .with_batch_exporter(
            opentelemetry_otlp::new_pipeline()
                .tracing()
                .with_exporter(otlp_exporter)
                .build_span_exporter()?,
            runtime::Tokio
        )
        .with_config(

# Fix 6: telemetry.rs - fix sysinfo API changes
/workspaces/riptide/crates/riptide-core/src/telemetry.rs:520-522:
        // Network metrics removed in sysinfo 0.32
        // system.refresh_networks_list();
        // for (_interface_name, network_data) in system.networks() {

EOF

echo "✅ Fix script prepared. Apply fixes manually for safety."