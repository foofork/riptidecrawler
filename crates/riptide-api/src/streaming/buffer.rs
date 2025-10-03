//! Buffer management and backpressure handling for streaming operations.
//!
//! This module provides dynamic buffer sizing, backpressure detection, and
//! adaptive throttling to handle varying client connection speeds.

use super::error::StreamingResult;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, warn};

/// Dynamic buffer configuration
#[derive(Debug, Clone)]
pub struct BufferConfig {
    /// Initial buffer size
    pub initial_size: usize,
    /// Maximum buffer size before dropping messages
    pub max_size: usize,
    /// Minimum buffer size
    pub min_size: usize,
    /// Growth factor when expanding buffer
    pub growth_factor: f64,
    /// Shrink factor when reducing buffer
    pub shrink_factor: f64,
    /// Time threshold for slow sends (milliseconds)
    pub slow_send_threshold_ms: u64,
    /// Maximum number of slow sends before marking connection as slow
    pub max_slow_sends: usize,
    /// Backpressure detection window (number of messages)
    pub backpressure_window: usize,
}

impl Default for BufferConfig {
    fn default() -> Self {
        Self {
            initial_size: 256, // Increased from 128
            max_size: 2048,    // Dynamic maximum
            min_size: 64,
            growth_factor: 1.5,
            shrink_factor: 0.75,
            slow_send_threshold_ms: 100,
            max_slow_sends: 10,
            backpressure_window: 50,
        }
    }
}

/// Buffer statistics for monitoring
#[derive(Debug, Default, Clone)]
pub struct BufferStats {
    pub current_size: usize,
    pub peak_size: usize,
    pub total_messages: usize,
    pub dropped_messages: usize,
    pub slow_sends: usize,
    pub average_send_time_ms: f64,
    pub resizes: usize,
}

/// Dynamic buffer with adaptive sizing
pub struct DynamicBuffer {
    config: BufferConfig,
    current_capacity: AtomicUsize,
    stats: Arc<RwLock<BufferStats>>,
    send_times: Arc<RwLock<VecDeque<Duration>>>,
}

impl Default for DynamicBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl DynamicBuffer {
    /// Create a new dynamic buffer with default configuration
    pub fn new() -> Self {
        Self::with_config(BufferConfig::default())
    }

    /// Create a new dynamic buffer with custom configuration
    pub fn with_config(config: BufferConfig) -> Self {
        Self {
            current_capacity: AtomicUsize::new(config.initial_size),
            config,
            stats: Arc::new(RwLock::new(BufferStats::default())),
            send_times: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    /// Get current buffer capacity
    pub fn capacity(&self) -> usize {
        self.current_capacity.load(Ordering::Relaxed)
    }

    /// Create a new channel with current buffer capacity
    pub async fn create_channel<T>(&self) -> (mpsc::Sender<T>, mpsc::Receiver<T>) {
        let capacity = self.capacity();
        mpsc::channel(capacity)
    }

    /// Record a send operation and adjust buffer size if needed
    pub async fn record_send(&self, duration: Duration) -> StreamingResult<()> {
        let mut stats = self.stats.write().await;
        stats.total_messages += 1;

        let duration_ms = duration.as_millis() as f64;

        // Update running average
        let total_time = stats.average_send_time_ms * (stats.total_messages - 1) as f64;
        stats.average_send_time_ms = (total_time + duration_ms) / stats.total_messages as f64;

        // Track slow sends
        if duration_ms > self.config.slow_send_threshold_ms as f64 {
            stats.slow_sends += 1;
        }

        // Add to send times window
        let mut send_times = self.send_times.write().await;
        send_times.push_back(duration);

        // Maintain window size
        while send_times.len() > self.config.backpressure_window {
            send_times.pop_front();
        }

        drop(stats);
        drop(send_times);

        // Adjust buffer size based on recent performance
        self.adjust_buffer_size().await?;

        Ok(())
    }

    /// Record a dropped message
    pub async fn record_drop(&self) {
        let mut stats = self.stats.write().await;
        stats.dropped_messages += 1;
    }

    /// Check if connection is experiencing backpressure
    pub async fn is_under_backpressure(&self) -> bool {
        let send_times = self.send_times.read().await;

        if send_times.len() < self.config.backpressure_window / 2 {
            return false;
        }

        let slow_count = send_times
            .iter()
            .filter(|d| d.as_millis() > self.config.slow_send_threshold_ms as u128)
            .count();

        let slow_ratio = slow_count as f64 / send_times.len() as f64;
        slow_ratio > 0.5
    }

    /// Adjust buffer size based on recent performance
    async fn adjust_buffer_size(&self) -> StreamingResult<()> {
        let is_backpressure = self.is_under_backpressure().await;
        let current_capacity = self.capacity();

        if is_backpressure && current_capacity > self.config.min_size {
            // Shrink buffer to reduce memory usage for slow clients
            let new_capacity = ((current_capacity as f64 * self.config.shrink_factor) as usize)
                .max(self.config.min_size);

            if new_capacity != current_capacity {
                self.current_capacity.store(new_capacity, Ordering::Relaxed);

                let mut stats = self.stats.write().await;
                stats.resizes += 1;
                stats.current_size = new_capacity;

                debug!(
                    old_capacity = current_capacity,
                    new_capacity = new_capacity,
                    "Shrunk buffer due to backpressure"
                );
            }
        } else if !is_backpressure && current_capacity < self.config.max_size {
            // Check if we should grow the buffer
            let stats = self.stats.read().await;
            let drop_ratio = if stats.total_messages > 0 {
                stats.dropped_messages as f64 / stats.total_messages as f64
            } else {
                0.0
            };

            // Grow buffer if we're dropping messages
            if drop_ratio > 0.01 {
                // More than 1% drop rate
                let new_capacity = ((current_capacity as f64 * self.config.growth_factor) as usize)
                    .min(self.config.max_size);

                if new_capacity != current_capacity {
                    drop(stats);
                    self.current_capacity.store(new_capacity, Ordering::Relaxed);

                    let mut stats = self.stats.write().await;
                    stats.resizes += 1;
                    stats.current_size = new_capacity;
                    stats.peak_size = stats.peak_size.max(new_capacity);

                    debug!(
                        old_capacity = current_capacity,
                        new_capacity = new_capacity,
                        drop_ratio = drop_ratio,
                        "Grew buffer due to drops"
                    );
                }
            }
        }

        Ok(())
    }

    /// Get current buffer statistics
    pub async fn stats(&self) -> BufferStats {
        let mut stats = self.stats.read().await.clone();
        stats.current_size = self.capacity();
        stats.peak_size = stats.peak_size.max(stats.current_size);
        stats
    }

    /// Reset statistics
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = BufferStats::default();

        let mut send_times = self.send_times.write().await;
        send_times.clear();
    }
}

/// Backpressure handler for individual connections
pub struct BackpressureHandler {
    connection_id: String,
    buffer: Arc<DynamicBuffer>,
    drop_threshold: usize,
    metrics: BackpressureMetrics,
}

/// Backpressure metrics for monitoring
#[derive(Debug, Default)]
pub struct BackpressureMetrics {
    pub total_messages: usize,
    pub dropped_messages: usize,
    pub slow_sends: usize,
    pub average_send_time_ms: f64,
    pub last_drop_time: Option<Instant>,
}

impl BackpressureHandler {
    /// Create a new backpressure handler
    pub fn new(connection_id: String, buffer: Arc<DynamicBuffer>) -> Self {
        Self {
            connection_id,
            buffer,
            drop_threshold: 1000,
            metrics: BackpressureMetrics::default(),
        }
    }

