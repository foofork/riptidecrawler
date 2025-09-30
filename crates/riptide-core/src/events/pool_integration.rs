//! Integration layer for adding event emission to existing pool operations
//!
//! This module provides wrapper and extension traits that add event emission
//! capabilities to the existing AdvancedInstancePool without requiring
//! significant changes to the existing codebase.

use super::*;
use crate::events::types::*;
use crate::instance_pool::AdvancedInstancePool;
use crate::types::{ExtractedDoc, ExtractionMode};
use anyhow::Result;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{info, warn, error};

/// Extension trait for AdvancedInstancePool to add event emission capabilities
#[async_trait]
pub trait PoolEventEmitter {
    /// Emit pool events through the provided event bus
    async fn set_event_bus(&self, event_bus: Arc<EventBus>);

    /// Extract with event emission
    async fn extract_with_events(
        &self,
        html: &str,
        url: &str,
        mode: ExtractionMode,
        event_bus: &EventBus,
    ) -> Result<ExtractedDoc>;

    /// Get pool metrics as an event-friendly structure
    async fn get_pool_metrics_for_events(&self) -> PoolMetrics;
}

/// Event-aware wrapper for AdvancedInstancePool
pub struct EventAwareInstancePool {
    pool: Arc<AdvancedInstancePool>,
    event_bus: Option<Arc<EventBus>>,
    pool_id: String,
}

impl EventAwareInstancePool {
    /// Create a new event-aware pool wrapper
    pub fn new(pool: Arc<AdvancedInstancePool>) -> Self {
        Self {
            pool_id: uuid::Uuid::new_v4().to_string(),
            pool,
            event_bus: None,
        }
    }

    /// Set the event bus for this pool
    pub fn with_event_bus(mut self, event_bus: Arc<EventBus>) -> Self {
        self.event_bus = Some(event_bus);
        self
    }

    /// Get the pool ID
    pub fn pool_id(&self) -> &str {
        &self.pool_id
    }

    /// Extract content with comprehensive event emission
    pub async fn extract(
        &self,
        html: &str,
        url: &str,
        mode: ExtractionMode,
    ) -> Result<ExtractedDoc> {
        let start_time = Instant::now();

        // Emit extraction started event
        if let Some(event_bus) = &self.event_bus {
            let event = ExtractionEvent::new(
                ExtractionOperation::Started,
                url.to_string(),
                format!("{:?}", &mode),
                &format!("pool-{}", self.pool_id),
            );

            if let Err(e) = event_bus.emit(event).await {
                warn!(error = %e, "Failed to emit extraction started event");
            }
        }

        // Perform the actual extraction
        let extraction_result = self.pool.extract(html, url, mode.clone()).await;

        let duration = start_time.elapsed();

        // Emit completion or failure event
        if let Some(event_bus) = &self.event_bus {
            let event = match &extraction_result {
                Ok(doc) => {
                    ExtractionEvent::new(
                        ExtractionOperation::Completed,
                        url.to_string(),
                        format!("{:?}", &mode),
                        &format!("pool-{}", self.pool_id),
                    )
                    .with_duration(duration)
                    .with_content_length(doc.text.len())
                }
                Err(error) => {
                    ExtractionEvent::new(
                        ExtractionOperation::Failed,
                        url.to_string(),
                        format!("{:?}", &mode),
                        &format!("pool-{}", self.pool_id),
                    )
                    .with_duration(duration)
                    .with_error(error.to_string())
                }
            };

            if let Err(e) = event_bus.emit(event).await {
                warn!(error = %e, "Failed to emit extraction completion event");
            }
        }

        extraction_result
    }

    /// Get current pool status for health monitoring
    pub async fn get_pool_status(&self) -> (usize, usize, usize) {
        self.pool.get_pool_status().await
    }

