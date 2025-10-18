//! Event handlers for the RipTide event system
//!
//! This module provides concrete implementations of event handlers that integrate
//! with existing monitoring, telemetry, and health check systems.

use super::*;
use crate::monitoring::collector::MetricsCollector;
use opentelemetry::trace::{Span, Status, Tracer};
use opentelemetry::{global, KeyValue};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// Logging event handler that writes events to structured logs
pub struct LoggingEventHandler {
    name: String,
    config: HandlerConfig,
}

impl LoggingEventHandler {
    pub fn new() -> Self {
        Self {
            name: "logging_handler".to_string(),
            config: HandlerConfig {
                enabled: true,
                event_types: vec!["*".to_string()],
                min_severity: EventSeverity::Debug,
                ..Default::default()
            },
        }
    }

    pub fn with_min_severity(mut self, severity: EventSeverity) -> Self {
        self.config.min_severity = severity;
        self
    }
}

#[async_trait]
impl EventHandler for LoggingEventHandler {
    fn name(&self) -> &str {
        &self.name
    }

    fn can_handle(&self, _event_type: &str) -> bool {
        self.config.enabled
    }

    async fn handle(&self, event: &dyn Event) -> Result<()> {
        if event.severity() < self.config.min_severity {
            return Ok(());
        }

        // Create structured log entry
        let json_data = event.to_json().unwrap_or_else(|_| "{}".to_string());

        match event.severity() {
            EventSeverity::Trace => {
                tracing::trace!(
                    event_id = %event.event_id(),
                    event_type = %event.event_type(),
                    source = %event.source(),
                    timestamp = %event.timestamp(),
                    data = %json_data,
                    "Event emitted"
                );
            }
            EventSeverity::Debug => {
                tracing::debug!(
                    event_id = %event.event_id(),
                    event_type = %event.event_type(),
                    source = %event.source(),
                    timestamp = %event.timestamp(),
                    data = %json_data,
                    "Event emitted"
                );
            }
            EventSeverity::Info => {
                tracing::info!(
                    event_id = %event.event_id(),
                    event_type = %event.event_type(),
                    source = %event.source(),
                    timestamp = %event.timestamp(),
                    data = %json_data,
                    "Event emitted"
                );
            }
            EventSeverity::Warn => {
                tracing::warn!(
                    event_id = %event.event_id(),
                    event_type = %event.event_type(),
                    source = %event.source(),
                    timestamp = %event.timestamp(),
                    data = %json_data,
                    "Event emitted"
                );
            }
            EventSeverity::Error => {
                tracing::error!(
                    event_id = %event.event_id(),
                    event_type = %event.event_type(),
                    source = %event.source(),
                    timestamp = %event.timestamp(),
                    data = %json_data,
                    "Event emitted"
                );
            }
            EventSeverity::Critical => {
                tracing::error!(
                    event_id = %event.event_id(),
                    event_type = %event.event_type(),
                    source = %event.source(),
                    timestamp = %event.timestamp(),
                    data = %json_data,
                    "CRITICAL EVENT emitted"
                );
            }
        }

        Ok(())
    }

    fn config(&self) -> HandlerConfig {
        self.config.clone()
    }
}

impl Default for LoggingEventHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Metrics event handler that integrates with the monitoring system
pub struct MetricsEventHandler {
    name: String,
    config: HandlerConfig,
    metrics_collector: Arc<MetricsCollector>,
}

impl MetricsEventHandler {
    pub fn new(metrics_collector: Arc<MetricsCollector>) -> Self {
        Self {
            name: "metrics_handler".to_string(),
            config: HandlerConfig {
                enabled: true,
                event_types: vec![
                    "pool.*".to_string(),
                    "extraction.*".to_string(),
                    "metrics.*".to_string(),
                    "health.*".to_string(),
                ],
                min_severity: EventSeverity::Info,
                ..Default::default()
            },
            metrics_collector,
        }
    }

    async fn handle_pool_event(&self, event: &dyn Event) -> Result<()> {
        // Extract pool-specific metrics from event
        if let Ok(json_str) = event.to_json() {
            if let Ok(pool_event) =
                serde_json::from_str::<crate::events::types::PoolEvent>(&json_str)
            {
                // Record pool operation metrics
                let _ = self
                    .metrics_collector
                    .record_pool_operation(
                        pool_event.operation.as_str(),
                        1.0,
                        event.timestamp().timestamp_millis() as u64,
                    )
                    .await;

                // If metrics are included, record them
                if let Some(metrics) = &pool_event.metrics {
                    let _ = self
                        .metrics_collector
                        .record_pool_state(
                            metrics.available_instances,
                            metrics.active_instances,
                            metrics.total_instances,
                        )
                        .await;
                }
            }
        }
        Ok(())
    }

