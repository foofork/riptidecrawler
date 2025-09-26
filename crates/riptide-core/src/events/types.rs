//! Event type definitions for the RipTide event system
//!
//! This module defines specific event types that are emitted throughout the system.

use super::*;
use std::fmt;

/// Pool operation events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolEvent {
    base: BaseEvent,
    pub operation: PoolOperation,
    pub pool_id: String,
    pub instance_id: Option<String>,
    pub metrics: Option<PoolMetrics>,
}

impl PoolEvent {
    pub fn new(operation: PoolOperation, pool_id: String, source: &str) -> Self {
        let severity = match operation {
            PoolOperation::InstanceCreationFailed | PoolOperation::CircuitBreakerTripped => {
                EventSeverity::Error
            }
            PoolOperation::InstanceUnhealthy | PoolOperation::PoolExhausted => EventSeverity::Warn,
            PoolOperation::InstanceHealthy | PoolOperation::HealthCheck | PoolOperation::MemoryCleanup => EventSeverity::Debug,
            _ => EventSeverity::Info,
        };

        let event_type = format!("pool.{}", operation.as_str());
        let base = BaseEvent::new(&event_type, source, severity);

        Self {
            base,
            operation,
            pool_id,
            instance_id: None,
            metrics: None,
        }
    }

    pub fn with_instance_id(mut self, instance_id: String) -> Self {
        self.instance_id = Some(instance_id);
        self
    }

    pub fn with_metrics(mut self, metrics: PoolMetrics) -> Self {
        self.metrics = Some(metrics);
        self
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.base.add_metadata(key, value);
    }
}

impl Event for PoolEvent {
    fn event_type(&self) -> &'static str {
        "pool.operation"
    }

    fn event_id(&self) -> &str {
        &self.base.event_id
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.base.timestamp
    }

    fn source(&self) -> &str {
        &self.base.source
    }

    fn severity(&self) -> EventSeverity {
        self.base.severity
    }

    fn metadata(&self) -> &HashMap<String, String> {
        &self.base.metadata
    }

    fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(Into::into)
    }
}

/// Pool operations that can generate events
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PoolOperation {
    InstanceCreated,
    InstanceDestroyed,
    InstanceAcquired,
    InstanceReleased,
    InstanceCreationFailed,
    InstanceUnhealthy,
    InstanceHealthy,
    HealthCheck,
    MemoryCleanup,
    PoolWarmup,
    PoolExhausted,
    CircuitBreakerTripped,
    CircuitBreakerReset,
}

impl PoolOperation {
    pub fn as_str(&self) -> &'static str {
        match self {
            PoolOperation::InstanceCreated => "instance_created",
            PoolOperation::InstanceDestroyed => "instance_destroyed",
            PoolOperation::InstanceAcquired => "instance_acquired",
            PoolOperation::InstanceReleased => "instance_released",
            PoolOperation::InstanceCreationFailed => "instance_creation_failed",
            PoolOperation::InstanceUnhealthy => "instance_unhealthy",
            PoolOperation::InstanceHealthy => "instance_healthy",
            PoolOperation::HealthCheck => "health_check",
            PoolOperation::MemoryCleanup => "memory_cleanup",
            PoolOperation::PoolWarmup => "pool_warmup",
            PoolOperation::PoolExhausted => "pool_exhausted",
            PoolOperation::CircuitBreakerTripped => "circuit_breaker_tripped",
            PoolOperation::CircuitBreakerReset => "circuit_breaker_reset",
        }
    }
}

/// Pool metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolMetrics {
    pub available_instances: usize,
    pub active_instances: usize,
    pub total_instances: usize,
    pub pending_acquisitions: usize,
    pub success_rate: f64,
    pub avg_acquisition_time_ms: u64,
    pub avg_latency_ms: u64,
}

/// Extraction operation events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionEvent {
    base: BaseEvent,
    pub operation: ExtractionOperation,
    pub url: String,
    pub extraction_mode: String,
    pub duration_ms: Option<u64>,
    pub content_length: Option<usize>,
    pub error_details: Option<String>,
}

