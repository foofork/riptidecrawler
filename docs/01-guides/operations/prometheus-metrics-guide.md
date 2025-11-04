# Prometheus Metrics Guide for Parser Performance

This guide documents the Prometheus metrics added for monitoring parser performance, fallback behavior, and execution paths in the RipTide extraction system.

## Overview

The parser performance metrics provide detailed insights into:
- **Parser Strategy Usage**: Track which extraction strategies are being used (WASM, Native, CSS, Headless)
- **Execution Paths**: Monitor whether extractions go through direct or headless browser paths
- **Performance**: Measure execution duration for each strategy
- **Reliability**: Track success rates, fallbacks, and errors
- **Quality**: Monitor confidence scores for extraction results

## Metrics Reference

### 1. Parser Attempts Counter

**Metric Name**: `riptide_extraction_parser_attempts_total`

**Type**: Counter

**Description**: Total number of parser execution attempts by strategy and path

**Labels**:
- `strategy`: The extraction strategy used (`wasm`, `native`, `css`, `headless`)
- `path`: The execution path taken (`direct`, `headless`)

**Example Queries**:
```promql
# Total attempts by strategy
sum by (strategy) (riptide_extraction_parser_attempts_total)

# Attempts per second by path
rate(riptide_extraction_parser_attempts_total[5m])

# Strategy usage percentage
100 * rate(riptide_extraction_parser_attempts_total{strategy="wasm"}[5m]) /
      rate(riptide_extraction_parser_attempts_total[5m])
```

### 2. Parser Results Counter

**Metric Name**: `riptide_extraction_parser_results_total`

**Type**: Counter

**Description**: Parser execution results by strategy and outcome

**Labels**:
- `strategy`: The extraction strategy used (`wasm`, `native`, `css`, `headless`)
- `path`: The execution path taken (`direct`, `headless`)
- `outcome`: The execution outcome (`success`, `fallback`, `error`)

**Example Queries**:
```promql
# Success rate by strategy
sum by (strategy) (rate(riptide_extraction_parser_results_total{outcome="success"}[5m])) /
sum by (strategy) (rate(riptide_extraction_parser_results_total[5m]))

# Error rate percentage
100 * sum(rate(riptide_extraction_parser_results_total{outcome="error"}[5m])) /
      sum(rate(riptide_extraction_parser_results_total[5m]))

# Fallback rate by strategy
rate(riptide_extraction_parser_results_total{outcome="fallback"}[5m])
```

### 3. Parser Fallbacks Counter

**Metric Name**: `riptide_extraction_parser_fallbacks_total`

**Type**: Counter

**Description**: Number of fallback events between strategies

**Labels**:
- `from_strategy`: The strategy that failed (`wasm`, `native`, `css`, `headless`)
- `to_strategy`: The strategy fallen back to (`wasm`, `native`, `css`, `headless`)
- `path`: The execution path (`direct`, `headless`)

**Example Queries**:
```promql
# Most common fallback paths
topk(5, sum by (from_strategy, to_strategy) (
  rate(riptide_extraction_parser_fallbacks_total[5m])
))

# WASM to Native fallback rate
rate(riptide_extraction_parser_fallbacks_total{
  from_strategy="wasm",
  to_strategy="native"
}[5m])

# Fallback frequency by path
sum by (path) (rate(riptide_extraction_parser_fallbacks_total[5m]))
```

### 4. Parser Duration Histogram

**Metric Name**: `riptide_extraction_parser_duration_seconds`

**Type**: Histogram

**Description**: Parser execution duration in seconds

**Labels**:
- `strategy`: The extraction strategy used (`wasm`, `native`, `css`, `headless`)
- `path`: The execution path taken (`direct`, `headless`)

**Buckets**: 0.001, 0.005, 0.010, 0.050, 0.100, 0.500, 1.0 (1ms to 1s)

**Example Queries**:
```promql
# P95 latency by strategy
histogram_quantile(0.95, sum by (strategy, le) (
  rate(riptide_extraction_parser_duration_seconds_bucket[5m])
))

# Average duration by strategy
rate(riptide_extraction_parser_duration_seconds_sum[5m]) /
rate(riptide_extraction_parser_duration_seconds_count[5m])

# Percentage of requests under 100ms
100 * sum(rate(riptide_extraction_parser_duration_seconds_bucket{le="0.1"}[5m])) /
      sum(rate(riptide_extraction_parser_duration_seconds_count[5m]))
```

### 5. Confidence Score Histogram

**Metric Name**: `riptide_extraction_confidence_score`

**Type**: Histogram

**Description**: Extraction confidence scores by strategy

