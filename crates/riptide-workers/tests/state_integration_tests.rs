/*!
# State Transition Guard Integration Tests

Comprehensive integration tests for StateTransitionGuard to verify
thread safety, concurrent transitions, and state machine integrity.
*/

use riptide_workers::state::{JobState, StateTransitionError, StateTransitionGuard, WorkerState};
use std::sync::Arc;
use std::thread;

#[test]
fn test_worker_lifecycle_happy_path() {
    let guard = StateTransitionGuard::new();

    // Complete worker lifecycle
    assert!(guard
        .can_transition_worker(WorkerState::Idle, WorkerState::Processing)
        .is_ok());
    assert!(guard
        .can_transition_worker(WorkerState::Processing, WorkerState::Completed)
        .is_ok());
    assert!(guard
        .can_transition_worker(WorkerState::Completed, WorkerState::Idle)
        .is_ok());
}

#[test]
fn test_worker_failure_recovery_path() {
    let guard = StateTransitionGuard::new();

    // Worker fails and recovers
    assert!(guard
        .can_transition_worker(WorkerState::Idle, WorkerState::Processing)
        .is_ok());
    assert!(guard
        .can_transition_worker(WorkerState::Processing, WorkerState::Failed)
        .is_ok());
    assert!(guard
        .can_transition_worker(WorkerState::Failed, WorkerState::Idle)
        .is_ok());
}

#[test]
fn test_worker_graceful_shutdown() {
    let guard = StateTransitionGuard::new();

    // Graceful shutdown paths
    assert!(guard
        .can_transition_worker(WorkerState::Idle, WorkerState::ShuttingDown)
        .is_ok());
    assert!(guard
        .can_transition_worker(WorkerState::ShuttingDown, WorkerState::Terminated)
        .is_ok());

    // Shutdown during processing
    assert!(guard
        .can_transition_worker(WorkerState::Processing, WorkerState::ShuttingDown)
        .is_ok());
}

#[test]
fn test_worker_invalid_transitions_blocked() {
    let guard = StateTransitionGuard::new();

    // These transitions should not be allowed
    let invalid_transitions = vec![
        (WorkerState::Idle, WorkerState::Completed),
        (WorkerState::Idle, WorkerState::Failed),
        (WorkerState::Terminated, WorkerState::Idle),
        (WorkerState::Completed, WorkerState::Processing),
        (WorkerState::Terminated, WorkerState::Processing),
    ];

    for (from, to) in invalid_transitions {
        assert!(
            guard.can_transition_worker(from, to).is_err(),
            "Expected transition from {:?} to {:?} to be invalid",
            from,
            to
        );
    }
}

#[test]
fn test_job_lifecycle_happy_path() {
    let guard = StateTransitionGuard::new();

    // Complete job lifecycle
    assert!(guard
        .can_transition_job(JobState::Pending, JobState::Assigned)
        .is_ok());
    assert!(guard
        .can_transition_job(JobState::Assigned, JobState::Processing)
        .is_ok());
    assert!(guard
        .can_transition_job(JobState::Processing, JobState::Completed)
        .is_ok());
}

#[test]
fn test_job_retry_workflow() {
    let guard = StateTransitionGuard::new();

    // Job retry workflow
    assert!(guard
        .can_transition_job(JobState::Processing, JobState::Failed)
        .is_ok());
    assert!(guard
        .can_transition_job(JobState::Failed, JobState::Retrying)
        .is_ok());
    assert!(guard
        .can_transition_job(JobState::Retrying, JobState::Processing)
        .is_ok());

    // Retry exhaustion
    assert!(guard
        .can_transition_job(JobState::Retrying, JobState::Failed)
        .is_ok());
}

#[test]
fn test_job_pause_resume() {
    let guard = StateTransitionGuard::new();

    // Job pause/resume
    assert!(guard
        .can_transition_job(JobState::Processing, JobState::Paused)
        .is_ok());
    assert!(guard
        .can_transition_job(JobState::Paused, JobState::Processing)
        .is_ok());
    assert!(guard
        .can_transition_job(JobState::Paused, JobState::Cancelled)
        .is_ok());
}

#[test]
fn test_job_cancellation_paths() {
    let guard = StateTransitionGuard::new();

    // Various cancellation paths
    assert!(guard
        .can_transition_job(JobState::Pending, JobState::Cancelled)
        .is_ok());
    assert!(guard
        .can_transition_job(JobState::Processing, JobState::Cancelled)
        .is_ok());
    assert!(guard
        .can_transition_job(JobState::Paused, JobState::Cancelled)
        .is_ok());

    // Cannot restart cancelled jobs
    assert!(guard
        .can_transition_job(JobState::Cancelled, JobState::Processing)
        .is_err());
}