impl ExtractionEvent {
    pub fn new(
        operation: ExtractionOperation,
        url: String,
        extraction_mode: String,
        source: &str,
    ) -> Self {
        let severity = match operation {
            ExtractionOperation::Failed | ExtractionOperation::Timeout => EventSeverity::Error,
            ExtractionOperation::FallbackUsed => EventSeverity::Warn,
            _ => EventSeverity::Info,
        };

        let event_type = format!("extraction.{}", operation.as_str());
        let base = BaseEvent::new(&event_type, source, severity);

        Self {
            base,
            operation,
            url,
            extraction_mode,
            duration_ms: None,
            content_length: None,
            error_details: None,
        }
    }

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration_ms = Some(duration.as_millis() as u64);
        self
    }

    pub fn with_content_length(mut self, length: usize) -> Self {
        self.content_length = Some(length);
        self
    }

    pub fn with_error(mut self, error: String) -> Self {
        self.error_details = Some(error);
        self
    }
}

impl Event for ExtractionEvent {
    fn event_type(&self) -> &'static str {
        "extraction.operation"
    }

    fn event_id(&self) -> &str {
        &self.base.event_id
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.base.timestamp
    }

    fn source(&self) -> &str {
        &self.base.source
    }

    fn severity(&self) -> EventSeverity {
        self.base.severity
    }

    fn metadata(&self) -> &HashMap<String, String> {
        &self.base.metadata
    }

    fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(Into::into)
    }
}

/// Extraction operations
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ExtractionOperation {
    Started,
    Completed,
    Failed,
    Timeout,
    FallbackUsed,
}

impl ExtractionOperation {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExtractionOperation::Started => "started",
            ExtractionOperation::Completed => "completed",
            ExtractionOperation::Failed => "failed",
            ExtractionOperation::Timeout => "timeout",
            ExtractionOperation::FallbackUsed => "fallback_used",
        }
    }
}

/// Health check events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthEvent {
    base: BaseEvent,
    pub component: String,
    pub status: HealthStatus,
    pub details: Option<String>,
    pub metrics: Option<HashMap<String, f64>>,
}

impl HealthEvent {
    pub fn new(component: String, status: HealthStatus, source: &str) -> Self {
        let severity = match status {
            HealthStatus::Healthy => EventSeverity::Info,
            HealthStatus::Degraded => EventSeverity::Warn,
            HealthStatus::Unhealthy => EventSeverity::Error,
            HealthStatus::Critical => EventSeverity::Critical,
        };

        let event_type = format!("health.{}", status.as_str());
        let base = BaseEvent::new(&event_type, source, severity);

        Self {
            base,
            component,
            status,
            details: None,
            metrics: None,
        }
    }

    pub fn with_details(mut self, details: String) -> Self {
        self.details = Some(details);
        self
    }

    pub fn with_metrics(mut self, metrics: HashMap<String, f64>) -> Self {
        self.metrics = Some(metrics);
        self
    }
}

impl Event for HealthEvent {
    fn event_type(&self) -> &'static str {
        "health.check"
    }

    fn event_id(&self) -> &str {
        &self.base.event_id
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.base.timestamp
    }

    fn source(&self) -> &str {
        &self.base.source
    }

    fn severity(&self) -> EventSeverity {
        self.base.severity
    }

    fn metadata(&self) -> &HashMap<String, String> {
        &self.base.metadata
    }

    fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(Into::into)
    }
}

/// Health status levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Critical,
}

impl HealthStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            HealthStatus::Healthy => "healthy",
            HealthStatus::Degraded => "degraded",
            HealthStatus::Unhealthy => "unhealthy",
            HealthStatus::Critical => "critical",
        }
    }
}

impl fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Performance metrics event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsEvent {
    base: BaseEvent,
    pub metric_name: String,
    pub metric_value: f64,
    pub metric_type: MetricType,
    pub tags: HashMap<String, String>,
}

impl MetricsEvent {
    pub fn new(
        metric_name: String,
        metric_value: f64,
        metric_type: MetricType,
        source: &str,
    ) -> Self {
        let event_type = format!("metrics.{}", metric_type.as_str());
        let base = BaseEvent::new(&event_type, source, EventSeverity::Info);

        Self {
            base,
            metric_name,
            metric_value,
            metric_type,
            tags: HashMap::new(),
        }
    }

    pub fn with_tags(mut self, tags: HashMap<String, String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn add_tag(&mut self, key: &str, value: &str) {
        self.tags.insert(key.to_string(), value.to_string());
    }
}

impl Event for MetricsEvent {
    fn event_type(&self) -> &'static str {
        "metrics.update"
    }

