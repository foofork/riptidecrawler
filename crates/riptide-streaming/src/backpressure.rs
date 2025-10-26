//! Backpressure handling for streaming operations
//!
//! This module provides sophisticated backpressure control to prevent memory
//! exhaustion and ensure smooth streaming performance under varying load conditions.

use crate::{StreamingError, StreamingResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tokio::time::interval;
use uuid::Uuid;

/// Backpressure control configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpressureConfig {
    /// Maximum number of items in flight per stream
    pub max_in_flight: usize,
    /// Maximum memory usage in bytes
    pub max_memory_bytes: u64,
    /// Maximum total items across all streams
    pub max_total_items: usize,
    /// Backpressure activation threshold (0.0 - 1.0)
    pub activation_threshold: f64,
    /// Recovery threshold (0.0 - 1.0)
    pub recovery_threshold: f64,
    /// Check interval for resource monitoring
    pub check_interval: Duration,
    /// Enable adaptive backpressure
    pub adaptive: bool,
}

impl Default for BackpressureConfig {
    fn default() -> Self {
        Self {
            max_in_flight: 1000,
            max_memory_bytes: 100 * 1024 * 1024, // 100 MB
            max_total_items: 10000,
            activation_threshold: 0.8,
            recovery_threshold: 0.6,
            check_interval: Duration::from_millis(500),
            adaptive: true,
        }
    }
}

/// Backpressure status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackpressureStatus {
    Normal,
    Warning,
    Critical,
    Throttled,
}

/// Stream resource usage
#[derive(Debug, Clone)]
struct StreamResources {
    in_flight_items: usize,
    memory_usage: u64,
    last_activity: Instant,
    throttle_until: Option<Instant>,
}

/// Backpressure metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpressureMetrics {
    pub total_streams: usize,
    pub total_in_flight: usize,
    pub total_memory_usage: u64,
    pub status: BackpressureStatus,
    pub throttled_streams: usize,
    pub rejection_rate: f64,
    pub average_wait_time: Duration,
}

/// Backpressure controller for managing resource usage
#[derive(Debug)]
pub struct BackpressureController {
    config: BackpressureConfig,
    stream_resources: Arc<RwLock<HashMap<Uuid, StreamResources>>>,
    global_semaphore: Arc<Semaphore>,
    memory_semaphore: Arc<Semaphore>,
    metrics: Arc<RwLock<BackpressureMetrics>>,
    rejection_count: Arc<RwLock<u64>>,
    total_requests: Arc<RwLock<u64>>,
    wait_times: Arc<RwLock<Vec<Duration>>>,
}

impl BackpressureController {
    /// Create a new backpressure controller
    pub fn new(config: BackpressureConfig) -> Self {
        let global_semaphore = Arc::new(Semaphore::new(config.max_total_items));
        let memory_semaphore = Arc::new(Semaphore::new(
            (config.max_memory_bytes / 1024) as usize, // Convert to KB for semaphore
        ));

        let controller = Self {
            config: config.clone(),
            stream_resources: Arc::new(RwLock::new(HashMap::new())),
            global_semaphore,
            memory_semaphore,
            metrics: Arc::new(RwLock::new(BackpressureMetrics {
                total_streams: 0,
                total_in_flight: 0,
                total_memory_usage: 0,
                status: BackpressureStatus::Normal,
                throttled_streams: 0,
                rejection_rate: 0.0,
                average_wait_time: Duration::from_millis(0),
            })),
            rejection_count: Arc::new(RwLock::new(0)),
            total_requests: Arc::new(RwLock::new(0)),
            wait_times: Arc::new(RwLock::new(Vec::new())),
        };

        // Start background monitoring
        controller.start_monitoring();

        controller
    }

    /// Register a new stream
    pub async fn register_stream(&self, stream_id: Uuid) -> StreamingResult<()> {
        let mut resources = self.stream_resources.write().await;
        resources.insert(
            stream_id,
            StreamResources {
                in_flight_items: 0,
                memory_usage: 0,
                last_activity: Instant::now(),
                throttle_until: None,
            },
        );

        self.update_metrics().await;
        Ok(())
    }

