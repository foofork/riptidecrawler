# Telemetry Enhancement Implementation Guide

## Overview

This document describes the comprehensive OpenTelemetry instrumentation implemented in the RipTide API, providing distributed tracing, observability, and performance monitoring capabilities.

## Implementation Summary

### Components Implemented

#### TELEM-001: Handler Instrumentation
All HTTP handler functions are now instrumented with OpenTelemetry spans:
- `/crawl` - Batch crawling endpoint with URL count, cache mode tracking
- `/deepsearch` - Deep search with query and results tracking
- `/stealth/configure` - Stealth configuration with preset tracking
- `/stealth/test` - Stealth testing with success rate metrics
- All handlers include HTTP method, route, and timing information

#### TELEM-002: Pipeline Phase Instrumentation
Pipeline phases are instrumented with detailed spans (see riptide-core implementation):
- **Fetch Phase**: URL fetching with cache hit tracking
- **Gate Phase**: Quality gate decisions with scores
- **Extract Phase**: Content extraction with word counts
- **Store Phase**: Cache storage operations

#### TELEM-003: Custom Span Attributes
Rich contextual attributes added to all spans:
- URL counts and individual URLs
- Cache keys and hit/miss status
- Quality scores and gate decisions
- Processing times per phase
- Success/failure counts
- Error information when failures occur

#### TELEM-004: Distributed Trace Correlation
Full trace context propagation:
- **Inbound**: Extract trace context from `traceparent` and `tracestate` HTTP headers
- **Outbound**: Inject trace context into outbound HTTP requests
- W3C Trace Context standard compliance
- Automatic parent-child span relationships

#### TELEM-005: Trace Visualization Endpoint
New telemetry visualization endpoints:
- `GET /telemetry/status` - Telemetry configuration and status
- `GET /telemetry/traces` - List recent traces with metadata
- `GET /telemetry/traces/:trace_id` - Get complete trace tree with timing

#### TELEM-006: OpenTelemetry Export Configuration
Flexible exporter configuration via environment variables:
- **OTLP** (OpenTelemetry Protocol) - Default, supports Jaeger, Prometheus, etc.
- **Jaeger** - Direct Jaeger protocol support
- **Zipkin** - Zipkin protocol support
- Configurable endpoints, sampling, batching

#### TELEM-007: TelemetryConfig in AppConfig
Comprehensive telemetry configuration:
- Service name and version tracking
- Sampling ratio configuration (0.0 to 1.0)
- Export timeout and batch size controls
- Custom resource attributes support
- Trace propagation enable/disable

## Configuration

### Environment Variables

```bash
# Enable telemetry export (default: false)
TELEMETRY_ENABLED=true

# Service identification
TELEMETRY_SERVICE_NAME=riptide-api
TELEMETRY_SERVICE_VERSION=1.0.0

# Exporter configuration
TELEMETRY_EXPORTER_TYPE=otlp  # otlp, jaeger, zipkin, none
TELEMETRY_OTLP_ENDPOINT=http://localhost:4317

# Sampling configuration
TELEMETRY_SAMPLING_RATIO=1.0  # 1.0 = 100%, 0.1 = 10%

# Performance tuning
TELEMETRY_EXPORT_TIMEOUT_SECS=30
TELEMETRY_MAX_QUEUE_SIZE=2048
TELEMETRY_MAX_EXPORT_BATCH_SIZE=512
TELEMETRY_SCHEDULED_DELAY_MS=5000

# Trace propagation
TELEMETRY_ENABLE_TRACE_PROPAGATION=true

# Custom resource attributes (optional)
TELEMETRY_RESOURCE_ENVIRONMENT=production
TELEMETRY_RESOURCE_DEPLOYMENT=kubernetes
TELEMETRY_RESOURCE_REGION=us-east-1
```

### Quick Start with Jaeger

1. **Start Jaeger locally:**
```bash
docker run -d --name jaeger \
  -e COLLECTOR_OTLP_ENABLED=true \
  -p 16686:16686 \
  -p 4317:4317 \
  -p 4318:4318 \
  jaegertracing/all-in-one:latest
```

2. **Configure RipTide API:**
```bash
export TELEMETRY_ENABLED=true
export TELEMETRY_OTLP_ENDPOINT=http://localhost:4317
export TELEMETRY_EXPORTER_TYPE=otlp
export TELEMETRY_SERVICE_NAME=riptide-api
```

3. **Start the API:**
```bash
cargo run --bin riptide-api
```

4. **View traces:**
Open http://localhost:16686 in your browser

### Quick Start with Zipkin

1. **Start Zipkin:**
```bash
docker run -d -p 9411:9411 openzipkin/zipkin
```

2. **Configure:**
```bash
export TELEMETRY_ENABLED=true
export TELEMETRY_EXPORTER_TYPE=zipkin
export TELEMETRY_OTLP_ENDPOINT=http://localhost:9411
```

## API Endpoints

### Telemetry Status