    /// Emit pool health event
    pub async fn emit_pool_health_event(&self) -> Result<()> {
        if let Some(event_bus) = &self.event_bus {
            let (available, active, max_size) = self.pool.get_pool_status().await;
            let metrics = self.pool.get_metrics().await;

            // Calculate pool health based on metrics
            let health_status = if metrics.circuit_breaker_trips > 5 {
                HealthStatus::Critical
            } else if available == 0 {
                HealthStatus::Unhealthy
            } else if (active as f64 / max_size as f64) > 0.9 {
                HealthStatus::Degraded
            } else {
                HealthStatus::Healthy
            };

            let mut health_metrics = std::collections::HashMap::new();
            health_metrics.insert("available_instances".to_string(), available as f64);
            health_metrics.insert("active_instances".to_string(), active as f64);
            health_metrics.insert("max_instances".to_string(), max_size as f64);
            health_metrics.insert("pool_utilization".to_string(), active as f64 / max_size as f64);
            health_metrics.insert("success_rate".to_string(),
                if metrics.total_extractions > 0 {
                    metrics.successful_extractions as f64 / metrics.total_extractions as f64
                } else {
                    1.0
                }
            );

            let event = HealthEvent::new(
                format!("instance_pool_{}", self.pool_id),
                health_status,
                &format!("pool-{}", self.pool_id),
            )
            .with_details(format!("Pool utilization: {:.1}%, Available: {}, Active: {}",
                (active as f64 / max_size as f64) * 100.0, available, active))
            .with_metrics(health_metrics);

            event_bus.emit(event).await?;
        }

        Ok(())
    }

    /// Start periodic health monitoring
    pub async fn start_health_monitoring(&self, interval: Duration) -> Result<()> {
        if self.event_bus.is_none() {
            return Ok(()); // No event bus, skip monitoring
        }

        let pool_clone = Arc::new(self.clone());

        tokio::spawn(async move {
            let mut health_interval = tokio::time::interval(interval);

            loop {
                health_interval.tick().await;

                if let Err(e) = pool_clone.emit_pool_health_event().await {
                    error!(error = %e, "Failed to emit pool health event");
                }
            }
        });

        info!(pool_id = %self.pool_id, "Started pool health monitoring");
        Ok(())
    }
}

// Implement Clone for EventAwareInstancePool
impl Clone for EventAwareInstancePool {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            event_bus: self.event_bus.clone(),
            pool_id: self.pool_id.clone(),
        }
    }
}

#[async_trait]
impl EventEmitter for EventAwareInstancePool {
    async fn emit_event<E: Event + 'static>(&self, event: E) -> Result<()> {
        if let Some(event_bus) = &self.event_bus {
            event_bus.emit(event).await
        } else {
            warn!("No event bus configured for pool {}", self.pool_id);
            Ok(())
        }
    }
}

/// Event emission helper for enhanced pool operations
pub struct PoolEventEmissionHelper {
    event_bus: Arc<EventBus>,
    pool_id: String,
}

impl PoolEventEmissionHelper {
    pub fn new(event_bus: Arc<EventBus>, pool_id: String) -> Self {
        Self {
            event_bus,
            pool_id,
        }
    }

    /// Emit instance creation event
    pub async fn emit_instance_created(&self, instance_id: &str) -> Result<()> {
        let event = PoolEvent::new(
            PoolOperation::InstanceCreated,
            self.pool_id.clone(),
            &format!("pool-{}", self.pool_id),
        ).with_instance_id(instance_id.to_string());

        self.event_bus.emit(event).await
    }

    /// Emit instance destruction event
    pub async fn emit_instance_destroyed(&self, instance_id: &str) -> Result<()> {
        let event = PoolEvent::new(
            PoolOperation::InstanceDestroyed,
            self.pool_id.clone(),
            &format!("pool-{}", self.pool_id),
        ).with_instance_id(instance_id.to_string());

        self.event_bus.emit(event).await
    }

