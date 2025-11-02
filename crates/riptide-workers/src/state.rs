/*!
# Worker State Management

Comprehensive state machine management for worker and job lifecycle with
transition guards to prevent invalid state changes and race conditions.
*/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, warn};

/// Worker state representing lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerState {
    /// Worker is idle and ready to accept jobs
    Idle,
    /// Worker is currently processing a job
    Processing,
    /// Worker is paused (temporary suspension)
    Paused,
    /// Worker has failed (recoverable failure)
    Failed,
    /// Worker has completed processing
    Completed,
    /// Worker is shutting down gracefully
    ShuttingDown,
    /// Worker is terminated (permanent stop)
    Terminated,
}

impl fmt::Display for WorkerState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkerState::Idle => write!(f, "Idle"),
            WorkerState::Processing => write!(f, "Processing"),
            WorkerState::Paused => write!(f, "Paused"),
            WorkerState::Failed => write!(f, "Failed"),
            WorkerState::Completed => write!(f, "Completed"),
            WorkerState::ShuttingDown => write!(f, "ShuttingDown"),
            WorkerState::Terminated => write!(f, "Terminated"),
        }
    }
}

/// Job state representing processing lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JobState {
    /// Job is pending in the queue
    Pending,
    /// Job has been assigned to a worker
    Assigned,
    /// Job is currently being processed
    Processing,
    /// Job is paused (can be resumed)
    Paused,
    /// Job completed successfully
    Completed,
    /// Job failed (may be retried)
    Failed,
    /// Job is being retried
    Retrying,
    /// Job is cancelled by user
    Cancelled,
    /// Job timed out during processing
    TimedOut,
}

impl fmt::Display for JobState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JobState::Pending => write!(f, "Pending"),
            JobState::Assigned => write!(f, "Assigned"),
            JobState::Processing => write!(f, "Processing"),
            JobState::Paused => write!(f, "Paused"),
            JobState::Completed => write!(f, "Completed"),
            JobState::Failed => write!(f, "Failed"),
            JobState::Retrying => write!(f, "Retrying"),
            JobState::Cancelled => write!(f, "Cancelled"),
            JobState::TimedOut => write!(f, "TimedOut"),
        }
    }
}

/// State transition errors
#[derive(Error, Debug, Clone)]
pub enum StateTransitionError {
    /// Invalid state transition attempted
    #[error("Invalid transition from {from} to {to}: {reason}")]
    InvalidTransition {
        from: String,
        to: String,
        reason: String,
    },

    /// State transition not allowed due to current conditions
    #[error("Transition not allowed: {0}")]
    NotAllowed(String),

    /// Concurrent state modification detected
    #[error("Concurrent state modification detected: {0}")]
    ConcurrentModification(String),
}

/// State transition guard to prevent invalid state changes
///
/// This guard enforces a state machine with predefined valid transitions,
/// preventing race conditions and invalid state changes that could lead to
/// system instability.
///
/// # State Machine Rules
///
/// ## Worker State Transitions
/// - `Idle -> Processing`: Worker picks up a new job
/// - `Idle -> ShuttingDown`: Worker begins graceful shutdown
/// - `Processing -> Completed`: Job processing finished successfully
/// - `Processing -> Failed`: Job processing failed
/// - `Processing -> Paused`: Worker paused during processing
/// - `Processing -> ShuttingDown`: Worker shutdown during processing
/// - `Paused -> Processing`: Worker resumed from pause
/// - `Paused -> Failed`: Worker failed while paused
/// - `Paused -> ShuttingDown`: Worker shutdown while paused
/// - `Failed -> Idle`: Worker recovered from failure
/// - `Failed -> ShuttingDown`: Worker shutdown after failure
/// - `Completed -> Idle`: Worker ready for next job
/// - `Completed -> ShuttingDown`: Worker shutdown after completion
/// - `ShuttingDown -> Terminated`: Graceful shutdown complete
/// - `Failed -> Terminated`: Emergency termination after failure
///
/// ## Job State Transitions
/// - `Pending -> Assigned`: Job assigned to worker
/// - `Pending -> Cancelled`: Job cancelled before assignment
/// - `Assigned -> Processing`: Worker starts processing
/// - `Processing -> Completed`: Processing successful
/// - `Processing -> Failed`: Processing failed
/// - `Processing -> Paused`: Processing paused
/// - `Processing -> TimedOut`: Processing exceeded timeout
/// - `Processing -> Cancelled`: Job cancelled during processing
/// - `Paused -> Processing`: Job resumed
/// - `Paused -> Cancelled`: Job cancelled while paused
/// - `Failed -> Retrying`: Job retry initiated
/// - `Retrying -> Processing`: Retry attempt started
/// - `Retrying -> Failed`: All retries exhausted
///
pub struct StateTransitionGuard {
    /// Valid worker state transitions
    worker_transitions: HashMap<(WorkerState, WorkerState), String>,
    /// Valid job state transitions
    job_transitions: HashMap<(JobState, JobState), String>,
    /// Transition metrics (valid, invalid counts)
    metrics: Arc<parking_lot::RwLock<TransitionMetrics>>,
}