    fn event_id(&self) -> &str {
        &self.base.event_id
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.base.timestamp
    }

    fn source(&self) -> &str {
        &self.base.source
    }

    fn severity(&self) -> EventSeverity {
        self.base.severity
    }

    fn metadata(&self) -> &HashMap<String, String> {
        &self.base.metadata
    }

    fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(Into::into)
    }
}

/// Metric types
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
}

impl MetricType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MetricType::Counter => "counter",
            MetricType::Gauge => "gauge",
            MetricType::Histogram => "histogram",
            MetricType::Summary => "summary",
        }
    }
}

/// Generic system event for custom use cases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEvent {
    base: BaseEvent,
    pub category: String,
    pub data: serde_json::Value,
}

impl SystemEvent {
    pub fn new(category: String, data: serde_json::Value, severity: EventSeverity, source: &str) -> Self {
        let event_type = format!("system.{}", category);
        let base = BaseEvent::new(&event_type, source, severity);

        Self {
            base,
            category,
            data,
        }
    }
}

impl Event for SystemEvent {
    fn event_type(&self) -> &'static str {
        "system.event"
    }

    fn event_id(&self) -> &str {
        &self.base.event_id
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.base.timestamp
    }

    fn source(&self) -> &str {
        &self.base.source
    }

    fn severity(&self) -> EventSeverity {
        self.base.severity
    }

    fn metadata(&self) -> &HashMap<String, String> {
        &self.base.metadata
    }

    fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(Into::into)
    }
}

// Implement Event for BaseEvent as a fallback
impl Event for BaseEvent {
    fn event_type(&self) -> &'static str {
        // Note: This is a compromise - we'd need to use a different approach
        // for truly dynamic event types. For now, we'll use a placeholder.
        "base_event"
    }

    fn event_id(&self) -> &str {
        &self.event_id
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn source(&self) -> &str {
        &self.source
    }

    fn severity(&self) -> EventSeverity {
        self.severity
    }

    fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_event_creation() {
        let event = PoolEvent::new(
            PoolOperation::InstanceCreated,
            "test-pool".to_string(),
            "test_source",
        );

        assert_eq!(event.operation, PoolOperation::InstanceCreated);
        assert_eq!(event.pool_id, "test-pool");
        assert_eq!(event.severity(), EventSeverity::Info);
    }

    #[test]
    fn test_extraction_event_with_duration() {
        let event = ExtractionEvent::new(
            ExtractionOperation::Completed,
            "https://example.com".to_string(),
            "article".to_string(),
            "extractor",
        )
        .with_duration(Duration::from_millis(150));

        assert_eq!(event.duration_ms, Some(150));
        assert_eq!(event.url, "https://example.com");
    }

    #[test]
    fn test_health_event_severity() {
        let healthy_event = HealthEvent::new(
            "test_component".to_string(),
            HealthStatus::Healthy,
            "health_checker",
        );
        assert_eq!(healthy_event.severity(), EventSeverity::Info);

        let critical_event = HealthEvent::new(
            "test_component".to_string(),
            HealthStatus::Critical,
            "health_checker",
        );
        assert_eq!(critical_event.severity(), EventSeverity::Critical);
    }

    #[test]
    fn test_metrics_event_with_tags() {
        let mut event = MetricsEvent::new(
            "response_time".to_string(),
            125.5,
            MetricType::Histogram,
            "metrics_collector",
        );

        event.add_tag("endpoint", "/api/v1/extract");
        event.add_tag("method", "POST");

        assert_eq!(event.tags.get("endpoint"), Some(&"/api/v1/extract".to_string()));
        assert_eq!(event.tags.get("method"), Some(&"POST".to_string()));
    }

    #[test]
    fn test_system_event_serialization() {
        let data = serde_json::json!({
            "component": "test",
            "action": "startup",
            "config": {
                "threads": 4,
                "memory_limit": "1GB"
            }
        });

        let event = SystemEvent::new(
            "startup".to_string(),
            data,
            EventSeverity::Info,
            "system_manager",
        );

        let json = event.to_json().unwrap();
        assert!(json.contains("startup"));
        assert!(json.contains("system_manager"));
    }
}