    /// Attempt to acquire resources for processing an item
    pub async fn acquire(
        &self,
        stream_id: Uuid,
        estimated_memory: u64,
    ) -> StreamingResult<BackpressurePermit> {
        let start_time = Instant::now();
        *self.total_requests.write().await += 1;

        // Check if stream is throttled
        {
            let resources = self.stream_resources.read().await;
            if let Some(stream_res) = resources.get(&stream_id) {
                if let Some(throttle_until) = stream_res.throttle_until {
                    if Instant::now() < throttle_until {
                        *self.rejection_count.write().await += 1;
                        return Err(StreamingError::BackpressureExceeded);
                    }
                }
            }
        }

        // Check stream-specific limits
        {
            let resources = self.stream_resources.read().await;
            if let Some(stream_res) = resources.get(&stream_id) {
                if stream_res.in_flight_items >= self.config.max_in_flight {
                    *self.rejection_count.write().await += 1;
                    return Err(StreamingError::BackpressureExceeded);
                }
            }
        }

        // Try to acquire global semaphore permit (owned to avoid lifetime issues)
        let global_permit = match Arc::clone(&self.global_semaphore).try_acquire_owned() {
            Ok(permit) => permit,
            Err(_) => {
                *self.rejection_count.write().await += 1;
                return Err(StreamingError::BackpressureExceeded);
            }
        };

        // Try to acquire memory permit (owned to avoid lifetime issues)
        let memory_kb = (estimated_memory / 1024).max(1) as usize;
        let memory_permit =
            match Arc::clone(&self.memory_semaphore).try_acquire_many_owned(memory_kb as u32) {
                Ok(permit) => Some(permit),
                Err(_) => {
                    // Release global permit and reject
                    drop(global_permit);
                    *self.rejection_count.write().await += 1;
                    return Err(StreamingError::BackpressureExceeded);
                }
            };

        // Update stream resources
        {
            let mut resources = self.stream_resources.write().await;
            if let Some(stream_res) = resources.get_mut(&stream_id) {
                stream_res.in_flight_items += 1;
                stream_res.memory_usage += estimated_memory;
                stream_res.last_activity = Instant::now();
            }
        }

        // Record wait time
        let wait_time = start_time.elapsed();
        {
            let mut wait_times = self.wait_times.write().await;
            wait_times.push(wait_time);
            if wait_times.len() > 1000 {
                wait_times.remove(0);
            }
        }

        self.update_metrics().await;

        Ok(BackpressurePermit {
            stream_id,
            estimated_memory,
            controller: self.clone(),
            _global_permit: global_permit,
            _memory_permit: memory_permit,
        })
    }

    /// Release resources for a stream
    pub async fn release(&self, stream_id: Uuid, actual_memory: u64) {
        let mut resources = self.stream_resources.write().await;
        if let Some(stream_res) = resources.get_mut(&stream_id) {
            stream_res.in_flight_items = stream_res.in_flight_items.saturating_sub(1);
            stream_res.memory_usage = stream_res.memory_usage.saturating_sub(actual_memory);
            stream_res.last_activity = Instant::now();
        }

        drop(resources);
        self.update_metrics().await;
    }

    /// Unregister a stream
    pub async fn unregister_stream(&self, stream_id: Uuid) {
        let mut resources = self.stream_resources.write().await;
        resources.remove(&stream_id);
        drop(resources);
        self.update_metrics().await;
    }

