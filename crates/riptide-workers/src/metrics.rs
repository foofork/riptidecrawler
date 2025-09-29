use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Worker metrics collector
pub struct WorkerMetrics {
    /// Job processing counters
    pub jobs_submitted: AtomicU64,
    pub jobs_completed: AtomicU64,
    pub jobs_failed: AtomicU64,
    pub jobs_retried: AtomicU64,
    pub jobs_dead_letter: AtomicU64,

    /// Processing time statistics
    processing_times: Arc<RwLock<Vec<u64>>>,

    /// Queue size tracking
    queue_sizes: Arc<RwLock<HashMap<String, u64>>>,

    /// Worker health status
    worker_health: Arc<RwLock<HashMap<String, WorkerHealthStatus>>>,

    /// Start time for uptime calculation
    started_at: DateTime<Utc>,

    /// Job type statistics
    job_type_stats: Arc<RwLock<HashMap<String, JobTypeStats>>>,
}

impl WorkerMetrics {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            jobs_submitted: AtomicU64::new(0),
            jobs_completed: AtomicU64::new(0),
            jobs_failed: AtomicU64::new(0),
            jobs_retried: AtomicU64::new(0),
            jobs_dead_letter: AtomicU64::new(0),
            processing_times: Arc::new(RwLock::new(Vec::new())),
            queue_sizes: Arc::new(RwLock::new(HashMap::new())),
            worker_health: Arc::new(RwLock::new(HashMap::new())),
            started_at: Utc::now(),
            job_type_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record job submission
    pub fn record_job_submitted(&self, job_type: &str) {
        self.jobs_submitted.fetch_add(1, Ordering::Relaxed);

        tokio::spawn({
            let job_type = job_type.to_string();
            let job_type_stats = Arc::clone(&self.job_type_stats);
            async move {
                let mut stats = job_type_stats.write().await;
                let entry = stats.entry(job_type).or_insert_with(Default::default);
                entry.submitted += 1;
            }
        });
    }

    /// Record job completion
    pub fn record_job_completed(&self, job_type: &str, processing_time_ms: u64) {
        self.jobs_completed.fetch_add(1, Ordering::Relaxed);

        tokio::spawn({
            let job_type = job_type.to_string();
            let processing_times = Arc::clone(&self.processing_times);
            let job_type_stats = Arc::clone(&self.job_type_stats);
            async move {
                // Update processing times
                {
                    let mut times = processing_times.write().await;
                    times.push(processing_time_ms);
                    // Keep only last 1000 measurements
                    if times.len() > 1000 {
                        times.remove(0);
                    }
                }

                // Update job type stats
                {
                    let mut stats = job_type_stats.write().await;
                    let entry = stats.entry(job_type).or_insert_with(Default::default);
                    entry.completed += 1;
                    entry.total_processing_time_ms += processing_time_ms;
                }
            }
        });
    }

    /// Record job failure
    pub fn record_job_failed(&self, job_type: &str) {
        self.jobs_failed.fetch_add(1, Ordering::Relaxed);

        tokio::spawn({
            let job_type = job_type.to_string();
            let job_type_stats = Arc::clone(&self.job_type_stats);
            async move {
                let mut stats = job_type_stats.write().await;
                let entry = stats.entry(job_type).or_insert_with(Default::default);
                entry.failed += 1;
            }
        });
    }

    /// Record job retry
    pub fn record_job_retried(&self, job_type: &str) {
        self.jobs_retried.fetch_add(1, Ordering::Relaxed);

        tokio::spawn({
            let job_type = job_type.to_string();
            let job_type_stats = Arc::clone(&self.job_type_stats);
            async move {
                let mut stats = job_type_stats.write().await;
                let entry = stats.entry(job_type).or_insert_with(Default::default);
                entry.retried += 1;
            }
        });
    }

    /// Record job moved to dead letter queue
    pub fn record_job_dead_letter(&self, job_type: &str) {
        self.jobs_dead_letter.fetch_add(1, Ordering::Relaxed);

        tokio::spawn({
            let job_type = job_type.to_string();
            let job_type_stats = Arc::clone(&self.job_type_stats);
            async move {
                let mut stats = job_type_stats.write().await;
                let entry = stats.entry(job_type).or_insert_with(Default::default);
                entry.dead_letter += 1;
            }
        });
    }

