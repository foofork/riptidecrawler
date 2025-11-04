# Trace Backend Configuration

## Overview

The OTLP trace backend integration provides distributed tracing capabilities for the RipTide API. This document describes how to configure and use the trace storage and retrieval system.

## Architecture

The trace backend system consists of:

1. **TraceBackend Trait**: Abstract interface for trace storage/retrieval
2. **InMemoryTraceBackend**: Development backend with mock data
3. **OtlpTraceBackend**: Production backend for OTLP-compatible systems
4. **Telemetry Handlers**: HTTP endpoints for trace visualization

## Configuration

### Environment Variables

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `OTLP_TRACE_QUERY_ENDPOINT` | OTLP backend query endpoint | None | No (uses in-memory if not set) |
| `TELEMETRY_ENABLED` | Enable telemetry export | `false` | No |
| `TELEMETRY_SERVICE_NAME` | Service name for traces | `riptide-api` | No |
| `TELEMETRY_OTLP_ENDPOINT` | OTLP export endpoint | `http://localhost:4317` | No |
| `TELEMETRY_EXPORTER_TYPE` | Exporter type (otlp/jaeger/zipkin) | `otlp` | No |
| `TELEMETRY_SAMPLING_RATIO` | Trace sampling ratio (0.0-1.0) | `1.0` | No |

### Backend Selection

The system automatically selects the appropriate backend:

- **OTLP Backend**: Used when `OTLP_TRACE_QUERY_ENDPOINT` is set
- **In-Memory Backend**: Used for development (includes mock data)

## Usage

### Listing Recent Traces

```bash
# List all recent traces (last 5 minutes, limit 10)
curl http://localhost:3000/telemetry/traces

# Filter by service name
curl "http://localhost:3000/telemetry/traces?service=riptide-api"

# Custom time range and limit
curl "http://localhost:3000/telemetry/traces?time_range_secs=600&limit=20"
```

Response:
```json
[
  {
    "trace_id": "0af7651916cd43dd8448eb211c80319c",
    "root_span_id": "b7ad6b7169203331",
    "start_time": "2025-10-03T10:00:00Z",
    "duration_ms": 1234,
    "service_name": "riptide-api",
    "span_count": 4,
    "status": "OK",
    "attributes": {
      "http.method": "POST",
      "http.route": "/crawl"
    }
  }
]
```

### Getting Trace Tree

```bash
# Get complete trace tree with all spans
curl "http://localhost:3000/telemetry/traces/tree?trace_id=0af7651916cd43dd8448eb211c80319c"
```

Response:
```json
{
  "metadata": {
    "trace_id": "0af7651916cd43dd8448eb211c80319c",
    "root_span_id": "b7ad6b7169203331",
    "start_time": "2025-10-03T10:00:00Z",
    "duration_ms": 1234,
    "service_name": "riptide-api",
    "span_count": 4,
    "status": "OK",
    "attributes": {}
  },
  "root_span": {
    "span_id": "b7ad6b7169203331",
    "name": "crawl_handler",
    "parent_span_id": null,
    "kind": "SERVER",
    "start_offset_ms": 0,
    "duration_ms": 1234,
    "status": "OK",
    "attributes": {},
    "events": [],
    "children": [
      {
        "span_id": "00f067aa0ba902b7",
        "name": "pipeline.fetch",
        "parent_span_id": "b7ad6b7169203331",
        "kind": "INTERNAL",
        "start_offset_ms": 20,
        "duration_ms": 450,
        "status": "OK",
        "attributes": {},
        "events": [],
        "children": []
      }
    ]
  },
  "summary": {
    "total_spans": 4,
    "error_count": 0,
    "avg_span_duration_ms": 391.25,
    "max_span_duration_ms": 700,
    "critical_path_duration_ms": 1234,
    "services": ["riptide-api"]
  }
}
```

### Checking Telemetry Status

```bash
curl http://localhost:3000/telemetry/status
```

Response:
```json
{
  "enabled": true,
  "service_name": "riptide-api",
  "service_version": "0.9.0",
  "exporter_type": "otlp",
  "otlp_endpoint": "http://localhost:4317",
  "sampling_ratio": 1.0,
  "trace_propagation_enabled": true,
  "trace_backend": {
    "type": "in-memory",
    "healthy": true,
    "configured": true
  },
  "features": {
    "distributed_tracing": true,
    "custom_attributes": true,
    "trace_visualization": true,
    "trace_export": true,
    "trace_storage": true
  }
}
```

## Production Deployment

### With Jaeger

