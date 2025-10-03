//! Instance health monitoring and validation.
//!
//! Provides health check functionality for WASM component instances.

use anyhow::Result;
use std::time::Duration;
use tracing::{debug, error, info, warn};

use super::models::PooledInstance;
use super::pool::AdvancedInstancePool;
use crate::events::PoolOperation;

impl AdvancedInstancePool {
    /// Start continuous health monitoring for pool instances
    #[allow(dead_code)]
    pub async fn start_instance_health_monitoring(self: std::sync::Arc<Self>) -> Result<()> {
        let interval_ms = self.config.health_check_interval;
        let interval_duration = Duration::from_millis(interval_ms);
        info!(
            interval_secs = interval_duration.as_secs(),
            "Starting continuous instance health monitoring"
        );

        let mut interval_timer = tokio::time::interval(interval_duration);

        loop {
            interval_timer.tick().await;

            if let Err(e) = self.perform_instance_health_checks().await {
                error!(error = %e, "Instance health check failed");
            }
        }
    }

    /// Perform health checks on all instances in the pool
    pub(super) async fn perform_instance_health_checks(&self) -> Result<()> {
        let mut unhealthy_instances = Vec::new();
        let mut healthy_count = 0;

        // Check available instances
        let instance_health_data = {
            let instances = self.available_instances.lock().await;
            instances
                .iter()
                .map(|i| (i.id.clone(), i.created_at, i.failure_count))
                .collect::<Vec<_>>()
        };

        for (id, created_at, failure_count) in instance_health_data {
            // Simple health check without full instance
            let is_healthy =
                created_at.elapsed() <= Duration::from_secs(3600) && failure_count <= 5;

            if !is_healthy {
                let mut instances = self.available_instances.lock().await;
                if let Some(pos) = instances.iter().position(|i| i.id == id) {
                    let unhealthy_instance = instances.remove(pos).unwrap();
                    drop(instances);
                    unhealthy_instances.push(unhealthy_instance);
                }
            } else {
                healthy_count += 1;
            }
        }

        // Replace unhealthy instances
        if !unhealthy_instances.is_empty() {
            warn!(
                unhealthy_count = unhealthy_instances.len(),
                "Replacing unhealthy instances"
            );

            for unhealthy in unhealthy_instances {
                // Emit instance health degraded event
                self.emit_instance_health_event(&unhealthy, false).await;

                // Create replacement instance
                if let Ok(new_instance) = self.create_instance().await {
                    self.return_instance(new_instance).await;
                    info!("Replaced unhealthy instance with new healthy instance");
                } else {
                    error!("Failed to create replacement instance");
                }
            }
        }

        // Emit overall health metrics
        self.emit_pool_health_metrics(healthy_count).await;

        Ok(())
    }

    /// Validate health of a specific instance
    #[allow(dead_code)]
    pub(super) async fn validate_instance_health(&self, instance: &PooledInstance) -> bool {
        // Check age - instances older than 1 hour should be recycled
        if instance.created_at.elapsed() > Duration::from_secs(3600) {
            debug!(instance_id = %instance.id, "Instance expired due to age");
            return false;
        }

        // Check failure rate
        if instance.failure_count > 5 {
            debug!(instance_id = %instance.id, failure_count = instance.failure_count, "Instance has too many failures");
            return false;
        }

        // Check memory usage
        let memory_limit_bytes = self.config.memory_limit;
        if instance.memory_usage_bytes > memory_limit_bytes.unwrap_or(usize::MAX) as u64 {
            debug!(instance_id = %instance.id, memory_usage = instance.memory_usage_bytes, "Instance memory usage too high");
            return false;
        }

        // Check resource tracker health
        if instance.resource_tracker.grow_failures() > 10 {
            debug!(instance_id = %instance.id, grow_failures = instance.resource_tracker.grow_failures(), "Instance has too many memory grow failures");
            return false;
        }

        // Check if instance has been idle too long
        if instance.last_used.elapsed() > Duration::from_secs(1800) {
            // 30 minutes
            debug!(instance_id = %instance.id, "Instance idle too long, marking for replacement");
            return false;
        }

        true
    }

    /// Emit instance health event
    pub(super) async fn emit_instance_health_event(
        &self,
        instance: &PooledInstance,
        healthy: bool,
    ) {
        if let Some(event_bus) = &self.event_bus {
            let operation = if healthy {
                PoolOperation::InstanceHealthy
            } else {
                PoolOperation::InstanceUnhealthy
            };

            let mut event =
                crate::events::PoolEvent::new(operation, self.pool_id.clone(), "instance_pool")
                    .with_instance_id(instance.id.clone());

            // Add health metrics
            event.add_metadata("use_count", &instance.use_count.to_string());
            event.add_metadata("failure_count", &instance.failure_count.to_string());
            event.add_metadata(
                "memory_usage_bytes",
                &instance.memory_usage_bytes.to_string(),
            );
            event.add_metadata(
                "age_seconds",
                &instance.created_at.elapsed().as_secs().to_string(),
            );
            event.add_metadata(
                "grow_failures",
                &instance.resource_tracker.grow_failures().to_string(),
            );

            if let Err(e) = event_bus.emit(event).await {
                warn!(error = %e, instance_id = %instance.id, "Failed to emit instance health event");
            }
        }
    }

    /// Emit pool health metrics
    pub(super) async fn emit_pool_health_metrics(&self, healthy_count: usize) {
        if let Some(event_bus) = &self.event_bus {
            let (available, active, total) = self.get_pool_status().await;
            let metrics = self.get_pool_metrics_for_events().await;

            let mut event = crate::events::PoolEvent::new(
                PoolOperation::HealthCheck,
                self.pool_id.clone(),
                "instance_pool",
            );

            // Add comprehensive metrics
            event.add_metadata("healthy_instances", &healthy_count.to_string());
            event.add_metadata("available_instances", &available.to_string());
            event.add_metadata("active_instances", &active.to_string());
            event.add_metadata("total_instances", &total.to_string());
            event.add_metadata("success_rate", &metrics.success_rate.to_string());
            event.add_metadata("avg_latency_ms", &metrics.avg_latency_ms.to_string());

            if let Err(e) = event_bus.emit(event).await {
                warn!(error = %e, "Failed to emit pool health metrics");
            }
        }
    }

    /// Clear instances with high memory usage
    pub async fn clear_high_memory_instances(&self) -> Result<usize> {
        let memory_threshold =
            (self.config.memory_limit.unwrap_or(512 * 1024 * 1024) as f64 * 0.8) as u64; // 80% of limit
        let mut cleared = 0;

        let high_memory_instances = {
            let mut instances = self.available_instances.lock().await;
            let mut high_memory_instances = Vec::new();
            let mut i = 0;
            while i < instances.len() {
                if instances[i].memory_usage_bytes > memory_threshold {
                    let instance = instances.remove(i).unwrap();
                    high_memory_instances.push(instance);
                } else {
                    i += 1;
                }
            }
            high_memory_instances
        };

        for instance in high_memory_instances {
            info!(instance_id = %instance.id, memory_usage = instance.memory_usage_bytes,
                  "Clearing high memory instance");
            cleared += 1;

            // Emit instance destroyed event
            self.emit_instance_health_event(&instance, false).await;
        }

        // Create replacement instances
        for _ in 0..cleared {
            if let Ok(new_instance) = self.create_instance().await {
                self.return_instance(new_instance).await;
            }
        }

        info!(
            cleared_count = cleared,
            "Cleared high memory instances and created replacements"
        );
        Ok(cleared)
    }
}