```bash
GET /telemetry/status
```

Returns telemetry configuration and feature availability:
```json
{
  "enabled": true,
  "service_name": "riptide-api",
  "service_version": "1.0.0",
  "exporter_type": "otlp",
  "otlp_endpoint": "http://localhost:4317",
  "sampling_ratio": 1.0,
  "trace_propagation_enabled": true,
  "features": {
    "distributed_tracing": true,
    "custom_attributes": true,
    "trace_visualization": true,
    "trace_export": true
  }
}
```

### List Traces

```bash
GET /telemetry/traces?time_range_secs=300&limit=10&service=riptide-api
```

Returns recent traces with metadata:
```json
[
  {
    "trace_id": "0af7651916cd43dd8448eb211c80319c",
    "root_span_id": "b7ad6b7169203331",
    "start_time": "2025-10-03T10:00:00Z",
    "duration_ms": 1234,
    "service_name": "riptide-api",
    "span_count": 15,
    "status": "OK",
    "attributes": {
      "http.method": "POST",
      "http.route": "/crawl"
    }
  }
]
```

### Get Trace Tree

```bash
GET /telemetry/traces/:trace_id
```

Returns complete trace tree with timing:
```json
{
  "metadata": {
    "trace_id": "0af7651916cd43dd8448eb211c80319c",
    "root_span_id": "b7ad6b7169203331",
    "duration_ms": 1234,
    "span_count": 15
  },
  "root_span": {
    "span_id": "b7ad6b7169203331",
    "name": "crawl_handler",
    "kind": "SERVER",
    "start_offset_ms": 0,
    "duration_ms": 1234,
    "attributes": {
      "http.method": "POST",
      "url_count": "5"
    },
    "children": [
      {
        "span_id": "00f067aa0ba902b7",
        "name": "pipeline.fetch",
        "kind": "INTERNAL",
        "duration_ms": 450
      }
    ]
  },
  "summary": {
    "total_spans": 15,
    "error_count": 0,
    "avg_span_duration_ms": 82.3,
    "critical_path_duration_ms": 1234
  }
}
```

## Span Attributes Reference

### Handler Spans

All handler spans include:
- `http.method` - HTTP method (GET, POST, etc.)
- `http.route` - Route pattern (e.g., "/crawl")
- `http.status_code` - Response status code
- `otel.kind` - Span kind (SERVER for handlers)
- `otel.status_code` - Span status (OK, ERROR)

### Crawl Handler Specific
- `url_count` - Number of URLs in request
- `cache_mode` - Cache strategy used
- `concurrency` - Concurrency limit
- `use_spider` - Whether spider mode was used
- `successful_count` - Successfully processed URLs
- `failed_count` - Failed URLs
- `cache_hits` - Number of cache hits
- `cache_hit_rate` - Cache hit percentage

### Pipeline Phase Spans
- `url` - Target URL
- `cache_key` - Cache key used
- `cache_hit` - Boolean cache hit status
- `decision` - Gate decision (raw, probes_first, headless)
- `quality_score` - Content quality score
- `word_count` - Extracted word count
- `extractor` - Extractor type used

### Error Tracking
When spans encounter errors:
- `otel.status_code` = "ERROR"
- `exception.type` - Error type
- `exception.message` - Error message
- `exception.stacktrace` - Stack trace (if available)

## Distributed Tracing Flow

### Request Flow

```
Client Request
    |
    +--> Extract traceparent header
    |
    +--> Create crawl_handler span
         |
         +--> Create pipeline.fetch span
         |    |
         |    +--> Inject trace context in outbound HTTP request
         |    +--> Record cache_hit, duration
         |
         +--> Create pipeline.gate span
         |    +--> Record decision, quality_score
         |
         +--> Create pipeline.extract span
         |    +--> Record word_count, extractor
         |
         +--> Create pipeline.store span
              +--> Record cache_key
    |
    +--> Return response with timing
```

### Example Trace Context Headers

**Inbound Request:**
```
traceparent: 00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01
tracestate: rojo=00f067aa0ba902b7
```

**Outbound Request:**
```
traceparent: 00-0af7651916cd43dd8448eb211c80319c-00f067aa0ba902b7-01
```

## Performance Impact

### Overhead Analysis

- **Span Creation**: ~1-2μs per span
- **Attribute Recording**: ~0.5μs per attribute
- **Context Propagation**: ~2-3μs per request
- **Batch Export**: Async, no blocking

### Sampling Strategies

Configure sampling to reduce overhead:

```bash
# 100% sampling (development)
TELEMETRY_SAMPLING_RATIO=1.0

# 10% sampling (production)
TELEMETRY_SAMPLING_RATIO=0.1

# 1% sampling (high-traffic)
TELEMETRY_SAMPLING_RATIO=0.01
```

## Integration Examples

### Curl with Trace Context