    async fn handle_extraction_event(&self, event: &dyn Event) -> Result<()> {
        if let Ok(json_str) = event.to_json() {
            if let Ok(extraction_event) =
                serde_json::from_str::<crate::events::types::ExtractionEvent>(&json_str)
            {
                // Record extraction metrics
                let success = matches!(
                    extraction_event.operation,
                    crate::events::types::ExtractionOperation::Completed
                );

                if let Some(duration_ms) = extraction_event.duration_ms {
                    let _ = self
                        .metrics_collector
                        .record_extraction_time(Duration::from_millis(duration_ms), success)
                        .await;
                }

                // Record extraction outcome
                let _ = self
                    .metrics_collector
                    .record_extraction_outcome(
                        extraction_event.operation.as_str(),
                        &extraction_event.url,
                    )
                    .await;
            }
        }
        Ok(())
    }

    async fn handle_metrics_event(&self, event: &dyn Event) -> Result<()> {
        if let Ok(json_str) = event.to_json() {
            if let Ok(metrics_event) =
                serde_json::from_str::<crate::events::types::MetricsEvent>(&json_str)
            {
                // Forward custom metrics to collector
                let _ = self
                    .metrics_collector
                    .record_custom_metric(
                        &metrics_event.metric_name,
                        metrics_event.metric_value,
                        &metrics_event.tags,
                    )
                    .await;
            }
        }
        Ok(())
    }