/// Transition metrics for monitoring
#[derive(Debug, Default, Clone, Serialize)]
pub struct TransitionMetrics {
    /// Total valid worker transitions
    pub valid_worker_transitions: u64,
    /// Total invalid worker transitions
    pub invalid_worker_transitions: u64,
    /// Total valid job transitions
    pub valid_job_transitions: u64,
    /// Total invalid job transitions
    pub invalid_job_transitions: u64,
    /// Last invalid transition timestamp
    pub last_invalid_transition: Option<chrono::DateTime<chrono::Utc>>,
}

impl StateTransitionGuard {
    /// Create a new state transition guard with predefined rules
    pub fn new() -> Self {
        let mut guard = Self {
            worker_transitions: HashMap::new(),
            job_transitions: HashMap::new(),
            metrics: Arc::new(parking_lot::RwLock::new(TransitionMetrics::default())),
        };

        // Define valid worker state transitions
        guard.add_worker_transition(
            WorkerState::Idle,
            WorkerState::Processing,
            "Worker picked up new job",
        );
        guard.add_worker_transition(
            WorkerState::Idle,
            WorkerState::ShuttingDown,
            "Worker initiated graceful shutdown",
        );
        guard.add_worker_transition(
            WorkerState::Processing,
            WorkerState::Completed,
            "Job processing completed successfully",
        );
        guard.add_worker_transition(
            WorkerState::Processing,
            WorkerState::Failed,
            "Job processing failed",
        );
        guard.add_worker_transition(
            WorkerState::Processing,
            WorkerState::Paused,
            "Worker paused during processing",
        );
        guard.add_worker_transition(
            WorkerState::Processing,
            WorkerState::ShuttingDown,
            "Worker shutdown requested during processing",
        );
        guard.add_worker_transition(
            WorkerState::Paused,
            WorkerState::Processing,
            "Worker resumed from pause",
        );
        guard.add_worker_transition(
            WorkerState::Paused,
            WorkerState::Failed,
            "Worker failed while paused",
        );
        guard.add_worker_transition(
            WorkerState::Paused,
            WorkerState::ShuttingDown,
            "Worker shutdown while paused",
        );
        guard.add_worker_transition(
            WorkerState::Failed,
            WorkerState::Idle,
            "Worker recovered from failure",
        );
        guard.add_worker_transition(
            WorkerState::Failed,
            WorkerState::ShuttingDown,
            "Worker shutdown after failure",
        );
        guard.add_worker_transition(
            WorkerState::Failed,
            WorkerState::Terminated,
            "Emergency termination after failure",
        );
        guard.add_worker_transition(
            WorkerState::Completed,
            WorkerState::Idle,
            "Worker ready for next job",
        );
        guard.add_worker_transition(
            WorkerState::Completed,
            WorkerState::ShuttingDown,
            "Worker shutdown after completion",
        );
        guard.add_worker_transition(
            WorkerState::ShuttingDown,
            WorkerState::Terminated,
            "Graceful shutdown completed",
        );

        // Define valid job state transitions
        guard.add_job_transition(
            JobState::Pending,
            JobState::Assigned,
            "Job assigned to worker",
        );
        guard.add_job_transition(
            JobState::Pending,
            JobState::Cancelled,
            "Job cancelled before assignment",
        );
        guard.add_job_transition(
            JobState::Assigned,
            JobState::Processing,
            "Worker started processing job",
        );
        guard.add_job_transition(
            JobState::Processing,
            JobState::Completed,
            "Job processing successful",
        );
        guard.add_job_transition(
            JobState::Processing,
            JobState::Failed,
            "Job processing failed",
        );
        guard.add_job_transition(
            JobState::Processing,
            JobState::Paused,
            "Job processing paused",
        );
        guard.add_job_transition(
            JobState::Processing,
            JobState::TimedOut,
            "Job processing exceeded timeout",
        );
        guard.add_job_transition(
            JobState::Processing,
            JobState::Cancelled,
            "Job cancelled during processing",
        );
        guard.add_job_transition(JobState::Paused, JobState::Processing, "Job resumed");
        guard.add_job_transition(
            JobState::Paused,
            JobState::Cancelled,
            "Job cancelled while paused",
        );
        guard.add_job_transition(JobState::Failed, JobState::Retrying, "Job retry initiated");
        guard.add_job_transition(
            JobState::Retrying,
            JobState::Processing,
            "Retry attempt started",
        );
        guard.add_job_transition(
            JobState::Retrying,
            JobState::Failed,
            "All retry attempts exhausted",
        );

        guard
    }

