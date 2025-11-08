//! Backpressure and cancellation token management for resource control.
//!
//! This module provides:
//! - Concurrency limit enforcement via semaphores
//! - Graceful cancellation via tokio CancellationToken
//! - Real-time load metrics for monitoring
//! - RAII-based resource cleanup via BackpressureGuard

use crate::{RiptideError, RiptideResult};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio_util::sync::CancellationToken;

/// Manages backpressure and resource limits for concurrent operations.
///
/// Uses a semaphore to limit the number of concurrent operations and provides
/// graceful cancellation support via CancellationToken.
///
/// # Example
///
/// ```no_run
/// use riptide_facade::workflows::backpressure::BackpressureManager;
/// use tokio_util::sync::CancellationToken;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let manager = BackpressureManager::new(10); // Max 10 concurrent operations
/// let cancel_token = CancellationToken::new();
///
/// // Acquire permit (blocks if at capacity)
/// let guard = manager.acquire(&cancel_token).await?;
///
/// // Do work with resource...
/// // Guard automatically releases permit when dropped
/// drop(guard);
///
/// // Check current load
/// let load = manager.current_load();
/// println!("Current load: {:.1}%", load * 100.0);
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct BackpressureManager {
    semaphore: Arc<Semaphore>,
    active_count: Arc<AtomicUsize>,
    max_concurrency: usize,
}

impl BackpressureManager {
    /// Create a new backpressure manager with the specified concurrency limit.
    ///
    /// # Arguments
    ///
    /// * `max_concurrency` - Maximum number of concurrent operations allowed
    ///
    /// # Example
    ///
    /// ```
    /// use riptide_facade::workflows::backpressure::BackpressureManager;
    ///
    /// let manager = BackpressureManager::new(5);
    /// ```
    pub fn new(max_concurrency: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrency)),
            active_count: Arc::new(AtomicUsize::new(0)),
            max_concurrency,
        }
    }

    /// Acquire a permit for a new operation, respecting cancellation.
    ///
    /// This method will:
    /// - Block until a permit is available (if at capacity)
    /// - Return immediately if the cancellation token is triggered
    /// - Increment the active operation count
    /// - Return a guard that automatically releases the permit when dropped
    ///
    /// # Arguments
    ///
    /// * `cancel_token` - Token that can cancel the acquisition
    ///
    /// # Returns
    ///
    /// Returns a `BackpressureGuard` on success, which must be held for the
    /// duration of the operation. The permit is automatically released when
    /// the guard is dropped.
    ///
    /// # Errors
    ///
    /// - `RiptideError::Cancelled` if the cancellation token is triggered
    /// - `RiptideError::ResourceExhausted` if the semaphore is closed
    ///
    /// # Example
    ///
    /// ```no_run
    /// use riptide_facade::workflows::backpressure::BackpressureManager;
    /// use tokio_util::sync::CancellationToken;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = BackpressureManager::new(10);
    /// let cancel_token = CancellationToken::new();
    ///
    /// let guard = manager.acquire(&cancel_token).await?;
    /// // Perform operation...
    /// drop(guard); // Release permit
    /// # Ok(())
    /// # }
    /// ```
    pub async fn acquire(
        &self,
        cancel_token: &CancellationToken,
    ) -> RiptideResult<BackpressureGuard> {
        tokio::select! {
            permit = self.semaphore.clone().acquire_owned() => {
                let permit = permit.map_err(|_|
                    RiptideError::Other(anyhow::anyhow!("Resource exhausted: semaphore closed")))?;
                self.active_count.fetch_add(1, Ordering::SeqCst);

                tracing::debug!(
                    active = self.active_count.load(Ordering::SeqCst),
                    max = self.max_concurrency,
                    "Acquired backpressure permit"
                );

                Ok(BackpressureGuard {
                    _permit: permit,
                    active_count: self.active_count.clone(),
                })
            }
            _ = cancel_token.cancelled() => {
                tracing::debug!("Backpressure acquire cancelled");
                Err(RiptideError::Other(anyhow::anyhow!("Operation cancelled")))
            }
        }
    }

    /// Get the current load as a ratio (0.0 to 1.0).
    ///
    /// Returns the percentage of capacity currently in use:
    /// - 0.0 = no active operations
    /// - 0.5 = 50% capacity
    /// - 1.0 = at full capacity
    ///
    /// # Example
    ///
    /// ```
    /// use riptide_facade::workflows::backpressure::BackpressureManager;
    ///
    /// let manager = BackpressureManager::new(10);
    /// let load = manager.current_load();
    /// assert_eq!(load, 0.0); // No operations yet
    /// ```
    pub fn current_load(&self) -> f64 {
        let active = self.active_count.load(Ordering::SeqCst);
        active as f64 / self.max_concurrency as f64
    }

    /// Get the current number of active operations.
    ///
    /// # Example
    ///
    /// ```
    /// use riptide_facade::workflows::backpressure::BackpressureManager;
    ///
    /// let manager = BackpressureManager::new(10);
    /// assert_eq!(manager.active_operations(), 0);
    /// ```
    pub fn active_operations(&self) -> usize {
        self.active_count.load(Ordering::SeqCst)
    }

    /// Get the maximum concurrency limit.
    ///
    /// # Example
    ///
    /// ```
    /// use riptide_facade::workflows::backpressure::BackpressureManager;
    ///
    /// let manager = BackpressureManager::new(10);
    /// assert_eq!(manager.max_concurrency(), 10);
    /// ```
    pub fn max_concurrency(&self) -> usize {
        self.max_concurrency
    }

    /// Get the number of available permits.
    ///
    /// # Example
    ///
    /// ```
    /// use riptide_facade::workflows::backpressure::BackpressureManager;
    ///
    /// let manager = BackpressureManager::new(10);
    /// assert_eq!(manager.available_permits(), 10);
    /// ```
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }
}