```bash
# Start Jaeger all-in-one
docker run -d --name jaeger \
  -e COLLECTOR_OTLP_ENABLED=true \
  -p 16686:16686 \
  -p 4317:4317 \
  -p 4318:4318 \
  jaegertracing/all-in-one:latest

# Configure RipTide API
export TELEMETRY_ENABLED=true
export TELEMETRY_OTLP_ENDPOINT=http://localhost:4317
export OTLP_TRACE_QUERY_ENDPOINT=http://localhost:16686

# Start API
cargo run --bin riptide-api
```

### With Grafana Tempo

```bash
# Start Tempo
docker run -d --name tempo \
  -p 3200:3200 \
  -p 4317:4317 \
  grafana/tempo:latest

# Configure RipTide API
export TELEMETRY_ENABLED=true
export TELEMETRY_OTLP_ENDPOINT=http://localhost:4317
export OTLP_TRACE_QUERY_ENDPOINT=http://localhost:3200

# Start API
cargo run --bin riptide-api
```

## Development

### In-Memory Backend

For development and testing, the in-memory backend is automatically used:

```bash
# No OTLP_TRACE_QUERY_ENDPOINT set = in-memory backend
cargo run --bin riptide-api

# The backend is pre-populated with mock trace data
curl http://localhost:3000/telemetry/traces
```

### Adding Mock Data

The in-memory backend automatically populates mock data on startup. To customize:

```rust
use crate::handlers::trace_backend::InMemoryTraceBackend;

let backend = InMemoryTraceBackend::new();
backend.populate_mock_data().await; // Adds sample traces
```

## Extending the Backend

### Implementing Custom Backend

```rust
use crate::handlers::trace_backend::{TraceBackend, CompleteTrace, TraceMetadata};
use opentelemetry::trace::TraceId;

struct CustomBackend {
    // Your backend fields
}

impl TraceBackend for CustomBackend {
    fn list_traces(
        &self,
        time_range_secs: u64,
        limit: usize,
        service_filter: Option<String>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<TraceMetadata>, ApiError>> + Send + '_>> {
        Box::pin(async move {
            // Your implementation
            Ok(vec![])
        })
    }

    fn get_trace(
        &self,
        trace_id: &TraceId,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<CompleteTrace>, ApiError>> + Send + '_>> {
        Box::pin(async move {
            // Your implementation
            Ok(None)
        })
    }

    fn health_check(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send + '_>> {
        Box::pin(async move { true })
    }

    fn backend_type(&self) -> &str {
        "custom"
    }
}
```

## Troubleshooting

### No traces appearing

1. Check telemetry is enabled: `curl http://localhost:3000/telemetry/status`
2. Verify backend type in status response
3. Check backend health: Look for `"healthy": true` in status
4. Review logs for trace export errors

### Backend connection issues

```bash
# Check OTLP endpoint is reachable
curl http://localhost:4317

# Verify environment variables
echo $OTLP_TRACE_QUERY_ENDPOINT
echo $TELEMETRY_ENABLED
```

### Trace ID format errors

Trace IDs must be 32-character hexadecimal strings:
- ✅ Valid: `0af7651916cd43dd8448eb211c80319c`
- ❌ Invalid: `invalid`, `123`, `0af765`

## Performance Considerations

### Sampling

For high-traffic production environments, use sampling to reduce overhead:

```bash
export TELEMETRY_SAMPLING_RATIO=0.1  # Sample 10% of requests
```

### Backend Storage

The in-memory backend is limited to development use. For production:
- Use Jaeger for long-term trace storage
- Use Tempo for cost-effective storage with S3/GCS
- Implement retention policies in your backend

## Future Enhancements

### Planned Features

1. **Full OTLP Query Implementation**: Direct integration with Jaeger/Tempo APIs
2. **Trace Filtering**: Advanced filtering by tags, duration, status
3. **Trace Analytics**: Aggregate statistics and trends
4. **Custom Exporters**: Support for additional trace backends
5. **Real-time Streaming**: WebSocket-based trace streaming

### Contributing

To extend the trace backend:

1. Implement the `TraceBackend` trait
2. Add configuration parsing in `AppState::new_with_telemetry_and_api_config`
3. Update documentation with new backend type
4. Add integration tests

## References

- [OpenTelemetry Specification](https://opentelemetry.io/docs/specs/otlp/)
- [Jaeger Documentation](https://www.jaegertracing.io/docs/)
- [Grafana Tempo Documentation](https://grafana.com/docs/tempo/latest/)
- [OTLP Specification](https://opentelemetry.io/docs/specs/otlp/)