    async fn handle_health_event(&self, event: &dyn Event) -> Result<()> {
        if let Ok(json_str) = event.to_json() {
            if let Ok(health_event) =
                serde_json::from_str::<crate::events::types::HealthEvent>(&json_str)
            {
                // Record component health status
                let health_score = match health_event.status {
                    crate::events::types::HealthStatus::Healthy => 1.0,
                    crate::events::types::HealthStatus::Degraded => 0.7,
                    crate::events::types::HealthStatus::Unhealthy => 0.3,
                    crate::events::types::HealthStatus::Critical => 0.0,
                };

                let _ = self
                    .metrics_collector
                    .record_health_status(&health_event.component, health_score)
                    .await;

                // Record any additional health metrics
                if let Some(metrics) = &health_event.metrics {
                    for (metric_name, value) in metrics {
                        let full_metric_name =
                            format!("health.{}.{}", health_event.component, metric_name);
                        let _ = self
                            .metrics_collector
                            .record_custom_metric(&full_metric_name, *value, &HashMap::new())
                            .await;
                    }
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl EventHandler for MetricsEventHandler {
    fn name(&self) -> &str {
        &self.name
    }

    fn can_handle(&self, event_type: &str) -> bool {
        if !self.config.enabled {
            return false;
        }

        self.config.event_types.iter().any(|pattern| {
            pattern == "*"
                || pattern == event_type
                || (pattern.ends_with('*') && event_type.starts_with(&pattern[..pattern.len() - 1]))
        })
    }

    async fn handle(&self, event: &dyn Event) -> Result<()> {
        if event.severity() < self.config.min_severity {
            return Ok(());
        }

        let event_type = event.event_type();

        if event_type.starts_with("pool.") {
            self.handle_pool_event(event).await?;
        } else if event_type.starts_with("extraction.") {
            self.handle_extraction_event(event).await?;
        } else if event_type.starts_with("metrics.") {
            self.handle_metrics_event(event).await?;
        } else if event_type.starts_with("health.") {
            self.handle_health_event(event).await?;
        }

        Ok(())
    }

    fn config(&self) -> HandlerConfig {
        self.config.clone()
    }
}

/// Telemetry event handler that integrates with OpenTelemetry
pub struct TelemetryEventHandler {
    name: String,
    config: HandlerConfig,
}

impl TelemetryEventHandler {
    pub fn new() -> Self {
        Self {
            name: "telemetry_handler".to_string(),
            config: HandlerConfig {
                enabled: true,
                event_types: vec!["*".to_string()],
                min_severity: EventSeverity::Info,
                ..Default::default()
            },
        }
    }

    fn create_span_from_event(&self, event: &dyn Event) {
        let tracer = global::tracer("riptide-events");

        let mut span = tracer.start(event.event_type());

        // Add standard attributes
        span.set_attribute(KeyValue::new("event.id", event.event_id().to_string()));
        span.set_attribute(KeyValue::new("event.type", event.event_type().to_string()));
        span.set_attribute(KeyValue::new("event.source", event.source().to_string()));
        span.set_attribute(KeyValue::new(
            "event.severity",
            event.severity().to_string(),
        ));
        span.set_attribute(KeyValue::new(
            "event.timestamp",
            event.timestamp().to_rfc3339(),
        ));

        // Add metadata as attributes
        for (key, value) in event.metadata() {
            span.set_attribute(KeyValue::new(
                format!("event.metadata.{}", key),
                value.clone(),
            ));
        }

        // Set span status based on event severity
        match event.severity() {
            EventSeverity::Error | EventSeverity::Critical => {
                span.set_status(Status::Error {
                    description: format!("{} event: {}", event.severity(), event.event_type())
                        .into(),
                });
            }
            _ => {
                span.set_status(Status::Ok);
            }
        }

        // Span is automatically ended when it goes out of scope
        span.end();
    }
}

#[async_trait]
impl EventHandler for TelemetryEventHandler {
    fn name(&self) -> &str {
        &self.name
    }

    fn can_handle(&self, _event_type: &str) -> bool {
        self.config.enabled
    }

    async fn handle(&self, event: &dyn Event) -> Result<()> {
        if !event.should_trace() || event.severity() < self.config.min_severity {
            return Ok(());
        }

        // Create and immediately end the span
        self.create_span_from_event(event);

        debug!(
            event_id = %event.event_id(),
            event_type = %event.event_type(),
            "Created telemetry span for event"
        );

        Ok(())
    }

    fn config(&self) -> HandlerConfig {
        self.config.clone()
    }
}

impl Default for TelemetryEventHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Health event handler that maintains system health state
pub struct HealthEventHandler {
    name: String,
    config: HandlerConfig,
    health_state: Arc<Mutex<HashMap<String, ComponentHealth>>>,
    recent_events: Arc<Mutex<VecDeque<Arc<dyn Event>>>>,
}

#[derive(Debug, Clone)]
pub struct ComponentHealth {
    pub status: crate::events::types::HealthStatus,
    pub last_updated: DateTime<Utc>,
    pub failure_count: u32,
    pub success_count: u32,
    pub details: Option<String>,
    pub metrics: HashMap<String, f64>,
}

impl ComponentHealth {
    pub fn new() -> Self {
        Self {
            status: crate::events::types::HealthStatus::Healthy,
            last_updated: Utc::now(),
            failure_count: 0,
            success_count: 0,
            details: None,
            metrics: HashMap::new(),
        }
    }

    pub fn is_healthy(&self) -> bool {
        matches!(self.status, crate::events::types::HealthStatus::Healthy)
    }

    pub fn health_score(&self) -> f64 {
        let total = self.success_count + self.failure_count;
        if total == 0 {
            return 1.0;
        }

        let base_score = self.success_count as f64 / total as f64;

        // Apply status modifier
        let status_modifier = match self.status {
            crate::events::types::HealthStatus::Healthy => 1.0,
            crate::events::types::HealthStatus::Degraded => 0.8,
            crate::events::types::HealthStatus::Unhealthy => 0.5,
            crate::events::types::HealthStatus::Critical => 0.2,
        };

        base_score * status_modifier
    }
}

impl Default for ComponentHealth {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthEventHandler {
    pub fn new() -> Self {
        Self {
            name: "health_handler".to_string(),
            config: HandlerConfig {
                enabled: true,
                event_types: vec![
                    "health.*".to_string(),
                    "pool.*".to_string(),
                    "extraction.*".to_string(),
                ],
                min_severity: EventSeverity::Info,
                ..Default::default()
            },
            health_state: Arc::new(Mutex::new(HashMap::new())),
            recent_events: Arc::new(Mutex::new(VecDeque::with_capacity(1000))),
        }
    }

    /// Get the current health status of a component
    pub fn get_component_health(&self, component: &str) -> Option<ComponentHealth> {
        self.health_state.lock().unwrap().get(component).cloned()
    }

    /// Get overall system health
    pub fn get_system_health(&self) -> crate::events::types::HealthStatus {
        let health_state = self.health_state.lock().unwrap();

        if health_state.is_empty() {
            return crate::events::types::HealthStatus::Healthy;
        }

        let mut critical_count = 0;
        let mut unhealthy_count = 0;
        let mut degraded_count = 0;
        let total_count = health_state.len();

        for health in health_state.values() {
            match health.status {
                crate::events::types::HealthStatus::Critical => critical_count += 1,
                crate::events::types::HealthStatus::Unhealthy => unhealthy_count += 1,
                crate::events::types::HealthStatus::Degraded => degraded_count += 1,
                crate::events::types::HealthStatus::Healthy => {}
            }
        }

        // If any component is critical, system is critical
        if critical_count > 0 {
            return crate::events::types::HealthStatus::Critical;
        }

        // If more than 50% unhealthy, system is unhealthy
        if unhealthy_count as f64 / total_count as f64 > 0.5 {
            return crate::events::types::HealthStatus::Unhealthy;
        }

        // If any component is unhealthy or many degraded, system is degraded
        if unhealthy_count > 0 || degraded_count as f64 / total_count as f64 > 0.3 {
            return crate::events::types::HealthStatus::Degraded;
        }

        crate::events::types::HealthStatus::Healthy
    }

    /// Get recent health events
    pub fn get_recent_events(&self, limit: usize) -> Vec<Arc<dyn Event>> {
        let recent_events = self.recent_events.lock().unwrap();
        recent_events.iter().rev().take(limit).cloned().collect()
    }

    fn update_component_health(&self, component: &str, event: &dyn Event) {
        let mut health_state = self.health_state.lock().unwrap();
        let health = health_state.entry(component.to_string()).or_default();

        health.last_updated = Utc::now();

        // Update based on event severity
        match event.severity() {
            EventSeverity::Error | EventSeverity::Critical => {
                health.failure_count += 1;
                if health.status == crate::events::types::HealthStatus::Healthy {
                    health.status = crate::events::types::HealthStatus::Degraded;
                }
            }
            _ => {
                health.success_count += 1;
                // Gradually improve health on successful events
                if health.failure_count > 0 && health.success_count > health.failure_count * 2 {
                    health.status = crate::events::types::HealthStatus::Healthy;
                }
            }
        }
    }
}

#[async_trait]
impl EventHandler for HealthEventHandler {
    fn name(&self) -> &str {
        &self.name
    }

    fn can_handle(&self, event_type: &str) -> bool {
        if !self.config.enabled {
            return false;
        }

        self.config.event_types.iter().any(|pattern| {
            pattern == "*"
                || pattern == event_type
                || (pattern.ends_with('*') && event_type.starts_with(&pattern[..pattern.len() - 1]))
        })
    }

    async fn handle(&self, event: &dyn Event) -> Result<()> {
        if event.severity() < self.config.min_severity {
            return Ok(());
        }

        // Store recent event
        {
            let mut recent_events = self.recent_events.lock().unwrap();
            recent_events.push_back(Arc::new(BaseEvent {
                event_id: event.event_id().to_string(),
                event_type: event.event_type().to_string(),
                timestamp: event.timestamp(),
                source: event.source().to_string(),
                severity: event.severity(),
                metadata: event.metadata().clone(),
                context: None,
            }));

            // Keep only the last 1000 events
            while recent_events.len() > 1000 {
                recent_events.pop_front();
            }
        }

        // Extract component name from event
        let component = if event.event_type().starts_with("health.") {
            // For explicit health events, try to extract component from JSON
            if let Ok(json_str) = event.to_json() {
                if let Ok(health_event) =
                    serde_json::from_str::<crate::events::types::HealthEvent>(&json_str)
                {
                    health_event.component
                } else {
                    event.source().to_string()
                }
            } else {
                event.source().to_string()
            }
        } else {
            // For other events, use source as component
            event.source().to_string()
        };

        self.update_component_health(&component, event);

        debug!(
            component = %component,
            event_type = %event.event_type(),
            severity = %event.severity(),
            "Updated component health from event"
        );

        Ok(())
    }

    fn config(&self) -> HandlerConfig {
        self.config.clone()
    }
}

impl Default for HealthEventHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_logging_handler() {
        let handler = LoggingEventHandler::new();
        let event = BaseEvent::new("test.event", "test_source", EventSeverity::Info);

        assert!(handler.can_handle("test.event"));
        assert!(handler.handle(&event).await.is_ok());
    }

    #[tokio::test]
    async fn test_health_handler_system_health() {
        let handler = HealthEventHandler::new();

        // Initially should be healthy
        assert_eq!(
            handler.get_system_health(),
            crate::events::types::HealthStatus::Healthy
        );

        // Add a healthy component
        let healthy_event = BaseEvent::new("test.event", "component1", EventSeverity::Info);
        handler.handle(&healthy_event).await.unwrap();

        // Add an unhealthy component
        let error_event = BaseEvent::new("test.error", "component2", EventSeverity::Error);
        handler.handle(&error_event).await.unwrap();

        // System should still be healthy since only one component has issues
        assert_eq!(
            handler.get_system_health(),
            crate::events::types::HealthStatus::Degraded
        );
    }

    #[tokio::test]
    async fn test_component_health_scoring() {
        let mut health = ComponentHealth::new();

        // Initially healthy
        assert_eq!(health.health_score(), 1.0);

        // Add some failures
        health.failure_count = 2;
        health.success_count = 8;

        // Score should reflect success rate with status modifier
        let expected_score = (8.0 / 10.0) * 1.0; // 80% success rate with healthy status
        assert!((health.health_score() - expected_score).abs() < 0.01);

        // Change status to degraded
        health.status = crate::events::types::HealthStatus::Degraded;
        let expected_degraded_score = (8.0 / 10.0) * 0.8; // 80% success rate with degraded status
        assert!((health.health_score() - expected_degraded_score).abs() < 0.01);
    }
}
