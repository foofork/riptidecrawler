#![allow(dead_code, unused_imports)]

/// Local job management module for RipTide CLI
///
/// This module provides local job scheduling, tracking, and management
/// without requiring an external API server.
///
/// NOTE: Phase 1 Migration - This module is being transitioned to use riptide-workers library.
/// Local types in types.rs are deprecated in favor of riptide-workers types.
pub mod manager;
pub mod storage;
pub mod types;

pub use manager::JobManager;

// Re-export types from riptide-workers for CLI usage
pub use riptide_workers::{
    Job as WorkerJob, JobPriority as WorkerJobPriority, JobStatus as WorkerJobStatus,
};

// Keep legacy types for backward compatibility during migration
pub use types::{Job, JobPriority, JobStatus, LogLevel};