    /// Emit instance acquired event
    pub async fn emit_instance_acquired(&self, instance_id: &str) -> Result<()> {
        let event = PoolEvent::new(
            PoolOperation::InstanceAcquired,
            self.pool_id.clone(),
            &format!("pool-{}", self.pool_id),
        ).with_instance_id(instance_id.to_string());

        self.event_bus.emit(event).await
    }

    /// Emit instance released event
    pub async fn emit_instance_released(&self, instance_id: &str) -> Result<()> {
        let event = PoolEvent::new(
            PoolOperation::InstanceReleased,
            self.pool_id.clone(),
            &format!("pool-{}", self.pool_id),
        ).with_instance_id(instance_id.to_string());

        self.event_bus.emit(event).await
    }

    /// Emit instance unhealthy event
    pub async fn emit_instance_unhealthy(&self, instance_id: &str, reason: &str) -> Result<()> {
        let mut event = PoolEvent::new(
            PoolOperation::InstanceUnhealthy,
            self.pool_id.clone(),
            &format!("pool-{}", self.pool_id),
        ).with_instance_id(instance_id.to_string());

        event.add_metadata("reason", reason);

        self.event_bus.emit(event).await
    }

    /// Emit pool exhausted event
    pub async fn emit_pool_exhausted(&self, waiting_requests: usize) -> Result<()> {
        let mut event = PoolEvent::new(
            PoolOperation::PoolExhausted,
            self.pool_id.clone(),
            &format!("pool-{}", self.pool_id),
        );

        event.add_metadata("waiting_requests", &waiting_requests.to_string());

        self.event_bus.emit(event).await
    }

    /// Emit circuit breaker tripped event
    pub async fn emit_circuit_breaker_tripped(&self, failure_count: u64) -> Result<()> {
        let mut event = PoolEvent::new(
            PoolOperation::CircuitBreakerTripped,
            self.pool_id.clone(),
            &format!("pool-{}", self.pool_id),
        );

        event.add_metadata("failure_count", &failure_count.to_string());

        self.event_bus.emit(event).await
    }

    /// Emit circuit breaker reset event
    pub async fn emit_circuit_breaker_reset(&self) -> Result<()> {
        let event = PoolEvent::new(
            PoolOperation::CircuitBreakerReset,
            self.pool_id.clone(),
            &format!("pool-{}", self.pool_id),
        );

        self.event_bus.emit(event).await
    }

    /// Emit pool warmup event
    pub async fn emit_pool_warmup(&self, instances_created: usize) -> Result<()> {
        let mut event = PoolEvent::new(
            PoolOperation::PoolWarmup,
            self.pool_id.clone(),
            &format!("pool-{}", self.pool_id),
        );

        event.add_metadata("instances_created", &instances_created.to_string());

        self.event_bus.emit(event).await
    }

    /// Emit pool metrics event
    pub async fn emit_pool_metrics(&self, metrics: PoolMetrics) -> Result<()> {
        let event = PoolEvent::new(
            PoolOperation::InstanceAcquired, // Use as general pool activity
            self.pool_id.clone(),
            &format!("pool-{}", self.pool_id),
        ).with_metrics(metrics);

        self.event_bus.emit(event).await
    }
}

/// Configuration for pool event emission
#[derive(Debug, Clone)]
pub struct PoolEventConfig {
    pub emit_instance_lifecycle: bool,
    pub emit_health_events: bool,
    pub emit_metrics_events: bool,
    pub emit_circuit_breaker_events: bool,
    pub health_check_interval: Duration,
    pub metrics_emission_interval: Duration,
}

impl Default for PoolEventConfig {
    fn default() -> Self {
        Self {
            emit_instance_lifecycle: true,
            emit_health_events: true,
            emit_metrics_events: true,
            emit_circuit_breaker_events: true,
            health_check_interval: Duration::from_secs(30),
            metrics_emission_interval: Duration::from_secs(60),
        }
    }
}

/// Factory for creating event-aware pools
pub struct EventAwarePoolFactory {
    event_bus: Arc<EventBus>,
    config: PoolEventConfig,
}

