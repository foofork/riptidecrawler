/// Local job management module for RipTide CLI
///
/// This module provides local job scheduling, tracking, and management
/// without requiring an external API server.
pub mod manager;
pub mod storage;
pub mod types;

pub use manager::JobManager;
pub use storage::JobStorage;
pub use types::{Job, JobId, JobPriority, JobProgress, JobStatus, LogEntry, LogLevel};