    /// Add a valid worker state transition
    fn add_worker_transition(&mut self, from: WorkerState, to: WorkerState, reason: &str) {
        self.worker_transitions
            .insert((from, to), reason.to_string());
    }

    /// Add a valid job state transition
    fn add_job_transition(&mut self, from: JobState, to: JobState, reason: &str) {
        self.job_transitions.insert((from, to), reason.to_string());
    }

    /// Validate and execute worker state transition
    ///
    /// # Arguments
    ///
    /// * `from` - Current worker state
    /// * `to` - Desired target state
    ///
    /// # Returns
    ///
    /// `Ok(())` if transition is valid, `Err(StateTransitionError)` if invalid
    ///
    /// # Examples
    ///
    /// ```
    /// use riptide_workers::state::{StateTransitionGuard, WorkerState};
    ///
    /// let guard = StateTransitionGuard::new();
    /// assert!(guard.can_transition_worker(WorkerState::Idle, WorkerState::Processing).is_ok());
    /// assert!(guard.can_transition_worker(WorkerState::Idle, WorkerState::Completed).is_err());
    /// ```
    pub fn can_transition_worker(
        &self,
        from: WorkerState,
        to: WorkerState,
    ) -> Result<(), StateTransitionError> {
        // Allow self-transitions (no-op)
        if from == to {
            debug!(
                from = %from,
                to = %to,
                "Self-transition (no-op)"
            );
            return Ok(());
        }

        if let Some(reason) = self.worker_transitions.get(&(from, to)) {
            // Valid transition
            let mut metrics = self.metrics.write();
            metrics.valid_worker_transitions += 1;

            debug!(
                from = %from,
                to = %to,
                reason = %reason,
                "Valid worker state transition"
            );

            Ok(())
        } else {
            // Invalid transition
            let mut metrics = self.metrics.write();
            metrics.invalid_worker_transitions += 1;
            metrics.last_invalid_transition = Some(chrono::Utc::now());

            let error = StateTransitionError::InvalidTransition {
                from: from.to_string(),
                to: to.to_string(),
                reason: format!(
                    "No valid transition path from {} to {}. Check state machine rules.",
                    from, to
                ),
            };

            warn!(
                from = %from,
                to = %to,
                error = %error,
                "Invalid worker state transition blocked"
            );

            Err(error)
        }
    }