**Labels**:
- `strategy`: The extraction strategy used (`wasm`, `native`, `css`, `headless`)

**Buckets**: 0.0, 0.3, 0.6, 0.85, 0.95, 1.0 (confidence thresholds)

**Example Queries**:
```promql
# Average confidence by strategy
rate(riptide_extraction_confidence_score_sum[5m]) /
rate(riptide_extraction_confidence_score_count[5m])

# Percentage of high confidence (>0.85) extractions
100 * sum(rate(riptide_extraction_confidence_score_bucket{le="1.0"}[5m])) -
      sum(rate(riptide_extraction_confidence_score_bucket{le="0.85"}[5m])) /
      sum(rate(riptide_extraction_confidence_score_count[5m]))

# Low confidence extraction rate
rate(riptide_extraction_confidence_score_bucket{le="0.3"}[5m])
```

## Integration Points

### 1. ExtractionFacade Integration

Record metrics at the facade level to capture all extraction attempts:

```rust
use riptide_monitoring::parser_metrics::{
    ParserMetrics, ParserStrategy, ExecutionPath, ExecutionOutcome,
    record_extraction,
};
use std::time::Instant;

// In your extraction function
pub async fn extract(&self, url: &str) -> Result<ExtractionResult> {
    let start = Instant::now();
    let strategy = ParserStrategy::Wasm;
    let path = ExecutionPath::Direct;

    // Record attempt
    ParserMetrics::record_attempt(strategy, path);

    // Perform extraction
    let result = self.wasm_extractor.extract(url).await;

    // Record results
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
```

### 2. Hybrid Routing Integration

Record fallback events in hybrid extraction strategies:

```rust
// When falling back from WASM to Native
ParserMetrics::record_fallback(
    ParserStrategy::Wasm,
    ParserStrategy::Native,
    ExecutionPath::Direct,
);

// Record the new attempt
ParserMetrics::record_attempt(ParserStrategy::Native, ExecutionPath::Direct);
```

### 3. API Handler Integration

Expose metrics endpoint in your API:

```rust
use axum::{Router, routing::get};
use axum_prometheus::PrometheusMetricLayer;

pub fn create_router() -> Router {
    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    Router::new()
        .route("/metrics", get(|| async move {
            metric_handle.render()
        }))
        .layer(prometheus_layer)
}
```

## Grafana Dashboard Configuration

### Dashboard Panels

#### 1. Strategy Usage Overview
```json
{
  "title": "Parser Strategy Usage",
  "targets": [{
    "expr": "sum by (strategy) (rate(riptide_extraction_parser_attempts_total[5m]))",
    "legendFormat": "{{strategy}}"
  }],
  "type": "timeseries"
}
```

#### 2. Success Rate by Strategy
```json
{
  "title": "Extraction Success Rate",
  "targets": [{
    "expr": "sum by (strategy) (rate(riptide_extraction_parser_results_total{outcome=\"success\"}[5m])) / sum by (strategy) (rate(riptide_extraction_parser_results_total[5m]))",
    "legendFormat": "{{strategy}}"
  }],
  "type": "timeseries"
}
```

#### 3. P95 Latency by Strategy
```json
{
  "title": "P95 Extraction Latency",
  "targets": [{
    "expr": "histogram_quantile(0.95, sum by (strategy, le) (rate(riptide_extraction_parser_duration_seconds_bucket[5m])))",
    "legendFormat": "{{strategy}}"
  }],
  "type": "timeseries"
}
```

#### 4. Fallback Flow Sankey
```json
{
  "title": "Parser Fallback Flow",
  "targets": [{
    "expr": "sum by (from_strategy, to_strategy) (rate(riptide_extraction_parser_fallbacks_total[5m]))",
    "format": "table"
  }],
  "type": "sankey"
}
```

#### 5. Confidence Score Distribution
```json
{
  "title": "Extraction Confidence Distribution",
  "targets": [{
    "expr": "sum by (le) (rate(riptide_extraction_confidence_score_bucket[5m]))",
    "legendFormat": "{{le}}"
  }],
  "type": "heatmap"
}
```

### Complete Dashboard JSON

See `/workspaces/eventmesh/docs/grafana-parser-dashboard.json` for a complete pre-configured dashboard.

## Alerting Rules

### High Error Rate Alert
```yaml
- alert: HighParserErrorRate
  expr: |
    sum(rate(riptide_extraction_parser_results_total{outcome="error"}[5m])) /
    sum(rate(riptide_extraction_parser_results_total[5m])) > 0.05
  for: 5m
  labels:
    severity: warning
  annotations:
    summary: High parser error rate detected
    description: Parser error rate is {{ $value | humanizePercentage }} (threshold: 5%)
```

