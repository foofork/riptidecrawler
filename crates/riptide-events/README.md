# RipTide Events

Event system and telemetry integration for RipTide, providing distributed tracing, event emission, and OpenTelemetry integration.

## Overview

`riptide-events` implements a comprehensive event-driven architecture for RipTide, enabling real-time event emission, distributed tracing with OpenTelemetry, and structured event handling for monitoring and debugging.

## Features

- **Event Bus**: High-performance pub/sub event system
- **OpenTelemetry Integration**: Distributed tracing and metrics
- **Event Filtering**: Type-based and pattern-based filtering
- **Async Event Handlers**: Non-blocking event processing
- **Event Replay**: Debug and audit capabilities
- **Structured Events**: Strongly-typed event definitions
- **Trace Propagation**: Automatic context propagation
- **Custom Collectors**: Extensible collector system

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│                  RipTide Events                          │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │   Event     │  │  Telemetry  │  │   Trace     │     │
│  │    Bus      │  │  Collector  │  │   Context   │     │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘     │
│         │                │                 │            │
│         └────────────────┼─────────────────┘            │
│                          ▼                              │
│                  ┌───────────────┐                      │
│                  │ OpenTelemetry │                      │
│                  │   (OTLP)      │                      │
│                  └───────┬───────┘                      │
└────────────────────────────┼─────────────────────────────┘
                            │
                            ▼
                    ┌──────────────┐
                    │   Collector  │
                    │ (Jaeger/etc) │
                    └──────────────┘
```

## Usage

### Event Emission

```rust
use riptide_events::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize event bus
    let bus = EventBus::new();

    // Emit events
    bus.emit(Event::PageLoaded {
        url: "https://example.com".to_string(),
        duration_ms: 250,
        success: true,
    }).await;

    bus.emit(Event::ExtractionCompleted {
        url: "https://example.com".to_string(),
        strategy: "css".to_string(),
        content_size: 15240,
    }).await;

    Ok(())
}
```

### Event Subscription

```rust
use riptide_events::*;

let bus = EventBus::new();

// Subscribe to all events
let rx = bus.subscribe().await;

tokio::spawn(async move {
    while let Ok(event) = rx.recv().await {
        match event {
            Event::PageLoaded { url, duration_ms, .. } => {
                println!("Page loaded: {} in {}ms", url, duration_ms);
            }
            Event::ExtractionCompleted { strategy, .. } => {
                println!("Extraction completed using: {}", strategy);
            }
            _ => {}
        }
    }
});

// Subscribe to specific event types
let extraction_rx = bus.subscribe_to::<ExtractionCompleted>().await;
```

### OpenTelemetry Tracing

```rust
use riptide_events::*;
use tracing::{info, instrument};

#[instrument(skip(bus))]
async fn extract_content(url: &str, bus: &EventBus) -> Result<String> {
    // Spans automatically created and tracked
    info!("Starting extraction for {}", url);

    // Emit event with trace context
    bus.emit_with_trace(Event::ExtractionStarted {
        url: url.to_string(),
    }).await;

    // ... extraction logic ...

    bus.emit_with_trace(Event::ExtractionCompleted {
        url: url.to_string(),
        strategy: "css".to_string(),
        content_size: 1024,
    }).await;

    Ok("content".to_string())
}
```

### Distributed Tracing Setup

```rust
use riptide_events::*;

// Initialize telemetry
let telemetry = TelemetryConfig {
    service_name: "riptide-api".to_string(),
    endpoint: "http://localhost:4317".to_string(),
    sample_rate: 1.0,
};

telemetry.init().await?;

// Traces automatically exported to collector
```

### Custom Event Types

```rust
use riptide_events::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomEvent {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub data: serde_json::Value,
}

// Emit custom event
bus.emit(Event::Custom(CustomEvent {
    id: uuid::Uuid::new_v4().to_string(),
    timestamp: chrono::Utc::now(),
    data: json!({
        "action": "custom_action",
        "details": "..."
    }),
})).await;
```

## Event Types

### Core Events

```rust
pub enum Event {
    // Page events
    PageLoaded {
        url: String,
        duration_ms: u64,
        success: bool,
    },
    PageNavigationFailed {
        url: String,
        error: String,
    },

    // Extraction events
    ExtractionStarted {
        url: String,
    },
    ExtractionCompleted {
        url: String,
        strategy: String,
        content_size: usize,
    },
    ExtractionFailed {
        url: String,
        error: String,
    },

    // Cache events
    CacheHit {
        key: String,
    },
    CacheMiss {
        key: String,
    },