    /// Update queue size for a specific queue
    pub async fn update_queue_size(&self, queue_name: &str, size: u64) {
        let mut sizes = self.queue_sizes.write().await;
        sizes.insert(queue_name.to_string(), size);
    }

    /// Update worker health status
    pub async fn update_worker_health(&self, worker_id: &str, status: WorkerHealthStatus) {
        let mut health = self.worker_health.write().await;
        health.insert(worker_id.to_string(), status);
    }

    /// Get current metrics snapshot
    pub async fn get_snapshot(&self) -> WorkerMetricsSnapshot {
        let jobs_submitted = self.jobs_submitted.load(Ordering::Relaxed);
        let jobs_completed = self.jobs_completed.load(Ordering::Relaxed);
        let jobs_failed = self.jobs_failed.load(Ordering::Relaxed);
        let jobs_retried = self.jobs_retried.load(Ordering::Relaxed);
        let jobs_dead_letter = self.jobs_dead_letter.load(Ordering::Relaxed);

        let processing_times = self.processing_times.read().await.clone();
        let queue_sizes = self.queue_sizes.read().await.clone();
        let worker_health = self.worker_health.read().await;
        let job_type_stats = self.job_type_stats.read().await.clone();

        // Calculate processing time statistics
        let (avg_processing_time, p95_processing_time, p99_processing_time) =
            if !processing_times.is_empty() {
                let mut sorted_times = processing_times.clone();
                sorted_times.sort_unstable();

                let avg = processing_times.iter().sum::<u64>() / processing_times.len() as u64;
                let p95_idx = (processing_times.len() as f64 * 0.95) as usize;
                let p99_idx = (processing_times.len() as f64 * 0.99) as usize;

                let p95 = sorted_times.get(p95_idx.saturating_sub(1)).copied().unwrap_or(0);
                let p99 = sorted_times.get(p99_idx.saturating_sub(1)).copied().unwrap_or(0);

                (avg, p95, p99)
            } else {
                (0, 0, 0)
            };

        let uptime_seconds = (Utc::now() - self.started_at).num_seconds() as u64;

        // Calculate success rate
        let success_rate = if jobs_submitted > 0 {
            (jobs_completed as f64 / jobs_submitted as f64) * 100.0
        } else {
            0.0
        };

        // Count healthy workers
        let healthy_workers = worker_health.values()
            .filter(|status| status.is_healthy)
            .count();

        WorkerMetricsSnapshot {
            jobs_submitted,
            jobs_completed,
            jobs_failed,
            jobs_retried,
            jobs_dead_letter,
            avg_processing_time_ms: avg_processing_time,
            p95_processing_time_ms: p95_processing_time,
            p99_processing_time_ms: p99_processing_time,
            queue_sizes,
            worker_health: worker_health.clone(),
            job_type_stats,
            uptime_seconds,
            success_rate,
            total_workers: worker_health.len(),
            healthy_workers,
            timestamp: Utc::now(),
        }
    }

    /// Get jobs per second rate
    pub fn get_jobs_per_second(&self) -> f64 {
        let uptime = (Utc::now() - self.started_at).num_seconds();
        if uptime > 0 {
            self.jobs_completed.load(Ordering::Relaxed) as f64 / uptime as f64
        } else {
            0.0
        }
    }

    /// Get current queue depth total
    pub async fn get_total_queue_depth(&self) -> u64 {
        let sizes = self.queue_sizes.read().await;
        sizes.values().sum()
    }

    /// Reset metrics (useful for testing)
    pub async fn reset(&self) {
        self.jobs_submitted.store(0, Ordering::Relaxed);
        self.jobs_completed.store(0, Ordering::Relaxed);
        self.jobs_failed.store(0, Ordering::Relaxed);
        self.jobs_retried.store(0, Ordering::Relaxed);
        self.jobs_dead_letter.store(0, Ordering::Relaxed);

        let mut processing_times = self.processing_times.write().await;
        processing_times.clear();

        let mut queue_sizes = self.queue_sizes.write().await;
        queue_sizes.clear();

        let mut worker_health = self.worker_health.write().await;
        worker_health.clear();

        let mut job_type_stats = self.job_type_stats.write().await;
        job_type_stats.clear();
    }
}