### High Fallback Rate Alert
```yaml
- alert: HighFallbackRate
  expr: |
    sum(rate(riptide_extraction_parser_fallbacks_total[5m])) /
    sum(rate(riptide_extraction_parser_attempts_total[5m])) > 0.20
  for: 10m
  labels:
    severity: warning
  annotations:
    summary: High parser fallback rate detected
    description: Parser fallback rate is {{ $value | humanizePercentage }} (threshold: 20%)
```

### Slow Parser Performance Alert
```yaml
- alert: SlowParserPerformance
  expr: |
    histogram_quantile(0.95,
      sum by (strategy, le) (
        rate(riptide_extraction_parser_duration_seconds_bucket[5m])
      )
    ) > 1.0
  for: 10m
  labels:
    severity: warning
  annotations:
    summary: Slow parser performance for {{$labels.strategy}}
    description: P95 latency is {{ $value }}s (threshold: 1s)
```

### Low Confidence Score Alert
```yaml
- alert: LowExtractionConfidence
  expr: |
    sum(rate(riptide_extraction_confidence_score_sum[5m])) /
    sum(rate(riptide_extraction_confidence_score_count[5m])) < 0.7
  for: 15m
  labels:
    severity: info
  annotations:
    summary: Low extraction confidence detected
    description: Average confidence score is {{ $value }} (threshold: 0.7)
```

## Testing

### 1. Verify Metrics Registration
```bash
# Check metrics endpoint
curl http://localhost:3000/metrics | grep riptide_extraction_parser

# Expected output:
# riptide_extraction_parser_attempts_total{strategy="wasm",path="direct"} 100
# riptide_extraction_parser_results_total{strategy="wasm",path="direct",outcome="success"} 95
# ...
```

### 2. Load Testing
```bash
# Generate load to verify metrics
for i in {1..100}; do
  curl -X POST http://localhost:3000/extract \
    -H "Content-Type: application/json" \
    -d '{"url": "https://example.com"}' &
done

# Check metrics after load
curl http://localhost:3000/metrics | grep riptide_extraction_parser_attempts_total
```

### 3. Grafana Dashboard Verification
1. Import the dashboard JSON into Grafana
2. Navigate to the Parser Performance dashboard
3. Verify all panels are displaying data
4. Test time range selection and refresh

## Performance Considerations

### Metric Cardinality

The parser metrics have controlled cardinality:
- `strategy`: 4 possible values (wasm, native, css, headless)
- `path`: 2 possible values (direct, headless)
- `outcome`: 3 possible values (success, fallback, error)

**Total unique time series**: ~48 per metric type (considering label combinations)

### Memory Usage

Estimated memory per metric:
- Counter: ~200 bytes per time series
- Histogram: ~1.5KB per time series (with 7 buckets)

**Total memory**: ~25KB for all parser metrics

### Performance Impact

- Counter increment: ~50ns
- Histogram observation: ~200ns
- Label lookup: ~30ns

**Total overhead per extraction**: <1µs (negligible)

## Troubleshooting

### Metrics Not Appearing

1. **Check feature flag**: Ensure `prometheus` feature is enabled in `riptide-monitoring`
   ```toml
   riptide-monitoring = { path = "../riptide-monitoring", features = ["prometheus"] }
   ```

2. **Verify registration**: Check logs for metric registration errors
   ```bash
   grep "Failed to register" logs/riptide.log
   ```

3. **Confirm endpoint**: Verify metrics endpoint is accessible
   ```bash
   curl http://localhost:3000/metrics
   ```

### Missing Data in Grafana

1. **Check data source**: Verify Prometheus is scraping the metrics endpoint
2. **Verify time range**: Ensure the selected time range includes data
3. **Check queries**: Use Prometheus UI to test queries directly
4. **Review labels**: Confirm label values match expected values

### High Cardinality Issues

If you experience high cardinality:
1. Limit the number of unique `strategy` values
2. Consider aggregating less common strategies
3. Use recording rules for frequently queried metrics

## References

- [Prometheus Best Practices](https://prometheus.io/docs/practices/naming/)
- [Grafana Dashboard Guide](https://grafana.com/docs/grafana/latest/dashboards/)
- [PromQL Query Guide](https://prometheus.io/docs/prometheus/latest/querying/basics/)
- [RipTide Monitoring Documentation](/workspaces/eventmesh/crates/riptide-monitoring/README.md)

## Changelog

### 2025-10-28
- Initial implementation of parser performance metrics
- Added 5 core metrics: attempts, results, fallbacks, duration, confidence
- Created integration guide and Grafana dashboard templates
- Added alerting rules for error rates, fallbacks, and performance