impl EventAwarePoolFactory {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self {
            event_bus,
            config: PoolEventConfig::default(),
        }
    }

    pub fn with_config(mut self, config: PoolEventConfig) -> Self {
        self.config = config;
        self
    }

    /// Create an event-aware pool from an existing AdvancedInstancePool
    pub async fn create_event_aware_pool(
        &self,
        pool: Arc<AdvancedInstancePool>,
    ) -> Result<EventAwareInstancePool> {
        let event_aware_pool = EventAwareInstancePool::new(pool)
            .with_event_bus(self.event_bus.clone());

        // Start monitoring if configured
        if self.config.emit_health_events {
            event_aware_pool.start_health_monitoring(self.config.health_check_interval).await?;
        }

        // Start metrics emission if configured
        if self.config.emit_metrics_events {
            self.start_metrics_emission(&event_aware_pool).await?;
        }

        info!(
            pool_id = %event_aware_pool.pool_id(),
            "Created event-aware instance pool"
        );

        Ok(event_aware_pool)
    }

    async fn start_metrics_emission(&self, pool: &EventAwareInstancePool) -> Result<()> {
        let pool_clone = pool.clone();
        let interval = self.config.metrics_emission_interval;

        tokio::spawn(async move {
            let mut metrics_interval = tokio::time::interval(interval);

            loop {
                metrics_interval.tick().await;

                // Get current pool metrics and emit as event
                let (available, active, total) = pool_clone.get_pool_status().await;
                let performance_metrics = pool_clone.pool.get_metrics().await;

                let pool_metrics = PoolMetrics {
                    available_instances: available,
                    active_instances: active,
                    total_instances: total,
                    pending_acquisitions: 0, // TODO: Get from pool if available
                    success_rate: if performance_metrics.total_extractions > 0 {
                        performance_metrics.successful_extractions as f64 / performance_metrics.total_extractions as f64
                    } else {
                        1.0
                    },
                    avg_acquisition_time_ms: performance_metrics.semaphore_wait_time_ms as u64,
                    avg_latency_ms: performance_metrics.avg_processing_time_ms as u64,
                };

                let helper = PoolEventEmissionHelper::new(
                    pool_clone.event_bus.as_ref().unwrap().clone(),
                    pool_clone.pool_id().to_string(),
                );

                if let Err(e) = helper.emit_pool_metrics(pool_metrics).await {
                    error!(error = %e, "Failed to emit pool metrics");
                }
            }
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EventBus;
    use crate::events::handlers::LoggingEventHandler;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_event_aware_pool_creation() {
        // This test would require a real AdvancedInstancePool which needs WASM components
        // For now, we'll just test the configuration and factory
        let event_bus = Arc::new(EventBus::new());
        let _factory = EventAwarePoolFactory::new(event_bus);

        // Test that the factory is created successfully
        // TODO: Add actual test logic when WASM components are available
    }

    #[tokio::test]
    async fn test_pool_event_emission_helper() {
        let event_bus = Arc::new(EventBus::new());
        let handler = Arc::new(LoggingEventHandler::new());
        event_bus.register_handler(handler).await.unwrap();

        let helper = PoolEventEmissionHelper::new(event_bus, "test-pool".to_string());

        // Test emitting various pool events
        assert!(helper.emit_instance_created("test-instance-1").await.is_ok());
        assert!(helper.emit_instance_acquired("test-instance-1").await.is_ok());
        assert!(helper.emit_instance_released("test-instance-1").await.is_ok());
        assert!(helper.emit_instance_destroyed("test-instance-1").await.is_ok());

        // Give some time for event processing
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_pool_event_config() {
        let config = PoolEventConfig::default();

        assert!(config.emit_instance_lifecycle);
        assert!(config.emit_health_events);
        assert!(config.emit_metrics_events);
        assert!(config.emit_circuit_breaker_events);
        assert_eq!(config.health_check_interval, Duration::from_secs(30));
    }
}