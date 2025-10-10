//! WASM instance manager with per-worker tracking.
//!
//! Ensures single WASM instance per worker with:
//! - Instance lifecycle management
//! - Health monitoring
//! - Automatic cleanup of stale instances

use crate::resource_manager::{errors::Result, metrics::ResourceMetrics};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// WASM instance manager with single instance per worker
///
/// Maintains one WASM instance per worker thread/task, ensuring efficient
/// resource usage while preventing conflicts from shared instances.
pub struct WasmInstanceManager {
    worker_instances: RwLock<HashMap<String, WasmWorkerInstance>>,
    metrics: Arc<ResourceMetrics>,
}

/// WASM instance for a specific worker
#[derive(Debug)]
struct WasmWorkerInstance {
    pub worker_id: String,
    pub created_at: Instant,
    pub operations_count: u64,
    pub last_operation: Instant,
    pub is_healthy: bool,
    pub memory_usage: usize,
}

/// WASM guard that maintains instance lifetime
pub struct WasmGuard {
    #[allow(dead_code)] // Keeps manager Arc alive for lifetime of guard
    manager: Arc<WasmInstanceManager>,
}

impl WasmInstanceManager {
    /// Create a new WASM instance manager
    pub(crate) fn new(metrics: Arc<ResourceMetrics>) -> Result<Self> {
        info!("Initializing WASM instance manager");

        Ok(Self {
            worker_instances: RwLock::new(HashMap::new()),
            metrics,
        })
    }

    /// Acquire a WASM instance for the given worker
    ///
    /// If the worker doesn't have an instance yet, one will be created.
    /// The instance is tracked and its usage statistics are updated.
    ///
    /// # Arguments
    /// * `worker_id` - Unique identifier for the worker
    ///
    /// # Returns
    /// A `WasmGuard` that keeps the instance alive
    pub(crate) async fn acquire_instance(
        self: &Arc<Self>,
        worker_id: &str,
    ) -> Result<WasmGuard> {
        let mut instances = self.worker_instances.write().await;

        // Ensure single instance per worker (requirement)
        if !instances.contains_key(worker_id) {
            debug!(worker_id = %worker_id, "Creating new WASM instance for worker");

            let instance = WasmWorkerInstance {
                worker_id: worker_id.to_string(),
                created_at: Instant::now(),
                operations_count: 0,
                last_operation: Instant::now(),
                is_healthy: true,
                memory_usage: 0,
            };

            instances.insert(worker_id.to_string(), instance);
            self.metrics
                .wasm_instances
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }

        // Update instance usage
        if let Some(instance) = instances.get_mut(worker_id) {
            instance.operations_count += 1;
            instance.last_operation = Instant::now();

            debug!(
                worker_id = %worker_id,
                operations = instance.operations_count,
                "Updated WASM instance usage"
            );
        }

        Ok(WasmGuard {
            manager: self.clone(),
        })
    }

    /// Get health status of all WASM instances
    ///
    /// Returns a list of tuples containing:
    /// - Worker ID
    /// - Health status
    /// - Operation count
    /// - Memory usage
    /// - Instance age
    pub async fn get_instance_health(&self) -> Vec<(String, bool, u64, usize, Duration)> {
        let instances = self.worker_instances.read().await;
        instances
            .values()
            .map(|instance| {
                let age = instance.created_at.elapsed();
                (
                    instance.worker_id.clone(),
                    instance.is_healthy,
                    instance.operations_count,
                    instance.memory_usage,
                    age,
                )
            })
            .collect()
    }

    /// Get statistics for all WASM instances
    pub async fn get_all_stats(&self) -> Vec<WasmInstanceStats> {
        let instances = self.worker_instances.read().await;
        instances
            .values()
            .map(|instance| WasmInstanceStats {
                worker_id: instance.worker_id.clone(),
                operations_count: instance.operations_count,
                memory_usage_mb: instance.memory_usage,
                age: instance.created_at.elapsed(),
                idle_time: instance.last_operation.elapsed(),
                is_healthy: instance.is_healthy,
            })
            .collect()
    }

    /// Check if any WASM instance needs cleanup (idle for >1 hour)
    pub async fn needs_cleanup(&self) -> bool {
        let instances = self.worker_instances.read().await;
        let now = Instant::now();

        instances.values().any(|instance| {
            now.duration_since(instance.last_operation) > Duration::from_secs(3600)
        })
    }