    /// Validate and execute job state transition
    ///
    /// # Arguments
    ///
    /// * `from` - Current job state
    /// * `to` - Desired target state
    ///
    /// # Returns
    ///
    /// `Ok(())` if transition is valid, `Err(StateTransitionError)` if invalid
    pub fn can_transition_job(
        &self,
        from: JobState,
        to: JobState,
    ) -> Result<(), StateTransitionError> {
        // Allow self-transitions (no-op)
        if from == to {
            debug!(
                from = %from,
                to = %to,
                "Self-transition (no-op)"
            );
            return Ok(());
        }

        if let Some(reason) = self.job_transitions.get(&(from, to)) {
            // Valid transition
            let mut metrics = self.metrics.write();
            metrics.valid_job_transitions += 1;

            debug!(
                from = %from,
                to = %to,
                reason = %reason,
                "Valid job state transition"
            );

            Ok(())
        } else {
            // Invalid transition
            let mut metrics = self.metrics.write();
            metrics.invalid_job_transitions += 1;
            metrics.last_invalid_transition = Some(chrono::Utc::now());

            let error = StateTransitionError::InvalidTransition {
                from: from.to_string(),
                to: to.to_string(),
                reason: format!(
                    "No valid transition path from {} to {}. Check state machine rules.",
                    from, to
                ),
            };

            warn!(
                from = %from,
                to = %to,
                error = %error,
                "Invalid job state transition blocked"
            );

            Err(error)
        }
    }

    /// Get current transition metrics
    pub fn get_metrics(&self) -> TransitionMetrics {
        self.metrics.read().clone()
    }

    /// Reset transition metrics
    #[cfg(test)]
    pub fn reset_metrics(&self) {
        let mut metrics = self.metrics.write();
        *metrics = TransitionMetrics::default();
    }
}

impl Default for StateTransitionGuard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_worker_transitions() {
        let guard = StateTransitionGuard::new();