    /// Check if a message should be dropped due to backpressure with adaptive thresholds
    pub async fn should_drop_message(&mut self, queue_size: usize) -> bool {
        let is_backpressure = self.buffer.is_under_backpressure().await;
        let buffer_stats = self.buffer.stats().await;

        // Adaptive drop threshold based on connection performance
        let adaptive_threshold = if self.is_connection_slow() {
            self.drop_threshold / 2 // More aggressive dropping for slow connections
        } else {
            self.drop_threshold
        };

        // Consider memory pressure and error rate
        let should_drop = queue_size > adaptive_threshold
            || is_backpressure
            || (buffer_stats.dropped_messages as f64 / buffer_stats.total_messages.max(1) as f64)
                > 0.15; // 15% drop rate

        if should_drop {
            self.metrics.dropped_messages += 1;
            self.metrics.last_drop_time = Some(Instant::now());
            self.buffer.record_drop().await;

            // Adjust drop threshold dynamically
            if self.metrics.total_messages.is_multiple_of(100) {
                self.adjust_drop_threshold().await;
            }

            warn!(
                connection_id = %self.connection_id,
                queue_size = queue_size,
                adaptive_threshold = adaptive_threshold,
                is_backpressure = is_backpressure,
                is_slow = self.is_connection_slow(),
                "Dropping message due to backpressure"
            );

            true
        } else {
            false
        }
    }

    /// Adjust drop threshold based on connection performance
    async fn adjust_drop_threshold(&mut self) {
        let performance_ratio = if self.metrics.total_messages > 0 {
            (self.metrics.total_messages - self.metrics.dropped_messages) as f64
                / self.metrics.total_messages as f64
        } else {
            1.0
        };

        // If performance is good (low drop rate), increase threshold
        // If performance is bad (high drop rate), decrease threshold
        if performance_ratio > 0.95 {
            self.drop_threshold = (self.drop_threshold as f64 * 1.1) as usize;
            debug!(
                connection_id = %self.connection_id,
                new_threshold = self.drop_threshold,
                "Increased drop threshold due to good performance"
            );
        } else if performance_ratio < 0.8 {
            self.drop_threshold = (self.drop_threshold as f64 * 0.9) as usize;
            debug!(
                connection_id = %self.connection_id,
                new_threshold = self.drop_threshold,
                "Decreased drop threshold due to poor performance"
            );
        }

        // Keep threshold within reasonable bounds
        self.drop_threshold = self.drop_threshold.max(100).min(5000);
    }