    /// Clean up stale WASM instances
    ///
    /// Removes instances that have been idle for more than the specified duration.
    ///
    /// # Arguments
    /// * `idle_threshold` - Duration after which idle instances are cleaned up
    ///
    /// # Returns
    /// Number of instances cleaned up
    pub async fn cleanup_stale_instances(&self, idle_threshold: Duration) -> usize {
        let mut instances = self.worker_instances.write().await;
        let now = Instant::now();

        let count_before = instances.len();

        instances.retain(|_, instance| {
            let is_active = now.duration_since(instance.last_operation) < idle_threshold;
            if !is_active {
                debug!(
                    worker_id = %instance.worker_id,
                    idle_time = ?now.duration_since(instance.last_operation),
                    "Cleaning up stale WASM instance"
                );
            }
            is_active
        });

        let count_after = instances.len();
        let cleaned = count_before - count_after;

        if cleaned > 0 {
            info!(cleaned = cleaned, remaining = count_after, "Cleaned up stale WASM instances");
            self.metrics
                .wasm_instances
                .store(count_after, std::sync::atomic::Ordering::Relaxed);
        }

        cleaned
    }

    /// Get total number of active WASM instances
    pub async fn instance_count(&self) -> usize {
        self.worker_instances.read().await.len()
    }

    /// Get statistics for a specific worker
    pub async fn get_worker_stats(&self, worker_id: &str) -> Option<WasmInstanceStats> {
        let instances = self.worker_instances.read().await;
        instances.get(worker_id).map(|instance| WasmInstanceStats {
            worker_id: instance.worker_id.clone(),
            operations_count: instance.operations_count,
            memory_usage_mb: instance.memory_usage,
            age: instance.created_at.elapsed(),
            idle_time: instance.last_operation.elapsed(),
            is_healthy: instance.is_healthy,
        })
    }
}

/// Statistics for a WASM instance
#[derive(Debug, Clone, serde::Serialize)]
pub struct WasmInstanceStats {
    pub worker_id: String,
    pub operations_count: u64,
    pub memory_usage_mb: usize,
    pub age: Duration,
    pub idle_time: Duration,
    pub is_healthy: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_wasm_manager_creation() {
        let metrics = Arc::new(ResourceMetrics::new());
        let manager = Arc::new(WasmInstanceManager::new(metrics).unwrap());

        assert_eq!(manager.instance_count().await, 0);
    }

    #[tokio::test]
    async fn test_single_instance_per_worker() {
        let metrics = Arc::new(ResourceMetrics::new());
        let manager = Arc::new(WasmInstanceManager::new(metrics.clone()).unwrap());

        let worker_id = "worker_1";

        // Acquire instance multiple times for same worker
        let _guard1 = manager.acquire_instance(worker_id).await.unwrap();
        let _guard2 = manager.acquire_instance(worker_id).await.unwrap();

        // Should still have only one instance
        assert_eq!(manager.instance_count().await, 1);

        // Check operation count
        let stats = manager.get_worker_stats(worker_id).await.unwrap();
        assert_eq!(stats.operations_count, 2);
    }

    #[tokio::test]
    async fn test_multiple_workers() {
        let metrics = Arc::new(ResourceMetrics::new());
        let manager = Arc::new(WasmInstanceManager::new(metrics).unwrap());

        let _guard1 = manager.acquire_instance("worker_1").await.unwrap();
        let _guard2 = manager.acquire_instance("worker_2").await.unwrap();
        let _guard3 = manager.acquire_instance("worker_3").await.unwrap();

        assert_eq!(manager.instance_count().await, 3);
    }

    #[tokio::test]
    async fn test_instance_health_status() {
        let metrics = Arc::new(ResourceMetrics::new());
        let manager = Arc::new(WasmInstanceManager::new(metrics).unwrap());

        let _guard = manager.acquire_instance("worker_1").await.unwrap();

        let health = manager.get_instance_health().await;
        assert_eq!(health.len(), 1);

        let (worker_id, is_healthy, ops_count, _, _) = &health[0];
        assert_eq!(worker_id, "worker_1");
        assert!(is_healthy);
        assert_eq!(*ops_count, 1);
    }

    #[tokio::test]
    async fn test_cleanup_stale_instances() {
        let metrics = Arc::new(ResourceMetrics::new());
        let manager = Arc::new(WasmInstanceManager::new(metrics).unwrap());

        // Create instance
        let _guard = manager.acquire_instance("worker_1").await.unwrap();
        drop(_guard);

        assert_eq!(manager.instance_count().await, 1);

        // Should not cleanup immediately
        let cleaned = manager
            .cleanup_stale_instances(Duration::from_secs(3600))
            .await;
        assert_eq!(cleaned, 0);

        // Should cleanup with zero threshold
        let cleaned = manager.cleanup_stale_instances(Duration::from_secs(0)).await;
        assert_eq!(cleaned, 1);
        assert_eq!(manager.instance_count().await, 0);
    }

    #[tokio::test]
    async fn test_needs_cleanup() {
        let metrics = Arc::new(ResourceMetrics::new());
        let manager = Arc::new(WasmInstanceManager::new(metrics).unwrap());

        assert!(!manager.needs_cleanup().await);

        let _guard = manager.acquire_instance("worker_1").await.unwrap();

        // Fresh instance shouldn't need cleanup
        assert!(!manager.needs_cleanup().await);
    }
}
