# Telemetry and Trace Backend Configuration

This document describes how to configure OpenTelemetry tracing and trace backend integration for the riptide-api service.

## Overview

The riptide-api service provides comprehensive distributed tracing support with:
- **Trace Export**: Send spans to OTLP-compatible backends (Jaeger, Tempo, etc.)
- **Trace Query**: Retrieve and visualize traces from backends
- **In-Memory Fallback**: Development mode with in-memory trace storage
- **Graceful Degradation**: Continues operating if trace backend is unavailable

## Architecture

```
┌─────────────────┐
│  riptide-api    │
│                 │
│  ┌───────────┐  │      OTLP Export (4317)
│  │ Telemetry │──┼──────────────────────────┐
│  │  System   │  │                          │
│  └───────────┘  │                          ▼
│                 │                  ┌───────────────┐
│  ┌───────────┐  │   Query API      │    Jaeger     │
│  │   Trace   │──┼──────────────────│               │
│  │  Backend  │  │  (16686)         │  - Storage    │
│  └───────────┘  │                  │  - Query API  │
└─────────────────┘                  │  - UI         │
                                     └───────────────┘
```

## Configuration

### Environment Variables

#### Trace Export Configuration

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `TELEMETRY_ENABLED` | Enable telemetry export | `false` | `true` |
| `TELEMETRY_SERVICE_NAME` | Service name for traces | `riptide-api` | `my-service` |
| `TELEMETRY_SERVICE_VERSION` | Service version | `0.1.0` | `1.2.3` |
| `TELEMETRY_EXPORTER_TYPE` | Exporter type | `otlp` | `otlp`, `jaeger`, `zipkin`, `none` |
| `TELEMETRY_OTLP_ENDPOINT` | OTLP gRPC endpoint | `http://localhost:4317` | `http://jaeger:4317` |
| `TELEMETRY_SAMPLING_RATIO` | Sampling ratio (0.0-1.0) | `1.0` | `0.1` (10%) |
| `TELEMETRY_EXPORT_TIMEOUT_SECS` | Export timeout | `30` | `60` |
| `TELEMETRY_MAX_QUEUE_SIZE` | Max span queue size | `2048` | `4096` |
| `TELEMETRY_MAX_EXPORT_BATCH_SIZE` | Max export batch | `512` | `1024` |
| `TELEMETRY_SCHEDULED_DELAY_MS` | Batch delay | `5000` | `10000` |
| `TELEMETRY_ENABLE_TRACE_PROPAGATION` | Enable W3C trace context | `true` | `false` |

#### Trace Backend Query Configuration

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `OTLP_TRACE_QUERY_ENDPOINT` | Query endpoint URL | *none* | `http://localhost:16686` |
| `OTLP_TRACE_BACKEND_TYPE` | Backend type | `jaeger` | `jaeger`, `tempo`, `generic` |

#### Custom Resource Attributes

Any environment variable prefixed with `TELEMETRY_RESOURCE_` will be added as a resource attribute:

```bash
export TELEMETRY_RESOURCE_ENVIRONMENT=production
export TELEMETRY_RESOURCE_REGION=us-east-1
export TELEMETRY_RESOURCE_CLUSTER=cluster-1
```

## Deployment Scenarios

### 1. Development (In-Memory Backend)

For local development without external dependencies:

```bash
# Disable telemetry export, use in-memory backend
export TELEMETRY_ENABLED=false

# No trace backend configuration needed - uses in-memory storage
```

**Features:**
- Mock trace data for testing
- No external dependencies
- Traces available via `/telemetry/traces` API

### 2. Jaeger All-in-One (Recommended for Production)

Deploy Jaeger for both trace storage and querying:

```bash
# Start Jaeger all-in-one
docker run -d --name jaeger \
  -e COLLECTOR_OTLP_ENABLED=true \
  -p 16686:16686 \
  -p 4317:4317 \
  -p 4318:4318 \
  jaegertracing/all-in-one:latest

# Configure riptide-api
export TELEMETRY_ENABLED=true
export TELEMETRY_SERVICE_NAME=riptide-api
export TELEMETRY_EXPORTER_TYPE=otlp
export TELEMETRY_OTLP_ENDPOINT=http://localhost:4317

# Configure trace backend query
export OTLP_TRACE_QUERY_ENDPOINT=http://localhost:16686
export OTLP_TRACE_BACKEND_TYPE=jaeger
```

**Access:**
- Jaeger UI: http://localhost:16686
- riptide-api traces: http://localhost:3000/telemetry/traces

### 3. Grafana Tempo

Use Tempo for scalable trace storage:

```bash
# Start Tempo (example with Docker)
docker run -d --name tempo \
  -p 3200:3200 \
  -p 4317:4317 \
  grafana/tempo:latest

# Configure riptide-api
export TELEMETRY_ENABLED=true
export TELEMETRY_OTLP_ENDPOINT=http://localhost:4317

# Configure trace backend query
export OTLP_TRACE_QUERY_ENDPOINT=http://localhost:3200
export OTLP_TRACE_BACKEND_TYPE=tempo
```

**Note:** Tempo query API support is planned for future implementation.

### 4. OpenTelemetry Collector + Backend

Use the OTEL Collector as a gateway:

```bash
# Configure collector endpoint
export TELEMETRY_ENABLED=true
export TELEMETRY_OTLP_ENDPOINT=http://otel-collector:4317

# Backend query depends on final storage
export OTLP_TRACE_QUERY_ENDPOINT=http://jaeger:16686
export OTLP_TRACE_BACKEND_TYPE=jaeger
```

