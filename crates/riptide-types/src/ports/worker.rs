//! Worker service port for hexagonal architecture
//!
//! Provides backend-agnostic trait for background job processing,
//! enabling dependency inversion and testability.

use async_trait::async_trait;

/// Worker health status
#[derive(Debug, Clone)]
pub struct WorkerHealth {
    /// Overall health status
    pub overall_healthy: bool,
    /// Queue health
    pub queue_healthy: bool,
    /// Worker pool health
    pub worker_pool_healthy: bool,
    /// Scheduler health
    pub scheduler_healthy: bool,
    /// Number of active workers
    pub active_workers: usize,
    /// Pending jobs count
    pub pending_jobs: usize,
}

/// Worker service port trait
///
/// Defines the interface for background job processing.
///
/// # Example
///
/// ```rust,ignore
/// use riptide_types::ports::WorkerService;
///
/// async fn check_workers(service: &dyn WorkerService) -> Result<()> {
///     let health = service.health_check().await;
///     if !health.overall_healthy {
///         println!("Warning: Worker service unhealthy");
///     }
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait WorkerService: Send + Sync {
    /// Perform health check
    ///
    /// # Returns
    /// Health status of the worker service
    async fn health_check(&self) -> WorkerHealth;

    /// Get number of active workers
    ///
    /// # Returns
    /// Count of currently active workers
    async fn active_worker_count(&self) -> usize;

    /// Get pending jobs count
    ///
    /// # Returns
    /// Number of jobs waiting to be processed
    async fn pending_jobs_count(&self) -> usize;
}
