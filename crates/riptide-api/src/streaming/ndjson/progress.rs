//! Progress tracking for NDJSON streaming operations.
//!
//! This module contains progress tracking structures and utilities.

use serde::Serialize;
use std::time::Instant;

/// Progress tracking for long-running operations
#[derive(Serialize, Debug)]
pub struct OperationProgress {
    pub operation_id: String,
    pub operation_type: String,
    pub started_at: String,
    pub current_phase: String,
    pub progress_percentage: f64,
    pub items_completed: usize,
    pub items_total: usize,
    pub estimated_completion: Option<String>,
    pub current_item: Option<String>,
}

/// Estimate completion time for long-running operations
pub fn estimate_completion(start_time: Instant, completed: usize, total: usize) -> Option<String> {
    if completed == 0 || total == 0 {
        return None;
    }

    let elapsed = start_time.elapsed();
    let avg_time_per_item = elapsed.as_secs_f64() / completed as f64;
    let remaining_items = total.saturating_sub(completed);
    let estimated_remaining_secs = avg_time_per_item * remaining_items as f64;

    let completion_time =
        chrono::Utc::now() + chrono::Duration::seconds(estimated_remaining_secs as i64);
    Some(completion_time.to_rfc3339())
}