    /// Get current backpressure metrics
    pub async fn get_metrics(&self) -> BackpressureMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    /// Update internal metrics
    async fn update_metrics(&self) {
        let resources = self.stream_resources.read().await;

        let total_streams = resources.len();
        let total_in_flight = resources.values().map(|r| r.in_flight_items).sum();
        let total_memory_usage = resources.values().map(|r| r.memory_usage).sum();
        let throttled_streams = resources
            .values()
            .filter(|r| r.throttle_until.is_some_and(|t| Instant::now() < t))
            .count();

        // Calculate rejection rate
        let rejections = *self.rejection_count.read().await;
        let total_reqs = *self.total_requests.read().await;
        let rejection_rate = if total_reqs > 0 {
            rejections as f64 / total_reqs as f64
        } else {
            0.0
        };

        // Calculate average wait time
        let wait_times = self.wait_times.read().await;
        let average_wait_time = if !wait_times.is_empty() {
            let total: Duration = wait_times.iter().sum();
            total / wait_times.len() as u32
        } else {
            Duration::from_millis(0)
        };

        // Determine status
        let memory_usage_ratio = total_memory_usage as f64 / self.config.max_memory_bytes as f64;
        let items_usage_ratio = total_in_flight as f64 / self.config.max_total_items as f64;
        let max_usage = memory_usage_ratio.max(items_usage_ratio);

        let status = if max_usage >= self.config.activation_threshold {
            if max_usage >= 0.95 {
                BackpressureStatus::Critical
            } else {
                BackpressureStatus::Throttled
            }
        } else if max_usage >= 0.7 {
            BackpressureStatus::Warning
        } else {
            BackpressureStatus::Normal
        };

        let mut metrics = self.metrics.write().await;
        *metrics = BackpressureMetrics {
            total_streams,
            total_in_flight,
            total_memory_usage,
            status,
            throttled_streams,
            rejection_rate,
            average_wait_time,
        };
    }

    /// Start background monitoring task
    fn start_monitoring(&self) {
        let controller = self.clone();
        tokio::spawn(async move {
            let mut interval = interval(controller.config.check_interval);

            loop {
                interval.tick().await;
                controller.monitor_and_adjust().await;
            }
        });
    }

    /// Monitor resources and adjust throttling
    async fn monitor_and_adjust(&self) {
        if !self.config.adaptive {
            return;
        }

        let metrics = self.get_metrics().await;
        let now = Instant::now();

        // Clean up old streams
        {
            let mut resources = self.stream_resources.write().await;
            let inactive_streams: Vec<Uuid> = resources
                .iter()
                .filter(|(_, res)| now.duration_since(res.last_activity) > Duration::from_secs(300))
                .map(|(id, _)| *id)
                .collect();

            for stream_id in inactive_streams {
                resources.remove(&stream_id);
            }
        }

        // Adjust throttling based on current load
        match metrics.status {
            BackpressureStatus::Critical => {
                self.apply_throttling(Duration::from_secs(5)).await;
            }
            BackpressureStatus::Throttled => {
                self.apply_throttling(Duration::from_secs(2)).await;
            }
            BackpressureStatus::Warning => {
                self.apply_throttling(Duration::from_millis(500)).await;
            }
            BackpressureStatus::Normal => {
                self.clear_throttling().await;
            }
        }
    }

    /// Apply throttling to all streams
    async fn apply_throttling(&self, duration: Duration) {
        let mut resources = self.stream_resources.write().await;
        let throttle_until = Instant::now() + duration;

        for stream_res in resources.values_mut() {
            stream_res.throttle_until = Some(throttle_until);
        }
    }

    /// Clear throttling from all streams
    async fn clear_throttling(&self) {
        let mut resources = self.stream_resources.write().await;

        for stream_res in resources.values_mut() {
            stream_res.throttle_until = None;
        }
    }
}

impl Clone for BackpressureController {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            stream_resources: Arc::clone(&self.stream_resources),
            global_semaphore: Arc::clone(&self.global_semaphore),
            memory_semaphore: Arc::clone(&self.memory_semaphore),
            metrics: Arc::clone(&self.metrics),
            rejection_count: Arc::clone(&self.rejection_count),
            total_requests: Arc::clone(&self.total_requests),
            wait_times: Arc::clone(&self.wait_times),
        }
    }
}

/// Permit for processing an item with backpressure control
#[derive(Debug)]
pub struct BackpressurePermit {
    stream_id: Uuid,
    estimated_memory: u64,
    #[allow(dead_code)]
    controller: BackpressureController,
    _global_permit: tokio::sync::OwnedSemaphorePermit,
    _memory_permit: Option<tokio::sync::OwnedSemaphorePermit>,
}

impl BackpressurePermit {
    /// Get the stream ID this permit is for
    pub fn stream_id(&self) -> Uuid {
        self.stream_id
    }