    // Resource events
    ResourceAcquired {
        resource_type: String,
    },
    ResourceReleased {
        resource_type: String,
    },

    // System events
    HealthCheckCompleted {
        component: String,
        healthy: bool,
    },

    // Custom events
    Custom(serde_json::Value),
}
```

## Event Filtering

### Pattern-Based Filtering

```rust
use riptide_events::*;

// Filter by event type
let filter = EventFilter::type_eq("PageLoaded");

// Filter by field value
let filter = EventFilter::field_eq("url", "https://example.com");

// Combine filters
let filter = EventFilter::and(vec![
    EventFilter::type_eq("ExtractionCompleted"),
    EventFilter::field_contains("strategy", "css"),
]);

let rx = bus.subscribe_with_filter(filter).await;
```

### Time-Based Filtering

```rust
use riptide_events::*;

// Events within time range
let filter = EventFilter::time_range(
    start_time,
    end_time,
);

// Recent events only
let filter = EventFilter::since(Duration::from_secs(60));
```

## Telemetry Integration

### Metrics Collection

```rust
use riptide_events::*;

// Automatic metrics from events
let collector = MetricsCollector::new();

collector.record_event(&Event::PageLoaded {
    url: "https://example.com".to_string(),
    duration_ms: 250,
    success: true,
});

// Get aggregated metrics
let metrics = collector.get_metrics().await;
println!("Avg page load: {}ms", metrics.avg_page_load_ms);
println!("Total pages: {}", metrics.total_pages_loaded);
```

### Custom Spans

```rust
use tracing::{span, Level};

let span = span!(Level::INFO, "extraction", url = %url);
let _enter = span.enter();

// All events emitted here are part of this span
bus.emit(Event::ExtractionStarted { url }).await;
// ... work ...
bus.emit(Event::ExtractionCompleted { url, ... }).await;
```

## Event Replay

### Recording Events

```rust
use riptide_events::*;

// Enable event recording
let bus = EventBus::with_recording(true);

// All events are recorded
bus.emit(event1).await;
bus.emit(event2).await;
bus.emit(event3).await;

// Get recorded events
let events = bus.get_recorded_events().await;
```

### Replaying Events

```rust
use riptide_events::*;

// Load recorded events
let events = EventLog::load("events.json")?;

// Replay events
for event in events {
    bus.emit(event).await;
    tokio::time::sleep(Duration::from_millis(10)).await;
}
```

## Configuration

### Environment Variables

```bash
# Telemetry
export OTEL_ENDPOINT="http://localhost:4317"
export OTEL_SERVICE_NAME="riptide-api"
export OTEL_SAMPLE_RATE=1.0

# Event bus
export EVENT_BUS_BUFFER_SIZE=10000
export EVENT_RECORDING_ENABLED=false
export EVENT_LOG_PATH="/var/log/riptide-events.log"

# Logging
export RUST_LOG="info,riptide_events=debug"
```

### Programmatic Configuration

```rust
use riptide_events::*;

let config = EventBusConfig {
    buffer_size: 10000,
    enable_recording: false,
    telemetry: Some(TelemetryConfig {
        service_name: "riptide-api".to_string(),
        endpoint: "http://localhost:4317".to_string(),
        sample_rate: 1.0,
    }),
};

let bus = EventBus::with_config(config).await?;
```

## Integration with RipTide

This crate is used by:

- **riptide-api**: API request/response tracing
- **riptide-core**: Pipeline event emission
- **riptide-monitoring**: Metrics collection from events
- **riptide-workers**: Job lifecycle events

## Viewing Traces

### Jaeger

```bash
# Start Jaeger
docker run -d --name jaeger \
  -p 16686:16686 \
  -p 4317:4317 \
  jaegertracing/all-in-one:latest

# Access UI
open http://localhost:16686
```

### Zipkin

```bash
# Start Zipkin
docker run -d --name zipkin \
  -p 9411:9411 \
  openzipkin/zipkin

# Access UI
open http://localhost:9411
```

## Testing

```bash
# Run tests
cargo test -p riptide-events

# Run with telemetry (requires collector)
docker run -d -p 4317:4317 jaegertracing/all-in-one:latest
cargo test -p riptide-events --features telemetry

# Integration tests
cargo test -p riptide-events --test '*'
```

## License

Apache-2.0

## Related Crates

- **riptide-monitoring**: Metrics and monitoring
- **riptide-types**: Event type definitions
- **opentelemetry**: Telemetry primitives
