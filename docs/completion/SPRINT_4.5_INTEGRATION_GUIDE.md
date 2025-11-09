# Sprint 4.5 Integration Guide

## Missing Methods in BusinessMetrics

The following methods are referenced by existing facades but not yet implemented in BusinessMetrics:

### Session Tracking
```rust
// Add to BusinessMetrics
pub fn record_session_created(&self) {
    // Implementation needed
}

pub fn record_session_closed(&self) {
    // Implementation needed  
}

pub fn record_session_active(&self, count: usize) {
    // Implementation needed
}
```

### Browser Operations
```rust
pub fn record_browser_action(&self) {
    // Implementation needed
}

pub fn record_screenshot_taken(&self) {
    // Implementation needed
}
```

### Streaming Events
```rust
pub fn record_stream_started(&self) {
    // Implementation needed
}

pub fn record_stream_completed(&self) {
    // Implementation needed
}

pub fn record_stream_events_sent(&self, count: usize) {
    // Implementation needed
}
```

## Integration Steps

### Step 1: Complete BusinessMetrics Implementation

Add missing methods to `/workspaces/eventmesh/crates/riptide-facade/src/metrics/business.rs`:

```rust
impl BusinessMetrics {
    // Add session metrics
    pub sessions_created: Counter,
    pub sessions_active: Gauge,
    
    // Add browser metrics
    pub browser_actions_total: Counter,
    pub screenshots_taken: Counter,
    
    // Add streaming metrics
    pub streams_started: Counter,
    pub streams_completed: Counter,
    pub stream_events_sent: Counter,
    
    // Recording methods
    pub fn record_session_created(&self) {
        self.sessions_created.inc();
        self.sessions_active.inc();
    }
    
    pub fn record_session_closed(&self) {
        self.sessions_active.dec();
    }
    
    pub fn record_browser_action(&self) {
        self.browser_actions_total.inc();
    }
    
    pub fn record_screenshot_taken(&self) {
        self.screenshots_taken.inc();
    }
    
    pub fn record_stream_started(&self) {
        self.streams_started.inc();
    }
    
    pub fn record_stream_completed(&self) {
        self.streams_completed.inc();
    }
    
    pub fn record_stream_events_sent(&self, count: usize) {
        self.stream_events_sent.inc_by(count as f64);
    }
}
```

### Step 2: Update AppState

In `/workspaces/eventmesh/crates/riptide-api/src/state.rs`:

```rust
use riptide_facade::BusinessMetrics;
use crate::metrics_transport::TransportMetrics;

pub struct AppState {
    // Add both metrics
    pub business_metrics: Arc<BusinessMetrics>,
    pub transport_metrics: Arc<TransportMetrics>,
    
    // Existing fields...
}

impl AppState {
    pub fn new() -> anyhow::Result<Self> {
        let business_metrics = Arc::new(BusinessMetrics::new()?);
        let transport_metrics = Arc::new(TransportMetrics::new()?);
        
        // Create facades with business_metrics
        let pdf_facade = Arc::new(PdfFacade::new(
            config.clone(),
            business_metrics.clone(), // Changed from old RipTideMetrics
        ).await?);
        
        // ... etc
        
        Ok(Self {
            business_metrics,
            transport_metrics,
            // ... other fields
        })
    }
}
```

### Step 3: Update Prometheus Handler

In `/workspaces/eventmesh/crates/riptide-api/src/handlers/metrics.rs`:

```rust
use crate::metrics_transport::TransportMetrics;
use riptide_facade::BusinessMetrics;
use prometheus::Encoder;

pub async fn prometheus_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let encoder = prometheus::TextEncoder::new();
    
    // Gather metrics from both registries
    let mut metric_families = Vec::new();
    metric_families.extend(state.business_metrics.registry.gather());
    metric_families.extend(state.transport_metrics.registry.gather());
    
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();
    
    Response::builder()
        .header(CONTENT_TYPE, encoder.format_type())
        .body(Body::from(buffer))
        .unwrap()
}
```

### Step 4: Update Middleware

In HTTP middleware, use TransportMetrics:

```rust
// In request middleware
metrics.transport_metrics.record_http_request(
    method,
    path,
    status,
    duration,
);

// In streaming middleware
metrics.transport_metrics.record_streaming_message_sent();
```

### Step 5: Run Tests

```bash
# Test facade
cargo test -p riptide-facade

# Test API
cargo test -p riptide-api

# Full workspace test
cargo test --workspace

# Clippy
cargo clippy -p riptide-facade -- -D warnings
cargo clippy -p riptide-api -- -D warnings
```

## Quality Gate Checklist

Before merging:

- [ ] All facade methods compile
- [ ] BusinessMetrics has all required methods
- [ ] TransportMetrics used in all handlers
- [ ] Prometheus endpoint returns metrics from both registries
- [ ] All tests passing
- [ ] Zero clippy warnings
- [ ] Documentation updated

## Current Status

- ✅ BusinessMetrics structure created (581 LOC)
- ✅ TransportMetrics structure created (481 LOC)
- ✅ Prometheus dependencies added
- ⏳ Missing session/browser/streaming methods
- ⏳ AppState integration pending
- ⏳ Handler updates pending
- ⏳ Tests pending

## Estimated Time to Complete

- Missing methods: 30 minutes
- AppState update: 45 minutes
- Handler updates: 1 hour
- Testing & fixes: 1-2 hours

**Total:** 3-4 hours