        // Test valid transitions
        assert!(guard
            .can_transition_worker(WorkerState::Idle, WorkerState::Processing)
            .is_ok());
        assert!(guard
            .can_transition_worker(WorkerState::Processing, WorkerState::Completed)
            .is_ok());
        assert!(guard
            .can_transition_worker(WorkerState::Completed, WorkerState::Idle)
            .is_ok());
        assert!(guard
            .can_transition_worker(WorkerState::Processing, WorkerState::Failed)
            .is_ok());
        assert!(guard
            .can_transition_worker(WorkerState::Failed, WorkerState::Idle)
            .is_ok());
    }

    #[test]
    fn test_invalid_worker_transitions() {
        let guard = StateTransitionGuard::new();

        // Test invalid transitions
        assert!(guard
            .can_transition_worker(WorkerState::Idle, WorkerState::Completed)
            .is_err());
        assert!(guard
            .can_transition_worker(WorkerState::Idle, WorkerState::Failed)
            .is_err());
        assert!(guard
            .can_transition_worker(WorkerState::Terminated, WorkerState::Idle)
            .is_err());
        assert!(guard
            .can_transition_worker(WorkerState::Completed, WorkerState::Processing)
            .is_err());
    }

    #[test]
    fn test_self_transitions_allowed() {
        let guard = StateTransitionGuard::new();

        // Self-transitions should be allowed (no-op)
        assert!(guard
            .can_transition_worker(WorkerState::Idle, WorkerState::Idle)
            .is_ok());
        assert!(guard
            .can_transition_worker(WorkerState::Processing, WorkerState::Processing)
            .is_ok());
    }

    #[test]
    fn test_valid_job_transitions() {
        let guard = StateTransitionGuard::new();

        // Test valid job transitions
        assert!(guard
            .can_transition_job(JobState::Pending, JobState::Assigned)
            .is_ok());
        assert!(guard
            .can_transition_job(JobState::Assigned, JobState::Processing)
            .is_ok());
        assert!(guard
            .can_transition_job(JobState::Processing, JobState::Completed)
            .is_ok());
        assert!(guard
            .can_transition_job(JobState::Processing, JobState::Failed)
            .is_ok());
        assert!(guard
            .can_transition_job(JobState::Failed, JobState::Retrying)
            .is_ok());
    }

    #[test]
    fn test_invalid_job_transitions() {
        let guard = StateTransitionGuard::new();

        // Test invalid job transitions
        assert!(guard
            .can_transition_job(JobState::Pending, JobState::Completed)
            .is_err());
        assert!(guard
            .can_transition_job(JobState::Completed, JobState::Pending)
            .is_err());
        assert!(guard
            .can_transition_job(JobState::Cancelled, JobState::Processing)
            .is_err());
    }

    #[test]
    fn test_transition_metrics_tracking() {
        let guard = StateTransitionGuard::new();
        guard.reset_metrics();

        // Valid transitions should increment valid counter
        let _ = guard.can_transition_worker(WorkerState::Idle, WorkerState::Processing);
        let _ = guard.can_transition_worker(WorkerState::Processing, WorkerState::Completed);

        let metrics = guard.get_metrics();
        assert_eq!(metrics.valid_worker_transitions, 2);
        assert_eq!(metrics.invalid_worker_transitions, 0);

        // Invalid transitions should increment invalid counter
        let _ = guard.can_transition_worker(WorkerState::Idle, WorkerState::Completed);

        let metrics = guard.get_metrics();
        assert_eq!(metrics.valid_worker_transitions, 2);
        assert_eq!(metrics.invalid_worker_transitions, 1);
        assert!(metrics.last_invalid_transition.is_some());
    }

    #[test]
    fn test_error_messages_descriptive() {
        let guard = StateTransitionGuard::new();

        let result = guard.can_transition_worker(WorkerState::Idle, WorkerState::Failed);
        assert!(result.is_err());

        if let Err(StateTransitionError::InvalidTransition { from, to, reason }) = result {
            assert_eq!(from, "Idle");
            assert_eq!(to, "Failed");
            assert!(reason.contains("No valid transition path"));
        } else {
            panic!("Expected InvalidTransition error");
        }
    }

    #[test]
    fn test_concurrent_transitions() {
        use std::sync::Arc;
        use std::thread;

        let guard = Arc::new(StateTransitionGuard::new());
        let mut handles = vec![];

        // Spawn multiple threads attempting concurrent transitions
        for _ in 0..10 {
            let guard_clone = Arc::clone(&guard);
            let handle = thread::spawn(move || {
                let _ =
                    guard_clone.can_transition_worker(WorkerState::Idle, WorkerState::Processing);
                let _ = guard_clone
                    .can_transition_worker(WorkerState::Processing, WorkerState::Completed);
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify metrics are correctly tracked
        let metrics = guard.get_metrics();
        assert_eq!(metrics.valid_worker_transitions, 20); // 10 threads * 2 transitions
    }

    #[test]
    fn test_shutdown_transitions() {
        let guard = StateTransitionGuard::new();

        // Test graceful shutdown paths
        assert!(guard
            .can_transition_worker(WorkerState::Idle, WorkerState::ShuttingDown)
            .is_ok());
        assert!(guard
            .can_transition_worker(WorkerState::Processing, WorkerState::ShuttingDown)
            .is_ok());
        assert!(guard
            .can_transition_worker(WorkerState::ShuttingDown, WorkerState::Terminated)
            .is_ok());

        // Emergency termination from failed state
        assert!(guard
            .can_transition_worker(WorkerState::Failed, WorkerState::Terminated)
            .is_ok());
    }

    #[test]
    fn test_pause_resume_transitions() {
        let guard = StateTransitionGuard::new();

        // Test pause/resume workflow
        assert!(guard
            .can_transition_worker(WorkerState::Processing, WorkerState::Paused)
            .is_ok());
        assert!(guard
            .can_transition_worker(WorkerState::Paused, WorkerState::Processing)
            .is_ok());

        // Test job pause/resume
        assert!(guard
            .can_transition_job(JobState::Processing, JobState::Paused)
            .is_ok());
        assert!(guard
            .can_transition_job(JobState::Paused, JobState::Processing)
            .is_ok());
    }

    #[test]
    fn test_retry_workflow() {
        let guard = StateTransitionGuard::new();

        // Test complete retry workflow
        assert!(guard
            .can_transition_job(JobState::Processing, JobState::Failed)
            .is_ok());
        assert!(guard
            .can_transition_job(JobState::Failed, JobState::Retrying)
            .is_ok());
        assert!(guard
            .can_transition_job(JobState::Retrying, JobState::Processing)
            .is_ok());

        // Retry exhaustion path
        assert!(guard
            .can_transition_job(JobState::Retrying, JobState::Failed)
            .is_ok());
    }
}
