//! Retry policy with exponential backoff

use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, warn};

/// Retry policy with exponential backoff configuration
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_attempts: usize,
    /// Initial backoff duration in milliseconds
    pub initial_backoff_ms: u64,
    /// Maximum backoff duration in milliseconds
    pub max_backoff_ms: u64,
    /// Backoff multiplier for exponential increase
    pub backoff_multiplier: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff_ms: 100,
            max_backoff_ms: 10000,
            backoff_multiplier: 2.0,
        }
    }
}

impl RetryPolicy {
    /// Creates a new retry policy with custom configuration
    pub fn new(
        max_attempts: usize,
        initial_backoff_ms: u64,
        max_backoff_ms: u64,
        backoff_multiplier: f64,
    ) -> Self {
        Self {
            max_attempts,
            initial_backoff_ms,
            max_backoff_ms,
            backoff_multiplier,
        }
    }

    /// Calculates backoff duration for a given attempt number
    ///
    /// Uses exponential backoff: initial_backoff * (multiplier ^ attempt)
    /// Capped at max_backoff_ms
    pub fn backoff_duration(&self, attempt: usize) -> Duration {
        let backoff_ms = (self.initial_backoff_ms as f64
            * self.backoff_multiplier.powi(attempt as i32))
        .min(self.max_backoff_ms as f64) as u64;

        Duration::from_millis(backoff_ms)
    }

    /// Executes an async operation with retry logic
    ///
    /// # Arguments
    ///
    /// * `operation` - Async function to execute with retry
    ///
    /// # Errors
    ///
    /// Returns the last error if all retry attempts fail
    pub async fn execute<F, Fut, T, E>(&self, mut operation: F) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        let mut attempt = 0;

        loop {
            match operation().await {
                Ok(result) => {
                    if attempt > 0 {
                        debug!("Operation succeeded after {} retries", attempt);
                    }
                    return Ok(result);
                }
                Err(err) => {
                    attempt += 1;

                    if attempt >= self.max_attempts {
                        warn!(
                            "Operation failed after {} attempts: {}",
                            self.max_attempts, err
                        );
                        return Err(err);
                    }

                    let backoff = self.backoff_duration(attempt - 1);
                    warn!(
                        "Operation failed (attempt {}/{}): {}. Retrying in {:?}",
                        attempt, self.max_attempts, err, backoff
                    );

                    sleep(backoff).await;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_retry_policy_default() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_attempts, 3);
        assert_eq!(policy.initial_backoff_ms, 100);
        assert_eq!(policy.max_backoff_ms, 10000);
        assert_eq!(policy.backoff_multiplier, 2.0);
    }

    #[test]
    fn test_retry_policy_custom() {
        let policy = RetryPolicy::new(5, 200, 5000, 1.5);
        assert_eq!(policy.max_attempts, 5);
        assert_eq!(policy.initial_backoff_ms, 200);
        assert_eq!(policy.max_backoff_ms, 5000);
        assert_eq!(policy.backoff_multiplier, 1.5);
    }

    #[test]
    fn test_backoff_duration_exponential() {
        let policy = RetryPolicy::default();

        // First attempt: 100ms
        assert_eq!(policy.backoff_duration(0).as_millis(), 100);

        // Second attempt: 100 * 2^1 = 200ms
        assert_eq!(policy.backoff_duration(1).as_millis(), 200);

        // Third attempt: 100 * 2^2 = 400ms
        assert_eq!(policy.backoff_duration(2).as_millis(), 400);

        // Fourth attempt: 100 * 2^3 = 800ms
        assert_eq!(policy.backoff_duration(3).as_millis(), 800);
    }

    #[test]
    fn test_backoff_duration_capped() {
        let policy = RetryPolicy {
            max_attempts: 10,
            initial_backoff_ms: 100,
            max_backoff_ms: 1000,
            backoff_multiplier: 2.0,
        };

        // Should cap at 1000ms
        let duration = policy.backoff_duration(10);
        assert_eq!(duration.as_millis(), 1000);
    }

    #[tokio::test]
    async fn test_execute_success_first_attempt() {
        let policy = RetryPolicy::default();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let result = policy
            .execute(|| async {
                counter_clone.fetch_add(1, Ordering::SeqCst);
                Ok::<_, String>(42)
            })
            .await;

        assert_eq!(result, Ok(42));
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_execute_success_after_retries() {
        let policy = RetryPolicy {
            max_attempts: 3,
            initial_backoff_ms: 10, // Short for testing
            max_backoff_ms: 100,
            backoff_multiplier: 2.0,
        };

        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let result = policy
            .execute(|| async {
                let count = counter_clone.fetch_add(1, Ordering::SeqCst);
                if count < 2 {
                    Err("temporary error".to_string())
                } else {
                    Ok(42)
                }
            })
            .await;

        assert_eq!(result, Ok(42));
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_execute_all_attempts_fail() {
        let policy = RetryPolicy {
            max_attempts: 3,
            initial_backoff_ms: 10,
            max_backoff_ms: 100,
            backoff_multiplier: 2.0,
        };

        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let result = policy
            .execute(|| async {
                counter_clone.fetch_add(1, Ordering::SeqCst);
                Err::<i32, _>("persistent error".to_string())
            })
            .await;

        assert_eq!(result, Err("persistent error".to_string()));
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }
}
