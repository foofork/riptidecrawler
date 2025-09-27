//! Behavior Capture Module
//!
//! Captures system behavior including performance, memory, and functional outputs
//! for use as golden test baselines.

use super::*;
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use sysinfo::{System, SystemExt, ProcessExt, Process};
use tokio::time::timeout;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Captures complete system behavior for a test function
pub async fn capture_behavior<F, T>(
    test_name: &str,
    test_fn: F,
    config: &GoldenTestConfig,
) -> Result<BehaviorSnapshot, anyhow::Error>
where
    F: Fn() -> T + Send + 'static,
    T: std::future::Future<Output = Result<serde_json::Value, anyhow::Error>> + Send,
{
    let mut system = System::new_all();
    system.refresh_all();
    
    let memory_tracker = MemoryTracker::new();
    let performance_tracker = PerformanceTracker::new();
    
    // Warmup phase
    if config.verbose {
        println!("[{}] Starting warmup phase ({} iterations)", test_name, config.warmup_iterations);
    }
    
    for i in 0..config.warmup_iterations {
        if config.verbose {
            println!("[{}] Warmup iteration {}", test_name, i + 1);
        }
        
        let _ = timeout(
            Duration::from_secs(config.timeout_seconds),
            test_fn()
        ).await??;
        
        // Small delay between iterations
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    // Measurement phase
    if config.verbose {
        println!("[{}] Starting measurement phase ({} iterations)", test_name, config.measurement_iterations);
    }
    
    let mut latencies = Vec::new();
    let mut functional_outputs = HashMap::new();
    let mut error_patterns = Vec::new();
    
    memory_tracker.start_tracking();
    
    for i in 0..config.measurement_iterations {
        if config.verbose {
            println!("[{}] Measurement iteration {}", test_name, i + 1);
        }
        
        let start_time = Instant::now();
        let start_memory = memory_tracker.current_usage();
        
        let result = timeout(
            Duration::from_secs(config.timeout_seconds),
            test_fn()
        ).await;
        
        let duration = start_time.elapsed();
        latencies.push(duration);
        
        match result {
            Ok(Ok(output)) => {
                functional_outputs.insert(format!("iteration_{}", i), output);
            },
            Ok(Err(e)) => {
                error_patterns.push(format!("Error in iteration {}: {}", i, e));
            },
            Err(_) => {
                error_patterns.push(format!("Timeout in iteration {}", i));
            }
        }
        
        memory_tracker.record_peak(memory_tracker.current_usage());
        
        // Small delay between iterations
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    
    memory_tracker.stop_tracking();
    
    // Calculate performance metrics
    let performance_metrics = calculate_performance_metrics(&latencies);
    let memory_metrics = memory_tracker.get_metrics();
    let throughput_metrics = calculate_throughput_metrics(&latencies, &functional_outputs);
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    
    Ok(BehaviorSnapshot {
        timestamp,
        test_name: test_name.to_string(),
        performance_metrics,
        memory_metrics,
        throughput_metrics,
        functional_outputs,
        error_patterns,
    })
}

/// Tracks memory usage during test execution
struct MemoryTracker {
    system: System,
    initial_rss: AtomicU64,
    peak_rss: AtomicU64,
    current_rss: AtomicU64,
    initial_heap: AtomicU64,
    tracking: Arc<std::sync::atomic::AtomicBool>,
}

impl MemoryTracker {
    fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        let current_process = system.process(sysinfo::get_current_pid().unwrap()).unwrap();
        let initial_rss = current_process.memory();
        
        Self {
            system,
            initial_rss: AtomicU64::new(initial_rss * 1024), // Convert to bytes
            peak_rss: AtomicU64::new(initial_rss * 1024),
            current_rss: AtomicU64::new(initial_rss * 1024),
            initial_heap: AtomicU64::new(0), // TODO: Get heap info if available
            tracking: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
    
    fn start_tracking(&self) {
        self.tracking.store(true, Ordering::SeqCst);
        
        // Start background monitoring thread
        let tracking = self.tracking.clone();
        let peak_rss = Arc::new(AtomicU64::new(self.peak_rss.load(Ordering::SeqCst)));
        let current_rss = Arc::new(AtomicU64::new(self.current_rss.load(Ordering::SeqCst)));
        
        tokio::spawn(async move {
            let mut system = System::new_all();
            
            while tracking.load(Ordering::SeqCst) {
                system.refresh_all();
                
                if let Some(process) = system.process(sysinfo::get_current_pid().unwrap()) {
                    let memory_bytes = process.memory() * 1024;
                    current_rss.store(memory_bytes, Ordering::SeqCst);
                    
                    let current_peak = peak_rss.load(Ordering::SeqCst);
                    if memory_bytes > current_peak {
                        peak_rss.store(memory_bytes, Ordering::SeqCst);
                    }
                }
                
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });
    }
    
    fn stop_tracking(&self) {
        self.tracking.store(false, Ordering::SeqCst);
    }
    
    fn current_usage(&mut self) -> u64 {
        self.system.refresh_all();
        if let Some(process) = self.system.process(sysinfo::get_current_pid().unwrap()) {
            let memory_bytes = process.memory() * 1024;
            self.current_rss.store(memory_bytes, Ordering::SeqCst);
            memory_bytes
        } else {
            0
        }
    }
    
    fn record_peak(&self, memory_bytes: u64) {
        let current_peak = self.peak_rss.load(Ordering::SeqCst);
        if memory_bytes > current_peak {
            self.peak_rss.store(memory_bytes, Ordering::SeqCst);
        }
    }
    
    fn get_metrics(&self) -> MemoryMetrics {
        let initial = self.initial_rss.load(Ordering::SeqCst);
        let current = self.current_rss.load(Ordering::SeqCst);
        let peak = self.peak_rss.load(Ordering::SeqCst);
        let heap = self.initial_heap.load(Ordering::SeqCst);
        
        let efficiency = if peak > 0 {
            ((peak - current) as f64 / peak as f64) * 100.0
        } else {
            100.0
        };
        
        MemoryMetrics {
            rss_bytes: current,
            heap_bytes: heap,
            peak_rss_bytes: peak,
            memory_efficiency: efficiency,
        }
    }
}

/// Tracks performance metrics during test execution
struct PerformanceTracker {
    start_time: Option<Instant>,
    measurements: Vec<Duration>,
}

impl PerformanceTracker {
    fn new() -> Self {
        Self {
            start_time: None,
            measurements: Vec::new(),
        }
    }
}

/// Calculate performance metrics from latency measurements
fn calculate_performance_metrics(latencies: &[Duration]) -> PerformanceMetrics {
    if latencies.is_empty() {
        return PerformanceMetrics {
            p50_latency_ms: 0.0,
            p95_latency_ms: 0.0,
            p99_latency_ms: 0.0,
            mean_latency_ms: 0.0,
            min_latency_ms: 0.0,
            max_latency_ms: 0.0,
            std_dev_ms: 0.0,
        };
    }
    
    let mut sorted_latencies: Vec<f64> = latencies
        .iter()
        .map(|d| d.as_secs_f64() * 1000.0) // Convert to milliseconds
        .collect();
    sorted_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    let len = sorted_latencies.len();
    let mean = sorted_latencies.iter().sum::<f64>() / len as f64;
    
    let variance = sorted_latencies
        .iter()
        .map(|x| (x - mean).powi(2))
        .sum::<f64>() / len as f64;
    let std_dev = variance.sqrt();
    
    PerformanceMetrics {
        p50_latency_ms: percentile(&sorted_latencies, 50.0),
        p95_latency_ms: percentile(&sorted_latencies, 95.0),
        p99_latency_ms: percentile(&sorted_latencies, 99.0),
        mean_latency_ms: mean,
        min_latency_ms: sorted_latencies[0],
        max_latency_ms: sorted_latencies[len - 1],
        std_dev_ms: std_dev,
    }
}

/// Calculate throughput metrics
fn calculate_throughput_metrics(
    latencies: &[Duration],
    _functional_outputs: &HashMap<String, serde_json::Value>,
) -> ThroughputMetrics {
    if latencies.is_empty() {
        return ThroughputMetrics {
            pages_per_second: 0.0,
            requests_per_second: 0.0,
            bytes_processed_per_second: 0,
            operations_per_second: 0.0,
        };
    }
    
    let total_time = latencies.iter().sum::<Duration>().as_secs_f64();
    let operations_count = latencies.len() as f64;
    
    ThroughputMetrics {
        pages_per_second: operations_count / total_time,
        requests_per_second: operations_count / total_time,
        bytes_processed_per_second: 0, // TODO: Calculate from functional outputs
        operations_per_second: operations_count / total_time,
    }
}

/// Calculate percentile from sorted data
fn percentile(sorted_data: &[f64], percentile: f64) -> f64 {
    if sorted_data.is_empty() {
        return 0.0;
    }
    
    let index = (percentile / 100.0) * (sorted_data.len() - 1) as f64;
    let lower_index = index.floor() as usize;
    let upper_index = index.ceil() as usize;
    
    if lower_index == upper_index {
        sorted_data[lower_index]
    } else {
        let weight = index - lower_index as f64;
        sorted_data[lower_index] * (1.0 - weight) + sorted_data[upper_index] * weight
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_percentile_calculation() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(percentile(&data, 50.0), 3.0);
        assert_eq!(percentile(&data, 0.0), 1.0);
        assert_eq!(percentile(&data, 100.0), 5.0);
    }
    
    #[test]
    fn test_performance_metrics() {
        let latencies = vec![
            Duration::from_millis(100),
            Duration::from_millis(200),
            Duration::from_millis(150),
        ];
        
        let metrics = calculate_performance_metrics(&latencies);
        assert!(metrics.mean_latency_ms > 0.0);
        assert!(metrics.p50_latency_ms > 0.0);
    }
}