/// RAII guard for backpressure permits.
///
/// Automatically releases the semaphore permit and decrements the active
/// count when dropped. This ensures that resources are always properly
/// cleaned up, even in the presence of panics or early returns.
///
/// # Example
///
/// ```no_run
/// use riptide_facade::workflows::backpressure::BackpressureManager;
/// use tokio_util::sync::CancellationToken;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let manager = BackpressureManager::new(5);
/// let cancel_token = CancellationToken::new();
///
/// {
///     let guard = manager.acquire(&cancel_token).await?;
///     // Permit is held here
///     assert_eq!(manager.active_operations(), 1);
/// } // Guard dropped here, permit automatically released
///
/// assert_eq!(manager.active_operations(), 0);
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct BackpressureGuard {
    _permit: OwnedSemaphorePermit,
    active_count: Arc<AtomicUsize>,
}

impl Drop for BackpressureGuard {
    fn drop(&mut self) {
        let previous = self.active_count.fetch_sub(1, Ordering::SeqCst);
        tracing::debug!(active = previous - 1, "Released backpressure permit");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_backpressure_manager_creation() {
        let manager = BackpressureManager::new(10);
        assert_eq!(manager.max_concurrency(), 10);
        assert_eq!(manager.active_operations(), 0);
        assert_eq!(manager.current_load(), 0.0);
        assert_eq!(manager.available_permits(), 10);
    }

    #[tokio::test]
    async fn test_acquire_and_release() {
        let manager = BackpressureManager::new(5);
        let cancel_token = CancellationToken::new();

        // Acquire permit
        let guard = manager.acquire(&cancel_token).await.unwrap();
        assert_eq!(manager.active_operations(), 1);
        assert_eq!(manager.current_load(), 0.2); // 1/5 = 0.2
        assert_eq!(manager.available_permits(), 4);

        // Release permit
        drop(guard);
        assert_eq!(manager.active_operations(), 0);
        assert_eq!(manager.current_load(), 0.0);
        assert_eq!(manager.available_permits(), 5);
    }

    #[tokio::test]
    async fn test_multiple_acquires() {
        let manager = BackpressureManager::new(3);
        let cancel_token = CancellationToken::new();

        let guard1 = manager.acquire(&cancel_token).await.unwrap();
        let guard2 = manager.acquire(&cancel_token).await.unwrap();
        let guard3 = manager.acquire(&cancel_token).await.unwrap();

        assert_eq!(manager.active_operations(), 3);
        assert_eq!(manager.current_load(), 1.0); // At capacity
        assert_eq!(manager.available_permits(), 0);

        drop(guard1);
        assert_eq!(manager.active_operations(), 2);
        assert_eq!(manager.available_permits(), 1);

        drop(guard2);
        drop(guard3);
        assert_eq!(manager.active_operations(), 0);
        assert_eq!(manager.available_permits(), 3);
    }

    #[tokio::test]
    async fn test_concurrency_limit_enforced() {
        let manager = BackpressureManager::new(2);
        let cancel_token = CancellationToken::new();

        // Acquire max permits
        let _guard1 = manager.acquire(&cancel_token).await.unwrap();
        let _guard2 = manager.acquire(&cancel_token).await.unwrap();

        assert_eq!(manager.active_operations(), 2);
        assert_eq!(manager.available_permits(), 0);

        // Try to acquire another - should block
        let manager_clone = manager.clone();
        let cancel_clone = cancel_token.clone();
        let acquire_task = tokio::spawn(async move { manager_clone.acquire(&cancel_clone).await });

        // Give it time to start blocking
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Should still be 2 active (third is blocked)
        assert_eq!(manager.active_operations(), 2);

        // Abort the blocked task
        acquire_task.abort();
    }

    #[tokio::test]
    async fn test_cancellation_token() {
        let manager = BackpressureManager::new(1);
        let cancel_token = CancellationToken::new();

        // Acquire the only permit
        let _guard = manager.acquire(&cancel_token).await.unwrap();

        // Try to acquire with a different token that we'll cancel
        let cancel_token2 = CancellationToken::new();
        let manager_clone = manager.clone();
        let cancel_clone = cancel_token2.clone();

        let acquire_task = tokio::spawn(async move { manager_clone.acquire(&cancel_clone).await });

        // Give it time to start blocking
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Cancel the token
        cancel_token2.cancel();

        // Should get a cancellation error
        let result = acquire_task.await.unwrap();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cancelled"));
    }

    #[tokio::test]
    async fn test_load_metrics() {
        let manager = BackpressureManager::new(10);
        let cancel_token = CancellationToken::new();

        assert_eq!(manager.current_load(), 0.0);

        let guard1 = manager.acquire(&cancel_token).await.unwrap();
        assert_eq!(manager.current_load(), 0.1);

        let guard2 = manager.acquire(&cancel_token).await.unwrap();
        assert_eq!(manager.current_load(), 0.2);

        let guard3 = manager.acquire(&cancel_token).await.unwrap();
        let guard4 = manager.acquire(&cancel_token).await.unwrap();
        let guard5 = manager.acquire(&cancel_token).await.unwrap();
        assert_eq!(manager.current_load(), 0.5);

        drop(guard1);
        drop(guard2);
        drop(guard3);
        drop(guard4);
        drop(guard5);
        assert_eq!(manager.current_load(), 0.0);
    }

    #[tokio::test]
    async fn test_guard_cleanup_on_drop() {
        let manager = BackpressureManager::new(5);
        let cancel_token = CancellationToken::new();

        {
            let _guard = manager.acquire(&cancel_token).await.unwrap();
            assert_eq!(manager.active_operations(), 1);
        } // Guard dropped here

        // Should be cleaned up
        assert_eq!(manager.active_operations(), 0);
        assert_eq!(manager.available_permits(), 5);
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let manager = Arc::new(BackpressureManager::new(5));
        let cancel_token = CancellationToken::new();

        // Spawn multiple concurrent tasks
        let mut handles = vec![];
        for _ in 0..10 {
            let manager_clone = manager.clone();
            let cancel_clone = cancel_token.clone();

            let handle = tokio::spawn(async move {
                let _guard = manager_clone.acquire(&cancel_clone).await.unwrap();
                tokio::time::sleep(Duration::from_millis(10)).await;
            });
            handles.push(handle);
        }

        // Wait for all to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // All should be cleaned up
        assert_eq!(manager.active_operations(), 0);
        assert_eq!(manager.current_load(), 0.0);
    }

    #[tokio::test]
    async fn test_permits_released_on_panic() {
        let manager = Arc::new(BackpressureManager::new(5));
        let cancel_token = CancellationToken::new();

        let manager_clone = manager.clone();
        let cancel_clone = cancel_token.clone();

        // Spawn task that panics after acquiring
        let handle = tokio::spawn(async move {
            let _guard = manager_clone.acquire(&cancel_clone).await.unwrap();
            panic!("Intentional panic for testing");
        });

        // Wait for panic
        let _ = handle.await;

        // Permit should still be released
        assert_eq!(manager.active_operations(), 0);
        assert_eq!(manager.available_permits(), 5);
    }

    #[tokio::test]
    async fn test_clone_manager() {
        let manager = BackpressureManager::new(3);
        let manager_clone = manager.clone();
        let cancel_token = CancellationToken::new();

        let guard1 = manager.acquire(&cancel_token).await.unwrap();
        assert_eq!(manager.active_operations(), 1);
        assert_eq!(manager_clone.active_operations(), 1);

        let guard2 = manager_clone.acquire(&cancel_token).await.unwrap();
        assert_eq!(manager.active_operations(), 2);
        assert_eq!(manager_clone.active_operations(), 2);

        drop(guard1);
        drop(guard2);
        assert_eq!(manager.active_operations(), 0);
        assert_eq!(manager_clone.active_operations(), 0);
    }

    #[tokio::test]
    async fn test_full_capacity_blocking() {
        let manager = Arc::new(BackpressureManager::new(2));
        let cancel_token = CancellationToken::new();

        // Fill capacity
        let guard1 = manager.acquire(&cancel_token).await.unwrap();
        let guard2 = manager.acquire(&cancel_token).await.unwrap();

        assert_eq!(manager.current_load(), 1.0);

        // Try to acquire - should block until one is released
        let manager_clone = manager.clone();
        let cancel_clone = cancel_token.clone();
        let blocked_task =
            tokio::spawn(async move { manager_clone.acquire(&cancel_clone).await.unwrap() });

        // Give it time to block
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Release one permit
        drop(guard1);

        // Blocked task should now complete
        let guard3 = tokio::time::timeout(Duration::from_secs(1), blocked_task)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(manager.active_operations(), 2);

        drop(guard2);
        drop(guard3);
        assert_eq!(manager.active_operations(), 0);
    }

    #[tokio::test]
    async fn test_zero_concurrency() {
        // Edge case: zero concurrency
        let manager = BackpressureManager::new(0);
        assert_eq!(manager.max_concurrency(), 0);
        assert_eq!(manager.available_permits(), 0);

        // current_load should handle division by zero
        let load = manager.current_load();
        assert!(load.is_nan() || load == 0.0);
    }

    #[tokio::test]
    async fn test_high_concurrency() {
        // Test with high concurrency limit
        let manager = BackpressureManager::new(1000);
        let cancel_token = CancellationToken::new();

        let mut guards = vec![];
        for _ in 0..100 {
            guards.push(manager.acquire(&cancel_token).await.unwrap());
        }

        assert_eq!(manager.active_operations(), 100);
        assert_eq!(manager.current_load(), 0.1);

        guards.clear();
        assert_eq!(manager.active_operations(), 0);
    }
}