    /// Get the estimated memory usage
    pub fn estimated_memory(&self) -> u64 {
        self.estimated_memory
    }
}

impl Drop for BackpressurePermit {
    fn drop(&mut self) {
        let controller = self.controller.clone();
        let stream_id = self.stream_id;
        let memory = self.estimated_memory;

        tokio::spawn(async move {
            controller.release(stream_id, memory).await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_backpressure_controller_creation() {
        let config = BackpressureConfig::default();
        let controller = BackpressureController::new(config);

        let metrics = controller.get_metrics().await;
        assert_eq!(metrics.total_streams, 0);
        assert!(matches!(metrics.status, BackpressureStatus::Normal));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_stream_registration() {
        let test_future = async {
            let config = BackpressureConfig::default();
            let controller = BackpressureController::new(config);
            let stream_id = Uuid::new_v4();

            controller.register_stream(stream_id).await.unwrap();

            let metrics = controller.get_metrics().await;
            assert_eq!(metrics.total_streams, 1);
        };

        tokio::time::timeout(Duration::from_secs(5), test_future)
            .await
            .expect("Test should complete within 5 seconds");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_resource_acquisition() {
        let test_future = async {
            let config = BackpressureConfig::default();
            let controller = BackpressureController::new(config);
            let stream_id = Uuid::new_v4();

            controller.register_stream(stream_id).await.unwrap();

            let permit = controller.acquire(stream_id, 1024).await.unwrap();
            assert_eq!(permit.stream_id(), stream_id);
            assert_eq!(permit.estimated_memory(), 1024);

            let metrics = controller.get_metrics().await;
            assert_eq!(metrics.total_in_flight, 1);

            drop(permit);

            // Wait for cleanup
            sleep(Duration::from_millis(10)).await;

            let metrics = controller.get_metrics().await;
            assert_eq!(metrics.total_in_flight, 0);
        };

        tokio::time::timeout(Duration::from_secs(10), test_future)
            .await
            .expect("Test should complete within 10 seconds");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_backpressure_limits() {
        let test_future = async {
            let config = BackpressureConfig {
                max_in_flight: 2,
                max_total_items: 2,
                ..Default::default()
            };
            let controller = BackpressureController::new(config);
            let stream_id = Uuid::new_v4();

            controller.register_stream(stream_id).await.unwrap();

            // Acquire maximum permits
            let _permit1 = controller.acquire(stream_id, 1024).await.unwrap();
            let _permit2 = controller.acquire(stream_id, 1024).await.unwrap();

            // Third acquisition should fail
            let result = controller.acquire(stream_id, 1024).await;
            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err(),
                StreamingError::BackpressureExceeded
            ));
        };

        tokio::time::timeout(Duration::from_secs(10), test_future)
            .await
            .expect("Test should complete within 10 seconds");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_memory_limits() {
        let test_future = async {
            let config = BackpressureConfig {
                max_memory_bytes: 2048, // 2KB
                ..Default::default()
            };
            let controller = BackpressureController::new(config);
            let stream_id = Uuid::new_v4();

            controller.register_stream(stream_id).await.unwrap();

            // Acquire permit using all available memory
            let _permit1 = controller.acquire(stream_id, 2048).await.unwrap();

            // Second acquisition should fail due to memory limit
            let result = controller.acquire(stream_id, 1024).await;
            assert!(result.is_err());
        };

        tokio::time::timeout(Duration::from_secs(10), test_future)
            .await
            .expect("Test should complete within 10 seconds");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_metrics_calculation() {
        let test_future = async {
            let config = BackpressureConfig::default();
            let controller = BackpressureController::new(config);
            let stream_id = Uuid::new_v4();

            controller.register_stream(stream_id).await.unwrap();

            let _permit = controller.acquire(stream_id, 1024).await.unwrap();

            let metrics = controller.get_metrics().await;
            assert_eq!(metrics.total_streams, 1);
            assert_eq!(metrics.total_in_flight, 1);
            assert_eq!(metrics.total_memory_usage, 1024);
        };

        tokio::time::timeout(Duration::from_secs(10), test_future)
            .await
            .expect("Test should complete within 10 seconds");
    }
}
