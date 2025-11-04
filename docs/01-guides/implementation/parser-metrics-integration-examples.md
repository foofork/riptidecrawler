# Parser Metrics Integration Examples

This document provides practical code examples for integrating the parser performance metrics into different components of the RipTide extraction system.

## Table of Contents

1. [ExtractionFacade Integration](#extractionfacade-integration)
2. [Hybrid Strategy Integration](#hybrid-strategy-integration)
3. [API Handler Integration](#api-handler-integration)
4. [Testing Examples](#testing-examples)

## ExtractionFacade Integration

### Basic Integration

```rust
use riptide_monitoring::parser_metrics::{
    ParserMetrics, ParserStrategy, ExecutionPath, ExecutionOutcome,
    record_extraction,
};
use std::time::Instant;
use anyhow::Result;

pub struct ExtractionFacade {
    wasm_extractor: WasmExtractor,
    native_extractor: NativeExtractor,
}

impl ExtractionFacade {
    /// Extract content with metrics tracking
    pub async fn extract(&self, url: &str) -> Result<ExtractionResult> {
        let start = Instant::now();
        let strategy = ParserStrategy::Wasm;
        let path = ExecutionPath::Direct;

        // Record attempt
        ParserMetrics::record_attempt(strategy, path);

        // Perform extraction
        match self.wasm_extractor.extract(url).await {
            Ok(data) => {
                let duration = start.elapsed().as_secs_f64();

                // Record successful extraction
                record_extraction(
                    strategy,
                    path,
                    duration,
                    ExecutionOutcome::Success,
                    Some(data.confidence),
                );

                Ok(data)
            }
            Err(e) => {
                let duration = start.elapsed().as_secs_f64();

                // Record failure
                ParserMetrics::record_result(strategy, path, ExecutionOutcome::Error);
                ParserMetrics::record_duration(strategy, path, duration);

                Err(e)
            }
        }
    }
}
```

### With Fallback Strategy

```rust
impl ExtractionFacade {
    /// Extract with fallback support and metrics
    pub async fn extract_with_fallback(&self, url: &str) -> Result<ExtractionResult> {
        let start = Instant::now();

        // Try WASM first
        let strategy = ParserStrategy::Wasm;
        let path = ExecutionPath::Direct;

        ParserMetrics::record_attempt(strategy, path);

        match self.wasm_extractor.extract(url).await {
            Ok(data) if data.confidence > 0.7 => {
                let duration = start.elapsed().as_secs_f64();
                record_extraction(
                    strategy,
                    path,
                    duration,
                    ExecutionOutcome::Success,
                    Some(data.confidence),
                );
                Ok(data)
            }
            Ok(data) => {
                // Low confidence, trigger fallback
                let wasm_duration = start.elapsed().as_secs_f64();
                ParserMetrics::record_result(strategy, path, ExecutionOutcome::Fallback);
                ParserMetrics::record_duration(strategy, path, wasm_duration);

                // Record fallback
                let fallback_strategy = ParserStrategy::Native;
                ParserMetrics::record_fallback(strategy, fallback_strategy, path);

                // Try native extraction
                let fallback_start = Instant::now();
                ParserMetrics::record_attempt(fallback_strategy, path);

                match self.native_extractor.extract(url).await {
                    Ok(native_data) => {
                        let duration = fallback_start.elapsed().as_secs_f64();
                        record_extraction(
                            fallback_strategy,
                            path,
                            duration,
                            ExecutionOutcome::Success,
                            Some(native_data.confidence),
                        );
                        Ok(native_data)
                    }
                    Err(e) => {
                        let duration = fallback_start.elapsed().as_secs_f64();
                        ParserMetrics::record_result(
                            fallback_strategy,
                            path,
                            ExecutionOutcome::Error,
                        );
                        ParserMetrics::record_duration(fallback_strategy, path, duration);
                        Err(e)
                    }
                }
            }
            Err(e) => {
                let duration = start.elapsed().as_secs_f64();
                ParserMetrics::record_result(strategy, path, ExecutionOutcome::Error);
                ParserMetrics::record_duration(strategy, path, duration);
                Err(e)
            }
        }
    }
}
```

## Hybrid Strategy Integration

### Strategy Router with Metrics

```rust
use riptide_monitoring::parser_metrics::{
    ParserMetrics, ParserStrategy, ExecutionPath, ExecutionOutcome,
};

pub struct HybridRouter {
    gate: GateAnalyzer,
    wasm: WasmExtractor,
    headless: HeadlessExtractor,
}

impl HybridRouter {
    /// Route extraction through appropriate strategy
    pub async fn route_extraction(&self, url: &str, html: &str) -> Result<ExtractionResult> {
        // Analyze content to determine strategy
        let gate_decision = self.gate.analyze(html).await?;

        let (strategy, path) = match gate_decision {
            GateDecision::Direct => (ParserStrategy::Wasm, ExecutionPath::Direct),
            GateDecision::Headless => (ParserStrategy::Headless, ExecutionPath::Headless),
            GateDecision::ProbesFirst => (ParserStrategy::Native, ExecutionPath::Direct),
        };

        let start = Instant::now();
        ParserMetrics::record_attempt(strategy, path);

        let result = match strategy {
            ParserStrategy::Wasm => self.wasm.extract(url).await,
            ParserStrategy::Headless => self.headless.extract(url).await,
            ParserStrategy::Native => self.native.extract(url).await,
            _ => unreachable!(),
        };

        let duration = start.elapsed().as_secs_f64();

        match result {
            Ok(data) => {
                record_extraction(
                    strategy,
                    path,
                    duration,
                    ExecutionOutcome::Success,
                    Some(data.confidence),
                );
                Ok(data)
            }
            Err(e) => {
                ParserMetrics::record_result(strategy, path, ExecutionOutcome::Error);
                ParserMetrics::record_duration(strategy, path, duration);
                Err(e)
            }
        }
    }

    /// Route with cascading fallback
    pub async fn route_with_cascade(&self, url: &str, html: &str) -> Result<ExtractionResult> {
        let strategies = vec![
            (ParserStrategy::Wasm, ExecutionPath::Direct),
            (ParserStrategy::Native, ExecutionPath::Direct),
            (ParserStrategy::Headless, ExecutionPath::Headless),
        ];

        let mut last_error = None;

        for (idx, (strategy, path)) in strategies.iter().enumerate() {
            let start = Instant::now();
            ParserMetrics::record_attempt(*strategy, *path);

            // If this is a fallback, record it
            if idx > 0 {
                let (prev_strategy, prev_path) = strategies[idx - 1];
                ParserMetrics::record_fallback(prev_strategy, *strategy, *path);
            }

            let result = match strategy {
                ParserStrategy::Wasm => self.wasm.extract(url).await,
                ParserStrategy::Native => self.native.extract(url).await,
                ParserStrategy::Headless => self.headless.extract(url).await,
                _ => unreachable!(),
            };

            let duration = start.elapsed().as_secs_f64();

            match result {
                Ok(data) => {
                    let outcome = if idx > 0 {
                        ExecutionOutcome::Fallback
                    } else {
                        ExecutionOutcome::Success
                    };

                    record_extraction(
                        *strategy,
                        *path,
                        duration,
                        outcome,
                        Some(data.confidence),
                    );

                    return Ok(data);
                }
                Err(e) => {
                    ParserMetrics::record_result(*strategy, *path, ExecutionOutcome::Error);
                    ParserMetrics::record_duration(*strategy, *path, duration);
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All strategies failed")))
    }
}
```

## API Handler Integration

### Axum Handler with Metrics

```rust
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use riptide_monitoring::parser_metrics::{
    ParserMetrics, ParserStrategy, ExecutionPath, ExecutionOutcome,
};

#[derive(Deserialize)]
pub struct ExtractionRequest {
    url: String,
    strategy: Option<String>,
}

#[derive(Serialize)]
pub struct ExtractionResponse {
    content: String,
    confidence: f32,
    strategy_used: String,
    duration_ms: u64,
}

pub async fn extract_handler(
    State(facade): State<Arc<ExtractionFacade>>,
    Json(req): Json<ExtractionRequest>,
) -> Result<Json<ExtractionResponse>, StatusCode> {
    let start = Instant::now();

    // Determine strategy
    let strategy = match req.strategy.as_deref() {
        Some("wasm") => ParserStrategy::Wasm,
        Some("native") => ParserStrategy::Native,
        Some("headless") => ParserStrategy::Headless,
        _ => ParserStrategy::Wasm, // Default
    };

    let path = ExecutionPath::Direct;

    // Record attempt
    ParserMetrics::record_attempt(strategy, path);

    // Perform extraction
    let result = facade.extract(&req.url, strategy).await;
    let duration = start.elapsed();
    let duration_secs = duration.as_secs_f64();

    match result {
        Ok(data) => {
            // Record success
            record_extraction(
                strategy,
                path,
                duration_secs,
                ExecutionOutcome::Success,
                Some(data.confidence),
            );

            Ok(Json(ExtractionResponse {
                content: data.content,
                confidence: data.confidence,
                strategy_used: strategy.as_str().to_string(),
                duration_ms: duration.as_millis() as u64,
            }))
        }
        Err(e) => {
            // Record failure
            ParserMetrics::record_result(strategy, path, ExecutionOutcome::Error);
            ParserMetrics::record_duration(strategy, path, duration_secs);

            tracing::error!("Extraction failed: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
```

### Metrics Endpoint

```rust
use axum::{routing::get, Router};
use axum_prometheus::{PrometheusMetricLayer, metrics_exporter_prometheus::PrometheusHandle};

pub fn create_app() -> Router {
    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    Router::new()
        .route("/extract", post(extract_handler))
        .route("/metrics", get(metrics_endpoint))
        .layer(prometheus_layer)
        .with_state(metric_handle)
}

async fn metrics_endpoint(
    State(handle): State<PrometheusHandle>,
) -> impl IntoResponse {
    handle.render()
}
```

## Testing Examples

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use riptide_monitoring::parser_metrics::{
        ParserMetrics, ParserStrategy, ExecutionPath, ExecutionOutcome,
    };

    #[test]
    fn test_metrics_recording() {
        // Record a successful extraction
        ParserMetrics::record_attempt(ParserStrategy::Wasm, ExecutionPath::Direct);
        ParserMetrics::record_result(
            ParserStrategy::Wasm,
            ExecutionPath::Direct,
            ExecutionOutcome::Success,
        );
        ParserMetrics::record_duration(ParserStrategy::Wasm, ExecutionPath::Direct, 0.5);
        ParserMetrics::record_confidence(ParserStrategy::Wasm, 0.95);
    }

    #[test]
    fn test_fallback_recording() {
        // Record a fallback event
        ParserMetrics::record_fallback(
            ParserStrategy::Wasm,
            ParserStrategy::Native,
            ExecutionPath::Direct,
        );
    }

    #[tokio::test]
    async fn test_extraction_with_metrics() {
        let facade = ExtractionFacade::new();
        let result = facade.extract("https://example.com").await;

        assert!(result.is_ok());
        // Metrics should be recorded automatically
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;
    use serde_json::json;

    #[tokio::test]
    async fn test_metrics_endpoint() {
        let app = create_app();

        // Make an extraction request
        let req = Request::builder()
            .method("POST")
            .uri("/extract")
            .header("content-type", "application/json")
            .body(json!({
                "url": "https://example.com",
                "strategy": "wasm"
            }).to_string())
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Check metrics endpoint
        let metrics_req = Request::builder()
            .method("GET")
            .uri("/metrics")
            .body(Body::empty())
            .unwrap();

        let metrics_resp = app.oneshot(metrics_req).await.unwrap();
        assert_eq!(metrics_resp.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(metrics_resp.into_body()).await.unwrap();
        let metrics_text = String::from_utf8(body.to_vec()).unwrap();

        // Verify metrics are present
        assert!(metrics_text.contains("riptide_extraction_parser_attempts_total"));
        assert!(metrics_text.contains("riptide_extraction_parser_duration_seconds"));
    }

    #[tokio::test]
    async fn test_fallback_metrics() {
        let facade = ExtractionFacade::new();

        // Trigger fallback scenario
        let result = facade.extract_with_fallback("https://spa-example.com").await;
        assert!(result.is_ok());

        // Check metrics endpoint to verify fallback was recorded
        let app = create_app();
        let req = Request::builder()
            .method("GET")
            .uri("/metrics")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        let body = hyper::body::to_bytes(resp.into_body()).await.unwrap();
        let metrics_text = String::from_utf8(body.to_vec()).unwrap();

        assert!(metrics_text.contains("riptide_extraction_parser_fallbacks_total"));
    }
}
```

### Load Testing with Metrics Verification

```rust
#[cfg(test)]
mod load_tests {
    use super::*;
    use std::sync::Arc;
    use tokio::task::JoinSet;

    #[tokio::test]
    async fn test_concurrent_extractions_metrics() {
        let facade = Arc::new(ExtractionFacade::new());
        let mut tasks = JoinSet::new();

        // Spawn 100 concurrent extraction tasks
        for i in 0..100 {
            let facade = Arc::clone(&facade);
            tasks.spawn(async move {
                let url = format!("https://example.com/page{}", i);
                facade.extract(&url).await
            });
        }

        // Wait for all tasks to complete
        let mut successes = 0;
        let mut failures = 0;

        while let Some(result) = tasks.join_next().await {
            match result {
                Ok(Ok(_)) => successes += 1,
                Ok(Err(_)) => failures += 1,
                Err(_) => failures += 1,
            }
        }

        println!("Successes: {}, Failures: {}", successes, failures);

        // Verify metrics were recorded
        // In a real test, you'd query the metrics endpoint here
    }
}
```

## Performance Optimization Examples

### Batch Metrics Recording

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct BatchMetricsRecorder {
    pending: Arc<Mutex<Vec<MetricEntry>>>,
    batch_size: usize,
}

struct MetricEntry {
    strategy: ParserStrategy,
    path: ExecutionPath,
    duration: f64,
    outcome: ExecutionOutcome,
    confidence: Option<f32>,
}

impl BatchMetricsRecorder {
    pub fn new(batch_size: usize) -> Self {
        Self {
            pending: Arc::new(Mutex::new(Vec::with_capacity(batch_size))),
            batch_size,
        }
    }

    pub async fn record(&self, entry: MetricEntry) {
        let mut pending = self.pending.lock().await;
        pending.push(entry);

        if pending.len() >= self.batch_size {
            self.flush_internal(&mut pending).await;
        }
    }

    pub async fn flush(&self) {
        let mut pending = self.pending.lock().await;
        self.flush_internal(&mut pending).await;
    }

    async fn flush_internal(&self, pending: &mut Vec<MetricEntry>) {
        for entry in pending.drain(..) {
            record_extraction(
                entry.strategy,
                entry.path,
                entry.duration,
                entry.outcome,
                entry.confidence,
            );
        }
    }
}
```

### Async Metrics Recording

```rust
use tokio::spawn;

pub async fn extract_with_async_metrics(
    facade: &ExtractionFacade,
    url: &str,
) -> Result<ExtractionResult> {
    let start = Instant::now();
    let strategy = ParserStrategy::Wasm;
    let path = ExecutionPath::Direct;

    // Record attempt synchronously (fast)
    ParserMetrics::record_attempt(strategy, path);

    // Perform extraction
    let result = facade.extract(url).await;
    let duration = start.elapsed().as_secs_f64();

    // Record remaining metrics asynchronously
    let strategy_copy = strategy;
    let path_copy = path;

    spawn(async move {
        match &result {
            Ok(data) => {
                record_extraction(
                    strategy_copy,
                    path_copy,
                    duration,
                    ExecutionOutcome::Success,
                    Some(data.confidence),
                );
            }
            Err(_) => {
                ParserMetrics::record_result(
                    strategy_copy,
                    path_copy,
                    ExecutionOutcome::Error,
                );
                ParserMetrics::record_duration(strategy_copy, path_copy, duration);
            }
        }
    });

    result
}
```

## Monitoring and Debugging

### Custom Metrics Collector

```rust
use std::collections::HashMap;

pub struct MetricsCollector {
    strategy_counts: HashMap<String, u64>,
    path_counts: HashMap<String, u64>,
    avg_durations: HashMap<String, f64>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            strategy_counts: HashMap::new(),
            path_counts: HashMap::new(),
            avg_durations: HashMap::new(),
        }
    }

    pub fn record_extraction(&mut self,
        strategy: ParserStrategy,
        path: ExecutionPath,
        duration: f64,
    ) {
        // Update counts
        *self.strategy_counts.entry(strategy.as_str().to_string())
            .or_insert(0) += 1;
        *self.path_counts.entry(path.as_str().to_string())
            .or_insert(0) += 1;

        // Update average duration
        let key = format!("{}_{}", strategy.as_str(), path.as_str());
        let current_avg = self.avg_durations.get(&key).copied().unwrap_or(0.0);
        let count = self.strategy_counts[strategy.as_str()];
        let new_avg = (current_avg * (count - 1) as f64 + duration) / count as f64;
        self.avg_durations.insert(key, new_avg);
    }

    pub fn get_summary(&self) -> String {
        format!(
            "Strategy counts: {:?}\nPath counts: {:?}\nAvg durations: {:?}",
            self.strategy_counts,
            self.path_counts,
            self.avg_durations
        )
    }
}
```

This completes the integration examples. You can now integrate these patterns into your codebase based on your specific requirements.
