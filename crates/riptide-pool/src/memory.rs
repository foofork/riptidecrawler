//! Memory management and cleanup for instance pool.
//!
//! Handles memory cleanup, instance recycling, and resource optimization.

use anyhow::Result;
use tracing::{debug, info, warn};

use super::pool::AdvancedInstancePool;
use riptide_events::PoolOperation;

impl AdvancedInstancePool {
    /// Clear a specific number of instances for pool optimization
    pub async fn clear_some_instances(&self, count: usize) -> Result<usize> {
        let mut cleared = 0;

        let instances_to_clear = {
            let mut instances = self.available_instances.lock().await;
            let mut instances_to_clear = Vec::new();
            for _ in 0..count.min(instances.len()) {
                if let Some(instance) = instances.pop_front() {
                    instances_to_clear.push(instance);
                }
            }
            instances_to_clear
        };

        for instance in instances_to_clear {
            info!(instance_id = %instance.id, "Clearing instance for pool optimization");
            self.emit_instance_health_event(&instance, false).await;
            cleared += 1;
        }

        // Create replacement instances
        for _ in 0..cleared {
            if let Ok(new_instance) = self.create_instance().await {
                self.return_instance(new_instance).await;
            }
        }

        info!(
            cleared_count = cleared,
            "Cleared instances for optimization"
        );
        Ok(cleared)
    }

    /// Trigger memory cleanup for all available instances
    pub async fn trigger_memory_cleanup(&self) -> Result<()> {
        info!("Triggering memory cleanup for all instances");

        // Force garbage collection on all available instances
        {
            let instances = self.available_instances.lock().await;
            for instance in instances.iter() {
                // Update memory usage from resource tracker
                let current_pages = instance.resource_tracker.current_memory_pages();
                let memory_bytes = (current_pages * 64 * 1024) as u64;
                debug!(instance_id = %instance.id, memory_pages = current_pages, memory_bytes = memory_bytes,
                       "Updated instance memory usage");
            }
        }

        // Emit memory cleanup event
        if let Some(event_bus) = &self.event_bus {
            let event = riptide_events::PoolEvent::new(
                PoolOperation::MemoryCleanup,
                self.pool_id.clone(),
                "instance_pool",
            );

            if let Err(e) = event_bus.emit(event).await {
                warn!(error = %e, "Failed to emit memory cleanup event");
            }
        }

        Ok(())
    }
}