    /// Record a successful send operation
    pub async fn record_send_time(&mut self, duration: Duration) -> StreamingResult<()> {
        self.metrics.total_messages += 1;
        let duration_ms = duration.as_millis() as f64;

        // Update running average
        let total_time =
            self.metrics.average_send_time_ms * (self.metrics.total_messages - 1) as f64;
        self.metrics.average_send_time_ms =
            (total_time + duration_ms) / self.metrics.total_messages as f64;

        if duration_ms > 100.0 {
            // 100ms threshold
            self.metrics.slow_sends += 1;
        }

        // Record in the shared buffer
        self.buffer.record_send(duration).await?;

        Ok(())
    }

    /// Check if connection is consistently slow
    pub fn is_connection_slow(&self) -> bool {
        if self.metrics.total_messages < 10 {
            return false;
        }

        let slow_ratio = self.metrics.slow_sends as f64 / self.metrics.total_messages as f64;
        slow_ratio > 0.5 || self.metrics.average_send_time_ms > 100.0
    }

    /// Get current metrics
    pub fn metrics(&self) -> &BackpressureMetrics {
        &self.metrics
    }

    /// Get connection ID
    pub fn connection_id(&self) -> &str {
        &self.connection_id
    }
}

/// Global buffer manager for all streaming connections
pub struct BufferManager {
    buffers: Arc<RwLock<std::collections::HashMap<String, Arc<DynamicBuffer>>>>,
    default_config: BufferConfig,
}

impl BufferManager {
    /// Create a new buffer manager
    pub fn new() -> Self {
        Self {
            buffers: Arc::new(RwLock::new(std::collections::HashMap::new())),
            default_config: BufferConfig::default(),
        }
    }

    /// Get or create a buffer for a connection
    pub async fn get_buffer(&self, connection_id: &str) -> Arc<DynamicBuffer> {
        let mut buffers = self.buffers.write().await;

        buffers
            .entry(connection_id.to_string())
            .or_insert_with(|| Arc::new(DynamicBuffer::with_config(self.default_config.clone())))
            .clone()
    }

    /// Remove a buffer when connection closes
    pub async fn remove_buffer(&self, connection_id: &str) {
        let mut buffers = self.buffers.write().await;
        buffers.remove(connection_id);
    }

    /// Get statistics for all buffers
    pub async fn global_stats(&self) -> std::collections::HashMap<String, BufferStats> {
        let buffers = self.buffers.read().await;
        let mut stats = std::collections::HashMap::new();

        for (id, buffer) in buffers.iter() {
            stats.insert(id.clone(), buffer.stats().await);
        }

        stats
    }

    /// Clean up old unused buffers
    pub async fn cleanup_old_buffers(&self, max_age: Duration) {
        // Implementation would track creation time and remove old buffers
        // For now, this is a placeholder
        debug!("Cleaning up buffers older than {:?}", max_age);
    }
}

impl Default for BufferManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_dynamic_buffer_creation() {
        let buffer = DynamicBuffer::new();
        assert_eq!(buffer.capacity(), 256); // Default initial size
    }

    #[tokio::test]
    async fn test_buffer_stats() {
        let buffer = DynamicBuffer::new();
        let stats = buffer.stats().await;
        assert_eq!(stats.total_messages, 0);
        assert_eq!(stats.dropped_messages, 0);
    }

    #[tokio::test]
    async fn test_backpressure_detection() {
        let buffer = DynamicBuffer::new();

        // Simulate slow sends
        for _ in 0..30 {
            buffer
                .record_send(Duration::from_millis(150))
                .await
                .unwrap();
        }

        assert!(buffer.is_under_backpressure().await);
    }

    #[tokio::test]
    async fn test_backpressure_handler() {
        let buffer = Arc::new(DynamicBuffer::new());
        let mut handler = BackpressureHandler::new("test-conn".to_string(), buffer);

        // Should not drop with low queue size
        assert!(!handler.should_drop_message(10).await);

        // Should drop with high queue size
        assert!(handler.should_drop_message(1500).await);
    }

    #[tokio::test]
    async fn test_buffer_manager() {
        let manager = BufferManager::new();

        let buffer1 = manager.get_buffer("conn1").await;
        let buffer2 = manager.get_buffer("conn1").await;

        // Should return the same buffer for same connection
        assert!(Arc::ptr_eq(&buffer1, &buffer2));

        manager.remove_buffer("conn1").await;
        let buffer3 = manager.get_buffer("conn1").await;

        // Should create a new buffer after removal
        assert!(!Arc::ptr_eq(&buffer1, &buffer3));
    }

    #[tokio::test]
    async fn test_buffer_growth() {
        let mut config = BufferConfig::default();
        config.initial_size = 64;
        config.max_size = 256;
        config.growth_factor = 2.0;

        let buffer = DynamicBuffer::with_config(config);

        // Simulate high drop rate
        for _ in 0..100 {
            buffer.record_drop().await;
        }

        // Trigger adjustment
        buffer.record_send(Duration::from_millis(50)).await.unwrap();

        let stats = buffer.stats().await;
        assert!(stats.current_size > 64);
    }
}