impl Default for WorkerMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Worker health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerHealthStatus {
    pub worker_id: String,
    pub is_healthy: bool,
    pub last_heartbeat: DateTime<Utc>,
    pub jobs_processed: u64,
    pub jobs_failed: u64,
    pub current_job: Option<Uuid>,
    pub avg_processing_time_ms: u64,
}

/// Statistics for a specific job type
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JobTypeStats {
    pub submitted: u64,
    pub completed: u64,
    pub failed: u64,
    pub retried: u64,
    pub dead_letter: u64,
    pub total_processing_time_ms: u64,
}

impl JobTypeStats {
    /// Get average processing time
    pub fn avg_processing_time_ms(&self) -> u64 {
        if self.completed > 0 {
            self.total_processing_time_ms / self.completed
        } else {
            0
        }
    }

    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.submitted > 0 {
            (self.completed as f64 / self.submitted as f64) * 100.0
        } else {
            0.0
        }
    }
}

/// Snapshot of worker metrics at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerMetricsSnapshot {
    /// Job counters
    pub jobs_submitted: u64,
    pub jobs_completed: u64,
    pub jobs_failed: u64,
    pub jobs_retried: u64,
    pub jobs_dead_letter: u64,

    /// Processing time statistics
    pub avg_processing_time_ms: u64,
    pub p95_processing_time_ms: u64,
    pub p99_processing_time_ms: u64,

    /// Queue information
    pub queue_sizes: HashMap<String, u64>,

    /// Worker health information
    pub worker_health: HashMap<String, WorkerHealthStatus>,

    /// Job type statistics
    pub job_type_stats: HashMap<String, JobTypeStats>,

    /// System information
    pub uptime_seconds: u64,
    pub success_rate: f64,
    pub total_workers: usize,
    pub healthy_workers: usize,
    pub timestamp: DateTime<Utc>,
}

impl WorkerMetricsSnapshot {
    /// Get total queue depth across all queues
    pub fn total_queue_depth(&self) -> u64 {
        self.queue_sizes.values().sum()
    }

    /// Get jobs per second rate
    pub fn jobs_per_second(&self) -> f64 {
        if self.uptime_seconds > 0 {
            self.jobs_completed as f64 / self.uptime_seconds as f64
        } else {
            0.0
        }
    }

    /// Get failure rate as percentage
    pub fn failure_rate(&self) -> f64 {
        if self.jobs_submitted > 0 {
            (self.jobs_failed as f64 / self.jobs_submitted as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Check if the worker system is healthy
    pub fn is_system_healthy(&self) -> bool {
        // System is healthy if:
        // 1. At least one worker is healthy
        // 2. Success rate is above 90%
        // 3. No queues are severely backed up (>1000 items)

        let has_healthy_workers = self.healthy_workers > 0;
        let good_success_rate = self.success_rate >= 90.0;
        let reasonable_queue_sizes = self.queue_sizes.values().all(|&size| size < 1000);

        has_healthy_workers && good_success_rate && reasonable_queue_sizes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_creation() {
        let metrics = WorkerMetrics::new();
        let snapshot = metrics.get_snapshot().await;

        assert_eq!(snapshot.jobs_submitted, 0);
        assert_eq!(snapshot.jobs_completed, 0);
        assert_eq!(snapshot.total_workers, 0);
    }

    #[tokio::test]
    async fn test_job_recording() {
        let metrics = WorkerMetrics::new();

        metrics.record_job_submitted("test_job");
        metrics.record_job_completed("test_job", 100);

        let snapshot = metrics.get_snapshot().await;
        assert_eq!(snapshot.jobs_submitted, 1);
        assert_eq!(snapshot.jobs_completed, 1);
        assert!(snapshot.job_type_stats.contains_key("test_job"));
    }

    #[tokio::test]
    async fn test_queue_size_tracking() {
        let metrics = WorkerMetrics::new();

        metrics.update_queue_size("pending", 50).await;
        metrics.update_queue_size("processing", 10).await;

        let snapshot = metrics.get_snapshot().await;
        assert_eq!(snapshot.total_queue_depth(), 60);
    }

    #[test]
    fn test_job_type_stats() {
        let stats = JobTypeStats {
            submitted: 100,
            completed: 85,
            failed: 15,
            total_processing_time_ms: 8500,
            ..Default::default()
        };

        assert_eq!(stats.avg_processing_time_ms(), 100);
        assert_eq!(stats.success_rate(), 85.0);
    }
}