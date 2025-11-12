//! Resource management port for hexagonal architecture
//!
//! Provides backend-agnostic trait for managing system resources,
//! enabling dependency inversion and testability.

use crate::error::Result as RiptideResult;
use async_trait::async_trait;

/// Resource status information
#[derive(Debug, Clone)]
pub struct ResourceStatus {
    /// Current memory usage in bytes
    pub memory_usage: usize,
    /// Memory pressure indicator (true if approaching limits)
    pub memory_pressure: bool,
    /// Degradation score (0.0 = healthy, 1.0 = critical)
    pub degradation_score: f64,
    /// Active resource counts by type
    pub active_resources: std::collections::HashMap<String, usize>,
}

/// Resource manager port trait
///
/// Defines the interface for resource management implementations.
/// Handles resource allocation, monitoring, and lifecycle management.
///
/// # Example
///
/// ```rust,ignore
/// use riptide_types::ports::ResourceManager;
///
/// async fn check_resources(manager: &dyn ResourceManager) -> Result<()> {
///     let status = manager.get_resource_status().await;
///     if status.memory_pressure {
///         println!("Warning: Memory pressure detected");
///     }
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait ResourceManager: Send + Sync {
    /// Get current resource status
    ///
    /// # Returns
    /// Current status of all managed resources
    async fn get_resource_status(&self) -> ResourceStatus;

    /// Request resource allocation
    ///
    /// # Arguments
    /// * `resource_type` - Type of resource to allocate
    /// * `amount` - Amount to allocate
    ///
    /// # Returns
    /// * `Ok(())` - Allocation successful
    /// * `Err(_)` - Allocation failed (resource exhausted)
    async fn allocate(&self, resource_type: &str, amount: usize) -> RiptideResult<()>;

    /// Release allocated resources
    ///
    /// # Arguments
    /// * `resource_type` - Type of resource to release
    /// * `amount` - Amount to release
    async fn release(&self, resource_type: &str, amount: usize) -> RiptideResult<()>;

    /// Check if resource manager is healthy
    ///
    /// # Returns
    /// `true` if manager is operating normally
    async fn is_healthy(&self) -> bool;
}