```bash
# Generate a trace ID
TRACE_ID=$(openssl rand -hex 16)
SPAN_ID=$(openssl rand -hex 8)

# Make request with trace context
curl -X POST http://localhost:8080/crawl \
  -H "Content-Type: application/json" \
  -H "traceparent: 00-${TRACE_ID}-${SPAN_ID}-01" \
  -d '{
    "urls": ["https://example.com"],
    "options": {"cache_mode": "default"}
  }'

# View trace in Jaeger
echo "Trace ID: $TRACE_ID"
echo "View at: http://localhost:16686/trace/$TRACE_ID"
```

### Python Client with Propagation

```python
from opentelemetry import trace
from opentelemetry.propagate import inject
import requests

tracer = trace.get_tracer(__name__)

with tracer.start_as_current_span("client_request"):
    headers = {}
    inject(headers)  # Inject trace context

    response = requests.post(
        "http://localhost:8080/crawl",
        json={"urls": ["https://example.com"]},
        headers=headers
    )
```

## Monitoring and Alerting

### Key Metrics to Track

1. **Span Duration** - Track P50, P95, P99
2. **Error Rate** - Percentage of ERROR spans
3. **Cache Hit Rate** - From span attributes
4. **Pipeline Phase Timing** - Identify bottlenecks
5. **Service Dependencies** - Via span relationships

### Example Prometheus Queries

```promql
# Average request duration by route
histogram_quantile(0.95,
  sum(rate(http_server_duration_bucket{route="/crawl"}[5m])) by (le)
)

# Error rate
sum(rate(span_status{status="ERROR"}[5m]))
  / sum(rate(span_status[5m]))

# Cache hit rate
sum(span_attribute{key="cache_hit",value="true"})
  / sum(span_attribute{key="cache_hit"})
```

## Troubleshooting

### Common Issues

**1. No traces appearing in Jaeger**
- Check `TELEMETRY_ENABLED=true`
- Verify `TELEMETRY_OTLP_ENDPOINT` is correct
- Check Jaeger is running: `docker ps | grep jaeger`
- Review logs for export errors

**2. High memory usage**
- Reduce sampling: `TELEMETRY_SAMPLING_RATIO=0.1`
- Decrease queue size: `TELEMETRY_MAX_QUEUE_SIZE=1024`
- Reduce batch size: `TELEMETRY_MAX_EXPORT_BATCH_SIZE=256`

**3. Trace context not propagating**
- Verify `TELEMETRY_ENABLE_TRACE_PROPAGATION=true`
- Check traceparent header format
- Ensure W3C Trace Context propagator is registered

### Debug Logging

Enable detailed telemetry logging:

```bash
RUST_LOG=riptide_api::telemetry_config=debug,opentelemetry=debug
```

## Best Practices

### 1. Span Naming
- Use descriptive, hierarchical names: `handler.crawl`, `pipeline.fetch`
- Include operation type: `db.query`, `http.request`
- Keep names consistent across services

### 2. Attribute Selection
- Add attributes that aid debugging
- Avoid PII (personally identifiable information)
- Use semantic conventions where possible
- Limit high-cardinality attributes

### 3. Sampling
- Use higher sampling in development (100%)
- Reduce in production based on traffic
- Consider adaptive sampling for errors
- Always sample errors and slow requests

### 4. Error Handling
- Always set span status on errors
- Include exception details
- Add context about failure conditions
- Don't lose error information

### 5. Performance
- Batch span export (avoid synchronous export)
- Use appropriate queue sizes
- Configure export timeouts
- Monitor exporter health

## Migration Guide

### Enabling Telemetry in Existing Deployments

**Step 1**: Start with disabled export
```bash
TELEMETRY_ENABLED=false  # Spans created but not exported
```

**Step 2**: Enable with low sampling
```bash
TELEMETRY_ENABLED=true
TELEMETRY_SAMPLING_RATIO=0.01  # 1% sampling
```

**Step 3**: Gradually increase sampling
```bash
TELEMETRY_SAMPLING_RATIO=0.1  # 10% sampling
```

**Step 4**: Monitor and adjust
- Watch memory usage
- Check export latency
- Verify trace quality

## Future Enhancements

Planned improvements:
- [ ] Metrics export (Prometheus)
- [ ] Log correlation with traces
- [ ] Automatic error sampling
- [ ] Custom span processors
- [ ] Trace-based testing
- [ ] Real-time anomaly detection
- [ ] Advanced sampling strategies
- [ ] Multi-backend export

## References

- [OpenTelemetry Specification](https://opentelemetry.io/docs/specs/otel/)
- [W3C Trace Context](https://www.w3.org/TR/trace-context/)
- [Jaeger Documentation](https://www.jaegertracing.io/docs/)
- [OpenTelemetry Rust SDK](https://github.com/open-telemetry/opentelemetry-rust)

## Support

For questions or issues:
- Check the troubleshooting section above
- Review logs with `RUST_LOG=debug`
- Verify configuration with `/telemetry/status` endpoint
- Examine traces in your chosen backend (Jaeger/Zipkin)