#[test]
fn test_concurrent_state_transitions() {
    let guard = Arc::new(StateTransitionGuard::new());
    let mut handles = vec![];

    // Spawn 100 threads doing concurrent transitions
    for i in 0..100 {
        let guard_clone = Arc::clone(&guard);
        let handle = thread::spawn(move || {
            // Each thread performs multiple transitions
            let _ = guard_clone.can_transition_worker(WorkerState::Idle, WorkerState::Processing);
            let _ =
                guard_clone.can_transition_worker(WorkerState::Processing, WorkerState::Completed);
            let _ = guard_clone.can_transition_worker(WorkerState::Completed, WorkerState::Idle);

            // Some invalid transitions
            if i % 10 == 0 {
                let _ = guard_clone.can_transition_worker(WorkerState::Idle, WorkerState::Failed);
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify metrics are correctly tracked
    let metrics = guard.get_metrics();
    assert_eq!(metrics.valid_worker_transitions, 300); // 100 threads * 3 valid transitions
    assert_eq!(metrics.invalid_worker_transitions, 10); // 10 invalid transitions
}

#[test]
fn test_error_message_quality() {
    let guard = StateTransitionGuard::new();

    let result = guard.can_transition_worker(WorkerState::Idle, WorkerState::Completed);
    assert!(result.is_err());

    match result {
        Err(StateTransitionError::InvalidTransition { from, to, reason }) => {
            assert_eq!(from, "Idle");
            assert_eq!(to, "Completed");
            assert!(reason.contains("No valid transition path"));
            assert!(reason.contains("Idle"));
            assert!(reason.contains("Completed"));
        }
        _ => panic!("Expected InvalidTransition error"),
    }
}

#[test]
fn test_metrics_tracking_accuracy() {
    let guard = StateTransitionGuard::new();

    // Clear metrics
    let _ = guard.get_metrics();

    // Perform known transitions
    for _ in 0..5 {
        let _ = guard.can_transition_worker(WorkerState::Idle, WorkerState::Processing);
    }

    for _ in 0..3 {
        let _ = guard.can_transition_worker(WorkerState::Idle, WorkerState::Completed);
    }

    let metrics = guard.get_metrics();
    assert!(metrics.valid_worker_transitions >= 5);
    assert!(metrics.invalid_worker_transitions >= 3);
    assert!(metrics.last_invalid_transition.is_some());
}

#[test]
fn test_self_transitions_are_noops() {
    let guard = StateTransitionGuard::new();

    // Self-transitions should always succeed
    assert!(guard
        .can_transition_worker(WorkerState::Idle, WorkerState::Idle)
        .is_ok());
    assert!(guard
        .can_transition_worker(WorkerState::Processing, WorkerState::Processing)
        .is_ok());
    assert!(guard
        .can_transition_job(JobState::Pending, JobState::Pending)
        .is_ok());
}

#[test]
fn test_worker_pause_resume_workflow() {
    let guard = StateTransitionGuard::new();

    // Worker can be paused during processing
    assert!(guard
        .can_transition_worker(WorkerState::Processing, WorkerState::Paused)
        .is_ok());

    // Worker can resume from pause
    assert!(guard
        .can_transition_worker(WorkerState::Paused, WorkerState::Processing)
        .is_ok());

    // Worker can fail while paused
    assert!(guard
        .can_transition_worker(WorkerState::Paused, WorkerState::Failed)
        .is_ok());

    // Worker can shutdown while paused
    assert!(guard
        .can_transition_worker(WorkerState::Paused, WorkerState::ShuttingDown)
        .is_ok());
}

#[test]
fn test_emergency_termination() {
    let guard = StateTransitionGuard::new();

    // Emergency termination from failed state
    assert!(guard
        .can_transition_worker(WorkerState::Failed, WorkerState::Terminated)
        .is_ok());

    // Normal termination through shutdown
    assert!(guard
        .can_transition_worker(WorkerState::ShuttingDown, WorkerState::Terminated)
        .is_ok());
}

#[test]
fn test_job_timeout_handling() {
    let guard = StateTransitionGuard::new();

    // Job can timeout during processing
    assert!(guard
        .can_transition_job(JobState::Processing, JobState::TimedOut)
        .is_ok());

    // TimedOut jobs cannot be resumed (invalid transition)
    assert!(guard
        .can_transition_job(JobState::TimedOut, JobState::Processing)
        .is_err());
}