### 5. Production with Sampling

Reduce overhead with sampling in high-traffic environments:

```bash
export TELEMETRY_ENABLED=true
export TELEMETRY_SAMPLING_RATIO=0.1  # Sample 10% of traces
export TELEMETRY_OTLP_ENDPOINT=http://jaeger:4317
export OTLP_TRACE_QUERY_ENDPOINT=http://jaeger:16686
export OTLP_TRACE_BACKEND_TYPE=jaeger
```

## API Endpoints

### List Recent Traces

```bash
GET /telemetry/traces?time_range_secs=300&limit=10&service=riptide-api
```

**Response:**
```json
[
  {
    "trace_id": "0af7651916cd43dd8448eb211c80319c",
    "root_span_id": "b7ad6b7169203331",
    "start_time": "2025-11-01T19:00:00Z",
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

### Get Trace Tree

```bash
GET /telemetry/traces/:trace_id?trace_id=0af7651916cd43dd8448eb211c80319c
```

**Response:**
```json
{
  "metadata": {
    "trace_id": "0af7651916cd43dd8448eb211c80319c",
    "root_span_id": "b7ad6b7169203331",
    "start_time": "2025-11-01T19:00:00Z",
    "duration_ms": 1234,
    "service_name": "riptide-api",
    "span_count": 4,
    "status": "OK"
  },
  "root_span": {
    "span_id": "b7ad6b7169203331",
    "name": "crawl_handler",
    "kind": "SERVER",
    "start_offset_ms": 0,
    "duration_ms": 1234,
    "status": "OK",
    "children": [...]
  },
  "summary": {
    "total_spans": 4,
    "error_count": 0,
    "avg_span_duration_ms": 308.5,
    "max_span_duration_ms": 700,
    "critical_path_duration_ms": 1234
  }
}
```

### Get Telemetry Status

```bash
GET /telemetry/status
```

**Response:**
```json
{
  "enabled": true,
  "service_name": "riptide-api",
  "exporter_type": "otlp",
  "otlp_endpoint": "http://localhost:4317",
  "sampling_ratio": 1.0,
  "trace_backend": {
    "type": "jaeger",
    "healthy": true,
    "configured": true
  }
}
```

## Troubleshooting

### Traces Not Appearing

1. **Check telemetry is enabled:**
   ```bash
   curl http://localhost:3000/telemetry/status | jq .enabled
   ```

2. **Verify OTLP endpoint is reachable:**
   ```bash
   curl http://localhost:4317  # Should connect to Jaeger/Tempo
   ```

3. **Check logs for export errors:**
   ```bash
   grep -i "telemetry\|otlp\|trace" logs/riptide-api.log
   ```

### Query Backend Unavailable

The service gracefully degrades if the trace backend is unavailable:

- Trace export continues (if configured)
- Query endpoints return empty results or mock data
- API remains operational

**Check backend health:**
```bash
curl http://localhost:3000/telemetry/status | jq .trace_backend.healthy
```

### High Memory Usage

Reduce queue size and batch size:

```bash
export TELEMETRY_MAX_QUEUE_SIZE=1024
export TELEMETRY_MAX_EXPORT_BATCH_SIZE=256
```

### Network Timeouts

Increase export timeout:

```bash
export TELEMETRY_EXPORT_TIMEOUT_SECS=60
```

## Performance Considerations

### Overhead

- **Sampling:** Use `TELEMETRY_SAMPLING_RATIO < 1.0` for high-traffic services
- **Batch Export:** Default 5s delay balances latency vs. overhead
- **Queue Size:** Larger queues prevent span loss but use more memory

### Best Practices

1. **Production Sampling:** Start with 10% sampling, adjust based on traffic
2. **Resource Attributes:** Add environment, region, cluster for better filtering
3. **Backend Sizing:** Size Jaeger/Tempo storage based on retention needs
4. **Monitoring:** Track export errors and backend health

## Integration with Existing Systems

### Jaeger UI

Access Jaeger UI for advanced trace analysis:
- URL: http://localhost:16686
- Search by service, operation, tags
- View dependency graphs
- Analyze latency distributions

### Grafana + Tempo

Use Grafana for trace visualization:
1. Add Tempo as data source in Grafana
2. Use TraceQL for advanced queries
3. Correlate traces with metrics and logs

### Prometheus Integration

Combine traces with metrics:
```bash
# riptide-api exposes Prometheus metrics at /metrics
# Correlate trace IDs with metric exemplars
```

## Security

### Network Security

- Use TLS for OTLP endpoint in production
- Restrict access to query endpoints
- Consider authentication for trace queries

### Data Retention

Configure backend retention policies:
```bash
# Jaeger: Set storage retention
--span-storage.type=cassandra
--cassandra.query.timeout=30s

# Tempo: Configure retention in config.yaml
retention:
  traces: 168h  # 7 days
```

## Future Enhancements

- [ ] Tempo query API implementation
- [ ] Zipkin exporter support
- [ ] Custom trace sampling strategies
- [ ] Trace analytics and aggregations
- [ ] Real-time trace streaming
- [ ] Trace-based alerting

## References

- [OpenTelemetry Specification](https://opentelemetry.io/docs/specs/otel/)
- [Jaeger Documentation](https://www.jaegertracing.io/docs/)
- [Grafana Tempo](https://grafana.com/docs/tempo/)
- [W3C Trace Context](https://www.w3.org/TR/trace-context/)
